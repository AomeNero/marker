use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_global_shortcut::GlobalShortcutExt;
use tauri_plugin_opener::OpenerExt;
use tracing::{info, warn};

use crate::config::{
    lock_or_recover, AppConfig, AppState, GeneralConfig, LineWidthsConfig, SaveResult, Shortcuts,
};
use crate::error::{AppError, AppResult};
use crate::shortcuts::{parse_shortcut, register_shortcuts};

fn duplicate_shortcut_errors(shortcuts: &Shortcuts) -> Vec<String> {
    let s = crate::i18n::strings();
    let actions = [
        (s.toggle_drawing, shortcuts.toggle_drawing.as_str()),
        (s.clear_drawing, shortcuts.clear_drawing.as_str()),
    ];
    let mut failed = Vec::new();
    for i in 0..actions.len() {
        for j in (i + 1)..actions.len() {
            // Empty = unbound; multiple unbound shortcuts are allowed.
            if actions[i].1.is_empty() || actions[j].1.is_empty() {
                continue;
            }
            if actions[i].1 == actions[j].1 {
                failed.push(format!(
                    "Duplicate shortcut: {} and {}",
                    actions[i].0, actions[j].0
                ));
            }
        }
    }
    failed
}

/// Non-empty accel that fails to parse is a hard validation error.
fn invalid_shortcut_errors(shortcuts: &Shortcuts) -> Vec<String> {
    let s = crate::i18n::strings();
    let actions = [
        (s.toggle_drawing, shortcuts.toggle_drawing.as_str()),
        (s.clear_drawing, shortcuts.clear_drawing.as_str()),
    ];
    let mut failed = Vec::new();
    for (label, accel) in actions {
        if accel.is_empty() {
            continue;
        }
        if parse_shortcut(accel).is_none() {
            failed.push(format!("{}: {}", label, accel));
        }
    }
    failed
}

#[tauri::command]
pub fn get_config(state: tauri::State<'_, AppState>) -> AppConfig {
    lock_or_recover(&state.config).clone()
}

#[tauri::command]
pub fn get_overlay_pointer_position(
    window: tauri::WebviewWindow,
) -> Option<crate::monitor::OverlayPointerPosition> {
    crate::monitor::get_overlay_client_pointer_for(&window)
}

#[tauri::command]
pub fn get_overlay_monitor_logical_bounds(
    window: tauri::WebviewWindow,
) -> Option<crate::monitor::MonitorLogicalBounds> {
    crate::monitor::get_overlay_monitor_logical_bounds_for(&window)
}

#[tauri::command]
pub fn get_overlay_monitor_work_logical_bounds(
    window: tauri::WebviewWindow,
) -> Option<crate::monitor::MonitorLogicalBounds> {
    crate::monitor::get_overlay_monitor_work_logical_bounds_for(&window)
}

#[tauri::command]
pub fn is_pointer_over_toolbar_panel(app: AppHandle) -> bool {
    crate::overlay::is_pointer_over_toolbar_panel(&app)
}

/// Forward a toolbar action to exactly one overlay window — the one on the
/// cursor's monitor — so multi-display setups execute each action once.
/// The executor applies the action locally and broadcasts the new session
/// state, which sibling overlays apply idempotently.
#[tauri::command]
pub fn forward_toolbar_action(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    action: serde_json::Value,
) {
    let target = crate::overlay_windows::label_for_cursor(&app, &state)
        .unwrap_or_else(|| crate::overlay_windows::PRIMARY_LABEL.to_string());
    if let Err(e) = app.emit_to(&target, crate::overlay::TOOLBAR_ACTION_EVENT, action) {
        warn!("Failed to forward toolbar action to {}: {}", target, e);
    }
}

/// Record a stroke/edit op from `window` on the global timeline. A fresh op
/// also invalidates the redo branches of every overlay.
#[tauri::command]
pub fn timeline_commit_op(
    app: AppHandle,
    window: tauri::WebviewWindow,
    state: tauri::State<'_, AppState>,
    kind: String,
) {
    lock_or_recover(&state.timeline).commit(window.label(), &kind);
    if let Err(e) = app.emit(crate::timeline::REDO_CLEARED_EVENT, ()) {
        warn!("Failed to emit timeline redo-cleared: {}", e);
    }
}

fn emit_timeline_replay(app: &AppHandle, op: &crate::timeline::TimelineOp, event: &str) {
    // Empty owner = global clear op: replay on every overlay window.
    let targets: Vec<String> = if op.owner.is_empty() {
        crate::overlay_windows::overlay_labels(app)
    } else {
        vec![op.owner.clone()]
    };
    for label in targets {
        if let Err(e) = app.emit_to(&label, event, op.op_id) {
            warn!("Failed to emit {} to {}: {}", event, label, e);
        }
    }
}

