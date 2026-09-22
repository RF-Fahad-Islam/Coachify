use coachify_core::db;
use coachify_core::dispatcher::{self, ChangeEvent};
use rusqlite::params;
use serde_json::json;
use tempfile::tempdir;

fn setup_test_db() -> (tempfile::TempDir, rusqlite::Connection) {
    let dir = tempdir().expect("create temp dir");
    let db_path = dir.path().join("test_comms.db");
    let conn = db::open_connection_at(&db_path).expect("open test db");
    db::apply_all_migrations(&conn).expect("apply all migrations");
    (dir, conn)
}

fn dispatch_cmd(
    conn: &mut rusqlite::Connection,
    command: &str,
    payload: serde_json::Value,
    token: Option<&str>,
    idem_key: Option<&str>,
) -> (serde_json::Value, Vec<ChangeEvent>) {
    let payload_str = serde_json::to_string(&payload).unwrap();
    let (res_str, events) = dispatcher::dispatch(conn, command, &payload_str, token, idem_key);
    let val: serde_json::Value = serde_json::from_str(&res_str).unwrap();
    (val, events)
}

fn bootstrap_and_login(conn: &mut rusqlite::Connection) -> String {
    let (res, _) = dispatch_cmd(
        conn,
        "system.bootstrap_owner",
        json!({
            "username": "owner",
            "display_name": "Institution Owner",
            "password": "Password123!"
        }),
        None,
        None,
    );
    assert_eq!(res["ok"], true, "bootstrap failed: {:?}", res);

    let (res, _) = dispatch_cmd(
        conn,
        "auth.login",
        json!({
            "username": "owner",
            "password": "Password123!"
        }),
        None,
        None,
    );
    assert_eq!(res["ok"], true, "login failed: {:?}", res);
    res["data"]["session_token"].as_str().unwrap().to_string()
}

#[test]
fn test_outbox_enqueue_list_stats_and_details() {
    let (_dir, mut conn) = setup_test_db();
    let token = bootstrap_and_login(&mut conn);

    // Insert 5 test outbox records directly with distinct channels and statuses
    let now = db::now_utc_iso();
    conn.execute(
        "INSERT INTO outbox (id, channel, to_address, body, segments, encoding, status, scheduled_at, attempts, created_at)
         VALUES
         ('msg-1', 'sms', '01711000001', 'Test SMS 1', 1, 'gsm', 'queued', ?1, 0, ?1),
         ('msg-2', 'sms', '01711000002', 'Test SMS 2', 1, 'gsm', 'sent', ?1, 1, ?1),
         ('msg-3', 'whatsapp', '01711000003', 'Test WA 1', 1, 'utf8', 'failed', ?1, 3, ?1),
         ('msg-4', 'whatsapp', '01711000004', 'Test WA 2', 1, 'utf8', 'uncertain', ?1, 1, ?1),
         ('msg-5', 'email', 'student@example.com', 'Test Email 1', 1, 'utf8', 'cancelled', ?1, 0, ?1)",
        params![now],
    )
    .expect("insert test outbox records");

    // Also record a delivery attempt for msg-4
    conn.execute(
        "INSERT INTO delivery_attempts (id, outbox_id, attempted_at, outcome, http_status, response_excerpt, duration_ms)
         VALUES ('att-msg-4', 'msg-4', ?1, 'timeout', 408, 'Gateway timeout simulation', 5000)",
        params![now],
    )
    .expect("insert test delivery attempt");

    // 1: Query outbox stats
    let (stats_res, _) = dispatch_cmd(
        &mut conn,
        "comms.get_outbox_stats",
        json!({}),
        Some(&token),
        None,
    );
    assert_eq!(stats_res["ok"], true);
    let stats = &stats_res["data"];
    assert_eq!(stats["total_queued"], 1);
    assert_eq!(stats["total_sent"], 1);
    assert_eq!(stats["total_failed"], 1);
    assert_eq!(stats["total_uncertain"], 1);
    assert_eq!(stats["total_cancelled"], 1);

    // 2: List outbox with channel sms filter
    let (list_sms, _) = dispatch_cmd(
        &mut conn,
        "comms.list_outbox",
        json!({ "channel": "sms" }),
        Some(&token),
        None,
    );
    assert_eq!(list_sms["ok"], true);
    assert_eq!(list_sms["data"]["total_count"], 2);
    let sms_items = list_sms["data"]["items"].as_array().unwrap();
    assert_eq!(sms_items.len(), 2);

    // 3: List outbox with status queued filter
    let (list_queued, _) = dispatch_cmd(
        &mut conn,
        "comms.list_outbox",
        json!({ "status": "queued" }),
        Some(&token),
        None,
    );
    assert_eq!(list_queued["ok"], true);
    assert_eq!(list_queued["data"]["total_count"], 1);
    assert_eq!(list_queued["data"]["items"][0]["id"], "msg-1");

    // 4: Get outbox detail for msg-4 (uncertain message with attempt)
    let (detail_res, _) = dispatch_cmd(
        &mut conn,
        "comms.get_outbox_detail",
        json!({ "id": "msg-4" }),
        Some(&token),
        None,
    );
    assert_eq!(detail_res["ok"], true);
    let detail = &detail_res["data"];
    assert_eq!(detail["item"]["id"], "msg-4");
    assert_eq!(detail["item"]["status"], "uncertain");
    let attempts = detail["attempts"].as_array().unwrap();
    assert_eq!(attempts.len(), 1);
    assert_eq!(attempts[0]["outcome"], "timeout");
    assert_eq!(attempts[0]["http_status"], 408);
}

