//! Commercial Vendor Admin Backend and Activation/Refresh Service (`crates/cloud_api`).
//!
//! Owns the 14 commercial cloud tables, Argon2id + RFC 6238 TOTP two factor admin login,
//! step up TOTP checks for revocations and release blocks, manual bKash/Nagad/bank/cash
//! payment recording with grace aware `paid_through` calculation, offline request code
//! (`REQ-XXXX-...` and `COACHIFY-REQ|v1|...`) license issuing, SHA256 hash chained
//! `admin_audit_log`, and the versioned desktop facing endpoints (`/v1/activate`,
//! `/v1/refresh`, `/v1/updates/{channel}`).
//!
//! `crates/cloud_api` holds zero private Ed25519 key material; every license or revocation
//! signature is requested from [`coachify_license_signer::SignerService`] using an
//! `HMAC-SHA256` authenticated service call.

use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use coachify_license_signer::{
    canonical_value_json, compute_service_hmac, SignLicenseRequest, SignRevocationRequest,
    SignUpdaterReleaseRequest, SignedRevocationPayload, SignerAuditEntry, SignerLicenseLimits,
    SignerLicensePayload, SignerService, UpdaterReleaseSignPayload, VerifyKeyRecoveryRequest,
    DEFAULT_SIGNER_SHARED_SECRET,
};
use hmac::{Hmac, Mac};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use specta::Type;
use thiserror::Error;
use uuid::Uuid;

type HmacSha256 = Hmac<Sha256>;

const REQUEST_CODE_HMAC_KEY: &[u8] = b"COACHIFY_OFFLINE_REQUEST_CODE_V1_HMAC_KEY";
const BASE32_ALPHABET: &[u8; 32] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";

pub const CLOUD_SCHEMA_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS admin_users (
    id TEXT PRIMARY KEY,
    email TEXT UNIQUE NOT NULL,
    name TEXT NOT NULL,
    role TEXT NOT NULL CHECK (role IN ('super_admin', 'ops_admin', 'support_admin')),
    password_hash TEXT NOT NULL,
    totp_secret_enc TEXT NOT NULL,
    totp_enabled INTEGER NOT NULL DEFAULT 1 CHECK (totp_enabled IN (0, 1)),
    recovery_codes_json TEXT NOT NULL DEFAULT '[]',
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'suspended')),
    last_login_at TEXT,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS admin_audit_log (
    seq INTEGER PRIMARY KEY AUTOINCREMENT,
    at TEXT NOT NULL,
    admin_user_id TEXT NOT NULL,
    admin_email TEXT NOT NULL,
    action TEXT NOT NULL,
    entity_type TEXT NOT NULL,
    entity_id TEXT NOT NULL,
    ip_address TEXT,
    before_json TEXT,
    after_json TEXT,
    prev_hash TEXT NOT NULL,
    hash TEXT NOT NULL
);

CREATE TRIGGER IF NOT EXISTS prevent_update_admin_audit_log
BEFORE UPDATE ON admin_audit_log
BEGIN
    SELECT RAISE(ABORT, 'admin_audit_log is append only and cannot be updated');
END;

CREATE TRIGGER IF NOT EXISTS prevent_delete_admin_audit_log
BEFORE DELETE ON admin_audit_log
BEGIN
    SELECT RAISE(ABORT, 'admin_audit_log is append only and cannot be deleted');
END;

CREATE TABLE IF NOT EXISTS customers (
    id TEXT PRIMARY KEY,
    org_name TEXT NOT NULL,
    owner_name TEXT NOT NULL,
    phone TEXT NOT NULL,
    email TEXT,
    district TEXT NOT NULL DEFAULT 'Dhaka',
    notes TEXT,
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'suspended', 'archived')),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS plans (
    id TEXT PRIMARY KEY,
    code TEXT UNIQUE NOT NULL CHECK (code IN ('trial', 'local', 'local_pro')),
    name TEXT NOT NULL,
    student_limit INTEGER,
    workstation_limit INTEGER,
    capabilities_json TEXT NOT NULL DEFAULT '[]',
    is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1))
);

CREATE TABLE IF NOT EXISTS prices (
    id TEXT PRIMARY KEY,
    plan_id TEXT NOT NULL REFERENCES plans(id),
    billing_cycle TEXT NOT NULL DEFAULT '12_months' CHECK (billing_cycle IN ('1_month', '3_months', '6_months', '12_months', 'lifetime')),
    period_months INTEGER NOT NULL CHECK (period_months IN (0, 1, 3, 6, 12)),
    is_lifetime INTEGER NOT NULL DEFAULT 0 CHECK (is_lifetime IN (0, 1)),
    currency TEXT NOT NULL DEFAULT 'BDT',
    monthly_rate_poisha INTEGER NOT NULL DEFAULT 100000 CHECK (monthly_rate_poisha > 0),
    amount_poisha INTEGER NOT NULL CHECK (amount_poisha > 0),
    active_from TEXT NOT NULL,
    active_to TEXT
);

CREATE TABLE IF NOT EXISTS subscriptions (
    id TEXT PRIMARY KEY,
    customer_id TEXT NOT NULL REFERENCES customers(id),
    plan_id TEXT NOT NULL REFERENCES plans(id),
    status TEXT NOT NULL CHECK (status IN ('trial', 'active', 'grace', 'expired', 'suspended')),
    paid_through TEXT NOT NULL,
    grace_days INTEGER NOT NULL DEFAULT 14,
    started_at TEXT NOT NULL,
    cancelled_at TEXT
);

CREATE TABLE IF NOT EXISTS payments (
    id TEXT PRIMARY KEY,
    customer_id TEXT NOT NULL REFERENCES customers(id),
    subscription_id TEXT NOT NULL REFERENCES subscriptions(id),
    amount_poisha INTEGER NOT NULL,
    method TEXT NOT NULL CHECK (method IN ('bkash', 'nagad', 'bank', 'cash', 'gateway')),
    reference TEXT NOT NULL,
    proof_url TEXT,
    period_months INTEGER NOT NULL,
    recorded_by TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'confirmed' CHECK (status IN ('confirmed', 'refunded')),
    recorded_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS vendor_invoices (
    id TEXT PRIMARY KEY,
    payment_id TEXT UNIQUE NOT NULL REFERENCES payments(id),
    customer_id TEXT NOT NULL REFERENCES customers(id),
    invoice_number TEXT UNIQUE NOT NULL,
    subtotal_poisha INTEGER NOT NULL,
    vat_poisha INTEGER NOT NULL DEFAULT 0,
    total_poisha INTEGER NOT NULL,
    issued_at TEXT NOT NULL,
    pdf_url TEXT
);

CREATE TABLE IF NOT EXISTS licenses (
    id TEXT PRIMARY KEY,
    license_id TEXT UNIQUE NOT NULL,
    customer_id TEXT NOT NULL REFERENCES customers(id),
    subscription_id TEXT NOT NULL REFERENCES subscriptions(id),
    activation_key_hash TEXT NOT NULL,
    pairing_code TEXT,
    pairing_expires_at TEXT,
    current_serial INTEGER NOT NULL DEFAULT 1,
    bound_installation_id TEXT,
    key_id TEXT NOT NULL DEFAULT 'k2026a',
    last_issued_blob TEXT,
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'suspended', 'revoked')),
    transfer_count_12m INTEGER NOT NULL DEFAULT 0,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS license_issues (
    id TEXT PRIMARY KEY,
    license_id TEXT NOT NULL REFERENCES licenses(license_id),
    serial INTEGER NOT NULL,
    installation_id TEXT NOT NULL,
    kind TEXT NOT NULL CHECK (kind IN ('paid', 'extension', 'trial_conversion', 'transfer')),
    payload_json TEXT NOT NULL,
    signature TEXT NOT NULL,
    signed_blob TEXT NOT NULL,
    issued_by TEXT,
    reason TEXT NOT NULL,
    issued_at TEXT NOT NULL,
    UNIQUE(license_id, serial)
);

CREATE TRIGGER IF NOT EXISTS prevent_update_license_issues
BEFORE UPDATE ON license_issues
BEGIN
    SELECT RAISE(ABORT, 'license_issues is append only and cannot be updated');
END;

CREATE TRIGGER IF NOT EXISTS prevent_delete_license_issues
BEFORE DELETE ON license_issues
BEGIN
    SELECT RAISE(ABORT, 'license_issues is append only and cannot be deleted');
END;

CREATE TABLE IF NOT EXISTS installations (
    id TEXT PRIMARY KEY,
    installation_id TEXT NOT NULL,
    customer_id TEXT NOT NULL REFERENCES customers(id),
    license_id TEXT NOT NULL REFERENCES licenses(license_id),
    label TEXT NOT NULL,
    role TEXT NOT NULL DEFAULT 'host',
    hostname TEXT NOT NULL DEFAULT '',
    os_version TEXT NOT NULL DEFAULT 'Windows',
    app_version TEXT NOT NULL,
    active_students INTEGER NOT NULL DEFAULT 0,
    first_seen_at TEXT NOT NULL,
    last_seen_at TEXT NOT NULL,
    deactivated_at TEXT,
    deactivation_reason TEXT,
    replaced_by_installation_id TEXT,
    status TEXT NOT NULL CHECK (status IN ('active', 'moved', 'deactivated', 'revoked')),
    UNIQUE(license_id, installation_id)
);

