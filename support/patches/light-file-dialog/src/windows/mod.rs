//! Windows 平台入口：分发到 GUI（Win32 API）、控制台（`dialog.exe`）或 basic-input。
//!
//! 对应 C 版 `tinyfiledialogs.c` 第 2876-3410 行的公共入口分发逻辑。

mod basic_input;
mod console;
mod gui;
mod utf;

use crate::helpers;
use crate::state;
use crate::types::{IconType, MessageBoxType, RgbColor};

/// 是否有图形显示（检查 DISPLAY / WAYLAND_DISPLAY 环境变量）。对应 C 版 `getenvDISPLAY`。
fn has_display() -> bool {
    std::env::var("DISPLAY").is_ok() || std::env::var("WAYLAND_DISPLAY").is_ok()
}

/// 是否在 SSH 会话中。
fn in_ssh() -> bool {
    std::env::var("SSH_CLIENT").is_ok()
}

/// 判断是否应使用图形模式。对应 C 版入口处的分发条件。
fn use_gui() -> bool {
    let force = state::force_console();
    let has_console = basic_input::in_console() || console::dialog_present();
    (force == 0 || !has_console) && (!in_ssh() || has_display())
}

/// 引号检测：若含引号，返回替换后的"INVALID"字符串。对应 C 版入口的 quote 检查。
fn safe_string(s: &str, label: &str) -> String {
    if helpers::quote_detected(Some(s)) {
        format!("INVALID {} WITH QUOTES OR BACKSPLASH", label)
    } else {
        s.to_string()
    }
}

pub fn beep() {
    gui::beep();
}

pub fn notify_popup(title: &str, message: &str, icon: IconType) -> i32 {
    let title = safe_string(title, "TITLE");
    let message = safe_string(message, "MESSAGE");
    if use_gui() {
        gui::notify_popup(&title, &message, icon)
    } else {
        // 控制台/basic-input：退化为消息框。
        message_box(&title, &message, MessageBoxType::Ok, icon, 0);
        0
    }
}

pub fn message_box(
    title: &str,
    message: &str,
    dialog_type: MessageBoxType,
    icon: IconType,
    default_button: i32,
) -> i32 {
    let title = safe_string(title, "TITLE");
    let message = safe_string(message, "MESSAGE");
    if use_gui() {
        gui::message_box(&title, &message, dialog_type, icon, default_button)
    } else if console::dialog_present() {
        console::message_box(&title, &message, dialog_type, icon, default_button)
    } else {
        basic_input::message_box(&title, &message, dialog_type, icon)
    }
}

pub fn input_box(title: &str, message: &str, default_input: Option<&str>) -> Option<String> {
    let title = safe_string(title, "TITLE");
    let message = safe_string(message, "MESSAGE");
    if use_gui() {
        gui::input_box(&title, &message, default_input)
    } else if console::dialog_present() {
        console::input_box(&title, &message, default_input)
    } else {
        basic_input::input_box(&title, &message, default_input)
    }
}

pub fn save_file_dialog(
    title: &str,
    default_path: &str,
    filter_patterns: &[&str],
    filter_description: &str,
) -> Option<String> {
    let title = safe_string(title, "TITLE");
    if use_gui() {
        gui::save_file_dialog(&title, default_path, filter_patterns, filter_description)
    } else if console::dialog_present() {
        console::save_file_dialog(&title, default_path)
    } else {
        basic_input::save_file_dialog(&title)
    }
}

pub fn open_file_dialog(
    title: &str,
    default_path: &str,
    filter_patterns: &[&str],
    filter_description: &str,
    allow_multiple: bool,
) -> Option<String> {
    let title = safe_string(title, "TITLE");
    // 多选在控制台/basic-input 模式下不支持。
    if use_gui() {
        let result = gui::open_file_dialog(
            &title,
            default_path,
            filter_patterns,
            filter_description,
            allow_multiple,
        )?;
        if allow_multiple {
            // 过滤不存在的文件。对应 C 版 ensureFilesExist。
            let filtered = helpers::ensure_files_exist(&result);
            if filtered.is_empty() {
                return None;
            }
            Some(filtered)
        } else {
            if !helpers::file_exists(&result) {
                return None;
            }
            Some(result)
        }
    } else if console::dialog_present() {
        console::open_file_dialog(&title, default_path)
    } else {
        basic_input::open_file_dialog(&title)
    }
}

pub fn select_folder_dialog(title: &str, default_path: &str) -> Option<String> {
    let title = safe_string(title, "TITLE");
    if use_gui() {
        let result = gui::select_folder_dialog(&title, default_path)?;
        if !helpers::dir_exists(&result) {
            return None;
        }
        Some(result)
    } else if console::dialog_present() {
        console::select_folder_dialog(&title, default_path)
    } else {
        basic_input::select_folder_dialog(&title)
    }
}

pub fn color_chooser(
    title: &str,
    default_hex: Option<&str>,
    default_rgb: Option<RgbColor>,
) -> Option<(String, RgbColor)> {
    let title = safe_string(title, "TITLE");
    if use_gui() {
        gui::color_chooser(&title, default_hex, default_rgb)
    } else if console::dialog_present() {
        // dialog.exe 无颜色选择器，走 basic-input。
        basic_input::color_chooser(&title, default_hex, default_rgb)
    } else {
        basic_input::color_chooser(&title, default_hex, default_rgb)
    }
}

// ============ 查询模式 ============

pub fn query_notify_popup(_icon: IconType) -> i32 {
    if use_gui() {
        state::set_response("windows_wchar");
        1
    } else if console::dialog_present() {
        state::set_response("dialog");
        0
    } else {
        state::set_response("basicinput");
        0
    }
}

pub fn query_message_box(_icon: IconType) -> i32 {
    query_notify_popup(_icon)
}

pub fn query_input_box() -> i32 {
    query_notify_popup(IconType::Info)
}

pub fn query_save_file_dialog() -> i32 {
    query_notify_popup(IconType::Info)
}

pub fn query_open_file_dialog() -> i32 {
    query_notify_popup(IconType::Info)
}

pub fn query_select_folder_dialog() -> i32 {
    query_notify_popup(IconType::Info)
}

pub fn query_color_chooser() -> i32 {
    query_notify_popup(IconType::Info)
}
