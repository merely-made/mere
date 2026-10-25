//! Windows GUI 对话框实现（Win32 API）。
//!
//! 对应 C 版 `tinyfiledialogs.c` 第 508-2064 行的 W（wchar_t）版本函数，
//! 以及第 2067-2426 行的 `xxxWinGui` 适配层。
//!
//! Rust 版不再区分 char/wchar_t 双层（因为 Rust 字符串统一为 UTF-8），
//! 直接在函数内部转 UTF-16 调 Win32 API。
//!
//! 与 C 版的差异：notify_popup 和密码框不再依赖 HTA/mshta.exe
//! （mshta 在 Windows 11 上启动后立即退出，无法显示窗口），
//! 改用 Win32 原生 API：通知用 Shell_NotifyIconW 气球通知，
//! 密码框用自建窗口 + Edit(ES_PASSWORD) 控件。

use std::fs;
use std::os::windows::process::CommandExt;
use std::path::Path;
use std::process::Command;

use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM, TRUE};
use windows_sys::Win32::Graphics::Gdi::{
    GetStockObject, GetSysColor, GetSysColorBrush, SetBkColor, SetTextColor, SYSTEM_FONT,
};
use windows_sys::Win32::System::Com::{
    CoInitializeEx, CoTaskMemFree, CoUninitialize, COINIT_APARTMENTTHREADED, COINIT_MULTITHREADED,
};
use windows_sys::Win32::System::Diagnostics::Debug::MessageBeep;
use windows_sys::Win32::System::SystemInformation::OSVERSIONINFOW;
use windows_sys::Win32::UI::Controls::Dialogs::{
    CC_ANYCOLOR, CC_FULLOPEN, CC_RGBINIT, CHOOSECOLORW, ChooseColorW, GetOpenFileNameW,
    GetSaveFileNameW, OFN_ALLOWMULTISELECT, OFN_EXPLORER, OFN_FILEMUSTEXIST, OFN_NOCHANGEDIR,
    OFN_OVERWRITEPROMPT, OFN_PATHMUSTEXIST, OPENFILENAMEW,
};
use windows_sys::Win32::UI::Shell::{
    Shell_NotifyIconW, SHBrowseForFolderW, SHGetPathFromIDListW, BROWSEINFOW, NOTIFYICONDATAW,
    NIF_INFO, NIM_ADD, NIM_DELETE, NIIF_ERROR, NIIF_INFO, NIIF_WARNING,
};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::SetFocus;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetForegroundWindow,
    GetMessageW, GetWindowTextW, GetWindowTextLengthW, MessageBoxW, PostQuitMessage,
    RegisterClassExW, SendMessageW, SetForegroundWindow, SetWindowPos, TranslateMessage,
    CREATESTRUCTW, MSG, WNDCLASSEXW, WS_OVERLAPPED, WS_CAPTION, WS_SYSMENU, WS_VISIBLE, WS_CHILD,
    WS_TABSTOP, WS_EX_DLGMODALFRAME, WS_EX_TOPMOST, WS_EX_TOOLWINDOW, CW_USEDEFAULT, ES_PASSWORD,
    ES_AUTOHSCROLL, MB_DEFBUTTON1, MB_DEFBUTTON2, MB_DEFBUTTON3, MB_ICONERROR, MB_ICONINFORMATION,
    MB_ICONQUESTION, MB_ICONWARNING, MB_OK, MB_OKCANCEL, MB_TOPMOST, MB_YESNO, MB_YESNOCANCEL,
    WM_COMMAND, WM_CREATE, WM_CLOSE, WM_CTLCOLOREDIT, WM_CTLCOLORSTATIC, WM_DESTROY, WM_SETFONT,
    WM_SETFOCUS, IDNO, IDOK, IDYES, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER, SWP_SHOWWINDOW,
};

use crate::helpers;
use crate::types::{IconType, MessageBoxType, RgbColor};
use super::utf;

/// `CREATE_NO_WINDOW` 标志，让子进程不弹出控制台窗口。
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// `BIF_USENEWUI` = `BIF_NEWDIALOGSTYLE | BIF_EDITBOX` = 0x50。
/// windows-sys 0.61 未导出 `BIF_USENEWUI`，这里用等值常量。
const BIF_USENEWUI: u32 = 0x40 | 0x10;

/// `BFFM_INITIALIZED` 消息值。
const BFFM_INITIALIZED: u32 = 1;
/// `BFFM_SELCHANGED` 消息值。
const BFFM_SELCHANGED: u32 = 2;
/// `BFFM_SETSELECTIONW` 消息值。
const BFFM_SETSELECTIONW: u32 = 1127;

/// 获取 Windows 主版本号。对应 C 版 `windowsVersion`。
/// 通过 `RtlGetVersion` 动态加载（避开 manifest 兼容性谎言）。
fn windows_version() -> i32 {
    use windows_sys::Win32::System::LibraryLoader::{GetModuleHandleW, GetProcAddress};

    type RtlGetVersion = unsafe extern "system" fn(*mut OSVERSIONINFOW) -> i32;

    unsafe {
        let ntdll = GetModuleHandleW(to_wide_z("ntdll.dll").as_ptr());
        if !ntdll.is_null() {
            let proc = GetProcAddress(ntdll, c"RtlGetVersion".as_ptr() as *const u8);
            if let Some(func) = proc {
                let func: RtlGetVersion = std::mem::transmute(func);
                let mut info: OSVERSIONINFOW = std::mem::zeroed();
                info.dwOSVersionInfoSize = std::mem::size_of::<OSVERSIONINFOW>() as u32;
                if func(&mut info) == 0 {
                    return info.dwMajorVersion as i32;
                }
            }
        }
    }
    // 回退：有 PowerShell 则假定 Vista+（6），否则 0。
    if powershell_present() { 6 } else { 0 }
}

