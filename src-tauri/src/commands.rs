use crate::backup::{BackupStatus, RestorePreview};
use crate::storage::{AppInfo, Preferences, Storage, Theme};
use std::sync::Mutex;
use tauri::{Manager, State};

pub struct DesktopState(pub Mutex<Result<Storage, String>>);
async fn external<T: Send + 'static>(
    app: tauri::AppHandle,
    operation: impl FnOnce(&mut crate::integrations::Engine) -> Result<T, String> + Send + 'static,
) -> Result<T, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<crate::integrations::IntegrationState>();
        let mut engine = state
            .engine
            .lock()
            .map_err(|_| "Integration state unavailable")?;
        operation(engine.as_mut().map_err(|e| e.clone())?)
    })
    .await
    .map_err(|_| "Integration worker did not complete".to_string())?
}
#[tauri::command]
pub async fn get_integration_status(
    app: tauri::AppHandle,
) -> Result<Vec<crate::integrations::Status>, String> {
    external(app, |e| Ok(e.status())).await
}
#[tauri::command]
pub async fn cancel_integration_connection(app: tauri::AppHandle) {
    app.state::<crate::integrations::IntegrationState>()
        .cancel
        .store(true, std::sync::atomic::Ordering::SeqCst);
}
#[tauri::command]
pub async fn connect_integration(
    service: crate::integrations::Service,
    app: tauri::AppHandle,
) -> Result<Option<Vec<crate::integrations::Status>>, String> {
    #[cfg(windows)]
    let owner = app
        .get_webview_window("main")
        .ok_or("Main window unavailable")?
        .hwnd()
        .map_err(|_| "Window unavailable")?
        .0 as isize;
    #[cfg(not(windows))]
    let owner = 0;
    app.state::<crate::integrations::IntegrationState>()
        .cancel
        .store(false, std::sync::atomic::Ordering::SeqCst);
    tauri::async_runtime::spawn_blocking(move || {
        use std::io::Read;
        let path =
            crate::dialogs::google_client(owner).map_err(|_| "Google client selection failed")?;
        let Some(path) = path else {
            return Ok(None);
        };
        let mut bytes = vec![];
        std::fs::File::open(path)
            .map_err(|_| "Could not open the selected Google client file")?
            .take(16_385)
            .read_to_end(&mut bytes)
            .map_err(|_| "Could not read the Google client file")?;
        let state = app.state::<crate::integrations::IntegrationState>();
        let mut engine = state.engine.lock().map_err(|_| "Integration unavailable")?;
        let engine = engine.as_mut().map_err(|e| e.clone())?;
        engine.connect(service, &bytes, &state.cancel)?;
        Ok(Some(engine.status()))
    })
    .await
    .map_err(|_| "Google connection worker stopped".to_string())?
}
#[tauri::command]
pub async fn disconnect_integration(
    service: crate::integrations::Service,
    app: tauri::AppHandle,
) -> Result<crate::integrations::Receipt, String> {
    external(app, move |e| e.disconnect(service)).await
}
#[tauri::command]
pub async fn prepare_external_report(
    request: crate::reports::Request,
    service: crate::integrations::Service,
    email: Option<crate::integrations::output::Email>,
    app: tauri::AppHandle,
) -> Result<crate::integrations::Prepared, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let report = app
            .state::<DesktopState>()
            .access(|s| crate::reports::build(s, &request))?;
        let state = app.state::<crate::integrations::IntegrationState>();
        let mut engine = state.engine.lock().map_err(|_| "Integration unavailable")?;
        engine
            .as_mut()
            .map_err(|e| e.clone())?
            .prepare(report, service, email)
    })
    .await
    .map_err(|_| "Report preparation worker stopped".to_string())?
}
#[tauri::command]
pub async fn confirm_external_report(
    token: String,
    confirmed: bool,
    app: tauri::AppHandle,
) -> Result<crate::integrations::Receipt, String> {
    external(app, move |e| e.confirm(&token, confirmed)).await
}
#[tauri::command]
pub async fn cancel_external_report(token: String, app: tauri::AppHandle) -> Result<(), String> {
    external(app, move |e| {
        e.cancel(&token);
        Ok(())
    })
    .await
}
impl DesktopState {
    fn business_access<T>(
        &self,
        operation: impl FnOnce(&mut Storage) -> crate::workspace::Result<T>,
    ) -> crate::workspace::Result<T> {
        let mut state = self.0.lock().map_err(|_| {
            crate::workspace::unavailable("Storage is unavailable. Restart the application.")
        })?;
        let storage = state
            .as_mut()
            .map_err(|e| crate::workspace::unavailable(e))?;
        if storage.recovery_blocked {
            return Err(crate::workspace::unavailable(
                "Recovery is pending. Restart before changing records.",
            ));
        }
        operation(storage)
    }
    fn access<T>(
        &self,
        operation: impl FnOnce(&mut Storage) -> crate::backup::Result<T>,
    ) -> Result<T, String> {
        let mut state = self
            .0
            .lock()
            .map_err(|_| "Storage is unavailable. Restart the application.")?;
        let storage = state.as_mut().map_err(|e| e.clone())?;
        if storage.recovery_blocked {
            return Err(
                "Recovery is pending. Restart the application before making changes.".into(),
            );
        }
        operation(storage).map_err(|e| e.to_string())
    }
}

