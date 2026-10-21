// Copyright 2026 Mark Alan Boykin
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
            .max(1)
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
    /// False means stress is unknown, never an assertion of weak stress.
    #[serde(default = "known_stress", skip_serializing_if = "is_known_stress")]
    pub stress_known: bool,
}
fn known_stress() -> bool { true }
fn is_known_stress(value: &bool) -> bool { *value }

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

impl SceneInput {
    pub fn parse(json: &str) -> Result<Self, SceneInputError> {
        check(
            json.len() <= MAX_SCENE_INPUT_BYTES,
            "input",
            "input exceeds one MiB",
        )?;
        let value: serde_json::Value =
            serde_json::from_str(json).map_err(|e| error("input", e.to_string()))?;
        strict_core_records(&value, "input")?;
        let input: Self =
            serde_json::from_value(value).map_err(|e| error("input", e.to_string()))?;
        input.validate()?;
        Ok(input)
    }
    pub fn validate(&self) -> Result<(), SceneInputError> {
        check(
            self.version == BACKDROP_SCENE_VERSION,
            "version",
            "unsupported backdrop scene version",
        )?;
        source(&self.source, "source")?;
        check(
            !self.revision.trim().is_empty(),
            "revision",
            "a disclosed revision is required",
        )?;
        match &self.facts {
            SceneFacts::MetreGrid(grid) => {
                check(
                    (0..=32).contains(&grid.feet)
                        && (1..=8).contains(&grid.positions_per_foot)
                        && grid.lines.len() <= 256,
                    "metre",
                    "metre extent exceeds limits",
                )?;
                unique(grid.lines.iter().map(|l| l.key.as_str()), "metre.lines")?;
                let keys: Vec<String> = grid
                    .lines
                    .iter()
                    .flat_map(|line| {
                        std::iter::once(line.key.clone()).chain(
                            (0..line.syllables.len()).map(|i| format!("{}/syllable/{i}", line.key)),
                        )
                    })
                    .collect();
                unique(keys.iter().map(String::as_str), "metre.occurrences")?;
                let mut count = grid.lines.len();
                for line in &grid.lines {
                    source(&line.source, "metre.line.source")?;
                    check(
                        line.syllables.len() <= 512,
                        "metre.line.syllables",
                        "too many syllables",
                    )?;
                    if let Some(alignment) = &line.alignment {
                        check(
                            alignment.len() == line.syllables.len()
                                && alignment.iter().flatten().all(|span| {
                                    span.length > 0
                                        && span.end().is_some_and(|end| end <= grid.extent())
                                }),
                            "metre.line.alignment",
                            "one bounded positive span or unplaced value is required per syllable",
                        )?;
                    }
                    count += line.syllables.len();
                    check(
                        line.rhyme_tail
                            .as_ref()
                            .is_none_or(|s| !s.trim().is_empty())
                            && line
                                .rhyme_group
                                .as_ref()
                                .is_none_or(|s| !s.trim().is_empty()),
                        "metre.line.rhyme",
                        "unknown rhyme values must be absent, not blank",
                    )?;
                }
                check(count <= MAX_MARKS, "metre", "too many marks")?;
            },
            SceneFacts::TemplatedSequence(sequence) => {
                sequence.validate().map_err(|e| error("template", e))?
            },
            SceneFacts::Fretboard(board) => {
                check(
                    board.strings.iter().all(|s| s.open_midi <= 127),
                    "fretboard.strings",
                    "tuning must use valid MIDI pitches",
                )?;
                check(
                    !board.strings.is_empty()
                        && board.strings.len() <= 24
                        && (1..=48).contains(&board.frets)
                        && board.positions.len() <= MAX_MARKS,
                    "fretboard",
                    "instrument extent exceeds limits",
                )?;
                unique(
                    board.positions.iter().map(|p| p.key.as_str()),
                    "fretboard.positions",
                )?;
                let mut occupied = HashSet::new();
                for p in &board.positions {
                    source(&p.source, "fretboard.position.source")?;
                    let Some(string) = board.strings.get(p.string_index as usize) else {
                        return Err(error("fretboard.position.string_index", "missing string"));
                    };
                    check(
                        p.fret <= board.frets
                            && p.pitch_class < 12
                            && (u16::from(string.open_midi) + p.fret) <= 127
                            && (u16::from(string.open_midi) + p.fret) % 12
                                == u16::from(p.pitch_class),
                        "fretboard.position",
                        "position disagrees with the disclosed tuning or pitch class",
                    )?;
                    check(
                        occupied.insert((p.string_index, p.fret)),
                        "fretboard.positions",
                        "two occurrences occupy one instrument position",
                    )?;
                }
            },
            SceneFacts::TerrainCoverage(map) => {
                let bound = map.extent.w.max(map.extent.h) * 4.0;
                check(
                    map.contours
                        .iter()
                        .chain(map.unmeasured.iter())
                        .flat_map(|v| v.iter())
                        .chain(map.coverage.iter().flat_map(|c| c.contour.iter()))
                        .all(|p| p.x.abs() <= bound && p.y.abs() <= bound),
                    "terrain.contours",
                    "geometry exceeds the disclosed plane's supported margin",
                )?;
                let vertices: usize = map
                    .contours
                    .iter()
                    .map(Vec::len)
                    .chain(map.unmeasured.iter().map(Vec::len))
                    .chain(map.coverage.iter().map(|c| c.contour.len()))
                    .sum();
                check(
                    vertices <= 131_072,
                    "terrain",
                    "total contour budget exceeded",
                )?;
                check(
                    map.extent.w.is_finite()
                        && map.extent.h.is_finite()
                        && map.extent.w > 0.0
                        && map.extent.h > 0.0
                        && map.extent.w <= 100_000.0
                        && map.extent.h <= 100_000.0,
                    "terrain.extent",
                    "extent must be finite and positive",
                )?;
                check(
                    map.devices.len() <= 512
                        && map.contours.len() <= 512
                        && map.coverage.len() <= 1536
                        && map.unmeasured.len() <= 512
                        && map.links.len() <= 4096,
                    "terrain",
                    "scene exceeds geometry limits",
                )?;
                unique(
                    map.devices.iter().map(|d| d.key.as_str()),
                    "terrain.devices",
                )?;
                for device in &map.devices {
                    source(&device.source, "terrain.device.source")?;
                    if let Some(p) = device.position {
                        check(
                            finite(p)
                                && p.x >= 0.0
                                && p.y >= 0.0
                                && p.x <= map.extent.w
                                && p.y <= map.extent.h,
                            "terrain.device.position",
                            "position lies outside its disclosed plane",
                        )?;
                    }
                }
                for points in &map.contours {
                    poly(points, 2, "terrain.contours")?;
                }
                for points in &map.unmeasured {
                    poly(points, 3, "terrain.unmeasured")?;
                }
                for c in &map.coverage {
                    source(&c.source, "terrain.coverage.source")?;
                    poly(&c.contour, 3, "terrain.coverage.contour")?;
                    check(
                        !c.basis.trim().is_empty(),
                        "terrain.coverage.basis",
                        "coverage needs its disclosed basis",
                    )?;
                    check(
                        map.devices
                            .iter()
                            .any(|d| d.key == c.device && d.position.is_some()),
                        "terrain.coverage.device",
                        "coverage requires a placed device",
                    )?;
                }
                for link in &map.links {
                    check(
                        map.devices.iter().any(|d| d.key == link.from)
                            && map.devices.iter().any(|d| d.key == link.to)
                            && !link.why.trim().is_empty(),
                        "terrain.links",
                        "link needs known endpoints and a reason",
                    )?;
                }
            },
        }
        Ok(())
    }
    /// Pure compilation. Rejection cannot mutate a host's active scene.
    pub fn compile(&self) -> Result<BackdropScene, SceneInputError> {
        self.validate()?;
        let mut out = BackdropScene {
            version: BACKDROP_SCENE_VERSION,
            revision: self.revision.clone(),
            title: self.title.clone(),
            scene: Scene::new(),
            marks: vec![],
            guides: vec![],
            relations: vec![],
            fields: vec![],
            unplaced: vec![],
            diagnostics: vec![],
        };
        let src = out.scene.intern_source(self.source.clone());
        match &self.facts {
            SceneFacts::MetreGrid(grid) => compile_metre(&mut out, src, grid),
            SceneFacts::TemplatedSequence(sequence) => compile_sequence(&mut out, src, sequence),
            SceneFacts::Fretboard(board) => compile_fretboard(&mut out, src, board),
            SceneFacts::TerrainCoverage(map) => compile_terrain(&mut out, src, map),
        }
        unique(out.marks.iter().map(|m| m.key.as_str()), "marks")?;
        let bytes = serde_json::to_vec(self).map_err(|e| error("input", e.to_string()))?;
        out.scene.generation = bytes.into_iter().fold(0xcbf2_9ce4_8422_2325_u64, |h, b| {
            (h ^ u64::from(b)).wrapping_mul(0x100_0000_01b3)
        });
        Ok(out)
    }
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
/// Shared groups and coordinates. Domain-specific marks and judgments stay
/// with their owning recipe/adapter.
fn draw_template(out: &mut BackdropScene, src: sceno::SourceIx, template: &SequenceTemplate) {
    for (i, label) in template.lanes.iter().enumerate() {
        if !label.is_empty() {
            out.guides.push(GuideLabel {
                at: Vec2::new(
                    template.x(template.extent as f32 / 2.0),
                    template.y(i as u16) - 40.0,
                ),
                text: label.clone(),
            });
        }
    }
    let depths = template.depths();
    let mut lane_depth = vec![0; template.lanes.len()];
    for group in &template.groups {
        lane_depth[group.lane as usize] =
            lane_depth[group.lane as usize].max(depths[group.key.as_str()]);
    }
    for group in &template.groups {
        let depth = depths[group.key.as_str()];
        let max_depth = lane_depth[group.lane as usize];
        let y = template.y(group.lane) + 33.0 + (max_depth - depth) as f32 * 20.0;
        let a = template.x(group.span.start as f32);
        let b = template.x(group.span.end().unwrap() as f32) - 5.0;
        guide(
            out,
            src,
            vec![
                Vec2::new(a, y),
                Vec2::new(a, y + 4.0),
                Vec2::new(b, y + 4.0),
                Vec2::new(b, y),
            ],
        );
        if !group.label.is_empty() {
            out.guides.push(GuideLabel {
                at: Vec2::new((a + b) / 2.0, y + 16.0),
                text: group.label.clone(),
            });
        }
    }
}
fn compile_sequence(out: &mut BackdropScene, src: sceno::SourceIx, sequence: &TemplatedSequence) {
    use std::collections::HashMap;
    let t = &sequence.template;
    draw_template(out, src, t);
    out.scene.bounds = Rect::new(
        Vec2::ZERO,
        Size2::new(t.x(t.extent as f32) + 48.0, 154.0 * t.lanes.len() as f32),
    );
    let source = out.scene.sources[src.0 as usize].clone();
    let mut anchors = HashMap::new();
    for slot in &sequence.slots {
        let centre = t.centre(slot.lane, slot.span);
        let at = Vec2::new(centre.x, centre.y + 20.0);
        let id = mark(
            out,
            slot.key.clone(),
            &source,
            slot.label.clone(),
            format!(
                "Expected: {} · {} ticks at {} · {}",
                slot.expectation, slot.span.length, slot.span.start, t.unit_label
            ),
            MarkRole::Node,
            at,
            Footprint::Circle { radius: 3.0 },
            true,
            Some(Vec2::new(at.x, at.y + 12.0)),
        );
        anchors.insert(slot.key.as_str(), (id, at));
        if !sequence
            .occurrences
            .iter()
            .any(|o| o.slot.as_ref() == Some(&slot.key))
        {
            out.diagnostics.push(format!(
                "{}: no source occurrence aligned; expectation remains open",
                slot.key
            ));
        }
    }
    for occurrence in &sequence.occurrences {
        let Some(p) = &occurrence.placement else {
            out.unplaced.push(Unplaced {
                key: occurrence.key.clone(),
                source: occurrence.source.clone(),
                label: occurrence.label.clone(),
                reason: format!("No supplied alignment · {}", occurrence.detail),
            });
            continue;
        };
        let centre = t.centre(p.lane, p.span);
        let at = Vec2::new(centre.x, centre.y - 10.0);
        let width = t.x(p.span.length as f32) - t.x(0.0) - 8.0;
        let id = mark(
            out,
            occurrence.key.clone(),
            &occurrence.source,
            occurrence.label.clone(),
            format!(
                "{} · start {} / span {} ticks · {}",
                occurrence.detail, p.span.start, p.span.length, t.unit_label
            ),
            MarkRole::Other,
            at,
            Footprint::Rect {
                size: Size2::new(width.max(3.0), 12.0),
            },
            true,
            Some(Vec2::new(at.x, at.y - 12.0)),
        );
        anchors.insert(occurrence.key.as_str(), (id, at));
        if let Some(slot) = &occurrence.slot {
            let (to, point) = anchors[slot.as_str()];
            relation(
                out,
                id,
                to,
                vec![at, point],
                "template.alignment",
                "Alignment".into(),
                "Source-declared alignment to expectation; no conformity judgment inferred".into(),
            );
        }
    }
    for r in &sequence.relations {
        if let (Some(&(from, a)), Some(&(to, b))) =
            (anchors.get(r.from.as_str()), anchors.get(r.to.as_str()))
        {
            let points = (0..=24)
                .map(|i| {
                    let u = i as f32 / 24.0;
                    Vec2::new(
                        a.x + (b.x - a.x) * u,
                        a.y + (b.y - a.y) * u - 22.0 * 4.0 * u * (1.0 - u),
                    )
                })
                .collect();
            relation(
                out,
                from,
                to,
                points,
                &r.kind,
                r.label.clone(),
                r.why.clone(),
            );
        } else {
            out.diagnostics.push(format!(
                "{}: relation has an unplaced endpoint; {}",
                r.label, r.why
            ));
        }
    }
}
fn compile_metre(out: &mut BackdropScene, src: sceno::SourceIx, grid: &MetreGrid) {
    let slot = 47.0;
    let row = 154.0;
    let n = grid.feet as usize * grid.positions_per_foot as usize;
    let max_syllables = grid.extent() as usize;
    let template = SequenceTemplate {
        lanes: vec![String::new(); grid.lines.len().max(1)],
        extent: max_syllables.max(1) as u32,
        ticks_per_unit: 1,
        unit_label: "metrical position".into(),
        groups: grid
            .lines
            .iter()
            .enumerate()
            .flat_map(|(i, line)| {
                (0..grid.feet).map(move |foot| TemplateGroup {
                    key: format!("{}/foot/{foot}", line.key),
                    lane: i as u16,
                    span: Span {
                        start: u32::from(foot) * u32::from(grid.positions_per_foot),
                        length: u32::from(grid.positions_per_foot),
                    },
                    parent: None,
                    label: String::new(),
                })
            })
            .collect(),
    };
    draw_template(out, src, &template);
    // Source text stays visible even when syllable analysis is unavailable.
    // Caption width reserves presentation space, not invented metrical units.
    let caption_width = |line: &VerseLine| line.text.chars().count() as f32 * 11.0;
    let text_right = grid.lines.iter().filter(|line| line.syllables.is_empty())
        .map(|line| 48.0 + caption_width(line) + 38.0).fold(0.0_f32, f32::max);
    let right = (template.x(max_syllables as f32) + 38.0).max(text_right);
    out.scene.bounds = Rect::new(
        Vec2::ZERO,
        Size2::new(right + 90.0, 42.0 + row * grid.lines.len().max(1) as f32),
    );
    let mut rows = Vec::new();
    for (i, line) in grid.lines.iter().enumerate() {
        let y = template.y(i as u16);
        let detail = if n == 0 {
            format!("{} supplied units; form unspecified; rhyme {}", line.syllables.len(),
                line.rhyme_group.as_deref().unwrap_or("unspecified"))
        } else {
            format!("{} syllables / {} expected; rhyme {}", line.syllables.len(), n,
                line.rhyme_group.as_deref().unwrap_or("unspecified"))
        };
        let id = mark(
            out,
            line.key.clone(),
            &line.source,
            line.text.clone(),
            detail,
            MarkRole::Node,
            Vec2::new(24.0, y),
            Footprint::Circle { radius: 5.0 },
            true,
            None,
        );
        out.scene.items[id.0 as usize].hit = Some(Footprint::Rect {
            size: Size2::new(36.0, 44.0),
        });
        rows.push(id);
        if line.syllables.is_empty() {
            out.guides.push(GuideLabel {
                at: Vec2::new(48.0 + caption_width(line) / 2.0, y + 17.0),
                text: line.text.clone(),
            });
        }
        for (j, syllable) in line.syllables.iter().enumerate() {
            let Some(span) = grid.placement(i, j) else {
                out.unplaced.push(Unplaced {
                    key: format!("{}/syllable/{j}", line.key),
                    source: line.source.clone(),
                    label: syllable.text.clone(),
                    reason: "Authored unplaced syllable; lexical stress is retained".into(),
                });
                continue;
            };
            let sx = template.centre(i as u16, span).x;
            let h = if !syllable.stress_known { 12.0 } else if syllable.stressed { 24.0 } else { 7.0 };
            mark(
                out,
                format!("{}/syllable/{j}", line.key),
                &line.source,
                syllable.text.clone(),
                format!(
                    "{} stress · syllable {} · start {} / span {} metrical positions",
                    if !syllable.stress_known { "unknown" } else if syllable.stressed { "strong" } else { "weak" },
                    j + 1,
                    span.start,
                    span.length
                ),
                if !syllable.stress_known {
                    MarkRole::Node
                } else if syllable.stressed {
                    MarkRole::StrongStress
                } else {
                    MarkRole::WeakStress
                },
                Vec2::new(sx, y - h / 2.0),
                Footprint::Rect {
                    size: Size2::new((slot * span.length as f32 - 8.0).max(3.0), h),
                },
                true,
                Some(Vec2::new(sx, y + 17.0)),
            );
        }
        if let Some(group) = &line.rhyme_group {
            out.guides.push(GuideLabel {
                at: Vec2::new(right, y),
                text: group.clone(),
            });
        }
        if n > 0 && line.syllables.len() != n {
            out.diagnostics.push(format!(
                "{}: {} syllables, template expects {n}",
                line.key,
                line.syllables.len()
            ));
        }
    }
    for b in 0..grid.lines.len() {
        let line = &grid.lines[b];
        let Some(group) = &line.rhyme_group else {
            continue;
        };
        let Some(a) = (0..b)
            .rev()
            .find(|&a| grid.lines[a].rhyme_group.as_ref() == Some(group))
        else {
            continue;
        };
        let previous = &grid.lines[a];
        let matches = previous
            .rhyme_tail
            .as_ref()
            .zip(line.rhyme_tail.as_ref())
            .is_some_and(|(a, b)| a == b);
        let y1 = template.y(a as u16);
        let y2 = template.y(b as u16);
        let points = (0..=24)
            .map(|step| {
                let t = step as f32 / 24.0;
                Vec2::new(right + 6.0 + 52.0 * 4.0 * t * (1.0 - t), y1 + (y2 - y1) * t)
            })
            .collect();
        relation(
            out,
            rows[a],
            rows[b],
            points,
            if matches {
                "derived.rhyme"
            } else {
                "expected.rhyme"
            },
            group.clone(),
            if matches {
                format!(
                    "Source-disclosed rhyme tails match: {}",
                    line.rhyme_tail.as_deref().unwrap()
                )
            } else {
                format!("Template group {group}; rhyme tails differ or are unknown")
            },
        );
        if !matches {
            out.diagnostics.push(format!(
                "{} / {}: expected rhyme {group} is not established",
                previous.key, line.key
            ));
        }
    }
}
fn compile_fretboard(out: &mut BackdropScene, src: sceno::SourceIx, board: &Fretboard) {
    let cell = 84.0;
    let top = 48.0;
    let open = 62.0;
    let nut = 94.0;
    let width = nut + board.frets as f32 * cell + 24.0;
    let height = top + board.strings.len() as f32 * 48.0;
    out.scene.bounds = Rect::new(Vec2::ZERO, Size2::new(width, height));
    for (s, string) in board.strings.iter().enumerate() {
        let y = top + s as f32 * 48.0;
        guide(
            out,
            src,
            vec![Vec2::new(44.0, y), Vec2::new(width - 16.0, y)],
        );
        out.guides.push(GuideLabel {
            at: Vec2::new(24.0, y),
            text: string.label.clone(),
        });
    }
    for fr in 0..=board.frets {
        let x = nut + fr as f32 * cell;
        guide(
            out,
            src,
            vec![Vec2::new(x, top - 18.0), Vec2::new(x, height - 24.0)],
        );
        if fr < board.frets {
            out.guides.push(GuideLabel {
                at: Vec2::new(x + cell / 2.0, 20.0),
                text: (fr + 1).to_string(),
            });
        }
    }
    out.guides.push(GuideLabel {
        at: Vec2::new(open, 20.0),
        text: "open".into(),
    });
    let mut ids = Vec::new();
    for p in &board.positions {
        let x = if p.fret == 0 {
            open
        } else {
            nut + (p.fret as f32 - 0.5) * cell
        };
        let y = top + p.string_index as f32 * 48.0;
        ids.push(mark(
            out,
            p.key.clone(),
            &p.source,
            p.label.clone(),
            format!(
                "string {}, fret {}, pitch class {}",
                p.string_index + 1,
                p.fret,
                p.pitch_class
            ),
            match p.role {
                ToneRole::Root => MarkRole::Root,
                ToneRole::Third => MarkRole::Third,
                ToneRole::Fifth => MarkRole::Fifth,
                ToneRole::Other => MarkRole::Other,
            },
            Vec2::new(x, y),
            Footprint::Circle { radius: 5.0 },
            true,
            Some(Vec2::new(x, y + 24.0)),
        ));
        out.scene.items.last_mut().unwrap().hit = Some(Footprint::Rect {
            size: Size2::new(40.0, 40.0),
        });
    }
    for b in 0..board.positions.len() {
        if let Some(a) = (0..b).find(|&a| board.positions[a].source == board.positions[b].source) {
            let pa = out.scene.items[ids[a].0 as usize].transform.translate;
            let pb = out.scene.items[ids[b].0 as usize].transform.translate;
            relation(out,ids[a],ids[b],vec![pa,pb],"same-resource",board.positions[b].label.clone(),"One source resource realized at different instrument positions; octaves may differ".into());
        }
    }
}
fn compile_terrain(out: &mut BackdropScene, src: sceno::SourceIx, map: &TerrainCoverage) {
    out.scene.bounds = Rect::new(Vec2::ZERO, map.extent);
    for points in &map.contours {
        guide(out, src, points.clone());
    }
    let mut ids = std::collections::HashMap::new();
    for d in &map.devices {
        if let Some(at) = d.position {
            let id = mark(
                out,
                d.key.clone(),
                &d.source,
                d.label.clone(),
                format!("{}; placed in source coordinates", d.realm),
                MarkRole::Node,
                at,
                Footprint::Circle { radius: 5.0 },
                true,
                Some(Vec2::new(at.x, at.y + 24.0)),
            );
            out.scene.items.last_mut().unwrap().hit = Some(Footprint::Circle { radius: 22.0 });
            ids.insert(&d.key, id);
        } else {
            out.unplaced.push(Unplaced {
                key: d.key.clone(),
                source: d.source.clone(),
                label: d.label.clone(),
                reason: "Position unknown; no coverage can be anchored".into(),
            });
        }
    }
    for c in &map.coverage {
        let region = out.scene.regions.len();
        out.scene.regions.push(Region {
            members: vec![ids[&c.device]],
            space: Scene::WORLD,
            contour: Some(Footprint::Polygon {
                points: c.contour.clone(),
            }),
            label: Some(format!("coverage:{:?}", c.tier)),
            confidence: 1.0,
        });
        out.fields.push(FieldReading {
            region,
            device: Some(c.device.clone()),
            tier: Some(c.tier),
            source: c.source.clone(),
            basis: c.basis.clone(),
        });
    }
    for points in &map.unmeasured {
        let region = out.scene.regions.len();
        out.scene.regions.push(Region {
            members: vec![],
            space: Scene::WORLD,
            contour: Some(Footprint::Polygon {
                points: points.clone(),
            }),
            label: Some("unmeasured".into()),
            confidence: 1.0,
        });
        out.fields.push(FieldReading {
            region,
            device: None,
            tier: None,
            source: out.scene.sources[src.0 as usize].clone(),
            basis: "Source declares this region unmeasured".into(),
        });
    }
    for link in &map.links {
        let (Some(&from), Some(&to)) = (ids.get(&link.from), ids.get(&link.to)) else {
            out.diagnostics.push(format!(
                "{} → {}: route has an unplaced endpoint",
                link.from, link.to
            ));
            continue;
        };
        let a = out.scene.items[from.0 as usize].transform.translate;
        let b = out.scene.items[to.0 as usize].transform.translate;
        relation(
            out,
            from,
            to,
            vec![a, b],
            match link.kind {
                LinkKind::Authored => "authored",
                LinkKind::Derived => "derived",
            },
            format!("{} → {}", link.from, link.to),
            link.why.clone(),
        );
    }
}
