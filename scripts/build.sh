#!/usr/bin/env bash
#
# Manual build entry: full Tauri build, output goes to <repo>/target/
# (target dir is configured in .cargo/config.toml)
# Usage: bash scripts/build.sh
#
# Local machines have no TAURI_SIGNING_PRIVATE_KEY, so updater artifacts are
# disabled via --config. Official releases must use `npm run release`.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

npx tauri build --config '{"bundle":{"createUpdaterArtifacts":false}}'

echo ""
echo "==> Build finished. Artifacts:"
ls -lh target/release/marker.exe 2>/dev/null || true
find target/release/bundle -maxdepth 2 -type f \( -name '*.exe' -o -name '*.msi' \) \
  -exec ls -lh {} + 2>/dev/null || true
