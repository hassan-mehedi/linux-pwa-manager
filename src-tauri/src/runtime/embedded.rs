use std::process::Command;

use anyhow::{anyhow, Context, Result};
use serde::Serialize;
use tauri::webview::{PageLoadEvent, WebviewBuilder};
use tauri::{
    AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize, WebviewUrl, WebviewWindowBuilder,
    WindowEvent,
};
use url::Url;

use crate::models::webapp::{WebApp, WindowMode};
use crate::paths::ManagedPaths;
use crate::runtime::isolation::{shared_embedded_dir, webapp_profile_dir};
use crate::utils::normalize_url;

pub const EMBEDDED_PAGE_STATE_EVENT: &str = "embedded://page-state";
const DEFAULT_NAV_BAR_HEIGHT: f64 = 72.0;
/// Maximum number of navigation history entries per embedded session.
pub const MAX_HISTORY: usize = 50;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EmbeddedPageState {
    pub url: String,
    pub loading: bool,
    pub can_go_back: bool,
    pub can_go_forward: bool,
}

#[derive(Debug, Clone, Default)]
pub struct EmbeddedSessionState {
    history: Vec<String>,
    index: usize,
    pending_index: Option<usize>,
}

impl EmbeddedSessionState {
    fn new(initial_url: &str) -> Self {
        Self {
            history: vec![initial_url.to_owned()],
            index: 0,
            pending_index: None,
        }
    }

    fn can_go_back(&self) -> bool {
        self.index > 0
    }

    fn can_go_forward(&self) -> bool {
        self.index + 1 < self.history.len()
    }

    fn mark_regular_navigation(&mut self) {
        self.pending_index = None;
    }

    fn prepare_go_back(&mut self) -> Result<String> {
        if self.index == 0 {
            return Err(anyhow!("No previous page in embedded history."));
        }

        let target_index = self.index - 1;
        self.pending_index = Some(target_index);
        Ok(self.history[target_index].clone())
    }

    fn prepare_go_forward(&mut self) -> Result<String> {
        if self.index + 1 >= self.history.len() {
            return Err(anyhow!("No forward page in embedded history."));
        }

        let target_index = self.index + 1;
        self.pending_index = Some(target_index);
        Ok(self.history[target_index].clone())
    }

    fn record_finished_navigation(&mut self, url: &str, limit: usize) {
        if let Some(target_index) = self.pending_index.take() {
            if target_index < self.history.len() {
                self.history[target_index] = url.to_owned();
                self.index = target_index;
                return;
            }
        }

        if self.history.is_empty() {
            self.history.push(url.to_owned());
            self.index = 0;
            return;
        }

        if self.history[self.index] == url {
            return;
        }

        self.history.truncate(self.index + 1);
        self.history.push(url.to_owned());
        self.index = self.history.len() - 1;

        // Trim oldest entries if the history exceeds the cap.
        if self.history.len() > limit {
            let excess = self.history.len() - limit;
            self.history.drain(0..excess);
            self.index = self.index.saturating_sub(excess);
        }
    }
}

