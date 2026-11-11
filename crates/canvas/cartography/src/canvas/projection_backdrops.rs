// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Portable scene backdrops on a graph canvas. Paint and contact share one
//! resolved world footprint. Backdrops never become graph nodes or picks.

use super::{Canvas, backdrop_color};
use paint_list_api::{
    ColorF, CommonPlacement, LayoutPoint, LayoutRect, PaintCmd, PathCommand, PathData, PathItem,
    StrokeCap, StrokeJoin, StrokeStyle,
};
use sceno::{Footprint, Scene, SpaceId, Transform2, Vec2};
use seiche::{NodeCollider, StaticObstacle};

#[derive(Clone, Debug)]
pub(crate) struct ResolvedBackdrop {
    footprint: Footprint,
    transform: Transform2,
    kind: String,
    visible: bool,
    collidable: bool,
}

fn finite(t: Transform2) -> bool {
    t.translate.x.is_finite()
        && t.translate.y.is_finite()
        && t.scale.is_finite()
        && t.scale != 0.0
        && t.rotate.is_finite()
}

fn world(scene: &Scene, mut id: SpaceId) -> Result<Transform2, String> {
    let mut result = Transform2::IDENTITY;
    let mut seen = std::collections::HashSet::new();
    loop {
        if !seen.insert(id) {
            return Err("cyclic space ancestry".into());
        }
        let space = scene.spaces.get(id.0 as usize).ok_or("missing space")?;
        if !finite(space.transform) {
            return Err("space transform must be finite and invertible".into());
        }
        result = space.transform.then(&result);
        match space.parent {
            Some(parent) => id = parent,
            None => break,
        }
    }
    Ok(result)
}

fn points_valid(points: &[Vec2], minimum: usize) -> bool {
    points.len() >= minimum && points.iter().all(|p| p.x.is_finite() && p.y.is_finite())
}

// Every vertex must lie on the same side of every directed edge. This also
// rejects crossing polygons rather than silently filling their convex hull.
fn convex(points: &[Vec2]) -> bool {
    let mut orientation = 0.0_f32;
    for i in 0..points.len() {
        let a = points[i];
        let b = points[(i + 1) % points.len()];
        for p in points {
            let cross = (b.x - a.x) * (p.y - a.y) - (b.y - a.y) * (p.x - a.x);
            if cross.abs() < 1e-6 {
                continue;
            }
            if orientation == 0.0 {
                orientation = cross.signum();
            }
            if cross.signum() != orientation {
                return false;
            }
        }
    }
    orientation != 0.0
}

fn vertices(footprint: &Footprint) -> Vec<Vec2> {
    match footprint {
        Footprint::Rect { size } => vec![
            Vec2::new(-size.w / 2.0, -size.h / 2.0),
            Vec2::new(size.w / 2.0, -size.h / 2.0),
            Vec2::new(size.w / 2.0, size.h / 2.0),
            Vec2::new(-size.w / 2.0, size.h / 2.0),
        ],
        Footprint::Polygon { points } | Footprint::Path { points, .. } => points.clone(),
        _ => Vec::new(),
    }
}

fn obstacle(backdrop: &ResolvedBackdrop) -> Result<StaticObstacle, String> {
    let t = backdrop.transform;
    let collider = match &backdrop.footprint {
        Footprint::Circle { radius } => {
            let radius = radius * t.scale.abs();
            if radius < 1.0 {
                return Err("contact circle radius must be at least 1 world unit".into());
            }
            NodeCollider::Ball { radius }
        },
        Footprint::Rect { .. } | Footprint::Polygon { .. } => {
            let points = vertices(&backdrop.footprint);
            if !convex(&points) {
                return Err(
                    "concave or degenerate contact polygon needs a decomposition adapter".into(),
                );
            }
            NodeCollider::Hull {
                points: points
                    .iter()
                    .map(|p| (p.x * t.scale, p.y * t.scale))
                    .collect(),
                fallback: 1.0,
            }
        },
        _ => return Err("contact path or point needs a footprint adapter".into()),
    };
    Ok(StaticObstacle {
        collider,
        position: (t.translate.x, t.translate.y),
        rotation: t.rotate,
    })
}

