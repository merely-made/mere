/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Fixed-world atlas geometry for graph-canvas consumers.
//!
//! An atlas is only a projection: callers produce terrain paint and keep the
//! source scene, geographic truth, selection policy and motion policy. This
//! module makes paint, routes and footprint picking use one aspect-preserving
//! transform so a view cannot draw a field somewhere it does not pick it.

use std::sync::Arc;

use sceno::{Footprint, Rect, Vec2};
use sprigging::{ColorF, GraphViewport};

/// Caller-produced terrain or overlay paint in atlas-world coordinates.
#[derive(Clone, Debug, PartialEq)]
pub struct GraphCanvasAtlasPaint {
    pub anchor: Vec2,
    pub footprint: Footprint,
    pub fill: ColorF,
    pub stroke: Option<ColorF>,
    pub stroke_width: f32,
}

/// A clickable geographic area associated with an existing graph node.
#[derive(Clone, Debug, PartialEq)]
pub struct GraphCanvasAtlasField<Id> {
    pub id: Id,
    pub anchor: Vec2,
    pub footprint: Footprint,
    /// Higher values win. Equal priorities retain source order, giving a
    /// stable answer for overlapping parent and child areas.
    pub priority: i16,
    pub label: String,
}

/// A caller-produced fixed-world route. Endpoints travel through the same
/// projection as terrain and fields.
#[derive(Clone, Debug, PartialEq)]
pub struct GraphCanvasAtlasRoute {
    pub points: Vec<Vec2>,
    pub color: ColorF,
    pub width: f32,
}

/// A caller-shaped retained callout glyph. Points are leaf-local pixels around
/// the owning graph node's normalized position. This intentionally carries
/// outlines rather than text: the host that owns a font resource shapes text
/// once, then the atlas leaf only translates the resulting paths per frame.
#[derive(Clone, Debug, PartialEq)]
pub struct GraphCanvasAtlasCallout<Id> {
    pub id: Id,
    pub offset: Vec2,
    pub outlines: Arc<[GraphCanvasAtlasPaint]>,
    pub compound_outlines: Arc<[GraphCanvasAtlasCompoundOutline]>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GraphCanvasAtlasCompoundOutline {
    pub contours: Vec<Vec<Vec2>>,
    pub fill: ColorF,
    pub stroke: Option<ColorF>,
    pub stroke_width: f32,
}

/// Immutable atlas data suitable for caching beside a swatch.
#[derive(Clone, Debug, PartialEq)]
pub struct GraphCanvasAtlas<Id> {
    pub bounds: Rect,
    /// Shared paint data makes a terrain cache cheap to clone into a view.
    pub paint: Arc<[GraphCanvasAtlasPaint]>,
    pub fields: Vec<GraphCanvasAtlasField<Id>>,
    pub routes: Arc<[GraphCanvasAtlasRoute]>,
    pub callouts: Vec<GraphCanvasAtlasCallout<Id>>,
}

impl<Id> GraphCanvasAtlas<Id> {
    pub fn new(bounds: Rect) -> Self {
        Self {
            bounds,
            paint: Arc::from([]),
            fields: Vec::new(),
            routes: Arc::from([]),
            callouts: Vec::new(),
        }
    }

    #[must_use]
    pub fn with_paint(mut self, paint: impl Into<Arc<[GraphCanvasAtlasPaint]>>) -> Self {
        self.paint = paint.into();
        self
    }

    #[must_use]
    pub fn with_routes(mut self, routes: impl Into<Arc<[GraphCanvasAtlasRoute]>>) -> Self {
        self.routes = routes.into();
        self
    }

    #[must_use]
    pub fn with_fields(mut self, fields: Vec<GraphCanvasAtlasField<Id>>) -> Self {
        self.fields = fields;
        self
    }

    /// Supply retained label/callout outlines keyed to graph nodes.
    #[must_use]
    pub fn with_callouts(mut self, callouts: Vec<GraphCanvasAtlasCallout<Id>>) -> Self {
        self.callouts = callouts;
        self
    }

    /// Build the one transform used for every atlas layer in this view.
    pub fn view(&self, viewport: GraphViewport, width: f32, height: f32) -> GraphCanvasAtlasView {
        GraphCanvasAtlasView::new(self.bounds, viewport, width, height)
    }

