import { describe, expect, it } from 'vitest'
import type { DrawAction } from '../composables/drawingTypes'
import {
  annotationFileName,
  MarkerFileError,
  parseAnnotationFile,
  persistableActions,
  planAnnotationLoad,
  serializeAnnotationFile,
  type LocalScreen,
  type MarkerScreen,
} from './annotationFile'

function makeAction(overrides: Partial<DrawAction> = {}): DrawAction {
  return {
    tool: 'pen',
    color: '#FF0000',
    lineWidth: 3,
    opacity: 1,
    points: [
      { x: 10, y: 20 },
      { x: 30, y: 40 },
    ],
    ...overrides,
  }
}

function makeScreen(overrides: Partial<MarkerScreen> = {}): MarkerScreen {
  return {
    name: 'DISPLAY A',
    x: 0,
    y: 0,
    width: 1920,
    height: 1080,
    scale: 1,
    actions: [makeAction()],
    ...overrides,
  }
}

function makeLocal(overrides: Partial<LocalScreen> = {}): LocalScreen {
  return {
    label: 'overlay',
    primary: true,
    name: 'DISPLAY A',
    x: 0,
    y: 0,
    width: 1920,
    height: 1080,
    scale: 1,
    ...overrides,
  }
}

describe('persistableActions', () => {
  it('keeps persistent strokes verbatim', () => {
    const actions = [makeAction(), makeAction({ tool: 'text', text: 'hi' })]
    expect(persistableActions(actions)).toHaveLength(2)
  })

  it('drops laser strokes', () => {
    const actions = [makeAction({ tool: 'laser' }), makeAction()]
    expect(persistableActions(actions)).toHaveLength(1)
    expect(persistableActions(actions)[0]!.tool).toBe('pen')
  })

  it('returns copies, not live references', () => {
    const actions = [makeAction()]
    const copy = persistableActions(actions)
    copy[0]!.points.push({ x: 99, y: 99 })
    expect(actions[0]!.points).toHaveLength(2)
  })
})

describe('annotationFileName', () => {
  it('formats marker + yyyyMMddHHmmss + extension', () => {
    const date = new Date(2026, 8, 3, 15, 30, 0)
    expect(annotationFileName(date)).toBe('marker20260903153000.marker')
  })

  it('zero-pads single digits', () => {
    const date = new Date(2026, 0, 5, 7, 8, 9)
    expect(annotationFileName(date)).toBe('marker20260105070809.marker')
  })
})

describe('serializeAnnotationFile / parseAnnotationFile round-trip', () => {
  it('restores screens and actions intact', () => {
    const screens = [
      makeScreen(),
      makeScreen({
        name: null,
        x: 1920,
        actions: [makeAction({ tool: 'arrow', points: [{ x: 1, y: 2, pressure: 0.5 }] })],
      }),
    ]
    const parsed = parseAnnotationFile(serializeAnnotationFile(screens, '1.2.3', '2026-09-03T00:00:00Z'))
    expect(parsed.version).toBe(1)
    expect(parsed.app).toBe('1.2.3')
    expect(parsed.savedAt).toBe('2026-09-03T00:00:00Z')
    expect(parsed.screens).toHaveLength(2)
    expect(parsed.screens[1]!.actions[0]!.points[0]!.pressure).toBe(0.5)
    expect(parsed.screens[1]!.x).toBe(1920)
  })

  it('keeps text and outline metadata', () => {
    const screens = [
      makeScreen({
        actions: [
          makeAction({
            tool: 'text',
            text: '标注',
            fontSize: 24,
            textOutline: { enabled: true, colorMode: 'fixed', color: '#000000', width: 2 },
          }),
        ],
      }),
    ]
    const parsed = parseAnnotationFile(serializeAnnotationFile(screens, '1.0.0'))
    const action = parsed.screens[0]!.actions[0]!
    expect(action.text).toBe('标注')
    expect(action.fontSize).toBe(24)
    expect(action.textOutline).toEqual({ enabled: true, colorMode: 'fixed', color: '#000000', width: 2 })
  })

  it('keeps hit geometry (bbox / rectHit / ellipseHit) and attached erasers', () => {
    const eraser = makeAction({ tool: 'eraser', lineWidth: 20 })
    const screens = [
      makeScreen({
        actions: [
          makeAction({
            tool: 'rect',
            bbox: { x1: 0, y1: 0, x2: 10, y2: 10 },
            rectHit: { x0: -1, y0: -1, x1: 11, y1: 11 },
            ellipseHit: { cx: 5, cy: 5, rx: 5, ry: 5 },
            attachedErasers: [eraser],
          }),
        ],
      }),
    ]
    const parsed = parseAnnotationFile(serializeAnnotationFile(screens, '1.0.0'))
    const action = parsed.screens[0]!.actions[0]!
    expect(action.bbox).toEqual({ x1: 0, y1: 0, x2: 10, y2: 10 })
    expect(action.rectHit).toEqual({ x0: -1, y0: -1, x1: 11, y1: 11 })
    expect(action.ellipseHit).toEqual({ cx: 5, cy: 5, rx: 5, ry: 5 })
    expect(action.attachedErasers).toHaveLength(1)
  })
})

