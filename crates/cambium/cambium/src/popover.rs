/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! An anchored popover: a button that opens a panel of content beside itself.
//!
//! Consumer-pull (Knot, 2026-09-24): its command row opens a path field for
//! Open and for Save As, and its status bar's chips open their detail; both
//! are the same interaction, so both are this. The status bar draws its chips
//! through it.
//!
//! Whether it is open is the caller's state ([`PopoverState`]), like the
//! detail popover's mode: the popover renders it and reports [`PopoverEvent`]s.
//! The trigger is a button with `aria-haspopup` and `aria-expanded`; the panel
//! is a dialog anchored to it by the sheet, so nothing is measured and the
//! popover works wherever its trigger sits. Escape and a click outside close
//! it and return focus to the trigger. [`POPOVER_CSS`] is the structural
//! minimum; colour is the host's.

use meristem::AnyView;

use crate::{
    GenetCtx, GenetElement, Key, NamedKey, OverlayDismiss, PointerClick, button, el, on_click,
    on_key, request_focus,
};

/// The erased view a popover and its content are made of.
pub type PopoverView<State, Action> = Box<dyn AnyView<State, Action, GenetCtx, GenetElement>>;

/// The structural sheet: the panel sits beside its trigger as its placement
/// says, and the outside layer covers the window beneath it.
pub const POPOVER_CSS: &str = "\
    .popover-anchor { position: relative; display: flex; } \
    .popover-dismiss { position: fixed; left: 0px; top: 0px; right: 0px; bottom: 0px; z-index: 50; } \
    .popover { position: absolute; z-index: 51; } \
    .popover[data-placement=below-start] { left: 0px; top: 100%; } \
    .popover[data-placement=below-end] { right: 0px; top: 100%; } \
    .popover[data-placement=above-start] { left: 0px; bottom: 100%; } \
    .popover[data-placement=above-end] { right: 0px; bottom: 100%; }";

/// Where the panel sits against its trigger: below or above it, lined up with
/// its start or its end.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PopoverPlacement {
    #[default]
    BelowStart,
    BelowEnd,
    AboveStart,
    AboveEnd,
}

impl PopoverPlacement {
    /// The `data-placement` token the sheet addresses.
    pub fn token(self) -> &'static str {
        match self {
            Self::BelowStart => "below-start",
            Self::BelowEnd => "below-end",
            Self::AboveStart => "above-start",
            Self::AboveEnd => "above-end",
        }
    }
}

/// Whether the popover is open, and whether its trigger takes focus back once
/// it closes. The caller stores it and applies events with [`Self::apply`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PopoverState {
    pub open: bool,
    /// One-shot: consumed by the trigger's focus request after a close.
    pub return_focus: bool,
}

/// What a popover reports.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PopoverEvent {
    /// The trigger was activated: open the popover, or close it if it is open.
    Toggle,
    /// The open popover was dismissed.
    Dismiss(OverlayDismiss),
}

impl PopoverState {
    pub fn apply(&mut self, event: PopoverEvent) {
        match event {
            PopoverEvent::Toggle => {
                self.return_focus = self.open;
                self.open = !self.open;
            },
            PopoverEvent::Dismiss(_) => self.close(),
        }
    }

    /// Close it from the caller's side, once its action has run, with focus
    /// back on the trigger.
    pub fn close(&mut self) {
        self.return_focus = self.open;
        self.open = false;
    }
}

/// What a popover shows: its trigger's label (also the dialog's name), where
/// the panel sits, whether it is open, and attributes the host puts on the
/// trigger to style or name it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Popover<'a> {
    pub label: &'a str,
    pub placement: PopoverPlacement,
    pub open: bool,
    /// One-shot: the trigger asks for focus, after a close.
    pub return_focus: bool,
    pub trigger_attrs: Vec<(&'static str, String)>,
}

impl<'a> Popover<'a> {
    /// A popover labelled `label`, open or not as `state` says.
    pub fn new(label: &'a str, state: &PopoverState) -> Self {
        Self {
            label,
            placement: PopoverPlacement::default(),
            open: state.open,
            return_focus: state.return_focus,
            trigger_attrs: Vec::new(),
        }
    }

    #[must_use]
    pub fn with_placement(mut self, placement: PopoverPlacement) -> Self {
        self.placement = placement;
        self
    }

    /// Put `name="value"` on the trigger; a later value for the same name
    /// replaces the earlier one.
    #[must_use]
    pub fn with_trigger_attr(mut self, name: &'static str, value: impl Into<String>) -> Self {
        self.trigger_attrs.push((name, value.into()));
        self
    }
}

