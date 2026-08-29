import { describe, it, expect } from 'vitest'
import {
  createDefaultLineWidths,
  defaultLineWidth,
  eraserLineWidth,
  ERASER_WIDTH_SCALE,
  getWidthOptions,
  highlighterLineWidth,
  HIGHLIGHTER_WIDTH_SCALE,
  normalizeLineWidth,
  resolveLineWidths,
  resolveWidthPresets,
  SELECT_TOOL_DEF,
  setWidthPresets,
  TOOL_DEFS,
  toolLineWidthGroup,
} from './tools'

describe('tool shortcut keys', () => {
  it('maps tools to the 1-7 + letter layout', () => {
    expect(SELECT_TOOL_DEF.key).toBe('S')
    const keys = Object.fromEntries(TOOL_DEFS.map((d) => [d.id, d.key]))
    expect(keys).toEqual({
      select: 'S',
      pen: '1',
      highlighter: '2',
      laser: '3',
      arrow: '4',
      rect: '5',
      ellipse: '6',
      line: '7',
      eraser: 'E',
      text: 'T',
      stamp: 'N',
    })
  })
})

describe('line width helpers', () => {
  it('defaults every width group to the middle preset', () => {
    const widths = createDefaultLineWidths()
    for (const value of Object.values(widths)) {
      expect(value).toBe(defaultLineWidth())
    }
    expect(defaultLineWidth()).toBe(6)
  })

  it('maps stroke tools to shared group; others are separate', () => {
    expect(toolLineWidthGroup('select')).toBe('stroke')
    expect(toolLineWidthGroup('pen')).toBe('stroke')
    expect(toolLineWidthGroup('laser')).toBe('stroke')
    expect(toolLineWidthGroup('arrow')).toBe('stroke')
    expect(toolLineWidthGroup('rect')).toBe('stroke')
    expect(toolLineWidthGroup('highlighter')).toBe('highlighter')
    expect(toolLineWidthGroup('eraser')).toBe('eraser')
    expect(toolLineWidthGroup('text')).toBe('text')
    expect(toolLineWidthGroup('stamp')).toBe('text')
  })

  it('scales lineWidth for eraser and highlighter', () => {
    expect(ERASER_WIDTH_SCALE).toBe(8)
    expect(HIGHLIGHTER_WIDTH_SCALE).toBe(7)
    expect(eraserLineWidth(3)).toBe(24)
    expect(highlighterLineWidth(3)).toBe(21)
  })

  it('normalizes line width to the closest active preset', () => {
    // Default presets [2,4,6,10,16]: 5 is equidistant to 4/6 → larger wins.
    expect(normalizeLineWidth(5)).toBe(6)
    expect(normalizeLineWidth(12)).toBe(10)
    expect(normalizeLineWidth(0)).toBe(2)
    expect(normalizeLineWidth('3')).toBe(defaultLineWidth())
  })

  it('normalizes against custom presets after setWidthPresets', () => {
    setWidthPresets([1, 4, 7, 10, 13])
    try {
      expect(normalizeLineWidth(5)).toBe(4)
      expect(normalizeLineWidth(8)).toBe(7)
      expect(defaultLineWidth()).toBe(7)
    } finally {
      setWidthPresets(undefined)
    }
  })

  it('resolves persisted line widths with fallbacks', () => {
    expect(resolveLineWidths()).toEqual(createDefaultLineWidths())
    // 8 snaps to 10 (equidistant to 6/10 → larger); eraser 4 snaps within its
    // 3-step subset [2,6,16] to 6 (equidistant → larger).
    expect(resolveLineWidths({ stroke: 8, eraser: 4 })).toEqual({
      stroke: 10,
      highlighter: defaultLineWidth(),
      eraser: 6,
      text: defaultLineWidth(),
    })
  })

  it('gives the eraser a compact 3-step width picker', () => {
    const options = getWidthOptions('eraser')
    expect(options.map((o) => o.value)).toEqual([2, 6, 16])
    expect(options.map((o) => o.labelKey)).toEqual(['widths.xs', 'widths.m', 'widths.xl'])
    // Other tools keep the full 5-step set.
    expect(getWidthOptions('pen').map((o) => o.value)).toEqual([2, 4, 6, 10, 16])
    expect(getWidthOptions('text').map((o) => o.value)).toEqual([2, 4, 6, 10, 16])
  })

  it('resolves width presets with fallbacks for invalid arrays', () => {
    expect(resolveWidthPresets(undefined)).toEqual([2, 4, 6, 10, 16])
    expect(resolveWidthPresets([1, 2, 3])).toEqual([2, 4, 6, 10, 16])
    expect(resolveWidthPresets([1, 2, 3, 4, 500])).toEqual([2, 4, 6, 10, 16])
    expect(resolveWidthPresets([3, 6, 9, 12, 20])).toEqual([3, 6, 9, 12, 20])
  })
})
