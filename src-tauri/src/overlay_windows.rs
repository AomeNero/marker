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

    /// Arrangement-relative identity: position relative to the topology's
    /// bounding-box origin + size + scale. Normalizing by the bounding box
    /// keeps identity stable when the OS primary changes and shifts every
    /// absolute coordinate.
    fn topology_key(&self, origin: (i32, i32)) -> (i32, i32, u32, u32, i32) {
        (
            self.x - origin.0,
            self.y - origin.1,
            self.width,
            self.height,
            (self.scale_factor * 100.0).round() as i32,
        )
    }
}

fn topology_origin(specs: &[MonitorSpec]) -> (i32, i32) {
    (
        specs.iter().map(|m| m.x).min().unwrap_or(0),
        specs.iter().map(|m| m.y).min().unwrap_or(0),
    )
}

/// One topology diff between two enumerations.
#[allow(dead_code)] // wired by the hotplug topology watcher later in this feature
#[derive(Debug, Clone, PartialEq)]
pub enum TopoChange {
    /// Monitor present in `new` with no counterpart in `old`.
    Added(MonitorSpec),
    /// Monitor present in `old` with no counterpart in `new`.
    Removed(MonitorSpec),
    /// Matched monitor whose bounds or scale changed.
    Repositioned { from: MonitorSpec, to: MonitorSpec },
}

/// Diff two monitor topologies. Match priority: arrangement key first (stable
/// geometry), OS name second (survives resolution changes within a session —
/// geometry keys diverge when a display switches mode). Exact matches emit
/// nothing.
#[allow(dead_code)] // wired by the hotplug topology watcher later in this feature
pub fn diff_topology(old: &[MonitorSpec], new: &[MonitorSpec]) -> Vec<TopoChange> {
    let origin = topology_origin(new);
    let mut used = vec![false; new.len()];
    let mut changes = Vec::new();

    for o in old {
        let key = o.topology_key(origin);
        let mut idx = new
            .iter()
            .enumerate()
            .position(|(i, n)| !used[i] && n.topology_key(origin) == key);
        if idx.is_none() {
            if let Some(name) = &o.name {
                idx = new.iter().enumerate().position(|(i, n)| {
                    !used[i] && n.name.as_deref() == Some(name.as_str())
                });
            }
        }
        match idx {
            Some(i) => {
                used[i] = true;
                if !o.same_geometry(&new[i]) {
                    changes.push(TopoChange::Repositioned {
                        from: o.clone(),
                        to: new[i].clone(),
                    });
                }
            }
            None => changes.push(TopoChange::Removed(o.clone())),
        }
    }
    for (i, n) in new.iter().enumerate() {
        if !used[i] {
            changes.push(TopoChange::Added(n.clone()));
        }
    }
    changes
}

