//! Unix 平台对话框实现。
//!
//! 对应 C 版 `tinyfiledialogs.c` 第 4447-8137 行的 Unix 实现。
//! 通过 shell 命令调用 osascript / zenity / kdialog / yad / tkinter / dialog 等后端。
//!
//! 后端优先级链：
//! osascript → kdialog → zenity 族 → yad → gxmessage/gmessage/xmessage
//! → tkinter3 → tkinter2 → xdialog/gdialog → dialog/whiptail → basic-input

mod detect;

use std::process::Command;
use std::sync::{Mutex, OnceLock};

use crate::helpers;
use crate::state;
use crate::types::{IconType, MessageBoxType, RgbColor};

use detect as d;

/// shell 命令执行器：运行 `sh -c <cmd>`，返回 stdout（去尾换行）。
fn run(cmd: &str) -> String {
    let output = Command::new("sh").arg("-c").arg(cmd).output();
    match output {
        Ok(o) => {
            let s = String::from_utf8_lossy(&o.stdout);
            s.trim_end_matches(['\r', '\n']).to_string()
        }
        Err(_) => String::new(),
    }
}

/// shell 命令执行器（不读输出）。对应 C 版 `popen` + `pclose`。
fn run_silent(cmd: &str) {
    let _ = Command::new("sh").arg("-c").arg(cmd).spawn();
}

/// 引号检测：含 `'` 或 `"` 时返回安全替换串。对应 C 版入口的 quote 检查。
fn safe_string(s: &str, label: &str) -> String {
    if helpers::quote_detected(Some(s)) {
        format!("INVALID {} WITH QUOTES", label)
    } else {
        s.to_string()
    }
}

/// 上次操作的目录（对应 C 版 `lLastDirectory`）。
fn last_dir() -> &'static Mutex<String> {
    static DIR: OnceLock<Mutex<String>> = OnceLock::new();
    DIR.get_or_init(|| Mutex::new(String::from("$PWD")))
}

/// 获取 `--attach` 参数（用于 zenity/kdialog 附着到活动窗口）。
fn attach_arg() -> String {
    if d::xprop_present() && !d::in_ssh() {
        " --attach=$(xprop -root 32x '\\t$0' _NET_ACTIVE_WINDOW | cut -f 2) ".to_string()
    } else {
        String::new()
    }
}

/// 图标类型 → zenity icon 名。
fn zenity_icon(icon: IconType) -> &'static str {
    match icon {
        IconType::Error => "dialog-error",
        IconType::Warning => "dialog-warning",
        IconType::Question => "dialog-question",
        IconType::Info => "dialog-information",
    }
}

/// 图标类型 → osascript icon 名。
fn osa_icon(icon: IconType) -> &'static str {
    match icon {
        IconType::Error => "stop",
        IconType::Warning => "caution",
        _ => "note",
    }
}

/// 图标类型 → kdialog/notify-send icon 名。
fn kd_icon(icon: IconType) -> &'static str {
    match icon {
        IconType::Error => "error",
        IconType::Warning => "warning",
        _ => "info",
    }
}

/// 发出蜂鸣声。对应 C 版 `tinyfd_beep`（Unix）。
pub fn beep() {
    if d::pactl_present() {
        run_silent("thnum=$(pactl load-module module-sine frequency=440);sleep .3;pactl unload-module $thnum");
    } else if d::osascript_present() && d::afplay_present() {
        run_silent("afplay /System/Library/Sounds/Ping.aiff");
    } else if d::osascript_present() {
        run_silent("osascript -e 'tell application \"System Events\" to beep'");
    } else if d::speakertest_present() {
        run_silent("( speaker-test -t sine -f 440 > /dev/tty )& pid=$!;sleep .5;kill -9 $pid");
    } else if d::ffplay_present() {
        run_silent("ffplay -f lavfi -i sine=f=440:d=0.15 -autoexit -nodisp");
    } else if d::play_present() {
        run_silent("play -q -n synth .3 sine 440");
    } else {
        run_silent("printf '\\a' > /dev/tty");
    }
}

// ============ notifyPopup ============

/// 通知弹窗。对应 C 版 `tinyfd_notifyPopup`（Unix）。
pub fn notify_popup(title: &str, message: &str, icon: IconType) -> i32 {
    let title = safe_string(title, "TITLE");
    let message = safe_string(message, "MESSAGE");
    let icon_str = kd_icon(icon);

    // SSH 且无 dunst → 退化为消息框。
    if d::in_ssh() && !d::dunstify_present() && !d::dunst_present() {
        message_box(&title, &message, MessageBoxType::Ok, icon, 0);
        return 1;
    }

    if d::osascript_present() {
        let cmd = format!(
            "osascript -e 'try' -e 'display notification \"{}\" with title \"{}\"' -e 'end try'",
            message.replace('"', "\\\""),
            title.replace('"', "\\\"")
        );
        run_silent(&cmd);
    } else if d::kdialog_present() == 2 {
        let cmd = format!(
            "kdialog --passivepopup \"{}\" 5 --title \"{}\"",
            message.replace('"', "\\\""),
            title.replace('"', "\\\"")
        );
        run_silent(&cmd);
    } else if d::yad_present() {
        let cmd = format!(
            "yad --notification --text=\"{}\\n{}\"",
            title.replace('"', "\\\""),
            message.replace('"', "\\\"")
        );
        run_silent(&cmd);
    } else if d::notifysend_present() {
        // \n\t 替换。
        let msg = message.replace('\n', " | ").replace('\t', "  ");
        let combined = format!("{} | {}", title, msg);
        let cmd = format!("notify-send -i '{}' '{}'", icon_str, combined.replace('\'', "'\\''"));
        run_silent(&cmd);
    } else if d::zenity_version() >= 3018 {
        let cmd = format!(
            "zenity --notification --text \"{}\\n{}\" 2>/dev/null",
            title.replace('"', "\\\""),
            message.replace('"', "\\\"")
        );
        run_silent(&cmd);
    } else {
        // 退化为消息框。
        message_box(&title, &message, MessageBoxType::Ok, icon, 0);
    }
    1
}

