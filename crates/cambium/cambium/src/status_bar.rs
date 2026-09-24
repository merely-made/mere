/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! A status bar: one message and a row of chips, each able to open a popover
//! of detail and actions above itself.
//!
//! Consumer-pull (Knot, 2026-09-23): its design pass put every authority a
//! document carries — save state, catalog, retention, later sync and lock —
//! in one bottom row, each a chip that opens its own detail. Turnstone's
//! status lines are the predicted next consumer.
//!
//! Which chip is open is the caller's state ([`StatusBarState`]), like the
//! detail popover's mode: the bar renders it and reports [`StatusBarEvent`]s.
//! The interaction is the bar's. A chip is a button with `aria-expanded`; its
//! popover is a dialog anchored above the chip by the sheet, so nothing is
//! measured and the bar works wherever it is docked; Escape and a click outside
//! close it and return focus to the chip.
//!
//! Severity rides as `data-severity` (`quiet`, `warning`, `refused`) on the
//! message and on each chip, for the host's sheet to weight. The message is a
//! polite live region, so a refusal posted there is announced once.
//! [`STATUS_BAR_CSS`] is the structural minimum; colour is the host's.

use meristem::AnyView;

use crate::{
    GenetCtx, GenetElement, Key, NamedKey, OverlayDismiss, PointerClick, button, el, on_click,
    on_key, request_focus,
};

/// The erased view a status bar and its popover content are made of.
pub type StatusView<State, Action> = Box<dyn AnyView<State, Action, GenetCtx, GenetElement>>;

/// The structural sheet: the popover sits above its chip, the outside layer
/// covers the window beneath it. No colour; the host's sheet adds weight.
pub const STATUS_BAR_CSS: &str = "\
    .status-bar { display: flex; align-items: center; min-width: 0; } \
    .status-message { flex-grow: 1; min-width: 0; overflow: hidden; white-space: nowrap; text-overflow: ellipsis; } \
    .status-chips { display: flex; align-items: center; } \
    .status-chip-anchor { position: relative; } \
    .status-dismiss { position: fixed; left: 0px; top: 0px; right: 0px; bottom: 0px; z-index: 50; } \
    .status-popover { position: absolute; right: 0px; bottom: 100%; z-index: 51; }";

/// How much weight a message or chip carries.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum StatusSeverity {
    #[default]
    Quiet,
    Warning,
    Refused,
}

impl StatusSeverity {
    /// The `data-severity` token.
    pub fn token(self) -> &'static str {
        match self {
            Self::Quiet => "quiet",
            Self::Warning => "warning",
            Self::Refused => "refused",
        }
    }
}

/// One chip: a stable key, its visible label, its severity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatusChip {
    pub key: String,
    pub label: String,
    pub severity: StatusSeverity,
}

impl StatusChip {
    pub fn new(key: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            severity: StatusSeverity::Quiet,
        }
    }

    #[must_use]
    pub fn with_severity(mut self, severity: StatusSeverity) -> Self {
        self.severity = severity;
        self
    }
}

/// What the bar shows: the message, its severity, the chips, and the name the
/// bar's region is announced by.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StatusBar<'a> {
    pub message: &'a str,
    pub severity: StatusSeverity,
    pub chips: &'a [StatusChip],
    pub label: &'a str,
}

impl<'a> StatusBar<'a> {
    pub fn new(message: &'a str, chips: &'a [StatusChip]) -> Self {
        Self {
            message,
            severity: StatusSeverity::Quiet,
            chips,
            label: "Status",
        }
    }

    #[must_use]
    pub fn with_severity(mut self, severity: StatusSeverity) -> Self {
        self.severity = severity;
        self
    }

    #[must_use]
    pub fn with_label(mut self, label: &'a str) -> Self {
        self.label = label;
        self
    }
}

/// Which chip's popover is open, and which chip gets focus back once it
/// closes. The caller stores it and applies events with [`Self::apply`].
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StatusBarState {
    pub open: Option<String>,
    /// One-shot: consumed by the chip's focus request after a close.
    pub return_focus: Option<String>,
}

/// What a status bar reports.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StatusBarEvent {
    /// A chip was activated: open its popover, or close it if it is open.
    Toggle(String),
    /// The open popover was dismissed.
    Dismiss(OverlayDismiss),
}

impl StatusBarState {
    pub fn apply(&mut self, event: StatusBarEvent) {
        match event {
            StatusBarEvent::Toggle(key) => {
                if self.open.as_deref() == Some(key.as_str()) {
                    self.open = None;
                    self.return_focus = Some(key);
                } else {
                    self.open = Some(key);
                    self.return_focus = None;
                }
            },
            StatusBarEvent::Dismiss(_) => self.return_focus = self.open.take(),
        }
    }
}

