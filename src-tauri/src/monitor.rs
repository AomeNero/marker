use serde::Serialize;
use tauri::{AppHandle, WebviewWindow};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OverlayPointerPosition {
    pub x: f64,
    pub y: f64,
    pub screen_x: i32,
    pub screen_y: i32,
}

/// Screen-space cursor position in physical/global pixels (or platform equivalent).
#[cfg(target_os = "windows")]
pub fn get_cursor_screen_pos() -> Option<(i32, i32)> {
    use crate::win32::{GetCursorPos, POINT};
    unsafe {
        let mut pt = POINT { x: 0, y: 0 };
        if GetCursorPos(&mut pt) == 0 {
            return None;
        }
        Some((pt.x, pt.y))
    }
}

#[cfg(target_os = "macos")]
pub fn get_cursor_screen_pos() -> Option<(i32, i32)> {
    #[repr(C)]
    struct CGPoint {
        x: f64,
        y: f64,
    }
    extern "C" {
        fn CGEventCreate(source: *const std::ffi::c_void) -> *mut std::ffi::c_void;
        fn CGEventGetLocation(event: *const std::ffi::c_void) -> CGPoint;
        fn CFRelease(cf: *const std::ffi::c_void);
    }

    unsafe {
        let event = CGEventCreate(std::ptr::null());
        if event.is_null() {
            return None;
        }
        let pt = CGEventGetLocation(event);
        CFRelease(event);
        Some((pt.x as i32, pt.y as i32))
    }
}

/// Cursor position in `window`'s client coordinates (CSS pixels in that
/// window's webview), or `None` when the cursor is outside this window —
/// so only the overlay whose monitor the cursor is on reports a position.
pub fn get_overlay_client_pointer_for(window: &WebviewWindow) -> Option<OverlayPointerPosition> {
    let (screen_x, screen_y) = get_cursor_screen_pos()?;
    let pos = window.outer_position().ok()?;
    let size = window.outer_size().ok()?;
    if screen_x < pos.x
        || screen_y < pos.y
        || screen_x >= pos.x + size.width as i32
        || screen_y >= pos.y + size.height as i32
    {
        return None;
    }
    let dx = (screen_x - pos.x) as f64;
    let dy = (screen_y - pos.y) as f64;
    #[cfg(not(target_os = "macos"))]
    {
        let scale = window.scale_factor().ok()?;
        Some(OverlayPointerPosition {
            x: dx / scale,
            y: dy / scale,
            screen_x,
            screen_y,
        })
    }
    #[cfg(target_os = "macos")]
    {
        Some(OverlayPointerPosition {
            x: dx,
            y: dy,
            screen_x,
            screen_y,
        })
    }
}

/// Returns (x, y, width, height) of the monitor containing the cursor.
#[cfg(target_os = "windows")]
pub fn get_cursor_monitor_rect() -> Option<(i32, i32, u32, u32)> {
    crate::win32::get_cursor_monitor_rect_win32()
}

/// Work area (taskbar/Dock excluded) of the monitor containing the cursor.
#[cfg(target_os = "windows")]
pub fn get_cursor_monitor_work_rect() -> Option<(i32, i32, u32, u32)> {
    crate::win32::get_cursor_monitor_work_rect_win32()
}

#[cfg(target_os = "macos")]
pub fn get_cursor_monitor_work_rect() -> Option<(i32, i32, u32, u32)> {
    // Approximation: the Dock sits at the bottom edge; keep a fixed allowance.
    let (x, y, w, h) = get_cursor_monitor_rect()?;
    Some((x, y, w, h.saturating_sub(64)))
}

