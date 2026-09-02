//! Multi-monitor overlay window orchestration.
//!
//! The static `overlay` window (declared in `tauri.conf.json`) always serves the
//! monitor where the cursor is when annotation activates — identical to the
//! pre-multi-monitor single-screen path. Additional monitors get dynamic
//! windows `overlay-2`, `overlay-3`, … created hidden ahead of time and only
//! positioned + shown at activation. Window labels are the ownership handles
//! used by the timeline / shared-state layers; the registry maps label → monitor.

use serde::Serialize;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder};
use tracing::{info, warn};

use crate::config::{lock_or_recover, AppState};

/// Label of the static overlay window declared in tauri.conf.json.
pub const PRIMARY_LABEL: &str = "overlay";
/// Lowest suffix of dynamic overlay windows (`overlay-2`, `overlay-3`, …).
pub const DYNAMIC_LABEL_BASE: usize = 2;

/// A connected monitor in physical pixels.
#[derive(Debug, Clone, PartialEq)]
pub struct MonitorSpec {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub scale_factor: f64,
    /// OS-reported name. Renumbered on Windows after replug, so topology
    /// matching uses geometry first and only falls back to the name.
    pub name: Option<String>,
}

impl MonitorSpec {
    pub fn contains_point(&self, x: i32, y: i32) -> bool {
        x >= self.x
            && x < self.x + self.width as i32
            && y >= self.y
            && y < self.y + self.height as i32
    }

    /// Geometry identity (position, size, scale) — deliberately excludes the
    /// OS name, which Windows renumbers across replug.
    fn same_geometry(&self, other: &MonitorSpec) -> bool {
        self.x == other.x
            && self.y == other.y
            && self.width == other.width
            && self.height == other.height
            && (self.scale_factor * 100.0).round() == (other.scale_factor * 100.0).round()
    }

    /// Pairing rule shared by activation and the hotplug watcher: geometry
    /// first, OS name second (survives resolution changes within a session).
    fn matches(&self, other: &MonitorSpec) -> bool {
        self.same_geometry(other) || self.name.is_some() && self.name == other.name
    }
}

/// Deterministic label assignment for one activation: the cursor's monitor is
/// served by the static `overlay` window; remaining monitors pair with dynamic
/// labels in reading order (top-to-bottom, left-to-right).
pub fn assign_labels(monitors: &[MonitorSpec], cursor: &MonitorSpec) -> Vec<(String, MonitorSpec)> {
    let total_dynamic = monitors.len().saturating_sub(1);
    let labels: Vec<String> = (DYNAMIC_LABEL_BASE..DYNAMIC_LABEL_BASE + total_dynamic)
        .map(|i| format!("overlay-{i}"))
        .collect();
    let mut rest: Vec<&MonitorSpec> = monitors.iter().filter(|m| **m != *cursor).collect();
    rest.sort_by_key(|m| (m.y, m.x));
    let mut pairs = vec![(PRIMARY_LABEL.to_string(), cursor.clone())];
    pairs.extend(labels.into_iter().zip(rest.into_iter().cloned()));
    pairs
}

/// Registry entry: which monitor a window currently serves.
#[derive(Debug, Clone)]
pub struct MonitorEntry {
    pub spec: MonitorSpec,
    /// Window currently hidden (session inactive or monitor lost).
    pub hidden: bool,
}

/// label → assignment.
pub type MonitorRegistry = HashMap<String, MonitorEntry>;

/// Sorted overlay labels (static first, then numeric suffix order).
pub fn label_sort_key(label: &str) -> (bool, usize) {
    match label.strip_prefix("overlay-").and_then(|s| s.parse().ok()) {
        Some(n) => (true, n),
        None => (false, 0), // "overlay" sorts before dynamic labels
    }
}

/// Lowest unused dynamic overlay label (`overlay-2`, `overlay-3`, …).
fn next_dynamic_label(taken: &[String]) -> String {
    let mut i = DYNAMIC_LABEL_BASE;
    loop {
        let candidate = format!("overlay-{i}");
        if !taken.iter().any(|l| l == &candidate) {
            return candidate;
        }
        i += 1;
    }
}