#[tauri::command]
pub async fn get_reference_data(
    state: State<'_, DesktopState>,
) -> crate::workspace::Result<serde_json::Value> {
    state.business_access(|s| crate::workspace::references(s))
}
#[tauri::command]
pub async fn save_reference_record(
    input: crate::workspace::ReferenceWrite,
    state: State<'_, DesktopState>,
) -> crate::workspace::Result<serde_json::Value> {
    state.business_access(|s| crate::workspace::save_reference(s, input))
}
#[tauri::command]
pub async fn list_settlement_records(
    filter: crate::workspace::RecordFilter,
    state: State<'_, DesktopState>,
) -> crate::workspace::Result<Vec<serde_json::Value>> {
    state.business_access(|s| crate::workspace::settlements(s, filter))
}
#[tauri::command]
pub async fn get_settlement_record(
    id: String,
    state: State<'_, DesktopState>,
) -> crate::workspace::Result<serde_json::Value> {
    state.business_access(|s| crate::workspace::settlement(s, &id))
}
#[tauri::command]
pub async fn preview_settlement(
    input: crate::workspace::SettlementWrite,
    state: State<'_, DesktopState>,
) -> crate::workspace::Result<serde_json::Value> {
    state.business_access(|s| crate::workspace::preview(s, &input))
}
#[tauri::command]
pub async fn save_settlement_record(
    input: crate::workspace::SettlementWrite,
    confirm_negative: bool,
    state: State<'_, DesktopState>,
) -> crate::workspace::Result<serde_json::Value> {
    state.business_access(|s| crate::workspace::save_settlement(s, input, confirm_negative))
}
#[tauri::command]
pub async fn finalize_settlement_record(
    id: String,
    expected_version: String,
    confirm_negative: bool,
    state: State<'_, DesktopState>,
) -> crate::workspace::Result<serde_json::Value> {
    state.business_access(|s| {
        crate::workspace::finalize(s, &id, &expected_version, confirm_negative)
    })
}
#[tauri::command]
pub async fn list_payroll_records(
    filter: crate::workspace::RecordFilter,
    state: State<'_, DesktopState>,
) -> crate::workspace::Result<Vec<serde_json::Value>> {
    state.business_access(|s| crate::workspace::payrolls(s, filter))
}
#[tauri::command]
pub async fn save_driver_payroll(
    input: crate::workspace::DriverPayrollWrite,
    state: State<'_, DesktopState>,
) -> crate::workspace::Result<serde_json::Value> {
    state.business_access(|s| crate::workspace::save_driver(s, input))
}
#[tauri::command]
pub async fn save_owner_payroll(
    input: crate::workspace::OwnerPayrollWrite,
    state: State<'_, DesktopState>,
) -> crate::workspace::Result<serde_json::Value> {
    state.business_access(|s| crate::workspace::save_owner(s, input))
}
#[tauri::command]
pub async fn get_record_history(
    entity: crate::workspace::Entity,
    id: String,
    state: State<'_, DesktopState>,
) -> crate::workspace::Result<Vec<serde_json::Value>> {
    state.business_access(|s| crate::workspace::events(s, entity, &id))
}
#[tauri::command]
pub async fn get_preferences(state: State<'_, DesktopState>) -> Result<Preferences, String> {
    state.access(|s| Ok(s.preferences()?))
}
#[tauri::command]
pub async fn set_theme_preference(
    theme: Theme,
    state: State<'_, DesktopState>,
) -> Result<Preferences, String> {
    state.access(|s| Ok(s.set_theme(theme)?))
}
#[tauri::command]
pub async fn get_app_info(state: State<'_, DesktopState>) -> Result<AppInfo, String> {
    state.access(|s| Ok(s.info()?))
}

