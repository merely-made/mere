/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Experiment E1b's probe: one window, two retained Cambium sessions joined as
//! AccessKit subtrees, for a person to walk with a screen reader (app
//! composition brief §8, AC3, ruled 2026-10-07).
//!
//! The window holds a host menu button and two panes. Pane A is a text field
//! (type into it; an input method's candidate window sits at its caret). Pane B
//! is a list with a counting button. The window's title shows B's count and
//! where focus is, for a sighted operator; the panes are not painted.
//!
//! ```text
//! cargo run -p cambium-winit-a11y --example e1b_two_sessions
//! ```
//!
//! The walk, on each system (Narrator or NVDA, VoiceOver, Orca on Fedora and
//! Mint), is done when:
//!
//! 1. the reader walks the host menu, A's field, and B's list to its last
//!    item, reading names;
//! 2. typing in A is echoed;
//! 3. B's button, activated by the reader's default action, raises B's count
//!    (the title and the button's name both say so);
//! 4. Tab cycles host, A, B and host;
//! 5. composing with a CJK input method in A puts the candidate window at A's
//!    field.
//!
//! Positive controls, which must fail the walk:
//!
//! - `--omit-a` leaves A's graft out: A's field must be unreachable.
//! - `--unboxed` strips B's button of its box: on Windows, Narrator stops
//!   there, as it did on 2026-08-20.
//!
//! `--guests 0|1|2` (default 2) and the printed resident memory are the
//! brief's recorded, unbarred measure. `--self-check` runs without a window:
//! it reads the joined tree through the AT-SPI consumer and exits.

use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use accesskit::{
    Action, ActionHandler, ActionRequest, ActivationHandler, DeactivationHandler, Node,
    NodeId as A11yNodeId, Rect, Role, Tree, TreeId, TreeUpdate,
};
use cambium::CompositionEvent;
use cambium::{
    AnyView, ContainedSession, FocusExit, GenetAppRunner, GenetCtx, GenetElement, Key, KeyEvent,
    NamedKey, PointerClick, ResolvedSurfaceEvent, RetainedSurfaceSession, RunnerSurfaceSession,
    SurfaceEffect, clickable, el, focusable, on_key, text,
};
use cambium_rootstock::{OwnedLayout, ProducerRegistry};
use genet_scripted_dom::{NodeId, ScriptedDom};
use layout_dom_api::LayoutDom;
use mere_surface_api::{
    ProviderId, SourceKindId, SurfaceAvailability, SurfaceDescriptor, SurfaceId, SurfaceSourceShape,
};
use sprigging::LeafRegistry;
use uxtree::{ActionTarget, Composition, GraftTable, Grafts, graft_node, tree_id_for_path};
use winit::application::ApplicationHandler;
use winit::dpi::{LogicalPosition, LogicalSize};
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy};
use winit::keyboard::{Key as WinitKey, ModifiersState, NamedKey as WinitNamedKey};
use winit::window::{Window, WindowId};

const PANE: (f32, f32) = (240.0, 160.0);
const A_ORIGIN: (f64, f64) = (0.0, 40.0);
const B_ORIGIN: (f64, f64) = (240.0, 40.0);
const WINDOW: (f64, f64) = (480.0, 220.0);
const MENU: A11yNodeId = A11yNodeId(2);
const A_GRAFT: A11yNodeId = A11yNodeId(10);
const B_GRAFT: A11yNodeId = A11yNodeId(11);

fn place(x: i32, y: i32, w: i32, h: i32) -> String {
    format!("position:absolute;left:{x}px;top:{y}px;width:{w}px;height:{h}px;")
}

// --- The two guests -------------------------------------------------------

#[derive(Default)]
struct Notes {
    text: String,
}

#[derive(Default)]
struct Counter {
    count: u32,
}

type NotesView = Box<dyn AnyView<Notes, (), GenetCtx, GenetElement>>;
type CounterView = Box<dyn AnyView<Counter, (), GenetCtx, GenetElement>>;

