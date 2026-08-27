#!/usr/bin/env node
/**
 * Verify package-lock.json dependency engine ranges against the pinned Node version.
 * Runs without node_modules so CI can use it before npm ci.
 * Unknown/complex semver patterns are skipped (no false positives).
 */

import { readFileSync } from 'node:fs'
import { join, dirname } from 'node:path'
import { fileURLToPath } from 'node:url'

const ROOT = join(dirname(fileURLToPath(import.meta.url)), '..')
const pkg = JSON.parse(readFileSync(join(ROOT, 'package.json'), 'utf8'))
const lock = JSON.parse(readFileSync(join(ROOT, 'package-lock.json'), 'utf8'))

const pinnedNode = pkg.engines?.node
if (!pinnedNode) process.exit(0)

function parseVersion(version) {
  const cleaned = String(version).replace(/^v/, '').trim()
  const parts = cleaned.split('.').map((p) => Number(p))
  if (!parts.length || parts.some((n) => Number.isNaN(n))) return null
  return {
    major: parts[0],
    minor: parts[1] ?? 0,
    patch: parts[2] ?? 0,
  }
}

function compare(a, b) {
  if (a.major !== b.major) return a.major - b.major
  if (a.minor !== b.minor) return a.minor - b.minor
  return a.patch - b.patch
}

function satisfiesSingle(version, range) {
  range = range.trim()

  if (/^\d+$/.test(range)) {
    return version.major === Number(range)
  }

  if (range.startsWith('>=')) {
    const min = parseVersion(range.slice(2).trim())
    if (!min) return true
    return compare(version, min) >= 0
  }

  if (range.startsWith('^')) {
    const base = parseVersion(range.slice(1).trim())
    if (!base) return true
    if (base.major > 0) {
      return version.major === base.major && compare(version, base) >= 0
    }
    if (base.minor > 0) {
      return version.major === 0 && version.minor === base.minor && compare(version, base) >= 0
    }
    return version.major === 0 && version.minor === 0 && compare(version, base) >= 0
  }

  return true
}

function satisfies(version, range) {
  return range.split('||').some((part) => satisfiesSingle(version, part))
}

const nodeVersion = parseVersion(pinnedNode)
if (!nodeVersion) {
  console.error(`✖ Invalid engines.node in package.json: ${pinnedNode}`)
  process.exit(1)
}

const versionLabel = `${nodeVersion.major}.${nodeVersion.minor}.${nodeVersion.patch}`
const failures = []

for (const [pkgPath, meta] of Object.entries(lock.packages ?? {})) {
  const range = meta?.engines?.node
  if (!range || typeof range !== 'string') continue
  if (!satisfies(nodeVersion, range)) {
    const name = pkgPath.replace(/^node_modules\//, '') || lock.name
    failures.push({ name, range })
  }
}

if (failures.length === 0) process.exit(0)

console.error(`\n✖ Pinned Node ${versionLabel} does not satisfy lockfile engine requirements:\n`)
for (const { name, range } of failures.sort((a, b) => a.name.localeCompare(b.name))) {
  console.error(`  • ${name}: requires node ${range}`)
}
console.error(`
Bump engines.node / .nvmrc / .node-version together, then re-run:

  npm run check:engines
`)
process.exit(1)
