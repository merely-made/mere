// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! Revision-qualified proposals for disclosed spans. No handles, IO, source
//! authority, history or persistence are carried here. The owner accepts an
//! intent against current data, then applies its own domain validation.
use crate::{
    backdrops::{SceneFacts, SceneInput},
    template::{Placement, Span},
};
use sceno::SourceRef;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpanEdit {
    Move,
    Start,
    End,
}
impl SpanEdit {
    pub fn apply(
        self,
        original: Span,
        delta: f32,
        extent: u32,
        quantum: u32,
    ) -> Result<Span, String> {
        let end = original.end().ok_or("Span overflow")?;
        if original.length == 0 || end > extent || quantum == 0 || !delta.is_finite() {
            return Err("Invalid span, grid or delta".into());
        }
        // Selecting an off-grid source span must not author a snapped edit.
        if delta == 0.0 {
            return Ok(original);
        }
        let snap = |v: f32| {
            ((v / quantum as f32).round() * quantum as f32).clamp(0.0, extent as f32) as u32
        };
        Ok(match self {
            Self::Move => Span {
                start: snap(original.start as f32 + delta).min(extent - original.length),
                ..original
            },
            Self::Start => {
                let start = snap(original.start as f32 + delta).min(end - 1);
                Span {
                    start,
                    length: end - start,
                }
            },
            Self::End => {
                let end = snap(end as f32 + delta).max(original.start + 1);
                Span {
                    start: original.start,
                    length: end - original.start,
                }
            },
        })
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpanTarget {
    pub lane: u16,
    pub span: Span,
    pub extent: u32,
    pub ticks_per_unit: u16,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpanEditIntent {
    pub source: SourceRef,
    pub revision: String,
    pub occurrence: String,
    pub lane: u16,
    pub before: Span,
    pub after: Span,
}
impl SceneInput {
    /// Only supplied, placed occurrences; expected roles/groups are not edit targets.
    /// This discloses geometry, not permission to mutate its source.
    pub fn span_target(&self, key: &str) -> Option<SpanTarget> {
        match &self.facts {
            SceneFacts::MetreGrid(grid) => grid.lines.iter().enumerate().find_map(|(row, line)| {
                (0..line.syllables.len())
                    .find(|i| key == format!("{}/syllable/{i}", line.key))
                    .and_then(|i| grid.placement(row, i))
                    .map(|span| SpanTarget {
                        lane: row as u16,
                        span,
                        extent: grid.extent(),
                        ticks_per_unit: 1,
                    })
            }),
            SceneFacts::TemplatedSequence(seq) => seq
                .occurrences
                .iter()
                .find(|o| o.key == key)
                .and_then(|o| o.placement.as_ref())
                .map(|p| SpanTarget {
                    lane: p.lane,
                    span: p.span,
                    extent: seq.template.extent,
                    ticks_per_unit: seq.template.ticks_per_unit,
                }),
            _ => None,
        }
    }
}
impl SpanEditIntent {
    pub fn propose(
        input: &SceneInput,
        key: &str,
        edit: SpanEdit,
        delta: f32,
        quantum: u32,
    ) -> Result<Self, String> {
        input.validate().map_err(|e| e.to_string())?;
        let target = input
            .span_target(key)
            .ok_or("No placed source span at this occurrence")?;
        Ok(Self {
            source: input.source.clone(),
            revision: input.revision.clone(),
            occurrence: key.into(),
            lane: target.lane,
            before: target.span,
            after: edit.apply(target.span, delta, target.extent, quantum)?,
        })
    }
    pub fn validate(&self, input: &SceneInput) -> Result<(), String> {
        input.validate().map_err(|e| e.to_string())?;
        let target = input
            .span_target(&self.occurrence)
            .ok_or("Occurrence is absent or unplaced")?;
        if self.source != input.source
            || self.revision != input.revision
            || self.lane != target.lane
            || self.before != target.span
        {
            return Err(
                "Scene source, revision or occurrence span changed; start this edit again".into(),
            );
        }
        if self.after.length == 0 || self.after.end().is_none_or(|end| end > target.extent) {
            return Err("Proposed span lies outside the disclosed extent".into());
        }
        Ok(())
    }
    /// Build copied facts for a preview. The caller assigns its new revision,
    /// validates the domain and decides whether/where to retain the result.
    pub fn proposed_facts(&self, input: &SceneInput) -> Result<SceneFacts, String> {
        self.validate(input)?;
        let mut facts = input.facts.clone();
        if self.before == self.after {
            return Ok(facts);
        }
        match &mut facts {
            SceneFacts::MetreGrid(grid) => {
                let (row, index) = grid
                    .lines
                    .iter()
                    .enumerate()
                    .find_map(|(row, line)| {
                        (0..line.syllables.len())
                            .find(|i| self.occurrence == format!("{}/syllable/{i}", line.key))
                            .map(|i| (row, i))
                    })
                    .ok_or("Absent syllable")?;
                let spans = (0..grid.lines[row].syllables.len())
                    .map(|i| grid.placement(row, i))
                    .collect();
                grid.lines[row].alignment.get_or_insert(spans)[index] = Some(self.after);
            },
            SceneFacts::TemplatedSequence(seq) => {
                let occurrence = seq
                    .occurrences
                    .iter_mut()
                    .find(|o| o.key == self.occurrence)
                    .ok_or("Absent occurrence")?;
                occurrence.placement = Some(Placement {
                    lane: self.lane,
                    span: self.after,
                });
            },
            _ => return Err("Scene has no editable source spans".into()),
        }
        Ok(facts)
    }
}