    /// Convert source-world coordinates to the normalized graph coordinates
    /// retained by `GraphCanvasNode` and drag/home-motion state.
    pub fn normalize_world(&self, world: Vec2) -> (f32, f32) {
        (
            (world.x - self.bounds.origin.x) / self.bounds.size.w.abs().max(f32::EPSILON),
            (world.y - self.bounds.origin.y) / self.bounds.size.h.abs().max(f32::EPSILON),
        )
    }

    /// Convert retained normalized graph coordinates back to source-world.
    pub fn denormalize(&self, point: (f32, f32)) -> Vec2 {
        Vec2::new(
            self.bounds.origin.x + point.0 * self.bounds.size.w,
            self.bounds.origin.y + point.1 * self.bounds.size.h,
        )
    }

    /// Pick the topmost geographic area at a local canvas point. This checks
    /// the actual footprint, never its rectangular DOM region.
    pub fn pick(
        &self,
        view: GraphCanvasAtlasView,
        local: (f32, f32),
    ) -> Option<&GraphCanvasAtlasField<Id>> {
        let world = view.unproject(local)?;
        self.fields
            .iter()
            .enumerate()
            .filter(|(_, field)| {
                field.footprint.contains(Vec2::new(
                    world.x - field.anchor.x,
                    world.y - field.anchor.y,
                ))
            })
            .max_by_key(|(index, field)| (field.priority, std::cmp::Reverse(*index)))
            .map(|(_, field)| field)
    }

    /// Keyboard focus follows the same explicit overlap order as pointer pick.
    pub fn focus_order(&self) -> Vec<&GraphCanvasAtlasField<Id>> {
        let mut fields: Vec<_> = self.fields.iter().enumerate().collect();
        fields.sort_by_key(|(index, field)| (std::cmp::Reverse(field.priority), *index));
        fields.into_iter().map(|(_, field)| field).collect()
    }

    /// Route ribbons use geographic coordinates, independently of callouts.
    pub fn projected_routes(
        &self,
        view: GraphCanvasAtlasView,
    ) -> Vec<GraphCanvasAtlasProjectedShape> {
        self.routes
            .iter()
            .flat_map(|route| {
                route.points.windows(2).filter_map(move |pair| {
                    let a = view.project(pair[0]);
                    let b = view.project(pair[1]);
                    let dx = b.0 - a.0;
                    let dy = b.1 - a.1;
                    let length = dx.hypot(dy);
                    if !length.is_finite() || length <= f32::EPSILON {
                        return None;
                    }
                    let half = route.width * view.scale * 0.5;
                    let (nx, ny) = (-dy / length * half, dx / length * half);
                    Some(GraphCanvasAtlasProjectedShape {
                        points: vec![
                            (a.0 + nx, a.1 + ny),
                            (b.0 + nx, b.1 + ny),
                            (b.0 - nx, b.1 - ny),
                            (a.0 - nx, a.1 - ny),
                        ],
                        fill: route.color,
                        stroke: None,
                        stroke_width: 0.0,
                    })
                })
            })
            .collect()
    }

    /// Convert caller paint into projected polygon points for a leaf painter.
    pub fn projected_paint(
        &self,
        view: GraphCanvasAtlasView,
    ) -> Vec<GraphCanvasAtlasProjectedShape> {
        self.paint
            .iter()
            .filter_map(|paint| {
                let points = footprint_points(&paint.footprint, paint.anchor)?
                    .into_iter()
                    .map(|point| view.project(point))
                    .collect();
                Some(GraphCanvasAtlasProjectedShape {
                    points,
                    fill: paint.fill,
                    stroke: paint.stroke,
                    stroke_width: paint.stroke_width,
                })
            })
            .collect()
    }
}

impl<Id: PartialEq> GraphCanvasAtlas<Id> {
    /// Project visible field feedback. The geometry remains fixed-world while
    /// selection and hover are only a paint overlay, which keeps accessibility
    /// and pointer picking independent from transient callout motion.
    pub fn projected_field_feedback(
        &self,
        view: GraphCanvasAtlasView,
        selected: Option<&Id>,
        hovered: Option<&Id>,
    ) -> Vec<GraphCanvasAtlasProjectedShape> {
        self.fields
            .iter()
            .filter_map(|field| {
                let points = footprint_points(&field.footprint, field.anchor)?
                    .into_iter()
                    .map(|point| view.project(point))
                    .collect();
                let selected = selected == Some(&field.id);
                let hovered = hovered == Some(&field.id);
                let alpha = if selected {
                    0.24
                } else if hovered {
                    0.14
                } else {
                    0.0
                };
                Some(GraphCanvasAtlasProjectedShape {
                    points,
                    fill: ColorF {
                        r: 0.95,
                        g: 0.72,
                        b: 0.25,
                        a: alpha,
                    },
                    stroke: Some(ColorF {
                        r: if selected { 0.95 } else { 0.55 },
                        g: if selected { 0.72 } else { 0.62 },
                        b: if selected { 0.25 } else { 0.75 },
                        a: if selected || hovered { 0.95 } else { 0.45 },
                    }),
                    stroke_width: if selected { 2.0 } else { 1.0 },
                })
            })
            .collect()
    }

