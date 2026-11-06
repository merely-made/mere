// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Portable, disclosed-data scenes. Sources supply scansion, instrument
//! positions and coverage contours; this module supplies geometry, never
//! analysis, radio prediction, persistence or application authority.

use crate::template::{SequenceTemplate, Span, TemplateGroup, TemplatedSequence};
use sceno::{
    Backdrop, Footprint, InstanceId, ProjectedItem, Rect, Region, Representation, RoutedRelation,
    Scene, Size2, SourceRef, Transform2, Vec2,
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

pub const BACKDROP_SCENE_VERSION: u16 = 1;
pub const MAX_SCENE_INPUT_BYTES: usize = 1024 * 1024;
const MAX_MARKS: usize = 8192;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SceneInput {
    pub version: u16,
    /// Provenance of the disclosed reading and its environmental guides.
    pub source: SourceRef,
    pub revision: String,
    pub title: String,
    pub facts: SceneFacts,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "data",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum SceneFacts {
    MetreGrid(MetreGrid),
    TemplatedSequence(TemplatedSequence),
    Fretboard(Fretboard),
    TerrainCoverage(TerrainCoverage),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MetreGrid {
    pub feet: u16,
    /// Two for binary feet; sources may disclose other templates explicitly.
    #[serde(default = "binary_foot", skip_serializing_if = "is_binary_foot")]
    pub positions_per_foot: u16,
    pub lines: Vec<VerseLine>,
}
impl MetreGrid {
    pub fn extent(&self) -> u32 {
        self.lines
            .iter()
            .map(|l| l.syllables.len() as u32)
            .max()
            .unwrap_or(1)
            .max(u32::from(self.feet) * u32::from(self.positions_per_foot))
    }
    pub fn placement(&self, line: usize, syllable: usize) -> Option<Span> {
        let row = self.lines.get(line)?;
        row.syllables.get(syllable)?;
        match &row.alignment {
            Some(spans) => spans.get(syllable).copied().flatten(),
            None => Some(Span {
                start: syllable as u32,
                length: 1,
            }),
        }
    }
}
fn binary_foot() -> u16 {
    2
}
fn is_binary_foot(value: &u16) -> bool {
    *value == 2
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VerseLine {
    pub key: String,
    pub source: SourceRef,
    pub text: String,
    pub syllables: Vec<Syllable>,
    /// Omitted: legacy ordinal alignment. Present: one explicit placement (or
    /// unplaced value) per source syllable, independent of candidate stress.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub alignment: Option<Vec<Option<Span>>>,
    /// The template's expected group, independently of the observed tail.
    pub rhyme_group: Option<String>,
    /// Supplied by the source's pronunciation/scansion adapter, never inferred
    /// from spelling here. Unknown tails do not establish a rhyme.
    pub rhyme_tail: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Syllable {
    pub text: String,
    pub stressed: bool,
    /// False when the source supplied a syllable but no stress judgment.
    /// Legacy records keep their historical known-stress interpretation.
    #[serde(default = "stress_is_known", skip_serializing_if = "is_true")]
    pub stress_known: bool,
}

fn stress_is_known() -> bool {
    true
}
fn is_true(value: &bool) -> bool {
    *value
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Fretboard {
    /// Highest displayed string first. No guitar-specific tuning is assumed.
    pub strings: Vec<TunedString>,
    pub frets: u16,
    pub positions: Vec<FrettedPosition>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TunedString {
    pub label: String,
    pub open_midi: u8,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FrettedPosition {
    pub key: String,
    /// Several positions can point at the same pitch-class resource.
    pub source: SourceRef,
    pub string_index: u16,
    pub fret: u16,
    pub pitch_class: u8,
    pub label: String,
    pub role: ToneRole,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToneRole {
    Root,
    Third,
    Fifth,
    Other,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TerrainCoverage {
    /// All coordinates and contours use this source's own plane. This is not
    /// a geographic projection and never fabricates a location or a scale.
    pub extent: Size2,
    pub contours: Vec<Vec<Vec2>>,
    pub devices: Vec<RadioDevice>,
    pub coverage: Vec<Coverage>,
    pub unmeasured: Vec<Vec<Vec2>>,
    pub links: Vec<RadioLink>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RadioDevice {
    pub key: String,
    pub source: SourceRef,
    pub label: String,
    pub position: Option<Vec2>,
    pub realm: String,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Coverage {
    pub source: SourceRef,
    pub device: String,
    pub tier: CoverageTier,
    pub contour: Vec<Vec2>,
    /// The host must disclose the basis, including illustrative data.
    pub basis: String,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CoverageTier {
    Fringe,
    Usable,
    Near,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RadioLink {
    pub from: String,
    pub to: String,
    pub kind: LinkKind,
    pub why: String,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LinkKind {
    Authored,
    Derived,
}

/// A renderer role, distinct from the source identity and the scene geometry.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MarkRole {
    Node,
    WeakStress,
    StrongStress,
    Root,
    Third,
    Fifth,
    Other,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Mark {
    pub key: String,
    pub instance: InstanceId,
    pub label: String,
    pub detail: String,
    pub role: MarkRole,
    pub selectable: bool,
    /// A label baseline in scene coordinates; `None` uses the source's
    /// accessible reading only. A remote viewer needs no source lookup.
    pub label_at: Option<Vec2>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GuideLabel {
    pub at: Vec2,
    pub text: String,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RelationReading {
    pub index: usize,
    pub label: String,
    pub why: String,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FieldReading {
    pub region: usize,
    pub device: Option<String>,
    pub tier: Option<CoverageTier>,
    pub source: SourceRef,
    pub basis: String,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Unplaced {
    pub key: String,
    pub source: SourceRef,
    pub label: String,
    pub reason: String,
}

/// Geometry plus its presentation readings, analogous to CompiledProjection's
/// labels. Export this whole value; a bare Scene does not carry label strings.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BackdropScene {
    pub version: u16,
    pub revision: String,
    pub title: String,
    pub scene: Scene,
    pub marks: Vec<Mark>,
    pub guides: Vec<GuideLabel>,
    pub relations: Vec<RelationReading>,
    pub fields: Vec<FieldReading>,
    pub unplaced: Vec<Unplaced>,
    pub diagnostics: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SceneInputError {
    pub path: String,
    pub reason: String,
}
impl std::fmt::Display for SceneInputError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.path, self.reason)
    }
}
impl std::error::Error for SceneInputError {}
fn error(path: impl Into<String>, reason: impl Into<String>) -> SceneInputError {
    SceneInputError {
        path: path.into(),
        reason: reason.into(),
    }
}
fn check(ok: bool, path: &str, reason: &str) -> Result<(), SceneInputError> {
    if ok { Ok(()) } else { Err(error(path, reason)) }
}
fn source(source: &SourceRef, path: &str) -> Result<(), SceneInputError> {
    check(
        !source.adapter.trim().is_empty() && !source.id.trim().is_empty(),
        path,
        "source identity must be nonempty",
    )
}
fn finite(p: Vec2) -> bool {
    p.x.is_finite() && p.y.is_finite()
}
fn poly(points: &[Vec2], minimum: usize, path: &str) -> Result<(), SceneInputError> {
    check(
        points.len() >= minimum && points.len() <= 4096 && points.iter().all(|p| finite(*p)),
        path,
        "contour needs finite points within the geometry limit",
    )?;
    if minimum >= 3 {
        let area: f64 = points
            .iter()
            .zip(points.iter().cycle().skip(1))
            .map(|(a, b)| f64::from(a.x) * f64::from(b.y) - f64::from(b.x) * f64::from(a.y))
            .sum();
        check(
            area.is_finite() && area.abs() > f64::EPSILON,
            path,
            "polygon must enclose nonzero area",
        )?;
    }
    Ok(())
}
fn unique<'a>(keys: impl Iterator<Item = &'a str>, path: &str) -> Result<(), SceneInputError> {
    let mut seen = HashSet::new();
    for key in keys {
        check(
            !key.trim().is_empty() && seen.insert(key),
            path,
            "occurrence keys must be nonempty and unique",
        )?;
    }
    Ok(())
}

// Core Sceno records keep their existing permissive stored-format serde.
// This disclosed-input boundary is strict, including those nested records.
pub(crate) fn strict_core_records(
    value: &serde_json::Value,
    path: &str,
) -> Result<(), SceneInputError> {
    match value {
        serde_json::Value::Object(map) => {
            let allowed: Option<&[&str]> = if map.contains_key("adapter") {
                Some(&["adapter", "id"])
            } else if map.contains_key("x") || map.contains_key("y") {
                Some(&["x", "y"])
            } else if map.contains_key("w") || map.contains_key("h") {
                Some(&["w", "h"])
            } else {
                None
            };
            if let Some(allowed) = allowed {
                for key in map.keys() {
                    check(
                        allowed.contains(&key.as_str()),
                        path,
                        &format!("unknown key {key}"),
                    )?;
                }
            }
            for (key, child) in map {
                strict_core_records(child, &format!("{path}.{key}"))?;
            }
        },
        serde_json::Value::Array(values) => {
            for (i, child) in values.iter().enumerate() {
                strict_core_records(child, &format!("{path}[{i}]"))?;
            }
        },
        _ => {},
    }
    Ok(())
}

fn guide(out: &mut BackdropScene, source: sceno::SourceIx, points: Vec<Vec2>) {
    out.scene.backdrops.push(Backdrop {
        source,
        space: Scene::WORLD,
        transform: Transform2::IDENTITY,
        footprint: Footprint::Path { points, width: 1.0 },
        kind: "scene.guide".into(),
        visible: true,
        collidable: false,
    });
}
fn mark(
    out: &mut BackdropScene,
    key: String,
    source: &SourceRef,
    label: String,
    detail: String,
    role: MarkRole,
    at: Vec2,
    footprint: Footprint,
    selectable: bool,
    label_at: Option<Vec2>,
) -> InstanceId {
    let instance = InstanceId(out.scene.items.len() as u32);
    let source = out.scene.intern_source(source.clone());
    out.scene.items.push(ProjectedItem {
        source,
        space: Scene::WORLD,
        transform: Transform2::translation(at.x, at.y),
        footprint,
        representation: Representation::Open {
            kind: if selectable {
                "backdrop.site"
            } else {
                "backdrop.stress"
            }
            .into(),
        },
        layer: if selectable { 2 } else { 1 },
        visible: true,
        hit: (!selectable).then_some(Footprint::Point),
        channels: vec![],
    });
    out.marks.push(Mark {
        key,
        instance,
        label,
        detail,
        role,
        selectable,
        label_at,
    });
    instance
}
fn relation(
    out: &mut BackdropScene,
    from: InstanceId,
    to: InstanceId,
    points: Vec<Vec2>,
    kind: &str,
    label: String,
    why: String,
) {
    let index = out.scene.relations.len();
    out.scene.relations.push(RoutedRelation {
        from,
        to,
        space: Scene::WORLD,
        points,
        kind: Some(kind.into()),
        weight: None,
        link_key: None,
        marks: sceno::LinkMarks::default(),
        strength: None,
    });
    out.relations.push(RelationReading { index, label, why });
}

#[path = "backdrops/metre.rs"]
mod metre;

use metre::compile_metre;

#[path = "backdrops/sequence.rs"]
mod sequence;

use sequence::draw_template;

use sequence::compile_sequence;

#[path = "backdrops/fretboard.rs"]
mod fretboard;

use fretboard::compile_fretboard;

#[path = "backdrops/terrain.rs"]
mod terrain;

use terrain::compile_terrain;

#[path = "backdrops/validation.rs"]
mod validation;
