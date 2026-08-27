# Help tools grid: 2-column compact layout

**Date:** 2026-07-22  
**Status:** Approved design  
**Scope:** `docs/help.html` In-session tools section styling

## Problem

The help page “In-session tools” section uses a 3-column card grid with 10 tools. The last row contains only Stamp, leaving empty space on the right and an unbalanced layout.

## Decision

Use a **2-column, inline-compact** card layout (option C from design review):

- Desktop: 2 columns × 5 rows — full grid, no orphan card
- Each card: keyboard shortcut (`kbd`) sits beside title + description (horizontal flex), so cards are shorter and overall section height stays close to the current 3-column layout
- Keep existing visual language (borders, radius, hover, typography)
- Mobile: keep the existing single-column breakpoint

## Out of scope

- Store screenshot HTML under `assets/store-screenshots/`
- i18n / copy changes (en / zh-CN strings stay as-is)
- New tool groupings, extra cards, or content reordering
- App UI (toolbar / overlay) — help docs only

## Implementation

### Files

| File | Change |
|------|--------|
| `docs/styles.css` | `.help-tool-grid` → `repeat(2, minmax(0, 1fr))`; `.help-tool-card` → flex row layout for `kbd` + text block; tighten padding/gaps as needed for compact height |
| `docs/help.html` | Only if markup needs a wrapper around `h3`+`p` for flex alignment; prefer CSS-only if possible |

### Layout rules

1. `.help-tool-grid { grid-template-columns: repeat(2, minmax(0, 1fr)); }`
2. Card internal structure: `kbd` left (or top-left aligned), title + description stacked to the right
3. Preserve `data-reveal-delay` / scroll-reveal behavior
4. Existing `@media` rule that sets `.help-tool-grid` to `1fr` on small screens remains authoritative

### Acceptance

- [ ] Ten tool cards fill a complete 2×5 grid on desktop — no trailing empty cells
- [ ] Stamp long description remains readable without clipping
- [ ] Hover / reveal animations still work
- [ ] Narrow viewport stacks to one column
- [ ] No copy or locale file changes

## Verification

Open `docs/help.html` in a browser (or local docs server), check the tools section at desktop and mobile widths.
