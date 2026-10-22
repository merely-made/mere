//! # light-file-dialog
//!
//! [`tinyfiledialogs`](https://sourceforge.net/projects/tinyfiledialogs/) 的 Rust 移植版。
//!
//! 提供跨平台的本地对话框：消息框、输入框、通知、文件选择、文件夹选择、颜色选择。
//! 支持 Windows（Win32 API）、macOS（AppleScript）和 Linux/Unix（zenity/kdialog/yad 等）。
//!
//! ## 示例
//!
//! ```no_run
//! use light_file_dialog::dialog::*;
//! use light_file_dialog::{IconType, MessageBoxType};
//!
//! // Builder API（推荐）
//! MessageBox::new("hello", "Welcome to light-file-dialog")
//!     .icon(IconType::Info)
//!     .show();
//!
//! // 函数式 API（仍可用）
//! // light_file_dialog::message_box(
//! //     Some("hello"),
//! //     Some("Welcome"),
//! //     MessageBoxType::Ok,
//! //     IconType::Info,
//! //     1,
//! // );
//! ```
//!
//! ## 说明
//!
//! - 对应 C 版 v3.21.3。
//! - 字符串一律使用 UTF-8（Rust 原生 `&str`/`String`）。
//! - 全局状态（`tinyfd_verbose` 等）改用原子类型，线程安全。
//! - 对话框函数在所有平台上签名一致，通过 `cfg` 选择实现。
//! - 提供两层 API：顶层函数式（对应 C 版签名）和 [`dialog`] 模块的 Builder 模式。

mod helpers;
mod state;
pub mod types;

/// Builder 模式 API。
///
/// 每个 Builder 用 `new()` 接收必填参数，链式配置后 `.show()` 执行。
/// 详见 [模块文档](self)。
pub mod dialog;

#[cfg(windows)]
mod windows;
#[cfg(not(windows))]
mod unix;

// 平台实现需要通过 `dialog_impl` 模块导出统一接口。
#[cfg(windows)]
use windows as dialog_impl;
#[cfg(not(windows))]
use unix as dialog_impl;

pub use helpers::*;
pub use state::*;
pub use types::*;

/// `tinyfd_query` 特殊标题。传给对话框函数的 `title` 参数时，
/// 不会显示对话框，而是查询当前会使用的后端。
pub const QUERY_TITLE: &str = "tinyfd_query";

/// 发出蜂鸣声。对应 C 版 `tinyfd_beep`。
pub fn beep() {
    dialog_impl::beep();
}

/// 显示通知弹窗。对应 C 版 `tinyfd_notifyPopup`。
///
/// - `title`：标题，`None` 或空串表示无标题。
/// - `message`：消息内容，可含 `\n` `\t`。
/// - `icon`：图标类型。
///
/// 返回值仅在 `title == Some(QUERY_TITLE)` 时有意义：1 表示图形模式，0 表示控制台模式。
pub fn notify_popup(title: Option<&str>, message: Option<&str>, icon: IconType) -> i32 {
    let title = title.unwrap_or("");
    let message = message.unwrap_or("");
    if title == QUERY_TITLE {
        return dialog_impl::query_notify_popup(icon);
    }
    dialog_impl::notify_popup(title, message, icon)
}

/// 显示消息框。对应 C 版 `tinyfd_messageBox`。
///
/// - `title`：标题。
/// - `message`：消息内容，可含 `\n` `\t`。
/// - `dialog_type`：对话框类型（按钮组合）。
/// - `icon`：图标类型。
/// - `default_button`：默认按钮（0=cancel/no，1=ok/yes，2=no in yesnocancel）。
///
/// 返回按下的按钮。当 `title == Some(QUERY_TITLE)` 时返回的值表示图形(1)/控制台(0)模式，
/// 此时应调用 [`response`] 获取后端名。
pub fn message_box(
    title: Option<&str>,
    message: Option<&str>,
    dialog_type: MessageBoxType,
    icon: IconType,
    default_button: i32,
) -> MessageBoxButton {
    let title = title.unwrap_or("");
    let message = message.unwrap_or("");
    if title == QUERY_TITLE {
        let mode = dialog_impl::query_message_box(icon);
        // 查询模式下返回的 Ok 表示图形模式，Cancel 表示控制台模式。
        return if mode == 1 {
            MessageBoxButton::Ok
        } else {
            MessageBoxButton::Cancel
        };
    }
    let result = dialog_impl::message_box(title, message, dialog_type, icon, default_button);
    MessageBoxButton::from_int(result)
}

