use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::models::webapp::AppInfo;

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

        let data_dir = data_root.join("webapp-manager");
        let config_dir = config_root.join("webapp-manager");
        let state_dir = state_root.join("webapp-manager");
        let icons_dir = data_dir.join("icons");
        let profiles_dir = data_dir.join("profiles");
        let db_path = data_dir.join("webapps.db");
        let desktop_dir = data_root.join("applications");

        for directory in [
            &data_dir,
            &config_dir,
            &state_dir,
            &icons_dir,
            &profiles_dir,
            &desktop_dir,
        ] {
            fs::create_dir_all(directory)
                .with_context(|| format!("Failed to create directory {}.", directory.display()))?;
        }

        Ok(Self {
            data_dir,
            config_dir,
            state_dir,
            db_path,
            icons_dir,
            profiles_dir,
            desktop_dir,
        })
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
