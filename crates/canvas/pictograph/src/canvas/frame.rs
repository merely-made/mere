// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The per-frame render step for [`Canvas`](crate::canvas::Canvas). Factored from
//! `lib.rs` to keep files under the workspace 600-LOC ceiling.

use std::collections::{HashMap, HashSet};

use crate::canvas::underlay::{canvas_paint_list_demoted_from_arrangement, identity_arrangement};
use genet_scripted_dom::NodeId as DomNodeId;
use kernel::geometry::PortablePoint;
use kernel::graph::NodeKey;
use layout_dom_api::LayoutDomMut;
use netrender::Scene;
use paint_list_api::{
    AlphaType, ColorF, CommonPlacement, DeviceIntSize, ExtendMode, GradientStop, IdNamespace,
    ImageItem, ImageKey, ImageRendering, ImageResource, LayoutPoint, LayoutRect, LayoutSize,
    LayoutTransform, PaintCmd, PaintList, PathCommand, PathData, PathItem, RadialGradientItem,
    RadialGradientPayload, RectItem, StrokeCap, StrokeJoin, StrokeStyle, TransformKind,
    TransformSpec,
};
use paint_list_render::{CompositeLayer, composite_paint_layers};
use seiche::NodeCollider;

use super::build::{
    background_cmds, bridge_ring_overlay, community_ring_overlay, field_overlay, marquee_rect_cmds,
    qual, set_class, set_style,
};
use super::edge_cells::link_overlay_from_segments;
use super::fold_projection::FoldProjection;
use super::{Canvas, Face, NodeShape, NodeState, PAN_DECAY};

/// Show one badge element per `(centre, text)`, made as needed and reused,
/// and hide the rest. Badges sit on the stage above every gnode.
fn place_fold_badges(
    dom: &mut genet_scripted_dom::ScriptedDom,
    stage: DomNodeId,
    pool: &mut Vec<(DomNodeId, DomNodeId, String)>,
    badges: &[((f32, f32), String)],
) {
    while pool.len() < badges.len() {
        let badge = dom.create_element(qual("div"));
        dom.set_attribute(badge, qual("class"), "gbadge");
        let text = dom.create_text("");
        dom.append_child(badge, text);
        dom.append_child(stage, badge);
        pool.push((badge, text, String::new()));
    }
    for (index, (badge, text, shown)) in pool.iter_mut().enumerate() {
        let Some(((x, y), wanted)) = badges.get(index) else {
            set_style(dom, *badge, "display: none;");
            continue;
        };
        if shown != wanted {
            dom.set_text_content(*text, wanted);
            *shown = wanted.clone();
        }
        set_style(
            dom,
            *badge,
            &format!(
                "transform: translate({}px, {}px); z-index: 2147483000;",
                x - FOLD_BADGE_HALF.0,
                y - FOLD_BADGE_HALF.1
            ),
        );
    }
}

/// Half a badge's drawn size, px, so its centre lands on its anchor.
const FOLD_BADGE_HALF: (f32, f32) = (12.0, 9.0);

impl Canvas {
    /// Why the most recent frame could not be translated, when applicable.
    pub fn last_translation_error(&self) -> Option<paint_list_render::TranslationError> {
        self.last_translation_error
    }

    /// Advance one frame at viewport `(w, h)` and return the composited content
    /// scene plus whether the host should request another frame (sim still
    /// settling, pan still gliding, or a node being dragged). Does not present.
    pub fn frame(&mut self, w: u32, h: u32) -> (Scene, bool) {
        self.reset_frame_time();
        self.frame_observed(w, h, None, &mut super::frame_profile::Unprofiled)
    }

    /// Advance and compose a frame while measuring its CPU stages. The host
    /// supplies a monotonic clock in milliseconds (for example browser
    /// `performance.now`). Ordinary [`Self::frame`] does not read any clock.
    pub fn frame_profiled(
        &mut self,
        w: u32,
        h: u32,
        now_ms: impl FnMut() -> f64,
    ) -> (Scene, bool, super::CanvasFrameProfile) {
        self.reset_frame_time();
        let mut observer = super::frame_profile::Profiled::new(now_ms);
        let (scene, moving) = self.frame_observed(w, h, None, &mut observer);
        (scene, moving, observer.profile)
    }