// ============ messageBox ============

/// 消息框。对应 C 版 `tinyfd_messageBox`（Unix）。
///
/// 返回 0=cancel/no，1=ok/yes，2=no（yesnocancel）。
pub fn message_box(
    title: &str,
    message: &str,
    dialog_type: MessageBoxType,
    icon: IconType,
    default_button: i32,
) -> i32 {
    let title = safe_string(title, "TITLE");
    let message = safe_string(message, "MESSAGE");

    if d::osascript_present() {
        osa_message_box(&title, &message, dialog_type, icon, default_button)
    } else if d::kdialog_present() > 0 {
        kd_message_box(&title, &message, dialog_type, icon, default_button)
    } else if d::zenity_present() {
        zenity_message_box(&title, &message, dialog_type, icon)
    } else if d::yad_present() {
        yad_message_box(&title, &message, dialog_type, icon)
    } else if d::gxmessage_present() || d::gmessage_present() || d::xmessage_present() {
        xmessage_message_box(&title, &message, dialog_type, icon, default_button)
    } else if d::tkinter3_present() {
        tkinter_message_box("python3", &title, &message, dialog_type, icon)
    } else if d::tkinter2_present() {
        tkinter_message_box("python2", &title, &message, dialog_type, icon)
    } else if d::dialog_present() || d::whiptail_present() {
        console_message_box(&title, &message, dialog_type, default_button)
    } else {
        basic_message_box(&message, dialog_type)
    }
}

/// osascript 消息框。
fn osa_message_box(
    title: &str,
    message: &str,
    dialog_type: MessageBoxType,
    icon: IconType,
    default_button: i32,
) -> i32 {
    let icon_str = osa_icon(icon);
    let (buttons, default_name, cancel_name) = match dialog_type {
        MessageBoxType::Ok => ("{\"OK\"}", "\"OK\"", ""),
        MessageBoxType::OkCancel => ("{\"Cancel\", \"OK\"}", if default_button == 0 { "\"Cancel\"" } else { "\"OK\"" }, "\"Cancel\""),
        MessageBoxType::YesNo => ("{\"No\", \"Yes\"}", if default_button == 0 { "\"No\"" } else { "\"Yes\"" }, "\"No\""),
        MessageBoxType::YesNoCancel => ("{\"No\", \"Yes\", \"Cancel\"}",
            match default_button { 0 => "\"No\"", 1 => "\"Yes\"", _ => "\"Cancel\"" },
            "\"Cancel\""),
    };
    let cancel_clause = if cancel_name.is_empty() {
        String::new()
    } else {
        format!(" cancel button {}", cancel_name)
    };
    let cmd = format!(
        "osascript -e 'try' -e 'set {{vButton}} to {{button returned}} of (display dialog \"{}\" with title \"{}\" with icon {} buttons {} default button {}{})' \
         -e 'if vButton is \"Yes\" then' -e 'return 1' \
         -e 'else if vButton is \"OK\" then' -e 'return 1' \
         -e 'else if vButton is \"No\" then' -e 'return 2' \
         -e 'else' -e 'return 0' -e 'end if' \
         -e 'on error number -128' -e '0' -e 'end try'",
        message.replace('"', "\\\""),
        title.replace('"', "\\\""),
        icon_str, buttons, default_name, cancel_clause
    );
    let out = run(&cmd);
    out.trim().parse().unwrap_or(0)
}

/// kdialog 消息框。
fn kd_message_box(
    title: &str,
    message: &str,
    dialog_type: MessageBoxType,
    icon: IconType,
    default_button: i32,
) -> i32 {
    let attach = attach_arg();
    let icon_str = kd_icon(icon);
    let defaultno = if default_button == 0 { " --defaultno" } else { "" };

    let (subcmd, is_yesnocancel) = match (dialog_type, icon) {
        (MessageBoxType::YesNoCancel, _) => ("yesnocancel", true),
        (MessageBoxType::YesNo, IconType::Warning) | (MessageBoxType::YesNo, IconType::Error) => ("warningyesno", false),
        (MessageBoxType::OkCancel, IconType::Warning) | (MessageBoxType::OkCancel, IconType::Error) => ("warningyesno", false),
        (MessageBoxType::YesNo, _) => ("yesno", false),
        (MessageBoxType::OkCancel, _) => ("yesno", false),
        (_, IconType::Error) => ("error", false),
        (_, IconType::Warning) => ("sorry", false),
        _ => ("msgbox", false),
    };

    let okcancel_labels = if dialog_type == MessageBoxType::OkCancel {
        " --yes-label Ok --no-label Cancel"
    } else {
        ""
    };

    let cmd = if is_yesnocancel {
        format!(
            "kdialog{} --{} \"{}\" --title \"{}\";x=$?;if [ $x = 0 ];then echo 1;elif [ $x = 1 ];then echo 2;else echo 0;fi",
            attach, subcmd, message.replace('"', "\\\""), title.replace('"', "\\\"")
        )
    } else {
        format!(
            "kdialog{}{} --{} \"{}\" --title \"{}\"{};if [ $? = 0 ];then echo 1;else echo 0;fi",
            attach, defaultno, subcmd, message.replace('"', "\\\""), title.replace('"', "\\\""), okcancel_labels
        )
    };
    let _ = icon_str; // kdialog 自带图标判断。
    let out = run(&cmd);
    out.trim().parse().unwrap_or(0)
}

