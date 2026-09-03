import type { DrawAction, Point, Tool, TextOutlineStyle } from '../composables/drawingTypes'

/**
 * `.marker` file format — version 1.
 *
 * A file holds one slice of DrawActions per screen, in that screen's logical
 * overlay pixels, plus the screen identity needed to route slices back on
 * load: physical work-area geometry + OS monitor name (geometry matches
 * first; Windows renumbers names across replug, mirroring MonitorSpec).
 */
export const MARKER_FILE_VERSION = 1
export const MARKER_FILE_EXTENSION = 'marker'

const KNOWN_TOOLS: readonly Tool[] = [
  'select',
  'pen',
  'highlighter',
  'laser',
  'arrow',
  'rect',
  'ellipse',
  'line',
  'eraser',
  'text',
  'stamp',
]

/** Transient tools excluded from files — laser trails decay in real time. */
const UNPERSISTED_TOOLS: readonly Tool[] = ['laser']

/** Screen identity + slice as written to / read from files. */
export interface MarkerScreen {
  /** OS monitor name at save time (may be null). */
  name: string | null
  /** Physical work-area geometry + scale factor at save time. */
  x: number
  y: number
  width: number
  height: number
  scale: number
  actions: DrawAction[]
}

export interface MarkerFile {
  version: number
  /** App version that wrote the file (informational). */
  app: string
  /** ISO timestamp. */
  savedAt: string
  screens: MarkerScreen[]
}

export type MarkerFileErrorCode = 'unreadable' | 'unsupported-version'

export class MarkerFileError extends Error {
  readonly code: MarkerFileErrorCode

  constructor(code: MarkerFileErrorCode, message: string) {
    super(message)
    this.name = 'MarkerFileError'
    this.code = code
  }
}

// ---------------------------------------------------------------------------
// Serialization
// ---------------------------------------------------------------------------

/** Deep-copy points (and nested eraser records) so files never alias live state. */
function copyAction(action: DrawAction): DrawAction {
  const copy: DrawAction = { ...action, points: action.points.map((point) => ({ ...point })) }
  if (action.attachedErasers) {
    copy.attachedErasers = action.attachedErasers.map(copyAction)
  }
  return copy
}

/** Strip transient strokes and detach from live drawing state before writing. */
export function persistableActions(actions: readonly DrawAction[]): DrawAction[] {
  return actions
    .filter((action) => !UNPERSISTED_TOOLS.includes(action.tool))
    .map(copyAction)
}

export function serializeAnnotationFile(
  screens: readonly MarkerScreen[],
  app: string,
  savedAt = new Date().toISOString(),
): string {
  const file: MarkerFile = {
    version: MARKER_FILE_VERSION,
    app,
    savedAt,
    screens: screens.map((screen) => ({
      ...screen,
      actions: persistableActions(screen.actions),
    })),
  }
  return JSON.stringify(file)
}

/** `marker20260903153000.marker` — the auto-save naming scheme. */
export function annotationFileName(date: Date): string {
  const pad = (n: number) => String(n).padStart(2, '0')
  const stamp =
    `${date.getFullYear()}${pad(date.getMonth() + 1)}${pad(date.getDate())}` +
    `${pad(date.getHours())}${pad(date.getMinutes())}${pad(date.getSeconds())}`
  return `marker${stamp}.${MARKER_FILE_EXTENSION}`
}

// ---------------------------------------------------------------------------
// Parsing + validation
// ---------------------------------------------------------------------------

const finite = (v: unknown): v is number => typeof v === 'number' && Number.isFinite(v)

function parsePoint(raw: unknown): Point | null {
  if (typeof raw !== 'object' || raw === null) return null
  const p = raw as Record<string, unknown>
  if (!finite(p.x) || !finite(p.y)) return null
  const point: Point = { x: p.x, y: p.y }
  if (finite(p.pressure)) point.pressure = p.pressure
  return point
}

function parseTextOutline(raw: unknown): TextOutlineStyle | undefined {
  if (typeof raw !== 'object' || raw === null) return undefined
  const o = raw as Record<string, unknown>
  if (typeof o.enabled !== 'boolean' || typeof o.color !== 'string') return undefined
  if (o.colorMode !== 'auto' && o.colorMode !== 'fixed') return undefined
  if (!finite(o.width)) return undefined
  return { enabled: o.enabled, colorMode: o.colorMode, color: o.color, width: o.width }
}

interface Box {
  x1: number
  y1: number
  x2: number
  y2: number
}

function parseBox(raw: unknown): Box | undefined {
  if (typeof raw !== 'object' || raw === null) return undefined
  const b = raw as Record<string, unknown>
  if (!finite(b.x1) || !finite(b.y1) || !finite(b.x2) || !finite(b.y2)) return undefined
  return { x1: b.x1, y1: b.y1, x2: b.x2, y2: b.y2 }
}

function parseRectHit(raw: unknown): DrawAction['rectHit'] | undefined {
  if (typeof raw !== 'object' || raw === null) return undefined
  const r = raw as Record<string, unknown>
  if (!finite(r.x0) || !finite(r.y0) || !finite(r.x1) || !finite(r.y1)) return undefined
  return { x0: r.x0, y0: r.y0, x1: r.x1, y1: r.y1 }
}

