// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

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
    pub(super) fn contains(&self, key: &str) -> bool {
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
    pub(super) fn fit(&self, width: u32, height: u32) -> Result<SurfaceFit, String> {
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
                        Some(alpha(
                            color,
                            if forward {
                                0.58
                            } else if mark.role == MarkRole::StrongStress {
                                0.50
                            } else {
                                0.16
                            },
                        )),
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
            .map_err(|error| error.to_string())?
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
        Ok(
            composite_paint_layers(DeviceIntSize::new(width as i32, height as i32), &layers)
                .map_err(|error| error.to_string())?
                .scene,
        )
    }
}
