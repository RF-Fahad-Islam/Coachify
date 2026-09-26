use coachify_core::db;
use coachify_core::dispatcher::dispatch_at;
use coachify_core::lan::{
    get_firewall_guidance, get_or_create_host_config, set_sleep_inhibition, LanHostStatus,
};
use rusqlite::Connection;
use serde_json::{json, Value};
use std::net::IpAddr;
use std::sync::Mutex;

static SERIAL_TEST_MUTEX: Mutex<()> = Mutex::new(());

fn setup_test_env() -> (Connection, String) {
    let mut conn = Connection::open_in_memory().expect("open in-memory db");
    db::apply_all_migrations(&conn).expect("apply all migrations");

    let now = "2026-09-26T10:00:00Z";
    conn.execute(
        "INSERT INTO institution (id, name, address, phones, currency, timezone, created_at, updated_at)
         VALUES (1, 'Apex Coaching Center', 'Dhaka', '01700000000', 'BDT', 'Asia/Dhaka', ?1, ?1)",
        [now],
    )
    .unwrap();

    let boot_payload = json!({
        "username": "owner",
        "display_name": "Principal Owner",
        "password": "OwnerPassword123!"
    })
    .to_string();

    let (res_str, _) = dispatch_at(
        &mut conn,
        "system.bootstrap_owner",
        &boot_payload,
        None,
        None,
        now,
    );
    let res: Value = serde_json::from_str(&res_str).unwrap();
    assert!(res["ok"].as_bool().unwrap(), "bootstrap failed: {res}");
    let token = res["data"]["session_token"].as_str().unwrap().to_string();

    (conn, token)
}

#[test]
fn test_cert_generation_and_persistence_ac1() {
    let (conn, _token) = setup_test_env();
    let now = "2026-09-26T10:05:00Z";

    // 1. Initial config fetch generates X.509 cert and key
    let config1 = get_or_create_host_config(&conn, now).expect("generate initial config");
    assert_eq!(config1.id, "default");
    assert!(!config1.is_enabled);
    assert_eq!(config1.port, 50443);
    assert!(config1.tls_cert_pem.contains("BEGIN CERTIFICATE"));
    assert!(config1.tls_key_pem.contains("BEGIN PRIVATE KEY"));
    assert!(config1.tls_fingerprint_sha256.starts_with("SHA256:"));
    assert_eq!(config1.tls_fingerprint_sha256.split(':').count(), 33); // "SHA256" + 32 hex bytes

    // 2. Subsequent fetch returns identical key and cert without re-generating
    let config2 =
        get_or_create_host_config(&conn, "2026-09-26T10:06:00Z").expect("load existing config");
    assert_eq!(config1.tls_cert_pem, config2.tls_cert_pem);
    assert_eq!(config1.tls_key_pem, config2.tls_key_pem);
    assert_eq!(
        config1.tls_fingerprint_sha256,
        config2.tls_fingerprint_sha256
    );
}

#[test]
fn test_license_entitlement_gating_ac2() {
    let (mut conn, token) = setup_test_env();
    let now = "2026-09-26T10:01:00Z";

    // 1. Fresh database has automatic 14-day trial which includes LAN entitlement
    let (res_trial, _) = dispatch_at(
        &mut conn,
        "lan.get_host_status",
        "{}",
        Some(&token),
        None,
        now,
    );
    let env_trial: Value = serde_json::from_str(&res_trial).unwrap();
    assert!(env_trial["ok"].as_bool().unwrap());
    assert!(env_trial["data"]["license_entitled"].as_bool().unwrap());

    // 2. Overwrite license with single-PC 'local' plan (without lan_multi_device capability)
    conn.execute(
        "UPDATE licenses SET tier = 'local', capabilities_json = '[\"core\"]' WHERE is_active = 1",
        [],
    )
    .unwrap();

    // 3. Status now reflects unentitled
    let (res_status, _) = dispatch_at(
        &mut conn,
        "lan.get_host_status",
        "{}",
        Some(&token),
        None,
        now,
    );
    let env_status: Value = serde_json::from_str(&res_status).unwrap();
    assert!(env_status["ok"].as_bool().unwrap());
    assert!(!env_status["data"]["license_entitled"].as_bool().unwrap());

    // 4. Starting host on single-PC tier fails with LICENSE_FEATURE_NOT_ENTITLED
    let start_payload = json!({ "port": 50450 }).to_string();
    let (res_start, _) = dispatch_at(
        &mut conn,
        "lan.start_host",
        &start_payload,
        Some(&token),
        None,
        now,
    );
    let env_start: Value = serde_json::from_str(&res_start).unwrap();
    assert!(!env_start["ok"].as_bool().unwrap());
    assert_eq!(
        env_start["error"]["code"].as_str().unwrap(),
        "LICENSE_FEATURE_NOT_ENTITLED"
    );

    // 5. Upgrade tier to 'local_pro'
    conn.execute(
        "UPDATE licenses SET tier = 'local_pro', capabilities_json = '[\"core\", \"lan_multi_device\"]' WHERE is_active = 1",
        [],
    )
    .unwrap();

    let (res_status_pro, _) = dispatch_at(
        &mut conn,
        "lan.get_host_status",
        "{}",
        Some(&token),
        None,
        now,
    );
    let env_status_pro: Value = serde_json::from_str(&res_status_pro).unwrap();
    assert!(env_status_pro["ok"].as_bool().unwrap());
    assert!(env_status_pro["data"]["license_entitled"]
        .as_bool()
        .unwrap());
}

