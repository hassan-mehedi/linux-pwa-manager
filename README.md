# Linux PWA Manager

Linux desktop app for turning websites into app-style launchers with isolated profiles, generated `.desktop` entries, and a shared GUI/CLI core.

## Stack

- Tauri v2
- React + TypeScript + Vite
- Rust backend with SQLite

## Current MVP

- Create, edit, delete, list, and launch saved web apps
- Detect installed browsers and store the preferred runtime
- Normalize URLs and prevent duplicates
- Auto-fetch or upload icons and save them under XDG data directories
- Generate per-app `.desktop` launchers
- Provide a CLI for add/edit/remove/list/launch/browsers
- Save global defaults for new web apps and launch-on-login behavior
- Export and import backups as JSON, including icons and settings
- Keep tray-enabled setups running in the background when the main window closes

## Linux prerequisites

On Debian/Ubuntu-based systems, install:

```bash
sudo apt-get update
sudo apt-get install -y \
  libappindicator3-dev \
  libgtk-3-dev \
  libjavascriptcoregtk-4.1-dev \
  rpm \
  librsvg2-dev \
  libsoup-3.0-dev \
  libwebkit2gtk-4.1-dev \
  patchelf
```

## Local development

```bash
npm install
npm run check:ts
npm run fmt:check
npm run lint:rust
npm test
npm run tauri dev
```

## CLI examples

```bash
linux-pwa-manager add --name YouTube --url youtube.com --browser chromium
linux-pwa-manager list --json
linux-pwa-manager launch YouTube
linux-pwa-manager export ~/backups/web-apps.json
linux-pwa-manager import ~/backups/web-apps.json --replace-existing
```

`linux-pwa-manager add` now uses the saved default browser and default window mode when those flags are omitted.

## XDG paths

- Database: `$XDG_DATA_HOME/linux-pwa-manager/webapps.db`
- Icons: `$XDG_DATA_HOME/linux-pwa-manager/icons/`
- Profiles: `$XDG_DATA_HOME/linux-pwa-manager/profiles/`
- Desktop entries: `$XDG_DATA_HOME/applications/`
- Config: `$XDG_CONFIG_HOME/linux-pwa-manager/`
- State: `$XDG_STATE_HOME/linux-pwa-manager/`

## Backup Format

Backups are JSON files that include:

- Saved web apps
- Embedded custom icons as data URLs
- Saved application settings

Imports can either merge into the current library or replace it completely.

## License

MIT. See [LICENSE](./LICENSE).
