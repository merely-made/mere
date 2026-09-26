/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! The mere view's headed harness (reservoir plan V2b, step 2).
//!
//! A host of its own around [`mere_view`]: a catalog like Knot's as the graph,
//! three sessions, an action slot and a theme of custom properties. It carries
//! out what the view asks, declines what it cannot do with its reason, and
//! tells the scenario lane what it holds. One scenario, headed:
//!
//! ```text
//! MERE_VIEW_SCENARIO=crates/cambium/mere-view/examples/scenarios/<name>.scn \
//! MERE_VIEW_CAPTURE_DIR=<dir> MERE_VIEW_RECEIPT=<dir>/receipt.txt \
//!   cargo run -p mere-view --example harness
//! ```
//!
//! Beyond taproot's verbs and the lane's `resize`, a scenario may press keys
//! through the runner's own keyboard path (`key tab`, `key shift+tab`,
//! `key enter`, `key space`, or `key tab-until <attr>=<value>`), and may
//! `act` one of `decline-next`, `empty`, `unavailable`, `building`, `ready`
//! and `theme-dark`.

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::rc::Rc;

use cambium::{GRAPH_CANVAS_SWATCH_CSS, Key, KeyEvent, NamedKey};
use cambium_genet_winit_host::{
    AppCtx, CaptureRecord, CloseDisposition, HostHooks, HostOptions, Init, LaneApp, LaneConfig,
    ProbeSnapshot, Runner, ScenarioLane, WindowFrame, run,
};
use incipit::SessionId;
use layout_dom_api::{LayoutDom, LocalName, Namespace};
use mere_view::{
    GraphModel, HostAction, MERE_VIEW_CSS, MereView, MereViewElement, MereViewModel,
    MereViewRequest, MereViewState, NodeEntry, NodeState, Provenance, RelationEntry, SessionEntry,
    SessionStep, StatusNote, ViewStatus, mere_view,
};
use sprigging::ColorF;
use uuid::Uuid;

/// The key the graph's paint leaf is registered under.
const GRAPH_LEAF: u64 = 0x6d65_7265;

/// The host's tokens. A host themes the view by these alone; the rules below
/// spend them and name no colour of their own.
const LIGHT: &str = ":root { --paper: #f7f5f0; --ink: #1f2328; --muted: #5d6470; --accent: #2f5d8a; --warn: #8a4b00; --rule: #d9d4c7; --wash: #ece8de; }";
const DARK: &str = ":root { --paper: #16191d; --ink: #e8e6e1; --muted: #9aa1ab; --accent: #8cb4de; --warn: #f0b35a; --rule: #353a42; --wash: #22262c; }";
/// A Cambium tree has no `html` or `body`: the base colour, background and
/// font go on `:root`, which the tokens already use.
const RULES: &str = "\
    :root { background: var(--paper); color: var(--ink); font-family: sans-serif; font-size: 13px; } \
    .mere-view { background: var(--paper); color: var(--ink); } \
    .mere-view-mint { padding: 6px 8px 0; } \
    .mere-view-bar { padding: 0 6px; border-bottom: 1px solid var(--rule); } \
    .mere-view-title { font-weight: bold; } \
    button { margin: 0 2px; padding: 2px 6px; border: 1px solid var(--rule); border-radius: 3px; background: var(--wash); color: var(--ink); } \
    button[aria-pressed=\"true\"] { border-color: var(--accent); color: var(--accent); } \
    .mere-view-notice { color: var(--warn); background: var(--paper); } \
    .mere-view-notice:not(:empty) { padding: 4px 8px; border: 1px solid var(--warn); border-radius: 3px; } \
    .mere-view-sessions { border-right: 1px solid var(--rule); border-bottom: 1px solid var(--rule); } \
    .mere-view-session { padding: 6px 8px; border-bottom: 1px solid var(--rule); } \
    .mere-view-session[data-attached=\"true\"] .mere-view-session-name { color: var(--accent); font-weight: bold; } \
    .mere-view-session[data-trashed=\"true\"] { color: var(--muted); } \
    .mere-view-session-state { color: var(--muted); font-size: 12px; } \
    .mere-view-state { padding: 16px; } \
    .graph-canvas-swatch-node:focus-visible { outline: 2px solid var(--accent); }";