/// zenity 消息框。
fn zenity_message_box(
    title: &str,
    message: &str,
    dialog_type: MessageBoxType,
    icon: IconType,
) -> i32 {
    let attach = attach_arg();
    let icon_str = zenity_icon(icon);
    let icon_flag = if d::zenity_version() >= 3900 {
        format!("--icon={}", icon_str)
    } else {
        format!("--icon-name={}", icon_str)
    };

    let cmd = match dialog_type {
        MessageBoxType::Ok => format!(
            "zenity{} --info --title=\"{}\" --no-wrap --text=\"{}\" {} 2>/dev/null;echo 1",
            attach, title.replace('"', "\\\""), message.replace('"', "\\\""), icon_flag
        ),
        MessageBoxType::OkCancel => format!(
            "zenity{} --question --ok-label=Ok --cancel-label=Cancel --title=\"{}\" --no-wrap --text=\"{}\" {} 2>/dev/null;if [ $? = 0 ];then echo 1;else echo 0;fi",
            attach, title.replace('"', "\\\""), message.replace('"', "\\\""), icon_flag
        ),
        MessageBoxType::YesNo => format!(
            "zenity{} --question --title=\"{}\" --no-wrap --text=\"{}\" {} 2>/dev/null;if [ $? = 0 ];then echo 1;else echo 0;fi",
            attach, title.replace('"', "\\\""), message.replace('"', "\\\""), icon_flag
        ),
        MessageBoxType::YesNoCancel => format!(
            "szAnswer=$(zenity{} --list --column \"\" --hide-header \"Yes\" \"No\" --title=\"{}\" --text=\"{}\" {} 2>/dev/null);if [ $? = 1 ];then echo 0;elif [ \"$szAnswer\" = \"No\" ];then echo 2;else echo 1;fi",
            attach, title.replace('"', "\\\""), message.replace('"', "\\\""), icon_flag
        ),
    };
    let out = run(&cmd);
    out.trim().parse().unwrap_or(0)
}

/// yad 消息框。
fn yad_message_box(
    title: &str,
    message: &str,
    dialog_type: MessageBoxType,
    icon: IconType,
) -> i32 {
    let icon_str = zenity_icon(icon);
    let buttons = match dialog_type {
        MessageBoxType::Ok => "--button=Ok:1".to_string(),
        MessageBoxType::OkCancel => "--button=Ok:1 --button=Cancel:0".to_string(),
        MessageBoxType::YesNo => "--button=Yes:1 --button=No:0".to_string(),
        MessageBoxType::YesNoCancel => "--button=Yes:1 --button=No:2 --button=Cancel:0".to_string(),
    };
    let cmd = format!(
        "yad {} --title=\"{}\" --text=\"{}\" --image={} 2>/dev/null;echo $?",
        buttons, title.replace('"', "\\\""), message.replace('"', "\\\""), icon_str
    );
    let out = run(&cmd);
    out.trim().parse().unwrap_or(0)
}

/// gxmessage/gmessage/xmessage 消息框。
fn xmessage_message_box(
    title: &str,
    message: &str,
    dialog_type: MessageBoxType,
    _icon: IconType,
    default_button: i32,
) -> i32 {
    let exe = if d::gxmessage_present() {
        "gxmessage"
    } else if d::gmessage_present() {
        "gmessage"
    } else {
        "xmessage"
    };
    let (buttons, default_btn) = match dialog_type {
        MessageBoxType::Ok => ("Ok:1", "Ok"),
        MessageBoxType::OkCancel => ("Ok:1,Cancel:0", if default_button == 0 { "Cancel" } else { "Ok" }),
        MessageBoxType::YesNo => ("Yes:1,No:0", if default_button == 0 { "No" } else { "Yes" }),
        MessageBoxType::YesNoCancel => ("Yes:1,No:2,Cancel:0",
            match default_button { 0 => "No", 1 => "Yes", _ => "Cancel" }),
    };
    let cmd = format!(
        "{} -buttons {} -default {} -center \"{}\" -title \"{}\";echo $?",
        exe, buttons, default_btn, message.replace('"', "\\\""), title.replace('"', "\\\"")
    );
    let out = run(&cmd);
    out.trim().parse().unwrap_or(0)
}

/// tkinter 消息框。
fn tkinter_message_box(python: &str, title: &str, message: &str, dialog_type: MessageBoxType, icon: IconType) -> i32 {
    let (func, default, icon_str) = match (dialog_type, icon) {
        (MessageBoxType::OkCancel, _) => ("askokcancel", "OK", "warning"),
        (MessageBoxType::YesNo, _) => ("askyesno", "YES", "question"),
        (MessageBoxType::YesNoCancel, _) => ("askyesnocancel", "YES", "question"),
        _ => ("showinfo", "OK", "info"),
    };
    let msg_escaped = message.replace('\\', "\\\\").replace('\'', "\\'").replace('\n', "\\n");
    let title_escaped = title.replace('\'', "\\'");
    let cmd = format!(
        "{} -S -c \"import tkinter;from tkinter import messagebox;root=tkinter.Tk();root.withdraw();res=messagebox.{}(default='{}',icon='{}',title='{}',message='{}');\\nif res is False:\\n\\tprint(0)\\nelse:\\n\\tprint(1)\"",
        python, func, default, icon_str, title_escaped, msg_escaped
    );
    let out = run(&cmd);
    match dialog_type {
        MessageBoxType::YesNoCancel => {
            // None→0, False→2, True→1
            match out.trim() {
                "None" => 0,
                "False" => 2,
                _ => 1,
            }
        }
        _ => out.trim().parse().unwrap_or(0),
    }
}

