/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use super::*;

#[test]
fn the_view_fits_its_tile_from_the_full_centre_to_a_side_stack() {
    // Requirement 1: the graph takes what the tile leaves it.
    let host = Host::new(model(), 900, 600);
    let wide = MereView {
        model: &host.model,
        state: &host.view,
        leaf_key: 7,
        width: 900,
        height: 600,
    };
    assert_eq!(wide.graph_size(), (680, 560));
    let swatch = wide.swatch();
    assert_eq!((swatch.width, swatch.height), (680, 560));

    let narrow_host = Host::new(model(), 280, 600);
    let (dom, runner) = runner(narrow_host);
    let dom = dom.borrow();
    assert_eq!(attr(&dom, runner.root(), "data-width"), Some("narrow"));
    let narrow = MereView {
        model: &runner.state().model,
        state: &runner.state().view,
        leaf_key: 7,
        width: 280,
        height: 600,
    };
    // Two bar rows when narrow: (600 - 64) * 0.6.
    assert_eq!(narrow.graph_size(), (280, 321));
}

#[test]
fn an_empty_sessions_list_leaves_the_full_graph_area_available() {
    let mut empty = model();
    empty.sessions.clear();
    empty.can_mint = false;

    for (width, height, expected) in [(900, 600, (900, 560)), (280, 600, (280, 536))] {
        let host = Host::new(empty.clone(), width, height);
        let (dom, runner) = runner(host);
        let root = runner.root();
        let dom = dom.borrow();

        let view = MereView {
            model: &runner.state().model,
            state: &runner.state().view,
            leaf_key: 7,
            width,
            height,
        };
        assert_eq!(view.graph_size(), expected);
        let swatch = view.swatch();
        assert_eq!((swatch.width, swatch.height), expected);

        let sessions = find(&dom, root, "class", "mere-view-sessions").expect("sessions list");
        assert_eq!(attr(&dom, sessions, "style"), Some("display:none"));
        let main = find(&dom, root, "class", "mere-view-main").expect("graph area");
        let expected_style = format!("width:{}px;height:{}px", expected.0, expected.1);
        assert_eq!(attr(&dom, main, "style"), Some(expected_style.as_str()));
    }
}

#[test]
fn the_new_session_action_keeps_the_sessions_area_when_the_list_is_empty() {
    let mut empty = model();
    empty.sessions.clear();
    empty.can_mint = true;

    let host = Host::new(empty, 900, 600);
    let (dom, runner) = runner(host);
    let dom = dom.borrow();
    let root = runner.root();

    let sessions = find(&dom, root, "class", "mere-view-sessions").expect("sessions list");
    assert_eq!(attr(&dom, sessions, "style"), Some("width:220px"));
    assert!(find(&dom, sessions, "data-request", "mint").is_some());
    let view = MereView {
        model: &runner.state().model,
        state: &runner.state().view,
        leaf_key: 7,
        width: 900,
        height: 600,
    };
    assert_eq!(view.graph_size(), (680, 560));
}

#[test]
fn the_layout_tracks_session_area_changes() {
    let mut empty = model();
    empty.sessions.clear();
    empty.can_mint = false;
    let mut populated = empty.clone();
    populated.sessions.push(session(1, "Reading", false, false));

    let mut state = MereViewState::default();
    state.lay_out(&empty, 900, 600);
    let empty_position = state.position("a").expect("empty-layout node");

    state.lay_out(&populated, 900, 600);
    let populated_position = state.position("a").expect("populated-layout node");
    assert_ne!(empty_position, populated_position);

    state.lay_out(&empty, 900, 600);
    assert_eq!(state.position("a"), Some(empty_position));
}

#[test]
fn a_layout_switch_moves_nodes_and_keeps_every_key() {
    let graph = model().graph;
    let spectral = lay_out(&graph, "spectral.default", 680, 560);
    let grid = lay_out(&graph, "grid.default", 680, 560);
    let keys = |laid: &std::collections::HashMap<String, (f32, f32)>| {
        let mut keys: Vec<_> = laid.keys().cloned().collect();
        keys.sort();
        keys
    };
    assert_eq!(keys(&spectral), ["a", "b", "c", "d"]);
    assert_eq!(keys(&spectral), keys(&grid));
    for at in spectral.values().chain(grid.values()) {
        assert!(
            (0.0..=1.0).contains(&at.0) && (0.0..=1.0).contains(&at.1),
            "{at:?}"
        );
    }
    assert_ne!(spectral, grid, "the strategies place nodes differently");
    // An unknown strategy falls back to the default rather than stacking nodes.
    assert_eq!(lay_out(&graph, "no.such", 680, 560), spectral);
}

