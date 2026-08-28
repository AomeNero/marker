//! Windows “绿色版” / portable mode.
//!
//! When a marker file named `marker.portable` sits next to the executable,
//! config, logs, and WebView2 user data are stored under `{exe_dir}/data/` instead
//! of the system AppData directories.

#[cfg(target_os = "windows")]
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// Unique marker name to avoid accidental activation from a generic `portable` file.
const MARKER_NAME: &str = "marker.portable";

static PORTABLE_ROOT: OnceLock<Option<PathBuf>> = OnceLock::new();

/// Directory containing the executable when portable mode is active.
pub fn portable_root() -> Option<&'static Path> {
    PORTABLE_ROOT.get_or_init(detect_portable_root).as_deref()
}

pub fn is_portable() -> bool {
    portable_root().is_some()
}

/// `{exe_dir}/data` when portable; `None` otherwise.
pub fn data_dir() -> Option<PathBuf> {
    portable_root().map(|root| root.join("data"))
}

/// Call before any WebView is created so Edge/WebView2 does not write under AppData.
pub fn apply_webview_user_data_dir() {
    #[cfg(target_os = "windows")]
    if let Some(data) = data_dir() {
        let webview = data.join("webview");
        if fs::create_dir_all(&webview).is_ok() {
            // Must be set before WebView2 initializes.
            // SAFETY: no concurrent env mutation at this point in process startup.
            unsafe {
                std::env::set_var("WEBVIEW2_USER_DATA_FOLDER", &webview);
            }
        }
    }
}

fn detect_portable_root() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let dir = exe.parent()?.to_path_buf();
    detect_portable_root_in(&dir)
}

fn detect_portable_root_in(dir: &Path) -> Option<PathBuf> {
    if dir.join(MARKER_NAME).is_file() {
        Some(dir.to_path_buf())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn detects_portable_marker() {
        let dir = std::env::temp_dir().join(format!("marker-portable-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        assert!(detect_portable_root_in(&dir).is_none());

        fs::write(dir.join("portable"), b"").unwrap();
        assert!(
            detect_portable_root_in(&dir).is_none(),
            "generic 'portable' file must not activate portable mode"
        );

        fs::write(dir.join(MARKER_NAME), b"").unwrap();
        assert_eq!(
            detect_portable_root_in(&dir).as_deref(),
            Some(dir.as_path())
        );

        let _ = fs::remove_dir_all(&dir);
    }
}
