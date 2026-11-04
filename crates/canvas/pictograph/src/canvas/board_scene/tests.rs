// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

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
        folded: 0,
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
    let scale = ((1400.0_f32 - 100.0) / 560.0)
        .min((900.0 - 180.0) / 156.0)
        .min(1.0);
    assert_eq!(transform.scale, scale);
    assert_eq!(
        transform.origin,
        (
            (1400.0 - 560.0 * scale) * 0.5 + 140.0 * scale,
            116.0 + 78.0 * scale
        )
    );
    // A board wider than the viewport shrinks to fit.
    let wide = page_fit().transform(BoardRect::new(0.0, 0.0, 2600.0, 100.0), 1400.0, 900.0);
    assert_eq!(wide.scale, 0.5);
}

#[test]
fn an_empty_board_is_ground_only() {
    let scene = BoardScene::default().paint(&PhysicsBoard::new(), 640, 480, page_fit());
    assert_eq!(
        rects(&scene),
        vec![([0.0, 0.0, 640.0, 480.0], BOARD_GROUND)]
    );
}

#[test]
fn cards_are_drawn_where_the_board_holds_them_not_at_their_slots() {
    let scene_desc = BoardScene {
        relations: Vec::new(),
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
    assert_eq!(
        [(body[0] + body[2]) / 2.0, (body[1] + body[3]) / 2.0],
        [cx, cy]
    );

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
        relations: Vec::new(),
        bounds: BoardRect::new(-140.0, -78.0, 280.0, 156.0),
        backdrops: Vec::new(),
        cards: vec![lead, card("1", 0.0, 400.0)],
    };
    let drawn = rects(&scene_desc.paint(&PhysicsBoard::new(), 1400, 900, page_fit()));
    // Ground, then held edge + shadow + lead body, then shadow + body.
    let colors: Vec<_> = drawn.iter().map(|(_, color)| *color).collect();
    assert_eq!(
        colors,
        vec![BOARD_GROUND, HELD_EDGE, SHADOW, LEAD_CARD, SHADOW, CARD]
    );
    let (edge, body) = (drawn[1].0, drawn[3].0);
    assert_eq!(edge[0], body[0] - 3.0);
    assert_eq!(edge[3], body[3] + 3.0);
}

