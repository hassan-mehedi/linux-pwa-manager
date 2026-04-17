use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};

use crate::models::webapp::WebApp;
use crate::paths::ManagedPaths;

pub fn write_desktop_entry(paths: &ManagedPaths, webapp: &WebApp) -> Result<()> {
    let executable = launcher_executable_path()?;
    let entry_path = desktop_entry_path(paths, &webapp.id);
    let content = desktop_entry_content(webapp, &executable);

    fs::write(&entry_path, content)
        .with_context(|| format!("Failed to write desktop entry {}.", entry_path.display()))?;
    refresh_desktop_database(paths);
    Ok(())
}

pub fn sync_manager_desktop_entry(paths: &ManagedPaths) -> Result<()> {
    let executable = launcher_executable_path()?;
    let entry_path = manager_desktop_entry_path(paths);
    let content = manager_desktop_entry_content(&executable);

    fs::write(&entry_path, content)
        .with_context(|| format!("Failed to write desktop entry {}.", entry_path.display()))?;
    refresh_desktop_database(paths);
    Ok(())
}

pub fn remove_desktop_entry(paths: &ManagedPaths, webapp_id: &str) -> Result<()> {
    let entry_path = desktop_entry_path(paths, webapp_id);
    if entry_path.exists() {
        fs::remove_file(entry_path)?;
    }
    refresh_desktop_database(paths);
    Ok(())
}

pub fn sync_autostart_entry(paths: &ManagedPaths, enabled: bool) -> Result<()> {
    let entry_path = autostart_entry_path(paths);

    if !enabled {
        if entry_path.exists() {
            fs::remove_file(&entry_path).with_context(|| {
                format!("Failed to remove autostart entry {}.", entry_path.display())
            })?;
        }
        return Ok(());
    }

    if let Some(parent) = entry_path.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!("Failed to create autostart directory {}.", parent.display())
        })?;
    }

    let executable = launcher_executable_path()?;
    let content = autostart_entry_content(&executable);
    fs::write(&entry_path, content)
        .with_context(|| format!("Failed to write autostart entry {}.", entry_path.display()))?;
    Ok(())
}

fn refresh_desktop_database(paths: &ManagedPaths) {
    let _ = Command::new("update-desktop-database")
        .arg(&paths.desktop_dir)
        .status();
}

pub fn desktop_entry_path(paths: &ManagedPaths, webapp_id: &str) -> std::path::PathBuf {
    paths
        .desktop_dir
        .join(format!("webapp-{webapp_id}.desktop"))
}

pub fn manager_desktop_entry_path(paths: &ManagedPaths) -> PathBuf {
    paths.desktop_dir.join("webapp-manager.desktop")
}

pub fn autostart_entry_path(paths: &ManagedPaths) -> PathBuf {
    paths.config_dir.join("autostart/webapp-manager.desktop")
}

/// Truncate a value at the first ASCII control character (newline, carriage
/// return, NUL, etc.) to prevent line injection into .desktop file fields.
/// Anything at or after the first control character is discarded so that an
/// attacker cannot append extra key=value lines to the generated file.
fn sanitize_entry_field(value: &str) -> String {
    match value.find(|c: char| c.is_ascii_control()) {
        Some(pos) => value[..pos].to_owned(),
        None => value.to_owned(),
    }
}

fn launcher_executable_path() -> Result<PathBuf> {
    if let Ok(appimage) = std::env::var("APPIMAGE") {
        let path = PathBuf::from(appimage);
        if path.exists() {
            return Ok(path);
        }
    }

    std::env::current_exe().context("Failed to locate current executable.")
}

fn escape_exec_arg(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len() + 2);
    escaped.push('"');
    for ch in value.chars() {
        match ch {
            '"' | '\\' => {
                escaped.push('\\');
                escaped.push(ch);
            }
            '%' => escaped.push_str("%%"),
            _ => escaped.push(ch),
        }
    }
    escaped.push('"');
    escaped
}

fn desktop_entry_content(webapp: &WebApp, executable: &Path) -> String {
    // Note: sanitize_entry_field prevents newline injection. The Exec field additionally
    // interprets '%'-prefixed field codes (e.g. %u, %f) per the FreeDesktop spec, so the
    // executable and id must be escaped for Exec parsing rather than copied verbatim.
    let icon = webapp.icon_path.clone().unwrap_or_default();
    let exec = escape_exec_arg(&sanitize_entry_field(&executable.to_string_lossy()));
    let exec_id = escape_exec_arg(&sanitize_entry_field(&webapp.id));
    let startup_id = sanitize_entry_field(&webapp.id);
    format!(
        "[Desktop Entry]\nVersion=1.1\nType=Application\nName={name}\nComment=Web App: {url}\nExec={exec} launch {exec_id}\nIcon={icon}\nCategories={category};\nStartupWMClass=webapp-{id}\nStartupNotify=true\nX-WebApp-Manager=true\n",
        name = sanitize_entry_field(&webapp.name),
        url = sanitize_entry_field(&webapp.url),
        exec = exec,
        id = startup_id,
        exec_id = exec_id,
        icon = sanitize_entry_field(&icon),
        category = sanitize_entry_field(&webapp.category)
    )
}

