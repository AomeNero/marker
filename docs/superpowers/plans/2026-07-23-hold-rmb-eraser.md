# 按住右键擦除实现计划

> **致代理执行者：** 必需子技能：使用 superpowers:subagent-driven-development（推荐）或 superpowers:executing-plans 按任务执行本计划。步骤使用复选框（`- [ ]`）语法跟踪。

**目标：** 按住鼠标右键 ≥ 250ms 临时擦除；松开恢复之前的工具；右键短按仍打开快速色盘。

**架构：** 抽取纯 RMB 按住手势状态机（`src/utils/rmbHoldErase.ts`）并配 Vitest 覆盖。`DrawingOverlay.vue` 在 `pointerdown`（button 2）启动计时器，超时激活橡皮擦 + `startDraw`，`pointerup` 恢复，仅在短按松开时打开色盘并在 pending/active 期间抑制 `contextmenu`。

**技术栈：** Vue 3、Vitest、既有 `useDrawing`（`startDraw` / `draw` / `endDraw`）、覆盖层指针捕获

## 全局约束

- 规格：`docs/superpowers/specs/2026-07-23-hold-rmb-eraser-design.md`
- 时间阈值：**250ms**（`RMB_HOLD_ERASE_MS`）——代码常量，不是用户设置
- 始终开启——**无**设置 / 配置 / IPC
- 短按：快速色盘；长按：临时橡皮擦
- 文本框打开：**不**启动按住擦除（保留双击右键确认）
- `pending` 或 `active` 期间抑制色盘；色盘从**短按 `pointerup`** 打开，而非原始 `contextmenu`
- 无环形菜单（#41 提案 2）
- i18n：同步 `en.ts` 与 `zh-CN.ts`
- 逻辑优先纯函数；计时器与绘制副作用保留在覆盖层

---

## 文件映射

| 文件 | 角色 |
|------|------|
| `src/utils/rmbHoldErase.ts` | 纯手势阶段 + 守卫 + 松开结果 |
| `src/utils/rmbHoldErase.test.ts` | 阶段 / 抑制 / canStart 单元测试 |
| `src/components/DrawingOverlay.vue` | 计时器、指针 button-2 路径、接线起止绘制、contextmenu 守卫 |
| `src/i18n/en.ts`、`src/i18n/zh-CN.ts` | 帮助文案：短按 = 色盘，长按 = 擦除 |
| `docs/help.html` + `docs/i18n.js`（若有 RMB 行） | 站点帮助与应用内帮助保持同步 |

---

### 任务 1：纯 RMB 按住手势辅助函数 + 测试

**文件：**
- 创建：`src/utils/rmbHoldErase.ts`
- 创建：`src/utils/rmbHoldErase.test.ts`

**接口：**
- 产出：
  - `RMB_HOLD_ERASE_MS = 250`
  - `type RmbHoldPhase = 'idle' | 'pending' | 'active'`
  - `type RmbHoldGesture = { phase: RmbHoldPhase; toolBefore: string | null }`
  - `IDLE_RMB_HOLD: RmbHoldGesture`
  - `canStartRmbHoldErase({ active, penetration, textBoxOpen, quickColorsOpen }): boolean`
  - `startRmbHoldPending(): RmbHoldGesture` → `{ phase: 'pending', toolBefore: null }`
  - `activateRmbHoldErase(gesture, currentTool: string): RmbHoldGesture`（非 `pending` 时 no-op；记录 `toolBefore: currentTool`，`phase: 'active'`）
  - `releaseRmbHold(gesture): { next; openPalette; finishErase; restoreTool }`
  - `cancelRmbHold(gesture): { next; openPalette: false; finishErase; restoreTool }`（中止且不开色盘）
  - `shouldBlockQuickColors(gesture): boolean` → `pending` 或 `active` 时为 true

- [ ] **步骤 1：编写失败测试**

创建 `src/utils/rmbHoldErase.test.ts`：

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

- [ ] **步骤 2：运行测试——预期 FAIL（模块缺失）**

运行：`npx vitest run src/utils/rmbHoldErase.test.ts`

预期：FAIL——找不到模块 `./rmbHoldErase`

- [ ] **步骤 3：实现 `src/utils/rmbHoldErase.ts`**

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

- [ ] **步骤 4：运行测试——预期 PASS**

运行：`npx vitest run src/utils/rmbHoldErase.test.ts`

预期：全部测试 PASS

- [ ] **步骤 5：提交**

```bash
git add src/utils/rmbHoldErase.ts src/utils/rmbHoldErase.test.ts
git commit -m "feat(drawing): add rmb hold-to-erase gesture state helper"
```

