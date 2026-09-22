use coachify_core::db;
use coachify_core::ledger::{
    self, CreateAdjustmentInput, CreateManualChargeInput, CreatePaymentInput, CreateRefundInput,
};
use rusqlite::Connection;
use tempfile::tempdir;

fn setup_proptest_db() -> (tempfile::TempDir, Connection) {
    let dir = tempdir().expect("create temp dir");
    let db_path = dir.path().join("proptest_ledger.db");
    let conn = db::open_connection_at(&db_path).expect("open db");
    db::apply_all_migrations(&conn).expect("apply migrations");

    // Performance optimizations for heavy 100,000-op stress testing
    conn.execute_batch(
        "
        PRAGMA synchronous = OFF;
        PRAGMA journal_mode = MEMORY;
        PRAGMA cache_size = -64000;
    ",
    )
    .expect("set pragma");

    (dir, conn)
}

fn seed_test_entities(conn: &Connection, student_count: usize) -> (Vec<String>, Vec<String>) {
    // 1. Fee Heads
    let kinds = ["tuition", "admission", "exam", "material", "fine", "other"];
    let mut fee_head_ids = Vec::new();
    for (i, kind) in kinds.iter().enumerate() {
        let id = format!("fh_{}", i);
        let name = format!("Fee Head {}", i);
        conn.execute(
            "INSERT INTO fee_heads (id, name, kind, created_at, updated_at) VALUES (?1, ?2, ?3, '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')",
            rusqlite::params![id, name, kind],
        ).unwrap();
        fee_head_ids.push(id);
    }

    // 2. Students
    let mut student_ids = Vec::new();
    for i in 0..student_count {
        let id = format!("std_{:04}", i);
        let name = format!("Student {:04}", i);
        let phone = format!("01719{:06}", i);
        conn.execute(
            "INSERT INTO students (id, student_code, name, gender, dob, admission_date, phone_norm, present_address, status, created_at, updated_at)
             VALUES (?1, ?1, ?2, 'male', '2008-01-01', '2026-01-01', ?3, 'Dhaka, Bangladesh', 'active', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')",
            rusqlite::params![id, name, phone],
        ).unwrap();
        // Initialize balance
        conn.execute(
            "INSERT INTO student_balances (student_id, total_charged_poisha, total_paid_poisha, total_adjusted_poisha, total_refunded_poisha, balance_poisha, updated_at)
             VALUES (?1, 0, 0, 0, 0, 0, '2026-01-01T00:00:00Z')",
            rusqlite::params![id],
        ).unwrap();
        student_ids.push(id);
    }

    (student_ids, fee_head_ids)
}

// Deterministic pseudo-random number generator (Xorshift64) for repeatability without external overhead
struct FastRng {
    state: u64,
}

impl FastRng {
    fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 { 0xdeadbeefcafebabe } else { seed },
        }
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }

    fn gen_range(&mut self, low: u64, high: u64) -> u64 {
        if low >= high {
            return low;
        }
        low + (self.next_u64() % (high - low))
    }
}

