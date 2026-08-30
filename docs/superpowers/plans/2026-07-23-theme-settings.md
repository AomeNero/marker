# 主题设置（深色 / 浅色 / 跟随系统）实现计划

> **致代理执行者：** 必需子技能：使用 superpowers:subagent-driven-development（推荐）或 superpowers:executing-plans 按任务执行本计划。步骤使用复选框（`- [ ]`）语法跟踪。

**目标：** 新增深色 / 浅色 / 跟随系统的外观设置，使设置 UI 与浮动界面（工具栏、Space 面板、色盘）跟随偏好，同时 Mac 标题栏/背景与 Windows 托盘图标保持同步。

**架构：** 持久化 `general.theme`（`dark` | `light` | `system`，默认 `dark`）。前端 `useAppTheme` 解析偏好 → `html[data-theme]` + `color-scheme`，`system` 时监听 `prefers-color-scheme`。CSS 语义类使用共享 `--ui-*` 令牌。Rust `apply_app_theme` 为原生设置外观（macOS）与 Windows 托盘图标解析同一偏好。

**技术栈：** Vue 3、Vitest、Tauri 2（Rust）、`src/style.css` 语义 rgba 类、经 `save_general` 的 `config.json`

## 全局约束

- 作用面：仅设置窗口 + 浮动界面——**不含**画布笔迹颜色或白板底色
- 默认偏好：`dark`（既有安装不变）
- 托盘：macOS 保持 `iconAsTemplate: true`；Windows 按**解析**主题切换 `icon.png` / `icon-light.png`
- 跟随系统：经 `matchMedia('(prefers-color-scheme: dark)')` 实时；变化时重新应用 CSS **并**重新调用 `apply_app_theme`
- 不使用 Tailwind 透明度修饰符（`text-white/45` 等）——仅显式 `rgba` / CSS 变量（Mac WebKit）
- 新配置字段必须使用 `#[serde(default)]`；保持 TS `AppConfig` 同步
- i18n：`en.ts` 与 `zh-CN.ts` 双语加键
- 规格：`docs/superpowers/specs/2026-07-23-theme-settings-design.md`

---

## 文件映射

| 文件 | 角色 |
|------|------|
| `src-tauri/src/config.rs` | `ThemePreference` 枚举 + `general.theme` 字段 + 测试 |
| `src-tauri/src/theme.rs` | 解析偏好；应用原生主题 + Windows 托盘 |
| `src-tauri/src/commands.rs` | `apply_app_theme`；由 `save_general` 调用 |
| `src-tauri/src/macos.rs` | 主题感知的设置窗口背景 / `set_theme` |
| `src-tauri/src/win32.rs`（或 `theme.rs`） | Windows 系统深色检测 + 托盘图标切换 |
| `src-tauri/src/lib.rs` | `mod theme`；注册命令；setup `apply_app_theme` |
| `src-tauri/icons/icon-light.png` | 解析为浅色时的 Windows 托盘图标 |
| `src/types/app.d.ts` | `theme?: 'dark' \| 'light' \| 'system'` |
| `src/composables/useAppTheme.ts` | 解析 / 应用 / 监听系统 |
| `src/composables/useAppTheme.test.ts` | 单元测试 |
| `src/style.css` | `--ui-*` 令牌 + 迁移语义类 |
| `src/components/settings/GeneralTab.vue` | 外观分段 UI |
| `src/components/SettingsView.vue` | 主题状态 + 硬编码 → 令牌 |
| `src/components/settings/DiagnosticsTab.vue` | 文本域颜色 → 令牌 |
| `src/App.vue` | 设置模式提前主题初始化 |
| `src/components/DrawingOverlay.vue` | 配置加载 / 变化时的主题 |
| `src/components/ToolbarWindow.vue` | 同上 |
| `src/i18n/en.ts`、`src/i18n/zh-CN.ts` | 外观字符串 |

---

### 任务 1：配置——`ThemePreference` + TypeScript 类型

**文件：**
- 修改：`src-tauri/src/config.rs`
- 修改：`src/types/app.d.ts`
- 测试：`src-tauri/src/config.rs` `#[cfg(test)]`

**接口：**
- 产出：`ThemePreference { Dark, Light, System }`（serde `camelCase`：`dark`/`light`/`system`）；`GeneralConfig.theme: ThemePreference` 默认 `Dark`；`normalized()` 将未知值钳制为 `Dark`

- [ ] **步骤 1：编写失败的 Rust 测试**

加入 `config.rs` 测试模块：

```rust
#[test]
fn theme_defaults_to_dark() {
    let config = AppConfig::default();
    assert_eq!(config.general.theme, ThemePreference::Dark);
}

#[test]
fn theme_deserializes_missing_as_dark() {
    let json = r#"{
        "shortcuts": {
            "toggleDrawing": "Ctrl+Shift+D",
            "clearDrawing": "Ctrl+Shift+C"
        },
        "general": {}
    }"#;
    let config: AppConfig = serde_json::from_str(json).unwrap();
    assert_eq!(config.general.theme, ThemePreference::Dark);
}

#[test]
fn theme_roundtrip_light_and_system() {
    for theme in [ThemePreference::Light, ThemePreference::System] {
        let mut config = AppConfig::default();
        config.general.theme = theme;
        let json = serde_json::to_string(&config).unwrap();
        let parsed: AppConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.general.theme, theme);
    }
    let light_json = serde_json::to_string(&AppConfig {
        general: GeneralConfig {
            theme: ThemePreference::Light,
            ..GeneralConfig::default()
        },
        ..AppConfig::default()
    })
    .unwrap();
    assert!(light_json.contains("\"theme\":\"light\""));
}

#[test]
fn normalized_preserves_system_theme() {
    let g = GeneralConfig {
        theme: ThemePreference::System,
        ..GeneralConfig::default()
    };
    assert_eq!(g.normalized().theme, ThemePreference::System);
}
```