---

### 任务 2：将按住擦除接入 `DrawingOverlay.vue`

**文件：**
- 修改：`src/components/DrawingOverlay.vue`

**接口：**
- 消费：任务 1 的全部导出
- 产出：button-2 指针路径，激活期间以橡皮擦调用 `startDraw` / `draw` / `endDraw`；短按松开经既有 `quickColorsPos` + `showQuickColors` 打开色盘

- [ ] **步骤 1：导入辅助函数并添加模块级手势状态**

在 `DrawingOverlay.vue` 其他导入 / RMB 文字点击状态附近：

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

添加：

```ts
let rmbHoldGesture: RmbHoldGesture = IDLE_RMB_HOLD
let rmbHoldTimer: ReturnType<typeof setTimeout> | null = null
let rmbHoldPointerId: number | null = null
```

- [ ] **步骤 2：添加清除/激活辅助函数**

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
    // 若保留引用则优先复用最近的指针事件；否则仅调用 startDraw——
    // pointerdown 捕获见步骤 3。
  )
  startDraw({ x: clientX, y: clientY })
  logActionEvent('rmb hold erase', { toolBefore: rmbHoldGesture.toolBefore })
}
```

**重要：** 步骤 3 中，`pointerdown` button 2 时在启动 pending 阶段调用 `capturePointer(e)`，保证 move/up 可靠；在闭包中保存 `e.clientX/Y` 供计时回调使用。

- [ ] **步骤 3：处理 button 2 的 `pointerdown`**

修改 `onPointerDown`：button 0 保持现有提前返回，并在 `if (e.button !== 0) return` **之前**或替代位置加入 button-2 分支：

```ts
async function onPointerDown(e: PointerEvent) {
  if (e.button === 2) {
    onRmbPointerDown(e)
    return
  }
  if (e.button !== 0) return
  // ... 既有 LMB 主体不变 ...
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
  // macOS：Ctrl 拖动后的 Control+click 映射为 RMB——跳过（与 onContextMenu 同精神）
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

细化 `activateHoldEraseFromTimer`：使用**当前** `lastPointerX` / `lastPointerY`（`onPointerMove` 中更新），使擦除从激活时刻的指针位置开始，而非仅按下点：

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

超时回调相应改为 `activateHoldEraseFromTimer()`。

- [ ] **步骤 4：扩展 `onPointerMove` / `onPointerUp` 支持 RMB 按住**

在 `onPointerMove` 既有绘制分支之后，确保 `rmbHoldGesture.phase === 'active'` 且 `isDrawing` 时既有的 `draw` / `drawBatch` 路径已运行（`isDrawing` 为 true 即可）。若左键与右键共用 `isDrawing` 路径则无需新分支——验证 `onPointerMove` 是否以 `buttons === 1` 门控。若按主键门控，加入：

```ts
// move 内，绘制中：
if (isDrawing.value && (rmbHoldGesture.phase === 'active' || /* 既有 */ true)) {
  // 既有 drawBatch 路径
}
```

检查现有 `onPointerMove`——若只检查 `isDrawing` / `isDragging`，`startDraw` 执行后 RMB 按住擦除无需改动即可工作。

在 `onPointerUp`：

```ts
function onPointerUp(e: PointerEvent) {
  // 既有 capturedPointerId 不匹配守卫保留

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

  // ... 既有 LMB onPointerUp 主体 ...
}
```

同时在 `abortActivePointerInteraction` 清除按住状态：

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

（若 abort 已调用 `endDraw`，保持顺序：先 endDraw 再恢复工具。）

- [ ] **步骤 5：守卫 `onContextMenu`**

在 `onContextMenu` 顶部（`preventDefault` / 文字处理之后酌情）：

```ts
function onContextMenu(e: MouseEvent) {
  e.preventDefault()
  if (handleTextBoxContextMenu(e)) return
  if (shouldBlockQuickColors(rmbHoldGesture)) return
  if (performance.now() < suppressQuickColorsUntil) return
  // pointerup 已打开色盘时，普通短按不再在此重复打开。
  // 仅作手势已 idle 时的回退（例如文字路径已返回）：
  if (!active.value || penetrationMode.value || isDrawing.value) return
  if (isMacOS() && e.ctrlKey && pointerMovedSinceDown) return
  openQuickColorsAt(e.clientX, e.clientY)
}
```

重构现有主体为调用 `openQuickColorsAt` 以避免重复。**不要**重复打开色盘：短按在 `pointerup` 打开；手势已 idle 时 `contextmenu` 回退可用（`showQuickColors` 已开时二次打开无害——或已开则提前返回）。

- [ ] **步骤 6：无单元框架下的手动健全性检查**

运行：`npx vitest run src/utils/rmbHoldErase.test.ts`

预期：PASS

方便时对触碰文件运行 `npm run lint`。

- [ ] **步骤 7：提交**

```bash
git add src/components/DrawingOverlay.vue
git commit -m "feat(drawing): hold right-click to erase, release restores tool"
```

---

### 任务 3：帮助 / i18n 文案 + 站点帮助同步

**文件：**
- 修改：`src/i18n/en.ts`
- 修改：`src/i18n/zh-CN.ts`
- 修改：`src/components/SettingsView.vue`（仅在需要第二行帮助时）
- 修改：`docs/i18n.js` 与 `docs/help.html`（若记载右键取色）

**接口：**
- 产出：提及右键短按 vs 长按的帮助字符串

- [ ] **步骤 1：更新应用内帮助字符串**

在 `en.ts` help 段，修改/新增：

```ts
rightClickColor: 'Quick color picker',
rightClickErase: 'Hold to erase',
mouseRightClick: 'Right-click',
mouseRightClickHold: 'Hold right-click',
```

或保留一行并扩展标签：

```ts
rightClickColor: 'Colors (tap) / erase (hold)',
```

偏好**两行**：在 `SettingsView.vue` 帮助卡既有颜色行旁（颜色 + 按住擦除），与锁定的 UX 一致。

`zh-CN.ts`：

```ts
rightClickColor: '快速选色（点按）',
rightClickErase: '按住擦除',
mouseRightClickHold: '长按右键',
```

- [ ] **步骤 2：在 `SettingsView.vue` 添加帮助行**

在既有右键取色行旁（约 639 行）加：

```vue
<div class="help-row">
  <span class="help-label">{{ t('help.rightClickErase') }}</span>
  <span class="help-keys-plain">{{ t('help.mouseRightClickHold') }}</span>
</div>
```

（使用相邻行相同的标记模式。）

- [ ] **步骤 3：同步 `docs/i18n.js` / 帮助页**

若存在 `helpPage.draw.rightClick`，更新 EN/ZH 以一句话提及按住擦除，如 EN：`Color picker (click) · Erase (hold)`。

- [ ] **步骤 4：提交**

```bash
git add src/i18n/en.ts src/i18n/zh-CN.ts src/components/SettingsView.vue docs/i18n.js docs/help.html
git commit -m "docs(help): document hold right-click to erase"
```

---

### 任务 4：手动 QA 清单（无代码）

**文件：** 无

- [ ] **步骤 1：运行自动化套件**

运行：`npm test`

预期：PASS（含新 `rmbHoldErase` 测试）

- [ ] **步骤 2：覆盖层手动检查（Windows 和/或 macOS）**

| # | 操作 | 预期 |
|---|--------|----------|
| 1 | 右键短按 | 打开快速色盘；工具不变 |
| 2 | 按住右键 ≥ 250ms 后拖动 | 按当前橡皮擦模式擦除；光标/工具显示橡皮擦 |
| 3 | 按住后松开 | 恢复之前的工具 |
| 4 | 已是橡皮擦时按住 | 擦除；松开后保持橡皮擦 |
| 5 | 打开文本框；按住右键 | 不擦除；双击右键仍确认 |
| 6 | 穿透模式 | 无按住擦除 / 无色盘（既有） |
| 7 | macOS Ctrl 拖动后的 Control+click | 不误触发擦除或色盘 |

- [ ] **步骤 3：在 GitHub issue #41 评论**（发布时）

注明按住擦除已发布；环形菜单因产品调性仍不在范围。

---

## 规格覆盖自审

| 规格要求 | 任务 |
|------------------|------|
| 250ms 阈值 | 任务 1 常量 + 任务 2 计时器 |
| 短按 = 色盘，长按 = 擦除 | 任务 2 松开 / 激活 |
| 无设置 | 所有任务 |
| 文本框禁用按住 | `canStartRmbHoldErase` + 任务 2 |
| pending/active 抑制 contextmenu | 任务 2 `shouldBlockQuickColors` |
| 色盘来自短按 pointerup | 任务 2 |
| 恢复之前的工具 | 任务 1/2 `restoreTool` |
| 帮助文案 | 任务 3 |
| 无环形菜单 | 显式省略 |
| 测试 | 任务 1（+ 任务 4 套件） |

## 占位符扫描

无 TBD /「稍后实现」步骤。覆盖层接线给出了具体函数形态；实现者须对齐 `DrawingOverlay.vue` 中已有的 `capturePointer` / `releaseCapturedPointer` 辅助函数。