/// 探测 `powershell.exe` 是否存在。对应 C 版 `powershellPresent`。
fn powershell_present() -> bool {
    static CACHED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *CACHED.get_or_init(|| {
        let out = Command::new("where")
            .arg("powershell.exe")
            .creation_flags(CREATE_NO_WINDOW)
            .output();
        match out {
            Ok(o) => o.status.success() && !o.stdout.is_empty(),
            Err(_) => false,
        }
    })
}

/// 简单的 COM 初始化 RAII 守卫。
struct CoInit {
    ok: bool,
}

impl CoInit {
    /// 多线程初始化（用于文件对话框）。对应 C 版 `CoInitializeEx(NULL, 0)`。
    fn new() -> Self {
        let hr = unsafe { CoInitializeEx(std::ptr::null(), COINIT_MULTITHREADED as u32) };
        // S_OK(0) 或 S_FALSE(1) 都算成功。
        Self { ok: hr == 0 || hr == 1 }
    }

    /// 单元线程初始化（用于文件夹对话框）。对应 C 版 `CoInitializeEx(NULL, COINIT_APARTMENTTHREADED)`。
    fn new_apartment() -> Self {
        let hr = unsafe { CoInitializeEx(std::ptr::null(), COINIT_APARTMENTTHREADED as u32) };
        Self { ok: hr == 0 || hr == 1 }
    }
}

impl Drop for CoInit {
    fn drop(&mut self) {
        if self.ok {
            unsafe { CoUninitialize() };
        }
    }
}

/// 把 `&str` 转成以 `\0` 结尾的 `Vec<u16>`。
fn to_wide_z(s: &str) -> Vec<u16> {
    utf::to_wide_z(s)
}

/// 把 UTF-16 `\0` 结尾的 buffer 转成 `String`。
fn from_wide_z(buf: &[u16]) -> String {
    let len = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
    utf::utf16_to_utf8(&buf[..len])
}

/// 构造 OPENFILENAME 的过滤器字符串。
/// 格式：`描述\0模式1;模式2\0描述\0模式\0...\0\0`。
/// 对应 C 版 saveFileDialogW / openFileDialogW 的过滤器构造。
fn build_filter(patterns: &[&str], description: &str) -> Vec<u16> {
    let mut s = String::new();
    if !description.is_empty() {
        s.push_str(description);
        s.push('\n');
    } else if !patterns.is_empty() {
        // 无描述时，模式本身作为描述。
        s.push_str(&patterns.join(";"));
        s.push('\n');
    }
    if !patterns.is_empty() {
        s.push_str(&patterns.join(";"));
        s.push('\n');
    }
    // 追加 "All Files" 过滤器。
    s.push_str("All Files\n*.*\n");
    // 把 \n 换成 \0。
    let mut filter: Vec<u16> = s.replace('\n', "\0").encode_utf16().collect();
    filter.push(0); // 双 \0 结尾。
    filter.push(0);
    filter
}

/// 发出蜂鸣声。对应 C 版 `tinyfd_beep`（Windows）。
pub fn beep() {
    if windows_version() > 5 {
        // Vista+：用 Beep(440, 300)。
        unsafe extern "system" {
            fn Beep(freq: u32, dur: u32) -> i32;
        }
        unsafe { Beep(440, 300) };
    } else {
        // XP：MessageBeep(MB_OK)。
        unsafe { MessageBeep(0) };
    }
}

/// 消息框。对应 C 版 `tinyfd_messageBoxW`。
///
/// 返回 0=cancel/no，1=ok/yes，2=no（yesnocancel）。
pub fn message_box(
    title: &str,
    message: &str,
    dialog_type: MessageBoxType,
    icon: IconType,
    default_button: i32,
) -> i32 {
    let mut flags = MB_TOPMOST;
    flags |= match icon {
        IconType::Warning => MB_ICONWARNING,
        IconType::Error => MB_ICONERROR,
        IconType::Question => MB_ICONQUESTION,
        IconType::Info => MB_ICONINFORMATION,
    };
    flags |= match dialog_type {
        MessageBoxType::Ok => MB_OK,
        MessageBoxType::OkCancel => MB_OKCANCEL,
        MessageBoxType::YesNo => MB_YESNO,
        MessageBoxType::YesNoCancel => MB_YESNOCANCEL,
    };
    flags |= match default_button {
        2 => MB_DEFBUTTON3,
        1 => MB_DEFBUTTON2,
        _ => MB_DEFBUTTON1,
    };

    let hwnd = unsafe { GetForegroundWindow() };
    let result = unsafe {
        MessageBoxW(
            hwnd,
            to_wide_z(message).as_ptr(),
            to_wide_z(title).as_ptr(),
            flags,
        )
    };
    match result {
        IDOK | IDYES => 1,
        IDNO => 2,
        _ => 0,
    }
}