闭合枚举：非法 JSON 主题字符串反序列化失败；`load_config` 已回退 `AppConfig::default()`（深色）。

- [ ] **步骤 2：运行测试——预期失败**

```bash
cd src-tauri
cargo test theme_defaults_to_dark theme_deserializes_missing_as_dark theme_roundtrip_light_and_system -- --nocapture
```

预期：编译错误——找不到 `ThemePreference`。

- [ ] **步骤 3：实现 `ThemePreference` 并接入 `GeneralConfig`**

在 `config.rs` 其他枚举附近（`GeneralConfig` 之前）：

```rust
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub enum ThemePreference {
    #[default]
    Dark,
    Light,
    System,
}
```

在 `GeneralConfig` 加字段（与其他 general 字段一起）：

```rust
#[serde(default, rename = "theme")]
pub theme: ThemePreference,
```

在 `Default for GeneralConfig`：

```rust
theme: ThemePreference::Dark,
```

`normalized()` 中闭合枚举无需钳制——保持原样（若日后存字符串再补 match 断言）。

顺手在既有 `default_config_roundtrip` 断言：

```rust
assert_eq!(parsed.general.theme, ThemePreference::Dark);
```

- [ ] **步骤 4：更新 TypeScript 类型**

在 `src/types/app.d.ts` 的 `general` 内：

```typescript
theme?: 'dark' | 'light' | 'system'
```

- [ ] **步骤 5：运行 Rust 测试——预期通过**

```bash
cd src-tauri
cargo test theme_ -- --nocapture
```

预期：PASS。

- [ ] **步骤 6：提交**

```bash
git add src-tauri/src/config.rs src/types/app.d.ts
git commit -m "feat(config): add general.theme preference (dark/light/system)"
```

---

### 任务 2：Rust `theme` 模块——解析 + 原生应用 + IPC

**文件：**
- 创建：`src-tauri/src/theme.rs`
- 修改：`src-tauri/src/macos.rs`（`SETTINGS_BG`、`style_settings_builder`、`configure_settings_window`）
- 修改：`src-tauri/src/commands.rs`
- 修改：`src-tauri/src/lib.rs`
- 创建：`src-tauri/icons/icon-light.png`（浅色任务栏上的深色图形；与 `icon.png` 同像素尺寸）

**接口：**
- 消费：来自 `config` 的 `ThemePreference`
- 产出：
  - `ResolvedTheme { Dark, Light }`
  - `resolve_theme(preference: &ThemePreference) -> ResolvedTheme`
  - `apply_app_theme(app: &AppHandle, preference: &ThemePreference)`
  - `#[tauri::command] apply_app_theme(app, preference: ThemePreference)`
  - `save_general` 保存后调用 `apply_app_theme`
  - setup 在配置加载后调用 `apply_app_theme`

- [ ] **步骤 1：新增 Windows 专属 `winreg` 依赖**

在 `src-tauri/Cargo.toml`：

```toml
[target.'cfg(target_os = "windows")'.dependencies]
winreg = "0.55"
```

- [ ] **步骤 2：新增 `theme.rs`（解析 + 单元测试）**

