//! Builder 模式 API。
//!
//! 在 [`crate`] 顶层函数式 API 之上提供链式 Builder，调用更简洁：
//! //! ```no_run
//! use light_file_dialog::dialog::*;
//! use light_file_dialog::{IconType, MessageBoxType};
//!
//! MessageBox::new("hello", "Welcome")
//!     .icon(IconType::Info)
//!     .buttons(MessageBoxType::Ok)
//!     .show();
//!
//! OpenFileDialog::new("选择文件")
//!     .filter(&["*.txt", "*.json"])
//!     .filter_description("text files")
//!     .multiple()
//!     .show();
//! ```
//!
//! ## 设计
//!
//! - 每个 Builder 用 `new()` 接收必填参数，其余配置用链式方法。
//! - [`Dialog`] trait 统一 `show()` 接口；`show()` 消费 Builder 并返回结果。
//! - 空串等价于"无"（对应函数式 API 的 `None`），无需 `Option`。
//! - 查询后端用 [`DialogBackend::query`]，替代 `QUERY_TITLE` 魔法字符串。

use crate::types::{IconType, MessageBoxType, MessageBoxButton, RgbColor};

/// 对话框 Builder 的统一接口。
///
/// 每个 Builder 实现 `show()`，消费 self 并返回对话框结果。
pub trait Dialog: Sized {
    /// 对话框结果类型。
    type Output;

    /// 显示对话框，返回结果。
    fn show(self) -> Self::Output;
}

// ───────────────────────── MessageBox ─────────────────────────

/// 消息框 Builder。
///
/// 对应 [`crate::message_box`]。
pub struct MessageBox {
    title: String,
    message: String,
    buttons: MessageBoxType,
    icon: IconType,
    default_button: i32,
}

impl MessageBox {
    /// 创建消息框。`title`/`message` 为空串等价于 `None`。
    pub fn new(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            message: message.into(),
            buttons: MessageBoxType::Ok,
            icon: IconType::Info,
            default_button: 1,
        }
    }

    /// 设置按钮组合。默认 [`MessageBoxType::Ok`]。
    pub fn buttons(mut self, buttons: MessageBoxType) -> Self {
        self.buttons = buttons;
        self
    }

    /// 设置图标。默认 [`IconType::Info`]。
    pub fn icon(mut self, icon: IconType) -> Self {
        self.icon = icon;
        self
    }

    /// 设置默认按钮（0=cancel/no，1=ok/yes，2=no in yesnocancel）。默认 1。
    pub fn default_button(mut self, button: i32) -> Self {
        self.default_button = button;
        self
    }
}

impl Dialog for MessageBox {
    type Output = MessageBoxButton;

    fn show(self) -> Self::Output {
        crate::message_box(
            Some(&self.title),
            Some(&self.message),
            self.buttons,
            self.icon,
            self.default_button,
        )
    }
}

// ───────────────────────── Notify ─────────────────────────

/// 通知弹窗 Builder。
///
/// 对应 [`crate::notify_popup`]。
pub struct Notify {
    title: String,
    message: String,
    icon: IconType,
}

impl Notify {
    /// 创建通知。`title`/`message` 为空串等价于 `None`。
    pub fn new(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            message: message.into(),
            icon: IconType::Info,
        }
    }

    /// 设置图标。默认 [`IconType::Info`]。
    pub fn icon(mut self, icon: IconType) -> Self {
        self.icon = icon;
        self
    }
}

impl Dialog for Notify {
    type Output = i32;

    fn show(self) -> Self::Output {
        crate::notify_popup(Some(&self.title), Some(&self.message), self.icon)
    }
}

// ───────────────────────── InputBox ─────────────────────────

/// 输入框 / 密码框 Builder。
///
/// 对应 [`crate::input_box`]。默认是普通输入框，调用 [`InputBox::password`]
/// 切换为密码框。
pub struct InputBox {
    title: String,
    message: String,
    default_input: Option<String>,
}

impl InputBox {
    /// 创建输入框。`title`/`message` 为空串等价于 `None`。
    pub fn new(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            message: message.into(),
            default_input: Some(String::new()),
        }
    }

    /// 设置默认输入值。默认为空串。
    pub fn default(mut self, value: impl Into<String>) -> Self {
        self.default_input = Some(value.into());
        self
    }

    /// 切换为密码框（不显示输入内容）。
    pub fn password(mut self) -> Self {
        self.default_input = None;
        self
    }
}

impl Dialog for InputBox {
    type Output = Option<String>;

    fn show(self) -> Self::Output {
        crate::input_box(
            Some(&self.title),
            Some(&self.message),
            self.default_input.as_deref(),
        )
    }
}

// ───────────────────────── OpenFileDialog ─────────────────────────

/// 打开文件对话框 Builder。
///
/// 对应 [`crate::open_file_dialog`]。
pub struct OpenFileDialog {
    title: String,
    default_path: String,
    filter_patterns: Vec<String>,
    filter_description: String,
    allow_multiple: bool,
}

impl OpenFileDialog {
    /// 创建打开文件对话框。`title` 为空串等价于 `None`。
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            default_path: String::new(),
            filter_patterns: Vec::new(),
            filter_description: String::new(),
            allow_multiple: false,
        }
    }

    /// 设置默认路径。
    pub fn default_path(mut self, path: impl Into<String>) -> Self {
        self.default_path = path.into();
        self
    }

    /// 设置过滤器模式，如 `&["*.txt", "*.json"]`。
    pub fn filter(mut self, patterns: &[&str]) -> Self {
        self.filter_patterns = patterns.iter().map(|s| (*s).to_string()).collect();
        self
    }

    /// 设置过滤器描述，如 `"text files"`。
    pub fn filter_description(mut self, desc: impl Into<String>) -> Self {
        self.filter_description = desc.into();
        self
    }

    /// 允许多选。多选时返回的多个路径以 `|` 分隔。
    pub fn multiple(mut self) -> Self {
        self.allow_multiple = true;
        self
    }
}

