use std::ffi::c_void;

use dispatch2::DispatchQueue;
use tauri::window::Color;
use tauri::{ActivationPolicy, AppHandle, Manager, Theme, TitleBarStyle, WebviewWindow};

use crate::theme::ResolvedTheme;

const SETTINGS_BG_DARK: Color = Color(30, 30, 32, 255); // #1e1e20
const SETTINGS_BG_LIGHT: Color = Color(245, 245, 247, 255); // #f5f5f7

/// AppKit `NSFloatingWindowLevel` — Tauri `always_on_top` uses this; every
/// overlay window sits here.
const NS_FLOATING_WINDOW_LEVEL: isize = 3;

/// Toolbar level: one step above all overlay windows. Multi-display keeps ONE
/// toolbar window, so a fixed level above the (per-screen, non-overlapping)
/// overlays replaces the old child-window stacking — no re-ordering needed
/// after canvas clicks.
const NS_TOOLBAR_WINDOW_LEVEL: isize = NS_FLOATING_WINDOW_LEVEL + 1;

/// `NSWindowSharingNone` — omit this window from screen capture / screencapture
/// (legacy window-list APIs; pairs with sync `orderOut` for in-app copy).
const NS_WINDOW_SHARING_NONE: isize = 0;

type Sel = *const c_void;

extern "C" {
    fn sel_registerName(name: *const std::ffi::c_char) -> Sel;
    fn objc_msgSend();
    fn pthread_main_np() -> i32;
}

unsafe fn msg_send_void(receiver: *mut c_void, sel: Sel) {
    let f: unsafe extern "C" fn(*mut c_void, Sel) =
        std::mem::transmute(objc_msgSend as *const c_void);
    f(receiver, sel);
}

unsafe fn msg_send_void_id(receiver: *mut c_void, sel: Sel, arg: *mut c_void) {
    let f: unsafe extern "C" fn(*mut c_void, Sel, *mut c_void) =
        std::mem::transmute(objc_msgSend as *const c_void);
    f(receiver, sel, arg);
}

unsafe fn msg_send_iset(receiver: *mut c_void, sel: Sel, value: isize) {
    let f: unsafe extern "C" fn(*mut c_void, Sel, isize) =
        std::mem::transmute(objc_msgSend as *const c_void);
    f(receiver, sel, value);
}

unsafe fn msg_send_bool_arg(receiver: *mut c_void, sel: Sel, value: bool) {
    // ObjC BOOL is a signed char on Apple platforms.
    let f: unsafe extern "C" fn(*mut c_void, Sel, i8) =
        std::mem::transmute(objc_msgSend as *const c_void);
    f(receiver, sel, if value { 1 } else { 0 });
}

fn is_main_thread() -> bool {
    unsafe { pthread_main_np() != 0 }
}

fn run_on_appkit_main(f: impl FnOnce() + Send + 'static) {
    if is_main_thread() {
        f();
    } else {
        // Same serial queue as tao's set_level_async / set_ignore_mouse_events.
        // exec_sync runs after any blocks already queued on that queue.
        DispatchQueue::main().exec_sync(f);
    }
}

fn exclude_toolbar_from_capture_now(toolbar_ns: *mut c_void) {
    if toolbar_ns.is_null() {
        return;
    }
    unsafe {
        let sel = sel_registerName(c"setSharingType:".as_ptr());
        msg_send_iset(toolbar_ns, sel, NS_WINDOW_SHARING_NONE);
    }
}

fn hide_toolbar_for_capture_now(toolbar: &WebviewWindow) {
    let Ok(toolbar_ns) = toolbar.ns_window() else {
        let _ = toolbar.hide();
        return;
    };
    if toolbar_ns.is_null() {
        let _ = toolbar.hide();
        return;
    }
    unsafe {
        exclude_toolbar_from_capture_now(toolbar_ns);
        // `orderOut:` must run on the AppKit queue before `screencapture`; Tauri's
        // `hide()` is async via the event loop and often still visible mid-capture.
        let order_out = sel_registerName(c"orderOut:".as_ptr());
        msg_send_void_id(toolbar_ns, order_out, std::ptr::null_mut());
    }
}

/// Synchronously `orderOut` the toolbar so in-app `screencapture` cannot
/// include it. Also pins `NSWindowSharingNone` (Windows `WDA_EXCLUDEFROMCAPTURE` analog).
pub fn hide_toolbar_ns_window_for_capture(toolbar: &WebviewWindow) {
    let toolbar = toolbar.clone();
    run_on_appkit_main(move || {
        hide_toolbar_for_capture_now(&toolbar);
    });
}

/// Mark the toolbar so capture APIs that respect sharing type omit it.
pub fn exclude_toolbar_ns_window_from_capture(toolbar: &WebviewWindow) {
    let toolbar = toolbar.clone();
    run_on_appkit_main(move || {
        if let Ok(ns) = toolbar.ns_window() {
            exclude_toolbar_from_capture_now(ns);
        }
    });
}

fn set_accepts_mouse_moved_now(ns_window: *mut c_void, accepts: bool) {
    if ns_window.is_null() {
        return;
    }
    unsafe {
        let sel = sel_registerName(c"setAcceptsMouseMovedEvents:".as_ptr());
        msg_send_bool_arg(ns_window, sel, accepts);
    }
}

fn make_key_window_now(ns_window: *mut c_void) {
    if ns_window.is_null() {
        return;
    }
    unsafe {
        // Key only — do not orderFront (would fight child stacking / z-order).
        let sel = sel_registerName(c"makeKeyWindow".as_ptr());
        msg_send_void(ns_window, sel);
    }
}