fn notes_view(state: &Notes) -> NotesView {
    let typing: fn(&mut Notes, KeyEvent) = |state, event| match event.key {
        Key::Character(text) => state.text.push_str(&text),
        Key::Named(NamedKey::Backspace) => {
            state.text.pop();
        },
        Key::Composition(CompositionEvent::Commit(text)) => state.text.push_str(&text),
        _ => {},
    };
    Box::new(
        el(
            "div",
            on_key(
                el("input", ())
                    .attr("type", "text")
                    .attr("aria-label", "A field")
                    .attr("value", state.text.clone())
                    .attr("style", place(10, 10, 200, 30)),
                typing,
            ),
        )
        .attr("role", "main")
        .attr("aria-label", "A notes")
        .attr("style", "position:relative;width:240px;height:160px;"),
    )
}

fn counter_view(state: &Counter) -> CounterView {
    Box::new(
        el(
            "div",
            (
                focusable(clickable(
                    el("button", text(format!("B count {}", state.count)))
                        .attr("style", place(10, 10, 100, 30)),
                    |state: &mut Counter, _| state.count += 1,
                )),
                el("div", text("B first item"))
                    .attr("role", "listitem")
                    .attr("style", place(10, 50, 200, 24)),
                el("div", text("B last item"))
                    .attr("role", "listitem")
                    .attr("style", place(10, 80, 200, 24)),
            ),
        )
        .attr("role", "list")
        .attr("aria-label", "B list")
        .attr("style", "position:relative;width:240px;height:160px;"),
    )
}

fn descriptor(id: &str) -> SurfaceDescriptor {
    SurfaceDescriptor {
        provider_id: ProviderId::from("e1b"),
        surface_id: SurfaceId::from(id),
        label: id.to_owned(),
        accepted_source: SurfaceSourceShape::One(SourceKindId::from("e1b.source")),
    }
}

fn session_a() -> Box<dyn RetainedSurfaceSession> {
    let dom = std::rc::Rc::new(std::cell::RefCell::new(ScriptedDom::new()));
    let runner = GenetAppRunner::new(dom, notes_view, Notes::default());
    let mut session = ContainedSession::new(RunnerSurfaceSession::new(
        descriptor("e1b.a"),
        runner,
        |_: &Notes| SurfaceAvailability::Available,
        |_: &mut Notes, _| {},
        |_action: ()| Vec::new(),
    ));
    session.set_focus_exits(true);
    Box::new(session)
}

fn session_b() -> Box<dyn RetainedSurfaceSession> {
    let dom = std::rc::Rc::new(std::cell::RefCell::new(ScriptedDom::new()));
    let runner = GenetAppRunner::new(dom, counter_view, Counter::default());
    let mut session = ContainedSession::new(RunnerSurfaceSession::new(
        descriptor("e1b.b"),
        runner,
        |_: &Counter| SurfaceAvailability::Available,
        |_: &mut Counter, _| {},
        |_action: ()| Vec::new(),
    ));
    session.set_focus_exits(true);
    Box::new(session)
}

// --- The host -------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Pane {
    A,
    B,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Stop {
    Menu,
    Pane(Pane),
}

#[derive(Clone, Copy, Default)]
struct Options {
    omit_a: bool,
    unboxed: bool,
    guests: u8,
    self_check: bool,
}

impl Options {
    fn parse() -> Self {
        let mut options = Self {
            guests: 2,
            ..Self::default()
        };
        let mut args = std::env::args().skip(1);
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--omit-a" => options.omit_a = true,
                "--unboxed" => options.unboxed = true,
                "--self-check" => options.self_check = true,
                "--guests" => {
                    options.guests = args
                        .next()
                        .and_then(|count| count.parse().ok())
                        .filter(|count| *count <= 2)
                        .expect("--guests takes 0, 1 or 2");
                },
                other => panic!("unknown argument {other}"),
            }
        }
        options
    }
}

struct Projected {
    update: TreeUpdate,
    nodes: HashMap<A11yNodeId, NodeId>,
    field: Option<(f32, f32, f32, f32)>,
}

fn project(session: &mut dyn RetainedSurfaceSession) -> Projected {
    let dom = session.dom();
    let focus = session.focus();
    let dom = dom.borrow();
    let layout = OwnedLayout::new(&*dom, &[""], PANE.0, PANE.1, &[], &HashMap::new());
    let focus = focus.map(|node| dom.opaque_id(node));
    let mut leaves = LeafRegistry::new();
    let (update, nodes) = cambium_winit_a11y::project_tree(
        &*dom,
        &layout,
        &mut leaves,
        &mut ProducerRegistry::new(),
        focus,
    );
    let field = session
        .focusables()
        .first()
        .and_then(|node| layout.painted_rect(&*dom, *node));
    Projected {
        update,
        nodes,
        field,
    }
}

