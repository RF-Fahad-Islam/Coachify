use coachify_core::handle_rpc_with_events;
use tauri::{command, Emitter};

#[command]
fn rpc(
    app: tauri::AppHandle,
    command: String,
    payload: serde_json::Value,
    session_token: Option<String>,
    idempotency_key: Option<String>,
) -> Result<serde_json::Value, String> {
    let payload_str = serde_json::to_string(&payload).unwrap_or_else(|_| "{}".to_string());
    let (response_str, events) = handle_rpc_with_events(
        &command,
        &payload_str,
        session_token.as_deref(),
        idempotency_key.as_deref(),
    );

    // AC-8: Broadcast typed change events over Tauri event channel
    for event in events {
        let _ = app.emit("coachify://change", &event);
    }

    serde_json::from_str(&response_str).map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|_app| {
            // AC-1, AC-8: Run startup migrations on staging copy and perform startup checks.
            // If migration fails, capture recovery state so UI displays the Migration Recovery Screen.
            if let Err(e) = coachify_core::migrations::run_startup_lifecycle() {
                eprintln!("Coachify entered migration recovery mode at startup: {e}");
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![rpc])
        .run(tauri::generate_context!())
        .expect("error while running Coachify desktop application");
}
