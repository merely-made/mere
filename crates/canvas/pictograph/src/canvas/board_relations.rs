// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! A board's edges: a relation between two cards, drawn as a line between
//! their positions on the board, over the backdrops and under the cards. Its
//! open `kind` picks a stable paint, as a backdrop's does.

use netrender::{Scene, ScenePath};
use paint_list_api::{
    ColorF, CommonPlacement, LayoutPoint, LayoutRect, PaintCmd, PathCommand, PathData, StrokeCap,
    StrokeItem, StrokeJoin,
};

use crate::canvas::board_scene::{BoardFit, BoardScene, backdrop_color};
use crate::canvas::physics_board::PhysicsBoard;

/// An edge between two cards, by their ids. A relation naming a card the
/// scene does not hold is not drawn.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BoardRelation {
    pub from: String,
    pub to: String,
    pub kind: Option<String>,
}

/// The stroke an edge is drawn with, in viewport px.
const EDGE_WIDTH: f32 = 1.5;

/// One edge in viewport px, with its paint.
pub(crate) struct Edge {
    from: (f32, f32),
    to: (f32, f32),
    color: [f32; 4],
}

/// An edge's paint: its kind's backdrop tone, lifted so it reads on them.
fn edge_color(kind: Option<&str>) -> [f32; 4] {
    let [r, g, b, _] = backdrop_color(kind.unwrap_or(""));
    [(r * 3.0).min(1.0), (g * 3.0).min(1.0), (b * 3.0).min(1.0), 0.85]
}

impl Edge {
    pub(crate) fn push(&self, scene: &mut Scene) {
        let mut path = ScenePath::new();
        path.move_to(self.from.0, self.from.1).line_to(self.to.0, self.to.1);
        scene.push_shape_stroked(path, self.color, EDGE_WIDTH);
    }

    pub(crate) fn command(&self) -> PaintCmd {
        let (x0, y0) = (self.from.0.min(self.to.0), self.from.1.min(self.to.1));
        let (x1, y1) = (self.from.0.max(self.to.0), self.from.1.max(self.to.1));
        let [r, g, b, a] = self.color;
        PaintCmd::DrawStroke(StrokeItem {
            placement: CommonPlacement::new(LayoutRect::new(
                LayoutPoint::new(x0 - EDGE_WIDTH, y0 - EDGE_WIDTH),
                LayoutPoint::new(x1 + EDGE_WIDTH, y1 + EDGE_WIDTH),
            )),
            path: PathData {
                commands: vec![
                    PathCommand::MoveTo(LayoutPoint::new(self.from.0, self.from.1)),
                    PathCommand::LineTo(LayoutPoint::new(self.to.0, self.to.1)),
                ],
            },
            color: ColorF::new(r, g, b, a),
            width: EDGE_WIDTH,
            cap: StrokeCap::Round,
            join: StrokeJoin::Round,
            dash: None,
        })
    }
}

impl BoardScene {
    /// Each drawable relation as a line between its cards' centres at the
    /// board's positions, in viewport px.
    pub(crate) fn edges(&self, board: &PhysicsBoard, width: u32, height: u32, fit: BoardFit) -> Vec<Edge> {
        if self.relations.is_empty() {
            return Vec::new();
        }
        let transform = fit.transform(self.framed(fit, board), width as f32, height as f32);
        let at = |id: &str| {
            let card = self.cards.iter().find(|card| card.id == id)?;
            Some(transform.to_viewport(self.position(card, board)))
        };
        self.relations
            .iter()
            .filter_map(|relation| {
                Some(Edge {
                    from: at(&relation.from)?,
                    to: at(&relation.to)?,
                    color: edge_color(relation.kind.as_deref()),
                })
            })
            .collect()
    }

    /// How many relations are drawn: both their cards are on the board.
    pub fn drawn_relations(&self) -> usize {
        self.relations
            .iter()
            .filter(|r| {
                self.cards.iter().any(|c| c.id == r.from) && self.cards.iter().any(|c| c.id == r.to)
            })
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canvas::board_scene::{BoardCard, BoardFootprint};
    use netrender::SceneOp;

    fn card(id: &str, x: f32) -> BoardCard {
        BoardCard {
            id: id.into(),
            title: id.into(),
            slot: (x, 0.0),
            site: "fixture".into(),
            footprint: BoardFootprint::Rect { width: 100.0, height: 60.0 },
            pinned: false,
            lead: false,
            folded: 0,
        }
    }

    #[test]
    fn edges_draw_between_held_cards_under_them() {
        let scene = BoardScene {
            cards: vec![card("a", 0.0), card("b", 300.0)],
            relations: vec![
                BoardRelation { from: "a".into(), to: "b".into(), kind: Some("route".into()) },
                BoardRelation { from: "a".into(), to: "gone".into(), kind: None },
            ],
            ..Default::default()
        };
        assert_eq!(scene.drawn_relations(), 1);
        let painted = scene.paint(&PhysicsBoard::new(), 800, 400, BoardFit::new(0.0, 0.0, 0.0, 0.0));
        let ops: Vec<bool> = painted
            .ops
            .iter()
            .filter_map(|op| match op {
                SceneOp::Shape(_) => Some(true),
                SceneOp::Rect(_) => Some(false),
                _ => None,
            })
            .collect();
        let edge = ops.iter().position(|shape| *shape).expect("one edge is drawn");
        assert_eq!(ops.iter().filter(|shape| **shape).count(), 1);
        assert!(ops[edge + 1..].iter().any(|shape| !shape), "cards are drawn over the edge");
    }

    #[test]
    fn a_folded_card_carries_its_hidden_count() {
        let mut stand_in = card("a", 0.0);
        assert_eq!(stand_in.labelled(), "a");
        stand_in.folded = 4;
        assert_eq!(stand_in.labelled(), "a +4");
    }
}