pub fn launch_embedded(app: &AppHandle, webapp: &WebApp, paths: &ManagedPaths) -> Result<()> {
    ensure_session(app, &webapp.id, &webapp.url)?;

    let shell_label = shell_window_label(&webapp.id);
    if let Some(window) = app.get_webview_window(&shell_label) {
        window.show()?;
        window.unminimize()?;
        apply_window_mode(&window, &webapp.window_mode)?;
        window.set_focus()?;
        return Ok(());
    }

    let data_dir = if webapp.isolated {
        webapp_profile_dir(paths, webapp)?
    } else {
        shared_embedded_dir(paths)?
    };

    let shell_url =
        WebviewUrl::App(format!("index.html?mode=embedded&webapp={}", webapp.id).into());
    let shell = WebviewWindowBuilder::new(app, &shell_label, shell_url)
        .title(&webapp.name)
        .inner_size(1280.0, 800.0)
        .build()?;
    apply_window_mode(&shell, &webapp.window_mode)?;

    let child_label = content_webview_label(&webapp.id);
    let shell_label_for_events = shell_label.clone();
    let app_handle_for_events = app.clone();
    let webapp_id_for_events = webapp.id.clone();
    let content_builder =
        WebviewBuilder::new(&child_label, WebviewUrl::External(webapp.url.parse()?))
            .data_directory(data_dir)
            .on_page_load(move |_, payload| {
                if matches!(payload.event(), PageLoadEvent::Finished) {
                    let _ = record_finished_navigation(
                        &app_handle_for_events,
                        &webapp_id_for_events,
                        payload.url().as_str(),
                    );
                }

                let page_state = page_state(
                    &app_handle_for_events,
                    &webapp_id_for_events,
                    payload.url().to_string(),
                    matches!(payload.event(), PageLoadEvent::Started),
                );

                let _ = app_handle_for_events.emit_to(
                    &shell_label_for_events,
                    EMBEDDED_PAGE_STATE_EVENT,
                    page_state.unwrap_or_else(|_| EmbeddedPageState {
                        url: payload.url().to_string(),
                        loading: matches!(payload.event(), PageLoadEvent::Started),
                        can_go_back: false,
                        can_go_forward: false,
                    }),
                );
            });

    let host_window = app
        .get_window(&shell_label)
        .ok_or_else(|| anyhow!("Embedded shell window was not created."))?;
    let (position, size) = content_bounds(
        &shell,
        if webapp.nav_bar {
            DEFAULT_NAV_BAR_HEIGHT
        } else {
            0.0
        },
    )?;
    host_window.add_child(content_builder, position, size)?;

    let app_handle = app.clone();
    let webapp_id = webapp.id.clone();
    shell.on_window_event(move |event| match event {
        WindowEvent::Resized(_) | WindowEvent::ScaleFactorChanged { .. } => {
            let _ = resize_embedded_content(&app_handle, &webapp_id, DEFAULT_NAV_BAR_HEIGHT);
        }
        WindowEvent::CloseRequested { .. } => {
            if let Some(main) = app_handle.get_webview_window("main") {
                if matches!(main.is_visible(), Ok(false)) {
                    app_handle.exit(0);
                }
            }
        }
        _ => {}
    });

    resize_embedded_content(
        app,
        &webapp.id,
        if webapp.nav_bar {
            DEFAULT_NAV_BAR_HEIGHT
        } else {
            0.0
        },
    )?;
    Ok(())
}

pub fn current_url(app: &AppHandle, target: &str) -> Result<String> {
    let webview = embedded_content(app, target)?;
    Ok(webview.url()?.to_string())
}

pub fn current_state(app: &AppHandle, target: &str) -> Result<EmbeddedPageState> {
    let url = current_url(app, target)?;
    page_state(app, target, url, false)
}

pub fn navigate(app: &AppHandle, target: &str, url: &str) -> Result<String> {
    let normalized = normalize_url(url)?;
    mark_regular_navigation(app, target)?;
    let webview = embedded_content(app, target)?;
    webview.navigate(normalized.clone())?;
    Ok(normalized.to_string())
}

pub fn go_back(app: &AppHandle, target: &str) -> Result<()> {
    let url = {
        let state = app.state::<crate::AppState>();
        let mut sessions = state
            .embedded_sessions
            .lock()
            .map_err(|_| anyhow!("Embedded session state is unavailable."))?;
        let session = sessions
            .get_mut(target)
            .ok_or_else(|| anyhow!("Embedded session is unavailable."))?;
        session.prepare_go_back()?
    };

    embedded_content(app, target)?.navigate(Url::parse(&url)?)?;
    Ok(())
}

pub fn go_forward(app: &AppHandle, target: &str) -> Result<()> {
    let url = {
        let state = app.state::<crate::AppState>();
        let mut sessions = state
            .embedded_sessions
            .lock()
            .map_err(|_| anyhow!("Embedded session state is unavailable."))?;
        let session = sessions
            .get_mut(target)
            .ok_or_else(|| anyhow!("Embedded session is unavailable."))?;
        session.prepare_go_forward()?
    };

    embedded_content(app, target)?.navigate(Url::parse(&url)?)?;
    Ok(())
}

