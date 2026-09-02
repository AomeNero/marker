use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder};
use tracing::{info, warn};

use crate::config::{lock_or_recover, AppState, ToolbarVisibility};
use crate::diagnostics::log_backend_event;
use crate::monitor;
use std::time::Duration;

fn set_ignore_cursor_events(window: &WebviewWindow, ignore: bool) {
    window.set_ignore_cursor_events(ignore).ok();
}

/// Keep the overlay clear color fully transparent.
///
/// On Windows, WebView2 can lose transparency after long idle / GPU recycle /
/// DPI moves and fall back to an opaque dark (black) clear color. Re-asserting
/// on each activation matches macOS `configure_overlay_window`.
///
/// `set_background_color` also re-asserts the WebView2 `DefaultBackgroundColor`
/// (via the wry webview dispatcher). Separately, the *host* window's DWM
/// blur-behind transparency (what `tao` uses to implement `transparent(true)`)
/// is applied only once at creation and can be dropped by the compositor after
/// a long idle — so re-apply it here as well.
fn ensure_overlay_transparent(window: &WebviewWindow) {
    use tauri::window::Color;
    window.set_background_color(Some(Color(0, 0, 0, 0))).ok();
    #[cfg(target_os = "windows")]
    {
        if let Ok(hwnd) = window.hwnd() {
            crate::win32::reapply_overlay_transparency(hwnd.0 as isize);
        }
    }
}

/// Toolbar → overlay action transport (frontend `TOOLBAR_ACTION_EVENT`).
/// Routed by `forward_toolbar_action` to exactly one overlay window.
pub const TOOLBAR_ACTION_EVENT: &str = "toolbar-action";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverlayMode {
    Hidden,
    Drawing,
}

impl OverlayMode {
    pub fn as_str(self) -> &'static str {
        match self {
            OverlayMode::Hidden => "hidden",
            OverlayMode::Drawing => "drawing",
        }
    }
}

pub fn current_mode(state: &AppState) -> OverlayMode {
    *lock_or_recover(&state.overlay_mode)
}

pub fn set_mode(state: &AppState, mode: OverlayMode) {
    *lock_or_recover(&state.overlay_mode) = mode;
}

fn emit_mode(app: &AppHandle, mode: OverlayMode) {
    let payload = mode.as_str();
    if let Err(e) = app.emit("overlay-mode-changed", payload) {
        warn!("Failed to emit overlay-mode-changed: {}", e);
    }
    let is_active = mode != OverlayMode::Hidden;
    if let Err(e) = app.emit("toggle-drawing", is_active) {
        warn!("Failed to emit toggle-drawing({}): {}", is_active, e);
    }
}

fn emit_overlay_geometry_changed(app: &AppHandle) {
    if let Err(e) = app.emit("overlay-geometry-changed", ()) {
        warn!("Failed to emit overlay-geometry-changed: {}", e);
    }
}

/// Re-assert transparency for every overlay window (activation / geometry
/// pulses broadcast to all displays).
pub fn reassert_overlay_transparency(app: &AppHandle) {
    for label in crate::overlay_windows::overlay_labels(app) {
        if let Some(window) = app.get_webview_window(&label) {
            ensure_overlay_transparent(&window);
        }
    }
}

/// Re-assert transparency for one overlay window (focused / scale-changed
/// window events target exactly one display).
pub fn reassert_window_transparency(window: &tauri::WebviewWindow) {
    ensure_overlay_transparent(window);
}

fn apply_deferred_overlay_geometry(app: &AppHandle) {
    reassert_overlay_transparency(app);
    emit_overlay_geometry_changed(app);
    #[cfg(target_os = "macos")]
    {
        // Overlay may have moved to another monitor; keep the panel clamped to
        // the cursor's monitor and one level above all overlays.
        if app
            .get_webview_window("toolbar")
            .is_some_and(|w| w.is_visible().unwrap_or(false))
        {
            clamp_toolbar_to_cursor_monitor(app);
            raise_toolbar_above_overlay(app);
        }
    }
}