/// 通知弹窗。对应 C 版 `tinyfd_notifyPopupW`。
///
/// 实现方式：Shell_NotifyIconW 气球通知（Windows 10/11 显示为 Toast）。
/// 不依赖 HTA/mshta.exe（mshta 在 Windows 11 上无法显示窗口）。
pub fn notify_popup(title: &str, message: &str, icon: IconType) -> i32 {
    // 创建隐藏的消息窗口作为托盘图标的宿主。
    let class_name = to_wide_z("tinyfd_notify");
    let wc = WNDCLASSEXW {
        cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
        lpfnWndProc: Some(DefWindowProcW),
        lpszClassName: class_name.as_ptr(),
        hInstance: std::ptr::null_mut(),
        hCursor: std::ptr::null_mut(),
        hbrBackground: std::ptr::null_mut(),
        lpszMenuName: std::ptr::null(),
        style: 0,
        cbClsExtra: 0,
        cbWndExtra: 0,
        hIcon: std::ptr::null_mut(),
        hIconSm: std::ptr::null_mut(),
    };
    let atom = unsafe { RegisterClassExW(&wc) };
    if atom == 0 {
        // 注册失败，退化为消息框。
        message_box(title, message, MessageBoxType::Ok, icon, 0);
        return 0;
    }

    // 创建消息窗口（不可见，仅用于承载托盘图标）。
    // WS_EX_TOOLWINDOW 避免在任务栏显示。
    let hwnd = unsafe {
        CreateWindowExW(
            WS_EX_TOOLWINDOW,
            class_name.as_ptr(),
            to_wide_z("tinyfd").as_ptr(),
            0,
            0, 0, 0, 0,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null(),
        )
    };
    if hwnd.is_null() {
        message_box(title, message, MessageBoxType::Ok, icon, 0);
        return 0;
    }

    // 构造 NOTIFYICONDATAW 并显示气球通知。
    let mut nid: NOTIFYICONDATAW = unsafe { std::mem::zeroed() };
    nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
    nid.hWnd = hwnd;
    nid.uID = 1;
    nid.uFlags = NIF_INFO;
    nid.dwInfoFlags = match icon {
        IconType::Info | IconType::Question => NIIF_INFO,
        IconType::Warning => NIIF_WARNING,
        IconType::Error => NIIF_ERROR,
    };

    // szInfoTitle 最大 64 字符，szInfo 最大 256 字符。
    let title_w = to_wide_z(title);
    let msg_w = to_wide_z(message);
    for (i, &c) in title_w.iter().take(63).enumerate() {
        nid.szInfoTitle[i] = c;
    }
    for (i, &c) in msg_w.iter().take(255).enumerate() {
        nid.szInfo[i] = c;
    }

    let added = unsafe { Shell_NotifyIconW(NIM_ADD, &mut nid) };
    if added == 0 {
        // 气球失败，退化为消息框。
        unsafe { DestroyWindow(hwnd) };
        message_box(title, message, MessageBoxType::Ok, icon, 0);
        return 0;
    }

    // 等待几秒让用户看到通知，然后移除图标。
    // Shell 气球通知由系统控制显示时长，这里等待后清理托盘图标。
    std::thread::sleep(std::time::Duration::from_secs(4));
    unsafe { Shell_NotifyIconW(NIM_DELETE, &mut nid) };
    unsafe { DestroyWindow(hwnd) };
    1
}

/// 输入框 / 密码框。对应 C 版 `tinyfd_inputBoxW`。
///
/// `default_input == None` 表示密码框，走 Win32 原生窗口（Edit + ES_PASSWORD）。
/// `Some(text)` 走 VBScript `InputBox`（cscript.exe，不依赖 mshta）。
pub fn input_box(title: &str, message: &str, default_input: Option<&str>) -> Option<String> {
    if default_input.is_none() {
        // 密码框：Win32 原生窗口。
        return password_dialog(title, message);
    }

    // 普通输入框：VBScript InputBox。
    let temp = std::env::temp_dir();
    let result_path = temp.join("tinyfd.txt");
    let ok = input_box_vbs(&temp, title, message, default_input.unwrap_or(""));

    // 读取结果文件。cscript //U 输出 UTF-16 LE（带 BOM FF FE）。
    if ok {
        if let Ok(bytes) = fs::read(&result_path) {
            let _ = fs::remove_file(&result_path);
            let content = decode_utf16_le(&bytes);
            return parse_input_result(&content);
        }
    }
    let _ = fs::remove_file(&result_path);
    None
}

/// 解析 inputBox 结果：以 "1" 开头表示 OK（返回后续内容），"0" 表示取消。
/// 对应 C 版 fgetws 读一行后的处理：trim 换行，检查首字符。
fn parse_input_result(content: &str) -> Option<String> {
    // C 版 fgetws 只读一行，这里取第一行并 trim 换行符。
    let line = content.lines().next().unwrap_or("");
    let line = line.trim_end_matches(['\r', '\n']);
    // C 版：objFile.Write 1 & txt_input.Value → "1password"
    //       objFile.Write 0 → "0"
    // VBS：WScript.Echo "1" & result → "1result"
    //      WScript.Echo 0 → "0"
    if line.starts_with('1') {
        Some(line[1..].to_string())
    } else {
        None
    }
}