#[test]
fn test_outbox_cancellation_and_resend_lifecycle() {
    let (_dir, mut conn) = setup_test_db();
    let token = bootstrap_and_login(&mut conn);

    let now = db::now_utc_iso();
    conn.execute(
        "INSERT INTO outbox (id, channel, to_address, body, status, scheduled_at, attempts, created_at)
         VALUES
         ('cancel-target', 'sms', '01711000099', 'To be cancelled', 'queued', ?1, 0, ?1),
         ('resend-target', 'sms', '01711000098', 'To be resent', 'failed', ?1, 3, ?1)",
        params![now],
    )
    .unwrap();

    // 1: Cancel message in queued status: succeeds
    let (c_res, _) = dispatch_cmd(
        &mut conn,
        "comms.cancel_message",
        json!({ "id": "cancel-target" }),
        Some(&token),
        None,
    );
    assert_eq!(c_res["ok"], true);

    let status1: String = conn
        .query_row(
            "SELECT status FROM outbox WHERE id = 'cancel-target'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(status1, "cancelled");

    // 2: Resend message in failed status: resets to queued
    let (r_res, _) = dispatch_cmd(
        &mut conn,
        "comms.resend_message",
        json!({ "id": "resend-target" }),
        Some(&token),
        None,
    );
    assert_eq!(r_res["ok"], true);

    let status2: String = conn
        .query_row(
            "SELECT status FROM outbox WHERE id = 'resend-target'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(status2, "queued");
}

#[test]
fn test_provider_config_dpapi_secret_encryption_and_masking() {
    let (_dir, mut conn) = setup_test_db();
    let token = bootstrap_and_login(&mut conn);

    // Save provider config with a plain secret string
    let plain_secret = "secret_greenweb_api_token_xyz_987";
    let (save_res, _) = dispatch_cmd(
        &mut conn,
        "comms.save_provider_config",
        json!({
            "channel": "sms",
            "adapter": "greenweb",
            "is_active": true,
            "config_json": "{\"sender_id\":\"Coachify\"}",
            "secret": plain_secret
        }),
        Some(&token),
        None,
    );
    assert_eq!(save_res["ok"], true, "save config failed: {:?}", save_res);

    // Verify in database that config_json does NOT contain the plain secret
    let (config_json, secret_ref): (String, Option<String>) = conn
        .query_row(
            "SELECT config_json, secret_ref FROM provider_configs WHERE channel = 'sms' AND adapter = 'greenweb'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .expect("query provider config");

    assert!(!config_json.contains(plain_secret));
    assert!(secret_ref.is_some());
    let secret_blob = secret_ref.unwrap();
    assert!(!secret_blob.contains(plain_secret));

    // Call comms.get_provider_configs and verify secret is masked
    let (get_res, _) = dispatch_cmd(
        &mut conn,
        "comms.get_provider_configs",
        json!({}),
        Some(&token),
        None,
    );
    assert_eq!(get_res["ok"], true);
    let configs = get_res["data"].as_array().unwrap();
    let greenweb = configs
        .iter()
        .find(|c| c["channel"] == "sms" && c["adapter"] == "greenweb")
        .expect("greenweb config must exist");

    assert_eq!(greenweb["is_active"], true);
    assert_eq!(greenweb["secret_is_set"], true);
    assert_eq!(greenweb["masked_secret"], "••••••••••••");
}

#[test]
fn test_network_timeout_transitions_to_uncertain_status_with_zero_auto_retries() {
    let (_dir, mut conn) = setup_test_db();
    let token = bootstrap_and_login(&mut conn);

    // Insert an outbox message scheduled in the past
    let past = "2026-01-01 10:00:00";
    conn.execute(
        "INSERT INTO outbox (id, channel, to_address, body, status, scheduled_at, attempts, created_at)
         VALUES ('uncertain-target', 'sms', '01711999888', 'Crucial payment confirmation', 'queued', ?1, 0, ?1)",
        params![past],
    )
    .unwrap();

    // 1: Enable mock timeout simulation
    coachify_core::comms::set_mock_timeout(true);

    // 2: Flush queue with force_bypass_quiet true
    let (flush_res1, _) = dispatch_cmd(
        &mut conn,
        "comms.flush_queue",
        json!({ "force_bypass_quiet": true }),
        Some(&token),
        None,
    );
    assert_eq!(flush_res1["ok"], true);
    assert_eq!(flush_res1["data"]["processed_count"], 1);

    // 3: Verify outbox status transitioned to 'uncertain' and attempts count increased
    let (status1, attempts1, last_err1): (String, i64, Option<String>) = conn
        .query_row(
            "SELECT status, attempts, last_error FROM outbox WHERE id = 'uncertain-target'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .unwrap();
    assert_eq!(status1, "uncertain");
    assert_eq!(attempts1, 1);
    assert!(last_err1.unwrap().contains("timeout"));

    // Verify delivery attempts row recorded outcome 'timeout'
    let outcome: String = conn
        .query_row(
            "SELECT outcome FROM delivery_attempts WHERE outbox_id = 'uncertain-target'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(outcome, "timeout");

    // 4: Turn OFF mock timeout and call flush_queue again: ZERO automatic retries!
    coachify_core::comms::set_mock_timeout(false);

    let (flush_res2, _) = dispatch_cmd(
        &mut conn,
        "comms.flush_queue",
        json!({ "force_bypass_quiet": true }),
        Some(&token),
        None,
    );
    assert_eq!(flush_res2["ok"], true);
    assert_eq!(
        flush_res2["data"]["processed_count"], 0,
        "Uncertain message must never be retried automatically"
    );

    // 5: User explicitly triggers resend via comms.resend_message
    let (resend_res, _) = dispatch_cmd(
        &mut conn,
        "comms.resend_message",
        json!({ "id": "uncertain-target" }),
        Some(&token),
        None,
    );
    assert_eq!(resend_res["ok"], true);

    // Now flush_queue picks it up and sends successfully!
    let (flush_res3, _) = dispatch_cmd(
        &mut conn,
        "comms.flush_queue",
        json!({ "force_bypass_quiet": true }),
        Some(&token),
        None,
    );
    assert_eq!(flush_res3["ok"], true);
    assert_eq!(flush_res3["data"]["processed_count"], 1);

    let status2: String = conn
        .query_row(
            "SELECT status FROM outbox WHERE id = 'uncertain-target'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(status2, "sent");
}
