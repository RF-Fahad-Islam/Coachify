use crate::db;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use specta::Type;
use thiserror::Error;

/// Lease duration in seconds (5 minutes).
const LEASE_DURATION_SECS: i64 = 300;

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Error, Debug)]
pub enum JobError {
    #[error("Database error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("Job not found: {0}")]
    NotFound(String),
    #[error("Job already claimed: {0}")]
    AlreadyClaimed(String),
    #[error("Invalid job kind: {0}")]
    InvalidKind(String),
    #[error("Serialization error: {0}")]
    Serde(String),
}

impl JobError {
    pub fn code(&self) -> &'static str {
        match self {
            JobError::Sqlite(_) => "job_database_error",
            JobError::NotFound(_) => "job_not_found",
            JobError::AlreadyClaimed(_) => "job_already_claimed",
            JobError::InvalidKind(_) => "job_invalid_kind",
            JobError::Serde(_) => "job_serde_error",
        }
    }
}

// ---------------------------------------------------------------------------
// Job Kinds
// ---------------------------------------------------------------------------

pub const JOB_SCHEDULED_BACKUP: &str = "scheduled_backup";
pub const JOB_VERIFY_BACKUP: &str = "verify_backup";
pub const JOB_RESTORE_DRILL: &str = "restore_drill";
pub const JOB_HOURLY_SNAPSHOT: &str = "hourly_snapshot";
pub const JOB_PRUNE_BACKUPS: &str = "prune_backups";
pub const JOB_DB_MAINTENANCE: &str = "db_maintenance";

// ---------------------------------------------------------------------------
// Data Types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct JobItem {
    pub id: String,
    pub kind: String,
    pub status: String,
    pub payload_json: String,
    pub run_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub lease_until: Option<String>,
    pub attempts: u32,
    pub max_attempts: u32,
    pub last_error: Option<String>,
    pub result_json: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

// ---------------------------------------------------------------------------
// Job Queue Operations
// ---------------------------------------------------------------------------

/// Enqueues a new job. If a pending/running job of the same kind already exists,
/// returns its ID instead of creating a duplicate (deduplication).
pub fn enqueue_job(
    conn: &Connection,
    kind: &str,
    payload_json: &str,
    run_at: &str,
) -> Result<String, JobError> {
    // Deduplication: check for existing pending/running job of same kind
    let existing: Option<String> = conn
        .query_row(
            "SELECT id FROM jobs WHERE kind = ?1 AND status IN ('pending', 'running') LIMIT 1",
            [kind],
            |row| row.get(0),
        )
        .ok();

    if let Some(id) = existing {
        return Ok(id);
    }

    let id = uuid::Uuid::now_v7().to_string();
    let now = db::now_utc_iso();

    conn.execute(
        "INSERT INTO jobs (id, kind, status, payload_json, run_at, attempts, max_attempts, created_at, updated_at)
         VALUES (?1, ?2, 'pending', ?3, ?4, 0, 3, ?5, ?5)",
        rusqlite::params![id, kind, payload_json, run_at, now],
    )?;

    Ok(id)
}

/// Claims the next ready job by kind. Sets status to 'running' and acquires
/// a lease that expires in 5 minutes.
pub fn claim_next_job(conn: &Connection, kinds: &[&str]) -> Result<Option<JobItem>, JobError> {
    if kinds.is_empty() {
        return Ok(None);
    }

    let now = db::now_utc_iso();

    // Build a parameterized query for the kinds
    let placeholders: Vec<String> = kinds
        .iter()
        .enumerate()
        .map(|(i, _)| format!("?{}", i + 1))
        .collect();
    let kinds_clause = placeholders.join(", ");
    let sql = format!(
        "SELECT id, kind, status, payload_json, run_at, started_at, completed_at,
                lease_until, attempts, max_attempts, last_error, result_json,
                created_at, updated_at
         FROM jobs
         WHERE status = 'pending'
           AND kind IN ({kinds_clause})
           AND run_at <= ?{}
           AND attempts < max_attempts
         ORDER BY run_at ASC
         LIMIT 1",
        kinds.len() + 1,
    );

    let mut stmt = conn.prepare(&sql)?;

    // Build params dynamically
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = kinds
        .iter()
        .map(|k| Box::new(k.to_string()) as Box<dyn rusqlite::types::ToSql>)
        .collect();
    params.push(Box::new(now.clone()));

    let job = stmt
        .query_row(
            rusqlite::params_from_iter(params.iter().map(|p| p.as_ref())),
            map_job_row,
        )
        .ok();

    if let Some(ref job) = job {
        // Compute lease expiry
        let lease_until = compute_lease_until(&now);

        // Claim it
        conn.execute(
            "UPDATE jobs SET status = 'running', started_at = ?2, lease_until = ?3,
                    attempts = attempts + 1, updated_at = ?2
             WHERE id = ?1 AND status = 'pending'",
            rusqlite::params![job.id, now, lease_until],
        )?;
    }

    Ok(job)
}

