# Hold right-click to erase

**Date:** 2026-07-23  
**Status:** Approved design  
**Issue:** [#41](https://github.com/ifer47/marker/issues/41) (partial — eraser hold only; no radial menu)  
**Scope:** Annotation overlay pointer interaction (Mac + Windows)

## Problem

Heavy erase while annotating currently requires switching to eraser (`7` / toolbar) then back. Right-click today only opens the quick color palette. Users want a faster erase gesture without losing short-press color picking.

## Decisions (locked)

| Topic | Choice |
|-------|--------|
| Gesture | **Hold** RMB ≥ time threshold → temporary eraser; **release** RMB → restore previous tool |
| Short-press | Unchanged: open quick color palette |
| Disambiguation | **Time threshold** (~250ms), not drag-distance-only |
| Settings | **Always on**, no preference toggle |
| Text editing | **Disabled** while text box is open (preserve double-RMB commit) |
| Radial / pie menu | **Out of scope** (does not match keyboard-first product tone) |

## Out of scope

- Configurable RMB mode (color vs erase)
- Long-press radial tool picker (#41 proposal 2)
- Changing LMB semantics or numeric tool shortcuts
- Hold-to-erase while text box is open
- Changing macOS Control+click drag rules beyond existing guards

## Behavior

```
RMB pointerdown
    │
    ├─ text box open / penetration / inactive / quick colors open
    │       → no hold-erase (existing contextmenu / text rules apply)
    │
    ├─ start hold timer (~250ms)
    │
    ├─ pointerup before threshold
    │       → clear timer; open quick colors at pointer (explicitly;
    │         ignore/suppress contextmenu during the pending gesture)
    │
    └─ threshold reached while still held
            → remember toolBeforeRmb
            → currentTool = eraser
            → suppress palette for this gesture
            → start eraser stroke (then move continues erase)
            → pointerup → end stroke; restore toolBeforeRmb
```

### Edge cases

| Case | Result |
|------|--------|
| Already on eraser | Hold still erases; release leaves eraser selected |
| Threshold crossed then release with no move | Tiny/no-op stroke handling follows existing `startDraw` / `endDraw` rules; tool still restores |
| Quick colors already open | No new hold-erase (same as today’s RMB guard) |
| Penetration mode | No hold-erase |
| macOS Control+click after Ctrl-drag | Keep existing “do not treat as RMB palette” guard; do not start hold-erase in that path |
| Modifier shape tools (`toolBeforeModifier`) | Hold-erase should not fight an in-progress LMB draw; only button=2 path |

### Threshold

- Default **250ms** (constant in overlay / small helper; tweakable in code, not a user setting).
- Once hold-erase activates, **do not** open the color palette for that press, even if `contextmenu` fires later.

## Architecture

Primary surface: `DrawingOverlay.vue` pointer handlers (scheme aligned with LMB draw + existing `toolBeforeModifier` temporary tool pattern).

```
pointerdown (button === 2)
  → arm timer + set rmbHoldPending; capture pointer if appropriate
pointermove (while armed / active)
  → if hold-erase active: continueDraw
timer fire
  → rmbHoldPending = false; rmbEraseActive = true
  → remember toolBeforeRmb; currentTool = eraser
  → startDraw at current point
pointerup / pointercancel / leave (button 2)
  → clear timer
  → if rmbEraseActive: endDraw + restore toolBeforeRmb; clear flags
  → else if was short press (pending cleared without activate):
        open quick colors at pointer (do not rely on late contextmenu alone)
contextmenu
  → always preventDefault (already)
  → if rmbHoldPending || rmbEraseActive: return (no palette)
  → else: existing onContextMenu / text double-RMB
```

**Platform note:** `contextmenu` may fire on down or up depending on OS. While `rmbHoldPending` or `rmbEraseActive`, never open the palette from `contextmenu`. Short-press palette is opened explicitly on early `pointerup` (or only if a non-suppressed `contextmenu` arrives after a completed short press with no hold activation). Prefer one code path: **open palette from short `pointerup`**, and treat `contextmenu` as suppressible noise during the gesture.

Optional small pure helpers (testable):

- hold-state transitions: pending → active → idle; short-press vs activate
- constants: `RMB_HOLD_ERASE_MS` (250)

No Rust / config / IPC changes.

## UX / docs

- Help / README: add one line under draw tips — “Hold right-click to erase; release to restore tool. Short right-click still opens colors.”
- i18n: en + zh-CN help strings if settings help lists RMB (keep both locales in sync).
- Issue #41: implement this slice only; leave radial menu declined or deferred in the issue comment when shipping.

## Acceptance

1. Short RMB (< threshold) still opens quick colors.
2. Hold RMB ≥ threshold switches to eraser cursor/tool and dragging erases under current eraser mode (stroke/object).
3. Release RMB restores the tool that was active at hold start.
4. Text box open: hold does not switch to eraser; double-RMB commit still works.
5. No new settings UI.
6. Vitest covers threshold vs short-press suppress flag / restore tool helper if extracted; manual QA on Win + Mac for palette + erase feel.

## Non-goals reminder

This does **not** close all of #41 — only the efficient erase path. Radial menu remains rejected for product-tone reasons unless revisited later.
