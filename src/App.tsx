import { useEffect, useMemo, useRef, useState } from "react";
import { EmbeddedShell } from "./components/EmbeddedShell";
import {
  AppsIcon,
  BrowserIcon,
  CloseIcon,
  ExternalIcon,
  GlobeIcon,
  LayoutIcon,
  MenuIcon,
  MinimizeIcon,
  MoonIcon,
  PencilIcon,
  PlayIcon,
  PlusIcon,
  SearchIcon,
  SettingsIcon,
  ShieldIcon,
  SquareIcon,
  SunIcon,
  TrayIcon,
  TrashIcon,
} from "./components/Icons";
import { DeleteConfirmModal } from "./components/DeleteConfirmModal";
import { SettingsView } from "./components/SettingsView";
import { WebAppForm } from "./components/WebAppForm";
import { WebAppList } from "./components/WebAppList";
import { useBrowsers } from "./hooks/useBrowsers";
import { useWebApps } from "./hooks/useWebApps";
import {
  closeCurrentWindow,
  errorMessage,
  getSettings,
  minimizeCurrentWindow,
  saveSettings,
  toggleCurrentWindowMaximize,
  type AppSettings,
  toAssetUrl,
  type BrowserChoice,
  type DetectedBrowser,
  type ThemePreference,
  type WebAppPayload,
  type WindowMode,
} from "./lib/tauri";

function hostnameFromUrl(url: string) {
  try {
    const target = new URL(url.includes("://") ? url : `https://${url}`);
    return target.hostname.replace(/^www\./, "");
  } catch {
    return url;
  }
}

function browserLabel(choice: BrowserChoice, browsers: DetectedBrowser[]) {
  if (choice === "auto") {
    return "Automatic";
  }

  if (choice === "embedded") {
    return "Embedded";
  }

  return browsers.find((browser) => browser.id === choice)?.name ?? choice;
}

function windowModeLabel(mode: WindowMode) {
  switch (mode) {
    case "maximized":
      return "Start maximized";
    case "fullscreen":
      return "Start fullscreen";
    default:
      return "Standard window";
  }
}

const defaultAppSettings: AppSettings = {
  httpTimeoutSecs: 10,
  maxIconSizeMb: 5,
  historyLimit: 50,
  defaultBrowser: "auto",
  defaultWindowMode: "normal",
  launchOnLogin: false,
  theme: "light",
};

function applyTheme(theme: ThemePreference) {
  document.documentElement.dataset.theme = theme;
}

