# 按住右键擦除

**日期：** 2026-07-23
**状态：** 已批准设计
**Issue：** [#41](https://github.com/AomeNero/marker/issues/41)（部分——仅擦除按住；无环形菜单）
**范围：** 标注覆盖层指针交互（Mac + Windows）

## 问题

标注时高频擦除目前需要切换到橡皮擦（`7` / 工具栏）再切回。右键当前只会打开快速色盘。用户想要一个不丢失短按取色能力的更快擦除手势。

## 决策（已锁定）

| 主题 | 选择 |
|-------|--------|
| 手势 | **按住**右键 ≥ 时间阈值 → 临时橡皮擦；**松开**右键 → 恢复之前的工具 |
| 短按 | 不变：打开快速色盘 |
| 消歧 | **时间阈值**（约 250ms），而非仅拖动距离 |
| 设置 | **始终开启**，无偏好开关 |
| 文字编辑 | 文本框打开时**禁用**（保留双击右键确认） |
| 环形 / 饼状菜单 | **范围外**（与键盘优先的产品调性不符） |

## 范围外

- 可配置的右键模式（取色 vs 擦除）
- 长按环形工具选择器（#41 提案 2）
- 更改左键语义或数字工具快捷键
- 文本框打开时的按住擦除
- 在既有守卫之外更改 macOS Control+点击 拖动规则

## 行为

```
RMB pointerdown
    │
    ├─ 文本框打开 / 穿透 / 未激活 / 快速色盘打开
    │       → 不进入按住擦除（适用既有 contextmenu / 文字规则）
    │
    ├─ 启动按住计时（~250ms）
    │
    ├─ 阈值前 pointerup
    │       → 清除计时；在指针处打开快速色盘（显式打开；
    │         手势期间忽略/抑制 contextmenu）
    │
    └─ 按住达到阈值
            → 记录 toolBeforeRmb
            → currentTool = eraser
            → 本次手势抑制色盘
            → 开始擦除笔迹（后续移动持续擦除）
            → pointerup → 结束笔迹；恢复 toolBeforeRmb
```

### 边界情况

| 情况 | 结果 |
|------|--------|
| 已是橡皮擦 | 按住仍擦除；松开后橡皮擦保持选中 |
| 达到阈值后未移动即松开 | 按 `startDraw` / `endDraw` 既有规则处理微小/空操作笔迹；工具仍恢复 |
| 快速色盘已打开 | 不进入按住擦除（与现有 RMB 守卫一致） |
| 穿透模式 | 不进入按住擦除 |
| macOS Ctrl 拖动后的 Control+点击 | 保留现有「不当作 RMB 色盘」守卫；该路径不启动按住擦除 |
| 修饰键形状工具（`toolBeforeModifier`） | 按住擦除不应与进行中的左键绘制冲突；仅 button=2 路径 |

### 阈值

- 默认 **250ms**（覆盖层内常量 / 小工具函数；可在代码中调整，不是用户设置）。
- 按住擦除激活后，即使后续触发 `contextmenu`，也**不要**为该次按压打开色盘。

## 架构

主界面：`DrawingOverlay.vue` 指针处理（方案与左键绘制 + 现有 `toolBeforeModifier` 临时工具模式对齐）。

```
pointerdown (button === 2)
  → 启动计时 + 置 rmbHoldPending；酌情捕获指针
pointermove（armed / active 期间）
  → 若按住擦除激活：continueDraw
计时触发
  → rmbHoldPending = false; rmbEraseActive = true
  → 记录 toolBeforeRmb; currentTool = eraser
  → 在当前点 startDraw
pointerup / pointercancel / leave (button 2)
  → 清除计时
  → 若 rmbEraseActive: endDraw + 恢复 toolBeforeRmb; 清除标志
  → 否则若是短按（pending 未激活即清除）：
        在指针处打开快速色盘（不要只依赖迟到的 contextmenu）
contextmenu
  → 始终 preventDefault（现状）
  → 若 rmbHoldPending || rmbEraseActive: return（不开色盘）
  → 否则：现有 onContextMenu / 文字双击右键
```

**平台注记：** `contextmenu` 视操作系统可能在按下或松开时触发。`rmbHoldPending` 或 `rmbEraseActive` 期间绝不从 `contextmenu` 打开色盘。短按色盘在提前的 `pointerup` 中显式打开（或仅当完整短按后收到未被抑制的 `contextmenu`）。偏好单一代码路径：**从短按 `pointerup` 打开色盘**，`contextmenu` 在手势期间视为可抑制噪声。

可选的小型纯函数（可测试）：

- 按住状态转移：pending → active → idle；短按 vs 激活
- 常量：`RMB_HOLD_ERASE_MS`（250）

无 Rust / 配置 / IPC 变更。

## UX / 文档

- 帮助 / README：绘制技巧下加一行——「按住右键擦除；松开恢复工具。右键短按仍打开色盘。」
- i18n：en + zh-CN 帮助字符串（若设置帮助列出 RMB，保持双语言同步）。
- Issue #41：只实现这一片；发布时在 issue 评论中注明环形菜单被否决或延后。

## 验收

1. 右键短按（< 阈值）仍打开快速色盘。
2. 按住右键 ≥ 阈值切换橡皮擦光标/工具，拖动按当前橡皮擦模式（轨迹/对象）擦除。
3. 松开右键恢复按住开始时激活的工具。
4. 文本框打开：按住不切橡皮擦；双击右键确认仍可用。
5. 无新设置 UI。
6. Vitest 覆盖阈值 vs 短按抑制标志 / 工具恢复辅助函数（若抽取）；Win + Mac 手动 QA 色盘与擦除手感。

## 非目标提醒

此项**不**完全关闭 #41——只做高效擦除路径。出于产品调性，环形菜单维持否决，除非日后重新评估。
