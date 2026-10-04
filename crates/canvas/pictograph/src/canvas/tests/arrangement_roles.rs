// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Arrangement roles through the canvas (dynamics grammar plan, G7): the
//! arrangement brief's Spiral probe rerun on the canvas, the three scopes,
//! drags, the pick and stop handoffs, Settled, and the board's roles and
//! encoded axes.

use super::*;
use seiche::{Axes, Role, RoleTable};

const N: usize = 60;
/// The brief's spacing: the canvas's 36-pixel face times 1.6.
const SPACING: f32 = 57.6;
const GOLDEN: f32 = 2.399_963_3;
const FACE: f32 = 36.0;

/// The brief's generator (`Code/testing/mere/arr-dyn/arr_dyn_probe.rs`).
struct Lcg(u64);
impl Lcg {
    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        self.0 >> 33
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
}

fn slot(ordinal: usize) -> PortablePoint {
    let n = ordinal as f32;
    let (r, a) = (SPACING * n.sqrt(), n * GOLDEN);
    PortablePoint::new(r * a.cos(), r * a.sin())
}

/// The brief's independent graph: a random recursive tree plus fifteen cross
/// edges, with shuffled recency ordinals, on a canvas that has picked a Spiral
/// of those ordinals.
fn spiral_canvas() -> (Canvas, Vec<NodeKey>, Vec<usize>) {
    let mut rng = Lcg(0x5eed_0001);
    let mut edges = Vec::new();
    for k in 1..N {
        edges.push((rng.below(k), k));
    }
    while edges.len() < N - 1 + 15 {
        let (a, b) = (rng.below(N), rng.below(N));
        if a != b && !edges.contains(&(a, b)) && !edges.contains(&(b, a)) {
            edges.push((a, b));
        }
    }
    let mut ordinal: Vec<usize> = (0..N).collect();
    for i in (1..N).rev() {
        let j = rng.below(i + 1);
        ordinal.swap(i, j);
    }
    let mut graph = Graph::new();
    let keys: Vec<_> = (0..N)
        .map(|i| graph.add_node(format!("https://probe-{i}.example"), slot(ordinal[i])))
        .collect();
    for (a, b) in edges {
        graph.assert_relation(keys[a], keys[b], crate::canvas::build::hyperlink());
    }
    let mut canvas = Canvas::with_graph(graph);
    canvas.resize(1400, 900);
    canvas.set_physics_paused(true);
    canvas.set_layout_strategy(Some("test.spiral".to_string()));
    let slots: Vec<_> = (0..N).map(|i| (keys[i], slot(ordinal[i]))).collect();
    canvas.apply_strategy_positions(&slots);
    canvas.fit_to_content();
    (canvas, keys, ordinal)
}

fn ranks(values: &[f32]) -> Vec<f32> {
    let mut idx: Vec<usize> = (0..values.len()).collect();
    idx.sort_by(|&a, &b| values[a].total_cmp(&values[b]));
    let mut r = vec![0.0; values.len()];
    for (rank, &i) in idx.iter().enumerate() {
        r[i] = rank as f32;
    }
    r
}

/// The Spiral's reading: Spearman's ρ between recency ordinal and distance
/// from the centroid.
fn rho(canvas: &Canvas, keys: &[NodeKey], ordinal: &[usize]) -> f32 {
    let at: Vec<PortablePoint> = keys
        .iter()
        .map(|k| canvas.view.position_of(*k).unwrap())
        .collect();
    let (cx, cy) = (
        at.iter().map(|p| p.x).sum::<f32>() / N as f32,
        at.iter().map(|p| p.y).sum::<f32>() / N as f32,
    );
    let radii: Vec<f32> = at.iter().map(|p| (p.x - cx).hypot(p.y - cy)).collect();
    let ords: Vec<f32> = ordinal.iter().map(|&o| o as f32).collect();
    let (ra, rb) = (ranks(&ords), ranks(&radii));
    let m = (N as f32 - 1.0) / 2.0;
    let (mut num, mut da, mut db) = (0.0, 0.0, 0.0);
    for i in 0..N {
        num += (ra[i] - m) * (rb[i] - m);
        da += (ra[i] - m).powi(2);
        db += (rb[i] - m).powi(2);
    }
    num / (da.sqrt() * db.sqrt())
}