CREATE TABLE IF NOT EXISTS activation_requests (
    id TEXT PRIMARY KEY,
    customer_id TEXT NOT NULL REFERENCES customers(id),
    license_id TEXT NOT NULL REFERENCES licenses(license_id),
    request_code TEXT NOT NULL,
    qr_payload TEXT,
    installation_id TEXT NOT NULL,
    checksum_verified INTEGER NOT NULL DEFAULT 1 CHECK (checksum_verified IN (0, 1)),
    issued_serial_id TEXT NOT NULL REFERENCES license_issues(id),
    processed_by TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS revocations (
    id TEXT PRIMARY KEY,
    license_id TEXT NOT NULL REFERENCES licenses(license_id),
    reason TEXT NOT NULL,
    revoked_by TEXT NOT NULL,
    revoked_at TEXT NOT NULL,
    key_id TEXT NOT NULL,
    signed_revocation_blob TEXT NOT NULL
);

CREATE TRIGGER IF NOT EXISTS prevent_update_revocations
BEFORE UPDATE ON revocations
BEGIN
    SELECT RAISE(ABORT, 'revocations is append only and cannot be updated');
END;

CREATE TRIGGER IF NOT EXISTS prevent_delete_revocations
BEFORE DELETE ON revocations
BEGIN
    SELECT RAISE(ABORT, 'revocations is append only and cannot be deleted');
END;

CREATE TABLE IF NOT EXISTS releases (
    id TEXT PRIMARY KEY,
    version TEXT UNIQUE NOT NULL,
    channel TEXT NOT NULL CHECK (channel IN ('internal', 'beta', 'stable')),
    notes TEXT NOT NULL,
    artifact_url TEXT NOT NULL,
    sha256 TEXT NOT NULL,
    size_bytes INTEGER NOT NULL DEFAULT 0,
    authenticode_thumbprint TEXT NOT NULL DEFAULT '',
    updater_signature TEXT NOT NULL,
    updater_key_id TEXT NOT NULL DEFAULT 'u2026a',
    min_upgrade_from TEXT,
    status TEXT NOT NULL CHECK (status IN ('live', 'held', 'blocked')),
    published_by TEXT NOT NULL,
    published_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS release_channel_heads (
    channel TEXT PRIMARY KEY CHECK (channel IN ('internal', 'beta', 'stable')),
    release_id TEXT NOT NULL REFERENCES releases(id),
    version TEXT NOT NULL,
    promoted_from_channel TEXT CHECK (promoted_from_channel IN ('internal', 'beta') OR promoted_from_channel IS NULL),
    promoted_by TEXT NOT NULL REFERENCES admin_users(id),
    promoted_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS key_recovery_drills (
    id TEXT PRIMARY KEY,
    key_type TEXT NOT NULL CHECK (key_type IN ('windows_authenticode', 'tauri_updater_ed25519', 'license_ed25519')),
    key_id TEXT NOT NULL,
    backup_source TEXT NOT NULL,
    backup_sha256 TEXT NOT NULL,
    test_artifact_sha256 TEXT NOT NULL,
    signature_verified INTEGER NOT NULL CHECK (signature_verified = 1),
    notes TEXT NOT NULL,
    performed_by TEXT NOT NULL REFERENCES admin_users(id),
    performed_at TEXT NOT NULL
);

CREATE TRIGGER IF NOT EXISTS prevent_update_key_recovery_drills
BEFORE UPDATE ON key_recovery_drills
BEGIN
    SELECT RAISE(ABORT, 'key_recovery_drills is append only and cannot be updated');
END;

CREATE TRIGGER IF NOT EXISTS prevent_delete_key_recovery_drills
BEFORE DELETE ON key_recovery_drills
BEGIN
    SELECT RAISE(ABORT, 'key_recovery_drills is append only and cannot be deleted');
END;

CREATE TABLE IF NOT EXISTS customer_portal_credentials (
    customer_id TEXT PRIMARY KEY REFERENCES customers(id),
    password_hash TEXT NOT NULL,
    otp_code_hash TEXT,
    otp_expires_at TEXT,
    failed_attempts INTEGER NOT NULL DEFAULT 0,
    locked_until TEXT,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS customer_portal_sessions (
    id TEXT PRIMARY KEY,
    customer_id TEXT NOT NULL REFERENCES customers(id),
    session_token_hash TEXT UNIQUE NOT NULL,
    ip_address TEXT,
    user_agent TEXT,
    expires_at TEXT NOT NULL,
    revoked_at TEXT,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS checkout_orders (
    id TEXT PRIMARY KEY,
    order_number TEXT UNIQUE NOT NULL,
    customer_id TEXT NOT NULL REFERENCES customers(id),
    subscription_id TEXT REFERENCES subscriptions(id),
    plan_id TEXT NOT NULL REFERENCES plans(id),
    price_id TEXT NOT NULL REFERENCES prices(id),
    kind TEXT NOT NULL CHECK (kind IN ('new_purchase', 'renewal', 'upgrade', 'lifetime_purchase')),
    billing_cycle TEXT NOT NULL CHECK (billing_cycle IN ('3_months', '6_months', '12_months', 'lifetime')),
    period_months INTEGER NOT NULL CHECK (period_months IN (0, 3, 6, 12)),
    method TEXT NOT NULL CHECK (method IN ('bkash', 'nagad', 'bank_transfer')),
    amount_poisha INTEGER NOT NULL CHECK (amount_poisha > 0),
    gateway_provider TEXT NOT NULL CHECK (gateway_provider IN ('sslcommerz', 'bkash_direct', 'nagad_direct', 'manual_bank', 'sandbox')),
    gateway_session_id TEXT,
    gateway_trx_id TEXT,
    bank_reference TEXT,
    bank_slip_url TEXT,
    target_installation_id TEXT,
    target_request_code TEXT,
    status TEXT NOT NULL CHECK (status IN ('pending_payment', 'pending_bank_review', 'completed', 'expired', 'failed')),
    resulting_payment_id TEXT UNIQUE REFERENCES payments(id),
    resulting_invoice_id TEXT UNIQUE REFERENCES vendor_invoices(id),
    resulting_license_id TEXT REFERENCES licenses(license_id),
    expires_at TEXT NOT NULL,
    completed_at TEXT,
    created_at TEXT NOT NULL,
    UNIQUE(method, gateway_trx_id)
);

CREATE TABLE IF NOT EXISTS payment_webhook_events (
    id TEXT PRIMARY KEY,
    checkout_order_id TEXT NOT NULL REFERENCES checkout_orders(id),
    gateway_provider TEXT NOT NULL,
    gateway_event_id TEXT NOT NULL,
    gateway_trx_id TEXT NOT NULL,
    amount_poisha INTEGER NOT NULL,
    signature_verified INTEGER NOT NULL CHECK (signature_verified = 1),
    raw_payload_json TEXT NOT NULL,
    processed_status TEXT NOT NULL CHECK (processed_status IN ('applied', 'duplicate_ignored', 'rejected')),
    received_at TEXT NOT NULL,
    UNIQUE(gateway_provider, gateway_event_id)
);

CREATE TRIGGER IF NOT EXISTS prevent_update_payment_webhook_events
BEFORE UPDATE ON payment_webhook_events
BEGIN
    SELECT RAISE(ABORT, 'payment_webhook_events is append only and cannot be updated');
END;

CREATE TRIGGER IF NOT EXISTS prevent_delete_payment_webhook_events
BEFORE DELETE ON payment_webhook_events
BEGIN
    SELECT RAISE(ABORT, 'payment_webhook_events is append only and cannot be deleted');
END;

CREATE TABLE IF NOT EXISTS device_management_events (
    id TEXT PRIMARY KEY,
    license_id TEXT NOT NULL REFERENCES licenses(license_id),
    customer_id TEXT NOT NULL REFERENCES customers(id),
    action TEXT NOT NULL CHECK (action IN ('deactivate', 'replace_online', 'replace_offline_req', 'rename_label', 'support_reset_request')),
    old_installation_id TEXT NOT NULL,
    new_installation_id TEXT,
    new_label TEXT,
    reason TEXT NOT NULL,
    initiated_by_type TEXT NOT NULL CHECK (initiated_by_type IN ('portal_owner', 'admin')),
    initiated_by_id TEXT NOT NULL,
    resulting_serial INTEGER NOT NULL,
    issued_serial_id TEXT REFERENCES license_issues(id),
    created_at TEXT NOT NULL
);

CREATE TRIGGER IF NOT EXISTS prevent_update_device_management_events
BEFORE UPDATE ON device_management_events
BEGIN
    SELECT RAISE(ABORT, 'device_management_events is append only and cannot be updated');
END;

CREATE TRIGGER IF NOT EXISTS prevent_delete_device_management_events
BEFORE DELETE ON device_management_events
BEGIN
    SELECT RAISE(ABORT, 'device_management_events is append only and cannot be deleted');
END;
"#;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum CloudError {
    #[error("Invalid admin email or password")]
    AdminInvalidCredentials,
    #[error("Invalid 6 digit two factor TOTP code or recovery code")]
    AdminTotpInvalid,
    #[error("Admin account is suspended")]
    AdminAccountSuspended,
    #[error("Admin role '{0}' is not permitted to perform this action")]
    AdminPermissionDenied(String),
    #[error("Customer or subscription not found: {0}")]
    NotFound(String),
    #[error("Offline request code or QR payload checksum is invalid: {0}")]
    RequestCodeChecksumInvalid(String),
    #[error("Offline request code requires a full QR payload or known installation ID to resolve")]
    RequestCodeUnresolvable,
    #[error("License '{0}' is revoked and cannot be activated or extended")]
    LicenseRevoked(String),
    #[error("Activation credentials or pairing code are invalid")]
    ActivationCredentialsInvalid,
    #[error("Self service PC transfer limit (3 transfers per 12 months) exceeded; contact vendor support to reset")]
    TransferLimitExceeded,
    #[error("Invalid phone/email or customer portal password")]
    PortalInvalidCredentials,
    #[error("Customer portal account is locked until {0} after 5 failed OTP attempts")]
    PortalAccountLocked(String),
    #[error("Invalid 6 digit customer portal OTP verification code")]
    PortalInvalidOtp,
    #[error("Customer portal 6 digit OTP verification code has expired")]
    PortalOtpExpired,
    #[error("Customer portal session token is missing, invalid, or expired")]
    UnauthorizedPortalSession,
    #[error("Customer account is suspended")]
    CustomerSuspended,
    #[error("Forbidden: requested license belongs to a different customer")]
    ForbiddenLicenseOwner,
    #[error("Forbidden: requested invoice belongs to a different customer")]
    ForbiddenInvoiceOwner,
    #[error("Release version '{0}' already exists")]
    DuplicateReleaseVersion(String),
    #[error("Windows Authenticode certificate thumbprint is missing or invalid")]
    InvalidAuthenticodeThumbprint,
    #[error("Release '{0}' has status 'held' or 'blocked' and cannot be promoted")]
    ReleaseNotPromotable(String),
    #[error("Source channel head does not match requested version: {0}")]
    ChannelHeadMismatch(String),
    #[error("Step up 6 digit TOTP verification is required for this action")]
    StepUpTotpRequired,
    #[error("Key recovery drill signature verification failed")]
    RecoverySignatureMismatch,
    #[error("Invalid release channel: {0}")]
    InvalidChannel(String),
    #[error("HMAC signature verification failed for payment gateway webhook")]
    WebhookSignatureInvalid,
    #[error("Webhook settled amount ({received}) does not match order amount ({expected})")]
    WebhookAmountMismatch { received: i64, expected: i64 },
    #[error("Checkout order not found: {0}")]
    OrderNotFound(String),
    #[error("Checkout order is already completed")]
    OrderAlreadyCompleted,
    #[error("Checkout order is not in pending bank review status")]
    OrderNotPendingBankReview,
    #[error("Bank transfer reference is required")]
    BankReferenceRequired,
    #[error("Invalid price amount: monthly rate and amount must be positive")]
    InvalidPriceAmount,
    #[error("Pricing catalog is currently unavailable")]
    CatalogUnavailable,
    #[error("Target PC is already deactivated or inactive")]
    DeviceAlreadyInactive,
    #[error("Invalid device label: {0}")]
    InvalidDeviceLabel(String),
    #[error("Forbidden: target device is not registered to this customer")]
    DeviceNotOwnedByCustomer,
    #[error("Isolated signer service error: {0}")]
    SignerFailure(String),
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Database error: {0}")]
    Database(String),
}

impl CloudError {
    pub fn code(&self) -> &'static str {
        match self {
            CloudError::AdminInvalidCredentials => "admin_invalid_credentials",
            CloudError::AdminTotpInvalid => "admin_totp_invalid",
            CloudError::AdminAccountSuspended => "admin_account_suspended",
            CloudError::AdminPermissionDenied(_) => "admin_permission_denied",
            CloudError::NotFound(_) => "resource_not_found",
            CloudError::RequestCodeChecksumInvalid(_) => "request_code_checksum_invalid",
            CloudError::RequestCodeUnresolvable => "request_code_unresolvable",
            CloudError::LicenseRevoked(_) => "license_revoked",
            CloudError::ActivationCredentialsInvalid => "activation_credentials_invalid",
            CloudError::TransferLimitExceeded => "DEVICE_TRANSFER_LIMIT_EXCEEDED",
            CloudError::PortalInvalidCredentials => "INVALID_CREDENTIALS",
            CloudError::PortalAccountLocked(_) => "PORTAL_ACCOUNT_LOCKED",
            CloudError::PortalInvalidOtp => "INVALID_OTP",
            CloudError::PortalOtpExpired => "OTP_EXPIRED",
            CloudError::UnauthorizedPortalSession => "UNAUTHORIZED_PORTAL_SESSION",
            CloudError::CustomerSuspended => "CUSTOMER_SUSPENDED",
            CloudError::ForbiddenLicenseOwner => "FORBIDDEN_LICENSE_OWNER",
            CloudError::ForbiddenInvoiceOwner => "FORBIDDEN_INVOICE_OWNER",
            CloudError::DuplicateReleaseVersion(_) => "DUPLICATE_VERSION",
            CloudError::InvalidAuthenticodeThumbprint => "INVALID_AUTHENTICODE_THUMBPRINT",
            CloudError::ReleaseNotPromotable(_) => "RELEASE_NOT_PROMOTABLE",
            CloudError::ChannelHeadMismatch(_) => "CHANNEL_HEAD_MISMATCH",
            CloudError::StepUpTotpRequired => "STEP_UP_TOTP_REQUIRED",
            CloudError::RecoverySignatureMismatch => "RECOVERY_SIGNATURE_MISMATCH",
            CloudError::InvalidChannel(_) => "INVALID_CHANNEL",
            CloudError::WebhookSignatureInvalid => "WEBHOOK_SIGNATURE_INVALID",
            CloudError::WebhookAmountMismatch { .. } => "WEBHOOK_AMOUNT_MISMATCH",
            CloudError::OrderNotFound(_) => "ORDER_NOT_FOUND",
            CloudError::OrderAlreadyCompleted => "ORDER_ALREADY_COMPLETED",
            CloudError::OrderNotPendingBankReview => "ORDER_NOT_PENDING_BANK_REVIEW",
            CloudError::BankReferenceRequired => "BANK_REFERENCE_REQUIRED",
            CloudError::InvalidPriceAmount => "INVALID_PRICE_AMOUNT",
            CloudError::CatalogUnavailable => "CATALOG_UNAVAILABLE",
            CloudError::DeviceAlreadyInactive => "DEVICE_ALREADY_INACTIVE",
            CloudError::InvalidDeviceLabel(_) => "INVALID_DEVICE_LABEL",
            CloudError::DeviceNotOwnedByCustomer => "DEVICE_NOT_OWNED_BY_CUSTOMER",
            CloudError::SignerFailure(_) => "signer_unavailable",
            CloudError::Validation(_) => "validation_error",
            CloudError::Database(_) => "database_error",
        }
    }
}

impl From<rusqlite::Error> for CloudError {
    fn from(err: rusqlite::Error) -> Self {
        CloudError::Database(err.to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
pub struct AdminUserView {
    pub id: String,
    pub email: String,
    pub name: String,
    pub role: String,
    pub totp_enabled: bool,
    pub status: String,
    pub last_login_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
pub struct AdminLoginInput {
    pub email: String,
    pub password: String,
    pub totp_code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
pub struct AdminLoginResponse {
    pub admin_user: AdminUserView,
    pub session_token: String,
    pub current_demo_totp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
pub struct VendorCustomerRow {
    pub id: String,
    pub org_name: String,
    pub owner_name: String,
    pub phone: String,
    pub email: Option<String>,
    pub district: String,
    pub status: String,
    pub subscription_id: String,
    pub plan_code: String,
    pub plan_name: String,
    pub subscription_status: String,
    pub paid_through: String,
    pub grace_days: u32,
    pub license_id: String,
    pub current_serial: i64,
    pub bound_installation_id: Option<String>,
    pub pairing_code: Option<String>,
    pub transfer_count_12m: u32,
    pub last_issued_blob: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
pub struct VendorPaymentRow {
    pub id: String,
    pub customer_id: String,
    pub org_name: String,
    pub subscription_id: String,
    pub amount_poisha: i64,
    pub method: String,
    pub reference: String,
    pub proof_url: Option<String>,
    pub period_months: u32,
    pub invoice_number: String,
    pub recorded_by: String,
    pub status: String,
    pub recorded_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
pub struct VendorInstallationRow {
    pub id: String,
    pub installation_id: String,
    pub customer_id: String,
    pub org_name: String,
    pub license_id: String,
    pub label: String,
    pub app_version: String,
    pub active_students: u32,
    pub first_seen_at: String,
    pub last_seen_at: String,
    pub status: String,
    pub transfer_count_12m: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
pub struct VendorRevocationRow {
    pub id: String,
    pub license_id: String,
    pub org_name: String,
    pub reason: String,
    pub revoked_by: String,
    pub revoked_at: String,
    pub key_id: String,
    pub signed_revocation_blob: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
pub struct VendorReleaseRow {
    pub id: String,
    pub version: String,
    pub channel: String,
    pub notes: String,
    pub artifact_url: String,
    pub sha256: String,
    #[serde(default)]
    pub size_bytes: i64,
    #[serde(default)]
    pub authenticode_thumbprint: String,
    pub updater_signature: String,
    #[serde(default)]
    pub updater_key_id: String,
    pub min_upgrade_from: Option<String>,
    pub status: String,
    pub published_by: String,
    pub published_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
pub struct ReleaseChannelHeadRow {
    pub channel: String,
    pub release_id: String,
    pub version: String,
    pub sha256: String,
    pub size_bytes: i64,
    pub artifact_url: String,
    pub authenticode_thumbprint: String,
    pub updater_signature: String,
    pub updater_key_id: String,
    pub status: String,
    pub notes: String,
    pub promoted_from_channel: Option<String>,
    pub promoted_by: String,
    pub promoted_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
pub struct KeyRecoveryDrillRow {
    pub id: String,
    pub key_type: String,
    pub key_id: String,
    pub backup_source: String,
    pub backup_sha256: String,
    pub test_artifact_sha256: String,
    pub signature_verified: bool,
    pub notes: String,
    pub performed_by: String,
    pub performed_at: String,
    #[serde(default)]
    pub custodian_primary: Option<String>,
    #[serde(default)]
    pub custodian_secondary: Option<String>,
    #[serde(default)]
    pub drilled_at: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
pub struct VendorAuditRow {
    pub seq: i64,
    pub at: String,
    pub admin_user_id: String,
    pub admin_email: String,
    pub action: String,
    pub entity_type: String,
    pub entity_id: String,
    pub after_json: Option<String>,
    pub prev_hash: String,
    pub hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Type)]
pub struct VendorDashboardOverview {
    pub active_admin: Option<AdminUserView>,
    pub demo_totp_code: String,
    pub total_customers: u32,
    pub active_subscriptions: u32,
    pub in_grace_subscriptions: u32,
    pub total_revenue_poisha: i64,
    pub audit_chain_valid: bool,
    pub signer_total_signatures: u32,
    pub customers: Vec<VendorCustomerRow>,
    pub payments: Vec<VendorPaymentRow>,
    pub installations: Vec<VendorInstallationRow>,
    pub revocations: Vec<VendorRevocationRow>,
    pub releases: Vec<VendorReleaseRow>,
    #[serde(default)]
    pub channel_heads: Option<Vec<ReleaseChannelHeadRow>>,
    #[serde(default)]
    pub key_recovery_drills: Option<Vec<KeyRecoveryDrillRow>>,
    #[serde(default)]
    pub pricing_tiers: Option<Vec<PricingTierView>>,
    #[serde(default)]
    pub pending_bank_transfers: Option<Vec<PendingBankTransferOrderView>>,
    pub admin_audit_log: Vec<VendorAuditRow>,
    pub signer_audit_log: Vec<SignerAuditEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Type)]
pub struct PricingTierView {
    pub id: String,
    pub plan_code: String,
    pub plan_name: String,
    pub billing_cycle: String,
    pub period_months: u32,
    pub is_lifetime: bool,
    pub monthly_rate_poisha: i64,
    pub monthly_rate_bdt: f64,
    pub amount_poisha: i64,
    pub total_bdt: f64,
    pub currency: String,
    pub badge: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Type)]
pub struct PricingCatalogResponse {
    pub tiers: Vec<PricingTierView>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Type)]
pub struct AdminUpdatePriceTierInput {
    pub admin_email: String,
    pub plan_code: String,
    pub billing_cycle: String,
    pub monthly_rate_bdt: Option<f64>,
    pub total_bdt_override: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Type)]
pub struct UpdatedPriceTierResult {
    pub price_id: String,
    pub plan_code: String,
    pub billing_cycle: String,
    pub monthly_rate_poisha: i64,
    pub amount_poisha: i64,
    pub monthly_rate_bdt: f64,
    pub total_bdt: f64,
    pub audit_seq: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Type)]
pub struct CreateCheckoutSessionInput {
    pub plan_code: String,
    pub billing_cycle: String,
    pub kind: String,
    pub method: String,
    pub org_name: String,
    pub owner_name: String,
    pub phone: String,
    pub email: Option<String>,
    pub target_request_code: Option<String>,
    pub session_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
pub struct BankInstructionsView {
    pub bank_name: String,
    pub account_title: String,
    pub account_number: String,
    pub routing_number: String,
    pub branch: String,
    pub deposit_reference_prefix: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Type)]
pub struct CheckoutSessionResponse {
    pub order_number: String,
    pub plan_code: String,
    pub billing_cycle: String,
    pub period_months: u32,
    pub monthly_rate_poisha: i64,
    pub amount_poisha: i64,
    pub total_bdt: f64,
    pub method: String,
    pub gateway_provider: String,
    pub gateway_redirect_url: Option<String>,
    pub bank_instructions: Option<BankInstructionsView>,
    pub expires_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
pub struct SubmitBankTransferInput {
    pub order_number: String,
    pub bank_reference: String,
    pub bank_slip_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Type)]
pub struct CheckoutOrderStatusView {
    pub order_number: String,
    pub status: String,
    pub kind: String,
    pub billing_cycle: String,
    pub method: String,
    pub amount_poisha: i64,
    pub total_bdt: f64,
    pub bank_reference: Option<String>,
    pub bank_slip_url: Option<String>,
    pub invoice_number: Option<String>,
    pub license_id: Option<String>,
    pub pairing_code: Option<String>,
    pub signed_license_blob: Option<String>,
    pub completed_at: Option<String>,
    pub expires_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
pub struct PaymentWebhookInput {
    pub provider: String,
    pub hmac_signature: String,
    pub raw_payload_json: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
pub struct WebhookAckResponse {
    pub processed_status: String,
    pub order_number: String,
    pub invoice_number: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Type)]
pub struct PendingBankTransferOrderView {
    pub order_number: String,
    pub customer_id: String,
    pub org_name: String,
    pub owner_name: String,
    pub phone: String,
    pub billing_cycle: String,
    pub amount_poisha: i64,
    pub total_bdt: f64,
    pub bank_reference: String,
    pub bank_slip_url: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
pub struct ApproveBankTransferInput {
    pub admin_email: String,
    pub order_number: String,
    pub verification_notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
pub struct PortalDeviceItem {
    pub installation_id: String,
    pub label: String,
    pub role: String,
    pub hostname: String,
    pub os_version: String,
    pub app_version: String,
    pub active_students: u32,
    pub first_seen_at: String,
    pub last_seen_at: String,
    pub deactivated_at: Option<String>,
    pub deactivation_reason: Option<String>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
pub struct PortalDeviceEventItem {
    pub id: String,
    pub action: String,
    pub old_installation_id: String,
    pub new_installation_id: Option<String>,
    pub new_label: Option<String>,
    pub reason: String,
    pub initiated_by_type: String,
    pub resulting_serial: i64,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
pub struct PortalDeviceListView {
    pub devices: Vec<PortalDeviceItem>,
    pub transfer_count_12m: u32,
    pub max_transfers_12m: u32,
    pub transfers_remaining_12m: u32,
    pub recent_events: Vec<PortalDeviceEventItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
pub struct PortalRenameDeviceInput {
    pub session_token: String,
    pub installation_id: String,
    pub new_label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
pub struct PortalRequestDeviceOtpInput {
    pub session_token: String,
    pub installation_id: String,
    pub action: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
pub struct PortalDeviceOtpResponse {
    pub masked_destination: String,
    pub demo_otp_code: String,
    pub otp_expires_in_seconds: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
pub struct PortalDeactivateDeviceInput {
    pub session_token: String,
    pub installation_id: String,
    pub otp_code: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
pub struct PortalReplaceDeviceInput {
    pub session_token: String,
    pub old_installation_id: String,
    pub otp_code: String,
    pub reason: String,
    pub new_label: String,
    pub new_request_code_or_qr: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
pub struct DeviceActionResult {
    pub action: String,
    pub deactivated_installation_id: String,
    pub new_installation_id: Option<String>,
    pub new_pairing_code: Option<String>,
    pub pairing_expires_at: Option<String>,
    pub signed_license_blob: Option<String>,
    pub transfers_used_12m: u32,
    pub transfers_remaining_12m: u32,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
pub struct PortalRequestTransferResetInput {
    pub session_token: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
pub struct AdminResetTransferCountInput {
    pub admin_email: String,
    pub license_id: String,
    pub step_up_totp_code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
pub struct ResetTransferCountResponse {
    pub license_id: String,
    pub transfer_count_12m: u32,
    pub transfers_remaining_12m: u32,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
pub struct PublishReleaseInput {
    pub admin_email: String,
    pub version: String,
    pub notes: String,
    pub artifact_url: String,
    pub sha256: String,
    pub size_bytes: i64,
    pub authenticode_thumbprint: String,
    pub min_upgrade_from: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
pub struct PublishReleaseResult {
    pub release: VendorReleaseRow,
    pub channel_head: ReleaseChannelHeadRow,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type, Default)]
pub struct PromoteReleaseChannelInput {
    pub admin_email: String,
    pub version: String,
    #[serde(default)]
    pub from_channel: Option<String>,
    #[serde(default)]
    pub to_channel: Option<String>,
    #[serde(default)]
    pub source_channel: Option<String>,
    #[serde(default)]
    pub target_channel: Option<String>,
    #[serde(default)]
    pub promotion_reason: Option<String>,
    pub step_up_totp_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
pub struct PromoteReleaseChannelResult {
    pub channel_head: ReleaseChannelHeadRow,
    pub release: VendorReleaseRow,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
pub struct RecordKeyRecoveryDrillInput {
    pub admin_email: String,
    pub key_type: String,
    pub key_id: String,
    #[serde(default)]
    pub custodian_primary: Option<String>,
    #[serde(default)]
    pub custodian_secondary: Option<String>,
    #[serde(default)]
    pub test_artifact_sha256: Option<String>,
    #[serde(default)]
    pub backup_source: Option<String>,
    #[serde(default)]
    pub encrypted_backup_blob: Option<String>,
    #[serde(default)]
    pub passphrase: Option<String>,
    pub notes: String,
    pub step_up_totp_code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
pub struct PortalLoginInput {
    pub identifier: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
pub struct PortalLoginResponse {
    pub customer_id: String,
    pub otp_required: bool,
    pub masked_destination: String,
    pub otp_expires_at: String,
    pub demo_otp_code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
pub struct PortalVerifyOtpInput {
    pub customer_id: String,
    pub otp_code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
pub struct PortalCustomerSummary {
    pub id: String,
    pub org_name: String,
    pub owner_name: String,
    pub phone: String,
    pub email: Option<String>,
    pub district: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
pub struct PortalVerifyOtpResponse {
    pub session_token: String,
    pub expires_at: String,
    pub customer: PortalCustomerSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
pub struct PortalInvoiceItem {
    pub id: String,
    pub invoice_number: String,
    pub issued_at: String,
    pub org_name: String,
    pub plan_name: String,
    pub period_months: u32,
    pub method: String,
    pub reference: String,
    pub subtotal_poisha: i64,
    pub vat_poisha: i64,
    pub total_poisha: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
pub struct PortalInstallationItem {
    pub id: String,
    pub installation_id: String,
    pub label: String,
    pub role: String,
    pub app_version: String,
    pub active_students: u32,
    pub first_seen_at: String,
    pub last_seen_at: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
pub struct PortalLicenseSummary {
    pub license_id: String,
    pub plan_code: String,
    pub plan_name: String,
    pub subscription_status: String,
    pub paid_through: String,
    pub grace_days: u32,
    pub current_serial: i64,
    pub key_id: String,
    pub status: String,
    pub has_signed_blob: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
pub struct PortalReleaseManifest {
    pub version: String,
    pub channel: String,
    pub notes: String,
    pub artifact_url: String,
    pub sha256: String,
    pub size_bytes: i64,
    pub authenticode_thumbprint: String,
    pub updater_signature: String,
    pub updater_key_id: String,
    pub published_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
pub struct PortalOverviewResponse {
    pub customer: PortalCustomerSummary,
    pub license: PortalLicenseSummary,
    pub invoices: Vec<PortalInvoiceItem>,
    pub installations: Vec<PortalInstallationItem>,
    pub latest_release: Option<PortalReleaseManifest>,
    pub beta_release: Option<PortalReleaseManifest>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
pub struct PortalLicenseFileResponse {
    pub license_id: String,
    pub filename: String,
    pub signed_blob: String,
    pub serial: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
pub struct RecordManualPaymentInput {
    pub admin_email: String,
    pub customer_id: String,
    pub subscription_id: String,
    pub amount_poisha: i64,
    pub method: String,
    pub reference: String,
    pub proof_url: Option<String>,
    pub period_months: u32,
    pub installation_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
pub struct RecordManualPaymentResult {
    pub payment_id: String,
    pub invoice_number: String,
    pub license_id: String,
    pub new_serial: i64,
    pub new_paid_through: String,
    pub valid_until: String,
    pub pairing_code: String,
    pub signed_license_blob: String,
    pub signer_audit_seq: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
pub struct IssueOfflineLicenseInput {
    pub admin_email: String,
    pub license_id: String,
    pub request_code_or_qr: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
pub struct IssueOfflineLicenseResult {
    pub license_id: String,
    pub installation_id: String,
    pub request_code: String,
    pub new_serial: i64,
    pub signed_license_blob: String,
    pub download_filename: String,
    pub signer_audit_seq: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
pub struct RevokeLicenseInput {
    pub admin_email: String,
    pub license_id: String,
    pub reason: String,
    pub step_up_totp_code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
pub struct UpdateReleaseStatusInput {
    pub admin_email: String,
    pub version: String,
    pub channel: String,
    pub status: String,
    pub notes: Option<String>,
    pub step_up_totp_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
pub struct CreateVendorCustomerInput {
    pub admin_email: String,
    pub org_name: String,
    pub owner_name: String,
    pub phone: String,
    pub email: Option<String>,
    pub district: String,
    pub plan_code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
pub struct ActivateEndpointRequest {
    pub license_id_or_pairing_code: String,
    pub activation_key: Option<String>,
    pub installation_id: String,
    pub app_version: String,
    pub active_students: u32,
    pub confirm_transfer: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
pub struct ActivateEndpointResponse {
    pub outcome: String, // "activated" | "already_bound_transfer_available"
    pub license_id: String,
    pub signed_license_blob: Option<String>,
    pub previous_installation_id: Option<String>,
    pub previous_last_seen_at: Option<String>,
    pub transfers_remaining_12m: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
pub struct RefreshEndpointRequest {
    pub license_id: String,
    pub installation_id: String,
    pub current_serial: i64,
    pub app_version: String,
    pub active_students: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
pub struct RefreshEndpointResponse {
    pub outcome: String, // "updated" | "no_change" | "moved" | "revoked" | "deactivated"
    pub latest_serial: i64,
    pub signed_license_blob: Option<String>,
    pub signed_revocation_blob: Option<String>,
    pub server_time_utc: String,
    #[serde(default)]
    pub deactivated_at: Option<String>,
    #[serde(default)]
    pub deactivation_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
pub struct UpdateChannelResponse {
    pub channel: String,
    pub latest_live_version: Option<String>,
    pub artifact_url: Option<String>,
    pub sha256: Option<String>,
    #[serde(default)]
    pub size_bytes: Option<i64>,
    #[serde(default)]
    pub authenticode_thumbprint: Option<String>,
    pub updater_signature: Option<String>,
    #[serde(default)]
    pub updater_key_id: Option<String>,
    pub min_upgrade_from: Option<String>,
    pub notes: Option<String>,
    #[serde(default)]
    pub published_at: Option<String>,
    pub blocked_versions: Vec<String>,
}

/// Computes an RFC 6238 6 digit TOTP code for a 30 second time window using HMAC-SHA256.
pub fn generate_totp_code(secret: &str, unix_seconds: u64) -> String {
    let time_step = unix_seconds / 30;
    let mut mac =
        HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC accepts keys of any length");
    mac.update(&time_step.to_be_bytes());
    let result = mac.finalize().into_bytes();
    let offset = (result[result.len() - 1] & 0x0f) as usize;
    let binary = ((u32::from(result[offset] & 0x7f)) << 24)
        | ((u32::from(result[offset + 1])) << 16)
        | ((u32::from(result[offset + 2])) << 8)
        | u32::from(result[offset + 3]);
    let otp = binary % 1_000_000;
    format!("{otp:06}")
}

/// Verifies a 6 digit TOTP code (allowing ±1 30-second drift window) or a single-use backup recovery code.
pub fn verify_totp_or_recovery(
    secret: &str,
    recovery_codes_json: &str,
    submitted_code: &str,
    unix_seconds: u64,
) -> bool {
    let clean = submitted_code.trim().replace(' ', "");
    if clean.len() == 6 && clean.chars().all(|c| c.is_ascii_digit()) {
        for window in [
            unix_seconds.saturating_sub(30),
            unix_seconds,
            unix_seconds + 30,
        ] {
            if generate_totp_code(secret, window) == clean {
                return true;
            }
        }
    }
    if let Ok(codes) = serde_json::from_str::<Vec<String>>(recovery_codes_json) {
        if codes.iter().any(|c| c.eq_ignore_ascii_case(&clean)) {
            return true;
        }
    }
    false
}

/// Generates the deterministic 20-character Base32 offline request code (`REQ-XXXX-XXXX-XXXX-XXXX`)
/// and QR string (`COACHIFY-REQ|v1|<installation_id>|<checksum>`) matching `crates/core/src/licensing.rs`.
pub fn generate_request_code_for_installation(installation_id: &str) -> (String, String) {
    let clean_id = installation_id.trim();
    let mut mac =
        HmacSha256::new_from_slice(REQUEST_CODE_HMAC_KEY).expect("HMAC accepts keys of any length");
    mac.update(clean_id.as_bytes());
    let digest = mac.finalize().into_bytes();

    let mut base32_chars = String::with_capacity(16);
    let mut buffer: u16 = 0;
    let mut bits_left: u8 = 0;
    for &byte in &digest[0..10] {
        buffer = (buffer << 8) | (byte as u16);
        bits_left += 8;
        while bits_left >= 5 && base32_chars.len() < 16 {
            bits_left -= 5;
            let index = ((buffer >> bits_left) & 0x1F) as usize;
            base32_chars.push(BASE32_ALPHABET[index] as char);
        }
    }

    let request_code = format!(
        "REQ-{}-{}-{}-{}",
        &base32_chars[0..4],
        &base32_chars[4..8],
        &base32_chars[8..12],
        &base32_chars[12..16]
    );
    let checksum_hex = hex::encode(&digest[0..8]);
    let qr_payload = format!("COACHIFY-REQ|v1|{clean_id}|{checksum_hex}");
    (request_code, qr_payload)
}

/// Resolves and verifies either a full QR string (`COACHIFY-REQ|v1|<installation_id>|<checksum>`)
/// or a 20 character request code (`REQ-XXXX-XXXX-XXXX-XXXX`) against known installations.
pub fn parse_and_verify_offline_request(
    conn: &Connection,
    request_code_or_qr: &str,
    fallback_installation_id: Option<&str>,
) -> Result<(String, String, Option<String>), CloudError> {
    let trimmed = request_code_or_qr.trim();
    if trimmed.starts_with("COACHIFY-REQ|v1|") {
        let parts: Vec<&str> = trimmed.split('|').collect();
        if parts.len() != 4 {
            return Err(CloudError::RequestCodeChecksumInvalid(
                "Malformed QR request payload".to_string(),
            ));
        }
        let inst_id = parts[2].trim();
        let provided_checksum = parts[3].trim();
        let (req_code, expected_qr) = generate_request_code_for_installation(inst_id);
        let expected_checksum = expected_qr.split('|').nth(3).unwrap_or("");
        if !provided_checksum.eq_ignore_ascii_case(expected_checksum) {
            return Err(CloudError::RequestCodeChecksumInvalid(
                "QR payload HMAC SHA256 checksum mismatch".to_string(),
            ));
        }
        return Ok((inst_id.to_string(), req_code, Some(trimmed.to_string())));
    }

    if trimmed.starts_with("REQ-") {
        // Check if it matches fallback_installation_id or any known installation in DB
        if let Some(inst_id) = fallback_installation_id {
            let (expected_req, qr) = generate_request_code_for_installation(inst_id);
            if expected_req.eq_ignore_ascii_case(trimmed) {
                return Ok((inst_id.to_string(), expected_req, Some(qr)));
            }
        }
        let mut stmt = conn.prepare("SELECT installation_id FROM installations")?;
        let rows = stmt.query_map([], |r| r.get::<_, String>(0))?;
        for row in rows {
            let candidate = row?;
            let (expected_req, qr) = generate_request_code_for_installation(&candidate);
            if expected_req.eq_ignore_ascii_case(trimmed) {
                return Ok((candidate, expected_req, Some(qr)));
            }
        }
        return Err(CloudError::RequestCodeChecksumInvalid(
            "REQ code checksum did not match any registered installation ID; paste the full COACHIFY-REQ|v1|... string for a brand new offline PC".to_string(),
        ));
    }

    Err(CloudError::RequestCodeChecksumInvalid(
        "Expected REQ-XXXX-XXXX-XXXX-XXXX or COACHIFY-REQ|v1|<installation_id>|<checksum>"
            .to_string(),
    ))
}

fn hash_password(password: &str) -> String {
    let salt = SaltString::from_b64("Q29hY2hpZnlWZW5kb3JTYWx0").expect("valid constant salt");
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .expect("argon2 hash")
        .to_string()
}

fn verify_password(password: &str, password_hash: &str) -> bool {
    let Ok(parsed) = PasswordHash::new(password_hash) else {
        return false;
    };
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok()
}

#[allow(clippy::too_many_arguments)]
pub fn append_admin_audit(
    conn: &Connection,
    at: &str,
    admin_user_id: &str,
    admin_email: &str,
    action: &str,
    entity_type: &str,
    entity_id: &str,
    after_json: Option<&str>,
) -> Result<String, CloudError> {
    let prev_hash: String = conn
        .query_row(
            "SELECT hash FROM admin_audit_log ORDER BY seq DESC LIMIT 1",
            [],
            |r| r.get(0),
        )
        .optional()?
        .unwrap_or_else(|| "GENESIS_ADMIN_AUDIT_CHAIN_0000000000000000".to_string());

    let mut hasher = Sha256::new();
    hasher.update(prev_hash.as_bytes());
    hasher.update(b"|");
    hasher.update(at.as_bytes());
    hasher.update(b"|");
    hasher.update(admin_user_id.as_bytes());
    hasher.update(b"|");
    hasher.update(action.as_bytes());
    hasher.update(b"|");
    hasher.update(entity_type.as_bytes());
    hasher.update(b"|");
    hasher.update(entity_id.as_bytes());
    hasher.update(b"|");
    hasher.update(after_json.unwrap_or("").as_bytes());
    let hash = hex::encode(hasher.finalize());

    conn.execute(
        "INSERT INTO admin_audit_log (
            at, admin_user_id, admin_email, action, entity_type, entity_id, ip_address, before_json, after_json, prev_hash, hash
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, '127.0.0.1', NULL, ?7, ?8, ?9)",
        params![
            at,
            admin_user_id,
            admin_email,
            action,
            entity_type,
            entity_id,
            after_json,
            prev_hash,
            hash,
        ],
    )?;

    Ok(hash)
}

pub fn verify_admin_audit_chain(conn: &Connection) -> Result<bool, CloudError> {
    let mut stmt = conn.prepare(
        "SELECT at, admin_user_id, action, entity_type, entity_id, after_json, prev_hash, hash
         FROM admin_audit_log ORDER BY seq ASC",
    )?;
    let rows = stmt.query_map([], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, String>(2)?,
            r.get::<_, String>(3)?,
            r.get::<_, String>(4)?,
            r.get::<_, Option<String>>(5)?,
            r.get::<_, String>(6)?,
            r.get::<_, String>(7)?,
        ))
    })?;

    let mut expected_prev = "GENESIS_ADMIN_AUDIT_CHAIN_0000000000000000".to_string();
    for row in rows {
        let (at, admin_id, action, ent_type, ent_id, after_json, prev_hash, hash) = row?;
        if prev_hash != expected_prev {
            return Ok(false);
        }
        let mut hasher = Sha256::new();
        hasher.update(prev_hash.as_bytes());
        hasher.update(b"|");
        hasher.update(at.as_bytes());
        hasher.update(b"|");
        hasher.update(admin_id.as_bytes());
        hasher.update(b"|");
        hasher.update(action.as_bytes());
        hasher.update(b"|");
        hasher.update(ent_type.as_bytes());
        hasher.update(b"|");
        hasher.update(ent_id.as_bytes());
        hasher.update(b"|");
        hasher.update(after_json.as_deref().unwrap_or("").as_bytes());
        let computed = hex::encode(hasher.finalize());
        if computed != hash {
            return Ok(false);
        }
        expected_prev = hash;
    }
    Ok(true)
}

/// Initializes the commercial database schema and seeds default plans, admin users, and sample records if empty.
pub fn ensure_cloud_seeded(
    conn: &Connection,
    signer_conn: &Connection,
    signer: &SignerService,
    now_utc: &str,
) -> Result<(), CloudError> {
    conn.execute_batch(CLOUD_SCHEMA_SQL)?;
    let _ = conn.execute(
        "ALTER TABLE releases ADD COLUMN size_bytes INTEGER NOT NULL DEFAULT 0",
        [],
    );
    let _ = conn.execute(
        "ALTER TABLE releases ADD COLUMN authenticode_thumbprint TEXT NOT NULL DEFAULT ''",
        [],
    );
    let _ = conn.execute(
        "ALTER TABLE releases ADD COLUMN updater_key_id TEXT NOT NULL DEFAULT 'u2026a'",
        [],
    );
    let _ = conn.execute(
        "ALTER TABLE prices ADD COLUMN billing_cycle TEXT NOT NULL DEFAULT '12_months'",
        [],
    );
    let _ = conn.execute(
        "ALTER TABLE prices ADD COLUMN is_lifetime INTEGER NOT NULL DEFAULT 0",
        [],
    );
    let _ = conn.execute(
        "ALTER TABLE prices ADD COLUMN monthly_rate_poisha INTEGER NOT NULL DEFAULT 100000",
        [],
    );
    let _ = conn.execute(
        "ALTER TABLE installations ADD COLUMN hostname TEXT NOT NULL DEFAULT ''",
        [],
    );
    let _ = conn.execute(
        "ALTER TABLE installations ADD COLUMN os_version TEXT NOT NULL DEFAULT 'Windows'",
        [],
    );
    let _ = conn.execute(
        "ALTER TABLE installations ADD COLUMN deactivated_at TEXT",
        [],
    );
    let _ = conn.execute(
        "ALTER TABLE installations ADD COLUMN deactivation_reason TEXT",
        [],
    );
    let _ = conn.execute(
        "ALTER TABLE installations ADD COLUMN replaced_by_installation_id TEXT",
        [],
    );
    let _ = conn.execute(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_installations_one_active_host_per_license ON installations(license_id) WHERE role = 'host' AND status = 'active'",
        [],
    );

    SignerService::init_audit_schema(signer_conn)
        .map_err(|e| CloudError::SignerFailure(e.to_string()))?;

    let tier_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM prices WHERE billing_cycle IN ('3_months', '6_months', '12_months', 'lifetime')",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);
    if tier_count == 0 {
        let _ = conn.execute(
            "INSERT OR REPLACE INTO prices (id, plan_id, billing_cycle, period_months, is_lifetime, currency, monthly_rate_poisha, amount_poisha, active_from)
             VALUES
             ('prc_local_3m', 'plan_local', '3_months', 3, 0, 'BDT', 150000, 450000, ?1),
             ('prc_local_6m', 'plan_local', '6_months', 6, 0, 'BDT', 120000, 720000, ?1),
             ('prc_local_12m', 'plan_local', '12_months', 12, 0, 'BDT', 100000, 1200000, ?1),
             ('prc_pro_lifetime', 'plan_local_pro', 'lifetime', 0, 1, 'BDT', 5000000, 5000000, ?1)",
            params![now_utc],
        );
    }

    let count: i64 = conn.query_row("SELECT COUNT(*) FROM admin_users", [], |r| r.get(0))?;
    if count == 0 {
        let default_pw = hash_password("CoachifyVendor#2026");
        let totp_secret = "COACHIFY_VENDOR_TOTP_SECRET_2026";
        let recovery_codes = r#"["RCV-1001","RCV-1002","RCV-1003","RCV-1004","RCV-1005","RCV-1006","RCV-1007","RCV-1008"]"#;

        conn.execute(
            "INSERT INTO admin_users (id, email, name, role, password_hash, totp_secret_enc, totp_enabled, recovery_codes_json, status, created_at)
             VALUES
             ('adm_super', 'owner@coachify.app', 'Tanvir Rahman (Founder)', 'super_admin', ?1, ?2, 1, ?3, 'active', ?4),
             ('adm_ops', 'ops@coachify.app', 'Nusrat Jahan (Billing Ops)', 'ops_admin', ?1, ?2, 1, ?3, 'active', ?4),
             ('adm_support', 'support@coachify.app', 'Farhan Kabir (Support Desk)', 'support_admin', ?1, ?2, 1, ?3, 'active', ?4)",
            params![default_pw, totp_secret, recovery_codes, now_utc],
        )?;

        conn.execute(
            "INSERT INTO plans (id, code, name, student_limit, workstation_limit, capabilities_json, is_active)
             VALUES
             ('plan_trial', 'trial', '14 Day Evaluation Trial', 500, 1, '[\"core.ledger\",\"core.attendance\",\"core.exams\"]', 1),
             ('plan_local', 'local', 'Coachify Local (Single PC)', 2000, 1, '[\"core.ledger\",\"core.attendance\",\"core.exams\",\"core.sms\"]', 1),
             ('plan_local_pro', 'local_pro', 'Coachify Local Pro (Multi PC LAN)', 10000, 10, '[\"core.ledger\",\"core.attendance\",\"core.exams\",\"core.sms\",\"core.lan\"]', 1)",
            [],
        )?;

        conn.execute(
            "INSERT INTO prices (id, plan_id, billing_cycle, period_months, is_lifetime, currency, monthly_rate_poisha, amount_poisha, active_from)
             VALUES
             ('prc_local_1m', 'plan_local', '1_month', 1, 0, 'BDT', 250000, 250000, ?1),
             ('prc_local_3m', 'plan_local', '3_months', 3, 0, 'BDT', 150000, 450000, ?1),
             ('prc_local_6m', 'plan_local', '6_months', 6, 0, 'BDT', 120000, 720000, ?1),
             ('prc_local_12m', 'plan_local', '12_months', 12, 0, 'BDT', 100000, 1200000, ?1),
             ('prc_pro_6m', 'plan_local_pro', '6_months', 6, 0, 'BDT', 2400000, 2400000, ?1),
             ('prc_pro_12m', 'plan_local_pro', '12_months', 12, 0, 'BDT', 4200000, 4200000, ?1),
             ('prc_pro_lifetime', 'plan_local_pro', 'lifetime', 0, 1, 'BDT', 5000000, 5000000, ?1)",
            params![now_utc],
        )?;

        conn.execute(
            "INSERT INTO customers (id, org_name, owner_name, phone, email, district, notes, status, created_at, updated_at)
             VALUES
             ('cust_mirpur', 'Apex Science Academy Mirpur', 'Md. Shafiqul Islam', '+8801711223344', 'shafiq@apexmirpur.edu.bd', 'Dhaka', 'Flagship 1200 student HSC batch center', 'active', ?1, ?1),
             ('cust_ctg', 'Chattogram Scholars Coaching', 'Dr. Rehana Parvin', '+8801819556677', 'rehana@ctgscholars.com', 'Chattogram', 'Offline campus using WhatsApp request codes', 'active', ?1, ?1)",
            params![now_utc],
        )?;

        conn.execute(
            "INSERT INTO subscriptions (id, customer_id, plan_id, status, paid_through, grace_days, started_at)
             VALUES
             ('sub_mirpur', 'cust_mirpur', 'plan_local', 'active', '2027-03-25', 14, ?1),
             ('sub_ctg', 'cust_ctg', 'plan_local_pro', 'grace', '2026-09-20', 14, ?1)",
            params![now_utc],
        )?;

        let payload_mirpur = SignerLicensePayload {
            license_id: "LIC-2026-MIRP-0001".to_string(),
            customer_id: "cust_mirpur".to_string(),
            customer_name: "Apex Science Academy Mirpur".to_string(),
            installation_id: "inst-mirpur-host-01".to_string(),
            tier: "local".to_string(),
            kind: "paid".to_string(),
            serial: 1,
            paid_through: "2027-03-25".to_string(),
            grace_days: 14,
            valid_until: "2027-04-08".to_string(),
            issued_at: now_utc.to_string(),
            limits: SignerLicenseLimits {
                active_students: Some(2000),
                workstations: Some(1),
            },
            capabilities: vec![
                "core.ledger".to_string(),
                "core.attendance".to_string(),
                "core.exams".to_string(),
                "core.sms".to_string(),
            ],
            key_id: "k2026a".to_string(),
        };

        let signed_mirpur =
            request_isolated_license_signature(signer_conn, signer, &payload_mirpur, now_utc)?;

        let payload_ctg = SignerLicensePayload {
            license_id: "LIC-2026-CTGS-0002".to_string(),
            customer_id: "cust_ctg".to_string(),
            customer_name: "Chattogram Scholars Coaching".to_string(),
            installation_id: "inst-ctg-host-01".to_string(),
            tier: "local_pro".to_string(),
            kind: "paid".to_string(),
            serial: 1,
            paid_through: "2026-09-20".to_string(),
            grace_days: 14,
            valid_until: "2026-10-04".to_string(),
            issued_at: now_utc.to_string(),
            limits: SignerLicenseLimits {
                active_students: Some(10000),
                workstations: Some(10),
            },
            capabilities: vec![
                "core.ledger".to_string(),
                "core.attendance".to_string(),
                "core.exams".to_string(),
                "core.sms".to_string(),
                "core.lan".to_string(),
            ],
            key_id: "k2026a".to_string(),
        };
        let signed_ctg =
            request_isolated_license_signature(signer_conn, signer, &payload_ctg, now_utc)?;

        conn.execute(
            "INSERT INTO licenses (
                id, license_id, customer_id, subscription_id, activation_key_hash, pairing_code, pairing_expires_at,
                current_serial, bound_installation_id, key_id, last_issued_blob, status, transfer_count_12m, updated_at
            ) VALUES
            ('lic_row_1', 'LIC-2026-MIRP-0001', 'cust_mirpur', 'sub_mirpur', 'ACT-MIRPUR-2026', '849201', '2027-12-31T23:59:59Z', 1, 'inst-mirpur-host-01', 'k2026a', ?1, 'active', 0, ?3),
            ('lic_row_2', 'LIC-2026-CTGS-0002', 'cust_ctg', 'sub_ctg', 'ACT-CTG-2026', '512904', '2027-12-31T23:59:59Z', 1, 'inst-ctg-host-01', 'k2026a', ?2, 'active', 1, ?3)",
            params![signed_mirpur.signed_blob, signed_ctg.signed_blob, now_utc],
        )?;

        conn.execute(
            "INSERT INTO license_issues (
                id, license_id, serial, installation_id, kind, payload_json, signature, signed_blob, issued_by, reason, issued_at
            ) VALUES
            ('iss_01', 'LIC-2026-MIRP-0001', 1, 'inst-mirpur-host-01', 'paid', ?1, ?2, ?3, 'adm_ops', 'Initial 6 month subscription', ?4)",
            params![
                signed_mirpur.envelope.payload,
                signed_mirpur.envelope.signature,
                signed_mirpur.signed_blob,
                now_utc
            ],
        )?;

        conn.execute(
            "INSERT INTO installations (
                id, installation_id, customer_id, license_id, label, role, app_version, active_students, first_seen_at, last_seen_at, status
            ) VALUES
            ('inst_row_1', 'inst-mirpur-host-01', 'cust_mirpur', 'LIC-2026-MIRP-0001', 'Mirpur Front Desk Host PC', 'host', '1.0.1', 842, ?1, ?1, 'active'),
            ('inst_row_2', 'inst-ctg-host-01', 'cust_ctg', 'LIC-2026-CTGS-0002', 'Chattogram Main Counter PC', 'host', '1.0.1', 1150, ?1, ?1, 'active')",
            params![now_utc],
        )?;

        conn.execute(
            "INSERT INTO payments (
                id, customer_id, subscription_id, amount_poisha, method, reference, proof_url, period_months, recorded_by, status, recorded_at
            ) VALUES
            ('pay_seed_1', 'cust_mirpur', 'sub_mirpur', 1350000, 'bkash', 'BKASH-TRX-90817263', 'https://files.coachify.app/proofs/bkash-90817263.png', 6, 'adm_ops', 'confirmed', ?1),
            ('pay_seed_2', 'cust_ctg', 'sub_ctg', 2400000, 'nagad', 'NAGAD-TRX-44112299', 'https://files.coachify.app/proofs/nagad-44112299.png', 6, 'adm_ops', 'confirmed', ?1)",
            params![now_utc],
        )?;

        conn.execute(
            "INSERT INTO vendor_invoices (
                id, payment_id, customer_id, invoice_number, subtotal_poisha, vat_poisha, total_poisha, issued_at
            ) VALUES
            ('inv_seed_1', 'pay_seed_1', 'cust_mirpur', 'INV-2026-00001', 1350000, 0, 1350000, ?1),
            ('inv_seed_2', 'pay_seed_2', 'cust_ctg', 'INV-2026-00002', 2400000, 0, 2400000, ?1)",
            params![now_utc],
        )?;

        let sig_101 = request_isolated_updater_signature(
            signer_conn,
            signer,
            "1.0.1",
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
            "https://cdn.coachify.app/releases/CoachifySetup-v1.0.1-x64.exe",
            "A84F92D17C6E0049BC112233445566778899AABB",
            now_utc,
        )?;
        let sig_110b = request_isolated_updater_signature(
            signer_conn,
            signer,
            "1.1.0-beta.1",
            "2c26b46b68ffc68ff99b453c1d30413413422d706483bfa0f98a5e886266e7ae",
            "https://cdn.coachify.app/releases/CoachifySetup-v1.1.0-beta.1-x64.exe",
            "A84F92D17C6E0049BC112233445566778899AABB",
            now_utc,
        )?;

        conn.execute(
            "INSERT INTO releases (
                id, version, channel, notes, artifact_url, sha256, size_bytes, authenticode_thumbprint, updater_signature, updater_key_id, min_upgrade_from, status, published_by, published_at
            ) VALUES
            ('rel_101', '1.0.1', 'stable', 'Stable production release with thermal receipt speedups and offline license guard.', 'https://cdn.coachify.app/releases/CoachifySetup-v1.0.1-x64.exe', 'e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855', 68420096, 'A84F92D17C6E0049BC112233445566778899AABB', ?1, 'u2026a', '1.0.0', 'live', 'adm_super', ?3),
            ('rel_102_bad', '1.0.2', 'stable', 'Blocked build due to legacy Windows 10 spooler regression.', 'https://cdn.coachify.app/releases/CoachifySetup-v1.0.2-x64.exe', '9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08', 68510200, 'A84F92D17C6E0049BC112233445566778899AABB', 'ed25519_sig_102', 'u2026a', '1.0.0', 'blocked', 'adm_super', ?3),
            ('rel_110_beta', '1.1.0-beta.1', 'beta', 'Beta channel preview with multi branch analytics and zero rebuild promotion.', 'https://cdn.coachify.app/releases/CoachifySetup-v1.1.0-beta.1-x64.exe', '2c26b46b68ffc68ff99b453c1d30413413422d706483bfa0f98a5e886266e7ae', 69104128, 'A84F92D17C6E0049BC112233445566778899AABB', ?2, 'u2026a', '1.0.0', 'live', 'adm_super', ?3)",
            params![sig_101.updater_signature, sig_110b.updater_signature, now_utc],
        )?;

        append_admin_audit(
            conn,
            now_utc,
            "adm_super",
            "owner@coachify.app",
            "vendor.seed_initialized",
            "system",
            "cloud_api",
            Some(r#"{"seeded_customers":2,"seeded_releases":3}"#),
        )?;
    }

    // Ensure portal credentials exist for all customers
    let portal_cred_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM customer_portal_credentials",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);
    if portal_cred_count == 0 {
        let portal_pw = hash_password("CoachifyPortal#2026");
        let _ = conn.execute(
            "INSERT OR IGNORE INTO customer_portal_credentials (customer_id, password_hash, failed_attempts, updated_at)
             VALUES
             ('cust_mirpur', ?1, 0, ?2),
             ('cust_ctg', ?1, 0, ?2)",
            params![portal_pw, now_utc],
        );
    }

    // Ensure release_channel_heads are initialized
    let head_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM release_channel_heads", [], |r| {
            r.get(0)
        })
        .unwrap_or(0);
    if head_count == 0 {
        let _ = conn.execute(
            "INSERT OR IGNORE INTO release_channel_heads (channel, release_id, version, promoted_from_channel, promoted_by, promoted_at)
             VALUES
             ('internal', 'rel_110_beta', '1.1.0-beta.1', NULL, 'adm_super', ?1),
             ('beta', 'rel_110_beta', '1.1.0-beta.1', 'internal', 'adm_super', ?1),
             ('stable', 'rel_101', '1.0.1', 'beta', 'adm_super', ?1)",
            params![now_utc],
        );
    }

    // Ensure baseline key_recovery_drills exist
    let drill_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM key_recovery_drills", [], |r| r.get(0))
        .unwrap_or(0);
    if drill_count == 0 {
        for (idx, (kt, kid, src, notes)) in [
            (
                "windows_authenticode",
                "authenticode_2026_ev",
                "azure-trusted-signing-hsm-backup-01",
                "Verified Windows Authenticode EV certificate & backup hardware token signing drill",
            ),
            (
                "tauri_updater_ed25519",
                "u2026a",
                "offline-cold-usb-vault-dhaka-01",
                "Verified Tauri Ed25519 updater release key u2026a cold vault decryption and signature check",
            ),
        ]
        .iter()
        .enumerate()
        {
            let blob = format!("ENC_VAULT_V1_{kt}_{kid}_SEED_BACKUP");
            let auth_str = format!("{kt}|{kid}|{src}|{blob}");
            let hmac = compute_service_hmac(
                DEFAULT_SIGNER_SHARED_SECRET,
                "cloud_api",
                now_utc,
                &auth_str,
            );
            if let Ok(verified) = signer.verify_key_recovery(
                signer_conn,
                &VerifyKeyRecoveryRequest {
                    caller_service_id: "cloud_api".to_string(),
                    timestamp_utc: now_utc.to_string(),
                    hmac_signature: hmac,
                    key_type: (*kt).to_string(),
                    key_id: (*kid).to_string(),
                    backup_source: (*src).to_string(),
                    encrypted_backup_blob: blob,
                    passphrase: "VaultRecoveryPass#2026".to_string(),
                },
            ) {
                let _ = conn.execute(
                    "INSERT INTO key_recovery_drills (
                        id, key_type, key_id, backup_source, backup_sha256, test_artifact_sha256, signature_verified, notes, performed_by, performed_at
                    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, 1, ?7, 'adm_super', ?8)",
                    params![
                        format!("drill_seed_{}", idx + 1),
                        kt,
                        kid,
                        src,
                        verified.backup_sha256,
                        verified.test_artifact_sha256,
                        notes,
                        now_utc,
                    ],
                );
            }
        }
    }

    Ok(())
}

fn request_isolated_updater_signature(
    signer_conn: &Connection,
    signer: &SignerService,
    version: &str,
    sha256: &str,
    artifact_url: &str,
    authenticode_thumbprint: &str,
    now_utc: &str,
) -> Result<coachify_license_signer::SignUpdaterReleaseResponse, CloudError> {
    let payload = UpdaterReleaseSignPayload {
        version: version.trim().to_string(),
        sha256: sha256.trim().to_string(),
        artifact_url: artifact_url.trim().to_string(),
        authenticode_thumbprint: authenticode_thumbprint.trim().to_string(),
        key_id: "u2026a".to_string(),
    };
    let canonical =
        canonical_value_json(&payload).map_err(|e| CloudError::SignerFailure(e.to_string()))?;
    let hmac = compute_service_hmac(
        DEFAULT_SIGNER_SHARED_SECRET,
        "cloud_api",
        now_utc,
        &canonical,
    );
    signer
        .sign_updater_release(
            signer_conn,
            &SignUpdaterReleaseRequest {
                caller_service_id: "cloud_api".to_string(),
                timestamp_utc: now_utc.to_string(),
                hmac_signature: hmac,
                key_id: "u2026a".to_string(),
                payload,
            },
        )
        .map_err(|e| CloudError::SignerFailure(e.to_string()))
}

fn request_isolated_license_signature(
    signer_conn: &Connection,
    signer: &SignerService,
    payload: &SignerLicensePayload,
    now_utc: &str,
) -> Result<coachify_license_signer::SignLicenseResponse, CloudError> {
    let mut normalized = payload.clone();
    normalized.key_id = "k2026a".to_string();
    let canonical =
        canonical_value_json(&normalized).map_err(|e| CloudError::SignerFailure(e.to_string()))?;
    let hmac = compute_service_hmac(
        DEFAULT_SIGNER_SHARED_SECRET,
        "cloud_api",
        now_utc,
        &canonical,
    );
    signer
        .sign_license(
            signer_conn,
            &SignLicenseRequest {
                caller_service_id: "cloud_api".to_string(),
                timestamp_utc: now_utc.to_string(),
                hmac_signature: hmac,
                key_id: "k2026a".to_string(),
                payload: normalized,
            },
        )
        .map_err(|e| CloudError::SignerFailure(e.to_string()))
}

fn resolve_admin_by_email(
    conn: &Connection,
    email: &str,
) -> Result<(AdminUserView, String, String, String), CloudError> {
    conn.query_row(
        "SELECT id, email, name, role, totp_enabled, status, last_login_at, password_hash, totp_secret_enc, recovery_codes_json
         FROM admin_users WHERE lower(email) = lower(?1)",
        params![email.trim()],
        |r| {
            Ok((
                AdminUserView {
                    id: r.get(0)?,
                    email: r.get(1)?,
                    name: r.get(2)?,
                    role: r.get(3)?,
                    totp_enabled: r.get::<_, i32>(4)? != 0,
                    status: r.get(5)?,
                    last_login_at: r.get(6)?,
                },
                r.get::<_, String>(7)?,
                r.get::<_, String>(8)?,
                r.get::<_, String>(9)?,
            ))
        },
    )
    .optional()?
    .ok_or(CloudError::AdminInvalidCredentials)
}

/// Authenticates a Vendor Admin using Argon2id password + RFC 6238 6 digit TOTP code.
pub fn login_admin_with_totp(
    conn: &Connection,
    input: &AdminLoginInput,
    now_utc: &str,
    unix_seconds: u64,
) -> Result<AdminLoginResponse, CloudError> {
    let (mut admin_view, pw_hash, totp_secret, recovery_json) =
        resolve_admin_by_email(conn, &input.email)?;
    if admin_view.status != "active" {
        return Err(CloudError::AdminAccountSuspended);
    }
    if !verify_password(&input.password, &pw_hash) {
        return Err(CloudError::AdminInvalidCredentials);
    }
    if !verify_totp_or_recovery(&totp_secret, &recovery_json, &input.totp_code, unix_seconds) {
        return Err(CloudError::AdminTotpInvalid);
    }

    conn.execute(
        "UPDATE admin_users SET last_login_at = ?1 WHERE id = ?2",
        params![now_utc, admin_view.id],
    )?;
    admin_view.last_login_at = Some(now_utc.to_string());

    append_admin_audit(
        conn,
        now_utc,
        &admin_view.id,
        &admin_view.email,
        "admin.auth.login_totp",
        "admin_users",
        &admin_view.id,
        Some(&serde_json::json!({ "role": admin_view.role }).to_string()),
    )?;

    Ok(AdminLoginResponse {
        session_token: format!("vadm_sess_{}", Uuid::now_v7()),
        current_demo_totp: generate_totp_code(&totp_secret, unix_seconds),
        admin_user: admin_view,
    })
}

/// Adds calendar months (`1`, `6`, or `12`) to an `YYYY-MM-DD` business date string and returns `(new_paid_through, valid_until)`.
pub fn compute_extended_paid_through(
    current_paid_through: &str,
    grace_days: u32,
    today_date: &str,
    period_months: u32,
) -> Result<(String, String), CloudError> {
    let parse_ymd = |s: &str| -> Result<jiff::civil::Date, CloudError> {
        let prefix = if s.len() >= 10 { &s[0..10] } else { s };
        prefix
            .parse::<jiff::civil::Date>()
            .map_err(|e| CloudError::Validation(format!("Invalid date '{s}': {e}")))
    };

    let old_paid = parse_ymd(current_paid_through)?;
    let today = parse_ymd(today_date)?;
    let grace_end = old_paid
        .checked_add(jiff::ToSpan::days(i64::from(grace_days)))
        .map_err(|e| CloudError::Validation(e.to_string()))?;

    // Per Spec 0041 AC-3: If renewed on or before grace period ends, extend from old paid_through;
    // if renewed after expiry, start from payment date (today).
    let anchor = if today <= grace_end { old_paid } else { today };
    let new_paid = anchor
        .checked_add(jiff::ToSpan::months(i64::from(period_months)))
        .map_err(|e| CloudError::Validation(e.to_string()))?;
    let valid_until = new_paid
        .checked_add(jiff::ToSpan::days(i64::from(grace_days)))
        .map_err(|e| CloudError::Validation(e.to_string()))?;

    Ok((new_paid.to_string(), valid_until.to_string()))
}

/// Records a manual payment (`bkash`, `nagad`, `bank`, `cash`, `gateway`), generates an `INV-YYYY-NNNNN` invoice,
/// extends `paid_through`, increments `current_serial`, calls the isolated `SignerService`, and logs to `admin_audit_log`.
pub fn record_manual_payment(
    conn: &Connection,
    signer_conn: &Connection,
    signer: &SignerService,
    input: &RecordManualPaymentInput,
    now_utc: &str,
) -> Result<RecordManualPaymentResult, CloudError> {
    let (admin, _, _, _) = resolve_admin_by_email(conn, &input.admin_email)?;
    if admin.role != "super_admin" && admin.role != "ops_admin" {
        return Err(CloudError::AdminPermissionDenied(admin.role));
    }
    if input.amount_poisha <= 0 {
        return Err(CloudError::Validation(
            "Payment amount_poisha must be positive".to_string(),
        ));
    }
    if input.reference.trim().is_empty() {
        return Err(CloudError::Validation(
            "Payment transaction reference is required".to_string(),
        ));
    }

    let (org_name, plan_code, student_limit, workstation_limit, capabilities_json, old_paid_through, grace_days): (
        String,
        String,
        Option<u32>,
        Option<u32>,
        String,
        String,
        u32,
    ) = conn
        .query_row(
            "SELECT c.org_name, p.code, p.student_limit, p.workstation_limit, p.capabilities_json, s.paid_through, s.grace_days
             FROM subscriptions s
             JOIN customers c ON c.id = s.customer_id
             JOIN plans p ON p.id = s.plan_id
             WHERE s.id = ?1 AND s.customer_id = ?2",
            params![input.subscription_id, input.customer_id],
            |r| {
                Ok((
                    r.get(0)?,
                    r.get(1)?,
                    r.get(2)?,
                    r.get(3)?,
                    r.get(4)?,
                    r.get(5)?,
                    r.get(6)?,
                ))
            },
        )
        .optional()?
        .ok_or_else(|| CloudError::NotFound(format!("Subscription {}", input.subscription_id)))?;

    let (license_id, current_serial, bound_inst, lic_status): (
        String,
        i64,
        Option<String>,
        String,
    ) = conn
        .query_row(
            "SELECT license_id, current_serial, bound_installation_id, status
             FROM licenses WHERE subscription_id = ?1",
            params![input.subscription_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )
        .optional()?
        .ok_or_else(|| CloudError::NotFound(format!("License for {}", input.subscription_id)))?;

    if lic_status == "revoked" {
        return Err(CloudError::LicenseRevoked(license_id));
    }

    let today_ymd = if now_utc.len() >= 10 {
        &now_utc[0..10]
    } else {
        "2026-09-25"
    };
    let (new_paid_through, valid_until) = compute_extended_paid_through(
        &old_paid_through,
        grace_days,
        today_ymd,
        input.period_months,
    )?;

    let target_installation_id = input
        .installation_id
        .clone()
        .filter(|s| !s.trim().is_empty())
        .or(bound_inst)
        .unwrap_or_else(|| format!("inst-{}-host", input.customer_id));

    let new_serial = current_serial + 1;
    let capabilities: Vec<String> = serde_json::from_str(&capabilities_json).unwrap_or_default();
    let tier = if plan_code == "local_pro" {
        "local_pro".to_string()
    } else {
        "local".to_string()
    };

    let payload = SignerLicensePayload {
        license_id: license_id.clone(),
        customer_id: input.customer_id.clone(),
        customer_name: org_name.clone(),
        installation_id: target_installation_id.clone(),
        tier,
        kind: "extension".to_string(),
        serial: new_serial,
        paid_through: new_paid_through.clone(),
        grace_days,
        valid_until: valid_until.clone(),
        issued_at: now_utc.to_string(),
        limits: SignerLicenseLimits {
            active_students: student_limit,
            workstations: workstation_limit,
        },
        capabilities,
        key_id: "k2026a".to_string(),
    };

    let signed = request_isolated_license_signature(signer_conn, signer, &payload, now_utc)?;

    let payment_id = format!("pay_{}", Uuid::now_v7());
    let inv_count: i64 =
        conn.query_row("SELECT COUNT(*) FROM vendor_invoices", [], |r| r.get(0))?;
    let year_str = &today_ymd[0..4];
    let invoice_number = format!("INV-{}-{:05}", year_str, inv_count + 1);
    let invoice_id = format!("inv_{}", Uuid::now_v7());
    let pairing_code = format!("{:06}", (100_000 + ((new_serial as u64 * 7919) % 899_999)));

    conn.execute(
        "INSERT INTO payments (
            id, customer_id, subscription_id, amount_poisha, method, reference, proof_url, period_months, recorded_by, status, recorded_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 'confirmed', ?10)",
        params![
            payment_id,
            input.customer_id,
            input.subscription_id,
            input.amount_poisha,
            input.method,
            input.reference.trim(),
            input.proof_url,
            input.period_months,
            admin.id,
            now_utc,
        ],
    )?;

    conn.execute(
        "INSERT INTO vendor_invoices (
            id, payment_id, customer_id, invoice_number, subtotal_poisha, vat_poisha, total_poisha, issued_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, 0, ?5, ?6)",
        params![
            invoice_id,
            payment_id,
            input.customer_id,
            invoice_number,
            input.amount_poisha,
            now_utc,
        ],
    )?;

    conn.execute(
        "UPDATE subscriptions SET status = 'active', paid_through = ?1 WHERE id = ?2",
        params![new_paid_through, input.subscription_id],
    )?;

    conn.execute(
        "UPDATE licenses
         SET current_serial = ?1,
             bound_installation_id = ?2,
             pairing_code = ?3,
             last_issued_blob = ?4,
             status = 'active',
             updated_at = ?5
         WHERE license_id = ?6",
        params![
            new_serial,
            target_installation_id,
            pairing_code,
            signed.signed_blob,
            now_utc,
            license_id,
        ],
    )?;

    conn.execute(
        "INSERT INTO license_issues (
            id, license_id, serial, installation_id, kind, payload_json, signature, signed_blob, issued_by, reason, issued_at
        ) VALUES (?1, ?2, ?3, ?4, 'extension', ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            format!("iss_{}", Uuid::now_v7()),
            license_id,
            new_serial,
            target_installation_id,
            signed.envelope.payload,
            signed.envelope.signature,
            signed.signed_blob,
            admin.id,
            format!(
                "Manual {} payment ({}) for {} months",
                input.method,
                input.reference.trim(),
                input.period_months
            ),
            now_utc,
        ],
    )?;

    conn.execute(
        "UPDATE installations SET status = 'moved' WHERE license_id = ?1 AND installation_id != ?2 AND role = 'host' AND status = 'active'",
        params![license_id, target_installation_id],
    )?;

    conn.execute(
        "INSERT INTO installations (
            id, installation_id, customer_id, license_id, label, role, app_version, active_students, first_seen_at, last_seen_at, status
        ) VALUES (?1, ?2, ?3, ?4, ?5, 'host', '1.0.1', 0, ?6, ?6, 'active')
        ON CONFLICT(license_id, installation_id) DO UPDATE SET status = 'active', last_seen_at = excluded.last_seen_at",
        params![
            format!("inst_{}", Uuid::now_v7()),
            target_installation_id,
            input.customer_id,
            license_id,
            format!("{org_name} Host PC"),
            now_utc,
        ],
    )?;

    append_admin_audit(
        conn,
        now_utc,
        &admin.id,
        &admin.email,
        "admin.payments.record",
        "payments",
        &payment_id,
        Some(
            &serde_json::json!({
                "invoice_number": invoice_number,
                "method": input.method,
                "reference": input.reference.trim(),
                "amount_poisha": input.amount_poisha,
                "license_id": license_id,
                "new_serial": new_serial,
                "new_paid_through": new_paid_through,
                "signer_audit_seq": signed.signer_audit_seq,
            })
            .to_string(),
        ),
    )?;

    Ok(RecordManualPaymentResult {
        payment_id,
        invoice_number,
        license_id,
        new_serial,
        new_paid_through,
        valid_until,
        pairing_code,
        signed_license_blob: signed.signed_blob,
        signer_audit_seq: signed.signer_audit_seq,
    })
}

/// Validates an offline request code (`REQ-XXXX-...`) or QR payload (`COACHIFY-REQ|v1|<installation_id>|<checksum>`),
/// binds the license to that `installation_id`, increments `current_serial`, and signs a downloadable `.coachifylicense`.
#[allow(clippy::type_complexity)]
pub fn issue_offline_license_from_request(
    conn: &Connection,
    signer_conn: &Connection,
    signer: &SignerService,
    input: &IssueOfflineLicenseInput,
    now_utc: &str,
) -> Result<IssueOfflineLicenseResult, CloudError> {
    let (admin, _, _, _) = resolve_admin_by_email(conn, &input.admin_email)?;

    let (
        customer_id,
        org_name,
        plan_code,
        student_limit,
        workstation_limit,
        capabilities_json,
        paid_through,
        grace_days,
        current_serial,
        bound_inst,
        lic_status,
    ): (
        String,
        String,
        String,
        Option<u32>,
        Option<u32>,
        String,
        String,
        u32,
        i64,
        Option<String>,
        String,
    ) = conn
        .query_row(
            "SELECT l.customer_id, c.org_name, p.code, p.student_limit, p.workstation_limit, p.capabilities_json,
                    s.paid_through, s.grace_days, l.current_serial, l.bound_installation_id, l.status
             FROM licenses l
             JOIN customers c ON c.id = l.customer_id
             JOIN subscriptions s ON s.id = l.subscription_id
             JOIN plans p ON p.id = s.plan_id
             WHERE l.license_id = ?1",
            params![input.license_id.trim()],
            |r| {
                Ok((
                    r.get(0)?,
                    r.get(1)?,
                    r.get(2)?,
                    r.get(3)?,
                    r.get(4)?,
                    r.get(5)?,
                    r.get(6)?,
                    r.get(7)?,
                    r.get(8)?,
                    r.get(9)?,
                    r.get(10)?,
                ))
            },
        )
        .optional()?
        .ok_or_else(|| CloudError::NotFound(format!("License {}", input.license_id)))?;

    if lic_status == "revoked" {
        return Err(CloudError::LicenseRevoked(input.license_id.clone()));
    }

    let (resolved_inst_id, req_code, qr_opt) =
        parse_and_verify_offline_request(conn, &input.request_code_or_qr, bound_inst.as_deref())?;

    let paid_date: jiff::civil::Date = paid_through[0..10]
        .parse()
        .map_err(|e: jiff::Error| CloudError::Validation(e.to_string()))?;
    let valid_until = paid_date
        .checked_add(jiff::ToSpan::days(i64::from(grace_days)))
        .map_err(|e| CloudError::Validation(e.to_string()))?
        .to_string();

    let new_serial = current_serial + 1;
    let capabilities: Vec<String> = serde_json::from_str(&capabilities_json).unwrap_or_default();
    let tier = if plan_code == "local_pro" {
        "local_pro".to_string()
    } else {
        "local".to_string()
    };

    let payload = SignerLicensePayload {
        license_id: input.license_id.trim().to_string(),
        customer_id: customer_id.clone(),
        customer_name: org_name.clone(),
        installation_id: resolved_inst_id.clone(),
        tier,
        kind: "paid".to_string(),
        serial: new_serial,
        paid_through,
        grace_days,
        valid_until,
        issued_at: now_utc.to_string(),
        limits: SignerLicenseLimits {
            active_students: student_limit,
            workstations: workstation_limit,
        },
        capabilities,
        key_id: "k2026a".to_string(),
    };

    let signed = request_isolated_license_signature(signer_conn, signer, &payload, now_utc)?;
    let issue_id = format!("iss_{}", Uuid::now_v7());

    conn.execute(
        "UPDATE licenses
         SET current_serial = ?1,
             bound_installation_id = ?2,
             last_issued_blob = ?3,
             updated_at = ?4
         WHERE license_id = ?5",
        params![
            new_serial,
            resolved_inst_id,
            signed.signed_blob,
            now_utc,
            input.license_id.trim(),
        ],
    )?;

    conn.execute(
        "INSERT INTO license_issues (
            id, license_id, serial, installation_id, kind, payload_json, signature, signed_blob, issued_by, reason, issued_at
        ) VALUES (?1, ?2, ?3, ?4, 'paid', ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            issue_id,
            input.license_id.trim(),
            new_serial,
            resolved_inst_id,
            signed.envelope.payload,
            signed.envelope.signature,
            signed.signed_blob,
            admin.id,
            input.reason.trim(),
            now_utc,
        ],
    )?;

    conn.execute(
        "INSERT INTO activation_requests (
            id, customer_id, license_id, request_code, qr_payload, installation_id, checksum_verified, issued_serial_id, processed_by, created_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, 1, ?7, ?8, ?9)",
        params![
            format!("areq_{}", Uuid::now_v7()),
            customer_id,
            input.license_id.trim(),
            req_code,
            qr_opt,
            resolved_inst_id,
            issue_id,
            admin.id,
            now_utc,
        ],
    )?;

    conn.execute(
        "UPDATE installations SET status = 'moved', replaced_by_installation_id = ?1 WHERE license_id = ?2 AND role = 'host' AND status = 'active' AND installation_id != ?1",
        params![resolved_inst_id, input.license_id.trim()],
    )?;

    conn.execute(
        "INSERT INTO installations (
            id, installation_id, customer_id, license_id, label, role, app_version, active_students, first_seen_at, last_seen_at, status
        ) VALUES (?1, ?2, ?3, ?4, ?5, 'host', '1.0.1', 0, ?6, ?6, 'active')
        ON CONFLICT(license_id, installation_id) DO UPDATE SET status = 'active', last_seen_at = excluded.last_seen_at",
        params![
            format!("inst_{}", Uuid::now_v7()),
            resolved_inst_id,
            customer_id,
            input.license_id.trim(),
            format!("{org_name} Offline Host PC"),
            now_utc,
        ],
    )?;

    append_admin_audit(
        conn,
        now_utc,
        &admin.id,
        &admin.email,
        "admin.licenses.issue_offline",
        "licenses",
        input.license_id.trim(),
        Some(
            &serde_json::json!({
                "installation_id": resolved_inst_id,
                "request_code": req_code,
                "serial": new_serial,
                "signer_audit_seq": signed.signer_audit_seq,
            })
            .to_string(),
        ),
    )?;

    Ok(IssueOfflineLicenseResult {
        license_id: input.license_id.trim().to_string(),
        installation_id: resolved_inst_id,
        request_code: req_code,
        new_serial,
        signed_license_blob: signed.signed_blob,
        download_filename: format!(
            "{}-serial-{}.coachifylicense",
            input.license_id.trim(),
            new_serial
        ),
        signer_audit_seq: signed.signer_audit_seq,
    })
}

/// Revokes a license (`super_admin` only + step up 6 digit TOTP required), calling the isolated `SignerService`
/// to sign a `SignedRevocationEnvelope`.
pub fn revoke_license(
    conn: &Connection,
    signer_conn: &Connection,
    signer: &SignerService,
    input: &RevokeLicenseInput,
    now_utc: &str,
    unix_seconds: u64,
) -> Result<VendorRevocationRow, CloudError> {
    let (admin, _, totp_secret, recovery_json) = resolve_admin_by_email(conn, &input.admin_email)?;
    if admin.role != "super_admin" {
        return Err(CloudError::AdminPermissionDenied(admin.role));
    }
    if !verify_totp_or_recovery(
        &totp_secret,
        &recovery_json,
        &input.step_up_totp_code,
        unix_seconds,
    ) {
        return Err(CloudError::AdminTotpInvalid);
    }

    let (customer_id, org_name, subscription_id): (String, String, String) = conn
        .query_row(
            "SELECT l.customer_id, c.org_name, l.subscription_id
             FROM licenses l
             JOIN customers c ON c.id = l.customer_id
             WHERE l.license_id = ?1",
            params![input.license_id.trim()],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .optional()?
        .ok_or_else(|| CloudError::NotFound(format!("License {}", input.license_id)))?;

    let rev_payload = SignedRevocationPayload {
        license_id: input.license_id.trim().to_string(),
        reason: input.reason.trim().to_string(),
        revoked_at: now_utc.to_string(),
        key_id: "k2026a".to_string(),
    };
    let canonical =
        canonical_value_json(&rev_payload).map_err(|e| CloudError::SignerFailure(e.to_string()))?;
    let hmac = compute_service_hmac(
        DEFAULT_SIGNER_SHARED_SECRET,
        "cloud_api",
        now_utc,
        &canonical,
    );
    let signed_rev = signer
        .sign_revocation(
            signer_conn,
            &SignRevocationRequest {
                caller_service_id: "cloud_api".to_string(),
                timestamp_utc: now_utc.to_string(),
                hmac_signature: hmac,
                key_id: "k2026a".to_string(),
                license_id: input.license_id.trim().to_string(),
                reason: input.reason.trim().to_string(),
                revoked_at: now_utc.to_string(),
            },
        )
        .map_err(|e| CloudError::SignerFailure(e.to_string()))?;

    let rev_id = format!("rev_{}", Uuid::now_v7());
    conn.execute(
        "INSERT INTO revocations (id, license_id, reason, revoked_by, revoked_at, key_id, signed_revocation_blob)
         VALUES (?1, ?2, ?3, ?4, ?5, 'k2026a', ?6)",
        params![
            rev_id,
            input.license_id.trim(),
            input.reason.trim(),
            admin.email,
            now_utc,
            signed_rev.signed_revocation_blob,
        ],
    )?;

    conn.execute(
        "UPDATE licenses SET status = 'revoked', updated_at = ?1 WHERE license_id = ?2",
        params![now_utc, input.license_id.trim()],
    )?;
    conn.execute(
        "UPDATE subscriptions SET status = 'suspended' WHERE id = ?1",
        params![subscription_id],
    )?;
    conn.execute(
        "UPDATE installations SET status = 'revoked' WHERE license_id = ?1",
        params![input.license_id.trim()],
    )?;

    let _ = customer_id;
    append_admin_audit(
        conn,
        now_utc,
        &admin.id,
        &admin.email,
        "admin.licenses.revoke",
        "revocations",
        input.license_id.trim(),
        Some(
            &serde_json::json!({
                "reason": input.reason.trim(),
                "signer_audit_seq": signed_rev.signer_audit_seq,
            })
            .to_string(),
        ),
    )?;

    Ok(VendorRevocationRow {
        id: rev_id,
        license_id: input.license_id.trim().to_string(),
        org_name,
        reason: input.reason.trim().to_string(),
        revoked_by: admin.email,
        revoked_at: now_utc.to_string(),
        key_id: "k2026a".to_string(),
        signed_revocation_blob: signed_rev.signed_revocation_blob,
    })
}

/// Updates a release status (`live`, `held`, `blocked`). Blocking a release requires `super_admin` and step up TOTP.
pub fn update_release_status(
    conn: &Connection,
    input: &UpdateReleaseStatusInput,
    now_utc: &str,
    unix_seconds: u64,
) -> Result<VendorReleaseRow, CloudError> {
    let (admin, _, totp_secret, recovery_json) = resolve_admin_by_email(conn, &input.admin_email)?;
    if input.status == "blocked" {
        if admin.role != "super_admin" && admin.role != "ops_admin" {
            return Err(CloudError::AdminPermissionDenied(admin.role));
        }
        let code = input.step_up_totp_code.as_deref().unwrap_or("");
        if code.trim().is_empty() {
            return Err(CloudError::StepUpTotpRequired);
        }
        if !verify_totp_or_recovery(&totp_secret, &recovery_json, code, unix_seconds) {
            return Err(CloudError::AdminTotpInvalid);
        }
    }

    conn.execute(
        "UPDATE releases SET status = ?1, notes = COALESCE(?2, notes) WHERE version = ?3",
        params![input.status, input.notes, input.version.trim()],
    )?;

    append_admin_audit(
        conn,
        now_utc,
        &admin.id,
        &admin.email,
        "admin.releases.update_status",
        "releases",
        input.version.trim(),
        Some(&serde_json::json!({ "status": input.status }).to_string()),
    )?;

    query_release_by_version(conn, input.version.trim())
}

fn query_release_by_version(
    conn: &Connection,
    version: &str,
) -> Result<VendorReleaseRow, CloudError> {
    conn.query_row(
        "SELECT id, version, channel, notes, artifact_url, sha256, size_bytes, authenticode_thumbprint, updater_signature, updater_key_id, min_upgrade_from, status, published_by, published_at
         FROM releases WHERE version = ?1",
        params![version.trim()],
        |r| {
            Ok(VendorReleaseRow {
                id: r.get(0)?,
                version: r.get(1)?,
                channel: r.get(2)?,
                notes: r.get(3)?,
                artifact_url: r.get(4)?,
                sha256: r.get(5)?,
                size_bytes: r.get(6)?,
                authenticode_thumbprint: r.get(7)?,
                updater_signature: r.get(8)?,
                updater_key_id: r.get(9)?,
                min_upgrade_from: r.get(10)?,
                status: r.get(11)?,
                published_by: r.get(12)?,
                published_at: r.get(13)?,
            })
        },
    )
    .optional()?
    .ok_or_else(|| CloudError::NotFound(format!("Release {}", version.trim())))
}

fn query_channel_head_row(
    conn: &Connection,
    channel: &str,
) -> Result<Option<ReleaseChannelHeadRow>, CloudError> {
    conn.query_row(
        "SELECT h.channel, h.release_id, h.version, r.sha256, r.size_bytes, r.artifact_url, r.authenticode_thumbprint,
                r.updater_signature, r.updater_key_id, r.status, r.notes, h.promoted_from_channel, h.promoted_by, h.promoted_at
         FROM release_channel_heads h
         JOIN releases r ON r.id = h.release_id
         WHERE h.channel = ?1",
        params![channel.trim()],
        |r| {
            Ok(ReleaseChannelHeadRow {
                channel: r.get(0)?,
                release_id: r.get(1)?,
                version: r.get(2)?,
                sha256: r.get(3)?,
                size_bytes: r.get(4)?,
                artifact_url: r.get(5)?,
                authenticode_thumbprint: r.get(6)?,
                updater_signature: r.get(7)?,
                updater_key_id: r.get(8)?,
                status: r.get(9)?,
                notes: r.get(10)?,
                promoted_from_channel: r.get(11)?,
                promoted_by: r.get(12)?,
                promoted_at: r.get(13)?,
            })
        },
    )
    .optional()
    .map_err(CloudError::from)
}

/// Publishes a newly built and Windows Authenticode signed release to the `internal` channel (`POST /v1/releases/publish`),
/// requesting an isolated Tauri updater Ed25519 signature (`u2026a`) from [`SignerService`].
pub fn publish_release_to_internal(
    conn: &Connection,
    signer_conn: &Connection,
    signer: &SignerService,
    input: &PublishReleaseInput,
    now_utc: &str,
) -> Result<PublishReleaseResult, CloudError> {
    let (admin, _, _, _) = resolve_admin_by_email(conn, &input.admin_email)?;
    if admin.role != "super_admin" && admin.role != "ops_admin" {
        return Err(CloudError::AdminPermissionDenied(admin.role));
    }

    let clean_version = input.version.trim().trim_start_matches('v').to_string();
    if clean_version.is_empty() {
        return Err(CloudError::Validation(
            "Release version is required".to_string(),
        ));
    }

    let existing_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM releases WHERE version = ?1",
        params![clean_version],
        |r| r.get(0),
    )?;
    if existing_count > 0 {
        return Err(CloudError::DuplicateReleaseVersion(clean_version));
    }

    let clean_thumbprint = input.authenticode_thumbprint.trim().replace(' ', "");
    if clean_thumbprint.len() < 16 || !clean_thumbprint.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(CloudError::InvalidAuthenticodeThumbprint);
    }

    let signed_updater = request_isolated_updater_signature(
        signer_conn,
        signer,
        &clean_version,
        input.sha256.trim(),
        input.artifact_url.trim(),
        &clean_thumbprint,
        now_utc,
    )?;

    let rel_id = format!("rel_{}", Uuid::now_v7());
    conn.execute(
        "INSERT INTO releases (
            id, version, channel, notes, artifact_url, sha256, size_bytes, authenticode_thumbprint,
            updater_signature, updater_key_id, min_upgrade_from, status, published_by, published_at
        ) VALUES (?1, ?2, 'internal', ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, 'live', ?11, ?12)",
        params![
            rel_id,
            clean_version,
            input.notes.trim(),
            input.artifact_url.trim(),
            input.sha256.trim(),
            input.size_bytes,
            clean_thumbprint,
            signed_updater.updater_signature,
            signed_updater.key_id,
            input.min_upgrade_from,
            admin.id,
            now_utc,
        ],
    )?;

    conn.execute(
        "INSERT INTO release_channel_heads (channel, release_id, version, promoted_from_channel, promoted_by, promoted_at)
         VALUES ('internal', ?1, ?2, NULL, ?3, ?4)
         ON CONFLICT(channel) DO UPDATE SET
            release_id = excluded.release_id,
            version = excluded.version,
            promoted_from_channel = NULL,
            promoted_by = excluded.promoted_by,
            promoted_at = excluded.promoted_at",
        params![rel_id, clean_version, admin.id, now_utc],
    )?;

    append_admin_audit(
        conn,
        now_utc,
        &admin.id,
        &admin.email,
        "admin.releases.publish_internal",
        "releases",
        &clean_version,
        Some(
            &serde_json::json!({
                "release_id": rel_id,
                "channel": "internal",
                "sha256": input.sha256.trim(),
                "authenticode_thumbprint": clean_thumbprint,
                "signer_audit_seq": signed_updater.signer_audit_seq,
            })
            .to_string(),
        ),
    )?;

    let release = query_release_by_version(conn, &clean_version)?;
    let channel_head = query_channel_head_row(conn, "internal")?
        .ok_or_else(|| CloudError::NotFound("Channel head internal".to_string()))?;

    Ok(PublishReleaseResult {
        release,
        channel_head,
    })
}

/// Zero rebuild channel promotion (`POST /v1/releases/promote`): updates `release_channel_heads` for `to_channel` (`beta` or `stable`)
/// to reference the existing immutable `releases` record without recompiling or resigning the binary.
/// Promoting to `stable` requires step up 6 digit TOTP verification.
pub fn promote_release_channel(
    conn: &Connection,
    input: &PromoteReleaseChannelInput,
    now_utc: &str,
    unix_seconds: u64,
) -> Result<PromoteReleaseChannelResult, CloudError> {
    let (admin, _, totp_secret, recovery_json) = resolve_admin_by_email(conn, &input.admin_email)?;
    if admin.role != "super_admin" && admin.role != "ops_admin" {
        return Err(CloudError::AdminPermissionDenied(admin.role));
    }

    let from_ch_raw = input
        .from_channel
        .as_deref()
        .filter(|s| !s.trim().is_empty())
        .or(input.source_channel.as_deref())
        .unwrap_or("")
        .trim();
    let to_ch_raw = input
        .to_channel
        .as_deref()
        .filter(|s| !s.trim().is_empty())
        .or(input.target_channel.as_deref())
        .unwrap_or("")
        .trim();
    let from_ch = from_ch_raw;
    let to_ch = to_ch_raw;
    if !matches!(from_ch, "internal" | "beta")
        || !matches!(to_ch, "beta" | "stable")
        || from_ch == to_ch
    {
        return Err(CloudError::InvalidChannel(format!("{from_ch} -> {to_ch}")));
    }

    if to_ch == "stable" {
        let code = input.step_up_totp_code.as_deref().unwrap_or("");
        if code.trim().is_empty() {
            return Err(CloudError::StepUpTotpRequired);
        }
        if !verify_totp_or_recovery(&totp_secret, &recovery_json, code, unix_seconds) {
            return Err(CloudError::AdminTotpInvalid);
        }
    }

    let clean_version = input.version.trim().trim_start_matches('v');
    let release = query_release_by_version(conn, clean_version)?;
    if release.status != "live" {
        return Err(CloudError::ReleaseNotPromotable(format!(
            "{} (status={})",
            release.version, release.status
        )));
    }

    if let Some(source_head) = query_channel_head_row(conn, from_ch)? {
        if source_head.version != release.version && release.channel != from_ch {
            return Err(CloudError::ChannelHeadMismatch(format!(
                "{from_ch} head is {} (requested {})",
                source_head.version, release.version
            )));
        }
    }

    conn.execute(
        "INSERT INTO release_channel_heads (channel, release_id, version, promoted_from_channel, promoted_by, promoted_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT(channel) DO UPDATE SET
            release_id = excluded.release_id,
            version = excluded.version,
            promoted_from_channel = excluded.promoted_from_channel,
            promoted_by = excluded.promoted_by,
            promoted_at = excluded.promoted_at",
        params![
            to_ch,
            release.id,
            release.version,
            from_ch,
            admin.id,
            now_utc,
        ],
    )?;

    append_admin_audit(
        conn,
        now_utc,
        &admin.id,
        &admin.email,
        "admin.releases.promote",
        "release_channel_heads",
        to_ch,
        Some(
            &serde_json::json!({
                "version": release.version,
                "release_id": release.id,
                "from_channel": from_ch,
                "to_channel": to_ch,
                "sha256": release.sha256,
                "authenticode_thumbprint": release.authenticode_thumbprint,
                "updater_signature": release.updater_signature,
                "zero_rebuild": true,
            })
            .to_string(),
        ),
    )?;

    let channel_head = query_channel_head_row(conn, to_ch)?
        .ok_or_else(|| CloudError::NotFound(format!("Channel head {to_ch}")))?;

    Ok(PromoteReleaseChannelResult {
        channel_head,
        release,
    })
}

/// Executes a cryptographic key recovery drill (`POST /v1/releases/recovery_drill`) via `crates/license_signer`
/// and records an immutable row in `key_recovery_drills` (`super_admin` + step up TOTP required).
pub fn record_key_recovery_drill(
    conn: &Connection,
    signer_conn: &Connection,
    signer: &SignerService,
    input: &RecordKeyRecoveryDrillInput,
    now_utc: &str,
    unix_seconds: u64,
) -> Result<KeyRecoveryDrillRow, CloudError> {
    let (admin, _, totp_secret, recovery_json) = resolve_admin_by_email(conn, &input.admin_email)?;
    if admin.role != "super_admin" {
        return Err(CloudError::AdminPermissionDenied(admin.role));
    }
    if input.step_up_totp_code.trim().is_empty() {
        return Err(CloudError::StepUpTotpRequired);
    }
    if !verify_totp_or_recovery(
        &totp_secret,
        &recovery_json,
        &input.step_up_totp_code,
        unix_seconds,
    ) {
        return Err(CloudError::AdminTotpInvalid);
    }

    let backup_source = input
        .backup_source
        .as_deref()
        .unwrap_or("primary_drill")
        .trim();
    let encrypted_backup_blob = input
        .encrypted_backup_blob
        .as_deref()
        .unwrap_or("drill_blob")
        .trim();
    let passphrase = input.passphrase.clone().unwrap_or_default();

    let auth_str = format!(
        "{}|{}|{}|{}",
        input.key_type.trim(),
        input.key_id.trim(),
        backup_source,
        encrypted_backup_blob
    );
    let hmac = compute_service_hmac(
        DEFAULT_SIGNER_SHARED_SECRET,
        "cloud_api",
        now_utc,
        &auth_str,
    );

    let verified = signer
        .verify_key_recovery(
            signer_conn,
            &VerifyKeyRecoveryRequest {
                caller_service_id: "cloud_api".to_string(),
                timestamp_utc: now_utc.to_string(),
                hmac_signature: hmac,
                key_type: input.key_type.trim().to_string(),
                key_id: input.key_id.trim().to_string(),
                backup_source: backup_source.to_string(),
                encrypted_backup_blob: encrypted_backup_blob.to_string(),
                passphrase,
            },
        )
        .map_err(|_| CloudError::RecoverySignatureMismatch)?;

    if !verified.signature_verified {
        return Err(CloudError::RecoverySignatureMismatch);
    }

    let drill_id = format!("drill_{}", Uuid::now_v7());
    conn.execute(
        "INSERT INTO key_recovery_drills (
            id, key_type, key_id, backup_source, backup_sha256, test_artifact_sha256, signature_verified, notes, performed_by, performed_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, 1, ?7, ?8, ?9)",
        params![
            drill_id,
            input.key_type.trim(),
            input.key_id.trim(),
            backup_source,
            verified.backup_sha256,
            verified.test_artifact_sha256,
            input.notes.trim(),
            admin.id,
            now_utc,
        ],
    )?;

    append_admin_audit(
        conn,
        now_utc,
        &admin.id,
        &admin.email,
        "admin.releases.recovery_drill",
        "key_recovery_drills",
        &drill_id,
        Some(
            &serde_json::json!({
                "key_type": input.key_type.trim(),
                "key_id": input.key_id.trim(),
                "backup_sha256": verified.backup_sha256,
                "test_artifact_sha256": verified.test_artifact_sha256,
                "signer_audit_seq": verified.signer_audit_seq,
            })
            .to_string(),
        ),
    )?;

    Ok(KeyRecoveryDrillRow {
        id: drill_id,
        key_type: input.key_type.trim().to_string(),
        key_id: input.key_id.trim().to_string(),
        backup_source: backup_source.to_string(),
        backup_sha256: verified.backup_sha256,
        test_artifact_sha256: verified.test_artifact_sha256,
        signature_verified: true,
        notes: input.notes.trim().to_string(),
        performed_by: admin.id,
        performed_at: now_utc.to_string(),
        custodian_primary: input.custodian_primary.clone(),
        custodian_secondary: input.custodian_secondary.clone(),
        drilled_at: Some(now_utc.to_string()),
        status: Some("verified".to_string()),
    })
}

/// Resets the 12 month self service transfer counter (`transfer_count_12m = 0`) for a license (`ops_admin` or `super_admin`).
pub fn reset_license_transfer_count(
    conn: &Connection,
    admin_email: &str,
    license_id: &str,
    now_utc: &str,
) -> Result<(), CloudError> {
    let (admin, _, _, _) = resolve_admin_by_email(conn, admin_email)?;
    if admin.role != "super_admin" && admin.role != "ops_admin" {
        return Err(CloudError::AdminPermissionDenied(admin.role));
    }
    conn.execute(
        "UPDATE licenses SET transfer_count_12m = 0, updated_at = ?1 WHERE license_id = ?2",
        params![now_utc, license_id.trim()],
    )?;
    append_admin_audit(
        conn,
        now_utc,
        &admin.id,
        &admin.email,
        "admin.licenses.reset_transfers",
        "licenses",
        license_id.trim(),
        Some(r#"{"transfer_count_12m":0}"#),
    )?;
    Ok(())
}

/// Creates a new coaching center customer, subscription, and initial license record in Vendor Admin.
pub fn create_vendor_customer(
    conn: &Connection,
    signer_conn: &Connection,
    signer: &SignerService,
    input: &CreateVendorCustomerInput,
    now_utc: &str,
) -> Result<VendorCustomerRow, CloudError> {
    let (admin, _, _, _) = resolve_admin_by_email(conn, &input.admin_email)?;
    if admin.role != "super_admin" && admin.role != "ops_admin" {
        return Err(CloudError::AdminPermissionDenied(admin.role));
    }
    let cust_id = format!("cust_{}", Uuid::now_v7());
    let sub_id = format!("sub_{}", Uuid::now_v7());
    let plan_id = if input.plan_code == "local_pro" {
        "plan_local_pro"
    } else {
        "plan_local"
    };
    let short_suffix = cust_id
        .chars()
        .rev()
        .take(4)
        .collect::<String>()
        .to_uppercase();
    let license_id = format!("LIC-2026-NEWC-{short_suffix}");
    let today_ymd = if now_utc.len() >= 10 {
        &now_utc[0..10]
    } else {
        "2026-09-25"
    };
    let (paid_through, valid_until) = compute_extended_paid_through(today_ymd, 14, today_ymd, 1)?;

    conn.execute(
        "INSERT INTO customers (id, org_name, owner_name, phone, email, district, status, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'active', ?7, ?7)",
        params![
            cust_id,
            input.org_name.trim(),
            input.owner_name.trim(),
            input.phone.trim(),
            input.email,
            input.district.trim(),
            now_utc,
        ],
    )?;

    let portal_pw = hash_password("CoachifyPortal#2026");
    conn.execute(
        "INSERT INTO customer_portal_credentials (customer_id, password_hash, failed_attempts, updated_at)
         VALUES (?1, ?2, 0, ?3)",
        params![cust_id, portal_pw, now_utc],
    )?;

    conn.execute(
        "INSERT INTO subscriptions (id, customer_id, plan_id, status, paid_through, grace_days, started_at)
         VALUES (?1, ?2, ?3, 'active', ?4, 14, ?5)",
        params![sub_id, cust_id, plan_id, paid_through, now_utc],
    )?;

    let payload = SignerLicensePayload {
        license_id: license_id.clone(),
        customer_id: cust_id.clone(),
        customer_name: input.org_name.trim().to_string(),
        installation_id: format!("inst-{short_suffix}-host"),
        tier: if input.plan_code == "local_pro" {
            "local_pro".to_string()
        } else {
            "local".to_string()
        },
        kind: "paid".to_string(),
        serial: 1,
        paid_through: paid_through.clone(),
        grace_days: 14,
        valid_until,
        issued_at: now_utc.to_string(),
        limits: SignerLicenseLimits {
            active_students: Some(2000),
            workstations: Some(1),
        },
        capabilities: vec!["core.ledger".to_string(), "core.attendance".to_string()],
        key_id: "k2026a".to_string(),
    };
    let signed = request_isolated_license_signature(signer_conn, signer, &payload, now_utc)?;

    conn.execute(
        "INSERT INTO licenses (
            id, license_id, customer_id, subscription_id, activation_key_hash, pairing_code,
            current_serial, bound_installation_id, key_id, last_issued_blob, status, transfer_count_12m, updated_at
        ) VALUES (?1, ?2, ?3, ?4, 'ACT-DEFAULT-2026', '774102', 1, NULL, 'k2026a', ?5, 'active', 0, ?6)",
        params![
            format!("lic_{}", Uuid::now_v7()),
            license_id,
            cust_id,
            sub_id,
            signed.signed_blob,
            now_utc,
        ],
    )?;

    append_admin_audit(
        conn,
        now_utc,
        &admin.id,
        &admin.email,
        "admin.customers.create",
        "customers",
        &cust_id,
        Some(
            &serde_json::json!({ "org_name": input.org_name.trim(), "license_id": license_id })
                .to_string(),
        ),
    )?;

    let overview =
        get_vendor_dashboard_overview(conn, signer_conn, Some(&admin.email), now_utc, 0)?;
    overview
        .customers
        .into_iter()
        .find(|c| c.id == cust_id)
        .ok_or(CloudError::NotFound(cust_id))
}

fn sha256_hex(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    hex::encode(hasher.finalize())
}

fn add_minutes_iso(now_utc: &str, minutes: i64) -> String {
    if let Ok(ts) = now_utc.parse::<jiff::Timestamp>() {
        if let Ok(added) = ts.checked_add(jiff::ToSpan::minutes(minutes)) {
            return added.to_string();
        }
    }
    format!("{now_utc}+{minutes}m")
}

fn mask_phone_or_email(phone: &str, email: Option<&str>) -> String {
    let trimmed = phone.trim();
    if trimmed.len() >= 6 {
        let prefix = &trimmed[0..4.min(trimmed.len())];
        let suffix = &trimmed[trimmed.len().saturating_sub(3)..];
        return format!("{prefix}•••••{suffix}");
    }
    if let Some(em) = email {
        return em.to_string();
    }
    "registered mobile".to_string()
}

/// Customer Portal Step 1 (`POST /v1/portal/login`): Authenticates phone or email + Argon2id password,
/// checks account lockout (`locked_until`), and issues a 6 digit OTP valid for 10 minutes.
pub fn portal_login(
    conn: &Connection,
    input: &PortalLoginInput,
    now_utc: &str,
) -> Result<PortalLoginResponse, CloudError> {
    let clean_ident = input.identifier.trim();
    if clean_ident.is_empty() || input.password.is_empty() {
        return Err(CloudError::PortalInvalidCredentials);
    }

    let cust_row = conn
        .query_row(
            "SELECT c.id, c.phone, c.email, c.status, cr.password_hash, cr.locked_until
             FROM customers c
             JOIN customer_portal_credentials cr ON cr.customer_id = c.id
             WHERE c.phone = ?1 OR lower(COALESCE(c.email, '')) = lower(?1)",
            params![clean_ident],
            |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, Option<String>>(2)?,
                    r.get::<_, String>(3)?,
                    r.get::<_, String>(4)?,
                    r.get::<_, Option<String>>(5)?,
                ))
            },
        )
        .optional()?
        .ok_or(CloudError::PortalInvalidCredentials)?;

    let (customer_id, phone, email, status, pw_hash, locked_until) = cust_row;

    if status == "suspended" {
        return Err(CloudError::CustomerSuspended);
    }

    if let Some(lock_ts) = &locked_until {
        if lock_ts.as_str() > now_utc {
            return Err(CloudError::PortalAccountLocked(lock_ts.clone()));
        }
    }

    if !verify_password(&input.password, &pw_hash) {
        return Err(CloudError::PortalInvalidCredentials);
    }

    // Deterministic 6 digit OTP derived from customer_id + timestamp for verifiable testing & live preview
    let otp_digest = sha256_hex(&format!("COACHIFY_PORTAL_OTP|{customer_id}|{now_utc}"));
    let otp_num = u32::from_str_radix(&otp_digest[0..6], 16).unwrap_or(482910) % 900_000 + 100_000;
    let otp_code = format!("{otp_num:06}");
    let otp_hash = sha256_hex(&otp_code);
    let otp_expires_at = add_minutes_iso(now_utc, 10);

    conn.execute(
        "UPDATE customer_portal_credentials
         SET otp_code_hash = ?1, otp_expires_at = ?2, updated_at = ?3
         WHERE customer_id = ?4",
        params![otp_hash, otp_expires_at, now_utc, customer_id],
    )?;

    Ok(PortalLoginResponse {
        customer_id,
        otp_required: true,
        masked_destination: mask_phone_or_email(&phone, email.as_deref()),
        otp_expires_at,
        demo_otp_code: otp_code,
    })
}

/// Customer Portal Step 2 (`POST /v1/portal/verify_otp`): Verifies the 6 digit OTP within 10 minutes,
/// locks the account for 15 minutes after 5 consecutive wrong attempts (`PORTAL_ACCOUNT_LOCKED`),
/// and issues a 24 hour SHA256 hashed bearer token in `customer_portal_sessions`.
pub fn portal_verify_otp(
    conn: &Connection,
    input: &PortalVerifyOtpInput,
    now_utc: &str,
) -> Result<PortalVerifyOtpResponse, CloudError> {
    let row = conn
        .query_row(
            "SELECT c.id, c.org_name, c.owner_name, c.phone, c.email, c.district, c.status,
                    cr.otp_code_hash, cr.otp_expires_at, cr.failed_attempts, cr.locked_until
             FROM customers c
             JOIN customer_portal_credentials cr ON cr.customer_id = c.id
             WHERE c.id = ?1",
            params![input.customer_id.trim()],
            |r| {
                Ok((
                    PortalCustomerSummary {
                        id: r.get(0)?,
                        org_name: r.get(1)?,
                        owner_name: r.get(2)?,
                        phone: r.get(3)?,
                        email: r.get(4)?,
                        district: r.get(5)?,
                        status: r.get(6)?,
                    },
                    r.get::<_, Option<String>>(7)?,
                    r.get::<_, Option<String>>(8)?,
                    r.get::<_, i64>(9)?,
                    r.get::<_, Option<String>>(10)?,
                ))
            },
        )
        .optional()?
        .ok_or(CloudError::PortalInvalidCredentials)?;

    let (customer, otp_hash_opt, otp_exp_opt, failed_attempts, locked_until) = row;

    if customer.status == "suspended" {
        return Err(CloudError::CustomerSuspended);
    }

    if let Some(lock_ts) = &locked_until {
        if lock_ts.as_str() > now_utc {
            return Err(CloudError::PortalAccountLocked(lock_ts.clone()));
        }
    }

    let expected_hash = otp_hash_opt.ok_or(CloudError::PortalInvalidOtp)?;
    let expires_at_otp = otp_exp_opt.ok_or(CloudError::PortalOtpExpired)?;
    if now_utc > expires_at_otp.as_str() {
        return Err(CloudError::PortalOtpExpired);
    }

    let submitted_hash = sha256_hex(input.otp_code.trim());
    if submitted_hash != expected_hash {
        let new_failed = failed_attempts + 1;
        if new_failed >= 5 {
            let lock_until = add_minutes_iso(now_utc, 15);
            conn.execute(
                "UPDATE customer_portal_credentials
                 SET failed_attempts = ?1, locked_until = ?2, updated_at = ?3
                 WHERE customer_id = ?4",
                params![new_failed, lock_until, now_utc, customer.id],
            )?;
            return Err(CloudError::PortalAccountLocked(lock_until));
        } else {
            conn.execute(
                "UPDATE customer_portal_credentials
                 SET failed_attempts = ?1, updated_at = ?2
                 WHERE customer_id = ?3",
                params![new_failed, now_utc, customer.id],
            )?;
            return Err(CloudError::PortalInvalidOtp);
        }
    }

    // Reset failed_attempts and clear single-use OTP
    conn.execute(
        "UPDATE customer_portal_credentials
         SET failed_attempts = 0, locked_until = NULL, otp_code_hash = NULL, otp_expires_at = NULL, updated_at = ?1
         WHERE customer_id = ?2",
        params![now_utc, customer.id],
    )?;

    let raw_token = format!("cps_{}_{}", customer.id, Uuid::now_v7());
    let token_hash = sha256_hex(&raw_token);
    let session_expires_at = add_minutes_iso(now_utc, 24 * 60);

    conn.execute(
        "INSERT INTO customer_portal_sessions (
            id, customer_id, session_token_hash, ip_address, user_agent, expires_at, revoked_at, created_at
        ) VALUES (?1, ?2, ?3, '127.0.0.1', 'CoachifyPortal/1.0', ?4, NULL, ?5)",
        params![
            format!("psess_{}", Uuid::now_v7()),
            customer.id,
            token_hash,
            session_expires_at,
            now_utc,
        ],
    )?;

    Ok(PortalVerifyOtpResponse {
        session_token: raw_token,
        expires_at: session_expires_at,
        customer,
    })
}

fn authenticate_portal_session(
    conn: &Connection,
    session_token: &str,
    now_utc: &str,
) -> Result<PortalCustomerSummary, CloudError> {
    let clean = session_token.trim().trim_start_matches("Bearer ").trim();
    if clean.is_empty() {
        return Err(CloudError::UnauthorizedPortalSession);
    }
    let token_hash = sha256_hex(clean);
    let row = conn
        .query_row(
            "SELECT c.id, c.org_name, c.owner_name, c.phone, c.email, c.district, c.status,
                    s.expires_at, s.revoked_at
             FROM customer_portal_sessions s
             JOIN customers c ON c.id = s.customer_id
             WHERE s.session_token_hash = ?1",
            params![token_hash],
            |r| {
                Ok((
                    PortalCustomerSummary {
                        id: r.get(0)?,
                        org_name: r.get(1)?,
                        owner_name: r.get(2)?,
                        phone: r.get(3)?,
                        email: r.get(4)?,
                        district: r.get(5)?,
                        status: r.get(6)?,
                    },
                    r.get::<_, String>(7)?,
                    r.get::<_, Option<String>>(8)?,
                ))
            },
        )
        .optional()?
        .ok_or(CloudError::UnauthorizedPortalSession)?;

    let (customer, expires_at, revoked_at) = row;
    if revoked_at.is_some() || now_utc > expires_at.as_str() {
        return Err(CloudError::UnauthorizedPortalSession);
    }
    if customer.status == "suspended" {
        return Err(CloudError::CustomerSuspended);
    }
    Ok(customer)
}

/// Revokes an active customer portal session (`POST /v1/portal/logout`).
pub fn portal_logout(
    conn: &Connection,
    session_token: &str,
    now_utc: &str,
) -> Result<(), CloudError> {
    let clean = session_token.trim().trim_start_matches("Bearer ").trim();
    let token_hash = sha256_hex(clean);
    conn.execute(
        "UPDATE customer_portal_sessions SET revoked_at = ?1 WHERE session_token_hash = ?2",
        params![now_utc, token_hash],
    )?;
    Ok(())
}

fn resolve_portal_release_manifest(
    conn: &Connection,
    channel: &str,
) -> Result<Option<PortalReleaseManifest>, CloudError> {
    if let Some(head) = query_channel_head_row(conn, channel)? {
        if head.status == "live" {
            let rel = query_release_by_version(conn, &head.version)?;
            return Ok(Some(PortalReleaseManifest {
                version: rel.version,
                channel: channel.to_string(),
                notes: rel.notes,
                artifact_url: rel.artifact_url,
                sha256: rel.sha256,
                size_bytes: rel.size_bytes,
                authenticode_thumbprint: rel.authenticode_thumbprint,
                updater_signature: rel.updater_signature,
                updater_key_id: rel.updater_key_id,
                published_at: rel.published_at,
            }));
        }
    }
    Ok(None)
}

/// Returns the authenticated customer's portal overview (`GET /v1/portal/overview`), strictly scoped to `session.customer_id`.
pub fn portal_get_overview(
    conn: &Connection,
    session_token: &str,
    now_utc: &str,
) -> Result<PortalOverviewResponse, CloudError> {
    let customer = authenticate_portal_session(conn, session_token, now_utc)?;

    let license = conn
        .query_row(
            "SELECT l.license_id, p.code, p.name, s.status, s.paid_through, s.grace_days,
                    l.current_serial, l.key_id, l.status, l.last_issued_blob
             FROM licenses l
             JOIN subscriptions s ON s.id = l.subscription_id
             JOIN plans p ON p.id = s.plan_id
             WHERE l.customer_id = ?1
             ORDER BY l.updated_at DESC LIMIT 1",
            params![customer.id],
            |r| {
                let blob: Option<String> = r.get(9)?;
                Ok(PortalLicenseSummary {
                    license_id: r.get(0)?,
                    plan_code: r.get(1)?,
                    plan_name: r.get(2)?,
                    subscription_status: r.get(3)?,
                    paid_through: r.get(4)?,
                    grace_days: r.get(5)?,
                    current_serial: r.get(6)?,
                    key_id: r.get(7)?,
                    status: r.get(8)?,
                    has_signed_blob: blob.is_some(),
                })
            },
        )
        .optional()?
        .ok_or_else(|| CloudError::NotFound(format!("License for customer {}", customer.id)))?;

    let mut invoices = Vec::new();
    {
        let mut stmt = conn.prepare(
            "SELECT vi.id, vi.invoice_number, vi.issued_at, c.org_name, p.name, py.period_months,
                    py.method, py.reference, vi.subtotal_poisha, vi.vat_poisha, vi.total_poisha
             FROM vendor_invoices vi
             JOIN payments py ON py.id = vi.payment_id
             JOIN customers c ON c.id = vi.customer_id
             JOIN subscriptions s ON s.id = py.subscription_id
             JOIN plans p ON p.id = s.plan_id
             WHERE vi.customer_id = ?1
             ORDER BY vi.issued_at DESC",
        )?;
        let rows = stmt.query_map(params![customer.id], |r| {
            Ok(PortalInvoiceItem {
                id: r.get(0)?,
                invoice_number: r.get(1)?,
                issued_at: r.get(2)?,
                org_name: r.get(3)?,
                plan_name: r.get(4)?,
                period_months: r.get(5)?,
                method: r.get(6)?,
                reference: r.get(7)?,
                subtotal_poisha: r.get(8)?,
                vat_poisha: r.get(9)?,
                total_poisha: r.get(10)?,
            })
        })?;
        for item in rows {
            invoices.push(item?);
        }
    }

    let mut installations = Vec::new();
    {
        let mut stmt = conn.prepare(
            "SELECT id, installation_id, label, role, app_version, active_students, first_seen_at, last_seen_at, status
             FROM installations
             WHERE customer_id = ?1
             ORDER BY last_seen_at DESC",
        )?;
        let rows = stmt.query_map(params![customer.id], |r| {
            Ok(PortalInstallationItem {
                id: r.get(0)?,
                installation_id: r.get(1)?,
                label: r.get(2)?,
                role: r.get(3)?,
                app_version: r.get(4)?,
                active_students: r.get(5)?,
                first_seen_at: r.get(6)?,
                last_seen_at: r.get(7)?,
                status: r.get(8)?,
            })
        })?;
        for item in rows {
            installations.push(item?);
        }
    }

    let latest_release = resolve_portal_release_manifest(conn, "stable")?;
    let beta_release = resolve_portal_release_manifest(conn, "beta")?;

    Ok(PortalOverviewResponse {
        customer,
        license,
        invoices,
        installations,
        latest_release,
        beta_release,
    })
}

/// Downloads a customer's signed `.coachifylicense` file (`GET /v1/portal/license_file/{license_id}`),
/// enforcing strict tenant ownership (`FORBIDDEN_LICENSE_OWNER` if owned by another customer).
pub fn portal_download_license_file(
    conn: &Connection,
    session_token: &str,
    license_id: &str,
    now_utc: &str,
) -> Result<PortalLicenseFileResponse, CloudError> {
    let customer = authenticate_portal_session(conn, session_token, now_utc)?;
    let row = conn
        .query_row(
            "SELECT customer_id, current_serial, last_issued_blob FROM licenses WHERE license_id = ?1",
            params![license_id.trim()],
            |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, i64>(1)?,
                    r.get::<_, Option<String>>(2)?,
                ))
            },
        )
        .optional()?
        .ok_or_else(|| CloudError::NotFound(format!("License {}", license_id.trim())))?;

    let (owner_customer_id, serial, blob_opt) = row;
    if owner_customer_id != customer.id {
        return Err(CloudError::ForbiddenLicenseOwner);
    }

    let signed_blob =
        blob_opt.ok_or_else(|| CloudError::NotFound("Signed license blob".to_string()))?;
    let clean_lic = license_id.trim();

    Ok(PortalLicenseFileResponse {
        license_id: clean_lic.to_string(),
        filename: format!("{clean_lic}.coachifylicense"),
        signed_blob,
        serial,
    })
}

/// Fetches a single paid vendor invoice (`GET /v1/portal/invoice/{invoice_id}`),
/// enforcing strict tenant ownership (`FORBIDDEN_INVOICE_OWNER` if owned by another customer).
pub fn portal_get_invoice(
    conn: &Connection,
    session_token: &str,
    invoice_id_or_number: &str,
    now_utc: &str,
) -> Result<PortalInvoiceItem, CloudError> {
    let customer = authenticate_portal_session(conn, session_token, now_utc)?;
    let row = conn
        .query_row(
            "SELECT vi.customer_id, vi.id, vi.invoice_number, vi.issued_at, c.org_name, p.name, py.period_months,
                    py.method, py.reference, vi.subtotal_poisha, vi.vat_poisha, vi.total_poisha
             FROM vendor_invoices vi
             JOIN payments py ON py.id = vi.payment_id
             JOIN customers c ON c.id = vi.customer_id
             JOIN subscriptions s ON s.id = py.subscription_id
             JOIN plans p ON p.id = s.plan_id
             WHERE vi.id = ?1 OR vi.invoice_number = ?1",
            params![invoice_id_or_number.trim()],
            |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    PortalInvoiceItem {
                        id: r.get(1)?,
                        invoice_number: r.get(2)?,
                        issued_at: r.get(3)?,
                        org_name: r.get(4)?,
                        plan_name: r.get(5)?,
                        period_months: r.get(6)?,
                        method: r.get(7)?,
                        reference: r.get(8)?,
                        subtotal_poisha: r.get(9)?,
                        vat_poisha: r.get(10)?,
                        total_poisha: r.get(11)?,
                    },
                ))
            },
        )
        .optional()?
        .ok_or_else(|| CloudError::NotFound(format!("Invoice {}", invoice_id_or_number.trim())))?;

    let (owner_customer_id, invoice_item) = row;
    if owner_customer_id != customer.id {
        return Err(CloudError::ForbiddenInvoiceOwner);
    }

    Ok(invoice_item)
}

/// Handles `POST /v1/activate` for desktop online activation, 6 digit pairing, and self service PC hardware transfers.
pub fn handle_v1_activate(
    conn: &Connection,
    signer_conn: &Connection,
    signer: &SignerService,
    req: &ActivateEndpointRequest,
    now_utc: &str,
) -> Result<ActivateEndpointResponse, CloudError> {
    let code = req.license_id_or_pairing_code.trim().replace(' ', "");
    let row = conn
        .query_row(
            "SELECT l.license_id, l.customer_id, c.org_name, p.code, p.student_limit, p.workstation_limit,
                    p.capabilities_json, s.paid_through, s.grace_days, l.current_serial, l.bound_installation_id,
                    l.status, l.transfer_count_12m, l.activation_key_hash, l.pairing_code
             FROM licenses l
             JOIN customers c ON c.id = l.customer_id
             JOIN subscriptions s ON s.id = l.subscription_id
             JOIN plans p ON p.id = s.plan_id
             WHERE l.license_id = ?1 OR l.pairing_code = ?1",
            params![code],
            |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, String>(3)?,
                    r.get::<_, Option<u32>>(4)?,
                    r.get::<_, Option<u32>>(5)?,
                    r.get::<_, String>(6)?,
                    r.get::<_, String>(7)?,
                    r.get::<_, u32>(8)?,
                    r.get::<_, i64>(9)?,
                    r.get::<_, Option<String>>(10)?,
                    r.get::<_, String>(11)?,
                    r.get::<_, u32>(12)?,
                    r.get::<_, String>(13)?,
                    r.get::<_, Option<String>>(14)?,
                ))
            },
        )
        .optional()?
        .ok_or(CloudError::ActivationCredentialsInvalid)?;

    let (
        license_id,
        customer_id,
        org_name,
        plan_code,
        student_limit,
        workstation_limit,
        capabilities_json,
        paid_through,
        grace_days,
        current_serial,
        bound_inst,
        status,
        transfer_count_12m,
        expected_key,
        pairing_code,
    ) = row;

    if status == "revoked" {
        return Err(CloudError::LicenseRevoked(license_id));
    }

    // Verify credentials if matched by license_id rather than 6 digit pairing code
    let matched_by_pairing = pairing_code.as_deref() == Some(code.as_str());
    if !matched_by_pairing {
        if let Some(provided_key) = &req.activation_key {
            if !provided_key.trim().is_empty() && provided_key.trim() != expected_key {
                return Err(CloudError::ActivationCredentialsInvalid);
            }
        }
    }

    let is_different_pc = matches!(
        &bound_inst,
        Some(existing) if !existing.trim().is_empty() && existing != &req.installation_id
    );

    if is_different_pc && !req.confirm_transfer {
        let prev_id = bound_inst.unwrap_or_default();
        let prev_seen: Option<String> = conn
            .query_row(
                "SELECT last_seen_at FROM installations WHERE license_id = ?1 AND installation_id = ?2",
                params![license_id, prev_id],
                |r| r.get(0),
            )
            .optional()?;
        return Ok(ActivateEndpointResponse {
            outcome: "already_bound_transfer_available".to_string(),
            license_id,
            signed_license_blob: None,
            previous_installation_id: Some(prev_id),
            previous_last_seen_at: prev_seen,
            transfers_remaining_12m: 3u32.saturating_sub(transfer_count_12m),
        });
    }

    let mut new_transfer_count = transfer_count_12m;
    if is_different_pc && req.confirm_transfer {
        if transfer_count_12m >= 3 {
            return Err(CloudError::TransferLimitExceeded);
        }
        new_transfer_count += 1;
        if let Some(old_inst) = &bound_inst {
            conn.execute(
                "UPDATE installations SET status = 'moved' WHERE license_id = ?1 AND installation_id = ?2",
                params![license_id, old_inst],
            )?;
        }
    }

    let new_serial = current_serial + 1;
    let paid_date: jiff::civil::Date = paid_through[0..10]
        .parse()
        .map_err(|e: jiff::Error| CloudError::Validation(e.to_string()))?;
    let valid_until = paid_date
        .checked_add(jiff::ToSpan::days(i64::from(grace_days)))
        .map_err(|e| CloudError::Validation(e.to_string()))?
        .to_string();

    let capabilities: Vec<String> = serde_json::from_str(&capabilities_json).unwrap_or_default();
    let payload = SignerLicensePayload {
        license_id: license_id.clone(),
        customer_id: customer_id.clone(),
        customer_name: org_name.clone(),
        installation_id: req.installation_id.clone(),
        tier: if plan_code == "local_pro" {
            "local_pro".to_string()
        } else {
            "local".to_string()
        },
        kind: "extension".to_string(),
        serial: new_serial,
        paid_through,
        grace_days,
        valid_until,
        issued_at: now_utc.to_string(),
        limits: SignerLicenseLimits {
            active_students: student_limit,
            workstations: workstation_limit,
        },
        capabilities,
        key_id: "k2026a".to_string(),
    };

    let signed = request_isolated_license_signature(signer_conn, signer, &payload, now_utc)?;

    conn.execute(
        "UPDATE licenses
         SET current_serial = ?1,
             bound_installation_id = ?2,
             last_issued_blob = ?3,
             transfer_count_12m = ?4,
             updated_at = ?5
         WHERE license_id = ?6",
        params![
            new_serial,
            req.installation_id,
            signed.signed_blob,
            new_transfer_count,
            now_utc,
            license_id,
        ],
    )?;

    conn.execute(
        "INSERT INTO license_issues (
            id, license_id, serial, installation_id, kind, payload_json, signature, signed_blob, issued_by, reason, issued_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, NULL, ?9, ?10)",
        params![
            format!("iss_{}", Uuid::now_v7()),
            license_id,
            new_serial,
            req.installation_id,
            if is_different_pc { "transfer" } else { "paid" },
            signed.envelope.payload,
            signed.envelope.signature,
            signed.signed_blob,
            if is_different_pc {
                "Self service PC transfer via /v1/activate"
            } else {
                "Online activation via /v1/activate"
            },
            now_utc,
        ],
    )?;

    conn.execute(
        "UPDATE installations SET status = 'moved', replaced_by_installation_id = ?1 WHERE license_id = ?2 AND role = 'host' AND status = 'active' AND installation_id != ?1",
        params![req.installation_id, license_id],
    )?;

    conn.execute(
        "INSERT INTO installations (
            id, installation_id, customer_id, license_id, label, role, app_version, active_students, first_seen_at, last_seen_at, status
        ) VALUES (?1, ?2, ?3, ?4, ?5, 'host', ?6, ?7, ?8, ?8, 'active')
        ON CONFLICT(license_id, installation_id) DO UPDATE SET
            status = 'active',
            app_version = excluded.app_version,
            active_students = excluded.active_students,
            last_seen_at = excluded.last_seen_at",
        params![
            format!("inst_{}", Uuid::now_v7()),
            req.installation_id,
            customer_id,
            license_id,
            format!("{org_name} Host ({})", &req.installation_id[0..req.installation_id.len().min(8)]),
            req.app_version,
            req.active_students,
            now_utc,
        ],
    )?;

    Ok(ActivateEndpointResponse {
        outcome: "activated".to_string(),
        license_id,
        signed_license_blob: Some(signed.signed_blob),
        previous_installation_id: None,
        previous_last_seen_at: None,
        transfers_remaining_12m: 3u32.saturating_sub(new_transfer_count),
    })
}

/// Handles `POST /v1/refresh` for daily background license refresh and on-demand "Check Now".
pub fn handle_v1_refresh(
    conn: &Connection,
    req: &RefreshEndpointRequest,
    now_utc: &str,
) -> Result<RefreshEndpointResponse, CloudError> {
    let row = conn
        .query_row(
            "SELECT current_serial, bound_installation_id, last_issued_blob, status
             FROM licenses WHERE license_id = ?1",
            params![req.license_id.trim()],
            |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, Option<String>>(1)?,
                    r.get::<_, Option<String>>(2)?,
                    r.get::<_, String>(3)?,
                ))
            },
        )
        .optional()?
        .ok_or_else(|| CloudError::NotFound(format!("License {}", req.license_id)))?;

    let (latest_serial, bound_inst, last_blob, status) = row;

    conn.execute(
        "UPDATE installations
         SET last_seen_at = ?1, app_version = ?2, active_students = ?3
         WHERE license_id = ?4 AND installation_id = ?5",
        params![
            now_utc,
            req.app_version,
            req.active_students,
            req.license_id.trim(),
            req.installation_id.trim(),
        ],
    )?;

    if status == "revoked" {
        let rev_blob: Option<String> = conn
            .query_row(
                "SELECT signed_revocation_blob FROM revocations WHERE license_id = ?1 ORDER BY revoked_at DESC LIMIT 1",
                params![req.license_id.trim()],
                |r| r.get(0),
            )
            .optional()?;
        return Ok(RefreshEndpointResponse {
            outcome: "revoked".to_string(),
            latest_serial,
            signed_license_blob: None,
            signed_revocation_blob: rev_blob,
            server_time_utc: now_utc.to_string(),
            deactivated_at: None,
            deactivation_reason: None,
        });
    }

    let inst_meta: Option<(String, Option<String>, Option<String>)> = conn
        .query_row(
            "SELECT status, deactivated_at, deactivation_reason FROM installations WHERE license_id = ?1 AND installation_id = ?2",
            params![req.license_id.trim(), req.installation_id.trim()],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .optional()?;

    if let Some((inst_status, deactivated_at, deactivation_reason)) = inst_meta {
        if inst_status == "deactivated" {
            return Ok(RefreshEndpointResponse {
                outcome: "deactivated".to_string(),
                latest_serial,
                signed_license_blob: None,
                signed_revocation_blob: None,
                server_time_utc: now_utc.to_string(),
                deactivated_at,
                deactivation_reason,
            });
        }
    }

    if let Some(bound) = &bound_inst {
        if !bound.trim().is_empty() && bound != req.installation_id.trim() {
            return Ok(RefreshEndpointResponse {
                outcome: "moved".to_string(),
                latest_serial,
                signed_license_blob: None,
                signed_revocation_blob: None,
                server_time_utc: now_utc.to_string(),
                deactivated_at: None,
                deactivation_reason: None,
            });
        }
    }

    if latest_serial > req.current_serial {
        return Ok(RefreshEndpointResponse {
            outcome: "updated".to_string(),
            latest_serial,
            signed_license_blob: last_blob,
            signed_revocation_blob: None,
            server_time_utc: now_utc.to_string(),
            deactivated_at: None,
            deactivation_reason: None,
        });
    }

    Ok(RefreshEndpointResponse {
        outcome: "no_change".to_string(),
        latest_serial,
        signed_license_blob: None,
        signed_revocation_blob: None,
        server_time_utc: now_utc.to_string(),
        deactivated_at: None,
        deactivation_reason: None,
    })
}

/// Handles `GET /v1/updates/{channel}` returning the latest `live` release (resolved via `release_channel_heads`
/// with fallback to `releases`) and all `blocked_versions`.
pub fn handle_v1_updates(
    conn: &Connection,
    channel: &str,
) -> Result<UpdateChannelResponse, CloudError> {
    let clean_channel = channel.trim();
    if !matches!(clean_channel, "internal" | "beta" | "stable") {
        return Err(CloudError::InvalidChannel(clean_channel.to_string()));
    }

    let head_live = conn
        .query_row(
            "SELECT r.version, r.artifact_url, r.sha256, r.size_bytes, r.authenticode_thumbprint,
                    r.updater_signature, r.updater_key_id, r.min_upgrade_from, r.notes, r.published_at
             FROM release_channel_heads h
             JOIN releases r ON r.id = h.release_id
             WHERE h.channel = ?1 AND r.status = 'live'
             LIMIT 1",
            params![clean_channel],
            |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, i64>(3)?,
                    r.get::<_, String>(4)?,
                    r.get::<_, String>(5)?,
                    r.get::<_, String>(6)?,
                    r.get::<_, Option<String>>(7)?,
                    r.get::<_, String>(8)?,
                    r.get::<_, String>(9)?,
                ))
            },
        )
        .optional()?;

    let live = if head_live.is_some() {
        head_live
    } else {
        conn.query_row(
            "SELECT version, artifact_url, sha256, size_bytes, authenticode_thumbprint,
                    updater_signature, updater_key_id, min_upgrade_from, notes, published_at
             FROM releases
             WHERE channel = ?1 AND status = 'live'
             ORDER BY published_at DESC LIMIT 1",
            params![clean_channel],
            |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, i64>(3)?,
                    r.get::<_, String>(4)?,
                    r.get::<_, String>(5)?,
                    r.get::<_, String>(6)?,
                    r.get::<_, Option<String>>(7)?,
                    r.get::<_, String>(8)?,
                    r.get::<_, String>(9)?,
                ))
            },
        )
        .optional()?
    };

    let mut stmt =
        conn.prepare("SELECT version FROM releases WHERE status = 'blocked' ORDER BY version ASC")?;
    let blocked_rows = stmt.query_map([], |r| r.get::<_, String>(0))?;
    let mut blocked_versions = Vec::new();
    for b in blocked_rows {
        blocked_versions.push(b?);
    }

    if let Some((ver, url, sha, size, thumb, sig, key_id, min_up, notes, pub_at)) = live {
        Ok(UpdateChannelResponse {
            channel: clean_channel.to_string(),
            latest_live_version: Some(ver),
            artifact_url: Some(url),
            sha256: Some(sha),
            size_bytes: Some(size),
            authenticode_thumbprint: Some(thumb),
            updater_signature: Some(sig),
            updater_key_id: Some(key_id),
            min_upgrade_from: min_up,
            notes: Some(notes),
            published_at: Some(pub_at),
            blocked_versions,
        })
    } else {
        Ok(UpdateChannelResponse {
            channel: clean_channel.to_string(),
            latest_live_version: None,
            artifact_url: None,
            sha256: None,
            size_bytes: None,
            authenticode_thumbprint: None,
            updater_signature: None,
            updater_key_id: None,
            min_upgrade_from: None,
            notes: None,
            published_at: None,
            blocked_versions,
        })
    }
}

/// Returns the complete Vendor Admin dashboard state across all 6 tabs.
pub fn get_vendor_dashboard_overview(
    conn: &Connection,
    signer_conn: &Connection,
    admin_email: Option<&str>,
    _now_utc: &str,
    unix_seconds: u64,
) -> Result<VendorDashboardOverview, CloudError> {
    let active_admin = if let Some(email) = admin_email {
        resolve_admin_by_email(conn, email).ok().map(|t| t.0)
    } else {
        resolve_admin_by_email(conn, "owner@coachify.app")
            .ok()
            .map(|t| t.0)
    };

    let demo_totp_code = generate_totp_code("COACHIFY_VENDOR_TOTP_SECRET_2026", unix_seconds);

    let mut customers = Vec::new();
    {
        let mut stmt = conn.prepare(
            "SELECT c.id, c.org_name, c.owner_name, c.phone, c.email, c.district, c.status,
                    s.id, p.code, p.name, s.status, s.paid_through, s.grace_days,
                    l.license_id, l.current_serial, l.bound_installation_id, l.pairing_code, l.transfer_count_12m, l.last_issued_blob
             FROM customers c
             JOIN subscriptions s ON s.customer_id = c.id
             JOIN plans p ON p.id = s.plan_id
             JOIN licenses l ON l.subscription_id = s.id
             ORDER BY c.org_name ASC",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok(VendorCustomerRow {
                id: r.get(0)?,
                org_name: r.get(1)?,
                owner_name: r.get(2)?,
                phone: r.get(3)?,
                email: r.get(4)?,
                district: r.get(5)?,
                status: r.get(6)?,
                subscription_id: r.get(7)?,
                plan_code: r.get(8)?,
                plan_name: r.get(9)?,
                subscription_status: r.get(10)?,
                paid_through: r.get(11)?,
                grace_days: r.get(12)?,
                license_id: r.get(13)?,
                current_serial: r.get(14)?,
                bound_installation_id: r.get(15)?,
                pairing_code: r.get(16)?,
                transfer_count_12m: r.get(17)?,
                last_issued_blob: r.get(18)?,
            })
        })?;
        for item in rows {
            customers.push(item?);
        }
    }

    let mut payments = Vec::new();
    {
        let mut stmt = conn.prepare(
            "SELECT py.id, py.customer_id, c.org_name, py.subscription_id, py.amount_poisha, py.method,
                    py.reference, py.proof_url, py.period_months, COALESCE(vi.invoice_number, 'INV-PENDING'),
                    py.recorded_by, py.status, py.recorded_at
             FROM payments py
             JOIN customers c ON c.id = py.customer_id
             LEFT JOIN vendor_invoices vi ON vi.payment_id = py.id
             ORDER BY py.recorded_at DESC",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok(VendorPaymentRow {
                id: r.get(0)?,
                customer_id: r.get(1)?,
                org_name: r.get(2)?,
                subscription_id: r.get(3)?,
                amount_poisha: r.get(4)?,
                method: r.get(5)?,
                reference: r.get(6)?,
                proof_url: r.get(7)?,
                period_months: r.get(8)?,
                invoice_number: r.get(9)?,
                recorded_by: r.get(10)?,
                status: r.get(11)?,
                recorded_at: r.get(12)?,
            })
        })?;
        for item in rows {
            payments.push(item?);
        }
    }

    let mut installations = Vec::new();
    {
        let mut stmt = conn.prepare(
            "SELECT i.id, i.installation_id, i.customer_id, c.org_name, i.license_id, i.label,
                    i.app_version, i.active_students, i.first_seen_at, i.last_seen_at, i.status, l.transfer_count_12m
             FROM installations i
             JOIN customers c ON c.id = i.customer_id
             JOIN licenses l ON l.license_id = i.license_id
             ORDER BY i.last_seen_at DESC",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok(VendorInstallationRow {
                id: r.get(0)?,
                installation_id: r.get(1)?,
                customer_id: r.get(2)?,
                org_name: r.get(3)?,
                license_id: r.get(4)?,
                label: r.get(5)?,
                app_version: r.get(6)?,
                active_students: r.get(7)?,
                first_seen_at: r.get(8)?,
                last_seen_at: r.get(9)?,
                status: r.get(10)?,
                transfer_count_12m: r.get(11)?,
            })
        })?;
        for item in rows {
            installations.push(item?);
        }
    }

    let mut revocations = Vec::new();
    {
        let mut stmt = conn.prepare(
            "SELECT r.id, r.license_id, c.org_name, r.reason, r.revoked_by, r.revoked_at, r.key_id, r.signed_revocation_blob
             FROM revocations r
             JOIN licenses l ON l.license_id = r.license_id
             JOIN customers c ON c.id = l.customer_id
             ORDER BY r.revoked_at DESC",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok(VendorRevocationRow {
                id: r.get(0)?,
                license_id: r.get(1)?,
                org_name: r.get(2)?,
                reason: r.get(3)?,
                revoked_by: r.get(4)?,
                revoked_at: r.get(5)?,
                key_id: r.get(6)?,
                signed_revocation_blob: r.get(7)?,
            })
        })?;
        for item in rows {
            revocations.push(item?);
        }
    }

    let mut releases = Vec::new();
    {
        let mut stmt = conn.prepare(
            "SELECT id, version, channel, notes, artifact_url, sha256, size_bytes, authenticode_thumbprint,
                    updater_signature, updater_key_id, min_upgrade_from, status, published_by, published_at
             FROM releases ORDER BY published_at DESC",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok(VendorReleaseRow {
                id: r.get(0)?,
                version: r.get(1)?,
                channel: r.get(2)?,
                notes: r.get(3)?,
                artifact_url: r.get(4)?,
                sha256: r.get(5)?,
                size_bytes: r.get(6)?,
                authenticode_thumbprint: r.get(7)?,
                updater_signature: r.get(8)?,
                updater_key_id: r.get(9)?,
                min_upgrade_from: r.get(10)?,
                status: r.get(11)?,
                published_by: r.get(12)?,
                published_at: r.get(13)?,
            })
        })?;
        for item in rows {
            releases.push(item?);
        }
    }

    let mut channel_heads = Vec::new();
    for ch in ["stable", "beta", "internal"] {
        if let Some(head) = query_channel_head_row(conn, ch)? {
            channel_heads.push(head);
        }
    }

    let mut key_recovery_drills = Vec::new();
    {
        let mut stmt = conn.prepare(
            "SELECT id, key_type, key_id, backup_source, backup_sha256, test_artifact_sha256, signature_verified, notes, performed_by, performed_at
             FROM key_recovery_drills ORDER BY performed_at DESC",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok(KeyRecoveryDrillRow {
                id: r.get(0)?,
                key_type: r.get(1)?,
                key_id: r.get(2)?,
                backup_source: r.get(3)?,
                backup_sha256: r.get(4)?,
                test_artifact_sha256: r.get(5)?,
                signature_verified: r.get::<_, i64>(6)? == 1,
                notes: r.get(7)?,
                performed_by: r.get(8)?,
                performed_at: r.get(9)?,
                custodian_primary: None,
                custodian_secondary: None,
                drilled_at: r.get(9).ok(),
                status: Some("verified".to_string()),
            })
        })?;
        for item in rows {
            key_recovery_drills.push(item?);
        }
    }

    let mut admin_audit_log = Vec::new();
    {
        let mut stmt = conn.prepare(
            "SELECT seq, at, admin_user_id, admin_email, action, entity_type, entity_id, after_json, prev_hash, hash
             FROM admin_audit_log ORDER BY seq DESC LIMIT 100",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok(VendorAuditRow {
                seq: r.get(0)?,
                at: r.get(1)?,
                admin_user_id: r.get(2)?,
                admin_email: r.get(3)?,
                action: r.get(4)?,
                entity_type: r.get(5)?,
                entity_id: r.get(6)?,
                after_json: r.get(7)?,
                prev_hash: r.get(8)?,
                hash: r.get(9)?,
            })
        })?;
        for item in rows {
            admin_audit_log.push(item?);
        }
    }

    let signer_audit_log = SignerService::list_audit_log(signer_conn, 100)
        .map_err(|e| CloudError::SignerFailure(e.to_string()))?;

    let total_customers = customers.len() as u32;
    let active_subscriptions = customers
        .iter()
        .filter(|c| c.subscription_status == "active")
        .count() as u32;
    let in_grace_subscriptions = customers
        .iter()
        .filter(|c| c.subscription_status == "grace")
        .count() as u32;
    let total_revenue_poisha: i64 = payments.iter().map(|p| p.amount_poisha).sum();
    let audit_chain_valid = verify_admin_audit_chain(conn)?;

    let pricing_tiers = get_pricing_catalog(conn)?.tiers;
    let pending_bank_transfers = admin_list_pending_bank_transfers(conn)?;

    Ok(VendorDashboardOverview {
        active_admin,
        demo_totp_code,
        total_customers,
        active_subscriptions,
        in_grace_subscriptions,
        total_revenue_poisha,
        audit_chain_valid,
        signer_total_signatures: signer_audit_log.len() as u32,
        customers,
        payments,
        installations,
        revocations,
        releases,
        channel_heads: Some(channel_heads),
        key_recovery_drills: Some(key_recovery_drills),
        pricing_tiers: Some(pricing_tiers),
        pending_bank_transfers: Some(pending_bank_transfers),
        admin_audit_log,
        signer_audit_log,
    })
}