interface EllipseHit {
  cx: number
  cy: number
  rx: number
  ry: number
}

function parseEllipseHit(raw: unknown): EllipseHit | undefined {
  if (typeof raw !== 'object' || raw === null) return undefined
  const e = raw as Record<string, unknown>
  if (!finite(e.cx) || !finite(e.cy) || !finite(e.rx) || !finite(e.ry)) return undefined
  return { cx: e.cx, cy: e.cy, rx: e.rx, ry: e.ry }
}

function parseAction(raw: unknown): DrawAction | null {
  if (typeof raw !== 'object' || raw === null) return null
  const a = raw as Record<string, unknown>
  if (!KNOWN_TOOLS.includes(a.tool as Tool)) return null
  const tool = a.tool as Tool
  if (UNPERSISTED_TOOLS.includes(tool)) return null
  if (typeof a.color !== 'string' || !finite(a.lineWidth) || a.lineWidth <= 0) return null
  if (!finite(a.opacity) || a.opacity < 0 || a.opacity > 1) return null
  if (!Array.isArray(a.points) || a.points.length === 0) return null
  const points: Point[] = []
  for (const rawPoint of a.points) {
    const point = parsePoint(rawPoint)
    if (!point) return null
    points.push(point)
  }

  const action: DrawAction = { tool, color: a.color, lineWidth: a.lineWidth, opacity: a.opacity, points }
  if (typeof a.pointerType === 'string') action.pointerType = a.pointerType
  if (typeof a.text === 'string') action.text = a.text
  if (finite(a.fontSize)) action.fontSize = a.fontSize
  if (finite(a.textWidth)) action.textWidth = a.textWidth
  const outline = parseTextOutline(a.textOutline)
  if (outline) action.textOutline = outline
  const bbox = parseBox(a.bbox)
  if (bbox) action.bbox = bbox
  const rectHit = parseRectHit(a.rectHit)
  if (rectHit) action.rectHit = rectHit
  const ellipseHit = parseEllipseHit(a.ellipseHit)
  if (ellipseHit) action.ellipseHit = ellipseHit
  if (Array.isArray(a.attachedErasers)) {
    const erasers: DrawAction[] = []
    for (const rawEraser of a.attachedErasers) {
      const eraser = parseAction(rawEraser)
      // Only eraser strokes may ride along as attached hit records.
      if (!eraser || eraser.tool !== 'eraser') return null
      erasers.push(eraser)
    }
    if (erasers.length > 0) action.attachedErasers = erasers
  }
  return action
}

function parseScreen(raw: unknown): MarkerScreen | null {
  if (typeof raw !== 'object' || raw === null) return null
  const s = raw as Record<string, unknown>
  if (!finite(s.x) || !finite(s.y) || !finite(s.width) || !finite(s.height) || !finite(s.scale)) {
    return null
  }
  if (s.scale <= 0) return null
  if (s.name !== null && typeof s.name !== 'string') return null
  if (!Array.isArray(s.actions)) return null
  const actions: DrawAction[] = []
  for (const rawAction of s.actions) {
    const action = parseAction(rawAction)
    if (!action) return null
    actions.push(action)
  }
  return {
    name: s.name,
    x: s.x,
    y: s.y,
    width: s.width,
    height: s.height,
    scale: s.scale,
    actions,
  }
}

/**
 * Parse and fully validate a `.marker` payload. Invalid actions/screens are
 * rejected with `unreadable` rather than silently repaired — a half-restored
 * board is worse than a clear error. Unknown tools inside a v1 file also
 * count as unreadable (the version gate is the forward-compat mechanism).
 */
export function parseAnnotationFile(json: string): MarkerFile {
  let raw: unknown
  try {
    raw = JSON.parse(json)
  } catch {
    throw new MarkerFileError('unreadable', 'not valid JSON')
  }
  if (typeof raw !== 'object' || raw === null || Array.isArray(raw)) {
    throw new MarkerFileError('unreadable', 'not an object')
  }
  const obj = raw as Record<string, unknown>
  if (obj.version !== MARKER_FILE_VERSION) {
    throw new MarkerFileError('unsupported-version', `version ${String(obj.version)}`)
  }
  if (!Array.isArray(obj.screens)) {
    throw new MarkerFileError('unreadable', 'screens missing')
  }
  const screens: MarkerScreen[] = []
  for (const rawScreen of obj.screens) {
    const screen = parseScreen(rawScreen)
    if (!screen) throw new MarkerFileError('unreadable', 'invalid screen')
    screens.push(screen)
  }
  return {
    version: MARKER_FILE_VERSION,
    app: typeof obj.app === 'string' ? obj.app : '',
    savedAt: typeof obj.savedAt === 'string' ? obj.savedAt : '',
    screens,
  }
}

// ---------------------------------------------------------------------------
// Load routing: match file screens to live overlay windows
// ---------------------------------------------------------------------------

