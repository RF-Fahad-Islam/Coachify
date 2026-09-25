use coachify_core::ai::privacy::{PrivacyRedactionEngine, PrivacySettings};
use coachify_core::comms::{self, SaveProviderConfigPayload};
use coachify_core::crypto;
use coachify_core::db;
use coachify_core::diagnostics;
use coachify_core::dispatcher::{self, ChangeEvent};
use serde_json::json;
use std::io::Read;
use tempfile::tempdir;

fn setup_test_db() -> (tempfile::TempDir, rusqlite::Connection) {
    let dir = tempdir().expect("create temp dir");
    let db_path = dir.path().join("test_encrypted_secrets.db");
    let conn = db::open_connection_at(&db_path).expect("open test db");
    db::apply_all_migrations(&conn).expect("apply all migrations");
    (dir, conn)
}

fn dispatch_cmd(
    conn: &mut rusqlite::Connection,
    command: &str,
    payload: serde_json::Value,
    token: Option<&str>,
) -> (serde_json::Value, Vec<ChangeEvent>) {
    let payload_str = serde_json::to_string(&payload).unwrap();
    let (res_str, events) = dispatcher::dispatch(conn, command, &payload_str, token, None);
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
    );
    assert_eq!(res["ok"], true);

    let (res, _) = dispatch_cmd(
        conn,
        "auth.login",
        json!({
            "username": "owner",
            "password": "Password123!"
        }),
        None,
    );
    assert_eq!(res["ok"], true);
    res["data"]["session_token"].as_str().unwrap().to_string()
}

#[test]
fn test_provider_and_ai_secrets_encrypted_at_rest_and_config_json_sanitized() {
    crypto::set_mock_dpapi_force(true);
    crypto::set_mock_dpapi_failure(false);
    let (_dir, mut conn) = setup_test_db();
    let token = bootstrap_and_login(&mut conn);

    let raw_sms_secret = "sk-sms-super-secret-gateway-key-998877";
    let raw_smtp_password = "smtp-master-password-445566";
    let raw_ai_token = "sk-openrouter-secret-ai-token-112233";

    // 1. Save SMS provider config with explicit secret AND accidental secret in config_json
    comms::save_provider_config(
        &conn,
        &SaveProviderConfigPayload {
            channel: "sms".to_string(),
            adapter: "bulksmsbd".to_string(),
            is_active: true,
            config_json: json!({
                "sender_id": "8809617600000",
                "api_key": raw_sms_secret
            })
            .to_string(),
            secret: Some(raw_sms_secret.to_string()),
        },
    )
    .expect("save sms provider config");

    // 2. Save Email provider where password was only passed inside config_json (no explicit secret param)
    comms::save_provider_config(
        &conn,
        &SaveProviderConfigPayload {
            channel: "email".to_string(),
            adapter: "smtp".to_string(),
            is_active: true,
            config_json: json!({
                "host": "smtp.coaching.bd",
                "port": 587,
                "smtp_pass": raw_smtp_password
            })
            .to_string(),
            secret: None,
        },
    )
    .expect("save email provider config");

    // 3. Save AI Gateway credentials
    let (ai_res, _) = dispatch_cmd(
        &mut conn,
        "ai.update_gateway_config",
        json!({
            "gateway_url": "https://openrouter.ai/api/v1",
            "token": raw_ai_token,
            "openrouter_key": raw_ai_token
        }),
        Some(&token),
    );
    assert_eq!(ai_res["ok"], true);

    // Verify raw SQLite columns contain ZERO plaintext occurrences of any of the 3 secrets
    let rows: Vec<(String, String, Option<String>)> = {
        let mut stmt = conn
            .prepare("SELECT channel, config_json, secret_ref FROM provider_configs")
            .unwrap();
        stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
            .unwrap()
            .map(|r| r.unwrap())
            .collect()
    };

    assert_eq!(rows.len(), 2);
    for (_channel, cfg_json, secret_ref) in &rows {
        assert!(!cfg_json.contains(raw_sms_secret));
        assert!(!cfg_json.contains(raw_smtp_password));
        assert!(cfg_json.contains("[REDACTED_TO_DPAPI]"));
        let sec = secret_ref.as_ref().expect("secret_ref must be populated");
        assert!(sec.starts_with("DPAPI_V1:"));
        assert!(!sec.contains(raw_sms_secret));
        assert!(!sec.contains(raw_smtp_password));
    }

    // Verify in-memory decryption on the same machine succeeds
    let decrypted_sms = comms::decrypt_provider_secret(&conn, "sms", "bulksmsbd")
        .unwrap()
        .unwrap();
    assert_eq!(decrypted_sms, raw_sms_secret);

    let decrypted_smtp = comms::decrypt_provider_secret(&conn, "email", "smtp")
        .unwrap()
        .unwrap();
    assert_eq!(decrypted_smtp, raw_smtp_password);

    // Check security.get_secrets_health via RPC
    let (health_res, _) = dispatch_cmd(
        &mut conn,
        "security.get_secrets_health",
        json!({}),
        Some(&token),
    );
    assert_eq!(health_res["ok"], true);
    assert_eq!(health_res["data"]["all_healthy"], true);
    assert_eq!(health_res["data"]["zero_plaintext_leaks"], true);
    assert_eq!(health_res["data"]["foreign_machine_count"], 0);
    assert_eq!(health_res["data"]["configured_count"].as_u64().unwrap(), 3);

    // AC-2: Simulate copying database to another machine (DPAPI unprotect fails)
    crypto::set_mock_dpapi_failure(true);

    let foreign_decrypt = comms::decrypt_provider_secret(&conn, "sms", "bulksmsbd");
    assert!(foreign_decrypt.is_err());
    assert!(foreign_decrypt
        .unwrap_err()
        .to_string()
        .contains("provider_secret_foreign_machine"));

    let (health_foreign, _) = dispatch_cmd(
        &mut conn,
        "security.get_secrets_health",
        json!({}),
        Some(&token),
    );
    assert_eq!(health_foreign["ok"], true);
    assert_eq!(health_foreign["data"]["all_healthy"], false);
    assert_eq!(
        health_foreign["data"]["foreign_machine_count"]
            .as_u64()
            .unwrap(),
        3
    );

    crypto::set_mock_dpapi_failure(false);
    crypto::set_mock_dpapi_force(false);
}

