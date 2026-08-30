# Windows 托盘跟随任务栏主题实现计划

> **致代理执行者：** 必需子技能：使用 superpowers:subagent-driven-development（推荐）或 superpowers:executing-plans 按任务执行本计划。步骤使用复选框（`- [ ]`）语法跟踪。

**目标：** 让 Windows 系统托盘图标跟随 `SystemUsesLightTheme`（任务栏 / 溢出弹出层），包括 Marker 运行中的实时更新，且独立于 `general.theme`。

**架构：** 拆分 Windows 外壳图标：设置窗口图标仍使用应用 `ResolvedTheme`；托盘使用由 `SystemUsesLightTheme` 驱动的专用 `apply_windows_tray_icon`。后台线程阻塞在 Personalize 键的 `RegNotifyChangeKeyValue` 上，每次变化重新应用托盘图标。

**技术栈：** Tauri 2（Rust）、`winreg` 0.55、`windows-sys` 0.59（`Win32_System_Registry`）、既有 `icon.png` / `icon-light.png`

## 全局约束

- 托盘信号：始终 `SystemUsesLightTheme`——绝不使用 `general.theme` / `AppsUseLightTheme`
- 实时更新：应用运行期间注册表通知（非轮询）
- 设置 WebView 主题 + 设置窗口图标：不变（仍走 `apply_app_theme` / `ResolvedTheme`）
- macOS：不变（`iconAsTemplate`）
- `SystemUsesLightTheme` 缺失 / 不可读 → 视为浅色 shell → 深色图形 `icon.png`
- 深色 shell（`0`）→ `icon-light.png`；浅色 shell（`1`）→ `icon.png`
- 规格：`docs/superpowers/specs/2026-07-26-windows-tray-system-theme-design.md`
- 实现前丢弃任何「托盘强制永久黑色」的本地 WIP（先对齐 HEAD 再应用本计划）

---

## 文件映射

| 文件 | 角色 |
|------|------|
| `src-tauri/Cargo.toml` | 新增 Windows 专属 `windows-sys`（`RegNotifyChangeKeyValue`） |
| `src-tauri/src/theme.rs` | shell 明暗检测、托盘应用、监听器；`apply_app_theme` 停止更新托盘 |
| `src-tauri/src/lib.rs` | 托盘 setup 之后：首次 `apply_windows_tray_icon` + `start_windows_tray_theme_watcher` |

无前端 / 配置 / i18n 变更。

---

### 任务 1：托盘图标选择 + 与应用主题解耦

**文件：**
- 修改：`src-tauri/src/theme.rs`
- 测试：`src-tauri/src/theme.rs` `#[cfg(test)]`

**接口：**
- 产出：
  - `pub fn windows_system_shell_is_light() -> bool`（`#[cfg(windows)]`）——读取 `SystemUsesLightTheme`；缺失/错误 → `true`
  - `fn tray_icon_png_for_shell_light(shell_is_light: bool) -> &'static [u8]`——供测试的纯映射
  - `pub fn apply_windows_tray_icon(app: &AppHandle)`——按 shell 明暗设置托盘；失败 `warn!`
- 消费：既有 `load_icon_from_png`（保留或内联）；`app.tray_by_id("main")`
- 变更：`update_windows_chrome_icons` / `apply_app_theme` **不得**再调用 `tray.set_icon`

- [ ] **步骤 1：编写失败测试**

加入 `theme.rs` 测试模块：

```rust
#[cfg(target_os = "windows")]
#[test]
fn tray_png_dark_shell_uses_light_glyph() {
    let bytes = tray_icon_png_for_shell_light(false);
    assert_eq!(
        bytes,
        include_bytes!("../icons/icon-light.png") as &[u8]
    );
}

#[cfg(target_os = "windows")]
#[test]
fn tray_png_light_shell_uses_dark_glyph() {
    let bytes = tray_icon_png_for_shell_light(true);
    assert_eq!(bytes, include_bytes!("../icons/icon.png") as &[u8]);
}
```

保留既有 `resolve_dark_and_light_are_fixed`。

- [ ] **步骤 2：运行测试确认失败**

运行：

```bash
cd src-tauri && cargo test --lib theme::tests::tray_png -- --nocapture
```

预期：FAIL——找不到 `tray_icon_png_for_shell_light`（或类似）。

- [ ] **步骤 3：实现 shell 读取 + 托盘应用；应用主题路径停止触碰托盘**

在 `theme.rs` 中替换 Windows 外壳/托盘块，使得：

1. 纯映射：

