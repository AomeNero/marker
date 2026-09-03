//! `.marker` annotation files — thin Rust shim under the frontend's router.
//!
//! The frontend owns the format (serialization, validation, screen matching —
//! see `src/utils/annotationFile.ts`); the backend only touches the disk, the
//! file dialog, the screen registry, and the global undo timeline.

use std::fs;
use std::path::PathBuf;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_dialog::DialogExt;
use time::macros::format_description;
use tracing::warn;

use crate::config::AppState;

/// Tray menu → primary overlay: run a file action in the frontend orchestrator.
pub const FILE_REQUEST_EVENT: &str = "annotations-file-request";

const MARKER_EXT: &str = "marker";
const STAMP: &[time::format_description::BorrowedFormatItem<'static>] =
    format_description!("[year][month][day][hour][minute][second]");

// ---------------------------------------------------------------------------
// Save directory + file IO
// ---------------------------------------------------------------------------

/// Directory for saved annotation files. Portable builds: `data\annotations\`
/// beside the executable. Installed builds: `Documents\Marker\` — Program
/// Files is not writable, and a user-visible folder keeps the files findable
/// for sharing.
pub fn annotations_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = match crate::portable::data_dir() {
        Some(data) => data.join("annotations"),
        None => app
            .path()
            .document_dir()
            .map_err(|e| e.to_string())?
            .join("Marker"),
    };
    fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    Ok(dir)
}

/// `marker20260903153000` — local time, UTC fallback when the offset is
/// unavailable. Mirrors the frontend's `annotationFileName`.
fn annotation_stamp() -> String {
    let now = time::OffsetDateTime::now_local().unwrap_or_else(|_| time::OffsetDateTime::now_utc());
    now.format(&STAMP).unwrap_or_else(|_| "0".repeat(14))
}

/// Selected file: on-disk path plus raw JSON content (validated by the frontend).
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnnotationFilePayload {
    pub path: String,
    pub content: String,
}

fn read_marker_file(path: PathBuf) -> Result<AnnotationFilePayload, String> {
    let content = fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(AnnotationFilePayload {
        path: path.display().to_string(),
        content,
    })
}

/// File dialog → raw `.marker` content. `Ok(None)` = user cancelled.
#[tauri::command]
pub fn pick_annotations_file(app: AppHandle) -> Result<Option<AnnotationFilePayload>, String> {
    let s = crate::i18n::strings();
    let picked = app
        .dialog()
        .file()
        .set_title(s.open_annotations)
        .add_filter(s.annotations_file, &[MARKER_EXT])
        .blocking_pick_file();
    let Some(picked) = picked else {
        return Ok(None);
    };
    let path = picked.into_path().map_err(|e| e.to_string())?;
    read_marker_file(path).map(Some)
}

/// Read an explicit `.marker` path (file-association double-click).
#[tauri::command]
pub fn read_annotations_file(path: String) -> Result<AnnotationFilePayload, String> {
    read_marker_file(PathBuf::from(path))
}

/// Validate + write a serialized `.marker` payload into the save directory
/// under an auto-generated timestamp name. Returns the full written path.
#[tauri::command]
pub fn save_annotations_file(app: AppHandle, content: String) -> Result<String, String> {
    if serde_json::from_str::<serde_json::Value>(&content).is_err() {
        return Err("invalid annotation payload".into());
    }
    let dir = annotations_dir(&app)?;
    let path = dir.join(format!("marker{}.{MARKER_EXT}", annotation_stamp()));
    fs::write(&path, content).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(path.display().to_string())
}

// ---------------------------------------------------------------------------
// Screen specs for the frontend load router
// ---------------------------------------------------------------------------

/// Live (or last-known) overlay label ↔ screen pairing, mirrored from
/// `MonitorSpec` for `src/utils/annotationFile.ts`.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OverlayScreenSpec {
    pub label: String,
    pub primary: bool,
    pub name: Option<String>,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub scale: f64,
}

fn spec_from_label(label: &str, spec: &crate::overlay_windows::MonitorSpec) -> OverlayScreenSpec {
    OverlayScreenSpec {
        label: label.to_string(),
        primary: label == crate::overlay_windows::PRIMARY_LABEL,
        name: spec.name.clone(),
        x: spec.x,
        y: spec.y,
        width: spec.width,
        height: spec.height,
        scale: spec.scale_factor,
    }
}

/// Current overlay↔screen pairing. Prefers the session registry (kept fresh
/// by activation and the hotplug watcher); falls back to a fresh enumeration
/// paired the same way activation pairs them.
#[tauri::command]
pub fn get_overlay_screen_specs(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Vec<OverlayScreenSpec> {
    {
        let registry = crate::config::lock_or_recover(&state.monitors);
        if !registry.is_empty() {
            let mut specs: Vec<OverlayScreenSpec> = registry
                .iter()
                .map(|(label, entry)| spec_from_label(label, &entry.spec))
                .collect();
            specs.sort_by_cached_key(|s| crate::overlay_windows::label_sort_key(&s.label));
            return specs;
        }
    }
    let monitors = crate::overlay_windows::enumerate_monitors(&app);
    let Some(cursor) = crate::overlay_windows::cursor_monitor(&monitors) else {
        warn!("No cursor monitor available for screen specs");
        return Vec::new();
    };
    let specs: Vec<crate::overlay_windows::MonitorSpec> =
        monitors.iter().map(|(m, _)| m.clone()).collect();
    crate::overlay_windows::assign_labels(&specs, &cursor)
        .into_iter()
        .map(|(label, spec)| spec_from_label(&label, &spec))
        .collect()
}

// ---------------------------------------------------------------------------
// Undo timeline for loads
// ---------------------------------------------------------------------------

/// Record the global timeline op for a frontend-orchestrated load. `open`
/// folds existing ops so one Ctrl+Z restores the pre-open board; `insert`
/// stacks on top. Both invalidate every overlay's redo branch.
#[tauri::command]
pub fn record_load_op(
    app: AppHandle,
    state: State<'_, AppState>,
    mode: String,
) -> Result<(), String> {
    {
        let mut timeline = crate::config::lock_or_recover(&state.timeline);
        match mode.as_str() {
            "open" => {
                timeline.begin_global_load();
            }
            "insert" => {
                timeline.commit("", "insert");
            }
            other => return Err(format!("unknown load mode: {other}")),
        }
    }
    if let Err(e) = app.emit(crate::timeline::REDO_CLEARED_EVENT, ()) {
        warn!("Failed to emit timeline redo-cleared: {}", e);
    }
    crate::timeline::broadcast_state(&app, &state.timeline);
    Ok(())
}

// ---------------------------------------------------------------------------
// Tray entry point
// ---------------------------------------------------------------------------

/// Tray menu asked for a file action: make sure drawing mode is on (strokes
/// and their webviews must be live), then hand off to the primary overlay —
/// it hosts the orchestrator (dialog, routing, toasts). `path` bypasses the
/// dialog for file-association opens.
pub fn request_file_action(
    app: &AppHandle,
    state: &crate::AppState,
    mode: &str,
    path: Option<&str>,
) {
    if crate::overlay::current_mode(state) == crate::overlay::OverlayMode::Hidden {
        crate::overlay::activate_drawing(app, state);
    }
    let payload = serde_json::json!({ "mode": mode, "path": path });
    if let Err(e) = app.emit_to(
        crate::overlay_windows::PRIMARY_LABEL,
        FILE_REQUEST_EVENT,
        payload,
    ) {
        warn!("Failed to emit {FILE_REQUEST_EVENT}: {e}");
    }
}
