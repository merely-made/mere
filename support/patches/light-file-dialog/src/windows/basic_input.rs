//! Windows basic-input 模式：当既无图形界面、也无 `dialog.exe` 时，
//! 直接从控制台 stdin 读取输入。
//!
//! 对应 C 版 `tinyfiledialogs.c` 第 2864-3410 行的 basicinput 分支。

use std::io::{self, Write};

use windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE;
use windows_sys::Win32::System::Console::{
    GetConsoleMode, GetStdHandle, ReadConsoleW, SetConsoleMode, WriteConsoleW,
    CONSOLE_MODE, ENABLE_ECHO_INPUT, ENABLE_LINE_INPUT, STD_INPUT_HANDLE, STD_OUTPUT_HANDLE,
};

use crate::helpers;
use crate::types::{IconType, MessageBoxType, RgbColor};
use super::utf;

/// 获取控制台窗口句柄。对应 C 版 `GetConsoleWindow`。
fn console_window() -> isize {
    extern "system" {
        fn GetConsoleWindow() -> isize;
    }
    unsafe { GetConsoleWindow() }
}

/// 判断是否在控制台中运行。
pub fn in_console() -> bool {
    console_window() != 0
}

/// 用 WriteConsoleW 输出 UTF-8 字符串（先转 UTF-16）。对应 C 版 `writeUtf8`。
fn write_out(s: &str) {
    let wide = utf::utf8_to_utf16(s);
    if wide.is_empty() {
        return;
    }
    let handle = unsafe { GetStdHandle(STD_OUTPUT_HANDLE) };
    if handle.is_null() {
        // 回退到标准输出。
        let _ = io::stdout().write_all(s.as_bytes());
        let _ = io::stdout().flush();
        return;
    }
    let mut written: u32 = 0;
    unsafe {
        WriteConsoleW(
            handle,
            wide.as_ptr() as *const _,
            wide.len() as u32,
            &mut written,
            std::ptr::null(),
        );
    }
}

/// 读取单个字符（无需回车）。对应 C 版 `_getch()`。
fn read_char() -> char {
    let handle = unsafe { GetStdHandle(STD_INPUT_HANDLE) };
    if handle.is_null() || handle == INVALID_HANDLE_VALUE {
        // 回退：读一行取首字符。
        let mut line = String::new();
        let _ = io::stdin().read_line(&mut line);
        return line.chars().next().unwrap_or('\n');
    }

    let mut old_mode: CONSOLE_MODE = 0;
    unsafe {
        if GetConsoleMode(handle, &mut old_mode) == 0 {
            let mut line = String::new();
            let _ = io::stdin().read_line(&mut line);
            return line.chars().next().unwrap_or('\n');
        }
        // 设为单字符模式：关闭行缓冲和回显。
        let new_mode = old_mode & !(ENABLE_LINE_INPUT | ENABLE_ECHO_INPUT);
        SetConsoleMode(handle, new_mode);
    }

    let mut buf = [0u16; 1];
    let mut read: u32 = 0;
    let ch = unsafe {
        ReadConsoleW(handle, buf.as_mut_ptr() as *mut _, 1, &mut read, std::ptr::null());
        SetConsoleMode(handle, old_mode);
        if read > 0 {
            char::from_u32(buf[0] as u32).unwrap_or('\n')
        } else {
            '\n'
        }
    };
    ch.to_ascii_lowercase()
}

/// 读取一行（带回显）。对应 C 版 `fgets(lBuff, ..., stdin)`。
fn read_line_echo() -> String {
    let mut line = String::new();
    let _ = io::stdin().read_line(&mut line);
    line.trim_end_matches(['\r', '\n']).to_string()
}

/// 读取一行（不回显，用于密码）。对应 C 版 `SetConsoleMode(hStdin, mode & ~ENABLE_ECHO_INPUT)`。
fn read_line_no_echo() -> String {
    let handle = unsafe { GetStdHandle(STD_INPUT_HANDLE) };
    if handle.is_null() || handle == INVALID_HANDLE_VALUE {
        // 无法关闭回显，回退。
        return read_line_echo();
    }
    let mut old_mode: CONSOLE_MODE = 0;
    unsafe {
        if GetConsoleMode(handle, &mut old_mode) == 0 {
            return read_line_echo();
        }
        SetConsoleMode(handle, old_mode & !ENABLE_ECHO_INPUT);
    }
    // 用 ReadConsoleW 读取 UTF-16。
    let mut buf = [0u16; helpers::MAX_PATH_OR_CMD];
    let mut read: u32 = 0;
    let result = unsafe {
        let r = ReadConsoleW(
            handle,
            buf.as_mut_ptr() as *mut _,
            buf.len() as u32,
            &mut read,
            std::ptr::null(),
        );
        SetConsoleMode(handle, old_mode);
        if r != 0 && read > 0 {
            let s = utf::utf16_to_utf8(&buf[..read as usize]);
            s.trim_end_matches(['\r', '\n']).to_string()
        } else {
            String::new()
        }
    };
    // 输出换行以模拟回车。
    write_out("\n");
    result
}