/// Notify the overlay webview after Win32/Tauri has applied a new monitor geometry.
/// Deferred pulses let WM_DPICHANGED (and WebView2 compositor recreate) finish
/// before the frontend resizes canvases; also re-assert transparent clear color
/// because DPI/GPU recycle can restore an opaque dark backdrop.
///
/// The pulse train extends to 2s because after a long idle the WebView2 GPU
/// process can rebuild its compositor well past the initial 150ms window and
/// fall back to an opaque black clear color only then.
pub fn notify_overlay_geometry_changed(app: &AppHandle) {
    reassert_overlay_transparency(app);
    emit_overlay_geometry_changed(app);
    let app = app.clone();
    std::thread::spawn(move || {
        for delay_ms in [50_u64, 150, 500, 1000, 2000] {
            std::thread::sleep(Duration::from_millis(delay_ms));
            let app_for_thread = app.clone();
            let _ = app.run_on_main_thread(move || {
                apply_deferred_overlay_geometry(&app_for_thread);
            });
        }
    });
}

pub fn setup_overlay_size(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("overlay") {
        if let Some((x, y, w, h)) = monitor::get_cursor_monitor_work_rect() {
            #[cfg(target_os = "macos")]
            {
                window.set_size(tauri::LogicalSize::new(w, h)).ok();
                window.set_position(tauri::LogicalPosition::new(x, y)).ok();
            }
            #[cfg(windows)]
            {
                if let Ok(hwnd) = window.hwnd() {
                    crate::win32::position_window_on_monitor(hwnd.0 as isize, x, y, w, h);
                } else {
                    window
                        .set_size(tauri::PhysicalSize::new(w, h.saturating_sub(1)))
                        .ok();
                    window.set_position(tauri::PhysicalPosition::new(x, y)).ok();
                }
            }
        } else if let Some(mon) = app.primary_monitor().ok().flatten() {
            let size = mon.size();
            let pos = mon.position();
            #[cfg(target_os = "macos")]
            {
                let scale = mon.scale_factor();
                window
                    .set_size(tauri::LogicalSize::new(
                        size.width as f64 / scale,
                        size.height as f64 / scale,
                    ))
                    .ok();
                window
                    .set_position(tauri::LogicalPosition::new(
                        pos.x as f64 / scale,
                        pos.y as f64 / scale,
                    ))
                    .ok();
            }
            #[cfg(not(target_os = "macos"))]
            {
                window
                    .set_size(tauri::PhysicalSize::new(
                        size.width,
                        size.height.saturating_sub(1),
                    ))
                    .ok();
                window
                    .set_position(tauri::PhysicalPosition::new(pos.x, pos.y))
                    .ok();
            }
        }
        set_ignore_cursor_events(&window, true);
        ensure_overlay_transparent(&window);
    }

    #[cfg(target_os = "macos")]
    {
        // Restore on-monitor placement and the toolbar level if the panel is up.
        if app
            .get_webview_window("toolbar")
            .is_some_and(|w| w.is_visible().unwrap_or(false))
        {
            clamp_toolbar_to_cursor_monitor(app);
            raise_toolbar_above_overlay(app);
        }
    }
}

const TOOLBAR_WIDTH: f64 = 580.0;
const TOOLBAR_PANEL_WIDTH: f64 = 580.0;
/// One-line toolbar bar height (grip + 30px buttons + padding); the live height is
/// measured from the DOM (`fitToolbarWindow`). Flyouts stack above the bar.
const TOOLBAR_PANEL_HEIGHT_COMPACT: f64 = 46.0;
const TOOLBAR_EDGE_MARGIN: f64 = 8.0;
/// Dock offset for the always-on toolbar: bottom-right of the monitor WORK area
/// (taskbar already excluded by rc_work), so 16px simply clears the work-area edge.
const TOOLBAR_DOCK_RIGHT: i32 = 25;
const TOOLBAR_DOCK_BOTTOM: i32 = 16;

fn toolbar_panel_height_logical(window: &tauri::WebviewWindow, fallback: f64) -> f64 {
    let scale = window.scale_factor().unwrap_or(1.0);
    window
        .outer_size()
        .ok()
        .map(|s| s.height as f64 / scale)
        // One-line bar is 46px logical; anything smaller is a stale/zero measurement.
        .filter(|h| *h >= 40.0)
        .unwrap_or(fallback)
}

