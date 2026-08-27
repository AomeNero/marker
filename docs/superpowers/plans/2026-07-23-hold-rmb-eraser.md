# Hold Right-Click to Erase Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Hold right mouse button ≥ 250ms to temporarily erase; release restores the previous tool; short right-click still opens the quick color palette.

**Architecture:** Extract a pure RMB-hold gesture state machine (`src/utils/rmbHoldErase.ts`) with Vitest coverage. `DrawingOverlay.vue` arms a timer on `pointerdown` (button 2), activates eraser + `startDraw` on timeout, restores on `pointerup`, and opens the palette only on short release while suppressing `contextmenu` during pending/active.

**Tech Stack:** Vue 3, Vitest, existing `useDrawing` (`startDraw` / `draw` / `endDraw`), overlay pointer capture

## Global Constraints

- Spec: `docs/superpowers/specs/2026-07-23-hold-rmb-eraser-design.md`
- Time threshold: **250ms** (`RMB_HOLD_ERASE_MS`) — code constant, not a user setting
- Always on — **no** settings / config / IPC
- Short press: quick colors; long press: temporary eraser
- Text box open: **do not** start hold-erase (preserve double-RMB commit)
- Suppress palette while `pending` or `active`; open palette from **short `pointerup`**, not raw `contextmenu`
- No radial menu (#41 proposal 2)
- i18n: sync `en.ts` and `zh-CN.ts`
- Prefer pure helpers for logic; keep timer + drawing side effects in the overlay

---

## File map

| File | Role |
|------|------|
| `src/utils/rmbHoldErase.ts` | Pure gesture phases + guards + release outcomes |
| `src/utils/rmbHoldErase.test.ts` | Unit tests for phases / suppress / canStart |
| `src/components/DrawingOverlay.vue` | Timer, pointer button-2 path, wire start/end draw, contextmenu guard |
| `src/i18n/en.ts`, `src/i18n/zh-CN.ts` | Help copy: short = colors, hold = erase |
| `docs/help.html` + `docs/i18n.js` (if RMB row exists) | Keep site help in sync with in-app help |

---

### Task 1: Pure RMB-hold gesture helper + tests

**Files:**
- Create: `src/utils/rmbHoldErase.ts`
- Create: `src/utils/rmbHoldErase.test.ts`

**Interfaces:**
- Produces:
  - `RMB_HOLD_ERASE_MS = 250`
  - `type RmbHoldPhase = 'idle' | 'pending' | 'active'`
  - `type RmbHoldGesture = { phase: RmbHoldPhase; toolBefore: string | null }`
  - `IDLE_RMB_HOLD: RmbHoldGesture`
  - `canStartRmbHoldErase({ active, penetration, textBoxOpen, quickColorsOpen }): boolean`
  - `startRmbHoldPending(): RmbHoldGesture` → `{ phase: 'pending', toolBefore: null }`
  - `activateRmbHoldErase(gesture, currentTool: string): RmbHoldGesture` (no-op unless `pending`; stores `toolBefore: currentTool`, `phase: 'active'`)
  - `releaseRmbHold(gesture): { next; openPalette; finishErase; restoreTool }`
  - `cancelRmbHold(gesture): { next; openPalette: false; finishErase; restoreTool }` (abort without palette)
  - `shouldBlockQuickColors(gesture): boolean` → true when `pending` or `active`

- [ ] **Step 1: Write the failing tests**

Create `src/utils/rmbHoldErase.test.ts`:

```ts
import { describe, expect, it } from 'vitest'
import {
  IDLE_RMB_HOLD,
  RMB_HOLD_ERASE_MS,
  activateRmbHoldErase,
  canStartRmbHoldErase,
  cancelRmbHold,
  releaseRmbHold,
  shouldBlockQuickColors,
  startRmbHoldPending,
} from './rmbHoldErase'

describe('rmbHoldErase', () => {
  it('exposes 250ms threshold', () => {
    expect(RMB_HOLD_ERASE_MS).toBe(250)
  })

  it('canStart is false when text box open or penetrating', () => {
    expect(
      canStartRmbHoldErase({
        active: true,
        penetration: false,
        textBoxOpen: true,
        quickColorsOpen: false,
      }),
    ).toBe(false)
    expect(
      canStartRmbHoldErase({
        active: true,
        penetration: true,
        textBoxOpen: false,
        quickColorsOpen: false,
      }),
    ).toBe(false)
  })

  it('canStart is true for normal annotation', () => {
    expect(
      canStartRmbHoldErase({
        active: true,
        penetration: false,
        textBoxOpen: false,
        quickColorsOpen: false,
      }),
    ).toBe(true)
  })

  it('short release opens palette and does not finish erase', () => {
    const pending = startRmbHoldPending()
    expect(shouldBlockQuickColors(pending)).toBe(true)
    const out = releaseRmbHold(pending)
    expect(out.openPalette).toBe(true)
    expect(out.finishErase).toBe(false)
    expect(out.restoreTool).toBeNull()
    expect(out.next).toEqual(IDLE_RMB_HOLD)
  })

  it('activate then release finishes erase and restores tool', () => {
    const pending = startRmbHoldPending()
    const active = activateRmbHoldErase(pending, 'pen')
    expect(active).toEqual({ phase: 'active', toolBefore: 'pen' })
    expect(shouldBlockQuickColors(active)).toBe(true)
    const out = releaseRmbHold(active)
    expect(out.openPalette).toBe(false)
    expect(out.finishErase).toBe(true)
    expect(out.restoreTool).toBe('pen')
    expect(out.next).toEqual(IDLE_RMB_HOLD)
  })

  it('activate while already eraser still restores eraser', () => {
    const active = activateRmbHoldErase(startRmbHoldPending(), 'eraser')
    const out = releaseRmbHold(active)
    expect(out.restoreTool).toBe('eraser')
  })

  it('cancel never opens palette', () => {
    const pending = startRmbHoldPending()
    expect(cancelRmbHold(pending).openPalette).toBe(false)
    const active = activateRmbHoldErase(startRmbHoldPending(), 'highlighter')
    const out = cancelRmbHold(active)
    expect(out.openPalette).toBe(false)
    expect(out.finishErase).toBe(true)
    expect(out.restoreTool).toBe('highlighter')
  })

  it('activate is no-op from idle', () => {
    expect(activateRmbHoldErase(IDLE_RMB_HOLD, 'pen')).toEqual(IDLE_RMB_HOLD)
  })
})
```

- [ ] **Step 2: Run tests — expect FAIL (module missing)**

Run: `npx vitest run src/utils/rmbHoldErase.test.ts`

Expected: FAIL — cannot find module `./rmbHoldErase`

- [ ] **Step 3: Implement `src/utils/rmbHoldErase.ts`**

```ts
export const RMB_HOLD_ERASE_MS = 250

export type RmbHoldPhase = 'idle' | 'pending' | 'active'

export type RmbHoldGesture = {
  phase: RmbHoldPhase
  toolBefore: string | null
}

export const IDLE_RMB_HOLD: RmbHoldGesture = {
  phase: 'idle',
  toolBefore: null,
}

export type RmbHoldEnd = {
  next: RmbHoldGesture
  openPalette: boolean
  finishErase: boolean
  restoreTool: string | null
}

export function canStartRmbHoldErase(opts: {
  active: boolean
  penetration: boolean
  textBoxOpen: boolean
  quickColorsOpen: boolean
}): boolean {
  return opts.active && !opts.penetration && !opts.textBoxOpen && !opts.quickColorsOpen
}

export function startRmbHoldPending(): RmbHoldGesture {
  return { phase: 'pending', toolBefore: null }
}

export function activateRmbHoldErase(
  gesture: RmbHoldGesture,
  currentTool: string,
): RmbHoldGesture {
  if (gesture.phase !== 'pending') return gesture
  return { phase: 'active', toolBefore: currentTool }
}

export function shouldBlockQuickColors(gesture: RmbHoldGesture): boolean {
  return gesture.phase === 'pending' || gesture.phase === 'active'
}

export function releaseRmbHold(gesture: RmbHoldGesture): RmbHoldEnd {
  if (gesture.phase === 'pending') {
    return {
      next: IDLE_RMB_HOLD,
      openPalette: true,
      finishErase: false,
      restoreTool: null,
    }
  }
  if (gesture.phase === 'active') {
    return {
      next: IDLE_RMB_HOLD,
      openPalette: false,
      finishErase: true,
      restoreTool: gesture.toolBefore,
    }
  }
  return {
    next: IDLE_RMB_HOLD,
    openPalette: false,
    finishErase: false,
    restoreTool: null,
  }
}

export function cancelRmbHold(gesture: RmbHoldGesture): RmbHoldEnd {
  if (gesture.phase === 'active') {
    return {
      next: IDLE_RMB_HOLD,
      openPalette: false,
      finishErase: true,
      restoreTool: gesture.toolBefore,
    }
  }
  return {
    next: IDLE_RMB_HOLD,
    openPalette: false,
    finishErase: false,
    restoreTool: null,
  }
}
```

- [ ] **Step 4: Run tests — expect PASS**

Run: `npx vitest run src/utils/rmbHoldErase.test.ts`

Expected: all tests PASS

- [ ] **Step 5: Commit**

```bash
git add src/utils/rmbHoldErase.ts src/utils/rmbHoldErase.test.ts
git commit -m "feat(drawing): add rmb hold-to-erase gesture state helper"
```

---

### Task 2: Wire hold-erase into `DrawingOverlay.vue`

**Files:**
- Modify: `src/components/DrawingOverlay.vue`

**Interfaces:**
- Consumes: all exports from Task 1
- Produces: button-2 pointer path that calls `startDraw` / `draw` / `endDraw` with eraser while active; short release opens palette via existing `quickColorsPos` + `showQuickColors`

- [ ] **Step 1: Import helpers and add module-level gesture state**

Near other imports / RMB text-click state in `DrawingOverlay.vue`:

```ts
import {
  IDLE_RMB_HOLD,
  RMB_HOLD_ERASE_MS,
  activateRmbHoldErase,
  canStartRmbHoldErase,
  cancelRmbHold,
  releaseRmbHold,
  shouldBlockQuickColors,
  startRmbHoldPending,
  type RmbHoldGesture,
} from '../utils/rmbHoldErase'
```

Add:

```ts
let rmbHoldGesture: RmbHoldGesture = IDLE_RMB_HOLD
let rmbHoldTimer: ReturnType<typeof setTimeout> | null = null
let rmbHoldPointerId: number | null = null
```

- [ ] **Step 2: Add clear/activate helpers**

```ts
function clearRmbHoldTimer() {
  if (rmbHoldTimer !== null) {
    clearTimeout(rmbHoldTimer)
    rmbHoldTimer = null
  }
}

function openQuickColorsAt(clientX: number, clientY: number) {
  if (!active.value || penetrationMode.value || isDrawing.value) return
  hideToolbarPopupForCanvasInteraction()
  quickColorsPos.value = { x: clientX, y: clientY }
  showQuickColors.value = true
  logActionEvent('quick colors opened', { reason: 'context-menu' })
}

function activateHoldEraseFromTimer(clientX: number, clientY: number) {
  if (rmbHoldGesture.phase !== 'pending') return
  rmbHoldGesture = activateRmbHoldErase(rmbHoldGesture, currentTool.value)
  if (rmbHoldGesture.phase !== 'active') return
  hideToolbarPopupForCanvasInteraction()
  currentTool.value = 'eraser'
  capturePointer(
    // Prefer reusing last pointer event if you keep a ref; otherwise call startDraw only —
    // see Step 3 for pointerdown capture.
  )
  startDraw({ x: clientX, y: clientY })
  logActionEvent('rmb hold erase', { toolBefore: rmbHoldGesture.toolBefore })
}
```

**Important:** In Step 3, on `pointerdown` button 2, call `capturePointer(e)` when starting pending so move/up are reliable; store `e.clientX/Y` in closure for the timer callback.

- [ ] **Step 3: Handle `pointerdown` for button 2**

Change `onPointerDown` so button 0 keeps current early return, and add a button-2 branch **before** or **instead of** `if (e.button !== 0) return`:

```ts
async function onPointerDown(e: PointerEvent) {
  if (e.button === 2) {
    onRmbPointerDown(e)
    return
  }
  if (e.button !== 0) return
  // ... existing LMB body unchanged ...
}

function onRmbPointerDown(e: PointerEvent) {
  if (
    !canStartRmbHoldErase({
      active: active.value,
      penetration: penetrationMode.value,
      textBoxOpen: !!textBoxPos.value,
      quickColorsOpen: showQuickColors.value,
    })
  ) {
    return
  }
  // macOS: Control+click mapped as RMB after Ctrl-drag — skip (same spirit as onContextMenu)
  if (isMacOS() && e.ctrlKey && pointerMovedSinceDown) return

  clearRmbHoldTimer()
  rmbHoldGesture = startRmbHoldPending()
  rmbHoldPointerId = e.pointerId
  pointerDownClient = { x: e.clientX, y: e.clientY }
  pointerMovedSinceDown = false
  const startX = e.clientX
  const startY = e.clientY
  capturePointer(e)
  rmbHoldTimer = setTimeout(() => {
    rmbHoldTimer = null
    activateHoldEraseFromTimer(startX, startY)
  }, RMB_HOLD_ERASE_MS)
}
```

Refine `activateHoldEraseFromTimer` to use **current** `lastPointerX` / `lastPointerY` (updated in `onPointerMove`) so erase starts at the pointer position at activation time, not only the down point:

```ts
function activateHoldEraseFromTimer() {
  if (rmbHoldGesture.phase !== 'pending') return
  rmbHoldGesture = activateRmbHoldErase(rmbHoldGesture, currentTool.value)
  if (rmbHoldGesture.phase !== 'active') return
  hideToolbarPopupForCanvasInteraction()
  currentTool.value = 'eraser'
  startDraw({ x: lastPointerX, y: lastPointerY })
  logActionEvent('rmb hold erase', { toolBefore: rmbHoldGesture.toolBefore })
}
```

And in the timeout: `activateHoldEraseFromTimer()`.

- [ ] **Step 4: Extend `onPointerMove` / `onPointerUp` for RMB hold**

In `onPointerMove`, after existing drawing branch, ensure when `rmbHoldGesture.phase === 'active'` and `isDrawing`, the existing `draw` / `drawBatch` path already runs (because `isDrawing` is true). No extra branch if LMB and RMB share the same `isDrawing` path — verify `onPointerMove` does not require `buttons === 1`. If it gates on primary button, add:

```ts
// inside move, when drawing:
if (isDrawing.value && (rmbHoldGesture.phase === 'active' || /* existing */ true)) {
  // existing drawBatch path
}
```

Inspect current `onPointerMove` — if it only checks `isDrawing` / `isDragging`, RMB hold erase works without change once `startDraw` ran.

In `onPointerUp`:

```ts
function onPointerUp(e: PointerEvent) {
  // Existing capturedPointerId mismatch guard stays

  if (rmbHoldPointerId !== null && e.pointerId === rmbHoldPointerId) {
    clearRmbHoldTimer()
    const end = releaseRmbHold(rmbHoldGesture)
    rmbHoldGesture = end.next
    rmbHoldPointerId = null

    if (end.finishErase) {
      const wasDrawing = isDrawing.value
      releaseCapturedPointer()
      endDraw()
      if (end.restoreTool !== null) {
        currentTool.value = end.restoreTool as Tool
      }
      markPointerInteractionEnded()
      resetPointerGestureState()
      if (wasDrawing) {
        logDiagnostic('pointer', 'stroke end', {
          pointerType: e.pointerType,
          button: e.button,
          reason: 'rmb-hold-erase',
        })
      }
      return
    }

    if (end.openPalette) {
      releaseCapturedPointer()
      markPointerInteractionEnded()
      resetPointerGestureState()
      openQuickColorsAt(e.clientX, e.clientY)
      return
    }
  }

  // ... existing LMB onPointerUp body ...
}
```

Also clear hold state in `abortActivePointerInteraction`:

```ts
clearRmbHoldTimer()
if (rmbHoldGesture.phase !== 'idle') {
  const end = cancelRmbHold(rmbHoldGesture)
  rmbHoldGesture = end.next
  rmbHoldPointerId = null
  if (end.finishErase && end.restoreTool !== null) {
    currentTool.value = end.restoreTool as Tool
  }
}
```

(If abort already calls `endDraw`, keep order: endDraw then restore tool.)

- [ ] **Step 5: Guard `onContextMenu`**

At the top of `onContextMenu` (after `preventDefault` / text handler as appropriate):

```ts
function onContextMenu(e: MouseEvent) {
  e.preventDefault()
  if (handleTextBoxContextMenu(e)) return
  if (shouldBlockQuickColors(rmbHoldGesture)) return
  if (performance.now() < suppressQuickColorsUntil) return
  // Prefer not opening palette here for plain short-click anymore if pointerup already opened it.
  // Keep as fallback only when gesture is idle (e.g. text path already returned):
  if (!active.value || penetrationMode.value || isDrawing.value) return
  if (isMacOS() && e.ctrlKey && pointerMovedSinceDown) return
  openQuickColorsAt(e.clientX, e.clientY)
}
```

Refactor existing body to call `openQuickColorsAt` to avoid duplication. **Do not** open palette twice: short press opens on `pointerup`; idle `contextmenu` fallback is OK if gesture already idle (second open is harmless if `showQuickColors` already true — or early-return if already open).

- [ ] **Step 6: Manual sanity in unitless run**

Run: `npx vitest run src/utils/rmbHoldErase.test.ts`

Expected: PASS

Run: `npm run lint` on touched files if convenient.

- [ ] **Step 7: Commit**

```bash
git add src/components/DrawingOverlay.vue
git commit -m "feat(drawing): hold right-click to erase, release restores tool"
```

---

### Task 3: Help / i18n copy + site help sync

**Files:**
- Modify: `src/i18n/en.ts`
- Modify: `src/i18n/zh-CN.ts`
- Modify: `src/components/SettingsView.vue` (only if a second help row is needed)
- Modify: `docs/i18n.js` and `docs/help.html` if they document right-click colors

**Interfaces:**
- Produces: help strings that mention short vs hold RMB

- [ ] **Step 1: Update in-app help strings**

In `en.ts` help section, change / add:

```ts
rightClickColor: 'Quick color picker',
rightClickErase: 'Hold to erase',
mouseRightClick: 'Right-click',
mouseRightClickHold: 'Hold right-click',
```

Or keep one row and expand the label:

```ts
rightClickColor: 'Colors (tap) / erase (hold)',
```

Prefer **two rows** next to the existing color row in `SettingsView.vue` help card (color + hold erase), matching the locked UX.

`zh-CN.ts`:

```ts
rightClickColor: '快速选色（点按）',
rightClickErase: '按住擦除',
mouseRightClickHold: '长按右键',
```

- [ ] **Step 2: Add help row in `SettingsView.vue`**

Beside the existing right-click color row (~line 639), add:

```vue
<div class="help-row">
  <span class="help-label">{{ t('help.rightClickErase') }}</span>
  <span class="help-keys-plain">{{ t('help.mouseRightClickHold') }}</span>
</div>
```

(Use the same markup pattern as neighboring rows.)

- [ ] **Step 3: Sync `docs/i18n.js` / help page**

If `helpPage.draw.rightClick` exists, update EN/ZH to mention hold-to-erase in one short phrase, e.g. EN: `Color picker (click) · Erase (hold)`.

- [ ] **Step 4: Commit**

```bash
git add src/i18n/en.ts src/i18n/zh-CN.ts src/components/SettingsView.vue docs/i18n.js docs/help.html
git commit -m "docs(help): document hold right-click to erase"
```

---

### Task 4: Manual QA checklist (no code)

**Files:** none

- [ ] **Step 1: Run automated suite**

Run: `npm test`

Expected: PASS (including new `rmbHoldErase` tests)

- [ ] **Step 2: Manual overlay checks (Windows and/or macOS)**

| # | Action | Expected |
|---|--------|----------|
| 1 | Short RMB | Quick colors open; tool unchanged |
| 2 | Hold RMB ≥ 250ms then drag | Erases with current eraser mode; cursor/tool shows eraser |
| 3 | Release after hold | Previous tool restored |
| 4 | Hold while already on eraser | Erases; stays on eraser after release |
| 5 | Open text box; hold RMB | No erase; double-RMB still commits |
| 6 | Penetration mode | No hold-erase / no palette (existing) |
| 7 | macOS Control+click after Ctrl-drag | Does not spuriously erase or open palette |

- [ ] **Step 3: Comment on GitHub issue #41** (when shipping)

Note that hold-to-erase shipped; radial menu remains out of scope for product tone.

---

## Spec coverage self-review

| Spec requirement | Task |
|------------------|------|
| 250ms threshold | Task 1 constant + Task 2 timer |
| Short = colors, hold = erase | Task 2 release / activate |
| No settings | All tasks |
| Text box disables hold | `canStartRmbHoldErase` + Task 2 |
| Suppress contextmenu pending/active | Task 2 `shouldBlockQuickColors` |
| Palette from short pointerup | Task 2 |
| Restore previous tool | Task 1/2 `restoreTool` |
| Help copy | Task 3 |
| No radial menu | Explicitly omitted |
| Tests | Task 1 (+ Task 4 suite) |

## Placeholder scan

No TBD / “implement later” steps. Overlay wiring shows concrete function shapes; implementer must match existing `capturePointer` / `releaseCapturedPointer` helpers already in `DrawingOverlay.vue`.
