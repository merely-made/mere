/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! The view tree against Knot's requirements (reservoir plan V2b), as far as a
//! tree can show them. Painting, tile sizing on screen and theming are the
//! headed harness's.

use std::{cell::RefCell, rc::Rc};

use cambium::{DomHandle, GenetAppRunner, PointerClick};
use genet_scripted_dom::{NodeId, ScriptedDom};
use incipit::SessionId;
use layout_dom_api::{LayoutDom, LocalName, Namespace, NodeKind};
use uuid::Uuid;

use super::*;
use crate::tier::{DEFAULT_MAX_ARCHIVE_NODES, ON_SHELF, SHELF_NODE_KEY, more_node_key};

struct Host {
    model: MereViewModel,
    view: MereViewState,
    width: u32,
    height: u32,
    requests: Vec<MereViewRequest>,
}

impl Host {
    fn new(model: MereViewModel, width: u32, height: u32) -> Self {
        let mut view = MereViewState::default();
        view.lay_out(&model, width, height);
        Self {
            model,
            view,
            width,
            height,
            requests: Vec::new(),
        }
    }
}

fn render(host: &Host) -> MereViewElement<Host, ()> {
    mere_view(
        MereView {
            model: &host.model,
            state: &host.view,
            leaf_key: 7,
            width: host.width,
            height: host.height,
        },
        |host: &mut Host, event| {
            host.view
                .apply_and_lay_out(event, &host.model, host.width, host.height)
        },
        |host: &mut Host, request| host.requests.push(request),
    )
}

type Runner =
    GenetAppRunner<Host, fn(&Host) -> MereViewElement<Host, ()>, MereViewElement<Host, ()>, ()>;

fn runner(host: Host) -> (DomHandle, Runner) {
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let runner = GenetAppRunner::new(
        dom.clone(),
        render as fn(&Host) -> MereViewElement<Host, ()>,
        host,
    );
    (dom, runner)
}

fn attr<'a>(dom: &'a ScriptedDom, node: NodeId, name: &str) -> Option<&'a str> {
    dom.attribute(node, &Namespace::from(""), &LocalName::from(name))
}

fn find(dom: &ScriptedDom, root: NodeId, name: &str, value: &str) -> Option<NodeId> {
    if attr(dom, root, name) == Some(value) {
        return Some(root);
    }
    dom.dom_children(root)
        .find_map(|child| find(dom, child, name, value))
}

fn find_all(dom: &ScriptedDom, root: NodeId, name: &str, found: &mut Vec<NodeId>) {
    if attr(dom, root, name).is_some() {
        found.push(root);
    }
    for child in dom.dom_children(root).collect::<Vec<_>>() {
        find_all(dom, child, name, found);
    }
}

fn text(dom: &ScriptedDom, node: NodeId) -> String {
    if dom.kind(node) == NodeKind::Text {
        return dom.text(node).unwrap_or_default().to_string();
    }
    dom.dom_children(node)
        .map(|child| text(dom, child))
        .collect()
}

fn session_id(n: u128) -> SessionId {
    SessionId::from_uuid(Uuid::from_u128(n))
}

fn session(n: u128, name: &str, attached: bool, trashed: bool) -> SessionEntry {
    let steps = if trashed {
        vec![SessionStep::Restore]
    } else if attached {
        vec![SessionStep::Fork, SessionStep::Trash]
    } else {
        vec![SessionStep::Switch, SessionStep::Fork, SessionStep::Trash]
    };
    SessionEntry {
        id: session_id(n),
        name: name.into(),
        detail: None,
        attached,
        trashed,
        steps,
    }
}

fn node(key: &str, label: &str, state: NodeState) -> NodeEntry {
    NodeEntry {
        key: key.into(),
        label: label.into(),
        state,
    }
}

fn relation(key: &str, from: &str, to: &str, provenance: Provenance) -> RelationEntry {
    RelationEntry {
        key: key.into(),
        from: from.into(),
        to: to.into(),
        provenance,
    }
}

fn model() -> MereViewModel {
    MereViewModel {
        title: "divination".into(),
        status: ViewStatus::Ready,
        sessions: vec![
            session(1, "Morning reading", true, false),
            session(2, "Evening reading", false, false),
            session(3, "Old draft", false, true),
        ],
        can_mint: true,
        graph: GraphModel {
            nodes: vec![
                node("a", "Alpha", NodeState::Available),
                node("b", "Beta", NodeState::Unavailable),
                node("c", "Gamma", NodeState::Dirty),
                node("d", "Delta", NodeState::Open),
            ],
            relations: vec![
                relation("ab-link", "a", "b", Provenance::Extracted),
                relation("ab-note", "a", "b", Provenance::Authored),
                relation("bc", "b", "c", Provenance::Suggested),
                relation("cd", "c", "d", Provenance::Other("containment".into())),
            ],
        },
        layout: "spectral.default".into(),
        layouts: vec![
            ("spectral.default".into(), "Spectral".into()),
            ("grid.default".into(), "Grid".into()),
        ],
        actions: vec![
            HostAction::new("new", "New"),
            HostAction::new("open", "Open"),
        ],
        notice: None,
        ..MereViewModel::default()
    }
}

