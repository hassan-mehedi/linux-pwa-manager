use std::fs;

use anyhow::{Context, Result};
use tauri::State;

use crate::desktop::entry;
use crate::models::settings::AppSettings;
use crate::AppState;

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> Result<AppSettings, String> {
    state
        .settings
        .lock()
        .map(|s| s.clone())
        .map_err(|_| "Settings state is unavailable.".to_owned())
}

fn validate_settings(s: &AppSettings) -> Result<(), String> {
    if s.http_timeout_secs == 0 {
        return Err("http_timeout_secs must be at least 1.".into());
    }
    if s.http_timeout_secs > 300 {
        return Err("http_timeout_secs must not exceed 300.".into());
    }
    if s.max_icon_size_mb == 0 {
        return Err("max_icon_size_mb must be at least 1.".into());
    }
    if s.max_icon_size_mb > 50 {
        return Err("max_icon_size_mb must not exceed 50.".into());
    }
    if s.history_limit == 0 {
        return Err("history_limit must be at least 1.".into());
    }
    if s.history_limit > 10_000 {
        return Err("history_limit must not exceed 10000.".into());
    }
    Ok(())
}

pub fn persist_settings(paths: &crate::paths::ManagedPaths, settings: &AppSettings) -> Result<()> {
    validate_settings(settings).map_err(anyhow::Error::msg)?;
    write_settings(paths, settings)?;
    entry::sync_autostart_entry(paths, settings.launch_on_login)?;
    Ok(())
}

#[tauri::command]
pub fn save_settings(payload: AppSettings, state: State<'_, AppState>) -> Result<(), String> {
    validate_settings(&payload)?;

    let mut guard = state
        .settings
        .lock()
        .map_err(|_| "Settings state is unavailable.".to_owned())?;

    persist_settings(&state.paths, &payload).map_err(|e| e.to_string())?;
    *guard = payload;
    Ok(())
}

pub fn load_settings(paths: &crate::paths::ManagedPaths) -> AppSettings {
    let path = settings_path(paths);
    if !path.exists() {
        return AppSettings::default();
    }
    match fs::read_to_string(&path)
        .context("read")
        .and_then(|s| toml::from_str::<AppSettings>(&s).context("parse"))
    {
        Ok(s) => s,
        Err(e) => {
            log::warn!(
                "Failed to load settings from {}: {e}. Using defaults.",
                path.display()
            );
            AppSettings::default()
        }
    }
}

fn write_settings(paths: &crate::paths::ManagedPaths, settings: &AppSettings) -> Result<()> {
    let path = settings_path(paths);
    let tmp_path = path.with_extension("toml.tmp");
    let content = toml::to_string(settings).context("Failed to serialize settings.")?;
    fs::write(&tmp_path, &content).with_context(|| {
        format!(
            "Failed to write temporary settings file to {}.",
            tmp_path.display()
        )
    })?;
    fs::rename(&tmp_path, &path)
        .with_context(|| format!("Failed to persist settings to {}.", path.display()))?;
    Ok(())
}

fn settings_path(paths: &crate::paths::ManagedPaths) -> std::path::PathBuf {
    paths.config_dir.join("settings.toml")
}
