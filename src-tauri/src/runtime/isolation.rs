use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::models::webapp::WebApp;
use crate::paths::ManagedPaths;

pub fn webapp_profile_dir(paths: &ManagedPaths, webapp: &WebApp) -> Result<PathBuf> {
    let directory = paths.profiles_dir.join(&webapp.id);
    fs::create_dir_all(&directory).with_context(|| {
        format!(
            "Failed to create profile directory {}.",
            directory.display()
        )
    })?;
    #[cfg(unix)]
    restrict_to_owner(&directory)?;
    Ok(directory)
}

pub fn shared_embedded_dir(paths: &ManagedPaths) -> Result<PathBuf> {
    let directory = paths.data_dir.join("embedded-shared");
    fs::create_dir_all(&directory).with_context(|| {
        format!(
            "Failed to create shared embedded directory {}.",
            directory.display()
        )
    })?;
    #[cfg(unix)]
    restrict_to_owner(&directory)?;
    Ok(directory)
}

/// Set directory permissions to 0700 (owner read/write/execute only).
/// This prevents other local users from accessing browser profile data.
#[cfg(unix)]
fn restrict_to_owner(directory: &std::path::Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let perms = fs::Permissions::from_mode(0o700);
    fs::set_permissions(directory, perms)
        .with_context(|| format!("Failed to restrict permissions on {}.", directory.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::webapp::{BrowserChoice, WebApp, WindowMode};
    use std::os::unix::fs::PermissionsExt;

    fn dummy_webapp(id: &str) -> WebApp {
        WebApp {
            id: id.to_owned(),
            name: "Test".to_owned(),
            url: "https://example.com".to_owned(),
            icon_path: None,
            icon_data_url: None,
            category: "Other".to_owned(),
            browser: BrowserChoice::Auto,
            nav_bar: false,
            isolated: true,
            tray: false,
            window_mode: WindowMode::Normal,
            created_at: String::new(),
            updated_at: String::new(),
        }
    }

    fn temp_paths() -> crate::paths::ManagedPaths {
        let root = std::env::temp_dir().join(format!("webapp-isolation-{}", uuid::Uuid::new_v4()));
        let data_dir = root.join("data");
        let profiles_dir = data_dir.join("profiles");
        std::fs::create_dir_all(&profiles_dir).unwrap();
        crate::paths::ManagedPaths {
            data_dir: data_dir.clone(),
            config_dir: root.join("config"),
            state_dir: root.join("state"),
            db_path: data_dir.join("webapps.db"),
            icons_dir: data_dir.join("icons"),
            profiles_dir,
            desktop_dir: data_dir.join("applications"),
        }
    }

    #[test]
    fn webapp_profile_dir_has_restricted_permissions() {
        let paths = temp_paths();
        let webapp = dummy_webapp("test-app");

        let dir = webapp_profile_dir(&paths, &webapp).unwrap();

        let mode = std::fs::metadata(&dir).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o700, "Profile dir must be owner-only (0700)");

        std::fs::remove_dir_all(paths.data_dir.parent().unwrap()).unwrap();
    }
}
