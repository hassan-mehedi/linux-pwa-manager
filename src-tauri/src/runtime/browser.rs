use std::collections::HashSet;
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{anyhow, Context, Result};

use crate::desktop::entry;
use crate::models::webapp::{BrowserChoice, WebApp, WindowMode};
use crate::paths::ManagedPaths;
use crate::runtime::isolation::webapp_profile_dir;

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DetectedBrowser {
    pub id: BrowserChoice,
    pub name: String,
    pub path: String,
    pub version: Option<String>,
    pub supports_app_mode: bool,
}

#[derive(Debug, Clone)]
struct BrowserDefinition {
    id: BrowserChoice,
    name: &'static str,
    commands: &'static [&'static str],
    flatpak_apps: &'static [(&'static str, &'static str)],
}

pub fn detect_browsers() -> Vec<DetectedBrowser> {
    let definitions = [
        BrowserDefinition {
            id: BrowserChoice::Chrome,
            name: "Google Chrome",
            commands: &["google-chrome", "google-chrome-stable"],
            flatpak_apps: &[("com.google.Chrome", "Google Chrome")],
        },
        BrowserDefinition {
            id: BrowserChoice::Chromium,
            name: "Chromium",
            commands: &["chromium", "chromium-browser"],
            flatpak_apps: &[("org.chromium.Chromium", "Chromium")],
        },
        BrowserDefinition {
            id: BrowserChoice::Brave,
            name: "Brave",
            commands: &["brave-browser"],
            flatpak_apps: &[("com.brave.Browser", "Brave")],
        },
        BrowserDefinition {
            id: BrowserChoice::Edge,
            name: "Microsoft Edge",
            commands: &["microsoft-edge", "microsoft-edge-stable"],
            flatpak_apps: &[("com.microsoft.Edge", "Microsoft Edge")],
        },
        BrowserDefinition {
            id: BrowserChoice::Vivaldi,
            name: "Vivaldi",
            commands: &["vivaldi", "vivaldi-stable"],
            flatpak_apps: &[("com.vivaldi.Vivaldi", "Vivaldi")],
        },
        BrowserDefinition {
            id: BrowserChoice::Firefox,
            name: "Firefox",
            commands: &["firefox"],
            flatpak_apps: &[
                ("org.mozilla.firefox", "Firefox"),
                ("app.zen_browser.zen", "Zen"),
                ("io.gitlab.librewolf-community", "Librewolf"),
                ("one.ablaze.floorp", "Floorp"),
            ],
        },
    ];

    let mut seen: HashSet<String> = HashSet::new();
    let mut browsers = Vec::new();

    for definition in definitions {
        let mut detected = false;

        for command in definition.commands {
            let Some(path) = resolve_from_path(command) else {
                continue;
            };
            let target = path.to_string_lossy().into_owned();
            if !seen.insert(target.clone()) {
                continue;
            }

            let version = Command::new(&path)
                .arg("--version")
                .output()
                .ok()
                .and_then(|output| String::from_utf8(output.stdout).ok())
                .map(|value| value.trim().to_owned())
                .filter(|value| !value.is_empty());

            browsers.push(DetectedBrowser {
                id: definition.id.clone(),
                name: definition.name.to_owned(),
                path: target,
                version,
                supports_app_mode: !matches!(definition.id, BrowserChoice::Firefox),
            });
            detected = true;
            break;
        }

        if detected {
            continue;
        }

        for (app_id, display_name) in definition.flatpak_apps {
            let Some((target, version)) = resolve_flatpak_app(app_id) else {
                continue;
            };
            if !seen.insert(target.clone()) {
                continue;
            }

            browsers.push(DetectedBrowser {
                id: definition.id.clone(),
                name: (*display_name).to_owned(),
                path: target,
                version,
                supports_app_mode: !matches!(definition.id, BrowserChoice::Firefox),
            });
            break;
        }
    }

    browsers
}

