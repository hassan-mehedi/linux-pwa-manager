use clap::ValueEnum;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ValueEnum, Default)]
#[serde(rename_all = "snake_case")]
pub enum BrowserChoice {
    #[default]
    Auto,
    Firefox,
    Chromium,
    Chrome,
    Brave,
    Edge,
    Vivaldi,
    Embedded,
}

impl BrowserChoice {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Firefox => "firefox",
            Self::Chromium => "chromium",
            Self::Chrome => "chrome",
            Self::Brave => "brave",
            Self::Edge => "edge",
            Self::Vivaldi => "vivaldi",
            Self::Embedded => "embedded",
        }
    }

    pub fn from_db(value: &str) -> Self {
        match value {
            "firefox" => Self::Firefox,
            "chromium" => Self::Chromium,
            "chrome" => Self::Chrome,
            "brave" => Self::Brave,
            "edge" => Self::Edge,
            "vivaldi" => Self::Vivaldi,
            "embedded" => Self::Embedded,
            _ => Self::Auto,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ValueEnum, Default)]
#[serde(rename_all = "snake_case")]
pub enum WindowMode {
    #[default]
    Normal,
    Maximized,
    Fullscreen,
}

impl WindowMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Maximized => "maximized",
            Self::Fullscreen => "fullscreen",
        }
    }

    pub fn from_db(value: &str) -> Self {
        match value {
            "maximized" => Self::Maximized,
            "fullscreen" => Self::Fullscreen,
            _ => Self::Normal,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebApp {
    pub id: String,
    pub name: String,
    pub url: String,
    pub icon_path: Option<String>,
    pub icon_data_url: Option<String>,
    pub category: String,
    pub browser: BrowserChoice,
    pub nav_bar: bool,
    pub isolated: bool,
    pub tray: bool,
    pub window_mode: WindowMode,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebAppPayload {
    pub id: Option<String>,
    pub name: String,
    pub url: String,
    pub category: String,
    pub browser: BrowserChoice,
    pub nav_bar: bool,
    pub isolated: bool,
    pub tray: bool,
    pub window_mode: WindowMode,
    pub uploaded_icon_data_url: Option<String>,
    pub icon_source_url: Option<String>,
    pub clear_icon: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    pub data_dir: String,
    pub config_dir: String,
    pub state_dir: String,
    pub db_path: String,
    pub desktop_dir: String,
    pub profiles_dir: String,
    pub icons_dir: String,
}