/// Marks a job as completed with an optional result.
pub fn complete_job(
    conn: &Connection,
    job_id: &str,
    result_json: Option<&str>,
) -> Result<(), JobError> {
    let now = db::now_utc_iso();
    let affected = conn.execute(
        "UPDATE jobs SET status = 'completed', completed_at = ?2, result_json = ?3,
                lease_until = NULL, updated_at = ?2
         WHERE id = ?1",
        rusqlite::params![job_id, now, result_json],
    )?;
    if affected == 0 {
        return Err(JobError::NotFound(job_id.to_string()));
    }
    Ok(())
}

/// Marks a job as failed with an error message. If attempts < max_attempts,
/// the job is reset to 'pending' for retry.
pub fn fail_job(conn: &Connection, job_id: &str, error: &str) -> Result<(), JobError> {
    let now = db::now_utc_iso();

    // Check if we should retry
    let (attempts, max_attempts): (u32, u32) = conn
        .query_row(
            "SELECT attempts, max_attempts FROM jobs WHERE id = ?1",
            [job_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|_| JobError::NotFound(job_id.to_string()))?;

    let new_status = if attempts < max_attempts {
        "pending"
    } else {
        "failed"
    };

    conn.execute(
        "UPDATE jobs SET status = ?2, last_error = ?3, lease_until = NULL,
                completed_at = CASE WHEN ?2 = 'failed' THEN ?4 ELSE NULL END,
                updated_at = ?4
         WHERE id = ?1",
        rusqlite::params![job_id, new_status, error, now],
    )?;

    Ok(())
}

/// Cancels a pending job.
pub fn cancel_job(conn: &Connection, job_id: &str) -> Result<(), JobError> {
    let now = db::now_utc_iso();
    let affected = conn.execute(
        "UPDATE jobs SET status = 'cancelled', completed_at = ?2, lease_until = NULL, updated_at = ?2
         WHERE id = ?1 AND status IN ('pending', 'running')",
        rusqlite::params![job_id, now],
    )?;
    if affected == 0 {
        return Err(JobError::NotFound(job_id.to_string()));
    }
    Ok(())
}

/// Reclaims expired leases: running jobs whose lease has expired are reset to pending.
pub fn reclaim_expired_leases(conn: &Connection) -> Result<u32, JobError> {
    let now = db::now_utc_iso();
    let affected = conn.execute(
        "UPDATE jobs SET status = 'pending', lease_until = NULL, updated_at = ?1
         WHERE status = 'running' AND lease_until IS NOT NULL AND lease_until < ?1",
        [&now],
    )?;
    Ok(affected as u32)
}

/// Lists jobs by status.
pub fn list_jobs(
    conn: &Connection,
    status: Option<&str>,
    limit: u32,
) -> Result<Vec<JobItem>, JobError> {
    if let Some(s) = status {
        let mut stmt = conn.prepare(
            "SELECT id, kind, status, payload_json, run_at, started_at, completed_at,
                    lease_until, attempts, max_attempts, last_error, result_json,
                    created_at, updated_at
             FROM jobs WHERE status = ?1
             ORDER BY created_at DESC LIMIT ?2",
        )?;
        let items: Vec<JobItem> = stmt
            .query_map(rusqlite::params![s, limit], map_job_row)?
            .filter_map(|r| r.ok())
            .collect();
        Ok(items)
    } else {
        let mut stmt = conn.prepare(
            "SELECT id, kind, status, payload_json, run_at, started_at, completed_at,
                    lease_until, attempts, max_attempts, last_error, result_json,
                    created_at, updated_at
             FROM jobs
             ORDER BY created_at DESC LIMIT ?1",
        )?;
        let items: Vec<JobItem> = stmt
            .query_map([limit], map_job_row)?
            .filter_map(|r| r.ok())
            .collect();
        Ok(items)
    }
}

/// Gets a single job by ID.
pub fn get_job_status(conn: &Connection, job_id: &str) -> Result<Option<JobItem>, JobError> {
    let item = conn
        .query_row(
            "SELECT id, kind, status, payload_json, run_at, started_at, completed_at,
                    lease_until, attempts, max_attempts, last_error, result_json,
                    created_at, updated_at
             FROM jobs WHERE id = ?1",
            [job_id],
            map_job_row,
        )
        .ok();
    Ok(item)
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn map_job_row(row: &rusqlite::Row) -> rusqlite::Result<JobItem> {
    Ok(JobItem {
        id: row.get(0)?,
        kind: row.get(1)?,
        status: row.get(2)?,
        payload_json: row.get(3)?,
        run_at: row.get(4)?,
        started_at: row.get(5)?,
        completed_at: row.get(6)?,
        lease_until: row.get(7)?,
        attempts: row.get(8)?,
        max_attempts: row.get(9)?,
        last_error: row.get(10)?,
        result_json: row.get(11)?,
        created_at: row.get(12)?,
        updated_at: row.get(13)?,
    })
}

fn compute_lease_until(now: &str) -> String {
    // Parse the ISO timestamp and add the lease duration
    if let Ok(ts) = jiff::Timestamp::from_str(now) {
        let span = jiff::Span::new().seconds(LEASE_DURATION_SECS);
        if let Ok(future) = ts.checked_add(span) {
            return future.to_string();
        }
    }
    // Fallback: just return now (lease will be immediately reclaimable)
    now.to_string()
}

use std::str::FromStr;

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_test_db() -> (tempfile::TempDir, Connection) {
        let dir = tempfile::tempdir().expect("tempdir");
        let db_path = dir.path().join("jobs_test.db");
        let conn = db::open_connection_at(&db_path).expect("open test db");
        db::apply_all_migrations(&conn).expect("apply migrations");
        (dir, conn)
    }

    #[test]
    fn test_enqueue_and_deduplication() {
        let (_dir, conn) = setup_test_db();
        let now = db::now_utc_iso();

        let id1 = enqueue_job(&conn, JOB_SCHEDULED_BACKUP, "{}", &now).expect("enqueue 1");
        let id2 = enqueue_job(&conn, JOB_SCHEDULED_BACKUP, "{}", &now).expect("enqueue duplicate");
        assert_eq!(id1, id2, "Duplicate pending job must be deduplicated");

        let id3 = enqueue_job(&conn, JOB_RESTORE_DRILL, "{}", &now).expect("enqueue drill");
        assert_ne!(id1, id3, "Different job kinds must have distinct IDs");
    }

    #[test]
    fn test_claim_and_complete_lifecycle() {
        let (_dir, conn) = setup_test_db();
        let now = db::now_utc_iso();

        let job_id =
            enqueue_job(&conn, JOB_HOURLY_SNAPSHOT, "{\"pages\":256}", &now).expect("enqueue");

        // Claim
        let claimed = claim_next_job(&conn, &[JOB_HOURLY_SNAPSHOT])
            .expect("claim")
            .expect("job should be found");
        assert_eq!(claimed.id, job_id);

        let status = get_job_status(&conn, &job_id)
            .expect("get status")
            .expect("found");
        assert_eq!(status.status, "running");
        assert_eq!(status.attempts, 1);
        assert!(status.lease_until.is_some());

        // Complete
        complete_job(&conn, &job_id, Some("{\"bytes\":10240}")).expect("complete");

        let final_status = get_job_status(&conn, &job_id)
            .expect("get status")
            .expect("found");
        assert_eq!(final_status.status, "completed");
        assert!(final_status.completed_at.is_some());
        assert!(final_status.lease_until.is_none());
        assert_eq!(
            final_status.result_json.as_deref(),
            Some("{\"bytes\":10240}")
        );
    }

    #[test]
    fn test_failure_and_retry_limit() {
        let (_dir, conn) = setup_test_db();
        let now = db::now_utc_iso();

        let job_id = enqueue_job(&conn, JOB_PRUNE_BACKUPS, "{}", &now).expect("enqueue");

        // Attempt 1: fail -> resets to pending
        let _ = claim_next_job(&conn, &[JOB_PRUNE_BACKUPS]).expect("claim 1");
        fail_job(&conn, &job_id, "disk busy").expect("fail 1");
        let s1 = get_job_status(&conn, &job_id)
            .expect("get status")
            .expect("found");
        assert_eq!(s1.status, "pending");
        assert_eq!(s1.attempts, 1);

        // Attempt 2: fail -> resets to pending
        let _ = claim_next_job(&conn, &[JOB_PRUNE_BACKUPS]).expect("claim 2");
        fail_job(&conn, &job_id, "timeout").expect("fail 2");
        let s2 = get_job_status(&conn, &job_id)
            .expect("get status")
            .expect("found");
        assert_eq!(s2.status, "pending");
        assert_eq!(s2.attempts, 2);

        // Attempt 3: fail -> marks as permanently failed
        let _ = claim_next_job(&conn, &[JOB_PRUNE_BACKUPS]).expect("claim 3");
        fail_job(&conn, &job_id, "unrecoverable").expect("fail 3");
        let s3 = get_job_status(&conn, &job_id)
            .expect("get status")
            .expect("found");
        assert_eq!(s3.status, "failed");
        assert_eq!(s3.attempts, 3);
        assert_eq!(s3.last_error.as_deref(), Some("unrecoverable"));
    }

    #[test]
    fn test_reclaim_expired_leases() {
        let (_dir, conn) = setup_test_db();

        // Insert a stuck running job with an expired lease (1 hour ago)
        conn.execute(
            "INSERT INTO jobs (id, kind, status, payload_json, run_at, started_at, lease_until, attempts, max_attempts, created_at, updated_at)
             VALUES ('stuck-1', 'scheduled_backup', 'running', '{}', datetime('now', '-2 hours'), datetime('now', '-2 hours'), datetime('now', '-1 hour'), 1, 3, datetime('now', '-2 hours'), datetime('now', '-2 hours'))",
            [],
        )
        .expect("insert stuck job");

        let reclaimed = reclaim_expired_leases(&conn).expect("reclaim");
        assert_eq!(reclaimed, 1);

        let status = get_job_status(&conn, "stuck-1")
            .expect("get status")
            .expect("found");
        assert_eq!(status.status, "pending");
        assert!(status.lease_until.is_none());
    }
}
