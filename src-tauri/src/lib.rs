pub mod application;
pub mod backup;
mod commands;
mod dialogs;
pub mod domain;
mod export_settings;
pub mod integrations;
mod preview_cache;
pub mod reports;
pub mod repositories;
mod storage;
pub mod workspace;
use commands::DesktopState;
use std::sync::Mutex;
pub use storage::Storage;
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
            app.manage(integrations::IntegrationState {
                engine: Mutex::new(integrations::Engine::new(&directory)),
                cancel: std::sync::atomic::AtomicBool::new(false),
            });
            app.manage(export_settings::ExportState(Mutex::new(())));
            app.manage(DesktopState(Mutex::new(storage)));
            let handle = app.handle().clone();
            std::thread::spawn(move || loop {
                if let Ok(mut state) = handle.state::<DesktopState>().0.lock() {
                    if let Ok(storage) = state.as_mut() {
                        // Failure is persisted in backup history; no false successful status.
                        if let Err(error) = storage.automatic_backup(crate::backup::now()) {
                            eprintln!("Automatic backup did not complete: {error}");
                        }
                    }
                }
                std::thread::sleep(std::time::Duration::from_secs(60));
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_app_info,
            commands::get_preferences,
            commands::set_theme_preference,
            commands::get_backup_status,
            commands::select_backup_folder,
            commands::backup_now,
            commands::set_automatic_backup,
            commands::select_restore_backup,
            commands::cancel_restore,
            commands::restore_backup,
            commands::get_reference_data,
            commands::save_reference_record,
            commands::list_settlement_records,
            commands::get_settlement_record,
            commands::preview_settlement,
            commands::save_settlement_record,
            commands::finalize_settlement_record,
            commands::list_payroll_records,
            commands::save_driver_payroll,
            commands::save_owner_payroll,
            commands::get_record_history,
            commands::preview_report,
            commands::export_report,
            commands::preview_report_print,
            commands::get_integration_status,
            commands::connect_integration,
            commands::cancel_integration_connection,
            commands::disconnect_integration,
            commands::prepare_external_report,
            commands::confirm_external_report,
            commands::cancel_external_report,
            commands::save_report_as,
            commands::get_export_settings,
            commands::select_export_folder,
            commands::reset_export_folder,
            commands::open_export_folder,
        ])
        .run(tauri::generate_context!())
        .expect("TWX Payroll could not start. Existing application data has not been reset.");
}