struct Host {
    options: Options,
    a: Option<Box<dyn RetainedSurfaceSession>>,
    b: Option<Box<dyn RetainedSurfaceSession>>,
    focus: Stop,
    a_tree: TreeId,
    b_tree: TreeId,
    table: GraftTable<Pane>,
    a_nodes: HashMap<A11yNodeId, NodeId>,
    b_nodes: HashMap<A11yNodeId, NodeId>,
    /// A's field box in window coordinates, for the input method.
    a_field: Option<(f64, f64, f64, f64)>,
}

impl Host {
    fn new(options: Options) -> Self {
        let (a_tree, b_tree) = (tree_id_for_path("e1b/a#1"), tree_id_for_path("e1b/b#1"));
        let mut table = GraftTable::new(TreeId::ROOT);
        let a = (options.guests >= 1).then(session_a);
        let b = (options.guests >= 2).then(session_b);
        if a.is_some() && !options.omit_a {
            table.insert(a_tree, Pane::A);
        }
        if b.is_some() {
            table.insert(b_tree, Pane::B);
        }
        Self {
            options,
            a,
            b,
            focus: Stop::Menu,
            a_tree,
            b_tree,
            table,
            a_nodes: HashMap::new(),
            b_nodes: HashMap::new(),
            a_field: None,
        }
    }

    fn a_grafted(&self) -> bool {
        self.a.is_some() && !self.options.omit_a
    }

    fn session(&mut self, pane: Pane) -> Option<&mut Box<dyn RetainedSurfaceSession>> {
        match pane {
            Pane::A => self.a.as_mut(),
            Pane::B => self.b.as_mut(),
        }
    }

    /// Host stops in Tab order: the menu, then each live pane.
    fn order(&self) -> Vec<Stop> {
        let mut order = vec![Stop::Menu];
        if self.a_grafted() {
            order.push(Stop::Pane(Pane::A));
        }
        if self.b.is_some() {
            order.push(Stop::Pane(Pane::B));
        }
        order
    }

    /// Move to the next or previous host stop; a pane is entered at its first
    /// (or, backward, last) focusable.
    fn step(&mut self, forward: bool) {
        let order = self.order();
        let at = order
            .iter()
            .position(|stop| *stop == self.focus)
            .unwrap_or(0);
        let next = if forward {
            (at + 1) % order.len()
        } else {
            (at + order.len() - 1) % order.len()
        };
        self.focus = order[next];
        if let Stop::Pane(pane) = self.focus
            && let Some(session) = self.session(pane)
        {
            session.set_focus(None);
            session.focus_traverse(forward);
        }
    }

    fn follow(&mut self, effects: Vec<SurfaceEffect>) {
        for effect in effects {
            if let SurfaceEffect::FocusExit(exit) = effect {
                self.step(exit == FocusExit::Forward);
            }
        }
    }

    fn key(&mut self, event: KeyEvent) {
        let tab = matches!(event.key, Key::Named(NamedKey::Tab));
        match self.focus {
            Stop::Menu if tab => self.step(!event.mods.shift),
            Stop::Menu => {},
            Stop::Pane(pane) => {
                let effects = match self.session(pane) {
                    Some(session) => session.dispatch(ResolvedSurfaceEvent::Key(event)),
                    None => Vec::new(),
                };
                self.follow(effects);
            },
        }
    }

    fn act(&mut self, request: &ActionRequest) {
        let (pane, nodes) = match self.table.route(request) {
            ActionTarget::Host => {
                if request.action == Action::Focus && request.target_node == MENU {
                    self.focus = Stop::Menu;
                }
                return;
            },
            ActionTarget::Guest(Pane::A) => (Pane::A, &self.a_nodes),
            ActionTarget::Guest(Pane::B) => (Pane::B, &self.b_nodes),
            ActionTarget::Unknown(_) => return,
        };
        let Some(&target) = nodes.get(&request.target_node) else {
            return;
        };
        match request.action {
            Action::Focus => {
                self.focus = Stop::Pane(pane);
                if let Some(session) = self.session(pane) {
                    session.set_focus(Some(target));
                }
            },
            Action::Click => {
                self.focus = Stop::Pane(pane);
                let effects = match self.session(pane) {
                    Some(session) => session.dispatch(ResolvedSurfaceEvent::Click {
                        target,
                        event: PointerClick::at((0.0, 0.0)),
                    }),
                    None => Vec::new(),
                };
                self.follow(effects);
            },
            _ => {},
        }
    }

