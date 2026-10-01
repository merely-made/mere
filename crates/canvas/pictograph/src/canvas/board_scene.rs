// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! A board scene: what a host knows of a scene that is not a graph, painted
//! where a [`PhysicsBoard`] holds its items.
//!
//! A remote projection arrives as cards at score slots, each with a
//! footprint, over backdrops. The host describes that once as a
//! [`BoardScene`]; the board moves the cards (the slots are anchors, never
//! written back), and [`BoardScene::paint`] draws them at the board's
//! positions, fitted into the host's viewport by a [`BoardFit`]. The same
//! description gives the board its items ([`BoardScene::items`]) and counts
//! overlapping footprints ([`BoardScene::overlaps`]).

use netrender::Scene;

use crate::canvas::physics_board::{BoardItem, PhysicsBoard};

/// The board's ground, behind every backdrop.
pub const BOARD_GROUND: [f32; 4] = [0.025, 0.045, 0.057, 1.0];
/// The size, in viewport px, a card whose footprint is not a rectangle is
/// drawn at.
pub const UNSIZED_CARD: (f32, f32) = (120.0, 80.0);
const LEAD_CARD: [f32; 4] = [0.16, 0.31, 0.35, 1.0];
const CARD: [f32; 4] = [0.23, 0.28, 0.38, 1.0];
const HELD_EDGE: [f32; 4] = [0.85, 0.72, 0.35, 1.0];
const SHADOW: [f32; 4] = [0.01, 0.02, 0.025, 0.45];
const COLLIDABLE_EDGE: [f32; 4] = [0.78, 0.61, 0.31, 0.9];

/// An axis-aligned rectangle: its top-left corner and its size.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BoardRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl BoardRect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

/// A card's footprint, centred on its position.
#[derive(Clone, Debug, PartialEq)]
pub enum BoardFootprint {
    /// A rectangle in score units.
    Rect { width: f32, height: f32 },
    /// Any other shape: drawn at [`UNSIZED_CARD`], measured by its local
    /// bounds when it has an extent.
    Other { bounds: Option<BoardRect> },
}

impl BoardFootprint {
    /// Local bounds in score units, for overlap.
    pub fn bounds(&self) -> Option<BoardRect> {
        match self {
            BoardFootprint::Rect { width, height } => Some(BoardRect::new(
                -width / 2.0,
                -height / 2.0,
                *width,
                *height,
            )),
            BoardFootprint::Other { bounds } => *bounds,
        }
    }
}

impl From<&sceno::Footprint> for BoardFootprint {
    fn from(footprint: &sceno::Footprint) -> Self {
        match footprint {
            sceno::Footprint::Rect { size } => BoardFootprint::Rect {
                width: size.w,
                height: size.h,
            },
            other => BoardFootprint::Other {
                bounds: other.bounds().map(|rect| {
                    BoardRect::new(rect.origin.x, rect.origin.y, rect.size.w, rect.size.h)
                }),
            },
        }
    }
}

/// One card: its stable id, its slot (the score position), its site (the
/// grouping the Kinds law reads), its footprint, whether a person holds it
/// where it is, and whether it is the lead card.
#[derive(Clone, Debug, PartialEq)]
pub struct BoardCard {
    pub id: String,
    pub slot: (f32, f32),
    pub site: String,
    pub footprint: BoardFootprint,
    /// Drawn with a held edge, so a placed card does not look like one the
    /// arrangement happened to put there.
    pub pinned: bool,
    /// Drawn in the lead tone.
    pub lead: bool,
}

/// A region behind the cards. `kind` is open; it picks a stable fallback
/// paint ([`backdrop_color`]). A collidable backdrop gets an edge.
#[derive(Clone, Debug, PartialEq)]
pub struct BoardBackdrop {
    pub rect: BoardRect,
    pub kind: String,
    pub collidable: bool,
}

/// A board's cards and backdrops in score units, with the bounds the fit
/// frames.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BoardScene {
    pub bounds: BoardRect,
    pub backdrops: Vec<BoardBackdrop>,
    pub cards: Vec<BoardCard>,
}

/// How a board is fitted into a viewport: margins in viewport px. The board
/// is centred across the space the side margins leave and hangs from the top
/// margin; it is never scaled above 1.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BoardFit {
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
}

impl BoardFit {
    pub fn new(left: f32, right: f32, top: f32, bottom: f32) -> Self {
        Self {
            left,
            right,
            top,
            bottom,
        }
    }