/** A live overlay window's screen, as reported by the backend registry. */
export interface LocalScreen {
  label: string
  /** Serves the cursor/primary screen — fallback target for unmatched slices. */
  primary: boolean
  name: string | null
  x: number
  y: number
  width: number
  height: number
  scale: number
}

/** Actions routed to one overlay window (possibly empty). */
export interface LoadAssignment {
  label: string
  actions: DrawAction[]
  /** Actions whose points were clamped into the target screen. */
  clampedCount: number
}

export interface LoadPlan {
  assignments: LoadAssignment[]
  /** Unmatched screens' content, moved to the primary screen. */
  fallback: LoadAssignment | null
  /** File screens with no matching live screen (their content went to fallback). */
  missingScreens: number
}

/** Mirrors MonitorSpec::same_geometry — position, size, rounded scale. */
function sameGeometry(a: MarkerScreen, b: LocalScreen): boolean {
  return (
    a.x === b.x &&
    a.y === b.y &&
    a.width === b.width &&
    a.height === b.height &&
    Math.round(a.scale * 100) === Math.round(b.scale * 100)
  )
}

function nameEquals(a: MarkerScreen, b: LocalScreen): boolean {
  return a.name !== null && b.name !== null && a.name === b.name
}

function clamp(v: number, max: number): number {
  return Math.min(Math.max(v, 0), max)
}

/** Clamp an action's geometry into a target screen's logical size (in place). */
function clampAction(action: DrawAction, maxW: number, maxH: number): boolean {
  let clamped = false
  for (const point of action.points) {
    if (point.x < 0 || point.x > maxW) {
      point.x = clamp(point.x, maxW)
      clamped = true
    }
    if (point.y < 0 || point.y > maxH) {
      point.y = clamp(point.y, maxH)
      clamped = true
    }
  }
  if (action.bbox) {
    action.bbox.x1 = clamp(action.bbox.x1, maxW)
    action.bbox.x2 = clamp(action.bbox.x2, maxW)
    action.bbox.y1 = clamp(action.bbox.y1, maxH)
    action.bbox.y2 = clamp(action.bbox.y2, maxH)
    clamped = true
  }
  if (action.rectHit) {
    action.rectHit.x0 = clamp(action.rectHit.x0, maxW)
    action.rectHit.x1 = clamp(action.rectHit.x1, maxW)
    action.rectHit.y0 = clamp(action.rectHit.y0, maxH)
    action.rectHit.y1 = clamp(action.rectHit.y1, maxH)
    clamped = true
  }
  if (action.ellipseHit) {
    action.ellipseHit.cx = clamp(action.ellipseHit.cx, maxW)
    action.ellipseHit.cy = clamp(action.ellipseHit.cy, maxH)
    clamped = true
  }
  return clamped
}

function clampActions(actions: DrawAction[], screen: LocalScreen): number {
  const maxW = screen.width / screen.scale
  const maxH = screen.height / screen.scale
  let clamped = 0
  for (const action of actions) {
    if (clampAction(action, maxW, maxH)) clamped += 1
  }
  return clamped
}

/**
 * Route file screens onto live overlay windows. Matching mirrors the app's
 * MonitorSpec pairing rule: geometry first, OS name second. Screens with no
 * live counterpart move — in file order — onto the primary screen, clamped
 * inside its bounds; data is never dropped.
 */
export function planAnnotationLoad(
  screens: readonly MarkerScreen[],
  local: readonly LocalScreen[],
): LoadPlan {
  if (local.length === 0) {
    return { assignments: [], fallback: null, missingScreens: screens.length }
  }
  const primary = local.find((screen) => screen.primary) ?? local[0]
  const used = new Set<string>()
  const matched = new Array<boolean>(screens.length).fill(false)
  const assignments: LoadAssignment[] = local.map((screen) => ({ label: screen.label, actions: [], clampedCount: 0 }))
  const byLabel = new Map(assignments.map((assignment) => [assignment.label, assignment]))
  const unmatched: DrawAction[] = []
  let missingScreens = 0

  // Pass 1: geometry, pass 2: name — same preference order as MonitorSpec.
  for (const pass of ['geometry', 'name'] as const) {
    screens.forEach((screen, index) => {
      if (matched[index]) return
      const hit = local.find(
        (candidate) =>
          !used.has(candidate.label) &&
          (pass === 'geometry' ? sameGeometry(screen, candidate) : nameEquals(screen, candidate)),
      )
      if (!hit) return
      matched[index] = true
      used.add(hit.label)
      const assignment = byLabel.get(hit.label)!
      assignment.actions.push(...screen.actions)
      assignment.clampedCount += clampActions(screen.actions, hit)
    })
  }

  screens.forEach((screen, index) => {
    if (matched[index]) return
    missingScreens += 1
    unmatched.push(...screen.actions)
  })

  const fallbackAssignment = byLabel.get(primary.label)!
  if (unmatched.length > 0) {
    fallbackAssignment.clampedCount += clampActions(unmatched, primary)
  }
  return {
    assignments,
    fallback: unmatched.length > 0 ? { label: primary.label, actions: unmatched, clampedCount: fallbackAssignment.clampedCount } : null,
    missingScreens,
  }
}
