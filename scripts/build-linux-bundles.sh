#!/usr/bin/env bash

set -euo pipefail

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
runtime_dir=${XDG_CACHE_HOME:-"$HOME/.cache"}/tauri
runtime_file="$runtime_dir/runtime-x86_64"
runtime_url="https://github.com/AppImage/type2-runtime/releases/download/continuous/runtime-x86_64"

download_runtime() {
  mkdir -p "$runtime_dir"

  if command -v curl >/dev/null 2>&1; then
    curl -fsSL "$runtime_url" -o "$runtime_file"
  elif command -v wget >/dev/null 2>&1; then
    wget -qO "$runtime_file" "$runtime_url"
  else
    echo "missing downloader: install curl or wget" >&2
    exit 1
  fi

  chmod 755 "$runtime_file"
}

if [[ ! -f "$runtime_file" ]]; then
  echo "Downloading AppImage runtime to $runtime_file"
  download_runtime
fi

cd "$repo_root"

NO_STRIP=true \
LDAI_RUNTIME_FILE="$runtime_file" \
npm run tauri build -- --bundles deb,rpm,appimage
