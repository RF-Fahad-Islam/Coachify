use coachify_core::ai::gateway::{
    clear_mock_gateway, set_mock_gateway_response, GatewayChoice, GatewayChoiceMessage,
    GatewayFunctionCall, GatewayResponse, GatewayToolCall, GatewayUsage,
};
use coachify_core::db;
use coachify_core::dispatcher::{self, ChangeEvent};
use serde_json::json;
use tempfile::tempdir;

fn setup_test_db() -> (tempfile::TempDir, rusqlite::Connection) {
    let dir = tempdir().expect("create temp dir");
    let db_path = dir.path().join("test_ai_foundation.db");
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
            "display_name": "Center Director",
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

static STUDENT_SEQ: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(80000);

fn create_student_helper(
    conn: &mut rusqlite::Connection,
    token: &str,
    name: &str,
) -> (String, String) {
    let seq = STUDENT_SEQ.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let student_phone = format!("0171{:07}", seq);
    let guardian_phone = format!("0181{:07}", seq);

    let (res, _) = dispatch_cmd(
        conn,
        "students.create",
        json!({
            "student": {
                "name": name,
                "gender": "male",
                "dob": "2008-01-01",
                "present_address": "Dhanmondi, Dhaka",
                "phone": student_phone,
                "guardians": [
                    {
                        "is_primary": true,
                        "receives_sms": true,
                        "new_guardian": {
                            "name": format!("Guardian of {}", name),
                            "relationship": "father",
                            "phone": guardian_phone
                        }
                    }
                ]
            }
        }),
        Some(token),
        None,
    );
    assert_eq!(res["ok"], true, "create student failed: {:?}", res);
    let id = res["data"]["id"].as_str().unwrap().to_string();
    let code = res["data"]["student_code"].as_str().unwrap().to_string();
    (id, code)
}

#[test]
fn test_wallet_balance_and_credit_transactions() {
    let (_dir, mut conn) = setup_test_db();
    let token = bootstrap_and_login(&mut conn);

    // Initial wallet balance (500 promotional credits granted by default migration)
    let (res, _) = dispatch_cmd(
        &mut conn,
        "ai.get_wallet_balance",
        json!({}),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true);
    assert_eq!(res["data"]["balance_credits"], 500);
    assert_eq!(res["data"]["is_active"], true);

    // Record credit grant (purchase)
    let (grant_res, events) = dispatch_cmd(
        &mut conn,
        "ai.record_credit_grant",
        json!({
            "amount": 500,
            "tx_type": "purchase",
            "reference_id": "inv_12345",
            "description": "Purchased 500 AI credits pack"
        }),
        Some(&token),
        None,
    );
    assert_eq!(grant_res["ok"], true);
    assert_eq!(grant_res["data"]["amount"], 500);
    assert_eq!(grant_res["data"]["balance_after"], 1000);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].topic, "ai_credit_wallets");

    // Check balance updated
    let (bal_res, _) = dispatch_cmd(
        &mut conn,
        "ai.get_wallet_balance",
        json!({}),
        Some(&token),
        None,
    );
    assert_eq!(bal_res["ok"], true);
    assert_eq!(bal_res["data"]["balance_credits"], 1000);

    // Ledger query
    let (ledger_res, _) = dispatch_cmd(
        &mut conn,
        "ai.get_credit_ledger",
        json!({ "limit": 10 }),
        Some(&token),
        None,
    );
    assert_eq!(ledger_res["ok"], true);
    let items = ledger_res["data"].as_array().unwrap();
    assert!(items.len() >= 2); // Initial grant + new purchase
    assert_eq!(items[0]["transaction_type"], "purchase");
    assert_eq!(items[0]["amount"], 500);

    // Sync credits
    let (sync_res, sync_events) =
        dispatch_cmd(&mut conn, "ai.sync_credits", json!({}), Some(&token), None);
    assert_eq!(sync_res["ok"], true);
    assert!(sync_res["data"]["last_synced_at"].is_string());
    assert_eq!(sync_events[0].topic, "ai_credit_wallets");
}