fn resolve(scene: &Scene) -> Result<Vec<ResolvedBackdrop>, String> {
    scene
        .backdrops
        .iter()
        .enumerate()
        .map(|(index, backdrop)| {
            let read = || -> Result<ResolvedBackdrop, String> {
                if scene.sources.get(backdrop.source.0 as usize).is_none() {
                    return Err("missing source provenance".into());
                }
                if !finite(backdrop.transform) {
                    return Err("transform must be finite and invertible".into());
                }
                let transform = world(scene, backdrop.space)?.then(&backdrop.transform);
                if !finite(transform) {
                    return Err("world transform is not finite and invertible".into());
                }
                let valid = match &backdrop.footprint {
                    Footprint::Point => true,
                    Footprint::Circle { radius } => radius.is_finite() && *radius > 0.0,
                    Footprint::Rect { size } => {
                        size.w.is_finite() && size.h.is_finite() && size.w > 0.0 && size.h > 0.0
                    },
                    Footprint::Polygon { points } => points_valid(points, 3),
                    Footprint::Path { points, width } => {
                        points_valid(points, 2) && width.is_finite() && *width > 0.0
                    },
                };
                if !valid {
                    return Err("invalid footprint".into());
                }
                if let Some(bounds) = backdrop.footprint.bounds() {
                    let corners = [
                        bounds.origin,
                        Vec2::new(bounds.origin.x + bounds.size.w, bounds.origin.y),
                        Vec2::new(bounds.origin.x, bounds.origin.y + bounds.size.h),
                        Vec2::new(
                            bounds.origin.x + bounds.size.w,
                            bounds.origin.y + bounds.size.h,
                        ),
                    ];
                    if corners
                        .into_iter()
                        .map(|p| transform.apply(p))
                        .any(|p| !p.x.is_finite() || !p.y.is_finite())
                    {
                        return Err("world footprint is not finite".into());
                    }
                }
                let resolved = ResolvedBackdrop {
                    footprint: backdrop.footprint.clone(),
                    transform,
                    kind: backdrop.kind.clone(),
                    visible: backdrop.visible,
                    collidable: backdrop.collidable,
                };
                if resolved.collidable {
                    obstacle(&resolved)?;
                }
                Ok(resolved)
            };
            read().map_err(|error| format!("scene.backdrops[{index}]: {error}"))
        })
        .collect()
}

impl Canvas {
    /// Bind a portable scene's backdrop layer atomically. A refusal leaves the
    /// previous layer and obstacles intact. Invisible tangible geometry still
    /// collides; unknown appearance kinds use the existing stable kind paint.
    pub fn set_projection_backdrops(&mut self, scene: &Scene) -> Result<(), String> {
        let backdrops = resolve(scene)?;
        let obstacles = backdrops
            .iter()
            .filter(|b| b.collidable)
            .map(obstacle)
            .collect::<Result<Vec<_>, _>>()?;
        self.physics.set_static_obstacles(obstacles);
        self.projection_backdrops = backdrops;
        self.settle_physics(super::SETTLE_TICKS);
        Ok(())
    }

