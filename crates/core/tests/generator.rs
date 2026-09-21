use coachify_core::db;
use coachify_core::dispatcher;
use coachify_core::generator::{DatasetGenerator, DemoProfile, SYSTEM_CLI_USER_ID};
use rusqlite::Connection;
use serde_json::json;
use std::fs;
use std::time::Instant;

#[test]
fn test_demo_generation_performance_and_counts() {
    let mut gen = DatasetGenerator::new(
        DemoProfile::Demo,
        Some(42),
        Some("test_demo_counts.db".to_string()),
    )
    .expect("create generator");

    let start = Instant::now();
    let summary = gen.generate().expect("generate demo dataset");
    let elapsed = start.elapsed().as_millis();

    assert_eq!(summary.profile, "demo_500");
    assert_eq!(summary.seed, 42);
    assert_eq!(summary.student_count, 500);
    // Sibling grouping: 500 students should result in fewer guardians (~20% sibling share)
    assert!(
        summary.guardian_count >= 350 && summary.guardian_count <= 450,
        "Guardians count {} should reflect ~20% sibling grouping",
        summary.guardian_count
    );
    assert!(summary.batch_count > 0);
    assert!(summary.charge_count > 0);
    assert!(summary.payment_count > 0);
    assert!(
        elapsed < 5000,
        "Generation took {}ms, expected < 5000ms (AC-1)",
        elapsed
    );

    // Clean up
    let path = std::path::PathBuf::from(&summary.db_path);
    if path.exists() {
        let _ = fs::remove_file(&path);
    }
}

