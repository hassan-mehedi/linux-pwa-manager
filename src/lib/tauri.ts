import { convertFileSrc, invoke } from "@tauri-apps/api/core";

export type BrowserChoice =
  | "auto"
  | "firefox"
  | "chromium"
  | "chrome"
  | "brave"
  | "edge"
  | "vivaldi"
  | "embedded";

export type WindowMode = "normal" | "maximized" | "fullscreen";

export interface WebApp {
  id: string;
  name: string;
  url: string;
  iconPath: string | null;
  iconDataUrl?: string | null;
  category: string;
  browser: BrowserChoice;
  navBar: boolean;
  isolated: boolean;
  tray: boolean;
  windowMode: WindowMode;
  createdAt: string;
  updatedAt: string;
}

export interface WebAppPayload {
  id?: string;
  name: string;
  url: string;
  category: string;
  browser: BrowserChoice;
  navBar: boolean;
  isolated: boolean;
  tray: boolean;
  windowMode: WindowMode;
  uploadedIconDataUrl?: string | null;
  iconSourceUrl?: string | null;
  clearIcon: boolean;
}

export interface DetectedBrowser {
  id: BrowserChoice;
  name: string;
  path: string;
  version: string | null;
  supportsAppMode: boolean;
}

export interface AppInfo {
  dataDir: string;
  configDir: string;
  stateDir: string;
  dbPath: string;
  desktopDir: string;
  profilesDir: string;
  iconsDir: string;
}

export interface AppSettings {
  httpTimeoutSecs: number;
  maxIconSizeMb: number;
  historyLimit: number;
  defaultBrowser: BrowserChoice;
  defaultWindowMode: WindowMode;
  launchOnLogin: boolean;
}

export interface EmbeddedPageState {
  url: string;
  loading: boolean;
  canGoBack: boolean;
  canGoForward: boolean;
}

export const EMBEDDED_PAGE_STATE_EVENT = "embedded://page-state";

export function listWebApps() {
  return invoke<WebApp[]>("list_webapps");
}

export function getWebApp(target: string) {
  return invoke<WebApp>("get_webapp", { target });
}

export function createWebApp(payload: WebAppPayload) {
  return invoke<WebApp>("create_webapp", { payload });
}

export function updateWebApp(payload: WebAppPayload) {
  return invoke<WebApp>("update_webapp", { payload });
}

export function deleteWebApp(target: string) {
  return invoke<void>("delete_webapp", { target });
}

export function launchWebApp(target: string) {
  return invoke<void>("launch_webapp", { target });
}

export function listBrowsers() {
  return invoke<DetectedBrowser[]>("list_browsers");
}

export function getAppInfo() {
  return invoke<AppInfo>("get_app_info");
}

export function getSettings() {
  return invoke<AppSettings>("get_settings");
}

export function saveSettings(settings: AppSettings) {
  return invoke<void>("save_settings", { payload: settings });
}

export function openManagedPath(path: string) {
  return invoke<void>("open_managed_path", { path });
}

export function exportBackup(path: string) {
  return invoke<void>("export_backup", { path });
}

export function importBackup(path: string, replaceExisting: boolean) {
  return invoke<void>("import_backup", { path, replaceExisting });
}

export function getEmbeddedCurrentUrl(target: string) {
  return invoke<string>("embedded_current_url", { target });
}

export function getEmbeddedState(target: string) {
  return invoke<EmbeddedPageState>("embedded_state", { target });
}

export function embeddedNavigate(target: string, url: string) {
  return invoke<string>("embedded_navigate", { target, url });
}

export function embeddedGoBack(target: string) {
  return invoke<void>("embedded_go_back", { target });
}

export function embeddedGoForward(target: string) {
  return invoke<void>("embedded_go_forward", { target });
}

export function embeddedReload(target: string) {
  return invoke<void>("embedded_reload", { target });
}

export function embeddedResize(target: string, topInset: number) {
  return invoke<void>("embedded_resize", { target, topInset });
}

export function embeddedOpenInBrowser(target: string) {
  return invoke<void>("embedded_open_in_browser", { target });
}

export function toAssetUrl(path: string | null | undefined) {
  return path ? convertFileSrc(path) : null;
}

export function errorMessage(error: unknown, fallback: string) {
  if (error instanceof Error && error.message.trim()) {
    return error.message;
  }

  if (typeof error === "string" && error.trim()) {
    return error;
  }

  if (
    typeof error === "object" &&
    error !== null &&
    "message" in error &&
    typeof error.message === "string" &&
    error.message.trim()
  ) {
    return error.message;
  }

  return fallback;
}