describe('parseAnnotationFile rejection', () => {
  const expectError = (json: string, code: 'unreadable' | 'unsupported-version') => {
    try {
      parseAnnotationFile(json)
      expect.unreachable(`expected ${code}`)
    } catch (e) {
      expect(e).toBeInstanceOf(MarkerFileError)
      expect((e as MarkerFileError).code).toBe(code)
    }
  }

  it('rejects non-JSON payloads', () => {
    expectError('not json', 'unreadable')
  })

  it('rejects non-object payloads', () => {
    expectError('[1,2,3]', 'unreadable')
  })

  it('rejects unknown versions (forward-compat gate)', () => {
    expectError(JSON.stringify({ version: 2, screens: [] }), 'unsupported-version')
  })

  it('rejects missing screens', () => {
    expectError(JSON.stringify({ version: 1 }), 'unreadable')
  })

  it('rejects an action with unknown tool', () => {
    expectError(
      JSON.stringify({ version: 1, screens: [makeScreen({ actions: [makeAction({ tool: 'spray' as never })] })] }),
      'unreadable',
    )
  })

  it('rejects an action with non-finite coordinates', () => {
    expectError(
      JSON.stringify({
        version: 1,
        screens: [makeScreen({ actions: [{ ...makeAction(), points: [{ x: Number.NaN, y: 0 }] }] })],
      }),
      'unreadable',
    )
  })

  it('rejects an action with zero width or out-of-range opacity', () => {
    expectError(JSON.stringify({ version: 1, screens: [makeScreen({ actions: [makeAction({ lineWidth: 0 })] })] }), 'unreadable')
    expectError(JSON.stringify({ version: 1, screens: [makeScreen({ actions: [makeAction({ opacity: 5 })] })] }), 'unreadable')
  })

  it('rejects laser strokes smuggled into a hand-edited file', () => {
    expectError(
      JSON.stringify({ version: 1, screens: [makeScreen({ actions: [makeAction({ tool: 'laser' })] })] }),
      'unreadable',
    )
  })

  it('rejects attached payloads that are not eraser strokes', () => {
    expectError(
      JSON.stringify({
        version: 1,
        screens: [makeScreen({ actions: [makeAction({ attachedErasers: [makeAction({ tool: 'pen' })] })] })],
      }),
      'unreadable',
    )
  })
})