    /// Translate caller-shaped callout outlines to moving normalized graph
    /// nodes. No DOM rebuild is needed when a retained leaf receives new node
    /// positions from an animation tick.
    pub fn projected_callouts<Kind>(
        &self,
        view: GraphCanvasAtlasView,
        nodes: &[crate::GraphCanvasNode<Id, Kind>],
    ) -> Vec<GraphCanvasAtlasProjectedShape> {
        self.callouts
            .iter()
            .filter_map(|callout| {
                let node = nodes.iter().find(|node| node.id == callout.id)?;
                let (x, y) = view.project_normalized(node.position);
                Some(callout.outlines.iter().filter_map(move |outline| {
                    let points = footprint_points(&outline.footprint, outline.anchor)?
                        .into_iter()
                        .map(|point| {
                            (
                                x + callout.offset.x + point.x,
                                y + callout.offset.y + point.y,
                            )
                        })
                        .collect();
                    Some(GraphCanvasAtlasProjectedShape {
                        points,
                        fill: outline.fill,
                        stroke: outline.stroke,
                        stroke_width: outline.stroke_width,
                    })
                }))
            })
            .flatten()
            .collect()
    }

    pub fn projected_callout_paths<Kind>(
        &self,
        view: GraphCanvasAtlasView,
        nodes: &[crate::GraphCanvasNode<Id, Kind>],
    ) -> Vec<GraphCanvasAtlasProjectedCompoundPath> {
        self.callouts
            .iter()
            .filter_map(|callout| {
                let node = nodes.iter().find(|node| node.id == callout.id)?;
                let (x, y) = view.project_normalized(node.position);
                Some(callout.compound_outlines.iter().map(move |outline| {
                    GraphCanvasAtlasProjectedCompoundPath {
                        contours: outline
                            .contours
                            .iter()
                            .map(|contour| {
                                contour
                                    .iter()
                                    .map(|point| {
                                        (
                                            x + callout.offset.x + point.x,
                                            y + callout.offset.y + point.y,
                                        )
                                    })
                                    .collect()
                            })
                            .collect(),
                        fill: outline.fill,
                        stroke: outline.stroke,
                        stroke_width: outline.stroke_width,
                    }
                }))
            })
            .flatten()
            .collect()
    }
}

/// A leaf-local, aspect-preserving world projection.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GraphCanvasAtlasView {
    bounds: Rect,
    scale: f32,
    center: (f32, f32),
}

impl GraphCanvasAtlasView {
    fn new(bounds: Rect, viewport: GraphViewport, width: f32, height: f32) -> Self {
        let usable_w = width.max(1.0);
        let usable_h = height.max(1.0);
        let world_w = bounds.size.w.abs().max(f32::EPSILON);
        let world_h = bounds.size.h.abs().max(f32::EPSILON);
        let scale = (usable_w / world_w).min(usable_h / world_h) * viewport.zoom.max(0.01);
        Self {
            bounds,
            scale,
            center: (
                usable_w * (0.5 + viewport.pan.0),
                usable_h * (0.5 + viewport.pan.1),
            ),
        }
    }

