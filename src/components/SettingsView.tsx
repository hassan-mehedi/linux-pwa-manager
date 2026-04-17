import { useEffect, useRef, useState } from "react";
import { DatabaseIcon, FolderIcon, SettingsIcon } from "./Icons";
import {
  errorMessage,
  exportBackup,
  getAppInfo,
  getSettings,
  importBackup,
  openManagedPath,
  saveSettings,
  type AppInfo,
  type AppSettings,
  type BrowserChoice,
  type WindowMode,
} from "../lib/tauri";

function PathRow({
  label,
  value,
  icon: Icon,
  onOpen,
  onCopy,
}: {
  label: string;
  value: string;
  icon: typeof DatabaseIcon;
  onOpen: () => void;
  onCopy: () => void;
}) {
  return (
    <div className="drawer-path-row">
      <div className="drawer-path-icon">
        <Icon />
      </div>
      <div className="drawer-path-copy">
        <dt>{label}</dt>
        <dd>{value}</dd>
      </div>
      <div className="detail-actions" style={{ marginLeft: "auto" }}>
        <button className="ghost-button" type="button" onClick={onCopy}>
          Copy
        </button>
        <button className="ghost-button" type="button" onClick={onOpen}>
          Open
        </button>
      </div>
    </div>
  );
}

