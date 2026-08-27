use std::fs;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_opener::OpenerExt;
use time::macros::format_description;
use time::OffsetDateTime;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::{lock_or_recover, AppState};
use crate::error::{AppError, AppResult};
use crate::overlay;

const LOG_TAIL_BYTES: u64 = 96 * 1024;
const MAX_DIAGNOSTIC_EVENTS: usize = 500;
const GITHUB_ISSUE_REPO: &str = "https://github.com/ifer47/marker/issues/new";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticEvent {
    pub ts: u64,
    pub level: String,
    pub category: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticBundle {
    pub exported_at: String,
    pub app_version: String,
    pub app_identifier: String,
    pub platform: String,
    pub arch: String,
    pub os_version: Option<String>,
    pub locale: Option<String>,
    pub overlay_mode: String,
    pub whiteboard_mode: bool,
    pub config: Value,
    pub description: Option<String>,
    pub frontend_events: Vec<DiagnosticEvent>,
    pub log_tail: String,
}

#[allow(dead_code)]
pub struct LogGuard(pub tracing_appender::non_blocking::WorkerGuard);

pub fn log_dir(app: &AppHandle) -> AppResult<PathBuf> {
    let logs = if let Some(data) = crate::portable::data_dir() {
        data.join("logs")
    } else {
        let dir = app
            .path()
            .app_log_dir()
            .map_err(|e| AppError::Other(e.to_string()))?;
        dir.join("logs")
    };
    fs::create_dir_all(&logs).map_err(|e| AppError::Other(e.to_string()))?;
    Ok(logs)
}

pub fn init_tracing(app: &AppHandle) -> AppResult<LogGuard> {
    let logs = log_dir(app)?;
    let file_appender = tracing_appender::rolling::RollingFileAppender::new(
        tracing_appender::rolling::Rotation::DAILY,
        &logs,
        "marker.log",
    );
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "marker=info".parse().unwrap());

    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer())
        .with(
            tracing_subscriber::fmt::layer()
                .with_ansi(false)
                .with_writer(non_blocking),
        )
        .init();

    info!("Diagnostic log directory: {}", logs.display());
    Ok(LogGuard(guard))
}

pub fn append_event(state: &AppState, event: DiagnosticEvent) {
    let mut events = lock_or_recover(&state.diagnostic_events);
    events.push(event);
    if events.len() > MAX_DIAGNOSTIC_EVENTS {
        let drain = events.len() - MAX_DIAGNOSTIC_EVENTS;
        events.drain(0..drain);
    }
}

pub fn log_backend_event(
    state: &AppState,
    category: &str,
    message: &str,
    detail: Option<Value>,
    level: &str,
) {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);
    append_event(
        state,
        DiagnosticEvent {
            ts,
            level: level.to_string(),
            category: category.to_string(),
            message: message.to_string(),
            detail,
        },
    );
}

pub fn snapshot_events(state: &AppState) -> Vec<DiagnosticEvent> {
    lock_or_recover(&state.diagnostic_events).clone()
}

fn read_log_tail(dir: &Path) -> String {
    let mut candidates: Vec<PathBuf> = fs::read_dir(dir)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
        .collect();
    candidates.sort();

    let Some(path) = candidates.last() else {
        return String::new();
    };

    read_tail(path, LOG_TAIL_BYTES).unwrap_or_default()
}

fn read_tail(path: &Path, max_bytes: u64) -> std::io::Result<String> {
    let mut file = fs::File::open(path)?;
    let len = file.metadata()?.len();
    let start = len.saturating_sub(max_bytes);
    file.seek(SeekFrom::Start(start))?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;
    let mut text = String::from_utf8_lossy(&buf).into_owned();
    if start > 0 {
        if let Some(idx) = text.find('\n') {
            text = text[idx + 1..].to_string();
        }
    }
    Ok(text)
}

#[cfg(target_os = "windows")]
fn os_version() -> Option<String> {
    std::env::var("OS").ok()
}

#[cfg(target_os = "macos")]
fn os_version() -> Option<String> {
    std::process::Command::new("sw_vers")
        .arg("-productVersion")
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|s| s.trim().to_string())
}

pub fn build_bundle(
    app: &AppHandle,
    state: &AppState,
    description: Option<String>,
) -> AppResult<DiagnosticBundle> {
    let config = lock_or_recover(&state.config).clone();
    let overlay_mode = overlay::current_mode(state);
    let whiteboard_mode = *lock_or_recover(&state.whiteboard_mode);
    let frontend_events = snapshot_events(state);

    let config_value = serde_json::to_value(&config).map_err(|e| AppError::Other(e.to_string()))?;
    let logs = log_dir(app).unwrap_or_else(|_| PathBuf::from("."));

    Ok(DiagnosticBundle {
        exported_at: exported_at_iso(),
        app_version: app.package_info().version.to_string(),
        app_identifier: app.config().identifier.clone(),
        platform: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        os_version: os_version(),
        locale: config.general.locale.clone(),
        overlay_mode: overlay_mode.as_str().to_string(),
        whiteboard_mode,
        config: config_value,
        description,
        frontend_events,
        log_tail: read_log_tail(&logs),
    })
}