fn overlaps(canvas: &Canvas, keys: &[NodeKey]) -> usize {
    let at: Vec<PortablePoint> = keys
        .iter()
        .map(|k| canvas.view.position_of(*k).unwrap())
        .collect();
    (0..N)
        .flat_map(|a| ((a + 1)..N).map(move |b| (a, b)))
        .filter(|&(a, b)| (at[a].x - at[b].x).hypot(at[a].y - at[b].y) < FACE)
        .count()
}

fn play(canvas: &mut Canvas, frames: usize) {
    canvas.set_physics_paused(false);
    for _ in 0..frames {
        canvas.frame(1400, 900);
    }
}

/// G7's first done-condition: the brief's probe through the canvas. The seed
/// reads ρ ≈ 1 (the positive control); seeded lets the order fall (the brief
/// measured 0.30 at 6 s in seiche alone); anchored holds it by its spring
/// while the graph moves (0.79 at 5 s) and, once it rests, its glide home
/// restores the arrangement exactly (F45); and every pinned item stays
/// exactly at its position while the seeded ones move.
#[test]
fn the_spiral_probe_through_the_canvas() {
    let (canvas, keys, ordinal) = spiral_canvas();
    let seed = rho(&canvas, &keys, &ordinal);
    assert!(
        seed > 0.99,
        "the instrument reads the seed as ordered: {seed}"
    );

    let run = |table: RoleTable, frames: usize| {
        let (mut canvas, keys, ordinal) = spiral_canvas();
        canvas.set_arrangement_roles(table);
        play(&mut canvas, frames);
        let r = rho(&canvas, &keys, &ordinal);
        (canvas, keys, r)
    };
    let (seeded, _, seeded_rho) = run(RoleTable::uniform(Role::Seeded), SETTLE_TICKS as usize);
    let (mut anchored, _, anchored_rho) = run(RoleTable::uniform(Role::Anchored), 300);
    let moving_overlaps = overlaps(&anchored, &keys);
    let mut frames = 300;
    while (anchored.settle_count() == 0 || anchored.anchored_home_count() < N) && frames < 3000 {
        anchored.frame(1400, 900);
        frames += 1;
    }
    let rest_rho = rho(&anchored, &keys, &ordinal);
    let mut mixed = RoleTable::uniform(Role::Seeded);
    let pinned: Vec<NodeKey> = keys.iter().copied().step_by(3).collect();
    for key in &pinned {
        mixed.items.insert(*key, Role::Pinned);
    }
    let (mixed_canvas, _, mixed_rho) = run(mixed, SETTLE_TICKS as usize);
    println!(
        "spiral probe through the canvas: seed rho {seed:.3}; seeded at 6 s rho {seeded_rho:.3} \
         ({} overlaps); anchored at 5 s, moving, rho {anchored_rho:.3} ({moving_overlaps} \
         overlaps, stiffness {}); anchored at rest, home after {frames} frames, rho {rest_rho:.3} \
         ({} overlaps, the arrangement's own); a third pinned at 6 s rho {mixed_rho:.3}",
        overlaps(&seeded, &keys),
        anchored.anchor_stiffness(),
        overlaps(&anchored, &keys),
    );
    assert!(seeded_rho < 0.5, "seeded, the order falls: {seeded_rho}");
    assert!(
        anchored_rho > 0.7,
        "anchored holds it while moving: {anchored_rho}"
    );
    assert_eq!(
        anchored.anchored_home_count(),
        N,
        "every anchored item came home"
    );
    for (i, key) in keys.iter().enumerate() {
        assert_eq!(
            anchored.view.position_of(*key),
            Some(slot(ordinal[i])),
            "anchored item {i} exactly home at rest"
        );
    }
    let mut moved = 0;
    for (i, key) in keys.iter().enumerate() {
        let at = mixed_canvas.view.position_of(*key).unwrap();
        let home = slot(ordinal[i]);
        let off = (at.x - home.x).hypot(at.y - home.y);
        if pinned.contains(key) {
            assert!(off < 1e-3, "pinned item {i} moved {off}");
        } else if off > 1.0 {
            moved += 1;
        }
    }
    println!(
        "a third pinned: {} pinned items exact, {moved} of {} seeded items moved over a unit",
        pinned.len(),
        N - pinned.len()
    );
    assert!(moved > N / 2, "the seeded two thirds moved: {moved}");
}

