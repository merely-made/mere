// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! Source-disclosed sequence structure, independent of words, chords or clocks.
//! Integer ticks describe alignment; hosts do not infer performance or validity.
use sceno::{SourceRef, Vec2};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Span {
    pub start: u32,
    pub length: u32,
}
impl Span {
    pub fn end(self) -> Option<u32> {
        self.start.checked_add(self.length)
    }
    pub fn contains(self, other: Self) -> bool {
        self.start <= other.start && self.end().zip(other.end()).is_some_and(|(a, b)| b <= a)
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TemplateGroup {
    pub key: String,
    pub lane: u16,
    pub span: Span,
    /// Parent must precede child. Depth is bounded to three levels.
    pub parent: Option<String>,
    pub label: String,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SequenceTemplate {
    pub lanes: Vec<String>,
    pub extent: u32,
    pub ticks_per_unit: u16,
    pub unit_label: String,
    pub groups: Vec<TemplateGroup>,
}
impl SequenceTemplate {
    pub fn validate(&self) -> Result<(), String> {
        if self.lanes.is_empty()
            || self.lanes.len() > 256
            || self.ticks_per_unit == 0
            || self.extent == 0
            || self.extent > 65536
            || self.extent as f32 / self.ticks_per_unit as f32 > 512.0
            || self.groups.len() > 4096
            || self.unit_label.trim().is_empty()
        {
            return Err("template extent exceeds supported limits".into());
        }
        let mut seen = HashMap::<&str, (&TemplateGroup, usize)>::new();
        for group in &self.groups {
            self.validate_placement(group.lane, group.span)?;
            if group.key.trim().is_empty() || seen.contains_key(group.key.as_str()) {
                return Err("template group keys must be nonempty and unique".into());
            }
            let depth = if let Some(parent) = &group.parent {
                let (p, depth) = seen
                    .get(parent.as_str())
                    .ok_or("group parent must precede child")?;
                if p.lane != group.lane || !p.span.contains(group.span) || *depth >= 2 {
                    return Err("group is outside its parent or nesting limit".into());
                }
                depth + 1
            } else {
                0
            };
            seen.insert(&group.key, (group, depth));
        }
        Ok(())
    }
    pub fn validate_placement(&self, lane: u16, span: Span) -> Result<(), String> {
        if usize::from(lane) >= self.lanes.len()
            || span.length == 0
            || span.end().is_none_or(|end| end > self.extent)
        {
            return Err("placement lies outside disclosed template".into());
        }
        Ok(())
    }
    pub fn x(&self, tick: f32) -> f32 {
        48.0 + tick / self.ticks_per_unit as f32 * 47.0
    }
    pub fn y(&self, lane: u16) -> f32 {
        60.0 + f32::from(lane) * 154.0
    }
    pub fn centre(&self, lane: u16, span: Span) -> Vec2 {
        Vec2::new(
            self.x(span.start as f32 + span.length as f32 / 2.0),
            self.y(lane),
        )
    }
    pub fn depths(&self) -> HashMap<&str, usize> {
        let mut depths = HashMap::new();
        for group in &self.groups {
            let depth = group
                .parent
                .as_deref()
                .and_then(|key| depths.get(key))
                .map_or(0, |d| d + 1);
            depths.insert(group.key.as_str(), depth);
        }
        depths
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TemplateSlot {
    pub key: String,
    pub lane: u16,
    pub span: Span,
    pub label: String,
    pub expectation: String,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Placement {
    pub lane: u16,
    pub span: Span,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TemplateOccurrence {
    pub key: String,
    pub source: SourceRef,
    pub label: String,
    pub detail: String,
    /// Absent alignment remains an unplaced reading.
    pub placement: Option<Placement>,
    /// Declared alignment to an expectation, not a claim of conformity.
    pub slot: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TemplateRelation {
    pub from: String,
    pub to: String,
    pub kind: String,
    pub label: String,
    pub why: String,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TemplatedSequence {
    pub template: SequenceTemplate,
    pub slots: Vec<TemplateSlot>,
    pub occurrences: Vec<TemplateOccurrence>,
    pub relations: Vec<TemplateRelation>,
}
impl TemplatedSequence {
    pub fn validate(&self) -> Result<(), String> {
        self.template.validate()?;
        if self.slots.len() + self.occurrences.len() > 8192 || self.relations.len() > 4096 {
            return Err("sequence exceeds geometry limits".into());
        }
        let mut keys = HashSet::new();
        for key in self
            .slots
            .iter()
            .map(|s| &s.key)
            .chain(self.occurrences.iter().map(|o| &o.key))
        {
            if key.trim().is_empty() || !keys.insert(key) {
                return Err("sequence keys must be nonempty and unique".into());
            }
        }
        for slot in &self.slots {
            self.template.validate_placement(slot.lane, slot.span)?;
            if slot.expectation.trim().is_empty() {
                return Err("slot needs a disclosed expectation".into());
            }
        }
        for occurrence in &self.occurrences {
            if occurrence.source.adapter.trim().is_empty() || occurrence.source.id.trim().is_empty()
            {
                return Err("occurrence needs source identity".into());
            }
            if let Some(p) = &occurrence.placement {
                self.template.validate_placement(p.lane, p.span)?;
            }
            if let Some(key) = &occurrence.slot {
                if !self.slots.iter().any(|s| &s.key == key) {
                    return Err("occurrence names an absent expectation".into());
                }
            }
        }
        for relation in &self.relations {
            if !keys.contains(&relation.from)
                || !keys.contains(&relation.to)
                || relation.kind.trim().is_empty()
                || relation.why.trim().is_empty()
            {
                return Err("relation needs known endpoints, kind and explanation".into());
            }
        }
        Ok(())
    }
}
