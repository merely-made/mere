// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! A forme's local tile regions embedded in the graph's world coordinates.
//! This view constraint has a Forme identity, not a fabricated scalar-field identity.
//! It holds active tile appearances while unrelated bodies keep their own law.
use super::*;
use paint_list_api::{
    ColorF, CommonPlacement, LayoutPoint, LayoutRect, PaintCmd, PathCommand, PathData, StrokeCap,
    StrokeItem, StrokeJoin,
};

#[derive(Clone, Debug, PartialEq)]
pub struct FormeCell {
    pub member: uuid::Uuid,
    pub bounds: [f32; 4],
}
#[derive(Clone, Debug, PartialEq)]
pub struct FormeRegion {
    pub id: uuid::Uuid,
    pub bounds: [f32; 4],
    pub cells: Vec<FormeCell>,
    pub locked: bool,
    pub visible: bool,
}
impl FormeRegion {
    pub fn cell_world_bounds(&self, cell: &FormeCell) -> [f32; 4] {
        let [x, y, r, b] = self.bounds;
        let [cx, cy, cr, cb] = cell.bounds;
        [
            x + cx * (r - x),
            y + cy * (b - y),
            x + cr * (r - x),
            y + cb * (b - y),
        ]
    }
    fn contains(&self, p: (f32, f32)) -> bool {
        p.0 >= self.bounds[0]
            && p.0 <= self.bounds[2]
            && p.1 >= self.bounds[1]
            && p.1 <= self.bounds[3]
    }
}
impl Canvas {
    /// Replace only this view's forme constraints. Explicit item pins retain priority.
    pub fn set_forme_region(&mut self, region: Option<FormeRegion>) -> Result<(), String> {
        if let Some(r) = &region {
            if r.bounds.iter().any(|v| !v.is_finite())
                || !(r.bounds[2] - r.bounds[0]).is_finite()
                || !(r.bounds[3] - r.bounds[1]).is_finite()
                || r.bounds[2] <= r.bounds[0]
                || r.bounds[3] <= r.bounds[1]
            {
                return Err("invalid forme extent".into());
            }
            let mut seen = HashSet::new();
            for c in &r.cells {
                if !seen.insert(c.member)
                    || c.bounds
                        .iter()
                        .any(|v| !v.is_finite() || *v < 0. || *v > 1.)
                    || c.bounds[2] <= c.bounds[0]
                    || c.bounds[3] <= c.bounds[1]
                {
                    return Err("invalid forme cell".into());
                }
                if self.graph.get_node_by_id(c.member).is_none() {
                    return Err("forme member is absent".into());
                }
            }
        }
        let same_placement =
            self.forme_region
                .as_ref()
                .zip(region.as_ref())
                .is_some_and(|(before, after)| {
                    before.bounds == after.bounds && before.cells == after.cells
                });
        if same_placement {
            self.forme_region = region;
            return Ok(());
        }
        let held: Vec<_> = self.forme_held.drain().collect();
        for key in held {
            if self.arrangement_role_of(key) != Role::Pinned {
                self.physics.unpin(key);
            }
        }
        self.forme_region = region;
        self.apply_forme_region();
        self.physics.settle(SETTLE_TICKS / 3);
        Ok(())
    }
    pub fn forme_region(&self) -> Option<&FormeRegion> {
        self.forme_region.as_ref()
    }
    pub fn forme_region_hovered(&self) -> bool {
        self.forme_region
            .as_ref()
            .is_some_and(|r| r.visible && r.contains(self.world_point_at(self.cursor)))
    }
    pub(crate) fn apply_forme_region(&mut self) {
        let Some(region) = self.forme_region.clone() else {
            return;
        };
        for cell in &region.cells {
            let Some((key, _)) = self.graph.get_node_by_id(cell.member) else {
                continue;
            };
            if self.arrangement_role_of(key) == Role::Pinned {
                continue;
            }
            let [x, y, r, b] = region.cell_world_bounds(cell);
            let at = Point2D::new(x + (r - x) / 2., y + (b - y) / 2.);
            self.physics.pin(key, at);
            self.view.set_position(key, at);
            self.forme_held.insert(key);
        }
    }
    pub(crate) fn forme_member(&self, member: uuid::Uuid) -> bool {
        self.forme_region
            .as_ref()
            .is_some_and(|r| r.cells.iter().any(|c| c.member == member))
    }
    pub(crate) fn forme_overlay(&self) -> Vec<PaintCmd> {
        let Some(region) = self.forme_region.as_ref().filter(|r| r.visible) else {
            return Vec::new();
        };
        let selected = region
            .cells
            .iter()
            .any(|c| self.selected_members().contains(&c.member));
        let guides = self.forme_region_hovered() || selected || !region.locked;
        let mut cmds = vec![outline(region.bounds, self.style.node_color, 0.28)];
        if guides {
            cmds.extend(
                region
                    .cells
                    .iter()
                    .map(|c| outline(region.cell_world_bounds(c), self.style.node_color, 0.65)),
            );
        }
        cmds
    }
}
fn outline([x, y, r, b]: [f32; 4], mut color: ColorF, alpha: f32) -> PaintCmd {
    color.a = alpha;
    let p = LayoutPoint::new;
    PaintCmd::DrawStroke(StrokeItem {
        placement: CommonPlacement::new(LayoutRect::new(p(x, y), p(r, b))),
        path: PathData {
            commands: vec![
                PathCommand::MoveTo(p(x, y)),
                PathCommand::LineTo(p(r, y)),
                PathCommand::LineTo(p(r, b)),
                PathCommand::LineTo(p(x, b)),
                PathCommand::LineTo(p(x, y)),
            ],
        },
        color,
        width: 1.5,
        cap: StrokeCap::Round,
        join: StrokeJoin::Round,
        dash: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> (Canvas, uuid::Uuid, uuid::Uuid, uuid::Uuid) {
        let mut c = Canvas::with_sample_graph();
        let ids: Vec<_> = c.graph.nodes().map(|(_, n)| n.id).take(3).collect();
        c.set_forme_region(Some(FormeRegion {
            id: uuid::Uuid::from_u128(44),
            bounds: [0., 0., 400., 240.],
            cells: vec![
                FormeCell {
                    member: ids[0],
                    bounds: [0., 0., 0.5, 1.],
                },
                FormeCell {
                    member: ids[1],
                    bounds: [0.5, 0., 1., 1.],
                },
            ],
            locked: true,
            visible: true,
        }))
        .unwrap();
        (c, ids[0], ids[1], ids[2])
    }
    #[test]
    fn local_tile_positions_hold_while_unrelated_dynamics_remain_enabled() {
        let (mut c, a, b, outside) = fixture();
        assert!(!c.physics_paused());
        let other = c.graph.get_node_key_by_id(outside).unwrap();
        assert!(!c.forme_held.contains(&other));
        for _ in 0..30 {
            c.frame(900, 600);
        }
        let at = |c: &Canvas, id| {
            let k = c.graph.get_node_key_by_id(id).unwrap();
            c.view.position_of(k).unwrap()
        };
        assert_eq!(at(&c, a), Point2D::new(100., 120.));
        assert_eq!(at(&c, b), Point2D::new(300., 120.));
        c.select_member(a);
        c.clear_selection();
        c.frame(900, 600);
        assert_eq!(at(&c, a), Point2D::new(100., 120.));
        let mut r = c.forme_region().unwrap().clone();
        r.bounds = [200., 0., 600., 240.];
        c.set_forme_region(Some(r)).unwrap();
        c.frame(900, 600);
        assert_eq!(at(&c, a), Point2D::new(300., 120.));
        assert_eq!(at(&c, b), Point2D::new(500., 120.));
    }
    #[test]
    fn explicit_pins_win_and_removing_forme_releases_only_its_holds() {
        let (mut c, a, b, _) = fixture();
        c.select_member(a);
        c.pin_focused();
        let k = c.graph.get_node_key_by_id(a).unwrap();
        let before = c.view.position_of(k).unwrap();
        let mut r = c.forme_region().unwrap().clone();
        r.bounds = [200., 0., 600., 240.];
        c.set_forme_region(Some(r)).unwrap();
        c.frame(900, 600);
        assert_eq!(c.view.position_of(k).unwrap(), before);
        c.set_forme_region(None).unwrap();
        assert!(c.forme_held.is_empty());
        assert_eq!(
            c.arrangement_role_of(c.graph().get_node_key_by_id(a).unwrap()),
            Role::Pinned
        );
        assert_ne!(
            c.arrangement_role_of(c.graph().get_node_key_by_id(b).unwrap()),
            Role::Pinned
        );
    }
    #[test]
    fn quiet_boundary_reveals_cells_on_hover_and_selection_and_hides_without_unpinning() {
        let (mut c, a, _, _) = fixture();
        c.cursor = (-10000., -10000.);
        assert_eq!(c.forme_overlay().len(), 1);
        let (x, y) = c.screen_point_of((100., 100.));
        c.cursor_moved(x, y);
        assert!(c.forme_region_hovered());
        assert_eq!(c.forme_overlay().len(), 3);
        c.cursor = (-10000., -10000.);
        c.select_member(a);
        assert_eq!(c.forme_overlay().len(), 3);
        let mut r = c.forme_region().unwrap().clone();
        r.visible = false;
        c.set_forme_region(Some(r)).unwrap();
        assert!(c.forme_overlay().is_empty());
        assert_eq!(c.forme_held.len(), 2);
    }
    #[test]
    fn invalid_region_is_atomic_and_graph_switch_clears_view_constraints() {
        let (mut c, _, _, _) = fixture();
        let before = c.forme_region().unwrap().clone();
        let mut bad = before.clone();
        bad.cells[0].member = uuid::Uuid::from_u128(9999);
        assert!(c.set_forme_region(Some(bad)).is_err());
        assert_eq!(c.forme_region(), Some(&before));
        c.set_graph(Graph::new());
        assert!(c.forme_region().is_none());
        assert!(c.forme_held.is_empty());
    }
}