/// 控制台消息框（dialog/whiptail）。
fn console_message_box(
    title: &str,
    message: &str,
    dialog_type: MessageBoxType,
    default_button: i32,
) -> i32 {
    let defaultno = if default_button == 0 { " --defaultno" } else { "" };
    let exe = if d::dialog_present() { "dialog" } else { "whiptail" };
    let backtitle = " --backtitle \"tab: move focus\"";

    let (type_flag, is_menu) = match dialog_type {
        MessageBoxType::Ok => (" --msgbox".to_string(), false),
        MessageBoxType::OkCancel => (
            format!("{} --yes-label Ok --no-label Cancel --yesno", defaultno),
            false,
        ),
        MessageBoxType::YesNo => (format!("{} --yesno", defaultno), false),
        MessageBoxType::YesNoCancel => (" --menu".to_string(), true),
    };

    let title_arg = format!(" --title \"{}\"", title.replace('"', " "));

    let cmd = if is_menu {
        format!(
            "{}{}{} \"{}\" 0 60 0 Yes \"\" No \"\" Cancel \"\"",
            exe, title_arg, backtitle, message.replace('"', " ")
        )
    } else {
        format!(
            "{}{}{}{} \"{}\" 10 60",
            exe, title_arg, backtitle, type_flag, message.replace('"', " ")
        )
    };

    let temp = "/tmp/tinyfd.txt";
    let full_cmd = format!("{} > {} 2>&1", cmd, temp);
    let status = Command::new("sh").arg("-c").arg(&full_cmd).status();
    let exit = status.map(|s| s.code().unwrap_or(-1)).unwrap_or(-1);

    if is_menu {
        let res = std::fs::read_to_string(temp).unwrap_or_default();
        let _ = std::fs::remove_file(temp);
        let res = res.trim();
        match res {
            "Yes" => 1,
            "No" => 2,
            _ => 0,
        }
    } else {
        let _ = std::fs::remove_file(temp);
        if exit == 0 { 1 } else { 0 }
    }
}

/// basic-input 消息框。
fn basic_message_box(message: &str, dialog_type: MessageBoxType) -> i32 {
    use std::io::Write;
    println!("{}", message);
    let _ = std::io::stdout().flush();
    match dialog_type {
        MessageBoxType::Ok => {
            print!("(press enter to continue): ");
            let _ = std::io::stdout().flush();
            let mut line = String::new();
            let _ = std::io::stdin().read_line(&mut line);
            1
        }
        MessageBoxType::OkCancel => {
            print!("[O]kay/[C]ancel: ");
            let _ = std::io::stdout().flush();
            loop {
                let mut line = String::new();
                let _ = std::io::stdin().read_line(&mut line);
                let c = line.trim().chars().next().unwrap_or('\n').to_ascii_lowercase();
                if c == 'o' { return 1; }
                if c == 'c' { return 0; }
            }
        }
        MessageBoxType::YesNo => {
            print!("y/n: ");
            let _ = std::io::stdout().flush();
            loop {
                let mut line = String::new();
                let _ = std::io::stdin().read_line(&mut line);
                let c = line.trim().chars().next().unwrap_or('\n').to_ascii_lowercase();
                if c == 'y' { return 1; }
                if c == 'n' { return 0; }
            }
        }
        MessageBoxType::YesNoCancel => {
            print!("[Y]es/[N]o/[C]ancel: ");
            let _ = std::io::stdout().flush();
            loop {
                let mut line = String::new();
                let _ = std::io::stdin().read_line(&mut line);
                let c = line.trim().chars().next().unwrap_or('\n').to_ascii_lowercase();
                if c == 'y' { return 1; }
                if c == 'n' { return 2; }
                if c == 'c' { return 0; }
            }
        }
    }
}

// ============ inputBox ============

/// 输入框 / 密码框。对应 C 版 `tinyfd_inputBox`（Unix）。
pub fn input_box(title: &str, message: &str, default_input: Option<&str>) -> Option<String> {
    let title = safe_string(title, "TITLE");
    let message = safe_string(message, "MESSAGE");
    if let Some(d) = default_input {
        if helpers::quote_detected(Some(d)) {
            return None;
        }
    }

    let cmd = if d::osascript_present() {
        osa_input_box(&title, &message, default_input)
    } else if d::kdialog_present() > 0 {
        kd_input_box(&title, &message, default_input)
    } else if d::zenity_present() {
        zenity_input_box(&title, &message, default_input)
    } else if d::yad_present() {
        yad_input_box(&title, &message, default_input)
    } else if d::tkinter3_present() {
        tkinter_input_box("python3", &title, &message, default_input)
    } else if d::tkinter2_present() {
        tkinter_input_box("python2", &title, &message, default_input)
    } else if d::gxmessage_present() || d::xmessage_present() {
        xmessage_input_box(&title, &message, default_input)
    } else {
        return basic_input_box(&title, &message, default_input);
    };

    let out = run(&cmd);
    // 首字符 1=OK，0=取消。
    out.strip_prefix('1').map(|s| s.to_string())
}

fn osa_input_box(title: &str, message: &str, default_input: Option<&str>) -> String {
    let hidden = if default_input.is_none() { " hidden answer true" } else { "" };
    let default_str = default_input.unwrap_or("");
    format!(
        "osascript -e 'try' -e 'display dialog \"{}\" default answer \"{}\"{} with title \"{}\" with icon note' \
         -e '\"1\" & text returned of result' \
         -e 'on error number -128' -e '0' -e 'end try'",
        message.replace('"', "\\\""),
        default_str.replace('"', "\\\""),
        hidden,
        title.replace('"', "\\\"")
    )
}

fn kd_input_box(title: &str, message: &str, default_input: Option<&str>) -> String {
    let attach = attach_arg();
    let (flag, default_val) = if default_input.is_none() {
        ("--password", "")
    } else {
        ("--inputbox", default_input.unwrap_or(""))
    };
    format!(
        "szAnswer=$(kdialog{} {} \"{}\" \"{}\" --title \"{}\");if [ $? = 0 ];then echo 1$szAnswer;else echo 0$szAnswer;fi",
        attach, flag, message.replace('"', "\\\""), default_val.replace('"', "\\\""), title.replace('"', "\\\"")
    )
}