fn sheet(tokens: &str) -> String {
    format!("{GRAPH_CANVAS_SWATCH_CSS}\n{MERE_VIEW_CSS}\n{tokens}\n{RULES}")
}

/// Node paint by state. A real host derives these from its tokens; the words
/// in each node's label carry the state either way.
fn palette(state: &NodeState) -> ColorF {
    let (r, g, b, a) = match state {
        NodeState::Available => (0.18, 0.36, 0.54, 1.0),
        NodeState::Open => (0.12, 0.55, 0.35, 1.0),
        NodeState::Dirty => (0.72, 0.45, 0.05, 1.0),
        NodeState::Unavailable => (0.55, 0.55, 0.55, 0.45),
    };
    ColorF { r, g, b, a }
}

// ------------------------------------------------------------------ the host

struct Harness {
    model: MereViewModel,
    view: MereViewState,
    tile: (u32, u32),
    events: Vec<String>,
    requests: usize,
    decline_next: bool,
    next_session: u128,
    /// Each layout laid out so far, with the placement it gave.
    placements: BTreeMap<String, u64>,
    keys_seen: BTreeSet<Vec<String>>,
}

type Child = MereViewElement<Harness, ()>;
type Logic = fn(&Harness) -> Child;

fn root(harness: &Harness) -> Child {
    mere_view(
        harness.view_of(),
        |harness: &mut Harness, event| harness.view.apply(event),
        |harness: &mut Harness, request| harness.carry_out(request),
    )
}

impl Harness {
    fn new() -> Self {
        let mut harness = Self {
            model: catalog(),
            view: MereViewState::default(),
            tile: (1100, 700),
            events: Vec::new(),
            requests: 0,
            decline_next: false,
            next_session: 4,
            placements: BTreeMap::new(),
            keys_seen: BTreeSet::new(),
        };
        harness.relayout();
        harness
    }