/// F22: an item's role overrides the recipe's, and the explicit pin is the
/// item-level pinned case. Realized: the seeded item gets no spring.
#[test]
fn an_item_role_overrides_the_recipe_role() {
    let (mut canvas, keys, _) = spiral_canvas();
    canvas.set_arrangement_role(Role::Anchored);
    let member = canvas.graph().get_node(keys[2]).unwrap().id;
    assert!(canvas.set_member_role(member, Some(Role::Seeded)));
    assert_eq!(
        canvas.arrangement_role_of(keys[0]),
        Role::Anchored,
        "recipe"
    );
    assert_eq!(canvas.arrangement_role_of(keys[2]), Role::Seeded, "item");
    canvas.select_only(keys[0]);
    assert!(canvas.pin_focused());
    assert_eq!(
        canvas.arrangement_role_of(keys[0]),
        Role::Pinned,
        "explicit pin"
    );
    assert!(canvas.release_focused());
    assert_eq!(canvas.arrangement_role_of(keys[0]), Role::Anchored);
    play(&mut canvas, 1);
    assert_eq!(
        canvas.physics.anchor_count(),
        N - 1,
        "all anchored but the seeded item"
    );
    assert!(canvas.set_member_role(member, None));
    assert_eq!(
        canvas.arrangement_role_of(keys[2]),
        Role::Anchored,
        "back to the recipe"
    );
}

/// F22: a group's role overrides the recipe's, and an item's overrides its
/// group's. Groups are sites: three pages of one site and one of another.
/// Realized: the group's pinned items hold exactly while the rest plays.
#[test]
fn an_item_role_overrides_its_group_role() {
    let mut graph = Graph::new();
    let urls = [
        "https://same.example/a",
        "https://same.example/b",
        "https://same.example/c",
        "https://other.example/d",
    ];
    let keys: Vec<_> = urls
        .iter()
        .map(|url| graph.add_node(url.to_string(), PortablePoint::new(0.0, 0.0)))
        .collect();
    for pair in keys.windows(2) {
        graph.assert_relation(pair[0], pair[1], crate::canvas::build::hyperlink());
    }
    let mut canvas = Canvas::with_graph(graph);
    canvas.resize(800, 600);
    canvas.set_physics_paused(true);
    canvas.set_layout_strategy(Some("test.groups".to_string()));
    let slots: Vec<_> = keys
        .iter()
        .enumerate()
        .map(|(i, k)| (*k, PortablePoint::new(i as f32 * 60.0, 0.0)))
        .collect();
    canvas.apply_strategy_positions(&slots);
    let site = canvas.role_group_of(keys[0]).unwrap();
    assert_eq!(
        canvas.role_group_of(keys[2]).as_deref(),
        Some(site.as_str())
    );
    assert_ne!(
        canvas.role_group_of(keys[3]).as_deref(),
        Some(site.as_str())
    );
    canvas.set_group_role(&site, Some(Role::Pinned));
    let member = canvas.graph().get_node(keys[2]).unwrap().id;
    assert!(canvas.set_member_role(member, Some(Role::Seeded)));
    assert_eq!(canvas.arrangement_role_of(keys[0]), Role::Pinned, "group");
    assert_eq!(canvas.arrangement_role_of(keys[1]), Role::Pinned, "group");
    assert_eq!(
        canvas.arrangement_role_of(keys[2]),
        Role::Seeded,
        "item over group"
    );
    assert_eq!(
        canvas.arrangement_role_of(keys[3]),
        Role::Seeded,
        "the recipe's"
    );
    play(&mut canvas, 120);
    for i in [0, 1] {
        assert_eq!(
            canvas.view.position_of(keys[i]),
            Some(slots[i].1),
            "pinned {i}"
        );
    }
    assert!(
        canvas.view.position_of(keys[2]) != Some(slots[2].1),
        "the overridden item moved"
    );
    assert!(canvas.set_member_role(member, None));
    assert_eq!(
        canvas.arrangement_role_of(keys[2]),
        Role::Pinned,
        "back to its group"
    );
}

