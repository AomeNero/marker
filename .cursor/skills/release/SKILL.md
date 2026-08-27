---
description: >-
  Marker release workflow. Use when the user asks to release, publish, bump
  version, create a tag, or cut a new version. Never release manually.
---

# Marker Release

## Single entry point

**Always use the release script — never manually bump version, commit, or tag.**

```bash
# 1. Pre-flight (optional, also runs inside release)
npm run release:check

# 2. Dry run
npm run release patch --dry-run

# 3. Publish (patch | minor | major)
npm run release patch
```

The script automatically:

1. Asserts clean git tree + `master` branch
2. Runs the same checks as CI (`build:fe`, test, lint, format, `cargo fmt --check`)
3. Bumps `package.json`, `package-lock.json`, `Cargo.toml`, `Cargo.lock`
4. Commits `chore(release): vX.Y.Z`
5. Tags `vX.Y.Z` and pushes branch + tag
6. Triggers GitHub Actions Release workflow (multi-platform build + upload)

## Release notes format (automatic)

Configured in:

| File | Role |
|------|------|
| `.github/release.yml` | Category titles (✨ New, 🛠 Fixes, …) for GitHub API |
| `.github/workflows/release.yml` | Heading normalization + git-log fallback when no PR notes |

Output format:

```markdown
## What's New in vX.Y.Z

### ✨ New
- ...

### 🛠 Fixes
- ...

### 🧹 Improvements
- ...

**Full Changelog**: https://github.com/ifer47/marker/compare/vA.B.C...vX.Y.Z
```

Release notes are derived from **Conventional Commits** since the previous tag. Use `npm run commit` (czg) for daily commits so notes categorize correctly.

## Agent rules

- Do **not** hand-edit version in four files — use `npm run release`
- Do **not** `git tag` / `git push` tags manually unless fixing a failed release
- Do **not** write release notes by hand unless user explicitly asks to edit after publish
- Pick bump level: `patch` (fixes), `minor` (features), `major` (breaking)
- Requires `gh` CLI authenticated (`gh auth login`)

## After release

Monitor: https://github.com/ifer47/marker/actions/workflows/release.yml

If release notes job fails, fix `.github/workflows/release.yml` on `master`, delete + recreate tag (see skill `cross-platform-tauri-ui` not needed — use release workflow fix + retag).

## Prerequisites

- Node >= 20, Rust stable, `gh` CLI
- On Windows: no bash required for release (Node script is cross-platform)
