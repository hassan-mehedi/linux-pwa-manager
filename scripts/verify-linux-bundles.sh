#!/usr/bin/env bash

set -euo pipefail

fail() {
  echo "bundle smoke failed: $*" >&2
  exit 1
}

assert_contains() {
  local haystack=$1
  local needle=$2

  if ! grep -Fq -- "$needle" <<<"$haystack"; then
    fail "expected to find '$needle'"
  fi
}

assert_file() {
  local path=$1

  [[ -f "$path" ]] || fail "missing file: $path"
}

find_single_file() {
  local dir=$1
  local pattern=$2
  local -a matches=()

  [[ -d "$dir" ]] || fail "missing directory: $dir"

  mapfile -d '' matches < <(find "$dir" -maxdepth 1 -type f -name "$pattern" -print0 | sort -z)

  if [[ ${#matches[@]} -ne 1 ]]; then
    fail "expected exactly one $pattern in $dir, found ${#matches[@]}"
  fi

  printf '%s\n' "${matches[0]}"
}

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
bundle_root=${1:-"$repo_root/src-tauri/target/release/bundle"}
tauri_config="$repo_root/src-tauri/tauri.conf.json"
cargo_manifest="$repo_root/src-tauri/Cargo.toml"
verify_appimage=${VERIFY_APPIMAGE:-1}

[[ -d "$bundle_root" ]] || fail "bundle root does not exist: $bundle_root"

product_name=$(node -e 'const fs=require("fs"); const cfg=JSON.parse(fs.readFileSync(process.argv[1], "utf8")); process.stdout.write(cfg.productName);' "$tauri_config")
version=$(node -e 'const fs=require("fs"); const cfg=JSON.parse(fs.readFileSync(process.argv[1], "utf8")); process.stdout.write(cfg.version);' "$tauri_config")
binary_name=$(sed -n 's/^name = "\(.*\)"/\1/p' "$cargo_manifest" | head -n 1)

[[ -n "$binary_name" ]] || fail "could not determine Cargo package name"

package_name=$(printf '%s' "$product_name" | tr '[:upper:]' '[:lower:]' | sed -E 's/[^a-z0-9]+/-/g; s/^-+//; s/-+$//')
desktop_entry="$product_name.desktop"
icon_name="$binary_name.png"
release_binary="$repo_root/src-tauri/target/release/$binary_name"

assert_file "$release_binary"
"$release_binary" --help >/dev/null

echo "Verifying Debian bundle"
deb_file=$(find_single_file "$bundle_root/deb" '*.deb')
deb_dir=${deb_file%.deb}
assert_file "$deb_file"

ar_members=$(ar t "$deb_file")
assert_contains "$ar_members" "debian-binary"
assert_contains "$ar_members" "control.tar"
assert_contains "$ar_members" "data.tar"

control_archive=$(find_single_file "$deb_dir" 'control.tar.*')
data_archive=$(find_single_file "$deb_dir" 'data.tar.*')

deb_control=$(tar -xOf "$control_archive" control)
assert_contains "$deb_control" "Package: $package_name"
assert_contains "$deb_control" "Version: $version"
assert_contains "$deb_control" "Depends:"
assert_contains "$deb_control" "libgtk-3-0"
assert_contains "$deb_control" "libwebkit2gtk-4.1-0"

deb_files=$(tar -tf "$data_archive")
assert_contains "$deb_files" "usr/bin/$binary_name"
assert_contains "$deb_files" "usr/share/applications/$desktop_entry"
assert_contains "$deb_files" "usr/share/icons/hicolor/128x128/apps/$icon_name"

deb_desktop=$(tar -xOf "$data_archive" "usr/share/applications/$desktop_entry")
assert_contains "$deb_desktop" "Exec=$binary_name"
assert_contains "$deb_desktop" "Icon=$binary_name"
assert_contains "$deb_desktop" "Name=$product_name"

echo "Verifying RPM bundle"
rpm_file=$(find_single_file "$bundle_root/rpm" '*.rpm')
assert_file "$rpm_file"

rpm_name=$(rpm -qp --qf '%{NAME}\n' "$rpm_file")
rpm_version=$(rpm -qp --qf '%{VERSION}\n' "$rpm_file")
[[ "$rpm_name" == "$package_name" ]] || fail "unexpected rpm name: $rpm_name"
[[ "$rpm_version" == "$version" ]] || fail "unexpected rpm version: $rpm_version"

rpm_files=$(rpm -qlp "$rpm_file")
assert_contains "$rpm_files" "/usr/bin/$binary_name"
assert_contains "$rpm_files" "/usr/share/applications/$desktop_entry"
assert_contains "$rpm_files" "/usr/share/icons/hicolor/128x128/apps/$icon_name"

rpm_requires=$(rpm -qp --requires "$rpm_file")
assert_contains "$rpm_requires" "libgtk-3.so.0"
assert_contains "$rpm_requires" "libwebkit2gtk-4.1.so.0"

if [[ "$verify_appimage" != "0" ]]; then
  echo "Verifying AppImage bundle"
  [[ -d "$bundle_root/appimage" ]] || fail "missing AppImage bundle directory: $bundle_root/appimage"
  appimage_file=$(find_single_file "$bundle_root/appimage" '*.AppImage')
  assert_file "$appimage_file"
  [[ -x "$appimage_file" ]] || fail "AppImage is not executable: $appimage_file"

  appimage_tmp=$(mktemp -d)
  trap 'rm -rf "$appimage_tmp"' EXIT

  (
    cd "$appimage_tmp"
    "$appimage_file" --appimage-extract >/dev/null
  )

  assert_file "$appimage_tmp/squashfs-root/AppRun"
  assert_file "$appimage_tmp/squashfs-root/usr/bin/$binary_name"
  assert_file "$appimage_tmp/squashfs-root/usr/share/icons/hicolor/128x128/apps/$icon_name"

  appimage_desktop="$appimage_tmp/squashfs-root/usr/share/applications/$desktop_entry"
  assert_file "$appimage_desktop"

  appimage_desktop_contents=$(<"$appimage_desktop")
  assert_contains "$appimage_desktop_contents" "Icon=$binary_name"
  assert_contains "$appimage_desktop_contents" "Name=$product_name"
else
  echo "Skipping AppImage verification"
fi

echo "Linux bundle smoke verification passed"