pub fn reload(app: &AppHandle, target: &str) -> Result<()> {
    embedded_content(app, target)?.reload()?;
    Ok(())
}

pub fn resize_embedded_content(app: &AppHandle, target: &str, top_inset: f64) -> Result<()> {
    let shell = app
        .get_webview_window(&shell_window_label(target))
        .ok_or_else(|| anyhow!("Embedded shell is not open."))?;
    let content = embedded_content(app, target)?;
    let (position, size) = content_bounds(&shell, top_inset)?;
    content.set_position(position)?;
    content.set_size(size)?;
    Ok(())
}

pub fn open_in_default_browser(app: &AppHandle, target: &str) -> Result<()> {
    let url = current_url(app, target)?;
    Command::new("xdg-open")
        .arg(url)
        .spawn()
        .context("Failed to open the current page in the default browser.")?;
    Ok(())
}

fn embedded_content(app: &AppHandle, target: &str) -> Result<tauri::Webview> {
    app.get_webview(&content_webview_label(target))
        .ok_or_else(|| anyhow!("Embedded content view is not available."))
}

fn ensure_session(app: &AppHandle, target: &str, initial_url: &str) -> Result<()> {
    let state = app.state::<crate::AppState>();
    let mut sessions = state
        .embedded_sessions
        .lock()
        .map_err(|_| anyhow!("Embedded session state is unavailable."))?;
    sessions
        .entry(target.to_owned())
        .or_insert_with(|| EmbeddedSessionState::new(initial_url));
    Ok(())
}

fn mark_regular_navigation(app: &AppHandle, target: &str) -> Result<()> {
    let state = app.state::<crate::AppState>();
    let mut sessions = state
        .embedded_sessions
        .lock()
        .map_err(|_| anyhow!("Embedded session state is unavailable."))?;
    if let Some(session) = sessions.get_mut(target) {
        session.mark_regular_navigation();
    }
    Ok(())
}

fn record_finished_navigation(app: &AppHandle, target: &str, url: &str) -> Result<()> {
    let state = app.state::<crate::AppState>();
    let limit = state.settings.lock().map(|s| s.history_limit).unwrap_or(50);
    let mut sessions = state
        .embedded_sessions
        .lock()
        .map_err(|_| anyhow!("Embedded session state is unavailable."))?;
    let session = sessions
        .entry(target.to_owned())
        .or_insert_with(|| EmbeddedSessionState::new(url));
    session.record_finished_navigation(url, limit);
    Ok(())
}

fn page_state(
    app: &AppHandle,
    target: &str,
    url: String,
    loading: bool,
) -> Result<EmbeddedPageState> {
    let state = app.state::<crate::AppState>();
    let sessions = state
        .embedded_sessions
        .lock()
        .map_err(|_| anyhow!("Embedded session state is unavailable."))?;
    Ok(page_state_from_session(sessions.get(target), url, loading))
}

fn content_bounds(
    shell: &tauri::WebviewWindow,
    top_inset_logical: f64,
) -> Result<(PhysicalPosition<i32>, PhysicalSize<u32>)> {
    let size = shell.inner_size()?;
    let scale_factor = shell.scale_factor()?;
    let top_inset = (top_inset_logical * scale_factor).round().max(0.0) as u32;
    let content_height = size.height.saturating_sub(top_inset).max(1);
    Ok((
        PhysicalPosition::new(0, top_inset as i32),
        PhysicalSize::new(size.width.max(1), content_height),
    ))
}

pub fn shell_window_label(id: &str) -> String {
    format!("embedded-shell-{id}")
}

pub fn content_webview_label(id: &str) -> String {
    format!("embedded-content-{id}")
}

fn page_state_from_session(
    session: Option<&EmbeddedSessionState>,
    url: String,
    loading: bool,
) -> EmbeddedPageState {
    EmbeddedPageState {
        url,
        loading,
        can_go_back: session
            .map(EmbeddedSessionState::can_go_back)
            .unwrap_or(false),
        can_go_forward: session
            .map(EmbeddedSessionState::can_go_forward)
            .unwrap_or(false),
    }
}