/// Drag `key` by screen `(dx, dy)` in ten steps, a frame each, as a hand does.
fn drag_by(canvas: &mut Canvas, key: NodeKey, dx: f32, dy: f32) -> PortablePoint {
    let start = canvas.screen_position_of(key).expect("on screen");
    let end = (start.0 + dx, start.1 + dy);
    canvas.pointer_down(PointerButton::Left, start.0, start.1);
    for step in 1..=10 {
        let t = step as f32 / 10.0;
        assert!(canvas.cursor_moved(start.0 + dx * t, start.1 + dy * t));
        canvas.frame(1400, 900);
    }
    let dropped = canvas.view.position_of(key).unwrap();
    assert!(canvas.pointer_up(PointerButton::Left, end.0, end.1));
    dropped
}

/// F19 and F45: a dragged anchored item returns, with physics on by its
/// spring and then, once the graph rests, by a glide that ends exactly home;
/// with physics off it jumps back. A dragged seeded item stays where it was
/// dropped, its drop becoming its position, and nothing returns it (the
/// control in the same run). The outermost Spiral item is the anchored one:
/// its spring alone rests it about 220 units from home, its edges pulling.
#[test]
fn a_drag_follows_the_items_role() {
    let distance = |a: PortablePoint, b: PortablePoint| (a.x - b.x).hypot(a.y - b.y);
    let outward = |home: PortablePoint| {
        let out = 160.0 / home.x.hypot(home.y);
        (home.x * out, home.y * out)
    };
    for playing in [true, false] {
        let (mut canvas, keys, ordinal) = spiral_canvas();
        canvas.set_arrangement_role(Role::Anchored);
        let anchored = keys[(0..N).max_by_key(|&i| ordinal[i]).unwrap()];
        let seeded = keys[(0..N).find(|&i| ordinal[i] == N - 2).unwrap()];
        let member = canvas.graph().get_node(seeded).unwrap().id;
        canvas.set_member_role(member, Some(Role::Seeded));
        canvas.set_physics_paused(!playing);
        let homes = [anchored, seeded].map(|k| canvas.arrangement_slot(k).unwrap());
        let mut drops = homes;
        for (i, key) in [anchored, seeded].into_iter().enumerate() {
            let (dx, dy) = outward(homes[i]);
            drops[i] = drag_by(&mut canvas, key, dx, dy);
        }
        assert!(distance(drops[0], homes[0]) > 100.0, "the drag moved it");
        let slots = [anchored, seeded].map(|k| canvas.arrangement_slot(k).unwrap());
        assert_eq!(
            slots[0], homes[0],
            "an anchored drag leaves its position alone"
        );
        assert_eq!(slots[1], drops[1], "a seeded drop becomes its position");
        if playing {
            let mut rested_at = None;
            let mut frames = 0;
            while canvas.anchored_home_count() < N - 1 && frames < 6000 {
                canvas.step_layout();
                frames += 1;
                if rested_at.is_none() && canvas.settle_count() > 0 {
                    rested_at = canvas.view.position_of(anchored);
                }
            }
            let rested = rested_at.expect("the graph came to rest");
            let end = canvas.view.position_of(anchored).unwrap();
            let seeded_end = canvas.view.position_of(seeded).unwrap();
            println!(
                "drag, playing: anchored dropped {:.1} from home, rested {:.1} from it on its \
                 spring, home after {frames} frames ({:.3} off); seeded ends {:.1} from its old \
                 position",
                distance(drops[0], homes[0]),
                distance(rested, homes[0]),
                distance(end, homes[0]),
                distance(seeded_end, homes[1]),
            );
            assert!(
                distance(rested, homes[0]) > 100.0,
                "the spring alone rests it off home"
            );
            assert_eq!(end, homes[0], "the glide ends exactly home");
            assert!(
                distance(seeded_end, homes[1]) > 100.0,
                "nothing returns a seeded item"
            );
        } else {
            for _ in 0..SETTLE_TICKS {
                canvas.frame(1400, 900);
            }
            let ends = [anchored, seeded].map(|k| canvas.view.position_of(k).unwrap());
            assert_eq!(ends[0], homes[0], "jumped back with physics off");
            assert_eq!(ends[1], drops[1], "seeded stays exactly, paused");
        }
    }
}