    fn view_of(&self) -> MereView<'_> {
        MereView {
            model: &self.model,
            state: &self.view,
            leaf_key: GRAPH_LEAF,
            width: self.tile.0,
            height: self.tile.1,
        }
    }

    fn relayout(&mut self) {
        self.view.lay_out(&self.model, self.tile.0, self.tile.1);
        let mut keys: Vec<String> = self
            .model
            .graph
            .nodes
            .iter()
            .map(|n| n.key.clone())
            .collect();
        keys.sort();
        let mut hasher = DefaultHasher::new();
        for key in &keys {
            let (x, y) = self.view.position(key).unwrap_or((0.5, 0.5));
            ((x * 1000.0) as i32, (y * 1000.0) as i32).hash(&mut hasher);
        }
        self.placements
            .insert(self.view_of().layout().to_string(), hasher.finish());
        self.keys_seen.insert(keys);
    }

    fn name_of(&self, id: SessionId) -> String {
        self.model
            .sessions
            .iter()
            .find(|session| session.id == id)
            .map(|session| session.name.clone())
            .unwrap_or_default()
    }

    /// Carry out what the view asked, or decline it with the reason.
    fn carry_out(&mut self, request: MereViewRequest) {
        self.requests += 1;
        let what = match &request {
            MereViewRequest::Activate(key) => format!("open {key}"),
            MereViewRequest::Mint => "mint a session".to_string(),
            MereViewRequest::Session(id, step) => format!("{} {}", step.token(), self.name_of(*id)),
            MereViewRequest::Host(key) => format!("run {key}"),
            MereViewRequest::Layout(id) => format!("lay out by {id}"),
        };
        self.events.push(format!("request {what}"));
        if std::mem::take(&mut self.decline_next) {
            self.decline(format!("The host declined to {what}."));
            return;
        }
        self.model.notice = None;
        match request {
            MereViewRequest::Activate(key) => {
                let Some(node) = self.model.graph.nodes.iter_mut().find(|n| n.key == key) else {
                    return;
                };
                match node.state {
                    NodeState::Unavailable => {
                        let reason = format!("{} is unavailable: its file is missing.", node.label);
                        self.decline(reason);
                        return;
                    },
                    NodeState::Available => node.state = NodeState::Open,
                    NodeState::Open | NodeState::Dirty => {},
                }
                self.events.push(format!("opened {key}"));
            },
            MereViewRequest::Mint => {
                let name = format!("Session {}", self.next_session);
                self.push_session(name);
            },
            MereViewRequest::Session(id, SessionStep::Switch) => {
                for session in &mut self.model.sessions {
                    session.attached = session.id == id;
                }
            },
            MereViewRequest::Session(id, SessionStep::Fork) => {
                let name = format!("Fork of {}", self.name_of(id));
                self.push_session(name);
            },
            MereViewRequest::Session(id, step @ (SessionStep::Trash | SessionStep::Restore)) => {
                if let Some(session) = self.model.sessions.iter_mut().find(|s| s.id == id) {
                    session.trashed = step == SessionStep::Trash;
                }
            },
            MereViewRequest::Host(key) => {
                if ["configure", "retry", "cancel"].contains(&key.as_str()) {
                    self.model.status = ViewStatus::Ready;
                }
            },
            MereViewRequest::Layout(id) => self.model.layout = id,
        }
        for session in &mut self.model.sessions {
            session.steps = steps_for(session.attached, session.trashed);
        }
        self.relayout();
    }

    fn decline(&mut self, reason: String) {
        self.events.push(format!("declined: {reason}"));
        self.model.notice = Some(reason);
    }

    fn push_session(&mut self, name: String) {
        let id = session_id(self.next_session);
        self.next_session += 1;
        self.model.sessions.push(SessionEntry {
            id,
            name,
            detail: Some("just made".into()),
            attached: false,
            trashed: false,
            steps: Vec::new(),
        });
    }

    fn set_status(&mut self, label: &str) -> bool {
        let note = |message: &str, key: &str, action: &str| StatusNote {
            message: message.into(),
            action: Some(HostAction::new(key, action)),
        };
        self.model.status = match label {
            "empty" => ViewStatus::Empty(note(
                "No catalog is configured.",
                "configure",
                "Choose a catalog",
            )),
            "unavailable" => ViewStatus::Unavailable(note(
                "The reservoir is locked by another resident.",
                "retry",
                "Try again",
            )),
            "building" => {
                ViewStatus::Building(note("Still reading links.", "cancel", "Stop reading"))
            },
            "ready" => ViewStatus::Ready,
            _ => return false,
        };
        true
    }
}

fn session_id(n: u128) -> SessionId {
    SessionId::from_uuid(Uuid::from_u128(n))
}

fn steps_for(attached: bool, trashed: bool) -> Vec<SessionStep> {
    if trashed {
        vec![SessionStep::Restore]
    } else if attached {
        vec![SessionStep::Fork, SessionStep::Trash]
    } else {
        vec![SessionStep::Switch, SessionStep::Fork, SessionStep::Trash]
    }
}