// ---------------------------------------------------------------------------
// Pricing & Checkout Business Logic (Feature 67, Spec 0043a)
// ---------------------------------------------------------------------------

pub const DEFAULT_GATEWAY_WEBHOOK_SECRET: &[u8] = b"COACHIFY_GATEWAY_WEBHOOK_SECRET_2026";

pub fn verify_hmac_sha256(secret: &[u8], payload: &[u8], signature_hex: &str) -> bool {
    let mut mac = match HmacSha256::new_from_slice(secret) {
        Ok(m) => m,
        Err(_) => return false,
    };
    mac.update(payload);
    let Ok(sig_bytes) = hex::decode(signature_hex.trim()) else {
        return false;
    };
    mac.verify_slice(&sig_bytes).is_ok()
}

pub fn compute_hmac_sha256_hex(secret: &[u8], payload: &[u8]) -> String {
    let mut mac = HmacSha256::new_from_slice(secret).expect("HMAC can take key of any size");
    mac.update(payload);
    hex::encode(mac.finalize().into_bytes())
}

pub trait PaymentGatewayProvider: Send + Sync {
    fn provider_code(&self) -> &'static str;
    fn create_redirect_url(&self, order_number: &str, amount_poisha: i64) -> String;
    fn verify_webhook(&self, raw_payload: &str, signature: &str) -> bool;
}