/// Toolbar is an AppKit child above the overlay: enable mouse-moved and make it
/// key so hover/buttons work without a prior activating click.
pub fn activate_toolbar_for_pointer_interaction(toolbar: &WebviewWindow) {
    let toolbar = toolbar.clone();
    run_on_appkit_main(move || {
        let Ok(ns) = toolbar.ns_window() else {
            return;
        };
        set_accepts_mouse_moved_now(ns, true);
        make_key_window_now(ns);
    });
}

/// Pointer left the panel — give the overlay key status again for drawing.
pub fn activate_overlay_for_drawing(overlay: &WebviewWindow) {
    let overlay = overlay.clone();
    run_on_appkit_main(move || {
        let Ok(ns) = overlay.ns_window() else {
            return;
        };
        set_accepts_mouse_moved_now(ns, true);
        make_key_window_now(ns);
    });
}

/// Pin the toolbar **NSWindow** one level above all overlay windows so it stays
/// above ink on every display without an AppKit child relationship.
///
/// Do **not** call Tauri `set_always_on_top` on the toolbar: it exec-asyncs
/// `setLevel(NSFloatingWindowLevel)` and would race our higher level back down.
///
/// Runs on the AppKit/GCD main queue (same as tao window ops). A follow-up
/// `exec_async` re-asserts after any blocks that were already queued when called
/// from the main thread (without blocking — that would deadlock).
///
/// Only NSWindow selectors are used — never WKWebView / WryWebView (those crash).
pub fn set_toolbar_level_above_overlays(toolbar: &WebviewWindow) {
    let toolbar_sync = toolbar.clone();
    run_on_appkit_main(move || ensure_toolbar_level_now(&toolbar_sync));

    let toolbar_async = toolbar.clone();
    DispatchQueue::main().exec_async(move || ensure_toolbar_level_now(&toolbar_async));
}

fn ensure_toolbar_level_now(toolbar: &WebviewWindow) {
    let Ok(toolbar_ns) = toolbar.ns_window() else {
        return;
    };
    if toolbar_ns.is_null() {
        return;
    }
    unsafe {
        let set_level_sel = sel_registerName(c"setLevel:".as_ptr());
        msg_send_iset(toolbar_ns, set_level_sel, NS_TOOLBAR_WINDOW_LEVEL);
        // Keep capture-exclusion (tao may recreate window state).
        exclude_toolbar_from_capture_now(toolbar_ns);
        set_accepts_mouse_moved_now(toolbar_ns, true);
    }
}

/// Tray apps run as Accessory; a Regular policy is required to surface the settings window.
pub fn activate_for_settings(app: &AppHandle) {
    app.set_activation_policy(ActivationPolicy::Regular).ok();
}

pub fn restore_accessory_policy(app: &AppHandle) {
    app.set_activation_policy(ActivationPolicy::Accessory).ok();
}

pub fn style_settings_builder(
    builder: tauri::WebviewWindowBuilder<'_, tauri::Wry, AppHandle>,
) -> tauri::WebviewWindowBuilder<'_, tauri::Wry, AppHandle> {
    builder
        .title_bar_style(TitleBarStyle::Transparent)
        .theme(Some(Theme::Dark))
        .background_color(SETTINGS_BG_DARK)
}

/// Reads `AppleInterfaceStyle` via CFPreferences (`"Dark"` → dark).
pub fn system_appearance_is_dark() -> bool {
    unsafe {
        extern "C" {
            fn CFPreferencesCopyAppValue(key: *const c_void, app_id: *const c_void) -> *mut c_void;
            fn CFRelease(cf: *const c_void);
            fn CFStringCreateWithCString(
                alloc: *const c_void,
                c_str: *const std::ffi::c_char,
                encoding: u32,
            ) -> *mut c_void;
            fn CFStringCompare(
                the_string1: *const c_void,
                the_string2: *const c_void,
                compare_options: u64,
            ) -> i32;
        }
        const K_CF_STRING_ENCODING_UTF8: u32 = 0x0800_0100;
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
            return false; // missing → light
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

pub fn configure_settings_window(window: &WebviewWindow, resolved: ResolvedTheme) {
    let (theme, bg) = match resolved {
        ResolvedTheme::Dark => (Theme::Dark, SETTINGS_BG_DARK),
        ResolvedTheme::Light => (Theme::Light, SETTINGS_BG_LIGHT),
    };
    window.set_theme(Some(theme)).ok();
    window.set_background_color(Some(bg)).ok();
}

pub fn configure_overlay_window(app: &AppHandle) {
    // Use Tauri's API only. Wry already disables WKWebView's white background for
    // transparent windows; calling Objective-C selectors on WryWebView will crash.
    for label in crate::overlay_windows::overlay_labels(app) {
        if let Some(window) = app.get_webview_window(&label) {
            window.set_background_color(Some(Color(0, 0, 0, 0))).ok();
        }
    }
}

pub fn configure_toolbar_window(window: &WebviewWindow) {
    window.set_background_color(Some(Color(0, 0, 0, 0))).ok();
    // Same intent as Windows WDA_EXCLUDEFROMCAPTURE: omit panel from screenshots.
    exclude_toolbar_ns_window_from_capture(window);
    // Hover styles / tooltips without requiring an activating click first.
    let window = window.clone();
    run_on_appkit_main(move || {
        if let Ok(ns) = window.ns_window() {
            set_accepts_mouse_moved_now(ns, true);
        }
    });
}