    fn host_update(&self) -> TreeUpdate {
        let mut children = vec![MENU];
        if self.a_grafted() {
            children.push(A_GRAFT);
        }
        if self.b.is_some() {
            children.push(B_GRAFT);
        }
        let mut window = Node::new(Role::Window);
        window.set_label("E1b probe");
        window.set_bounds(Rect::new(0.0, 0.0, WINDOW.0, WINDOW.1));
        window.set_children(children);
        let mut menu = Node::new(Role::Button);
        menu.set_label("Host menu");
        menu.set_bounds(Rect::new(4.0, 4.0, 104.0, 34.0));
        menu.add_action(Action::Focus);
        let pane = (f64::from(PANE.0), f64::from(PANE.1));
        let mut nodes = vec![(A11yNodeId(1), window), (MENU, menu)];
        if self.a_grafted() {
            let mut graft = graft_node(Role::Pane, self.a_tree, A_ORIGIN, pane);
            graft.set_label("Pane A");
            nodes.push((A_GRAFT, graft));
        }
        if self.b.is_some() {
            let mut graft = graft_node(Role::Pane, self.b_tree, B_ORIGIN, pane);
            graft.set_label("Pane B");
            nodes.push((B_GRAFT, graft));
        }
        let focus = match self.focus {
            Stop::Pane(Pane::A) if self.a_grafted() => A_GRAFT,
            Stop::Pane(Pane::B) if self.b.is_some() => B_GRAFT,
            _ => MENU,
        };
        TreeUpdate {
            nodes,
            tree: Some(Tree::new(A11yNodeId(1))),
            tree_id: TreeId::ROOT,
            focus,
        }
    }

    fn composition(&mut self) -> Composition {
        let mut guests = Vec::new();
        self.a_field = None;
        if self.a_grafted() {
            let a = project(self.a.as_mut().expect("A is live").as_mut());
            self.a_field = a.field.map(|(x, y, w, h)| {
                (
                    A_ORIGIN.0 + f64::from(x),
                    A_ORIGIN.1 + f64::from(y),
                    f64::from(w),
                    f64::from(h),
                )
            });
            self.a_nodes = a.nodes;
            guests.push((self.a_tree, a.update));
        }
        if let Some(b) = self.b.as_mut() {
            let mut b = project(b.as_mut());
            if self.options.unboxed {
                strip_one_box(&mut b.update, "B count");
            }
            self.b_nodes = b.nodes;
            guests.push((self.b_tree, b.update));
        }
        Composition::new(self.host_update(), guests).expect("the probe composes a whole frame")
    }

    fn title(&mut self) -> String {
        let count = self
            .b
            .as_ref()
            .map(|b| {
                let dom = b.dom();
                let dom = dom.borrow();
                text_containing(&dom, "B count").unwrap_or_default()
            })
            .unwrap_or_default();
        let at = match self.focus {
            Stop::Menu => "host menu",
            Stop::Pane(Pane::A) => "pane A",
            Stop::Pane(Pane::B) => "pane B",
        };
        format!("E1b probe: focus on {at}; {count}")
    }
}

/// Positive control: one node with no box.
fn strip_one_box(update: &mut TreeUpdate, label_prefix: &str) {
    if let Some((_, node)) = update.nodes.iter_mut().find(|(_, node)| {
        node.label()
            .is_some_and(|label| label.starts_with(label_prefix))
    }) {
        node.clear_bounds();
    }
}

fn text_containing(dom: &ScriptedDom, needle: &str) -> Option<String> {
    fn walk(dom: &ScriptedDom, node: NodeId, needle: &str) -> Option<String> {
        if let Some(text) = dom.text(node)
            && text.contains(needle)
        {
            return Some(text.to_owned());
        }
        dom.dom_children(node)
            .find_map(|child| walk(dom, child, needle))
    }
    walk(dom, dom.document(), needle)
}

