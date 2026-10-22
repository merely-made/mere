# light-file-dialog

> [`tinyfiledialogs`](https://sourceforge.net/projects/tinyfiledialogs/) 的 Rust 移植版 —— 跨平台原生对话框库。
>
> A Rust port of [`tinyfiledialogs`](https://sourceforge.net/projects/tinyfiledialogs/) — cross-platform native dialog library.

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/platform-windows%20%7C%20macos%20%7C%20linux-blue.svg)](#平台支持--platform-support)
[![License](https://img.shields.io/badge/license-Zlib-green.svg)](LICENSE)
[![C Version](https://img.shields.io/badge/upstream%20C%20version-3.21.3-blue.svg)](https://sourceforge.net/projects/tinyfiledialogs/)

提供跨平台的本地对话框：消息框、输入框（含密码框）、通知、文件选择、文件夹选择、颜色选择。

Provides cross-platform native dialogs: message box, input box (with password), notification, file picker, folder picker, color chooser.

## 平台支持 / Platform Support

| 平台 / Platform | 实现方式 / Implementation |
| --- | --- |
| Windows | Win32 API（`MessageBoxW`、`GetOpenFileNameW`、`SHBrowseForFolderW`、`Shell_NotifyIconW` 等） |
| macOS | AppleScript |
| Linux/Unix | `zenity` / `kdialog` / `yad` / `Xdialog` 等 / `zenity` / `kdialog` / `yad` / `Xdialog` etc. |

## 两层 API / Two-Layer API

### 1. Builder 模式（推荐）/ Builder Pattern (Recommended)

链式配置，类型安全，无需 `Option<&str>`。

Chainable, type-safe, no `Option<&str>` noise.

```rust
use light_file_dialog::dialog::*;
use light_file_dialog::{IconType, RgbColor};

// 消息框 / Message box
MessageBox::new("hello", "Welcome to light-file-dialog")
    .icon(IconType::Info)
    .show();

// 密码框 / Password input
let pwd = InputBox::new("密码", "请输入密码")
    .password()
    .show();

// 打开文件（多选）/ Open file (multiple)
OpenFileDialog::new("选择文件")
    .filter(&["*.txt", "*.json"])
    .filter_description("text files")
    .multiple()
    .show();

// 颜色选择器 / Color chooser
ColorChooser::new("选择颜色")
    .default_hex("#FF0077")
    .default_rgb(RgbColor::new(0, 128, 255))
    .show();
```

### 2. 函数式 API（对应 C 版签名）/ Functional API (C-compatible signatures)

```rust
use light_file_dialog::{message_box, notify_popup, input_box, IconType, MessageBoxType};

message_box(
    Some("hello"),
    Some("Welcome"),
    MessageBoxType::Ok,
    IconType::Info,
    1,
);

notify_popup(Some("title"), Some("message"), IconType::Info);

// None 表示密码框 / None means password input
let pwd = input_box(Some("密码"), Some("请输入"), None);
```

## 可用对话框 / Available Dialogs

| Builder | 函数式 / Functional | 说明 / Description |
| --- | --- | --- |
| `MessageBox` | `message_box` | 消息框（ok/okcancel/yesno/yesnocancel）/ Message box |
| `Notify` | `notify_popup` | 系统通知 / System notification |
| `InputBox` | `input_box` | 输入框或密码框 / Input box or password box |
| `OpenFileDialog` | `open_file_dialog` | 打开文件（可多选）/ Open file (supports multiple) |
| `SaveFileDialog` | `save_file_dialog` | 保存文件 / Save file |
| `SelectFolder` | `select_folder_dialog` | 选择文件夹 / Select folder |
| `ColorChooser` | `color_chooser` | 颜色选择器 / Color chooser |
| `DialogBackend::query()` | `QUERY_TITLE` | 查询当前后端 / Query current backend |

## 快速开始 / Quick Start

`Cargo.toml`:

```toml
[dependencies]
light-file-dialog = "3.21.3"
```

```rust
use light_file_dialog::dialog::*;
use light_file_dialog::IconType;

fn main() {
    MessageBox::new("hello", "你好，世界！/ Hello, world!")
        .icon(IconType::Info)
        .show();
}
```

运行示例 / Run the example:

```bash
cargo run --example hello
```

## 与 C 版的差异 / Differences from C Version

- **字符串统一 UTF-8**：Rust 原生 `&str`/`String`，不再区分 `char`/`wchar_t` 双层。
- **线程安全**：全局状态（`tinyfd_verbose` 等）改用原子类型。
- **Windows 通知/密码框**：不依赖 HTA/`mshta.exe`（`mshta` 在 Windows 11 上启动即退出），改用 Win32 原生 API：
  - 通知：`Shell_NotifyIconW` 气球通知 / Toast notification
  - 密码框：自建窗口 + `Edit(ES_PASSWORD)` 控件 / Custom window with `Edit(ES_PASSWORD)`
- **Builder API**：在 C 版函数式签名之上新增 `dialog` 模块，提供链式 Builder。
- **类型安全**：`MessageBoxType`、`IconType`、`MessageBoxButton`、`RgbColor` 用枚举表达，替代 C 版的字符串参数。

- **UTF-8 everywhere**: Rust native `&str`/`String`, no more `char`/`wchar_t` dual layer.
- **Thread-safe**: Global state (`tinyfd_verbose`, etc.) uses atomic types.
- **Windows notify/password**: No HTA/`mshta.exe` dependency (`mshta` quits immediately on Windows 11), uses Win32 native APIs instead.
- **Builder API**: New `dialog` module with chainable builders on top of C-compatible functional API.
- **Type-safe**: Enums (`MessageBoxType`, `IconType`, `MessageBoxButton`, `RgbColor`) replace C string parameters.

## 全局状态 / Global State

```rust
use light_file_dialog::*;

set_verbose(1);        // 打印调用的命令 / Print commands invoked
set_silent(1);         // 隐藏后端错误 / Silence backend errors
set_force_console(0);  // 不强制控制台模式 / Don't force console mode
#[cfg(windows)]
set_win_utf8(1);       // Windows: char 按 UTF-8 处理 / Treat char as UTF-8

// 查询当前后端 / Query current backend
let backend = dialog::DialogBackend::query();
println!("graphic: {}, backend: {}", backend.graphic, backend.name);
```

## 项目结构 / Project Structure

```
src/
├── lib.rs          # 顶层函数式 API / Top-level functional API
├── dialog.rs       # Builder 模式 API / Builder pattern API
├── types.rs        # 公共类型（枚举）/ Public types (enums)
├── helpers.rs      # 跨平台工具函数 / Cross-platform helpers
├── state.rs        # 全局状态（原子类型）/ Global state (atomics)
├── windows/        # Windows 实现 / Windows implementation
│   ├── gui.rs      #   Win32 GUI 对话框 / Win32 GUI dialogs
│   ├── console.rs  #   控制台输入 / Console input
│   ├── basic_input.rs
│   └── utf.rs      #   UTF-8/UTF-16 转换 / UTF-8/UTF-16 conversion
└── unix/           # macOS/Linux 实现 / macOS/Linux implementation
    ├── mod.rs
    └── detect.rs   #   后端探测 / Backend detection
```

## 致谢 / Acknowledgements

- 原始 C 版作者 / Original C version by **Guillaume Vareille**
- 项目主页 / Project home: <https://sourceforge.net/projects/tinyfiledialogs/>

## 许可证 / License

Zlib，与上游 C 版一致 / Same as upstream C version.

## 状态 / Status

对应 C 版 v3.21.3，Windows 平台已完整测试。macOS/Linux 移植自原 C 版逻辑，未实测。

Ports C v3.21.3. Windows fully tested. macOS/Linux ported from C logic, not yet tested.