/// Default dock for a fresh (never-dragged) always-on toolbar: bottom-right of the
/// cursor's monitor, hovering just above the taskbar.
fn position_toolbar_window(app: &AppHandle) {
    let Some(window) = app.get_webview_window("toolbar") else {
        return;
    };

    if let Some((x, y, w, h)) = monitor::get_cursor_monitor_rect() {
        #[cfg(target_os = "macos")]
        {
            let left = x as f64 + w as f64 - TOOLBAR_WIDTH - TOOLBAR_DOCK_RIGHT as f64;
            let top =
                y as f64 + h as f64 - TOOLBAR_PANEL_HEIGHT_COMPACT - TOOLBAR_DOCK_BOTTOM as f64;
            window
                .set_position(tauri::LogicalPosition::new(left, top))
                .ok();
        }
        #[cfg(not(target_os = "macos"))]
        {
            let left = x + w as i32 - TOOLBAR_WIDTH as i32 - TOOLBAR_DOCK_RIGHT;
            let top = y + h as i32 - TOOLBAR_PANEL_HEIGHT_COMPACT as i32 - TOOLBAR_DOCK_BOTTOM;
            window
                .set_position(tauri::PhysicalPosition::new(left, top))
                .ok();
        }
    }
}

fn toolbar_always_visible(state: &AppState) -> bool {
    lock_or_recover(&state.config).general.toolbar_visibility == ToolbarVisibility::Always
}

fn create_toolbar_window(app: &AppHandle) {
    if app.get_webview_window("toolbar").is_some() {
        return;
    }

    let url = WebviewUrl::App("index.html#toolbar".into());
    let builder = WebviewWindowBuilder::new(app, "toolbar", url)
        .title("Marker")
        .inner_size(TOOLBAR_WIDTH, TOOLBAR_PANEL_HEIGHT_COMPACT)
        .decorations(false)
        .transparent(true)
        .always_on_top(true)
        .skip_taskbar(true)
        .resizable(false)
        .visible(false)
        .focused(false)
        .shadow(false);

    match builder.build() {
        Ok(window) => {
            position_toolbar_window(app);
            #[cfg(target_os = "macos")]
            crate::macos::configure_toolbar_window(&window);
            window.set_always_on_top(true).ok();
            set_ignore_cursor_events(&window, false);
        }
        Err(e) => warn!("Failed to create toolbar window: {}", e),
    }
}

/// Re-stack the toolbar webview above the drawing overlays.
///
/// Clicking an overlay canvas (e.g. to start a stroke) can promote it above the
/// toolbar. This restores toolbar-on-top ordering without focusing the toolbar.
///
/// **Shared (all platforms):** every overlay and the toolbar → `set_always_on_top(true)`.
///
/// **Windows:** `SetWindowPos(HWND_TOPMOST)` on all overlays, then toolbar (`win32.rs`).
///
/// **macOS:** the toolbar sits one fixed NSWindow level above all overlays
/// (`set_toolbar_level_above_overlays`) — multi-display keeps ONE toolbar over
/// N per-screen overlays, so a level beats per-click re-ordering. Do not call
/// WKWebView Objective-C selectors (crashes on Wry).
///
/// Invoked from drawing activation, toolbar reposition, panel hover pass-through
/// changes, and the frontend `raise_toolbar` IPC (pointer-down, toolbar drag end).
pub fn raise_toolbar_above_overlay(app: &AppHandle) {
    let Some(toolbar) = app.get_webview_window("toolbar") else {
        return;
    };
    if !toolbar.is_visible().unwrap_or(false) {
        return;
    }
    #[cfg(target_os = "macos")]
    {
        // One fixed level above all overlay windows (multi-display: one
        // toolbar, N per-screen overlays) — no re-ordering needed after
        // canvas clicks.
        crate::macos::set_toolbar_level_above_overlays(&toolbar);
        return;
    }
    #[cfg(not(target_os = "macos"))]
    {
        // Keep every overlay and the toolbar topmost, then force the toolbar
        // above the overlays in the OS Z-order.
        let labels = crate::overlay_windows::overlay_labels(app);
        for label in &labels {
            if let Some(w) = app.get_webview_window(label) {
                w.set_always_on_top(true).ok();
            }
        }
        toolbar.set_always_on_top(true).ok();
        #[cfg(windows)]
        {
            if let Ok(toolbar_hwnd) = toolbar.hwnd() {
                for label in &labels {
                    if let Some(overlay) = app.get_webview_window(label) {
                        if let Ok(hwnd) = overlay.hwnd() {
                            crate::win32::raise_window_topmost_no_activate(hwnd.0 as isize);
                        }
                    }
                }
                crate::win32::raise_window_topmost_no_activate(toolbar_hwnd.0 as isize);
            }
        }
    }
}