#[test]
fn test_ledger_100k_operations_property_and_invariants() {
    let (_dir, conn) = setup_proptest_db();
    let num_students = 50;
    let (student_ids, fee_head_ids) = seed_test_entities(&conn, num_students);

    let mut rng = FastRng::new(42);
    let total_operations = 100_000;
    let batch_size = 2_000;
    let mut idem_counter: u64 = 0;

    println!(
        "Starting 100,000-operation ledger invariant state machine test (batched transactions)..."
    );

    let start_time = std::time::Instant::now();

    for batch_start in (0..total_operations).step_by(batch_size) {
        conn.execute_batch("BEGIN;").expect("begin batch");

        for _op_offset in 0..batch_size {
            let op_type = rng.gen_range(0, 100);

            match op_type {
                // 0..30: Create Manual Charge (30%)
                0..=29 => {
                    let s_idx = rng.gen_range(0, num_students as u64) as usize;
                    let fh_idx = rng.gen_range(0, fee_head_ids.len() as u64) as usize;
                    let amount = rng.gen_range(100, 50000) as i64; // 1 BDT to 500 BDT

                    let res = ledger::create_manual_charge(
                        &conn,
                        CreateManualChargeInput {
                            student_id: student_ids[s_idx].clone(),
                            fee_head_id: fee_head_ids[fh_idx].clone(),
                            amount_poisha: amount,
                            due_date: "2026-09-15".to_string(),
                            period: Some("2026-09".to_string()),
                        },
                    );
                    assert!(res.is_ok(), "charge creation must succeed: {:?}", res);
                }

                // 30..65: Create Payment (35%)
                30..=64 => {
                    let s_idx = rng.gen_range(0, num_students as u64) as usize;
                    let amount = rng.gen_range(100, 60000) as i64;
                    idem_counter += 1;

                    let res = ledger::create_payment(
                        &conn,
                        CreatePaymentInput {
                            student_id: student_ids[s_idx].clone(),
                            amount_poisha: amount,
                            method: "cash".to_string(),
                            reference: None,
                            business_date: None,
                            idempotency_key: format!("proptest-pay-{}", idem_counter),
                            target_charge_ids: None,
                        },
                        None,
                        None,
                    );
                    assert!(res.is_ok(), "payment creation must succeed: {:?}", res);
                }

                // 65..75: Create Waiver (10%)
                65..=74 => {
                    let s_idx = rng.gen_range(0, num_students as u64) as usize;
                    let student_id = &student_ids[s_idx];

                    // Find an open charge for this student
                    let charge_opt: Option<(String, i64)> = conn.query_row(
                        "SELECT c.id,
                                c.net_amount_poisha
                                + COALESCE((SELECT SUM(adj.amount_poisha) FROM adjustments adj WHERE adj.charge_id = c.id AND adj.type = 'fine'), 0)
                                - COALESCE((SELECT SUM(adj.amount_poisha) FROM adjustments adj WHERE adj.charge_id = c.id AND adj.type = 'waiver'), 0)
                                - COALESCE((
                                    SELECT SUM(pa.amount_poisha)
                                    FROM payment_allocations pa
                                    JOIN payments p ON pa.payment_id = p.id
                                    WHERE pa.charge_id = c.id AND p.status = 'posted'
                                ), 0) as remaining_due
                         FROM charges c
                         WHERE c.student_id = ?1 AND c.status = 'open'
                         LIMIT 1",
                        rusqlite::params![student_id],
                        |r| Ok((r.get(0)?, r.get(1)?)),
                    ).ok();

                    if let Some((cid, remaining_due)) = charge_opt {
                        if remaining_due > 0 {
                            let amount = rng.gen_range(1, (remaining_due as u64).max(2)) as i64;
                            let _ = ledger::create_adjustment(
                                &conn,
                                CreateAdjustmentInput {
                                    student_id: student_id.clone(),
                                    charge_id: Some(cid),
                                    r#type: "waiver".to_string(),
                                    amount_poisha: amount,
                                    reason: "proptest waiver".to_string(),
                                    business_date: None,
                                },
                                None,
                            );
                        }
                    }
                }

                // 75..82: Create Fine (7%)
                75..=81 => {
                    let s_idx = rng.gen_range(0, num_students as u64) as usize;
                    let amount = rng.gen_range(50, 5000) as i64;

                    let res = ledger::create_adjustment(
                        &conn,
                        CreateAdjustmentInput {
                            student_id: student_ids[s_idx].clone(),
                            charge_id: None,
                            r#type: "fine".to_string(),
                            amount_poisha: amount,
                            reason: "proptest late fine".to_string(),
                            business_date: None,
                        },
                        None,
                    );
                    assert!(res.is_ok(), "fine creation must succeed: {:?}", res);
                }

                // 82..88: Create Refund (6%)
                82..=87 => {
                    let s_idx = rng.gen_range(0, num_students as u64) as usize;
                    let student_id = &student_ids[s_idx];

                    // Find a payment with unallocated funds
                    let pay_opt: Option<(String, i64)> = conn.query_row(
                        "SELECT p.id,
                                p.amount_poisha
                                - COALESCE((SELECT SUM(pa.amount_poisha) FROM payment_allocations pa WHERE pa.payment_id = p.id), 0)
                                - COALESCE((SELECT SUM(r.amount_poisha) FROM refunds r WHERE r.payment_id = p.id), 0) as unallocated
                         FROM payments p
                         WHERE p.student_id = ?1 AND p.status = 'posted'
                         HAVING unallocated > 0
                         LIMIT 1",
                        rusqlite::params![student_id],
                        |r| Ok((r.get(0)?, r.get(1)?)),
                    ).ok();

                    if let Some((pid, unallocated)) = pay_opt {
                        if unallocated > 0 {
                            let amount = rng.gen_range(1, (unallocated as u64).max(2)) as i64;
                            let _ = ledger::create_refund(
                                &conn,
                                CreateRefundInput {
                                    payment_id: pid,
                                    amount_poisha: amount,
                                    method: "cash".to_string(),
                                    reason: "proptest refund".to_string(),
                                    business_date: None,
                                },
                                None,
                            );
                        }
                    }
                }

                // 88..94: Void Payment (6%)
                88..=93 => {
                    let s_idx = rng.gen_range(0, num_students as u64) as usize;
                    let student_id = &student_ids[s_idx];

                    let pay_id_opt: Option<String> = conn.query_row(
                        "SELECT id FROM payments WHERE student_id = ?1 AND status = 'posted' LIMIT 1",
                        rusqlite::params![student_id],
                        |r| r.get(0),
                    ).ok();

                    if let Some(pid) = pay_id_opt {
                        let _ = ledger::void_payment(&conn, &pid, "proptest void payment", None);
                    }
                }

                // 94..100: Void Charge (6%)
                _ => {
                    let s_idx = rng.gen_range(0, num_students as u64) as usize;
                    let student_id = &student_ids[s_idx];

                    // Find a charge that has NO active allocations
                    let charge_id_opt: Option<String> = conn
                        .query_row(
                            "SELECT c.id FROM charges c
                         WHERE c.student_id = ?1 AND c.status = 'open'
                           AND NOT EXISTS (
                               SELECT 1 FROM payment_allocations pa
                               JOIN payments p ON pa.payment_id = p.id
                               WHERE pa.charge_id = c.id AND p.status = 'posted'
                           )
                         LIMIT 1",
                            rusqlite::params![student_id],
                            |r| r.get(0),
                        )
                        .ok();

                    if let Some(cid) = charge_id_opt {
                        let _ = ledger::void_charge(&conn, &cid, "proptest void charge");
                    }
                }
            }
        }

        conn.execute_batch("COMMIT;").expect("commit batch");

        // Periodic invariant assertion every 20,000 operations
        let current_ops = batch_start + batch_size;
        if current_ops % 20_000 == 0 {
            ledger::verify_all_ledger_invariants(&conn)
                .unwrap_or_else(|e| panic!("Invariant violation at op {}: {:?}", current_ops, e));

            // Verify zero balance drift between synchronous cache and live formula
            for sid in &student_ids {
                let cached_balance: i64 = conn
                    .query_row(
                        "SELECT balance_poisha FROM student_balances WHERE student_id = ?1",
                        rusqlite::params![sid],
                        |r| r.get(0),
                    )
                    .unwrap();

                let live_balance: i64 = conn.query_row(
                    "SELECT
                        COALESCE((SELECT SUM(net_amount_poisha) FROM charges WHERE student_id = ?1 AND status != 'voided'), 0)
                        + COALESCE((SELECT SUM(amount_poisha) FROM adjustments WHERE student_id = ?1 AND type = 'fine'), 0)
                        - COALESCE((SELECT SUM(amount_poisha) FROM adjustments WHERE student_id = ?1 AND type = 'waiver'), 0)
                        - COALESCE((SELECT SUM(amount_poisha) FROM payments WHERE student_id = ?1 AND status = 'posted'), 0)
                        + COALESCE((
                            SELECT SUM(r.amount_poisha)
                            FROM refunds r
                            JOIN payments p ON r.payment_id = p.id
                            WHERE p.student_id = ?1 AND p.status = 'posted'
                        ), 0)",
                    rusqlite::params![sid],
                    |r| r.get(0),
                ).unwrap();

                assert_eq!(
                    cached_balance, live_balance,
                    "Zero balance drift violation for student {} at op {}: cached={}, live={}",
                    sid, current_ops, cached_balance, live_balance
                );
            }

            println!(
                "Completed {}/{} operations (elapsed: {:.2?})",
                current_ops,
                total_operations,
                start_time.elapsed()
            );
        }
    }

    // Final full verification
    println!("Running final complete ledger invariant check...");
    ledger::verify_all_ledger_invariants(&conn)
        .expect("All 9 ledger invariants must hold after 100,000 ops");

    let elapsed = start_time.elapsed();
    println!("100,000-operation proptest completed in {:.2?} with 0 invariant violations and 0 balance drift!", elapsed);
}