/// Render a controlled status bar.
///
/// `content` is asked for the open chip's popover body, and only for it; a
/// chip whose `content` is `None` opens nothing. An open key that no chip
/// carries any longer opens nothing either, so a host can drop a chip without
/// first closing it.
pub fn status_bar<State, Action, Change, Content>(
    bar: StatusBar<'_>,
    state: &StatusBarState,
    on_change: Change,
    content: Content,
) -> StatusView<State, Action>
where
    State: 'static,
    Action: 'static,
    Change: Fn(&mut State, StatusBarEvent) + Clone + 'static,
    Content: Fn(&str) -> Option<StatusView<State, Action>>,
{
    let message = el::<_, State, Action>("span", bar.message.to_string())
        .attr("class", "status-message")
        .attr("role", "status")
        .attr("aria-live", "polite")
        .attr("data-severity", bar.severity.token());

    let chips: Vec<StatusView<State, Action>> = bar
        .chips
        .iter()
        .map(|chip| chip_view(chip, state, on_change.clone(), &content))
        .collect();

    Box::new(
        el::<_, State, Action>(
            "div",
            (
                message,
                el::<_, State, Action>("div", chips).attr("class", "status-chips"),
            ),
        )
        .attr("class", "status-bar")
        .attr("role", "region")
        .attr("aria-label", bar.label.to_string()),
    )
}

