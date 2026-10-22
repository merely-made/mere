// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Portable reading intent. Hosts persist this on a projection, never a source
//! document or a theme. Display scale and temporary Find state are separate.
use serde::{Deserialize, Serialize};
use crate::{FontRole, ThemeFontStyle, Typography};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "ReadingWire")]
pub struct ReadingPreferences {
    pub text_size: u16,
    /// Measure in ems, so changing type size retains the intended line length.
    pub column_em: u16,
}
#[derive(Deserialize)]
#[serde(default, deny_unknown_fields)]
struct ReadingWire { text_size: u16, column_em: u16 }
impl Default for ReadingWire {
    fn default() -> Self { Self { text_size: 18, column_em: 36 } }
}
impl Default for ReadingPreferences {
    fn default() -> Self { Self { text_size: 18, column_em: 36 } }
}
impl TryFrom<ReadingWire> for ReadingPreferences {
    type Error = String;
    fn try_from(w: ReadingWire) -> Result<Self, String> {
        let value = Self { text_size: w.text_size, column_em: w.column_em };
        value.validate()?;
        Ok(value)
    }
}
impl ReadingPreferences {
    pub const MIN_TEXT: u16 = 14;
    pub const MAX_TEXT: u16 = 28;
    pub const MIN_COLUMN: u16 = 24;
    pub const MAX_COLUMN: u16 = 48;
    pub fn validate(&self) -> Result<(), String> {
        if !(Self::MIN_TEXT..=Self::MAX_TEXT).contains(&self.text_size)
            || !(Self::MIN_COLUMN..=Self::MAX_COLUMN).contains(&self.column_em) {
            return Err("Reading text must be 14–28 px and its column 24–48 em.".into());
        }
        Ok(())
    }
    pub fn step_text(&mut self, larger: bool) {
        self.text_size = (i32::from(self.text_size) + if larger { 2 } else { -2 })
            .clamp(i32::from(Self::MIN_TEXT), i32::from(Self::MAX_TEXT)) as u16;
    }
    pub fn step_column(&mut self, wider: bool) {
        self.column_em = (i32::from(self.column_em) + if wider { 4 } else { -4 })
            .clamp(i32::from(Self::MIN_COLUMN), i32::from(Self::MAX_COLUMN)) as u16;
    }
    pub fn typography(self) -> Typography {
        let mut value = Typography::default();
        value.set(FontRole::Reading, ThemeFontStyle {
            size_px: Some(f32::from(self.text_size)), ..Default::default()
        }).expect("bounded reading size");
        value
    }
    pub fn width_px(self) -> f32 { f32::from(self.text_size) * f32::from(self.column_em) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn old_projections_default_and_invalid_intent_refuses() {
        assert_eq!(serde_json::from_str::<ReadingPreferences>("{}").unwrap(), ReadingPreferences::default());
        for wire in [r#"{"text_size":0}"#, r#"{"column_em":400}"#] {
            assert!(serde_json::from_str::<ReadingPreferences>(wire).is_err());
        }
    }
    #[test]
    fn choices_roundtrip_independently_and_keep_line_length() {
        let mut a = ReadingPreferences::default();
        let b = a;
        a.step_text(true); a.step_column(false);
        assert_eq!(b, ReadingPreferences::default());
        assert_eq!(a.width_px(), 640.0);
        assert_eq!(serde_json::from_str::<ReadingPreferences>(&serde_json::to_string(&a).unwrap()).unwrap(), a);
        for _ in 0..50 { a.step_text(true); a.step_column(false); }
        assert_eq!((a.text_size, a.column_em), (28, 24));
    }
}