```rust
#[cfg(target_os = "windows")]
fn tray_icon_png_for_shell_light(shell_is_light: bool) -> &'static [u8] {
    if shell_is_light {
        include_bytes!("../icons/icon.png")
    } else {
        include_bytes!("../icons/icon-light.png")
    }
}
```

2. 注册表读取（为应用 `system` 偏好保留 `windows_apps_use_dark_theme`）：

```rust
/// Personalize 下的 `SystemUsesLightTheme`——`1` = 浅色任务栏/弹出层，
/// `0` = 深色。键缺失 → 视为浅色（优先深色图形可见性）。
#[cfg(target_os = "windows")]
pub fn windows_system_shell_is_light() -> bool {
    use winreg::enums::HKEY_CURRENT_USER;
    use winreg::RegKey;
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let Ok(key) =
        hkcu.open_subkey(r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize")
    else {
        return true;
    };
    let light: u32 = key.get_value("SystemUsesLightTheme").unwrap_or(1);
    light != 0
}
```

3. 公开应用函数：

```rust
#[cfg(target_os = "windows")]
pub fn apply_windows_tray_icon(app: &AppHandle) {
    if let Err(e) = apply_windows_tray_icon_inner(app) {
        warn!("Failed to update Windows tray icon: {}", e);
    }
}

#[cfg(target_os = "windows")]
fn apply_windows_tray_icon_inner(
    app: &AppHandle,
) -> Result<(), Box<dyn std::error::Error>> {
    let Some(tray) = app.tray_by_id("main") else {
        return Ok(());
    };
    let bytes = tray_icon_png_for_shell_light(windows_system_shell_is_light());
    tray.set_icon(Some(load_icon_from_png(bytes)?))?;
    Ok(())
}
```

4. `update_windows_chrome_icons`——**仅设置窗口图标**（移除托盘分支）。更新 `windows_theme_icon_png` 上方注释：仅用于设置窗口标题栏图标，非托盘。

5. 为设置窗口保留 `load_icon_from_png` / `load_windows_theme_icon` / `windows_theme_icon_png`。

若工作区仍有「托盘永久黑色」的注释/代码，用上述内容替换（不得保留强制 `icon.png` 的托盘路径）。

- [ ] **步骤 4：运行测试确认通过**

运行：

```bash
cd src-tauri && cargo test --lib theme:: -- --nocapture
```

预期：PASS（含两条新托盘 PNG 测试与 `resolve_dark_and_light_are_fixed`）。

- [ ] **步骤 5：提交**

```bash
git add src-tauri/src/theme.rs
git commit -m "$(cat <<'EOF'
fix(ui): select Windows tray icon from taskbar theme

EOF
)"
```

---

### 任务 2：注册表监听器 + 启动接线

**文件：**
- 修改：`src-tauri/Cargo.toml`
- 修改：`src-tauri/Cargo.lock`（由 `cargo` 解析）
- 修改：`src-tauri/src/theme.rs`
- 修改：`src-tauri/src/lib.rs`

**接口：**
- 消费：`apply_windows_tray_icon(app: &AppHandle)`、`windows_system_shell_is_light`
- 产出：`pub fn start_windows_tray_theme_watcher(app: &AppHandle)`——派生守护线程；通知失败绝不 panic 应用

- [ ] **步骤 1：新增 `windows-sys` Windows 依赖**

在 `src-tauri/Cargo.toml` 的 `[target.'cfg(target_os = "windows")'.dependencies]` 下：

```toml
winreg = "0.55"
windows-sys = { version = "0.59", features = ["Win32_System_Registry", "Win32_Foundation"] }
```

运行：

```bash
cd src-tauri && cargo check
```

预期：解析成功；lockfile 更新。

- [ ] **步骤 2：在 `theme.rs` 实现监听器**

```rust
/// 监听 Personalize 注册表值，在任务栏 / 系统 shell 主题变化时刷新托盘图标。
/// 即发即忘的守护线程。
#[cfg(target_os = "windows")]
pub fn start_windows_tray_theme_watcher(app: &AppHandle) {
    let app = app.clone();
    std::thread::Builder::new()
        .name("windows-tray-theme".into())
        .spawn(move || {
            use winreg::enums::{HKEY_CURRENT_USER, KEY_NOTIFY, KEY_READ};
            use winreg::RegKey;
            use windows_sys::Win32::Foundation::ERROR_SUCCESS;
            use windows_sys::Win32::System::Registry::{
                RegNotifyChangeKeyValue, REG_NOTIFY_CHANGE_LAST_SET, REG_NOTIFY_CHANGE_NAME,
            };

            let hkcu = RegKey::predef(HKEY_CURRENT_USER);
            let path = r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize";
            let Ok(key) = hkcu.open_subkey_with_flags(path, KEY_READ | KEY_NOTIFY) else {
                warn!("Tray theme watcher: cannot open Personalize key");
                return;
            };

            loop {
                let status = unsafe {
                    RegNotifyChangeKeyValue(
                        key.raw_handle(),
                        0, // 不含子树
                        REG_NOTIFY_CHANGE_NAME | REG_NOTIFY_CHANGE_LAST_SET,
                        std::ptr::null_mut(),
                        0, // 同步
                    )
                };
                if status != ERROR_SUCCESS {
                    warn!(
                        "Tray theme watcher: RegNotifyChangeKeyValue failed ({})",
                        status
                    );
                    break;
                }
                apply_windows_tray_icon(&app);
            }
        })
        .ok();
}
```