/// 解码 UTF-16 LE 字节（带 BOM FF FE），回退到 UTF-8。
/// 对应 C 版 `ccs=UNICODE` 读取模式。
fn decode_utf16_le(bytes: &[u8]) -> String {
    // UTF-16 LE BOM: FF FE
    if bytes.len() >= 2 && bytes[0] == 0xFF && bytes[1] == 0xFE {
        let u16s: Vec<u16> = bytes[2..]
            .chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .collect();
        // 截掉末尾的 \0
        let u16s: Vec<u16> = u16s.into_iter().take_while(|&c| c != 0).collect();
        return utf::utf16_to_utf8(&u16s);
    }
    // 回退：UTF-8（含 BOM）或 ANSI。
    String::from_utf8_lossy(bytes)
        .trim_start_matches('\u{feff}')
        .to_string()
}

/// 系统颜色 COLOR_WINDOW 的值（Win32 常量，用于窗口背景刷子）。
const COLOR_WINDOW: usize = 5;

/// 子控件 ID。
const ID_EDIT: usize = 101;
const ID_OK: usize = 102;
const ID_CANCEL: usize = 103;

/// 密码框状态（通过 thread_local 传递给 WndProc 回调）。
struct PwdState {
    message: Vec<u16>,
    edit_hwnd: HWND,
    /// `None`=未完成，`Some(None)`=取消，`Some(Some(buf))`=OK
    result: Option<Option<Vec<u16>>>,
}

thread_local! {
    static PWD_STATE: std::cell::RefCell<PwdState> = std::cell::RefCell::new(PwdState {
        message: Vec::new(),
        edit_hwnd: std::ptr::null_mut(),
        result: None,
    });
}