#[tauri::command]
pub fn timeline_undo(app: AppHandle, state: tauri::State<'_, AppState>) {
    let Some(op) = lock_or_recover(&state.timeline).undo() else {
        return;
    };
    emit_timeline_replay(&app, &op, crate::timeline::UNDO_EVENT);
}

#[tauri::command]
pub fn timeline_redo(app: AppHandle, state: tauri::State<'_, AppState>) {
    let Some(op) = lock_or_recover(&state.timeline).redo() else {
        return;
    };
    emit_timeline_replay(&app, &op, crate::timeline::REDO_EVENT);
}

#[tauri::command]
pub fn timeline_reset(state: tauri::State<'_, AppState>) {
    lock_or_recover(&state.timeline).reset();
}

/// Toolbar clear-all entry point — same global path as the Alt+E shortcut.
#[tauri::command]
pub fn clear_all_drawings(app: AppHandle, state: tauri::State<'_, AppState>) {
    crate::clear_drawing(&app, &state);
}

#[tauri::command]
pub fn set_overlay_ignore_cursor_events(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    ignore: bool,
) {
    crate::overlay::set_overlay_ignore_cursor_events(&app, &state, ignore);
}

#[tauri::command]
pub fn save_shortcuts(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    shortcuts: Shortcuts,
) -> SaveResult {
    // Hard validation: invalid format or in-app duplicates block the whole save.
    let mut hard_failed = invalid_shortcut_errors(&shortcuts);
    hard_failed.extend(duplicate_shortcut_errors(&shortcuts));

    if !hard_failed.is_empty() {
        return SaveResult {
            ok: false,
            failed: Some(hard_failed),
        };
    }

    app.global_shortcut().unregister_all().ok();

    let s = crate::i18n::strings();
    let actions: Vec<(&str, &str)> = vec![
        (s.toggle_drawing, &shortcuts.toggle_drawing),
        (s.clear_drawing, &shortcuts.clear_drawing),
    ];

    // Soft validation: OS/other-app occupation — still persist config so one
    // occupied binding cannot block changing or clearing the others.
    let mut warnings = Vec::new();
    for (label, accel) in &actions {
        if accel.is_empty() {
            continue;
        }
        if let Some(shortcut) = parse_shortcut(accel) {
            if app.global_shortcut().register(shortcut).is_err() {
                warnings.push(format!("{}: {}", label, accel));
            }
        }
    }

    {
        let mut cfg = lock_or_recover(&state.config);
        cfg.shortcuts = shortcuts;
        crate::config::save_config(&app, &cfg);
    }
    // Re-bind from saved config (skips empty / logs OS failures).
    register_shortcuts(&app);

    if warnings.is_empty() {
        info!("Shortcuts saved successfully");
        SaveResult {
            ok: true,
            failed: None,
        }
    } else {
        warn!(
            "Shortcuts saved with OS registration warnings: {:?}",
            warnings
        );
        SaveResult {
            ok: true,
            failed: Some(warnings),
        }
    }
}

#[tauri::command]
pub fn save_general(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    general: GeneralConfig,
) -> AppResult<()> {
    let auto_start = {
        let mut cfg = lock_or_recover(&state.config);
        cfg.general = general.normalized();
        crate::config::save_config(&app, &cfg);
        cfg.general.auto_start
    };
    crate::config::apply_autostart_preference(&app, &state, auto_start);
    let snapshot = lock_or_recover(&state.config).clone();
    let theme = snapshot.general.theme;
    if let Err(e) = app.emit("config-changed", snapshot) {
        warn!("Failed to emit config-changed: {}", e);
    }
    if crate::overlay::current_mode(&state) != crate::overlay::OverlayMode::Hidden {
        crate::overlay::ensure_toolbar_window(&app, &state);
    }
    crate::theme::apply_app_theme(&app, &theme);
    info!("General config saved");
    Ok(())
}

/// Patch only `lineWidths` under the config lock so concurrent `save_general`
/// callers cannot clobber each other (and vice versa).
#[tauri::command]
pub fn save_line_widths(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    line_widths: LineWidthsConfig,
) -> AppResult<()> {
    let snapshot = {
        let mut cfg = lock_or_recover(&state.config);
        // Snap to the closest preset of the active (already-normalized) preset set.
        let normalized = line_widths.normalized_with(&cfg.general.width_presets);
        if cfg.general.line_widths == normalized {
            return Ok(());
        }
        cfg.general.line_widths = normalized;
        crate::config::save_config(&app, &cfg);
        cfg.clone()
    };
    if let Err(e) = app.emit("config-changed", snapshot) {
        warn!("Failed to emit config-changed: {}", e);
    }
    info!("Line widths saved");
    Ok(())
}