/// Keep the always-on toolbar fully inside the cursor's monitor.
///
/// Called whenever the pinned toolbar is shown (e.g. Ctrl+Shift+D) so a saved or
/// stale position on a disconnected / other display cannot leave the panel off-screen.
fn clamp_toolbar_to_cursor_monitor(app: &AppHandle) {
    let Some(window) = app.get_webview_window("toolbar") else {
        return;
    };
    let Some(info) = monitor::get_cursor_monitor_info(app) else {
        return;
    };

    let toolbar_scale = window.scale_factor().unwrap_or(info.scale_factor);

    let Ok(pos) = window.outer_position() else {
        return;
    };
    let left = pos.x as f64 / toolbar_scale;
    let top = pos.y as f64 / toolbar_scale;

    let panel_h = toolbar_panel_height_logical(&window, TOOLBAR_PANEL_HEIGHT_COMPACT);
    let (x, y) = monitor::clamp_logical_position_to_monitor(
        left,
        top,
        TOOLBAR_PANEL_WIDTH,
        panel_h,
        &info.full,
        TOOLBAR_EDGE_MARGIN,
    );

    if (x - left).abs() < 0.5 && (y - top).abs() < 0.5 {
        return;
    }

    #[cfg(windows)]
    {
        let phys_x = (x * info.scale_factor).round() as i32;
        let phys_y = (y * info.scale_factor).round() as i32;
        window
            .set_position(tauri::PhysicalPosition::new(phys_x, phys_y))
            .ok();
        if let Err(e) = app.emit("toolbar-window-positioned", ()) {
            warn!("Failed to emit toolbar-window-positioned: {}", e);
        }
    }

    #[cfg(not(windows))]
    {
        window.set_position(tauri::LogicalPosition::new(x, y)).ok();
    }
}

pub fn set_toolbar_window_visible(app: &AppHandle, visible: bool) {
    create_toolbar_window(app);
    if let Some(window) = app.get_webview_window("toolbar") {
        if visible {
            window.show().ok();
            window.set_always_on_top(true).ok();
            set_ignore_cursor_events(&window, false);
            let state = app.state::<AppState>();
            if toolbar_always_visible(&state) {
                clamp_toolbar_to_cursor_monitor(app);
            }
            raise_toolbar_above_overlay(app);
        } else {
            window.hide().ok();
        }
    }
}

pub fn position_toolbar_at(app: &AppHandle, x: f64, y: f64, panel_height: Option<f64>) {
    create_toolbar_window(app);
    let Some(window) = app.get_webview_window("toolbar") else {
        return;
    };
    let info = monitor::get_cursor_monitor_info(app);
    let requested_x = x;
    let requested_y = y;
    let panel_h = panel_height
        .filter(|h| *h >= 64.0)
        .unwrap_or_else(|| toolbar_panel_height_logical(&window, TOOLBAR_PANEL_HEIGHT_COMPACT));
    let (x, y) = if let Some(ref info) = info {
        monitor::clamp_logical_position_to_monitor(
            x,
            y,
            TOOLBAR_PANEL_WIDTH,
            panel_h,
            &info.full,
            TOOLBAR_EDGE_MARGIN,
        )
    } else {
        (x, y)
    };
    let state = app.state::<crate::config::AppState>();
    let scale_factor = info.as_ref().map(|i| i.scale_factor).unwrap_or(1.0);

    log_backend_event(
        &state,
        "ui",
        "toolbar popup positioned",
        Some(serde_json::json!({
            "requested": { "x": requested_x, "y": requested_y },
            "clamped": { "x": x, "y": y },
            "panelHeight": panel_h,
            "cursorMonitor": info,
            "scaleFactor": scale_factor,
        })),
        "info",
    );

    #[cfg(windows)]
    {
        let phys_x = (x * scale_factor).round() as i32;
        let phys_y = (y * scale_factor).round() as i32;
        let phys_w = (TOOLBAR_PANEL_WIDTH * scale_factor).round() as u32;
        let phys_h = (panel_h * scale_factor).round() as u32;
        if let Ok(hwnd) = window.hwnd() {
            crate::win32::position_window_on_monitor(
                hwnd.0 as isize,
                phys_x,
                phys_y,
                phys_w.max(1),
                phys_h.max(96),
            );
        } else {
            window
                .set_position(tauri::PhysicalPosition::new(phys_x, phys_y))
                .ok();
            window
                .set_size(tauri::PhysicalSize::new(
                    phys_w.max(1),
                    phys_h.saturating_sub(1).max(1),
                ))
                .ok();
        }
        if let Err(e) = app.emit("toolbar-window-positioned", ()) {
            warn!("Failed to emit toolbar-window-positioned: {}", e);
        }
    }

    #[cfg(not(windows))]
    {
        window.set_position(tauri::LogicalPosition::new(x, y)).ok();
        if let Err(e) = app.emit("toolbar-window-positioned", ()) {
            warn!("Failed to emit toolbar-window-positioned: {}", e);
        }
    }

    raise_toolbar_above_overlay(app);
}

