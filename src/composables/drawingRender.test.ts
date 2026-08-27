import { describe, it, expect, vi } from 'vitest'
import { drawText, drawStamp } from './drawingRender'
import type { DrawAction } from './drawingTypes'

function makeTextAction(overrides: Partial<DrawAction> = {}): DrawAction {
  return {
    tool: 'text',
    color: '#FF0000',
    lineWidth: 3,
    opacity: 1,
    points: [{ x: 10, y: 20 }],
    text: 'Hello',
    fontSize: 24,
    ...overrides,
  }
}

function makeTextContext(calls: string[]) {
  return {
    font: '',
    globalAlpha: 1,
    fillStyle: '',
    strokeStyle: '',
    lineWidth: 1,
    lineJoin: 'miter',
    miterLimit: 10,
    textBaseline: 'alphabetic',
    measureText: vi.fn(() => ({ fontBoundingBoxAscent: 18, fontBoundingBoxDescent: 6 })),
    strokeText: vi.fn(() => calls.push('stroke')),
    fillText: vi.fn(() => calls.push('fill')),
  } as unknown as CanvasRenderingContext2D
}

describe('drawingRender text outline', () => {
  it('draws outline before filling text when enabled', () => {
    const calls: string[] = []
    const ctx = makeTextContext(calls)

    drawText(
      ctx,
      makeTextAction({
        textOutline: { enabled: true, colorMode: 'fixed', color: '#FFFFFF', width: 4 },
      }),
    )

    expect(ctx.strokeStyle).toBe('#FFFFFF')
    expect(ctx.lineWidth).toBe(4)
    expect(ctx.strokeText).toHaveBeenCalledOnce()
    expect(ctx.fillText).toHaveBeenCalledOnce()
    expect(calls).toEqual(['stroke', 'fill'])
  })

  it('keeps legacy text rendering unchanged when outline is disabled', () => {
    const calls: string[] = []
    const ctx = makeTextContext(calls)

    drawText(ctx, makeTextAction())

    expect(ctx.strokeText).not.toHaveBeenCalled()
    expect(ctx.fillText).toHaveBeenCalledOnce()
    expect(calls).toEqual(['fill'])
  })

  it('resolves automatic outline color from text color', () => {
    const calls: string[] = []
    const ctx = makeTextContext(calls)

    drawText(
      ctx,
      makeTextAction({
        color: '#FFFFFF',
        textOutline: { enabled: true, colorMode: 'auto', color: '#FFFFFF', width: 3 },
      }),
    )

    expect(ctx.strokeStyle).toBe('#000000')
    expect(calls).toEqual(['stroke', 'fill'])
  })
})

describe('drawingRender stamp', () => {
  it('draws FastStone-style ring disc, fill, then label', () => {
    const calls: string[] = []
    const ctx = {
      globalAlpha: 1,
      fillStyle: '',
      strokeStyle: '',
      lineWidth: 1,
      lineJoin: 'miter',
      font: '',
      textAlign: 'start',
      textBaseline: 'alphabetic',
      shadowColor: '',
      shadowBlur: 0,
      shadowOffsetX: 0,
      shadowOffsetY: 0,
      save: vi.fn(),
      restore: vi.fn(),
      beginPath: vi.fn(),
      arc: vi.fn(),
      translate: vi.fn(),
      scale: vi.fn(),
      fill: vi.fn(() => calls.push('fill')),
      stroke: vi.fn(() => calls.push('stroke')),
      fillText: vi.fn(() => calls.push('label')),
    } as unknown as CanvasRenderingContext2D

    drawStamp(ctx, {
      tool: 'stamp',
      color: '#FF3B30',
      lineWidth: 3,
      opacity: 1,
      points: [{ x: 50, y: 60 }],
      text: '1',
      fontSize: 24,
    })

    expect(ctx.save).toHaveBeenCalled()
    expect(ctx.restore).toHaveBeenCalled()
    // shadow silhouette + crisp ring disc + inner fill
    expect(ctx.arc).toHaveBeenCalledTimes(3)
    expect(ctx.scale).not.toHaveBeenCalled()
    expect(ctx.fillText).toHaveBeenCalledOnce()
    expect(calls).toEqual(['fill', 'fill', 'fill', 'label'])
    expect(ctx.fillStyle).toBe('#FFFFFF')
  })
})
