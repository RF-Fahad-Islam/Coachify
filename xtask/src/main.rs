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

            let content = coachify_core::generate_ts_definitions();
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
            let expected = coachify_core::generate_ts_definitions();

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
        other => {
            eprintln!("Unknown xtask command: {}", other);
            eprintln!("Available commands: export-types, check-types, check-migrations");
            std::process::exit(1);
        }
    }
}
