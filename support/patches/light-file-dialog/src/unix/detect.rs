//! Unix 后端检测：检测系统中可用的图形对话框工具。
//!
//! 对应 C 版 `tinyfiledialogs.c` 第 3453-4440 行的各 `*Present` 函数。
//! 所有检测结果用 `OnceLock` 缓存，避免重复调用 `command -v`。

use std::process::Command;
use std::sync::OnceLock;

/// `CREATE_NO_WINDOW` 的 Unix 等价物：不需要。
const SH: &str = "sh";

/// 检测一个可执行命令是否存在。对应 C 版 `detectPresence`。
///
/// 通过 `command -v <exe> 2>/dev/null` 检测，输出非空且不含冒号、
/// 不以 "no " 开头则认为存在。
pub fn detect_presence(exe: &str) -> bool {
    let cmd = format!("command -v {} 2>/dev/null", exe);
    let output = Command::new(SH)
        .arg("-c")
        .arg(&cmd)
        .output();
    match output {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            let trimmed = stdout.trim();
            if trimmed.is_empty() {
                return false;
            }
            // 排除 "no xxx in ..." 之类的报错和含冒号的 shell 函数定义。
            !trimmed.contains(':') && !trimmed.starts_with("no ")
        }
        Err(_) => false,
    }
}

/// 执行命令并检查是否无输出（用于 python import 检测）。对应 C 版 `tryCommand`。
fn try_command(cmd: &str) -> bool {
    let output = Command::new(SH)
        .arg("-c")
        .arg(cmd)
        .output();
    match output {
        Ok(o) => o.stdout.is_empty(),
        Err(_) => false,
    }
}

/// 检测 DISPLAY 或 WAYLAND_DISPLAY 环境变量。对应 C 版 `getenvDISPLAY`。
pub fn has_display() -> bool {
    std::env::var("DISPLAY").is_ok() || std::env::var("WAYLAND_DISPLAY").is_ok()
}

/// 是否在 SSH 会话中。
pub fn in_ssh() -> bool {
    std::env::var("SSH_TTY").is_ok() || std::env::var("SSH_CLIENT").is_ok()
}

/// 是否在终端中运行。对应 C 版 `isTerminalRunning`。
#[allow(dead_code)]
pub fn is_terminal() -> bool {
    // isatty(stdout) 的等价检查
    #[cfg(unix)]
    {
        unsafe { libc_isatty(1) }
    }
    #[cfg(not(unix))]
    {
        false
    }
}

#[cfg(unix)]
#[allow(dead_code)]
unsafe extern "C" {
    fn isatty(fd: i32) -> i32;
}

#[cfg(unix)]
#[allow(dead_code)]
unsafe fn libc_isatty(fd: i32) -> bool {
    isatty(fd) != 0
}

/// 判断是否使用图形模式。对应 C 版 `graphicMode`。
pub fn graphic_mode() -> bool {
    static CACHED: OnceLock<bool> = OnceLock::new();
    *CACHED.get_or_init(|| {
        if !has_display() {
            // 无 DISPLAY 时，macOS/Haiku 默认有图形（除非 SSH_TTY）。
            if cfg!(target_os = "macos") || cfg!(target_os = "haiku") {
                return !in_ssh();
            }
            return false;
        }
        true
    })
}

// ============ 后端检测函数 ============
// 每个函数用 OnceLock 缓存，首次调用时检测。

macro_rules! present_fn {
    ($name:ident, $exe:expr) => {
        #[allow(dead_code)]
        pub fn $name() -> bool {
            static CACHED: OnceLock<bool> = OnceLock::new();
            *CACHED.get_or_init(|| detect_presence($exe) && graphic_mode())
        }
    };
}

present_fn!(xmessage_present, "xmessage");
present_fn!(gxmessage_present, "gxmessage");
present_fn!(gmessage_present, "gmessage");
present_fn!(notifysend_present, "notify-send");
present_fn!(xdialog_present, "Xdialog");
present_fn!(gdialog_present, "gdialog");

/// osascript（macOS）。SSH_TTY 下不可用。
pub fn osascript_present() -> bool {
    static CACHED: OnceLock<bool> = OnceLock::new();
    *CACHED.get_or_init(|| {
        cfg!(target_os = "macos")
            && detect_presence("osascript")
            && graphic_mode()
            && !in_ssh()
    })
}

/// afplay（macOS 声音播放）。
pub fn afplay_present() -> bool {
    static CACHED: OnceLock<bool> = OnceLock::new();
    *CACHED.get_or_init(|| detect_presence("afplay"))
}

/// yad。
pub fn yad_present() -> bool {
    static CACHED: OnceLock<bool> = OnceLock::new();
    *CACHED.get_or_init(|| detect_presence("yad") && graphic_mode())
}

