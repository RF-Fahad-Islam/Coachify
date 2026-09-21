use coachify_core::crypto::{set_mock_dpapi_failure, set_mock_dpapi_force};
use coachify_core::handle_rpc_with_events;
use coachify_core::setup::{
    DetectDrivesResponse, GenerateRecoveryPreviewResponse, InitializeInstitutionPayload,
    InitializeInstitutionResponse, InspectDatabasePayload, InspectDatabaseResponse,
    ReprintSheetPayload, ReprintSheetResponse, SetupStatusResponse, UnlockWithKeyPayload,
    UnlockWithKeyResponse, ValidatePathPayload,
};
use coachify_core::storage::PathValidationResult;
use coachify_core::{LoginPayload, LoginResponse, RpcEnvelope};
use tempfile::tempdir;

#[test]
fn test_setup_status_and_drive_detection() {
    set_mock_dpapi_force(true);

    // 1. Uninitialized setup status
    let (status_json, _) = handle_rpc_with_events("setup.get_status", "{}", None, None);
    let status_env: RpcEnvelope<SetupStatusResponse> = serde_json::from_str(&status_json).unwrap();
    assert!(status_env.ok);
    let status = status_env.data.unwrap();
    assert!(!status.default_data_dir.is_empty());

    // 2. Detect drives
    let (drives_json, _) = handle_rpc_with_events("setup.detect_drives", "{}", None, None);
    let drives_env: RpcEnvelope<DetectDrivesResponse> = serde_json::from_str(&drives_json).unwrap();
    assert!(drives_env.ok);
    let drives = drives_env.data.unwrap();
    assert!(!drives.drives.is_empty());
    assert!(!drives.recommended_path.is_empty());

    // 3. Validate path
    let temp = tempdir().unwrap();
    let val_payload = ValidatePathPayload {
        path: temp.path().to_string_lossy().to_string(),
        kind: "data".to_string(),
        expected_institution_id: None,
    };
    let (val_json, _) = handle_rpc_with_events(
        "setup.validate_path",
        &serde_json::to_string(&val_payload).unwrap(),
        None,
        None,
    );
    let val_env: RpcEnvelope<PathValidationResult> = serde_json::from_str(&val_json).unwrap();
    assert!(val_env.ok);
    assert!(val_env.data.unwrap().valid);

    set_mock_dpapi_force(false);
}

#[test]
fn test_recovery_preview_and_confirmation_gate() {
    set_mock_dpapi_force(true);

    // 1. Generate preview
    let (prev_json, _) =
        handle_rpc_with_events("setup.generate_recovery_preview", "{}", None, None);
    let prev_env: RpcEnvelope<GenerateRecoveryPreviewResponse> =
        serde_json::from_str(&prev_json).unwrap();
    assert!(prev_env.ok);
    let prev = prev_env.data.unwrap();
    assert!(prev.recovery_key.starts_with("CKREC-"));
    assert_eq!(prev.checksum.len(), 4);

    // 2. Mismatched confirmation code must be rejected
    let temp = tempdir().unwrap();
    let payload = InitializeInstitutionPayload {
        institution_name: "Confirmation Test Academy".to_string(),
        institution_name_bn: None,
        address: None,
        phone: None,
        email: None,
        academic_session_name: "2026".to_string(),
        student_id_prefix: None,
        receipt_prefix: None,
        data_dir: temp.path().to_string_lossy().to_string(),
        backup_destination: None,
        owner_display_name: "Owner Name".to_string(),
        owner_username: "owner_gate".to_string(),
        owner_password: "super_secure_password_123".to_string(),
        recovery_key: prev.recovery_key.clone(),
        recovery_key_confirmation: "XXXX".to_string(), // deliberately wrong
    };

    let (init_json, _) = handle_rpc_with_events(
        "setup.initialize_institution",
        &serde_json::to_string(&payload).unwrap(),
        None,
        None,
    );
    let init_env: RpcEnvelope<InitializeInstitutionResponse> =
        serde_json::from_str(&init_json).unwrap();
    assert!(!init_env.ok);
    let err = init_env.error.unwrap();
    assert_eq!(err.code, "setup_failed");
    assert!(err.message.contains("confirmation mismatch"));

    set_mock_dpapi_force(false);
}

