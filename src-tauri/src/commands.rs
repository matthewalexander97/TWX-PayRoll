use crate::storage::{AppInfo, Preferences, Storage, Theme};
use std::sync::Mutex;
use tauri::State;

pub struct DesktopState(pub Mutex<Result<Storage, String>>);
impl DesktopState {
    fn access<T>(
        &self,
        operation: impl FnOnce(&mut Storage) -> Result<T, rusqlite::Error>,
    ) -> Result<T, String> {
        let mut state = self
            .0
            .lock()
            .map_err(|_| "Storage is unavailable. Restart the application.")?;
        let storage = state.as_mut().map_err(|error| error.clone())?;
        operation(storage)
            .map_err(|_| "Local settings operation failed. Check disk access and try again.".into())
    }
}

// Async commands dispatch off the UI thread; the mutex serializes this sole connection.
#[tauri::command]
pub async fn get_preferences(state: State<'_, DesktopState>) -> Result<Preferences, String> {
    state.access(|storage| storage.preferences())
}
#[tauri::command]
pub async fn set_theme_preference(
    theme: Theme,
    state: State<'_, DesktopState>,
) -> Result<Preferences, String> {
    state.access(|storage| storage.set_theme(theme))
}
#[tauri::command]
pub async fn get_app_info(state: State<'_, DesktopState>) -> Result<AppInfo, String> {
    state.access(|storage| storage.info())
}