/// zenity。
pub fn zenity_present() -> bool {
    static CACHED: OnceLock<bool> = OnceLock::new();
    *CACHED.get_or_init(|| detect_presence("zenity") && graphic_mode())
}

/// zenity 版本号（主版本*1000 + 次版本）。对应 C 版 `tfd_zenityVersion`。
pub fn zenity_version() -> i32 {
    static CACHED: OnceLock<i32> = OnceLock::new();
    *CACHED.get_or_init(|| {
        if !zenity_present() {
            return 0;
        }
        let output = Command::new(SH)
            .arg("-c")
            .arg("zenity --version")
            .output();
        match output {
            Ok(o) => {
                let v = String::from_utf8_lossy(&o.stdout);
                parse_version(&v)
            }
            Err(_) => 0,
        }
    })
}

/// 解析 "主.次[.补丁]" 为 `主*1000 + 次`。
fn parse_version(s: &str) -> i32 {
    let parts: Vec<&str> = s.trim().split('.').collect();
    if parts.len() < 2 {
        return 0;
    }
    let major: i32 = parts[0].parse().unwrap_or(0);
    let minor: i32 = parts[1].parse().unwrap_or(0);
    major * 1000 + minor
}

/// kdialog。返回 0=不存在，2=支持 attach+passivepopup，1=基础版。
/// 对应 C 版 `tfd_kdialogPresent`。
pub fn kdialog_present() -> i32 {
    static CACHED: OnceLock<i32> = OnceLock::new();
    *CACHED.get_or_init(|| {
        if !detect_presence("kdialog") || !graphic_mode() {
            return 0;
        }
        // 若非 KDE 桌面且 zenity 存在，让位 zenity。
        let desktop = std::env::var("XDG_SESSION_DESKTOP").unwrap_or_default();
        if !desktop.is_empty()
            && desktop != "KDE"
            && desktop != "lxqt"
            && zenity_present()
        {
            return 0;
        }
        // 检测 --attach 和 --passivepopup 支持。
        let mut level = 1;
        let out = Command::new(SH)
            .arg("-c")
            .arg("kdialog --help 2>&1")
            .output();
        if let Ok(o) = out {
            let help = format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr));
            if help.contains("--attach") && help.contains("--passivepopup") {
                level = 2;
            }
        }
        level
    })
}

/// xprop（用于 zenity/kdialog 的 --attach）。对应 C 版 `tfd_xpropPresent`。
pub fn xprop_present() -> bool {
    static CACHED: OnceLock<bool> = OnceLock::new();
    *CACHED.get_or_init(|| {
        if !detect_presence("xprop") {
            return false;
        }
        // 验证 xprop 可用（xwayland 兼容性）。
        let out = Command::new(SH)
            .arg("-c")
            .arg("xprop -root 32x '\\t$0' _NET_ACTIVE_WINDOW 2>/dev/null")
            .output();
        match out {
            Ok(o) => !o.stdout.is_empty(),
            Err(_) => false,
        }
    })
}

/// python3。
pub fn python3_present() -> bool {
    static CACHED: OnceLock<bool> = OnceLock::new();
    *CACHED.get_or_init(|| detect_presence("python3"))
}

/// python2。
pub fn python2_present() -> bool {
    static CACHED: OnceLock<bool> = OnceLock::new();
    *CACHED.get_or_init(|| detect_presence("python2"))
}

/// python3 + tkinter。对应 C 版 `tkinter3Present`。
pub fn tkinter3_present() -> bool {
    static CACHED: OnceLock<bool> = OnceLock::new();
    *CACHED.get_or_init(|| {
        if !python3_present() || !graphic_mode() {
            return false;
        }
        if cfg!(target_os = "macos") && in_ssh() {
            return false;
        }
        try_command("python3 -S -c \"try:\n\timport tkinter;\nexcept:\n\tprint(0);\"")
    })
}

/// python2 + Tkinter。对应 C 版 `tkinter2Present`。
pub fn tkinter2_present() -> bool {
    static CACHED: OnceLock<bool> = OnceLock::new();
    *CACHED.get_or_init(|| {
        if !python2_present() || !graphic_mode() {
            return false;
        }
        if cfg!(target_os = "macos") && in_ssh() {
            return false;
        }
        try_command("python2 -S -c \"try:\n\timport Tkinter;\nexcept:\n\tprint 0;\"")
    })
}

/// pactl（PulseAudio）。
pub fn pactl_present() -> bool {
    static CACHED: OnceLock<bool> = OnceLock::new();
    *CACHED.get_or_init(|| detect_presence("pactl"))
}

/// speaker-test。
pub fn speakertest_present() -> bool {
    static CACHED: OnceLock<bool> = OnceLock::new();
    *CACHED.get_or_init(|| detect_presence("speaker-test"))
}

/// ffplay。
pub fn ffplay_present() -> bool {
    static CACHED: OnceLock<bool> = OnceLock::new();
    *CACHED.get_or_init(|| detect_presence("ffplay"))
}

