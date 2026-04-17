use anyhow::Result;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager};

use crate::db;
use crate::paths::ManagedPaths;

const TRAY_ID: &str = "linux-pwa-manager-tray";
const OPEN_MANAGER_ID: &str = "tray-open-manager";
const QUIT_ID: &str = "tray-quit";
const LAUNCH_PREFIX: &str = "tray-launch:";

pub fn has_tray_webapps(paths: &ManagedPaths) -> Result<bool> {
    Ok(db::list_webapps(paths)?
        .into_iter()
        .any(|webapp| webapp.tray))
}

pub fn refresh(app: Option<&AppHandle>, paths: &ManagedPaths) -> Result<()> {
    let Some(app) = app else {
        return Ok(());
    };

    let tray_webapps = db::list_webapps(paths)?
        .into_iter()
        .filter(|webapp| webapp.tray)
        .collect::<Vec<_>>();

    if app.tray_by_id(TRAY_ID).is_some() {
        app.remove_tray_by_id(TRAY_ID);
    }

    if tray_webapps.is_empty() {
        return Ok(());
    }

    let menu = Menu::new(app)?;
    for webapp in tray_webapps {
        let item = MenuItem::with_id(
            app,
            format!("{LAUNCH_PREFIX}{}", webapp.id),
            &webapp.name,
            true,
            None::<&str>,
        )?;
        menu.append(&item)?;
    }

    let separator = PredefinedMenuItem::separator(app)?;
    let open_manager = MenuItem::with_id(app, OPEN_MANAGER_ID, "Open Manager", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, QUIT_ID, "Quit", true, None::<&str>)?;
    menu.append(&separator)?;
    menu.append(&open_manager)?;
    menu.append(&quit)?;

    let mut builder = TrayIconBuilder::with_id(TRAY_ID)
        .menu(&menu)
        .tooltip("Linux PWA Manager")
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| {
            if let Err(error) = handle_menu_event(app, event.id.as_ref()) {
                eprintln!("tray menu event failed: {error:#}");
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let Err(error) = handle_tray_icon_event(tray.app_handle(), event) {
                eprintln!("tray icon event failed: {error:#}");
            }
        });

    if let Some(icon) = app.default_window_icon().cloned() {
        builder = builder.icon(icon);
    }

    builder.build(app)?;
    Ok(())
}

fn handle_menu_event(app: &AppHandle, event_id: &str) -> Result<()> {
    if event_id == OPEN_MANAGER_ID {
        return show_manager_window(app);
    }

    if event_id == QUIT_ID {
        app.exit(0);
        return Ok(());
    }

    if let Some(target) = event_id.strip_prefix(LAUNCH_PREFIX) {
        let paths = app.state::<crate::AppState>().paths.clone();
        crate::commands::webapp::launch(Some(app), &paths, target)?;
    }

    Ok(())
}

fn handle_tray_icon_event(app: &AppHandle, event: TrayIconEvent) -> Result<()> {
    if let TrayIconEvent::Click {
        button: MouseButton::Left,
        button_state: MouseButtonState::Up,
        ..
    } = event
    {
        show_manager_window(app)?;
    }

    Ok(())
}

fn show_manager_window(app: &AppHandle) -> Result<()> {
    if let Some(window) = app.get_webview_window("main") {
        window.show()?;
        window.unminimize()?;
        window.set_focus()?;
    }

    Ok(())
}
