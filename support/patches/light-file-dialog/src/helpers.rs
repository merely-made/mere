//! 通用辅助函数，对应 C 版 `tinyfiledialogs.c` 前部的静态工具函数。
//!
//! 这些函数跨平台共用，涉及路径拆分、字符串替换、引号检测、文件名校验等。

use std::path::Path;

/// 路径分隔符：Windows 为 `\`，unix 为 `/`。
pub const SLASH: &str = if cfg!(windows) { "\\" } else { "/" };

/// 最大路径/命令长度，对应 C 版 `MAX_PATH_OR_CMD`。
pub const MAX_PATH_OR_CMD: usize = 1024;

/// 多选文件的最大数量，对应 C 版 `MAX_MULTIPLE_FILES`。
pub const MAX_MULTIPLE_FILES: usize = 1024;

/// 分配失败时的回退多选文件数，对应 C 版 `LOW_MULTIPLE_FILES`。
pub const LOW_MULTIPLE_FILES: usize = 32;

/// 取路径中最后一个 `/` 或 `\` 之前的部分（即去掉文件名，保留目录）。
/// 对应 C 版 `getPathWithoutFinalSlash`。
///
/// `None` 表示路径无分隔符或为空，调用方应据此处理。
pub fn path_without_final_slash(path: &str) -> &str {
    match path.rfind(['/', '\\']) {
        Some(idx) => &path[..idx],
        None => "",
    }
}

/// 取路径中最后一个 `/` 或 `\` 之后的部分（即文件名）。
/// 对应 C 版 `getLastName`。
pub fn last_name(path: &str) -> &str {
    match path.rfind(['/', '\\']) {
        Some(idx) => &path[idx + 1..],
        None => path,
    }
}

/// 确保字符串以路径分隔符结尾（原地修改）。
/// 对应 C 版 `ensureFinalSlash`。
pub fn ensure_final_slash(s: &mut String) {
    if !s.is_empty() && !s.ends_with('/') && !s.ends_with('\\') {
        s.push_str(SLASH);
    }
}

/// 把 `source` 中所有 `old` 子串替换为 `new`，返回新字符串。
/// 对应 C 版 `tfd_replaceSubStr`。
pub fn replace_substring(source: &str, old: &str, new: &str) -> String {
    if old.is_empty() {
        return source.to_string();
    }
    source.replace(old, new)
}

/// 检测字符串是否包含单引号、双引号或反引号。
/// 对应 C 版 `tfd_quoteDetected`，用于防止 shell 命令注入。
pub fn quote_detected(s: Option<&str>) -> bool {
    match s {
        Some(text) => text.chars().any(|c| c == '\'' || c == '"' || c == '`'),
        None => false,
    }
}

/// 校验文件名（不含路径）是否合法：非空且不含 `\\/:*?"<>|`。
/// 对应 C 版 `filenameValid`。
pub fn filename_valid(name: &str) -> bool {
    !name.is_empty()
        && !name
            .chars()
            .any(|c| matches!(c, '\\' | '/' | ':' | '*' | '?' | '"' | '<' | '>' | '|'))
}

/// 检测文件是否存在。对应 C 版 `fileExists`。
pub fn file_exists(path: &str) -> bool {
    !path.is_empty() && Path::new(path).is_file()
}

/// 检测目录是否存在。对应 C 版 unix 的 `dirExists` / windows 的 `dirExists`。
pub fn dir_exists(path: &str) -> bool {
    if path.is_empty() {
        return true;
    }
    // C 版对 `X:` 形式的盘符根直接返回 true。
    #[cfg(windows)]
    if path.len() == 2 && path.as_bytes()[1] == b':' {
        return true;
    }
    Path::new(path).is_dir()
}

/// 获取当前工作目录，对应 C 版 `getCurDir`。
/// 失败时返回空字符串。
pub fn current_dir() -> String {
    std::env::current_dir()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// 把多选结果（`|` 分隔）按文件存在性过滤，返回过滤后的字符串。
/// 对应 C 版 `ensureFilesExist`。
pub fn ensure_files_exist(input: &str) -> String {
    input
        .split('|')
        .filter(|p| !p.is_empty() && file_exists(p))
        .collect::<Vec<_>>()
        .join("|")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_split_unix() {
        assert_eq!(path_without_final_slash("/home/user/file.txt"), "/home/user");
        assert_eq!(last_name("/home/user/file.txt"), "file.txt");
    }

    #[test]
    fn path_split_windows() {
        assert_eq!(path_without_final_slash(r"C:\dir\file.txt"), r"C:\dir");
        assert_eq!(last_name(r"C:\dir\file.txt"), "file.txt");
    }

    #[test]
    fn path_split_no_separator() {
        assert_eq!(path_without_final_slash("file.txt"), "");
        assert_eq!(last_name("file.txt"), "file.txt");
    }

    #[test]
    fn path_split_trailing_slash() {
        assert_eq!(path_without_final_slash("/home/user/"), "/home/user");
        assert_eq!(last_name("/home/user/"), "");
    }

    #[test]
    fn replace_substring_basic() {
        assert_eq!(replace_substring("hello world", "world", "rust"), "hello rust");
        assert_eq!(replace_substring("a.b.c", ".", "-"), "a-b-c");
        assert_eq!(replace_substring("abc", "", "x"), "abc");
    }

    #[test]
    fn quote_detection() {
        assert!(quote_detected(Some("has'quote")));
        assert!(quote_detected(Some(r#"has"quote"#)));
        assert!(quote_detected(Some("has`quote")));
        assert!(!quote_detected(Some("clean")));
        assert!(!quote_detected(None));
    }

    #[test]
    fn filename_validation() {
        assert!(filename_valid("file.txt"));
        assert!(!filename_valid(""));
        assert!(!filename_valid("a/b"));
        assert!(!filename_valid("a:b"));
        assert!(!filename_valid("a*b"));
    }
}
