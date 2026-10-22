//! 对话框相关的公共类型定义。
//!
//! 这些类型对应 C 版 `tinyfiledialogs.h` 中的字符串参数（如 `"ok"` / `"info"`），
//! 在 Rust 端用枚举表达以获得类型安全。

/// 消息框类型，对应 C 版 `aDialogType` 参数。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageBoxType {
    /// `"ok"` —— 仅一个 OK 按钮。
    Ok,
    /// `"okcancel"` —— OK / Cancel。
    OkCancel,
    /// `"yesno"` —— Yes / No。
    YesNo,
    /// `"yesnocancel"` —— Yes / No / Cancel。
    YesNoCancel,
}

impl MessageBoxType {
    /// 转换为 C 版使用的字符串。
    pub fn as_str(self) -> &'static str {
        match self {
            MessageBoxType::Ok => "ok",
            MessageBoxType::OkCancel => "okcancel",
            MessageBoxType::YesNo => "yesno",
            MessageBoxType::YesNoCancel => "yesnocancel",
        }
    }
}

/// 图标类型，对应 C 版 `aIconType` 参数。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IconType {
    /// `"info"`
    Info,
    /// `"warning"`
    Warning,
    /// `"error"`
    Error,
    /// `"question"`
    Question,
}

impl IconType {
    /// 转换为 C 版使用的字符串。
    pub fn as_str(self) -> &'static str {
        match self {
            IconType::Info => "info",
            IconType::Warning => "warning",
            IconType::Error => "error",
            IconType::Question => "question",
        }
    }
}

/// 消息框按钮返回值，对应 C 版 `tinyfd_messageBox` 的返回值。
///
/// - `Cancel`/`No`（非 yesnocancel）→ 0
/// - `Ok`/`Yes` → 1
/// - `No`（yesnocancel 中的 No）→ 2
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageBoxButton {
    /// 0 —— Cancel，或 yesno 的 No。
    Cancel = 0,
    /// 1 —— Ok 或 Yes。
    Ok = 1,
    /// 2 —— yesnocancel 中的 No。
    No = 2,
}

impl MessageBoxButton {
    /// 从 C 版的整数返回值转换。
    pub fn from_int(value: i32) -> Self {
        match value {
            1 => MessageBoxButton::Ok,
            2 => MessageBoxButton::No,
            _ => MessageBoxButton::Cancel,
        }
    }

    /// 转换为 C 版的整数返回值。
    pub fn as_int(self) -> i32 {
        self as i32
    }
}

/// RGB 颜色（每通道 8 位）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RgbColor {
    /// 红色分量 0-255。
    pub r: u8,
    /// 绿色分量 0-255。
    pub g: u8,
    /// 蓝色分量 0-255。
    pub b: u8,
}

impl RgbColor {
    /// 从三个字节构造。
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// 从 `#RRGGBB` 形式的字符串解析。
    /// 仅接受恰好 7 字符且首字符为 `#`、后 6 位为十六进制数字的输入。
    pub fn from_hex(hex: &str) -> Option<Self> {
        let b = hex.as_bytes();
        if b.len() != 7 || b[0] != b'#' {
            return None;
        }
        let h = |c: u8| -> Option<u8> {
            match c {
                b'0'..=b'9' => Some(c - b'0'),
                b'a'..=b'f' => Some(c - b'a' + 10),
                b'A'..=b'F' => Some(c - b'A' + 10),
                _ => None,
            }
        };
        Some(Self::new(
            (h(b[1])? << 4) | h(b[2])?,
            (h(b[3])? << 4) | h(b[4])?,
            (h(b[5])? << 4) | h(b[6])?,
        ))
    }

    /// 转换为 `#RRGGBB` 形式的字符串（小写）。
    pub fn to_hex(self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rgb_hex_roundtrip() {
        let c = RgbColor::new(0xff, 0x00, 0xa0);
        assert_eq!(c.to_hex(), "#ff00a0");
        assert_eq!(RgbColor::from_hex("#ff00a0"), Some(c));
    }

    #[test]
    fn rgb_hex_black_white() {
        assert_eq!(RgbColor::new(0, 0, 0).to_hex(), "#000000");
        assert_eq!(RgbColor::new(255, 255, 255).to_hex(), "#ffffff");
        assert_eq!(RgbColor::from_hex("#FFFFFF"), Some(RgbColor::new(255, 255, 255)));
    }

    #[test]
    fn rgb_hex_invalid() {
        assert_eq!(RgbColor::from_hex(""), None);
        assert_eq!(RgbColor::from_hex("#12345"), None);
        assert_eq!(RgbColor::from_hex("#1234567"), None);
        assert_eq!(RgbColor::from_hex("123456"), None);
        assert_eq!(RgbColor::from_hex("#gggggg"), None);
    }
}
