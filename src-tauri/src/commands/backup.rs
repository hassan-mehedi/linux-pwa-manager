use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State};
use uuid::Uuid;

use crate::commands::{settings, tray, webapp};
use crate::db;
use crate::desktop::entry;
use crate::models::settings::AppSettings;
use crate::models::webapp::WebApp;
use crate::paths::ManagedPaths;
use crate::utils::normalize_url;
use crate::AppState;

const BACKUP_VERSION: u32 = 1;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BackupFile {
    version: u32,
    settings: AppSettings,
    webapps: Vec<WebApp>,
}

#[tauri::command]
pub fn open_managed_path(path: String, state: State<'_, AppState>) -> Result<(), String> {
    open_path(&state.paths, Path::new(&path)).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn export_backup(path: String, state: State<'_, AppState>) -> Result<(), String> {
    export_to_path(&state.paths, Path::new(&path)).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn import_backup(
    path: String,
    replace_existing: bool,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let imported_settings = import_from_path(&state.paths, Path::new(&path), replace_existing)
        .map_err(|error| error.to_string())?;

    if let Ok(mut guard) = state.settings.lock() {
        *guard = imported_settings;
    }

    tray::refresh(Some(&app), &state.paths).map_err(|error| error.to_string())
}

pub fn open_path(paths: &ManagedPaths, path: &Path) -> Result<()> {
    ensure_managed_path(paths, path)?;
    Command::new("xdg-open")
        .arg(path)
        .spawn()
        .with_context(|| format!("Failed to open {}.", path.display()))?;
    Ok(())
}

pub fn export_to_path(paths: &ManagedPaths, path: &Path) -> Result<()> {
    let mut webapps = db::list_webapps(paths)?;
    for item in &mut webapps {
        item.icon_data_url = crate::icons::icon_data_url_from_path(item.icon_path.as_deref())?;
    }

    let backup = BackupFile {
        version: BACKUP_VERSION,
        settings: settings::load_settings(paths),
        webapps,
    };

    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create backup directory {}.", parent.display()))?;
    }

    let content = serde_json::to_vec_pretty(&backup).context("Failed to serialize backup.")?;
    fs::write(path, content)
        .with_context(|| format!("Failed to write backup file {}.", path.display()))?;
    Ok(())
}

pub fn import_from_path(
    paths: &ManagedPaths,
    path: &Path,
    replace_existing: bool,
) -> Result<AppSettings> {
    let content = fs::read(path)
        .with_context(|| format!("Failed to read backup file {}.", path.display()))?;
    let backup: BackupFile =
        serde_json::from_slice(&content).context("Failed to parse backup file.")?;

    if backup.version != BACKUP_VERSION {
        return Err(anyhow!(
            "Unsupported backup version {}. Expected {}.",
            backup.version,
            BACKUP_VERSION
        ));
    }

    if replace_existing {
        for existing in db::list_webapps(paths)? {
            webapp::delete(paths, &existing.id)?;
        }
    }

    let mut seen_urls = HashSet::new();
    let mut seen_ids = HashSet::new();

    for item in backup.webapps {
        let name = validate_required_text(&item.name, "Name")?;
        let category = validate_required_text(&item.category, "Category")?;
        let normalized_url = normalize_url(&item.url)?.to_string();

        if !seen_urls.insert(normalized_url.clone()) {
            return Err(anyhow!("Backup contains duplicate URL {}.", normalized_url));
        }
        db::ensure_unique_url(paths, &normalized_url, None)?;

        let mut id = item.id.trim().to_owned();
        if id.is_empty() || seen_ids.contains(&id) || db::find_webapp(paths, &id)?.is_some() {
            id = Uuid::new_v4().to_string();
        }
        seen_ids.insert(id.clone());

        let icon_path = match item.icon_data_url.as_deref() {
            Some(data_url) => Some(crate::icons::save_uploaded_icon_data_url(
                paths, &id, data_url,
            )?),
            None => {
                crate::icons::remove_icon(paths, &id)?;
                None
            }
        };

        let imported = WebApp {
            id: id.clone(),
            name,
            url: normalized_url,
            icon_path,
            icon_data_url: None,
            category,
            browser: item.browser,
            nav_bar: item.nav_bar,
            isolated: item.isolated,
            tray: item.tray,
            window_mode: item.window_mode,
            created_at: String::new(),
            updated_at: String::new(),
        };

        db::insert_webapp(paths, &imported)?;
        let stored = db::find_webapp(paths, &id)?
            .ok_or_else(|| anyhow!("Failed to reload imported web app."))?;
        entry::write_desktop_entry(paths, &stored)?;
    }

    settings::persist_settings(paths, &backup.settings)?;
    Ok(backup.settings)
}

fn validate_required_text(value: &str, label: &str) -> Result<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(anyhow!("{label} is required."));
    }
    Ok(trimmed.to_owned())
}