#[test]
fn test_diagnostics_bundle_and_logs_and_ai_prompts_contain_zero_secrets() {
    crypto::set_mock_dpapi_force(true);
    crypto::set_mock_dpapi_failure(false);
    let (db_dir, conn) = setup_test_db();

    let leaked_api_key = "sk-live-secret9876543210abcdef";
    let leaked_bearer = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.secretPayloadToken";
    let leaked_recovery = "CKREC-ABCD-EFGH-JKLM-NPQR-STVW-XYZ2";
    let leaked_smtp_pass = "SuperSecretSmtpPass99!";

    // Save a provider with secret
    comms::save_provider_config(
        &conn,
        &SaveProviderConfigPayload {
            channel: "sms".to_string(),
            adapter: "greenweb".to_string(),
            is_active: true,
            config_json: json!({
                "sender": "Coachify",
                "api_key": leaked_api_key
            })
            .to_string(),
            secret: Some(leaked_api_key.to_string()),
        },
    )
    .unwrap();

    // Write a log file containing accidental secret patterns and phone numbers
    let logs_dir = db_dir.path().join("logs");
    std::fs::create_dir_all(&logs_dir).unwrap();
    let log_file = logs_dir.join("2026-09-25.log");
    std::fs::write(
        &log_file,
        format!(
            "2026-09-25T10:00:00Z ERROR comms: Dispatch failed to 01712345678 with key={} and Authorization: Bearer {} recovery={} smtp_pass=\"{}\"\n",
            leaked_api_key, leaked_bearer, leaked_recovery, leaked_smtp_pass
        ),
    )
    .unwrap();

    // Export diagnostics bundle
    let out_dir = tempdir().unwrap();
    let zip_path_str = diagnostics::export_diagnostics_bundle(
        &conn,
        db_dir.path(),
        &out_dir.path().display().to_string(),
    )
    .expect("export diagnostics bundle");

    let zip_file = std::fs::File::open(&zip_path_str).unwrap();
    let mut archive = zip::ZipArchive::new(zip_file).unwrap();

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).unwrap();
        let mut content = String::new();
        entry.read_to_string(&mut content).unwrap();

        assert!(
            !content.contains(leaked_api_key),
            "Entry {} leaked API key: {}",
            entry.name(),
            content
        );
        assert!(
            !content.contains(leaked_bearer),
            "Entry {} leaked Bearer token: {}",
            entry.name(),
            content
        );
        assert!(
            !content.contains(leaked_recovery),
            "Entry {} leaked Recovery key: {}",
            entry.name(),
            content
        );
        assert!(
            !content.contains(leaked_smtp_pass),
            "Entry {} leaked SMTP password: {}",
            entry.name(),
            content
        );
        assert!(
            !content.contains("01712345678"),
            "Entry {} leaked BD phone number: {}",
            entry.name(),
            content
        );
    }

    // Verify AI PrivacyRedactionEngine::scrub_prompt also scrubs secrets even if phone numbers are allowed
    let mut engine = PrivacyRedactionEngine::new(PrivacySettings {
        allow_phone_numbers: true,
        ..Default::default()
    });
    let prompt_input = format!(
        "Please debug why api_key={} and Bearer {} with recovery {} failed",
        leaked_api_key, leaked_bearer, leaked_recovery
    );
    let scrubbed_prompt = engine.scrub_prompt(&prompt_input);
    assert!(!scrubbed_prompt.contains(leaked_api_key));
    assert!(!scrubbed_prompt.contains(leaked_bearer));
    assert!(!scrubbed_prompt.contains(leaked_recovery));
    assert!(engine.redacted_categories.contains(&"secrets".to_string()));

    crypto::set_mock_dpapi_force(false);
}
