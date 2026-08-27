//! Confine the OS cursor to a single monitor while drawing on macOS.
//!
//! macOS has no public `ClipCursor` API. We poll cursor position and warp it back
//! with `CGWarpMouseCursorPosition` when it leaves the overlay monitor bounds.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::thread::{self, JoinHandle};
use std::time::Duration;

type MonitorBounds = (i32, i32, u32, u32);

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
struct CGPoint {
    x: f64,
    y: f64,
}

extern "C" {
    fn CGEventCreate(source: *const std::ffi::c_void) -> *mut std::ffi::c_void;
    fn CGEventGetLocation(event: *const std::ffi::c_void) -> CGPoint;
    fn CFRelease(cf: *const std::ffi::c_void);
    fn CGWarpMouseCursorPosition(point: CGPoint);
}

static CLIP_ACTIVE: AtomicBool = AtomicBool::new(false);
static CLIP_BOUNDS: Mutex<Option<MonitorBounds>> = Mutex::new(None);
static CLIP_THREAD: Mutex<Option<JoinHandle<()>>> = Mutex::new(None);

fn cursor_location() -> Option<CGPoint> {
    unsafe {
        let event = CGEventCreate(std::ptr::null());
        if event.is_null() {
            return None;
        }
        let pt = CGEventGetLocation(event);
        CFRelease(event);
        Some(pt)
    }
}

/// Clamp a point into monitor bounds (global display coordinates, points).
fn clamp_to_bounds(pt: CGPoint, (x, y, w, h): MonitorBounds) -> CGPoint {
    let left = x as f64;
    let top = y as f64;
    // Match Win32 `RECT` semantics: right/bottom edges are exclusive.
    let right = left + w as f64;
    let bottom = top + h as f64;
    let max_x = (right - 1.0).max(left);
    let max_y = (bottom - 1.0).max(top);
    CGPoint {
        x: pt.x.clamp(left, max_x),
        y: pt.y.clamp(top, max_y),
    }
}

fn needs_warp(pt: CGPoint, clamped: CGPoint) -> bool {
    (pt.x - clamped.x).abs() > 0.5 || (pt.y - clamped.y).abs() > 0.5
}

fn enforcement_loop() {
    while CLIP_ACTIVE.load(Ordering::Acquire) {
        let bounds = CLIP_BOUNDS.lock().ok().and_then(|guard| *guard);
        if let Some(bounds) = bounds {
            if let Some(pt) = cursor_location() {
                let clamped = clamp_to_bounds(pt, bounds);
                if needs_warp(pt, clamped) {
                    unsafe {
                        CGWarpMouseCursorPosition(clamped);
                    }
                }
            }
        }
        thread::sleep(Duration::from_millis(8));
    }
}

fn join_clip_thread() {
    if let Ok(mut thread_guard) = CLIP_THREAD.lock() {
        if let Some(handle) = thread_guard.take() {
            let _ = handle.join();
        }
    }
}

/// Start confining the cursor to `bounds` (x, y, width, height in screen points).
pub fn start_cursor_clip(bounds: MonitorBounds) {
    stop_cursor_clip();
    if let Ok(mut stored) = CLIP_BOUNDS.lock() {
        *stored = Some(bounds);
    }
    CLIP_ACTIVE.store(true, Ordering::Release);
    let handle = thread::spawn(enforcement_loop);
    if let Ok(mut thread_guard) = CLIP_THREAD.lock() {
        *thread_guard = Some(handle);
    }
}

/// Pause confinement without clearing stored bounds (penetration mode).
pub fn stop_cursor_clip() {
    CLIP_ACTIVE.store(false, Ordering::Release);
    join_clip_thread();
}

/// Stop confinement and drop cached bounds (drawing session ended).
pub fn release_cursor_clip() {
    stop_cursor_clip();
    if let Ok(mut stored) = CLIP_BOUNDS.lock() {
        *stored = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamp_keeps_point_inside_monitor() {
        let bounds = (100, 200, 800, 600);
        let pt = CGPoint { x: 500.0, y: 400.0 };
        let clamped = clamp_to_bounds(pt, bounds);
        assert_eq!(clamped, pt);
    }

    #[test]
    fn clamp_pulls_point_from_right_edge() {
        let bounds = (0, 0, 1920, 1080);
        let pt = CGPoint {
            x: 2000.0,
            y: 540.0,
        };
        let clamped = clamp_to_bounds(pt, bounds);
        assert_eq!(clamped.x, 1919.0);
        assert_eq!(clamped.y, 540.0);
    }

    #[test]
    fn clamp_pulls_point_from_top_left() {
        let bounds = (100, 50, 800, 600);
        let pt = CGPoint { x: 0.0, y: 0.0 };
        let clamped = clamp_to_bounds(pt, bounds);
        assert_eq!(clamped.x, 100.0);
        assert_eq!(clamped.y, 50.0);
    }
}