    pub fn project(self, world: Vec2) -> (f32, f32) {
        let world_center = Vec2::new(
            self.bounds.origin.x + self.bounds.size.w / 2.0,
            self.bounds.origin.y + self.bounds.size.h / 2.0,
        );
        (
            self.center.0 + (world.x - world_center.x) * self.scale,
            self.center.1 + (world.y - world_center.y) * self.scale,
        )
    }

    /// Project an existing normalized graph/home point through this atlas.
    pub fn project_normalized(self, point: (f32, f32)) -> (f32, f32) {
        self.project(Vec2::new(
            self.bounds.origin.x + point.0 * self.bounds.size.w,
            self.bounds.origin.y + point.1 * self.bounds.size.h,
        ))
    }

    pub fn unproject(self, local: (f32, f32)) -> Option<Vec2> {
        if !self.scale.is_finite() || self.scale <= 0.0 {
            return None;
        }
        let world_center = Vec2::new(
            self.bounds.origin.x + self.bounds.size.w / 2.0,
            self.bounds.origin.y + self.bounds.size.h / 2.0,
        );
        Some(Vec2::new(
            world_center.x + (local.0 - self.center.0) / self.scale,
            world_center.y + (local.1 - self.center.1) / self.scale,
        ))
    }

    /// Inverse pointer mapping for the existing normalized graph drag API.
    pub fn unproject_normalized(self, local: (f32, f32)) -> Option<(f32, f32)> {
        let world = self.unproject(local)?;
        Some((
            (world.x - self.bounds.origin.x) / self.bounds.size.w.abs().max(f32::EPSILON),
            (world.y - self.bounds.origin.y) / self.bounds.size.h.abs().max(f32::EPSILON),
        ))
    }
}