fn exported_at_iso() -> String {
    let format = format_description!("[year]-[month]-[day]T[hour]:[minute]:[second]Z");
    OffsetDateTime::now_utc()
        .format(format)
        .unwrap_or_else(|_| "unknown".to_string())
}

pub fn write_bundle(path: &Path, bundle: &DiagnosticBundle) -> AppResult<()> {
    let json = serde_json::to_string_pretty(bundle).map_err(|e| AppError::Other(e.to_string()))?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| AppError::Other(e.to_string()))?;
    }
    fs::write(path, json).map_err(|e| AppError::Other(e.to_string()))?;
    info!("Exported diagnostics to {}", path.display());
    Ok(())
}

fn percent_encode(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for byte in input.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char)
            }
            b' ' => out.push('+'),
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

pub fn github_issue_url(title: &str, body: &str) -> String {
    format!(
        "{GITHUB_ISSUE_REPO}?title={}&body={}",
        percent_encode(title),
        percent_encode(body)
    )
}

pub fn issue_body_from_bundle(bundle: &DiagnosticBundle) -> String {
    let mut lines = vec![
        "## 问题描述 / Description".to_string(),
        bundle
            .description
            .clone()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| "_请补充复现步骤 / Please add reproduction steps_".to_string()),
        String::new(),
        "## 环境 / Environment".to_string(),
        format!(
            "- Marker v{} ({})",
            bundle.app_version, bundle.app_identifier
        ),
        format!(
            "- OS: {} {} ({})",
            bundle.platform,
            bundle.os_version.as_deref().unwrap_or("unknown"),
            bundle.arch
        ),
        format!(
            "- Overlay: {}, whiteboard: {}",
            bundle.overlay_mode, bundle.whiteboard_mode
        ),
        String::new(),
        "## 诊断包 / Diagnostics".to_string(),
        "1. 在 Marker 中：**设置 → 应用诊断 → 导出日志**，保存 JSON 文件。".to_string(),
        "2. 在本 Issue 编辑框中 **拖拽该 JSON 文件**（或粘贴）作为附件，再点击 Submit。".to_string(),
        String::new(),
        "1. In Marker: **Settings → Diagnostics → Export logs**, save the JSON file.".to_string(),
        "2. **Drag the JSON file** into this issue editor (or paste) as an attachment, then click Submit.".to_string(),
    ];
    if !bundle.frontend_events.is_empty() {
        lines.push(String::new());
        lines.push(format!(
            "_Recent events captured: {} (full details in the attached JSON)._",
            bundle.frontend_events.len()
        ));
    }
    lines.join("\n")
}

#[tauri::command]
pub fn append_diagnostic_event(state: State<'_, AppState>, event: DiagnosticEvent) {
    append_event(&state, event);
}

#[tauri::command]
pub async fn export_diagnostics(
    app: AppHandle,
    state: State<'_, AppState>,
    description: Option<String>,
) -> AppResult<Option<String>> {
    let bundle = build_bundle(&app, &state, description)?;
    let default_name = format!(
        "marker-diagnostics-{}.json",
        bundle.exported_at.replace(':', "-")
    );
    let Some(path) = app
        .dialog()
        .file()
        .set_title("Marker Diagnostics")
        .set_file_name(&default_name)
        .add_filter("JSON", &["json"])
        .blocking_save_file()
    else {
        return Ok(None);
    };

    let file_path = path
        .into_path()
        .map_err(|e| AppError::Other(e.to_string()))?;
    write_bundle(&file_path, &bundle)?;
    Ok(Some(file_path.to_string_lossy().into_owned()))
}

#[tauri::command]
pub async fn open_github_issue_report(
    app: AppHandle,
    state: State<'_, AppState>,
    title: String,
    description: Option<String>,
) -> AppResult<()> {
    let bundle = build_bundle(&app, &state, description)?;
    let body = issue_body_from_bundle(&bundle);
    let url = github_issue_url(&title, &body);
    if !url.starts_with("https://github.com/ifer47/marker/") {
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

    #[test]
    fn percent_encode_handles_spaces() {
        assert_eq!(percent_encode("hello world"), "hello+world");
    }

    #[test]
    fn github_issue_url_uses_repo() {
        let url = github_issue_url("bug", "steps");
        assert!(url.starts_with(GITHUB_ISSUE_REPO));
        assert!(url.contains("title=bug"));
        assert!(url.contains("body=steps"));
    }
}
