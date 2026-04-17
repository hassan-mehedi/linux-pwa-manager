use anyhow::{anyhow, Result};
use tauri::{AppHandle, State};
use uuid::Uuid;

use crate::commands::{icon, tray};
use crate::db;
use crate::desktop::entry;
use crate::models::webapp::{BrowserChoice, WebApp, WebAppPayload};
use crate::paths::ManagedPaths;
use crate::runtime::{browser, embedded};
use crate::utils::normalize_url;
use crate::AppState;

#[tauri::command]
pub async fn list_webapps(state: State<'_, AppState>) -> Result<Vec<WebApp>, String> {
    list(&state.paths).map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn get_webapp(target: String, state: State<'_, AppState>) -> Result<WebApp, String> {
    db::find_webapp(&state.paths, &target)
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "Web app not found.".to_owned())
        .and_then(|webapp| decorate_webapp(webapp).map_err(|error| error.to_string()))
}

#[tauri::command]
pub async fn create_webapp(
    payload: WebAppPayload,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<WebApp, String> {
    let (timeout_secs, max_bytes) = state
        .settings
        .lock()
        .map(|s| (s.http_timeout_secs, s.max_icon_size_mb * 1024 * 1024))
        .unwrap_or((10, 5 * 1024 * 1024));

    create(&state.paths, payload, timeout_secs, max_bytes)
        .await
        .and_then(|webapp| {
            tray::refresh(Some(&app), &state.paths)?;
            Ok(webapp)
        })
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn update_webapp(
    payload: WebAppPayload,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<WebApp, String> {
    let (timeout_secs, max_bytes) = state
        .settings
        .lock()
        .map(|s| (s.http_timeout_secs, s.max_icon_size_mb * 1024 * 1024))
        .unwrap_or((10, 5 * 1024 * 1024));

    update(&state.paths, payload, timeout_secs, max_bytes)
        .await
        .and_then(|webapp| {
            tray::refresh(Some(&app), &state.paths)?;
            Ok(webapp)
        })
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn delete_webapp(
    target: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    delete(&state.paths, &target)
        .and_then(|_| tray::refresh(Some(&app), &state.paths))
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn launch_webapp(
    target: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    launch(Some(&app), &state.paths, &target).map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn embedded_current_url(target: String, app: AppHandle) -> Result<String, String> {
    crate::runtime::embedded::current_url(&app, &target).map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn embedded_state(
    target: String,
    app: AppHandle,
) -> Result<crate::runtime::embedded::EmbeddedPageState, String> {
    crate::runtime::embedded::current_state(&app, &target).map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn embedded_navigate(
    target: String,
    url: String,
    app: AppHandle,
) -> Result<String, String> {
    crate::runtime::embedded::navigate(&app, &target, &url).map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn embedded_go_back(target: String, app: AppHandle) -> Result<(), String> {
    crate::runtime::embedded::go_back(&app, &target).map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn embedded_go_forward(target: String, app: AppHandle) -> Result<(), String> {
    crate::runtime::embedded::go_forward(&app, &target).map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn embedded_reload(target: String, app: AppHandle) -> Result<(), String> {
    crate::runtime::embedded::reload(&app, &target).map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn embedded_resize(target: String, top_inset: f64, app: AppHandle) -> Result<(), String> {
    crate::runtime::embedded::resize_embedded_content(&app, &target, top_inset)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn embedded_open_in_browser(target: String, app: AppHandle) -> Result<(), String> {
    crate::runtime::embedded::open_in_default_browser(&app, &target)
        .map_err(|error| error.to_string())
}

pub fn list(paths: &ManagedPaths) -> Result<Vec<WebApp>> {
    db::list_webapps(paths)?
        .into_iter()
        .map(decorate_webapp)
        .collect()
}

pub async fn create(
    paths: &ManagedPaths,
    payload: WebAppPayload,
    timeout_secs: u64,
    max_bytes: u64,
) -> Result<WebApp> {
    let name = validate_required_text(&payload.name, "name")?;
    let category = validate_required_text(&payload.category, "category")?;
    let normalized_url = normalize_url(&payload.url)?;
    db::ensure_unique_url(paths, normalized_url.as_str(), None)?;

    let id = payload.id.unwrap_or_else(|| Uuid::new_v4().to_string());
    let icon_path = if payload.clear_icon {
        crate::icons::remove_icon(paths, &id)?;
        None
    } else {
        icon::store_icon(
            paths,
            &id,
            normalized_url.as_str(),
            payload.uploaded_icon_data_url.as_deref(),
            payload.icon_source_url.as_deref(),
            timeout_secs,
            max_bytes,
        )
        .await?
    };

    let webapp = WebApp {
        id: id.clone(),
        name,
        url: normalized_url.to_string(),
        icon_path,
        icon_data_url: None,
        category,
        browser: payload.browser,
        nav_bar: payload.nav_bar,
        isolated: payload.isolated,
        tray: payload.tray,
        window_mode: payload.window_mode,
        created_at: String::new(),
        updated_at: String::new(),
    };

    db::insert_webapp(paths, &webapp)?;
    let stored = db::find_webapp(paths, &id)?
        .ok_or_else(|| anyhow!("Failed to read back saved web app."))?;
    entry::write_desktop_entry(paths, &stored)?;
    tray::refresh(None, paths)?;
    decorate_webapp(stored)
}

pub async fn update(
    paths: &ManagedPaths,
    payload: WebAppPayload,
    timeout_secs: u64,
    max_bytes: u64,
) -> Result<WebApp> {
    let name = validate_required_text(&payload.name, "name")?;
    let category = validate_required_text(&payload.category, "category")?;
    let id = payload
        .id
        .clone()
        .ok_or_else(|| anyhow!("Missing web app ID for update."))?;
    let existing = db::find_webapp(paths, &id)?.ok_or_else(|| anyhow!("Web app not found."))?;
    let normalized_url = normalize_url(&payload.url)?;
    db::ensure_unique_url(paths, normalized_url.as_str(), Some(&id))?;

    let icon_path = if payload.clear_icon {
        crate::icons::remove_icon(paths, &id)?;
        None
    } else if payload.uploaded_icon_data_url.is_some() {
        // User uploaded a new icon — always store it.
        icon::store_icon(
            paths,
            &id,
            normalized_url.as_str(),
            payload.uploaded_icon_data_url.as_deref(),
            payload.icon_source_url.as_deref(),
            timeout_secs,
            max_bytes,
        )
        .await?
    } else if existing.icon_path.is_some()
        && normalized_url.as_str() == existing.url
        && payload.icon_source_url.is_none()
    {
        // URL unchanged, no explicit icon override, existing icon present — skip refetch.
        existing.icon_path.clone()
    } else {
        // URL changed or explicit icon URL provided or no existing icon — fetch fresh.
        let fetched = icon::store_icon(
            paths,
            &id,
            normalized_url.as_str(),
            None,
            payload.icon_source_url.as_deref(),
            timeout_secs,
            max_bytes,
        )
        .await?;
        fetched.or(existing.icon_path.clone())
    };

    let webapp = WebApp {
        id: id.clone(),
        name,
        url: normalized_url.to_string(),
        icon_path,
        icon_data_url: None,
        category,
        browser: payload.browser,
        nav_bar: payload.nav_bar,
        isolated: payload.isolated,
        tray: payload.tray,
        window_mode: payload.window_mode,
        created_at: existing.created_at,
        updated_at: existing.updated_at,
    };

    db::update_webapp(paths, &webapp)?;
    let stored =
        db::find_webapp(paths, &id)?.ok_or_else(|| anyhow!("Failed to reload updated web app."))?;
    entry::write_desktop_entry(paths, &stored)?;
    tray::refresh(None, paths)?;
    decorate_webapp(stored)
}

pub fn delete(paths: &ManagedPaths, target: &str) -> Result<()> {
    let webapp = db::find_webapp(paths, target)?.ok_or_else(|| anyhow!("Web app not found."))?;
    db::delete_webapp(paths, &webapp.id)?;
    crate::icons::remove_icon(paths, &webapp.id)?;
    entry::remove_desktop_entry(paths, &webapp.id)?;
    let profile_dir = paths.profiles_dir.join(&webapp.id);
    if profile_dir.exists() {
        std::fs::remove_dir_all(profile_dir)?;
    }
    tray::refresh(None, paths)?;
    Ok(())
}

pub fn launch(app: Option<&AppHandle>, paths: &ManagedPaths, target: &str) -> Result<()> {
    let webapp = db::find_webapp(paths, target)?.ok_or_else(|| anyhow!("Web app not found."))?;
    match webapp.browser {
        BrowserChoice::Embedded => {
            let handle =
                app.ok_or_else(|| anyhow!("Embedded launches require the GUI runtime."))?;
            embedded::launch_embedded(handle, &webapp, paths)?;
        }
        _ => browser::launch_system_browser(&webapp, paths)?,
    }

    Ok(())
}

fn decorate_webapp(mut webapp: WebApp) -> Result<WebApp> {
    webapp.icon_data_url = crate::icons::icon_data_url_from_path(webapp.icon_path.as_deref())?;
    Ok(webapp)
}

fn validate_required_text(value: &str, field_name: &str) -> Result<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        let label = match field_name {
            "name" => "Name",
            "category" => "Category",
            _ => field_name,
        };
        return Err(anyhow!("{label} is required."));
    }

    Ok(trimmed.to_owned())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use crate::models::webapp::WindowMode;

    fn temp_paths() -> ManagedPaths {
        let root =
            std::env::temp_dir().join(format!("webapp-manager-command-{}", uuid::Uuid::new_v4()));
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

    #[test]
    fn list_includes_icon_data_urls_for_stored_icons() {
        let paths = temp_paths();
        crate::db::init(&paths).unwrap();

        let icon_path = paths.icons_dir.join("mail.png");
        image::RgbaImage::from_pixel(1, 1, image::Rgba([30, 90, 180, 255]))
            .save(&icon_path)
            .unwrap();

        crate::db::insert_webapp(
            &paths,
            &WebApp {
                id: "mail".to_owned(),
                name: "Mail".to_owned(),
                url: "https://mail.example.com".to_owned(),
                icon_path: Some(crate::paths::display_path(&icon_path)),
                icon_data_url: None,
                category: "Internet".to_owned(),
                browser: BrowserChoice::Auto,
                nav_bar: false,
                isolated: true,
                tray: false,
                window_mode: WindowMode::Normal,
                created_at: String::new(),
                updated_at: String::new(),
            },
        )
        .unwrap();

        let items = list(&paths).unwrap();
        assert_eq!(items.len(), 1);
        assert!(items[0]
            .icon_data_url
            .as_deref()
            .is_some_and(|value| value.starts_with("data:image/png;base64,")));

        fs::remove_dir_all(paths.data_dir.parent().unwrap()).unwrap();
    }
}
