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

use crate::canvas::board_relations::{BoardRelation, Edge};
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
            BoardFootprint::Rect { width, height } => {
                Some(BoardRect::new(-width / 2.0, -height / 2.0, *width, *height))
            },
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
    /// How many other cards a fold hides behind this one: its title carries
    /// "+N" when there are any.
    pub folded: usize,
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
    /// Edges between cards, drawn under them ([`BoardRelation`]).
    pub relations: Vec<BoardRelation>,
}

/// How a board is fitted into a viewport: margins in viewport px. The board
/// is centred across the space the side margins leave and hangs from the top
/// margin; it is never scaled above 1.
///
/// The fit is also the board's camera (stack-lift D3): `zoom` and `pan` move
/// the view about the viewport's centre, and with `card_scale` set the cards
/// keep that size whatever the camera does, the camera framing where the
/// board holds the cards rather than their slots. Without it, cards scale
/// with the view, as a fitted score always has.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BoardFit {
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
    /// Frame the cards' edges ([`BoardScene::frame`]), so the margins are
    /// measured from them; `false` frames the scene's bounds alone, which for
    /// a scene whose bounds cover card centres lets cards cross the margins.
    pub frame_edges: bool,
    /// The camera's zoom over the fit, about the viewport's centre.
    pub zoom: f32,
    /// The camera's pan, in viewport px, after the zoom.
    pub pan: (f32, f32),
    /// Cards drawn at their footprint times this, whatever the camera's
    /// scale; `None` scales them with it.
    pub card_scale: Option<f32>,
}

impl BoardFit {
    pub fn new(left: f32, right: f32, top: f32, bottom: f32) -> Self {
        Self {
            left,
            right,
            top,
            bottom,
            frame_edges: true,
            zoom: 1.0,
            pan: (0.0, 0.0),
            card_scale: None,
        }
    }

    /// Cards keep their footprint times `scale` while the camera moves.
    pub fn fixed_cards(mut self, scale: f32) -> Self {
        self.card_scale = Some(scale);
        self
    }

    /// The camera at `zoom` over the fit, panned by `pan` viewport px.
    pub fn camera(mut self, zoom: f32, pan: (f32, f32)) -> Self {
        self.zoom = zoom;
        self.pan = pan;
        self
    }

    /// The scene's bounds alone are framed; see [`frame_edges`](Self::frame_edges).
    pub fn framing_bounds(mut self) -> Self {
        self.frame_edges = false;
        self
    }

