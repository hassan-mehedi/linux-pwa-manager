import { AppsIcon, PlusIcon } from "./Icons";
import { toAssetUrl, type WebApp } from "../lib/tauri";

interface WebAppListProps {
  items: WebApp[];
  selectedId: string | null;
  onSelect: (id: string) => void;
  onLaunch: (id: string) => void;
  onCreate?: () => void;
}

function hostnameFromUrl(url: string) {
  try {
    const target = new URL(url.includes("://") ? url : `https://${url}`);
    return target.hostname.replace(/^www\./, "");
  } catch {
    return url;
  }
}

export function WebAppList({
  items,
  selectedId,
  onSelect,
  onLaunch,
  onCreate,
}: WebAppListProps) {
  if (items.length === 0) {
    return (
      <div className="empty-state empty-library-state">
        <div className="empty-state-icon" aria-hidden="true">
          <AppsIcon />
        </div>

        <div className="empty-state-copy">
          <h3>Your library is empty</h3>
          <p>Create a web app to save a site as its own launcher with isolated profiles and window options.</p>
        </div>

        {onCreate ? (
          <button className="ghost-button" type="button" onClick={onCreate}>
            <PlusIcon />
            <span>New Web App</span>
          </button>
        ) : null}
      </div>
    );
  }

  return (
    <ul className="mint-webapp-list" role="listbox" aria-label="Web apps">
      {items.map((item) => {
        const iconUrl = item.iconDataUrl ?? toAssetUrl(item.iconPath);
        return (
          <li key={item.id}>
            <button
              className={`mint-webapp-row${item.id === selectedId ? " selected" : ""}`}
              type="button"
              role="option"
              aria-selected={item.id === selectedId}
              onClick={() => onSelect(item.id)}
              onDoubleClick={() => onLaunch(item.id)}
            >
              <div className="webapp-row-main">
                <div className="mint-webapp-icon" aria-hidden="true">
                  {iconUrl ? (
                    <img src={iconUrl} alt="" />
                  ) : (
                    <span>{item.name.slice(0, 1).toUpperCase()}</span>
                  )}
                </div>

                <div className="webapp-row-copy">
                  <strong className="mint-webapp-name">{item.name}</strong>
                  <span className="webapp-row-url">{hostnameFromUrl(item.url)}</span>
                </div>
              </div>

              <div className="webapp-row-meta" aria-hidden="true">
                <span>{item.category}</span>
                <span>{item.browser === "embedded" ? "Embedded" : "Browser"}</span>
              </div>
            </button>
          </li>
        );
      })}
    </ul>
  );
}