#[cfg(target_os = "macos")]
pub fn get_cursor_monitor_rect() -> Option<(i32, i32, u32, u32)> {
    #[repr(C)]
    struct CGPoint {
        x: f64,
        y: f64,
    }
    extern "C" {
        fn CGEventCreate(source: *const std::ffi::c_void) -> *mut std::ffi::c_void;
        fn CGEventGetLocation(event: *const std::ffi::c_void) -> CGPoint;
        fn CFRelease(cf: *const std::ffi::c_void);
    }

    let (cx, cy) = unsafe {
        let event = CGEventCreate(std::ptr::null());
        if event.is_null() {
            return None;
        }
        let pt = CGEventGetLocation(event);
        CFRelease(event);
        (pt.x as i32, pt.y as i32)
    };

    let monitor = xcap::Monitor::from_point(cx, cy).ok()?;
    let x = monitor.x().ok()?;
    let y = monitor.y().ok()?;
    let w = monitor.width().ok()?;
    let h = monitor.height().ok()?;
    Some((x, y, w, h))
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MonitorLogicalBounds {
    pub left: f64,
    pub top: f64,
    pub width: f64,
    pub height: f64,
}

/// Monitor bounds in logical coordinates for toolbar window positioning.
pub fn get_overlay_monitor_logical_bounds_for(window: &WebviewWindow) -> Option<MonitorLogicalBounds> {
    let monitor = window.current_monitor().ok()??;
    let pos = monitor.position();
    let size = monitor.size();
    let scale = monitor.scale_factor();
    Some(MonitorLogicalBounds {
        left: pos.x as f64 / scale,
        top: pos.y as f64 / scale,
        width: size.width as f64 / scale,
        height: size.height as f64 / scale,
    })
}

/// Work-area bounds of the window's monitor (taskbar excluded) in logical
/// coords, for docking the toolbar just above the taskbar.
pub fn get_overlay_monitor_work_logical_bounds_for(
    window: &WebviewWindow,
) -> Option<MonitorLogicalBounds> {
    #[cfg(target_os = "windows")]
    {
        let pos = window.outer_position().ok()?;
        let size = window.outer_size().ok()?;
        let cx = pos.x + (size.width as i32 / 2);
        let cy = pos.y + (size.height as i32 / 2);
        let (x, y, w, h) = crate::win32::get_monitor_work_rect_at_point_win32(cx, cy)?;
        let scale = window.scale_factor().ok()?;
        Some(MonitorLogicalBounds {
            left: x as f64 / scale,
            top: y as f64 / scale,
            width: w as f64 / scale,
            height: h as f64 / scale,
        })
    }
    #[cfg(not(target_os = "windows"))]
    {
        let b = get_overlay_monitor_logical_bounds_for(window)?;
        Some(MonitorLogicalBounds {
            height: (b.height - 64.0).max(b.height * 0.5),
            ..b
        })
    }
}

/// Full-monitor bounds, work-area bounds and scale of the monitor containing
/// the cursor — the toolbar docks and clamps against the *cursor's* screen
/// (decision: toolbar follows the cursor monitor).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CursorMonitorInfo {
    pub full: MonitorLogicalBounds,
    pub work: MonitorLogicalBounds,
    pub scale_factor: f64,
}

pub fn get_cursor_monitor_info(app: &AppHandle) -> Option<CursorMonitorInfo> {
    let (cx, cy) = get_cursor_screen_pos()?;
    let monitors = app.available_monitors().ok()?;
    for m in monitors {
        let pos = m.position();
        let size = m.size();
        if cx >= pos.x
            && cx < pos.x + size.width as i32
            && cy >= pos.y
            && cy < pos.y + size.height as i32
        {
            let scale = m.scale_factor();
            let full_pos = m.position();
            let full_size = m.size();
            let work = m.work_area();
            let work_pos = work.position;
            let work_size = work.size;
            return Some(CursorMonitorInfo {
                full: MonitorLogicalBounds {
                    left: full_pos.x as f64 / scale,
                    top: full_pos.y as f64 / scale,
                    width: full_size.width as f64 / scale,
                    height: full_size.height as f64 / scale,
                },
                work: MonitorLogicalBounds {
                    left: work_pos.x as f64 / scale,
                    top: work_pos.y as f64 / scale,
                    width: work_size.width as f64 / scale,
                    height: work_size.height as f64 / scale,
                },
                scale_factor: scale,
            });
        }
    }
    None
}

pub fn clamp_logical_position_to_monitor(
    left: f64,
    top: f64,
    panel_width: f64,
    panel_height: f64,
    monitor: &MonitorLogicalBounds,
    margin: f64,
) -> (f64, f64) {
    let min_left = monitor.left + margin;
    let min_top = monitor.top + margin;
    let max_left = (monitor.left + monitor.width - panel_width - margin).max(min_left);
    let max_top = (monitor.top + monitor.height - panel_height - margin).max(min_top);
    (left.clamp(min_left, max_left), top.clamp(min_top, max_top))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamp_logical_position_keeps_panel_inside_monitor() {
        let monitor = MonitorLogicalBounds {
            left: 0.0,
            top: 0.0,
            width: 1920.0,
            height: 1080.0,
        };
        let (x, y) = clamp_logical_position_to_monitor(100.0, 200.0, 272.0, 400.0, &monitor, 8.0);
        assert_eq!(x, 100.0);
        assert_eq!(y, 200.0);
    }

    #[test]
    fn clamp_logical_position_limits_right_edge() {
        let monitor = MonitorLogicalBounds {
            left: 0.0,
            top: 0.0,
            width: 1920.0,
            height: 1080.0,
        };
        let (x, _) = clamp_logical_position_to_monitor(2000.0, 200.0, 272.0, 400.0, &monitor, 8.0);
        assert_eq!(x, 1920.0 - 272.0 - 8.0);
    }
}
