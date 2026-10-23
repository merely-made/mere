// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Portable reading intent. Hosts persist this on a projection, never a source
//! document or a theme. Display scale and temporary Find state are separate.
use crate::{FontRole, ThemeFontStyle, Typography};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "ReadingWire")]
pub struct ReadingPreferences {
    pub text_size: u16,
    /// Measure in ems, so changing type size retains the intended line length.
    pub column_em: u16,
}
#[derive(Deserialize)]
#[serde(default, deny_unknown_fields)]
struct ReadingWire {
    text_size: u16,
    column_em: u16,
}
impl Default for ReadingWire {
    fn default() -> Self {
        Self {
            text_size: 18,
            column_em: 36,
        }
    }
}
impl Default for ReadingPreferences {
    fn default() -> Self {
        Self {
            text_size: 18,
            column_em: 36,
        }
    }
}
impl TryFrom<ReadingWire> for ReadingPreferences {
    type Error = String;
    fn try_from(w: ReadingWire) -> Result<Self, String> {
        let value = Self {
            text_size: w.text_size,
            column_em: w.column_em,
        };
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
            || !(Self::MIN_COLUMN..=Self::MAX_COLUMN).contains(&self.column_em)
        {
            return Err("Reading text must be 14–28 px and its column 24–48 em.".into());
        }
        Ok(())
    }
    pub fn step_text(&mut self, larger: bool) {
        self.text_size = (i32::from(self.text_size) + if larger { 2 } else { -2 })
            .clamp(i32::from(Self::MIN_TEXT), i32::from(Self::MAX_TEXT))
            as u16;
    }
    pub fn step_column(&mut self, wider: bool) {
        self.column_em = (i32::from(self.column_em) + if wider { 4 } else { -4 })
            .clamp(i32::from(Self::MIN_COLUMN), i32::from(Self::MAX_COLUMN))
            as u16;
    }
    pub fn typography(self) -> Typography {
        let mut value = Typography::default();
        value
            .set(
                FontRole::Reading,
                ThemeFontStyle {
                    size_px: Some(f32::from(self.text_size)),
                    ..Default::default()
                },
            )
            .expect("bounded reading size");
        value
    }
    pub fn width_px(self) -> f32 {
        f32::from(self.text_size) * f32::from(self.column_em)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn old_projections_default_and_invalid_intent_refuses() {
        assert_eq!(
            serde_json::from_str::<ReadingPreferences>("{}").unwrap(),
            ReadingPreferences::default()
        );
        for wire in [r#"{"text_size":0}"#, r#"{"column_em":400}"#] {
            assert!(serde_json::from_str::<ReadingPreferences>(wire).is_err());
        }
    }
    #[test]
    fn choices_roundtrip_independently_and_keep_line_length() {
        let mut a = ReadingPreferences::default();
        let b = a;
        a.step_text(true);
        a.step_column(false);
        assert_eq!(b, ReadingPreferences::default());
        assert_eq!(a.width_px(), 640.0);
        assert_eq!(
            serde_json::from_str::<ReadingPreferences>(&serde_json::to_string(&a).unwrap())
                .unwrap(),
            a
        );
        for _ in 0..50 {
            a.step_text(true);
            a.step_column(false);
        }
        assert_eq!((a.text_size, a.column_em), (28, 24));
    }
}

/// A bounded text quote at the leading visible line. Geometry is only a fallback
/// for the identical source revision; a changed source must resolve the quote.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReadingAnchor {
    pub source: String,
    pub revision: u64,
    pub text: usize,
    pub offset: usize,
    pub exact: String,
    pub prefix: String,
    pub suffix: String,
    pub line_offset: f32,
    pub fallback_y: f32,
}
impl ReadingAnchor {
    pub fn validate(&self) -> Result<(), String> {
        if self.source.len() > 8192 || self.exact.len() > 512 || self.prefix.len() > 128
            || self.suffix.len() > 128 || self.exact.is_empty()
            || !self.line_offset.is_finite() || !(-2.0..=2.0).contains(&self.line_offset)
            || !self.fallback_y.is_finite() || self.fallback_y < 0.0 {
            return Err("Invalid reading anchor".into());
        }
        Ok(())
    }
}
/// A projection's visit trail, separate from content lineage and layout undo.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReadingTrail<T> {
    entries: Vec<T>,
    #[serde(default)]
    anchors: Vec<Option<ReadingAnchor>>,
    cursor: usize,
}
impl<T> Default for ReadingTrail<T> {
    fn default() -> Self { Self { entries: Vec::new(), anchors:Vec::new(), cursor: 0 } }
}
impl<T: Clone + PartialEq> ReadingTrail<T> {
    pub fn validate(&self) -> Result<(), String> {
        if self.anchors.len()>self.entries.len() {return Err("Invalid reading trail anchors".into());}
        for anchor in self.anchors.iter().flatten() {anchor.validate()?;}
        if self.entries.len() > 128 || (!self.entries.is_empty() && self.cursor >= self.entries.len()) || (self.entries.is_empty() && self.cursor != 0) {
            return Err("Invalid reading trail".into());
        }
        Ok(())
    }
    pub fn current_anchor(&self)->Option<&ReadingAnchor> {self.anchors.get(self.cursor).and_then(Option::as_ref)}
    pub fn anchor_current(&mut self,anchor:ReadingAnchor) {
        if self.current().is_some() {self.anchors.resize(self.entries.len(),None);self.anchors[self.cursor]=Some(anchor);}
    }
    pub fn current(&self) -> Option<&T> { self.entries.get(self.cursor) }
    pub fn can_back(&self) -> bool { self.cursor > 0 }
    pub fn can_forward(&self) -> bool { self.cursor + 1 < self.entries.len() }
    pub fn visit(&mut self, value: T) {
        if self.current() == Some(&value) { return; }
        self.entries.truncate(if self.entries.is_empty() { 0 } else { self.cursor + 1 });
        self.anchors.resize(self.entries.len(),None);
        self.anchors.push(None);
        self.entries.push(value);
        if self.entries.len() > 128 { self.entries.remove(0); self.anchors.remove(0); }
        self.cursor = self.entries.len() - 1;
    }
    pub fn step(&mut self, forward: bool) -> Option<T> {
        if forward { if !self.can_forward() { return None; } self.cursor += 1; }
        else { if !self.can_back() { return None; } self.cursor -= 1; }
        self.current().cloned()
    }
}
#[cfg(test)]
mod trail_tests {
    use super::*;
    #[test]
    fn visits_branch_and_roundtrip_without_confusing_layout_history() {
        let mut trail = ReadingTrail::default();
        trail.visit(1); trail.visit(2); trail.visit(2); trail.visit(3);
        assert_eq!(trail.step(false), Some(2));
        let mut cold: ReadingTrail<i32> = serde_json::from_str(&serde_json::to_string(&trail).unwrap()).unwrap();
        cold.validate().unwrap();
        assert_eq!(cold.step(false), Some(1));
        assert_eq!(cold.step(true), Some(2));
        cold.visit(4); assert!(!cold.can_forward());
        for i in 0..200 { cold.visit(i); }
        cold.validate().unwrap(); assert_eq!(cold.entries.len(),128);
        assert!(serde_json::from_str::<ReadingTrail<i32>>(r#"{"entries":[1],"cursor":90}"#).unwrap().validate().is_err());
    }
}
