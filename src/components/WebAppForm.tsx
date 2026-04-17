import { useEffect, useState } from "react";
import { BrowserSelect } from "./BrowserSelect";
import { CloseIcon, SettingsIcon } from "./Icons";
import { IconPicker, siteIconPreviewUrl, type IconMode } from "./IconPicker";
import {
  type AppSettings,
  errorMessage,
  type BrowserChoice,
  type DetectedBrowser,
  type WebApp,
  type WebAppPayload,
  type WindowMode,
} from "../lib/tauri";

interface WebAppFormProps {
  browsers: DetectedBrowser[];
  defaults: Pick<AppSettings, "defaultBrowser" | "defaultWindowMode">;
  initialValue?: WebApp | null;
  onClose: () => void;
  onSubmit: (payload: WebAppPayload) => Promise<void>;
}

const categories = [
  "Internet",
  "Office",
  "Development",
  "Social",
  "Entertainment",
  "Other",
];

function blankPayload(defaults: Pick<AppSettings, "defaultBrowser" | "defaultWindowMode">): WebAppPayload {
  return {
    name: "",
    url: "",
    category: "Internet",
    browser: defaults.defaultBrowser,
    navBar: false,
    isolated: true,
    tray: false,
    windowMode: defaults.defaultWindowMode,
    uploadedIconDataUrl: null,
    clearIcon: false,
  };
}

function advancedEnabled(form: WebAppPayload) {
  return form.navBar || !form.isolated || form.tray || form.windowMode !== "normal";
}

function SwitchRow({
  label,
  description,
  checked,
  onChange,
}: {
  label: string;
  description?: string;
  checked: boolean;
  onChange: (next: boolean) => void;
}) {
  return (
    <label className="mint-switch-row">
      <div className="mint-switch-copy">
        <span>{label}</span>
        {description ? <small>{description}</small> : null}
      </div>
      <input
        type="checkbox"
        role="switch"
        aria-label={label}
        checked={checked}
        onChange={(event) => onChange(event.target.checked)}
      />
      <span className="mint-switch-ui" aria-hidden="true">
        <span />
      </span>
    </label>
  );
}