#[test]
fn test_host_listener_lifecycle_ac3_ac4_ac5() {
    let _lock = SERIAL_TEST_MUTEX.lock().unwrap();
    let (mut conn, token) = setup_test_env();
    let now = "2026-09-26T10:02:00Z";

    // Use a test port
    let test_port: u16 = 50488;
    let start_payload = json!({
        "port": test_port,
        "bind_address": "127.0.0.1"
    })
    .to_string();

    // 1. Start host
    let (start_res, events) = dispatch_at(
        &mut conn,
        "lan.start_host",
        &start_payload,
        Some(&token),
        None,
        now,
    );
    let env_start: Value = serde_json::from_str(&start_res).unwrap();
    assert!(
        env_start["ok"].as_bool().unwrap(),
        "Failed to start host: {start_res}"
    );
    let host_status: LanHostStatus = serde_json::from_value(env_start["data"].clone()).unwrap();
    assert!(host_status.is_running);
    assert_eq!(host_status.port, test_port);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].topic, "lan:changed");

    // 2. Ping host via manual IP ping (AC-7)
    let ping_payload = json!({
        "ip": "127.0.0.1",
        "port": test_port,
        "expected_fingerprint": host_status.fingerprint
    })
    .to_string();

    let (ping_res, _) = dispatch_at(
        &mut conn,
        "lan.ping_host",
        &ping_payload,
        Some(&token),
        None,
        now,
    );
    let env_ping: Value = serde_json::from_str(&ping_res).unwrap();
    assert!(env_ping["ok"].as_bool().unwrap(), "Ping failed: {ping_res}");
    assert!(env_ping["data"]["reachable"].as_bool().unwrap());
    assert_eq!(env_ping["data"]["product"].as_str().unwrap(), "coachify");
    assert_eq!(env_ping["data"]["protocol_version"].as_u64().unwrap(), 1);
    assert!(env_ping["data"]["fingerprint_matched"].as_bool().unwrap());

    // 2b. Ping host with mismatched fingerprint
    let wrong_fp_payload = json!({
        "ip": "127.0.0.1",
        "port": test_port,
        "expected_fingerprint": "SHA256:00:11:22:33:44:55:66:77:88:99:AA:BB:CC:DD:EE:FF:00:11:22:33:44:55:66:77:88:99:AA:BB:CC:DD:EE:FF"
    })
    .to_string();

    let (ping_wrong_res, _) = dispatch_at(
        &mut conn,
        "lan.ping_host",
        &wrong_fp_payload,
        Some(&token),
        None,
        now,
    );
    let env_wrong_ping: Value = serde_json::from_str(&ping_wrong_res).unwrap();
    assert!(env_wrong_ping["ok"].as_bool().unwrap());
    assert!(env_wrong_ping["data"]["reachable"].as_bool().unwrap());
    assert!(!env_wrong_ping["data"]["fingerprint_matched"]
        .as_bool()
        .unwrap());

    // 3. Stop host (AC-3)
    let (stop_res, stop_events) =
        dispatch_at(&mut conn, "lan.stop_host", "{}", Some(&token), None, now);
    let env_stop: Value = serde_json::from_str(&stop_res).unwrap();
    assert!(
        env_stop["ok"].as_bool().unwrap(),
        "Failed to stop host: {stop_res}"
    );
    assert!(!env_stop["data"]["is_running"].as_bool().unwrap());
    assert_eq!(stop_events.len(), 1);

    // 4. Ping after stop returns unreachable
    let (ping_after_stop, _) = dispatch_at(
        &mut conn,
        "lan.ping_host",
        &ping_payload,
        Some(&token),
        None,
        now,
    );
    let env_after: Value = serde_json::from_str(&ping_after_stop).unwrap();
    assert!(env_after["ok"].as_bool().unwrap());
    assert!(!env_after["data"]["reachable"].as_bool().unwrap());
}

