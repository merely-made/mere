//! Windows 专用：UTF-8 / UTF-16 / MBCS 编码转换。
//!
//! 对应 C 版 `tinyfd_utf8to16` / `tinyfd_utf16to8` / `tinyfd_utf8toMbcs` /
//! `tinyfd_mbcsTo8` / `tinyfd_mbcsTo16` / `tinyfd_utf16toMbcs`。
//!
//! C 版用 `MultiByteToWideChar` / `WideCharToMultiByte`，Rust 版同样调用这两个 Win32 API。
//! 与 C 版不同的是，Rust 版返回 `Vec<u16>` / `String`，由调用方负责生命周期，
//! 不再需要 C 版的静态缓存。

use windows_sys::Win32::Globalization::{
    MultiByteToWideChar, WideCharToMultiByte, CP_ACP, CP_UTF8, MB_ERR_INVALID_CHARS,
    WC_ERR_INVALID_CHARS,
};

/// UTF-8 字符串 → UTF-16 (`u16` 序列，不含结尾 `\0`)。
///
/// 对应 C 版 `tinyfd_utf8to16`。若 UTF-8 含非法序列，回退为按 MBCS（ANSI 代码页）解释。
pub fn utf8_to_utf16(utf8: &str) -> Vec<u16> {
    let bytes = utf8.as_bytes();
    // 先探测所需 wchar_t 数量。
    let len = unsafe {
        MultiByteToWideChar(
            CP_UTF8,
            MB_ERR_INVALID_CHARS,
            bytes.as_ptr(),
            bytes.len() as i32,
            std::ptr::null_mut(),
            0,
        )
    };
    if len <= 0 {
        // UTF-8 解码失败，回退到 MBCS。对应 C 版 tinyfd_utf8to16 的 fallback。
        return mbcs_to_utf16(bytes);
    }
    let mut buf = vec![0u16; len as usize];
    unsafe {
        MultiByteToWideChar(
            CP_UTF8,
            MB_ERR_INVALID_CHARS,
            bytes.as_ptr(),
            bytes.len() as i32,
            buf.as_mut_ptr(),
            len,
        );
    }
    // 去掉可能的结尾 \0。
    while buf.last() == Some(&0) {
        buf.pop();
    }
    buf
}

/// MBCS（ANSI 代码页）字节 → UTF-16。
///
/// 对应 C 版 `tinyfd_mbcsTo16`。
pub fn mbcs_to_utf16(mbcs: &[u8]) -> Vec<u16> {
    let len = unsafe {
        MultiByteToWideChar(
            CP_ACP,
            0,
            mbcs.as_ptr(),
            mbcs.len() as i32,
            std::ptr::null_mut(),
            0,
        )
    };
    if len <= 0 {
        return vec![];
    }
    let mut buf = vec![0u16; len as usize];
    unsafe {
        MultiByteToWideChar(
            CP_ACP,
            0,
            mbcs.as_ptr(),
            mbcs.len() as i32,
            buf.as_mut_ptr(),
            len,
        );
    }
    while buf.last() == Some(&0) {
        buf.pop();
    }
    buf
}

/// UTF-16 → UTF-8 `String`。
///
/// 对应 C 版 `tinyfd_utf16to8`。失败时返回空串。
pub fn utf16_to_utf8(utf16: &[u16]) -> String {
    if utf16.is_empty() {
        return String::new();
    }
    let len = unsafe {
        WideCharToMultiByte(
            CP_UTF8,
            WC_ERR_INVALID_CHARS,
            utf16.as_ptr(),
            utf16.len() as i32,
            std::ptr::null_mut(),
            0,
            std::ptr::null(),
            std::ptr::null_mut(),
        )
    };
    if len <= 0 {
        return String::new();
    }
    let mut buf = vec![0u8; len as usize];
    unsafe {
        WideCharToMultiByte(
            CP_UTF8,
            WC_ERR_INVALID_CHARS,
            utf16.as_ptr(),
            utf16.len() as i32,
            buf.as_mut_ptr(),
            len,
            std::ptr::null(),
            std::ptr::null_mut(),
        );
    }
    while buf.last() == Some(&0) {
        buf.pop();
    }
    String::from_utf8_lossy(&buf).into_owned()
}

/// UTF-16 → MBCS（ANSI 代码页）字节。
///
/// 对应 C 版 `tinyfd_utf16toMbcs`。
#[allow(dead_code)]
pub fn utf16_to_mbcs(utf16: &[u16]) -> Vec<u8> {
    if utf16.is_empty() {
        return vec![];
    }
    let len = unsafe {
        WideCharToMultiByte(
            CP_ACP,
            0,
            utf16.as_ptr(),
            utf16.len() as i32,
            std::ptr::null_mut(),
            0,
            std::ptr::null(),
            std::ptr::null_mut(),
        )
    };
    if len <= 0 {
        return vec![];
    }
    let mut buf = vec![0u8; len as usize];
    unsafe {
        WideCharToMultiByte(
            CP_ACP,
            0,
            utf16.as_ptr(),
            utf16.len() as i32,
            buf.as_mut_ptr(),
            len,
            std::ptr::null(),
            std::ptr::null_mut(),
        );
    }
    while buf.last() == Some(&0) {
        buf.pop();
    }
    buf
}

/// UTF-8 → MBCS（先转 UTF-16 再转 MBCS）。对应 C 版 `tinyfd_utf8toMbcs`。
#[allow(dead_code)]
pub fn utf8_to_mbcs(utf8: &str) -> Vec<u8> {
    let utf16 = utf8_to_utf16(utf8);
    utf16_to_mbcs(&utf16)
}

/// MBCS → UTF-8（先转 UTF-16 再转 UTF-8）。对应 C 版 `tinyfd_mbcsTo8`。
#[allow(dead_code)]
pub fn mbcs_to_utf8(mbcs: &[u8]) -> String {
    let utf16 = mbcs_to_utf16(mbcs);
    utf16_to_utf8(&utf16)
}

/// 把 `&str` 转成以 `\0` 结尾的 UTF-16 `Vec<u16>`，方便传给 Win32 API。
pub fn to_wide_z(utf8: &str) -> Vec<u16> {
    let mut v = utf8_to_utf16(utf8);
    v.push(0);
    v
}

/// 把 UTF-16 slice（不含 `\0`）转成以 `\0` 结尾的 `Vec<u16>`。
#[allow(dead_code)]
pub fn wide_to_wide_z(utf16: &[u16]) -> Vec<u16> {
    let mut v: Vec<u16> = utf16.iter().copied().take_while(|&c| c != 0).collect();
    v.push(0);
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utf8_utf16_roundtrip_ascii() {
        let s = "hello world";
        let w = utf8_to_utf16(s);
        assert_eq!(utf16_to_utf8(&w), s);
    }

    #[test]
    fn utf8_utf16_roundtrip_chinese() {
        let s = "你好，世界！";
        let w = utf8_to_utf16(s);
        assert_eq!(utf16_to_utf8(&w), s);
    }

    #[test]
    fn to_wide_z_has_nul_terminator() {
        let w = to_wide_z("abc");
        assert_eq!(w, vec![b'a' as u16, b'b' as u16, b'c' as u16, 0]);
    }
}
