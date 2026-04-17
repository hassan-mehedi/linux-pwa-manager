import { useEffect, useMemo, useRef, useState } from "react";
import { EmbeddedShell } from "./components/EmbeddedShell";
import {
  AppsIcon,
  BrowserIcon,
  ExternalIcon,
  GlobeIcon,
  LayoutIcon,
  MenuIcon,
  PencilIcon,
  PlayIcon,
  PlusIcon,
  SettingsIcon,
  ShieldIcon,
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
  errorMessage,
  getSettings,
  type AppSettings,
  toAssetUrl,
  type BrowserChoice,
  type DetectedBrowser,
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
};

export default function App() {
  const mode = new URLSearchParams(window.location.search).get("mode");
  if (mode === "embedded") {
    return <EmbeddedShell />;
  }

  const { webApps, loading, error, refresh: refreshWebApps, create, update, remove, launch } = useWebApps();
  const { browsers, error: browserError, refresh } = useBrowsers();
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [drawerOpen, setDrawerOpen] = useState(false);
  const [editingId, setEditingId] = useState<string | null>(null);
  const [isCreating, setIsCreating] = useState(false);
  const [actionError, setActionError] = useState<string | null>(null);
  const [pendingDelete, setPendingDelete] = useState<string | null>(null);
  const [searchQuery, setSearchQuery] = useState("");
  const [appSettings, setAppSettings] = useState<AppSettings>(defaultAppSettings);
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
    setSelectedId((current) => {
      if (current && filteredWebApps.some((item) => item.id === current)) {
        return current;
      }
      return filteredWebApps[0]?.id ?? null;
    });
  }, [filteredWebApps]);

  const selectedWebApp = webApps.find((item) => item.id === selectedId) ?? null;
  const editingWebApp = webApps.find((item) => item.id === editingId) ?? null;
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
        <header className="app-topbar">
          <div className="app-brand">
            <div className="app-brand-mark" aria-hidden="true">
              <AppsIcon />
            </div>
            <div className="app-brand-copy">
              <p className="app-eyebrow">Web App Manager</p>
              <h1>Modern launchers for the sites you use every day</h1>
            </div>
          </div>

          <div className="app-topbar-actions">
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
              <div style={{ padding: "8px 12px 4px" }}>
                <div style={{ position: "relative" }}>
                  <input
                    id="search-input"
                    className="mint-input"
                    type="search"
                    placeholder="Search by name or URL…"
                    value={searchQuery}
                    onChange={(e) => setSearchQuery(e.target.value)}
                    aria-label="Search web apps"
                    style={{ width: "100%", boxSizing: "border-box" }}
                  />
                  {searchQuery ? (
                    <button
                      type="button"
                      aria-label="Clear search"
                      onClick={() => setSearchQuery("")}
                      style={{
                        position: "absolute",
                        right: "8px",
                        top: "50%",
                        transform: "translateY(-50%)",
                        background: "none",
                        border: "none",
                        cursor: "pointer",
                        padding: "0",
                        fontSize: "1rem",
                        lineHeight: 1,
                      }}
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
            ) : (
              <div className="empty-state detail-empty">
                <SettingsIcon />
                <h3>No web app selected</h3>
                <p>Pick one from the library or create a new launcher to configure runtime and advanced window options.</p>
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
                  className="ghost-button"
                  type="button"
                  title="Rescan for installed browsers"
                  onClick={refresh}
                  style={{ marginLeft: "auto", padding: "4px 8px", fontSize: "0.75rem" }}
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
              onSettingsSaved={setAppSettings}
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