#[test]
fn nodes_a_strategy_puts_on_one_point_are_parted() {
    // Beta and Gamma relate to Alpha alike, so Spectral gives them one point.
    let graph = GraphModel {
        nodes: vec![
            node("a", "Alpha", NodeState::Available),
            node("b", "Beta", NodeState::Available),
            node("c", "Gamma", NodeState::Available),
        ],
        relations: vec![
            relation("ab", "a", "b", Provenance::Extracted),
            relation("ac", "a", "c", Provenance::Extracted),
        ],
    };
    let (width, height) = (680.0, 560.0);
    let laid = lay_out(&graph, "spectral.default", width as u32, height as u32);
    let pixels = |key: &str| {
        let (x, y) = laid[key];
        (x * width, y * height)
    };
    let (b, c) = (pixels("b"), pixels("c"));
    let apart = ((b.0 - c.0).powi(2) + (b.1 - c.1).powi(2)).sqrt();
    let separation = crate::layout::SEPARATION;
    assert!(
        apart >= separation - 0.1,
        "Beta and Gamma are {apart} px apart"
    );

    // A catalog like Knot's, where equal nodes also sit at the area's edge:
    // every pair stays apart, in the full centre and in a side tile.
    let keys = ["i", "g", "s", "p", "t", "m", "j", "r", "l", "d", "x", "k"];
    let catalog = GraphModel {
        nodes: keys
            .iter()
            .map(|key| node(key, key, NodeState::Available))
            .collect(),
        relations: [
            ("i", "g"),
            ("i", "s"),
            ("i", "p"),
            ("i", "j"),
            ("i", "k"),
            ("g", "s"),
            ("p", "x"),
            ("t", "m"),
            ("j", "t"),
            ("r", "k"),
            ("d", "x"),
            ("d", "r"),
        ]
        .iter()
        .map(|(from, to)| relation(&format!("{from}{to}"), from, to, Provenance::Extracted))
        .collect(),
    };
    for (w, h) in [(880_u32, 660_u32), (280, 381)] {
        let laid = lay_out(&catalog, "spectral.default", w, h);
        let at = |key: &str| (laid[key].0 * w as f32, laid[key].1 * h as f32);
        for (n, first) in keys.iter().enumerate() {
            for second in &keys[n + 1..] {
                let (a, b) = (at(first), at(second));
                let apart = ((a.0 - b.0).powi(2) + (a.1 - b.1).powi(2)).sqrt();
                assert!(
                    apart >= separation - 0.1,
                    "{first} and {second} are {apart} px apart at {w}x{h}"
                );
            }
        }
    }
    assert_eq!(
        lay_out(&graph, "spectral.default", 680, 560),
        laid,
        "the same every time"
    );
}

/// G2b's goldens for the mere view (dynamics grammar plan, G2b, step 1): each
/// graph-only layout on a fixed 40-node graph, hashed (FNV-1a over the keys
/// and the normalized positions' `f32` bits, in key order) before the
/// spectral coordinates move out of cartography into the channel registry.
#[test]
fn the_graph_only_layouts_match_their_goldens() {
    let nodes: Vec<NodeEntry> = (0..40)
        .map(|i| {
            node(
                &format!("n{i:02}"),
                &format!("Node {i}"),
                NodeState::Available,
            )
        })
        .collect();
    let mut relations = Vec::new();
    for i in 1..40usize {
        let parent = (i * 7 + 3) % i;
        relations.push(relation(
            &format!("t{i}"),
            &format!("n{i:02}"),
            &format!("n{parent:02}"),
            Provenance::Extracted,
        ));
    }
    for i in 0..12usize {
        let (a, b) = ((i * 11) % 40, (i * 17 + 5) % 40);
        if a != b {
            relations.push(relation(
                &format!("c{i}"),
                &format!("n{a:02}"),
                &format!("n{b:02}"),
                Provenance::Authored,
            ));
        }
    }
    let graph = GraphModel { nodes, relations };
    let hash = |laid: &std::collections::HashMap<String, (f32, f32)>| {
        let mut sorted: Vec<_> = laid.iter().collect();
        sorted.sort_by(|a, b| a.0.cmp(b.0));
        let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
        for (key, (x, y)) in sorted {
            for byte in key
                .bytes()
                .chain(x.to_bits().to_le_bytes())
                .chain(y.to_bits().to_le_bytes())
            {
                hash ^= u64::from(byte);
                hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
            }
        }
        hash
    };
    #[rustfmt::skip]
    const GOLDENS: &[(&str, u64)] = &[
        ("phyllotaxis.default", 0x3c67e1f148c0d711),
        ("grid.default", 0xfb33676b42ee1dae),
        ("spectral.default", 0x32662ebd01212de7),
        ("penrose.default", 0x5e0e955f65e4443a),
        ("lsystem.default", 0xbf7a15341f05086f),
    ];
    let mut taken = Vec::new();
    for id in cartography::adapters::GRAPH_ONLY_STRATEGIES {
        let laid = lay_out(&graph, id, 680, 560);
        assert_eq!(laid.len(), 40, "{id} places every node");
        println!("        (\"{id}\", 0x{:016x}),", hash(&laid));
        taken.push((*id, hash(&laid)));
    }
    assert_eq!(
        hash(&lay_out(&graph, "spectral.default", 680, 560)),
        taken[2].1,
        "the same layout twice"
    );
    assert!(
        !GOLDENS.is_empty(),
        "no goldens recorded yet: paste the lines above"
    );
    assert_eq!(taken, GOLDENS, "a graph-only layout moved from its golden");
}