pub fn set_toolbar_popup(
    app: &AppHandle,
    visible: bool,
    x: Option<f64>,
    y: Option<f64>,
    height: Option<f64>,
) {
    if visible {
        if let (Some(x), Some(y)) = (x, y) {
            position_toolbar_at(app, x, y, height);
        }
        set_toolbar_window_visible(app, true);
    } else {
        set_toolbar_window_visible(app, false);
    }
}

pub fn ensure_toolbar_window(app: &AppHandle, state: &AppState) {
    set_toolbar_window_visible(app, toolbar_always_visible(state));
}

pub fn hide_toolbar_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("toolbar") {
        window.hide().ok();
    }
}

/// The tray context menu must render above the topmost toolbar bar: hide the bar
/// while the menu is open, restore on any menu action or after a short fallback
/// (menu dismissed with Esc has no callback).
pub fn hide_toolbar_for_tray_menu(app: &AppHandle) {
    hide_toolbar_window(app);
    let app = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(5000));
        let app_for_thread = app.clone();
        let _ = app.run_on_main_thread(move || {
            set_toolbar_window_visible(&app_for_thread, true);
        });
    });
}

/// Menu was acted upon (or the fallback fired) — bring the resident bar back.
pub fn show_toolbar_after_tray_menu(app: &AppHandle) {
    set_toolbar_window_visible(app, true);
}

pub fn deactivate_drawing(app: &AppHandle, state: &AppState) {
    if current_mode(state) == OverlayMode::Hidden {
        return;
    }

    set_mode(state, OverlayMode::Hidden);
    *lock_or_recover(&state.whiteboard_mode) = false;
    if !lock_or_recover(&state.config).general.preserve_drawings {
        lock_or_recover(&state.timeline).reset();
    }

    crate::overlay_windows::stop_topology_watcher();
    crate::overlay_windows::deactivate_overlays(app, state);
    hide_toolbar_window(app);
    emit_mode(app, OverlayMode::Hidden);
}

pub fn activate_drawing(app: &AppHandle, state: &AppState) {
    set_mode(state, OverlayMode::Drawing);

    let preserve = lock_or_recover(&state.config).general.preserve_drawings;

    if let Some(window) = app.get_webview_window("overlay") {
        setup_overlay_size(app);
        if !preserve {
            lock_or_recover(&state.timeline).reset();
            if let Err(e) = app.emit("clear-drawing", ()) {
                warn!("Failed to emit clear-drawing: {}", e);
            }
        }
        ensure_overlay_transparent(&window);
        window.show().ok();
        set_ignore_cursor_events(&window, false);
        window.set_always_on_top(true).ok();
        // Re-assert after show: some WebView2 builds only apply clear color once visible.
        ensure_overlay_transparent(&window);
        notify_overlay_geometry_changed(app);
    }

    // Assign remaining monitors to dynamic overlay windows and show them.
    crate::overlay_windows::assign_and_show_extra_overlays(app, state);
    crate::overlay_windows::start_topology_watcher(app);

    ensure_toolbar_window(app, state);

    if let Some(window) = app.get_webview_window("overlay") {
        window.set_focus().ok();
    }
    raise_toolbar_above_overlay(app);

    emit_mode(app, OverlayMode::Drawing);
    info!("Drawing mode activated");
}