#[test]
fn backdrops_take_their_kind_paint_and_collidable_ones_an_edge() {
    let scene_desc = BoardScene {
        relations: Vec::new(),
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
    assert!(
        drawn[2..6]
            .iter()
            .all(|(_, color)| *color == COLLIDABLE_EDGE)
    );
    assert_eq!(drawn[6].1, backdrop_color("field"));
    assert_eq!(backdrop_color("shore"), backdrop_color("shore"));
}

#[test]
fn overlap_counts_shared_area_not_touching_edges() {
    let board = PhysicsBoard::new();
    let touching = BoardScene {
        relations: Vec::new(),
        cards: vec![
            card("a", 0.0, 0.0),
            card("b", 280.0, 0.0),
            card("c", 900.0, 0.0),
        ],
        ..BoardScene::default()
    };
    assert_eq!(touching.overlaps(&board), 0);
    let overlapping = BoardScene {
        relations: Vec::new(),
        cards: vec![card("a", 0.0, 0.0), card("b", 279.0, 0.0)],
        ..BoardScene::default()
    };
    assert_eq!(overlapping.overlaps(&board), 1);
    // A point has no extent to share.
    let mut point = card("p", 0.0, 0.0);
    point.footprint = BoardFootprint::from(&sceno::Footprint::Point);
    let with_point = BoardScene {
        relations: Vec::new(),
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
        relations: Vec::new(),
        bounds: BoardRect::new(0.0, 0.0, 140.0, 0.0),
        backdrops: Vec::new(),
        cards: vec![first, second],
    };
    assert_eq!(
        scene_desc.frame(),
        BoardRect::new(-60.0, -40.0, 260.0, 80.0)
    );
    let fit = BoardFit::new(24.0, 24.0, 24.0, 24.0);
    let rects = scene_desc.card_rects(&PhysicsBoard::new(), 982, 627, fit);
    assert_eq!(rects.len(), 2);
    assert!(
        rects
            .iter()
            .all(|(_, _, rect)| (rect.y - 24.0).abs() < 0.01),
        "both cards start at the top margin: {rects:?}"
    );
    assert_eq!(rects[1].1, "Card 1");
    // Framing the bounds alone, as the old page does, the cards cross the
    // top margin by half their height.
    let old = scene_desc.card_rects(&PhysicsBoard::new(), 982, 627, fit.framing_bounds());
    assert!(
        old.iter()
            .all(|(_, _, rect)| (rect.y - (24.0 - 40.0)).abs() < 0.01),
        "{old:?}"
    );
}

#[test]
fn titled_paint_sets_each_title_and_untitled_paint_sets_none() {
    let scene_desc = BoardScene {
        relations: Vec::new(),
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
    let titled = scene_desc
        .paint_titled(&board, 800, 400, page_fit(), &mut text)
        .unwrap();
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
    let untitled = nameless
        .paint_titled(&board, 800, 400, page_fit(), &mut text)
        .unwrap();
    assert_eq!(glyphs(&untitled), 0);
}

#[test]
fn font_roles_ride_the_root_and_default_leaves_the_sheet_alone() {
    let mut text = BoardText::new();
    assert_eq!(text.rooted(TITLE_SHEET), TITLE_SHEET);
    text.set_font_roles("--tabard-font-interface: \"Probe Sans\";");
    let rooted = text.rooted(TITLE_SHEET);
    assert!(rooted.starts_with(":root { --tabard-font-interface: \"Probe Sans\";"));
    assert!(rooted.ends_with(TITLE_SHEET));
    text.set_font_roles("");
    assert_eq!(text.rooted(TITLE_SHEET), TITLE_SHEET);
    // A title is a root of its own document, so the declaration reaches it.
    let card = [("0".into(), "t".into(), BoardRect::new(0.0, 0.0, 80.0, 16.0))];
    let probe = ".board-title { position: absolute; color: var(--probe, #00ff00); }";
    let paint = |text: &mut BoardText| {
        format!(
            "{:?}",
            text.labels(&card, 80, 16, probe).unwrap().commands()
        )
    };
    let before = paint(&mut text);
    text.set_font_roles("--probe: #ff0000;");
    assert_ne!(paint(&mut text), before);
}

/// A title is one non-wrapping line, centred in its card. Livery ignored
/// `text-align` on a nowrap line until genet `6fca091dc26`, and the titles
/// sat at the card's left padding.
#[test]
fn titles_are_centred_in_their_cards() {
    let scene_desc = BoardScene {
        relations: Vec::new(),
        bounds: BoardRect::new(-140.0, -78.0, 560.0, 156.0),
        backdrops: Vec::new(),
        cards: vec![card("0", 0.0, 0.0), card("1", 280.0, 0.0)],
    };
    let board = PhysicsBoard::new();
    let mut text = BoardText::new();
    let titled = scene_desc
        .paint_titled(&board, 800, 400, page_fit(), &mut text)
        .unwrap();
    let glyphs: Vec<(f32, f32)> = titled
        .ops
        .iter()
        .filter_map(|op| match op {
            SceneOp::GlyphRun(run) => Some(run),
            _ => None,
        })
        .flat_map(|run| run.glyphs.iter().map(|glyph| (glyph.x, glyph.y)))
        .collect();
    for (id, _, rect) in scene_desc.card_rects(&board, 800, 400, page_fit()) {
        let xs: Vec<f32> = glyphs
            .iter()
            .filter(|(x, y)| {
                (rect.x..rect.x + rect.width).contains(x)
                    && (rect.y..rect.y + rect.height).contains(y)
            })
            .map(|(x, _)| *x)
            .collect();
        assert!(!xs.is_empty(), "card {id} has its title's glyphs");
        let first = xs.iter().copied().fold(f32::INFINITY, f32::min);
        let last = xs.iter().copied().fold(f32::NEG_INFINITY, f32::max);
        // The last origin stops one advance short of the text's end, so
        // the right gap reads up to one glyph wider than the left.
        let left_gap = first - rect.x;
        let right_gap = rect.x + rect.width - last;
        assert!(
            right_gap - left_gap >= 0.0 && right_gap - left_gap <= 14.0,
            "card {id}'s title is centred: left gap {left_gap}, right gap {right_gap} in {rect:?}",
        );
    }
}
