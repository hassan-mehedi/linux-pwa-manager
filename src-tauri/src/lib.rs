pub mod cli;
pub mod commands;
pub mod db;
pub mod desktop;
pub mod icons;
pub mod models;
pub mod paths;
pub mod runtime;
pub mod utils;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use anyhow::Result;
use clap::Parser;
use tauri::Manager;

use crate::cli::{Cli, Command};
use crate::models::settings::AppSettings;
use crate::models::webapp::BrowserChoice;
use crate::paths::ManagedPaths;

#[derive(Clone)]
pub struct AppState {
    pub paths: ManagedPaths,
    pub embedded_sessions: Arc<Mutex<HashMap<String, runtime::embedded::EmbeddedSessionState>>>,
    pub settings: Arc<Mutex<AppSettings>>,
}

#[derive(Clone)]
enum StartupAction {
    LaunchEmbedded(String),
}

pub fn bootstrap() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        None => run_tauri(None),
        Some(Command::Launch { target }) => {
            let paths = ManagedPaths::discover()?;
            db::init(&paths)?;
            let webapp = db::find_webapp(&paths, &target)?
                .ok_or_else(|| anyhow::anyhow!("Web app not found."))?;
            if webapp.browser == BrowserChoice::Embedded {
                run_tauri(Some(StartupAction::LaunchEmbedded(target)))
            } else {
                tauri::async_runtime::block_on(cli::run(Command::Launch { target }))
            }
        }
        Some(command) => tauri::async_runtime::block_on(cli::run(command)),
    }
}

fn run_tauri(startup_action: Option<StartupAction>) -> Result<()> {
    let paths = ManagedPaths::discover()?;
    db::init(&paths)?;
    crate::desktop::entry::sync_manager_desktop_entry(&paths)?;
    let settings = commands::settings::load_settings(&paths);
    crate::desktop::entry::sync_autostart_entry(&paths, settings.launch_on_login)?;
    let state = AppState {
        paths: paths.clone(),
        embedded_sessions: Arc::new(Mutex::new(HashMap::new())),
        settings: Arc::new(Mutex::new(settings)),
    };

    tauri::Builder::default()
        .manage(state)
        .setup(move |app| {
            crate::commands::tray::refresh(Some(app.handle()), &paths)?;
            if let Some(window) = app.get_webview_window("main") {
                let paths_for_close = paths.clone();
                let window_for_close = window.clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        if crate::commands::tray::has_tray_webapps(&paths_for_close)
                            .unwrap_or(false)
                        {
                            api.prevent_close();
                            let _ = window_for_close.hide();
                        }
                    }
                });
            }
            if let Some(StartupAction::LaunchEmbedded(target)) = startup_action.clone() {
                crate::commands::webapp::launch(Some(app.handle()), &paths, &target)?;
                if let Some(window) = app.get_webview_window("main") {
                    window.hide()?;
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::backup::open_managed_path,
            commands::backup::export_backup,
            commands::backup::import_backup,
            commands::browser::list_browsers,
            commands::get_app_info,
            commands::settings::get_settings,
            commands::settings::save_settings,
            commands::webapp::get_webapp,
            commands::webapp::list_webapps,
            commands::webapp::create_webapp,
            commands::webapp::update_webapp,
            commands::webapp::delete_webapp,
            commands::webapp::launch_webapp,
            commands::webapp::embedded_current_url,
            commands::webapp::embedded_state,
            commands::webapp::embedded_navigate,
            commands::webapp::embedded_go_back,
            commands::webapp::embedded_go_forward,
            commands::webapp::embedded_reload,
            commands::webapp::embedded_resize,
            commands::webapp::embedded_open_in_browser,
        ])
        .run(tauri::generate_context!())?;

    Ok(())
}
