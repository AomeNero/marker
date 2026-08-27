#!/usr/bin/env node
/**
 * Merge Tauri updater JSON fragments (from parallel CI matrix jobs) into one latest.json.
 *
 * Usage:
 *   node scripts/merge-updater-json.mjs <fragments-dir> <output-file>
 */

import { readFileSync, writeFileSync, readdirSync, statSync } from 'node:fs'
import { join, resolve } from 'node:path'

const [fragmentsDir, outputFile] = process.argv.slice(2)
if (!fragmentsDir || !outputFile) {
  console.error('Usage: node scripts/merge-updater-json.mjs <fragments-dir> <output-file>')
  process.exit(1)
}

function collectJsonFiles(dir) {
  const abs = resolve(dir)
  const entries = readdirSync(abs)
  const files = []
  for (const entry of entries) {
    const path = join(abs, entry)
    const stat = statSync(path)
    if (stat.isDirectory()) {
      files.push(...collectJsonFiles(path))
    } else if (entry.endsWith('.json')) {
      files.push(path)
    }
  }
  return files
}

const files = collectJsonFiles(fragmentsDir)
if (!files.length) {
  console.error(`No JSON fragments found under ${fragmentsDir}`)
  process.exit(1)
}

/** @type {{ version: string, notes: string, pub_date: string, platforms: Record<string, unknown> }} */
const merged = { version: '', notes: '', pub_date: '', platforms: {} }

for (const file of files) {
  const data = JSON.parse(readFileSync(file, 'utf8'))
  if (!data.platforms || typeof data.platforms !== 'object') {
    console.warn(`Skipping ${file}: missing platforms`)
    continue
  }
  if (data.version) merged.version = data.version
  if (data.notes && data.notes.length >= merged.notes.length) merged.notes = data.notes
  if (data.pub_date && data.pub_date >= merged.pub_date) merged.pub_date = data.pub_date
  Object.assign(merged.platforms, data.platforms)
}

if (!merged.version || !Object.keys(merged.platforms).length) {
  console.error('Merged updater JSON is empty or invalid')
  process.exit(1)
}

writeFileSync(resolve(outputFile), `${JSON.stringify(merged, null, 2)}\n`, 'utf8')
console.log(`Wrote ${outputFile} with platforms: ${Object.keys(merged.platforms).join(', ')}`)
