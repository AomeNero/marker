#!/usr/bin/env node
/**
 * Marker release automation.
 *
 * Usage:
 *   npm run release:check          # pre-release validation only
 *   npm run release patch          # bump patch → commit → tag → push
 *   npm run release minor          # bump minor
 *   npm run release major          # bump major
 *   npm run release patch --dry-run
 */

import { spawnSync } from 'node:child_process'
import { bumpVersion, readVersion, writeVersion } from './lib/version.mjs'

const args = process.argv.slice(2)
const dryRun = args.includes('--dry-run')
const level = args.find((a) => ['patch', 'minor', 'major'].includes(a))

function run(cmd, { capture = false, allowFail = false } = {}) {
  const result = spawnSync(cmd, {
    stdio: capture ? 'pipe' : 'inherit',
    shell: true,
    encoding: 'utf8',
  })
  if (result.status !== 0 && !allowFail) {
    process.exit(result.status ?? 1)
  }
  return capture ? (result.stdout ?? '').trim() : ''
}

function header(title) {
  console.log(`\n▶ ${title}`)
}

function assertCleanTree() {
  header('Checking git working tree')
  const status = run('git status --porcelain', { capture: true })
  if (status) {
    console.error('\n✖ Working tree is not clean. Commit or stash changes before releasing.')
    process.exit(1)
  }
}

function assertBranch() {
  header('Checking branch')
  const branch = run('git branch --show-current', { capture: true })
  if (branch !== 'master') {
    console.error(`\n✖ Releases must be cut from master (current: ${branch}).`)
    process.exit(1)
  }
}

function assertGhAvailable() {
  header('Checking GitHub CLI')
  run('gh auth status', { allowFail: false })
}

function runChecks() {
  header('Lockfile engine compatibility')
  run('npm run check:engines')

  header('Frontend typecheck + build')
  run('npm run build:fe')

  header('Frontend tests')
  run('npm test')

  header('Frontend lint')
  run('npm run lint')

  header('Frontend format')
  run('npm run format:check')

  header('Rust format')
  const cargo = spawnSync('cargo', ['fmt', '--check'], {
    cwd: 'src-tauri',
    stdio: 'inherit',
    shell: true,
  })
  if (cargo.status !== 0) {
    console.error('\n✖ cargo fmt --check failed. Run: cd src-tauri && cargo fmt')
    process.exit(1)
  }
}

function previewNotes(tag) {
  header('Release notes preview')
  try {
    const prevTag = run(`git describe --tags --abbrev=0 ${tag}^`, { capture: true, allowFail: true })
    if (!prevTag) {
      console.log('(No previous tag — notes will be generated on CI)')
      return
    }
    const log = run(`git log ${prevTag}..HEAD --pretty=format:%s`, { capture: true })
    const lines = log.split('\n').filter(Boolean)
    console.log(`Commits since ${prevTag} (${lines.length}):`)
    for (const line of lines) {
      if (/^chore\(release\):/i.test(line)) continue
      console.log(`  • ${line}`)
    }
    console.log(`\nCI will format these into ✨ New / 🛠 Fixes / 🧹 Improvements sections.`)
    console.log(`Compare: https://github.com/AomeNero/marker/compare/${prevTag}...${tag}`)
  } catch {
    console.log('(Could not preview — notes will be generated on CI)')
  }
}

function release() {
  if (!level) {
    console.error(`
Usage:
  npm run release:check
  npm run release patch [--dry-run]
  npm run release minor [--dry-run]
  npm run release major [--dry-run]
`)
    process.exit(1)
  }

  assertCleanTree()
  assertBranch()
  assertGhAvailable()
  runChecks()

  const current = readVersion()
  const next = bumpVersion(current, level)
  const tag = `v${next}`

  console.log(`\nCurrent version: ${current}`)
  console.log(`Next version:    ${next}`)
  console.log(`Tag:             ${tag}`)

  previewNotes(tag)

  if (dryRun) {
    console.log('\n✔ Dry run complete — no files changed, no git operations performed.')
    return
  }

  header('Bumping version files')
  writeVersion(next)

  header('Creating release commit')
  run('git add package.json package-lock.json src-tauri/Cargo.toml src-tauri/Cargo.lock')
  run(`git commit -m "chore(release): ${tag}"`)

  header('Creating tag')
  run(`git tag ${tag}`)

  header('Pushing to origin')
  run('git push origin master')
  run(`git push origin ${tag}`)

  console.log(`
✔ Release ${tag} started.

  Release page : https://github.com/AomeNero/marker/releases/tag/${tag}
  Actions      : https://github.com/AomeNero/marker/actions/workflows/release.yml

Notes are generated automatically by .github/workflows/release.yml
using .github/release.yml categories + conventional commit fallback.
`)
}

if (args.includes('--check') || args[0] === 'check') {
  assertCleanTree()
  assertBranch()
  runChecks()
  previewNotes(`v${readVersion()}`)
  console.log('\n✔ release:check passed — safe to run npm run release patch|minor|major')
} else {
  release()
}