    /// Score units to viewport px for `bounds` in a `width` × `height`
    /// viewport.
    pub fn transform(&self, bounds: BoardRect, width: f32, height: f32) -> BoardTransform {
        let scale = ((width - self.left - self.right) / bounds.width.max(1.0))
            .min((height - self.top - self.bottom) / bounds.height.max(1.0))
            .min(1.0);
        BoardTransform {
            scale,
            origin: (
                self.left + (width - self.left - self.right - bounds.width * scale) * 0.5
                    - bounds.x * scale,
                self.top - bounds.y * scale,
            ),
        }
    }
}

/// A uniform scale and an offset, score units to viewport px.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BoardTransform {
    pub scale: f32,
    pub origin: (f32, f32),
}

impl BoardTransform {
    pub fn to_viewport(&self, (x, y): (f32, f32)) -> (f32, f32) {
        (self.origin.0 + x * self.scale, self.origin.1 + y * self.scale)
    }
}

impl BoardScene {
    /// The board's items: one per card, at its slot.
    pub fn items(&self) -> Vec<BoardItem> {
        self.cards
            .iter()
            .map(|card| BoardItem {
                id: card.id.clone(),
                slot: card.slot,
                site: card.site.clone(),
            })
            .collect()
    }

    /// Where the board put the card, or its slot if the board has not seen
    /// it yet.
    pub fn position(&self, card: &BoardCard, board: &PhysicsBoard) -> (f32, f32) {
        board.position(&card.id).unwrap_or(card.slot)
    }

    /// Pairs of cards whose footprints share positive area at the board's
    /// positions. Touching edges are adjacent, not overlapping.
    pub fn overlaps(&self, board: &PhysicsBoard) -> usize {
        let bounds: Vec<(f32, f32, f32, f32)> = self
            .cards
            .iter()
            .filter_map(|card| {
                let (x, y) = self.position(card, board);
                let local = card.footprint.bounds()?;
                Some((
                    x + local.x,
                    y + local.y,
                    x + local.x + local.width,
                    y + local.y + local.height,
                ))
            })
            .collect();
        bounds
            .iter()
            .enumerate()
            .map(|(index, &(left, top, right, bottom))| {
                bounds[index + 1..]
                    .iter()
                    .filter(|&&(other_left, other_top, other_right, other_bottom)| {
                        left < other_right
                            && other_left < right
                            && top < other_bottom
                            && other_top < bottom
                    })
                    .count()
            })
            .sum()
    }

    /// Paint the ground, the backdrops and the cards at the board's
    /// positions into a `width` × `height` viewport.
    pub fn paint(&self, board: &PhysicsBoard, width: u32, height: u32, fit: BoardFit) -> Scene {
        let mut scene = Scene::new(width, height);
        scene.push_rect(0.0, 0.0, width as f32, height as f32, BOARD_GROUND);
        if self.cards.is_empty() && self.backdrops.is_empty() {
            return scene;
        }
        let transform = fit.transform(self.bounds, width as f32, height as f32);
        let scale = transform.scale;
        for backdrop in &self.backdrops {
            let (x0, y0) = transform.to_viewport((backdrop.rect.x, backdrop.rect.y));
            let x1 = x0 + backdrop.rect.width * scale;
            let y1 = y0 + backdrop.rect.height * scale;
            scene.push_rect(x0, y0, x1, y1, backdrop_color(&backdrop.kind));
            if backdrop.collidable {
                let stroke = 2.0;
                scene.push_rect(x0, y0, x1, y0 + stroke, COLLIDABLE_EDGE);
                scene.push_rect(x0, y1 - stroke, x1, y1, COLLIDABLE_EDGE);
                scene.push_rect(x0, y0, x0 + stroke, y1, COLLIDABLE_EDGE);
                scene.push_rect(x1 - stroke, y0, x1, y1, COLLIDABLE_EDGE);
            }
        }
        for card in &self.cards {
            let (center_x, center_y) = transform.to_viewport(self.position(card, board));
            let (card_w, card_h) = match card.footprint {
                BoardFootprint::Rect { width, height } => (width * scale, height * scale),
                BoardFootprint::Other { .. } => UNSIZED_CARD,
            };
            let (half_w, half_h) = (card_w * 0.5, card_h * 0.5);
            if card.pinned {
                // Outside the card, so it reads as something done to the
                // item rather than part of it.
                scene.push_rect(
                    center_x - half_w - 3.0,
                    center_y - half_h - 3.0,
                    center_x + half_w + 3.0,
                    center_y + half_h + 3.0,
                    HELD_EDGE,
                );
            }
            scene.push_rect(
                center_x - half_w - 5.0,
                center_y - half_h + 6.0,
                center_x + half_w + 5.0,
                center_y + half_h + 11.0,
                SHADOW,
            );
            scene.push_rect(
                center_x - half_w,
                center_y - half_h,
                center_x + half_w,
                center_y + half_h,
                if card.lead { LEAD_CARD } else { CARD },
            );
        }
        scene
    }
}

