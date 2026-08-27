import raw from './afdian-sponsors.json'

export interface AfdianSponsorEntry {
  name: string
  amount: string
  plan: string
  remark: string
}

export interface AfdianSponsorsData {
  syncedAt: string | null
  sponsors: AfdianSponsorEntry[]
}

function normalizeSponsors(data: unknown): AfdianSponsorsData {
  if (!data || typeof data !== 'object') {
    return { syncedAt: null, sponsors: [] }
  }

  const record = data as Record<string, unknown>
  const syncedAt = typeof record.syncedAt === 'string' ? record.syncedAt : null

  if (Array.isArray(record.sponsors)) {
    const sponsors = record.sponsors
      .map((entry) => {
        if (!entry || typeof entry !== 'object') return null
        const item = entry as Record<string, unknown>
        const name = typeof item.name === 'string' ? item.name.trim() : ''
        if (!name) return null
        return {
          name,
          amount: typeof item.amount === 'string' ? item.amount : '',
          plan: typeof item.plan === 'string' ? item.plan : '',
          remark: typeof item.remark === 'string' ? item.remark : '',
        }
      })
      .filter((entry): entry is AfdianSponsorEntry => entry !== null)

    return { syncedAt, sponsors }
  }

  if (Array.isArray(record.names)) {
    const sponsors = record.names
      .filter((name): name is string => typeof name === 'string' && name.trim().length > 0)
      .map((name) => ({ name: name.trim(), amount: '', plan: '', remark: '' }))

    return { syncedAt, sponsors }
  }

  return { syncedAt, sponsors: [] }
}

const afdianSponsors = normalizeSponsors(raw)

export default afdianSponsors
