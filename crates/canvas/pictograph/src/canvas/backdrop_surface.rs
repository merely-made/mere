// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Independently embeddable realization of Scenomise's disclosed-data scenes.
//! No graph, application session, persistence, private GPU, clock, or source
//! mutation is required. Hosts lend fonts, theme roles and a viewport.

use super::{BoardFit, BoardRect, BoardText};
use paint_list_api::{
    ColorF, CommonPlacement, DashPattern, DeviceIntSize, LayoutPoint, LayoutRect, PaintCmd,
    PaintList, PathCommand, PathData, PathItem, StrokeCap, StrokeJoin, StrokeStyle,
};
use paint_list_render::{CompositeLayer, composite_paint_layers};
use sceno::{Footprint, Vec2};
use scenomise::backdrops::{
    BackdropScene, CoverageTier, MarkRole, SceneFacts, SceneInput, SceneInputError,
};
use std::collections::BTreeSet;

/// Applied appearance belongs to the embedding host. Defaults are the quiet
/// study palette, not an activation of a Tabard theme.
#[derive(Clone, Debug)]
pub struct SurfaceTheme {
    pub ground: [f32; 4],
    pub text: [f32; 4],
    pub guide: [f32; 4],
    pub category: [f32; 4],
    pub root: [f32; 4],
    pub fifth: [f32; 4],
    pub warning: [f32; 4],
}
impl Default for SurfaceTheme {
    fn default() -> Self {
        Self {
            ground: [0.13, 0.15, 0.12, 1.0],
            text: [0.88, 0.90, 0.85, 1.0],
            guide: [0.57, 0.63, 0.54, 0.28],
            category: [0.63, 0.73, 0.60, 1.0],
            root: [0.84, 0.72, 0.48, 1.0],
            fifth: [0.62, 0.74, 0.79, 1.0],
            warning: [0.86, 0.64, 0.58, 1.0],
        }
    }
}

/// Presentation transform. Sequence rows can use the available height without
/// shortening musical spans or changing the source's coordinates.
struct SurfaceFit {
    scale_x: f32,
    scale_y: f32,
    origin: (f32, f32),
}
impl SurfaceFit {
    fn to_viewport(&self, (x, y): (f32, f32)) -> (f32, f32) {
        (
            self.origin.0 + x * self.scale_x,
            self.origin.1 + y * self.scale_y,
        )
    }
}

pub struct BackdropSurface {
    compiled: BackdropScene,
    selected: Option<String>,
    retained: BTreeSet<String>,
    text: BoardText,
    font_ready: bool,
    fill_template_height: bool,
    pub show_coverage: bool,
    pub show_links: bool,
    /// Embedding owner opts into span affordances when it supplies editing.
    pub show_span_handles: bool,
}