```rust
use crate::config::ThemePreference;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use tracing::warn;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ResolvedTheme {
    Dark,
    Light,
}

pub fn resolve_theme(preference: &ThemePreference) -> ResolvedTheme {
    match preference {
        ThemePreference::Dark => ResolvedTheme::Dark,
        ThemePreference::Light => ResolvedTheme::Light,
        ThemePreference::System => {
            if system_prefers_dark() {
                ResolvedTheme::Dark
            } else {
                ResolvedTheme::Light
            }
        }
    }
}

fn system_prefers_dark() -> bool {
    #[cfg(target_os = "windows")]
    {
        windows_apps_use_dark_theme()
    }
    #[cfg(target_os = "macos")]
    {
        crate::macos::system_appearance_is_dark()
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        true
    }
}

/// `HKCU\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize` 下的
/// `AppsUseLightTheme` DWORD——`1` = 浅色应用，`0` = 深色。键缺失 → 视为深色。
#[cfg(target_os = "windows")]
fn windows_apps_use_dark_theme() -> bool {
    use winreg::enums::HKEY_CURRENT_USER;
    use winreg::RegKey;
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let Ok(key) = hkcu.open_subkey(
        r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize",
    ) else {
        return true;
    };
    let light: u32 = key.get_value("AppsUseLightTheme").unwrap_or(0);
    light == 0
}

pub fn apply_app_theme(app: &AppHandle, preference: &ThemePreference) {
    let resolved = resolve_theme(preference);

    if let Some(win) = app.get_webview_window("settings") {
        #[cfg(target_os = "macos")]
        crate::macos::configure_settings_window(&win, resolved);
        #[cfg(not(target_os = "macos"))]
        {
            let _ = win.set_theme(Some(match resolved {
                ResolvedTheme::Dark => tauri::Theme::Dark,
                ResolvedTheme::Light => tauri::Theme::Light,
            }));
        }
    }

    #[cfg(target_os = "windows")]
    if let Err(e) = update_windows_tray_icon(app, resolved) {
        warn!("Failed to update tray icon: {}", e);
    }
}

#[cfg(target_os = "windows")]
fn update_windows_tray_icon(
    app: &AppHandle,
    resolved: ResolvedTheme,
) -> Result<(), Box<dyn std::error::Error>> {
    use tauri::image::Image;
    let Some(tray) = app.tray_by_id("main") else {
        return Ok(());
    };
    let bytes: &[u8] = match resolved {
        ResolvedTheme::Dark => include_bytes!("../icons/icon.png"),
        ResolvedTheme::Light => include_bytes!("../icons/icon-light.png"),
    };
    let icon = Image::from_bytes(bytes)?;
    tray.set_icon(Some(icon))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_dark_and_light_are_fixed() {
        assert_eq!(
            resolve_theme(&ThemePreference::Dark),
            ResolvedTheme::Dark
        );
        assert_eq!(
            resolve_theme(&ThemePreference::Light),
            ResolvedTheme::Light
        );
    }
}
```

- [ ] **步骤 3：更新 macOS 设置外观 + 系统探测**

在 `macos.rs`，替换 `SETTINGS_BG` 并更新配置辅助函数。用既有 `objc_msgSend` 模式新增外观探测：

```rust
use crate::theme::ResolvedTheme;

const SETTINGS_BG_DARK: Color = Color(30, 30, 32, 255); // #1e1e20
const SETTINGS_BG_LIGHT: Color = Color(245, 245, 247, 255); // #f5f5f7

/// 从 `NSUserDefaults` 读取 `AppleInterfaceStyle`（`"Dark"` → 深色）。
pub fn system_appearance_is_dark() -> bool {
    unsafe {
        extern "C" {
            fn objc_getClass(name: *const std::ffi::c_char) -> *mut c_void;
            fn sel_registerName(name: *const std::ffi::c_char) -> Sel;
        }
        let ns_user_defaults = objc_getClass(c"NSUserDefaults".as_ptr());
        if ns_user_defaults.is_null() {
            return true;
        }
        let standard_sel = sel_registerName(c"standardUserDefaults".as_ptr());
        let defaults = msg_send_ptr(ns_user_defaults, standard_sel);
        if defaults.is_null() {
            return true;
        }
        // 以 CFString/NSString "AppleInterfaceStyle" 调 stringForKey:
        // 多数托盘应用采用的简单可靠检查：
        extern "C" {
            fn CFPreferencesCopyAppValue(
                key: *const c_void,
                app_id: *const c_void,
            ) -> *mut c_void;
            fn CFRelease(cf: *const c_void);
            fn CFStringCreateWithCString(
                alloc: *const c_void,
                cStr: *const std::ffi::c_char,
                encoding: u32,
            ) -> *mut c_void;
            fn CFStringCompare(
                theString1: *const c_void,
                theString2: *const c_void,
                compareOptions: u64,
            ) -> i32;
        }
        const K_CF_STRING_ENCODING_UTF8: u32 = 0x08000100;
        let key = CFStringCreateWithCString(
            std::ptr::null(),
            c"AppleInterfaceStyle".as_ptr(),
            K_CF_STRING_ENCODING_UTF8,
        );
        let app = CFStringCreateWithCString(
            std::ptr::null(),
            c"Apple Global Domain".as_ptr(),
            K_CF_STRING_ENCODING_UTF8,
        );
        let value = CFPreferencesCopyAppValue(key, app);
        CFRelease(key);
        CFRelease(app);
        if value.is_null() {
            return false; // 缺失 → 浅色（macOS 历史默认）
        }
        let dark = CFStringCreateWithCString(
            std::ptr::null(),
            c"Dark".as_ptr(),
            K_CF_STRING_ENCODING_UTF8,
        );
        let cmp = CFStringCompare(value, dark, 0);
        CFRelease(dark);
        CFRelease(value);
        cmp == 0 // kCFCompareEqualTo
    }
}

pub fn style_settings_builder(
    builder: tauri::WebviewWindowBuilder<'_, tauri::Wry, AppHandle>,
) -> tauri::WebviewWindowBuilder<'_, tauri::Wry, AppHandle> {
    builder
        .title_bar_style(TitleBarStyle::Transparent)
        .theme(Some(Theme::Dark))
        .background_color(SETTINGS_BG_DARK)
}

pub fn configure_settings_window(window: &WebviewWindow, resolved: ResolvedTheme) {
    let (theme, bg) = match resolved {
        ResolvedTheme::Dark => (Theme::Dark, SETTINGS_BG_DARK),
        ResolvedTheme::Light => (Theme::Light, SETTINGS_BG_LIGHT),
    };
    window.set_theme(Some(theme)).ok();
    window.set_background_color(Some(bg)).ok();
}
```

