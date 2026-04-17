import { useEffect, useRef, useState } from "react";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { getCurrentWindow } from "@tauri-apps/api/window";
import {
  ArrowLeftIcon,
  ArrowRightIcon,
  BrowserIcon,
  HomeIcon,
  RefreshIcon,
} from "./Icons";
import {
  EMBEDDED_PAGE_STATE_EVENT,
  errorMessage,
  embeddedGoBack,
  embeddedGoForward,
  embeddedNavigate,
  embeddedOpenInBrowser,
  embeddedReload,
  embeddedResize,
  getEmbeddedState,
  getWebApp,
  toAssetUrl,
  type EmbeddedPageState,
  type WebApp,
} from "../lib/tauri";

const currentWindow = getCurrentWindow();
const currentWebviewWindow = getCurrentWebviewWindow();

export function EmbeddedShell() {
  const params = new URLSearchParams(window.location.search);
  const target = params.get("webapp") ?? "";
  const headerRef = useRef<HTMLElement | null>(null);
  const [webApp, setWebApp] = useState<WebApp | null>(null);
  const [currentUrl, setCurrentUrl] = useState("");
  const [draftUrl, setDraftUrl] = useState("");
  const [loading, setLoading] = useState(true);
  const [canGoBack, setCanGoBack] = useState(false);
  const [canGoForward, setCanGoForward] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!target) {
      setError("Missing embedded web app target.");
      setLoading(false);
      return;
    }

    let cancelled = false;
    async function load() {
      try {
        const [item, state] = await Promise.all([getWebApp(target), getEmbeddedState(target)]);
        if (!cancelled) {
          setWebApp(item);
          setCurrentUrl(state.url);
          setDraftUrl(state.url);
          setLoading(state.loading);
          setCanGoBack(state.canGoBack);
          setCanGoForward(state.canGoForward);
        }
      } catch (err) {
        if (!cancelled) {
          setError(errorMessage(err, "Failed to load embedded web app."));
          setLoading(false);
        }
      }
    }

    void load();
    return () => {
      cancelled = true;
    };
  }, [target]);

  useEffect(() => {
    if (!target) {
      return;
    }

    let unlistenPageState: (() => void) | undefined;
    let unlistenResize: (() => void) | undefined;
    let observer: ResizeObserver | undefined;

    async function bind() {
      unlistenPageState = await currentWebviewWindow.listen<EmbeddedPageState>(
        EMBEDDED_PAGE_STATE_EVENT,
        ({ payload }) => {
          setCurrentUrl(payload.url);
          setDraftUrl(payload.url);
          setLoading(payload.loading);
          setCanGoBack(payload.canGoBack);
          setCanGoForward(payload.canGoForward);
        },
      );

      const syncBounds = () => {
        const topInset = webApp?.navBar ? headerRef.current?.offsetHeight ?? 0 : 0;
        void embeddedResize(target, topInset).catch((err) => {
          setError(errorMessage(err, "Failed to resize embedded view."));
        });
      };

      unlistenResize = await currentWindow.onResized(syncBounds);
      observer = new ResizeObserver(syncBounds);
      if (headerRef.current) {
        observer.observe(headerRef.current);
      }
      syncBounds();
    }

    void bind();

    return () => {
      unlistenPageState?.();
      unlistenResize?.();
      observer?.disconnect();
    };
  }, [target, webApp?.navBar]);

  async function handleNavigate(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!target || !draftUrl.trim()) {
      return;
    }

    try {
      setLoading(true);
      setError(null);
      const normalized = await embeddedNavigate(target, draftUrl.trim());
      setCurrentUrl(normalized);
      setDraftUrl(normalized);
    } catch (err) {
      setError(errorMessage(err, "Failed to navigate."));
      setLoading(false);
    }
  }

  async function handleReload() {
    if (!target) {
      return;
    }

    try {
      setLoading(true);
      setError(null);
      await embeddedReload(target);
    } catch (err) {
      setError(errorMessage(err, "Failed to reload the page."));
      setLoading(false);
    }
  }

  async function handleBack() {
    if (!target || !canGoBack) {
      return;
    }

    try {
      setLoading(true);
      setError(null);
      await embeddedGoBack(target);
    } catch (err) {
      setError(errorMessage(err, "Failed to go back."));
      setLoading(false);
    }
  }

  async function handleForward() {
    if (!target || !canGoForward) {
      return;
    }

    try {
      setLoading(true);
      setError(null);
      await embeddedGoForward(target);
    } catch (err) {
      setError(errorMessage(err, "Failed to go forward."));
      setLoading(false);
    }
  }

  async function handleOpenInBrowser() {
    if (!target) {
      return;
    }

    try {
      setError(null);
      await embeddedOpenInBrowser(target);
    } catch (err) {
      setError(errorMessage(err, "Failed to open the page in the browser."));
    }
  }

  if (!target) {
    return <main className="embedded-shell embedded-empty">Missing embedded target.</main>;
  }

  return (
    <main className={`embedded-shell${webApp?.navBar ? "" : " chrome-hidden"}`}>
      {webApp?.navBar ? (
        <header className="embedded-chrome" ref={headerRef} data-tauri-drag-region>
          <div className="embedded-brand">
            <div className="embedded-brand-icon">
              {webApp?.iconPath ? (
                <img src={toAssetUrl(webApp.iconPath) ?? undefined} alt="" />
              ) : (
                <span>{webApp?.name.slice(0, 1).toUpperCase() ?? "W"}</span>
              )}
            </div>
            <div>
              <strong>{webApp?.name ?? "Web App"}</strong>
              <span>{loading ? "Loading…" : currentUrl}</span>
            </div>
          </div>

          <form className="embedded-nav" onSubmit={handleNavigate}>
            <div className="embedded-button-row">
              <button
                className="icon-button"
                type="button"
                disabled={!canGoBack}
                aria-label="Back"
                data-tooltip="Back"
                title="Back"
                onClick={() => void handleBack()}
              >
                <ArrowLeftIcon />
              </button>
              <button
                className="icon-button"
                type="button"
                disabled={!canGoForward}
                aria-label="Forward"
                data-tooltip="Forward"
                title="Forward"
                onClick={() => void handleForward()}
              >
                <ArrowRightIcon />
              </button>
              <button
                className="icon-button"
                type="button"
                aria-label="Home"
                data-tooltip="Home"
                title="Home"
                onClick={() => webApp && void embeddedNavigate(target, webApp.url)}
              >
                <HomeIcon />
              </button>
              <button
                className="icon-button"
                type="button"
                aria-label="Reload"
                data-tooltip="Reload"
                title="Reload"
                onClick={() => void handleReload()}
              >
                <RefreshIcon />
              </button>
            </div>

            <div className="embedded-url-wrap">
              <input
                className="embedded-url"
                value={draftUrl}
                onChange={(event) => setDraftUrl(event.target.value)}
                spellCheck={false}
              />
              <button className="button" type="submit">
                Go
              </button>
            </div>

            <button
              className="icon-button"
              type="button"
              aria-label="Browser"
              data-tooltip="Open in browser"
              title="Open in browser"
              onClick={() => void handleOpenInBrowser()}
            >
              <BrowserIcon />
            </button>
          </form>
        </header>
      ) : null}

      <section className="embedded-stage">
        {loading ? <div className="embedded-overlay">Loading {webApp?.name ?? "web app"}…</div> : null}
        {error ? <div className="embedded-overlay error-banner">{error}</div> : null}
      </section>
    </main>
  );
}