fn zenity_input_box(title: &str, message: &str, default_input: Option<&str>) -> String {
    let attach = attach_arg();
    let entry = match default_input {
        Some(d) => format!(" --entry-text=\"{}\"", d.replace('"', "\\\"")),
        None => " --hide-text".to_string(),
    };
    format!(
        "szAnswer=$(zenity{} --entry --title=\"{}\" --text=\"{}\"{} 2>/dev/null);if [ $? = 0 ];then echo 1$szAnswer;else echo 0$szAnswer;fi",
        attach, title.replace('"', "\\\""), message.replace('"', "\\\""), entry
    )
}

fn yad_input_box(title: &str, message: &str, default_input: Option<&str>) -> String {
    let entry = match default_input {
        Some(d) => format!(" --entry-text=\"{}\"", d.replace('"', "\\\"")),
        None => " --hide-text".to_string(),
    };
    format!(
        "szAnswer=$(yad --entry --title=\"{}\" --text=\"{}\"{} 2>/dev/null);if [ $? = 0 ];then echo 1$szAnswer;else echo 0$szAnswer;fi",
        title.replace('"', "\\\""), message.replace('"', "\\\""), entry
    )
}

fn xmessage_input_box(title: &str, message: &str, default_input: Option<&str>) -> String {
    let exe = if d::gxmessage_present() { "gxmessage" } else { "xmessage" };
    let default_val = default_input.unwrap_or("");
    format!(
        "szAnswer=$({} -buttons Ok:1,Cancel:0 -center \"{}\" -title \"{}\" -entrytext \"{}\");echo $?$szAnswer",
        exe, message.replace('"', "\\\""), title.replace('"', "\\\""), default_val.replace('"', "\\\"")
    )
}

fn tkinter_input_box(python: &str, title: &str, message: &str, default_input: Option<&str>) -> String {
    let (initial, show) = match default_input {
        Some(d) => (d, ""),
        None => ("", ",show='*'"),
    };
    let msg_esc = message.replace('\\', "\\\\").replace('\'', "\\'").replace('\n', "\\n");
    let title_esc = title.replace('\'', "\\'");
    let init_esc = initial.replace('\'', "\\'");
    format!(
        "{} -S -c \"import tkinter;from tkinter import simpledialog;root=tkinter.Tk();root.withdraw();res=simpledialog.askstring(title='{}',prompt='{}',initialvalue='{}'{});if res is None:print(0)else:print('1'+res)\"",
        python, title_esc, msg_esc, init_esc, show
    )
}

/// basic-input 输入框。
fn basic_input_box(title: &str, message: &str, default_input: Option<&str>) -> Option<String> {
    use std::io::Write;
    if !title.is_empty() {
        println!("{}", title);
    }
    if !message.is_empty() {
        println!("{}", message);
    }
    let default = default_input.unwrap_or("");
    if !default.is_empty() {
        println!("{}", default);
    }
    print!("(esc+enter to cancel): ");
    let _ = std::io::stdout().flush();
    let mut line = String::new();
    let _ = std::io::stdin().read_line(&mut line);
    let line = line.trim_end_matches(['\r', '\n']);
    if line.contains('\u{1b}') {
        return None;
    }
    if line.is_empty() {
        // 空输入时返回默认值（若有）。
        if default.is_empty() {
            return None;
        }
        return Some(default.to_string());
    }
    Some(line.to_string())
}

// ============ 文件对话框公共 ============

/// 构造过滤器字符串（kdialog 格式：`"*.txt *.doc | desc"`）。
fn kd_filter(patterns: &[&str], description: &str) -> String {
    if patterns.is_empty() {
        return String::new();
    }
    let pats = patterns.join(" ");
    if description.is_empty() {
        format!("\"{}\"", pats)
    } else {
        format!("\"{} | {}\"", pats, description)
    }
}

/// 构造 zenity/yad 过滤器（`'desc | *.txt *.doc'`，可叠加多个）。
fn zenity_filter(patterns: &[&str], description: &str) -> Vec<String> {
    let mut filters = Vec::new();
    if !patterns.is_empty() {
        let desc = if description.is_empty() {
            patterns.join(" ")
        } else {
            description.to_string()
        };
        filters.push(format!("'{} | {}'", desc, patterns.join(" ")));
    }
    filters.push("'All files | *'".to_string());
    filters
}

/// 规范化默认路径：非绝对路径时前缀加 lLastDirectory。
fn normalize_path(path: &str) -> String {
    if path.is_empty() {
        return String::new();
    }
    if path.starts_with('/') || path.starts_with('~') {
        return path.to_string();
    }
    let dir = last_dir().lock().unwrap().clone();
    if dir == "$PWD" {
        format!("./{}", path)
    } else {
        format!("{}/{}", dir, path)
    }
}

// ============ saveFileDialog ============