#[test]
fn test_rate_limiter_for_hello_endpoint_ac4() {
    use coachify_core::lan::RateLimiter;
    use std::net::Ipv4Addr;

    let limiter = RateLimiter::new();
    let ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 55));

    // First 30 requests within 60s window are allowed
    for i in 1..=30 {
        assert!(limiter.check(ip), "Request {i} should be allowed");
    }

    // 31st request is blocked
    assert!(!limiter.check(ip), "Request 31 should be rejected with 429");
}

#[test]
fn test_firewall_guidance_ac8() {
    let guidance = get_firewall_guidance();
    assert!(!guidance.profile.is_empty());
    assert!(guidance
        .netsh_command
        .contains("dir=in action=allow protocol=TCP localport=50443 profile=private"));
    assert!(guidance
        .powershell_command
        .contains("Set-NetConnectionProfile -NetworkCategory Private"));
}

#[test]
fn test_sleep_inhibition_lifecycle_ac9() {
    set_sleep_inhibition(true);
    set_sleep_inhibition(false);
}

#[test]
fn test_port_conflict_detection() {
    let _lock = SERIAL_TEST_MUTEX.lock().unwrap();
    let (mut conn, token) = setup_test_env();
    let now = "2026-09-26T10:05:00Z";

    // Bind a standard TCP socket to simulate an occupied port
    let occupied = std::net::TcpListener::bind("127.0.0.1:0").expect("bind mock port");
    let port = occupied.local_addr().unwrap().port();

    let start_payload = json!({
        "port": port,
        "bind_address": "127.0.0.1"
    })
    .to_string();

    let (res, _) = dispatch_at(
        &mut conn,
        "lan.start_host",
        &start_payload,
        Some(&token),
        None,
        now,
    );
    let env: Value = serde_json::from_str(&res).unwrap();
    assert!(!env["ok"].as_bool().unwrap());
    assert_eq!(env["error"]["code"].as_str().unwrap(), "LAN_PORT_IN_USE");
}

#[test]
fn test_update_host_config_and_validation() {
    let (mut conn, token) = setup_test_env();
    let now = "2026-09-26T10:05:00Z";

    // 1. Valid config update
    let valid_payload = json!({
        "port": 50444,
        "bind_address": "127.0.0.1",
        "prevent_sleep": false,
        "auto_start_on_boot": true
    })
    .to_string();

    let (res, _) = dispatch_at(
        &mut conn,
        "lan.update_host_config",
        &valid_payload,
        Some(&token),
        None,
        now,
    );
    let env: Value = serde_json::from_str(&res).unwrap();
    assert!(env["ok"].as_bool().unwrap());
    assert_eq!(env["data"]["port"].as_u64().unwrap(), 50444);
    assert_eq!(env["data"]["bind_address"].as_str().unwrap(), "127.0.0.1");
    assert!(!env["data"]["prevent_sleep"].as_bool().unwrap());
    assert!(env["data"]["auto_start_on_boot"].as_bool().unwrap());

    // 2. Invalid port (below 1024)
    let invalid_low = json!({ "port": 80 }).to_string();
    let (res_low, _) = dispatch_at(
        &mut conn,
        "lan.update_host_config",
        &invalid_low,
        Some(&token),
        None,
        now,
    );
    let env_low: Value = serde_json::from_str(&res_low).unwrap();
    assert!(!env_low["ok"].as_bool().unwrap());
    assert_eq!(
        env_low["error"]["code"].as_str().unwrap(),
        "INVALID_PORT_RANGE"
    );

    // 3. Invalid port (above 65535 - exceeds u16 integer range)
    let invalid_high = json!({ "port": 70000 }).to_string();
    let (res_high, _) = dispatch_at(
        &mut conn,
        "lan.update_host_config",
        &invalid_high,
        Some(&token),
        None,
        now,
    );
    let env_high: Value = serde_json::from_str(&res_high).unwrap();
    assert!(!env_high["ok"].as_bool().unwrap());
    assert_eq!(
        env_high["error"]["code"].as_str().unwrap(),
        "validation_error"
    );
}

#[test]
fn test_discover_hosts_timeout_resilience() {
    let payload = json!({ "timeout_ms": 100 }).to_string();
    let (mut conn, token) = setup_test_env();
    let now = "2026-09-26T10:05:00Z";

    let (res, _) = dispatch_at(
        &mut conn,
        "lan.discover_hosts",
        &payload,
        Some(&token),
        None,
        now,
    );
    let env: Value = serde_json::from_str(&res).unwrap();
    assert!(env["ok"].as_bool().unwrap());
    assert!(env["data"]["hosts"].is_array());
}
