use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let args: Vec<String> = env::args().collect();
    let command = args.get(1).map(|s| s.as_str()).unwrap_or("export-types");

    match command {
        "export-types" => {
            println!("Exporting TypeScript types to ui/src/gen/rpc.ts...");
            let out_path = PathBuf::from("ui/src/gen/rpc.ts");
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent).expect("Failed to create parent directory for rpc.ts");
            }

            let content = coachify_core::generate_ts_definitions();
            fs::write(&out_path, content).expect("Failed to write rpc.ts");
            println!("TypeScript types successfully exported to {:?}", out_path);
        }
        other => {
            eprintln!("Unknown xtask command: {}", other);
            std::process::exit(1);
        }
    }
}