/// 保存文件对话框。对应 C 版 `tinyfd_saveFileDialog`（Unix）。
pub fn save_file_dialog(
    title: &str,
    default_path: &str,
    filter_patterns: &[&str],
    filter_description: &str,
) -> Option<String> {
    let title = safe_string(title, "TITLE");
    let path = normalize_path(default_path);

    let cmd = if d::osascript_present() {
        let dir = helpers::path_without_final_slash(&path);
        let name = helpers::last_name(&path);
        let loc = if dir.is_empty() { String::new() } else { format!(" default location \"{}\"", dir) };
        let nm = if name.is_empty() { String::new() } else { format!(" default name \"{}\"", name) };
        format!(
            "osascript -e 'try' -e 'POSIX path of (choose file name with prompt \"{}\"{}{})' -e 'on error number -128' -e 'end try'",
            title.replace('"', "\\\""), loc, nm
        )
    } else if d::kdialog_present() > 0 {
        let attach = attach_arg();
        let filter = kd_filter(filter_patterns, filter_description);
        let filter_arg = if filter.is_empty() { String::new() } else { format!(" {}", filter) };
        format!(
            "kdialog{} --getsavefilename \"{}\"{} --title \"{}\"",
            attach, path, filter_arg, title.replace('"', "\\\"")
        )
    } else if d::zenity_present() {
        let attach = attach_arg();
        let filters = zenity_filter(filter_patterns, filter_description).join(" ");
        format!(
            "zenity{} --file-selection --save --confirm-overwrite --title=\"{}\" --filename=\"{}\" {} 2>/dev/null",
            attach, title.replace('"', "\\\""), path, filters
        )
    } else if d::yad_present() {
        let filters = zenity_filter(filter_patterns, filter_description).join(" ");
        format!(
            "yad --file --save --confirm-overwrite --title=\"{}\" --filename=\"{}\" {} 2>/dev/null",
            title.replace('"', "\\\""), path, filters
        )
    } else if d::tkinter3_present() {
        tkinter_file_dialog("python3", "asksaveasfilename", &title, &path, filter_patterns, filter_description, false)
    } else if d::tkinter2_present() {
        tkinter_file_dialog("python2", "asksaveasfilename", &title, &path, filter_patterns, filter_description, false)
    } else {
        // 回退：inputBox 手输路径。
        let msg = format!("Save file in {}", helpers::current_dir());
        let input = input_box(&title, &msg, Some(""))?;
        let dir = helpers::path_without_final_slash(&input);
        let name = helpers::last_name(&input);
        if !helpers::dir_exists(dir) || !helpers::filename_valid(name) {
            return None;
        }
        return Some(input);
    };

    let result = run(&cmd);
    if result.is_empty() {
        return None;
    }
    // 验证目录和文件名。
    let dir = helpers::path_without_final_slash(&result);
    if !dir.is_empty() && !helpers::dir_exists(dir) {
        return None;
    }
    let name = helpers::last_name(&result);
    if !helpers::filename_valid(name) {
        return None;
    }
    // 记住最后目录。
    if !dir.is_empty() {
        *last_dir().lock().unwrap() = dir.to_string();
    }
    Some(result)
}

// ============ openFileDialog ============

/// 打开文件对话框。对应 C 版 `tinyfd_openFileDialog`（Unix）。
pub fn open_file_dialog(
    title: &str,
    default_path: &str,
    filter_patterns: &[&str],
    filter_description: &str,
    allow_multiple: bool,
) -> Option<String> {
    let title = safe_string(title, "TITLE");
    let path = normalize_path(default_path);

    let cmd = if d::osascript_present() {
        osa_open_file(&title, &path, filter_patterns, allow_multiple)
    } else if d::kdialog_present() > 0 {
        let attach = attach_arg();
        let filter = kd_filter(filter_patterns, filter_description);
        let filter_arg = if filter.is_empty() { String::new() } else { format!(" {}", filter) };
        let multi = if allow_multiple { " --multiple --separate-output" } else { "" };
        format!(
            "kdialog{} --getopenfilename \"{}\"{}{} --title \"{}\"",
            attach, path, filter_arg, multi, title.replace('"', "\\\"")
        )
    } else if d::zenity_present() {
        let attach = attach_arg();
        let filters = zenity_filter(filter_patterns, filter_description).join(" ");
        let multi = if allow_multiple { " --multiple" } else { "" };
        format!(
            "zenity{} --file-selection{} --title=\"{}\" --filename=\"{}\" {} 2>/dev/null",
            attach, multi, title.replace('"', "\\\""), path, filters
        )
    } else if d::yad_present() {
        let filters = zenity_filter(filter_patterns, filter_description).join(" ");
        let multi = if allow_multiple { " --multiple" } else { "" };
        format!(
            "yad --file{} --title=\"{}\" --filename=\"{}\" {} 2>/dev/null",
            multi, title.replace('"', "\\\""), path, filters
        )
    } else if d::tkinter3_present() {
        tkinter_file_dialog("python3", "askopenfilename", &title, &path, filter_patterns, filter_description, allow_multiple)
    } else if d::tkinter2_present() {
        tkinter_file_dialog("python2", "askopenfilename", &title, &path, filter_patterns, filter_description, allow_multiple)
    } else {
        // 回退：inputBox 手输路径。
        let msg = format!("Open file from {}", helpers::current_dir());
        let input = input_box(&title, &msg, Some(""))?;
        if !helpers::file_exists(&input) {
            return None;
        }
        return Some(input);
    };

    let result = run(&cmd);
    if result.is_empty() {
        return None;
    }

    // kdialog 多选输出每行一个文件，转为 | 分隔。
    let result = if allow_multiple && d::kdialog_present() > 0 {
        result.replace('\n', "|")
    } else {
        result
    };

    // 验证文件存在性。
    if allow_multiple && result.contains('|') {
        let filtered = helpers::ensure_files_exist(&result);
        if filtered.is_empty() {
            return None;
        }
        // 记住最后目录（取最后一个文件的目录）。
        if let Some(last) = filtered.split('|').next_back() {
            let dir = helpers::path_without_final_slash(last);
            if !dir.is_empty() {
                *last_dir().lock().unwrap() = dir.to_string();
            }
        }
        Some(filtered)
    } else {
        if !helpers::file_exists(&result) {
            return None;
        }
        let dir = helpers::path_without_final_slash(&result);
        if !dir.is_empty() {
            *last_dir().lock().unwrap() = dir.to_string();
        }
        Some(result)
    }
}

