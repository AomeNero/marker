#!/usr/bin/env bash
#
# Manual build entry: full Tauri build, output goes to <repo>/target/
# (target dir is configured in .cargo/config.toml)
# Usage: bash scripts/build.sh
#
# Updater artifacts are disabled here so no minisign key is needed for a quick
# local build. For official releases use `npm run build:release` — see docs/releasing.md.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

npx tauri build --config '{"bundle":{"createUpdaterArtifacts":false}}'

echo ""
echo "==> Build finished. Artifacts:"
ls -lh target/release/marker.exe 2>/dev/null || true
find target/release/bundle -maxdepth 2 -type f \( -name '*.exe' -o -name '*.msi' \) \
  -exec ls -lh {} + 2>/dev/null || true