更新 `lib.rs` 构建设置窗口时的调用点：

```rust
let preference = lock_or_recover(&app.state::<AppState>().config)
    .general
    .theme
    .clone();
macos::configure_settings_window(&window, theme::resolve_theme(&preference));
```

- [ ] **步骤 4：创建 `icon-light.png`**

- 来源：`src-tauri/icons/icon.png`
- 产出**深色透明底**（或深色单色）托盘图形，尺寸与 `icon.png` 相同，在浅色 Windows 任务栏上可读
- 保存为 `src-tauri/icons/icon-light.png`
- 不改 `tauri.conf.json` 的 `trayIcon.iconAsTemplate`（Mac 保持 `true`）
- 若用脚本生成（如 ImageMagick）：加深/阈值化图形使其在 `#f3f3f3` 任务栏上保持可见；浅色主题资源不要用纯白图形

- [ ] **步骤 5：命令 + 接线 `save_general` + setup**

在 `commands.rs`：

```rust
#[tauri::command]
pub fn apply_app_theme(
    app: AppHandle,
    preference: crate::config::ThemePreference,
) -> AppResult<()> {
    crate::theme::apply_app_theme(&app, &preference);
    Ok(())
}
```

`save_general` 末尾、emit 之后：

```rust
crate::theme::apply_app_theme(&app, &snapshot.general.theme);
```

在 `lib.rs`：

```rust
mod theme;
// generate_handler![..., commands::apply_app_theme]
```

setup 中 `load_config` / 赋值 state 之后：

```rust
theme::apply_app_theme(&handle, &loaded.general.theme);
```

- [ ] **步骤 6：编译 + 测试**

```bash
cd src-tauri
cargo test theme:: -- --nocapture
cargo clippy -- -D warnings
```

预期：PASS / 无警告。

- [ ] **步骤 7：提交**

```bash
git add src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/src/theme.rs src-tauri/src/macos.rs src-tauri/src/commands.rs src-tauri/src/lib.rs src-tauri/icons/icon-light.png
git commit -m "feat(theme): apply native settings chrome and Windows tray icons"
```

---

### 任务 3：前端 `useAppTheme` 组合式函数

**文件：**
- 创建：`src/composables/useAppTheme.ts`
- 创建：`src/composables/useAppTheme.test.ts`

**接口：**
- 消费：`invoke('apply_app_theme', { preference })`
- 产出：
  - `export type ThemePreference = 'dark' | 'light' | 'system'`
  - `export type ResolvedTheme = 'dark' | 'light'`
  - `resolveTheme(preference: ThemePreference): ResolvedTheme`
  - `applyTheme(preference: ThemePreference): ResolvedTheme`
  - `watchSystemTheme(preference: () => ThemePreference, onResolved?: (r: ResolvedTheme) => void): () => void`

- [ ] **步骤 1：编写失败测试**

```typescript
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { resolveTheme, applyTheme, watchSystemTheme } from './useAppTheme'

const invoke = vi.fn()
vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => invoke(...args),
}))

function mockMatchMedia(matchesDark: boolean) {
  const listeners: Array<(e: MediaQueryListEvent) => void> = []
  const mql = {
    matches: matchesDark,
    media: '(prefers-color-scheme: dark)',
    addEventListener: (_: string, cb: (e: MediaQueryListEvent) => void) => {
      listeners.push(cb)
    },
    removeEventListener: (_: string, cb: (e: MediaQueryListEvent) => void) => {
      const i = listeners.indexOf(cb)
      if (i >= 0) listeners.splice(i, 1)
    },
    dispatch(matches: boolean) {
      mql.matches = matches
      listeners.forEach((cb) => cb({ matches } as MediaQueryListEvent))
    },
  }
  vi.stubGlobal('matchMedia', () => mql)
  return mql
}

describe('useAppTheme', () => {
  beforeEach(() => {
    invoke.mockReset()
    invoke.mockResolvedValue(undefined)
    document.documentElement.dataset.theme = ''
    document.documentElement.style.colorScheme = ''
  })

  afterEach(() => {
    vi.unstubAllGlobals()
  })

  it('resolveTheme maps fixed preferences', () => {
    expect(resolveTheme('dark')).toBe('dark')
    expect(resolveTheme('light')).toBe('light')
  })

  it('resolveTheme system follows matchMedia', () => {
    mockMatchMedia(true)
    expect(resolveTheme('system')).toBe('dark')
    mockMatchMedia(false)
    expect(resolveTheme('system')).toBe('light')
  })

  it('applyTheme sets dataset and color-scheme and invokes Rust', async () => {
    mockMatchMedia(true)
    const resolved = await applyTheme('light')
    expect(resolved).toBe('light')
    expect(document.documentElement.dataset.theme).toBe('light')
    expect(document.documentElement.style.colorScheme).toBe('light')
    expect(invoke).toHaveBeenCalledWith('apply_app_theme', { preference: 'light' })
  })

  it('watchSystemTheme re-applies when OS theme changes', async () => {
    const mql = mockMatchMedia(true)
    const stop = watchSystemTheme(() => 'system')
    mql.dispatch(false)
    await Promise.resolve()
    expect(document.documentElement.dataset.theme).toBe('light')
    expect(invoke).toHaveBeenCalledWith('apply_app_theme', { preference: 'system' })
    stop()
  })
})
```