/// basic-input 消息框。对应 C 版 basicinput messageBox 分支。
pub fn message_box(
    _title: &str,
    message: &str,
    dialog_type: MessageBoxType,
    _icon: IconType,
) -> i32 {
    write_out(message);
    write_out("\n");
    match dialog_type {
        MessageBoxType::Ok => {
            write_out("(press enter to continue): ");
            read_char();
            1
        }
        MessageBoxType::OkCancel => {
            write_out("[O]kay/[C]ancel: ");
            loop {
                let c = read_char();
                if c == 'o' {
                    return 1;
                }
                if c == 'c' {
                    return 0;
                }
            }
        }
        MessageBoxType::YesNo => {
            write_out("y/n: ");
            loop {
                let c = read_char();
                if c == 'y' {
                    return 1;
                }
                if c == 'n' {
                    return 0;
                }
            }
        }
        MessageBoxType::YesNoCancel => {
            write_out("[Y]es/[N]o/[C]ancel: ");
            loop {
                let c = read_char();
                if c == 'y' {
                    return 1;
                }
                if c == 'n' {
                    return 2;
                }
                if c == 'c' {
                    return 0;
                }
            }
        }
    }
}

/// basic-input 输入框/密码框。对应 C 版 basicinput inputBox 分支。
pub fn input_box(title: &str, message: &str, default_input: Option<&str>) -> Option<String> {
    if !title.is_empty() {
        write_out(title);
        write_out("\n");
    }
    if !message.is_empty() {
        write_out(message);
        write_out("\n");
    }
    if default_input.is_none() {
        // 密码框。
        write_out("(esc+enter to cancel): ");
        let input = read_line_no_echo();
        if input.contains('\u{1b}') {
            return None;
        }
        if input.is_empty() {
            return None;
        }
        Some(input)
    } else {
        write_out("(esc+enter to cancel): ");
        let input = read_line_echo();
        if input.contains('\u{1b}') || input.is_empty() {
            return None;
        }
        Some(input)
    }
}

/// basic-input 保存文件对话框。
/// 对应 C 版：用 inputBox 让用户手输路径，再校验。
pub fn save_file_dialog(title: &str) -> Option<String> {
    let curdir = helpers::current_dir();
    let msg = format!("Save file in {}", curdir);
    let input = input_box(title, &msg, Some(""))?;
    let dir = helpers::path_without_final_slash(&input);
    let name = helpers::last_name(&input);
    if !helpers::dir_exists(dir) {
        return None;
    }
    if !helpers::filename_valid(name) {
        return None;
    }
    Some(input)
}

/// basic-input 打开文件对话框。
pub fn open_file_dialog(title: &str) -> Option<String> {
    let curdir = helpers::current_dir();
    let msg = format!("Open file from {}", curdir);
    let input = input_box(title, &msg, Some(""))?;
    if !helpers::file_exists(&input) {
        return None;
    }
    Some(input)
}

/// basic-input 选择文件夹对话框。
pub fn select_folder_dialog(title: &str) -> Option<String> {
    let curdir = helpers::current_dir();
    let msg = format!("Select folder from {}", curdir);
    let input = input_box(title, &msg, Some(""))?;
    if !helpers::dir_exists(&input) {
        return None;
    }
    Some(input)
}

/// basic-input 颜色选择器。
pub fn color_chooser(
    title: &str,
    default_hex: Option<&str>,
    default_rgb: Option<RgbColor>,
) -> Option<(String, RgbColor)> {
    let default = match default_hex {
        Some(h) => h.to_string(),
        None => default_rgb.map(|c| c.to_hex()).unwrap_or_else(|| "#000000".to_string()),
    };
    let input = input_box(title, "Enter hex rgb color (i.e. #f5ca20)", Some(&default))?;
    let rgb = RgbColor::from_hex(&input)?;
    Some((rgb.to_hex(), rgb))
}