/// A stable fallback paint for an open backdrop kind: an unfamiliar scene
/// still gets a distinct, deterministic face from its own data. Hosts with
/// native art can paint over it.
pub fn backdrop_color(kind: &str) -> [f32; 4] {
    const PALETTE: [[f32; 4]; 5] = [
        [0.10, 0.19, 0.22, 1.0],
        [0.16, 0.17, 0.25, 1.0],
        [0.18, 0.14, 0.20, 1.0],
        [0.13, 0.21, 0.17, 1.0],
        [0.22, 0.18, 0.12, 1.0],
    ];
    let hash = kind.bytes().fold(0x811c_9dc5_u32, |hash, byte| {
        (hash ^ u32::from(byte)).wrapping_mul(0x0100_0193)
    });
    PALETTE[hash as usize % PALETTE.len()]
}

#[cfg(test)]
mod tests {
    use super::*;
    use netrender::SceneOp;

    fn card(id: &str, x: f32, y: f32) -> BoardCard {
        BoardCard {
            id: id.into(),
            slot: (x, y),
            site: "fixture".into(),
            footprint: BoardFootprint::Rect {
                width: 280.0,
                height: 156.0,
            },
            pinned: false,
            lead: false,
        }
    }

    fn rects(scene: &Scene) -> Vec<([f32; 4], [f32; 4])> {
        scene
            .ops
            .iter()
            .filter_map(|op| match op {
                SceneOp::Rect(rect) => Some(([rect.x0, rect.y0, rect.x1, rect.y1], rect.color)),
                _ => None,
            })
            .collect()
    }

    /// The old page's fit: 50 px each side, hung 116 px down, 64 px below.
    fn page_fit() -> BoardFit {
        BoardFit::new(50.0, 50.0, 116.0, 64.0)
    }

    #[test]
    fn the_fit_centres_across_and_hangs_from_the_top() {
        let bounds = BoardRect::new(-140.0, -78.0, 560.0, 156.0);
        let transform = page_fit().transform(bounds, 1400.0, 900.0);
        // The formula the old page drew with: (W - 100) / w, (H - 180) / h,
        // never above 1; centred in W; hung at 116.
        let scale = ((1400.0_f32 - 100.0) / 560.0).min((900.0 - 180.0) / 156.0).min(1.0);
        assert_eq!(transform.scale, scale);
        assert_eq!(
            transform.origin,
            ((1400.0 - 560.0 * scale) * 0.5 + 140.0 * scale, 116.0 + 78.0 * scale)
        );
        // A board wider than the viewport shrinks to fit.
        let wide = page_fit().transform(BoardRect::new(0.0, 0.0, 2600.0, 100.0), 1400.0, 900.0);
        assert_eq!(wide.scale, 0.5);
    }

    #[test]
    fn an_empty_board_is_ground_only() {
        let scene = BoardScene::default().paint(&PhysicsBoard::new(), 640, 480, page_fit());
        assert_eq!(rects(&scene), vec![([0.0, 0.0, 640.0, 480.0], BOARD_GROUND)]);
    }

    #[test]
    fn cards_are_drawn_where_the_board_holds_them_not_at_their_slots() {
        let scene_desc = BoardScene {
            bounds: BoardRect::new(-140.0, -78.0, 560.0, 156.0),
            backdrops: Vec::new(),
            cards: vec![card("0", 0.0, 0.0), card("1", 140.0, 0.0)],
        };
        let mut board = PhysicsBoard::new();
        // Before the board knows the cards, they are drawn at their slots.
        let before = rects(&scene_desc.paint(&board, 1400, 900, page_fit()));
        let transform = page_fit().transform(scene_desc.bounds, 1400.0, 900.0);
        let (cx, cy) = transform.to_viewport((140.0, 0.0));
        let body = before.last().unwrap().0;
        assert_eq!([(body[0] + body[2]) / 2.0, (body[1] + body[3]) / 2.0], [cx, cy]);

        board.sync(scene_desc.items());
        assert!(board.drag_start("1"));
        assert!(board.drag_move(300.0, 40.0));
        board.tick();
        let after = rects(&scene_desc.paint(&board, 1400, 900, page_fit()));
        let (hx, hy) = transform.to_viewport(board.position("1").unwrap());
        let body = after.last().unwrap().0;
        assert!(((body[0] + body[2]) / 2.0 - hx).abs() < 0.01);
        assert!(((body[1] + body[3]) / 2.0 - hy).abs() < 0.01);
        assert!((board.position("1").unwrap().0 - 300.0).abs() < 1.0);
        // The slot is untouched: the score is read, never written.
        assert_eq!(scene_desc.cards[1].slot, (140.0, 0.0));
    }