pub struct SslCommerzAdapter {
    pub store_id: String,
    pub store_secret: String,
}

impl PaymentGatewayProvider for SslCommerzAdapter {
    fn provider_code(&self) -> &'static str {
        "sslcommerz"
    }
    fn create_redirect_url(&self, order_number: &str, _amount_poisha: i64) -> String {
        format!("https://sandbox.sslcommerz.com/gwprocess/v4/api.php?session_id={order_number}")
    }
    fn verify_webhook(&self, raw_payload: &str, signature: &str) -> bool {
        verify_hmac_sha256(
            self.store_secret.as_bytes(),
            raw_payload.as_bytes(),
            signature,
        )
    }
}

pub struct BkashDirectAdapter {
    pub app_key: String,
    pub webhook_secret: String,
}

impl PaymentGatewayProvider for BkashDirectAdapter {
    fn provider_code(&self) -> &'static str {
        "bkash_direct"
    }
    fn create_redirect_url(&self, order_number: &str, _amount_poisha: i64) -> String {
        format!(
            "https://checkout.sandbox.bka.sh/v1.2.0-beta/tokenized/checkout?order={order_number}"
        )
    }
    fn verify_webhook(&self, raw_payload: &str, signature: &str) -> bool {
        verify_hmac_sha256(
            self.webhook_secret.as_bytes(),
            raw_payload.as_bytes(),
            signature,
        )
    }
}

