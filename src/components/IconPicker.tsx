import { useRef, useState } from "react";
import { CloseIcon, FolderIcon, SparkIcon } from "./Icons";
import { toAssetUrl } from "../lib/tauri";

export type IconMode = "keep" | "site" | "clear" | "upload";

interface IconPickerProps {
  name: string;
  url: string;
  uploadedIconDataUrl: string | null;
  iconPath: string | null;
  iconDataUrl?: string | null;
  iconMode: IconMode;
  canUseStoredIcon: boolean;
  onChange: (value: string | null) => void;
  onModeChange: (mode: IconMode) => void;
}

export function siteIconPreviewUrl(url: string) {
  try {
    const target = new URL(url.includes("://") ? url : `https://${url}`);
    return `https://www.google.com/s2/favicons?domain=${target.hostname}&sz=128`;
  } catch {
    return null;
  }
}

export function IconPicker({
  name,
  url,
  uploadedIconDataUrl,
  iconPath,
  iconDataUrl,
  iconMode,
  canUseStoredIcon,
  onChange,
  onModeChange,
}: IconPickerProps) {
  const inputRef = useRef<HTMLInputElement | null>(null);
  const [error, setError] = useState<string | null>(null);
  const preview =
    uploadedIconDataUrl
      ?? (iconMode === "clear"
        ? null
        : iconMode === "site"
          ? siteIconPreviewUrl(url)
          : iconDataUrl ?? toAssetUrl(iconPath));

  async function handleFileChange(event: React.ChangeEvent<HTMLInputElement>) {
    const file = event.target.files?.[0];
    if (!file) {
      return;
    }

    if (!["image/png", "image/jpeg", "image/x-icon", "image/vnd.microsoft.icon"].includes(file.type)) {
      setError("Use a PNG, JPEG, or ICO file.");
      return;
    }

    const reader = new FileReader();
    reader.onload = () => {
      onChange(typeof reader.result === "string" ? reader.result : null);
      onModeChange("upload");
      setError(null);
    };
    reader.onerror = () => {
      setError("Failed to read the selected icon.");
    };
    reader.readAsDataURL(file);
  }

  return (
    <div className="icon-picker-row">
      <div className="mint-icon-preview">
        {preview ? (
          <img src={preview} alt={`${name || "Web app"} icon`} />
        ) : (
          <span>{(name || "W").slice(0, 1).toUpperCase()}</span>
        )}
      </div>

      <div className="mint-icon-actions">
        <button
          className="mint-icon-action"
          type="button"
          aria-label="Choose icon"
          data-tooltip="Choose icon"
          title="Choose icon"
          onClick={() => inputRef.current?.click()}
        >
          <FolderIcon />
        </button>

        {canUseStoredIcon ? (
          <button
            className="mint-icon-action"
            type="button"
            aria-label="Use saved icon"
            data-tooltip="Use saved icon"
            title="Use saved icon"
            onClick={() => {
              onChange(null);
              onModeChange("keep");
              setError(null);
            }}
          >
            <FolderIcon />
          </button>
        ) : null}

        <button
          className="mint-icon-action"
          type="button"
          aria-label="Use site icon"
          data-tooltip="Use site icon"
          title="Use site icon"
          onClick={() => {
            onChange(null);
            onModeChange("site");
            setError(null);
          }}
        >
          <SparkIcon />
        </button>

        <button
          className="mint-icon-action"
          type="button"
          aria-label="Clear custom icon"
          data-tooltip="Clear custom icon"
          title="Clear custom icon"
          onClick={() => {
            onChange(null);
            onModeChange("clear");
            setError(null);
          }}
        >
          <CloseIcon />
        </button>
      </div>

      {error ? <p className="mint-field-help error">{error}</p> : null}

      <input
        ref={inputRef}
        hidden
        type="file"
        accept=".png,.jpg,.jpeg,.ico,image/png,image/jpeg,image/x-icon,image/vnd.microsoft.icon"
        onChange={handleFileChange}
      />
    </div>
  );
}
