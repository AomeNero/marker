# Help Tools Grid Compact Layout Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the help page “In-session tools” section a full 2×5 grid with compact inline cards so Stamp no longer leaves empty cells on the right.

**Architecture:** CSS-only change to the existing `.help-tool-grid` / `.help-tool-card` rules in `docs/styles.css`. Use CSS Grid on each card so the top-level `kbd` sits beside `h3`+`p` without wrapping HTML. Keep the mobile single-column media query as-is.

**Tech Stack:** Static HTML help page (`docs/help.html`), `docs/styles.css`

## Global Constraints

- Scope is help docs styling only — do not change store screenshots, i18n strings, or app Vue UI
- Prefer CSS-only; only touch `docs/help.html` if CSS cannot align the layout
- Preserve scroll-reveal (`t-scroll-reveal`, `data-reveal-delay`) and hover styles
- Nested `<kbd>` inside Stamp’s `<p>` must remain styled as inline keys; only the direct-child shortcut `kbd` moves into the side column
- At `max-width: 1024px`, `.help-tool-grid` stays `grid-template-columns: 1fr`

**Spec:** `docs/superpowers/specs/2026-07-22-help-tools-grid-compact-design.md`

---

## File map

| File | Role |
|------|------|
| `docs/styles.css` | Change tool grid to 2 columns; compact card internal layout |
| `docs/help.html` | No change expected (verify only) |

---

### Task 1: Two-column compact tool cards

**Files:**
- Modify: `docs/styles.css` (`.help-tool-grid` ~1358–1361, `.help-tool-card` ~1363–1400)
- Verify: `docs/help.html` (tools section ~257–313) — markup unchanged

**Interfaces:**
- Consumes: Existing DOM — each `.help-tool-card` is `kbd` + `h3` + `p` (Stamp `p` may contain nested `kbd`)
- Produces: Desktop 2×5 filled grid; compact horizontal card layout

- [ ] **Step 1: Update `.help-tool-grid` to two columns**

In `docs/styles.css`, replace:

```css
.help-tool-grid {
  grid-template-columns: repeat(3, minmax(0, 1fr));
  margin-bottom: 16px;
}
```

with:

```css
.help-tool-grid {
  grid-template-columns: repeat(2, minmax(0, 1fr));
  margin-bottom: 16px;
}
```

- [ ] **Step 2: Restyle `.help-tool-card` as a compact 2-column grid**

Replace the card block from `.help-tool-card` through `.help-tool-card p` with:

```css
.help-tool-card {
  display: grid;
  grid-template-columns: auto minmax(0, 1fr);
  grid-template-rows: auto auto;
  column-gap: 12px;
  row-gap: 2px;
  align-items: start;
  padding: 12px 14px;
  border: 1px solid var(--line);
  border-radius: 10px;
  background: linear-gradient(180deg, rgba(255, 255, 255, 0.98), rgba(250, 250, 245, 0.98)),
    var(--surface);
  box-shadow: 0 1px 0 rgba(18, 18, 18, 0.04);
  transition:
    box-shadow var(--duration-fast) var(--ease-smooth-out),
    transform var(--duration-fast) var(--ease-smooth-out);
}

.help-tool-card:hover {
  box-shadow: 0 12px 28px rgba(18, 18, 18, 0.08);
  transform: translateY(-2px);
}

.help-tool-card.t-scroll-reveal.is-revealed:hover {
  transform: translateY(-2px);
}

.help-tool-card > kbd {
  grid-column: 1;
  grid-row: 1 / span 2;
  align-self: start;
  margin-bottom: 0;
}

.help-tool-card h3 {
  grid-column: 2;
  margin: 0;
  font-family: var(--font-display);
  font-size: 16px;
  font-weight: 950;
  line-height: 1.2;
}

.help-tool-card p {
  grid-column: 2;
  margin: 0;
  color: var(--muted);
  font-size: 13px;
  line-height: 1.5;
}
```

Do **not** change the `@media (max-width: 1024px)` rule that sets `.help-tool-grid { grid-template-columns: 1fr; }`.

- [ ] **Step 3: Visual verify in browser**

Open `docs/help.html` (file URL or any local static server). Check:

1. Desktop (~1200px+): 10 cards in 2 columns × 5 rows — no empty cell beside Stamp
2. Stamp description fully readable; nested `kbd` inside the paragraph still look like keys
3. Hover lift still works; scroll-reveal still runs
4. Narrow viewport (≤1024px): single column stack

Expected: all four checks pass.

- [ ] **Step 4: Commit**

```bash
git add docs/styles.css
git commit -m "ui(docs): compact help tools grid to two columns"
```

---

## Spec coverage (self-review)

| Spec requirement | Task |
|------------------|------|
| 2×5 desktop grid, no orphan | Task 1 Step 1 |
| Inline compact kbd + text | Task 1 Step 2 |
| Mobile 1-col preserved | Task 1 Step 2 note + Step 3 |
| No copy / i18n / screenshots | Global Constraints |
| Hover / reveal preserved | Task 1 Step 2 + Step 3 |
| Stamp long desc readable | Task 1 Step 3 |

No placeholders. Single subsystem — one plan is enough.