#[test]
fn test_deterministic_repeatability_ac8() {
    let mut gen1 = DatasetGenerator::new(
        DemoProfile::Demo,
        Some(12345),
        Some("test_repeat_1.db".to_string()),
    )
    .expect("create gen1");
    let sum1 = gen1.generate().expect("generate 1");

    let mut gen2 = DatasetGenerator::new(
        DemoProfile::Demo,
        Some(12345),
        Some("test_repeat_2.db".to_string()),
    )
    .expect("create gen2");
    let sum2 = gen2.generate().expect("generate 2");

    assert_eq!(sum1.student_count, sum2.student_count);
    assert_eq!(sum1.guardian_count, sum2.guardian_count);
    assert_eq!(sum1.batch_count, sum2.batch_count);
    assert_eq!(sum1.charge_count, sum2.charge_count);
    assert_eq!(sum1.payment_count, sum2.payment_count);

    let conn1 = Connection::open(&sum1.db_path).unwrap();
    let conn2 = Connection::open(&sum2.db_path).unwrap();

    let student1: (String, String) = conn1
        .query_row(
            "SELECT student_code, name FROM students WHERE id = 'std-000001'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();

    let student2: (String, String) = conn2
        .query_row(
            "SELECT student_code, name FROM students WHERE id = 'std-000001'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();

    assert_eq!(
        student1, student2,
        "Deterministic seed must generate identical student records"
    );

    drop(conn1);
    drop(conn2);
    let _ = fs::remove_file(&sum1.db_path);
    let _ = fs::remove_file(&sum2.db_path);
}

#[test]
fn test_demographics_and_bengali_script_ac2() {
    let mut gen = DatasetGenerator::new(
        DemoProfile::Demo,
        Some(999),
        Some("test_demographics.db".to_string()),
    )
    .expect("create generator");
    let summary = gen.generate().expect("generate demo dataset");

    let conn = Connection::open(&summary.db_path).unwrap();

    // Check Bengali script name presence
    let bengali_names_count: i64 = conn
        .query_row(
            "SELECT count(*) FROM students WHERE name_bn IS NOT NULL AND length(name_bn) > 0",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(
        bengali_names_count, 500,
        "All demo students must have a Bengali script name"
    );

    // Check phone numbers format
    let valid_phones_count: i64 = conn
        .query_row(
            "SELECT count(*) FROM students WHERE phone_norm LIKE '8801%' OR phone_norm LIKE '+8801%'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(
        valid_phones_count, 500,
        "All demo students must have a valid BD phone number"
    );

    // Check sibling guardian relationships
    let shared_guardians: i64 = conn
        .query_row(
            "SELECT count(*) FROM (
                SELECT guardian_id FROM student_guardians GROUP BY guardian_id HAVING count(student_id) > 1
             )",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert!(
        shared_guardians > 0,
        "Expected multiple shared guardians for siblings (~20%)"
    );

    drop(conn);
    let _ = fs::remove_file(&summary.db_path);
}

#[test]
fn test_academic_batches_and_roster_bounds_ac3() {
    let mut gen = DatasetGenerator::new(
        DemoProfile::Demo,
        Some(777),
        Some("test_batches.db".to_string()),
    )
    .expect("create generator");
    let summary = gen.generate().expect("generate demo dataset");

    let conn = Connection::open(&summary.db_path).unwrap();

    let max_enrollment: i64 = conn
        .query_row(
            "SELECT coalesce(max(cnt), 0) FROM (
                SELECT count(*) as cnt FROM enrollments GROUP BY batch_id
             )",
            [],
            |r| r.get(0),
        )
        .unwrap();

    assert!(
        max_enrollment <= 40,
        "Batch capacity was exceeded: max is {}, expected <= 40",
        max_enrollment
    );

    drop(conn);
    let _ = fs::remove_file(&summary.db_path);
}

#[test]
fn test_temporal_financial_clustering_ac4() {
    let mut gen = DatasetGenerator::new(
        DemoProfile::Demo,
        Some(555),
        Some("test_financial.db".to_string()),
    )
    .expect("create generator");
    let summary = gen.generate().expect("generate demo dataset");

    let conn = Connection::open(&summary.db_path).unwrap();

    // Check payment dates clustering between 5th and 10th
    let cluster_count: i64 = conn
        .query_row(
            "SELECT count(*) FROM payments WHERE CAST(substr(business_date, 9, 2) AS INTEGER) BETWEEN 5 AND 10",
            [],
            |r| r.get(0),
        )
        .unwrap();

    let total_payments: i64 = conn
        .query_row("SELECT count(*) FROM payments", [], |r| r.get(0))
        .unwrap();

    let cluster_ratio = (cluster_count as f64) / (total_payments as f64);
    assert!(
        cluster_ratio >= 0.65,
        "Expected >= 65% of payments clustered between 5th and 10th, got {:.2}%",
        cluster_ratio * 100.0
    );

    // Verify student balances cache table
    let balance_rows: i64 = conn
        .query_row("SELECT count(*) FROM student_balances", [], |r| r.get(0))
        .unwrap();
    assert_eq!(
        balance_rows, 500,
        "Student balances cache must be populated for all students"
    );

    drop(conn);
    let _ = fs::remove_file(&summary.db_path);
}

#[test]
fn test_fts5_trigram_search_performance_ac5() {
    let mut gen =
        DatasetGenerator::new(DemoProfile::Demo, Some(42), Some("test_fts.db".to_string()))
            .expect("create generator");
    let summary = gen.generate().expect("generate demo dataset");

    let conn = Connection::open(&summary.db_path).unwrap();

    let start = Instant::now();
    let hits: i64 = conn
        .query_row(
            "SELECT count(*) FROM student_search_index WHERE student_search_index MATCH 'রহিম*'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    let query_ms = start.elapsed().as_millis();

    assert!(hits >= 0);
    assert!(
        query_ms < 50,
        "FTS5 query took {}ms, expected < 50ms (AC-5)",
        query_ms
    );

    drop(conn);
    let _ = fs::remove_file(&summary.db_path);
}

#[test]
fn test_integrity_and_wal_mode_ac7() {
    let mut gen = DatasetGenerator::new(
        DemoProfile::Demo,
        Some(42),
        Some("test_integrity.db".to_string()),
    )
    .expect("create generator");
    let summary = gen.generate().expect("generate demo dataset");

    let conn = Connection::open(&summary.db_path).unwrap();

    let integrity: String = conn
        .query_row("PRAGMA integrity_check;", [], |r| r.get(0))
        .unwrap();
    assert_eq!(integrity, "ok", "Integrity check must pass");

    let journal_mode: String = conn
        .query_row("PRAGMA journal_mode;", [], |r| r.get(0))
        .unwrap();
    assert_eq!(
        journal_mode.to_lowercase(),
        "wal",
        "Must be WAL journal mode"
    );

    drop(conn);
    let _ = fs::remove_file(&summary.db_path);
}

#[test]
fn test_metadata_table_ac10() {
    let mut gen = DatasetGenerator::new(
        DemoProfile::Demo,
        Some(42),
        Some("test_metadata.db".to_string()),
    )
    .expect("create generator");
    let summary = gen.generate().expect("generate demo dataset");

    let conn = Connection::open(&summary.db_path).unwrap();

    let (profile, seed, count): (String, i64, i64) = conn
        .query_row(
            "SELECT profile, seed, student_count FROM demo_metadata WHERE id = 1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .unwrap();

    assert_eq!(profile, "demo_500");
    assert_eq!(seed, 42);
    assert_eq!(count, 500);

    drop(conn);
    let _ = fs::remove_file(&summary.db_path);
}

#[test]
fn test_path_sandboxing_and_production_protection_ac9() {
    // Path traversal attempt should fail
    let err = DatasetGenerator::new(DemoProfile::Demo, Some(42), Some("../evil.db".to_string()));
    assert!(err.is_err());

    // Specifying coachify.db (production) should fail with DbError::Config
    let prod_err =
        DatasetGenerator::new(DemoProfile::Demo, Some(42), Some("coachify.db".to_string()));
    assert!(prod_err.is_err());
    match prod_err {
        Err(db::DbError::Config(msg)) => {
            assert!(msg.contains("active production database"));
        }
        other => panic!("Expected DbError::Config, got: {:?}", other),
    }
}

#[test]
fn test_database_switching_ac11() {
    let mut gen = DatasetGenerator::new(
        DemoProfile::Demo,
        Some(42),
        Some("test_switch.db".to_string()),
    )
    .expect("create generator");
    let summary = gen.generate().expect("generate demo dataset");

    // Switching to the generated demo db
    let switched_path = db::switch_to_demo_database(std::path::Path::new(&summary.db_path))
        .expect("switch to demo db");
    let summary_buf = std::path::PathBuf::from(&summary.db_path);
    let expected_canon = summary_buf.canonicalize().unwrap().display().to_string();
    assert_eq!(switched_path, expected_canon);
    assert!(db::is_demo_database_active());
    assert_eq!(
        db::get_active_db_path().unwrap(),
        summary_buf.canonicalize().unwrap()
    );

    // Switching back to production
    let prod_path = db::switch_to_production_database().expect("switch to prod db");
    assert!(!db::is_demo_database_active());
    assert_eq!(
        db::get_active_db_path().unwrap(),
        std::path::PathBuf::from(&prod_path)
    );

    let _ = fs::remove_file(&summary.db_path);
}

#[test]
fn test_audit_trail_compliance_ac12() {
    let mut gen = DatasetGenerator::new(
        DemoProfile::Demo,
        Some(42),
        Some("test_audit.db".to_string()),
    )
    .expect("create generator");
    let summary = gen.generate().expect("generate demo dataset");

    let conn = Connection::open(&summary.db_path).unwrap();

    let (action, user_id): (String, Option<String>) = conn
        .query_row(
            "SELECT action, user_id FROM audit_log WHERE action = 'generator.generate_dataset'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();

    assert_eq!(action, "generator.generate_dataset");
    assert_eq!(user_id.as_deref(), Some(SYSTEM_CLI_USER_ID));

    // Verify hash chain
    let chain_res = coachify_core::audit::verify_audit_log_chain(&conn).expect("verify chain");
    assert!(chain_res.is_valid, "Audit chain in demo db must be valid");

    drop(conn);
    let _ = fs::remove_file(&summary.db_path);
}

#[test]
fn test_dispatcher_generator_commands() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let db_path = dir.path().join("test_dispatcher_gen.db");
    let mut conn = db::open_connection_at(&db_path).expect("open test db");
    db::apply_all_migrations(&conn).expect("apply migrations");

    // Test generator.get_status
    let (res, _) = dispatcher::dispatch(&mut conn, "generator.get_status", "{}", None, None);
    let val: serde_json::Value = serde_json::from_str(&res).unwrap();
    assert_eq!(val["ok"], true);
    assert_eq!(val["data"]["is_demo_active"], false);

    // Test generator.switch_database with invalid path
    let (res_err, _) = dispatcher::dispatch(
        &mut conn,
        "generator.switch_database",
        &json!({ "target_filename": "../traversal.db" }).to_string(),
        None,
        None,
    );
    let val_err: serde_json::Value = serde_json::from_str(&res_err).unwrap();
    assert_eq!(val_err["ok"], false);
    assert_eq!(val_err["error"]["code"], "validation_error");

    // Test generator.switch_to_production_database
    let (res_prod, _) = dispatcher::dispatch(
        &mut conn,
        "generator.switch_to_production_database",
        "{}",
        None,
        None,
    );
    let val_prod: serde_json::Value = serde_json::from_str(&res_prod).unwrap();
    assert_eq!(val_prod["ok"], true);
}