fn resident_memory() -> Option<String> {
    let statm = std::fs::read_to_string("/proc/self/statm").ok()?;
    let pages: u64 = statm.split_whitespace().nth(1)?.parse().ok()?;
    Some(format!("{} MiB resident", pages * 4096 / (1024 * 1024)))
}

// --- The window -----------------------------------------------------------

struct Shared {
    /// The host's tree for an activation request, refreshed every frame.
    initial: Arc<Mutex<Option<TreeUpdate>>>,
    /// Set when the adapter activates (a screen reader arrived), possibly on
    /// its own thread: the next frame resends every guest as new.
    activated: Arc<AtomicBool>,
    actions: Arc<Mutex<VecDeque<ActionRequest>>>,
}

struct Activation {
    initial: Arc<Mutex<Option<TreeUpdate>>>,
    activated: Arc<AtomicBool>,
    proxy: EventLoopProxy<()>,
}

impl ActivationHandler for Activation {
    fn request_initial_tree(&mut self) -> Option<TreeUpdate> {
        self.activated.store(true, Ordering::SeqCst);
        let _ = self.proxy.send_event(());
        self.initial.lock().ok().and_then(|initial| initial.clone())
    }
}

struct Actions {
    queue: Arc<Mutex<VecDeque<ActionRequest>>>,
    proxy: EventLoopProxy<()>,
}

impl ActionHandler for Actions {
    fn do_action(&mut self, request: ActionRequest) {
        if let Ok(mut queue) = self.queue.lock() {
            queue.push_back(request);
        }
        let _ = self.proxy.send_event(());
    }
}

struct Deactivation;

impl DeactivationHandler for Deactivation {
    fn deactivate_accessibility(&mut self) {}
}

struct App {
    host: Host,
    grafts: Grafts,
    shared: Shared,
    proxy: EventLoopProxy<()>,
    window: Option<Window>,
    adapter: Option<accesskit_winit::Adapter>,
    modifiers: ModifiersState,
}

