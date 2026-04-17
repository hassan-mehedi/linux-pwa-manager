pub mod backup;
pub mod browser;
pub mod icon;
pub mod settings;
pub mod tray;
pub mod webapp;

use tauri::State;

use crate::models::webapp::AppInfo;
use crate::AppState;

#[tauri::command]
pub fn get_app_info(state: State<'_, AppState>) -> Result<AppInfo, String> {
    Ok(state.paths.app_info())
}