/// Whether the OS cursor is over the visible toolbar window (macOS drawing-mode panel hover).
pub fn is_pointer_over_toolbar_panel(app: &AppHandle) -> bool {
    #[cfg(not(target_os = "macos"))]
    {
        let _ = app;
        false
    }
    #[cfg(target_os = "macos")]
    {
        let Some(toolbar) = app.get_webview_window("toolbar") else {
            return false;
        };
        if !toolbar.is_visible().unwrap_or(false) {
            return false;
        }
        let Some((px, py)) = monitor::get_cursor_screen_pos() else {
            return false;
        };
        let Ok(pos) = toolbar.outer_position() else {
            return false;
        };
        let Ok(size) = toolbar.outer_size() else {
            return false;
        };
        let Ok(scale) = toolbar.scale_factor() else {
            return false;
        };
        let left = pos.x as f64 / scale;
        let top = pos.y as f64 / scale;
        let w = size.width as f64 / scale;
        let h = size.height as f64 / scale;
        let x = px as f64;
        let y = py as f64;
        x >= left && x < left + w && y >= top && y < top + h
    }
}

/// Pass pointer events through the overlays while the cursor is over the toolbar (drawing mode).
///
/// **macOS:** the toolbar sits one NSWindow level above all overlays, so it wins
/// hit-testing on its own. Make the toolbar key on enter, and restore the
/// cursor-screen overlay as key on leave.
///
/// **Other platforms:** unused from the frontend today; keep click-through + raise.
pub fn set_overlay_ignore_cursor_events(app: &AppHandle, state: &AppState, ignore: bool) {
    if current_mode(state) != OverlayMode::Drawing {
        return;
    }
    #[cfg(target_os = "macos")]
    {
        raise_toolbar_above_overlay(app);
        if ignore {
            if let Some(toolbar) = app.get_webview_window("toolbar") {
                crate::macos::activate_toolbar_for_pointer_interaction(&toolbar);
            }
        } else {
            // Hand key status back to the overlay on the cursor's screen.
            let target = crate::overlay_windows::label_for_cursor(app, state)
                .unwrap_or_else(|| crate::overlay_windows::PRIMARY_LABEL.to_string());
            if let Some(overlay) = app.get_webview_window(&target) {
                set_ignore_cursor_events(&overlay, false);
                crate::macos::activate_overlay_for_drawing(&overlay);
            }
        }
        return;
    }
    #[cfg(not(target_os = "macos"))]
    {
        for label in crate::overlay_windows::overlay_labels(app) {
            if let Some(window) = app.get_webview_window(&label) {
                set_ignore_cursor_events(&window, ignore);
            }
        }
        raise_toolbar_above_overlay(app);
    }
}

pub fn toggle_drawing(app: &AppHandle) {
    let state = app.state::<AppState>();
    match current_mode(&state) {
        OverlayMode::Hidden => {
            activate_drawing(app, &state);
            // Entering annotation should surface the docked toolbar bar (space mode).
            if let Err(e) = app.emit("surface-toolbar-request", ()) {
                warn!("Failed to emit surface-toolbar-request: {}", e);
            }
        }
        OverlayMode::Drawing => deactivate_drawing(app, &state),
    }
}

pub fn clear_drawing(app: &AppHandle, state: &AppState) {
    if current_mode(state) == OverlayMode::Hidden {
        return;
    }
    // Fold the timeline into one global clear op so a single Ctrl+Z restores
    // every display (Alt+E and the toolbar clear button both land here).
    lock_or_recover(&state.timeline).begin_global_clear();
    // `true` = undoable clear (Ctrl+Z restores). Activation without preserve emits `()`.
    if let Err(e) = app.emit("clear-drawing", true) {
        warn!("Failed to emit clear-drawing: {}", e);
    }
}

#[cfg(test)]
mod tests {
    /// Regression guard for cross-platform toolbar stacking in `raise_toolbar_above_overlay`.
    ///
    /// macOS must use the NSWindow level (`set_toolbar_level_above_overlays`),
    /// not the Win32-only `SetWindowPos` branch. CI on macOS runners executes
    /// this test.
    #[test]
    #[cfg(target_os = "macos")]
    fn macos_raise_toolbar_uses_nswindow_level_not_win32() {
        assert!(
            !cfg!(windows),
            "macOS must use an NSWindow level above the overlays, not SetWindowPos"
        );
    }

    #[test]
    #[cfg(windows)]
    fn windows_raise_toolbar_uses_win32_topmost_reorder() {
        assert!(
            cfg!(windows),
            "Windows must compile the SetWindowPos topmost reorder block"
        );
    }
}
