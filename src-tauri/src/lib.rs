use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

mod commands;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(Mutex::new(CancellationToken::new()))
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_oauth::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            commands::download,
            commands::cancel_download
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