/// A painter-ready closed shape. The module deliberately does not choose a
/// terrain renderer; a GraphCanvas host may cache this in its own paint leaf.
#[derive(Clone, Debug, PartialEq)]
pub struct GraphCanvasAtlasProjectedShape {
    pub points: Vec<(f32, f32)>,
    pub fill: ColorF,
    pub stroke: Option<ColorF>,
    pub stroke_width: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GraphCanvasAtlasProjectedCompoundPath {
    pub contours: Vec<Vec<(f32, f32)>>,
    pub fill: ColorF,
    pub stroke: Option<ColorF>,
    pub stroke_width: f32,
}

fn footprint_points(footprint: &Footprint, anchor: Vec2) -> Option<Vec<Vec2>> {
    let local = match footprint {
        Footprint::Point => return None,
        Footprint::Circle { radius } => (0..24)
            .map(|index| {
                let angle = index as f32 * std::f32::consts::TAU / 24.0;
                Vec2::new(radius * angle.cos(), radius * angle.sin())
            })
            .collect(),
        Footprint::Rect { size } => vec![
            Vec2::new(-size.w / 2.0, -size.h / 2.0),
            Vec2::new(size.w / 2.0, -size.h / 2.0),
            Vec2::new(size.w / 2.0, size.h / 2.0),
            Vec2::new(-size.w / 2.0, size.h / 2.0),
        ],
        Footprint::Polygon { points } if points.len() >= 3 => points.clone(),
        Footprint::Path { .. } | Footprint::Polygon { .. } => return None,
    };
    Some(
        local
            .into_iter()
            .map(|point| Vec2::new(point.x + anchor.x, point.y + anchor.y))
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use sceno::Size2;

    fn atlas() -> GraphCanvasAtlas<&'static str> {
        GraphCanvasAtlas::new(Rect::new(Vec2::new(0.0, 0.0), Size2::new(100.0, 50.0))).with_fields(
            vec![
                GraphCanvasAtlasField {
                    id: "region",
                    anchor: Vec2::new(50.0, 25.0),
                    footprint: Footprint::Rect {
                        size: Size2::new(80.0, 40.0),
                    },
                    priority: 0,
                    label: "Region".into(),
                },
                GraphCanvasAtlasField {
                    id: "site",
                    anchor: Vec2::new(50.0, 25.0),
                    footprint: Footprint::Polygon {
                        points: vec![
                            Vec2::new(-10.0, -10.0),
                            Vec2::new(10.0, 0.0),
                            Vec2::new(-10.0, 10.0),
                        ],
                    },
                    priority: 2,
                    label: "Site".into(),
                },
            ],
        )
    }

    #[test]
    fn projection_preserves_aspect_and_round_trips() {
        let atlas = atlas();
        let view = atlas.view(GraphViewport::default(), 300.0, 300.0);
        let a = view.project(Vec2::new(0.0, 0.0));
        let b = view.project(Vec2::new(100.0, 50.0));
        assert_eq!(b.0 - a.0, 300.0);
        assert_eq!(b.1 - a.1, 150.0);
        assert_eq!(
            view.unproject(view.project(Vec2::new(20.0, 30.0))),
            Some(Vec2::new(20.0, 30.0))
        );
        assert_eq!(atlas.normalize_world(Vec2::new(20.0, 30.0)), (0.2, 0.6));
        let world = atlas.denormalize((0.2, 0.6));
        assert!((world.x - 20.0).abs() < 0.00001 && (world.y - 30.0).abs() < 0.00001);
        for viewport in [GraphViewport::default(), GraphViewport { pan: (0.2, -0.1), zoom: 1.7 }] {
            let view = atlas.view(viewport, 300.0, 300.0);
            let back = view.unproject_normalized(view.project_normalized((0.2, 0.6))).unwrap();
            assert!((back.0 - 0.2).abs() < 0.00001 && (back.1 - 0.6).abs() < 0.00001);
        }
    }

    #[test]
    fn polygon_pick_rejects_its_bounding_box_corner_and_priority_is_stable() {
        let atlas = atlas();
        let view = atlas.view(GraphViewport::default(), 200.0, 100.0);
        assert_eq!(
            atlas
                .pick(view, view.project(Vec2::new(58.0, 17.0)))
                .map(|field| field.id),
            Some("region")
        );
        assert_eq!(
            atlas
                .pick(view, view.project(Vec2::new(55.0, 25.0)))
                .map(|field| field.id),
            Some("site")
        );
        assert_eq!(
            atlas
                .focus_order()
                .into_iter()
                .map(|field| field.id)
                .collect::<Vec<_>>(),
            vec!["site", "region"]
        );
    }

    #[test]
    fn caller_paint_becomes_closed_leaf_geometry() {
        let atlas =
            GraphCanvasAtlas::<()>::new(Rect::new(Vec2::new(0.0, 0.0), Size2::new(10.0, 10.0)))
                .with_paint(Arc::from([GraphCanvasAtlasPaint {
                    anchor: Vec2::new(5.0, 5.0),
                    footprint: Footprint::Polygon {
                        points: vec![
                            Vec2::new(-2.0, -2.0),
                            Vec2::new(2.0, 0.0),
                            Vec2::new(-2.0, 2.0),
                        ],
                    },
                    fill: ColorF {
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0,
                    },
                    stroke: None,
                    stroke_width: 0.0,
                }]));
        let shapes = atlas.projected_paint(atlas.view(GraphViewport::default(), 100.0, 100.0));
        assert_eq!(shapes.len(), 1);
        assert_eq!(shapes[0].points.len(), 3);
    }

    #[test]
    fn shaped_callout_tracks_normalized_node_without_reprojecting_the_field() {
        let atlas = GraphCanvasAtlas::new(Rect::new(Vec2::ZERO, Size2::new(100.0, 50.0)))
            .with_callouts(vec![GraphCanvasAtlasCallout {
                id: 7_u8,
                offset: Vec2::new(4.0, -3.0),
                outlines: Arc::from([GraphCanvasAtlasPaint {
                    anchor: Vec2::ZERO,
                    footprint: Footprint::Rect {
                        size: Size2::new(6.0, 4.0),
                    },
                    fill: ColorF::new(1.0, 1.0, 1.0, 1.0),
                    stroke: None,
                    stroke_width: 0.0,
                }]),
                compound_outlines: Arc::from([]),
            }]);
        let nodes = vec![crate::GraphCanvasNode {
            id: 7,
            kind: (),
            position: (0.5, 0.5),
            label: "kept accessible by the native target".into(),
            key: None,
        }];
        let view = atlas.view(GraphViewport::default(), 200.0, 100.0);
        let shapes = atlas.projected_callouts(view, &nodes);
        assert_eq!(shapes.len(), 1);
        assert_eq!(shapes[0].points[0], (101.0, 45.0));
    }
}