/// F24: a pick while playing plays on from the landed positions, and a pick
/// while paused stays paused; a stop returns anchored items and leaves
/// seeded and pinned ones where they are.
#[test]
fn a_pick_keeps_play_and_a_stop_returns_by_role() {
    let (mut canvas, keys, ordinal) = spiral_canvas();
    play(&mut canvas, 60);
    let slots: Vec<_> = (0..N)
        .map(|i| (keys[i], slot(ordinal[N - 1 - i])))
        .collect();
    canvas.pick_layout_strategy(Some("test.other".to_string()));
    assert!(canvas.physics_paused(), "the pick pauses while it lands");
    canvas.apply_strategy_positions(&slots);
    assert!(!canvas.physics_paused(), "then plays on");
    assert_eq!(
        canvas.view.position_of(keys[0]),
        Some(slots[0].1),
        "from the landed positions"
    );
    canvas.set_physics_paused(true);
    canvas.pick_layout_strategy(Some("test.paused".to_string()));
    canvas.apply_strategy_positions(&slots);
    assert!(canvas.physics_paused(), "a paused pick stays paused");

    canvas.set_arrangement_role(Role::Anchored);
    let (seeded, pinned) = (keys[3], keys[4]);
    for (key, role) in [(seeded, Role::Seeded), (pinned, Role::Pinned)] {
        let member = canvas.graph().get_node(key).unwrap().id;
        canvas.set_member_role(member, Some(role));
    }
    play(&mut canvas, 90);
    let before: HashMap<_, _> = canvas.view.positions().collect();
    canvas.set_physics_paused(true);
    let stop = canvas.take_stop_return().expect("anchored items return");
    assert!(canvas.take_stop_return().is_none(), "taken once");
    let to: HashMap<_, _> = stop.to.iter().copied().collect();
    for (i, key) in keys.iter().enumerate() {
        let now = canvas.view.position_of(*key).unwrap();
        assert_eq!(to[key], now, "the canvas placed the return");
        if *key == seeded {
            assert_eq!(now, before[key], "seeded stays");
        } else if *key == pinned {
            assert_eq!(now, slots[i].1, "pinned stays at its position");
        } else {
            assert_eq!(now, slots[i].1, "anchored returned");
        }
    }
    canvas.frame(1400, 900);
    assert_eq!(canvas.view.position_of(seeded), Some(before[&seeded]));
}