    #[test]
    fn held_cards_get_an_edge_and_the_lead_its_tone() {
        let mut lead = card("0", 0.0, 0.0);
        lead.lead = true;
        lead.pinned = true;
        let scene_desc = BoardScene {
            bounds: BoardRect::new(-140.0, -78.0, 280.0, 156.0),
            backdrops: Vec::new(),
            cards: vec![lead, card("1", 0.0, 400.0)],
        };
        let drawn = rects(&scene_desc.paint(&PhysicsBoard::new(), 1400, 900, page_fit()));
        // Ground, then held edge + shadow + lead body, then shadow + body.
        let colors: Vec<_> = drawn.iter().map(|(_, color)| *color).collect();
        assert_eq!(colors, vec![BOARD_GROUND, HELD_EDGE, SHADOW, LEAD_CARD, SHADOW, CARD]);
        let (edge, body) = (drawn[1].0, drawn[3].0);
        assert_eq!(edge[0], body[0] - 3.0);
        assert_eq!(edge[3], body[3] + 3.0);
    }

    #[test]
    fn backdrops_take_their_kind_paint_and_collidable_ones_an_edge() {
        let scene_desc = BoardScene {
            bounds: BoardRect::new(0.0, 0.0, 400.0, 200.0),
            backdrops: vec![
                BoardBackdrop {
                    rect: BoardRect::new(0.0, 0.0, 400.0, 200.0),
                    kind: "shore".into(),
                    collidable: true,
                },
                BoardBackdrop {
                    rect: BoardRect::new(10.0, 10.0, 50.0, 50.0),
                    kind: "field".into(),
                    collidable: false,
                },
            ],
            cards: Vec::new(),
        };
        let drawn = rects(&scene_desc.paint(&PhysicsBoard::new(), 1400, 900, page_fit()));
        assert_eq!(drawn.len(), 1 + 5 + 1);
        assert_eq!(drawn[1].1, backdrop_color("shore"));
        assert!(drawn[2..6].iter().all(|(_, color)| *color == COLLIDABLE_EDGE));
        assert_eq!(drawn[6].1, backdrop_color("field"));
        assert_eq!(backdrop_color("shore"), backdrop_color("shore"));
    }

    #[test]
    fn overlap_counts_shared_area_not_touching_edges() {
        let board = PhysicsBoard::new();
        let touching = BoardScene {
            cards: vec![card("a", 0.0, 0.0), card("b", 280.0, 0.0), card("c", 900.0, 0.0)],
            ..BoardScene::default()
        };
        assert_eq!(touching.overlaps(&board), 0);
        let overlapping = BoardScene {
            cards: vec![card("a", 0.0, 0.0), card("b", 279.0, 0.0)],
            ..BoardScene::default()
        };
        assert_eq!(overlapping.overlaps(&board), 1);
        // A point has no extent to share.
        let mut point = card("p", 0.0, 0.0);
        point.footprint = BoardFootprint::from(&sceno::Footprint::Point);
        let with_point = BoardScene {
            cards: vec![card("a", 0.0, 0.0), point],
            ..BoardScene::default()
        };
        assert_eq!(with_point.overlaps(&board), 0);
    }

    #[test]
    fn footprints_convert_from_sceno() {
        let rect = sceno::Footprint::Rect {
            size: sceno::Size2::new(120.0, 80.0),
        };
        assert_eq!(
            BoardFootprint::from(&rect),
            BoardFootprint::Rect {
                width: 120.0,
                height: 80.0
            }
        );
        let circle = BoardFootprint::from(&sceno::Footprint::Circle { radius: 5.0 });
        assert_eq!(
            circle.bounds(),
            Some(BoardRect::new(-5.0, -5.0, 10.0, 10.0))
        );
    }
}