pub fn launch_system_browser(webapp: &WebApp, paths: &ManagedPaths) -> Result<()> {
    let browsers = detect_browsers();
    let Some(browser) = select_browser(&browsers, &webapp.browser) else {
        return launch_with_system_default(webapp);
    };

    if let Some(app_id) = browser.path.strip_prefix("flatpak:") {
        return launch_flatpak_browser(app_id, &browser.id, webapp, paths, &browser.name);
    }

    let mut command = Command::new(&browser.path);
    apply_desktop_hints(&mut command, paths, webapp, &browser.id);
    apply_wayland_window_identity_workaround(&mut command, &browser.id);
    apply_window_mode_args(&mut command, &browser.id, &webapp.window_mode);

    if webapp.isolated {
        let profile_dir = webapp_profile_dir(paths, webapp)?;
        match browser.id {
            BrowserChoice::Firefox => {
                command.arg("--profile").arg(profile_dir);
                command.arg("--new-window").arg(&webapp.url);
            }
            _ => {
                command.arg(format!("--class={}", webapp_window_class(webapp)));
                command.arg(format!("--app={}", webapp.url));
                command.arg(format!("--user-data-dir={}", profile_dir.to_string_lossy()));
            }
        }
    } else {
        match browser.id {
            BrowserChoice::Firefox => {
                command.arg("--new-window").arg(&webapp.url);
            }
            _ => {
                command.arg(format!("--class={}", webapp_window_class(webapp)));
                command.arg(format!("--app={}", webapp.url));
            }
        }
    }

    command
        .spawn()
        .with_context(|| format!("Failed to launch {}.", browser.name))?;

    Ok(())
}

fn launch_flatpak_browser(
    app_id: &str,
    browser_id: &BrowserChoice,
    webapp: &WebApp,
    paths: &ManagedPaths,
    browser_name: &str,
) -> Result<()> {
    let mut command = Command::new("flatpak");
    command.arg("run");
    apply_flatpak_desktop_hints(&mut command, paths, webapp, browser_id);
    apply_flatpak_wayland_window_identity_workaround(&mut command, browser_id);
    apply_window_mode_args(&mut command, browser_id, &webapp.window_mode);

    if webapp.isolated {
        let profile_dir = webapp_profile_dir(paths, webapp)?;
        command.arg(format!(
            "--filesystem={}:create",
            profile_dir.to_string_lossy()
        ));
        command.arg(app_id);

        match browser_id {
            BrowserChoice::Firefox => {
                command.arg("--profile").arg(profile_dir);
                command.arg("--new-window").arg(&webapp.url);
            }
            _ => {
                command.arg(format!("--class={}", webapp_window_class(webapp)));
                command.arg(format!("--app={}", webapp.url));
                command.arg(format!("--user-data-dir={}", profile_dir.to_string_lossy()));
            }
        }
    } else {
        command.arg(app_id);
        match browser_id {
            BrowserChoice::Firefox => {
                command.arg("--new-window").arg(&webapp.url);
            }
            _ => {
                command.arg(format!("--class={}", webapp_window_class(webapp)));
                command.arg(format!("--app={}", webapp.url));
            }
        }
    }

    command
        .spawn()
        .with_context(|| format!("Failed to launch {}.", browser_name))?;

    Ok(())
}

fn launch_with_system_default(webapp: &WebApp) -> Result<()> {
    if !matches!(webapp.browser, BrowserChoice::Auto) {
        return Err(anyhow!("No compatible browser is installed."));
    }

    Command::new("xdg-open")
        .arg(&webapp.url)
        .spawn()
        .context("Failed to launch the system default browser.")?;

    Ok(())
}

fn apply_desktop_hints(
    command: &mut Command,
    paths: &ManagedPaths,
    webapp: &WebApp,
    browser_id: &BrowserChoice,
) {
    let desktop_file = entry::desktop_entry_path(paths, &webapp.id);
    command.env("BAMF_DESKTOP_FILE_HINT", desktop_file);
    if is_chromium_family(browser_id) {
        command.env("CHROME_DESKTOP", webapp_desktop_filename(webapp));
    }
}

fn apply_flatpak_desktop_hints(
    command: &mut Command,
    paths: &ManagedPaths,
    webapp: &WebApp,
    browser_id: &BrowserChoice,
) {
    let desktop_file = entry::desktop_entry_path(paths, &webapp.id);
    command.arg(format!(
        "--env=BAMF_DESKTOP_FILE_HINT={}",
        desktop_file.to_string_lossy()
    ));
    if is_chromium_family(browser_id) {
        command.arg(format!(
            "--env=CHROME_DESKTOP={}",
            webapp_desktop_filename(webapp)
        ));
    }
}

fn apply_wayland_window_identity_workaround(command: &mut Command, browser_id: &BrowserChoice) {
    if is_wayland_session() && is_chromium_family(browser_id) {
        // Chromium's custom class/app matching is unreliable on GNOME Wayland for
        // standalone URL apps. Prefer XWayland here so the per-webapp desktop entry
        // can be matched through the requested window class.
        command.arg("--ozone-platform=x11");
    }
}