/// F30: Settled holds the last settle's positions and replaces them on the
/// next; under a law that never rests it is left as it was. The resting run
/// before it is the positive control. On the sample graph, whose energy falls
/// below the floor within about 600 frames under Springs (the 60-node probe
/// graph does not in 7 200: `Code/testing/mere/grammar-g7/settle-energy-probe.log`).
#[test]
fn settled_holds_the_last_settle_and_a_living_law_leaves_it() {
    let mut canvas = Canvas::with_sample_graph();
    canvas.resize(1400, 900);
    canvas.fit_to_content();
    let keys: Vec<_> = canvas.graph().nodes().map(|(k, _)| k).collect();
    canvas.set_physics_paused(false);
    let rest = |canvas: &mut Canvas, count: u64| {
        let mut frames = 0;
        while canvas.settle_count() < count && frames < 9000 {
            canvas.frame(1400, 900);
            frames += 1;
        }
        frames
    };
    let frames = rest(&mut canvas, 1);
    assert_eq!(canvas.settle_count(), 1, "the playing graph came to rest");
    let first: HashMap<_, _> = canvas
        .settled_positions()
        .unwrap()
        .iter()
        .copied()
        .collect();
    assert_eq!(first[&keys[0]], canvas.view.position_of(keys[0]).unwrap());

    // Disturb it: a drag, then rest again, replaces Settled.
    drag_by(&mut canvas, keys[0], 80.0, 0.0);
    let more = rest(&mut canvas, 2);
    println!("first settle after {frames} frames, the second {more} after the drag");
    assert_eq!(canvas.settle_count(), 2, "a second settle");
    let second: HashMap<_, _> = canvas
        .settled_positions()
        .unwrap()
        .iter()
        .copied()
        .collect();
    assert_ne!(first[&keys[0]], second[&keys[0]], "replaced");

    // Picked like any other arrangement, it is where Restore returns.
    canvas.pick_layout_strategy(Some(SETTLED_ARRANGEMENT.to_string()));
    let settled = canvas.settled_positions().unwrap().to_vec();
    canvas.apply_strategy_positions(&settled);
    assert_eq!(canvas.arrangement_slot(keys[0]), Some(second[&keys[0]]));
    assert!(
        !canvas.physics_paused(),
        "picked while playing, it plays on"
    );

    // A law that never rests leaves it as it was.
    canvas.set_physics_law(PhysicsLaw::Orbit);
    for _ in 0..1200 {
        canvas.frame(1400, 900);
    }
    assert_eq!(canvas.settle_count(), 2, "Orbit never settles");
    let after: HashMap<_, _> = canvas
        .settled_positions()
        .unwrap()
        .iter()
        .copied()
        .collect();
    assert_eq!(after, second, "Settled unchanged under a living law");
    assert!(
        canvas.view.position_of(keys[0]) != Some(second[&keys[0]]),
        "while the graph kept moving"
    );
    assert!(canvas.restore_arrangement());
    assert_eq!(canvas.view.position_of(keys[0]), Some(second[&keys[0]]));
}

fn card(id: &str, x: f32, y: f32) -> BoardItem {
    BoardItem {
        id: id.to_string(),
        slot: (x, y),
        site: "fixture".to_string(),
    }
}

fn settle_board(board: &mut PhysicsBoard) {
    for _ in 0..SETTLE_TICKS {
        board.tick();
    }
}