    pub(crate) fn projection_backdrop_commands(&self) -> Vec<PaintCmd> {
        self.projection_backdrops
            .iter()
            .filter(|b| b.visible)
            .filter_map(|b| {
                let screen = |p: Vec2| {
                    let p = b.transform.apply(p);
                    let (x, y) = self
                        .camera
                        .to_screen(kernel::geometry::PortablePoint::new(p.x, p.y));
                    LayoutPoint::new(x, y)
                };
                let mut commands = Vec::new();
                let closed = !matches!(b.footprint, Footprint::Path { .. });
                let points = match &b.footprint {
                    Footprint::Point => return None,
                    Footprint::Circle { radius } => {
                        // Four cubic arcs, transformed exactly like the contact disc.
                        let r = *radius;
                        let k = r * 0.5522848;
                        commands.push(PathCommand::MoveTo(screen(Vec2::new(r, 0.0))));
                        for (c1, c2, to) in [
                            (Vec2::new(r, k), Vec2::new(k, r), Vec2::new(0.0, r)),
                            (Vec2::new(-k, r), Vec2::new(-r, k), Vec2::new(-r, 0.0)),
                            (Vec2::new(-r, -k), Vec2::new(-k, -r), Vec2::new(0.0, -r)),
                            (Vec2::new(k, -r), Vec2::new(r, -k), Vec2::new(r, 0.0)),
                        ] {
                            commands.push(PathCommand::CurveTo {
                                control1: screen(c1),
                                control2: screen(c2),
                                to: screen(to),
                            });
                        }
                        // Bounds from the transformed disc rather than just arc endpoints.
                        let c = screen(Vec2::ZERO);
                        let r = r * b.transform.scale.abs() * self.camera.zoom;
                        vec![
                            LayoutPoint::new(c.x - r, c.y - r),
                            LayoutPoint::new(c.x + r, c.y + r),
                        ]
                    },
                    _ => {
                        let points: Vec<_> =
                            vertices(&b.footprint).into_iter().map(screen).collect();
                        for (i, p) in points.iter().enumerate() {
                            commands.push(if i == 0 {
                                PathCommand::MoveTo(*p)
                            } else {
                                PathCommand::LineTo(*p)
                            });
                        }
                        points
                    },
                };
                if closed {
                    commands.push(PathCommand::Close);
                }
                let color = backdrop_color(&b.kind);
                let color = ColorF::new(color[0], color[1], color[2], color[3]);
                let width = match &b.footprint {
                    Footprint::Path { width, .. } => {
                        width * b.transform.scale.abs() * self.camera.zoom
                    },
                    _ => 2.0,
                };
                let stroke = (!closed || b.collidable).then_some(StrokeStyle {
                    color: if b.collidable {
                        ColorF::new(0.85, 0.73, 0.39, 1.0)
                    } else {
                        color
                    },
                    width,
                    cap: StrokeCap::Round,
                    join: StrokeJoin::Round,
                    dash: None,
                });
                let padding = if stroke.is_some() { width / 2.0 } else { 0.0 };
                let min_x = points.iter().map(|p| p.x).fold(f32::INFINITY, f32::min) - padding;
                let min_y = points.iter().map(|p| p.y).fold(f32::INFINITY, f32::min) - padding;
                let max_x = points.iter().map(|p| p.x).fold(f32::NEG_INFINITY, f32::max) + padding;
                let max_y = points.iter().map(|p| p.y).fold(f32::NEG_INFINITY, f32::max) + padding;
                Some(PaintCmd::DrawPath(PathItem {
                    placement: CommonPlacement::new(LayoutRect::new(
                        LayoutPoint::new(min_x, min_y),
                        LayoutPoint::new(max_x, max_y),
                    )),
                    path: PathData { commands },
                    fill: closed.then_some(color),
                    stroke,
                }))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use euclid::default::Point2D;
    use sceno::{Backdrop, Size2, SourceRef, Space};

    fn fixture(collidable: bool, visible: bool) -> Scene {
        let mut scene = Scene::new();
        let source = scene.intern_source(SourceRef::new("fixture", "wall"));
        scene.backdrops.push(Backdrop {
            source,
            space: Scene::WORLD,
            transform: Transform2::IDENTITY,
            footprint: Footprint::Rect {
                size: Size2::new(80.0, 80.0),
            },
            kind: "props".into(),
            visible,
            collidable,
        });
        scene
    }

    #[test]
    fn tangible_and_invisible_backdrops_exclude_nodes_with_intangible_control() {
        let run = |collidable, visible| {
            let scene = fixture(collidable, visible);
            let resolved = resolve(&scene).unwrap();
            let mut sim = seiche::Simulation::new();
            let node = seiche::NodeKey::new(0);
            sim.sync_nodes([(node, Point2D::new(35.0, 0.0))]);
            sim.set_node_colliders([(node, NodeCollider::Ball { radius: 5.0 })]);
            // The same ordinary centering law runs in all three cases.
            // A completely idle body can park during deep overlap correction.
            sim.add_force(seiche::Boundary::default());
            sim.set_static_obstacles(
                resolved
                    .iter()
                    .filter(|b| b.collidable)
                    .map(obstacle)
                    .collect::<Result<_, _>>()
                    .unwrap(),
            );
            for _ in 0..super::super::SETTLE_TICKS {
                sim.tick(1.0 / 60.0);
            }
            (
                sim.position_of(node).unwrap(),
                sim.view().hit_test(Point2D::new(35.0, 0.0)),
            )
        };
        let intangible = run(false, true).0;
        let tangible = run(true, true).0;
        let invisible = run(true, false).0;
        assert!(
            intangible.x.abs() + 5.0 < 40.0,
            "the intangible control follows the law inside the wall: {intangible:?}"
        );
        assert!(
            tangible.x >= 44.5,
            "node clears wall's right edge: {tangible:?}"
        );
        assert_eq!(
            tangible, invisible,
            "paint visibility is independent of contact"
        );
    }

    #[test]
    fn nested_space_rotation_and_scale_reach_both_paint_and_contact() {
        let mut scene = fixture(true, true);
        scene.spaces.push(Space {
            parent: Some(Scene::WORLD),
            name: None,
            transform: Transform2 {
                translate: Vec2::new(100.0, 50.0),
                scale: 2.0,
                rotate: std::f32::consts::FRAC_PI_2,
            },
        });
        scene.backdrops[0].space = SpaceId(1);
        scene.backdrops[0].transform.translate = Vec2::new(10.0, 0.0);
        let resolved = resolve(&scene).unwrap();
        let obstacle = obstacle(&resolved[0]).unwrap();
        assert!((obstacle.position.0 - 100.0).abs() < 1e-4);
        assert!((obstacle.position.1 - 70.0).abs() < 1e-4);
        assert_eq!(obstacle.rotation, std::f32::consts::FRAC_PI_2);
        let NodeCollider::Hull { points, .. } = obstacle.collider else {
            panic!()
        };
        assert_eq!(points[0], (-80.0, -80.0));
        let mut canvas = Canvas::new();
        canvas.set_projection_backdrops(&scene).unwrap();
        let commands = canvas.projection_backdrop_commands();
        let PaintCmd::DrawPath(paint) = &commands[0] else {
            panic!()
        };
        let world = resolved[0].transform.apply(Vec2::new(-40.0, -40.0));
        let screen = canvas
            .camera
            .to_screen(kernel::geometry::PortablePoint::new(world.x, world.y));
        let PathCommand::MoveTo(point) = paint.path.commands[0] else {
            panic!()
        };
        assert_eq!((point.x, point.y), screen);
        assert!(paint.stroke.is_some());
    }

    #[test]
    fn unknown_kinds_paint_and_refusals_preserve_the_bound_layer() {
        let mut scene = fixture(false, true);
        scene.backdrops[0].kind = "unknown:shore".into();
        let mut canvas = Canvas::new();
        canvas.set_projection_backdrops(&scene).unwrap();
        assert_eq!(canvas.projection_backdrop_commands().len(), 1);
        scene.backdrops[0].collidable = true;
        scene.backdrops[0].footprint = Footprint::Path {
            points: vec![Vec2::ZERO, Vec2::new(5.0, 5.0)],
            width: 2.0,
        };
        assert!(
            canvas
                .set_projection_backdrops(&scene)
                .unwrap_err()
                .starts_with("scene.backdrops[0]:")
        );
        assert_eq!(canvas.projection_backdrops[0].kind, "unknown:shore");
        assert!(!canvas.projection_backdrops[0].collidable);
        scene.spaces[0].parent = Some(Scene::WORLD);
        assert!(resolve(&scene).unwrap_err().contains("cyclic"));
    }

    #[test]
    fn concave_contact_geometry_refuses_instead_of_filling_its_hull() {
        let mut scene = fixture(true, true);
        scene.backdrops[0].footprint = Footprint::Polygon {
            points: vec![
                Vec2::new(0.0, 0.0),
                Vec2::new(30.0, 0.0),
                Vec2::new(15.0, 10.0),
                Vec2::new(30.0, 30.0),
                Vec2::new(0.0, 30.0),
            ],
        };
        assert!(resolve(&scene).unwrap_err().contains("decomposition"));
        scene.backdrops[0].collidable = false;
        assert!(resolve(&scene).is_ok());
    }
}