/// play（sox）。
pub fn play_present() -> bool {
    static CACHED: OnceLock<bool> = OnceLock::new();
    *CACHED.get_or_init(|| detect_presence("play"))
}

/// dunstify。
pub fn dunstify_present() -> bool {
    static CACHED: OnceLock<bool> = OnceLock::new();
    *CACHED.get_or_init(|| {
        if !detect_presence("dunstify") {
            return false;
        }
        let out = Command::new(SH)
            .arg("-c")
            .arg("dunstify -s 2>/dev/null")
            .output();
        match out {
            Ok(o) => String::from_utf8_lossy(&o.stdout).contains("name:dunst"),
            Err(_) => false,
        }
    })
}

/// dunst（通过 ps 检测）。
pub fn dunst_present() -> bool {
    static CACHED: OnceLock<bool> = OnceLock::new();
    *CACHED.get_or_init(|| {
        let out = Command::new(SH)
            .arg("-c")
            .arg("ps -e 2>/dev/null | grep dunst | grep -v grep")
            .output();
        match out {
            Ok(o) => !o.stdout.is_empty(),
            Err(_) => false,
        }
    })
}

// ============ 控制台模式检测 ============

/// dialog（ncurses）。对应 C 版 `dialogNameOnly` + `dialogName`。
pub fn dialog_present() -> bool {
    if crate::state::allow_curses_dialogs() == 0 {
        return false;
    }
    static CACHED: OnceLock<bool> = OnceLock::new();
    *CACHED.get_or_init(|| {
        // macOS 优先检测 /opt/local/bin/dialog。
        if cfg!(target_os = "macos") && detect_presence("/opt/local/bin/dialog") {
            return true;
        }
        detect_presence("dialog")
    })
}

/// whiptail。
pub fn whiptail_present() -> bool {
    static CACHED: OnceLock<bool> = OnceLock::new();
    *CACHED.get_or_init(|| detect_presence("whiptail"))
}

/// 是否有可用的终端模拟器（用于非 tty 环境下打开终端运行 dialog）。
/// 对应 C 版 `terminalName`。
#[allow(dead_code)]
pub fn terminal_name() -> Option<String> {
    static CACHED: OnceLock<Option<String>> = OnceLock::new();
    CACHED.get_or_init(detect_terminal).clone()
}

#[allow(dead_code)]
fn detect_terminal() -> Option<String> {
    // 选择 shell。
    let has_bash = detect_presence("bash");
    let has_dialog = dialog_present() || whiptail_present();

    // shell 前缀
    let shell = if has_bash {
        "bash -c "
    } else if has_dialog {
        "sh -c "
    } else {
        // 无 bash 也无 dialog，无法开终端。
        return None;
    };

    // 终端优先级列表：(命令名, 参数风格)
    // xterm 用 -e，但有些终端用 -x。
    let terminals: &[(&str, &str)] = &[
        ("xterm", "-e"),
        ("terminator", "-x"),
        ("lxterminal", "-e"),
        ("konsole", "-e"),
        ("kterm", "-e"),
        ("tilix", "-e"),
        ("xfce4-terminal", "-x"),
        ("mate-terminal", "-x"),
        ("Eterm", "-e"),
        ("evilvte", "-e"),
        ("pterm", "-e"),
    ];

    for (term, _) in terminals {
        if detect_presence(term) {
            let cmd = if *term == "xterm" {
                format!(
                    "{} -fa 'DejaVu Sans Mono' -fs 10 -title light-file-dialog -e {}",
                    term, shell
                )
            } else {
                // 用 -e 或 -x。
                let flag = if *term == "terminator"
                    || *term == "xfce4-terminal"
                    || *term == "mate-terminal"
                {
                    "-x"
                } else {
                    "-e"
                };
                format!("{} {} {}", term, flag, shell)
            };
            return Some(cmd);
        }
    }
    None
}

/// macOS ≥ 10.9。对应 C 版 `osx9orBetter`。
#[allow(dead_code)]
pub fn osx9_or_better() -> bool {
    if !cfg!(target_os = "macos") {
        return false;
    }
    static CACHED: OnceLock<bool> = OnceLock::new();
    *CACHED.get_or_init(|| {
        let out = Command::new(SH)
            .arg("-c")
            .arg("sw_vers -productVersion 2>/dev/null")
            .output();
        match out {
            Ok(o) => {
                let v = String::from_utf8_lossy(&o.stdout);
                let parts: Vec<&str> = v.trim().split('.').collect();
                if parts.len() >= 2 {
                    let major: i32 = parts[0].parse().unwrap_or(0);
                    let minor: i32 = parts[1].parse().unwrap_or(0);
                    major * 100 + minor >= 1009
                } else {
                    false
                }
            }
            Err(_) => false,
        }
    })
}