- [ ] **步骤 2：运行测试——预期失败**

```bash
npm test -- src/composables/useAppTheme.test.ts
```

预期：FAIL——模块未找到。

- [ ] **步骤 3：实现组合式函数**

```typescript
import { invoke } from '@tauri-apps/api/core'

export type ThemePreference = 'dark' | 'light' | 'system'
export type ResolvedTheme = 'dark' | 'light'

function systemPrefersDark(): boolean {
  return window.matchMedia('(prefers-color-scheme: dark)').matches
}

export function resolveTheme(preference: ThemePreference): ResolvedTheme {
  if (preference === 'system') return systemPrefersDark() ? 'dark' : 'light'
  return preference
}

export async function applyTheme(preference: ThemePreference): Promise<ResolvedTheme> {
  const resolved = resolveTheme(preference)
  document.documentElement.dataset.theme = resolved
  document.documentElement.style.colorScheme = resolved
  try {
    await invoke('apply_app_theme', { preference })
  } catch (error) {
    console.error('Failed to apply native theme:', error)
  }
  return resolved
}

export function watchSystemTheme(
  getPreference: () => ThemePreference,
  onResolved?: (resolved: ResolvedTheme) => void,
): () => void {
  const mql = window.matchMedia('(prefers-color-scheme: dark)')
  const handler = () => {
    if (getPreference() !== 'system') return
    void applyTheme('system').then((resolved) => onResolved?.(resolved))
  }
  mql.addEventListener('change', handler)
  return () => mql.removeEventListener('change', handler)
}
```

- [ ] **步骤 4：运行测试——预期通过**

```bash
npm test -- src/composables/useAppTheme.test.ts
```

预期：PASS。

- [ ] **步骤 5：提交**

```bash
git add src/composables/useAppTheme.ts src/composables/useAppTheme.test.ts
git commit -m "feat(theme): add useAppTheme resolve/apply/watch helpers"
```

---

### 任务 4：CSS 令牌——深色迁移 + 浅色调色板

**文件：**
- 修改：`src/style.css`

**接口：**
- 产出：`html[data-theme='dark']` 与 `html[data-theme='light']`（属性缺失时以深色回退）定义 `--ui-*` 令牌；全部 `.settings-*` / `.overlay-*` / `.ui-*` 外壳颜色使用 `var(--ui-…)`

- [ ] **步骤 1：在 `@theme { … }` 后 / `@layer base` 前插入令牌块**

深色值**精确匹配**当前硬编码 rgba/hex。浅色值：浅灰表面 + 深色文本；强调色保持 `rgb(10, 132, 255)`。