/// A catalog like Knot's: documents as nodes, links read out of them, a few
/// relations a person made, and two suggestions.
fn catalog() -> MereViewModel {
    let nodes = [
        ("index", "Index", NodeState::Open),
        ("garden", "Garden notes", NodeState::Available),
        ("seeds", "Seeds", NodeState::Available),
        ("prompts", "Prompts", NodeState::Available),
        ("tarot", "Tarot notes", NodeState::Dirty),
        ("moon", "Moon phases", NodeState::Unavailable),
        ("journal", "Journal", NodeState::Available),
        ("archive", "Archive 2025", NodeState::Unavailable),
        ("reading", "Reading list", NodeState::Available),
        ("drafts", "Drafts", NodeState::Available),
        ("ideas", "Ideas", NodeState::Available),
        ("links", "Links", NodeState::Available),
    ]
    .map(|(key, label, state)| NodeEntry {
        key: key.into(),
        label: label.into(),
        state,
    });
    let relations = [
        ("index-garden", "index", "garden", Provenance::Extracted),
        ("index-garden-note", "index", "garden", Provenance::Authored),
        ("index-seeds", "index", "seeds", Provenance::Extracted),
        ("index-prompts", "index", "prompts", Provenance::Extracted),
        ("index-journal", "index", "journal", Provenance::Extracted),
        ("index-links", "index", "links", Provenance::Extracted),
        ("garden-seeds", "garden", "seeds", Provenance::Authored),
        ("prompts-ideas", "prompts", "ideas", Provenance::Suggested),
        ("tarot-moon", "tarot", "moon", Provenance::Extracted),
        ("journal-tarot", "journal", "tarot", Provenance::Extracted),
        (
            "journal-archive",
            "journal",
            "archive",
            Provenance::Extracted,
        ),
        ("reading-links", "reading", "links", Provenance::Extracted),
        ("drafts-ideas", "drafts", "ideas", Provenance::Authored),
        ("drafts-reading", "drafts", "reading", Provenance::Suggested),
    ]
    .map(|(key, from, to, provenance)| RelationEntry {
        key: key.into(),
        from: from.into(),
        to: to.into(),
        provenance,
    });
    let session = |n: u128, name: &str, attached: bool, trashed: bool| SessionEntry {
        id: session_id(n),
        name: name.into(),
        detail: None,
        attached,
        trashed,
        steps: steps_for(attached, trashed),
    };
    MereViewModel {
        title: "notes".into(),
        status: ViewStatus::Ready,
        sessions: vec![
            session(1, "Morning notes", true, false),
            session(2, "Evening notes", false, false),
            session(3, "Old draft", false, true),
        ],
        can_mint: true,
        graph: GraphModel {
            nodes: nodes.to_vec(),
            relations: relations.to_vec(),
        },
        layout: "spectral.default".into(),
        layouts: vec![
            ("spectral.default".into(), "Spectral".into()),
            ("phyllotaxis.default".into(), "Spiral".into()),
            ("grid.default".into(), "Grid".into()),
        ],
        actions: vec![
            HostAction::new("new", "New"),
            HostAction::new("open", "Open"),
            HostAction::new("recent", "Recent"),
        ],
        notice: None,
    }
}

// ------------------------------------------------------------------ the lane

struct HarnessLane {
    sheet: String,
}

/// The focused element, by the attribute that names it.
fn focus_name(runner: &Runner<Harness, Logic, Child>) -> String {
    let Some(node) = runner.focus() else {
        return "none".into();
    };
    let dom = runner.dom();
    let dom = dom.borrow();
    for name in [
        "data-key",
        "data-step",
        "data-action",
        "data-layout",
        "data-request",
        "data-relation-id",
    ] {
        if let Some(value) = dom.attribute(node, &Namespace::from(""), &LocalName::from(name)) {
            return format!("{name}={value}");
        }
    }
    "other".into()
}

fn count_attr(runner: &Runner<Harness, Logic, Child>, name: &str) -> usize {
    fn walk(
        dom: &genet_scripted_dom::ScriptedDom,
        node: genet_scripted_dom::NodeId,
        name: &LocalName,
    ) -> usize {
        let here = usize::from(dom.attribute(node, &Namespace::from(""), name).is_some());
        here + dom
            .dom_children(node)
            .map(|child| walk(dom, child, name))
            .sum::<usize>()
    }
    let dom = runner.dom();
    let dom = dom.borrow();
    walk(&dom, runner.root(), &LocalName::from(name))
}

