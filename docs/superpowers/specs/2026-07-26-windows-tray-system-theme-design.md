# Windows 托盘图标跟随任务栏 / 弹出层主题

**日期：** 2026-07-26
**状态：** 已批准设计
**范围：** 仅 Windows 系统托盘图标（macOS 不变）

## 问题

Windows 区分**应用颜色模式**（`AppsUseLightTheme`）与**系统外观**
（`SystemUsesLightTheme`——任务栏、开始菜单、通知溢出 / 托盘弹出层）。

Marker 此前把托盘图形与应用解析主题绑定（或强制永久黑色图标，避免白色图形在浅色弹出层上消失）。
在**深色**任务栏/弹出层与应用主题不同的机器上，托盘图标会错误或难以辨认。

## 决策（已锁定）

| 主题 | 选择 |
|-------|--------|
| 托盘信号 | 始终 `SystemUsesLightTheme`——独立于 `general.theme` |
| 实时更新 | 是——应用运行中即时响应（无需重启） |
| 设置窗口图标（标题栏 + 任务栏按钮） | 与托盘相同的 shell 信号（任务栏对比度优先于应用内外观） |
| 设置 WebView / CSS 主题 | 仍跟随 `general.theme` → `ResolvedTheme` / `AppsUseLightTheme` |
| macOS | 不变——保持 `iconAsTemplate` |
| 托盘颜色的用户设置 | 范围外 |

## 架构

```
SystemUsesLightTheme（注册表）/ 高对比度
        │
        ├─ 启动：install_main_tray 使用 shell 图形
        ├─ apply_windows_shell_icons（托盘 + 设置窗口图标）
        └─ RegNotifyChangeKeyValue 监听 Personalize
                 └─ 明暗变化时重新应用 shell 图标

general.theme → resolve_theme（system 时用 AppsUseLightTheme）
        │
        └─ apply_app_theme
                 ├─ 设置 WebView 主题
                 └─ 同时重新应用 shell 图标（图标不使用 ResolvedTheme）
```

### 图标选择（Windows 托盘 + 设置任务栏按钮）

| `SystemUsesLightTheme` | Shell | 图形 |
|------------------------|-------|--------|
| `0` | 深色任务栏 / 弹出层 | `icon-light.png`（浅色） |
| `1` | 浅色任务栏 / 弹出层 | `icon.png`（深色） |
| 缺失 / 读取错误 | 视为**深色** shell（Windows 回退） | `icon-light.png` |
| 高对比度开启 | 按 `COLOR_MENU` 亮度 | 浅底用深色图形，反之浅色 |

仅使用现有资源；无新 PNG。

### 启动

托盘**不在** `tauri.conf.json` 声明。`install_main_tray` 在 setup 中以
`theme::main_tray_icon()` 构建，使首个绘制的图形已匹配 shell（无配置默认闪烁）。

### 监听器

Personalize 上的 `RegNotifyChangeKeyValue` 可能因无关值（`AppsUseLightTheme` 等）触发。
监听器重新解析 shell 明暗，**仅当该布尔值变化**时调用 `apply_windows_shell_icons`。

## 组件

### `theme.rs`（Windows）

- `windows_system_shell_is_light() -> bool`——高对比度或 `SystemUsesLightTheme`
- `apply_windows_shell_icons(app)`——按 shell 设置托盘 + 设置窗口图标
- `start_windows_tray_theme_watcher(app)`——后台线程：
  1. 打开 `HKCU\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize`
  2. `RegNotifyChangeKeyValue` 监听值变化
  3. 收到通知 → 明暗变化则 `apply_windows_shell_icons` → 重新挂通知
- 设置图标失败：仅 `warn!`；绝不因托盘导致应用启动失败

### `lib.rs` setup

- i18n 之后：`install_main_tray`，随后启动监听器
  （`#[cfg(target_os = "windows")]`）

### `apply_app_theme`

- 应用偏好仅用于设置 WebView / 原生窗口主题
- Windows 图标始终来自 shell 的 `apply_windows_shell_icons`

## 范围外

- 用轮询替代注册表通知
- 前端 `matchMedia` 驱动托盘颜色
- 设置中独立的托盘颜色偏好
- 更改 macOS 托盘行为
- 监听任务栏强调色 / 壁纸颜色（只关心 shell 明与暗）

## 测试

**单元（Rust）：**

- Shell 浅 → 深色图形路径；shell 深 → 浅色图形路径
- 注册表值缺失 → 深色 shell 默认（`icon-light.png`）

**手动（Windows）：**

- 深色弹出层 → 浅色 Marker 托盘图形可见
- 浅色弹出层 → 深色图形可见
- 浅色任务栏 + 深色 Marker 外观 → 设置任务栏按钮使用**深色**图形
- Marker 运行中在个性化里更改任务栏主题 → 托盘与设置图标无需重启即更新
- 更改 Marker 应用主题深 ↔ 浅 → 托盘与设置**图标**保持任务栏信号；设置 UI 颜色仍跟随应用主题

## 验收

1. 托盘与设置任务栏按钮对比度匹配 shell（深 shell → 浅图标，浅 shell → 深图标）。
2. 个性化实时变化无需重启 Marker 即更新 shell 图标。
3. 应用主题偏好绝不强制托盘 / 设置窗口图标。
4. macOS 行为不变。