#[test]
fn test_full_setup_and_disaster_recovery_workflow() {
    set_mock_dpapi_force(true);

    let data_temp = tempdir().unwrap();
    let backup_temp = tempdir().unwrap();

    // 1. Generate recovery preview
    let (prev_json, _) =
        handle_rpc_with_events("setup.generate_recovery_preview", "{}", None, None);
    let prev_env: RpcEnvelope<GenerateRecoveryPreviewResponse> =
        serde_json::from_str(&prev_json).unwrap();
    let preview = prev_env.data.unwrap();

    // 2. Initialize Institution
    let payload = InitializeInstitutionPayload {
        institution_name: "Uttara Model College".to_string(),
        institution_name_bn: Some("উত্তরা মডেল কলেজ".to_string()),
        address: Some("Sector 4, Uttara, Dhaka".to_string()),
        phone: Some("01811223344".to_string()),
        email: Some("info@uttaramodel.edu.bd".to_string()),
        academic_session_name: "2026".to_string(),
        student_id_prefix: Some("UMC-".to_string()),
        receipt_prefix: Some("REC-".to_string()),
        data_dir: data_temp.path().to_string_lossy().to_string(),
        backup_destination: Some(backup_temp.path().to_string_lossy().to_string()),
        owner_display_name: "Principal Rahman".to_string(),
        owner_username: "principal".to_string(),
        owner_password: "principal_password_secure_99".to_string(),
        recovery_key: preview.recovery_key.clone(),
        recovery_key_confirmation: preview.checksum.clone(),
    };

    let (init_json, _) = handle_rpc_with_events(
        "setup.initialize_institution",
        &serde_json::to_string(&payload).unwrap(),
        None,
        None,
    );
    let init_env: RpcEnvelope<InitializeInstitutionResponse> =
        serde_json::from_str(&init_json).unwrap();
    assert!(
        init_env.ok,
        "Initialization should succeed: {:?}",
        init_env.error
    );
    let init_data = init_env.data.unwrap();
    assert_eq!(init_data.user.username, "principal");
    assert_eq!(init_data.user.display_name, "Principal Rahman");
    assert!(!init_data.session_token.is_empty());

    let inst_id = init_data.institution_id;
    let db_dir = data_temp.path().join(&inst_id).join("db");
    let key_envelope_file = db_dir.join("key.envelope");
    let main_db_file = db_dir.join("main.db");

    assert!(key_envelope_file.exists(), "key.envelope must exist");
    assert!(main_db_file.exists(), "main.db must exist");

    // Verify backup marker written
    let marker_file = backup_temp.path().join(".coachify-backup-target");
    assert!(marker_file.exists(), "Backup target marker must be created");

    // 3. Inspect database on current (healthy) machine
    let inspect_payload = InspectDatabasePayload {
        db_dir: Some(db_dir.to_string_lossy().to_string()),
    };
    let (inspect_json, _) = handle_rpc_with_events(
        "recovery.inspect_database",
        &serde_json::to_string(&inspect_payload).unwrap(),
        None,
        None,
    );
    let inspect_env: RpcEnvelope<InspectDatabaseResponse> =
        serde_json::from_str(&inspect_json).unwrap();
    assert!(inspect_env.ok);
    let inspect = inspect_env.data.unwrap();
    assert!(inspect.database_exists);
    assert!(inspect.has_key_envelope);
    assert!(inspect.can_unwrap_dpapi);
    assert!(!inspect.needs_recovery);
    assert_eq!(
        inspect.institution_name,
        Some("Uttara Model College".to_string())
    );

    // 4. Reprint Sheet test with Owner session
    let reprint_payload = ReprintSheetPayload {
        owner_password: "principal_password_secure_99".to_string(),
    };
    let (reprint_json, _) = handle_rpc_with_events(
        "recovery.reprint_sheet",
        &serde_json::to_string(&reprint_payload).unwrap(),
        Some(&init_data.session_token),
        None,
    );
    let reprint_env: RpcEnvelope<ReprintSheetResponse> =
        serde_json::from_str(&reprint_json).unwrap();
    assert!(
        reprint_env.ok,
        "Reprint should succeed: {:?}",
        reprint_env.error
    );
    let reprint = reprint_env.data.unwrap();
    assert_eq!(reprint.institution_name, "Uttara Model College");
    assert_eq!(reprint.owner_username, "principal");
    assert_eq!(reprint.recovery_key_checksum, preview.checksum);

    // Verify recovery.reprint_sheet is audited
    let (audit_list_json, _) =
        handle_rpc_with_events("audit.list", "{}", Some(&init_data.session_token), None);
    let audit_env: serde_json::Value = serde_json::from_str(&audit_list_json).unwrap();
    let has_reprint_audit = audit_env["data"]["items"]
        .as_array()
        .map(|items| {
            items
                .iter()
                .any(|item| item["action"].as_str() == Some("recovery.reprint_sheet"))
        })
        .unwrap_or(false);
    assert!(
        has_reprint_audit,
        "recovery.reprint_sheet must create a permanent audit log entry"
    );

    // Reprint with incorrect password must fail
    let bad_reprint_payload = ReprintSheetPayload {
        owner_password: "wrong_password".to_string(),
    };
    let (bad_reprint_json, _) = handle_rpc_with_events(
        "recovery.reprint_sheet",
        &serde_json::to_string(&bad_reprint_payload).unwrap(),
        Some(&init_data.session_token),
        None,
    );
    let bad_reprint_env: RpcEnvelope<ReprintSheetResponse> =
        serde_json::from_str(&bad_reprint_json).unwrap();
    assert!(!bad_reprint_env.ok);
    assert_eq!(bad_reprint_env.error.unwrap().code, "invalid_password");

    // 5. Simulate Disaster: Database moved to foreign PC where DPAPI fails
    set_mock_dpapi_failure(true);

    let (foreign_inspect_json, _) = handle_rpc_with_events(
        "recovery.inspect_database",
        &serde_json::to_string(&inspect_payload).unwrap(),
        None,
        None,
    );
    let foreign_inspect_env: RpcEnvelope<InspectDatabaseResponse> =
        serde_json::from_str(&foreign_inspect_json).unwrap();
    assert!(foreign_inspect_env.ok);
    let foreign_inspect = foreign_inspect_env.data.unwrap();
    assert!(foreign_inspect.database_exists);
    assert!(foreign_inspect.has_key_envelope);
    assert!(!foreign_inspect.can_unwrap_dpapi);
    assert!(foreign_inspect.needs_recovery);

    // 6. Perform Emergency Recovery with Recovery Key + Reset Owner Password
    // Disable failure simulation to simulate local DPAPI on the new machine
    set_mock_dpapi_failure(false);

    let unlock_payload = UnlockWithKeyPayload {
        db_dir: Some(db_dir.to_string_lossy().to_string()),
        recovery_key: preview.recovery_key.clone(),
        reset_owner_password: Some("new_recovered_principal_pass_2026".to_string()),
    };
    let (unlock_json, _) = handle_rpc_with_events(
        "recovery.unlock_with_key",
        &serde_json::to_string(&unlock_payload).unwrap(),
        None,
        None,
    );
    let unlock_env: RpcEnvelope<UnlockWithKeyResponse> =
        serde_json::from_str(&unlock_json).unwrap();
    assert!(
        unlock_env.ok,
        "Unlock should succeed: {:?}",
        unlock_env.error
    );
    let unlock = unlock_env.data.unwrap();
    assert!(unlock.success);
    assert!(unlock.owner_password_reset);
    assert_eq!(unlock.institution_name, "Uttara Model College");

    // 7. Inspect database again - now DPAPI should succeed on new machine
    let (post_unlock_inspect_json, _) = handle_rpc_with_events(
        "recovery.inspect_database",
        &serde_json::to_string(&inspect_payload).unwrap(),
        None,
        None,
    );
    let post_inspect: RpcEnvelope<InspectDatabaseResponse> =
        serde_json::from_str(&post_unlock_inspect_json).unwrap();
    assert!(post_inspect.data.unwrap().can_unwrap_dpapi);

    // 8. Log in with the newly reset owner password
    let login_payload = LoginPayload {
        username: "principal".to_string(),
        password: "new_recovered_principal_pass_2026".to_string(),
        device_name: Some("Recovered PC".to_string()),
    };
    let (login_json, _) = handle_rpc_with_events(
        "auth.login",
        &serde_json::to_string(&login_payload).unwrap(),
        None,
        None,
    );
    let login_env: RpcEnvelope<LoginResponse> = serde_json::from_str(&login_json).unwrap();
    assert!(
        login_env.ok,
        "Login with reset password should succeed: {:?}",
        login_env.error
    );
    let login_data = login_env.data.unwrap();
    assert_eq!(login_data.user.username, "principal");
    assert!(login_data.roles.contains(&"Owner".to_string()));

    set_mock_dpapi_force(false);
}