```css
/* 主题令牌——显式 rgba 保证 macOS WebKit 一致性 */
html,
html[data-theme='dark'] {
  color-scheme: dark;
  --ui-bg: #1e1e20;
  --ui-bg-sidebar: #161618;
  --ui-bg-elevated: #2a2a2c;
  --ui-bg-subtle: rgba(255, 255, 255, 0.02);
  --ui-bg-subtle-hover: rgba(255, 255, 255, 0.03);
  --ui-border: rgba(255, 255, 255, 0.05);
  --ui-border-strong: rgba(255, 255, 255, 0.08);
  --ui-border-panel: rgba(255, 255, 255, 0.08);
  --ui-divider: rgba(255, 255, 255, 0.05);
  --ui-text-brand: rgba(255, 255, 255, 0.85);
  --ui-text-title: rgba(255, 255, 255, 0.75);
  --ui-text-heading: rgba(255, 255, 255, 0.85);
  --ui-text-label: rgba(255, 255, 255, 0.7);
  --ui-text-value: rgba(255, 255, 255, 0.65);
  --ui-text-muted: rgba(255, 255, 255, 0.45);
  --ui-text-subtle: rgba(255, 255, 255, 0.4);
  --ui-text-faint: rgba(255, 255, 255, 0.3);
  --ui-text-dim: rgba(255, 255, 255, 0.25);
  --ui-text-footer: rgba(255, 255, 255, 0.32);
  --ui-text-body: rgba(255, 255, 255, 0.5);
  --ui-text-icon: rgba(255, 255, 255, 0.35);
  --ui-text-icon-hover: rgba(255, 255, 255, 0.6);
  --ui-control-bg: rgba(255, 255, 255, 0.06);
  --ui-control-bg-hover: rgba(255, 255, 255, 0.1);
  --ui-control-border: rgba(255, 255, 255, 0.08);
  --ui-control-text: rgba(255, 255, 255, 0.65);
  --ui-kbd-bg: rgba(255, 255, 255, 0.1);
  --ui-kbd-border: rgba(255, 255, 255, 0.1);
  --ui-kbd-text: rgba(255, 255, 255, 0.7);
  --ui-accent: rgb(10, 132, 255);
  --ui-accent-soft: rgba(10, 132, 255, 0.15);
  --ui-accent-border: rgba(10, 132, 255, 0.4);
  --ui-accent-bg-active: rgba(10, 132, 255, 0.3);
  --ui-shadow-panel: 0 24px 48px rgba(0, 0, 0, 0.45), 0 4px 16px rgba(0, 0, 0, 0.25),
    inset 0 0.5px 0 rgba(255, 255, 255, 0.08);
  --ui-shadow-popover: 0 8px 32px rgba(0, 0, 0, 0.5);
  --ui-nav-text: rgba(255, 255, 255, 0.4);
  --ui-nav-text-hover: rgba(255, 255, 255, 0.6);
  --ui-nav-text-active: rgba(255, 255, 255, 0.9);
  --ui-nav-bg-hover: rgba(255, 255, 255, 0.05);
  --ui-nav-bg-active: rgba(255, 255, 255, 0.1);
  --ui-toggle-off: rgba(255, 255, 255, 0.2);
  --ui-swatch-ring: rgba(255, 255, 255, 0.1);
  --ui-swatch-ring-active: rgba(255, 255, 255, 0.75);
}

html[data-theme='light'] {
  color-scheme: light;
  --ui-bg: #f5f5f7;
  --ui-bg-sidebar: #ebebef;
  --ui-bg-elevated: #ffffff;
  --ui-bg-subtle: rgba(0, 0, 0, 0.02);
  --ui-bg-subtle-hover: rgba(0, 0, 0, 0.04);
  --ui-border: rgba(0, 0, 0, 0.06);
  --ui-border-strong: rgba(0, 0, 0, 0.1);
  --ui-border-panel: rgba(0, 0, 0, 0.1);
  --ui-divider: rgba(0, 0, 0, 0.06);
  --ui-text-brand: rgba(0, 0, 0, 0.88);
  --ui-text-title: rgba(0, 0, 0, 0.82);
  --ui-text-heading: rgba(0, 0, 0, 0.88);
  --ui-text-label: rgba(0, 0, 0, 0.75);
  --ui-text-value: rgba(0, 0, 0, 0.7);
  --ui-text-muted: rgba(0, 0, 0, 0.5);
  --ui-text-subtle: rgba(0, 0, 0, 0.45);
  --ui-text-faint: rgba(0, 0, 0, 0.35);
  --ui-text-dim: rgba(0, 0, 0, 0.28);
  --ui-text-footer: rgba(0, 0, 0, 0.38);
  --ui-text-body: rgba(0, 0, 0, 0.55);
  --ui-text-icon: rgba(0, 0, 0, 0.4);
  --ui-text-icon-hover: rgba(0, 0, 0, 0.65);
  --ui-control-bg: rgba(0, 0, 0, 0.04);
  --ui-control-bg-hover: rgba(0, 0, 0, 0.08);
  --ui-control-border: rgba(0, 0, 0, 0.1);
  --ui-control-text: rgba(0, 0, 0, 0.7);
  --ui-kbd-bg: rgba(0, 0, 0, 0.06);
  --ui-kbd-border: rgba(0, 0, 0, 0.1);
  --ui-kbd-text: rgba(0, 0, 0, 0.7);
  --ui-accent: rgb(10, 132, 255);
  --ui-accent-soft: rgba(10, 132, 255, 0.12);
  --ui-accent-border: rgba(10, 132, 255, 0.45);
  --ui-accent-bg-active: rgba(10, 132, 255, 0.22);
  --ui-shadow-panel: 0 24px 48px rgba(0, 0, 0, 0.12), 0 4px 16px rgba(0, 0, 0, 0.08),
    inset 0 0.5px 0 rgba(255, 255, 255, 0.8);
  --ui-shadow-popover: 0 8px 32px rgba(0, 0, 0, 0.14);
  --ui-nav-text: rgba(0, 0, 0, 0.45);
  --ui-nav-text-hover: rgba(0, 0, 0, 0.65);
  --ui-nav-text-active: rgba(0, 0, 0, 0.9);
  --ui-nav-bg-hover: rgba(0, 0, 0, 0.04);
  --ui-nav-bg-active: rgba(0, 0, 0, 0.08);
  --ui-toggle-off: rgba(0, 0, 0, 0.18);
  --ui-swatch-ring: rgba(0, 0, 0, 0.12);
  --ui-swatch-ring-active: rgba(0, 0, 0, 0.55);
}
```

迁移过程中按需补充更多令牌（状态绿/红、覆盖层工具按钮文本等）——同一「深色=现值 / 浅色=反转透明度」模式。

- [ ] **步骤 2：更新 `@layer base` 的设置背景**

将硬编码 `#1e1e20` / `color-scheme: dark` 替换为：

```css
html.settings,
html.settings body {
  height: 100%;
  background: var(--ui-bg) !important;
}

html.settings #app {
  height: 100%;
  background: var(--ui-bg);
}
```

（`color-scheme` 由 `html` 上的令牌块提供。）

- [ ] **步骤 3：语义类迁移到 `var(--ui-*)`**

重写每个硬编码白/黑 rgba 的外壳类。示例：

```css
.settings-card {
  border: 1px solid var(--ui-border);
  background: var(--ui-bg-subtle);
  /* … */
}
.overlay-panel {
  background: var(--ui-bg);
  border: 1px solid var(--ui-border-panel);
  box-shadow: var(--ui-shadow-panel);
}
.settings-text-label {
  color: var(--ui-text-label);
}
.ui-segment--active {
  border-color: var(--ui-accent-border);
  background: var(--ui-accent-soft);
  color: var(--ui-accent);
}
```