#[test]
fn the_view_is_a_labelled_region_holding_the_hosts_actions_and_layouts() {
    let (dom, runner) = runner(Host::new(model(), 900, 600));
    let dom = dom.borrow();
    let root = runner.root();
    assert_eq!(attr(&dom, root, "role"), Some("region"));
    assert_eq!(attr(&dom, root, "aria-label"), Some("Mere: divination"));
    assert_eq!(attr(&dom, root, "data-status"), Some("ready"));
    assert_eq!(attr(&dom, root, "data-width"), Some("wide"));

    // Requirement 3: the host's action slot, and minting.
    let new = find(&dom, root, "data-action", "new").expect("New");
    assert_eq!(text(&dom, new), "New");
    assert!(find(&dom, root, "data-action", "open").is_some());
    assert!(find(&dom, root, "data-request", "mint").is_some());

    // Requirement 6: the layouts the host offers, the one in force pressed.
    let spectral = find(&dom, root, "data-layout", "spectral.default").expect("Spectral");
    let grid = find(&dom, root, "data-layout", "grid.default").expect("Grid");
    assert_eq!(attr(&dom, spectral, "aria-pressed"), Some("true"));
    assert_eq!(attr(&dom, grid, "aria-pressed"), Some("false"));
}

#[test]
fn requests_go_to_the_host_and_change_nothing_themselves() {
    let (dom, mut runner) = runner(Host::new(model(), 900, 600));
    let root = runner.root();
    let click = |runner: &mut Runner, name: &str, value: &str| {
        let target = find(&dom.borrow(), root, name, value).expect(value);
        runner.dispatch_click(target, PointerClick::at((1.0, 1.0)));
    };
    click(&mut runner, "data-request", "mint");
    click(&mut runner, "data-action", "open");
    click(&mut runner, "data-layout", "grid.default");
    click(&mut runner, "data-key", "a");
    assert_eq!(
        runner.state().requests,
        [
            MereViewRequest::Mint,
            MereViewRequest::Host("open".into()),
            MereViewRequest::Layout("grid.default".into()),
            MereViewRequest::Activate("a".into()),
        ]
    );
    // Requirement 2: nothing moved until the host acts.
    assert_eq!(runner.state().model, model());
}

#[test]
fn each_session_says_its_state_in_words_and_offers_the_hosts_steps() {
    let (dom, mut runner) = runner(Host::new(model(), 900, 600));
    let root = runner.root();
    {
        let dom = dom.borrow();
        let sessions = find(&dom, root, "aria-label", "Sessions").expect("sessions list");
        let mut items = Vec::new();
        find_all(&dom, sessions, "data-session", &mut items);
        assert_eq!(items.len(), 3);

        let shown = find(&dom, root, "data-session", &Uuid::from_u128(1).to_string()).expect("1");
        assert!(text(&dom, shown).contains("Shown here"));
        let trashed = find(&dom, root, "data-session", &Uuid::from_u128(3).to_string()).expect("3");
        assert!(text(&dom, trashed).contains("In the trash"));
        let mut steps = Vec::new();
        find_all(&dom, trashed, "data-step", &mut steps);
        let tokens: Vec<_> = steps.iter().map(|&s| attr(&dom, s, "data-step")).collect();
        assert_eq!(tokens, [Some("restore")]);
    }
    let restore = find(&dom.borrow(), root, "aria-label", "Restore Old draft").expect("step");
    runner.dispatch_click(restore, PointerClick::at((1.0, 1.0)));
    let switch = find(
        &dom.borrow(),
        root,
        "aria-label",
        "Switch to Evening reading",
    )
    .expect("step");
    runner.dispatch_click(switch, PointerClick::at((1.0, 1.0)));
    assert_eq!(
        runner.state().requests,
        [
            MereViewRequest::Session(session_id(3), SessionStep::Restore),
            MereViewRequest::Session(session_id(2), SessionStep::Switch),
        ]
    );
}

#[test]
fn every_node_is_a_labelled_target_with_a_stable_key_and_its_state_in_words() {
    let (dom, runner) = runner(Host::new(model(), 900, 600));
    let dom = dom.borrow();
    let root = runner.root();
    // Requirements 4 and 7.
    for (key, label) in [
        ("a", "Alpha"),
        ("b", "Beta, unavailable"),
        ("c", "Gamma, unsaved"),
        ("d", "Delta, open"),
    ] {
        let target = find(&dom, root, "data-key", key).expect(key);
        assert_eq!(attr(&dom, target, "aria-label"), Some(label));
    }
}

#[test]
fn relations_keep_their_provenance_and_parallel_ones_stay_apart() {
    let (dom, runner) = runner(Host::new(model(), 900, 600));
    let dom = dom.borrow();
    let root = runner.root();
    // Requirement 5.
    let link = find(&dom, root, "data-relation-id", "ab-link").expect("link");
    let note = find(&dom, root, "data-relation-id", "ab-note").expect("note");
    assert_ne!(link, note, "two relations on one pair stay two cells");
    assert_eq!(attr(&dom, link, "data-kind"), Some("extracted"));
    assert_eq!(attr(&dom, note, "data-kind"), Some("authored"));
    assert_eq!(
        attr(&dom, link, "aria-label"),
        Some("extracted: Alpha to Beta")
    );
    let suggestion = find(&dom, root, "data-relation-id", "bc").expect("suggestion");
    assert_eq!(attr(&dom, suggestion, "data-kind"), Some("suggested"));
    let other = find(&dom, root, "data-relation-id", "cd").expect("other");
    assert_eq!(attr(&dom, other, "data-kind"), Some("containment"));
}

