use std::sync::atomic::{AtomicBool, Ordering};

pub struct Strings {
    pub settings: &'static str,
    pub help: &'static str,
    pub about: &'static str,
    pub quit: &'static str,
    pub window_title: &'static str,
    pub tray_tooltip: &'static str,
    pub toggle_drawing: &'static str,
    pub clear_drawing: &'static str,
    pub open_annotations: &'static str,
    pub insert_annotations: &'static str,
    pub save_annotations: &'static str,
    pub annotations_file: &'static str,
}

const ZH: Strings = Strings {
    settings: "设置",
    help: "使用帮助",
    about: "关于",
    quit: "退出",
    window_title: "Marker 设置",
    tray_tooltip: "Marker - 屏幕标注工具",
    toggle_drawing: "开始标注",
    clear_drawing: "清除标注",
    open_annotations: "打开标注文件",
    insert_annotations: "插入标注文件",
    save_annotations: "保存标注",
    annotations_file: "Marker 标注文件",
};

const EN: Strings = Strings {
    settings: "Settings",
    help: "Help",
    about: "About",
    quit: "Quit",
    window_title: "Marker Settings",
    tray_tooltip: "Marker - Screen annotation",
    toggle_drawing: "Toggle annotation",
    clear_drawing: "Clear annotations",
    open_annotations: "Open annotations",
    insert_annotations: "Insert annotations",
    save_annotations: "Save annotations",
    annotations_file: "Marker annotations",
};

static USE_CHINESE: AtomicBool = AtomicBool::new(false);

pub fn init(locale: Option<&str>) {
    let chinese = match locale {
        Some(l) => l.starts_with("zh"),
        None => detect_chinese(),
    };
    USE_CHINESE.store(chinese, Ordering::Relaxed);
}

pub fn set_locale(locale: &str) {
    USE_CHINESE.store(locale.starts_with("zh"), Ordering::Relaxed);
}

pub fn strings() -> &'static Strings {
    if USE_CHINESE.load(Ordering::Relaxed) {
        &ZH
    } else {
        &EN
    }
}

fn detect_chinese() -> bool {
    #[cfg(target_os = "windows")]
    {
        let lang = std::env::var("LANG")
            .or_else(|_| std::env::var("LANGUAGE"))
            .unwrap_or_default();
        if lang.starts_with("zh") {
            return true;
        }
        use std::os::raw::c_int;
        extern "system" {
            fn GetUserDefaultUILanguage() -> u16;
        }
        let lang_id = unsafe { GetUserDefaultUILanguage() } as c_int;
        let primary = lang_id & 0xFF;
        primary == 0x04
    }
    #[cfg(target_os = "macos")]
    {
        std::env::var("LANG").unwrap_or_default().starts_with("zh")
            || std::process::Command::new("defaults")
                .args(["read", "-g", "AppleLanguages"])
                .output()
                .map(|o| String::from_utf8_lossy(&o.stdout).contains("zh"))
                .unwrap_or(false)
    }
}