// Snapshot/restore I/O and native dialogs never block the frontend event thread.
async fn blocking<T: Send + 'static>(
    app: tauri::AppHandle,
    operation: impl FnOnce(&DesktopState) -> Result<T, String> + Send + 'static,
) -> Result<T, String> {
    tauri::async_runtime::spawn_blocking(move || operation(&app.state::<DesktopState>()))
        .await
        .map_err(|e| e.to_string())?
}
#[tauri::command]
pub async fn preview_report(
    request: crate::reports::Request,
    app: tauri::AppHandle,
) -> Result<crate::reports::Report, String> {
    blocking(app, move |state| {
        state.access(|s| crate::reports::build(s, &request))
    })
    .await
}
#[tauri::command]
pub async fn save_report_as(
    request: crate::reports::Request,
    format: crate::reports::Format,
    app: tauri::AppHandle,
) -> Result<Option<serde_json::Value>, String> {
    #[cfg(windows)]
    let owner = app
        .get_webview_window("main")
        .ok_or("Main window is unavailable")?
        .hwnd()
        .map_err(|e| e.to_string())?
        .0 as isize;
    #[cfg(not(windows))]
    let owner = 0;
    blocking(app,move|state|{
        let (report,app_data)=state.access(|s|Ok((crate::reports::build(s,&request)?,s.app_data.clone())))?;
        let bytes=crate::reports::render(&report,format).map_err(|e|e.to_string())?;
        let selected=crate::dialogs::save_report(&format!("{}.{}",report.default_name,format.extension()),format.extension(),owner).map_err(|e|e.to_string())?;
        selected.map(|path|{let saved=crate::reports::files::publish(&path,format,&bytes,&app_data).map_err(|e|e.to_string())?;Ok(serde_json::json!({"fileName":saved.file_name().unwrap_or_default().to_string_lossy(),"sizeBytes":bytes.len().to_string(),"message":"Saved complete report. Existing files were not overwritten."}))}).transpose()
    }).await
}
fn owner_window(app: &tauri::AppHandle) -> Result<isize, String> {
    #[cfg(windows)]
    {
        Ok(app
            .get_webview_window("main")
            .ok_or("Main window is unavailable")?
            .hwnd()
            .map_err(|e| e.to_string())?
            .0 as isize)
    }
    #[cfg(not(windows))]
    {
        let _ = app;
        Ok(0)
    }
}
async fn export_operation<T: Send + 'static>(
    app: tauri::AppHandle,
    operation: impl FnOnce(&tauri::AppHandle, &std::path::Path) -> crate::reports::Result<T>
        + Send
        + 'static,
) -> Result<T, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<crate::export_settings::ExportState>();
        let _guard = state
            .0
            .lock()
            .map_err(|_| "Export settings unavailable. Restart TWX Payroll.")?;
        let root = app.path().app_local_data_dir().map_err(|e| e.to_string())?;
        operation(&app, &root).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}
