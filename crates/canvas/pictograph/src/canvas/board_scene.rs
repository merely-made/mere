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
use paint_list_api::{
    ColorF, CommonPlacement, DeviceIntSize, LayoutPoint, LayoutRect, PaintCmd, PaintList, RectItem,
};
use paint_list_render::{CompositeLayer, composite_paint_layers};

use crate::canvas::build::qual;

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

/// One card: its stable id, its title, its slot (the score position), its
/// site (the grouping the Kinds law reads), its footprint, whether a person
/// holds it where it is, and whether it is the lead card.
#[derive(Clone, Debug, PartialEq)]
pub struct BoardCard {
    pub id: String,
    /// What the card is called: painted in it, and its accessible name.
    pub title: String,
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

    /// The region the fit frames: the scene's bounds, grown to take in every
    /// card's footprint at its slot, so a margin is a margin from the cards'
    /// edges rather than from their centres.
    pub fn frame(&self) -> BoardRect {
        let mut extents: Vec<BoardRect> = self
            .cards
            .iter()
            .filter_map(|card| {
                let local = card.footprint.bounds()?;
                Some(BoardRect::new(
                    card.slot.0 + local.x,
                    card.slot.1 + local.y,
                    local.width,
                    local.height,
                ))
            })
            .collect();
        if self.bounds.width > 0.0 || self.bounds.height > 0.0 {
            extents.push(self.bounds);
        }
        let Some(first) = extents.first().copied() else {
            return self.bounds;
        };
        let (mut x0, mut y0) = (first.x, first.y);
        let (mut x1, mut y1) = (first.x + first.width, first.y + first.height);
        for rect in &extents[1..] {
            x0 = x0.min(rect.x);
            y0 = y0.min(rect.y);
            x1 = x1.max(rect.x + rect.width);
            y1 = y1.max(rect.y + rect.height);
        }
        BoardRect::new(x0, y0, x1 - x0, y1 - y0)
    }

    /// Each card's body in viewport px, at the board's positions, in card
    /// order: its id, its title and its rectangle. What a host names in its
    /// accessibility tree, and where the titles are painted.
    pub fn card_rects(
        &self,
        board: &PhysicsBoard,
        width: u32,
        height: u32,
        fit: BoardFit,
    ) -> Vec<(String, String, BoardRect)> {
        let transform = fit.transform(self.frame(), width as f32, height as f32);
        self.cards
            .iter()
            .map(|card| {
                let (center_x, center_y) = transform.to_viewport(self.position(card, board));
                let (card_w, card_h) = card_size(card, transform.scale);
                (
                    card.id.clone(),
                    card.title.clone(),
                    BoardRect::new(
                        center_x - card_w * 0.5,
                        center_y - card_h * 0.5,
                        card_w,
                        card_h,
                    ),
                )
            })
            .collect()
    }

    /// The ground, the backdrops and the cards as filled rectangles in
    /// viewport px, back to front.
    fn fills(&self, board: &PhysicsBoard, width: u32, height: u32, fit: BoardFit) -> Vec<Fill> {
        let mut fills = vec![Fill([0.0, 0.0, width as f32, height as f32], BOARD_GROUND)];
        if self.cards.is_empty() && self.backdrops.is_empty() {
            return fills;
        }
        let transform = fit.transform(self.frame(), width as f32, height as f32);
        let scale = transform.scale;
        for backdrop in &self.backdrops {
            let (x0, y0) = transform.to_viewport((backdrop.rect.x, backdrop.rect.y));
            let x1 = x0 + backdrop.rect.width * scale;
            let y1 = y0 + backdrop.rect.height * scale;
            fills.push(Fill([x0, y0, x1, y1], backdrop_color(&backdrop.kind)));
            if backdrop.collidable {
                let stroke = 2.0;
                fills.push(Fill([x0, y0, x1, y0 + stroke], COLLIDABLE_EDGE));
                fills.push(Fill([x0, y1 - stroke, x1, y1], COLLIDABLE_EDGE));
                fills.push(Fill([x0, y0, x0 + stroke, y1], COLLIDABLE_EDGE));
                fills.push(Fill([x1 - stroke, y0, x1, y1], COLLIDABLE_EDGE));
            }
        }
        for card in &self.cards {
            let (center_x, center_y) = transform.to_viewport(self.position(card, board));
            let (card_w, card_h) = card_size(card, scale);
            let (half_w, half_h) = (card_w * 0.5, card_h * 0.5);
            if card.pinned {
                // Outside the card, so it reads as something done to the
                // item rather than part of it.
                fills.push(Fill(
                    [
                        center_x - half_w - 3.0,
                        center_y - half_h - 3.0,
                        center_x + half_w + 3.0,
                        center_y + half_h + 3.0,
                    ],
                    HELD_EDGE,
                ));
            }
            fills.push(Fill(
                [
                    center_x - half_w - 5.0,
                    center_y - half_h + 6.0,
                    center_x + half_w + 5.0,
                    center_y + half_h + 11.0,
                ],
                SHADOW,
            ));
            fills.push(Fill(
                [
                    center_x - half_w,
                    center_y - half_h,
                    center_x + half_w,
                    center_y + half_h,
                ],
                if card.lead { LEAD_CARD } else { CARD },
            ));
        }
        fills
    }