#[test]
fn a_declined_request_is_shown_as_the_host_worded_it() {
    let mut declined = model();
    declined.notice = Some("The resident refused: the session is already in the trash.".into());
    let (dom, runner) = runner(Host::new(declined, 900, 600));
    let dom = dom.borrow();
    let root = runner.root();
    let notice = find(&dom, root, "class", "mere-view-notice").expect("notice");
    assert_eq!(attr(&dom, notice, "role"), Some("status"));
    assert_eq!(attr(&dom, notice, "aria-live"), Some("polite"));
    assert_eq!(
        text(&dom, notice),
        "The resident refused: the session is already in the trash."
    );
}

#[test]
fn empty_unavailable_and_building_say_so_plainly_with_the_hosts_action() {
    // Requirement 8.
    let states = [
        (
            ViewStatus::Empty(StatusNote {
                message: "No catalog is configured.".into(),
                action: Some(HostAction::new("configure", "Choose a catalog")),
            }),
            "empty",
            "configure",
        ),
        (
            ViewStatus::Unavailable(StatusNote {
                message: "The reservoir is locked by another resident.".into(),
                action: Some(HostAction::new("retry", "Try again")),
            }),
            "unavailable",
            "retry",
        ),
        (
            ViewStatus::Building(StatusNote {
                message: "Still reading links.".into(),
                action: Some(HostAction::new("cancel", "Stop reading")),
            }),
            "building",
            "cancel",
        ),
    ];
    for (status, token, key) in states {
        let message = status.note().expect("a note").message.clone();
        let mut blocked = model();
        blocked.status = status;
        let (dom, mut runner) = runner(Host::new(blocked, 280, 600));
        let root = runner.root();
        {
            let dom = dom.borrow();
            assert_eq!(attr(&dom, root, "data-status"), Some(token));
            let panel = find(&dom, root, "class", "mere-view-state").expect("state panel");
            assert_eq!(attr(&dom, panel, "role"), Some("status"));
            assert!(text(&dom, panel).contains(&message), "{token}");
            assert!(
                find(&dom, root, "data-key", "a").is_none(),
                "no graph while {token}"
            );
        }
        let action = find(&dom.borrow(), root, "data-action", key).expect(key);
        runner.dispatch_click(action, PointerClick::at((1.0, 1.0)));
        assert_eq!(runner.state().requests, [MereViewRequest::Host(key.into())]);
    }
}

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
fn hover_and_focus_are_the_views_own_state() {
    let mut state = MereViewState::default();
    state.apply(MereViewEvent::Hover(Some("a".into())));
    state.apply(MereViewEvent::Focus(Some("b".into())));
    assert_eq!(state.hovered.as_deref(), Some("a"));
    assert_eq!(state.focused.as_deref(), Some("b"));
    let host = Host::new(model(), 900, 600);
    let view = MereView {
        model: &host.model,
        state: &state,
        leaf_key: 7,
        width: 900,
        height: 600,
    };
    let swatch = view.swatch();
    assert_eq!(swatch.hovered.as_deref(), Some("a"));
    assert_eq!(swatch.focus.as_deref(), Some("b"));
}

