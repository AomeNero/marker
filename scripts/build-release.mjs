#!/usr/bin/env node
/**
 * One-shot local release build:
 *   1. npx tauri build            — minisign updater signing via TAURI_SIGNING_PRIVATE_KEY
 *   2. scripts/build-portable.sh  — green/portable zip
 *   3. stage updater-dist/        — MSI + latest.json for marker.aomenero.com
 *
 * Usage:
 *   npm run build:release
 *   npm run build:release -- --stage-only   # stage updater-dist/ from existing target/ artifacts
 */

import { spawnSync } from 'node:child_process'
import { copyFileSync, existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs'
import { basename, join, resolve } from 'node:path'
import { homedir } from 'node:os'

const UPDATER_BASE_URL = 'https://marker.aomenero.com'
const DEFAULT_KEY_FILE = join(homedir(), '.tauri', 'marker-updater-v2.key')

const stageOnly = process.argv.includes('--stage-only')
const root = resolve(import.meta.dirname, '..')
const version = JSON.parse(readFileSync(join(root, 'package.json'), 'utf8')).version
const bundleDir = join(root, 'target', 'release', 'bundle')
const updaterDist = join(root, 'updater-dist')

function run(cmd, { env } = {}) {
  const result = spawnSync(cmd, { stdio: 'inherit', shell: true, env })
  if (result.status !== 0) {
    process.exit(result.status ?? 1)
  }
}

function stageUpdaterDist() {
  const msiName = `Marker_${version}_x64_zh-CN.msi`
  const msi = join(bundleDir, 'msi', msiName)
  const sig = `${msi}.sig`
  if (!existsSync(msi) || !existsSync(sig)) {
    console.error(`✖ Missing ${msi} or ${sig}. Run the tauri build first (npm run build:release).`)
    process.exit(1)
  }
  mkdirSync(updaterDist, { recursive: true })
  copyFileSync(msi, join(updaterDist, msiName))
  const latest = {
    version,
    pub_date: new Date().toISOString().replace(/\.\d{3}Z$/, 'Z'),
    platforms: {
      'windows-x86_64': {
        signature: readFileSync(sig, 'utf8'),
        url: `${UPDATER_BASE_URL}/${msiName}`,
      },
    },
  }
  writeFileSync(join(updaterDist, 'latest.json'), `${JSON.stringify(latest, null, 2)}\n`, 'utf8')
  console.log(`==> Staged ${join('updater-dist', msiName)} and latest.json (v${version})`)
  console.log(`==> Upload updater-dist/* to ${UPDATER_BASE_URL}/ so clients can auto-update.`)
}

if (stageOnly) {
  stageUpdaterDist()
  process.exit(0)
}

const keyFile = process.env.MARKER_UPDATER_KEY_FILE ?? DEFAULT_KEY_FILE
if (!existsSync(keyFile)) {
  console.error(`✖ Updater signing key not found: ${keyFile}`)
  console.error('  Set MARKER_UPDATER_KEY_FILE to the minisign secret key path.')
  process.exit(1)
}

console.log(`==> tauri build (updater signing key: ${keyFile})`)
run('npx tauri build', {
  env: {
    ...process.env,
    TAURI_SIGNING_PRIVATE_KEY: readFileSync(keyFile, 'utf8').trim(),
    TAURI_SIGNING_PRIVATE_KEY_PASSWORD: process.env.TAURI_SIGNING_PRIVATE_KEY_PASSWORD ?? '',
  },
})

console.log('==> portable zip')
run('bash scripts/build-portable.sh')

stageUpdaterDist()

console.log(`
✔ Release build complete (v${version}).

  Artifacts:
    target/release/bundle/msi/Marker_${version}_x64_zh-CN.msi
    target/release/bundle/nsis/Marker_${version}_x64-setup.exe
    target/release/bundle/portable/Marker_${version}_x64_portable.zip
    updater-dist/latest.json

  Next: create the GitHub release and upload updater-dist/* — see docs/releasing.md.
`)