    /// Paint the ground, the backdrops and the cards at the board's
    /// positions into a `width` × `height` viewport.
    pub fn paint(&self, board: &PhysicsBoard, width: u32, height: u32, fit: BoardFit) -> Scene {
        let mut scene = Scene::new(width, height);
        for Fill([x0, y0, x1, y1], color) in self.fills(board, width, height, fit) {
            scene.push_rect(x0, y0, x1, y1, color);
        }
        scene
    }

    /// [`paint`](Self::paint), with each card's title set in the card in
    /// `text`'s faces.
    pub fn paint_titled(
        &self,
        board: &PhysicsBoard,
        width: u32,
        height: u32,
        fit: BoardFit,
        text: &mut BoardText,
    ) -> Scene {
        let rects: Vec<PaintCmd> = self
            .fills(board, width, height, fit)
            .into_iter()
            .map(|Fill([x0, y0, x1, y1], [r, g, b, a])| {
                PaintCmd::DrawRect(RectItem {
                    placement: CommonPlacement::new(LayoutRect::new(
                        LayoutPoint::new(x0, y0),
                        LayoutPoint::new(x1, y1),
                    )),
                    color: ColorF::new(r, g, b, a),
                })
            })
            .collect();
        let viewport = DeviceIntSize::new(width as i32, height as i32);
        let titles = text.titles(&self.card_rects(board, width, height, fit), width, height);
        let mut layers = vec![CompositeLayer::commands_only(&rects)];
        if let Some(titles) = titles.as_ref() {
            layers.push(CompositeLayer {
                commands: titles.commands(),
                fonts: titles.fonts(),
                images: titles.images(),
            });
        }
        composite_paint_layers(viewport, &layers).scene
    }
}

/// A card's drawn size: its footprint scaled, or the unsized default.
fn card_size(card: &BoardCard, scale: f32) -> (f32, f32) {
    match card.footprint {
        BoardFootprint::Rect { width, height } => (width * scale, height * scale),
        BoardFootprint::Other { .. } => UNSIZED_CARD,
    }
}

/// One filled rectangle, `[x0, y0, x1, y1]` in viewport px, and its colour.
struct Fill([f32; 4], [f32; 4]);

/// The faces a board sets its card titles in, and the retained shaping state.
/// A browser lends no system faces, so a host registers the ones it ships.
pub struct BoardText {
    text: genet_livery::TextSystem,
    generation: u64,
}

impl Default for BoardText {
    fn default() -> Self {
        Self::new()
    }
}

impl BoardText {
    pub fn new() -> Self {
        Self {
            text: genet_livery::TextSystem::new(),
            generation: 0,
        }
    }

    /// Register one face, by its bytes.
    pub fn register_font(&mut self, bytes: Vec<u8>) {
        self.text.register_font_bytes(bytes);
    }