function ManagerApp({
  appSettings,
  onSettingsSaved,
}: {
  appSettings: AppSettings;
  onSettingsSaved: (settings: AppSettings) => void;
}) {
  const { webApps, loading, error, refresh: refreshWebApps, create, update, remove, launch } = useWebApps();
  const { browsers, error: browserError, refresh } = useBrowsers();
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [drawerOpen, setDrawerOpen] = useState(false);
  const [editingId, setEditingId] = useState<string | null>(null);
  const [isCreating, setIsCreating] = useState(false);
  const [actionError, setActionError] = useState<string | null>(null);
  const [pendingDelete, setPendingDelete] = useState<string | null>(null);
  const [searchQuery, setSearchQuery] = useState("");
  const deleteButtonRef = useRef<HTMLElement | null>(null);

  const filteredWebApps = useMemo(
    () =>
      searchQuery.trim()
        ? webApps.filter((app) => {
            const q = searchQuery.toLowerCase();
            return (
              app.name.toLowerCase().includes(q) ||
              app.url.toLowerCase().includes(q)
            );
          })
        : webApps,
    [webApps, searchQuery]
  );

  useEffect(() => {
    setSelectedId((current) => {
      if (current && filteredWebApps.some((item) => item.id === current)) {
        return current;
      }
      return filteredWebApps[0]?.id ?? null;
    });
  }, [filteredWebApps]);

  const selectedWebApp = webApps.find((item) => item.id === selectedId) ?? null;
  const editingWebApp = webApps.find((item) => item.id === editingId) ?? null;
  const hasSavedWebApps = webApps.length > 0;
  const isSearching = searchQuery.trim().length > 0;
  const errors = [error, browserError, actionError].filter(Boolean);

  const stats = [
    { label: "Saved apps", value: webApps.length },
    { label: "Embedded", value: webApps.filter((item) => item.browser === "embedded").length },
    { label: "Isolated", value: webApps.filter((item) => item.isolated).length },
    { label: "Tray ready", value: webApps.filter((item) => item.tray).length },
  ];

  async function handleSubmit(payload: WebAppPayload) {
    if (payload.id) {
      const item = await update(payload);
      setEditingId(null);
      setSelectedId(item.id);
    } else {
      const item = await create(payload);
      setSelectedId(item.id);
      setIsCreating(false);
    }
  }

  function handleRemove() {
    if (!selectedWebApp) return;
    deleteButtonRef.current = document.activeElement as HTMLElement;
    setPendingDelete(selectedWebApp.id);
  }

  async function confirmDelete(id: string) {
    setPendingDelete(null);
    try {
      setActionError(null);
      await remove(id);
      setSelectedId(null);
    } catch (err) {
      setActionError(errorMessage(err, "Failed to delete web app."));
    } finally {
      deleteButtonRef.current?.focus();
      deleteButtonRef.current = null;
    }
  }

  async function handleLaunch(id: string) {
    try {
      setActionError(null);
      await launch(id);
    } catch (err) {
      setActionError(errorMessage(err, "Failed to launch web app."));
    }
  }

  async function handleToggleTheme() {
    const nextTheme: ThemePreference = appSettings.theme === "dark" ? "light" : "dark";
    const nextSettings = { ...appSettings, theme: nextTheme };
    setActionError(null);
    onSettingsSaved(nextSettings);
    try {
      await saveSettings(nextSettings);
    } catch (err) {
      onSettingsSaved(appSettings);
      setActionError(errorMessage(err, "Failed to save theme preference."));
    }
  }

  async function handleMinimizeWindow() {
    try {
      setActionError(null);
      await minimizeCurrentWindow();
    } catch (err) {
      setActionError(errorMessage(err, "Failed to minimize the window."));
    }
  }

  async function handleToggleMaximizeWindow() {
    try {
      setActionError(null);
      await toggleCurrentWindowMaximize();
    } catch (err) {
      setActionError(errorMessage(err, "Failed to resize the window."));
    }
  }

  async function handleCloseWindow() {
    try {
      setActionError(null);
      await closeCurrentWindow();
    } catch (err) {
      setActionError(errorMessage(err, "Failed to close the window."));
    }
  }

  function openCreateDialog() {
    setEditingId(null);
    setIsCreating(true);
  }

  function openEditDialog() {
    if (!selectedWebApp) {
      return;
    }

    setIsCreating(false);
    setEditingId(selectedWebApp.id);
  }

  useEffect(() => {
    function handleKeyDown(event: KeyboardEvent) {
      const target = event.target as HTMLElement;
      const inInput = ["INPUT", "TEXTAREA", "SELECT"].includes(target.tagName);

      // Ctrl+F — focus search (allowed even when in input so the browser shortcut is overridden)
      if (event.ctrlKey && event.key === "f") {
        event.preventDefault();
        const searchEl = document.getElementById("search-input");
        searchEl?.focus();
        return;
      }

      if (inInput) return;

      // Alt+N / Delete: skip when a form is already open
      if (!isCreating && !editingId) {
        // Alt+N — open new web app form
        if (event.altKey && event.key === "n") {
          event.preventDefault();
          setEditingId(null);
          setIsCreating(true);
          return;
        }

        // Delete / Backspace — trigger delete modal on selected app
        if (event.key === "Delete" && selectedWebApp) {
          event.preventDefault();
          deleteButtonRef.current = document.activeElement as HTMLElement;
          setPendingDelete(selectedWebApp.id);
          return;
        }
      }

      // Escape — close form or clear selection
      if (event.key === "Escape") {
        if (isCreating) {
          setIsCreating(false);
        } else if (editingId) {
          setEditingId(null);
        } else if (pendingDelete) {
          setPendingDelete(null);
        } else {
          setSelectedId(null);
        }
        return;
      }
    }

    document.addEventListener("keydown", handleKeyDown);
    return () => document.removeEventListener("keydown", handleKeyDown);
  }, [selectedWebApp, isCreating, editingId, pendingDelete]);

  return (
    <main className="app-shell">
      <section className="app-frame">
        <header className="window-titlebar">
          <div
            className="window-titlebar-side window-titlebar-side-start"
            data-tauri-drag-region
            onDoubleClick={() => void handleToggleMaximizeWindow()}
          >
            <div className="window-titlebar-launcher" aria-hidden="true">
              <div className="window-titlebar-mark" aria-hidden="true">
                <AppsIcon />
              </div>
            </div>
          </div>

          <div
            className="window-titlebar-center"
            data-tauri-drag-region
            onDoubleClick={() => void handleToggleMaximizeWindow()}
          >
            <strong>Linux PWA Manager</strong>
          </div>

          <div className="window-titlebar-side window-titlebar-side-end">
            <div
              className="window-titlebar-drag-fill"
              data-tauri-drag-region
              onDoubleClick={() => void handleToggleMaximizeWindow()}
            />
            <div className="window-titlebar-actions" aria-label="Window controls">
              <button
                className="window-control-button"
                type="button"
                aria-label="Minimize window"
                onClick={() => void handleMinimizeWindow()}
              >
                <MinimizeIcon />
              </button>
              <button
                className="window-control-button"
                type="button"
                aria-label="Maximize window"
                onClick={() => void handleToggleMaximizeWindow()}
              >
                <SquareIcon />
              </button>
              <button
                className="window-control-button danger"
                type="button"
                aria-label="Close window"
                onClick={() => void handleCloseWindow()}
              >
                <CloseIcon />
              </button>
            </div>
          </div>
        </header>

        <header className="app-topbar">
          <div className="app-brand">
            <div className="app-brand-mark" aria-hidden="true">
              <AppsIcon />
            </div>
            <div className="app-brand-copy">
              <p className="app-eyebrow">Linux PWA Manager</p>
              <h1>Modern launchers for the sites you use every day</h1>
            </div>
          </div>

          <div className="app-topbar-actions">
            <button
              className="icon-button theme-toggle-button"
              type="button"
              title={appSettings.theme === "dark" ? "Switch to light mode" : "Switch to dark mode"}
              aria-label={appSettings.theme === "dark" ? "Switch to light mode" : "Switch to dark mode"}
              onClick={() => void handleToggleTheme()}
            >
              {appSettings.theme === "dark" ? <SunIcon /> : <MoonIcon />}
            </button>

            <button
              className="ghost-button"
              type="button"
              aria-label={drawerOpen ? "Hide manager details" : "Show manager details"}
              onClick={() => setDrawerOpen((current) => !current)}
            >
              <MenuIcon />
              <span>Manager</span>
            </button>

            <button className="app-primary-button" type="button" onClick={openCreateDialog}>
              <PlusIcon />
              <span>New Web App</span>
            </button>
          </div>
        </header>

        {errors.length > 0 ? (
          <div className="manager-alerts" role="status" aria-live="polite">
            {errors.map((message) => (
              <p className="error-banner" key={message}>
                {message}
              </p>
            ))}
          </div>
        ) : null}

        <section className="workspace-grid">
          <section className="surface-panel library-panel">
            <div className="surface-heading">
              <div>
                <h2>Library</h2>
                <p>Select a launcher or double-click one to open it immediately.</p>
              </div>
            </div>

            <div className="manager-list-frame">
              <div className="manager-search">
                <div className="manager-search-field">
                  <input
                    id="search-input"
                    className="mint-input manager-search-input"
                    type="search"
                    placeholder="Search by name or URL…"
                    value={searchQuery}
                    onChange={(e) => setSearchQuery(e.target.value)}
                    aria-label="Search web apps"
                  />
                  {searchQuery ? (
                    <button
                      className="manager-search-clear"
                      type="button"
                      aria-label="Clear search"
                      onClick={() => setSearchQuery("")}
                    >
                      ×
                    </button>
                  ) : null}
                </div>
              </div>

              {loading ? (
                <div className="empty-state">
                  <h3>Loading web apps</h3>
                  <p>Reading saved launchers from your local library.</p>
                </div>
              ) : filteredWebApps.length === 0 && searchQuery.trim() ? (
                <div className="empty-state">
                  <h3>No results</h3>
                  <p>No web apps match your search. Try a different name or URL.</p>
                </div>
              ) : (
                <WebAppList
                  items={filteredWebApps}
                  selectedId={selectedId}
                  onSelect={setSelectedId}
                  onLaunch={handleLaunch}
                  onCreate={openCreateDialog}
                />
              )}
            </div>
          </section>

          <section className="surface-panel detail-panel" aria-live="polite">
            {selectedWebApp ? (
              <>
                <div className="detail-hero">
                  <div className="selection-icon detail-icon" aria-hidden="true">
                    {(() => {
                      const iconUrl = selectedWebApp.iconDataUrl ?? toAssetUrl(selectedWebApp.iconPath);
                      return iconUrl ? (
                        <img src={iconUrl} alt="" />
                      ) : (
                        <span>{selectedWebApp.name.slice(0, 1).toUpperCase()}</span>
                      );
                    })()}
                  </div>

                  <div className="detail-copy">
                    <p className="app-eyebrow">Selected web app</p>
                    <h2>{selectedWebApp.name}</h2>
                    <a className="detail-link" href={selectedWebApp.url} target="_blank" rel="noreferrer">
                      {selectedWebApp.url}
                    </a>
                  </div>
                </div>

                <div className="detail-actions">
                  <button className="app-primary-button" type="button" onClick={() => void handleLaunch(selectedWebApp.id)}>
                    <PlayIcon />
                    <span>Launch</span>
                  </button>
                  <button className="ghost-button" type="button" onClick={openEditDialog}>
                    <PencilIcon />
                    <span>Edit</span>
                  </button>
                  <button className="ghost-button danger" type="button" onClick={handleRemove}>
                    <TrashIcon />
                    <span>Delete</span>
                  </button>
                </div>

                <div className="detail-card-grid">
                  <article className="detail-card">
                    <span className="detail-card-label">Runtime</span>
                    <strong>{browserLabel(selectedWebApp.browser, browsers)}</strong>
                    <p>{selectedWebApp.browser === "embedded" ? "Built-in window" : "External browser"}</p>
                  </article>

                  <article className="detail-card">
                    <span className="detail-card-label">Window</span>
                    <strong>{windowModeLabel(selectedWebApp.windowMode)}</strong>
                    <p>{selectedWebApp.navBar ? "Navigation bar on" : "Navigation bar off"}</p>
                  </article>

                  <article className="detail-card">
                    <span className="detail-card-label">Profile</span>
                    <strong>{selectedWebApp.isolated ? "Dedicated profile" : "Shared browser session"}</strong>
                    <p>{selectedWebApp.isolated ? "Separate storage" : "Shared storage"}</p>
                  </article>

                  <article className="detail-card">
                    <span className="detail-card-label">Desktop</span>
                    <strong>{selectedWebApp.category}</strong>
                    <p>{selectedWebApp.tray ? "Tray enabled" : "Tray disabled"}</p>
                  </article>
                </div>

                <div className="selection-badges">
                  <span className="selection-badge">
                    <ExternalIcon />
                    {hostnameFromUrl(selectedWebApp.url)}
                  </span>
                  <span className="selection-badge">
                    <GlobeIcon />
                    {selectedWebApp.category}
                  </span>
                  <span className="selection-badge">
                    <BrowserIcon />
                    {browserLabel(selectedWebApp.browser, browsers)}
                  </span>
                  <span className="selection-badge">
                    <LayoutIcon />
                    {windowModeLabel(selectedWebApp.windowMode)}
                  </span>
                  <span className="selection-badge">
                    <ShieldIcon />
                    {selectedWebApp.isolated ? "Dedicated profile" : "Shared session"}
                  </span>
                  <span className="selection-badge">
                    <TrayIcon />
                    {selectedWebApp.tray ? "Tray icon" : "No tray"}
                  </span>
                </div>
              </>
            ) : hasSavedWebApps && isSearching ? (
              <div className="empty-state detail-empty detail-empty-compact">
                <div className="empty-state-icon" aria-hidden="true">
                  <SearchIcon />
                </div>
                <div className="empty-state-copy">
                  <h3>No matching web app selected</h3>
                  <p>Clear your search or pick a different launcher from the library to view its runtime and window settings.</p>
                </div>
                <button className="ghost-button" type="button" onClick={() => setSearchQuery("")}>
                  <SearchIcon />
                  <span>Clear Search</span>
                </button>
              </div>
            ) : (
              <div className="empty-state detail-empty">
                <div className="detail-empty-hero">
                  <div className="detail-empty-visual" aria-hidden="true">
                    <div className="detail-empty-icon">
                      <AppsIcon />
                    </div>
                    <span className="detail-empty-pill">
                      <BrowserIcon />
                      Pick a runtime
                    </span>
                    <span className="detail-empty-pill">
                      <LayoutIcon />
                      Tune the window
                    </span>
                    <span className="detail-empty-pill">
                      <ShieldIcon />
                      Isolate storage
                    </span>
                  </div>

                  <div className="detail-empty-copy">
                    <p className="app-eyebrow">First launch</p>
                    <h3>Create your first web app</h3>
                    <p>Turn any site into a desktop launcher with its own browser runtime, window behavior, and profile isolation.</p>
                  </div>
                </div>

                <div className="detail-empty-grid">
                  <article className="detail-empty-card">
                    <BrowserIcon />
                    <strong>Choose how it runs</strong>
                    <p>Use the built-in window or launch through any detected browser.</p>
                  </article>

                  <article className="detail-empty-card">
                    <LayoutIcon />
                    <strong>Shape the window</strong>
                    <p>Start normal, maximized, or fullscreen and decide whether navigation stays visible.</p>
                  </article>

                  <article className="detail-empty-card">
                    <ShieldIcon />
                    <strong>Control app data</strong>
                    <p>Keep sites in a dedicated profile when you want storage and sessions separated.</p>
                  </article>
                </div>

                <button className="app-primary-button" type="button" onClick={openCreateDialog}>
                  <PlusIcon />
                  <span>Create your first web app</span>
                </button>
              </div>
            )}
          </section>
        </section>
      </section>

      {drawerOpen ? (
        <>
          <button
            className="drawer-backdrop"
            type="button"
            aria-label="Close manager details"
            onClick={() => setDrawerOpen(false)}
          />

          <aside className="manager-drawer" aria-label="Manager details">
            <section className="drawer-section">
              <div className="drawer-section-header">
                <SettingsIcon />
                <h2>Manager overview</h2>
              </div>

              <div className="drawer-stat-grid">
                {stats.map((stat) => (
                  <article key={stat.label}>
                    <strong>{stat.value}</strong>
                    <span>{stat.label}</span>
                  </article>
                ))}
              </div>
            </section>

            <section className="drawer-section">
              <div className="drawer-section-header">
                <BrowserIcon />
                <h2>Detected browsers</h2>
                <button
                  className="ghost-button drawer-refresh-button"
                  type="button"
                  title="Rescan for installed browsers"
                  onClick={refresh}
                >
                  Refresh
                </button>
              </div>

              {browsers.length > 0 ? (
                <ul className="drawer-browser-list">
                  {browsers.map((browser) => (
                    <li key={browser.id}>
                      <div>
                        <strong>{browser.name}</strong>
                        <span>{browser.version ? `Version ${browser.version}` : "Version unknown"}</span>
                      </div>
                      <em>{browser.supportsAppMode ? "App mode" : "Basic"}</em>
                    </li>
                  ))}
                </ul>
              ) : (
                <p className="drawer-empty">No supported external browsers were detected.</p>
              )}
            </section>

            <SettingsView
              currentSettings={appSettings}
              onSettingsSaved={onSettingsSaved}
              onBackupImported={() => void refreshWebApps()}
            />
          </aside>
        </>
      ) : null}

      {isCreating ? (
        <WebAppForm
          browsers={browsers}
          defaults={appSettings}
          onClose={() => setIsCreating(false)}
          onSubmit={handleSubmit}
        />
      ) : null}

      {editingWebApp ? (
        <WebAppForm
          browsers={browsers}
          defaults={appSettings}
          initialValue={editingWebApp}
          onClose={() => setEditingId(null)}
          onSubmit={handleSubmit}
        />
      ) : null}

      {pendingDelete ? (
        <DeleteConfirmModal
          appName={webApps.find((app) => app.id === pendingDelete)?.name ?? "this web app"}
          onConfirm={() => void confirmDelete(pendingDelete)}
          onCancel={() => {
            setPendingDelete(null);
            deleteButtonRef.current?.focus();
            deleteButtonRef.current = null;
          }}
        />
      ) : null}
    </main>
  );
}

export default function App() {
  const mode = new URLSearchParams(window.location.search).get("mode");
  const [appSettings, setAppSettings] = useState<AppSettings>(defaultAppSettings);

  useEffect(() => {
    let cancelled = false;

    async function loadSettings() {
      try {
        const settings = await getSettings();
        if (!cancelled) {
          setAppSettings(settings);
        }
      } catch {
        // Keep built-in defaults if settings cannot be loaded here.
      }
    }

    void loadSettings();
    return () => {
      cancelled = true;
    };
  }, []);

  useEffect(() => {
    applyTheme(appSettings.theme);
  }, [appSettings.theme]);

  if (mode === "embedded") {
    return <EmbeddedShell />;
  }

  return <ManagerApp appSettings={appSettings} onSettingsSaved={setAppSettings} />;
}
