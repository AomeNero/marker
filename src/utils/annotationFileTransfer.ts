import { getVersion } from '@tauri-apps/api/app'
import { invoke } from '@tauri-apps/api/core'
import { emit, emitTo, listen } from '@tauri-apps/api/event'
import type { DrawAction } from '../composables/drawingTypes'
import { parseAnnotationFile, planAnnotationLoad, serializeAnnotationFile, type LocalScreen } from './annotationFile'

/**
 * Frontend orchestrator for `.marker` save / load. Format and routing logic
 * live in `annotationFile.ts` (unit-tested, pure); this module wires them to
 * the running overlays, the file dialog, and the backend commands.
 *
 * All overlay webviews are pre-created at startup, so event fan-out reaches
 * every screen's strokes without waiting on window creation.
 */

/** Backend → every overlay: reply with this screen's strokes. */
export const ANNOTATIONS_EXPORT_REQUEST_EVENT = 'annotations-export-request'
/** Overlay → orchestrator: one screen's stroke slice (label identifies it). */
export const ANNOTATIONS_EXPORT_EVENT = 'annotations-export'
/** Orchestrator → one overlay: apply this slice (mode open | insert). */
export const ANNOTATIONS_APPLY_LOAD_EVENT = 'annotations-apply-load'
/** Tray/backend → primary overlay: run a file action (mirror of Rust side). */
export const ANNOTATIONS_FILE_REQUEST_EVENT = 'annotations-file-request'

export type LoadMode = 'open' | 'insert'

/** Mirror of the Rust `OverlayScreenSpec` DTO (camelCase on the wire). */
export type OverlayScreenSpec = LocalScreen

export interface ExportReply {
  label: string
  actions: DrawAction[]
}

export type FileActionResult =
  | { kind: 'cancelled' }
  | { kind: 'saved'; path: string }
  | { kind: 'loaded'; loadedCount: number; missingScreens: number }

const EXPORT_COLLECT_TIMEOUT_MS = 1500

async function getScreenSpecs(): Promise<OverlayScreenSpec[]> {
  return invoke<OverlayScreenSpec[]>('get_overlay_screen_specs')
}

/** Ask every overlay for its strokes and collect replies until quiet. */
async function collectScreenExports(specs: OverlayScreenSpec[]): Promise<Map<string, DrawAction[]>> {
  const labels = new Set(specs.map((spec) => spec.label))
  const replies = new Map<string, DrawAction[]>()
  const unlisten = await listen<ExportReply>(ANNOTATIONS_EXPORT_EVENT, (event) => {
    const { label, actions } = event.payload
    if (labels.has(label)) replies.set(label, actions)
  })
  try {
    await emit(ANNOTATIONS_EXPORT_REQUEST_EVENT)
    const deadline = new Date(Date.now() + EXPORT_COLLECT_TIMEOUT_MS)
    while (replies.size < labels.size && Date.now() < deadline.getTime()) {
      await new Promise((resolve) => setTimeout(resolve, 25))
    }
  } finally {
    unlisten()
  }
  return replies
}

/** Serialize every screen's strokes and write a timestamped `.marker` file. */
export async function saveAnnotationsToFile(): Promise<FileActionResult> {
  const specs = await getScreenSpecs()
  const replies = await collectScreenExports(specs)
  const screens = specs.map((spec) => ({
    name: spec.name,
    x: spec.x,
    y: spec.y,
    width: spec.width,
    height: spec.height,
    scale: spec.scale,
    actions: replies.get(spec.label) ?? [],
  }))
  const content = serializeAnnotationFile(screens, await getVersion())
  const path = await invoke<string>('save_annotations_file', { content })
  return { kind: 'saved', path }
}

/**
 * Load a `.marker` file (dialog, or an explicit path from file association)
 * and route each screen's slice onto its overlay. `open` replaces the board;
 * `insert` stacks on top of it. Throws `MarkerFileError` for unreadable or
 * unsupported-version payloads.
 */
export async function loadAnnotationsFile(mode: LoadMode, presetPath?: string): Promise<FileActionResult> {
  const payload = presetPath
    ? await invoke<{ path: string; content: string }>('read_annotations_file', { path: presetPath })
    : await invoke<{ path: string; content: string } | null>('pick_annotations_file')
  if (!payload) return { kind: 'cancelled' }

  const file = parseAnnotationFile(payload.content)
  const specs = await getScreenSpecs()
  const plan = planAnnotationLoad(file.screens, specs)

  let loadedCount = 0
  for (const assignment of plan.assignments) {
    loadedCount += assignment.actions.length
    // Every overlay gets a slice (possibly empty): each must push the
    // matching local undo entry so the global op's broadcast stays aligned.
    await emitTo(assignment.label, ANNOTATIONS_APPLY_LOAD_EVENT, {
      mode,
      actions: assignment.actions,
    })
  }
  await invoke('record_load_op', { mode })
  return { kind: 'loaded', loadedCount, missingScreens: plan.missingScreens }
}