/// A dragged seeded card stays where it was dropped; a pinned card
/// refuses the drag and holds its slot under the law; an item override
/// beats a site group, which beats the recipe default. (G7.)
#[test]
fn a_drag_follows_the_role_and_scopes_override_in_order() {
    let mut board = PhysicsBoard::new();
    let mut roles = RoleTable::uniform(Role::Seeded);
    roles.groups.insert("pinned-site".into(), Role::Pinned);
    board.set_roles(roles);
    let mut held = card("p", 120.0, 0.0);
    held.site = "pinned-site".into();
    let mut overridden = card("o", 0.0, 120.0);
    overridden.site = "pinned-site".into();
    board.sync(vec![card("s", 0.0, 0.0), held, overridden]);
    assert!(board.set_item_role("o", Some(Role::Seeded)));
    assert_eq!(board.role_of("s"), Some(Role::Seeded), "recipe default");
    assert_eq!(board.role_of("p"), Some(Role::Pinned), "site group");
    assert_eq!(board.role_of("o"), Some(Role::Seeded), "item beats group");

    assert!(!board.drag_start("p"), "a pinned card stays");
    assert!(board.drag_start("s"));
    assert!(board.drag_move(-200.0, 90.0));
    board.tick();
    assert!(board.drag_end());
    settle_board(&mut board);
    let s = board.position("s").unwrap();
    assert!(
        (s.0 + 200.0).hypot(s.1 - 90.0) < 60.0,
        "a seeded card stays near its drop: {s:?}"
    );
    assert_eq!(board.position("p"), Some((120.0, 0.0)), "pinned exactly");
    let o = board.position("o").unwrap();
    assert!(o != (0.0, 120.0), "the overridden card moved freely: {o:?}");
}

/// F28: with x encoded, overlapping cards keep x exactly and separate on y
/// under the law; with nothing encoded the same cards move on x too (the
/// control). With both encoded no law acts: a lone card stays exactly and
/// an overlapping pair only separates. A drag on an encoded axis returns
/// to the value.
#[test]
fn an_encoded_axis_stays_fixed_while_the_free_axis_separates() {
    let cards = || {
        vec![
            card("a", 100.0, 0.0),
            card("b", 100.0, 4.0),
            card("c", 140.0, -3.0),
        ]
    };
    let run = |axes: Axes| {
        let mut board = PhysicsBoard::new();
        board.set_encoded_axes(axes);
        board.sync(cards());
        board.set_choice(PhysicsChoice {
            law: PhysicsLaw::Charge,
            ..PhysicsChoice::default()
        });
        settle_board(&mut board);
        board
    };
    let x = run(Axes { x: true, y: false });
    for (id, slot) in [("a", 100.0), ("b", 100.0), ("c", 140.0)] {
        assert_eq!(x.position(id).unwrap().0, slot, "{id}'s x is held");
    }
    assert_eq!(x.encoded_drift(), 0.0, "the encoding holds");
    let (a, b) = (x.position("a").unwrap(), x.position("b").unwrap());
    assert!((a.1 - b.1).abs() > 36.0, "y separated: {a:?} {b:?}");
    let free = run(Axes::NONE);
    assert!(
        free.position("c").unwrap().0 != 140.0,
        "free, x moves: {:?}",
        free.position("c")
    );

    let mut both = run(Axes::BOTH);
    assert_eq!(both.position("c"), Some((140.0, -3.0)), "no law acts");
    assert!(
        both.encoded_drift() > 0.0,
        "the overlapping pair moved off its values"
    );
    let gap = both.gap("a", "b").unwrap();
    assert!(
        gap >= 2.0 * seiche::NODE_BODY_RADIUS - 0.5,
        "the overlapping pair separated: {gap}"
    );

    // The data moves: an encoded card follows its value, a free one would not.
    both.sync(vec![
        card("a", 100.0, 0.0),
        card("b", 100.0, 4.0),
        card("c", 260.0, 40.0),
    ]);
    settle_board(&mut both);
    assert_eq!(
        both.position("c"),
        Some((260.0, 40.0)),
        "c follows its new values"
    );
    assert!(both.drag_start("c"));
    assert!(both.drag_move(300.0, 300.0));
    both.tick();
    assert!(both.drag_end());
    settle_board(&mut both);
    assert_eq!(
        both.position("c"),
        Some((260.0, 40.0)),
        "back to its values"
    );
}