fn chip_view<State, Action, Change, Content>(
    chip: &StatusChip,
    state: &StatusBarState,
    on_change: Change,
    content: &Content,
) -> StatusView<State, Action>
where
    State: 'static,
    Action: 'static,
    Change: Fn(&mut State, StatusBarEvent) + Clone + 'static,
    Content: Fn(&str) -> Option<StatusView<State, Action>>,
{
    let body = (state.open.as_deref() == Some(chip.key.as_str()))
        .then(|| content(&chip.key))
        .flatten();
    let open = body.is_some();

    let toggle = on_change.clone();
    let key = chip.key.clone();
    let trigger = button(
        chip.label.clone(),
        move |app: &mut State, _: PointerClick| {
            toggle(app, StatusBarEvent::Toggle(key.clone()));
        },
    )
    .attr("class", "status-chip")
    .attr("data-status-key", chip.key.clone())
    .attr("data-severity", chip.severity.token())
    .attr("aria-haspopup", "dialog")
    .attr("aria-expanded", if open { "true" } else { "false" });
    let trigger = request_focus(
        trigger,
        state.return_focus.as_deref() == Some(chip.key.as_str()),
    );

    let popover = body.map(|body| {
        let outside = on_change.clone();
        (
            on_click(
                el::<_, State, Action>("div", ())
                    .attr("class", "status-dismiss")
                    .attr("aria-hidden", "true"),
                move |app: &mut State, _: PointerClick| {
                    outside(app, StatusBarEvent::Dismiss(OverlayDismiss::OutsideClick));
                },
            ),
            el::<_, State, Action>("div", body)
                .attr("class", "status-popover")
                .attr("role", "dialog")
                .attr("aria-label", chip.label.clone()),
        )
    });

    // A passive listener on the anchor sees Escape from the chip or from
    // anything focused in its popover, without adding a Tab stop.
    let escape = on_change;
    Box::new(
        on_key(
            el::<_, State, Action>("div", (trigger, popover)).attr("class", "status-chip-anchor"),
            move |app: &mut State, event| {
                if open && matches!(event.key, Key::Named(NamedKey::Escape)) {
                    event.prevent_default();
                    escape(app, StatusBarEvent::Dismiss(OverlayDismiss::Escape));
                }
            },
        )
        .focusable(false),
    )
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use genet_scripted_dom::{NodeId, ScriptedDom};
    use layout_dom_api::{LayoutDom, LocalName, Namespace};

    use super::*;
    use crate::{DomHandle, GenetAppRunner, KeyEvent};

    #[derive(Default)]
    struct State {
        bar: StatusBarState,
        retried: usize,
    }

    fn chips() -> Vec<StatusChip> {
        vec![
            StatusChip::new("format", "Djot"),
            StatusChip::new("catalog", "Not catalogued").with_severity(StatusSeverity::Warning),
        ]
    }

    fn view(state: &State) -> StatusView<State, ()> {
        let chips = chips();
        status_bar(
            StatusBar::new("Catalog lookup failed.", &chips).with_severity(StatusSeverity::Warning),
            &state.bar,
            |app: &mut State, event| app.bar.apply(event),
            |key| {
                (key == "catalog").then(|| {
                    Box::new(
                        button("Retry catalog", |app: &mut State, _| app.retried += 1)
                            .attr("id", "retry"),
                    ) as StatusView<State, ()>
                })
            },
        )
    }

    type TestRunner =
        GenetAppRunner<State, fn(&State) -> StatusView<State, ()>, StatusView<State, ()>, ()>;

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

    fn runner() -> (DomHandle, TestRunner) {
        let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
        let runner = GenetAppRunner::new(
            dom.clone(),
            view as fn(&State) -> StatusView<State, ()>,
            State::default(),
        );
        (dom, runner)
    }

    #[test]
    fn the_bar_is_a_labelled_region_with_a_polite_message_and_weighted_chips() {
        let (dom, runner) = runner();
        let dom = dom.borrow();
        let root = runner.root();
        assert_eq!(attr(&dom, root, "role"), Some("region"));
        assert_eq!(attr(&dom, root, "aria-label"), Some("Status"));
        let message = find(&dom, root, "class", "status-message").expect("message");
        assert_eq!(attr(&dom, message, "role"), Some("status"));
        assert_eq!(attr(&dom, message, "aria-live"), Some("polite"));
        assert_eq!(attr(&dom, message, "data-severity"), Some("warning"));
        let format = find(&dom, root, "data-status-key", "format").expect("format chip");
        assert_eq!(attr(&dom, format, "data-severity"), Some("quiet"));
        assert_eq!(attr(&dom, format, "aria-expanded"), Some("false"));
        let catalog = find(&dom, root, "data-status-key", "catalog").expect("catalog chip");
        assert_eq!(attr(&dom, catalog, "data-severity"), Some("warning"));
        assert_eq!(attr(&dom, catalog, "aria-haspopup"), Some("dialog"));
        assert!(
            find(&dom, root, "role", "dialog").is_none(),
            "nothing open yet"
        );
    }

    #[test]
    fn a_chip_opens_its_popover_and_the_popover_acts() {
        let (dom, mut runner) = runner();
        let root = runner.root();
        let catalog = find(&dom.borrow(), root, "data-status-key", "catalog").expect("chip");
        runner.dispatch_click(catalog, PointerClick::at((1.0, 1.0)));
        assert_eq!(runner.state().bar.open.as_deref(), Some("catalog"));
        let panel = find(&dom.borrow(), root, "role", "dialog").expect("popover");
        assert_eq!(
            attr(&dom.borrow(), panel, "aria-label"),
            Some("Not catalogued")
        );
        let catalog = find(&dom.borrow(), root, "data-status-key", "catalog").expect("chip");
        assert_eq!(attr(&dom.borrow(), catalog, "aria-expanded"), Some("true"));
        let retry = find(&dom.borrow(), root, "id", "retry").expect("the host's action");
        runner.dispatch_click(retry, PointerClick::at((1.0, 1.0)));
        assert_eq!(runner.state().retried, 1);
    }

    #[test]
    fn escape_and_an_outside_click_close_it_and_return_focus_to_the_chip() {
        let (dom, mut runner) = runner();
        let root = runner.root();
        let chip = find(&dom.borrow(), root, "data-status-key", "catalog").expect("chip");
        runner.dispatch_click(chip, PointerClick::at((1.0, 1.0)));
        let retry = find(&dom.borrow(), root, "id", "retry").expect("inside");
        runner.set_focus(Some(retry));
        runner.dispatch_key(KeyEvent::new(Key::Named(NamedKey::Escape)));
        assert_eq!(runner.state().bar.open, None);
        let chip = find(&dom.borrow(), root, "data-status-key", "catalog").expect("chip");
        assert_eq!(runner.focus(), Some(chip));

        runner.dispatch_click(chip, PointerClick::at((1.0, 1.0)));
        let outside = find(&dom.borrow(), root, "class", "status-dismiss").expect("outside layer");
        runner.dispatch_click(outside, PointerClick::at((1.0, 1.0)));
        assert_eq!(runner.state().bar.open, None);
        let chip = find(&dom.borrow(), root, "data-status-key", "catalog").expect("chip");
        assert_eq!(runner.focus(), Some(chip));
    }

    #[test]
    fn a_chip_without_content_opens_nothing() {
        let (dom, mut runner) = runner();
        let root = runner.root();
        let format = find(&dom.borrow(), root, "data-status-key", "format").expect("chip");
        runner.dispatch_click(format, PointerClick::at((1.0, 1.0)));
        assert_eq!(runner.state().bar.open.as_deref(), Some("format"));
        assert!(find(&dom.borrow(), root, "role", "dialog").is_none());
        let format = find(&dom.borrow(), root, "data-status-key", "format").expect("chip");
        assert_eq!(attr(&dom.borrow(), format, "aria-expanded"), Some("false"));
    }

    #[test]
    fn toggling_the_open_chip_closes_it() {
        let mut state = StatusBarState::default();
        state.apply(StatusBarEvent::Toggle("catalog".into()));
        assert_eq!(state.open.as_deref(), Some("catalog"));
        state.apply(StatusBarEvent::Toggle("catalog".into()));
        assert_eq!(state.open, None);
        assert_eq!(state.return_focus.as_deref(), Some("catalog"));
        state.apply(StatusBarEvent::Toggle("format".into()));
        assert_eq!(state.open.as_deref(), Some("format"));
        assert_eq!(state.return_focus, None);
    }
}
