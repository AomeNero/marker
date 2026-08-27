#!/usr/bin/env bash
#
# Build a Windows portable (green) zip from the Tauri release output.
# Usage: bash scripts/build-portable.sh
#
# Layout inside the zip:
#   Marker.exe
#   WebView2Loader.dll  # required for windows-gnu toolchain builds
#   marker.portable   # marker → config/logs/webview under ./data/
#   README.txt

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
RELEASE_DIR="$ROOT/target/release"

VERSION=$(cd "$ROOT" && node -p "require('./package.json').version")
echo "==> Building portable zip for Marker v${VERSION}"

EXE=""
for candidate in "$RELEASE_DIR/Marker.exe" "$RELEASE_DIR/marker.exe"; do
  if [[ -f "$candidate" ]]; then
    EXE="$candidate"
    break
  fi
done

if [[ -z "$EXE" ]]; then
  echo "ERROR: Marker.exe / marker.exe not found in $RELEASE_DIR. Run 'npm run build' first."
  exit 1
fi

# windows-gnu builds need the loader DLL next to the exe (MSVC statically links it).
WEBVIEW2_LOADER=""
for candidate in \
  "$RELEASE_DIR/WebView2Loader.dll" \
  "$RELEASE_DIR/build"/webview2-com-sys-*/out/x64/WebView2Loader.dll
do
  if [[ -f "$candidate" ]]; then
    WEBVIEW2_LOADER="$candidate"
    break
  fi
done

STAGING="$RELEASE_DIR/portable-staging"
rm -rf "$STAGING"
mkdir -p "$STAGING"

cp "$EXE" "$STAGING/Marker.exe"
if [[ -n "$WEBVIEW2_LOADER" ]]; then
  cp "$WEBVIEW2_LOADER" "$STAGING/WebView2Loader.dll"
  echo "==> Included WebView2Loader.dll from $WEBVIEW2_LOADER"
else
  echo "WARNING: WebView2Loader.dll not found — portable zip may fail on windows-gnu builds."
fi
# Unique marker file — presence enables portable data dir next to the exe.
: > "$STAGING/marker.portable"

cat > "$STAGING/README.txt" <<EOF
Marker Portable (绿色免安装版) v${VERSION}
========================================

中文
----
1. 解压到任意目录（建议非 C:\\Program Files）。
2. 双击 Marker.exe 即可运行，无需安装。
3. 配置、日志与 WebView 缓存写在本目录下的 data\\ 文件夹，不写系统 AppData。
4. 请保留同目录下的 marker.portable 标记文件（删除后会改回写入系统目录）。
5. 迁移或备份：复制整个文件夹即可。
6. 更新：下载新版 portable.zip，覆盖 Marker.exe（可保留 data\\ 与 marker.portable）。
7. 需要系统已安装 Microsoft Edge WebView2 Runtime（Win10/11 通常已自带）。

English
-------
1. Extract anywhere (avoid Program Files if you want fully writable portable data).
2. Run Marker.exe — no installer.
3. Config, logs, and WebView cache live under .\\data\\ next to the exe.
4. Keep the marker.portable marker file next to the exe (removing it switches back to AppData).
5. To move or back up: copy the whole folder.
6. To update: download a newer portable.zip and replace Marker.exe (keep data\\ and marker.portable).
7. Requires Microsoft Edge WebView2 Runtime (usually preinstalled on Windows 10/11).

https://github.com/ifer47/marker/releases
EOF

OUTDIR="$RELEASE_DIR/bundle/portable"
mkdir -p "$OUTDIR"
OUTPUT="$OUTDIR/Marker_${VERSION}_x64_portable.zip"
rm -f "$OUTPUT"

if command -v zip >/dev/null 2>&1; then
  (cd "$STAGING" && zip -q -r "$OUTPUT" .)
elif command -v powershell.exe >/dev/null 2>&1; then
  WIN_STAGING=$(cygpath -w "$STAGING" 2>/dev/null || echo "$STAGING")
  WIN_OUTPUT=$(cygpath -w "$OUTPUT" 2>/dev/null || echo "$OUTPUT")
  powershell.exe -NoProfile -Command \
    "Compress-Archive -Path (Join-Path '$WIN_STAGING' '*') -DestinationPath '$WIN_OUTPUT' -Force"
else
  echo "ERROR: neither zip nor powershell.exe found to create the archive."
  exit 1
fi

rm -rf "$STAGING"

echo "==> Portable package created: $OUTPUT"