/// 显示输入框或密码框。对应 C 版 `tinyfd_inputBox`。
///
/// - `title`：标题。
/// - `message`：提示信息（`\n` `\t` 无效）。
/// - `default_input`：`None` 表示密码框；`Some("")` 表示普通输入框；`Some("text")` 表示带默认值。
///
/// 返回 `Some(用户输入)`，取消时返回 `None`。
pub fn input_box(
    title: Option<&str>,
    message: Option<&str>,
    default_input: Option<&str>,
) -> Option<String> {
    let title = title.unwrap_or("");
    let message = message.unwrap_or("");
    if title == QUERY_TITLE {
        let mode = dialog_impl::query_input_box();
        return if mode == 1 { Some(String::new()) } else { None };
    }
    dialog_impl::input_box(title, message, default_input)
}

/// 保存文件对话框。对应 C 版 `tinyfd_saveFileDialog`。
///
/// - `title`：标题。
/// - `default_path`：默认路径或文件名；以分隔符结尾表示仅目录。
/// - `filter_patterns`：过滤器模式，如 `&["*.txt", "*.json"]`。
/// - `filter_description`：过滤器描述，如 `Some("text files")`。
///
/// 返回所选文件完整路径，取消返回 `None`。
pub fn save_file_dialog(
    title: Option<&str>,
    default_path: Option<&str>,
    filter_patterns: &[&str],
    filter_description: Option<&str>,
) -> Option<String> {
    let title = title.unwrap_or("");
    if title == QUERY_TITLE {
        let mode = dialog_impl::query_save_file_dialog();
        return if mode == 1 { Some(String::new()) } else { None };
    }
    dialog_impl::save_file_dialog(
        title,
        default_path.unwrap_or(""),
        filter_patterns,
        filter_description.unwrap_or(""),
    )
}

/// 打开文件对话框。对应 C 版 `tinyfd_openFileDialog`。
///
/// - `title`：标题。
/// - `default_path`：默认路径。
/// - `filter_patterns`：过滤器模式。
/// - `filter_description`：过滤器描述。
/// - `allow_multiple`：是否允许多选。多选时返回的多个路径以 `|` 分隔。
///
/// 返回所选文件路径（多选以 `|` 分隔），取消返回 `None`。
pub fn open_file_dialog(
    title: Option<&str>,
    default_path: Option<&str>,
    filter_patterns: &[&str],
    filter_description: Option<&str>,
    allow_multiple: bool,
) -> Option<String> {
    let title = title.unwrap_or("");
    if title == QUERY_TITLE {
        let mode = dialog_impl::query_open_file_dialog();
        return if mode == 1 { Some(String::new()) } else { None };
    }
    dialog_impl::open_file_dialog(
        title,
        default_path.unwrap_or(""),
        filter_patterns,
        filter_description.unwrap_or(""),
        allow_multiple,
    )
}

/// 选择文件夹对话框。对应 C 版 `tinyfd_selectFolderDialog`。
///
/// 返回所选文件夹路径，取消返回 `None`。
pub fn select_folder_dialog(title: Option<&str>, default_path: Option<&str>) -> Option<String> {
    let title = title.unwrap_or("");
    if title == QUERY_TITLE {
        let mode = dialog_impl::query_select_folder_dialog();
        return if mode == 1 { Some(String::new()) } else { None };
    }
    dialog_impl::select_folder_dialog(title, default_path.unwrap_or(""))
}

/// 颜色选择器。对应 C 版 `tinyfd_colorChooser`。
///
/// - `title`：标题（Windows 的 `ChooseColor` 不支持标题）。
/// - `default_hex`：默认颜色 `#RRGGBB`。优先使用。
/// - `default_rgb`：当 `default_hex` 为 `None` 时使用。
///
/// 返回 `Some((hex_string, rgb))`，取消返回 `None`。
pub fn color_chooser(
    title: Option<&str>,
    default_hex: Option<&str>,
    default_rgb: Option<RgbColor>,
) -> Option<(String, RgbColor)> {
    let title = title.unwrap_or("");
    if title == QUERY_TITLE {
        let mode = dialog_impl::query_color_chooser();
        return if mode == 1 {
            Some((String::new(), RgbColor::default()))
        } else {
            None
        };
    }
    dialog_impl::color_chooser(title, default_hex, default_rgb)
}