impl LaneApp<Harness, Logic, Child> for HarnessLane {
    fn sheet(&self) -> &str {
        &self.sheet
    }

    fn snapshot(&self, ctx: &AppCtx<'_, Harness, Logic, Child>) -> ProbeSnapshot {
        let harness = ctx.runner.state();
        let model = &harness.model;
        let open: Vec<&str> = model
            .graph
            .nodes
            .iter()
            .filter(|node| node.state == NodeState::Open)
            .map(|node| node.key.as_str())
            .collect();
        let mut keys: Vec<&str> = model.graph.nodes.iter().map(|n| n.key.as_str()).collect();
        keys.sort();
        let view = harness.view_of();
        let (graph_width, graph_height) = view.graph_size();
        ProbeSnapshot::default()
            .with_field("tile", format!("{}x{}", harness.tile.0, harness.tile.1))
            .with_field(
                "width",
                if harness.tile.0 >= 560 {
                    "wide"
                } else {
                    "narrow"
                },
            )
            .with_field("graph", format!("{graph_width}x{graph_height}"))
            .with_field("status", model.status.token())
            .with_field("layout", view.layout().to_string())
            .with_field("keys", keys.join(","))
            .with_field("open", open.join(","))
            .with_field("sessions", model.sessions.len().to_string())
            .with_field(
                "trashed",
                model
                    .sessions
                    .iter()
                    .filter(|s| s.trashed)
                    .count()
                    .to_string(),
            )
            .with_field(
                "attached",
                model
                    .sessions
                    .iter()
                    .find(|s| s.attached)
                    .map(|s| s.name.clone())
                    .unwrap_or_default(),
            )
            .with_field(
                "notice",
                model.notice.clone().unwrap_or_else(|| "none".into()),
            )
            .with_field("requests", harness.requests.to_string())
            .with_field("focus", focus_name(ctx.runner))
            .with_field(
                "node_targets",
                count_attr(ctx.runner, "data-key").to_string(),
            )
            .with_field(
                "relation_cells",
                count_attr(ctx.runner, "data-relation-id").to_string(),
            )
            // A layout switch must move the nodes and keep every key.
            .with_field("layouts_laid", harness.placements.len().to_string())
            .with_field(
                "placements",
                harness
                    .placements
                    .values()
                    .collect::<BTreeSet<_>>()
                    .len()
                    .to_string(),
            )
            .with_field("key_sets", harness.keys_seen.len().to_string())
    }

    fn drain_events(&mut self, ctx: &mut AppCtx<'_, Harness, Logic, Child>) -> Vec<String> {
        let mut drained = Vec::new();
        ctx.runner
            .update(|harness| drained = std::mem::take(&mut harness.events));
        drained
    }

    fn act(&mut self, ctx: &mut AppCtx<'_, Harness, Logic, Child>, label: &str) -> bool {
        match label {
            "decline-next" => {
                ctx.runner.update(|harness| harness.decline_next = true);
                true
            },
            "theme-dark" => {
                self.sheet = sheet(DARK);
                *ctx.set_sheet = Some(self.sheet.clone());
                true
            },
            other => {
                let mut known = false;
                ctx.runner
                    .update(|harness| known = harness.set_status(other));
                known
            },
        }
    }

