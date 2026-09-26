//! Entitlement Seam
//!
//! Centralized policy engine evaluating operational capabilities, plan tiers,
//! grace periods, and device quotas against active licensing state.

use crate::db;
use crate::dispatcher::{CommandMeta, LicenseGate, RpcError};
use crate::licensing::{get_license_status, is_device_deactivated, LicenseStatusResponse};
use rusqlite::{Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use specta::Type;
use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum EntitlementError {
    #[error("This PC was deactivated from the customer portal. The app is in read-only mode until a new license is activated.")]
    DeviceDeactivated,

    #[error("Institution license has expired past the 14-day grace period. Database is in read-only mode.")]
    ExpiredReadOnly,

    #[error("{0}")]
    ClockOutOfSync(String),

    #[error("{message}")]
    FeatureNotEntitled { feature: String, message: String },

    #[error("Active workstation count has reached your license seat limit")]
    WorkstationLimitReached,

    #[error("Database error: {0}")]
    Db(String),
}

impl EntitlementError {
    pub fn code(&self) -> &'static str {
        match self {
            EntitlementError::DeviceDeactivated => "LICENSE_DEVICE_DEACTIVATED",
            EntitlementError::ExpiredReadOnly => "license_expired_read_only",
            EntitlementError::ClockOutOfSync(_) => "clock_out_of_sync",
            EntitlementError::FeatureNotEntitled { .. } => "LICENSE_FEATURE_NOT_ENTITLED",
            EntitlementError::WorkstationLimitReached => "LAN_WORKSTATION_LIMIT_REACHED",
            EntitlementError::Db(_) => "database_error",
        }
    }
}

impl From<rusqlite::Error> for EntitlementError {
    fn from(err: rusqlite::Error) -> Self {
        EntitlementError::Db(err.to_string())
    }
}

impl From<crate::licensing::LicenseError> for EntitlementError {
    fn from(err: crate::licensing::LicenseError) -> Self {
        EntitlementError::Db(err.to_string())
    }
}

impl From<EntitlementError> for RpcError {
    fn from(err: EntitlementError) -> Self {
        RpcError {
            code: err.code().to_string(),
            message: err.to_string(),
            field: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
pub struct WorkstationQuota {
    pub active_workstations: u32,
    pub workstation_limit: Option<u32>,
    pub seats_remaining: Option<u32>,
    pub license_tier: String,
}

/// Evaluates if the database is in read-only mode due to expired license past grace.
pub fn is_read_only(conn: &Connection, current_utc: &str) -> bool {
    let val_opt: Option<String> = conn
        .query_row(
            "SELECT value_json FROM settings WHERE key = 'license'",
            [],
            |row| row.get(0),
        )
        .optional()
        .unwrap_or(None);

    if let Some(json_str) = val_opt {
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&json_str) {
            if val
                .get("expired_past_grace")
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
                || val.get("status").and_then(|v| v.as_str()) == Some("expired_past_grace")
            {
                return true;
            }
        }
    }

    if let Ok(status) = get_license_status(conn, current_utc) {
        if status.is_read_only {
            return true;
        }
    }
    false
}

/// Authorizes a command execution against license gates, device state, grace periods, and clock synchronization.
pub fn authorize_command(
    conn: &Connection,
    meta: &CommandMeta,
    current_utc: &str,
) -> Result<(), EntitlementError> {
    if matches!(
        meta.gate,
        LicenseGate::Write | LicenseGate::Sms | LicenseGate::Lan
    ) {
        if is_device_deactivated(conn) {
            return Err(EntitlementError::DeviceDeactivated);
        }

        if is_read_only(conn, current_utc) {
            return Err(EntitlementError::ExpiredReadOnly);
        }

        if let Err(e) = db::check_clock_watermark(conn, current_utc) {
            return Err(EntitlementError::ClockOutOfSync(e.to_string()));
        }

        if meta.gate == LicenseGate::Lan {
            let status = get_license_status(conn, current_utc)?;
            let entitled = status.tier == "trial"
                || status.tier == "local_pro"
                || status
                    .capabilities
                    .iter()
                    .any(|c| c == "lan_multi_device" || c == "lan");
            if !entitled {
                return Err(EntitlementError::FeatureNotEntitled {
                    feature: "lan".to_string(),
                    message: "LAN multi-workstation hosting requires Coachify Local Pro. Upgrade your license to enable this feature.".to_string(),
                });
            }
        }
    }

    Ok(())
}

/// Computes the current workstation quota against the active license limits and devices table.
pub fn get_workstation_quota(
    conn: &Connection,
    current_utc: &str,
) -> Result<WorkstationQuota, EntitlementError> {
    let active_count: u32 = conn
        .query_row(
            "SELECT COUNT(*) FROM devices WHERE is_host = 0 AND status = 'active'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let license = get_license_status(conn, current_utc).ok();
    let limit = license.as_ref().and_then(|l| l.workstation_limit);
    let tier = license
        .as_ref()
        .map(|l| l.tier.clone())
        .unwrap_or_else(|| "starter".to_string());

    let remaining = limit.map(|lim| lim.saturating_sub(active_count));

    Ok(WorkstationQuota {
        active_workstations: active_count,
        workstation_limit: limit,
        seats_remaining: remaining,
        license_tier: tier,
    })
}

/// Guards that an available workstation seat exists before pairing or activating a workstation.
/// Fails with `EntitlementError::WorkstationLimitReached` if active workstations reach or exceed limit.
pub fn guard_workstation_seat(
    conn: &Connection,
    current_utc: &str,
) -> Result<(), EntitlementError> {
    let quota = get_workstation_quota(conn, current_utc)?;
    let limit = quota.workstation_limit.unwrap_or(1);
    if quota.active_workstations >= limit {
        return Err(EntitlementError::WorkstationLimitReached);
    }
    Ok(())
}

/// Convenience method to retrieve license status through the Entitlement seam.
pub fn get_status(
    conn: &Connection,
    current_utc: &str,
) -> Result<LicenseStatusResponse, EntitlementError> {
    get_license_status(conn, current_utc).map_err(EntitlementError::from)
}