#[test]
fn the_structural_sheet_names_no_colour() {
    // Requirement 9: colour is the host's, through its own sheet and tokens.
    // The only colour the sheet may name is the host's current one.
    let sheet = MERE_VIEW_CSS.to_ascii_lowercase();
    for colour in ["#", "rgb(", "rgba(", "hsl(", "hsla("] {
        assert!(!sheet.contains(colour), "the sheet names {colour:?}");
    }
    for (at, _) in sheet.match_indices("color:") {
        let value = sheet[at + "color:".len()..]
            .split([';', '}'])
            .next()
            .unwrap_or_default()
            .trim();
        assert_eq!(value, "currentcolor", "a colour the host did not give");
    }
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

#[test]
fn minting_heads_the_sessions_list() {
    let (dom, runner) = runner(Host::new(model(), 900, 600));
    let dom = dom.borrow();
    let root = runner.root();
    let sessions = find(&dom, root, "aria-label", "Sessions").expect("sessions list");
    assert!(find(&dom, sessions, "data-request", "mint").is_some());
    let bar = find(&dom, root, "class", "mere-view-bar").expect("bar");
    assert!(find(&dom, bar, "data-request", "mint").is_none());
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

fn overmap_model() -> MereViewModel {
    let mut model = model();
    model.placement = SessionPlacement::Overmap;
    model.archive = Some(ArchiveModel {
        entries: vec![
            ArchiveEntry {
                key: "saved".into(),
                name: "Codicil blake3:f612a".into(),
                detail: "Read-only archive".into(),
                can_open: true,
                session: Some(session_id(1)),
            },
            ArchiveEntry {
                key: "composed".into(),
                name: "Codicil blake3:0c0de".into(),
                detail: "Read-only archive".into(),
                can_open: true,
                session: None,
            },
            ArchiveEntry {
                key: "forgotten".into(),
                name: "Codicil blake3:90ne0".into(),
                detail: "Read-only archive".into(),
                can_open: false,
                session: Some(session_id(99)),
            },
        ],
        save_sessions: vec![session_id(1), session_id(2)],
        can_compose: true,
    });
    model
}

fn container_of(dom: &ScriptedDom, root: NodeId, archive: &str) -> Option<String> {
    let mut containers = Vec::new();
    find_all(dom, root, "data-container", &mut containers);
    containers.into_iter().find_map(|container| {
        find(dom, container, "data-archive", archive).map(|_| {
            attr(dom, container, "aria-label")
                .unwrap_or_default()
                .to_string()
        })
    })
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

fn tier_model() -> MereViewModel {
    let mut model = overmap_model();
    model.placement = SessionPlacement::Tier;
    model
}

fn tier_view(host: &Host) -> MereView<'_> {
    MereView {
        model: &host.model,
        state: &host.view,
        leaf_key: 7,
        width: host.width,
        height: host.height,
    }
}

fn hover(phase: cambium::HoverPhase) -> cambium::HoverEvent {
    cambium::HoverEvent::new(phase, (2.0, 2.0), (44.0, 44.0))
}

fn escape() -> cambium::KeyEvent {
    cambium::KeyEvent::new(cambium::Key::Named(cambium::NamedKey::Escape))
}

fn count(dom: &ScriptedDom, root: NodeId, name: &str, value: &str) -> usize {
    let mut found = Vec::new();
    find_all(dom, root, name, &mut found);
    found
        .into_iter()
        .filter(|node| attr(dom, *node, name) == Some(value))
        .count()
}

#[test]
fn the_tier_puts_sessions_and_archives_on_the_canvas_one_level_up() {
    let (dom, mut runner) = runner(Host::new(tier_model(), 900, 600));
    let root = runner.root();
    {
        let dom = dom.borrow();
        let host = runner.state();
        let view = tier_view(host);
        assert_eq!(attr(&dom, root, "data-sessions"), Some("tier"));
        assert_eq!(
            view.graph_size(),
            (900, 600 - 40),
            "the tier is on the graph's own canvas, which keeps the whole body"
        );
        assert_eq!(view.overmap_height(), 0, "no band above the canvas");
        let tier = view.tier_height() as f32;
        assert!(tier > 0.0);

        // Sessions and archives are canvas nodes in the tier; the graph's own
        // nodes are laid out below it.
        let swatch = view.swatch();
        for (node, (id, (_, y))) in swatch.graph.nodes.iter().zip(swatch.projected_positions()) {
            let in_tier = !matches!(node.kind, MereNodeKind::Graph(_));
            if in_tier {
                assert!(y < tier, "{id} sits in the tier");
            } else {
                assert!(y > tier, "{id} sits below the tier");
            }
        }
        assert_eq!(
            swatch
                .graph
                .nodes
                .iter()
                .filter(|node| node.id.starts_with(SESSION_NODE_PREFIX))
                .count(),
            3
        );

        // The shown session is marked, in words and as the current node.
        let shown = find(&dom, root, "data-key", &session_node_key(session_id(1)))
            .expect("a session is a canvas node");
        assert_eq!(
            attr(&dom, shown, "aria-label"),
            Some("Session Morning reading, shown here")
        );
        assert_eq!(attr(&dom, shown, "aria-current"), Some("true"));
        assert_eq!(attr(&dom, shown, "data-attached"), Some("true"));
        assert_eq!(attr(&dom, shown, "aria-expanded"), Some("false"));
        let trashed = find(&dom, root, "data-key", &session_node_key(session_id(3))).unwrap();
        assert_eq!(
            attr(&dom, trashed, "aria-label"),
            Some("Session Old draft, in the trash")
        );

        // An archive is a node joined to the session it was saved from; one
        // no live session holds says it is in the archive.
        let saved = find(&dom, root, "data-key", &archive_node_key("saved")).unwrap();
        assert_eq!(attr(&dom, saved, "aria-haspopup"), Some("dialog"));
        assert_eq!(attr(&dom, saved, "data-tier"), Some("archive"));
        let route = find(&dom, root, "data-relation-id", "saved-from:saved")
            .expect("the archive's route to its session");
        assert_eq!(attr(&dom, route, "data-kind"), Some(SAVED_FROM));
        let composed = find(&dom, root, "data-key", &archive_node_key("composed")).unwrap();
        assert_eq!(
            attr(&dom, composed, "aria-label"),
            Some("Codicil blake3:0c0de, in the archive")
        );
        assert!(find(&dom, root, "data-relation-id", "saved-from:composed").is_none());

        // No column, no band, no card until one is opened.
        assert_eq!(count(&dom, root, "data-container", "session"), 0);
        assert!(find(&dom, root, "class", "mere-view-overmap").is_none());
    }

    // New session is the tier's own control; a graph node is still the host's.
    let mint = find(&dom.borrow(), root, "data-request", "mint").unwrap();
    runner.dispatch_click(mint, PointerClick::at((1., 1.)));
    let alpha = find(&dom.borrow(), root, "data-key", "a").unwrap();
    runner.dispatch_click(alpha, PointerClick::at((1., 1.)));
    assert_eq!(
        runner.state().requests,
        [MereViewRequest::Mint, MereViewRequest::Activate("a".into())]
    );
}

#[test]
fn a_session_node_opens_in_place_into_its_card_one_at_a_time() {
    let model = tier_model();
    let before = model.clone();
    let (dom, mut runner) = runner(Host::new(model, 900, 600));
    let root = runner.root();
    let morning = session_node_key(session_id(1));
    let node = find(&dom.borrow(), root, "data-key", &morning).unwrap();
    runner.dispatch_click(node, PointerClick::at((1., 1.)));
    assert_eq!(
        runner.state().view.expanded,
        Some(TierCard::Session(session_id(1)))
    );
    assert!(
        runner.state().requests.is_empty(),
        "opening a card asks nothing of the host"
    );
    {
        let dom = dom.borrow();
        let host = runner.state();
        let swatch = tier_view(host).swatch();
        let card = find(&dom, root, "aria-label", "Session Morning reading").expect("the card");
        assert_eq!(attr(&dom, card, "data-container"), Some("session"));
        assert!(text(&dom, card).contains("Shown here"));
        let node = find(&dom, root, "data-key", &morning).unwrap();
        assert_eq!(attr(&dom, node, "aria-expanded"), Some("true"));

        // The card is the node's footprint: it sits where the canvas says,
        // and the archive's route ends at its border, not its centre.
        let region = swatch
            .projected_node_footprint(&morning)
            .expect("footprint");
        let style = attr(&dom, card, "style").unwrap();
        assert!(
            style.contains(&format!("left:{}px", region.left)),
            "{style}"
        );
        assert!(style.contains(&format!("top:{}px", region.top)), "{style}");
        let (_, route) = swatch
            .projected_relations()
            .into_iter()
            .find(|(relation, _)| relation.id == "saved-from:saved")
            .unwrap();
        let end = *route.last().unwrap();
        let on_edge = |value: f32, edge: f32| (value - edge).abs() < 0.5;
        assert!(
            on_edge(end.1, region.top)
                || on_edge(end.1, region.top + region.height)
                || on_edge(end.0, region.left)
                || on_edge(end.0, region.left + region.width),
            "the route ends at the card's border: {end:?} {region:?}"
        );
        let (_, saved) = swatch
            .projected_positions()
            .into_iter()
            .find(|(id, _)| **id == archive_node_key("saved"))
            .unwrap();
        assert!(
            saved.1 > region.top + region.height || saved.1 < region.top,
            "the archive node moves clear of the card"
        );
        // The card holds the session's archives as chips.
        assert!(find(&dom, card, "data-archive-chip", "saved").is_some());
    }

    // The card's controls are the session's requests.
    for label in ["Save Morning reading to archive", "Fork Morning reading"] {
        let control = find(&dom.borrow(), root, "aria-label", label).unwrap();
        runner.dispatch_click(control, PointerClick::at((1., 1.)));
    }
    assert_eq!(
        runner.state().requests,
        [
            MereViewRequest::Archive(ArchiveRequest::Save(session_id(1))),
            MereViewRequest::Session(session_id(1), SessionStep::Fork),
        ]
    );

    // Opening another session's node moves the one card there.
    let evening = find(
        &dom.borrow(),
        root,
        "data-key",
        &session_node_key(session_id(2)),
    )
    .unwrap();
    runner.dispatch_click(evening, PointerClick::at((1., 1.)));
    assert_eq!(
        runner.state().view.expanded,
        Some(TierCard::Session(session_id(2)))
    );
    assert_eq!(count(&dom.borrow(), root, "data-container", "session"), 1);
    assert!(find(&dom.borrow(), root, "aria-label", "Session Evening reading").is_some());

    // Close returns focus to the session's node; so does Escape.
    let close = find(
        &dom.borrow(),
        root,
        "aria-label",
        "Close session Evening reading",
    )
    .unwrap();
    assert_eq!(
        runner.focus(),
        Some(close),
        "the card takes focus as it opens"
    );
    runner.dispatch_click(close, PointerClick::at((1., 1.)));
    assert_eq!(runner.state().view.expanded, None);
    let evening = find(
        &dom.borrow(),
        root,
        "data-key",
        &session_node_key(session_id(2)),
    )
    .unwrap();
    assert_eq!(runner.focus(), Some(evening));
    runner.dispatch_click(evening, PointerClick::at((1., 1.)));
    let close = find(
        &dom.borrow(),
        root,
        "aria-label",
        "Close session Evening reading",
    )
    .unwrap();
    runner.set_focus(Some(close));
    runner.dispatch_key(escape());
    assert_eq!(runner.state().view.expanded, None);
    assert_eq!(runner.state().model, before, "a card is presentation only");
}

#[test]
fn an_archive_node_peeks_on_hover_and_pins_a_popover_on_click() {
    use cambium::HoverPhase;
    let (dom, mut runner) = runner(Host::new(tier_model(), 900, 600));
    let root = runner.root();
    let saved_key = archive_node_key("saved");
    let saved = find(&dom.borrow(), root, "data-key", &saved_key).unwrap();

    runner.dispatch_hover(saved, hover(HoverPhase::Enter));
    {
        let dom = dom.borrow();
        let peek = find(&dom, root, "role", "tooltip").expect("hover peeks");
        assert!(text(&dom, peek).contains("Saved from Morning reading"));
        assert!(
            find(&dom, peek, "data-step", "open-archive").is_none(),
            "a peek is words, not controls"
        );
    }
    runner.dispatch_hover(saved, hover(HoverPhase::Leave));
    assert!(find(&dom.borrow(), root, "role", "tooltip").is_none());

    // A click pins the controls, and the first takes focus.
    runner.dispatch_click(saved, PointerClick::at((1., 1.)));
    let open = {
        let dom = dom.borrow();
        let dialog = find(&dom, root, "role", "dialog").expect("a click pins");
        assert_eq!(
            attr(&dom, dialog, "aria-label"),
            Some("Codicil blake3:f612a")
        );
        assert!(find(&dom, dialog, "data-request", "compose-archives").is_some());
        let saved = find(&dom, root, "data-key", &saved_key).unwrap();
        assert_eq!(attr(&dom, saved, "aria-expanded"), Some("true"));
        find(
            &dom,
            dialog,
            "aria-label",
            "Open Codicil blake3:f612a as session",
        )
        .unwrap()
    };
    assert_eq!(runner.focus(), Some(open));
    // Hovering another node leaves a pinned popover where it is.
    let composed = find(
        &dom.borrow(),
        root,
        "data-key",
        &archive_node_key("composed"),
    )
    .unwrap();
    runner.dispatch_hover(composed, hover(HoverPhase::Enter));
    assert!(find(&dom.borrow(), root, "role", "dialog").is_some());
    assert!(find(&dom.borrow(), root, "role", "tooltip").is_none());
    runner.dispatch_hover(composed, hover(HoverPhase::Leave));

    let select = find(
        &dom.borrow(),
        root,
        "aria-label",
        "Select Codicil blake3:f612a for composition",
    )
    .unwrap();
    runner.dispatch_click(select, PointerClick::at((1., 1.)));
    // Escape dismisses and hands focus back to the archive's node.
    runner.set_focus(Some(open));
    runner.dispatch_key(escape());
    assert!(find(&dom.borrow(), root, "role", "dialog").is_none());
    let saved = find(&dom.borrow(), root, "data-key", &saved_key).unwrap();
    assert_eq!(runner.focus(), Some(saved));

    // Open as session asks for the session and closes the popover, so the
    // next click reaches the canvas rather than the dismiss layer.
    runner.dispatch_click(saved, PointerClick::at((1., 1.)));
    let open = find(
        &dom.borrow(),
        root,
        "aria-label",
        "Open Codicil blake3:f612a as session",
    )
    .unwrap();
    runner.dispatch_click(open, PointerClick::at((1., 1.)));
    assert!(find(&dom.borrow(), root, "role", "dialog").is_none());
    assert_eq!(runner.state().view.archive_popover, None);

    // A second archive's popover composes the two; an outside click closes it.
    let composed = find(
        &dom.borrow(),
        root,
        "data-key",
        &archive_node_key("composed"),
    )
    .unwrap();
    runner.dispatch_click(composed, PointerClick::at((1., 1.)));
    let select = find(
        &dom.borrow(),
        root,
        "aria-label",
        "Select Codicil blake3:0c0de for composition",
    )
    .unwrap();
    runner.dispatch_click(select, PointerClick::at((1., 1.)));
    let compose = find(&dom.borrow(), root, "data-request", "compose-archives").unwrap();
    runner.dispatch_click(compose, PointerClick::at((1., 1.)));
    let outside = find(
        &dom.borrow(),
        root,
        "class",
        "overlay-surface-dismiss-layer",
    )
    .unwrap();
    runner.dispatch_click(outside, PointerClick::at((1., 1.)));
    assert!(find(&dom.borrow(), root, "role", "dialog").is_none());
    assert_eq!(
        runner.state().requests,
        [
            MereViewRequest::Archive(ArchiveRequest::Open("saved".into())),
            MereViewRequest::Archive(ArchiveRequest::Compose(vec![
                "saved".into(),
                "composed".into()
            ])),
        ]
    );

    // A card's archive chip pins the same popover.
    let morning = find(
        &dom.borrow(),
        root,
        "data-key",
        &session_node_key(session_id(1)),
    )
    .unwrap();
    runner.dispatch_click(morning, PointerClick::at((1., 1.)));
    let chip = find(&dom.borrow(), root, "data-archive-chip", "saved").unwrap();
    runner.dispatch_click(chip, PointerClick::at((1., 1.)));
    assert!(find(&dom.borrow(), root, "role", "dialog").is_some());
}

#[test]
fn a_card_or_popover_closes_when_its_session_or_archive_is_gone() {
    let mut host = Host::new(tier_model(), 900, 600);
    host.view
        .apply(MereViewEvent::Expand(Some(TierCard::Session(session_id(
            2,
        )))));
    host.view.apply(MereViewEvent::ArchivePopover(
        "composed".into(),
        cambium::DetailPopoverEvent::TogglePinned,
    ));
    assert!(host.view.archive_popover.is_some());
    host.model
        .sessions
        .retain(|session| session.id != session_id(2));
    if let Some(archive) = &mut host.model.archive {
        archive.entries.retain(|entry| entry.key != "composed");
    }
    host.view.lay_out(&host.model, 900, 600);
    assert_eq!(host.view.expanded, None);
    assert_eq!(host.view.archive_popover, None);

    // Without sessions, archives or minting there is no tier at all.
    let mut bare = tier_model();
    bare.sessions.clear();
    bare.archive = None;
    bare.can_mint = false;
    let host = Host::new(bare, 900, 600);
    assert_eq!(tier_view(&host).tier_height(), 0);
    assert_eq!(tier_view(&host).graph_size(), (900, 560));
}

#[test]
fn every_node_has_a_kind_the_host_colours_by() {
    let host = Host::new(tier_model(), 900, 600);
    let swatch = tier_view(&host).swatch();
    let kind = |key: &str| {
        swatch
            .graph
            .nodes
            .iter()
            .find(|node| node.id == key)
            .map(|node| node.kind)
    };
    assert_eq!(kind("b"), Some(MereNodeKind::Graph(NodeState::Unavailable)));
    assert_eq!(
        kind(&session_node_key(session_id(1))),
        Some(MereNodeKind::Session {
            attached: true,
            trashed: false
        })
    );
    assert_eq!(
        kind(&archive_node_key("saved")),
        Some(MereNodeKind::Archive { held: true })
    );
    assert_eq!(
        kind(&archive_node_key("composed")),
        Some(MereNodeKind::Archive { held: false })
    );
    assert_eq!(kind(SHELF_NODE_KEY), Some(MereNodeKind::Shelf));
    // A palette that knows only graph states still reads a tier node.
    assert_eq!(
        MereNodeKind::Session {
            attached: true,
            trashed: false
        }
        .state(),
        NodeState::Open
    );
    // The palette sees each kind, and every target says its kind.
    let seen = RefCell::new(Vec::new());
    tier_view(&host).paint_leaf(|kind| {
        seen.borrow_mut().push(kind.token());
        sprigging::ColorF {
            r: 0.,
            g: 0.,
            b: 0.,
            a: 1.,
        }
    });
    for token in ["graph", "session", "archive", "shelf"] {
        assert!(seen.borrow().contains(&token), "the palette sees {token}");
    }
    let (dom, runner) = runner(host);
    let dom = dom.borrow();
    let shelf = find(&dom, runner.root(), "data-key", SHELF_NODE_KEY).unwrap();
    assert_eq!(attr(&dom, shelf, "data-node-kind"), Some("shelf"));
    let alpha = find(&dom, runner.root(), "data-key", "a").unwrap();
    assert_eq!(attr(&dom, alpha, "data-node-kind"), Some("graph"));
}

#[test]
fn loose_archives_hang_off_the_shelf_which_opens_into_its_card() {
    let (dom, mut runner) = runner(Host::new(tier_model(), 900, 600));
    let root = runner.root();
    {
        let dom = dom.borrow();
        let shelf = find(&dom, root, "data-key", SHELF_NODE_KEY).expect("a shelf node");
        assert_eq!(attr(&dom, shelf, "aria-label"), Some("Archive"));
        for loose in ["composed", "forgotten"] {
            let route = find(
                &dom,
                root,
                "data-relation-id",
                &format!("{ON_SHELF}:{loose}"),
            )
            .expect("a loose archive joins the shelf");
            assert_eq!(attr(&dom, route, "data-kind"), Some(ON_SHELF));
        }
        assert!(find(&dom, root, "data-relation-id", "on-shelf:saved").is_none());
        // New session and Compose are the tier's own controls.
        let mut controls = Vec::new();
        find_all(&dom, root, "class", &mut controls);
        let tier_controls = controls
            .into_iter()
            .find(|node| attr(&dom, *node, "class") == Some("mere-view-tier-controls"))
            .expect("the tier's controls");
        assert!(find(&dom, tier_controls, "data-request", "mint").is_some());
        assert!(find(&dom, tier_controls, "data-request", "compose-archives").is_some());
    }
    let shelf = find(&dom.borrow(), root, "data-key", SHELF_NODE_KEY).unwrap();
    runner.dispatch_click(shelf, PointerClick::at((1., 1.)));
    assert_eq!(runner.state().view.expanded, Some(TierCard::Shelf));
    let dom = dom.borrow();
    let card = find(&dom, root, "data-container", "shelf").expect("the shelf's card");
    assert_eq!(attr(&dom, card, "aria-label"), Some("Archive"));
    assert!(find(&dom, card, "data-request", "compose-archives").is_some());
    assert!(find(&dom, card, "data-archive-chip", "composed").is_some());
    assert!(find(&dom, card, "data-archive-chip", "forgotten").is_some());
    assert!(find(&dom, card, "data-archive-chip", "saved").is_none());
}

#[test]
fn an_open_card_grows_the_tier_and_the_graph_is_laid_out_again_below_it() {
    let (dom, mut runner) = runner(Host::new(tier_model(), 900, 600));
    let root = runner.root();
    let (rest, laid_before) = {
        let host = runner.state();
        (tier_view(host).tier_height(), host.view.position("a"))
    };
    let morning = find(
        &dom.borrow(),
        root,
        "data-key",
        &session_node_key(session_id(1)),
    )
    .unwrap();
    runner.dispatch_click(morning, PointerClick::at((1., 1.)));
    let host = runner.state();
    let view = tier_view(host);
    let open = view.tier_height();
    assert!(
        open > rest,
        "the tier grows to hold the card: {rest} to {open}"
    );
    assert_ne!(
        host.view.position("a"),
        laid_before,
        "the graph is laid out again in the smaller area"
    );
    let swatch = view.swatch();
    let card = swatch
        .projected_node_footprint(&session_node_key(session_id(1)))
        .unwrap();
    assert!(
        card.top + card.height <= open as f32,
        "the card sits in the tier"
    );
    for (node, (id, (_, y))) in swatch.graph.nodes.iter().zip(swatch.projected_positions()) {
        if matches!(node.kind, MereNodeKind::Graph(_)) {
            assert!(y > open as f32, "{id} sits below the grown tier");
        } else {
            assert!(y < open as f32, "{id} sits in the grown tier");
        }
    }
    // Closing it lets the tier rest again.
    runner.update(|host| {
        host.view.apply(MereViewEvent::Expand(None));
        host.view.lay_out(&host.model, host.width, host.height);
    });
    assert_eq!(tier_view(runner.state()).tier_height(), rest);
    assert_eq!(runner.state().view.position("a"), laid_before);
}

#[test]
fn past_a_few_archives_a_session_shows_one_more_node_that_opens_its_card() {
    let mut model = tier_model();
    let archive = model.archive.as_mut().unwrap();
    for n in 0..4 {
        archive.entries.push(ArchiveEntry {
            key: format!("evening-{n}"),
            name: format!("Codicil blake3:e{n}"),
            detail: "Read-only archive".into(),
            can_open: true,
            session: Some(session_id(2)),
        });
    }
    let (dom, mut runner) = runner(Host::new(model, 900, 600));
    let root = runner.root();
    let more_key = more_node_key(TierCard::Session(session_id(2)));
    {
        let dom = dom.borrow();
        let more = find(&dom, root, "data-key", &more_key).expect("one +N node");
        assert_eq!(
            attr(&dom, more, "aria-label"),
            Some("+4 archives of Session Evening reading")
        );
        assert_eq!(attr(&dom, more, "data-count"), Some("4"));
        assert!(
            find(&dom, root, "data-key", &archive_node_key("evening-0")).is_none(),
            "the four collapse into it"
        );
        assert!(
            find(
                &dom,
                root,
                "data-relation-id",
                "saved-from:more:00000000-0000-0000-0000-000000000002"
            )
            .is_some()
        );
        // Morning's single archive still shows as its own node.
        assert!(find(&dom, root, "data-key", &archive_node_key("saved")).is_some());
    }
    assert!(DEFAULT_MAX_ARCHIVE_NODES < 4);
    let more = find(&dom.borrow(), root, "data-key", &more_key).unwrap();
    runner.dispatch_click(more, PointerClick::at((1., 1.)));
    assert_eq!(
        runner.state().view.expanded,
        Some(TierCard::Session(session_id(2)))
    );
    // A +N node only opens: a second press keeps the card open.
    let more = find(&dom.borrow(), root, "data-key", &more_key).unwrap();
    runner.dispatch_click(more, PointerClick::at((1., 1.)));
    assert_eq!(
        runner.state().view.expanded,
        Some(TierCard::Session(session_id(2)))
    );
    // The card's chips stand in for the collapsed nodes: one pins its popover
    // beside the chip, and Escape hands focus back to the chip.
    let chip = find(&dom.borrow(), root, "data-archive-chip", "evening-2").unwrap();
    runner.dispatch_click(chip, PointerClick::at((1., 1.)));
    let open = {
        let dom = dom.borrow();
        let dialog = find(&dom, root, "role", "dialog").expect("the chip pins a popover");
        assert_eq!(attr(&dom, dialog, "aria-label"), Some("Codicil blake3:e2"));
        find(&dom, dialog, "data-step", "open-archive").unwrap()
    };
    runner.set_focus(Some(open));
    runner.dispatch_key(escape());
    let chip = find(&dom.borrow(), root, "data-archive-chip", "evening-2").unwrap();
    assert_eq!(runner.focus(), Some(chip));
}

#[test]
fn the_host_sets_how_many_archives_show_before_they_collapse() {
    let mut model = tier_model();
    let archive = model.archive.as_mut().unwrap();
    for n in 0..4 {
        archive.entries.push(ArchiveEntry {
            key: format!("evening-{n}"),
            name: format!("Codicil blake3:e{n}"),
            detail: "Read-only archive".into(),
            can_open: true,
            session: Some(session_id(2)),
        });
    }
    let nodes = |model: &MereViewModel| {
        let host = Host::new(model.clone(), 900, 600);
        tier_view(&host)
            .swatch()
            .graph
            .nodes
            .iter()
            .map(|node| node.id.clone())
            .collect::<Vec<_>>()
    };
    let more = more_node_key(TierCard::Session(session_id(2)));
    assert!(
        nodes(&model).contains(&more),
        "four pass the default of three"
    );
    model.max_archive_nodes = Some(4);
    let shown = nodes(&model);
    assert!(!shown.contains(&more), "the host lets four show");
    assert!(shown.contains(&archive_node_key("evening-3")));
    model.max_archive_nodes = Some(0);
    assert!(
        nodes(&model).contains(&more_node_key(TierCard::Session(session_id(1)))),
        "even one collapses at zero"
    );
}

#[test]
fn apply_and_lay_out_lays_the_graph_out_again_for_an_open_card() {
    let model = tier_model();
    let mut state = MereViewState::default();
    state.lay_out(&model, 900, 600);
    let before = state.position("a");
    state.apply_and_lay_out(
        MereViewEvent::Expand(Some(TierCard::Session(session_id(1)))),
        &model,
        900,
        600,
    );
    assert_eq!(state.expanded, Some(TierCard::Session(session_id(1))));
    assert_ne!(
        state.position("a"),
        before,
        "laid out again below the grown tier"
    );
}