    /// Lay the titles out over their card rectangles and paint them.
    fn titles(
        &mut self,
        cards: &[(String, String, BoardRect)],
        width: u32,
        height: u32,
    ) -> Option<genet_livery::LiveryPaintList> {
        use layout_dom_api::{LayoutDom, LayoutDomMut};
        if cards.iter().all(|(_, title, _)| title.is_empty()) {
            return None;
        }
        let mut dom = genet_scripted_dom::ScriptedDom::new();
        let root = dom.document();
        for (_, title, rect) in cards.iter().filter(|(_, title, _)| !title.is_empty()) {
            let line = dom.create_element(qual("div"));
            dom.set_attribute(line, qual("class"), "board-title");
            dom.set_attribute(
                line,
                qual("style"),
                &format!(
                    "left: {:.1}px; top: {:.1}px; width: {:.1}px;",
                    rect.x,
                    rect.y + (rect.height - TITLE_LINE) * 0.5,
                    rect.width
                ),
            );
            let words = dom.create_text(title);
            dom.append_child(line, words);
            dom.append_child(root, line);
        }
        let styles = genet_livery::resolve_styles(
            &dom,
            &genet_livery::StyleSet::cambium(&[TITLE_SHEET]),
            &genet_livery::Device::screen(width as f32, height as f32),
            &genet_livery::InteractionStates::default(),
        );
        let (styles, layout) = genet_livery::layout_with_text_system(
            &dom,
            &styles,
            width as f32,
            height as f32,
            genet_livery::ViewportSizes::uniform(width as f32, height as f32),
            &mut self.text,
            &std::collections::HashMap::new(),
        )
        .ok()?;
        self.generation += 1;
        Some(genet_livery::emit_paint_list_with_text_system(
            &dom,
            &styles,
            &layout,
            DeviceIntSize::new(width as i32, height as i32),
            self.generation,
            &mut self.text,
        ))
    }
}

/// A title's line height, px; it is centred in its card.
const TITLE_LINE: f32 = 16.0;
const TITLE_SHEET: &str = "\
    .board-title { position: absolute; margin: 0; padding: 0 6px; box-sizing: border-box; \
      font-family: Roboto, sans-serif; font-size: 13px; line-height: 16px; \
      color: #f0dfb8; text-align: center; white-space: nowrap; overflow: hidden; }";

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
            title: format!("Card {id}"),
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
        let transform = page_fit().transform(scene_desc.frame(), 1400.0, 900.0);
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

    /// The live board's bounds cover the cards' centres only. Framed by those
    /// alone, a card centred on the top edge hangs half outside the margin.
    #[test]
    fn the_fit_frames_the_cards_edges_so_the_top_margin_holds() {
        let mut first = card("0", 0.0, 0.0);
        first.footprint = BoardFootprint::Rect {
            width: 120.0,
            height: 80.0,
        };
        let mut second = card("1", 140.0, 0.0);
        second.footprint = first.footprint.clone();
        let scene_desc = BoardScene {
            bounds: BoardRect::new(0.0, 0.0, 140.0, 0.0),
            backdrops: Vec::new(),
            cards: vec![first, second],
        };
        assert_eq!(scene_desc.frame(), BoardRect::new(-60.0, -40.0, 260.0, 80.0));
        let fit = BoardFit::new(24.0, 24.0, 24.0, 24.0);
        let rects = scene_desc.card_rects(&PhysicsBoard::new(), 982, 627, fit);
        assert_eq!(rects.len(), 2);
        assert!(
            rects.iter().all(|(_, _, rect)| (rect.y - 24.0).abs() < 0.01),
            "both cards start at the top margin: {rects:?}"
        );
        assert_eq!(rects[1].1, "Card 1");
    }

    #[test]
    fn titled_paint_sets_each_title_and_untitled_paint_sets_none() {
        let scene_desc = BoardScene {
            bounds: BoardRect::new(-140.0, -78.0, 560.0, 156.0),
            backdrops: Vec::new(),
            cards: vec![card("0", 0.0, 0.0), card("1", 280.0, 0.0)],
        };
        let board = PhysicsBoard::new();
        let glyphs = |scene: &Scene| {
            scene
                .ops
                .iter()
                .filter(|op| matches!(op, SceneOp::GlyphRun(_)))
                .count()
        };
        let plain = scene_desc.paint(&board, 800, 400, page_fit());
        assert_eq!(glyphs(&plain), 0);
        let mut text = BoardText::new();
        let titled = scene_desc.paint_titled(&board, 800, 400, page_fit(), &mut text);
        assert!(glyphs(&titled) >= 2, "one run per title at least");
        // The rectangles underneath are the same ones.
        let rect_count = |scene: &Scene| {
            scene
                .ops
                .iter()
                .filter(|op| matches!(op, SceneOp::Rect(_)))
                .count()
        };
        assert_eq!(rect_count(&titled), rect_count(&plain));
        // No titles, no text layer.
        let mut nameless = scene_desc.clone();
        for card in &mut nameless.cards {
            card.title.clear();
        }
        let untitled = nameless.paint_titled(&board, 800, 400, page_fit(), &mut text);
        assert_eq!(glyphs(&untitled), 0);
    }
}
