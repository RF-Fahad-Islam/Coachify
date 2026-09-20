use coachify_core::handle_rpc;
use tauri::command;

#[command]
fn rpc(
    command: String,
    payload: serde_json::Value,
    idempotency_key: Option<String>,
) -> Result<serde_json::Value, String> {
    let _ = idempotency_key;
    let payload_str = serde_json::to_string(&payload).unwrap_or_else(|_| "{}".to_string());
    let response_str = handle_rpc(&command, &payload_str);
    serde_json::from_str(&response_str).map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|_app| {
            // AC-2: the integrity quick check runs once here, at startup, and the
            // application stops instead of serving commands on a corrupt database.
            coachify_core::db::run_startup_checks()
                .map_err(|e| format!("Coachify stopped at startup: {e}"))?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![rpc])
        .run(tauri::generate_context!())
        .expect("error while running Coachify desktop application");
}
