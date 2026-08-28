import { describe, it, expect } from 'vitest'
import {
  createDefaultLineWidths,
  defaultLineWidth,
  eraserLineWidth,
  ERASER_WIDTH_SCALE,
  highlighterLineWidth,
  HIGHLIGHTER_WIDTH_SCALE,
  normalizeLineWidth,
  resolveLineWidths,
  resolveWidthPresets,
  setWidthPresets,
  toolLineWidthGroup,
} from './tools'

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
    // 8 snaps to 10 (equidistant to 6/10 → larger); 4 stays 4.
    expect(resolveLineWidths({ stroke: 8, eraser: 4 })).toEqual({
      stroke: 10,
      highlighter: defaultLineWidth(),
      eraser: 4,
      text: defaultLineWidth(),
    })
  })

  it('resolves width presets with fallbacks for invalid arrays', () => {
    expect(resolveWidthPresets(undefined)).toEqual([2, 4, 6, 10, 16])
    expect(resolveWidthPresets([1, 2, 3])).toEqual([2, 4, 6, 10, 16])
    expect(resolveWidthPresets([1, 2, 3, 4, 500])).toEqual([2, 4, 6, 10, 16])
    expect(resolveWidthPresets([3, 6, 9, 12, 20])).toEqual([3, 6, 9, 12, 20])
  })
})