fn ensure_managed_path(paths: &ManagedPaths, path: &Path) -> Result<()> {
    let candidate = canonicalize_existing(path)?;
    let allowed_paths = [
        canonicalize_existing(&paths.data_dir)?,
        canonicalize_existing(&paths.config_dir)?,
        canonicalize_existing(&paths.state_dir)?,
        canonicalize_existing(&paths.db_path)?,
        canonicalize_existing(&paths.desktop_dir)?,
        canonicalize_existing(&paths.profiles_dir)?,
        canonicalize_existing(&paths.icons_dir)?,
    ];

    if allowed_paths
        .iter()
        .any(|allowed| candidate == *allowed || candidate.starts_with(allowed))
    {
        return Ok(());
    }

    Err(anyhow!(
        "Path is outside the managed Linux PWA Manager directories."
    ))
}

fn canonicalize_existing(path: &Path) -> Result<PathBuf> {
    fs::canonicalize(path).with_context(|| format!("Failed to resolve {}.", path.display()))
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use crate::models::webapp::{BrowserChoice, WindowMode};

    fn temp_paths() -> ManagedPaths {
        let root =
            std::env::temp_dir().join(format!("linux-pwa-manager-backup-{}", Uuid::new_v4()));
        let data_dir = root.join("data");
        let config_dir = root.join("config");
        let state_dir = root.join("state");
        let icons_dir = data_dir.join("icons");
        let profiles_dir = data_dir.join("profiles");
        let desktop_dir = data_dir.join("applications");

        for directory in [
            &data_dir,
            &config_dir,
            &state_dir,
            &icons_dir,
            &profiles_dir,
            &desktop_dir,
        ] {
            fs::create_dir_all(directory).unwrap();
        }

        ManagedPaths {
            data_dir: data_dir.clone(),
            config_dir,
            state_dir,
            db_path: data_dir.join("webapps.db"),
            icons_dir,
            profiles_dir,
            desktop_dir,
        }
    }

    fn sample_webapp(id: &str, url: &str) -> WebApp {
        WebApp {
            id: id.to_owned(),
            name: format!("App {id}"),
            url: url.to_owned(),
            icon_path: None,
            icon_data_url: None,
            category: "Internet".to_owned(),
            browser: BrowserChoice::Auto,
            nav_bar: false,
            isolated: true,
            tray: false,
            window_mode: WindowMode::Normal,
            created_at: String::new(),
            updated_at: String::new(),
        }
    }

    #[test]
    fn export_and_import_round_trip_restores_webapps_and_settings() {
        let source = temp_paths();
        let backup_path = source.data_dir.join("backup.json");
        db::init(&source).unwrap();
        settings::persist_settings(
            &source,
            &AppSettings {
                http_timeout_secs: 30,
                max_icon_size_mb: 7,
                history_limit: 70,
                default_browser: crate::models::webapp::BrowserChoice::Firefox,
                default_window_mode: crate::models::webapp::WindowMode::Maximized,
                launch_on_login: true,
                theme: crate::models::settings::ThemePreference::Dark,
            },
        )
        .unwrap();
        db::insert_webapp(&source, &sample_webapp("mail", "https://mail.example.com")).unwrap();
        export_to_path(&source, &backup_path).unwrap();

        let target = temp_paths();
        db::init(&target).unwrap();
        let imported_settings = import_from_path(&target, &backup_path, false).unwrap();

        let imported = db::list_webapps(&target).unwrap();
        assert_eq!(imported.len(), 1);
        assert_eq!(imported[0].url, "https://mail.example.com/");
        assert!(imported_settings.launch_on_login);

        fs::remove_dir_all(source.data_dir.parent().unwrap()).unwrap();
        fs::remove_dir_all(target.data_dir.parent().unwrap()).unwrap();
    }

    #[test]
    fn import_rejects_duplicate_urls_in_backup() {
        let paths = temp_paths();
        db::init(&paths).unwrap();
        let backup_path = paths.data_dir.join("invalid.json");
        let backup = BackupFile {
            version: BACKUP_VERSION,
            settings: AppSettings::default(),
            webapps: vec![
                sample_webapp("one", "https://example.com"),
                sample_webapp("two", "https://example.com"),
            ],
        };
        fs::write(&backup_path, serde_json::to_vec(&backup).unwrap()).unwrap();

        let error = import_from_path(&paths, &backup_path, false).unwrap_err();
        assert!(error.to_string().contains("duplicate URL"));

        fs::remove_dir_all(paths.data_dir.parent().unwrap()).unwrap();
    }
}
