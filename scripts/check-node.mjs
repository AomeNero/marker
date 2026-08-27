#!/usr/bin/env node
/**
 * Enforce project Node.js and npm versions (see package.json engines).
 * Runs automatically on npm install via preinstall.
 */

import { readFileSync } from 'node:fs'
import { spawnSync } from 'node:child_process'
import { join, dirname } from 'node:path'
import { fileURLToPath } from 'node:url'

const ROOT = join(dirname(fileURLToPath(import.meta.url)), '..')
const pkg = JSON.parse(readFileSync(join(ROOT, 'package.json'), 'utf8'))
const requiredNode = pkg.engines?.node
const requiredNpm = pkg.engines?.npm

if (!requiredNode || !requiredNpm) {
  process.exit(0)
}

function parseNode(version) {
  const match = version.replace(/^v/, '').match(/^(\d+)\.(\d+)\.(\d+)/)
  if (!match) return null
  return { major: +match[1], minor: +match[2], patch: +match[3] }
}

function parseSemver(version) {
  const match = version.match(/^(\d+)\.(\d+)\.(\d+)/)
  if (!match) return null
  return { major: +match[1], minor: +match[2], patch: +match[3] }
}

function sameSemver(a, b) {
  return a.major === b.major && a.minor === b.minor && a.patch === b.patch
}

const currentNode = parseNode(process.version)
const requiredNodeSemver = parseSemver(requiredNode)

const npmResult = spawnSync('npm', ['-v'], { encoding: 'utf8', shell: true })
const currentNpm = parseSemver((npmResult.stdout ?? '').trim())
const requiredNpmSemver = parseSemver(requiredNpm)

const errors = []

if (!currentNode || !requiredNodeSemver || !sameSemver(currentNode, requiredNodeSemver)) {
  errors.push(`Node.js ${requiredNode} required (current: ${process.version})`)
}

if (!currentNpm || !requiredNpmSemver || !sameSemver(currentNpm, requiredNpmSemver)) {
  errors.push(`npm ${requiredNpm} required (current: ${npmResult.stdout?.trim() || 'unknown'})`)
}

if (errors.length === 0) {
  process.exit(0)
}

console.error('\n✖ Node.js / npm version mismatch\n')
for (const err of errors) console.error(`  • ${err}`)
console.error(`
Use the project-pinned toolchain:

  nvm install    # reads .nvmrc → ${requiredNode}
  nvm use
  npm -v         # should be ${requiredNpm}

Or with fnm:

  fnm use        # reads .node-version → ${requiredNode}

See CONTRIBUTING.md → Prerequisites.
`)
process.exit(1)