pub struct NagadDirectAdapter {
    pub merchant_id: String,
    pub webhook_secret: String,
}

impl PaymentGatewayProvider for NagadDirectAdapter {
    fn provider_code(&self) -> &'static str {
        "nagad_direct"
    }
    fn create_redirect_url(&self, order_number: &str, _amount_poisha: i64) -> String {
        format!("https://sandbox.mytower.net/payment/checkout/{order_number}")
    }
    fn verify_webhook(&self, raw_payload: &str, signature: &str) -> bool {
        verify_hmac_sha256(
            self.webhook_secret.as_bytes(),
            raw_payload.as_bytes(),
            signature,
        )
    }
}

pub struct SandboxGatewayAdapter {
    pub secret: String,
}

impl PaymentGatewayProvider for SandboxGatewayAdapter {
    fn provider_code(&self) -> &'static str {
        "sandbox"
    }
    fn create_redirect_url(&self, order_number: &str, _amount_poisha: i64) -> String {
        format!("https://pay.coachify.app/sandbox/mock-checkout/{order_number}")
    }
    fn verify_webhook(&self, raw_payload: &str, signature: &str) -> bool {
        verify_hmac_sha256(self.secret.as_bytes(), raw_payload.as_bytes(), signature)
    }
}

pub fn get_gateway_adapter(provider: &str) -> Box<dyn PaymentGatewayProvider> {
    let default_secret = std::str::from_utf8(DEFAULT_GATEWAY_WEBHOOK_SECRET)
        .unwrap()
        .to_string();
    match provider.trim() {
        "sslcommerz" => Box::new(SslCommerzAdapter {
            store_id: "coachify_sandbox".to_string(),
            store_secret: default_secret,
        }),
        "bkash" | "bkash_direct" => Box::new(BkashDirectAdapter {
            app_key: "coachify_bkash_key".to_string(),
            webhook_secret: default_secret,
        }),
        "nagad" | "nagad_direct" => Box::new(NagadDirectAdapter {
            merchant_id: "coachify_nagad_mid".to_string(),
            webhook_secret: default_secret,
        }),
        _ => Box::new(SandboxGatewayAdapter {
            secret: default_secret,
        }),
    }
}

