//! Windows 控制台模式对话框（依赖 `dialog.exe`，即 ncurses 的 Windows 移植）。
//!
//! 对应 C 版 `tinyfiledialogs.c` 第 2429-2862 行的 `xxxWinConsole` 函数。
//! 这是 `tinyfd_allowCursesDialogs=1` 且系统装了 `dialog.exe` 时的回退路径。

use std::fs;
use std::os::windows::process::CommandExt;
use std::process::Command;

use crate::types::{IconType, MessageBoxType};

const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// 探测 `dialog.exe` 是否存在。对应 C 版 `dialogPresent`。
pub fn dialog_present() -> bool {
    static CACHED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *CACHED.get_or_init(|| {
        if crate::state::allow_curses_dialogs() == 0 {
            return false;
        }
        Command::new("where")
            .arg("dialog.exe")
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .map(|o| o.status.success() && !o.stdout.is_empty())
            .unwrap_or(false)
    })
}

/// 临时文件路径。
fn temp_txt() -> std::path::PathBuf {
    std::env::temp_dir().join("tinyfd.txt")
}

/// 执行 dialog.exe 命令并把 stdout/stderr 写到临时文件。
/// 返回退出码。
fn run_dialog(args: &str) -> i32 {
    let txt = temp_txt();
    // dialog.exe 用 Unix 分隔符，需要把 \ 换成 /。
    let cmd = format!("{} > \"{}\" 2>&1", args, txt.to_string_lossy());
    let status = Command::new("cmd")
        .creation_flags(CREATE_NO_WINDOW)
        .args(["/C", &cmd])
        .status();
    status.map(|s| s.code().unwrap_or(-1)).unwrap_or(-1)
}

/// 读取并清理临时文件。
fn read_temp() -> String {
    let txt = temp_txt();
    let content = fs::read_to_string(&txt).unwrap_or_default();
    let _ = fs::remove_file(&txt);
    content.trim_end_matches(['\r', '\n']).to_string()
}

/// 消息框（console）。对应 C 版 `messageBoxWinConsole`。
/// 返回 0=cancel/no，1=ok/yes，2=no。
pub fn message_box(
    title: &str,
    message: &str,
    dialog_type: MessageBoxType,
    _icon: IconType,
    default_button: i32,
) -> i32 {
    let title_flag = if title.is_empty() {
        String::new()
    } else {
        format!(" --title \"{}\"", title.replace('"', " "))
    };
    let backtitle = " --backtitle \"tab: move focus\"";
    let defaultno = if default_button == 0 { " --defaultno" } else { "" };

    let (type_flag, is_menu) = match dialog_type {
        MessageBoxType::Ok => (" --msgbox".to_string(), false),
        MessageBoxType::OkCancel => (
            format!("{} --yes-label Ok --no-label Cancel --yesno", defaultno),
            false,
        ),
        MessageBoxType::YesNo => (format!("{} --yesno", defaultno), false),
        MessageBoxType::YesNoCancel => (" --menu".to_string(), true),
    };

    let msg = message.replace('"', " ");
    let exit = if is_menu {
        // yesnocancel: menu 三选一
        let cmd = format!(
            "dialog{}{}{} \"{}\" 0 60 0 Yes \"\" No \"\" Cancel \"\"",
            title_flag, backtitle, type_flag, msg
        );
        run_dialog(&cmd)
    } else {
        let cmd = format!("dialog{}{}{} \"{}\" 10 60", title_flag, backtitle, type_flag, msg);
        run_dialog(&cmd)
    };

    if is_menu {
        let res = read_temp();
        match res.as_str() {
            "Yes" => 1,
            "No" => 2,
            _ => 0,
        }
    } else {
        // yesno/okcancel: 0=yes/ok, 1=cancel/no
        match exit {
            0 => 1,
            _ => 0,
        }
    }
}

/// 输入框（console）。对应 C 版 `inputBoxWinConsole`。
pub fn input_box(title: &str, message: &str, default_input: Option<&str>) -> Option<String> {
    let title_flag = if title.is_empty() {
        String::new()
    } else {
        format!(" --title \"{}\"", title.replace('"', " "))
    };
    let backtitle = " --backtitle \"tab: move focus\"";
    let msg = message.replace('"', " ");

    let (type_flag, default_arg) = if default_input.is_none() {
        // 密码框
        ("--passwordbox --insecure", String::new())
    } else {
        ("--inputbox", format!(" \"{}\"", default_input.unwrap_or("").replace('"', " ")))
    };

    let cmd = format!(
        "dialog{}{} --{} \"{}\" 10 60{}",
        title_flag, backtitle, type_flag, msg, default_arg
    );
    let _ = run_dialog(&cmd);
    let res = read_temp();
    if res.is_empty() {
        None
    } else {
        Some(res)
    }
}

/// 保存文件对话框（console）。对应 C 版 `saveFileDialogWinConsole`。
pub fn save_file_dialog(title: &str, default_path: &str) -> Option<String> {
    file_dialog(title, default_path, "--fselect")
}

/// 打开文件对话框（console）。
pub fn open_file_dialog(title: &str, default_path: &str) -> Option<String> {
    file_dialog(title, default_path, "--fselect")
}

/// 选择文件夹对话框（console）。对应 C 版 `selectFolderDialogWinConsole`。
pub fn select_folder_dialog(title: &str, default_path: &str) -> Option<String> {
    file_dialog(title, default_path, "--dselect")
}

fn file_dialog(title: &str, default_path: &str, kind: &str) -> Option<String> {
    let title_flag = if title.is_empty() {
        String::new()
    } else {
        format!(" --title \"{}\"", title.replace('"', " "))
    };
    // dialog.exe 用 Unix 分隔符。
    let path = default_path.replace('\\', "/");
    let cmd = format!("dialog{} --{} \"{}\" 0 60", title_flag, kind.trim_start_matches("--"), path);
    let _ = run_dialog(&cmd);
    let res = read_temp();
    if res.is_empty() {
        None
    } else {
        // 转回 Windows 分隔符。
        Some(res.replace('/', "\\"))
    }
}
