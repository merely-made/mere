//! light-file-dialog 的 Hello World 示例。
//! 对应 C 版 `hello.c`。
//!
//! 演示 Builder 模式 API（推荐）。函数式 API 见顶层 [`light_file_dialog`] 模块文档。

use light_file_dialog::dialog::*;
use light_file_dialog::{IconType, RgbColor, VERSION};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    light_file_dialog::set_verbose(if args.len() > 1 { 1 } else { 0 });
    light_file_dialog::set_silent(1);
    light_file_dialog::set_force_console(0);
    #[cfg(windows)]
    light_file_dialog::set_win_utf8(1);

    light_file_dialog::beep();

    // 查询当前会使用的后端。
    let backend = DialogBackend::query();
    let mode = if backend.graphic { "graphic mode" } else { "console mode" };
    let msg = format!("light-file-dialog v{}\n{}: {}", VERSION, mode, backend.name);
    MessageBox::new("hello", &msg).icon(IconType::Info).show();

    Notify::new("the title", "the message\n\tfrom outer-space")
        .icon(IconType::Info)
        .show();

    // 密码框。
    let password = InputBox::new("a password box", "your password will be revealed later")
        .password()
        .show();
    let password = match password {
        Some(p) => p,
        None => return,
    };
    MessageBox::new("your password as read", &password)
        .icon(IconType::Info)
        .show();

    // 保存文件对话框。
    let save_path = SaveFileDialog::new("let us save this password")
        .default_path("./passwordFile.txt")
        .filter(&["*.txt", "*.json"])
        .show();
    let save_path = match save_path {
        Some(p) => p,
        None => {
            MessageBox::new("Error", "Save file name is NULL")
                .icon(IconType::Error)
                .show();
            return;
        }
    };
    let _ = std::fs::write(&save_path, &password).map_err(|_| {
        MessageBox::new("Error", "Can not open this file in write mode")
            .icon(IconType::Error)
            .show();
    });

    // 打开文件对话框（允许多选）。
    let open_path = OpenFileDialog::new("let us read the password back")
        .default_path("../")
        .filter(&["*.txt", "*.json"])
        .filter_description("text files")
        .multiple()
        .show();
    let open_path = match open_path {
        Some(p) => p,
        None => {
            MessageBox::new("Error", "Open file name is NULL")
                .icon(IconType::Error)
                .default_button(0)
                .show();
            return;
        }
    };

    // 取第一个文件（多选以 | 分隔）。
    let first_file = open_path.split('|').next().unwrap_or(&open_path);
    if let Ok(content) = std::fs::read_to_string(first_file) {
        let line = content.lines().next().unwrap_or("");
        MessageBox::new("your password as it was saved", line)
            .icon(IconType::Info)
            .show();
    }

    // 选择文件夹。
    let folder = SelectFolder::new("let us just select a directory")
        .default_path("../../")
        .show();
    if let Some(ref f) = folder {
        MessageBox::new("The selected folder is", f)
            .icon(IconType::Info)
            .show();
    }

    // 颜色选择器。
    let color = ColorChooser::new("choose a nice color")
        .default_hex("#FF0077")
        .default_rgb(RgbColor::new(0, 128, 255))
        .show();
    if let Some((hex, _)) = color {
        MessageBox::new("The selected hexcolor is", &hex)
            .icon(IconType::Info)
            .show();
    }

    MessageBox::new("your read password was", &password)
        .icon(IconType::Info)
        .show();
}