describe('planAnnotationLoad', () => {
  it('routes each screen onto its geometry-matching overlay', () => {
    const local = [
      makeLocal(),
      makeLocal({ label: 'overlay-2', primary: false, name: 'DISPLAY B', x: 1920, width: 2560, height: 1440, scale: 1.25 }),
    ]
    const plan = planAnnotationLoad(
      [makeScreen({ actions: [makeAction()] }), makeScreen({ name: 'DISPLAY B', x: 1920, width: 2560, height: 1440, scale: 1.25, actions: [makeAction(), makeAction()] })],
      local,
    )
    expect(plan.missingScreens).toBe(0)
    expect(plan.fallback).toBeNull()
    expect(plan.assignments.find((a) => a.label === 'overlay')!.actions).toHaveLength(1)
    expect(plan.assignments.find((a) => a.label === 'overlay-2')!.actions).toHaveLength(2)
  })

  it('falls back to the monitor name when geometry changed', () => {
    const local = [makeLocal({ x: 100, y: 50, width: 1600, height: 900 })]
    const plan = planAnnotationLoad([makeScreen({ x: 0, y: 0, width: 1920, height: 1080 })], local)
    expect(plan.missingScreens).toBe(0)
    expect(plan.assignments[0]!.actions).toHaveLength(1)
    expect(plan.fallback).toBeNull()
  })

  it('moves unmatched screens onto the primary screen with clamped coordinates', () => {
    const local = [makeLocal({ width: 1000, height: 800, name: 'DISPLAY A' })]
    const plan = planAnnotationLoad(
      [
        makeScreen({ name: 'DISPLAY A' }),
        makeScreen({ name: 'DISPLAY C', x: 9999, actions: [makeAction({ points: [{ x: -50, y: 500 }, { x: 3000, y: 1200 }] })] }),
      ],
      local,
    )
    expect(plan.missingScreens).toBe(1)
    expect(plan.fallback).not.toBeNull()
    expect(plan.fallback!.label).toBe('overlay')
    expect(plan.fallback!.clampedCount).toBe(1)
    const restored = plan.fallback!.actions[0]!.points
    expect(restored[0]).toEqual({ x: 0, y: 500 })
    expect(restored[1]).toEqual({ x: 1000, y: 800 })
  })

  it('prefers geometry over name when both could match different screens', () => {
    // Screen A' carries A's geometry but B's name; B is live. Geometry pass
    // must win for A' before B claims anything.
    const local = [
      makeLocal({ label: 'overlay', primary: true, name: 'A' }),
      makeLocal({ label: 'overlay-2', primary: false, name: 'B', x: 1920 }),
    ]
    const plan = planAnnotationLoad(
      [
        makeScreen({ name: 'B', actions: [makeAction({ color: '#111111' })] }),
        makeScreen({ name: 'B', x: 1920, actions: [makeAction({ color: '#222222' })] }),
      ],
      local,
    )
    // First file screen has overlay geometry (0,0,1920,1080) → claims `overlay`
    // despite its B name; the real B screen then claims overlay-2 by geometry.
    expect(plan.assignments.find((a) => a.label === 'overlay')!.actions[0]!.color).toBe('#111111')
    expect(plan.assignments.find((a) => a.label === 'overlay-2')!.actions[0]!.color).toBe('#222222')
    expect(plan.missingScreens).toBe(0)
  })

  it('never routes a non-primary window as fallback', () => {
    const local = [
      makeLocal({ label: 'overlay', primary: true }),
      makeLocal({ label: 'overlay-2', primary: false, name: 'OTHER', x: 1920 }),
    ]
    const plan = planAnnotationLoad([makeScreen({ name: 'GHOST', x: 5000 })], local)
    expect(plan.fallback!.label).toBe('overlay')
  })

  it('returns an empty plan when no screens are live', () => {
    const plan = planAnnotationLoad([makeScreen()], [])
    expect(plan.assignments).toHaveLength(0)
    expect(plan.fallback).toBeNull()
    expect(plan.missingScreens).toBe(1)
  })

  it('treats scale rounding differences within 1% as the same geometry', () => {
    const local = [makeLocal({ scale: 1.2500001 })]
    const plan = planAnnotationLoad([makeScreen({ scale: 1.25 })], local)
    expect(plan.missingScreens).toBe(0)
  })
})