    /// The face layer: a palette-decoded derived vector or a textured favicon for each visible
    /// node, with any image resources the latter needs. Bare and Sprite currently add nothing;
    /// Sprite's stored data-URI still belongs to the host-facing representation path.
    ///
    /// This layer is NOT under the `.stage` camera transform (it is a bare
    /// command list, not the genet DOM), so the camera is applied here by
    /// projecting through `Camera::to_screen` (at the default camera that is
    /// `world * zoom + offset`). Favicon RGBA already has the `ImageResource` shape (RGBA8,
    /// straight alpha); derived IconVG lowers to the paint list's arbitrary paths. Both reach the
    /// host through its existing renderer contract.
    fn face_layer(
        &mut self,
        on_screen: &HashSet<NodeKey>,
        positions: &HashMap<NodeKey, PortablePoint>,
    ) -> (Vec<PaintCmd>, Vec<ImageResource>) {
        let mut face_cmds: Vec<PaintCmd> = Vec::new();
        let mut face_images: Vec<ImageResource> = Vec::new();
        for &key in on_screen {
            let Some(pos) = positions.get(&key) else {
                continue;
            };
            let face = self.node_face(key);
            let Some(bounds) = self.face_rect_at(key, *pos) else {
                continue;
            };
            if face == Face::Derived {
                let Some(address) = self.graph.get_node(key).map(|node| node.url().to_owned())
                else {
                    continue;
                };
                let side = bounds.width();
                let file = self
                    .derived_face_cache
                    .entry(crate::FaceKey::address(address))
                    .or_insert_with_key(|face| {
                        face.derive()
                            .expect("pictograph must encode every canonical node address")
                    });
                let commands = crate::canvas::derived_face::commands(
                    file,
                    self.derived_face_palette,
                    side,
                    bounds,
                )
                .expect("pictograph output must decode into flat paint-list paths");
                face_cmds.extend(commands);
                continue;
            }
            if face != Face::Favicon {
                continue;
            }
            // The node carries a reference; the pixels come from the
            // host-registered cache. An unresolved reference does not paint.
            let Some(favicon) = self
                .graph
                .get_node(key)
                .and_then(|node| node.favicon().copied())
            else {
                continue;
            };
            // Bump the LRU clock, then borrow the pixels. A painted favicon
            // shares the cache's immutable buffer with the frame sidecar.
            if !self.resolved_images.touch(&favicon.digest) {
                self.request_image(favicon);
                continue;
            }
            let Some((rgba, fav_w, fav_h)) = self.resolved_images.peek(&favicon.digest) else {
                continue;
            };
            if rgba.is_empty() || fav_w == 0 || fav_h == 0 {
                continue;
            }
            let img_key = ImageKey::new(IdNamespace(0), face_images.len() as u32);
            face_images.push(ImageResource {
                key: img_key,
                width: fav_w,
                height: fav_h,
                data: rgba.clone(),
            });
            // Billboard the favicon too: an upright screen-space square centered on the
            // node's projected anchor (not two projected corners, which would foreshorten
            // it with the ground). Matches the gnode. (Isometric camera P1.)
            // Inset within the face so the accent frames the icon: state /
            // selection must stay readable at a glance once an icon lands
            // (representations carry node identity).
            face_cmds.push(PaintCmd::DrawImage(ImageItem {
                placement: CommonPlacement::new(bounds),
                image_key: img_key,
                image_rendering: ImageRendering::Auto,
                alpha_type: AlphaType::Alpha,
                color: ColorF {
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                    a: 1.0,
                },
            }));
        }
        (face_cmds, face_images)
    }

    /// P3 fake height: a stem from each raised node's ground anchor up to its floating
    /// gnode, so the gnode reads as standing above its ground spot (where its edges meet).
    /// Composited before the gnode layer, so stems sit under the gnodes. Zero-height nodes
    /// contribute nothing, so this is empty until height-by-degree is on. (Isometric camera P3.)
    fn height_stem_cmds(
        &self,
        on_screen: &HashSet<NodeKey>,
        positions: &HashMap<NodeKey, PortablePoint>,
    ) -> Vec<PaintCmd> {
        let mut stem_cmds: Vec<PaintCmd> = Vec::new();
        for &key in on_screen {
            let Some(pos) = positions.get(&key) else {
                continue;
            };
            let Some(body) = self.node_billboard_at(key, *pos) else {
                continue;
            };
            if body.ground_anchor.y - body.center.y < 0.5 {
                continue;
            }
            let (gx, gy) = (body.ground_anchor.x, body.ground_anchor.y);
            stem_cmds.push(PaintCmd::DrawRect(RectItem {
                placement: CommonPlacement::new(LayoutRect::new(
                    LayoutPoint::new(gx - 1.0, body.center.y),
                    LayoutPoint::new(gx + 1.0, gy),
                )),
                color: ColorF {
                    r: 0.5,
                    g: 0.55,
                    b: 0.66,
                    a: 0.5,
                },
            }));
        }
        stem_cmds
    }