/// osascript 打开文件对话框。
fn osa_open_file(title: &str, path: &str, filter_patterns: &[&str], allow_multiple: bool) -> String {
    // 过滤器取扩展名（跳过 *.）。
    let exts: Vec<String> = filter_patterns
        .iter()
        .filter_map(|p| p.strip_prefix("*."))
        .map(|s| format!("\"{}\"", s))
        .collect();
    let of_type = if exts.is_empty() {
        String::new()
    } else {
        format!(" of type {{{}}}", exts.join(", "))
    };
    let dir = helpers::path_without_final_slash(path);
    let loc = if dir.is_empty() { String::new() } else { format!(" default location \"{}\"", dir) };

    if allow_multiple {
        // 多选：循环拼接，| 分隔。
        format!(
            "osascript -e 'try' -e 'set mylist to choose file with prompt \"{}\"{}{} multiple selections allowed true' \
             -e 'set mystring to \"\"' -e 'repeat with i from 1 to the count of mylist' \
             -e 'set mystring to mystring & POSIX path of item i of mylist & \"|\"' -e 'end repeat' \
             -e 'mystring' -e 'on error number -128' -e 'end try'",
            title.replace('"', "\\\""), loc, of_type
        )
    } else {
        format!(
            "osascript -e 'try' -e 'POSIX path of (choose file with prompt \"{}\"{}{})' -e 'on error number -128' -e 'end try'",
            title.replace('"', "\\\""), loc, of_type
        )
    }
}

/// tkinter 文件对话框。
fn tkinter_file_dialog(
    python: &str,
    func: &str,
    title: &str,
    path: &str,
    filter_patterns: &[&str],
    filter_description: &str,
    allow_multiple: bool,
) -> String {
    let dir = helpers::path_without_final_slash(path);
    let file = helpers::last_name(path);
    let title_esc = title.replace('\'', "\\'");
    let dir_esc = dir.replace('\'', "\\'");
    let file_esc = file.replace('\'', "\\'");

    // filetypes 参数。
    let mut filetypes = String::new();
    if !filter_patterns.is_empty() {
        let pats: Vec<String> = filter_patterns.iter().map(|p| format!("'{}'", p)).collect();
        let desc = if filter_description.is_empty() { "files" } else { filter_description };
        filetypes = format!(",filetypes=(('{}',({})),('All files','*'))", desc, pats.join(","));
    }

    let initialdir = if dir_esc.is_empty() { String::new() } else { format!(",initialdir='{}'", dir_esc) };
    let initialfile = if file_esc.is_empty() { String::new() } else { format!(",initialfile='{}'", file_esc) };

    if func == "askopenfilename" && allow_multiple {
        format!(
            "{} -S -c \"import tkinter;from tkinter import filedialog;root=tkinter.Tk();root.withdraw();lFiles=filedialog.askopenfilename(multiple=1,title='{}'{}{}{});if not isinstance(lFiles, tuple):print(lFiles)else:lFilesString='';for lFile in lFiles:lFilesString+=str(lFile)+'|';print(lFilesString[:-1])\"",
            python, title_esc, initialdir, initialfile, filetypes
        )
    } else {
        format!(
            "{} -S -c \"import tkinter;from tkinter import filedialog;root=tkinter.Tk();root.withdraw();res=filedialog.{}(title='{}'{}{}{});if not isinstance(res, tuple):print(res)\"",
            python, func, title_esc, initialdir, initialfile, filetypes
        )
    }
}

// ============ selectFolderDialog ============

/// 选择文件夹对话框。对应 C 版 `tinyfd_selectFolderDialog`（Unix）。
pub fn select_folder_dialog(title: &str, default_path: &str) -> Option<String> {
    let title = safe_string(title, "TITLE");
    let path = normalize_path(default_path);

    let cmd = if d::osascript_present() {
        let loc = if path.is_empty() { String::new() } else { format!(" default location \"{}\"", path) };
        format!(
            "osascript -e 'try' -e 'POSIX path of (choose folder with prompt \"{}\"{})' -e 'on error number -128' -e 'end try'",
            title.replace('"', "\\\""), loc
        )
    } else if d::kdialog_present() > 0 {
        let attach = attach_arg();
        format!(
            "kdialog{} --getexistingdirectory \"{}\" --title \"{}\"",
            attach, path, title.replace('"', "\\\"")
        )
    } else if d::zenity_present() {
        let attach = attach_arg();
        format!(
            "zenity{} --file-selection --directory --title=\"{}\" --filename=\"{}\" 2>/dev/null",
            attach, title.replace('"', "\\\""), path
        )
    } else if d::yad_present() {
        format!(
            "yad --file --directory --title=\"{}\" --filename=\"{}\" 2>/dev/null",
            title.replace('"', "\\\""), path
        )
    } else if d::tkinter3_present() {
        let title_esc = title.replace('\'', "\\'");
        let dir_esc = path.replace('\'', "\\'");
        format!(
            "python3 -S -c \"import tkinter;from tkinter import filedialog;root=tkinter.Tk();root.withdraw();res=filedialog.askdirectory(title='{}',initialdir='{}');if not isinstance(res, tuple):print(res)\"",
            title_esc, dir_esc
        )
    } else if d::tkinter2_present() {
        let title_esc = title.replace('\'', "\\'");
        let dir_esc = path.replace('\'', "\\'");
        format!(
            "python2 -S -c \"import Tkinter;import tkFileDialog;root=Tkinter.Tk();root.withdraw();res=tkFileDialog.askdirectory(title='{}',initialdir='{}');if not isinstance(res, tuple):print res\"",
            title_esc, dir_esc
        )
    } else {
        // 回退：inputBox 手输路径。
        let msg = format!("Select folder from {}", helpers::current_dir());
        let input = input_box(&title, &msg, Some(""))?;
        if !helpers::dir_exists(&input) {
            return None;
        }
        return Some(input);
    };

    let result = run(&cmd);
    if result.is_empty() || !helpers::dir_exists(&result) {
        return None;
    }
    let dir = helpers::path_without_final_slash(&result);
    if !dir.is_empty() {
        *last_dir().lock().unwrap() = dir.to_string();
    }
    Some(result)
}