    /// `key <chord>` and `key tab-until <attr>=<value>`, through the runner's
    /// own keyboard path.
    fn app_step(
        &mut self,
        ctx: &mut AppCtx<'_, Harness, Logic, Child>,
        line: &str,
    ) -> Result<(), String> {
        let mut parts = line.split_whitespace();
        if parts.next() != Some("key") {
            return Err(format!("unknown verb: {line}"));
        }
        let chord = parts.next().ok_or("key wants a key")?;
        if chord == "tab-until" {
            let target = parts.next().ok_or("tab-until wants <attr>=<value>")?;
            for _ in 0..120 {
                if focus_name(ctx.runner) == target {
                    return Ok(());
                }
                ctx.runner
                    .dispatch_key(KeyEvent::new(Key::Named(NamedKey::Tab)));
            }
            return Err(format!("Tab never reached {target}"));
        }
        let (shift, name) = chord
            .strip_prefix("shift+")
            .map_or((false, chord), |name| (true, name));
        let key = match name {
            "tab" => NamedKey::Tab,
            "enter" => NamedKey::Enter,
            "space" => NamedKey::Space,
            "escape" => NamedKey::Escape,
            other => return Err(format!("key: unknown key {other}")),
        };
        let mut event = KeyEvent::new(Key::Named(key));
        event.mods.shift = shift;
        ctx.runner.dispatch_key(event);
        Ok(())
    }

    fn busy(&mut self, _ctx: &mut AppCtx<'_, Harness, Logic, Child>) -> Option<bool> {
        Some(false)
    }

    /// Swapping only the tokens must change the frame.
    fn receipt_checks(&self, captures: &[CaptureRecord]) -> Vec<String> {
        let digest = |name: &str| captures.iter().find(|c| c.name == name).map(|c| c.digest);
        match (digest("r9_centre_light"), digest("r9_centre_dark")) {
            (Some(light), Some(dark)) if light == dark => {
                vec!["the dark tokens left the frame unchanged".to_string()]
            },
            _ => Vec::new(),
        }
    }
}

// ------------------------------------------------------------------ wiring

fn main() {
    let lane = LaneConfig::from_env("MERE_VIEW").map(|config| {
        let path = config.scenario.display().to_string();
        let lane = ScenarioLane::new(
            config,
            HarnessLane {
                sheet: sheet(LIGHT),
            },
        )
        .unwrap_or_else(|error| panic!("{error}"));
        eprintln!("[mere-view] scenario armed: {path}");
        lane
    });
    let lane = Rc::new(RefCell::new(lane));

    let after_frame_lane = lane.clone();
    let hooks: HostHooks<Harness, Logic, Child> = HostHooks {
        // Follow the window, and hand the host the graph's paint.
        frame: Box::new(|ctx: &mut AppCtx<'_, Harness, Logic, Child>| {
            let tile = (
                ctx.logical_size.0.round().max(1.0) as u32,
                ctx.logical_size.1.round().max(1.0) as u32,
            );
            if ctx.runner.state().tile != tile {
                ctx.runner.update(|harness| {
                    harness.tile = tile;
                    harness.relayout();
                });
            }
            let leaf = ctx.runner.state().view_of().paint_leaf(palette);
            ctx.leaves.insert(GRAPH_LEAF, Box::new(leaf));
            false
        }),
        after_dispatch: Box::new(|_ctx| {}),
        after_frame: Box::new(move |ctx: &mut AppCtx<'_, Harness, Logic, Child>| {
            if let Some(lane) = after_frame_lane.borrow_mut().as_mut() {
                lane.drive(ctx);
            }
        }),
        after_wake: Box::new(|_ctx| {}),
        close_request: Box::new(|_ctx, _request| CloseDisposition::Exit),
        focused_text: Box::new(|_runner: &Runner<Harness, Logic, Child>| None),
        key_intercept: Box::new(|_runner, _press| false),
    };

    let options = HostOptions {
        title: "mere view harness".into(),
        initial_logical_size: (1100.0, 700.0),
        size_env: Some(("MERE_VIEW_WIDTH".into(), "MERE_VIEW_HEIGHT".into())),
        window_frame: WindowFrame::Host,
        ..Default::default()
    };
    run(
        options,
        move |_window, _commands, _wake| Init {
            state: Harness::new(),
            logic: root as Logic,
            sheet: sheet(LIGHT),
            fonts: Vec::new(),
            images: Vec::new(),
        },
        hooks,
    )
    .expect("event loop");
}
