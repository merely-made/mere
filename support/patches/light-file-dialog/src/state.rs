//! 全局状态变量，对应 C 版 `tinyfiledialogs.c` 中的全局 `int` 变量。
//!
//! C 版用裸全局变量（非线程安全），Rust 版改用原子类型和互斥锁，
//! 既保留"运行时调整行为"的能力，又保证线程安全。

use std::sync::{Mutex, OnceLock};

/// 库版本号，对应 C 版 `tinyfd_version`。
pub const VERSION: &str = "3.21.3";

/// `tinyfd_verbose`：0（默认）或 1。在 unix 上打印命令行调用。
static VERBOSE: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(0);

/// `tinyfd_silent`：1（默认）或 0。在 unix 上隐藏被调用对话框的错误和警告。
static SILENT: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(1);

/// `tinyfd_allowCursesDialogs`：0（默认）或 1。是否允许 curses 对话框。
static ALLOW_CURSES: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(0);

/// `tinyfd_forceConsole`：0（默认）或 1。强制控制台模式。
static FORCE_CONSOLE: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(0);

/// `tinyfd_winUtf8`（仅 Windows）：1（默认）或 0。char 字符串是否按 UTF-8 处理。
#[cfg(windows)]
static WIN_UTF8: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(1);

/// `tinyfd_response[1024]`：当以 `"tinyfd_query"` 作为 aTitle 调用对话框函数时，
/// 会被填充为实际使用的后端名（如 `"zenity"`、`"windows_wchar"` 等）。
static RESPONSE: OnceLock<Mutex<String>> = OnceLock::new();

fn response_lock() -> &'static Mutex<String> {
    RESPONSE.get_or_init(|| Mutex::new(String::new()))
}

/// 设置 `tinyfd_verbose`。
pub fn set_verbose(value: i32) {
    VERBOSE.store(value, std::sync::atomic::Ordering::Relaxed);
}

/// 读取 `tinyfd_verbose`。
pub fn verbose() -> i32 {
    VERBOSE.load(std::sync::atomic::Ordering::Relaxed)
}

/// 设置 `tinyfd_silent`。
pub fn set_silent(value: i32) {
    SILENT.store(value, std::sync::atomic::Ordering::Relaxed);
}

/// 读取 `tinyfd_silent`。
pub fn silent() -> i32 {
    SILENT.load(std::sync::atomic::Ordering::Relaxed)
}

/// 设置 `tinyfd_allowCursesDialogs`。
pub fn set_allow_curses_dialogs(value: i32) {
    ALLOW_CURSES.store(value, std::sync::atomic::Ordering::Relaxed);
}

/// 读取 `tinyfd_allowCursesDialogs`。
pub fn allow_curses_dialogs() -> i32 {
    ALLOW_CURSES.load(std::sync::atomic::Ordering::Relaxed)
}

/// 设置 `tinyfd_forceConsole`。
pub fn set_force_console(value: i32) {
    FORCE_CONSOLE.store(value, std::sync::atomic::Ordering::Relaxed);
}

/// 读取 `tinyfd_forceConsole`。
pub fn force_console() -> i32 {
    FORCE_CONSOLE.load(std::sync::atomic::Ordering::Relaxed)
}

/// 设置 `tinyfd_winUtf8`（仅 Windows）。
#[cfg(windows)]
pub fn set_win_utf8(value: i32) {
    WIN_UTF8.store(value, std::sync::atomic::Ordering::Relaxed);
}

/// 读取 `tinyfd_winUtf8`（仅 Windows）。
#[cfg(windows)]
pub fn win_utf8() -> i32 {
    WIN_UTF8.load(std::sync::atomic::Ordering::Relaxed)
}

/// 写入 `tinyfd_response`。
pub(crate) fn set_response(value: &str) {
    let mut guard = response_lock().lock().unwrap();
    guard.clear();
    // 对应 C 版 char[1024]，截断到 1023 字节。
    let max = 1023.min(value.len());
    guard.push_str(&value[..max]);
}

/// 读取 `tinyfd_response` 的副本。
pub fn response() -> String {
    response_lock().lock().unwrap().clone()
}

/// 关于运行环境需求的说明文本，对应 C 版 `tinyfd_needs`。
pub fn needs() -> &'static str {
    if cfg!(windows) {
        "tiny file dialogs on Windows needs:\n   a graphic display\nor dialog.exe (curses console mode  ** Disabled by default **)\nor a console for basic input"
    } else {
        "tiny file dialogs on UNIX needs:\n   applescript or kdialog or yad or Xdialog\nor zenity (or matedialog, shellementary, qarma, shanty, boxer)\nor python (2 or 3) + tkinter + python-dbus (optional)\nor dialog (opens console if needed) ** Disabled by default **\nor xterm + bash (opens console for basic input)\nor existing console for basic input."
    }
}

/// 重置所有全局状态到默认值。主要用于测试。
pub fn reset_to_defaults() {
    set_verbose(0);
    set_silent(1);
    set_allow_curses_dialogs(0);
    set_force_console(0);
    #[cfg(windows)]
    set_win_utf8(1);
    set_response("");
}