赞助金色调色板在深色下保留为特殊 `--ui-credits-*` 令牌并配浅色变体（浅底上稍深的金色）——不得残留仅白色的文本。

**本任务验收：** `data-theme="dark"`（默认）时设置/覆盖层与当前 master 视觉一致；DevTools 切 `data-theme="light"` 时表面/文本翻转。

- [ ] **步骤 4：grep 组件中遗留的 Tailwind 透明度反模式**

```bash
rg "border-white/|text-white/|bg-white/|bg-\[#1e1e20\]|bg-\[#161618\]" src/components src/style.css
```

预期：零（或仅刻意的非主题画布部分）。若 Vue 模板命中，在任务 5/6 修复。

- [ ] **步骤 5：提交**

```bash
git add src/style.css
git commit -m "ui(theme): introduce dark/light CSS tokens for settings and overlay chrome"
```

---

### 任务 5：设置 UI + i18n + 提前应用

**文件：**
- 修改：`src/i18n/en.ts`、`src/i18n/zh-CN.ts`
- 修改：`src/components/settings/GeneralTab.vue`
- 修改：`src/components/SettingsView.vue`
- 修改：`src/App.vue`
- 修改：`src/components/settings/DiagnosticsTab.vue`（作用域文本域颜色）

**接口：**
- 消费：`ThemePreference`、`applyTheme`、`watchSystemTheme`、`save_general`
- 产出：常规页外观分段；设置外壳在标签加载前完成主题

- [ ] **步骤 1：新增 i18n 键**

`en.ts`（`settings` 内）：

```typescript
theme: 'Appearance',
themeDark: 'Dark',
themeLight: 'Light',
themeSystem: 'System',
```

`zh-CN.ts`：

```typescript
theme: '外观',
themeDark: '深色',
themeLight: '浅色',
themeSystem: '跟随系统',
```

- [ ] **步骤 2：接线 `GeneralTab` 外观卡片**

Props + emit：

```typescript
theme: ThemePreference
// emit 'update:theme': [value: ThemePreference]
```

从 `useAppTheme` 导入类型。在语言卡片**正下方**放置新的 `settings-card`：

```vue
<div class="settings-card">
  <div class="settings-card-row">
    <span class="settings-text-label">{{ t('settings.theme') }}</span>
    <div class="flex items-center gap-1">
      <button
        v-for="opt in themeOptions"
        :key="opt"
        type="button"
        class="px-2 py-1 rounded-md ui-segment leading-none transition-colors duration-120 whitespace-nowrap"
        :class="{ 'ui-segment--active': theme === opt }"
        @click="setTheme(opt)"
      >
        {{ t(`settings.theme${opt === 'dark' ? 'Dark' : opt === 'light' ? 'Light' : 'System'}`) }}
      </button>
    </div>
  </div>
</div>
```

`setTheme` 仿照 `setDragMode`：

```typescript
async function setTheme(next: ThemePreference) {
  if (next === props.theme) return
  emit('update:theme', next)
  await applyTheme(next)
  try {
    const cfg = await invoke<AppConfig>('get_config')
    if (!cfg.general) { /* 以 theme 初始化最小 general */ }
    cfg.general.theme = next
    await invoke('save_general', { general: cfg.general })
  } catch (error) {
    console.error('Failed to save theme:', error)
  }
}
```

- [ ] **步骤 3：接线 `SettingsView`**

```typescript
import { applyTheme, watchSystemTheme, type ThemePreference } from '../composables/useAppTheme'

const theme = ref<ThemePreference>('dark')
let stopThemeWatch: (() => void) | null = null

function resolveThemePref(general?: AppConfig['general']): ThemePreference {
  const t = general?.theme
  return t === 'light' || t === 'system' || t === 'dark' ? t : 'dark'
}

onMounted(async () => {
  const cfg = await invoke<AppConfig>('get_config')
  // …既有…
  theme.value = resolveThemePref(cfg.general)
  await applyTheme(theme.value)
  stopThemeWatch = watchSystemTheme(() => theme.value)
  // 扩展 config-changed：
  // theme.value = resolveThemePref(event.payload.general)
  // void applyTheme(theme.value)
})

onUnmounted(() => {
  stopThemeWatch?.()
})
```

向 `GeneralTab` 传 `:theme` / `@update:theme`。

替换模板硬编码：

- 根 `text-white` → 移除或改用令牌驱动的类
- 侧边栏 `bg-[#161618]` → 用 `background: var(--ui-bg-sidebar)` 的类（需要时在 `style.css` 加 `.settings-sidebar`）
- 内容 `bg-[#1e1e20]` → `var(--ui-bg)` / 类

- [ ] **步骤 4：`App.vue`（settings 模式）提前应用主题**

在添加 `.settings` 类时/前：

```typescript
import { applyTheme, type ThemePreference } from './composables/useAppTheme'

if (mode.value === 'settings') {
  document.documentElement.classList.add('settings')
  document.documentElement.dataset.theme = 'dark' // FOUC 防护
}

onMounted(async () => {
  if (mode.value === 'settings') {
    try {
      const cfg = await invoke<AppConfig>('get_config')
      const pref = (cfg.general?.theme as ThemePreference | undefined) ?? 'dark'
      await applyTheme(pref === 'light' || pref === 'system' || pref === 'dark' ? pref : 'dark')
    } catch { /* 保持深色 */ }
    await revealSettingsWindow()
    // …
  }
})
```

