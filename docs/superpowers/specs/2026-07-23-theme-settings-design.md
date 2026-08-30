# 主题设置：深色 / 浅色 / 跟随系统

**日期：** 2026-07-23
**状态：** 已批准设计
**Issue：** [#29](https://github.com/AomeNero/marker/issues/29)
**范围：** 设置窗口 + 浮动界面（工具栏、Space 面板、色盘）；Mac + Windows

## 问题

Marker 的 UI 颜色硬编码为深色（`src/style.css` 的 `rgba`、`macos.rs` 的 `Theme::Dark`）。用户无法切换浅色模式或跟随系统外观。托盘图标行为在 Mac 上友好（`iconAsTemplate`），但 Windows 没有深浅托盘资源切换。

## 决策（已锁定）

| 主题 | 选择 |
|-------|--------|
| 作用面 | 设置窗口 + 浮动面板（工具栏、Space、色盘）——**不含**画布笔迹颜色或白板底色 |
| 默认偏好 | `dark`（向后兼容；老用户无感知） |
| 托盘 | Mac：保持模板自动适配；Windows：按**解析后**主题切换深/浅图标 |
| 跟随系统 | 实时：监听 `prefers-color-scheme` 变化 |
| 实现 | CSS 变量 + `html[data-theme]`（不复制类树，不引主题库） |

## 范围外

- 标注工具栏的自定义强调色 / 色板编辑器
- 画布笔迹颜色、荧光笔不透明度、白板背景
- 独立于 UI 主题的「托盘图标颜色」设置
- 营销站 / 商店截图 HTML 主题

## 架构

```
config.json general.theme: "dark" | "light" | "system"
        │
        ▼
   save_general / get_config / config-changed
        │
        ├─► 前端 useAppTheme → resolve → data-theme + color-scheme
        │         (settings / overlay / toolbar webviews)
        │
        └─► Rust apply_app_theme
                  ├─ macOS: set_theme + SETTINGS_BG light/dark
                  └─ Windows: tray.set_icon(dark|light)
```

### 偏好 vs 解析

- **偏好**（`ThemePreference`）：用户选择的——`dark` | `light` | `system`
- **解析**（`ResolvedTheme`）：实际绘制的——`dark` | `light`
- 前端通过 `matchMedia('(prefers-color-scheme: dark)')` 解析 `system`
- Rust `apply_app_theme(preference)` **同样**解析 `system`（OS / Tauri API）用于原生设置窗与 Windows 托盘——不要只依赖 webview
- `system` 下 OS 外观变化时，前端重新执行 `applyTheme`（更新 CSS）**并**重新调用 `apply_app_theme` 以同步托盘图标

## 配置与 IPC

### Rust（`config.rs`）

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub enum ThemePreference {
    #[default]
    Dark,
    Light,
    System,
}

// GeneralConfig 中：
#[serde(default, rename = "theme")]
pub theme: ThemePreference,
```

- 默认 `Dark`；`#[serde(default)]` 使旧 `config.json` 顺利加载
- `normalized()` 将未知值映射为 `Dark`

### TypeScript（`app.d.ts`）

```typescript
theme?: 'dark' | 'light' | 'system'
```

### 命令

- 沿用既有 `save_general` + `config-changed` 持久化（无新保存命令）
- 新增 `apply_app_theme(preference)` 用于原生窗口 + 托盘更新
- `save_general` 持久化后也调用 `apply_app_theme`，使设置即时生效
- 应用启动：读取一次配置并调用 `apply_app_theme`

## 前端

### `useAppTheme` 组合式函数

- `resolveTheme(preference) → 'dark' | 'light'`
- `applyTheme(preference)` 设置 `document.documentElement.dataset.theme`、`colorScheme`，并调用 `apply_app_theme`
- 仅偏好为 `system` 时启用 `watchSystemTheme`；切到固定深/浅时移除监听

### 接入点

| 界面 | 时机 |
|---------|------|
| `App.vue`（settings 模式） | 提前初始化，使外壳在标签加载前匹配 |
| `DrawingOverlay.vue` | 首次 `get_config` + `config-changed` |
| `ToolbarWindow.vue` | 同上 |
| `SettingsView` / `GeneralTab` | UI 状态 + 保存路径 |

### 设置 UI

- 常规页：「外观」卡片置于语言附近
- 三个 `ui-segment` 选项：深色 / 浅色 / 跟随系统
- 与拖拽模式 / 橡皮擦模式相同的保存模式

### i18n（`en.ts` + `zh-CN.ts`）

- `settings.theme`、`settings.themeDark`、`settings.themeLight`、`settings.themeSystem`

## CSS 令牌（Mac 安全）

### 结构

```css
html[data-theme='dark'] { /* 现有 rgba 1:1 迁移 */ }
html[data-theme='light'] { /* 新浅色调色板 */ }
```

语义类（`.settings-*`、`.overlay-*`、`.ui-*`）保持同名；把硬编码的 `rgba` / hex 替换为 `var(--ui-…)`。

### 令牌分组（约 25–30 个）

| 分组 | 示例 |
|-------|----------|
| 表面 | `--ui-bg`、`--ui-bg-elevated`、`--ui-bg-subtle` |
| 边框 | `--ui-border`、`--ui-border-strong`、`--ui-divider` |
| 文本 | `--ui-text-primary` … `--ui-text-faint` |
| 控件 | `--ui-control-bg`、`--ui-control-bg-hover`、`--ui-control-border` |
| 强调 | `--ui-accent`、`--ui-accent-bg`、`--ui-accent-border` |
| 阴影 | `--ui-shadow-panel`、`--ui-shadow-popover` |

### 规则

- 深色令牌 = 现有硬编码值（默认状态零视觉变化）
- 浅色：浅灰表面 + 深色文本；强调色保持 `#0a84ff`
- 不使用 Tailwind 透明度修饰符（`text-white/45` 等）——为 WebKit 保留显式 `rgba` / 令牌值
- 同步迁移 `SettingsView.vue` 与 `DiagnosticsTab.vue` 中的作用域硬编码

### Mac / 覆盖层细节

| 关注点 | 处理 |
|---------|----------|
| 设置窗外观 | `macos.rs`：停止硬编码 `Theme::Dark`；`configure_settings_window(resolved)` 设置主题 + `SETTINGS_BG_DARK`（`#1e1e20`）/ `SETTINGS_BG_LIGHT`（`#f5f5f7`） |
| 覆盖层 / 工具栏窗口 | 保持透明 `set_background_color(0,0,0,0)`；仅 CSS 面板令牌变化 |
| 圆角渗色 | 保留 `overlay-panel-surface`；浅色 `.overlay-panel` 使用不透明浅色底，而非半透明白 |
| `color-scheme` | 随解析主题在 `html` 上同步 |

### 托盘

- macOS：保持 `iconAsTemplate: true`——无需双份资源
- Windows：现有 `icons/icon.png` = 深色解析主题托盘；新增 `icons/icon-light.png` 用于浅色；在 `apply_app_theme` 中按解析主题切换

## 测试

### 自动化

- Rust：默认值 / serde 往返 / `normalized()` 非法值 → `Dark`
- 前端：`useAppTheme` 解析 + mock `matchMedia` 下的 `dataset.theme` / `colorScheme`

### 手动 QA（macOS + Windows）

- 外观分段即时切换
- 深色 ↔ 浅色：卡片、分段、弹层、帮助表格、诊断文本域
- 系统：OS 主题变化实时更新应用
- Space 面板 / 工具栏 / 色盘
- Mac 设置标题栏 + 无白闪；覆盖层圆角无白边渗色
- Windows 托盘在浅色任务栏上可读；Mac 托盘模板正常
- 语言下拉与快捷键提示不被裁切
- 重启后偏好持久化

## 文件触碰清单

| 层 | 文件 |
|-------|-------|
| CSS | `src/style.css`；`SettingsView.vue`、`DiagnosticsTab.vue` 作用域样式 |
| 前端 | `useAppTheme.ts`（+测试）；`App.vue`；`DrawingOverlay.vue`；`ToolbarWindow.vue`；`GeneralTab.vue`；`app.d.ts`；`en.ts`；`zh-CN.ts` |
| Rust | `config.rs`；新 `theme.rs`；`commands.rs`；`macos.rs`；`lib.rs` |
| 资源 | `src-tauri/icons/icon-light.png`（Windows 托盘） |

## 成功标准

1. 用户可在常规设置选择深色 / 浅色 / 跟随系统
2. 既有安装默认仍为深色
3. 设置 + 浮动界面跟随偏好；画布颜色不变
4. 系统偏好实时更新，无需重启
5. Mac 标题栏 / 背景与 Windows 托盘与解析主题一致
6. 不新增 Tailwind 透明度类；保持 Mac/Windows 的 WebKit/Chromium 表现一致