    /// Score units to viewport px for `bounds` in a `width` × `height`
    /// viewport.
    pub fn transform(&self, bounds: BoardRect, width: f32, height: f32) -> BoardTransform {
        let scale = ((width - self.left - self.right) / bounds.width.max(1.0))
            .min((height - self.top - self.bottom) / bounds.height.max(1.0))
            .min(1.0);
        let origin = (
            self.left + (width - self.left - self.right - bounds.width * scale) * 0.5
                - bounds.x * scale,
            self.top - bounds.y * scale,
        );
        // The camera zooms about the viewport's centre, then pans.
        let (cx, cy) = (width * 0.5, height * 0.5);
        BoardTransform {
            scale: scale * self.zoom,
            origin: (
                cx + (origin.0 - cx) * self.zoom + self.pan.0,
                cy + (origin.1 - cy) * self.zoom + self.pan.1,
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
        (
            self.origin.0 + x * self.scale,
            self.origin.1 + y * self.scale,
        )
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

    pub(crate) fn framed(&self, fit: BoardFit, board: &PhysicsBoard) -> BoardRect {
        if fit.card_scale.is_some() {
            return self.held_frame(board);
        }
        if fit.frame_edges {
            self.frame()
        } else {
            self.bounds
        }
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
        let transform = fit.transform(self.framed(fit, board), width as f32, height as f32);
        self.cards
            .iter()
            .map(|card| {
                let (center_x, center_y) = transform.to_viewport(self.position(card, board));
                let (card_w, card_h) = card_size(card, fit.card_scale.unwrap_or(transform.scale));
                (
                    card.id.clone(),
                    card.labelled(),
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
        let mut fills = self.ground(board, width, height, fit);
        fills.extend(self.card_fills(board, width, height, fit));
        fills
    }

    /// The ground and the backdrops, which the edges are drawn over.
    fn ground(&self, board: &PhysicsBoard, width: u32, height: u32, fit: BoardFit) -> Vec<Fill> {
        let mut fills = vec![Fill([0.0, 0.0, width as f32, height as f32], BOARD_GROUND)];
        if self.cards.is_empty() && self.backdrops.is_empty() {
            return fills;
        }
        let transform = fit.transform(self.framed(fit, board), width as f32, height as f32);
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
        fills
    }

    /// The cards, over the edges.
    fn card_fills(
        &self,
        board: &PhysicsBoard,
        width: u32,
        height: u32,
        fit: BoardFit,
    ) -> Vec<Fill> {
        let mut fills = Vec::new();
        if self.cards.is_empty() {
            return fills;
        }
        let transform = fit.transform(self.framed(fit, board), width as f32, height as f32);
        let scale = transform.scale;
        for card in &self.cards {
            let (center_x, center_y) = transform.to_viewport(self.position(card, board));
            let (card_w, card_h) = card_size(card, fit.card_scale.unwrap_or(scale));
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
        for Fill([x0, y0, x1, y1], color) in self.ground(board, width, height, fit) {
            scene.push_rect(x0, y0, x1, y1, color);
        }
        for edge in self.edges(board, width, height, fit) {
            edge.push(&mut scene);
        }
        for Fill([x0, y0, x1, y1], color) in self.card_fills(board, width, height, fit) {
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
    ) -> Result<Scene, paint_list_render::TranslationError> {
        let rect = |Fill([x0, y0, x1, y1], [r, g, b, a]): Fill| {
            PaintCmd::DrawRect(RectItem {
                placement: CommonPlacement::new(LayoutRect::new(
                    LayoutPoint::new(x0, y0),
                    LayoutPoint::new(x1, y1),
                )),
                color: ColorF::new(r, g, b, a),
            })
        };
        let mut rects: Vec<PaintCmd> = self
            .ground(board, width, height, fit)
            .into_iter()
            .map(rect)
            .collect();
        rects.extend(
            self.edges(board, width, height, fit)
                .iter()
                .map(Edge::command),
        );
        rects.extend(
            self.card_fills(board, width, height, fit)
                .into_iter()
                .map(rect),
        );
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
        composite_paint_layers(viewport, &layers).map(|translated| translated.scene)
    }
}

impl BoardCard {
    /// The title as painted and named: with "+N" when a fold hides others
    /// behind this card.
    pub fn labelled(&self) -> String {
        if self.folded == 0 {
            self.title.clone()
        } else {
            format!("{} +{}", self.title, self.folded)
        }
    }
}

/// A card's drawn size: its footprint scaled, or the unsized default.
pub(crate) fn card_size(card: &BoardCard, scale: f32) -> (f32, f32) {
    match card.footprint {
        BoardFootprint::Rect { width, height } => (width * scale, height * scale),
        BoardFootprint::Other { .. } => UNSIZED_CARD,
    }
}

/// One filled rectangle, `[x0, y0, x1, y1]` in viewport px, and its colour.
pub(crate) struct Fill([f32; 4], [f32; 4]);

/// The faces a board sets its card titles in, and the retained shaping state.
/// A browser lends no system faces, so a host registers the ones it ships.
pub struct BoardText {
    text: genet_livery::TextSystem,
    generation: u64,
    font_roles: String,
}

impl Default for BoardText {
    fn default() -> Self {
        Self::new()
    }
}

/// A title's line height, px; it is centred in its card.
const TITLE_LINE: f32 = 16.0;
// The title's family names Tabard's interface role. A host publishes the
// role through `BoardText::set_font_roles`; until it does, the fallback is
// the face that renders: Plex Sans where the host lends it (Graphshell),
// else whatever the generic sans-serif resolves to among the lent faces.
const TITLE_SHEET: &str = "\
    .board-title { position: absolute; margin: 0; padding: 0 6px; box-sizing: border-box; \
      font-family: var(--tabard-font-interface, IBM Plex Sans), sans-serif; font-size: 13px; line-height: 16px; \
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
#[path = "board_scene/tests.rs"]
mod tests;

#[path = "board_scene/text.rs"]
mod text;