fn apply_flatpak_wayland_window_identity_workaround(
    command: &mut Command,
    browser_id: &BrowserChoice,
) {
    if is_wayland_session() && is_chromium_family(browser_id) {
        command.arg("--env=OZONE_PLATFORM=x11");
        command.arg("--env=ELECTRON_OZONE_PLATFORM_HINT=x11");
    }
}

fn webapp_window_class(webapp: &WebApp) -> String {
    format!("webapp-{}", webapp.id)
}

fn webapp_desktop_filename(webapp: &WebApp) -> String {
    format!("webapp-{}.desktop", webapp.id)
}

fn is_chromium_family(browser_id: &BrowserChoice) -> bool {
    matches!(
        browser_id,
        BrowserChoice::Chrome
            | BrowserChoice::Chromium
            | BrowserChoice::Brave
            | BrowserChoice::Edge
            | BrowserChoice::Vivaldi
    )
}

fn is_wayland_session() -> bool {
    env::var("XDG_SESSION_TYPE")
        .map(|value| value.eq_ignore_ascii_case("wayland"))
        .unwrap_or(false)
}

fn apply_window_mode_args(
    command: &mut Command,
    browser_id: &BrowserChoice,
    window_mode: &WindowMode,
) {
    match (browser_id, window_mode) {
        (_, WindowMode::Normal) => {}
        (BrowserChoice::Firefox, WindowMode::Fullscreen) => {
            command.arg("--kiosk");
        }
        (id, WindowMode::Maximized) if is_chromium_family(id) => {
            command.arg("--start-maximized");
        }
        (id, WindowMode::Fullscreen) if is_chromium_family(id) => {
            command.arg("--start-fullscreen");
        }
        _ => {}
    }
}

fn select_browser<'a>(
    browsers: &'a [DetectedBrowser],
    preferred: &BrowserChoice,
) -> Option<&'a DetectedBrowser> {
    match preferred {
        BrowserChoice::Auto => browsers.first(),
        choice => browsers
            .iter()
            .find(|browser| &browser.id == choice)
            .or_else(|| browsers.first()),
    }
}

fn resolve_from_path(command: &str) -> Option<PathBuf> {
    env::var_os("PATH").and_then(|paths| {
        env::split_paths(&paths).find_map(|directory| {
            let candidate = directory.join(command);
            is_executable(&candidate).then_some(candidate)
        })
    })
}

fn resolve_flatpak_app(app_id: &str) -> Option<(String, Option<String>)> {
    let output = Command::new("flatpak")
        .args(["info", app_id])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8(output.stdout).ok()?;
    let version = stdout
        .lines()
        .find_map(|line| line.trim().strip_prefix("Version:"))
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty());

    Some((format!("flatpak:{app_id}"), version))
}

fn is_executable(path: &Path) -> bool {
    path.exists() && path.is_file()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn detected(id: BrowserChoice, name: &str) -> DetectedBrowser {
        DetectedBrowser {
            id,
            name: name.to_owned(),
            path: format!("/usr/bin/{}", name.to_lowercase()),
            version: None,
            supports_app_mode: true,
        }
    }

    #[test]
    fn select_browser_uses_first_detected_for_auto() {
        let browsers = vec![
            detected(BrowserChoice::Chrome, "Chrome"),
            detected(BrowserChoice::Firefox, "Firefox"),
        ];

        let selected = select_browser(&browsers, &BrowserChoice::Auto).unwrap();

        assert_eq!(selected.id, BrowserChoice::Chrome);
    }

    #[test]
    fn select_browser_uses_requested_browser_when_available() {
        let browsers = vec![
            detected(BrowserChoice::Chrome, "Chrome"),
            detected(BrowserChoice::Firefox, "Firefox"),
        ];

        let selected = select_browser(&browsers, &BrowserChoice::Firefox).unwrap();

        assert_eq!(selected.id, BrowserChoice::Firefox);
    }

    #[test]
    fn select_browser_falls_back_to_first_detected_browser_when_preferred_is_missing() {
        let browsers = vec![
            detected(BrowserChoice::Chrome, "Chrome"),
            detected(BrowserChoice::Chromium, "Chromium"),
        ];

        let selected = select_browser(&browsers, &BrowserChoice::Firefox).unwrap();

        assert_eq!(selected.id, BrowserChoice::Chrome);
    }

    #[test]
    fn select_browser_returns_none_when_no_browser_is_detected() {
        assert!(select_browser(&[], &BrowserChoice::Auto).is_none());
        assert!(select_browser(&[], &BrowserChoice::Firefox).is_none());
    }
}
