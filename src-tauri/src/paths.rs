use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::models::webapp::AppInfo;

pub(crate) const APP_DIR_NAME: &str = "linux-pwa-manager";
pub(crate) const LEGACY_APP_DIR_NAME: &str = "webapp-manager";

#[derive(Debug, Clone)]
pub struct ManagedPaths {
    pub data_dir: PathBuf,
    pub config_dir: PathBuf,
    pub state_dir: PathBuf,
    pub db_path: PathBuf,
    pub icons_dir: PathBuf,
    pub profiles_dir: PathBuf,
    pub desktop_dir: PathBuf,
}

impl ManagedPaths {
    pub fn discover() -> Result<Self> {
        let data_root = dirs::data_local_dir()
            .or_else(|| dirs::home_dir().map(|home| home.join(".local/share")))
            .context("Failed to resolve XDG data directory.")?;
        let config_root = dirs::config_dir()
            .or_else(|| dirs::home_dir().map(|home| home.join(".config")))
            .context("Failed to resolve XDG config directory.")?;
        let state_root = dirs::state_dir()
            .or_else(|| dirs::home_dir().map(|home| home.join(".local/state")))
            .context("Failed to resolve XDG state directory.")?;

        Self::from_roots(&data_root, &config_root, &state_root)
    }

    fn from_roots(data_root: &Path, config_root: &Path, state_root: &Path) -> Result<Self> {
        let paths = Self::for_slug(data_root, config_root, state_root, APP_DIR_NAME);
        let legacy_paths = Self::for_slug(data_root, config_root, state_root, LEGACY_APP_DIR_NAME);

        migrate_legacy_dir(&legacy_paths.data_dir, &paths.data_dir)?;
        migrate_legacy_dir(&legacy_paths.config_dir, &paths.config_dir)?;
        migrate_legacy_dir(&legacy_paths.state_dir, &paths.state_dir)?;

        for directory in [
            &paths.data_dir,
            &paths.config_dir,
            &paths.state_dir,
            &paths.icons_dir,
            &paths.profiles_dir,
            &paths.desktop_dir,
        ] {
            fs::create_dir_all(directory)
                .with_context(|| format!("Failed to create directory {}.", directory.display()))?;
        }

        Ok(paths)
    }

    fn for_slug(data_root: &Path, config_root: &Path, state_root: &Path, slug: &str) -> Self {
        let data_dir = data_root.join(slug);
        let config_dir = config_root.join(slug);
        let state_dir = state_root.join(slug);

        Self {
            data_dir: data_dir.clone(),
            config_dir,
            state_dir,
            db_path: data_dir.join("webapps.db"),
            icons_dir: data_dir.join("icons"),
            profiles_dir: data_dir.join("profiles"),
            desktop_dir: data_root.join("applications"),
        }
    }

    pub fn app_info(&self) -> AppInfo {
        AppInfo {
            data_dir: display_path(&self.data_dir),
            config_dir: display_path(&self.config_dir),
            state_dir: display_path(&self.state_dir),
            db_path: display_path(&self.db_path),
            desktop_dir: display_path(&self.desktop_dir),
            profiles_dir: display_path(&self.profiles_dir),
            icons_dir: display_path(&self.icons_dir),
        }
    }
}

pub fn display_path(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn migrate_legacy_dir(legacy_path: &Path, target_path: &Path) -> Result<()> {
    if target_path.exists() || !legacy_path.exists() {
        return Ok(());
    }

    if let Some(parent) = target_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory {}.", parent.display()))?;
    }

    fs::rename(legacy_path, target_path).with_context(|| {
        format!(
            "Failed to migrate {} to {}.",
            legacy_path.display(),
            target_path.display()
        )
    })
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn migrates_legacy_managed_directories_to_new_slug() {
        let root = std::env::temp_dir().join(format!(
            "linux-pwa-manager-paths-{}",
            uuid::Uuid::new_v4()
        ));
        let data_root = root.join("data-root");
        let config_root = root.join("config-root");
        let state_root = root.join("state-root");

        let legacy_data = data_root.join(LEGACY_APP_DIR_NAME);
        let legacy_config = config_root.join(LEGACY_APP_DIR_NAME);
        let legacy_state = state_root.join(LEGACY_APP_DIR_NAME);

        fs::create_dir_all(legacy_data.join("icons")).unwrap();
        fs::create_dir_all(&legacy_config).unwrap();
        fs::create_dir_all(&legacy_state).unwrap();
        fs::write(legacy_data.join("webapps.db"), "db").unwrap();
        fs::write(legacy_config.join("settings.json"), "{}").unwrap();
        fs::write(legacy_state.join("runtime.json"), "{}").unwrap();

        let paths = ManagedPaths::from_roots(&data_root, &config_root, &state_root).unwrap();

        assert_eq!(paths.data_dir, data_root.join(APP_DIR_NAME));
        assert_eq!(paths.config_dir, config_root.join(APP_DIR_NAME));
        assert_eq!(paths.state_dir, state_root.join(APP_DIR_NAME));
        assert!(paths.db_path.exists());
        assert!(paths.config_dir.join("settings.json").exists());
        assert!(paths.state_dir.join("runtime.json").exists());
        assert!(!legacy_data.exists());
        assert!(!legacy_config.exists());
        assert!(!legacy_state.exists());

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn keeps_new_directories_when_they_already_exist() {
        let root = std::env::temp_dir().join(format!(
            "linux-pwa-manager-paths-{}",
            uuid::Uuid::new_v4()
        ));
        let data_root = root.join("data-root");
        let config_root = root.join("config-root");
        let state_root = root.join("state-root");

        let new_data = data_root.join(APP_DIR_NAME);
        let legacy_data = data_root.join(LEGACY_APP_DIR_NAME);
        fs::create_dir_all(&new_data).unwrap();
        fs::create_dir_all(&legacy_data).unwrap();
        fs::write(new_data.join("marker.txt"), "new").unwrap();
        fs::write(legacy_data.join("legacy.txt"), "old").unwrap();

        let paths = ManagedPaths::from_roots(&data_root, &config_root, &state_root).unwrap();

        assert!(paths.data_dir.join("marker.txt").exists());
        assert!(legacy_data.join("legacy.txt").exists());

        fs::remove_dir_all(root).unwrap();
    }
}
