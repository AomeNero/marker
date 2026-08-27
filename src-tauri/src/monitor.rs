use serde::Serialize;
use std::sync::Mutex;
use tauri::{AppHandle, Manager};

/// Monitor bounds captured when drawing mode activates (physical pixels).
type DrawingMonitorClip = (i32, i32, u32, u32);

static DRAWING_MONITOR_CLIP: Mutex<Option<DrawingMonitorClip>> = Mutex::new(None);

#[cfg(windows)]
fn clip_rect_from_monitor((x, y, w, h): DrawingMonitorClip) -> crate::win32::RECT {
    crate::win32::RECT {
        left: x,
        top: y,
        right: x + w as i32,
        bottom: y + h as i32,
    }
}

/// Remember the overlay monitor and confine the cursor to it (drawing mode).
pub fn remember_and_clip_drawing_monitor(app: &AppHandle) {
    #[cfg(target_os = "macos")]
    let rect = get_cursor_monitor_rect().or_else(|| get_overlay_monitor_rect(app));
    #[cfg(not(target_os = "macos"))]
    let rect = get_overlay_monitor_rect(app).or_else(get_cursor_monitor_rect);
    let Some(rect) = rect else {
        return;
    };
    if let Ok(mut stored) = DRAWING_MONITOR_CLIP.lock() {
        *stored = Some(rect);
    }
    apply_drawing_cursor_clip();
}

/// Re-apply cursor clip after leaving penetration mode.
pub fn apply_drawing_cursor_clip() {
    #[cfg(windows)]
    {
        let Some(bounds) = DRAWING_MONITOR_CLIP.lock().ok().and_then(|guard| *guard) else {
            return;
        };
        let rc = clip_rect_from_monitor(bounds);
        if !crate::win32::clip_cursor_to_rect(Some(&rc)) {
            tracing::warn!(
                "ClipCursor failed for monitor {},{}, {}x{}",
                bounds.0,
                bounds.1,
                bounds.2,
                bounds.3
            );
        }
    }
    #[cfg(target_os = "macos")]
    {
        let Some(bounds) = DRAWING_MONITOR_CLIP.lock().ok().and_then(|guard| *guard) else {
            return;
        };
        crate::macos_cursor::start_cursor_clip(bounds);
    }
}

/// Temporarily release cursor confinement (e.g. click-through) while keeping monitor bounds.
pub fn suspend_drawing_cursor_clip() {
    #[cfg(windows)]
    crate::win32::release_cursor_clip();
    #[cfg(target_os = "macos")]
    crate::macos_cursor::stop_cursor_clip();
}

/// Release cursor confinement and forget the active monitor bounds.
pub fn release_drawing_cursor_clip() {
    suspend_drawing_cursor_clip();
    #[cfg(target_os = "macos")]
    crate::macos_cursor::release_cursor_clip();
    if let Ok(mut stored) = DRAWING_MONITOR_CLIP.lock() {
        *stored = None;
    }
}

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

/// Cursor position in overlay client coordinates (CSS pixels in the webview).
pub fn get_overlay_client_pointer(app: &AppHandle) -> Option<OverlayPointerPosition> {
    let (screen_x, screen_y) = get_cursor_screen_pos()?;
    let (mon_x, mon_y, _, _) = get_cursor_monitor_rect()?;
    let dx = (screen_x - mon_x) as f64;
    let dy = (screen_y - mon_y) as f64;
    #[cfg(not(target_os = "macos"))]
    {
        let window = app.get_webview_window("overlay")?;
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
        app.get_webview_window("overlay")?;
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

#[cfg(target_os = "windows")]
fn get_monitor_rect_at_point(x: i32, y: i32) -> Option<(i32, i32, u32, u32)> {
    crate::win32::get_monitor_rect_at_point_win32(x, y)
}

#[cfg(target_os = "macos")]
fn get_monitor_rect_at_point(x: i32, y: i32) -> Option<(i32, i32, u32, u32)> {
    let monitor = xcap::Monitor::from_point(x, y).ok()?;
    let x = monitor.x().ok()?;
    let y = monitor.y().ok()?;
    let w = monitor.width().ok()?;
    let h = monitor.height().ok()?;
    Some((x, y, w, h))
}

/// Monitor bounds for the overlay window (used to confine the cursor while drawing).
pub fn get_overlay_monitor_rect(app: &AppHandle) -> Option<(i32, i32, u32, u32)> {
    let window = app.get_webview_window("overlay")?;
    let pos = window.outer_position().ok()?;
    let size = window.outer_size().ok()?;
    let cx = pos.x + (size.width as i32 / 2);
    let cy = pos.y + (size.height as i32 / 2);
    get_monitor_rect_at_point(cx, cy)
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MonitorLogicalBounds {
    pub left: f64,
    pub top: f64,
    pub width: f64,
    pub height: f64,
}

/// Overlay monitor bounds in logical coordinates for toolbar window positioning.
pub fn get_overlay_monitor_logical_bounds(app: &AppHandle) -> Option<MonitorLogicalBounds> {
    let window = app.get_webview_window("overlay")?;
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