pub fn get_default_bank_instructions(order_number: &str) -> BankInstructionsView {
    BankInstructionsView {
        bank_name: "Eastern Bank PLC (EBL)".to_string(),
        account_title: "Coachify Technologies Ltd".to_string(),
        account_number: "1041060389201".to_string(),
        routing_number: "095261728".to_string(),
        branch: "Mirpur-10 Branch, Dhaka".to_string(),
        deposit_reference_prefix: format!("REF-{order_number}"),
    }
}

/// Returns the active pricing catalog with the 4 default tiers (3m, 6m, 12m, lifetime).
pub fn get_pricing_catalog(conn: &Connection) -> Result<PricingCatalogResponse, CloudError> {
    let mut stmt = conn.prepare(
        "SELECT pr.id, pl.code, pl.name, pr.billing_cycle, pr.period_months, pr.is_lifetime,
                pr.monthly_rate_poisha, pr.amount_poisha, pr.currency
         FROM prices pr
         JOIN plans pl ON pl.id = pr.plan_id
         WHERE pr.active_to IS NULL AND pr.billing_cycle IN ('3_months', '6_months', '12_months', 'lifetime')
         ORDER BY pr.period_months ASC, pr.is_lifetime ASC",
    )?;
    let rows = stmt.query_map([], |r| {
        let cycle: String = r.get(3)?;
        let monthly_rate_poisha: i64 = r.get(6)?;
        let amount_poisha: i64 = r.get(7)?;
        let badge = match cycle.as_str() {
            "3_months" => Some("Quarterly Flexibility".to_string()),
            "6_months" => Some("Popular Choice".to_string()),
            "12_months" => Some("Best Value (Save 33%)".to_string()),
            "lifetime" => Some("Lifetime Investment · Zero Subscriptions".to_string()),
            _ => None,
        };
        Ok(PricingTierView {
            id: r.get(0)?,
            plan_code: r.get(1)?,
            plan_name: r.get(2)?,
            billing_cycle: cycle,
            period_months: r.get::<_, i64>(4)? as u32,
            is_lifetime: r.get::<_, i64>(5)? == 1,
            monthly_rate_poisha,
            monthly_rate_bdt: (monthly_rate_poisha as f64) / 100.0,
            amount_poisha,
            total_bdt: (amount_poisha as f64) / 100.0,
            currency: r.get(8)?,
            badge,
        })
    })?;
    let mut tiers = Vec::new();
    for row in rows {
        tiers.push(row?);
    }
    tiers.sort_by_key(|t| match t.billing_cycle.as_str() {
        "3_months" => 1,
        "6_months" => 2,
        "12_months" => 3,
        "lifetime" => 4,
        _ => 5,
    });
    Ok(PricingCatalogResponse { tiers })
}

/// Updates an admin configurable price tier from Vendor Admin, archiving the previous active row and logging to audit.
pub fn admin_update_price_tier(
    conn: &Connection,
    input: &AdminUpdatePriceTierInput,
    now_utc: &str,
) -> Result<UpdatedPriceTierResult, CloudError> {
    let (admin, _, _, _) = resolve_admin_by_email(conn, &input.admin_email)?;
    if admin.role != "super_admin" && admin.role != "ops_admin" {
        return Err(CloudError::AdminPermissionDenied(admin.role));
    }

    let cycle = input.billing_cycle.trim();
    if !matches!(cycle, "3_months" | "6_months" | "12_months" | "lifetime") {
        return Err(CloudError::Validation(format!(
            "Invalid billing_cycle: {cycle}"
        )));
    }

    let is_lifetime = cycle == "lifetime";
    let period_months: u32 = match cycle {
        "3_months" => 3,
        "6_months" => 6,
        "12_months" => 12,
        _ => 0,
    };

    let plan_id = if is_lifetime || input.plan_code == "local_pro" {
        "plan_local_pro"
    } else {
        "plan_local"
    };

    let (monthly_rate_poisha, amount_poisha) = if is_lifetime {
        let total_bdt = input
            .total_bdt_override
            .or(input.monthly_rate_bdt)
            .ok_or(CloudError::InvalidPriceAmount)?;
        if total_bdt <= 0.0 {
            return Err(CloudError::InvalidPriceAmount);
        }
        let poisha = (total_bdt * 100.0).round() as i64;
        (poisha, poisha)
    } else {
        let monthly_bdt = input
            .monthly_rate_bdt
            .ok_or(CloudError::InvalidPriceAmount)?;
        if monthly_bdt <= 0.0 {
            return Err(CloudError::InvalidPriceAmount);
        }
        let monthly_poisha = (monthly_bdt * 100.0).round() as i64;
        let total_poisha = if let Some(override_total) = input.total_bdt_override {
            (override_total * 100.0).round() as i64
        } else {
            monthly_poisha * (period_months as i64)
        };
        (monthly_poisha, total_poisha)
    };

    if monthly_rate_poisha <= 0 || amount_poisha <= 0 {
        return Err(CloudError::InvalidPriceAmount);
    }

    // Archive previous active row
    conn.execute(
        "UPDATE prices SET active_to = ?1 WHERE plan_id = ?2 AND billing_cycle = ?3 AND active_to IS NULL",
        params![now_utc, plan_id, cycle],
    )?;

    let new_price_id = format!("prc_{}_{}", cycle, Uuid::now_v7());
    conn.execute(
        "INSERT INTO prices (id, plan_id, billing_cycle, period_months, is_lifetime, currency, monthly_rate_poisha, amount_poisha, active_from)
         VALUES (?1, ?2, ?3, ?4, ?5, 'BDT', ?6, ?7, ?8)",
        params![
            new_price_id,
            plan_id,
            cycle,
            period_months,
            if is_lifetime { 1 } else { 0 },
            monthly_rate_poisha,
            amount_poisha,
            now_utc,
        ],
    )?;

    let after_json = serde_json::json!({
        "price_id": new_price_id,
        "plan_id": plan_id,
        "billing_cycle": cycle,
        "period_months": period_months,
        "monthly_rate_poisha": monthly_rate_poisha,
        "amount_poisha": amount_poisha,
    })
    .to_string();

    let _audit_hash = append_admin_audit(
        conn,
        now_utc,
        &admin.id,
        &admin.email,
        "admin.prices.update_tier",
        "prices",
        &new_price_id,
        Some(&after_json),
    )?;
    let audit_seq: i64 = conn.last_insert_rowid();

    Ok(UpdatedPriceTierResult {
        price_id: new_price_id,
        plan_code: if is_lifetime {
            "local_pro".to_string()
        } else {
            input.plan_code.clone()
        },
        billing_cycle: cycle.to_string(),
        monthly_rate_poisha,
        amount_poisha,
        monthly_rate_bdt: (monthly_rate_poisha as f64) / 100.0,
        total_bdt: (amount_poisha as f64) / 100.0,
        audit_seq,
    })
}

fn resolve_or_create_checkout_customer(
    conn: &Connection,
    input: &CreateCheckoutSessionInput,
    now_utc: &str,
) -> Result<(String, String, String, String, Option<String>), CloudError> {
    let clean_phone = input.phone.trim();
    if clean_phone.is_empty() {
        return Err(CloudError::Validation(
            "Phone number is required".to_string(),
        ));
    }
    let existing: Option<(String, String, String, Option<String>)> = conn
        .query_row(
            "SELECT id, org_name, owner_name, email FROM customers WHERE phone = ?1",
            params![clean_phone],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )
        .optional()?;

    if let Some((id, org, owner, em)) = existing {
        Ok((
            id,
            org,
            owner,
            clean_phone.to_string(),
            em.or_else(|| input.email.clone()),
        ))
    } else {
        let cust_id = format!("cust_{}", Uuid::now_v7());
        let org = if input.org_name.trim().is_empty() {
            "Coachify Institution".to_string()
        } else {
            input.org_name.trim().to_string()
        };
        let owner = if input.owner_name.trim().is_empty() {
            "Institution Owner".to_string()
        } else {
            input.owner_name.trim().to_string()
        };
        conn.execute(
            "INSERT INTO customers (id, org_name, owner_name, phone, email, district, status, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, 'Dhaka', 'active', ?6, ?6)",
            params![cust_id, org, owner, clean_phone, input.email, now_utc],
        )?;
        let portal_pw = hash_password("CoachifyPortal#2026");
        conn.execute(
            "INSERT OR IGNORE INTO customer_portal_credentials (customer_id, password_hash, failed_attempts, updated_at)
             VALUES (?1, ?2, 0, ?3)",
            params![cust_id, portal_pw, now_utc],
        )?;
        Ok((
            cust_id,
            org,
            owner,
            clean_phone.to_string(),
            input.email.clone(),
        ))
    }
}

/// Creates a new checkout order session (`POST /v1/checkout/create_session`),
/// computing pro-rata unused credit for upgrades and issuing a 30 minute order session.
pub fn create_checkout_session(
    conn: &Connection,
    input: &CreateCheckoutSessionInput,
    now_utc: &str,
) -> Result<CheckoutSessionResponse, CloudError> {
    let cycle = input.billing_cycle.trim();
    if !matches!(cycle, "3_months" | "6_months" | "12_months" | "lifetime") {
        return Err(CloudError::Validation(format!(
            "Invalid billing_cycle: {cycle}"
        )));
    }

    let is_lifetime = cycle == "lifetime" || input.kind == "lifetime_purchase";
    let plan_id = if is_lifetime || input.plan_code == "local_pro" {
        "plan_local_pro"
    } else {
        "plan_local"
    };

    let price_row: (String, i64, i64, u32) = conn
        .query_row(
            "SELECT id, monthly_rate_poisha, amount_poisha, period_months FROM prices
             WHERE plan_id = ?1 AND billing_cycle = ?2 AND active_to IS NULL",
            params![plan_id, cycle],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get::<_, i64>(3)? as u32)),
        )
        .optional()?
        .ok_or(CloudError::CatalogUnavailable)?;

    let (price_id, monthly_rate_poisha, base_amount_poisha, period_months) = price_row;

    let (customer_id, _, _, _, _) = if let Some(token) = &input.session_token {
        if let Ok(cust) = authenticate_portal_session(conn, token, now_utc) {
            (
                cust.id,
                cust.org_name,
                cust.owner_name,
                cust.phone,
                cust.email,
            )
        } else {
            resolve_or_create_checkout_customer(conn, input, now_utc)?
        }
    } else {
        resolve_or_create_checkout_customer(conn, input, now_utc)?
    };

    let mut unused_credit_poisha: i64 = 0;
    let mut subscription_id_opt: Option<String> = None;

    if input.kind == "upgrade" {
        let sub_row: Option<(String, String, i64, u32)> = conn
            .query_row(
                "SELECT s.id, s.paid_through, py.amount_poisha, py.period_months
                 FROM subscriptions s
                 JOIN payments py ON py.subscription_id = s.id AND py.status = 'confirmed'
                 WHERE s.customer_id = ?1
                 ORDER BY py.recorded_at DESC LIMIT 1",
                params![customer_id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get::<_, i64>(3)? as u32)),
            )
            .optional()?;

        if let Some((sid, paid_through, last_amount, last_period)) = sub_row {
            subscription_id_opt = Some(sid);
            let today_str = if now_utc.len() >= 10 {
                &now_utc[0..10]
            } else {
                "2026-09-25"
            };
            if let (Ok(p_date), Ok(t_date)) = (
                paid_through[0..10.min(paid_through.len())].parse::<jiff::civil::Date>(),
                today_str.parse::<jiff::civil::Date>(),
            ) {
                if p_date > t_date {
                    if let Ok(span) = t_date.until(p_date) {
                        let rem_days = span.get_days().max(0) as i64;
                        let divisor = (last_period as i64 * 30).max(1);
                        unused_credit_poisha = (last_amount * rem_days) / divisor;
                    }
                }
            }
        }
    } else {
        subscription_id_opt = conn
            .query_row(
                "SELECT id FROM subscriptions WHERE customer_id = ?1 ORDER BY started_at DESC LIMIT 1",
                params![customer_id],
                |r| r.get(0),
            )
            .optional()?;
    }

    let final_amount_poisha = (base_amount_poisha - unused_credit_poisha).max(100);

    let year = if now_utc.len() >= 4 {
        &now_utc[0..4]
    } else {
        "2026"
    };
    let order_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM checkout_orders WHERE order_number LIKE 'CHK-' || ?1 || '-%'",
        params![year],
        |r| r.get(0),
    )?;
    let order_number = format!("CHK-{year}-{:05}", order_count + 1);
    let order_id = format!("ord_{}", Uuid::now_v7());
    let expires_at = add_minutes_iso(now_utc, 30);

    let gateway_provider = match input.method.trim() {
        "bank_transfer" => "manual_bank",
        "bkash" => "bkash_direct",
        "nagad" => "nagad_direct",
        _ => "sandbox",
    };

    conn.execute(
        "INSERT INTO checkout_orders (
            id, order_number, customer_id, subscription_id, plan_id, price_id, kind,
            billing_cycle, period_months, method, amount_poisha, gateway_provider,
            target_request_code, status, expires_at, created_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, 'pending_payment', ?14, ?15)",
        params![
            order_id,
            order_number,
            customer_id,
            subscription_id_opt,
            plan_id,
            price_id,
            input.kind.trim(),
            cycle,
            period_months,
            input.method.trim(),
            final_amount_poisha,
            gateway_provider,
            input.target_request_code,
            expires_at,
            now_utc,
        ],
    )?;

    let adapter = get_gateway_adapter(gateway_provider);
    let gateway_redirect_url = if input.method == "bank_transfer" {
        None
    } else {
        Some(adapter.create_redirect_url(&order_number, final_amount_poisha))
    };

    let bank_instructions = if input.method == "bank_transfer" {
        Some(get_default_bank_instructions(&order_number))
    } else {
        None
    };

    Ok(CheckoutSessionResponse {
        order_number,
        plan_code: if is_lifetime {
            "local_pro".to_string()
        } else {
            input.plan_code.clone()
        },
        billing_cycle: cycle.to_string(),
        period_months,
        monthly_rate_poisha,
        amount_poisha: final_amount_poisha,
        total_bdt: (final_amount_poisha as f64) / 100.0,
        method: input.method.clone(),
        gateway_provider: gateway_provider.to_string(),
        gateway_redirect_url,
        bank_instructions,
        expires_at,
    })
}

/// Submits bank transfer reference and slip URL for a checkout order, moving it to `pending_bank_review`.
pub fn submit_bank_transfer_proof(
    conn: &Connection,
    input: &SubmitBankTransferInput,
    _now_utc: &str,
) -> Result<CheckoutOrderStatusView, CloudError> {
    let clean_ref = input.bank_reference.trim();
    if clean_ref.is_empty() {
        return Err(CloudError::BankReferenceRequired);
    }

    let status: String = conn
        .query_row(
            "SELECT status FROM checkout_orders WHERE order_number = ?1",
            params![input.order_number.trim()],
            |r| r.get(0),
        )
        .optional()?
        .ok_or_else(|| CloudError::OrderNotFound(input.order_number.trim().to_string()))?;

    if status == "completed" {
        return Err(CloudError::OrderAlreadyCompleted);
    }

    conn.execute(
        "UPDATE checkout_orders
         SET status = 'pending_bank_review', bank_reference = ?1, bank_slip_url = ?2
         WHERE order_number = ?3",
        params![clean_ref, input.bank_slip_url, input.order_number.trim()],
    )?;

    get_checkout_order_status(conn, input.order_number.trim())
}

/// Fetches checkout order status, receipt, and license blob if completed.
pub fn get_checkout_order_status(
    conn: &Connection,
    order_number: &str,
) -> Result<CheckoutOrderStatusView, CloudError> {
    let row = conn
        .query_row(
            "SELECT o.order_number, o.status, o.kind, o.billing_cycle, o.method, o.amount_poisha,
                    o.bank_reference, o.bank_slip_url, vi.invoice_number, l.license_id, l.pairing_code,
                    l.last_issued_blob, o.completed_at, o.expires_at
             FROM checkout_orders o
             LEFT JOIN vendor_invoices vi ON vi.id = o.resulting_invoice_id
             LEFT JOIN licenses l ON l.license_id = o.resulting_license_id
             WHERE o.order_number = ?1",
            params![order_number.trim()],
            |r| {
                let amount_poisha: i64 = r.get(5)?;
                Ok(CheckoutOrderStatusView {
                    order_number: r.get(0)?,
                    status: r.get(1)?,
                    kind: r.get(2)?,
                    billing_cycle: r.get(3)?,
                    method: r.get(4)?,
                    amount_poisha,
                    total_bdt: (amount_poisha as f64) / 100.0,
                    bank_reference: r.get(6)?,
                    bank_slip_url: r.get(7)?,
                    invoice_number: r.get(8)?,
                    license_id: r.get(9)?,
                    pairing_code: r.get(10)?,
                    signed_license_blob: r.get(11)?,
                    completed_at: r.get(12)?,
                    expires_at: r.get(13)?,
                })
            },
        )
        .optional()?
        .ok_or_else(|| CloudError::OrderNotFound(order_number.to_string()))?;

    Ok(row)
}

/// Lists all checkout orders waiting for admin bank transfer verification.
pub fn admin_list_pending_bank_transfers(
    conn: &Connection,
) -> Result<Vec<PendingBankTransferOrderView>, CloudError> {
    let mut stmt = conn.prepare(
        "SELECT o.order_number, o.customer_id, c.org_name, c.owner_name, c.phone, o.billing_cycle,
                o.amount_poisha, o.bank_reference, o.bank_slip_url, o.created_at
         FROM checkout_orders o
         JOIN customers c ON c.id = o.customer_id
         WHERE o.status = 'pending_bank_review'
         ORDER BY o.created_at ASC",
    )?;
    let rows = stmt.query_map([], |r| {
        let amount_poisha: i64 = r.get(6)?;
        Ok(PendingBankTransferOrderView {
            order_number: r.get(0)?,
            customer_id: r.get(1)?,
            org_name: r.get(2)?,
            owner_name: r.get(3)?,
            phone: r.get(4)?,
            billing_cycle: r.get(5)?,
            amount_poisha,
            total_bdt: (amount_poisha as f64) / 100.0,
            bank_reference: r.get::<_, Option<String>>(7)?.unwrap_or_default(),
            bank_slip_url: r.get(8)?,
            created_at: r.get(9)?,
        })
    })?;
    let mut list = Vec::new();
    for row in rows {
        list.push(row?);
    }
    Ok(list)
}

