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
        |host: &mut Host, event| host.view.apply(event),
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
    assert_eq!(narrow.graph_size(), (280, 336));
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
