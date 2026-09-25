use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrashIterationRecord {
    pub scenario: String,
    pub iteration: usize,
    pub checkpoint: String,
    pub killed: bool,
    pub verified: bool,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrashSuiteReport {
    pub timestamp: String,
    pub total_iterations: usize,
    pub passed_iterations: usize,
    pub failed_iterations: usize,
    pub total_duration_seconds: f64,
    pub scenarios: Vec<ScenarioSummary>,
    pub records: Vec<CrashIterationRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioSummary {
    pub name: String,
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
}

fn resolve_worker_binary() -> PathBuf {
    let mut exe = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("xtask.exe"));
    exe.pop();
    let bin_name = if cfg!(windows) {
        "crash_worker.exe"
    } else {
        "crash_worker"
    };
    let candidate = exe.join(bin_name);
    if candidate.exists() {
        return candidate;
    }

    let target_debug = PathBuf::from("target/debug").join(bin_name);
    if target_debug.exists() {
        return target_debug;
    }

    // Default to target/debug path
    target_debug
}

pub fn run_crash_suite(scenario_arg: Option<&str>, count_arg: Option<usize>) -> Result<(), String> {
    println!("Ensuring crash_worker binary is up to date...");
    let build_status = Command::new("cargo")
        .args(["build", "-p", "xtask", "--bin", "crash_worker"])
        .status()
        .map_err(|e| format!("Failed to build crash_worker: {}", e))?;
    if !build_status.success() {
        return Err("cargo build -p xtask --bin crash_worker failed".to_string());
    }
    let worker_bin = resolve_worker_binary();

    let run_all = scenario_arg.is_none() || scenario_arg == Some("all");
    let target_scenario = scenario_arg.unwrap_or("all");

    let mut records: Vec<CrashIterationRecord> = Vec::new();
    let overall_start = Instant::now();

    println!("\n=======================================================");
    println!("     Coachify Automated Crash & Failure Safety Suite    ");
    println!("=======================================================");
    println!("Target scenario: {}", target_scenario);
    println!("Worker binary:   {}", worker_bin.display());
    println!("OS:              {}", std::env::consts::OS);
    println!("=======================================================\n");

    // 1. Payments Crash Tests (AC-1: default 20)
    if run_all || target_scenario == "payments" {
        let count = count_arg.unwrap_or(20);
        println!(
            ">>> Running Scenario 1: Payments Power Loss ({} iterations)",
            count
        );
        run_scenario(
            "payments",
            "payment_before_commit",
            count,
            &worker_bin,
            &mut records,
        )?;
    }

    // 2. Backups Crash Tests (AC-2: default 10)
    if run_all || target_scenario == "backups" {
        let count = count_arg.unwrap_or(10);
        println!(
            "\n>>> Running Scenario 2: Backups Power Loss & Temp Sweep ({} iterations)",
            count
        );
        run_scenario(
            "backups",
            "backup_mid_stream",
            count,
            &worker_bin,
            &mut records,
        )?;
    }

    // 3. Migrations Crash Tests (AC-3: default 10)
    if run_all || target_scenario == "migrations" {
        let count = count_arg.unwrap_or(10);
        println!(
            "\n>>> Running Scenario 3: Migrations Power Loss & Sandbox Cleanup ({} iterations)",
            count
        );
        run_scenario(
            "migrations",
            "migration_mid_script",
            count,
            &worker_bin,
            &mut records,
        )?;
    }

    let overall_duration = overall_start.elapsed().as_secs_f64();
    let total_iterations = records.len();
    let passed_iterations = records.iter().filter(|r| r.verified).count();
    let failed_iterations = total_iterations - passed_iterations;

    // Collect summaries
    let mut scenario_summaries: Vec<ScenarioSummary> = Vec::new();
    for name in &["payments", "backups", "migrations"] {
        let matching: Vec<&CrashIterationRecord> =
            records.iter().filter(|r| r.scenario == *name).collect();
        if !matching.is_empty() {
            let total = matching.len();
            let passed = matching.iter().filter(|r| r.verified).count();
            scenario_summaries.push(ScenarioSummary {
                name: name.to_string(),
                total,
                passed,
                failed: total - passed,
            });
        }
    }

    let now_ts = coachify_core::db::now_utc_iso();
    let report = CrashSuiteReport {
        timestamp: now_ts.clone(),
        total_iterations,
        passed_iterations,
        failed_iterations,
        total_duration_seconds: overall_duration,
        scenarios: scenario_summaries,
        records,
    };

    // Save report to target/crash-reports/
    let report_dir = Path::new("target").join("crash-reports");
    let _ = fs::create_dir_all(&report_dir);
    let safe_ts = now_ts.replace([':', '.'], "-");
    let report_path = report_dir.join(format!("crash-summary-{}.json", safe_ts));
    let json_bytes = serde_json::to_string_pretty(&report).map_err(|e| e.to_string())?;
    fs::write(&report_path, json_bytes).map_err(|e| e.to_string())?;

    println!("\n=======================================================");
    println!("                   CRASH SUITE SUMMARY                 ");
    println!("=======================================================");
    for s in &report.scenarios {
        let rate = if s.total > 0 {
            (s.passed as f64 / s.total as f64) * 100.0
        } else {
            0.0
        };
        println!(
            "{:<12} | Passed: {:>2}/{:>2} ({:>5.1}%) | Failed: {:>2}",
            s.name, s.passed, s.total, rate, s.failed
        );
    }
    println!("-------------------------------------------------------");
    println!(
        "TOTAL:        Passed: {:>2}/{:>2} in {:.2}s",
        passed_iterations, total_iterations, overall_duration
    );
    println!("Report saved: {}", report_path.display());
    println!("=======================================================\n");

    if failed_iterations > 0 {
        return Err(format!(
            "Crash suite failed: {} / {} iterations failed",
            failed_iterations, total_iterations
        ));
    }

    Ok(())
}

fn run_scenario(
    scenario: &str,
    checkpoint: &str,
    count: usize,
    worker_bin: &Path,
    records: &mut Vec<CrashIterationRecord>,
) -> Result<(), String> {
    for iter in 0..count {
        let temp_dir = tempfile::tempdir().map_err(|e| e.to_string())?;
        let db_path = temp_dir.path().join("crash_target.db");
        let backup_dir = temp_dir.path().join("backups");
        let iter_start = Instant::now();

        // 1. Spawn worker child in "run" mode
        let mut child = Command::new(worker_bin)
            .arg("--scenario")
            .arg(scenario)
            .arg("--action")
            .arg("run")
            .arg("--db")
            .arg(&db_path)
            .arg("--backup-dir")
            .arg(&backup_dir)
            .arg("--iteration")
            .arg(iter.to_string())
            .env("COACHIFY_CRASH_CHECKPOINT", checkpoint)
            .stdout(Stdio::piped())
            .stdin(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|e| format!("Failed to spawn crash_worker: {}", e))?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| "Failed to capture child stdout".to_string())?;

        let mut reader = BufReader::new(stdout);
        let mut line = String::new();
        let mut checkpoint_reached = false;

        // Read line with a loop
        let wait_start = Instant::now();
        while wait_start.elapsed() < Duration::from_secs(30) {
            line.clear();
            if let Ok(n) = reader.read_line(&mut line) {
                if n == 0 {
                    break;
                }
                if line.trim() == "CHECKPOINT_REACHED" {
                    checkpoint_reached = true;
                    break;
                }
            }
        }

        if !checkpoint_reached {
            let _ = child.kill();
            let _ = child.wait();
            return Err(format!(
                "Scenario {} iteration {} failed to reach checkpoint '{}'",
                scenario, iter, checkpoint
            ));
        }

        // 2. Kill the process immediately at checkpoint (Windows TerminateProcess)
        child
            .kill()
            .map_err(|e| format!("Failed to kill child process: {}", e))?;
        let _ = child.wait();

        // 3. Spawn worker in "verify" mode
        let verify_status = Command::new(worker_bin)
            .arg("--scenario")
            .arg(scenario)
            .arg("--action")
            .arg("verify")
            .arg("--db")
            .arg(&db_path)
            .arg("--backup-dir")
            .arg(&backup_dir)
            .arg("--iteration")
            .arg(iter.to_string())
            .status()
            .map_err(|e| format!("Failed to spawn verify worker: {}", e))?;

        let verified = verify_status.success();
        let duration_ms = iter_start.elapsed().as_millis() as u64;

        records.push(CrashIterationRecord {
            scenario: scenario.to_string(),
            iteration: iter,
            checkpoint: checkpoint.to_string(),
            killed: true,
            verified,
            duration_ms,
        });

        if verified {
            print!(".");
        } else {
            print!("F");
        }
        use std::io::Write;
        let _ = std::io::stdout().flush();
    }
    println!(" OK");
    Ok(())
}
