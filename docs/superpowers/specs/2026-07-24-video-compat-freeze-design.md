# 视频兼容模式（冻结截图）

**日期：** 2026-07-24
**状态：** 已批准设计
**Issue：** [#30](https://github.com/AomeNero/marker/issues/30)
**范围：** 设置 + 覆盖层激活（Mac + Windows）；Linux 优雅降级

## 问题

透明置顶 WebView 叠在 GPU 加速视频上（如 Chrome/Edge 里的 Bilibili）会与浏览器硬件视频平面冲突。Marker 标注时视频区域绘制为黑色。

长期方案是原生绘制（2.0）。本设计是过渡期的**产品级**修复：冻结一张桌面截图作为不透明覆盖层背景，使真实视频平面在视觉上不再必需。

## 决策（已锁定）

| 主题 | 选择 |
|-------|--------|
| 模式 | 设置项 **视频兼容模式** → 每次进入标注自动捕获冻结帧 |
| 默认 | **开**（`videoCompatMode: true`） |
| 激活 | 设置开启时在 **Hidden → Drawing** 自动触发（不是每次会话的工具栏开关） |
| 捕获时机 | Rust `activate_drawing`，在 `window.show()` **之前** |
| 传递 | 事件 `freeze-frame-ready` 携带 PNG data URL（失败/跳过为 null） |
| 白板进入 | **不**捕获；与现状一致使用白色背景 |
| 穿透 | **允许**；进入穿透**清除**冻结帧 → 恢复透明 |
| 退出穿透 → 绘制 | **不**重新捕获（保持透明直到下一次 Hidden → Drawing） |
| 实现方式 | 共享显示器捕获内核；经事件传递冻结帧；**不要**复用 `copy_screen` 剪贴板路径做冻结 |

## 范围外

- 低帧率实时桌面合成
- 工具栏「刷新冻结帧」
- 原生绘制 / 架构重写（2.0）
- 自动识别视频网站
- 离开穿透后重新捕获
- 新的 Linux 捕获后端（跳过 → 表现为关闭）

## 架构

```
general.videoCompatMode（默认 true）
        │
        ▼
activate_drawing（在 window.show 之前）
  ├─ 若 videoCompatMode 且非白板进入路径：
  │     capture_monitor_png(标注显示器)
  │     emit("freeze-frame-ready", { dataUrl: "data:image/png;base64,..." | null })
  ├─ setup / 显示覆盖层 / 工具栏 / 裁剪光标
  └─ emit overlay-mode-changed "drawing"
        │
        ▼
DrawingOverlay
  ├─ freeze-frame-ready → 全屏底层 <img>（pointer-events: none）
  ├─ whiteboard → 清除冻结帧（或以白色覆盖）
  ├─ penetration → 清除冻结帧
  └─ hidden → 清除冻结帧
```

白板默认进入由前端在 `overlay-mode-changed` 后决定。设置开启时后端仍可能发出冻结帧；前端在应用白板进入时**必须忽略/清除**冻结帧，避免用户在白色底下瞥见冻结帧闪烁（更优：`defaultEntryMode == whiteboard` 且仅为该进入方式时，后端跳过捕获——见边界情况）。

**首选跳过规则（后端）：** 若 `general.defaultEntryMode == whiteboard`，`activate_drawing` 中跳过捕获（前端将立即进入白板）。若用户同一会话稍后退出白板回到屏幕标注，**不要**自动重捕。

## 配置与 IPC

### 配置

| 层 | 变更 |
|-------|-------|
| `config.rs` `GeneralConfig` | `video_compat_mode` ↔ `videoCompatMode`，**默认 `true`**，`#[serde(default = "default_true")]`（普通 `#[serde(default)]` 会把 bool 错误默认为 `false`） |
| `src/types/app.d.ts` | `general.videoCompatMode: boolean` |
| 持久化 | 既有 `save_general` / `get_config` / `config-changed` |

### 事件 / 内部函数

| 名称 | 角色 |
|------|------|
| `freeze-frame-ready` | 载荷 `{ dataUrl: string \| null }`——前端设置或清除底层 |
| `capture_monitor_rgba` / `capture_monitor_png` | `clipboard.rs`（或小型 `capture.rs`）的内部辅助函数；v1 不要求前端 invoke |
| `copy_screen` | 重构为调用共享捕获后写剪贴板（行为不变） |

无新 `save_*` 命令。

## 捕获细节

- **Windows：** 既有 BitBlt 路径 → RGBA/PNG（当前只写 `CF_DIB`；扩展为可返回像素供冻结使用）。
- **macOS：** 捕获标注显示器区域，避免只进系统剪贴板（冻结优先临时文件或内存路径，而非 `screencapture -c`）。
- **工具栏：** 捕获时工具栏可见则复用 `with_toolbar_excluded_from_capture`。覆盖层仍隐藏 → 通常无需排除覆盖层。
- **显示器：** 与绘制裁剪/标注目标相同的显示器选择（光标 / 记忆的覆盖层显示器——与 `remember_and_clip_drawing_monitor` 顺序对齐；捕获可能在裁剪记忆之前运行——使用同一显示器解析规则）。
- **失败：** 记日志 + 发出 `dataUrl: null`；标注照常激活；透明回退。

## 前端

### 设置

- `GeneralTab.vue`：白板与内容卡片旁的开关（与 `preserveDrawings` 相同的开关模式）。
- i18n 键（en + zh-CN），例如：
  - `settings.videoCompatMode`
  - `settings.videoCompatModeDesc`——说明进入即冻结、帮助 GPU 视频网站、穿透会清除冻结帧。

### 覆盖层

- 状态：`freezeFrameUrl: string | null`。
- 渲染：历史/绘制 canvas 之下的全视口底层；`pointer-events: none`。
- 监听：
  - `freeze-frame-ready` → 设置或清除 URL。
  - `overlay-mode-changed` → `hidden` / `penetration` → 清除；进入白板 → 清除。
  - `config-changed` → 更新本地 `videoCompatMode` ref；冻结可见时**关闭** → 立即清除底层；绘制中**开启** → 会话内不重捕。

## 边界情况

| 情况 | 结果 |
|------|--------|
| 设置开，屏幕进入 | 捕获 → 冻结底层 |
| 设置开，白板默认进入 | 跳过捕获（后端）/ 前端清除 |
| 屏幕 → W 白板 | 清除冻结帧；白底 |
| 白板 → 回到屏幕（同会话） | 不重捕；透明 |
| 绘制 → 穿透 | 清除冻结帧 |
| 穿透 → 绘制 | 不重捕 |
| `preserveDrawings` | 笔迹保留；冻结帧每次 Hidden → Drawing 仍刷新 |
| 会话中关闭设置 | 立即清除冻结帧 |
| 捕获失败 | 透明；不阻塞 |
| Linux / 无捕获 | 同捕获失败 |

## 测试

- Rust：默认 `videoCompatMode == true`；缺失 JSON 字段反序列化为 `true`。
- Rust：捕获失败不 panic；重构后 `copy_screen` 仍可用。
- 手动：Bilibili（或类似）开启设置 → 标注显示冻结帧而非黑块；关闭设置 → 黑块可能复现；穿透清除冻结帧；白板进入无冻结帧；工具栏复制屏幕仍正常。

## 非目标提醒

仅过渡修复。不替代 2.0 的原生覆盖层架构。