export function WebAppForm({
  browsers,
  defaults,
  initialValue,
  onClose,
  onSubmit,
}: WebAppFormProps) {
  const [form, setForm] = useState<WebAppPayload>(() => blankPayload(defaults));
  const [submitting, setSubmitting] = useState(false);
  const [showAdvanced, setShowAdvanced] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [iconMode, setIconMode] = useState<IconMode>("site");

  useEffect(() => {
    if (!initialValue) {
      const next = blankPayload(defaults);
      setForm(next);
      setShowAdvanced(advancedEnabled(next));
      setIconMode("site");
      return;
    }

    const next: WebAppPayload = {
      id: initialValue.id,
      name: initialValue.name,
      url: initialValue.url,
      category: initialValue.category,
      browser: initialValue.browser,
      navBar: initialValue.navBar,
      isolated: initialValue.isolated,
      tray: initialValue.tray,
      windowMode: initialValue.windowMode,
      uploadedIconDataUrl: null,
      clearIcon: false,
    };

    setForm(next);
    setShowAdvanced(advancedEnabled(next));
    setIconMode(initialValue.iconPath || initialValue.iconDataUrl ? "keep" : "site");
  }, [defaults, initialValue]);

  function updateField<K extends keyof WebAppPayload>(key: K, value: WebAppPayload[K]) {
    setForm((current) => ({ ...current, [key]: value }));
  }

  async function handleSubmit(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();

    if (!form.name.trim() || !form.url.trim()) {
      setError("Name and address are required.");
      return;
    }

    try {
      setSubmitting(true);
      setError(null);
      const trimmedUrl = form.url.trim();
      const iconSourceUrl =
        iconMode === "site" ? siteIconPreviewUrl(trimmedUrl) : null;
      await onSubmit({
        ...form,
        name: form.name.trim(),
        url: trimmedUrl,
        uploadedIconDataUrl: iconMode === "upload" ? form.uploadedIconDataUrl : null,
        iconSourceUrl,
        clearIcon: iconMode === "clear",
      });
    } catch (err) {
      setError(errorMessage(err, "Failed to save web app."));
    } finally {
      setSubmitting(false);
    }
  }

  return (
    <div className="mint-dialog-backdrop" role="presentation">
      <div className="mint-dialog" role="dialog" aria-modal="true" aria-labelledby="webapp-form-title">
        <header className="mint-dialog-titlebar">
          <div className="mint-dialog-copy">
            <p className="app-eyebrow">Web app settings</p>
            <h2 id="webapp-form-title">{initialValue ? "Edit Web App" : "Add a New Web App"}</h2>
            <p>Choose the runtime, icon, and launch behavior you want.</p>
          </div>

          <button className="mint-dialog-close" type="button" aria-label="Close dialog" onClick={onClose}>
            <CloseIcon />
          </button>
        </header>

        <form className="mint-dialog-body" onSubmit={handleSubmit}>
          <section className="form-section">
            <div className="surface-heading">
              <div>
                <h3>Basics</h3>
                <p>Name, destination, icon, and preferred runtime.</p>
              </div>
            </div>

            <div className="form-grid">
              <label className="mint-field-row">
                <span className="mint-field-label">Name</span>
                <input
                  className="mint-input"
                  value={form.name}
                  aria-label="Name"
                  onChange={(event) => updateField("name", event.target.value)}
                  placeholder="YouTube"
                />
              </label>

              <label className="mint-field-row">
                <span className="mint-field-label">Address</span>
                <input
                  className="mint-input"
                  value={form.url}
                  aria-label="Address"
                  onChange={(event) => updateField("url", event.target.value)}
                  placeholder="youtube.com"
                />
              </label>

              <label className="mint-field-row">
                <span className="mint-field-label">Category</span>
                <select
                  className="mint-input mint-select"
                  value={form.category}
                  aria-label="Category"
                  onChange={(event) => updateField("category", event.target.value)}
                >
                  {categories.map((category) => (
                    <option key={category} value={category}>
                      {category}
                    </option>
                  ))}
                </select>
              </label>

              <div className="mint-field-row">
                <span className="mint-field-label">Browser</span>
                <BrowserSelect
                  browsers={browsers}
                  value={form.browser}
                  onChange={(value) => updateField("browser", value as BrowserChoice)}
                />
              </div>
            </div>

            <div className="mint-field-row icon-row">
              <span className="mint-field-label">Icon</span>
              <IconPicker
                name={form.name}
                url={form.url}
                uploadedIconDataUrl={form.uploadedIconDataUrl ?? null}
                iconPath={initialValue?.iconPath ?? null}
                iconDataUrl={initialValue?.iconDataUrl ?? null}
                iconMode={iconMode}
                canUseStoredIcon={Boolean(initialValue?.iconPath || initialValue?.iconDataUrl)}
                onChange={(value) => updateField("uploadedIconDataUrl", value)}
                onModeChange={setIconMode}
              />
            </div>
          </section>

          <section className="form-section">
            <div className="surface-heading compact">
              <div>
                <h3>Advanced settings</h3>
              </div>

              <button
                className={`mint-icon-action mint-advanced-toggle${showAdvanced ? " active" : ""}`}
                type="button"
                aria-label={showAdvanced ? "Hide advanced settings" : "Show advanced settings"}
                aria-expanded={showAdvanced}
                aria-controls="advanced-settings-panel"
                data-tooltip={showAdvanced ? "Hide advanced settings" : "Show advanced settings"}
                title={showAdvanced ? "Hide advanced settings" : "Show advanced settings"}
                onClick={() => setShowAdvanced((current) => !current)}
              >
                <SettingsIcon />
              </button>
            </div>

            {showAdvanced ? (
              <div className="mint-advanced-panel" id="advanced-settings-panel">
                <label className="mint-field-row">
                  <span className="mint-field-label">Startup window</span>
                  <div className="mint-field-control">
                    <select
                      className="mint-input mint-select"
                      value={form.windowMode}
                      aria-label="Startup window"
                      onChange={(event) => updateField("windowMode", event.target.value as WindowMode)}
                    >
                      <option value="normal">Standard window</option>
                      <option value="maximized">Start maximized</option>
                      <option value="fullscreen">Start fullscreen</option>
                    </select>
                  </div>
                </label>

                <SwitchRow
                  label="Embedded navigation bar"
                  checked={form.navBar}
                  onChange={(value) => updateField("navBar", value)}
                />

                <SwitchRow
                  label="Isolated browser profile"
                  checked={form.isolated}
                  onChange={(value) => updateField("isolated", value)}
                />

                <SwitchRow
                  label="Tray icon"
                  checked={form.tray}
                  onChange={(value) => updateField("tray", value)}
                />
              </div>
            ) : null}
          </section>

          {error ? <p className="error-banner dialog">{error}</p> : null}

          <footer className="mint-dialog-actions">
            <button className="mint-secondary-button" type="button" onClick={onClose}>
              Cancel
            </button>
            <button className="mint-primary-button" type="submit" disabled={submitting}>
              {submitting ? "Saving..." : "Save web app"}
            </button>
          </footer>
        </form>
      </div>
    </div>
  );
}