impl App {
    /// Publish this frame's trees, in the order the adapter must take them.
    fn publish(&mut self) {
        let composition = self.host.composition();
        if let Ok(mut initial) = self.shared.initial.lock() {
            *initial = Some(composition.initial_host());
        }
        if self.shared.activated.swap(false, Ordering::SeqCst) {
            let _ = self.grafts.activate(&composition);
        }
        if let Some(adapter) = self.adapter.as_mut() {
            for update in self.grafts.frame(composition) {
                adapter.update_if_active(|| update);
            }
        }
        if let Some(window) = &self.window {
            window.set_title(&self.host.title());
            let typing = self.host.focus == Stop::Pane(Pane::A);
            window.set_ime_allowed(typing);
            if typing && let Some((x, y, w, h)) = self.host.a_field {
                window.set_ime_cursor_area(LogicalPosition::new(x, y), LogicalSize::new(w, h));
            }
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let attributes = Window::default_attributes()
            .with_title("E1b probe")
            .with_inner_size(LogicalSize::new(WINDOW.0, WINDOW.1))
            .with_visible(false);
        let window = event_loop
            .create_window(attributes)
            .expect("the probe's window opens");
        // The first frame's host tree answers activation; the adapter must
        // exist before the window is first shown.
        let composition = self.host.composition();
        *self.shared.initial.lock().expect("unpoisoned") = Some(composition.initial_host());
        let adapter = accesskit_winit::Adapter::with_direct_handlers(
            event_loop,
            &window,
            Activation {
                initial: Arc::clone(&self.shared.initial),
                activated: Arc::clone(&self.shared.activated),
                proxy: self.proxy.clone(),
            },
            Actions {
                queue: Arc::clone(&self.shared.actions),
                proxy: self.proxy.clone(),
            },
            Deactivation,
        );
        self.adapter = Some(adapter);
        window.set_visible(true);
        self.window = Some(window);
        self.publish();
        if let Some(memory) = resident_memory() {
            println!("E1b: {} guest(s), {memory}", self.host.options.guests);
        }
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, _event: ()) {
        let requests: Vec<ActionRequest> = match self.shared.actions.lock() {
            Ok(mut queue) => queue.drain(..).collect(),
            Err(_) => return,
        };
        for request in &requests {
            self.host.act(request);
        }
        if !requests.is_empty() || self.shared.activated.load(Ordering::SeqCst) {
            self.publish();
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        if let (Some(window), Some(adapter)) = (&self.window, self.adapter.as_mut()) {
            adapter.process_event(window, &event);
        }
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::ModifiersChanged(modifiers) => self.modifiers = modifiers.state(),
            WindowEvent::KeyboardInput { event, .. } if event.state == ElementState::Pressed => {
                if event.logical_key == WinitKey::Named(WinitNamedKey::Escape) {
                    event_loop.exit();
                    return;
                }
                let mods = cambium_winit::modifiers_from_winit(self.modifiers);
                if let Some(key) = cambium_winit::key_event_from_winit(&event.logical_key, mods) {
                    self.host.key(key);
                    self.publish();
                }
            },
            WindowEvent::Ime(ime) if self.host.focus == Stop::Pane(Pane::A) => {
                self.host.key(cambium_winit::ime_event_from_winit(&ime));
                self.publish();
            },
            _ => {},
        }
    }
}

/// Without a window: join the trees, read them through the AT-SPI consumer,
/// and walk Tab and a Click once, printing what a reader would meet.
fn self_check(options: Options) {
    use accesskit_consumer_unix::{Node as Read, Tree as ConsumerTree, TreeChangeHandler};

    struct Ignore;
    impl TreeChangeHandler for Ignore {
        fn node_added(&mut self, _: &Read) {}
        fn node_updated(&mut self, _: &Read, _: &Read) {}
        fn focus_moved(&mut self, _: Option<&Read>, _: Option<&Read>) {}
        fn node_removed(&mut self, _: &Read) {}
    }
    fn walk(node: Read<'_>, out: &mut Vec<String>) {
        if let Some(label) = node.label() {
            out.push(label);
        }
        for child in node.children() {
            walk(child, out);
        }
    }

    let mut host = Host::new(options);
    let mut grafts = Grafts::new();
    let composition = host.composition();
    let mut tree = ConsumerTree::new(grafts.activate(&composition), true);
    let mut push = |tree: &mut ConsumerTree, host: &mut Host| {
        for update in grafts.frame(host.composition()) {
            tree.update_and_process_changes(update, &mut Ignore);
        }
    };
    push(&mut tree, &mut host);
    let mut order = Vec::new();
    walk(tree.state().root(), &mut order);
    println!("reading order: {order:?}");

    let tab = KeyEvent::new(Key::Named(NamedKey::Tab));
    for _ in 0..3 {
        host.key(tab.clone());
        push(&mut tree, &mut host);
        let focus = tree.state().focus().and_then(|node| node.label());
        println!("Tab -> {focus:?}");
    }
    let button = tree
        .state()
        .root()
        .children()
        .flat_map(|pane| pane.children().collect::<Vec<_>>())
        .find_map(|guest_root| {
            fn find<'a>(node: Read<'a>) -> Option<Read<'a>> {
                if node
                    .label()
                    .is_some_and(|label| label.starts_with("B count"))
                {
                    return Some(node);
                }
                node.children().find_map(find)
            }
            find(guest_root)
        })
        .map(|button| button.locate());
    if let Some((target_node, target_tree)) = button {
        host.act(&ActionRequest {
            action: Action::Click,
            target_tree,
            target_node,
            data: None,
        });
        push(&mut tree, &mut host);
    }
    println!("{}", host.title());
    if let Some(memory) = resident_memory() {
        println!("{} guest(s), {memory}", options.guests);
    }
}

fn main() {
    let options = Options::parse();
    if options.self_check {
        self_check(options);
        return;
    }
    let event_loop = EventLoop::<()>::with_user_event()
        .build()
        .expect("an event loop");
    let mut app = App {
        host: Host::new(options),
        grafts: Grafts::new(),
        shared: Shared {
            initial: Arc::new(Mutex::new(None)),
            activated: Arc::new(AtomicBool::new(false)),
            actions: Arc::new(Mutex::new(VecDeque::new())),
        },
        proxy: event_loop.create_proxy(),
        window: None,
        adapter: None,
        modifiers: ModifiersState::default(),
    };
    event_loop.run_app(&mut app).expect("the probe runs");
}
