use serde::{Deserialize, Serialize};

use crate::models::webapp::{BrowserChoice, WindowMode};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum ThemePreference {
    #[default]
    Light,
    Dark,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct AppSettings {
    /// HTTP request timeout for icon fetching, in seconds.
    pub http_timeout_secs: u64,
    /// Maximum icon download size, in mebibytes.
    pub max_icon_size_mb: u64,
    /// Maximum navigation history entries per embedded session.
    pub history_limit: usize,
    /// Default browser for new web apps.
    pub default_browser: BrowserChoice,
    /// Default window mode for new web apps.
    pub default_window_mode: WindowMode,
    /// Launch the manager in the background when the user logs in.
    pub launch_on_login: bool,
    /// Preferred color theme for the manager UI.
    pub theme: ThemePreference,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            http_timeout_secs: 10,
            max_icon_size_mb: 5,
            history_limit: 50,
            default_browser: BrowserChoice::Auto,
            default_window_mode: WindowMode::Normal,
            launch_on_login: false,
            theme: ThemePreference::Light,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_settings_have_expected_values() {
        let s = AppSettings::default();
        assert_eq!(s.http_timeout_secs, 10);
        assert_eq!(s.max_icon_size_mb, 5);
        assert_eq!(s.history_limit, 50);
        assert_eq!(s.default_browser, BrowserChoice::Auto);
        assert_eq!(s.default_window_mode, WindowMode::Normal);
        assert!(!s.launch_on_login);
        assert_eq!(s.theme, ThemePreference::Light);
    }

    #[test]
    fn settings_round_trips_through_toml() {
        let original = AppSettings {
            http_timeout_secs: 30,
            max_icon_size_mb: 10,
            history_limit: 100,
            default_browser: BrowserChoice::Firefox,
            default_window_mode: WindowMode::Maximized,
            launch_on_login: true,
            theme: ThemePreference::Dark,
        };
        let serialized = toml::to_string(&original).unwrap();
        let deserialized: AppSettings = toml::from_str(&serialized).unwrap();
        assert_eq!(deserialized.http_timeout_secs, 30);
        assert_eq!(deserialized.max_icon_size_mb, 10);
        assert_eq!(deserialized.history_limit, 100);
        assert_eq!(deserialized.default_browser, BrowserChoice::Firefox);
        assert_eq!(deserialized.default_window_mode, WindowMode::Maximized);
        assert!(deserialized.launch_on_login);
        assert_eq!(deserialized.theme, ThemePreference::Dark);
    }

    #[test]
    fn missing_theme_defaults_to_light_when_loading_legacy_settings() {
        let legacy = r#"
http_timeout_secs = 20
max_icon_size_mb = 8
history_limit = 120
default_browser = "chrome"
default_window_mode = "fullscreen"
launch_on_login = true
"#;

        let deserialized: AppSettings = toml::from_str(legacy).unwrap();
        assert_eq!(deserialized.theme, ThemePreference::Light);
    }
}