#[test]
fn test_privacy_settings_defaults_and_updates() {
    let (_dir, mut conn) = setup_test_db();
    let token = bootstrap_and_login(&mut conn);

    // Check conservative defaults: all false
    let (res, _) = dispatch_cmd(
        &mut conn,
        "ai.get_privacy_settings",
        json!({}),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true);
    assert_eq!(res["data"]["allow_student_names"], false);
    assert_eq!(res["data"]["allow_phone_numbers"], false);
    assert_eq!(res["data"]["allow_guardian_info"], false);
    assert_eq!(res["data"]["allow_financial_figures"], false);
    assert_eq!(res["data"]["allow_exam_marks"], false);
    assert_eq!(res["data"]["allow_staff_names"], false);

    // Update settings: allow exam marks and student names
    let (update_res, events) = dispatch_cmd(
        &mut conn,
        "ai.update_privacy_settings",
        json!({
            "allow_student_names": true,
            "allow_phone_numbers": false,
            "allow_guardian_info": false,
            "allow_financial_figures": false,
            "allow_exam_marks": true,
            "allow_staff_names": false
        }),
        Some(&token),
        None,
    );
    assert_eq!(update_res["ok"], true);
    assert_eq!(update_res["data"]["allow_student_names"], true);
    assert_eq!(update_res["data"]["allow_exam_marks"], true);
    assert_eq!(update_res["data"]["allow_phone_numbers"], false);
    assert_eq!(events[0].topic, "ai_privacy_settings");

    // Verify persisted
    let (verify_res, _) = dispatch_cmd(
        &mut conn,
        "ai.get_privacy_settings",
        json!({}),
        Some(&token),
        None,
    );
    assert_eq!(verify_res["ok"], true);
    assert_eq!(verify_res["data"]["allow_student_names"], true);
    assert_eq!(verify_res["data"]["allow_exam_marks"], true);
}

#[test]
fn test_gateway_configuration_and_testing() {
    let (_dir, mut conn) = setup_test_db();
    let token = bootstrap_and_login(&mut conn);

    // Get gateway config
    let (res, _) = dispatch_cmd(
        &mut conn,
        "ai.get_gateway_config",
        json!({}),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true);
    assert!(res["data"]["gateway_url"].is_string());

    // Update gateway config
    let (update_res, events) = dispatch_cmd(
        &mut conn,
        "ai.update_gateway_config",
        json!({
            "gateway_url": "https://openrouter.ai/api/v1",
            "token": null,
            "openrouter_key": "sk-or-v1-testkey12345"
        }),
        Some(&token),
        None,
    );
    assert_eq!(update_res["ok"], true);
    assert_eq!(
        update_res["data"]["gateway_url"],
        "https://openrouter.ai/api/v1"
    );
    assert_eq!(update_res["data"]["is_configured"], true);
    assert_eq!(events[0].topic, "ai_gateway_config");

    // Test gateway connection with mock active
    set_mock_gateway_response(|_| {
        Ok(GatewayResponse {
            id: "mock-res-test".to_string(),
            choices: vec![GatewayChoice {
                index: 0,
                message: GatewayChoiceMessage {
                    role: "assistant".to_string(),
                    content: Some("pong".to_string()),
                    tool_calls: None,
                },
                finish_reason: Some("stop".to_string()),
            }],
            usage: GatewayUsage {
                prompt_tokens: 5,
                completion_tokens: 2,
                total_tokens: 7,
                credits_deducted: Some(1),
                balance_remaining: Some(99),
            },
        })
    });

    let (test_res, _) = dispatch_cmd(
        &mut conn,
        "ai.test_gateway_connection",
        json!({}),
        Some(&token),
        None,
    );
    assert_eq!(test_res["ok"], true);
    assert_eq!(test_res["data"]["ok"], true);
    assert_eq!(test_res["data"]["provider_status"], "mock_active");

    clear_mock_gateway();
}

