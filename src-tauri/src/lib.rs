#[cfg(not(any(target_os = "windows", target_os = "macos")))]
compile_error!("Marker only supports Windows and macOS.");

mod annotation_file;
mod clipboard;
mod commands;
mod config;
mod diagnostics;
mod error;
mod i18n;
#[cfg(target_os = "macos")]
mod macos;
mod monitor;
mod overlay;
mod overlay_windows;
mod portable;
mod shortcuts;
#[cfg(target_os = "windows")]
mod single_instance_win;
mod theme;
mod timeline;
#[cfg(target_os = "windows")]
mod win32;

use std::sync::Mutex;
use tauri::{
    menu::{MenuBuilder, MenuItemBuilder},
    tray::{TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder,
};
use tracing::{info, warn};

use config::{lock_or_recover, AppConfig, AppState};
use diagnostics::log_backend_event;
pub use overlay::{
    activate_drawing, clear_drawing, deactivate_drawing, raise_toolbar_above_overlay,
    set_toolbar_window_visible, setup_overlay_size, toggle_drawing,
};

pub fn rebuild_tray_menu(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let s = i18n::strings();
    if let Some(tray) = app.tray_by_id("main") {
        let settings_item = MenuItemBuilder::with_id("settings", s.settings).build(app)?;
        let open_item =
            MenuItemBuilder::with_id("open-annotations", s.open_annotations).build(app)?;
        let insert_item =
            MenuItemBuilder::with_id("insert-annotations", s.insert_annotations).build(app)?;
        let save_item =
            MenuItemBuilder::with_id("save-annotations", s.save_annotations).build(app)?;
        let help_item = MenuItemBuilder::with_id("help", s.help).build(app)?;
        let about_item = MenuItemBuilder::with_id("about", s.about).build(app)?;
        let quit_item = MenuItemBuilder::with_id("quit", s.quit).build(app)?;
        let menu = MenuBuilder::new(app)
            .item(&settings_item)
            .separator()
            .item(&open_item)
            .item(&insert_item)
            .item(&save_item)
            .separator()
            .item(&help_item)
            .item(&about_item)
            .separator()
            .item(&quit_item)
            .build()?;
        tray.set_menu(Some(menu))?;
        tray.set_tooltip(Some(s.tray_tooltip))?;
    }
    Ok(())
}

/// Create the tray with the correct glyph up front (avoids a flash from a
/// conf-default icon that may not match the Windows taskbar / flyout).
fn install_main_tray(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let icon = theme::main_tray_icon()?;
    let s = i18n::strings();
    let builder = TrayIconBuilder::with_id("main")
        .icon(icon)
        .tooltip(s.tray_tooltip)
        .show_menu_on_left_click(false);
    #[cfg(target_os = "macos")]
    let builder = builder.icon_as_template(true);
    let _tray = builder.build(app)?;
    Ok(())
}

fn open_settings(app: &AppHandle) {
    open_settings_tab(app, None);
}

fn focus_settings_window(app: &AppHandle, tab: Option<&str>) {
    let Some(win) = app.get_webview_window("settings") else {
        return;
    };
    #[cfg(target_os = "macos")]
    macos::activate_for_settings(app);
    win.show().ok();
    win.set_focus().ok();
    // Windows denies foreground to background processes: the tray menu handler
    // needs the win32 z-order bounce to reliably surface an existing window.
    #[cfg(windows)]
    if let Ok(hwnd) = win.hwnd() {
        crate::win32::force_window_foreground(hwnd.0 as isize);
    }
    if let Some(t) = tab {
        app.emit_to("settings", "switch-tab", t).ok();
    }
}

/// Second launch while the app is already running (desktop icon / pinned taskbar /
/// opening the .app again on macOS). Match tray left-click: toggle annotation.
/// A `.marker` argument (file-association double-click) opens that file instead.
///
/// Previously this only focused the settings window, which no-ops when settings
/// was never opened — so relaunch appeared to do nothing. `toggle_drawing` is the
/// same path as the tray and global shortcut; safe on macOS Accessory policy.
fn on_second_instance(app: &AppHandle, args: Vec<String>) {
    if let Some(path) = args.iter().find(|a| a.to_lowercase().ends_with(".marker")) {
        let state = app.state::<AppState>();
        log_backend_event(
            &state,
            "session",
            "annotations file requested",
            Some(serde_json::json!({ "reason": "second-instance", "path": path })),
            "info",
        );
        annotation_file::request_file_action(app, &state, "open", Some(path));
        return;
    }
    let state = app.state::<AppState>();
    log_backend_event(
        &state,
        "session",
        "toggle drawing requested",
        Some(serde_json::json!({ "reason": "second-instance" })),
        "info",
    );
    toggle_drawing(app);
}

/// Called from the settings webview after it mounts (window starts hidden to avoid white flash).
pub fn reveal_settings_window(app: &AppHandle) {
    focus_settings_window(app, None);
}

fn open_settings_tab(app: &AppHandle, tab: Option<&str>) {
    if app.get_webview_window("settings").is_some() {
        focus_settings_window(app, tab);
        return;
    }

    let hash = match tab {
        Some(t) => format!("index.html#settings/{}", t),
        None => "index.html#settings".to_string(),
    };
    let url = WebviewUrl::App(hash.into());
    let builder = WebviewWindowBuilder::new(app, "settings", url)
        .title(i18n::strings().window_title)
        .inner_size(660.0, 500.0)
        .min_inner_size(540.0, 420.0)
        .resizable(true)
        .center()
        .visible(false);

    #[cfg(target_os = "macos")]
    let builder = macos::style_settings_builder(builder);

    match builder.build() {
        #[cfg(target_os = "macos")]
        Ok(window) => {
            macos::activate_for_settings(app);
            let preference = lock_or_recover(&app.state::<AppState>().config)
                .general
                .theme;
            macos::configure_settings_window(&window, theme::resolve_theme(&preference));
        }
        #[cfg(not(target_os = "macos"))]
        Ok(_) => {
            let preference = lock_or_recover(&app.state::<AppState>().config)
                .general
                .theme;
            let resolved = theme::resolve_theme(&preference);
            theme::apply_app_theme(app, &preference);
            // First open: sync title-bar + taskbar icons (subsequent appearance
            // toggles only swap the cached title-bar HICON).
            #[cfg(target_os = "windows")]
            theme::apply_windows_settings_window_icons(app, resolved);
        }
        Err(e) => warn!("Failed to open settings window: {}", e),
    }
}

pub fn run() {
    // Redirect WebView2 profile before any webview is created (portable builds).
    portable::apply_webview_user_data_dir();

    let builder = tauri::Builder::default();
    #[cfg(target_os = "windows")]
    let builder = builder.plugin(single_instance_win::init(|app, args, _cwd| {
        on_second_instance(app, args);
    }));
    #[cfg(not(target_os = "windows"))]
    let builder = builder.plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
        on_second_instance(app, args);
    }));
    builder
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec![]),
        ))
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .manage(AppState {
            config: Mutex::new(AppConfig::default()),
            overlay_mode: Mutex::new(overlay::OverlayMode::Hidden),
            whiteboard_mode: Mutex::new(false),
            diagnostic_events: Mutex::new(Vec::new()),
            monitors: Mutex::new(std::collections::HashMap::new()),
            timeline: Mutex::new(timeline::Timeline::new()),
            pending_file_open: Mutex::new(None),
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_config,
            commands::get_overlay_pointer_position,
            commands::get_overlay_monitor_logical_bounds,
            commands::get_overlay_monitor_work_logical_bounds,
            commands::is_pointer_over_toolbar_panel,
            commands::forward_toolbar_action,
            commands::timeline_commit_op,
            commands::timeline_undo,
            commands::timeline_redo,
            commands::timeline_reset,
            commands::get_timeline_state,
            commands::clear_all_drawings,
            commands::set_overlay_ignore_cursor_events,
            commands::save_shortcuts,
            commands::save_general,
            commands::save_line_widths,
            commands::save_locale,
            commands::apply_app_theme,
            commands::exit_drawing,
            commands::set_toolbar_visible,
            commands::set_toolbar_popup,
            commands::raise_toolbar,
            commands::set_whiteboard_mode,
            commands::open_url,
            commands::reveal_settings_window,
            commands::is_portable,
            commands::supports_autostart,
            diagnostics::export_diagnostics,
            diagnostics::open_github_issue_report,
            diagnostics::append_diagnostic_event,
            clipboard::copy_screen,
            clipboard::copy_whiteboard,
            annotation_file::get_overlay_screen_specs,
            annotation_file::pick_annotations_file,
            annotation_file::read_annotations_file,
            annotation_file::save_annotations_file,
            annotation_file::record_load_op,
        ])
        .setup(|app| {
            let handle = app.handle().clone();
            let log_guard = diagnostics::init_tracing(&handle)?;
            app.manage(log_guard);
            if portable::is_portable() {
                info!("Starting Marker (portable mode)");
            } else {
                info!("Starting Marker");
            }

            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            let loaded = config::load_config(&handle);
            i18n::init(loaded.general.locale.as_deref());
            {
                let state = handle.state::<AppState>();
                *lock_or_recover(&state.config) = loaded.clone();
                config::apply_autostart_preference(&handle, &state, loaded.general.auto_start);
            }

            // Tray before other chrome so Windows never shows a mismatched default glyph.
            install_main_tray(&handle)?;
            rebuild_tray_menu(&handle).ok();

            if let Some(tray) = app.tray_by_id("main") {
                tray.on_menu_event(move |app, event| {
                    // Any menu action dismisses the menu — restore the resident toolbar.
                    crate::overlay::show_toolbar_after_tray_menu(app);
                    match event.id().as_ref() {
                        "settings" => open_settings(app),
                        "open-annotations" | "insert-annotations" | "save-annotations" => {
                            let state = app.state::<AppState>();
                            let mode = event
                                .id()
                                .as_ref()
                                .strip_suffix("-annotations")
                                .unwrap_or("");
                            annotation_file::request_file_action(app, &state, mode, None);
                        }
                        "help" => open_settings_tab(app, Some("help")),
                        "about" => open_settings_tab(app, Some("about")),
                        "quit" => app.exit(0),
                        _ => {}
                    }
                });
                let handle_click = handle.clone();
                let handle_menu = handle.clone();
                tray.on_tray_icon_event(move |_tray, event| {
                    if let TrayIconEvent::Click {
                        button,
                        button_state: tauri::tray::MouseButtonState::Up,
                        ..
                    } = event
                    {
                        match button {
                            tauri::tray::MouseButton::Left => {
                                let state = handle_click.state::<AppState>();
                                log_backend_event(
                                    &state,
                                    "session",
                                    "toggle drawing requested",
                                    Some(serde_json::json!({ "reason": "tray" })),
                                    "info",
                                );
                                toggle_drawing(&handle_click);
                            }
                            tauri::tray::MouseButton::Right => {
                                // The context menu must not be covered by the topmost toolbar bar.
                                crate::overlay::hide_toolbar_for_tray_menu(&handle_menu);
                            }
                            _ => {}
                        }
                    }
                });
            }

            #[cfg(target_os = "windows")]
            theme::start_windows_tray_theme_watcher(&handle);

            theme::apply_app_theme(&handle, &loaded.general.theme);

            setup_overlay_size(&handle);

            #[cfg(target_os = "macos")]
            macos::configure_overlay_window(&handle);

            #[cfg(target_os = "windows")]
            {
                if let Some(window) = handle.get_webview_window("overlay") {
                    window
                        .set_background_color(Some(tauri::window::Color(0, 0, 0, 0)))
                        .ok();
                }
            }

            shortcuts::register_shortcuts(&handle);

            // Pre-create hidden overlay windows for extra monitors so the first
            // multi-screen activation never waits on webview startup.
            let precreate_handle = handle.clone();
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_millis(1500));
                let _ = precreate_handle.run_on_main_thread({
                    let inner = precreate_handle.clone();
                    move || {
                        let extra = overlay_windows::enumerate_monitors(&inner)
                            .len()
                            .saturating_sub(1);
                        if extra > 0 {
                            overlay_windows::ensure_extra_overlay_windows(&inner, extra);
                        }
                    }
                });
            });

            let ctrlc_handle = handle.clone();
            ctrlc::set_handler(move || {
                ctrlc_handle.exit(0);
            })
            .ok();

            // Cold-start `.marker` argument (file-association double-click while
            // not running): defer until overlay webviews have mounted, then open.
            if let Some(path) = std::env::args()
                .skip(1)
                .find(|a| a.to_lowercase().ends_with(".marker"))
            {
                *lock_or_recover(&handle.state::<AppState>().pending_file_open) = Some(path);
                let open_handle = handle.clone();
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_millis(800));
                    let state = open_handle.state::<AppState>();
                    let pending = lock_or_recover(&state.pending_file_open).take();
                    if let Some(path) = pending {
                        annotation_file::request_file_action(
                            &open_handle,
                            &state,
                            "open",
                            Some(&path),
                        );
                    }
                });
            }

            Ok(())
        })
        .on_window_event(|window, event| match event {
            tauri::WindowEvent::CloseRequested { api, .. }
                if window.label().starts_with("overlay") || window.label() == "toolbar" =>
            {
                api.prevent_close();
                window.hide().ok();
            }
            // WebView2 can lose the transparent clear color after long idle /
            // GPU recycle and repaint opaque black once it regains focus.
            tauri::WindowEvent::Focused(true) if window.label().starts_with("overlay") => {
                if let Some(w) = window.app_handle().get_webview_window(window.label()) {
                    overlay::reassert_window_transparency(&w);
                }
            }
            // DPI change / monitor move can rebuild the compositor and drop the
            // transparent backdrop; re-assert it as soon as the scale settles.
            tauri::WindowEvent::ScaleFactorChanged { .. }
                if window.label().starts_with("overlay") =>
            {
                if let Some(w) = window.app_handle().get_webview_window(window.label()) {
                    overlay::reassert_window_transparency(&w);
                }
            }
            tauri::WindowEvent::Destroyed if window.label() == "settings" => {
                #[cfg(target_os = "macos")]
                macos::restore_accessory_policy(window.app_handle());
            }
            _ => {}
        })
        .build(tauri::generate_context!())
        .expect("error while building Marker")
        .run(|app, event| {
            // macOS file association: Finder hands opened documents to the
            // running instance here (single-instance forwards cold launches).
            #[cfg(target_os = "macos")]
            if let tauri::RunEvent::Opened { urls } = event {
                if let Some(path) = urls.iter().find_map(|url| url.to_file_path().ok()) {
                    let state = app.state::<AppState>();
                    annotation_file::request_file_action(
                        app,
                        &state,
                        "open",
                        Some(&path.display().to_string()),
                    );
                }
            }
            #[cfg(not(target_os = "macos"))]
            {
                let _ = (app, event);
            }
        });
}