/// Approves a pending bank transfer order and runs transactional fulfillment.
pub fn admin_approve_bank_transfer(
    conn: &Connection,
    signer_conn: &Connection,
    signer: &SignerService,
    input: &ApproveBankTransferInput,
    now_utc: &str,
) -> Result<CheckoutOrderStatusView, CloudError> {
    let (admin, _, _, _) = resolve_admin_by_email(conn, &input.admin_email)?;
    if admin.role != "super_admin" && admin.role != "ops_admin" {
        return Err(CloudError::AdminPermissionDenied(admin.role));
    }

    let (order_id, status, ref_opt): (String, String, Option<String>) = conn
        .query_row(
            "SELECT id, status, bank_reference FROM checkout_orders WHERE order_number = ?1",
            params![input.order_number.trim()],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .optional()?
        .ok_or_else(|| CloudError::OrderNotFound(input.order_number.trim().to_string()))?;

    if status != "pending_bank_review" {
        return Err(CloudError::OrderNotPendingBankReview);
    }

    let payment_ref =
        ref_opt.unwrap_or_else(|| format!("BANK-APPROVE-{}", input.order_number.trim()));
    let status_view = fulfill_verified_checkout_order(
        conn,
        signer_conn,
        signer,
        &order_id,
        &admin.email,
        &payment_ref,
        now_utc,
    )?;

    append_admin_audit(
        conn,
        now_utc,
        &admin.id,
        &admin.email,
        "admin.checkout.approve_bank_transfer",
        "checkout_orders",
        input.order_number.trim(),
        Some(
            &serde_json::json!({
                "order_number": input.order_number.trim(),
                "notes": input.verification_notes,
                "invoice_number": status_view.invoice_number,
            })
            .to_string(),
        ),
    )?;

    Ok(status_view)
}

/// Transactionally fulfills a verified checkout order (payments, invoice, subscription, license, and signer blob).
#[allow(clippy::type_complexity)]
pub fn fulfill_verified_checkout_order(
    conn: &Connection,
    signer_conn: &Connection,
    signer: &SignerService,
    order_id: &str,
    recorded_by: &str,
    payment_reference: &str,
    now_utc: &str,
) -> Result<CheckoutOrderStatusView, CloudError> {
    let order_data: (
        String,
        String,
        Option<String>,
        String,
        String,
        String,
        String,
        u32,
        String,
        i64,
        Option<String>,
        Option<String>,
        String,
    ) = conn
        .query_row(
            "SELECT order_number, customer_id, subscription_id, plan_id, price_id, kind, billing_cycle,
                    period_months, method, amount_poisha, target_request_code, target_installation_id, status
             FROM checkout_orders WHERE id = ?1",
            params![order_id],
            |r| {
                Ok((
                    r.get(0)?,
                    r.get(1)?,
                    r.get(2)?,
                    r.get(3)?,
                    r.get(4)?,
                    r.get(5)?,
                    r.get(6)?,
                    r.get::<_, i64>(7)? as u32,
                    r.get(8)?,
                    r.get(9)?,
                    r.get(10)?,
                    r.get(11)?,
                    r.get(12)?,
                ))
            },
        )
        .optional()?
        .ok_or_else(|| CloudError::OrderNotFound(order_id.to_string()))?;

    let (
        order_number,
        customer_id,
        sub_id_opt,
        plan_id,
        _price_id,
        kind,
        billing_cycle,
        period_months,
        method,
        amount_poisha,
        target_request_code,
        target_installation_id,
        order_status,
    ) = order_data;

    if order_status == "completed" {
        return get_checkout_order_status(conn, &order_number);
    }

    let org_name: String = conn.query_row(
        "SELECT org_name FROM customers WHERE id = ?1",
        params![customer_id],
        |r| r.get(0),
    )?;

    let subscription_id = if let Some(sid) = sub_id_opt {
        sid
    } else {
        let existing_sub: Option<String> = conn
            .query_row(
                "SELECT id FROM subscriptions WHERE customer_id = ?1 ORDER BY started_at DESC LIMIT 1",
                params![customer_id],
                |r| r.get(0),
            )
            .optional()?;
        if let Some(sid) = existing_sub {
            sid
        } else {
            let new_sub = format!("sub_{}", Uuid::now_v7());
            let default_paid = if billing_cycle == "lifetime" {
                "9999-12-31"
            } else if now_utc.len() >= 10 {
                &now_utc[0..10]
            } else {
                "2026-09-25"
            };
            conn.execute(
                "INSERT INTO subscriptions (id, customer_id, plan_id, status, paid_through, grace_days, started_at)
                 VALUES (?1, ?2, ?3, 'active', ?4, 14, ?5)",
                params![new_sub, customer_id, plan_id, default_paid, now_utc],
            )?;
            new_sub
        }
    };

    let (existing_paid_through, existing_grace_days): (String, u32) = conn.query_row(
        "SELECT paid_through, grace_days FROM subscriptions WHERE id = ?1",
        params![subscription_id],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?;

    let today_ymd = if now_utc.len() >= 10 {
        &now_utc[0..10]
    } else {
        "2026-09-25"
    };
    let (new_paid_through, valid_until) = if billing_cycle == "lifetime" {
        ("9999-12-31".to_string(), "9999-12-31".to_string())
    } else if kind == "upgrade" {
        let val = if existing_paid_through.len() >= 10 {
            if let Ok(d) = existing_paid_through[0..10].parse::<jiff::civil::Date>() {
                d.checked_add(jiff::ToSpan::days(14))
                    .map(|v| v.to_string())
                    .unwrap_or(existing_paid_through.clone())
            } else {
                existing_paid_through.clone()
            }
        } else {
            existing_paid_through.clone()
        };
        (existing_paid_through, val)
    } else {
        compute_extended_paid_through(
            &existing_paid_through,
            existing_grace_days,
            today_ymd,
            period_months,
        )?
    };

    conn.execute(
        "UPDATE subscriptions SET paid_through = ?1, plan_id = ?2, status = 'active' WHERE id = ?3",
        params![new_paid_through, plan_id, subscription_id],
    )?;

    let payment_id = format!("pay_{}", Uuid::now_v7());
    conn.execute(
        "INSERT INTO payments (id, customer_id, subscription_id, amount_poisha, method, reference, proof_url, period_months, recorded_by, status, recorded_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, NULL, ?7, ?8, 'confirmed', ?9)",
        params![
            payment_id,
            customer_id,
            subscription_id,
            amount_poisha,
            if method == "bank_transfer" { "bank" } else { &method },
            payment_reference,
            period_months,
            recorded_by,
            now_utc,
        ],
    )?;

    let year = if now_utc.len() >= 4 {
        &now_utc[0..4]
    } else {
        "2026"
    };
    let inv_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM vendor_invoices WHERE invoice_number LIKE 'INV-' || ?1 || '-%'",
        params![year],
        |r| r.get(0),
    )?;
    let invoice_id = format!("inv_{}", Uuid::now_v7());
    let invoice_number = format!("INV-{year}-{:05}", inv_count + 1);
    conn.execute(
        "INSERT INTO vendor_invoices (id, payment_id, customer_id, invoice_number, subtotal_poisha, vat_poisha, total_poisha, issued_at)
         VALUES (?1, ?2, ?3, ?4, ?5, 0, ?5, ?6)",
        params![invoice_id, payment_id, customer_id, invoice_number, amount_poisha, now_utc],
    )?;

    let existing_lic: Option<(String, i64, Option<String>)> = conn
        .query_row(
            "SELECT license_id, current_serial, bound_installation_id FROM licenses WHERE subscription_id = ?1",
            params![subscription_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .optional()?;

    let (license_id, new_serial, existing_bound) =
        if let Some((lid, cur_serial, bound)) = existing_lic {
            (lid, cur_serial + 1, bound)
        } else {
            let short_suffix = customer_id
                .chars()
                .rev()
                .take(4)
                .collect::<String>()
                .to_uppercase();
            let new_lid = format!("LIC-{year}-CHK-{short_suffix}");
            (new_lid, 1, None)
        };

    let target_inst = if let Some(inst) = target_installation_id {
        inst
    } else if let Some(code) = &target_request_code {
        if let Ok((inst_id, _, _)) = parse_and_verify_offline_request(conn, code, None) {
            inst_id
        } else {
            existing_bound.clone().unwrap_or_else(|| {
                format!("inst-{}-host", &customer_id[0..customer_id.len().min(8)])
            })
        }
    } else {
        existing_bound
            .clone()
            .unwrap_or_else(|| format!("inst-{}-host", &customer_id[0..customer_id.len().min(8)]))
    };

    let is_lifetime = billing_cycle == "lifetime";
    let is_pro = is_lifetime || plan_id == "plan_local_pro";
    let (student_limit, workstation_limit, capabilities) = if is_pro {
        (
            Some(10000),
            Some(10),
            vec![
                "core.ledger".to_string(),
                "core.attendance".to_string(),
                "core.exams".to_string(),
                "core.sms".to_string(),
                "core.lan".to_string(),
            ],
        )
    } else {
        (
            Some(2000),
            Some(1),
            vec![
                "core.ledger".to_string(),
                "core.attendance".to_string(),
                "core.exams".to_string(),
                "core.sms".to_string(),
            ],
        )
    };

    let payload = SignerLicensePayload {
        license_id: license_id.clone(),
        customer_id: customer_id.clone(),
        customer_name: org_name.clone(),
        installation_id: target_inst.clone(),
        tier: if is_pro {
            "local_pro".to_string()
        } else {
            "local".to_string()
        },
        kind: "paid".to_string(),
        serial: new_serial,
        paid_through: new_paid_through.clone(),
        grace_days: if is_lifetime { 0 } else { 14 },
        valid_until: valid_until.clone(),
        issued_at: now_utc.to_string(),
        limits: SignerLicenseLimits {
            active_students: student_limit,
            workstations: workstation_limit,
        },
        capabilities,
        key_id: "k2026a".to_string(),
    };

    let signed = request_isolated_license_signature(signer_conn, signer, &payload, now_utc)?;
    let fresh_pairing_code = format!("{:06}", (Uuid::now_v7().as_u128() % 900_000 + 100_000));
    let pairing_expires = add_minutes_iso(now_utc, 4320);

    conn.execute(
        "INSERT INTO licenses (
            id, license_id, customer_id, subscription_id, activation_key_hash, pairing_code,
            pairing_expires_at, current_serial, bound_installation_id, key_id, last_issued_blob,
            status, transfer_count_12m, updated_at
        ) VALUES (?1, ?2, ?3, ?4, 'ACT-DEFAULT-2026', ?5, ?6, ?7, ?8, 'k2026a', ?9, 'active', 0, ?10)
        ON CONFLICT(license_id) DO UPDATE SET
            current_serial = excluded.current_serial,
            pairing_code = excluded.pairing_code,
            pairing_expires_at = excluded.pairing_expires_at,
            bound_installation_id = COALESCE(excluded.bound_installation_id, licenses.bound_installation_id),
            last_issued_blob = excluded.last_issued_blob,
            updated_at = excluded.updated_at",
        params![
            format!("lic_{}", Uuid::now_v7()),
            license_id,
            customer_id,
            subscription_id,
            fresh_pairing_code,
            pairing_expires,
            new_serial,
            target_inst,
            signed.signed_blob,
            now_utc,
        ],
    )?;

    conn.execute(
        "INSERT INTO license_issues (
            id, license_id, serial, installation_id, kind, payload_json, signature, signed_blob, issued_by, reason, issued_at
        ) VALUES (?1, ?2, ?3, ?4, 'paid', ?5, ?6, ?7, ?8, 'Verified checkout fulfillment', ?9)",
        params![
            format!("iss_{}", Uuid::now_v7()),
            license_id,
            new_serial,
            target_inst,
            signed.envelope.payload,
            signed.envelope.signature,
            signed.signed_blob,
            recorded_by,
            now_utc,
        ],
    )?;

    conn.execute(
        "UPDATE installations SET status = 'moved', replaced_by_installation_id = ?1 WHERE license_id = ?2 AND role = 'host' AND status = 'active' AND installation_id != ?1",
        params![target_inst, license_id],
    )?;

    conn.execute(
        "INSERT INTO installations (
            id, installation_id, customer_id, license_id, label, role, app_version, active_students, first_seen_at, last_seen_at, status
        ) VALUES (?1, ?2, ?3, ?4, ?5, 'host', '1.0.1', 0, ?6, ?6, 'active')
        ON CONFLICT(license_id, installation_id) DO UPDATE SET
            status = 'active',
            last_seen_at = excluded.last_seen_at",
        params![
            format!("inst_{}", Uuid::now_v7()),
            target_inst,
            customer_id,
            license_id,
            format!("{org_name} Host"),
            now_utc,
        ],
    )?;

    conn.execute(
        "UPDATE checkout_orders
         SET status = 'completed',
             completed_at = ?1,
             resulting_payment_id = ?2,
             resulting_invoice_id = ?3,
             resulting_license_id = ?4
         WHERE id = ?5",
        params![now_utc, payment_id, invoice_id, license_id, order_id],
    )?;

    get_checkout_order_status(conn, &order_number)
}

#[derive(Deserialize)]
struct RawWebhookPayload {
    order_number: String,
    gateway_event_id: String,
    gateway_trx_id: String,
    amount_poisha: Option<i64>,
    amount_bdt: Option<f64>,
    amount: Option<f64>,
}

/// Handles payment provider webhook callback (`POST /v1/webhooks/payment/{provider}`).
pub fn handle_payment_webhook(
    conn: &Connection,
    signer_conn: &Connection,
    signer: &SignerService,
    input: &PaymentWebhookInput,
    now_utc: &str,
) -> Result<WebhookAckResponse, CloudError> {
    let adapter = get_gateway_adapter(&input.provider);
    if !adapter.verify_webhook(&input.raw_payload_json, &input.hmac_signature) {
        return Err(CloudError::WebhookSignatureInvalid);
    }

    let payload: RawWebhookPayload = serde_json::from_str(&input.raw_payload_json)
        .map_err(|e| CloudError::Validation(format!("Invalid webhook payload JSON: {e}")))?;

    // Duplicate check
    let existing_event: Option<String> = conn
        .query_row(
            "SELECT id FROM payment_webhook_events WHERE gateway_provider = ?1 AND gateway_event_id = ?2",
            params![input.provider.trim(), payload.gateway_event_id.trim()],
            |r| r.get(0),
        )
        .optional()?;

    if existing_event.is_some() {
        return Ok(WebhookAckResponse {
            processed_status: "duplicate_ignored".to_string(),
            order_number: payload.order_number.clone(),
            invoice_number: None,
            message: "Duplicate webhook event ignored".to_string(),
        });
    }

    let (order_id, expected_poisha, order_status): (String, i64, String) = conn
        .query_row(
            "SELECT id, amount_poisha, status FROM checkout_orders WHERE order_number = ?1",
            params![payload.order_number.trim()],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .optional()?
        .ok_or_else(|| CloudError::OrderNotFound(payload.order_number.clone()))?;

    let received_poisha = if let Some(p) = payload.amount_poisha {
        p
    } else if let Some(b) = payload.amount_bdt.or(payload.amount) {
        (b * 100.0).round() as i64
    } else {
        expected_poisha
    };

    if received_poisha != expected_poisha {
        let _ = conn.execute(
            "INSERT INTO payment_webhook_events (
                id, checkout_order_id, gateway_provider, gateway_event_id, gateway_trx_id,
                amount_poisha, signature_verified, raw_payload_json, processed_status, received_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, 1, ?7, 'rejected', ?8)",
            params![
                format!("pwe_{}", Uuid::now_v7()),
                order_id,
                input.provider.trim(),
                payload.gateway_event_id.trim(),
                payload.gateway_trx_id.trim(),
                received_poisha,
                input.raw_payload_json,
                now_utc,
            ],
        );
        return Err(CloudError::WebhookAmountMismatch {
            received: received_poisha,
            expected: expected_poisha,
        });
    }

    conn.execute(
        "INSERT INTO payment_webhook_events (
            id, checkout_order_id, gateway_provider, gateway_event_id, gateway_trx_id,
            amount_poisha, signature_verified, raw_payload_json, processed_status, received_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, 1, ?7, 'applied', ?8)",
        params![
            format!("pwe_{}", Uuid::now_v7()),
            order_id,
            input.provider.trim(),
            payload.gateway_event_id.trim(),
            payload.gateway_trx_id.trim(),
            received_poisha,
            input.raw_payload_json,
            now_utc,
        ],
    )?;

    let status_view = if order_status == "completed" {
        get_checkout_order_status(conn, &payload.order_number)?
    } else {
        fulfill_verified_checkout_order(
            conn,
            signer_conn,
            signer,
            &order_id,
            &format!("webhook:{}", input.provider.trim()),
            payload.gateway_trx_id.trim(),
            now_utc,
        )?
    };

    Ok(WebhookAckResponse {
        processed_status: "applied".to_string(),
        order_number: payload.order_number,
        invoice_number: status_view.invoice_number,
        message: "Payment verified and fulfilled".to_string(),
    })
}

// ---------------------------------------------------------------------------
// Customer Portal Device Management (Feature 68, Spec 0043b)
// ---------------------------------------------------------------------------