export function SettingsView({
  onSettingsSaved,
  onBackupImported,
}: {
  onSettingsSaved?: (settings: AppSettings) => void;
  onBackupImported?: () => void;
}) {
  const [info, setInfo] = useState<AppInfo | null>(null);
  const [settings, setSettings] = useState<AppSettings | null>(null);
  const [saving, setSaving] = useState(false);
  const [saveError, setSaveError] = useState<string | null>(null);
  const [saveSuccess, setSaveSuccess] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [backupPath, setBackupPath] = useState("");
  const [backupBusy, setBackupBusy] = useState<"export" | "import" | null>(null);
  const [backupError, setBackupError] = useState<string | null>(null);
  const [backupSuccess, setBackupSuccess] = useState<string | null>(null);
  const [replaceExisting, setReplaceExisting] = useState(false);
  const successTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    let cancelled = false;

    async function load() {
      try {
        const [payload, s] = await Promise.all([getAppInfo(), getSettings()]);
        if (!cancelled) {
          setInfo(payload);
          setSettings(s);
          onSettingsSaved?.(s);
        }
      } catch (err) {
        if (!cancelled) {
          setError(errorMessage(err, "Failed to load app settings."));
        }
      }
    }

    void load();
    return () => { cancelled = true; };
  }, []);

  useEffect(() => {
    return () => {
      if (successTimerRef.current) clearTimeout(successTimerRef.current);
    };
  }, []);

  async function handleSave(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!settings) return;

    const { httpTimeoutSecs, maxIconSizeMb, historyLimit } = settings;
    if (
      !Number.isFinite(httpTimeoutSecs) || httpTimeoutSecs < 1 ||
      !Number.isFinite(maxIconSizeMb) || maxIconSizeMb < 1 ||
      !Number.isFinite(historyLimit) || historyLimit < 1
    ) {
      setSaveError("Please enter valid values for all numeric fields.");
      return;
    }

    try {
      setSaving(true);
      setSaveError(null);
      setSaveSuccess(false);
      await saveSettings(settings);
      onSettingsSaved?.(settings);
      setSaveSuccess(true);
      if (successTimerRef.current) clearTimeout(successTimerRef.current);
      successTimerRef.current = setTimeout(() => setSaveSuccess(false), 2000);
    } catch (err) {
      setSaveError(errorMessage(err, "Failed to save settings."));
    } finally {
      setSaving(false);
    }
  }

  function updateSetting<K extends keyof AppSettings>(key: K, value: AppSettings[K]) {
    setSettings((current) => current ? { ...current, [key]: value } : current);
  }

  async function copyPath(value: string) {
    try {
      await navigator.clipboard.writeText(value);
      setBackupSuccess(`Copied ${value}`);
      setBackupError(null);
    } catch (err) {
      setBackupError(errorMessage(err, "Failed to copy path."));
    }
  }

  async function handleOpenPath(value: string) {
    try {
      setBackupError(null);
      await openManagedPath(value);
    } catch (err) {
      setBackupError(errorMessage(err, "Failed to open path."));
    }
  }

  async function handleExportBackup() {
    if (!backupPath.trim()) {
      setBackupError("Enter a backup path first.");
      return;
    }

    try {
      setBackupBusy("export");
      setBackupError(null);
      await exportBackup(backupPath.trim());
      setBackupSuccess(`Exported backup to ${backupPath.trim()}`);
    } catch (err) {
      setBackupError(errorMessage(err, "Failed to export backup."));
    } finally {
      setBackupBusy(null);
    }
  }

  async function handleImportBackup() {
    if (!backupPath.trim()) {
      setBackupError("Enter a backup path first.");
      return;
    }

    try {
      setBackupBusy("import");
      setBackupError(null);
      await importBackup(backupPath.trim(), replaceExisting);
      const latest = await getSettings();
      setSettings(latest);
      onSettingsSaved?.(latest);
      onBackupImported?.();
      setBackupSuccess(`Imported backup from ${backupPath.trim()}`);
    } catch (err) {
      setBackupError(errorMessage(err, "Failed to import backup."));
    } finally {
      setBackupBusy(null);
    }
  }

  return (
    <>
      <section className="drawer-section">
        <div className="drawer-section-header">
          <SettingsIcon />
          <h2>Storage paths</h2>
        </div>
        <p className="drawer-empty">Directories used for the database, icons, profiles, and desktop files.</p>
        {error ? <p className="error-banner">{error}</p> : null}
        {info ? (
          <dl className="drawer-path-list">
            <PathRow label="Data" value={info.dataDir} icon={FolderIcon} onCopy={() => void copyPath(info.dataDir)} onOpen={() => void handleOpenPath(info.dataDir)} />
            <PathRow label="Config" value={info.configDir} icon={FolderIcon} onCopy={() => void copyPath(info.configDir)} onOpen={() => void handleOpenPath(info.configDir)} />
            <PathRow label="State" value={info.stateDir} icon={FolderIcon} onCopy={() => void copyPath(info.stateDir)} onOpen={() => void handleOpenPath(info.stateDir)} />
            <PathRow label="Database" value={info.dbPath} icon={DatabaseIcon} onCopy={() => void copyPath(info.dbPath)} onOpen={() => void handleOpenPath(info.dbPath)} />
            <PathRow label="Icons" value={info.iconsDir} icon={FolderIcon} onCopy={() => void copyPath(info.iconsDir)} onOpen={() => void handleOpenPath(info.iconsDir)} />
            <PathRow label="Profiles" value={info.profilesDir} icon={FolderIcon} onCopy={() => void copyPath(info.profilesDir)} onOpen={() => void handleOpenPath(info.profilesDir)} />
            <PathRow label="Desktop entries" value={info.desktopDir} icon={FolderIcon} onCopy={() => void copyPath(info.desktopDir)} onOpen={() => void handleOpenPath(info.desktopDir)} />
          </dl>
        ) : (
          <p className="drawer-empty">Loading paths…</p>
        )}
      </section>

      {settings ? (
        <section className="drawer-section">
          <div className="drawer-section-header">
            <SettingsIcon />
            <h2>Preferences</h2>
          </div>

          <form onSubmit={handleSave}>
            <div className="drawer-path-list" style={{ gap: "12px" }}>

              <label className="mint-field-row" style={{ flexDirection: "column", alignItems: "flex-start", gap: "4px" }}>
                <span className="mint-field-label">HTTP timeout (seconds)</span>
                <input
                  className="mint-input"
                  type="number"
                  min={1}
                  max={120}
                  value={settings.httpTimeoutSecs}
                  onChange={(e) => updateSetting("httpTimeoutSecs", e.target.value === "" ? NaN : Number(e.target.value))}
                />
              </label>

              <label className="mint-field-row" style={{ flexDirection: "column", alignItems: "flex-start", gap: "4px" }}>
                <span className="mint-field-label">Max icon size (MB)</span>
                <input
                  className="mint-input"
                  type="number"
                  min={1}
                  max={50}
                  value={settings.maxIconSizeMb}
                  onChange={(e) => updateSetting("maxIconSizeMb", e.target.value === "" ? NaN : Number(e.target.value))}
                />
              </label>

              <label className="mint-field-row" style={{ flexDirection: "column", alignItems: "flex-start", gap: "4px" }}>
                <span className="mint-field-label">Navigation history limit (entries)</span>
                <input
                  className="mint-input"
                  type="number"
                  min={5}
                  max={500}
                  value={settings.historyLimit}
                  onChange={(e) => updateSetting("historyLimit", e.target.value === "" ? NaN : Number(e.target.value))}
                />
              </label>

              <label className="mint-field-row" style={{ flexDirection: "column", alignItems: "flex-start", gap: "4px" }}>
                <span className="mint-field-label">Default browser</span>
                <select
                  className="mint-input mint-select"
                  value={settings.defaultBrowser}
                  onChange={(e) => updateSetting("defaultBrowser", e.target.value as BrowserChoice)}
                >
                  <option value="auto">Automatic</option>
                  <option value="chrome">Chrome</option>
                  <option value="chromium">Chromium</option>
                  <option value="firefox">Firefox</option>
                  <option value="brave">Brave</option>
                  <option value="edge">Edge</option>
                  <option value="vivaldi">Vivaldi</option>
                  <option value="embedded">Embedded</option>
                </select>
              </label>

              <label className="mint-field-row" style={{ flexDirection: "column", alignItems: "flex-start", gap: "4px" }}>
                <span className="mint-field-label">Default window mode</span>
                <select
                  className="mint-input mint-select"
                  value={settings.defaultWindowMode}
                  onChange={(e) => updateSetting("defaultWindowMode", e.target.value as WindowMode)}
                >
                  <option value="normal">Standard window</option>
                  <option value="maximized">Start maximized</option>
                  <option value="fullscreen">Start fullscreen</option>
                </select>
              </label>

              <label className="mint-switch-row">
                <div className="mint-switch-copy">
                  <span>Launch on login</span>
                  <small>Start the manager in the background after you sign in.</small>
                </div>
                <input
                  type="checkbox"
                  role="switch"
                  aria-label="Launch on login"
                  checked={settings.launchOnLogin}
                  onChange={(e) => updateSetting("launchOnLogin", e.target.checked)}
                />
                <span className="mint-switch-ui" aria-hidden="true">
                  <span />
                </span>
              </label>
            </div>

            {saveError ? <p className="error-banner" style={{ marginTop: "8px" }}>{saveError}</p> : null}
            {saveSuccess ? <p className="success-banner" style={{ marginTop: "8px" }}>Settings saved.</p> : null}

            <div style={{ marginTop: "12px" }}>
              <button className="mint-primary-button" type="submit" disabled={saving}>
                {saving ? "Saving…" : "Save settings"}
              </button>
            </div>
          </form>
        </section>
      ) : (
        <section className="drawer-section">
          <div className="drawer-section-header">
            <SettingsIcon />
            <h2>Preferences</h2>
          </div>
          <p className="drawer-empty">Loading preferences…</p>
        </section>
      )}

      <section className="drawer-section">
        <div className="drawer-section-header">
          <DatabaseIcon />
          <h2>Backup</h2>
        </div>
        <p className="drawer-empty">Export or import your web apps, icons, and settings as JSON.</p>

        <label className="mint-field-row" style={{ flexDirection: "column", alignItems: "flex-start", gap: "4px" }}>
          <span className="mint-field-label">Backup file path</span>
          <input
            className="mint-input"
            value={backupPath}
            onChange={(e) => setBackupPath(e.target.value)}
            placeholder="/home/mehedi/backups/web-apps-backup.json"
          />
        </label>

        <label className="mint-switch-row" style={{ marginTop: "12px" }}>
          <div className="mint-switch-copy">
            <span>Replace existing library on import</span>
            <small>Turn this on to wipe the current library before restoring the backup.</small>
          </div>
          <input
            type="checkbox"
            role="switch"
            aria-label="Replace existing library on import"
            checked={replaceExisting}
            onChange={(e) => setReplaceExisting(e.target.checked)}
          />
          <span className="mint-switch-ui" aria-hidden="true">
            <span />
          </span>
        </label>

        <div className="detail-actions" style={{ marginTop: "12px" }}>
          <button className="ghost-button" type="button" disabled={backupBusy !== null} onClick={() => void handleExportBackup()}>
            {backupBusy === "export" ? "Exporting…" : "Export backup"}
          </button>
          <button className="mint-primary-button" type="button" disabled={backupBusy !== null} onClick={() => void handleImportBackup()}>
            {backupBusy === "import" ? "Importing…" : "Import backup"}
          </button>
        </div>

        {backupError ? <p className="error-banner" style={{ marginTop: "8px" }}>{backupError}</p> : null}
        {backupSuccess ? <p className="success-banner" style={{ marginTop: "8px" }}>{backupSuccess}</p> : null}
      </section>
    </>
  );
}