    /// The living backdrop, as `(abstract commands, sprite commands, sprite
    /// images)`: drifting scene-decoration bodies as soft orbs behind the graph
    /// (under the edges), projected through the camera so they recline with the
    /// iso ground. Textured props (an opt-in sprite handle that resolves in the
    /// registry) billboard a quad in their own layer above the abstract scene;
    /// the orb / polygon paints the rest. (Physics scenes P1; scene-prop sprites.)
    fn scene_body_layers(&self) -> (Vec<PaintCmd>, Vec<PaintCmd>, Vec<ImageResource>) {
        let mut scene_cmds: Vec<PaintCmd> = Vec::new();
        let mut scene_sprite_cmds: Vec<PaintCmd> = Vec::new();
        let mut scene_sprite_images: Vec<ImageResource> = Vec::new();
        for body in self.view.scene_bodies() {
            // A prop wearing a registered sprite paints as a textured billboard over its footprint
            // (sized to its collider half-extent), oriented to the prop's rotation - upright when it
            // rests, tumbling when it tumbles - via a PushTransform spinning the quad about the prop's
            // projected anchor (identity at rest, so a resting prop is unrotated). Skips the abstract
            // paint; an unregistered handle falls through. (Scene-prop sprites - iso billboard.)
            if let Some(handle) = &body.sprite {
                if let Some((rgba, iw, ih)) = self.scene_sprite_textures.get(handle) {
                    if !rgba.is_empty() && *iw > 0 && *ih > 0 {
                        let (cx, cy) = self
                            .camera
                            .to_screen(PortablePoint::new(body.position.x, body.position.y));
                        let r = (scene_body_half(&body.collider) * self.camera.zoom).max(1.0);
                        let img_key =
                            ImageKey::new(IdNamespace(2), scene_sprite_images.len() as u32);
                        scene_sprite_images.push(ImageResource {
                            key: img_key,
                            width: *iw,
                            height: *ih,
                            data: rgba.clone(),
                        });
                        // Push a frame centred on the prop, rotated by its angle; draw the quad in
                        // that local frame (centred on the origin) so it spins with the prop.
                        scene_sprite_cmds.push(PaintCmd::PushTransform(TransformSpec {
                            origin: LayoutPoint::new(cx, cy),
                            transform: LayoutTransform::rotation(
                                0.0,
                                0.0,
                                1.0,
                                euclid::Angle::radians(body.rotation),
                            ),
                            kind: TransformKind::Standard,
                        }));
                        scene_sprite_cmds.push(PaintCmd::DrawImage(ImageItem {
                            placement: CommonPlacement::new(LayoutRect::new(
                                LayoutPoint::new(-r, -r),
                                LayoutPoint::new(r, r),
                            )),
                            image_key: img_key,
                            image_rendering: ImageRendering::Auto,
                            alpha_type: AlphaType::Alpha,
                            color: ColorF {
                                r: 1.0,
                                g: 1.0,
                                b: 1.0,
                                a: 1.0,
                            },
                        }));
                        scene_sprite_cmds.push(PaintCmd::PopTransform);
                        continue;
                    }
                }
            }
            // A ball (or a degenerate hull) paints as a soft radial-gradient orb — the calm
            // backdrop look. A square / rounded-square / hull paints as a filled polygon of its
            // (rotated) corners, projected per-corner through the camera so the shape reclines
            // with the iso ground and shows the body's true orientation. (Physics scenes P4b.)
            let polygon: Option<Vec<(f32, f32)>> = match &body.collider {
                NodeCollider::Ball { .. } => None,
                NodeCollider::Square { half } | NodeCollider::RoundedSquare { half, .. } => {
                    Some(vec![
                        (-half, -half),
                        (*half, -half),
                        (*half, *half),
                        (-half, *half),
                    ])
                },
                NodeCollider::Hull { points, .. } if points.len() >= 3 => Some(points.clone()),
                NodeCollider::Hull { .. } => None,
            };
            match polygon {
                Some(corners) => {
                    let (s, c) = body.rotation.sin_cos();
                    let mut commands = Vec::with_capacity(corners.len() + 1);
                    let (mut min_x, mut min_y, mut max_x, mut max_y) =
                        (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
                    for (i, &(lx, ly)) in corners.iter().enumerate() {
                        let wx = body.position.x + (lx * c - ly * s);
                        let wy = body.position.y + (lx * s + ly * c);
                        let (sx, sy) = self.camera.to_screen(PortablePoint::new(wx, wy));
                        min_x = min_x.min(sx);
                        min_y = min_y.min(sy);
                        max_x = max_x.max(sx);
                        max_y = max_y.max(sy);
                        let p = LayoutPoint::new(sx, sy);
                        commands.push(if i == 0 {
                            PathCommand::MoveTo(p)
                        } else {
                            PathCommand::LineTo(p)
                        });
                    }
                    commands.push(PathCommand::Close);
                    scene_cmds.push(PaintCmd::DrawPath(PathItem {
                        placement: CommonPlacement::new(LayoutRect::new(
                            LayoutPoint::new(min_x, min_y),
                            LayoutPoint::new(max_x, max_y),
                        )),
                        path: PathData { commands },
                        // A softened fill (so the big floor slabs no longer read as solid walls) plus
                        // a lighter lit edge, so a block / plank / domino has definition and a touch
                        // of dimension instead of blurring into the backdrop. (Scene polish.)
                        fill: Some(ColorF::new(0.42, 0.55, 0.85, 0.40)),
                        stroke: Some(StrokeStyle {
                            color: ColorF::new(0.64, 0.75, 0.98, 0.7),
                            width: 1.5,
                            cap: StrokeCap::Round,
                            join: StrokeJoin::Round,
                            dash: None,
                        }),
                    }));
                },
                None => {
                    let radius = match &body.collider {
                        NodeCollider::Ball { radius } => *radius,
                        NodeCollider::Hull { fallback, .. } => *fallback,
                        _ => continue,
                    };
                    let (cx, cy) = self
                        .camera
                        .to_screen(PortablePoint::new(body.position.x, body.position.y));
                    let r = (radius * self.camera.zoom).max(1.0);
                    let rect = LayoutRect::new(
                        LayoutPoint::new(cx - r, cy - r),
                        LayoutPoint::new(cx + r, cy + r),
                    );
                    scene_cmds.push(PaintCmd::DrawRadialGradient(RadialGradientItem {
                        placement: CommonPlacement::new(rect),
                        gradient: RadialGradientPayload {
                            center: LayoutPoint::new(cx, cy),
                            radius: LayoutSize::new(r, r),
                            extend_mode: ExtendMode::Clamp,
                            // A brighter core fading through a mid-stop to transparent, so a ball reads
                            // as a rounder, more present orb than a flat low-alpha disc. (Scene polish.)
                            stops: vec![
                                GradientStop {
                                    offset: 0.0,
                                    color: ColorF::new(0.54, 0.66, 0.94, 0.32),
                                },
                                GradientStop {
                                    offset: 0.55,
                                    color: ColorF::new(0.44, 0.57, 0.87, 0.18),
                                },
                                GradientStop {
                                    offset: 1.0,
                                    color: ColorF::new(0.42, 0.55, 0.85, 0.0),
                                },
                            ],
                        },
                        tile_size: LayoutSize::new(2.0 * r, 2.0 * r),
                        tile_spacing: LayoutSize::zero(),
                    }));
                },
            }
        }
        (scene_cmds, scene_sprite_cmds, scene_sprite_images)
    }

    /// Liquid pool: each PBF particle a soft watery orb; overlapping soft-alpha gradients read as
    /// a connected blob (a true iso-surface threshold is later polish). Painted above the backdrop
    /// scene, below the graph, and projected through the camera so it reclines with the iso ground.
    /// (Physics scenes P4c.)
    fn fluid_cmds(&self) -> Vec<PaintCmd> {
        let mut fluid_cmds: Vec<PaintCmd> = Vec::new();
        let fr = self.view.fluid_radius();
        for p in self.view.fluid_particles() {
            let (cx, cy) = self.camera.to_screen(PortablePoint::new(p.x, p.y));
            let r = (fr * 2.2 * self.camera.zoom).max(1.5);
            let rect = LayoutRect::new(
                LayoutPoint::new(cx - r, cy - r),
                LayoutPoint::new(cx + r, cy + r),
            );
            fluid_cmds.push(PaintCmd::DrawRadialGradient(RadialGradientItem {
                placement: CommonPlacement::new(rect),
                gradient: RadialGradientPayload {
                    center: LayoutPoint::new(cx, cy),
                    radius: LayoutSize::new(r, r),
                    extend_mode: ExtendMode::Clamp,
                    stops: vec![
                        GradientStop {
                            offset: 0.0,
                            color: ColorF::new(0.30, 0.62, 0.95, 0.5),
                        },
                        GradientStop {
                            offset: 1.0,
                            color: ColorF::new(0.30, 0.62, 0.95, 0.0),
                        },
                    ],
                },
                tile_size: LayoutSize::new(2.0 * r, 2.0 * r),
                tile_spacing: LayoutSize::zero(),
            }));
        }
        fluid_cmds
    }
}

/// The half-extent (world units) of a scene prop's collider, for sizing a sprite billboard over its
/// footprint. A hull takes the largest corner offset (bounding its outline). (Scene-prop sprites.)
fn scene_body_half(collider: &NodeCollider) -> f32 {
    match collider {
        NodeCollider::Ball { radius } => *radius,
        NodeCollider::Square { half } | NodeCollider::RoundedSquare { half, .. } => *half,
        NodeCollider::Hull { points, fallback } => points
            .iter()
            .map(|&(x, y)| x.abs().max(y.abs()))
            .fold(*fallback, f32::max),
    }
}

#[cfg(test)]
#[path = "frame/tests.rs"]
mod tests;

#[path = "frame/render_loop.rs"]
mod render_loop;
