// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The board's camera and its relation-reading layout (stack-lift D3, Mark:
//! "A relation-reading force on the board"). The scene's relations become
//! the board's links, which its law reads as edges, so linked cards cluster;
//! a camera with fixed cards frames where the board holds them, and zooms
//! and pans over that instead of shrinking the cards.

use crate::canvas::board_scene::{BoardRect, BoardScene};
use crate::canvas::physics_board::PhysicsBoard;

impl BoardScene {
    /// The links the board's law reads: each drawable relation once, as an
    /// unordered pair of card ids.
    pub fn links(&self) -> Vec<(String, String)> {
        let mut seen = std::collections::BTreeSet::new();
        self.relations
            .iter()
            .filter(|r| r.from != r.to)
            .filter(|r| self.cards.iter().any(|c| c.id == r.from) && self.cards.iter().any(|c| c.id == r.to))
            .filter_map(|r| {
                let pair = if r.from < r.to { (r.from.clone(), r.to.clone()) } else { (r.to.clone(), r.from.clone()) };
                seen.insert(pair.clone()).then_some(pair)
            })
            .collect()
    }

    /// The region the board holds the cards' centres in, or the scene's
    /// bounds when it holds none: what a camera with fixed cards frames.
    pub(crate) fn held_frame(&self, board: &PhysicsBoard) -> BoardRect {
        let mut held = self.cards.iter().map(|card| self.position(card, board));
        let Some((x, y)) = held.next() else { return self.bounds };
        let (mut x0, mut y0, mut x1, mut y1) = (x, y, x, y);
        for (x, y) in held {
            x0 = x0.min(x);
            y0 = y0.min(y);
            x1 = x1.max(x);
            y1 = y1.max(y);
        }
        BoardRect::new(x0, y0, x1 - x0, y1 - y0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canvas::SETTLE_TICKS;
    use crate::canvas::board_relations::BoardRelation;
    use crate::canvas::board_scene::{BoardCard, BoardFit, BoardFootprint};

    fn card(id: &str, x: f32, y: f32) -> BoardCard {
        BoardCard {
            id: id.into(),
            title: id.into(),
            slot: (x, y),
            site: "fixture".into(),
            footprint: BoardFootprint::Rect { width: 120.0, height: 40.0 },
            pinned: false,
            lead: false,
            folded: 0,
        }
    }

    fn relate(from: &str, to: &str) -> BoardRelation {
        BoardRelation { from: from.into(), to: to.into(), kind: None }
    }

    /// Two triangles of linked cards, dealt alternately along a row: under
    /// the board's default law each triangle gathers, so a card ends nearer
    /// its own triangle than the other. Control: the same cards unlinked
    /// show no such split.
    #[test]
    fn linked_cards_cluster_and_unlinked_ones_do_not() {
        let ids = ["a0", "b0", "a1", "b1", "a2", "b2"];
        // Not quite a line: a collinear start leaves every force on that line.
        let cards: Vec<BoardCard> =
            ids.iter().enumerate().map(|(i, id)| card(id, i as f32 * 300.0, (i % 2) as f32 * 50.0)).collect();
        let triangle = |p: &str| vec![relate(&format!("{p}0"), &format!("{p}1")),
            relate(&format!("{p}1"), &format!("{p}2")), relate(&format!("{p}2"), &format!("{p}0"))];
        let split = |links: Vec<BoardRelation>| {
            let scene = BoardScene { cards: cards.clone(), relations: links, ..Default::default() };
            let mut board = PhysicsBoard::new();
            board.sync(scene.items());
            board.set_links(scene.links());
            for _ in 0..SETTLE_TICKS * 4 {
                board.tick();
            }
            let mean = |pairs: &[(&str, &str)]| pairs.iter().map(|(x, y)| board.gap(x, y).unwrap()).sum::<f32>() / pairs.len() as f32;
            let within = mean(&[("a0", "a1"), ("a1", "a2"), ("a0", "a2"), ("b0", "b1"), ("b1", "b2"), ("b0", "b2")]);
            let across = mean(&[("a0", "b0"), ("a1", "b1"), ("a2", "b2"), ("a0", "b2"), ("a2", "b0"), ("a1", "b0")]);
            (within, across)
        };
        let mut links = triangle("a");
        links.extend(triangle("b"));
        let (within, across) = split(links);
        assert!(within * 1.5 < across, "each triangle gathers: within {within}, across {across}");
        let (within, across) = split(Vec::new());
        assert!(within * 1.5 >= across, "unlinked, no split: within {within}, across {across}");
    }

    #[test]
    fn links_are_unordered_pairs_of_drawn_cards() {
        let scene = BoardScene {
            cards: vec![card("a", 0.0, 0.0), card("b", 1.0, 0.0)],
            relations: vec![relate("a", "b"), relate("b", "a"), relate("a", "gone"), relate("a", "a")],
            ..Default::default()
        };
        assert_eq!(scene.links(), vec![("a".to_string(), "b".to_string())]);
    }

    /// With fixed cards a card's drawn size is its footprint at every zoom
    /// and viewport; the camera's zoom spreads the centres about the middle.
    /// Control: without fixed cards the same zoom scales the card too.
    #[test]
    fn the_camera_zooms_the_layout_and_fixed_cards_keep_their_size() {
        let scene = BoardScene { cards: vec![card("a", 0.0, 0.0), card("b", 4000.0, 0.0)], ..Default::default() };
        let mut board = PhysicsBoard::new();
        board.sync(scene.items());
        let rects = |fit: BoardFit, width| scene.card_rects(&board, width, 600, fit);
        let fixed = BoardFit::new(20.0, 20.0, 20.0, 20.0).fixed_cards(1.0);
        for (fit, width) in [(fixed, 800), (fixed, 1600), (fixed.camera(3.0, (0.0, 0.0)), 800)] {
            for (_, _, rect) in rects(fit, width) {
                assert_eq!((rect.width, rect.height), (120.0, 40.0));
            }
        }
        let centre = |fit: BoardFit| {
            let r = rects(fit, 800);
            r[1].2.x + r[1].2.width * 0.5 - (r[0].2.x + r[0].2.width * 0.5)
        };
        let near = centre(fixed);
        let zoomed = centre(fixed.camera(2.0, (0.0, 0.0)));
        assert!((zoomed - 2.0 * near).abs() < 0.5, "zoom 2 doubles the spread: {near} then {zoomed}");
        let scaled = BoardFit::new(20.0, 20.0, 20.0, 20.0);
        assert!(rects(scaled, 800)[0].2.width < 120.0, "a fitted score's cards shrink with it");
    }
}
