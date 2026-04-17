use crate::runtime::browser::DetectedBrowser;

#[tauri::command]
pub fn list_browsers() -> Result<Vec<DetectedBrowser>, String> {
    Ok(crate::runtime::browser::detect_browsers())
}
