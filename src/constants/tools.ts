import type { Component } from 'vue'
import { ref } from 'vue'
import {
  SquareDashedMousePointer,
  Pen,
  Highlighter,
  Crosshair,
  ArrowUpRight,
  Square,
  Circle,
  Minus,
  Eraser,
  Type,
  ListOrdered,
} from '@lucide/vue'
import type { Tool } from '../composables/drawingTypes'

export interface ToolDef {
  id: Tool
  icon: Component
  key: string
}

/** Select lives in the top action row; drawing tools use the grid below. */
export const SELECT_TOOL_DEF: ToolDef = { id: 'select', icon: SquareDashedMousePointer, key: 'V' }

export const TOOL_DEFS: ToolDef[] = [
  SELECT_TOOL_DEF,
  { id: 'pen', icon: Pen, key: '1' },
  { id: 'highlighter', icon: Highlighter, key: '2' },
  { id: 'arrow', icon: ArrowUpRight, key: '3' },
  { id: 'rect', icon: Square, key: '4' },
  { id: 'ellipse', icon: Circle, key: '5' },
  { id: 'line', icon: Minus, key: '6' },
  { id: 'eraser', icon: Eraser, key: '7' },
  { id: 'laser', icon: Crosshair, key: '8' },
  { id: 'text', icon: Type, key: 'T' },
  { id: 'stamp', icon: ListOrdered, key: 'N' },
]

/** Tools shown in the toolbar grid (excludes select). */
export const DRAWING_TOOL_DEFS: ToolDef[] = TOOL_DEFS.filter((d) => d.id !== 'select')

/** Pen-family tools collapsed into one toolbar button; shows the last used. */
export const PEN_GROUP_TOOLS: Tool[] = ['pen', 'highlighter', 'laser']
export const PEN_GROUP_DEFAULT: Tool = 'pen'

/** Shape tools collapsed into one toolbar button; shows the last used. */
export const SHAPE_GROUP_TOOLS: Tool[] = ['arrow', 'rect', 'ellipse', 'line', 'stamp']
export const SHAPE_GROUP_DEFAULT: Tool = 'ellipse'

export type ToolbarGroupId = 'pen' | 'shape'

/** Which collapsed group a tool belongs to (null = standalone button). */
export function toolbarGroupOf(tool: Tool): ToolbarGroupId | null {
  if (PEN_GROUP_TOOLS.includes(tool)) return 'pen'
  if (SHAPE_GROUP_TOOLS.includes(tool)) return 'shape'
  return null
}

export function toolDefOf(tool: Tool): ToolDef {
  return TOOL_DEFS.find((d) => d.id === tool) ?? SELECT_TOOL_DEF
}

export const TOOL_ICON_MAP: Record<Tool, Component> = Object.fromEntries(
  TOOL_DEFS.map((d) => [d.id, d.icon]),
) as Record<Tool, Component>

/** Default stroke-width presets (XS/S/M/L/XL); user-configurable via Settings. */
export const DEFAULT_WIDTH_PRESETS: number[] = [2, 4, 6, 10, 16]

/** Positional labels for the five width preset buttons. */
export const WIDTH_PRESET_LABEL_KEYS = ['xs', 's', 'm', 'l', 'xl'] as const

const widthPresetsState = ref<number[]>(DEFAULT_WIDTH_PRESETS.slice())

/** Validate a persisted preset array; anything else falls back to the defaults. */
export function resolveWidthPresets(value: unknown): number[] {
  if (
    Array.isArray(value) &&
    value.length === 5 &&
    value.every((v) => typeof v === 'number' && Number.isFinite(v) && v >= 1 && v <= 100)
  ) {
    return value.slice()
  }
  return DEFAULT_WIDTH_PRESETS.slice()
}

/** Seed the active preset set from config (overlay + toolbar windows each call this). */
export function setWidthPresets(value: unknown): void {
  widthPresetsState.value = resolveWidthPresets(value)
}

/** Active presets — reactive so toolbar buttons and Ctrl+wheel follow config changes. */
export function getWidthPresets(): number[] {
  return widthPresetsState.value
}

/** Middle preset (M) — default line width for every width group. */
export function defaultLineWidth(): number {
  return widthPresetsState.value[2] ?? DEFAULT_WIDTH_PRESETS[2]
}

/** Pen + shapes share one preset; highlighter / eraser / text are separate. */
export type LineWidthGroup = 'stroke' | 'highlighter' | 'eraser' | 'text'

export interface ToolLineWidths {
  stroke: number
  highlighter: number
  eraser: number
  text: number
}

const STROKE_TOOLS = new Set<Tool>(['pen', 'laser', 'arrow', 'rect', 'ellipse', 'line'])

export function toolLineWidthGroup(tool: Tool): LineWidthGroup {
  if (tool === 'highlighter') return 'highlighter'
  if (tool === 'eraser') return 'eraser'
  if (tool === 'text' || tool === 'stamp') return 'text'
  // select shares stroke group (width UI unused while select is active)
  return 'stroke'
}

export function isStrokeTool(tool: Tool): boolean {
  return STROKE_TOOLS.has(tool)
}

export function createDefaultLineWidths(): ToolLineWidths {
  const w = defaultLineWidth()
  return {
    stroke: w,
    highlighter: w,
    eraser: w,
    text: w,
  }
}

/** Snap a width to the closest active preset (ties prefer the larger preset). */
export function normalizeLineWidth(value: unknown): number {
  if (typeof value !== 'number' || !Number.isFinite(value)) return defaultLineWidth()
  let best = defaultLineWidth()
  let bestDistance = Infinity
  for (const preset of getWidthPresets()) {
    const distance = Math.abs(preset - value)
    if (distance < bestDistance || (distance === bestDistance && preset > best)) {
      best = preset
      bestDistance = distance
    }
  }
  return best
}

/** Resolve persisted line widths; missing/invalid values fall back to defaults. */
export function resolveLineWidths(partial?: Partial<ToolLineWidths> | null): ToolLineWidths {
  const defaults = createDefaultLineWidths()
  if (!partial) return defaults
  return {
    stroke: normalizeLineWidth(partial.stroke ?? defaults.stroke),
    highlighter: normalizeLineWidth(partial.highlighter ?? defaults.highlighter),
    eraser: normalizeLineWidth(partial.eraser ?? defaults.eraser),
    text: normalizeLineWidth(partial.text ?? defaults.text),
  }
}

/** Highlighter stroke width = lineWidth × scale (default 3 → 21px). */
export const HIGHLIGHTER_WIDTH_SCALE = 7

export function highlighterLineWidth(lineWidth: number): number {
  return lineWidth * HIGHLIGHTER_WIDTH_SCALE
}

/** Eraser stroke width = lineWidth × scale (default 3 → 24px, close to legacy 25px). */
export const ERASER_WIDTH_SCALE = 8

export function eraserLineWidth(lineWidth: number): number {
  return lineWidth * ERASER_WIDTH_SCALE
}