#[tauri::command]
pub fn save_locale(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    locale: String,
) -> AppResult<()> {
    {
        let mut cfg = lock_or_recover(&state.config);
        cfg.general.locale = Some(locale.clone());
        crate::config::save_config(&app, &cfg);
    }

    crate::i18n::set_locale(&locale);
    crate::rebuild_tray_menu(&app).map_err(|e| AppError::Other(e.to_string()))?;

    if let Some(win) = app.get_webview_window("settings") {
        if let Err(e) = win.set_title(crate::i18n::strings().window_title) {
            warn!("Failed to set settings window title: {}", e);
        }
    }

    info!("Locale changed to {}", locale);
    Ok(())
}

#[tauri::command]
pub fn apply_app_theme(
    app: AppHandle,
    preference: crate::config::ThemePreference,
) -> AppResult<()> {
    crate::theme::apply_app_theme(&app, &preference);
    Ok(())
}

#[tauri::command]
pub fn exit_drawing(app: AppHandle, state: tauri::State<'_, AppState>) {
    crate::deactivate_drawing(&app, &state);
}

#[tauri::command]
pub fn set_whiteboard_mode(state: tauri::State<'_, AppState>, active: bool) {
    *lock_or_recover(&state.whiteboard_mode) = active;
}

#[tauri::command]
pub fn set_toolbar_visible(app: AppHandle, visible: bool) {
    crate::set_toolbar_window_visible(&app, visible);
}

#[tauri::command]
pub fn set_toolbar_popup(
    app: AppHandle,
    visible: bool,
    x: Option<f64>,
    y: Option<f64>,
    height: Option<f64>,
) {
    crate::overlay::set_toolbar_popup(&app, visible, x, y, height);
}

#[tauri::command]
pub fn raise_toolbar(app: AppHandle) {
    crate::overlay::raise_toolbar_above_overlay(&app);
}

const ALLOWED_URL_PREFIXES: &[&str] = &[
    "https://github.com/",
    "https://apps.microsoft.com/",
    "https://marker.cn/",
];

fn is_allowed_open_url(url: &str) -> bool {
    ALLOWED_URL_PREFIXES
        .iter()
        .any(|prefix| url.starts_with(prefix))
}

#[tauri::command]
pub fn reveal_settings_window(app: AppHandle) {
    crate::reveal_settings_window(&app);
}

#[tauri::command]
pub fn is_portable() -> bool {
    crate::portable::is_portable()
}

#[tauri::command]
pub fn supports_autostart() -> bool {
    crate::config::supports_autostart()
}

#[tauri::command]
pub fn open_url(app: AppHandle, url: String) -> AppResult<()> {
    if !is_allowed_open_url(&url) {
        warn!("Blocked open_url for untrusted URL: {}", url);
        return Err(AppError::Other("URL not allowed".into()));
    }
    app.opener()
        .open_url(&url, None::<&str>)
        .map_err(|e| AppError::Other(e.to_string()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::default_shortcuts;

    #[test]
    fn duplicate_shortcut_errors_empty_for_defaults() {
        assert!(duplicate_shortcut_errors(&default_shortcuts()).is_empty());
    }

    #[test]
    fn duplicate_shortcut_errors_detects_collision() {
        let mut shortcuts = default_shortcuts();
        shortcuts.clear_drawing = shortcuts.toggle_drawing.clone();
        let errors = duplicate_shortcut_errors(&shortcuts);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("Duplicate shortcut"));
    }

    #[test]
    fn duplicate_shortcut_errors_ignores_empty_bindings() {
        let mut shortcuts = default_shortcuts();
        shortcuts.clear_drawing = String::new();
        assert!(duplicate_shortcut_errors(&shortcuts).is_empty());
    }

    #[test]
    fn invalid_shortcut_errors_allows_empty() {
        let mut shortcuts = default_shortcuts();
        shortcuts.clear_drawing = String::new();
        assert!(invalid_shortcut_errors(&shortcuts).is_empty());
    }

    #[test]
    fn invalid_shortcut_errors_rejects_garbage() {
        let mut shortcuts = default_shortcuts();
        shortcuts.clear_drawing = "NotAKey".into();
        let errors = invalid_shortcut_errors(&shortcuts);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("NotAKey"));
    }

    #[test]
    fn open_url_allowlist_permits_known_destinations() {
        assert!(is_allowed_open_url("https://github.com/AomeNero/marker"));
        assert!(is_allowed_open_url(
            "https://github.com/AomeNero/marker/issues"
        ));
        assert!(is_allowed_open_url(
            "https://apps.microsoft.com/store/detail/marker/9P123"
        ));
        assert!(is_allowed_open_url("https://marker.cn/help.html"));
    }

    #[test]
    fn open_url_allowlist_blocks_untrusted_destinations() {
        assert!(!is_allowed_open_url("https://example.com/"));
        assert!(!is_allowed_open_url("http://github.com/AomeNero/marker"));
        assert!(!is_allowed_open_url("https://afdian.com.evil.com/a/marker"));
        assert!(!is_allowed_open_url("https://marker.cn.evil.com/help"));
        assert!(!is_allowed_open_url("http://marker.cn/help.html"));
    }
}