#[test]
fn test_copilot_single_turn_and_deanonymization_hydration() {
    let (_dir, mut conn) = setup_test_db();
    let token = bootstrap_and_login(&mut conn);

    let (_student_id, student_code) = create_student_helper(&mut conn, &token, "Nafis Fuad");

    let pseudonym = format!("Student #{}", student_code);
    let expected_pseudonym = pseudonym.clone();
    let step = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let step_clone = step.clone();

    set_mock_gateway_response(move |_req| {
        let current_step = step_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        if current_step == 0 {
            // Step 1: Model requests student search tool
            Ok(GatewayResponse {
                id: "chatcmpl-mock-tool".to_string(),
                choices: vec![GatewayChoice {
                    index: 0,
                    message: GatewayChoiceMessage {
                        role: "assistant".to_string(),
                        content: None,
                        tool_calls: Some(vec![GatewayToolCall {
                            id: "call_search_1".to_string(),
                            r#type: "function".to_string(),
                            function: GatewayFunctionCall {
                                name: "search_students".to_string(),
                                arguments: json!({ "query": "Nafis", "limit": 5 }).to_string(),
                            },
                        }]),
                    },
                    finish_reason: Some("tool_calls".to_string()),
                }],
                usage: GatewayUsage {
                    prompt_tokens: 20,
                    completion_tokens: 15,
                    total_tokens: 35,
                    credits_deducted: Some(1),
                    balance_remaining: Some(99),
                },
            })
        } else {
            // Step 2: Model uses pseudonym from redacted tool output and replies
            Ok(GatewayResponse {
                id: "chatcmpl-mock-1".to_string(),
                choices: vec![GatewayChoice {
                    index: 0,
                    message: GatewayChoiceMessage {
                        role: "assistant".to_string(),
                        content: Some(format!(
                            "Analysis complete: {} is enrolled in Batch A and has perfect attendance.",
                            expected_pseudonym
                        )),
                        tool_calls: None,
                    },
                    finish_reason: Some("stop".to_string()),
                }],
                usage: GatewayUsage {
                    prompt_tokens: 40,
                    completion_tokens: 20,
                    total_tokens: 60,
                    credits_deducted: Some(1),
                    balance_remaining: Some(98),
                },
            })
        }
    });

    // Ask copilot
    let (res, events) = dispatch_cmd(
        &mut conn,
        "ai.ask_copilot",
        json!({
            "prompt": "How is student Nafis Fuad performing?"
        }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true, "ask_copilot failed: {:?}", res);
    let conv_id = res["data"]["conversation_id"].as_str().unwrap().to_string();
    let content = res["data"]["content"].as_str().unwrap();

    // Verify in-core de-anonymization re-hydration: "Student #S-XXXX" replaced with "Nafis Fuad"
    assert!(
        content.contains("Nafis Fuad"),
        "Expected response to be re-hydrated with real name 'Nafis Fuad', got: {}",
        content
    );
    assert!(res["data"]["protection_metadata"]["shield_active"]
        .as_bool()
        .unwrap());
    assert_eq!(events[0].topic, "ai_copilot");

    // Get conversation
    let (conv_res, _) = dispatch_cmd(
        &mut conn,
        "ai.get_conversation",
        json!({ "conversation_id": conv_id }),
        Some(&token),
        None,
    );
    assert_eq!(conv_res["ok"], true);
    assert_eq!(conv_res["data"]["id"], conv_id);
    let messages = conv_res["data"]["messages"].as_array().unwrap();
    assert_eq!(messages.len(), 2); // user + assistant

    // Verify audit log
    let (audit_res, _) = dispatch_cmd(
        &mut conn,
        "ai.get_audit_log",
        json!({ "limit": 10 }),
        Some(&token),
        None,
    );
    assert_eq!(audit_res["ok"], true);
    let logs = audit_res["data"].as_array().unwrap();
    assert!(!logs.is_empty());
    assert_eq!(logs[0]["action_type"], "prompt");

    // Clear conversation
    let (clear_res, clear_events) = dispatch_cmd(
        &mut conn,
        "ai.clear_conversation",
        json!({ "conversation_id": conv_id }),
        Some(&token),
        None,
    );
    assert_eq!(clear_res["ok"], true);
    assert_eq!(clear_events[0].topic, "ai_copilot");

    clear_mock_gateway();
}

#[test]
fn test_copilot_multi_hop_tool_execution() {
    let (_dir, mut conn) = setup_test_db();
    let token = bootstrap_and_login(&mut conn);

    create_student_helper(&mut conn, &token, "Tanvir Rahman");

    // Track round: round 1 returns a tool call to query_students_with_dues; round 2 returns final answer
    let call_count = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let count_clone = call_count.clone();

    set_mock_gateway_response(move |_req| {
        let count = count_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        if count == 0 {
            // Round 1: Model requests tool execution
            Ok(GatewayResponse {
                id: "chatcmpl-hop-1".to_string(),
                choices: vec![GatewayChoice {
                    index: 0,
                    message: GatewayChoiceMessage {
                        role: "assistant".to_string(),
                        content: None,
                        tool_calls: Some(vec![GatewayToolCall {
                            id: "call_tool_dues_1".to_string(),
                            r#type: "function".to_string(),
                            function: GatewayFunctionCall {
                                name: "query_students_with_dues".to_string(),
                                arguments: json!({ "min_due_taka": 0, "limit": 10 }).to_string(),
                            },
                        }]),
                    },
                    finish_reason: Some("tool_calls".to_string()),
                }],
                usage: GatewayUsage {
                    prompt_tokens: 30,
                    completion_tokens: 20,
                    total_tokens: 50,
                    credits_deducted: Some(1),
                    balance_remaining: Some(99),
                },
            })
        } else {
            // Round 2: Model consumes tool result and answers
            Ok(GatewayResponse {
                id: "chatcmpl-hop-2".to_string(),
                choices: vec![GatewayChoice {
                    index: 0,
                    message: GatewayChoiceMessage {
                        role: "assistant".to_string(),
                        content: Some("Found 0 students with outstanding dues.".to_string()),
                        tool_calls: None,
                    },
                    finish_reason: Some("stop".to_string()),
                }],
                usage: GatewayUsage {
                    prompt_tokens: 50,
                    completion_tokens: 15,
                    total_tokens: 65,
                    credits_deducted: Some(1),
                    balance_remaining: Some(98),
                },
            })
        }
    });

    let (res, _) = dispatch_cmd(
        &mut conn,
        "ai.ask_copilot",
        json!({
            "prompt": "Show me students with pending fees."
        }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true, "copilot multi-hop failed: {:?}", res);
    assert_eq!(call_count.load(std::sync::atomic::Ordering::SeqCst), 2);
    let tools_used = res["data"]["tools_used"].as_array().unwrap();
    assert_eq!(tools_used.len(), 1);
    assert_eq!(tools_used[0], "query_students_with_dues");

    clear_mock_gateway();
}

#[test]
fn test_copilot_exhausted_credits_blocks_invocation() {
    let (_dir, mut conn) = setup_test_db();
    let token = bootstrap_and_login(&mut conn);

    // Set wallet balance to 0 directly
    conn.execute(
        "UPDATE ai_credit_wallets SET balance_credits = 0 WHERE id = 'default'",
        [],
    )
    .unwrap();

    let (res, _) = dispatch_cmd(
        &mut conn,
        "ai.ask_copilot",
        json!({
            "prompt": "Hello Coachify Copilot!"
        }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], false);
    assert_eq!(res["error"]["code"], "credits_exhausted");
}

#[test]
fn test_controlled_read_only_tools_execution_and_clamping() {
    use coachify_core::ai::tools::execute_tool;

    let (_dir, mut conn) = setup_test_db();
    let token = bootstrap_and_login(&mut conn);

    for i in 1..=5 {
        create_student_helper(&mut conn, &token, &format!("Batch Student {}", i));
    }

    // 1. query_students_with_dues - requests 100 limit, should execute without crashing
    let dues_res = execute_tool(
        &conn,
        "query_students_with_dues",
        r#"{"limit": 100, "min_due_taka": 0}"#,
        true,
    );
    assert!(
        dues_res.is_ok(),
        "query_students_with_dues failed: {:?}",
        dues_res
    );
    let dues_val = dues_res.unwrap();
    assert!(dues_val.is_array());
    assert!(dues_val.as_array().unwrap().len() <= 50);

    // 1b. query_students_with_dues - financial permission false -> error
    let unauth_res = execute_tool(&conn, "query_students_with_dues", r#"{"limit": 10}"#, false);
    assert!(unauth_res.is_err());

    // 2. Create program and batch for get_batch_summary
    let (prog_res, _) = dispatch_cmd(
        &mut conn,
        "programs.create",
        json!({
            "program": {
                "name": "Class 10 Science",
                "code": "C10-SCI",
                "description": "Science"
            }
        }),
        Some(&token),
        None,
    );
    let program_id = prog_res["data"]["id"].as_str().unwrap();

    let (batch_res, _) = dispatch_cmd(
        &mut conn,
        "batches.create",
        json!({
            "batch": {
                "program_id": program_id,
                "name": "Morning Physics",
                "code": "MOR-PHY",
                "capacity": 40
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(batch_res["ok"], true);

    let batch_summary = execute_tool(
        &conn,
        "get_batch_summary",
        r#"{"batch_name_or_id": "MOR-PHY"}"#,
        false,
    );
    assert!(
        batch_summary.is_ok(),
        "get_batch_summary failed: {:?}",
        batch_summary
    );
    assert_eq!(batch_summary.unwrap()["batch_code"], "MOR-PHY");

    // 3. get_attendance_stats
    let att_res = execute_tool(&conn, "get_attendance_stats", r#"{"days": 7}"#, false);
    assert!(att_res.is_ok());

    // 4. get_today_operational_summary
    let op_res = execute_tool(&conn, "get_today_operational_summary", r#"{}"#, false);
    assert!(op_res.is_ok());

    // 5. search_students
    let search_res = execute_tool(
        &conn,
        "search_students",
        r#"{"query": "Student", "limit": 100}"#,
        false,
    );
    assert!(search_res.is_ok());
    let search_arr = search_res.unwrap();
    assert!(search_arr.as_array().unwrap().len() <= 50);

    // 6. get_exam_results_summary - non-existent exam returns ToolExecutionFailed error
    let exam_res = execute_tool(
        &conn,
        "get_exam_results_summary",
        r#"{"exam_id_or_title": "NonExistentExam"}"#,
        false,
    );
    assert!(exam_res.is_err());

    // 7. Unknown tool call
    let unknown_res = execute_tool(&conn, "malicious_drop_tables", r#"{}"#, true);
    assert!(unknown_res.is_err());
}

#[test]
fn test_phone_number_redaction_and_rehydration() {
    use coachify_core::ai::privacy::{PrivacyRedactionEngine, PrivacySettings};

    let mut engine = PrivacyRedactionEngine::new(PrivacySettings::default());

    let raw_prompt = "Please send updates to 01712345678 and +8801812345678 immediately.";
    let scrubbed = engine.scrub_prompt(raw_prompt);

    assert!(
        !scrubbed.contains("01712345678"),
        "Raw phone leaked in scrubbed prompt: {}",
        scrubbed
    );
    assert!(
        !scrubbed.contains("+8801812345678"),
        "Raw phone leaked in scrubbed prompt: {}",
        scrubbed
    );
    assert!(scrubbed.contains("[PHONE-1]"));
    assert!(scrubbed.contains("[PHONE-2]"));

    // Echoing back the scrubbed token re-hydrates properly
    let model_echo = "Message scheduled for [PHONE-1] and [PHONE-2].";
    let (rehydrated, count) = engine.rehydrate_content(model_echo);
    assert_eq!(count, 2);
    assert!(rehydrated.contains("01712345678"));
    assert!(rehydrated.contains("+8801812345678"));
}

#[test]
fn test_prompt_injection_safety_and_sql_immunity() {
    let (_dir, mut conn) = setup_test_db();
    let token = bootstrap_and_login(&mut conn);

    set_mock_gateway_response(|_req| {
        Ok(GatewayResponse {
            id: "chatcmpl-safe-1".to_string(),
            choices: vec![GatewayChoice {
                index: 0,
                message: GatewayChoiceMessage {
                    role: "assistant".to_string(),
                    content: Some("I am unable to perform raw SQL executions.".to_string()),
                    tool_calls: None,
                },
                finish_reason: Some("stop".to_string()),
            }],
            usage: GatewayUsage {
                prompt_tokens: 25,
                completion_tokens: 10,
                total_tokens: 35,
                credits_deducted: Some(1),
                balance_remaining: Some(99),
            },
        })
    });

    let injection_payload = "'; DROP TABLE students; DROP TABLE ai_credit_wallets; --";
    let (res, _) = dispatch_cmd(
        &mut conn,
        "ai.ask_copilot",
        json!({
            "prompt": injection_payload
        }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true);

    // Verify tables still exist and are completely intact
    let student_table_exists: bool = conn
        .query_row(
            "SELECT 1 FROM sqlite_master WHERE type='table' AND name='students'",
            [],
            |_| Ok(true),
        )
        .unwrap_or(false);
    assert!(
        student_table_exists,
        "students table was harmed by prompt injection!"
    );

    let wallet_exists: bool = conn
        .query_row(
            "SELECT 1 FROM sqlite_master WHERE type='table' AND name='ai_credit_wallets'",
            [],
            |_| Ok(true),
        )
        .unwrap_or(false);
    assert!(
        wallet_exists,
        "wallet table was harmed by prompt injection!"
    );

    clear_mock_gateway();
}
