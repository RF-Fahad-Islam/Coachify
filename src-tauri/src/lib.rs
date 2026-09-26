pub mod updater;

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
        .setup(|app| {
            // AC-1, AC-8: Run startup migrations on staging copy and perform startup checks.
            // If migration fails, capture recovery state so UI displays the Migration Recovery Screen.
            if let Err(e) = coachify_core::migrations::run_startup_lifecycle() {
                eprintln!("Coachify entered migration recovery mode at startup: {e}");
            }

            // Start workstation SSE background listener to receive real-time host change events
            let app_handle = app.handle().clone();
            coachify_core::lan::start_workstation_sse_background(move |events| {
                for event in events {
                    let _ = app_handle.emit("coachify://change", &event);
                }
            });

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                // AC-9: Window close behavior: when host is active, hide window instead of exiting
                if coachify_core::lan::lan_manager().is_running() {
                    let _ = window.hide();
                    api.prevent_close();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![rpc])
        .run(tauri::generate_context!())
        .expect("error while running Coachify desktop application");
}