/// 密码框窗口过程函数。
unsafe extern "system" fn password_wndproc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_CREATE => {
            let cs = &*(lparam as *const CREATESTRUCTW);
            let hinst = cs.hInstance;
            let hfont = GetStockObject(SYSTEM_FONT) as isize;

            let msg_wide = PWD_STATE.with(|s| s.borrow().message.clone());

            // 静态文本（消息提示）。
            let hstatic = CreateWindowExW(
                0,
                to_wide_z("STATIC").as_ptr(),
                msg_wide.as_ptr(),
                WS_CHILD | WS_VISIBLE,
                10, 10, 360, 30,
                hwnd, std::ptr::null_mut(), hinst, std::ptr::null(),
            );
            SendMessageW(hstatic, WM_SETFONT, hfont as usize, 1);

            // 密码输入框（ES_PASSWORD 显示圆点）。
            let hedit = CreateWindowExW(
                0,
                to_wide_z("EDIT").as_ptr(),
                to_wide_z("").as_ptr(),
                WS_CHILD | WS_VISIBLE | WS_TABSTOP | (ES_PASSWORD as u32) | (ES_AUTOHSCROLL as u32),
                10, 45, 360, 24,
                hwnd, ID_EDIT as usize as *mut _, hinst, std::ptr::null(),
            );
            SendMessageW(hedit, WM_SETFONT, hfont as usize, 1);
            PWD_STATE.with(|s| s.borrow_mut().edit_hwnd = hedit);

            // OK 按钮。
            let hok = CreateWindowExW(
                0,
                to_wide_z("BUTTON").as_ptr(),
                to_wide_z("OK").as_ptr(),
                WS_CHILD | WS_VISIBLE | WS_TABSTOP,
                200, 80, 80, 28,
                hwnd, ID_OK as usize as *mut _, hinst, std::ptr::null(),
            );
            SendMessageW(hok, WM_SETFONT, hfont as usize, 1);

            // Cancel 按钮。
            let hcancel = CreateWindowExW(
                0,
                to_wide_z("BUTTON").as_ptr(),
                to_wide_z("Cancel").as_ptr(),
                WS_CHILD | WS_VISIBLE | WS_TABSTOP,
                290, 80, 80, 28,
                hwnd, ID_CANCEL as usize as *mut _, hinst, std::ptr::null(),
            );
            SendMessageW(hcancel, WM_SETFONT, hfont as usize, 1);

            0
        }
        WM_COMMAND => {
            let id = wparam & 0xFFFF;
            let code = (wparam >> 16) & 0xFFFF;
            // BN_CLICKED = 0
            if code == 0 {
                match id {
                    ID_OK => {
                        let edit = PWD_STATE.with(|s| s.borrow().edit_hwnd);
                        let len = GetWindowTextLengthW(edit) as usize;
                        let mut buf = vec![0u16; len + 1];
                        GetWindowTextW(edit, buf.as_mut_ptr(), buf.len() as i32);
                        PWD_STATE.with(|s| s.borrow_mut().result = Some(Some(buf)));
                        DestroyWindow(hwnd);
                    }
                    ID_CANCEL => {
                        PWD_STATE.with(|s| s.borrow_mut().result = Some(None));
                        DestroyWindow(hwnd);
                    }
                    _ => {}
                }
            }
            0
        }
        WM_CTLCOLORSTATIC => {
            // STATIC 提示文字：灰色文字 + 窗口色背景，呈现 placeholder 淡化效果。
            let hdc = wparam as *mut std::ffi::c_void;
            let bg = unsafe { GetSysColor(COLOR_WINDOW as i32) };
            unsafe {
                SetTextColor(hdc, 0x808080);
                SetBkColor(hdc, bg);
            }
            unsafe { GetSysColorBrush(COLOR_WINDOW as i32) as isize }
        }
        WM_CTLCOLOREDIT => {
            // Edit 控件：保持默认黑字白底。
            let hdc = wparam as *mut std::ffi::c_void;
            let bg = unsafe { GetSysColor(COLOR_WINDOW as i32) };
            unsafe {
                SetBkColor(hdc, bg);
            }
            unsafe { GetSysColorBrush(COLOR_WINDOW as i32) as isize }
        }
        WM_SETFOCUS => {
            // 窗口获得焦点时，把焦点设到密码输入框。
            let edit = PWD_STATE.with(|s| s.borrow().edit_hwnd);
            if !edit.is_null() {
                unsafe { SetFocus(edit) };
            }
            0
        }
        WM_CLOSE => {
            PWD_STATE.with(|s| s.borrow_mut().result = Some(None));
            DestroyWindow(hwnd);
            0
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            0
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

// A window class belongs to the process, not to an individual prompt.
// Re-registering it returns zero after the first cancellation or answer.
fn password_class_registered() -> bool {
    static REGISTERED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *REGISTERED.get_or_init(|| {
        let class_name = to_wide_z("tinyfd_password");
        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: 0,
            lpfnWndProc: Some(password_wndproc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: std::ptr::null_mut(),
            hIcon: std::ptr::null_mut(),
            hCursor: std::ptr::null_mut(),
            hbrBackground: (COLOR_WINDOW + 1) as isize as _,
            lpszMenuName: std::ptr::null(),
            lpszClassName: class_name.as_ptr(),
            hIconSm: std::ptr::null_mut(),
        };
        let atom = unsafe { RegisterClassExW(&wc) };
        atom != 0
    })
}

/// 密码框：用 Win32 原生窗口 + Edit(ES_PASSWORD) 实现。
/// 不依赖 HTA/mshta.exe。
fn password_dialog(title: &str, message: &str) -> Option<String> {
    PWD_STATE.with(|s| {
        *s.borrow_mut() = PwdState {
            message: to_wide_z(message),
            edit_hwnd: std::ptr::null_mut(),
            result: None,
        };
    });

    if !password_class_registered() {
        return None;
    }
    let class_name = to_wide_z("tinyfd_password");

    let title_w = to_wide_z(title);
    let hwnd = unsafe {
        CreateWindowExW(
            WS_EX_DLGMODALFRAME | WS_EX_TOPMOST,
            class_name.as_ptr(),
            title_w.as_ptr(),
            WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            400,
            160,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null(),
        )
    };
    if hwnd.is_null() {
        return None;
    }
    unsafe {
        // A hidden resident launch can suppress ShowWindow's initial show.
        // Show this explicitly requested prompt without inheriting that state.
        SetWindowPos(
            hwnd, std::ptr::null_mut(), 0, 0, 0, 0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_SHOWWINDOW,
        );
        SetForegroundWindow(hwnd);
        // 焦点由 WM_SETFOCUS 处理转给 Edit 控件。
    }

    let mut msg: MSG = unsafe { std::mem::zeroed() };
    while unsafe { GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) } > 0 {
        unsafe {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }

    PWD_STATE.with(|s| {
        let st = s.borrow();
        match &st.result {
            Some(Some(buf)) => Some(from_wide_z(buf)),
            _ => None,
        }
    })
}

/// VBScript InputBox 实现。对应 C 版 `tinyfd_inputBoxW` 第1422-1442行。
fn input_box_vbs(temp: &Path, title: &str, message: &str, default: &str) -> bool {
    let vbs_path = temp.join("tinyfd.vbs");
    let txt_path = temp.join("tinyfd.txt");

    // VBScript 字符串转义：双引号用 "" 转义。
    let esc = |s: &str| s.replace('"', "\"\"");
    // C 版：message 和 default 中的 \n 替换为空格（InputBox 不支持换行）。
    let message_no_nl = message.replace('\n', " ");
    let default_no_nl = default.replace('\n', " ");

    // C 版单行冒号分隔格式：
    // Dim result:result=InputBox("msg","title","default"):If IsEmpty(result) then:WScript.Echo 0:Else: WScript.Echo "1" & result : End If
    // 注意：取消时 Echo 0（数字），OK 时 Echo "1" & result（字符串拼接）。
    let vbs = format!(
        "Dim result:result=InputBox(\"{}\",\"{}\",\"{}\"):If IsEmpty(result) then:WScript.Echo 0:Else: WScript.Echo \"1\" & result : End If",
        esc(&message_no_nl), esc(title), esc(&default_no_nl)
    );

    // VBS 文件带 UTF-8 BOM，让 cscript 以 UTF-8 解析。
    if write_utf8_bom(&vbs_path, &vbs).is_err() {
        return false;
    }

    // //U 让 cscript stdout 输出 UTF-16 LE（对齐 C 版），Rust 捕获后写入 txt。
    let output = Command::new("cscript.exe")
        .arg("//U")
        .arg("//Nologo")
        .arg(&vbs_path)
        .creation_flags(CREATE_NO_WINDOW)
        .output();
    let _ = fs::remove_file(&vbs_path);

    match output {
        Ok(o) => {
            if fs::write(&txt_path, &o.stdout).is_err() {
                return false;
            }
            o.status.success()
        }
        Err(_) => false,
    }
}

/// 写入带 UTF-8 BOM 的文件。cscript 默认按 ANSI 解析无 BOM 的文件，
/// 添加 BOM 确保 cscript 以 UTF-8 解析，避免中文乱码。
fn write_utf8_bom(path: &Path, content: &str) -> std::io::Result<()> {
    let mut bytes = Vec::with_capacity(content.len() + 3);
    bytes.extend_from_slice(&[0xEF, 0xBB, 0xBF]); // UTF-8 BOM
    bytes.extend_from_slice(content.as_bytes());
    fs::write(path, bytes)
}

/// 保存文件对话框。对应 C 版 `tinyfd_saveFileDialogW`。
pub fn save_file_dialog(
    title: &str,
    default_path: &str,
    filter_patterns: &[&str],
    filter_description: &str,
) -> Option<String> {
    let _co = CoInit::new();

    let dir = helpers::path_without_final_slash(default_path).to_string();
    let file_name = helpers::last_name(default_path).to_string();

    let filter = build_filter(filter_patterns, filter_description);
    // 缓冲区需容纳完整路径（不只是默认文件名），否则 GetSaveFileNameW 返回 FALSE。
    let buf_size = helpers::MAX_PATH_OR_CMD + 1;
    let mut file_buf = vec![0u16; buf_size];
    let initial = to_wide_z(&file_name);
    file_buf[..initial.len()].copy_from_slice(&initial);
    let title_z = to_wide_z(title);
    let dir_z = if dir.is_empty() {
        None
    } else {
        Some(to_wide_z(&dir))
    };
    let def_ext = if filter_patterns.is_empty() {
        None
    } else {
        Some(to_wide_z(filter_patterns[0]))
    };

    let mut ofn: OPENFILENAMEW = unsafe { std::mem::zeroed() };
    ofn.lStructSize = std::mem::size_of::<OPENFILENAMEW>() as u32;
    ofn.hwndOwner = unsafe { GetForegroundWindow() };
    ofn.lpstrFilter = filter.as_ptr();
    ofn.lpstrFile = file_buf.as_mut_ptr();
    ofn.nMaxFile = file_buf.len() as u32;
    if let Some(z) = &dir_z {
        ofn.lpstrInitialDir = z.as_ptr();
    }
    ofn.lpstrTitle = title_z.as_ptr();
    ofn.Flags = OFN_EXPLORER | OFN_OVERWRITEPROMPT | OFN_PATHMUSTEXIST | OFN_NOCHANGEDIR;
    // 默认扩展名：第一个过滤模式（对应 C 版，虽有瑕疵但保持一致）。
    if let Some(z) = &def_ext {
        ofn.lpstrDefExt = z.as_ptr();
    }

    let ok = unsafe { GetSaveFileNameW(&mut ofn) };
    if ok == TRUE {
        Some(from_wide_z(&file_buf))
    } else {
        None
    }
}

/// 打开文件对话框。对应 C 版 `tinyfd_openFileDialogW`。
///
/// 多选时返回 `|` 分隔的多个路径。
pub fn open_file_dialog(
    title: &str,
    default_path: &str,
    filter_patterns: &[&str],
    filter_description: &str,
    allow_multiple: bool,
) -> Option<String> {
    let _co = CoInit::new();

    let dir = helpers::path_without_final_slash(default_path).to_string();
    let file_name = helpers::last_name(default_path).to_string();

    let filter = build_filter(filter_patterns, filter_description);
    let title_z = to_wide_z(title);
    let dir_z = if dir.is_empty() {
        None
    } else {
        Some(to_wide_z(&dir))
    };

    // 多选时分配大缓冲区。
    let buf_size = if allow_multiple {
        helpers::MAX_MULTIPLE_FILES * helpers::MAX_PATH_OR_CMD + 1
    } else {
        helpers::MAX_PATH_OR_CMD + 1
    };
    let mut file_buf = vec![0u16; buf_size];
    let initial = to_wide_z(&file_name);
    file_buf[..initial.len()].copy_from_slice(&initial);

    let mut ofn: OPENFILENAMEW = unsafe { std::mem::zeroed() };
    ofn.lStructSize = std::mem::size_of::<OPENFILENAMEW>() as u32;
    ofn.hwndOwner = unsafe { GetForegroundWindow() };
    ofn.lpstrFilter = filter.as_ptr();
    ofn.lpstrFile = file_buf.as_mut_ptr();
    ofn.nMaxFile = file_buf.len() as u32;
    if let Some(z) = &dir_z {
        ofn.lpstrInitialDir = z.as_ptr();
    }
    ofn.lpstrTitle = title_z.as_ptr();
    ofn.Flags = OFN_EXPLORER | OFN_NOCHANGEDIR | OFN_PATHMUSTEXIST | OFN_FILEMUSTEXIST;
    if allow_multiple {
        ofn.Flags |= OFN_ALLOWMULTISELECT;
    }

    let ok = unsafe { GetOpenFileNameW(&mut ofn) };
    if ok != TRUE {
        return None;
    }

    let result = parse_open_file_result(&file_buf);
    Some(result)
}

/// 解析 GetOpenFileNameW 的返回缓冲区。
///
/// 单选：缓冲区就是完整路径。
/// 多选：`目录\0文件1\0文件2\0\0`，需拼成 `目录\文件1|目录\文件2`。
fn parse_open_file_result(buf: &[u16]) -> String {
    // 找第一个 \0。
    let first_nul = match buf.iter().position(|&c| c == 0) {
        Some(i) => i,
        None => return utf::utf16_to_utf8(buf),
    };

    // 如果第一个 \0 后面就是 \0（或越界），说明是单选。
    if first_nul + 1 >= buf.len() || buf[first_nul + 1] == 0 {
        return utf::utf16_to_utf8(&buf[..first_nul]);
    }

    // 多选：第一个 \0 前是目录，之后每个 \0 分隔的是文件名。
    let dir = utf::utf16_to_utf8(&buf[..first_nul]);
    let mut files = Vec::new();
    let mut start = first_nul + 1;
    loop {
        let end = buf[start..].iter().position(|&c| c == 0).map(|p| start + p);
        match end {
            Some(e) if e > start => {
                let name = utf::utf16_to_utf8(&buf[start..e]);
                files.push(format!("{}\\{}", dir, name));
                start = e + 1;
                if start >= buf.len() || buf[start] == 0 {
                    break;
                }
            }
            _ => break,
        }
    }
    files.join("|")
}

/// 文件夹选择对话框。对应 C 版 `tinyfd_selectFolderDialogW`。
///
/// 使用 `SHBrowseForFolderW`（经典样式，对应 C 版）。
pub fn select_folder_dialog(title: &str, default_path: &str) -> Option<String> {
    let _co = CoInit::new_apartment();

    let title_z = to_wide_z(title);
    let default_z = to_wide_z(default_path);
    let mut path_buf = [0u16; helpers::MAX_PATH_OR_CMD];

    let mut bi: BROWSEINFOW = unsafe { std::mem::zeroed() };
    bi.hwndOwner = unsafe { GetForegroundWindow() };
    bi.pszDisplayName = path_buf.as_mut_ptr();
    bi.lpszTitle = title_z.as_ptr();
    bi.ulFlags = BIF_USENEWUI;
    bi.lpfn = Some(browse_callback);
    bi.lParam = default_z.as_ptr() as LPARAM;

    let pidl = unsafe { SHBrowseForFolderW(&bi) };
    if pidl.is_null() {
        return None;
    }

    let mut path = vec![0u16; helpers::MAX_PATH_OR_CMD];
    let ok = unsafe { SHGetPathFromIDListW(pidl, path.as_mut_ptr()) };
    unsafe { CoTaskMemFree(pidl as *const _) };

    if ok == TRUE {
        let s = from_wide_z(&path);
        if s.is_empty() { None } else { Some(s) }
    } else {
        None
    }
}

/// `SHBrowseForFolderW` 的回调，设置初始选中目录。
unsafe extern "system" fn browse_callback(hwnd: HWND, msg: u32, _lp: LPARAM, data: LPARAM) -> i32 {
    use windows_sys::Win32::UI::WindowsAndMessaging::SendMessageW;
    if msg == BFFM_INITIALIZED {
        // data 是默认路径的 wide string 指针。
        SendMessageW(hwnd, BFFM_SETSELECTIONW, 1, data);
    } else if msg == BFFM_SELCHANGED {
        // 简化：不实现 TreeView 滚动到可见。
    }
    0
}

/// 颜色选择器。对应 C 版 `tinyfd_colorChooserW`。
pub fn color_chooser(
    _title: &str,
    default_hex: Option<&str>,
    default_rgb: Option<RgbColor>,
) -> Option<(String, RgbColor)> {
    let _co = CoInit::new();

    // 确定默认 RGB。
    let default = match default_hex {
        Some(hex) => RgbColor::from_hex(hex).unwrap_or(default_rgb.unwrap_or_default()),
        None => default_rgb.unwrap_or_default(),
    };

    let mut custom_colors = [0u32; 16];
    let mut cc: CHOOSECOLORW = unsafe { std::mem::zeroed() };
    cc.lStructSize = std::mem::size_of::<CHOOSECOLORW>() as u32;
    cc.hwndOwner = unsafe { GetForegroundWindow() };
    cc.rgbResult = rgb_to_colorref(default);
    cc.lpCustColors = custom_colors.as_mut_ptr();
    cc.Flags = CC_RGBINIT | CC_FULLOPEN | CC_ANYCOLOR;

    let ok = unsafe { ChooseColorW(&mut cc) };
    if ok == TRUE {
        let rgb = colorref_to_rgb(cc.rgbResult);
        Some((rgb.to_hex(), rgb))
    } else {
        None
    }
}

fn rgb_to_colorref(c: RgbColor) -> u32 {
    (c.r as u32) | ((c.g as u32) << 8) | ((c.b as u32) << 16)
}

fn colorref_to_rgb(val: u32) -> RgbColor {
    RgbColor::new(
        (val & 0xff) as u8,
        ((val >> 8) & 0xff) as u8,
        ((val >> 16) & 0xff) as u8,
    )
}

#[cfg(test)]
mod password_tests {
    use super::*;

    #[test]
    #[ignore = "requires an interactive Windows desktop; public prompts auto-cancel"]
    fn hidden_launch_does_not_hide_password_prompt() {
        use windows_sys::Win32::Foundation::{CloseHandle, WAIT_OBJECT_0};
        use windows_sys::Win32::System::Threading::{
            CreateProcessW, GetExitCodeProcess, PROCESS_INFORMATION, STARTF_USESHOWWINDOW,
            STARTUPINFOW, TerminateProcess, WaitForSingleObject,
        };
        let executable = std::env::current_exe().unwrap();
        let executable = to_wide_z(executable.to_str().unwrap());
        let mut command = to_wide_z(
            "fixture --exact windows::gui::password_tests::hidden_launch_password_child --ignored --nocapture",
        );
        let mut startup: STARTUPINFOW = unsafe { std::mem::zeroed() };
        startup.cb = std::mem::size_of::<STARTUPINFOW>() as u32;
        startup.dwFlags = STARTF_USESHOWWINDOW;
        startup.wShowWindow = 0; // The installed launcher's SW_HIDE.
        let mut process: PROCESS_INFORMATION = unsafe { std::mem::zeroed() };
        unsafe {
            assert_ne!(
                CreateProcessW(
                    executable.as_ptr(),
                    command.as_mut_ptr(),
                    std::ptr::null(),
                    std::ptr::null(),
                    0,
                    CREATE_NO_WINDOW,
                    std::ptr::null(),
                    std::ptr::null(),
                    &startup,
                    &mut process
                ),
                0
            );
            let waited = WaitForSingleObject(process.hProcess, 15000);
            if waited != WAIT_OBJECT_0 {
                TerminateProcess(process.hProcess, 124);
                WaitForSingleObject(process.hProcess, 5000);
            }
            let mut exit = 0;
            let got_exit = GetExitCodeProcess(process.hProcess, &mut exit);
            CloseHandle(process.hThread);
            CloseHandle(process.hProcess);
            assert_eq!(waited, WAIT_OBJECT_0, "owned fixture timed out");
            assert_ne!(got_exit, 0);
            assert_eq!(exit, 0, "fixture failed: 41=hidden, 42=unmasked");
        }
    }

    #[test]
    #[ignore = "child of the bounded hidden-launch desktop test; never collects input"]
    fn hidden_launch_password_child() {
        use std::time::{Duration, Instant};
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            FindWindowW, GWL_STYLE, GetDlgItem, GetWindowLongW, IsWindowVisible, PostMessageW,
        };
        for attempt in 0..2 {
            let title = format!(
                "Public hidden-startup fixture {} {attempt}",
                std::process::id()
            );
            let observer_title = title.clone();
            let observer = std::thread::spawn(move || {
                let deadline = Instant::now() + Duration::from_secs(5);
                loop {
                    let hwnd = unsafe {
                        FindWindowW(
                            to_wide_z("tinyfd_password").as_ptr(),
                            to_wide_z(&observer_title).as_ptr(),
                        )
                    };
                    if !hwnd.is_null() {
                        let visible = unsafe { IsWindowVisible(hwnd) != 0 };
                        if visible || Instant::now() >= deadline {
                            let edit = unsafe { GetDlgItem(hwnd, ID_EDIT as i32) };
                            let masked = !edit.is_null()
                                && unsafe { GetWindowLongW(edit, GWL_STYLE) } & ES_PASSWORD as i32
                                    != 0;
                            unsafe {
                                PostMessageW(hwnd, WM_CLOSE, 0, 0);
                            }
                            return (visible, masked);
                        }
                    }
                    assert!(
                        Instant::now() < deadline + Duration::from_secs(1),
                        "fixture window missing"
                    );
                    std::thread::sleep(Duration::from_millis(20));
                }
            });
            assert!(password_dialog(&title, "Public fixture; automatically cancelled").is_none());
            let (visible, masked) = observer.join().unwrap();
            if !visible {
                std::process::exit(41);
            }
            if !masked {
                std::process::exit(42);
            }
        }
    }

    #[test]
    fn password_windows_can_be_created_again_after_cancellation() {
        // Hidden windows exercise the real class and destruction path without
        // collecting input or requiring an interactive desktop.
        std::thread::spawn(|| {
            for _ in 0..3 {
                PWD_STATE.with(|state| {
                    *state.borrow_mut() = PwdState {
                        message: to_wide_z("Public lifecycle fixture"),
                        edit_hwnd: std::ptr::null_mut(),
                        result: None,
                    };
                });
                assert!(password_class_registered());
                let class_name = to_wide_z("tinyfd_password");
                let hwnd = unsafe {
                    CreateWindowExW(
                        0, class_name.as_ptr(), to_wide_z("Fixture").as_ptr(),
                        WS_OVERLAPPED, 0, 0, 400, 160,
                        std::ptr::null_mut(), std::ptr::null_mut(),
                        std::ptr::null_mut(), std::ptr::null(),
                    )
                };
                assert!(!hwnd.is_null(), "repeated password window creation");
                unsafe {
                    SendMessageW(hwnd, WM_CLOSE, 0, 0);
                }
                assert!(PWD_STATE.with(|state| matches!(state.borrow().result, Some(None))));
                let mut message: MSG = unsafe { std::mem::zeroed() };
                while unsafe { GetMessageW(&mut message, std::ptr::null_mut(), 0, 0) } > 0 {
                    unsafe {
                        TranslateMessage(&message);
                        DispatchMessageW(&message);
                    }
                }
            }
        }).join().unwrap();
    }
}