- [ ] **步骤 5：诊断页文本域**

将作用域 `rgba(255,…)` 替换为 `var(--ui-control-bg)` / `var(--ui-control-border)` / `var(--ui-text-value)`。

- [ ] **步骤 6：验证前端**

```bash
npm test -- src/composables/useAppTheme.test.ts
npm run lint
npx vue-tsc --noEmit
```

预期：PASS。

- [ ] **步骤 7：提交**

```bash
git add src/i18n/en.ts src/i18n/zh-CN.ts src/components/settings/GeneralTab.vue src/components/SettingsView.vue src/App.vue src/components/settings/DiagnosticsTab.vue src/style.css
git commit -m "feat(settings): add appearance control for dark/light/system theme"
```

---

### 任务 6：覆盖层 + 工具栏主题接线

**文件：**
- 修改：`src/components/DrawingOverlay.vue`
- 修改：`src/components/ToolbarWindow.vue`

**接口：**
- 消费：`applyTheme`、`watchSystemTheme`、`config-changed`
- 产出：覆盖层/工具栏外观实时跟随偏好（含 `system` 下的 OS 变化）

- [ ] **步骤 1：DrawingOverlay**

首次 `get_config` 成功后：

```typescript
import { applyTheme, watchSystemTheme, type ThemePreference } from '../composables/useAppTheme'

function resolveThemePref(general?: AppConfig['general']): ThemePreference {
  const t = general?.theme
  return t === 'light' || t === 'system' || t === 'dark' ? t : 'dark'
}

let currentTheme: ThemePreference = 'dark'
let stopThemeWatch: (() => void) | null = null

// onMounted 中 get_config 之后：
currentTheme = resolveThemePref(cfg.general)
await applyTheme(currentTheme)
stopThemeWatch = watchSystemTheme(() => currentTheme)

// config-changed 监听器中：
currentTheme = resolveThemePref(event.payload.general)
void applyTheme(currentTheme)

// onUnmounted：
stopThemeWatch?.()
```

- [ ] **步骤 2：ToolbarWindow**——在其配置加载与 `config-changed` 上采用相同模式。

- [ ] **步骤 3：冒烟测试**

```bash
npm test
npm run lint
npx vue-tsc --noEmit
cd src-tauri && cargo test && cargo clippy -- -D warnings
```

预期：全绿。

- [ ] **步骤 4：提交**

```bash
git add src/components/DrawingOverlay.vue src/components/ToolbarWindow.vue
git commit -m "feat(theme): sync overlay and toolbar chrome with appearance setting"
```

---

### 任务 7：手动 QA 清单（Mac + Windows）

**文件：** 除非发现 bug，否则无需

- [ ] **步骤 1：运行应用**

```bash
nvm use
npm run dev
```

- [ ] **步骤 2：核验清单**

| # | 检查 | 通过？ |
|---|-------|-------|
| 1 | 常规 → 外观显示深色 / 浅色 / 跟随系统 | |
| 2 | 默认深色；无 `theme` 的既有配置保持深色 | |
| 3 | 切换浅色：设置卡片、导航、分段、弹层、帮助表格 | |
| 4 | 切换跟随系统：跟随 OS；OS 变化无需重启即更新应用 | |
| 5 | Space 面板 / 工具栏 / 色盘跟随主题 | |
| 6 | 画布笔迹颜色不变 | |
| 7 | Mac：设置标题栏 + 背景匹配；无白闪；覆盖层圆角无渗色 | |
| 8 | Windows：浅色/系统浅色时托盘在浅色任务栏上可读 | |
| 9 | Mac 托盘：模板仍自动适配 | |
| 10 | 语言下拉 + 快捷键提示不被裁切 | |
| 11 | 重启后偏好持久化 | |

- [ ] **步骤 3：修复发现的 bug；需要时以 `fix(theme): …` 提交**

- [ ] **步骤 4：合并前最终检查**

```bash
npm test && npm run lint && npm run format:check && npx vue-tsc --noEmit
cd src-tauri && cargo fmt --check && cargo clippy -- -D warnings && cargo test
```

---

## 自审（计划 vs 规格）

| 规格要求 | 任务 |
|------------------|------|
| `general.theme` 深色/浅色/系统，默认深色 | 任务 1 |
| CSS 变量 + `data-theme` | 任务 4 |
| `useAppTheme` + 系统实时监听 | 任务 3、5、6 |
| 设置 UI 分段 + i18n | 任务 5 |
| 多 webview 应用 + `config-changed` | 任务 5、6 |
| macOS `set_theme` + SETTINGS_BG | 任务 2 |
| Windows 托盘图标切换；Mac 模板 | 任务 2 |
| OS 变化时重新调用原生应用 | 任务 3（`watchSystemTheme` → `applyTheme` → invoke） |
| 不含画布 / 白板 / 色板编辑器 | 范围外（全局约束） |
| 测试：Rust serde + 前端解析 | 任务 1、3 |
| 手动 Mac+Windows QA | 任务 7 |
