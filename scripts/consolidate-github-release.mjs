#!/usr/bin/env node
/**
 * Consolidate duplicate GitHub releases for one tag, upload merged latest.json, and publish.
 *
 * Usage:
 *   node scripts/consolidate-github-release.mjs <tag> <merged-latest.json>
 */

import { spawnSync } from 'node:child_process'
import { copyFileSync, createWriteStream, mkdtempSync, rmSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { pipeline } from 'node:stream/promises'

const [tag, mergedLatestPath] = process.argv.slice(2)
if (!tag || !mergedLatestPath) {
  console.error('Usage: node scripts/consolidate-github-release.mjs <tag> <merged-latest.json>')
  process.exit(1)
}

const repo = process.env.GH_REPO || 'ifer47/marker'
const GH_MAX_BUFFER = 10 * 1024 * 1024

function gh(args, { json = true, allowFail = false } = {}) {
  const result = spawnSync('gh', args, {
    encoding: 'utf8',
    env: process.env,
    maxBuffer: GH_MAX_BUFFER,
  })
  if (result.status !== 0 && !allowFail) {
    const err = (result.stderr || result.stdout || '').trim()
    throw new Error(`gh ${args.join(' ')} failed: ${err}`)
  }
  if (!json) return result.stdout ?? ''
  const out = (result.stdout ?? '').trim()
  return out ? JSON.parse(out) : null
}

function listReleasesForTag(tagName) {
  const result = spawnSync(
    'gh',
    [
      'api',
      `repos/${repo}/releases`,
      '--paginate',
      '--jq',
      `.[] | select(.tag_name == "${tagName}")`,
    ],
    {
      encoding: 'utf8',
      env: process.env,
      maxBuffer: GH_MAX_BUFFER,
    },
  )
  if (result.status !== 0) {
    const err = (result.stderr || result.stdout || '').trim()
    throw new Error(`gh api releases (tag=${tagName}) failed: ${err}`)
  }

  return (result.stdout ?? '')
    .trim()
    .split('\n')
    .filter(Boolean)
    .map((line) => JSON.parse(line))
}

function assetDownloadUrl(asset) {
  // Draft releases expose browser_download_url with untagged-* slugs that 404.
  // The API asset URL works with GITHUB_TOKEN + Accept: application/octet-stream.
  return asset.url || asset.browser_download_url
}

async function downloadAsset(asset, dest) {
  const token = process.env.GITHUB_TOKEN || process.env.GH_TOKEN
  const url = assetDownloadUrl(asset)
  const res = await fetch(url, {
    headers: token ? { Authorization: `Bearer ${token}`, Accept: 'application/octet-stream' } : {},
    redirect: 'follow',
  })
  if (!res.ok) throw new Error(`Download failed ${res.status}: ${url}`)
  await pipeline(res.body, createWriteStream(dest))
}

const releases = listReleasesForTag(tag)
if (!releases.length) {
  console.error(`No releases found for tag ${tag}`)
  process.exit(1)
}

releases.sort((a, b) => {
  if (a.draft !== b.draft) return a.draft ? 1 : -1
  return b.assets.length - a.assets.length
})

const primary = releases[0]
const duplicates = releases.slice(1)
console.log(
  `Primary release #${primary.id} (${primary.draft ? 'draft' : 'published'}, ${primary.assets.length} assets)`,
)

const primaryNames = new Set(primary.assets.map((a) => a.name))
const tmp = mkdtempSync(join(tmpdir(), 'marker-release-'))

try {
  for (const release of duplicates) {
    console.log(`Merging assets from release #${release.id} (${release.assets.length} assets)`)
    for (const asset of release.assets) {
      if (asset.name === 'latest.json' || primaryNames.has(asset.name)) continue
      const dest = join(tmp, asset.name)
      await downloadAsset(asset, dest)
      gh(['release', 'upload', tag, dest, '--clobber'], { json: false })
      primaryNames.add(asset.name)
      console.log(`  uploaded ${asset.name}`)
    }
    console.log(`Deleting duplicate release #${release.id}`)
    gh(['api', '--method', 'DELETE', `repos/${repo}/releases/${release.id}`], { json: false })
  }

  console.log(`Uploading merged latest.json`)
  const latestUpload = join(tmp, 'latest.json')
  copyFileSync(mergedLatestPath, latestUpload)
  gh(['release', 'upload', tag, latestUpload, '--clobber'], { json: false })

  console.log(`Publishing ${tag}`)
  gh(['release', 'edit', tag, '--draft=false', '--latest'], { json: false })
  console.log(`Done: https://github.com/${repo}/releases/tag/${tag}`)
} finally {
  rmSync(tmp, { recursive: true, force: true })
}