impl BackdropSurface {
    pub fn new(input: &SceneInput) -> Result<Self, SceneInputError> {
        Ok(Self {
            compiled: input.compile()?,
            selected: None,
            retained: BTreeSet::new(),
            text: BoardText::new(),
            font_ready: false,
            fill_template_height: matches!(&input.facts, SceneFacts::TemplatedSequence(_)),
            show_coverage: true,
            show_links: true,
            show_span_handles: false,
        })
    }
    pub fn scene(&self) -> &BackdropScene {
        &self.compiled
    }
    pub fn selected(&self) -> Option<&str> {
        self.selected.as_deref()
    }
    pub fn retained(&self) -> &BTreeSet<String> {
        &self.retained
    }
    pub fn dismiss_retained(&mut self) {
        self.retained.clear();
    }
    pub fn register_font(&mut self, bytes: Vec<u8>) {
        self.font_ready |= !bytes.is_empty();
        self.text.register_font(bytes);
    }
    /// Root custom-property declarations for the label document; see
    /// [`BoardText::set_font_roles`].
    pub fn set_font_roles(&mut self, declarations: &str) {
        self.text.set_font_roles(declarations);
    }
    fn contains(&self, key: &str) -> bool {
        self.compiled
            .marks
            .iter()
            .any(|m| m.selectable && m.key == key)
            || self.compiled.unplaced.iter().any(|m| m.key == key)
    }
    /// Select an occurrence, not its entire source. Same-resource appearances
    /// remain distinct; their connection is disclosed separately.
    pub fn select(&mut self, key: Option<&str>) -> Result<(), String> {
        if let Some(key) = key {
            if !self.contains(key) {
                return Err(format!("unknown scene occurrence {key}"));
            }
        }
        if self.selected.as_deref() != key {
            if let Some(previous) = self.selected.take() {
                self.retained.insert(previous);
            }
            self.selected = key.map(str::to_owned);
        }
        if let Some(key) = key {
            self.retained.remove(key);
        }
        Ok(())
    }
    /// A failed replacement leaves geometry, selection, fonts and toggles
    /// intact. Valid replacements retain only still-disclosed occurrence keys.
    pub fn replace(&mut self, input: &SceneInput) -> Result<(), SceneInputError> {
        let next = input.compile()?;
        self.compiled = next;
        self.fill_template_height = matches!(&input.facts, SceneFacts::TemplatedSequence(_));
        let keys: BTreeSet<_> = self
            .compiled
            .marks
            .iter()
            .filter(|m| m.selectable)
            .map(|m| m.key.clone())
            .chain(self.compiled.unplaced.iter().map(|m| m.key.clone()))
            .collect();
        self.retained.retain(|key| keys.contains(key));
        if self
            .selected
            .as_ref()
            .is_some_and(|key| !keys.contains(key))
        {
            self.selected = None;
        }
        Ok(())
    }
    fn fit(&self, width: u32, height: u32) -> Result<SurfaceFit, String> {
        if width <= 48 || height <= 48 {
            return Err("scene viewport must exceed its 24px margins".into());
        }
        let b = self.compiled.scene.bounds;
        let base = BoardFit::new(24.0, 24.0, 24.0, 24.0)
            .framing_bounds()
            .transform(
                BoardRect::new(b.origin.x, b.origin.y, b.size.w, b.size.h),
                width as f32,
                height as f32,
            );
        let scale_y = if self.fill_template_height {
            (height as f32 - 48.0) / b.size.h.max(1.0)
        } else {
            base.scale
        };
        Ok(SurfaceFit {
            scale_x: base.scale,
            scale_y,
            origin: (base.origin.0, 24.0 - b.origin.y * scale_y),
        })
    }
    /// Project scene geometry through the same fit used by paint and picking.
    /// Hosts can place interaction affordances without owning a second camera.
    pub fn viewport_point(&self, point: Vec2, width: u32, height: u32) -> Option<Vec2> {
        if !point.x.is_finite() || !point.y.is_finite() {
            return None;
        }
        let (x, y) = self
            .fit(width, height)
            .ok()?
            .to_viewport((point.x, point.y));
        Some(Vec2::new(x, y))
    }
    /// Inverse presentation transform, including a template's independent Y fit.
    pub fn scene_point(&self, x: f32, y: f32, width: u32, height: u32) -> Option<Vec2> {
        if !x.is_finite() || !y.is_finite() {
            return None;
        }
        let fit = self.fit(width, height).ok()?;
        Some(Vec2::new(
            (x - fit.origin.0) / fit.scale_x,
            (y - fit.origin.1) / fit.scale_y,
        ))
    }
    /// Visible rectangular occurrence bounds in logical viewport pixels.
    /// This is geometry only; the source owner decides whether a span is editable.
    pub fn occurrence_rect(&self, key: &str, width: u32, height: u32) -> Option<BoardRect> {
        let mark = self
            .compiled
            .marks
            .iter()
            .find(|m| m.selectable && m.key == key)?;
        let item = &self.compiled.scene.items[mark.instance.0 as usize];
        let Footprint::Rect { size } = item.footprint else {
            return None;
        };
        let fit = self.fit(width, height).ok()?;
        let (x, y) = fit.to_viewport((item.transform.translate.x, item.transform.translate.y));
        let w = size.w * fit.scale_x;
        let h = size.h * fit.scale_y;
        Some(BoardRect::new(x - w / 2.0, y - h / 2.0, w, h))
    }
    pub fn pick(&self, x: f32, y: f32, width: u32, height: u32) -> Option<&str> {
        let world = self.scene_point(x, y, width, height)?;
        self.compiled
            .marks
            .iter()
            .rev()
            .filter(|m| m.selectable)
            .find(|m| {
                let item = &self.compiled.scene.items[m.instance.0 as usize];
                let at = item.transform.translate;
                item.hit
                    .as_ref()
                    .unwrap_or(&item.footprint)
                    .contains(Vec2::new(world.x - at.x, world.y - at.y))
            })
            .map(|m| m.key.as_str())
    }
    /// Produce a normal retained Netrender scene on the host's rendering path.
    /// The same frame can be embedded in any compatible host or exported.
    pub fn paint(
        &mut self,
        width: u32,
        height: u32,
        theme: &SurfaceTheme,
    ) -> Result<netrender::Scene, String> {
        let fit = self.fit(width, height)?;
        for c in [
            theme.ground,
            theme.text,
            theme.guide,
            theme.category,
            theme.root,
            theme.fifth,
            theme.warning,
        ] {
            if c.iter().any(|v| !v.is_finite() || !(0.0..=1.0).contains(v)) {
                return Err("invalid applied scene theme".into());
            }
        }
        let point = |p: Vec2| {
            let (x, y) = fit.to_viewport((p.x, p.y));
            LayoutPoint::new(x, y)
        };
        let mut commands = Vec::new();
        commands.push(path(
            &[
                LayoutPoint::new(0.0, 0.0),
                LayoutPoint::new(width as f32, 0.0),
                LayoutPoint::new(width as f32, height as f32),
                LayoutPoint::new(0.0, height as f32),
            ],
            true,
            Some(theme.ground),
            None,
        ));
        let selected = self
            .compiled
            .marks
            .iter()
            .find(|m| Some(m.key.as_str()) == self.selected())
            .map(|m| m.instance);
        // Measured tiers first; opaque unmeasured regions mask them afterwards.
        for tier in [
            Some(CoverageTier::Fringe),
            Some(CoverageTier::Usable),
            Some(CoverageTier::Near),
            None,
        ] {
            for field in &self.compiled.fields {
                if field.tier != tier {
                    continue;
                }
                if tier.is_some()
                    && (!self.show_coverage || field.device.as_deref() != self.selected())
                {
                    continue;
                }
                let region = &self.compiled.scene.regions[field.region];
                let Some(Footprint::Polygon { points }) = &region.contour else {
                    return Err("unsupported field footprint".into());
                };
                let points: Vec<_> = points.iter().copied().map(point).collect();
                let color = if let Some(tier) = tier {
                    alpha(
                        theme.category,
                        match tier {
                            CoverageTier::Fringe => 0.11,
                            CoverageTier::Usable => 0.13,
                            CoverageTier::Near => 0.17,
                        },
                    )
                } else {
                    theme.ground
                };
                commands.push(path(&points, true, Some(color), None));
                if tier.is_none() {
                    hatch(&mut commands, &points, theme.guide);
                }
            }
        }
        for b in self.compiled.scene.backdrops.iter().filter(|b| b.visible) {
            let Footprint::Path { points, width } = &b.footprint else {
                return Err("unsupported guide footprint".into());
            };
            commands.push(path(
                &points.iter().copied().map(point).collect::<Vec<_>>(),
                false,
                None,
                Some(stroke(theme.guide, width * fit.scale_x, None)),
            ));
        }
        if self.show_links {
            for relation in &self.compiled.scene.relations {
                let touched = selected.is_some_and(|id| id == relation.from || id == relation.to);
                let kind = relation.kind.as_deref().unwrap_or("");
                if kind == "same-resource" && !touched {
                    continue;
                }
                let color = alpha(
                    if kind == "expected.rhyme" {
                        theme.warning
                    } else {
                        theme.category
                    },
                    if touched { 0.8 } else { 0.20 },
                );
                let dash = (!matches!(kind, "authored" | "same-resource")).then_some(DashPattern {
                    intervals: vec![4.0, 4.0],
                    offset: 0.0,
                });
                let points: Vec<_> = relation.points.iter().copied().map(point).collect();
                commands.push(path(&points, false, None, Some(stroke(color, 1.0, dash))));
                if kind == "same-resource" {
                    let shifted: Vec<_> = points
                        .iter()
                        .map(|p| LayoutPoint::new(p.x + 3.0, p.y))
                        .collect();
                    commands.push(path(&shifted, false, None, Some(stroke(color, 1.0, None))));
                }
            }
        }
        let mut labels = Vec::new();
        for mark in &self.compiled.marks {
            let item = &self.compiled.scene.items[mark.instance.0 as usize];
            let at = point(item.transform.translate);
            let forward = Some(mark.key.as_str()) == self.selected();
            let color = match mark.role {
                MarkRole::Root => theme.root,
                MarkRole::Fifth => theme.fifth,
                _ => theme.category,
            };
            if mark.selectable {
                // A selectable occurrence can occupy a disclosed span. Its
                // extent remains visible while attention marks its centre.
                if let Footprint::Rect { size } = item.footprint {
                    commands.push(path(
                        &rect(at, size.w * fit.scale_x, size.h * fit.scale_y),
                        true,
                        Some(alpha(color, if forward { 0.32 } else { 0.16 })),
                        Some(stroke(alpha(color, 0.45), 1.0, None)),
                    ));
                }
                if forward && self.show_span_handles {
                    if let Footprint::Rect { size } = item.footprint {
                        let half = size.w * fit.scale_x / 2.0;
                        let inset = 2.0_f32.min(half / 3.0);
                        for x in [at.x - half + inset, at.x + half - inset] {
                            commands.push(path(
                                &rect(LayoutPoint::new(x, at.y), 2.0, size.h * fit.scale_y * 0.8),
                                true,
                                Some(color),
                                None,
                            ));
                        }
                    }
                }
                if forward {
                    let disc = circle(
                        at,
                        if self.show_span_handles
                            && matches!(item.footprint, Footprint::Rect { .. })
                        {
                            4.0
                        } else {
                            15.0
                        },
                    );
                    commands.push(path(
                        &disc,
                        true,
                        Some(theme.ground),
                        Some(stroke(color, 1.5, None)),
                    ));
                } else {
                    commands.push(path(&rect(at, 8.0, 8.0), true, Some(color), None));
                    if self.retained.contains(&mark.key) {
                        commands.push(path(
                            &circle(at, 10.0),
                            true,
                            None,
                            Some(stroke(
                                theme.guide,
                                1.0,
                                Some(DashPattern {
                                    intervals: vec![2.0, 3.0],
                                    offset: 0.0,
                                }),
                            )),
                        ));
                    }
                }
            } else if let Footprint::Rect { size } = item.footprint {
                let opacity = if mark.role == MarkRole::StrongStress {
                    0.50
                } else {
                    0.22
                };
                commands.push(path(
                    &rect(at, size.w * fit.scale_x, size.h * fit.scale_y),
                    true,
                    Some(alpha(color, opacity)),
                    None,
                ));
            }
            if let Some(p) = mark.label_at {
                let p = point(p);
                labels.push((
                    mark.key.clone(),
                    mark.label.clone(),
                    BoardRect::new(p.x - 40.0, p.y - 8.0, 80.0, 16.0),
                ));
            }
        }
        for (i, label) in self.compiled.guides.iter().enumerate() {
            let p = point(label.at);
            let width = (label.text.chars().count() as f32 * 7.5).clamp(44.0, 600.0);
            labels.push((
                format!("guide-{i}"),
                label.text.clone(),
                BoardRect::new(p.x - width / 2.0, p.y - 8.0, width, 16.0),
            ));
        }
        let [r, g, b, a] = theme.text;
        if labels.is_empty() {
            return Ok(composite_paint_layers(
                DeviceIntSize::new(width as i32, height as i32),
                &[CompositeLayer::commands_only(&commands)],
            )
            .scene);
        }
        if !self.font_ready {
            return Err("embedding host must lend a font before scene painting".into());
        }
        // The family names Tabard's interface role; `set_font_roles` supplies it.
        let sheet = format!(
            ".board-title{{position:absolute;margin:0;padding:0;font-family:var(--tabard-font-interface, IBM Plex Sans), sans-serif;font-size:13px;line-height:16px;color:rgba({},{},{},{});text-align:center;white-space:nowrap;overflow:hidden;}}",
            r * 255.0,
            g * 255.0,
            b * 255.0,
            a
        );
        let text = self
            .text
            .labels(&labels, width, height, &sheet)
            .ok_or("scene label layout refused")?;
        let layers = [
            CompositeLayer::commands_only(&commands),
            CompositeLayer {
                commands: text.commands(),
                fonts: text.fonts(),
                images: text.images(),
            },
        ];
        Ok(composite_paint_layers(DeviceIntSize::new(width as i32, height as i32), &layers).scene)
    }
}
fn alpha(mut c: [f32; 4], a: f32) -> [f32; 4] {
    c[3] *= a;
    c
}
fn stroke(c: [f32; 4], width: f32, dash: Option<DashPattern>) -> StrokeStyle {
    StrokeStyle {
        color: ColorF::new(c[0], c[1], c[2], c[3]),
        width,
        cap: StrokeCap::Round,
        join: StrokeJoin::Round,
        dash,
    }
}
fn rect(c: LayoutPoint, w: f32, h: f32) -> Vec<LayoutPoint> {
    vec![
        LayoutPoint::new(c.x - w / 2.0, c.y - h / 2.0),
        LayoutPoint::new(c.x + w / 2.0, c.y - h / 2.0),
        LayoutPoint::new(c.x + w / 2.0, c.y + h / 2.0),
        LayoutPoint::new(c.x - w / 2.0, c.y + h / 2.0),
    ]
}
fn circle(c: LayoutPoint, r: f32) -> Vec<LayoutPoint> {
    (0..40)
        .map(|i| {
            let a = i as f32 * std::f32::consts::TAU / 40.0;
            LayoutPoint::new(c.x + r * libm::cosf(a), c.y + r * libm::sinf(a))
        })
        .collect()
}
fn path(
    points: &[LayoutPoint],
    closed: bool,
    fill: Option<[f32; 4]>,
    stroke: Option<StrokeStyle>,
) -> PaintCmd {
    let mut commands = Vec::new();
    for (i, p) in points.iter().enumerate() {
        commands.push(if i == 0 {
            PathCommand::MoveTo(*p)
        } else {
            PathCommand::LineTo(*p)
        });
    }
    if closed {
        commands.push(PathCommand::Close);
    }
    let padding = stroke.as_ref().map_or(0.0, |s| s.width / 2.0);
    let bounds = LayoutRect::new(
        LayoutPoint::new(
            points.iter().map(|p| p.x).fold(f32::INFINITY, f32::min) - padding,
            points.iter().map(|p| p.y).fold(f32::INFINITY, f32::min) - padding,
        ),
        LayoutPoint::new(
            points.iter().map(|p| p.x).fold(f32::NEG_INFINITY, f32::max) + padding,
            points.iter().map(|p| p.y).fold(f32::NEG_INFINITY, f32::max) + padding,
        ),
    );
    PaintCmd::DrawPath(PathItem {
        placement: CommonPlacement::new(bounds),
        path: PathData { commands },
        fill: fill.map(|c| ColorF::new(c[0], c[1], c[2], c[3])),
        stroke,
    })
}
// Even-odd clipping of diagonal hatch lines against a supplied polygon.
fn hatch(out: &mut Vec<PaintCmd>, polygon: &[LayoutPoint], color: [f32; 4]) {
    let min = polygon
        .iter()
        .map(|p| p.x + p.y)
        .fold(f32::INFINITY, f32::min);
    let max = polygon
        .iter()
        .map(|p| p.x + p.y)
        .fold(f32::NEG_INFINITY, f32::max);
    let mut k = (min / 10.0).floor() * 10.0;
    while k <= max {
        let mut hits = Vec::new();
        for i in 0..polygon.len() {
            let a = polygon[i];
            let b = polygon[(i + 1) % polygon.len()];
            let d = b.x + b.y - a.x - a.y;
            if d.abs() > 1e-6 {
                let t = (k - a.x - a.y) / d;
                if (0.0..1.0).contains(&t) {
                    hits.push(LayoutPoint::new(
                        a.x + (b.x - a.x) * t,
                        a.y + (b.y - a.y) * t,
                    ));
                }
            }
        }
        hits.sort_by(|a, b| a.x.total_cmp(&b.x));
        for pair in hits.chunks_exact(2) {
            out.push(path(pair, false, None, Some(stroke(color, 0.7, None))));
        }
        k += 10.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn input() -> SceneInput {
        SceneInput::parse(include_str!(
            "../../../../../support/design-studies/backdrops/fretboard.json"
        ))
        .unwrap()
    }
    #[test]
    fn picking_and_refused_replacement_preserve_occurrence_identity() {
        let mut surface = BackdropSurface::new(&input()).unwrap();
        surface.select(Some("string-5-fret-3")).unwrap();
        let before = surface.scene().clone();
        let mut invalid = input();
        invalid.version = 99;
        assert!(surface.replace(&invalid).is_err());
        assert_eq!(surface.scene(), &before);
        assert_eq!(surface.selected(), Some("string-5-fret-3"));
        let mark = surface
            .scene()
            .marks
            .iter()
            .find(|m| m.key == "string-5-fret-3")
            .unwrap();
        let p = surface.scene().scene.items[mark.instance.0 as usize]
            .transform
            .translate;
        let fit = surface.fit(800, 500).unwrap();
        let (x, y) = fit.to_viewport((p.x, p.y));
        assert_eq!(surface.pick(x, y, 800, 500), Some("string-5-fret-3"));
        surface.select(Some("string-2-fret-1")).unwrap();
        assert!(surface.retained().contains("string-5-fret-3"));
        assert!(surface.select(Some("missing")).is_err());
        assert_eq!(surface.selected(), Some("string-2-fret-1"));
    }
    #[test]
    fn progression_span_is_pickable_at_its_edge_and_refusal_preserves_attention() {
        let input = SceneInput::parse(include_str!(
            "../../../../../support/design-studies/backdrops/templates/progression.json"
        ))
        .unwrap();
        let mut surface = BackdropSurface::new(&input).unwrap();
        let mark = surface
            .scene()
            .marks
            .iter()
            .find(|m| m.key == "bar-2")
            .unwrap();
        let item = &surface.scene().scene.items[mark.instance.0 as usize];
        let Footprint::Rect { size } = item.footprint else {
            panic!("span footprint required")
        };
        let p = item.transform.translate;
        let fit = surface.fit(900, 540).unwrap();
        let (x, y) = fit.to_viewport((p.x + size.w / 2.0 - 1.0, p.y));
        assert_eq!(surface.pick(x, y, 900, 540), Some("bar-2"));
        surface.select(Some("bar-2")).unwrap();
        let before = surface.scene().clone();
        let mut invalid = input;
        let scenomise::backdrops::SceneFacts::TemplatedSequence(sequence) = &mut invalid.facts
        else {
            unreachable!()
        };
        sequence.occurrences[2]
            .placement
            .as_mut()
            .unwrap()
            .span
            .length = u32::MAX;
        assert!(surface.replace(&invalid).is_err());
        assert_eq!(surface.selected(), Some("bar-2"));
        assert_eq!(surface.scene(), &before);
    }
    #[test]
    fn font_roles_reach_the_label_sheet_only_when_set() {
        let mut surface = BackdropSurface::new(&input()).unwrap();
        assert_eq!(surface.text.rooted("a{}"), "a{}");
        surface.set_font_roles("--tabard-font-interface: Probe;");
        assert!(
            surface
                .text
                .rooted("a{}")
                .starts_with(":root { --tabard-font-interface: Probe;")
        );
    }
    #[test]
    fn timeline_rows_use_height_and_stretched_spans_remain_pickable() {
        let input = SceneInput::parse(include_str!(
            "../../../../../support/design-studies/backdrops/templates/progression.json"
        ))
        .unwrap();
        let surface = BackdropSurface::new(&input).unwrap();
        let fit = surface.fit(580, 300).unwrap();
        let supplied = surface
            .scene()
            .marks
            .iter()
            .find(|m| m.key == "bar-2")
            .unwrap();
        let expected = surface
            .scene()
            .marks
            .iter()
            .find(|m| m.key == "role-2")
            .unwrap();
        let at = |m: &scenomise::backdrops::Mark| {
            surface.scene().scene.items[m.instance.0 as usize]
                .transform
                .translate
        };
        let (_, supplied_y) = fit.to_viewport((at(supplied).x, at(supplied).y));
        let (_, expected_y) = fit.to_viewport((at(expected).x, at(expected).y));
        assert!(
            expected_y - supplied_y >= 48.0,
            "rows must leave room for labels and arcs"
        );
        let item = &surface.scene().scene.items[supplied.instance.0 as usize];
        let Footprint::Rect { size } = item.footprint else {
            panic!("span")
        };
        let (x, y) = fit.to_viewport((at(supplied).x, at(supplied).y + size.h * 0.45));
        assert_eq!(surface.pick(x, y, 580, 300), Some("bar-2"));
        let fretboard = BackdropSurface::new(&super::tests::input()).unwrap();
        let fit = fretboard.fit(580, 300).unwrap();
        assert_eq!(
            fit.scale_x, fit.scale_y,
            "instrument geometry retains its aspect ratio"
        );
    }
    #[test]
    fn interaction_geometry_round_trips_across_template_consumers_and_viewports() {
        for json in [
            include_str!(
                "../../../../../support/design-studies/backdrops/templates/progression.json"
            ),
            include_str!("../../../../../support/design-studies/backdrops/templates/metre.json"),
        ] {
            let input = SceneInput::parse(json).unwrap();
            let surface = BackdropSurface::new(&input).unwrap();
            for (w, h) in [(580, 300), (1050, 300), (420, 500)] {
                for mark in surface.scene().marks.iter().filter(|m| m.selectable) {
                    let item = &surface.scene().scene.items[mark.instance.0 as usize];
                    let at = item.transform.translate;
                    let point = surface.viewport_point(at, w, h).unwrap();
                    let restored = surface.scene_point(point.x, point.y, w, h).unwrap();
                    assert!((at.x - restored.x).abs() < 0.001);
                    assert!((at.y - restored.y).abs() < 0.001);
                    if let Some(rect) = surface.occurrence_rect(&mark.key, w, h) {
                        assert!((rect.x + rect.width / 2.0 - point.x).abs() < 0.001);
                        assert!((rect.y + rect.height / 2.0 - point.y).abs() < 0.001);
                        // The visible edge is also a valid hit, with the same stretched Y fit.
                        assert_eq!(
                            surface.pick(rect.x + 0.1, point.y, w, h),
                            Some(mark.key.as_str())
                        );
                    }
                }
            }
            assert!(surface.scene_point(f32::NAN, 0.0, 580, 300).is_none());
            assert!(surface.occurrence_rect("missing", 580, 300).is_none());
            assert!(surface.scene_point(0.0, 0.0, 40, 40).is_none());
        }
    }
    #[test]
    fn painting_requires_host_fonts_and_valid_viewport() {
        let mut surface = BackdropSurface::new(&input()).unwrap();
        assert!(
            surface
                .paint(800, 500, &SurfaceTheme::default())
                .unwrap_err()
                .contains("lend a font")
        );
        assert!(
            surface
                .paint(40, 40, &SurfaceTheme::default())
                .unwrap_err()
                .contains("viewport")
        );
    }
}