#[tauri::command]
pub async fn get_export_settings(
    app: tauri::AppHandle,
) -> Result<crate::export_settings::Status, String> {
    export_operation(app, |app, root| {
        crate::export_settings::status(root, || Ok(app.path().document_dir()?))
    })
    .await
}
#[tauri::command]
pub async fn select_export_folder(
    app: tauri::AppHandle,
) -> Result<Option<crate::export_settings::Status>, String> {
    let owner = owner_window(&app)?;
    export_operation(app, move |app, root| {
        let Some(path) = crate::dialogs::pick_owned(true, true, owner)? else {
            return Ok(None);
        };
        crate::export_settings::save(root, &path)?;
        Ok(Some(crate::export_settings::status(root, || {
            Ok(app.path().document_dir()?)
        })?))
    })
    .await
}
#[tauri::command]
pub async fn reset_export_folder(
    app: tauri::AppHandle,
) -> Result<crate::export_settings::Status, String> {
    export_operation(app, |app, root| {
        crate::export_settings::reset(root, || Ok(app.path().document_dir()?))?;
        crate::export_settings::status(root, || Ok(app.path().document_dir()?))
    })
    .await
}
#[tauri::command]
pub async fn open_export_folder(app: tauri::AppHandle) -> Result<(), String> {
    export_operation(app, |app, root| {
        let (path, _) =
            crate::export_settings::directory(root, || Ok(app.path().document_dir()?))?;
        let path = crate::export_settings::validate_folder(&path, root)?;
        crate::reports::files::open_local(&path)
    })
    .await
}
#[tauri::command]
pub async fn export_report(
    request: crate::reports::Request,
    format: crate::reports::Format,
    app: tauri::AppHandle,
) -> Result<Option<serde_json::Value>, String> {
    export_operation(app, move |app, root| {
        let report = app.state::<DesktopState>().access(|s| crate::reports::build(s, &request))?;
        let bytes = crate::reports::render(&report, format)?;
        let saved = crate::export_settings::publish(root, || Ok(app.path().document_dir()?), &report.default_name, format, &bytes)?;
        Ok(Some(serde_json::json!({"fileName":saved.file_name().unwrap_or_default().to_string_lossy(),"sizeBytes":bytes.len().to_string(),"message":"Saved complete report in the configured export folder. Existing files were not overwritten."})))
    }).await
}
#[tauri::command]
pub async fn preview_report_print(
    request: crate::reports::Request,
    app: tauri::AppHandle,
) -> Result<serde_json::Value, String> {
    export_operation(app,move|app,app_data|{
        let report=app.state::<DesktopState>().access(|s|crate::reports::build(s,&request))?;
        let path=crate::reports::files::preview_pdf(&report,app_data).map_err(|e|e.to_string())?;
        crate::reports::files::open_pdf(&path).map_err(|e|e.to_string())?;
        Ok(serde_json::json!({"fileName":path.file_name().unwrap_or_default().to_string_lossy(),"message":"Print PDF opened in your local viewer. Review it and choose Print there."}))
    }).await
}
#[tauri::command]
pub async fn get_backup_status(app: tauri::AppHandle) -> Result<BackupStatus, String> {
    blocking(app, |s| s.access(|db| db.backup_status())).await
}
#[tauri::command]
pub async fn select_backup_folder(app: tauri::AppHandle) -> Result<Option<BackupStatus>, String> {
    blocking(app, |s| {
        let path = crate::dialogs::pick(true).map_err(|e| e.to_string())?;
        path.map(|p| s.access(|db| db.configure_backup_destination(&p)))
            .transpose()
    })
    .await
}
#[tauri::command]
pub async fn backup_now(app: tauri::AppHandle) -> Result<BackupStatus, String> {
    blocking(app, |s| s.access(|db| db.backup_now())).await
}
#[tauri::command]
pub async fn set_automatic_backup(
    enabled: bool,
    app: tauri::AppHandle,
) -> Result<BackupStatus, String> {
    blocking(app, move |s| {
        s.access(|db| db.set_automatic_backup(enabled))
    })
    .await
}
#[tauri::command]
pub async fn select_restore_backup(
    app: tauri::AppHandle,
) -> Result<Option<RestorePreview>, String> {
    blocking(app, |s| {
        let path = crate::dialogs::pick(false).map_err(|e| e.to_string())?;
        path.map(|p| s.access(|db| db.prepare_restore(&p)))
            .transpose()
    })
    .await
}
#[tauri::command]
pub async fn cancel_restore(app: tauri::AppHandle) -> Result<(), String> {
    blocking(app, |s| s.access(|db| db.cancel_restore())).await
}
#[tauri::command]
pub async fn restore_backup(
    token: String,
    confirmed: bool,
    app: tauri::AppHandle,
) -> Result<BackupStatus, String> {
    blocking(app, move |s| {
        s.access(|db| db.restore_backup(&token, confirmed))
    })
    .await
}
