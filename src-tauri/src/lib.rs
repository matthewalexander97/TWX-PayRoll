mod commands;
mod storage;
use commands::DesktopState;
use std::sync::Mutex;
use storage::Storage;
use tauri::Manager;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _, _| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.unminimize();
                let _ = window.set_focus();
            }
        }))
        .setup(|app| {
            let directory = app.path().app_local_data_dir()?;
            let storage = Storage::open(&directory).map_err(|error| {
                format!("Local storage could not open: {error}. Existing data has not been reset.")
            });
            app.manage(DesktopState(Mutex::new(storage)));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_app_info,
            commands::get_preferences,
            commands::set_theme_preference
        ])
        .run(tauri::generate_context!())
        .expect("TWX Payroll could not start. Existing application data has not been reset.");
}