/// The overlay window label whose monitor currently contains the cursor —
/// the single executor for toolbar actions (the screen the user is on).
pub fn label_for_cursor(_app: &AppHandle, state: &AppState) -> Option<String> {
    let (cx, cy) = crate::monitor::get_cursor_screen_pos()?;
    let registry = lock_or_recover(&state.monitors);
    let mut labels: Vec<&String> = registry.keys().collect();
    labels.sort();
    for label in labels {
        if registry[label].spec.contains_point(cx, cy) {
            return Some(label.clone());
        }
    }
    None
}

/// Overlay window labels currently registered with the app, sorted
/// (static `overlay` first, then `overlay-2`, `overlay-3`, …).
pub fn overlay_labels(app: &AppHandle) -> Vec<String> {
    let mut labels: Vec<String> = app
        .webview_windows()
        .into_keys()
        .filter(|l| l == PRIMARY_LABEL || l.starts_with("overlay-"))
        .collect();
    labels.sort_by_key(|l| label_sort_key(l));
    labels
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MonitorAssigned {
    pub label: String,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub scale_factor: f64,
}

/// Enumerate connected monitors (physical pixels) paired with their Tauri
/// handles for placement.
pub fn enumerate_monitors(app: &AppHandle) -> Vec<(MonitorSpec, tauri::Monitor)> {
    let Ok(monitors) = app.available_monitors() else {
        warn!("Failed to enumerate monitors");
        return Vec::new();
    };
    monitors
        .into_iter()
        .map(|m| {
            let pos = m.position();
            let size = m.size();
            let spec = MonitorSpec {
                x: pos.x,
                y: pos.y,
                width: size.width,
                height: size.height,
                scale_factor: m.scale_factor(),
                name: m.name().cloned(),
            };
            (spec, m)
        })
        .collect()
}

/// The monitor containing the cursor, from an enumerated set.
pub fn cursor_monitor(monitors: &[(MonitorSpec, tauri::Monitor)]) -> Option<MonitorSpec> {
    let (cx, cy) = crate::monitor::get_cursor_screen_pos()?;
    monitors
        .iter()
        .find(|(m, _)| m.contains_point(cx, cy))
        .map(|(m, _)| m.clone())
}

// ---------------------------------------------------------------------------
// Side-effecting orchestration (thin layer over the pure logic above)
// ---------------------------------------------------------------------------

/// Create a dynamic overlay window (hidden, click-through until assigned).
/// Idempotent: returns the existing window when the label is already taken.
fn create_dynamic_overlay_window(app: &AppHandle, label: &str) -> Option<WebviewWindow> {
    if let Some(existing) = app.get_webview_window(label) {
        return Some(existing);
    }
    let url = WebviewUrl::App("index.html".into());
    let builder = WebviewWindowBuilder::new(app, label, url)
        .title("Marker")
        .transparent(true)
        .decorations(false)
        .always_on_top(true)
        .shadow(false)
        .skip_taskbar(true)
        .resizable(false)
        .visible(false)
        .focused(false);
    match builder.build() {
        Ok(window) => {
            window.set_ignore_cursor_events(true).ok();
            crate::overlay::reassert_window_transparency(&window);
            info!("created hidden overlay window {}", label);
            Some(window)
        }
        Err(e) => {
            warn!("Failed to create overlay window {}: {}", label, e);
            None
        }
    }
}

/// Create hidden dynamic overlay windows (`overlay-2..`) so activation never
/// waits on webview startup. Idempotent; skips labels that already exist.
pub fn ensure_extra_overlay_windows(app: &AppHandle, dynamic_count: usize) {
    for i in DYNAMIC_LABEL_BASE..DYNAMIC_LABEL_BASE + dynamic_count {
        let label = format!("overlay-{i}");
        create_dynamic_overlay_window(app, &label);
    }
}

/// Position an overlay window over a monitor's work area. Windows uses one
/// physical SetWindowPos (cross-DPI safe); macOS converts to logical using the
/// target monitor's own scale factor.
pub fn place_overlay_on_monitor(window: &WebviewWindow, monitor: &tauri::Monitor) {
    let work = monitor.work_area();
    let pos = work.position;
    let size = work.size;
    #[cfg(windows)]
    {
        if let Ok(hwnd) = window.hwnd() {
            crate::win32::position_window_on_monitor(
                hwnd.0 as isize,
                pos.x,
                pos.y,
                size.width,
                size.height,
            );
            return;
        }
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
    #[cfg(not(windows))]
    {
        let scale = monitor.scale_factor();
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
}

/// Show an overlay window without stealing keyboard focus.
pub fn show_overlay_no_activate(window: &WebviewWindow) {
    #[cfg(windows)]
    {
        if let Ok(hwnd) = window.hwnd() {
            crate::win32::show_window_no_activate(hwnd.0 as isize);
            return;
        }
    }
    #[cfg(target_os = "macos")]
    {
        crate::macos::order_front_regardless(window);
        return;
    }
    window.show().ok();
}

/// Assign non-cursor monitors to dynamic overlay windows, position + show
/// them, and record every assignment (including the static window's) in the
/// registry. Called from `activate_drawing` after `setup_overlay_size` placed
/// the static window on the cursor monitor.
///
/// Strokes live in each window's webview, so label↔monitor pairing prefers
/// the previous session's registry entries (geometry / name match) before
/// falling back to the deterministic reading order — preserve_drawings then
/// restores each display's own annotations even when the cursor screen or
/// monitor order changed between sessions.
pub fn assign_and_show_extra_overlays(app: &AppHandle, state: &AppState) {
    let monitors = enumerate_monitors(app);
    let Some(cursor) = cursor_monitor(&monitors) else {
        warn!("Cursor monitor not found during overlay assignment");
        return;
    };
    let specs: Vec<MonitorSpec> = monitors.iter().map(|(m, _)| m.clone()).collect();
    let proposed = assign_labels(&specs, &cursor);

    // Continuity pass: reuse a label's previous monitor when it re-appears.
    let pairs: Vec<(String, MonitorSpec)> = {
        let registry = lock_or_recover(&state.monitors);
        let mut taken: Vec<String> = Vec::new();
        let mut resolved: Vec<(String, MonitorSpec)> = Vec::new();
        for (label, spec) in proposed {
            if label == PRIMARY_LABEL {
                resolved.push((label, spec));
                continue;
            }
            let previous = registry
                .iter()
                .filter(|(l, entry)| {
                    entry.spec.matches(&spec) && !taken.contains(l) && l != &PRIMARY_LABEL
                })
                .map(|(l, _)| l.clone())
                .min(); // deterministic pick if several match
            match previous {
                Some(prev) => {
                    taken.push(prev.clone());
                    resolved.push((prev, spec));
                }
                None => {
                    taken.push(label.clone());
                    resolved.push((label, spec));
                }
            }
        }
        resolved
    };

    ensure_extra_overlay_windows(app, pairs.len().saturating_sub(1));

    let mut assigned: Vec<String> = Vec::new();
    for (label, spec) in &pairs {
        assigned.push(label.clone());
        if label == PRIMARY_LABEL {
            // The static window was already placed, shown and focused by
            // `activate_drawing`; just record the assignment.
            continue;
        }
        let Some(window) = app.get_webview_window(label) else {
            continue;
        };
        if let Some((_, tauri_monitor)) = monitors.iter().find(|(m, _)| m.same_geometry(spec)) {
            place_overlay_on_monitor(&window, tauri_monitor);
        }
        window.set_ignore_cursor_events(false).ok();
        show_overlay_no_activate(&window);
        window.set_always_on_top(true).ok();
        crate::overlay::reassert_window_transparency(&window);
        if let Err(e) = app.emit_to(
            label,
            "overlay-monitor-assigned",
            MonitorAssigned {
                label: label.clone(),
                x: spec.x,
                y: spec.y,
                width: spec.width,
                height: spec.height,
                scale_factor: spec.scale_factor,
            },
        ) {
            warn!("Failed to emit overlay-monitor-assigned: {}", e);
        }
    }

    // Update the registry; hide dynamic windows no longer backed by a monitor
    // (topology shrank since the last session).
    let mut registry = lock_or_recover(&state.monitors);
    for (label, spec) in &pairs {
        registry.insert(
            label.clone(),
            MonitorEntry {
                spec: spec.clone(),
                hidden: false,
            },
        );
    }
    for label in overlay_labels(app) {
        if !assigned.contains(&label) {
            if let Some(entry) = registry.get_mut(&label) {
                entry.hidden = true;
            }
            if let Some(window) = app.get_webview_window(&label) {
                window.set_ignore_cursor_events(true).ok();
                window.hide().ok();
            }
        }
    }
}

/// Hide every overlay window (webviews stay alive, strokes preserved) and
/// mark registry entries hidden. Inverse of activation.
pub fn deactivate_overlays(app: &AppHandle, state: &AppState) {
    let mut registry = lock_or_recover(&state.monitors);
    for label in overlay_labels(app) {
        if let Some(window) = app.get_webview_window(&label) {
            window.set_ignore_cursor_events(true).ok();
            window.hide().ok();
        }
        if let Some(entry) = registry.get_mut(&label) {
            entry.hidden = true;
        }
    }
}

// ---------------------------------------------------------------------------
// Hotplug topology watcher (annotation session only)
// ---------------------------------------------------------------------------

/// Session-scoped flag driving the 1s monitor-poll loop.
static TOPOLOGY_WATCH_ACTIVE: AtomicBool = AtomicBool::new(false);

/// Poll the monitor topology once per second while annotation is active.
/// A lost display hides its window (the webview keeps the strokes); a
/// restored display gets its old window back (geometry / name match); a
/// newly-connected display gets a fresh window.
pub fn start_topology_watcher(app: &AppHandle) {
    if TOPOLOGY_WATCH_ACTIVE.swap(true, Ordering::SeqCst) {
        return; // already running
    }
    let handle = app.clone();
    std::thread::spawn(move || {
        while TOPOLOGY_WATCH_ACTIVE.load(Ordering::SeqCst) {
            std::thread::sleep(Duration::from_millis(1000));
            if !TOPOLOGY_WATCH_ACTIVE.load(Ordering::SeqCst) {
                break;
            }
            let app_for_tick = handle.clone();
            let _ = handle.run_on_main_thread(move || {
                topology_watch_tick(&app_for_tick);
            });
        }
    });
}

pub fn stop_topology_watcher() {
    TOPOLOGY_WATCH_ACTIVE.store(false, Ordering::SeqCst);
}

fn assigned_payload(label: &str, spec: &MonitorSpec) -> MonitorAssigned {
    MonitorAssigned {
        label: label.to_string(),
        x: spec.x,
        y: spec.y,
        width: spec.width,
        height: spec.height,
        scale_factor: spec.scale_factor,
    }
}

/// Window action deferred out of the registry critical section (M-5 pattern:
/// collect under lock, apply after drop so no emit/window op holds the lock).
enum PendingAction {
    /// Restore a paired window to its (possibly moved) monitor.
    Restore {
        label: String,
        spec: MonitorSpec,
        monitor: tauri::Monitor,
    },
    /// Hide a window whose monitor vanished (webview data preserved).
    Hide { label: String },
    /// Freshly connected display → brand-new window.
    Create {
        spec: MonitorSpec,
        monitor: tauri::Monitor,
    },
}

fn topology_watch_tick(app: &AppHandle) {
    let state = app.state::<AppState>();
    if crate::overlay::current_mode(&state) != crate::overlay::OverlayMode::Drawing {
        return;
    }
    let monitors = enumerate_monitors(app);
    let specs: Vec<MonitorSpec> = monitors.iter().map(|(m, _)| m.clone()).collect();
    let mut pending: Vec<PendingAction> = Vec::new();

    {
        let mut registry = lock_or_recover(&state.monitors);
        let mut used = vec![false; monitors.len()];

        // Pair each registry entry with a monitor (shared pairing rule:
        // geometry first, OS name as tiebreaker for resolution changes).
        // Unpaired → lost; geometry drift → reposition. Steady state emits
        // nothing at all.
        let labels: Vec<String> = registry.keys().cloned().collect();
        for label in labels {
            let Some(entry) = registry.get_mut(&label) else {
                continue;
            };
            let idx = specs
                .iter()
                .enumerate()
                .find(|(i, m)| !used[*i] && entry.spec.matches(m))
                .map(|(i, _)| i);
            match idx {
                Some(i) => {
                    used[i] = true;
                    let m = specs[i].clone();
                    if !entry.spec.same_geometry(&m) || entry.hidden {
                        entry.spec = m.clone();
                        entry.hidden = false;
                        if let Some((_, tmon)) = monitors.get(i) {
                            pending.push(PendingAction::Restore {
                                label,
                                spec: m,
                                monitor: tmon.clone(),
                            });
                        }
                    }
                }
                None => {
                    if !entry.hidden {
                        entry.hidden = true;
                        pending.push(PendingAction::Hide { label });
                    }
                }
            }
        }

        // Newly connected displays (no registry pairing): a brand-new window.
        // Dormant windows of still-lost displays are never recycled — their
        // strokes belong to that display and reappear when it returns.
        for (i, (_, tmon)) in monitors.iter().enumerate() {
            if used[i] {
                continue;
            }
            pending.push(PendingAction::Create {
                spec: specs[i].clone(),
                monitor: tmon.clone(),
            });
        }
    }

    if pending.is_empty() {
        return;
    }
    // Session may have ended while we computed the diff.
    if crate::overlay::current_mode(&state) != crate::overlay::OverlayMode::Drawing {
        return;
    }

    for action in pending {
        match action {
            PendingAction::Hide { label } => {
                if let Some(window) = app.get_webview_window(&label) {
                    window.set_ignore_cursor_events(true).ok();
                    window.hide().ok();
                }
                let _ = app.emit_to(&label, "overlay-monitor-lost", ());
            }
            PendingAction::Restore {
                label,
                spec,
                monitor,
            } => {
                if let Some(window) = app.get_webview_window(&label) {
                    apply_overlay_assignment(&window, &monitor);
                }
                let _ = app.emit_to(
                    &label,
                    "overlay-monitor-restored",
                    assigned_payload(&label, &spec),
                );
            }
            PendingAction::Create { spec, monitor } => {
                let mut taken = overlay_labels(app);
                {
                    let registry = lock_or_recover(&state.monitors);
                    taken.extend(registry.keys().cloned());
                }
                let label = next_dynamic_label(&taken);
                create_dynamic_overlay_window(app, &label);
                if let Some(window) = app.get_webview_window(&label) {
                    apply_overlay_assignment(&window, &monitor);
                }
                lock_or_recover(&state.monitors).insert(
                    label.clone(),
                    MonitorEntry {
                        spec: spec.clone(),
                        hidden: false,
                    },
                );
                let _ = app.emit_to(
                    &label,
                    "overlay-monitor-restored",
                    assigned_payload(&label, &spec),
                );
            }
        }
    }

    crate::overlay::notify_overlay_geometry_changed(app);
    crate::overlay::raise_toolbar_above_overlay(app);
}

/// Place, reveal (no focus steal), topmost and re-assert transparency — the
/// shared tail of every overlay assignment/restore path.
fn apply_overlay_assignment(window: &WebviewWindow, monitor: &tauri::Monitor) {
    place_overlay_on_monitor(window, monitor);
    window.set_ignore_cursor_events(false).ok();
    show_overlay_no_activate(window);
    window.set_always_on_top(true).ok();
    crate::overlay::reassert_window_transparency(window);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mon(x: i32, y: i32, w: u32, h: u32, scale: f64, name: Option<&str>) -> MonitorSpec {
        MonitorSpec {
            x,
            y,
            width: w,
            height: h,
            scale_factor: scale,
            name: name.map(|s| s.to_string()),
        }
    }

    // ---- MonitorSpec ------------------------------------------------------

    #[test]
    fn contains_point_is_half_open() {
        let m = mon(0, 0, 1920, 1080, 1.0, None);
        assert!(m.contains_point(0, 0));
        assert!(m.contains_point(1919, 1079));
        assert!(!m.contains_point(1920, 0));
        assert!(!m.contains_point(-1, 0));
    }

    #[test]
    fn matches_geometry_first_then_name() {
        let a = mon(0, 0, 1920, 1080, 1.0, Some("A"));
        // Same geometry, renumbered OS name → still a match.
        assert!(a.matches(&mon(0, 0, 1920, 1080, 1.0, Some("B"))));
        // Resolution changed but the OS name survived (mode switch).
        assert!(a.matches(&mon(0, 0, 1280, 720, 1.0, Some("A"))));
        // Different geometry AND name → different monitor.
        assert!(!a.matches(&mon(1920, 0, 1920, 1080, 1.5, Some("C"))));
    }

    // ---- assign_labels ----------------------------------------------------

    #[test]
    fn assign_labels_cursor_screen_gets_static_window() {
        let monitors = [
            mon(0, 0, 1920, 1080, 1.0, Some("A")),
            mon(1920, 0, 1920, 1080, 1.0, Some("B")),
        ];
        let pairs = assign_labels(&monitors, &monitors[1]);
        assert_eq!(pairs[0].0, "overlay");
        assert_eq!(pairs[0].1, monitors[1]);
        assert_eq!(pairs.len(), 2);
        assert_eq!(pairs[1].0, "overlay-2");
        assert_eq!(pairs[1].1, monitors[0]);
    }

    #[test]
    fn assign_labels_remaining_in_reading_order() {
        // Two extra monitors stacked above/below and right.
        let monitors = [
            mon(0, 0, 1920, 1080, 1.0, Some("A")),     // cursor
            mon(0, -1080, 1920, 1080, 1.0, Some("T")), // above
            mon(1920, 0, 1920, 1080, 1.0, Some("R")),  // right
        ];
        let pairs = assign_labels(&monitors, &monitors[0]);
        let dynamic: Vec<(&str, &MonitorSpec)> =
            pairs[1..].iter().map(|(l, m)| (l.as_str(), m)).collect();
        // Reading order: top-to-bottom first, so the monitor above (T) gets
        // the lower label before the one to the right (R).
        assert_eq!(
            dynamic,
            vec![("overlay-2", &monitors[1]), ("overlay-3", &monitors[2])]
        );
    }

    #[test]
    fn assign_labels_single_monitor_only_static() {
        let monitors = [mon(0, 0, 1920, 1080, 1.0, Some("A"))];
        let pairs = assign_labels(&monitors, &monitors[0]);
        assert_eq!(pairs.len(), 1);
        assert_eq!(pairs[0].0, "overlay");
    }

    // ---- labels ------------------------------------------------------------

    #[test]
    fn label_sort_key_static_first_then_numeric() {
        let mut labels = vec!["overlay-10", "overlay", "overlay-2"];
        labels.sort_by_key(|l| label_sort_key(l));
        assert_eq!(labels, vec!["overlay", "overlay-2", "overlay-10"]);
    }

    #[test]
    fn next_dynamic_label_starts_at_two_and_skips_taken() {
        assert_eq!(next_dynamic_label(&[]), "overlay-2");
        assert_eq!(
            next_dynamic_label(&["overlay".into(), "overlay-2".into()]),
            "overlay-3"
        );
        assert_eq!(
            next_dynamic_label(&["overlay".into(), "overlay-2".into(), "overlay-3".into()]),
            "overlay-4"
        );
        // Gaps in the numbering are reused (window was destroyed / never made).
        assert_eq!(
            next_dynamic_label(&["overlay".into(), "overlay-3".into()]),
            "overlay-2"
        );
    }
}