fn apply_window_mode(window: &tauri::WebviewWindow, window_mode: &WindowMode) -> Result<()> {
    match window_mode {
        WindowMode::Normal => {
            window.set_fullscreen(false)?;
            window.unmaximize()?;
        }
        WindowMode::Maximized => {
            window.set_fullscreen(false)?;
            window.maximize()?;
        }
        WindowMode::Fullscreen => {
            window.unmaximize()?;
            window.set_fullscreen(true)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn regular_navigation_appends_history_and_updates_flags() {
        let mut session = EmbeddedSessionState::new("https://example.com");

        session.record_finished_navigation("https://example.com/inbox", 50);
        session.record_finished_navigation("https://example.com/settings", 50);

        assert_eq!(
            session.history,
            vec![
                "https://example.com".to_owned(),
                "https://example.com/inbox".to_owned(),
                "https://example.com/settings".to_owned()
            ]
        );
        assert_eq!(session.index, 2);
        assert!(session.can_go_back());
        assert!(!session.can_go_forward());
    }

    #[test]
    fn back_navigation_moves_index_without_growing_history() {
        let mut session = EmbeddedSessionState::new("https://example.com");
        session.record_finished_navigation("https://example.com/inbox", 50);
        session.record_finished_navigation("https://example.com/settings", 50);

        let url = session.prepare_go_back().unwrap();
        session.record_finished_navigation(&url, 50);

        assert_eq!(url, "https://example.com/inbox");
        assert_eq!(
            session.history,
            vec![
                "https://example.com".to_owned(),
                "https://example.com/inbox".to_owned(),
                "https://example.com/settings".to_owned()
            ]
        );
        assert_eq!(session.index, 1);
        assert!(session.can_go_back());
        assert!(session.can_go_forward());
    }

    #[test]
    fn redirected_back_navigation_replaces_target_history_entry() {
        let mut session = EmbeddedSessionState::new("https://example.com");
        session.record_finished_navigation("https://example.com/inbox", 50);
        session.record_finished_navigation("https://example.com/settings", 50);

        session.prepare_go_back().unwrap();
        session.record_finished_navigation("https://example.com/login", 50);

        assert_eq!(
            session.history,
            vec![
                "https://example.com".to_owned(),
                "https://example.com/login".to_owned(),
                "https://example.com/settings".to_owned()
            ]
        );
        assert_eq!(session.index, 1);
        assert!(session.can_go_forward());
    }

    #[test]
    fn regular_navigation_after_back_truncates_forward_history() {
        let mut session = EmbeddedSessionState::new("https://example.com");
        session.record_finished_navigation("https://example.com/inbox", 50);
        session.record_finished_navigation("https://example.com/settings", 50);

        let _ = session.prepare_go_back().unwrap();
        session.record_finished_navigation("https://example.com/inbox", 50);
        session.mark_regular_navigation();
        session.record_finished_navigation("https://example.com/profile", 50);

        assert_eq!(
            session.history,
            vec![
                "https://example.com".to_owned(),
                "https://example.com/inbox".to_owned(),
                "https://example.com/profile".to_owned()
            ]
        );
        assert_eq!(session.index, 2);
        assert!(!session.can_go_forward());
    }

    #[test]
    fn page_state_defaults_to_no_history_navigation_when_session_is_missing() {
        let state = page_state_from_session(None, "https://example.com".to_owned(), true);

        assert_eq!(state.url, "https://example.com");
        assert!(state.loading);
        assert!(!state.can_go_back);
        assert!(!state.can_go_forward);
    }

    #[test]
    fn history_is_capped_at_max_entries() {
        let mut session = EmbeddedSessionState::new("https://example.com/0");

        // Navigate to 60 different pages (initial + 59 more = 60 total entries without cap)
        for i in 1..60 {
            session.record_finished_navigation(&format!("https://example.com/{i}"), 50);
        }

        assert!(
            session.history.len() <= MAX_HISTORY,
            "History grew to {} entries, expected at most {MAX_HISTORY}",
            session.history.len()
        );
        // The most recent URL must always be present
        assert_eq!(session.history.last().unwrap(), "https://example.com/59");
    }
}