#[test]
fn the_shared_panel_hides_only_its_duplicate_graph_and_keeps_requests_out() {
    let mut model = model();
    model.hide_graph = true;
    model.archive = Some(ArchiveModel {
        entries: vec![
            ArchiveEntry {
                key: "one".into(),
                name: "Morning archive".into(),
                detail: "Read-only archive".into(),
                can_open: true,
                session: None,
            },
            ArchiveEntry {
                key: "two".into(),
                name: "Evening archive".into(),
                detail: "Read-only archive".into(),
                can_open: true,
                session: None,
            },
        ],
        save_sessions: vec![session_id(1)],
        can_compose: true,
    });
    let before = model.clone();
    let (dom, mut runner) = runner(Host::new(model, 280, 700));
    let root = runner.root();
    assert_eq!(attr(&dom.borrow(), root, "data-graph"), Some("hidden"));
    assert!(
        find(&dom.borrow(), root, "data-key", "a").is_none(),
        "the host's main canvas owns node targets"
    );
    let save = find(
        &dom.borrow(),
        root,
        "aria-label",
        "Save Morning reading to archive",
    )
    .unwrap();
    runner.dispatch_click(save, PointerClick::at((1., 1.)));
    let open = find(
        &dom.borrow(),
        root,
        "aria-label",
        "Open Morning archive as session",
    )
    .unwrap();
    runner.dispatch_click(open, PointerClick::at((1., 1.)));
    let compose = find(&dom.borrow(), root, "data-request", "compose-archives").unwrap();
    runner.dispatch_click(compose, PointerClick::at((1., 1.)));
    assert_eq!(
        runner.state().requests.len(),
        2,
        "fewer than two sources cannot compose"
    );
    for name in ["Morning archive", "Evening archive"] {
        let selection = find(
            &dom.borrow(),
            root,
            "aria-label",
            &format!("Select {name} for composition"),
        )
        .unwrap();
        runner.dispatch_click(selection, PointerClick::at((1., 1.)));
    }
    let compose = find(&dom.borrow(), root, "data-request", "compose-archives").unwrap();
    runner.dispatch_click(compose, PointerClick::at((1., 1.)));
    assert_eq!(
        runner.state().requests,
        [
            MereViewRequest::Archive(ArchiveRequest::Save(session_id(1))),
            MereViewRequest::Archive(ArchiveRequest::Open("one".into())),
            MereViewRequest::Archive(ArchiveRequest::Compose(vec!["one".into(), "two".into()]))
        ]
    );
    assert_eq!(
        runner.state().model,
        before,
        "controls are requests, never a store"
    );
    runner.update(|host| {
        host.model.archive.as_mut().unwrap().entries.remove(0);
        host.view.lay_out(&host.model, 280, 700);
    });
    assert_eq!(
        runner.state().view.selected_archives,
        ["two"],
        "refresh removes vanished stable identifiers"
    );
}