fn manager_desktop_entry_content(executable: &Path) -> String {
    let exec = escape_exec_arg(&sanitize_entry_field(&executable.to_string_lossy()));
    let icon = manager_icon_path()
        .map(|path| sanitize_entry_field(&path.to_string_lossy()))
        .unwrap_or_else(|| "webapp-manager".to_owned());
    format!(
        "[Desktop Entry]\nVersion=1.1\nType=Application\nName=Web Apps\nComment=Desktop manager for saved web apps\nExec={exec}\nIcon={icon}\nCategories=Utility;\nStartupWMClass=webapp-manager\nStartupNotify=true\n",
        exec = exec,
        icon = icon,
    )
}

fn autostart_entry_content(executable: &Path) -> String {
    let exec = escape_exec_arg(&sanitize_entry_field(&executable.to_string_lossy()));
    format!(
        "[Desktop Entry]\nVersion=1.1\nType=Application\nName=Web Apps\nExec={exec}\nTerminal=false\nX-GNOME-Autostart-enabled=true\n",
        exec = exec
    )
}

fn manager_icon_path() -> Option<PathBuf> {
    let source_icon = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("icons/icon.png");
    source_icon.exists().then_some(source_icon)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    use super::*;
    use crate::models::webapp::{BrowserChoice, WebApp, WindowMode};
    use crate::paths::ManagedPaths;

    fn test_webapp() -> WebApp {
        WebApp {
            id: "mail".to_owned(),
            name: "Mail".to_owned(),
            url: "https://mail.example.com".to_owned(),
            icon_path: Some("/tmp/icons/mail.png".to_owned()),
            icon_data_url: None,
            category: "Network".to_owned(),
            browser: BrowserChoice::Chrome,
            nav_bar: false,
            isolated: true,
            tray: false,
            window_mode: WindowMode::Normal,
            created_at: "2025-01-01T00:00:00Z".to_owned(),
            updated_at: "2025-01-01T00:00:00Z".to_owned(),
        }
    }

    fn temp_paths() -> ManagedPaths {
        let root =
            std::env::temp_dir().join(format!("webapp-manager-entry-{}", uuid::Uuid::new_v4()));
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
    fn desktop_entry_content_includes_expected_metadata() {
        let webapp = test_webapp();

        let content = desktop_entry_content(&webapp, Path::new("/opt/webapp-manager"));

        assert!(content.contains("Name=Mail"));
        assert!(content.contains("Comment=Web App: https://mail.example.com"));
        assert!(content.contains("Exec=\"/opt/webapp-manager\" launch \"mail\""));
        assert!(content.contains("Icon=/tmp/icons/mail.png"));
        assert!(content.contains("Categories=Network;"));
        assert!(content.contains("StartupWMClass=webapp-mail"));
        assert!(content.contains("X-WebApp-Manager=true"));
    }

    #[test]
    fn manager_desktop_entry_content_includes_expected_metadata() {
        let content = manager_desktop_entry_content(Path::new("/opt/webapp-manager"));

        assert!(content.contains("Name=Web Apps"));
        assert!(content.contains("Exec=\"/opt/webapp-manager\""));
        assert!(content.contains("StartupWMClass=webapp-manager"));
    }

    #[test]
    fn desktop_entry_quotes_exec_arguments() {
        let webapp = test_webapp();

        let content = desktop_entry_content(&webapp, Path::new("/opt/Web Apps/webapp-manager"));

        assert!(content.contains("Exec=\"/opt/Web Apps/webapp-manager\" launch \"mail\""));
    }

    #[test]
    fn desktop_entry_sanitizes_control_characters_in_name() {
        let mut webapp = test_webapp();
        webapp.name = "Mail\nIcon=/etc/passwd".to_owned();

        let content = desktop_entry_content(&webapp, Path::new("/opt/webapp-manager"));

        // The injected newline and everything after it must not appear
        assert!(!content.contains("Icon=/etc/passwd"));
        assert!(content.contains("Name=Mail")); // prefix before \n is retained
    }

    #[test]
    fn desktop_entry_sanitizes_carriage_return_in_category() {
        let mut webapp = test_webapp();
        webapp.category = "Network\rHacked".to_owned();

        let content = desktop_entry_content(&webapp, Path::new("/opt/webapp-manager"));

        assert!(!content.contains("Hacked"));
        assert!(content.contains("Categories=Network;")); // prefix before \r is retained
    }

    #[test]
    fn remove_desktop_entry_deletes_existing_file() {
        let paths = temp_paths();
        let entry_path = desktop_entry_path(&paths, "mail");
        fs::write(&entry_path, "placeholder").unwrap();

        remove_desktop_entry(&paths, "mail").unwrap();

        assert!(!entry_path.exists());
        fs::remove_dir_all(paths.data_dir.parent().unwrap()).unwrap();
    }

    #[test]
    fn sync_autostart_entry_writes_and_removes_file() {
        let paths = temp_paths();
        let entry_path = autostart_entry_path(&paths);

        sync_autostart_entry(&paths, true).unwrap();
        let content = fs::read_to_string(&entry_path).unwrap();
        assert!(content.contains("Name=Web Apps"));

        sync_autostart_entry(&paths, false).unwrap();
        assert!(!entry_path.exists());

        fs::remove_dir_all(paths.data_dir.parent().unwrap()).unwrap();
    }
}