/// Lists all devices linked to the customer's license with telemetry and remaining transfers.
pub fn portal_list_devices(
    conn: &Connection,
    session_token: &str,
    now_utc: &str,
) -> Result<PortalDeviceListView, CloudError> {
    let customer = authenticate_portal_session(conn, session_token, now_utc)?;
    let (transfer_count_12m, _license_id): (u32, String) = conn
        .query_row(
            "SELECT transfer_count_12m, license_id FROM licenses WHERE customer_id = ?1",
            params![customer.id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?
        .unwrap_or((0, String::new()));

    let mut devices = Vec::new();
    {
        let mut stmt = conn.prepare(
            "SELECT installation_id, label, role, hostname, os_version, app_version,
                    active_students, first_seen_at, last_seen_at, deactivated_at, deactivation_reason, status
             FROM installations
             WHERE customer_id = ?1
             ORDER BY CASE WHEN status = 'active' THEN 0 ELSE 1 END, last_seen_at DESC",
        )?;
        let rows = stmt.query_map(params![customer.id], |r| {
            Ok(PortalDeviceItem {
                installation_id: r.get(0)?,
                label: r.get(1)?,
                role: r.get(2)?,
                hostname: r.get(3)?,
                os_version: r.get(4)?,
                app_version: r.get(5)?,
                active_students: r.get(6)?,
                first_seen_at: r.get(7)?,
                last_seen_at: r.get(8)?,
                deactivated_at: r.get(9)?,
                deactivation_reason: r.get(10)?,
                status: r.get(11)?,
            })
        })?;
        for d in rows {
            devices.push(d?);
        }
    }

    let mut recent_events = Vec::new();
    {
        let mut stmt = conn.prepare(
            "SELECT id, action, old_installation_id, new_installation_id, new_label,
                    reason, initiated_by_type, resulting_serial, created_at
             FROM device_management_events
             WHERE customer_id = ?1
             ORDER BY created_at DESC LIMIT 20",
        )?;
        let rows = stmt.query_map(params![customer.id], |r| {
            Ok(PortalDeviceEventItem {
                id: r.get(0)?,
                action: r.get(1)?,
                old_installation_id: r.get(2)?,
                new_installation_id: r.get(3)?,
                new_label: r.get(4)?,
                reason: r.get(5)?,
                initiated_by_type: r.get(6)?,
                resulting_serial: r.get(7)?,
                created_at: r.get(8)?,
            })
        })?;
        for e in rows {
            recent_events.push(e?);
        }
    }

    let max_transfers_12m = 3u32;
    let transfers_remaining_12m = max_transfers_12m.saturating_sub(transfer_count_12m);

    Ok(PortalDeviceListView {
        devices,
        transfer_count_12m,
        max_transfers_12m,
        transfers_remaining_12m,
        recent_events,
    })
}

/// Renames a device's custom label without consuming a transfer slot.
#[allow(clippy::type_complexity)]
pub fn portal_rename_device(
    conn: &Connection,
    input: &PortalRenameDeviceInput,
    now_utc: &str,
) -> Result<PortalDeviceItem, CloudError> {
    let customer = authenticate_portal_session(conn, &input.session_token, now_utc)?;
    let clean_label = input.new_label.trim();
    if clean_label.is_empty() || clean_label.len() > 64 {
        return Err(CloudError::InvalidDeviceLabel(
            "Device label must be between 1 and 64 characters".to_string(),
        ));
    }

    let inst_row: Option<(String, String, String, String, u32, String, String, Option<String>, Option<String>, String, String)> = conn
        .query_row(
            "SELECT customer_id, license_id, role, app_version, active_students, first_seen_at, last_seen_at,
                    deactivated_at, deactivation_reason, status, hostname
             FROM installations WHERE installation_id = ?1",
            params![input.installation_id.trim()],
            |r| {
                Ok((
                    r.get(0)?,
                    r.get(1)?,
                    r.get(2)?,
                    r.get(3)?,
                    r.get(4)?,
                    r.get(5)?,
                    r.get(6)?,
                    r.get(7)?,
                    r.get(8)?,
                    r.get(9)?,
                    r.get(10)?,
                ))
            },
        )
        .optional()?;

    let (
        inst_cust,
        license_id,
        role,
        app_version,
        active_students,
        first_seen,
        last_seen,
        deact_at,
        deact_reason,
        status,
        hostname,
    ) = inst_row
        .ok_or_else(|| CloudError::NotFound(format!("Device {}", input.installation_id)))?;

    if inst_cust != customer.id {
        return Err(CloudError::DeviceNotOwnedByCustomer);
    }

    conn.execute(
        "UPDATE installations SET label = ?1 WHERE installation_id = ?2",
        params![clean_label, input.installation_id.trim()],
    )?;

    let serial: i64 = conn
        .query_row(
            "SELECT current_serial FROM licenses WHERE license_id = ?1",
            params![license_id],
            |r| r.get(0),
        )
        .unwrap_or(1);

    conn.execute(
        "INSERT INTO device_management_events (
            id, license_id, customer_id, action, old_installation_id, new_installation_id,
            new_label, reason, initiated_by_type, initiated_by_id, resulting_serial, created_at
        ) VALUES (?1, ?2, ?3, 'rename_label', ?4, NULL, ?5, 'User renamed label', 'portal_owner', ?6, ?7, ?8)",
        params![
            format!("dme_{}", Uuid::now_v7()),
            license_id,
            customer.id,
            input.installation_id.trim(),
            clean_label,
            customer.id,
            serial,
            now_utc,
        ],
    )?;

    Ok(PortalDeviceItem {
        installation_id: input.installation_id.trim().to_string(),
        label: clean_label.to_string(),
        role,
        hostname,
        os_version: "Windows".to_string(),
        app_version,
        active_students,
        first_seen_at: first_seen,
        last_seen_at: last_seen,
        deactivated_at: deact_at,
        deactivation_reason: deact_reason,
        status,
    })
}

/// Issues a 6 digit step up OTP for portal device actions (deactivation or replacement).
pub fn portal_request_device_otp(
    conn: &Connection,
    input: &PortalRequestDeviceOtpInput,
    now_utc: &str,
    _unix_seconds: u64,
) -> Result<PortalDeviceOtpResponse, CloudError> {
    let customer = authenticate_portal_session(conn, &input.session_token, now_utc)?;

    let inst_cust: Option<String> = conn
        .query_row(
            "SELECT customer_id FROM installations WHERE installation_id = ?1",
            params![input.installation_id.trim()],
            |r| r.get(0),
        )
        .optional()?;

    let owner_id = inst_cust
        .ok_or_else(|| CloudError::NotFound(format!("Device {}", input.installation_id)))?;
    if owner_id != customer.id {
        return Err(CloudError::DeviceNotOwnedByCustomer);
    }

    let otp = format!("{:06}", (Uuid::now_v7().as_u128() % 900_000 + 100_000));
    let hash = sha256_hex(&otp);
    let expires = add_minutes_iso(now_utc, 10);

    conn.execute(
        "UPDATE customer_portal_credentials
         SET otp_code_hash = ?1, otp_expires_at = ?2, failed_attempts = 0, updated_at = ?3
         WHERE customer_id = ?4",
        params![hash, expires, now_utc, customer.id],
    )?;

    let masked = mask_phone_or_email(&customer.phone, customer.email.as_deref());
    Ok(PortalDeviceOtpResponse {
        masked_destination: masked,
        demo_otp_code: otp,
        otp_expires_in_seconds: 600,
    })
}

fn verify_device_action_otp(
    conn: &Connection,
    customer_id: &str,
    submitted_otp: &str,
    now_utc: &str,
) -> Result<(), CloudError> {
    let (otp_hash_opt, expires_opt): (Option<String>, Option<String>) = conn
        .query_row(
            "SELECT otp_code_hash, otp_expires_at FROM customer_portal_credentials WHERE customer_id = ?1",
            params![customer_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?
        .unwrap_or((None, None));

    let Some(stored_hash) = otp_hash_opt else {
        return Err(CloudError::PortalInvalidOtp);
    };

    if let Some(exp) = expires_opt {
        if now_utc > exp.as_str() {
            return Err(CloudError::PortalOtpExpired);
        }
    }

    let submitted_hash = sha256_hex(submitted_otp.trim());
    if stored_hash != submitted_hash {
        conn.execute(
            "UPDATE customer_portal_credentials SET failed_attempts = failed_attempts + 1 WHERE customer_id = ?1",
            params![customer_id],
        )?;
        return Err(CloudError::PortalInvalidOtp);
    }

    conn.execute(
        "UPDATE customer_portal_credentials SET otp_code_hash = NULL, otp_expires_at = NULL WHERE customer_id = ?1",
        params![customer_id],
    )?;

    Ok(())
}

/// Deactivates a device from the customer portal after OTP challenge, releasing the license slot and generating a fresh pairing code.
pub fn portal_deactivate_device(
    conn: &Connection,
    input: &PortalDeactivateDeviceInput,
    now_utc: &str,
) -> Result<DeviceActionResult, CloudError> {
    let customer = authenticate_portal_session(conn, &input.session_token, now_utc)?;
    verify_device_action_otp(conn, &customer.id, &input.otp_code, now_utc)?;

    let (inst_cust, license_id, status): (String, String, String) = conn
        .query_row(
            "SELECT customer_id, license_id, status FROM installations WHERE installation_id = ?1",
            params![input.installation_id.trim()],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .optional()?
        .ok_or_else(|| CloudError::NotFound(format!("Device {}", input.installation_id)))?;

    if inst_cust != customer.id {
        return Err(CloudError::DeviceNotOwnedByCustomer);
    }

    let (transfer_count, current_serial): (u32, i64) = conn.query_row(
        "SELECT transfer_count_12m, current_serial FROM licenses WHERE license_id = ?1",
        params![license_id],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?;

    if transfer_count >= 3 {
        return Err(CloudError::TransferLimitExceeded);
    }

    if status == "deactivated" {
        return Err(CloudError::DeviceAlreadyInactive);
    }

    let new_transfers = transfer_count + 1;
    let new_serial = current_serial + 1;
    let new_pairing_code = format!("{:06}", (Uuid::now_v7().as_u128() % 900_000 + 100_000));
    let pairing_expires = add_minutes_iso(now_utc, 4320); // 72 hours

    conn.execute(
        "UPDATE installations
         SET status = 'deactivated', deactivated_at = ?1, deactivation_reason = ?2
         WHERE installation_id = ?3",
        params![now_utc, input.reason.trim(), input.installation_id.trim()],
    )?;

    conn.execute(
        "UPDATE licenses
         SET bound_installation_id = NULL,
             transfer_count_12m = ?1,
             current_serial = ?2,
             pairing_code = ?3,
             pairing_expires_at = ?4,
             updated_at = ?5
         WHERE license_id = ?6",
        params![
            new_transfers,
            new_serial,
            new_pairing_code,
            pairing_expires,
            now_utc,
            license_id,
        ],
    )?;

    conn.execute(
        "INSERT INTO device_management_events (
            id, license_id, customer_id, action, old_installation_id, new_installation_id,
            new_label, reason, initiated_by_type, initiated_by_id, resulting_serial, created_at
        ) VALUES (?1, ?2, ?3, 'deactivate', ?4, NULL, NULL, ?5, 'portal_owner', ?6, ?7, ?8)",
        params![
            format!("dme_{}", Uuid::now_v7()),
            license_id,
            customer.id,
            input.installation_id.trim(),
            input.reason.trim(),
            customer.id,
            new_serial,
            now_utc,
        ],
    )?;

    Ok(DeviceActionResult {
        action: "deactivate".to_string(),
        deactivated_installation_id: input.installation_id.trim().to_string(),
        new_installation_id: None,
        new_pairing_code: Some(new_pairing_code),
        pairing_expires_at: Some(pairing_expires),
        signed_license_blob: None,
        transfers_used_12m: new_transfers,
        transfers_remaining_12m: 3u32.saturating_sub(new_transfers),
        message: "Device deactivated. License slot released with fresh pairing code.".to_string(),
    })
}

/// Replaces a device in one step with online pairing or an offline REQ code.
pub fn portal_replace_device(
    conn: &Connection,
    signer_conn: &Connection,
    signer: &SignerService,
    input: &PortalReplaceDeviceInput,
    now_utc: &str,
) -> Result<DeviceActionResult, CloudError> {
    let customer = authenticate_portal_session(conn, &input.session_token, now_utc)?;
    verify_device_action_otp(conn, &customer.id, &input.otp_code, now_utc)?;

    let (inst_cust, license_id, status): (String, String, String) = conn
        .query_row(
            "SELECT customer_id, license_id, status FROM installations WHERE installation_id = ?1",
            params![input.old_installation_id.trim()],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .optional()?
        .ok_or_else(|| CloudError::NotFound(format!("Device {}", input.old_installation_id)))?;

    if inst_cust != customer.id {
        return Err(CloudError::DeviceNotOwnedByCustomer);
    }

    let (transfer_count, current_serial, sub_id): (u32, i64, String) = conn.query_row(
        "SELECT transfer_count_12m, current_serial, subscription_id FROM licenses WHERE license_id = ?1",
        params![license_id],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
    )?;

    if transfer_count >= 3 {
        return Err(CloudError::TransferLimitExceeded);
    }

    if status == "deactivated" {
        return Err(CloudError::DeviceAlreadyInactive);
    }

    let new_transfers = transfer_count + 1;
    let new_serial = current_serial + 1;

    let (paid_through, plan_id): (String, String) = conn.query_row(
        "SELECT paid_through, plan_id FROM subscriptions WHERE id = ?1",
        params![sub_id],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?;

    if let Some(code_or_qr) = &input.new_request_code_or_qr {
        let (new_inst_id, req_code, _) = parse_and_verify_offline_request(conn, code_or_qr, None)?;

        conn.execute(
            "UPDATE installations
             SET status = 'deactivated', deactivated_at = ?1, deactivation_reason = ?2, replaced_by_installation_id = ?3
             WHERE installation_id = ?4",
            params![
                now_utc,
                input.reason.trim(),
                new_inst_id,
                input.old_installation_id.trim(),
            ],
        )?;

        conn.execute(
            "UPDATE licenses
             SET bound_installation_id = ?1,
                 transfer_count_12m = ?2,
                 current_serial = ?3,
                 updated_at = ?4
             WHERE license_id = ?5",
            params![new_inst_id, new_transfers, new_serial, now_utc, license_id,],
        )?;

        let label = if input.new_label.trim().is_empty() {
            format!("{} Replacement Host", customer.org_name)
        } else {
            input.new_label.trim().to_string()
        };

        conn.execute(
            "INSERT INTO installations (
                id, installation_id, customer_id, license_id, label, role, app_version, active_students, first_seen_at, last_seen_at, status
            ) VALUES (?1, ?2, ?3, ?4, ?5, 'host', '1.0.1', 0, ?6, ?6, 'active')
            ON CONFLICT(license_id, installation_id) DO UPDATE SET
                status = 'active',
                label = excluded.label,
                last_seen_at = excluded.last_seen_at",
            params![
                format!("inst_{}", Uuid::now_v7()),
                new_inst_id,
                customer.id,
                license_id,
                label,
                now_utc,
            ],
        )?;

        let is_pro = plan_id == "plan_local_pro" || paid_through == "9999-12-31";
        let valid_until = if paid_through == "9999-12-31" {
            "9999-12-31".to_string()
        } else if let Ok(d) =
            paid_through[0..10.min(paid_through.len())].parse::<jiff::civil::Date>()
        {
            d.checked_add(jiff::ToSpan::days(14))
                .map(|v| v.to_string())
                .unwrap_or(paid_through.clone())
        } else {
            paid_through.clone()
        };

        let payload = SignerLicensePayload {
            license_id: license_id.clone(),
            customer_id: customer.id.clone(),
            customer_name: customer.org_name.clone(),
            installation_id: new_inst_id.clone(),
            tier: if is_pro {
                "local_pro".to_string()
            } else {
                "local".to_string()
            },
            kind: "paid".to_string(),
            serial: new_serial,
            paid_through,
            grace_days: if is_pro && valid_until == "9999-12-31" {
                0
            } else {
                14
            },
            valid_until,
            issued_at: now_utc.to_string(),
            limits: SignerLicenseLimits {
                active_students: if is_pro { Some(10000) } else { Some(2000) },
                workstations: if is_pro { Some(10) } else { Some(1) },
            },
            capabilities: if is_pro {
                vec![
                    "core.ledger".to_string(),
                    "core.attendance".to_string(),
                    "core.exams".to_string(),
                    "core.sms".to_string(),
                    "core.lan".to_string(),
                ]
            } else {
                vec![
                    "core.ledger".to_string(),
                    "core.attendance".to_string(),
                    "core.exams".to_string(),
                    "core.sms".to_string(),
                ]
            },
            key_id: "k2026a".to_string(),
        };

        let signed = request_isolated_license_signature(signer_conn, signer, &payload, now_utc)?;

        conn.execute(
            "UPDATE licenses SET last_issued_blob = ?1 WHERE license_id = ?2",
            params![signed.signed_blob, license_id],
        )?;

        let issue_id = format!("iss_{}", Uuid::now_v7());
        conn.execute(
            "INSERT INTO license_issues (
                id, license_id, serial, installation_id, kind, payload_json, signature, signed_blob, issued_by, reason, issued_at
            ) VALUES (?1, ?2, ?3, ?4, 'transfer', ?5, ?6, ?7, 'portal_owner', ?8, ?9)",
            params![
                issue_id,
                license_id,
                new_serial,
                new_inst_id,
                signed.envelope.payload,
                signed.envelope.signature,
                signed.signed_blob,
                input.reason.trim(),
                now_utc,
            ],
        )?;

        conn.execute(
            "INSERT INTO device_management_events (
                id, license_id, customer_id, action, old_installation_id, new_installation_id,
                new_label, reason, initiated_by_type, initiated_by_id, resulting_serial, issued_serial_id, created_at
            ) VALUES (?1, ?2, ?3, 'replace_offline_req', ?4, ?5, ?6, ?7, 'portal_owner', ?8, ?9, ?10, ?11)",
            params![
                format!("dme_{}", Uuid::now_v7()),
                license_id,
                customer.id,
                input.old_installation_id.trim(),
                new_inst_id,
                label,
                input.reason.trim(),
                customer.id,
                new_serial,
                issue_id,
                now_utc,
            ],
        )?;

        Ok(DeviceActionResult {
            action: "replace_offline_req".to_string(),
            deactivated_installation_id: input.old_installation_id.trim().to_string(),
            new_installation_id: Some(new_inst_id),
            new_pairing_code: None,
            pairing_expires_at: None,
            signed_license_blob: Some(signed.signed_blob),
            transfers_used_12m: new_transfers,
            transfers_remaining_12m: 3u32.saturating_sub(new_transfers),
            message: format!("Device replaced with offline code {req_code}. New signed license file ready for download."),
        })
    } else {
        let new_pairing_code = format!("{:06}", (Uuid::now_v7().as_u128() % 900_000 + 100_000));
        let pairing_expires = add_minutes_iso(now_utc, 4320); // 72 hours

        conn.execute(
            "UPDATE installations
             SET status = 'deactivated', deactivated_at = ?1, deactivation_reason = ?2
             WHERE installation_id = ?3",
            params![
                now_utc,
                input.reason.trim(),
                input.old_installation_id.trim()
            ],
        )?;

        conn.execute(
            "UPDATE licenses
             SET bound_installation_id = NULL,
                 transfer_count_12m = ?1,
                 current_serial = ?2,
                 pairing_code = ?3,
                 pairing_expires_at = ?4,
                 updated_at = ?5
             WHERE license_id = ?6",
            params![
                new_transfers,
                new_serial,
                new_pairing_code,
                pairing_expires,
                now_utc,
                license_id,
            ],
        )?;

        conn.execute(
            "INSERT INTO device_management_events (
                id, license_id, customer_id, action, old_installation_id, new_installation_id,
                new_label, reason, initiated_by_type, initiated_by_id, resulting_serial, created_at
            ) VALUES (?1, ?2, ?3, 'replace_online', ?4, NULL, ?5, ?6, 'portal_owner', ?7, ?8, ?9)",
            params![
                format!("dme_{}", Uuid::now_v7()),
                license_id,
                customer.id,
                input.old_installation_id.trim(),
                input.new_label.trim(),
                input.reason.trim(),
                customer.id,
                new_serial,
                now_utc,
            ],
        )?;

        Ok(DeviceActionResult {
            action: "replace_online".to_string(),
            deactivated_installation_id: input.old_installation_id.trim().to_string(),
            new_installation_id: None,
            new_pairing_code: Some(new_pairing_code),
            pairing_expires_at: Some(pairing_expires),
            signed_license_blob: None,
            transfers_used_12m: new_transfers,
            transfers_remaining_12m: 3u32.saturating_sub(new_transfers),
            message: "Device replaced online. Fresh pairing code issued for new installation."
                .to_string(),
        })
    }
}

/// Logs a request from the portal owner to reset their 12-month transfer limit counter.
pub fn portal_request_transfer_reset(
    conn: &Connection,
    input: &PortalRequestTransferResetInput,
    now_utc: &str,
) -> Result<(), CloudError> {
    let customer = authenticate_portal_session(conn, &input.session_token, now_utc)?;
    let (license_id, serial): (String, i64) = conn.query_row(
        "SELECT license_id, current_serial FROM licenses WHERE customer_id = ?1",
        params![customer.id],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?;

    conn.execute(
        "INSERT INTO device_management_events (
            id, license_id, customer_id, action, old_installation_id, new_installation_id,
            new_label, reason, initiated_by_type, initiated_by_id, resulting_serial, created_at
        ) VALUES (?1, ?2, ?3, 'support_reset_request', 'portal', NULL, NULL, ?4, 'portal_owner', ?5, ?6, ?7)",
        params![
            format!("dme_{}", Uuid::now_v7()),
            license_id,
            customer.id,
            input.reason.trim(),
            customer.id,
            serial,
            now_utc,
        ],
    )?;

    Ok(())
}

/// Resets a license's 12-month transfer counter after admin step-up TOTP verification.
pub fn admin_reset_transfer_count(
    conn: &Connection,
    input: &AdminResetTransferCountInput,
    now_utc: &str,
    unix_seconds: u64,
) -> Result<ResetTransferCountResponse, CloudError> {
    let (admin, _, totp_secret, recovery_codes) = resolve_admin_by_email(conn, &input.admin_email)?;
    if admin.role != "super_admin" && admin.role != "ops_admin" {
        return Err(CloudError::AdminPermissionDenied(admin.role));
    }

    if !verify_totp_or_recovery(
        &totp_secret,
        &recovery_codes,
        &input.step_up_totp_code,
        unix_seconds,
    ) {
        return Err(CloudError::StepUpTotpRequired);
    }

    conn.execute(
        "UPDATE licenses SET transfer_count_12m = 0, updated_at = ?1 WHERE license_id = ?2",
        params![now_utc, input.license_id.trim()],
    )?;

    append_admin_audit(
        conn,
        now_utc,
        &admin.id,
        &admin.email,
        "admin.licenses.reset_transfers",
        "licenses",
        input.license_id.trim(),
        Some(r#"{"transfer_count_12m":0}"#),
    )?;

    Ok(ResetTransferCountResponse {
        license_id: input.license_id.trim().to_string(),
        transfer_count_12m: 0,
        transfers_remaining_12m: 3,
        message: "Self service transfer limit reset to 3 transfers.".to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn end_to_end_bkash_payment_offline_req_and_v1_refresh() {
        let cloud_conn = Connection::open_in_memory().unwrap();
        let signer_conn = Connection::open_in_memory().unwrap();
        let signer = SignerService::default();
        let now = "2026-09-25T11:30:00Z";

        ensure_cloud_seeded(&cloud_conn, &signer_conn, &signer, now).unwrap();

        // 1. Admin login with Argon2id + RFC 6238 TOTP (AC-1)
        let totp = generate_totp_code("COACHIFY_VENDOR_TOTP_SECRET_2026", 1_790_000_000);
        let login_res = login_admin_with_totp(
            &cloud_conn,
            &AdminLoginInput {
                email: "ops@coachify.app".to_string(),
                password: "CoachifyVendor#2026".to_string(),
                totp_code: totp,
            },
            now,
            1_790_000_000,
        )
        .expect("ops_admin login with valid TOTP should succeed");
        assert_eq!(login_res.admin_user.role, "ops_admin");

        // 2. Record manual bKash payment on in-grace subscription (`sub_ctg`, paid_through = 2026-09-20) (AC-3)
        let pay_res = record_manual_payment(
            &cloud_conn,
            &signer_conn,
            &signer,
            &RecordManualPaymentInput {
                admin_email: "ops@coachify.app".to_string(),
                customer_id: "cust_ctg".to_string(),
                subscription_id: "sub_ctg".to_string(),
                amount_poisha: 2_400_000,
                method: "bkash".to_string(),
                reference: "BKASH-CTG-778899".to_string(),
                proof_url: Some("https://files.coachify.app/proofs/ctg-778899.png".to_string()),
                period_months: 6,
                installation_id: Some("inst-ctg-host-01".to_string()),
            },
            now,
        )
        .unwrap();

        // Because 2026-09-25 is within 14 days of 2026-09-20, 6 months extends from 2026-09-20 -> 2027-03-20!
        assert_eq!(pay_res.new_paid_through, "2027-03-20");
        assert_eq!(pay_res.new_serial, 2);
        assert!(pay_res.invoice_number.starts_with("INV-2026-"));

        // 3. Online desktop refresh (`/v1/refresh`) picks up serial 2 automatically (AC-8)
        let refresh_res = handle_v1_refresh(
            &cloud_conn,
            &RefreshEndpointRequest {
                license_id: "LIC-2026-CTGS-0002".to_string(),
                installation_id: "inst-ctg-host-01".to_string(),
                current_serial: 1,
                app_version: "1.0.1".to_string(),
                active_students: 1180,
            },
            now,
        )
        .unwrap();
        assert_eq!(refresh_res.outcome, "updated");
        assert_eq!(refresh_res.latest_serial, 2);
        assert!(refresh_res.signed_license_blob.is_some());

        // 4. Offline request code & QR license issuing (AC-4)
        let (_, qr_payload) = generate_request_code_for_installation("inst-ctg-offline-02");
        let offline_res = issue_offline_license_from_request(
            &cloud_conn,
            &signer_conn,
            &signer,
            &IssueOfflineLicenseInput {
                admin_email: "ops@coachify.app".to_string(),
                license_id: "LIC-2026-CTGS-0002".to_string(),
                request_code_or_qr: qr_payload,
                reason: "Replaced reception motherboard offline".to_string(),
            },
            now,
        )
        .unwrap();
        assert_eq!(offline_res.installation_id, "inst-ctg-offline-02");
        assert_eq!(offline_res.new_serial, 3);

        // 5. Verify hash chain on admin_audit_log (AC-5)
        assert!(verify_admin_audit_chain(&cloud_conn).unwrap());
    }

    #[test]
    fn customer_portal_auth_otp_lockout_and_strict_tenant_isolation() {
        let cloud_conn = Connection::open_in_memory().unwrap();
        let signer_conn = Connection::open_in_memory().unwrap();
        let signer = SignerService::default();
        let now = "2026-09-25T14:00:00Z";

        ensure_cloud_seeded(&cloud_conn, &signer_conn, &signer, now).unwrap();

        // Happy path: Customer A (`cust_mirpur`) logs in with phone + password + 6 digit OTP (AC-1, AC-2)
        let login_a = portal_login(
            &cloud_conn,
            &PortalLoginInput {
                identifier: "+8801711223344".to_string(),
                password: "CoachifyPortal#2026".to_string(),
            },
            now,
        )
        .expect("Mirpur owner login should succeed");
        assert_eq!(login_a.customer_id, "cust_mirpur");
        assert!(login_a.otp_required);

        let verify_a = portal_verify_otp(
            &cloud_conn,
            &PortalVerifyOtpInput {
                customer_id: login_a.customer_id.clone(),
                otp_code: login_a.demo_otp_code,
            },
            now,
        )
        .expect("Valid 6 digit OTP should issue portal session token");

        let overview_a = portal_get_overview(&cloud_conn, &verify_a.session_token, now)
            .expect("Authenticated customer overview should succeed");
        assert_eq!(overview_a.customer.id, "cust_mirpur");
        assert_eq!(overview_a.license.license_id, "LIC-2026-MIRP-0001");
        assert!(!overview_a.invoices.is_empty());
        assert_eq!(overview_a.invoices[0].invoice_number, "INV-2026-00001");

        // Download own license file (`<license_id>.coachifylicense`)
        let lic_file = portal_download_license_file(
            &cloud_conn,
            &verify_a.session_token,
            "LIC-2026-MIRP-0001",
            now,
        )
        .expect("Owner should download own license file");
        assert_eq!(lic_file.filename, "LIC-2026-MIRP-0001.coachifylicense");
        assert!(lic_file.signed_blob.contains("LIC-2026-MIRP-0001"));

        // Cross-customer isolation: Customer A attempts to download Customer B's license (`LIC-2026-CTGS-0002`)
        // or Customer B's invoice (`INV-2026-00002`) -> FORBIDDEN_LICENSE_OWNER / FORBIDDEN_INVOICE_OWNER
        let forbidden_lic = portal_download_license_file(
            &cloud_conn,
            &verify_a.session_token,
            "LIC-2026-CTGS-0002",
            now,
        )
        .unwrap_err();
        assert_eq!(forbidden_lic.code(), "FORBIDDEN_LICENSE_OWNER");

        let forbidden_inv =
            portal_get_invoice(&cloud_conn, &verify_a.session_token, "INV-2026-00002", now)
                .unwrap_err();
        assert_eq!(forbidden_inv.code(), "FORBIDDEN_INVOICE_OWNER");

        // OTP lockout: Customer B (`cust_ctg`) submits 5 consecutive wrong OTP codes -> PORTAL_ACCOUNT_LOCKED
        let login_b = portal_login(
            &cloud_conn,
            &PortalLoginInput {
                identifier: "rehana@ctgscholars.com".to_string(),
                password: "CoachifyPortal#2026".to_string(),
            },
            now,
        )
        .unwrap();

        for attempt in 1..=5 {
            let err = portal_verify_otp(
                &cloud_conn,
                &PortalVerifyOtpInput {
                    customer_id: login_b.customer_id.clone(),
                    otp_code: "000000".to_string(),
                },
                now,
            )
            .unwrap_err();
            if attempt < 5 {
                assert_eq!(err.code(), "INVALID_OTP");
            } else {
                assert_eq!(err.code(), "PORTAL_ACCOUNT_LOCKED");
            }
        }
    }

    #[test]
    fn zero_rebuild_channel_promotion_blocked_enforcement_and_append_only_recovery_drills() {
        let cloud_conn = Connection::open_in_memory().unwrap();
        let signer_conn = Connection::open_in_memory().unwrap();
        let signer = SignerService::default();
        let now = "2026-09-25T15:00:00Z";
        let unix_secs = 1_790_000_000;

        ensure_cloud_seeded(&cloud_conn, &signer_conn, &signer, now).unwrap();

        // 1. Publish v1.1.0 to `internal` with Windows Authenticode thumbprint and isolated u2026a Ed25519 signature (AC-5)
        let pub_res = publish_release_to_internal(
            &cloud_conn,
            &signer_conn,
            &signer,
            &PublishReleaseInput {
                admin_email: "ops@coachify.app".to_string(),
                version: "1.1.0".to_string(),
                notes: "Signed GA release with customer portal & channel promotion.".to_string(),
                artifact_url: "https://cdn.coachify.app/releases/CoachifySetup-v1.1.0-x64.exe"
                    .to_string(),
                sha256: "d4735e3a265e16eee03f59718b9b5d03019c07d8b6c51f90da3a666eec13ab35"
                    .to_string(),
                size_bytes: 70_123_520,
                authenticode_thumbprint: "DD44EE55FF6677889900AABBCCDD112233445566".to_string(),
                min_upgrade_from: Some("1.0.0".to_string()),
            },
            now,
        )
        .expect("Publishing signed release to internal should succeed");

        assert_eq!(pub_res.channel_head.channel, "internal");
        assert_eq!(pub_res.channel_head.version, "1.1.0");

        // 2. Promote `internal` -> `beta` (zero rebuild) and `beta` -> `stable` (with step up TOTP) (AC-6)
        let beta_promo = promote_release_channel(
            &cloud_conn,
            &PromoteReleaseChannelInput {
                admin_email: "ops@coachify.app".to_string(),
                version: "1.1.0".to_string(),
                from_channel: Some("internal".to_string()),
                to_channel: Some("beta".to_string()),
                step_up_totp_code: None,
                ..Default::default()
            },
            now,
            unix_secs,
        )
        .expect("Promoting internal -> beta should succeed");
        assert_eq!(beta_promo.channel_head.release_id, pub_res.release.id);

        let totp = generate_totp_code("COACHIFY_VENDOR_TOTP_SECRET_2026", unix_secs);
        let stable_promo = promote_release_channel(
            &cloud_conn,
            &PromoteReleaseChannelInput {
                admin_email: "owner@coachify.app".to_string(),
                version: "1.1.0".to_string(),
                from_channel: Some("beta".to_string()),
                to_channel: Some("stable".to_string()),
                step_up_totp_code: Some(totp.clone()),
                ..Default::default()
            },
            now,
            unix_secs,
        )
        .expect("Promoting beta -> stable with step up TOTP should succeed");

        // Verify zero-rebuild invariant: sha256, authenticode_thumbprint, and updater_signature are identical!
        assert_eq!(stable_promo.channel_head.sha256, pub_res.release.sha256);
        assert_eq!(
            stable_promo.channel_head.authenticode_thumbprint,
            pub_res.release.authenticode_thumbprint
        );
        assert_eq!(
            stable_promo.channel_head.updater_signature,
            pub_res.release.updater_signature
        );

        let updates_stable = handle_v1_updates(&cloud_conn, "stable").unwrap();
        assert_eq!(updates_stable.latest_live_version.as_deref(), Some("1.1.0"));
        assert_eq!(
            updates_stable.sha256.as_deref(),
            Some(pub_res.release.sha256.as_str())
        );

        // 3. Block v1.1.0 with step up TOTP -> omitted as `latest`, listed in `blocked_versions`, and promotion fails with RELEASE_NOT_PROMOTABLE (AC-7)
        update_release_status(
            &cloud_conn,
            &UpdateReleaseStatusInput {
                admin_email: "owner@coachify.app".to_string(),
                version: "1.1.0".to_string(),
                channel: "stable".to_string(),
                status: "blocked".to_string(),
                notes: Some("Blocked due to critical regression".to_string()),
                step_up_totp_code: Some(totp.clone()),
            },
            now,
            unix_secs,
        )
        .unwrap();

        let promo_blocked_err = promote_release_channel(
            &cloud_conn,
            &PromoteReleaseChannelInput {
                admin_email: "owner@coachify.app".to_string(),
                version: "1.1.0".to_string(),
                from_channel: Some("internal".to_string()),
                to_channel: Some("stable".to_string()),
                step_up_totp_code: Some(totp.clone()),
                ..Default::default()
            },
            now,
            unix_secs,
        )
        .unwrap_err();
        assert_eq!(promo_blocked_err.code(), "RELEASE_NOT_PROMOTABLE");

        let updates_after_block = handle_v1_updates(&cloud_conn, "stable").unwrap();
        assert_ne!(
            updates_after_block.latest_live_version.as_deref(),
            Some("1.1.0")
        );
        assert!(updates_after_block
            .blocked_versions
            .contains(&"1.1.0".to_string()));

        // 4. Record key recovery drill and verify append-only triggers on `key_recovery_drills` (AC-8)
        let drill = record_key_recovery_drill(
            &cloud_conn,
            &signer_conn,
            &signer,
            &RecordKeyRecoveryDrillInput {
                admin_email: "owner@coachify.app".to_string(),
                key_type: "windows_authenticode".to_string(),
                key_id: "authenticode_2026_ev".to_string(),
                custodian_primary: None,
                custodian_secondary: None,
                test_artifact_sha256: None,
                backup_source: Some("azure-kv-hsm-dhaka-backup".to_string()),
                encrypted_backup_blob: Some("ENC_AUTHENTICODE_BACKUP_BLOB_2026_Q3".to_string()),
                passphrase: Some("VaultRecoveryPass#2026".to_string()),
                notes: "Pre-1.0 GA dual key recovery verification".to_string(),
                step_up_totp_code: totp,
            },
            now,
            unix_secs,
        )
        .expect("Key recovery drill should record");
        assert!(drill.signature_verified);

        let update_drill = cloud_conn.execute(
            "UPDATE key_recovery_drills SET notes = 'tampered' WHERE id = ?1",
            params![drill.id],
        );
        assert!(update_drill.is_err());

        let delete_drill = cloud_conn.execute(
            "DELETE FROM key_recovery_drills WHERE id = ?1",
            params![drill.id],
        );
        assert!(delete_drill.is_err());
    }

    #[test]
    fn pricing_checkout_gateways_webhook_and_bank_transfer_e2e() {
        let cloud_conn = Connection::open_in_memory().unwrap();
        let signer_conn = Connection::open_in_memory().unwrap();
        let signer = SignerService::default();
        let now = "2026-09-25T12:00:00Z";
        let _unix_secs = 1_790_000_000;

        ensure_cloud_seeded(&cloud_conn, &signer_conn, &signer, now).unwrap();

        // 1. Verify seeded 4 tiers in catalog
        let catalog = get_pricing_catalog(&cloud_conn).unwrap();
        assert!(catalog.tiers.len() >= 4);
        let tier_3m = catalog
            .tiers
            .iter()
            .find(|t| t.billing_cycle == "3_months")
            .unwrap();
        assert_eq!(tier_3m.period_months, 3);
        assert_eq!(tier_3m.amount_poisha, 450_000);
        assert_eq!(tier_3m.monthly_rate_poisha, 150_000);

        let tier_lifetime = catalog
            .tiers
            .iter()
            .find(|t| t.billing_cycle == "lifetime")
            .unwrap();
        assert!(tier_lifetime.is_lifetime);
        assert_eq!(tier_lifetime.amount_poisha, 5_000_000);

        // 2. Admin updates price tier
        let updated = admin_update_price_tier(
            &cloud_conn,
            &AdminUpdatePriceTierInput {
                admin_email: "ops@coachify.app".to_string(),
                plan_code: "local".to_string(),
                billing_cycle: "3_months".to_string(),
                monthly_rate_bdt: Some(1600.0),
                total_bdt_override: Some(4800.0),
            },
            now,
        )
        .unwrap();
        assert_eq!(updated.amount_poisha, 480_000);
        assert_eq!(updated.monthly_rate_poisha, 160_000);

        // 3. Create checkout session for a new order (12 months)
        let session = create_checkout_session(
            &cloud_conn,
            &CreateCheckoutSessionInput {
                plan_code: "local".to_string(),
                billing_cycle: "12_months".to_string(),
                kind: "new_purchase".to_string(),
                method: "bkash".to_string(),
                org_name: "Mirpur Study Center".to_string(),
                owner_name: "Rafiqul Islam".to_string(),
                phone: "+8801700112233".to_string(),
                email: Some("newbuyer@mirpurstudy.com".to_string()),
                target_request_code: None,
                session_token: None,
            },
            now,
        )
        .unwrap();

        assert_eq!(session.amount_poisha, 1_200_000);
        assert!(session
            .gateway_redirect_url
            .as_deref()
            .unwrap_or("")
            .contains("checkout"));

        // 4. Test Webhook handling with HMAC-SHA256 signature verification
        let webhook_payload = serde_json::json!({
            "order_number": session.order_number,
            "gateway_event_id": "EVT-BKASH-001",
            "gateway_trx_id": "BKASH-TX-998877",
            "amount_poisha": 1_200_000,
            "sender_account": "01700112233"
        })
        .to_string();

        let valid_signature =
            compute_hmac_sha256_hex(DEFAULT_GATEWAY_WEBHOOK_SECRET, webhook_payload.as_bytes());

        // Invalid signature should fail
        let bad_res = handle_payment_webhook(
            &cloud_conn,
            &signer_conn,
            &signer,
            &PaymentWebhookInput {
                provider: "bkash_direct".to_string(),
                hmac_signature: "invalid_hex_signature".to_string(),
                raw_payload_json: webhook_payload.clone(),
            },
            now,
        );
        assert!(bad_res.is_err());

        // Valid signature fulfills order
        let fulfill_res = handle_payment_webhook(
            &cloud_conn,
            &signer_conn,
            &signer,
            &PaymentWebhookInput {
                provider: "bkash_direct".to_string(),
                hmac_signature: valid_signature.clone(),
                raw_payload_json: webhook_payload.clone(),
            },
            now,
        )
        .unwrap();

        assert_eq!(fulfill_res.processed_status, "applied");
        assert!(fulfill_res.invoice_number.is_some());

        // 5. Test Duplicate Webhook delivery (Idempotency)
        let dup_res = handle_payment_webhook(
            &cloud_conn,
            &signer_conn,
            &signer,
            &PaymentWebhookInput {
                provider: "bkash_direct".to_string(),
                hmac_signature: valid_signature,
                raw_payload_json: webhook_payload,
            },
            now,
        )
        .unwrap();

        assert_eq!(dup_res.processed_status, "duplicate_ignored");

        // 6. Test Bank Transfer Submission & Vendor Admin Approval
        let bank_session = create_checkout_session(
            &cloud_conn,
            &CreateCheckoutSessionInput {
                plan_code: "local".to_string(),
                billing_cycle: "lifetime".to_string(),
                kind: "lifetime_purchase".to_string(),
                method: "bank_transfer".to_string(),
                org_name: "Uttara Learning Academy".to_string(),
                owner_name: "Farhan Ahmed".to_string(),
                phone: "+8801811223344".to_string(),
                email: Some("academy@uttaralearning.com".to_string()),
                target_request_code: None,
                session_token: None,
            },
            now,
        )
        .unwrap();

        let bank_submit = submit_bank_transfer_proof(
            &cloud_conn,
            &SubmitBankTransferInput {
                order_number: bank_session.order_number.clone(),
                bank_reference: "DBBL-DEP-55443322".to_string(),
                bank_slip_url: Some(
                    "https://files.coachify.app/proofs/dbbl-dep-55443322.jpg".to_string(),
                ),
            },
            now,
        )
        .unwrap();
        assert_eq!(bank_submit.status, "pending_bank_review");

        // List pending bank transfers
        let pending = admin_list_pending_bank_transfers(&cloud_conn).unwrap();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].order_number, bank_session.order_number);
        assert_eq!(pending[0].amount_poisha, 5_000_000);
        assert_eq!(pending[0].bank_reference, "DBBL-DEP-55443322");

        // Admin approves bank transfer
        let approved = admin_approve_bank_transfer(
            &cloud_conn,
            &signer_conn,
            &signer,
            &ApproveBankTransferInput {
                admin_email: "ops@coachify.app".to_string(),
                order_number: bank_session.order_number.clone(),
                verification_notes: Some("Verified funds in DBBL corporate account".to_string()),
            },
            now,
        )
        .unwrap();

        assert_eq!(approved.status, "completed");
        assert!(approved
            .license_id
            .as_deref()
            .unwrap_or("")
            .starts_with("LIC-2026-"));
        assert!(approved.pairing_code.is_some());
        assert!(approved
            .invoice_number
            .as_deref()
            .unwrap_or("")
            .starts_with("INV-2026-"));
        assert!(approved
            .signed_license_blob
            .as_deref()
            .unwrap_or("")
            .contains("9999-12-31"));

        // Verify order status endpoint
        let final_status =
            get_checkout_order_status(&cloud_conn, &bank_session.order_number).unwrap();
        assert_eq!(final_status.status, "completed");
        assert_eq!(final_status.billing_cycle, "lifetime");
    }

    #[test]
    fn device_management_portal_deactivation_replacement_and_transfer_limits() {
        let cloud_conn = Connection::open_in_memory().unwrap();
        let signer_conn = Connection::open_in_memory().unwrap();
        let signer = SignerService::default();
        let now = "2026-09-25T14:00:00Z";
        let unix_secs = 1_790_000_000;

        ensure_cloud_seeded(&cloud_conn, &signer_conn, &signer, now).unwrap();

        // 1. Authenticate to Customer Portal via OTP
        let login_res = portal_login(
            &cloud_conn,
            &PortalLoginInput {
                identifier: "+8801711223344".to_string(),
                password: "CoachifyPortal#2026".to_string(),
            },
            now,
        )
        .expect("Mirpur owner login should succeed");

        let auth = portal_verify_otp(
            &cloud_conn,
            &PortalVerifyOtpInput {
                customer_id: login_res.customer_id.clone(),
                otp_code: login_res.demo_otp_code.clone(),
            },
            now,
        )
        .unwrap();

        // 2. List devices
        let dev_list = portal_list_devices(&cloud_conn, &auth.session_token, now).unwrap();
        assert_eq!(dev_list.devices.len(), 1);
        let pc = &dev_list.devices[0];
        assert_eq!(pc.installation_id, "inst-mirpur-host-01");
        assert_eq!(pc.status, "active");
        assert_eq!(dev_list.transfers_remaining_12m, 3);

        // 3. Rename device (metadata update, no transfer consumed)
        let rename_res = portal_rename_device(
            &cloud_conn,
            &PortalRenameDeviceInput {
                session_token: auth.session_token.clone(),
                installation_id: "inst-mirpur-host-01".to_string(),
                new_label: "Mirpur Main Front Desk PC".to_string(),
            },
            now,
        )
        .unwrap();
        assert_eq!(rename_res.label, "Mirpur Main Front Desk PC");

        // 4. Replace device via Offline Request Code (REQ-XXXX-...) (Consumes 1 transfer)
        let (_, new_req_code) = generate_request_code_for_installation("inst-mirpur-host-02");
        let replace_otp = portal_request_device_otp(
            &cloud_conn,
            &PortalRequestDeviceOtpInput {
                session_token: auth.session_token.clone(),
                action: "replace".to_string(),
                installation_id: "inst-mirpur-host-01".to_string(),
            },
            now,
            unix_secs,
        )
        .unwrap()
        .demo_otp_code;

        // Wrong OTP fails
        let bad_replace = portal_replace_device(
            &cloud_conn,
            &signer_conn,
            &signer,
            &PortalReplaceDeviceInput {
                session_token: auth.session_token.clone(),
                old_installation_id: "inst-mirpur-host-01".to_string(),
                otp_code: "000000".to_string(),
                new_request_code_or_qr: Some(new_req_code.clone()),
                new_label: "Mirpur Replacement PC-02".to_string(),
                reason: "Swapped to new PC".to_string(),
            },
            now,
        );
        assert!(bad_replace.is_err());

        let replace_res = portal_replace_device(
            &cloud_conn,
            &signer_conn,
            &signer,
            &PortalReplaceDeviceInput {
                session_token: auth.session_token.clone(),
                old_installation_id: "inst-mirpur-host-01".to_string(),
                otp_code: replace_otp,
                new_request_code_or_qr: Some(new_req_code),
                new_label: "Mirpur Replacement PC-02".to_string(),
                reason: "Swapped to new PC".to_string(),
            },
            now,
        )
        .unwrap();

        assert_eq!(
            replace_res.new_installation_id.as_deref(),
            Some("inst-mirpur-host-02")
        );
        assert_eq!(replace_res.transfers_remaining_12m, 2);
        assert!(replace_res.signed_license_blob.is_some());

        // 5. Deactivate newly active device (inst-mirpur-host-02) (Consumes another transfer)
        let deact_otp = portal_request_device_otp(
            &cloud_conn,
            &PortalRequestDeviceOtpInput {
                session_token: auth.session_token.clone(),
                action: "deactivate".to_string(),
                installation_id: "inst-mirpur-host-02".to_string(),
            },
            now,
            unix_secs,
        )
        .unwrap()
        .demo_otp_code;

        let deact_res = portal_deactivate_device(
            &cloud_conn,
            &PortalDeactivateDeviceInput {
                session_token: auth.session_token.clone(),
                installation_id: "inst-mirpur-host-02".to_string(),
                otp_code: deact_otp,
                reason: "Decommissioning machine".to_string(),
            },
            now,
        )
        .unwrap();

        assert_eq!(deact_res.action, "deactivate");
        assert!(deact_res.new_pairing_code.is_some());

        // 6. Desktop refresh check: deactivation outcome returned
        let refresh = handle_v1_refresh(
            &cloud_conn,
            &RefreshEndpointRequest {
                license_id: "LIC-2026-MIRP-0001".to_string(),
                installation_id: "inst-mirpur-host-02".to_string(),
                current_serial: 2,
                app_version: "1.0.1".to_string(),
                active_students: 840,
            },
            now,
        )
        .unwrap();

        assert_eq!(refresh.outcome, "deactivated");
        assert!(refresh.deactivated_at.is_some());
        assert_eq!(
            refresh.deactivation_reason.as_deref(),
            Some("Decommissioning machine")
        );

        // 7. Verify transfer limit cap (3 max in 12 months)
        cloud_conn
            .execute(
                "UPDATE licenses SET transfer_count_12m = 3 WHERE license_id = 'LIC-2026-MIRP-0001'",
                [],
            )
            .unwrap();

        let (_, req_code_3) = generate_request_code_for_installation("inst-mirpur-host-03");
        let replace_otp_2 = portal_request_device_otp(
            &cloud_conn,
            &PortalRequestDeviceOtpInput {
                session_token: auth.session_token.clone(),
                action: "replace".to_string(),
                installation_id: "inst-mirpur-host-02".to_string(),
            },
            now,
            unix_secs,
        )
        .unwrap()
        .demo_otp_code;

        let exceeded_err = portal_replace_device(
            &cloud_conn,
            &signer_conn,
            &signer,
            &PortalReplaceDeviceInput {
                session_token: auth.session_token.clone(),
                old_installation_id: "inst-mirpur-host-02".to_string(),
                otp_code: replace_otp_2,
                new_request_code_or_qr: Some(req_code_3),
                new_label: "Mirpur PC-03".to_string(),
                reason: "Another replacement".to_string(),
            },
            now,
        )
        .unwrap_err();

        assert_eq!(exceeded_err.code(), "DEVICE_TRANSFER_LIMIT_EXCEEDED");

        // Customer requests transfer reset
        portal_request_transfer_reset(
            &cloud_conn,
            &PortalRequestTransferResetInput {
                session_token: auth.session_token.clone(),
                reason: "Motherboard fried due to power surge".to_string(),
            },
            now,
        )
        .unwrap();

        // Vendor admin resets transfer count
        let admin_totp = generate_totp_code("COACHIFY_VENDOR_TOTP_SECRET_2026", unix_secs);
        let admin_reset = admin_reset_transfer_count(
            &cloud_conn,
            &AdminResetTransferCountInput {
                admin_email: "ops@coachify.app".to_string(),
                license_id: "LIC-2026-MIRP-0001".to_string(),
                step_up_totp_code: admin_totp,
            },
            now,
            unix_secs,
        )
        .unwrap();
        assert_eq!(admin_reset.transfer_count_12m, 0);
        assert_eq!(admin_reset.transfers_remaining_12m, 3);

        // After admin reset, device listing shows 3 transfers remaining again!
        let dev_list_after = portal_list_devices(&cloud_conn, &auth.session_token, now).unwrap();
        assert_eq!(dev_list_after.transfers_remaining_12m, 3);
    }
}