/// Deterministic label assignment for one activation: the cursor's monitor is
/// served by the static `overlay` window; remaining monitors pair with dynamic
/// labels in reading order (top-to-bottom, left-to-right).
pub fn assign_labels(
    monitors: &[MonitorSpec],
    cursor: &MonitorSpec,
) -> Vec<(String, MonitorSpec)> {
    let total_dynamic = monitors.len().saturating_sub(1);
    let labels: Vec<String> = (DYNAMIC_LABEL_BASE..DYNAMIC_LABEL_BASE + total_dynamic)
        .map(|i| format!("overlay-{i}"))
        .collect();
    let mut rest: Vec<&MonitorSpec> = monitors.iter().filter(|m| **m != *cursor).collect();
    rest.sort_by_key(|m| (m.y, m.x));
    let mut pairs = vec![(PRIMARY_LABEL.to_string(), cursor.clone())];
    pairs.extend(
        labels
            .into_iter()
            .zip(rest.into_iter().cloned()),
    );
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

/// Find the registry label that previously served `monitor`: exact geometry
/// match first, OS name as tiebreaker. Used when a monitor re-appears so its
/// old window (with the strokes still in that webview) is restored to it.
#[allow(dead_code)] // wired by the hotplug topology watcher later in this feature
pub fn find_window_for_monitor(
    registry: &MonitorRegistry,
    monitor: &MonitorSpec,
) -> Option<String> {
    let mut labels: Vec<&String> = registry.keys().collect();
    labels.sort();
    let mut name_match = None;
    for label in labels {
        let entry = &registry[label];
        if entry.spec.same_geometry(monitor) {
            return Some(label.clone());
        }
        if name_match.is_none() && entry.spec.name.is_some() && entry.spec.name == monitor.name {
            name_match = Some(label.clone());
        }
    }
    name_match
}

/// Sorted overlay labels (static first, then numeric suffix order).
pub fn label_sort_key(label: &str) -> (bool, usize) {
    match label.strip_prefix("overlay-").and_then(|s| s.parse().ok()) {
        Some(n) => (true, n),
        None => (false, 0), // "overlay" sorts before dynamic labels
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

/// Create hidden dynamic overlay windows (`overlay-2..`) so activation never
/// waits on webview startup. Idempotent; skips labels that already exist.
pub fn ensure_extra_overlay_windows(app: &AppHandle, dynamic_count: usize) {
    for i in DYNAMIC_LABEL_BASE..DYNAMIC_LABEL_BASE + dynamic_count {
        let label = format!("overlay-{i}");
        if app.get_webview_window(&label).is_some() {
            continue;
        }
        let url = WebviewUrl::App("index.html".into());
        let builder = WebviewWindowBuilder::new(app, &label, url)
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
            }
            Err(e) => warn!("Failed to create overlay window {}: {}", label, e),
        }
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
    window.show().ok();
}

/// Assign non-cursor monitors to dynamic overlay windows, position + show
/// them, and record every assignment (including the static window's) in the
/// registry. Called from `activate_drawing` after `setup_overlay_size` placed
/// the static window on the cursor monitor.
pub fn assign_and_show_extra_overlays(app: &AppHandle, state: &AppState) {
    let monitors = enumerate_monitors(app);
    let Some(cursor) = cursor_monitor(&monitors) else {
        warn!("Cursor monitor not found during overlay assignment");
        return;
    };
    let specs: Vec<MonitorSpec> = monitors.iter().map(|(m, _)| m.clone()).collect();
    let pairs = assign_labels(&specs, &cursor);
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
        if let Some((_, tauri_monitor)) = monitors
            .iter()
            .find(|(m, _)| m.same_geometry(spec))
        {
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
    fn topology_key_normalizes_by_bounding_box_origin() {
        // Same arrangement; second has the OS primary on the right monitor,
        // which shifts every absolute coordinate by -1920.
        let a = [mon(0, 0, 1920, 1080, 1.0, Some("A")), mon(1920, 0, 2560, 1440, 1.5, Some("B"))];
        let b = [mon(-1920, 0, 1920, 1080, 1.0, Some("A")), mon(0, 0, 2560, 1440, 1.5, Some("B"))];
        let origin_a = topology_origin(&a);
        let origin_b = topology_origin(&b);
        assert_eq!(a[1].topology_key(origin_a), b[1].topology_key(origin_b));
        assert_eq!(a[0].topology_key(origin_a), b[0].topology_key(origin_b));
    }

    // ---- diff_topology ----------------------------------------------------

    #[test]
    fn diff_identical_topologies_emits_nothing() {
        let t = [mon(0, 0, 1920, 1080, 1.0, Some("A")), mon(1920, 0, 1920, 1080, 1.0, Some("B"))];
        assert!(diff_topology(&t, &t).is_empty());
    }

    #[test]
    fn diff_detects_added_and_removed() {
        let old = [mon(0, 0, 1920, 1080, 1.0, Some("A"))];
        let new = [
            mon(0, 0, 1920, 1080, 1.0, Some("A")),
            mon(1920, 0, 1920, 1080, 1.0, Some("B")),
        ];
        let changes = diff_topology(&old, &new);
        assert_eq!(changes, vec![TopoChange::Added(new[1].clone())]);

        let changes = diff_topology(&new, &old);
        assert_eq!(changes, vec![TopoChange::Removed(new[1].clone())]);
    }

    #[test]
    fn diff_name_match_reports_reposition_on_resolution_change() {
        let old = [
            mon(0, 0, 1920, 1080, 1.0, Some("A")),
            mon(1920, 0, 1920, 1080, 1.0, Some("B")),
        ];
        // B switches resolution: geometry differs but the name survives.
        let new = [
            mon(0, 0, 1920, 1080, 1.0, Some("A")),
            mon(1920, 0, 1280, 720, 1.0, Some("B")),
        ];
        let changes = diff_topology(&old, &new);
        assert_eq!(
            changes,
            vec![TopoChange::Repositioned {
                from: old[1].clone(),
                to: new[1].clone()
            }]
        );
    }

    #[test]
    fn diff_geometry_match_survives_windows_renumbering() {
        let old = [
            mon(0, 0, 1920, 1080, 1.0, Some("\\\\.\\DISPLAY1")),
            mon(1920, 0, 1920, 1080, 1.5, Some("\\\\.\\DISPLAY2")),
        ];
        // Replug renumbers the OS names but the arrangement is unchanged.
        let new = [
            mon(0, 0, 1920, 1080, 1.0, Some("\\\\.\\DISPLAY2")),
            mon(1920, 0, 1920, 1080, 1.5, Some("\\\\.\\DISPLAY1")),
        ];
        assert!(diff_topology(&old, &new).is_empty());
    }

    #[test]
    fn diff_is_independent_of_input_order() {
        let old = [
            mon(1920, 0, 1920, 1080, 1.0, Some("B")),
            mon(0, 0, 1920, 1080, 1.0, Some("A")),
        ];
        let new = [
            mon(0, 0, 1920, 1080, 1.0, Some("A")),
            mon(-1920, 0, 1920, 1080, 1.0, Some("B")),
        ];
        // B moved from the right of A to the left: one reposition, order-agnostic.
        let changes = diff_topology(&old, &new);
        assert!(matches!(changes.as_slice(), &[TopoChange::Repositioned { .. }]));
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
        let dynamic: Vec<(&str, &MonitorSpec)> = pairs[1..]
            .iter()
            .map(|(l, m)| (l.as_str(), m))
            .collect();
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

    // ---- find_window_for_monitor -------------------------------------------

    #[test]
    fn find_window_exact_geometry_match_wins() {
        let mut registry = MonitorRegistry::new();
        registry.insert(
            "overlay-2".into(),
            MonitorEntry {
                spec: mon(1920, 0, 1920, 1080, 1.0, Some("\\\\.\\DISPLAY9")),
                hidden: true,
            },
        );
        let monitor = mon(1920, 0, 1920, 1080, 1.0, Some("\\\\.\\DISPLAY1"));
        assert_eq!(
            find_window_for_monitor(&registry, &monitor).as_deref(),
            Some("overlay-2")
        );
    }

    #[test]
    fn find_window_falls_back_to_name() {
        let mut registry = MonitorRegistry::new();
        registry.insert(
            "overlay-2".into(),
            MonitorEntry {
                spec: mon(0, 0, 1280, 720, 1.0, Some("B")),
                hidden: true,
            },
        );
        // Same name, different resolution (user changed it while unplugged).
        let monitor = mon(1920, 0, 1920, 1080, 1.0, Some("B"));
        assert_eq!(
            find_window_for_monitor(&registry, &monitor).as_deref(),
            Some("overlay-2")
        );
    }

    #[test]
    fn find_window_no_match_returns_none() {
        let mut registry = MonitorRegistry::new();
        registry.insert(
            "overlay-2".into(),
            MonitorEntry {
                spec: mon(0, 0, 1280, 720, 1.0, Some("B")),
                hidden: true,
            },
        );
        let monitor = mon(1920, 0, 1920, 1080, 1.5, Some("C"));
        assert_eq!(find_window_for_monitor(&registry, &monitor), None);
    }

    // ---- labels ------------------------------------------------------------

    #[test]
    fn label_sort_key_static_first_then_numeric() {
        let mut labels = vec!["overlay-10", "overlay", "overlay-2"];
        labels.sort_by_key(|l| label_sort_key(l));
        assert_eq!(labels, vec!["overlay", "overlay-2", "overlay-10"]);
    }
}
