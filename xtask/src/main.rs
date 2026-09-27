mod crash_suite;
mod export;

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn get_workspace_root() -> PathBuf {
    if Path::new("migrations/released.json").exists() {
        PathBuf::from(".")
    } else {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("workspace root")
            .to_path_buf()
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let command = args.get(1).map(|s| s.as_str()).unwrap_or("export-types");

    let workspace_root = get_workspace_root();

    match command {
        "export-types" => {
            let out_path = workspace_root.join("ui/src/gen/rpc.ts");
            println!("Exporting TypeScript types to {:?}...", out_path);
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent).expect("Failed to create parent directory for rpc.ts");
            }

            let content = export::generate_ts_contract();
            fs::write(&out_path, content).expect("Failed to write rpc.ts");
            println!("TypeScript types successfully exported to {:?}", out_path);
        }
        "check-types" => {
            let rpc_path = workspace_root.join("ui/src/gen/rpc.ts");
            println!(
                "Checking freshness of TypeScript contract at {:?}...",
                rpc_path
            );
            if !rpc_path.exists() {
                eprintln!(
                    "Error: Generated TypeScript contract file {:?} does not exist. Run 'cargo run -p xtask export-types' to generate it.",
                    rpc_path
                );
                std::process::exit(1);
            }

            let existing = fs::read_to_string(&rpc_path).expect("Failed to read rpc.ts");
            let expected = export::generate_ts_contract();

            // Normalize line endings to avoid CRLF vs LF differences
            let existing_norm = existing.replace("\r\n", "\n");
            let expected_norm = expected.replace("\r\n", "\n");

            if existing_norm != expected_norm {
                eprintln!(
                    "Error: TypeScript contract in {:?} is stale or modified.\nRun 'cargo run -p xtask export-types' to sync the contract.",
                    rpc_path
                );
                std::process::exit(1);
            }

            println!("TypeScript contract is fresh and in sync.");
        }
        "check-migrations" => {
            println!("Checking immutability of released migrations...");
            match coachify_core::migrations::verify_released_migrations(&workspace_root) {
                Ok(()) => {
                    println!("All released migrations are intact and unmodified.");
                }
                Err(err) => {
                    eprintln!("Error verifying released migrations: {}", err);
                    std::process::exit(1);
                }
            }

            println!("Verifying migration 0009 tracking schema...");
            match coachify_core::migrations::verify_migration_0009_schema(&workspace_root) {
                Ok(()) => {
                    println!("Migration 0009 tracking schema and triggers verified.");
                }
                Err(err) => {
                    eprintln!("Error verifying migration 0009: {}", err);
                    std::process::exit(1);
                }
            }
        }
        "generate-dataset" => {
            let mut profile: Option<coachify_core::generator::DemoProfile> = None;
            let mut seed: Option<u64> = None;
            let mut out_filename: Option<String> = None;

            let mut i = 2;
            while i < args.len() {
                match args[i].as_str() {
                    "--profile" => {
                        i += 1;
                        if i < args.len() {
                            match args[i].as_str() {
                                "demo" => {
                                    profile = Some(coachify_core::generator::DemoProfile::Demo)
                                }
                                "load" => {
                                    profile = Some(coachify_core::generator::DemoProfile::Load)
                                }
                                other => {
                                    eprintln!(
                                        "Error: Invalid profile '{}'. Must be 'demo' or 'load'.",
                                        other
                                    );
                                    std::process::exit(1);
                                }
                            }
                        } else {
                            eprintln!("Error: Missing value for --profile");
                            std::process::exit(1);
                        }
                    }
                    "--seed" => {
                        i += 1;
                        if i < args.len() {
                            match args[i].parse::<u64>() {
                                Ok(s) => seed = Some(s),
                                Err(_) => {
                                    eprintln!("Error: --seed must be a valid u64 integer");
                                    std::process::exit(1);
                                }
                            }
                        } else {
                            eprintln!("Error: Missing value for --seed");
                            std::process::exit(1);
                        }
                    }
                    "--out" => {
                        i += 1;
                        if i < args.len() {
                            out_filename = Some(args[i].clone());
                        } else {
                            eprintln!("Error: Missing value for --out");
                            std::process::exit(1);
                        }
                    }
                    "--" => {
                        // Cargo separator, skip
                    }
                    unknown => {
                        eprintln!("Error: Unknown argument '{}'", unknown);
                        eprintln!("Usage: cargo run -p xtask generate-dataset -- --profile demo|load [--seed <u64>] [--out <filename.db>]");
                        std::process::exit(1);
                    }
                }
                i += 1;
            }

            let profile = match profile {
                Some(p) => p,
                None => {
                    eprintln!("Error: --profile is required (must be 'demo' or 'load')");
                    eprintln!("Usage: cargo run -p xtask generate-dataset -- --profile demo|load [--seed <u64>] [--out <filename.db>]");
                    std::process::exit(1);
                }
            };

            println!(
                "Initializing dataset generator for profile '{:?}'...",
                profile
            );
            let mut gen = match coachify_core::generator::DatasetGenerator::new(
                profile,
                seed,
                out_filename,
            ) {
                Ok(g) => g,
                Err(e) => {
                    eprintln!("Error initializing generator: {}", e);
                    std::process::exit(1);
                }
            };

            println!("Starting generation...");
            match gen.generate() {
                Ok(summary) => {
                    println!("\nDataset generation completed successfully!");
                    println!("========================================");
                    println!("Profile:        {}", summary.profile);
                    println!("Seed:           {}", summary.seed);
                    println!("Students:       {}", summary.student_count);
                    println!("Guardians:      {}", summary.guardian_count);
                    println!("Batches:        {}", summary.batch_count);
                    println!("Charges:        {}", summary.charge_count);
                    println!("Payments:       {}", summary.payment_count);
                    println!("Database path:  {}", summary.db_path);
                    println!(
                        "Elapsed time:   {:.2}s",
                        (summary.duration_ms as f64) / 1000.0
                    );
                    println!("========================================");
                }
                Err(e) => {
                    eprintln!("Error during dataset generation: {}", e);
                    std::process::exit(1);
                }
            }
        }
        "crash-suite" => {
            let mut scenario: Option<String> = None;
            let mut count: Option<usize> = None;

            let mut i = 2;
            while i < args.len() {
                match args[i].as_str() {
                    "--scenario" => {
                        i += 1;
                        if i < args.len() {
                            scenario = Some(args[i].clone());
                        }
                    }
                    "--count" => {
                        i += 1;
                        if i < args.len() {
                            count = args[i].parse().ok();
                        }
                    }
                    "--" => {}
                    other => {
                        eprintln!("Unknown argument for crash-suite: {}", other);
                    }
                }
                i += 1;
            }

            if let Err(e) = crash_suite::run_crash_suite(scenario.as_deref(), count) {
                eprintln!("\nCrash suite failure: {}", e);
                std::process::exit(1);
            }
        }
        other => {
            eprintln!("Unknown xtask command: {}", other);
            eprintln!(
                "Available commands: export-types, check-types, check-migrations, generate-dataset, crash-suite"
            );
            std::process::exit(1);
        }
    }
}