/// Render a controlled popover.
///
/// `body` is the panel's content, asked for only while the popover is open;
/// a `None` body leaves the popover closed however its state reads, so a host
/// can drop the content without first closing it.
pub fn popover<State, Action, Change, Body>(
    popover: Popover<'_>,
    on_change: Change,
    body: Body,
) -> PopoverView<State, Action>
where
    State: 'static,
    Action: 'static,
    Change: Fn(&mut State, PopoverEvent) + Clone + 'static,
    Body: FnOnce() -> Option<PopoverView<State, Action>>,
{
    let body = if popover.open { body() } else { None };
    let open = body.is_some();

    let toggle = on_change.clone();
    let mut trigger = button(
        popover.label.to_string(),
        move |app: &mut State, _: PointerClick| toggle(app, PopoverEvent::Toggle),
    )
    .attr("aria-haspopup", "dialog")
    .attr("aria-expanded", if open { "true" } else { "false" });
    for (name, value) in popover.trigger_attrs {
        trigger = trigger.attr(name, value);
    }
    let trigger = request_focus(trigger, popover.return_focus);

    let panel = body.map(|body| {
        let outside = on_change.clone();
        (
            on_click(
                el::<_, State, Action>("div", ())
                    .attr("class", "popover-dismiss")
                    .attr("aria-hidden", "true"),
                move |app: &mut State, _: PointerClick| {
                    outside(app, PopoverEvent::Dismiss(OverlayDismiss::OutsideClick));
                },
            ),
            el::<_, State, Action>("div", body)
                .attr("class", "popover")
                .attr("role", "dialog")
                .attr("aria-label", popover.label.to_string())
                .attr("data-placement", popover.placement.token()),
        )
    });

    // A passive listener on the anchor sees Escape from the trigger or from
    // anything focused in the panel, without adding a Tab stop.
    let escape = on_change;
    Box::new(
        on_key(
            el::<_, State, Action>("div", (trigger, panel)).attr("class", "popover-anchor"),
            move |app: &mut State, event| {
                if open && matches!(event.key, Key::Named(NamedKey::Escape)) {
                    event.prevent_default();
                    escape(app, PopoverEvent::Dismiss(OverlayDismiss::Escape));
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
        popover: PopoverState,
        confirmed: usize,
    }

    fn view(state: &State) -> PopoverView<State, ()> {
        popover(
            Popover::new("Open", &state.popover)
                .with_placement(PopoverPlacement::BelowStart)
                .with_trigger_attr("id", "open"),
            |app: &mut State, event| app.popover.apply(event),
            || {
                Some(Box::new(
                    button("Confirm", |app: &mut State, _| {
                        app.confirmed += 1;
                        app.popover.close();
                    })
                    .attr("id", "confirm"),
                ) as PopoverView<State, ()>)
            },
        )
    }

    type TestRunner =
        GenetAppRunner<State, fn(&State) -> PopoverView<State, ()>, PopoverView<State, ()>, ()>;

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
            view as fn(&State) -> PopoverView<State, ()>,
            State::default(),
        );
        (dom, runner)
    }

    #[test]
    fn a_closed_popover_is_a_trigger_that_announces_its_dialog() {
        let (dom, runner) = runner();
        let dom = dom.borrow();
        let root = runner.root();
        let trigger = find(&dom, root, "id", "open").expect("trigger");
        assert_eq!(attr(&dom, trigger, "aria-haspopup"), Some("dialog"));
        assert_eq!(attr(&dom, trigger, "aria-expanded"), Some("false"));
        assert!(find(&dom, root, "role", "dialog").is_none());
    }

    #[test]
    fn the_trigger_opens_the_panel_where_its_placement_says() {
        let (dom, mut runner) = runner();
        let root = runner.root();
        let trigger = find(&dom.borrow(), root, "id", "open").expect("trigger");
        runner.dispatch_click(trigger, PointerClick::at((1.0, 1.0)));
        assert!(runner.state().popover.open);
        let dom_ref = dom.borrow();
        let panel = find(&dom_ref, root, "role", "dialog").expect("panel");
        assert_eq!(attr(&dom_ref, panel, "aria-label"), Some("Open"));
        assert_eq!(attr(&dom_ref, panel, "data-placement"), Some("below-start"));
        let trigger = find(&dom_ref, root, "id", "open").expect("trigger");
        assert_eq!(attr(&dom_ref, trigger, "aria-expanded"), Some("true"));
    }

    #[test]
    fn escape_an_outside_click_and_the_action_close_it_and_return_focus() {
        let (dom, mut runner) = runner();
        let root = runner.root();
        let trigger = find(&dom.borrow(), root, "id", "open").expect("trigger");

        runner.dispatch_click(trigger, PointerClick::at((1.0, 1.0)));
        let confirm = find(&dom.borrow(), root, "id", "confirm").expect("inside");
        runner.set_focus(Some(confirm));
        runner.dispatch_key(KeyEvent::new(Key::Named(NamedKey::Escape)));
        assert!(!runner.state().popover.open);
        let trigger = find(&dom.borrow(), root, "id", "open").expect("trigger");
        assert_eq!(runner.focus(), Some(trigger));

        runner.dispatch_click(trigger, PointerClick::at((1.0, 1.0)));
        let outside = find(&dom.borrow(), root, "class", "popover-dismiss").expect("layer");
        runner.dispatch_click(outside, PointerClick::at((1.0, 1.0)));
        assert!(!runner.state().popover.open);
        let trigger = find(&dom.borrow(), root, "id", "open").expect("trigger");
        assert_eq!(runner.focus(), Some(trigger));

        runner.dispatch_click(trigger, PointerClick::at((1.0, 1.0)));
        let confirm = find(&dom.borrow(), root, "id", "confirm").expect("inside");
        runner.dispatch_click(confirm, PointerClick::at((1.0, 1.0)));
        assert_eq!(runner.state().confirmed, 1);
        assert!(!runner.state().popover.open);
        let trigger = find(&dom.borrow(), root, "id", "open").expect("trigger");
        assert_eq!(runner.focus(), Some(trigger));
    }

    #[test]
    fn the_trigger_toggles_and_only_a_close_returns_focus() {
        let mut state = PopoverState::default();
        state.apply(PopoverEvent::Toggle);
        assert_eq!(
            state,
            PopoverState {
                open: true,
                return_focus: false
            }
        );
        state.apply(PopoverEvent::Toggle);
        assert_eq!(
            state,
            PopoverState {
                open: false,
                return_focus: true
            }
        );
        state.close();
        assert!(
            !state.return_focus,
            "closing a closed popover returns nothing"
        );
    }
}