#[test]
fn the_overmap_puts_sessions_in_containers_above_the_graph_with_their_archives_on_them() {
    let model = overmap_model();
    let before = model.clone();
    let beside = {
        let mut beside = model.clone();
        beside.placement = SessionPlacement::Beside;
        MereView {
            model: &beside,
            state: &MereViewState::default(),
            leaf_key: 7,
            width: 900,
            height: 600,
        }
        .graph_size()
    };
    let (dom, mut runner) = runner(Host::new(model, 900, 600));
    let root = runner.root();
    {
        let dom = dom.borrow();
        assert_eq!(attr(&dom, root, "data-sessions"), Some("overmap"));
        let view = MereView {
            model: &runner.state().model,
            state: &runner.state().view,
            leaf_key: 7,
            width: 900,
            height: 600,
        };
        let band = view.overmap_height();
        assert!(band > 0, "the overmap takes a band");
        assert_eq!(
            view.graph_size(),
            (900, 600 - 40 - band),
            "the graph keeps the full width, below the band"
        );
        assert!(view.graph_size().0 > beside.0, "no column beside the graph");

        // One container per session, then the shelf, then minting.
        let mut containers = Vec::new();
        find_all(&dom, root, "data-container", &mut containers);
        let kinds: Vec<_> = containers
            .iter()
            .map(|node| attr(&dom, *node, "data-container").unwrap())
            .collect();
        assert_eq!(kinds, ["session", "session", "session", "shelf", "mint"]);
        let attached = find(&dom, root, "aria-label", "Session Morning reading").unwrap();
        assert_eq!(attr(&dom, attached, "data-attached"), Some("true"));
        assert!(text(&dom, attached).contains("Shown here"));

        // An archive sits on the session it was saved from; one no live
        // session holds sits on the shelf.
        assert_eq!(
            container_of(&dom, root, "saved").as_deref(),
            Some("Session Morning reading")
        );
        assert_eq!(
            container_of(&dom, root, "composed").as_deref(),
            Some("Archive")
        );
        assert_eq!(
            container_of(&dom, root, "forgotten").as_deref(),
            Some("Archive")
        );

        // The side column is gone; the notice sits below the band.
        let mut lists = Vec::new();
        find_all(&dom, root, "aria-label", &mut lists);
        let sessions = lists
            .into_iter()
            .find(|node| attr(&dom, *node, "aria-label") == Some("Sessions"))
            .unwrap();
        assert!(
            attr(&dom, sessions, "style")
                .unwrap()
                .contains("display:none"),
            "no column of sessions beside the graph"
        );
    }

    // The containers' controls are the panel's requests.
    for label in [
        "Save Morning reading to archive",
        "Switch to Evening reading",
        "Open Codicil blake3:f612a as session",
    ] {
        let control = find(&dom.borrow(), root, "aria-label", label).unwrap();
        runner.dispatch_click(control, PointerClick::at((1., 1.)));
    }
    for name in ["Codicil blake3:f612a", "Codicil blake3:0c0de"] {
        let select = find(
            &dom.borrow(),
            root,
            "aria-label",
            &format!("Select {name} for composition"),
        )
        .unwrap();
        runner.dispatch_click(select, PointerClick::at((1., 1.)));
    }
    let compose = find(&dom.borrow(), root, "data-request", "compose-archives").unwrap();
    runner.dispatch_click(compose, PointerClick::at((1., 1.)));
    let mint = find(&dom.borrow(), root, "data-request", "mint").unwrap();
    runner.dispatch_click(mint, PointerClick::at((1., 1.)));
    assert_eq!(
        runner.state().requests,
        [
            MereViewRequest::Archive(ArchiveRequest::Save(session_id(1))),
            MereViewRequest::Session(session_id(2), SessionStep::Switch),
            MereViewRequest::Archive(ArchiveRequest::Open("saved".into())),
            MereViewRequest::Archive(ArchiveRequest::Compose(vec![
                "saved".into(),
                "composed".into()
            ])),
            MereViewRequest::Mint,
        ]
    );
    assert_eq!(
        runner.state().model,
        before,
        "controls are requests, never a store"
    );
}

#[test]
fn the_overmap_band_wraps_and_stops_at_its_share() {
    let model = overmap_model();
    let wide = MereView {
        model: &model,
        state: &MereViewState::default(),
        leaf_key: 7,
        width: 1400,
        height: 900,
    };
    let narrow = MereView { width: 600, ..wide };
    assert!(
        narrow.overmap_height() > wide.overmap_height(),
        "fewer containers to a row take more rows"
    );
    let short = MereView {
        height: 200,
        ..wide
    };
    assert!(
        short.overmap_height() <= ((200 - 40) as f32 * 0.6) as u32,
        "the band stops at its share of the body"
    );
    let mut without = model.clone();
    without.sessions.clear();
    without.archive = None;
    without.can_mint = false;
    assert_eq!(
        MereView {
            model: &without,
            ..wide
        }
        .overmap_height(),
        0,
        "no sessions, no band"
    );
}
