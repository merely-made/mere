// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Resolve logical stylesheet lengths for hosts whose retained layout uses
//! physical pixels. Inline placement remains host-owned. Strings, URLs and
//! comments are preserved byte-for-byte, including authored content.
pub fn stylesheet_at_display_scale(css: &str, scale: f32) -> String {
    let scale = if scale.is_finite() {
        scale.clamp(0.5, 4.0)
    } else {
        1.0
    };
    if scale == 1.0 {
        return css.to_owned();
    }
    let bytes = css.as_bytes();
    let mut out = String::with_capacity(css.len());
    let mut i = 0;
    while i < bytes.len() {
        let start = i;
        if bytes[i] == b'\'' || bytes[i] == b'"' {
            let quote = bytes[i];
            i += 1;
            while i < bytes.len() {
                if bytes[i] == b'\\' {
                    i = (i + 2).min(bytes.len());
                } else {
                    let end = bytes[i] == quote;
                    i += 1;
                    if end {
                        break;
                    }
                }
            }
        } else if css[i..].starts_with("/*") {
            i += 2;
            i = css[i..].find("*/").map_or(bytes.len(), |end| i + end + 2);
        } else if css[i..]
            .get(..4)
            .is_some_and(|s| s.eq_ignore_ascii_case("url("))
        {
            // URL tokens may legally contain unquoted filenames with px.
            i += 4;
            let mut quote = None;
            while i < bytes.len() {
                let ch = bytes[i];
                i += 1;
                if ch == b'\\' {
                    i = (i + 1).min(bytes.len());
                    continue;
                }
                if let Some(end) = quote {
                    if ch == end {
                        quote = None;
                    }
                } else if ch == b'\'' || ch == b'"' {
                    quote = Some(ch);
                } else if ch == b')' {
                    break;
                }
            }
        } else if (bytes[i].is_ascii_digit() || matches!(bytes[i], b'.' | b'+' | b'-'))
            && (i == 0
                || !(bytes[i - 1].is_ascii_alphanumeric()
                    || matches!(bytes[i - 1], b'_' | b'-' | b'#')))
        {
            i += 1;
            while i < bytes.len() && (bytes[i].is_ascii_digit() || bytes[i] == b'.') {
                i += 1;
            }
            if matches!(bytes.get(i), Some(b'e' | b'E')) {
                let mut end = i + 1;
                if matches!(bytes.get(end), Some(b'+' | b'-')) {
                    end += 1;
                }
                let digits = end;
                while bytes.get(end).is_some_and(u8::is_ascii_digit) {
                    end += 1;
                }
                if end > digits {
                    i = end;
                }
            }
            if css[i..]
                .get(..2)
                .is_some_and(|s| s.eq_ignore_ascii_case("px"))
                && (i + 2 == bytes.len()
                    || !(bytes[i + 2].is_ascii_alphanumeric() || bytes[i + 2] == b'-'))
                && let Ok(value) = css[start..i].parse::<f32>()
            {
                out.push_str(&format!("{:.3}px", value * scale));
                i += 2;
                continue;
            }
        } else {
            i += css[i..].chars().next().unwrap().len_utf8();
        }
        out.push_str(&css[start..i]);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn converts_lengths_without_rewriting_authored_strings_or_assets() {
        let css = "/* 13px */ .x13px { padding:-2px .5px; font:13px serif; content:'13px'; background:url(icon13px.svg); width:50%; }";
        let scaled = stylesheet_at_display_scale(css, 2.0);
        assert!(scaled.contains("padding:-4.000px 1.000px"));
        assert!(scaled.contains("font:26.000px"));
        for preserved in [
            "/* 13px */",
            ".x13px",
            "'13px'",
            "url(icon13px.svg)",
            "width:50%",
        ] {
            assert!(scaled.contains(preserved));
        }
        assert_eq!(stylesheet_at_display_scale(css, f32::NAN), css);
        let tricky = r#".icon{background:url("icon)13px.svg");content:"a\"13px";width:1e2px;}"#;
        let scaled = stylesheet_at_display_scale(tricky, 2.0);
        assert!(scaled.contains(r#"url("icon)13px.svg")"#));
        assert!(scaled.contains(r#"content:"a\"13px""#));
        assert!(scaled.contains("width:200.000px"));
    }
}