// ============ colorChooser ============

/// 颜色选择器。对应 C 版 `tinyfd_colorChooser`（Unix）。
pub fn color_chooser(
    title: &str,
    default_hex: Option<&str>,
    default_rgb: Option<RgbColor>,
) -> Option<(String, RgbColor)> {
    let title = safe_string(title, "TITLE");
    let default = match default_hex {
        Some(hex) => RgbColor::from_hex(hex).unwrap_or(default_rgb.unwrap_or_default()),
        None => default_rgb.unwrap_or_default(),
    };
    let hex = default.to_hex();

    let cmd = if d::osascript_present() {
        // osascript 输出 "R G B"（0-255）。
        format!(
            "osascript -e 'try' -e 'tell app (path to frontmost application as Unicode text) to set mycolor to choose color default color {{{}, {}, {}}}' \
             -e 'set mystring to ((item 1 of mycolor) div 256 as integer) as string' \
             -e 'repeat with i from 2 to the count of mycolor' \
             -e 'set mystring to mystring & \" \" & ((item i of mycolor) div 256 as integer) as string' \
             -e 'end repeat' -e 'mystring' -e 'on error number -128' -e 'end try'",
            default.r as i32 * 256, default.g as i32 * 256, default.b as i32 * 256
        )
    } else if d::kdialog_present() > 0 {
        let attach = attach_arg();
        format!("kdialog{} --getcolor --default '{}' --title \"{}\"", attach, hex, title.replace('"', "\\\""))
    } else if d::zenity_present() {
        let attach = attach_arg();
        format!(
            "zenity{} --color-selection --show-palette --color={} --title=\"{}\" 2>/dev/null",
            attach, hex, title.replace('"', "\\\"")
        )
    } else if d::yad_present() {
        format!("yad --color --init-color={} --title=\"{}\" 2>/dev/null", hex, title.replace('"', "\\\""))
    } else if d::tkinter3_present() {
        let title_esc = title.replace('\'', "\\'");
        format!(
            "python3 -S -c \"import tkinter;from tkinter import colorchooser;root=tkinter.Tk();root.withdraw();res=colorchooser.askcolor(color='{}',title='{}');if res[1] is not None:print(res[1])\"",
            hex, title_esc
        )
    } else {
        // 回退：inputBox 手输。
        let input = input_box(&title, "Enter hex rgb color (i.e. #f5ca20)", Some(&hex))?;
        let rgb = RgbColor::from_hex(&input)?;
        return Some((rgb.to_hex(), rgb));
    };

    let result = run(&cmd);
    if result.is_empty() {
        return None;
    }

    // 解析颜色输出。
    let rgb = if result.starts_with('#') {
        // zenity/yad/tkinter 输出 #RRGGBB 或 #RRGGBBAA。
        let hex_str = if result.len() > 7 {
            // #RRGGBBAA → #RRGGBB。
            format!("#{}{}{}", &result[1..3], &result[3..5], &result[5..7])
        } else {
            result.clone()
        };
        RgbColor::from_hex(&hex_str)?
    } else if result.starts_with("rgb(") || result.starts_with("rgba(") {
        // 解析 "rgb(R, G, B)" 或 "rgba(R, G, B, A)"。
        let inner = result
            .trim_start_matches("rgb(")
            .trim_start_matches("rgba(")
            .trim_end_matches(')');
        let parts: Vec<&str> = inner.split(',').collect();
        if parts.len() >= 3 {
            RgbColor::new(
                parts[0].trim().parse().ok()?,
                parts[1].trim().parse().ok()?,
                parts[2].trim().parse().ok()?,
            )
        } else {
            return None;
        }
    } else {
        // osascript/xdialog 输出 "R G B"。
        let parts: Vec<&str> = result.split_whitespace().collect();
        if parts.len() >= 3 {
            RgbColor::new(
                parts[0].parse().ok()?,
                parts[1].parse().ok()?,
                parts[2].parse().ok()?,
            )
        } else {
            return None;
        }
    };

    Some((rgb.to_hex(), rgb))
}

// ============ 查询模式 ============

pub fn query_notify_popup(_icon: IconType) -> i32 {
    let backend = if d::osascript_present() {
        "applescript"
    } else if d::kdialog_present() == 2 {
        "kdialog"
    } else if d::yad_present() {
        "yad"
    } else if d::notifysend_present() {
        "notifysend"
    } else if d::zenity_version() >= 3018 {
        "zenity3"
    } else if d::zenity_present() {
        "zenity"
    } else {
        "no_solution"
    };
    state::set_response(backend);
    if backend == "no_solution" { 0 } else { 1 }
}

pub fn query_message_box(_icon: IconType) -> i32 {
    let backend = if d::osascript_present() {
        "applescript"
    } else if d::kdialog_present() > 0 {
        "kdialog"
    } else if d::zenity_present() {
        "zenity"
    } else if d::yad_present() {
        "yad"
    } else if d::tkinter3_present() {
        "python3-tkinter"
    } else if d::tkinter2_present() {
        "python2-tkinter"
    } else if d::dialog_present() {
        "dialog"
    } else if d::whiptail_present() {
        "whiptail"
    } else {
        "basicinput"
    };
    state::set_response(backend);
    if backend == "basicinput" { 0 } else { 1 }
}

pub fn query_input_box() -> i32 {
    query_message_box(IconType::Info)
}

pub fn query_save_file_dialog() -> i32 {
    query_message_box(IconType::Info)
}

pub fn query_open_file_dialog() -> i32 {
    query_message_box(IconType::Info)
}

pub fn query_select_folder_dialog() -> i32 {
    query_message_box(IconType::Info)
}

pub fn query_color_chooser() -> i32 {
    query_message_box(IconType::Info)
}