实现者注记：

- 空 event + 同步标志的 `RegNotifyChangeKeyValue` 会阻塞到下一次变化；返回后调用 `apply_windows_tray_icon` 再循环重新挂通知。
- 退出时不 join 线程；进程退出即结束。
- Windows 上 `KEY_NOTIFY` 已含于 `KEY_READ`；显式 `| KEY_NOTIFY` 亦可。

- [ ] **步骤 3：在 `lib.rs` 接线启动**

在托盘菜单重建 / 托盘事件钩子挂好后（托盘 id `"main"` 已存在）加入：

```rust
#[cfg(target_os = "windows")]
{
    theme::apply_windows_tray_icon(&handle);
    theme::start_windows_tray_theme_watcher(&handle);
}
```

位置在 `rebuild_tray_menu` 与托盘事件钩子**之后**（同一 `setup` 闭包），且放在既有 `theme::apply_app_theme(...)` 调用**之后**亦可——顺序：`apply_app_theme`（设置外观）再托盘应用 + 监听器，避免托盘被任何遗留应用主题路径短暂设置。

确认 `apply_app_theme` 不再触碰托盘（任务 1）。

- [ ] **步骤 4：编译 + 单元测试**

运行：

```bash
cd src-tauri && cargo fmt && cargo clippy -- -D warnings && cargo test --lib theme::
```

预期：fmt 通过、clippy 通过、theme 测试 PASS。

- [ ] **步骤 5：提交**

```bash
git add src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/src/theme.rs src-tauri/src/lib.rs
git commit -m "$(cat <<'EOF'
fix(ui): watch SystemUsesLightTheme for live tray icon

EOF
)"
```

---

### 任务 3：Windows 手动验收

**文件：** 无（仅验证）

**接口：** 无

- [ ] **步骤 1：运行应用**

```bash
npm run dev
```

- [ ] **步骤 2：对照验收清单核验**

清单（逐项勾选）：

1. 深色任务栏 / 溢出弹出层 → 浅色 Marker 托盘图形（`icon-light.png`）在深色 mica 弹出层上可见。
2. 浅色任务栏 / 弹出层 → 深色图形（`icon.png`）可见。
3. Marker 运行中，切换 Windows **设置 → 个性化 → 颜色 → 选择模式**（或「Windows 模式」/任务栏相关模式）使弹出层明暗变化 → 托盘**无需重启 Marker** 即更新。
4. 在 Marker 设置中切换外观 深色 ↔ 浅色 ↔ 跟随系统 → **托盘保持**任务栏信号；设置窗口外观/图标仍跟随应用主题。
5. macOS 构建（若有）仍使用模板托盘——`#[cfg(windows)]` 门控下预期无回归；可选冒烟：托盘仍可点击。

- [ ] **步骤 3：任一项失败则在 `theme.rs` / `lib.rs` 修复；仅在提交仍为本地且符合 amend 规则时 amend，否则新建修复提交**

- [ ] **步骤 4：验证通过则无需代码提交；若任务 2 的 PR/提交正文已覆盖，留一条简短说明即可**

---

## 规格覆盖（自审）

| 规格要求 | 任务 |
|------------------|------|
| 托盘取自 `SystemUsesLightTheme` | 任务 1 |
| 独立于 `general.theme` | 任务 1（`apply_app_theme` 不触托盘） |
| Personalize 实时更新 | 任务 2 |
| 设置图标 / WebView 不变 | 任务 1（保留窗口图标路径） |
| 键缺失 → 浅 shell / 深色图形 | 任务 1（`unwrap_or(1)`） |
| macOS 不变 | 仅 `#[cfg(windows)]` |
| 图形映射单元测试 | 任务 1 |
| 手动验收 | 任务 3 |

无遗留占位符；各任务函数命名一致（`apply_windows_tray_icon`、`start_windows_tray_theme_watcher`、`windows_system_shell_is_light`、`tray_icon_png_for_shell_light`）。