impl Dialog for OpenFileDialog {
    type Output = Option<String>;

    fn show(self) -> Self::Output {
        let patterns: Vec<&str> = self.filter_patterns.iter().map(|s| s.as_str()).collect();
        crate::open_file_dialog(
            Some(&self.title),
            Some(&self.default_path),
            &patterns,
            Some(&self.filter_description),
            self.allow_multiple,
        )
    }
}

// ───────────────────────── SaveFileDialog ─────────────────────────

/// 保存文件对话框 Builder。
///
/// 对应 [`crate::save_file_dialog`]。
pub struct SaveFileDialog {
    title: String,
    default_path: String,
    filter_patterns: Vec<String>,
    filter_description: String,
}

impl SaveFileDialog {
    /// 创建保存文件对话框。`title` 为空串等价于 `None`。
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            default_path: String::new(),
            filter_patterns: Vec::new(),
            filter_description: String::new(),
        }
    }

    /// 设置默认路径或文件名。
    pub fn default_path(mut self, path: impl Into<String>) -> Self {
        self.default_path = path.into();
        self
    }

    /// 设置过滤器模式，如 `&["*.txt", "*.json"]`。
    pub fn filter(mut self, patterns: &[&str]) -> Self {
        self.filter_patterns = patterns.iter().map(|s| (*s).to_string()).collect();
        self
    }

    /// 设置过滤器描述，如 `"text files"`。
    pub fn filter_description(mut self, desc: impl Into<String>) -> Self {
        self.filter_description = desc.into();
        self
    }
}

impl Dialog for SaveFileDialog {
    type Output = Option<String>;

    fn show(self) -> Self::Output {
        let patterns: Vec<&str> = self.filter_patterns.iter().map(|s| s.as_str()).collect();
        crate::save_file_dialog(
            Some(&self.title),
            Some(&self.default_path),
            &patterns,
            Some(&self.filter_description),
        )
    }
}

// ───────────────────────── SelectFolder ─────────────────────────

/// 选择文件夹对话框 Builder。
///
/// 对应 [`crate::select_folder_dialog`]。
pub struct SelectFolder {
    title: String,
    default_path: String,
}

impl SelectFolder {
    /// 创建选择文件夹对话框。`title` 为空串等价于 `None`。
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            default_path: String::new(),
        }
    }

    /// 设置默认路径。
    pub fn default_path(mut self, path: impl Into<String>) -> Self {
        self.default_path = path.into();
        self
    }
}

impl Dialog for SelectFolder {
    type Output = Option<String>;

    fn show(self) -> Self::Output {
        crate::select_folder_dialog(Some(&self.title), Some(&self.default_path))
    }
}

// ───────────────────────── ColorChooser ─────────────────────────

/// 颜色选择器 Builder。
///
/// 对应 [`crate::color_chooser`]。
pub struct ColorChooser {
    title: String,
    default_hex: Option<String>,
    default_rgb: Option<RgbColor>,
}

impl ColorChooser {
    /// 创建颜色选择器。`title` 为空串等价于 `None`。
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            default_hex: None,
            default_rgb: None,
        }
    }

    /// 设置默认颜色（`#RRGGBB`）。优先于 `default_rgb`。
    pub fn default_hex(mut self, hex: impl Into<String>) -> Self {
        self.default_hex = Some(hex.into());
        self
    }

    /// 设置默认颜色（RGB）。当 `default_hex` 未设置时使用。
    pub fn default_rgb(mut self, rgb: RgbColor) -> Self {
        self.default_rgb = Some(rgb);
        self
    }
}

impl Dialog for ColorChooser {
    type Output = Option<(String, RgbColor)>;

    fn show(self) -> Self::Output {
        crate::color_chooser(
            Some(&self.title),
            self.default_hex.as_deref(),
            self.default_rgb,
        )
    }
}

// ───────────────────────── DialogBackend ─────────────────────────

/// 后端查询结果。对应函数式 API 中以 `QUERY_TITLE` 调用对话框函数的行为。
#[derive(Debug, Clone)]
pub struct DialogBackend {
    /// 是否图形模式。
    pub graphic: bool,
    /// 后端名称（如 `"windows_wchar"`、`"zenity"`）。
    pub name: String,
}

impl DialogBackend {
    /// 查询当前会使用的后端。
    ///
    /// 等价于以 `QUERY_TITLE` 调用 [`crate::input_box`] 后读取 [`crate::response`]。
    pub fn query() -> Self {
        let graphic = crate::input_box(Some(crate::QUERY_TITLE), None, None).is_some();
        Self {
            graphic,
            name: crate::response(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn message_box_builder_defaults() {
        let b = MessageBox::new("t", "m");
        assert_eq!(b.buttons, MessageBoxType::Ok);
        assert_eq!(b.icon, IconType::Info);
        assert_eq!(b.default_button, 1);
    }

    #[test]
    fn input_box_password_sets_none() {
        let b = InputBox::new("t", "m").password();
        assert!(b.default_input.is_none());
    }

    #[test]
    fn input_box_default_is_empty() {
        let b = InputBox::new("t", "m");
        assert_eq!(b.default_input.as_deref(), Some(""));
    }

    #[test]
    fn open_file_dialog_multiple() {
        let b = OpenFileDialog::new("t").multiple();
        assert!(b.allow_multiple);
    }

    #[test]
    fn color_chooser_defaults_none() {
        let b = ColorChooser::new("t");
        assert!(b.default_hex.is_none());
        assert!(b.default_rgb.is_none());
    }
}
