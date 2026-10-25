// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Structural scope and temporal history are independent. Hosts resolve actual
//! identities, contextual routes, reading anchors and destinations before accepting
//! a visit. Controls and history do not prescribe an application taxonomy.
use crate::{
    FocusPhase, HoverPhase, OptionalAction, ShellSlot, clickable, el, on_focus, on_hover,
    shell_control,
};
use serde::{Deserialize, Serialize};
use std::rc::Rc;

/// Release-based activation shared by graph hosts. Drags, modified gestures,
/// different members and a third click cannot accidentally open a reading.
#[derive(Clone, Debug)]
pub struct NodeActivation<M> {
    press: Option<(M, (f32, f32))>,
    last: Option<(M, (f32, f32), f64)>,
}
impl<M> Default for NodeActivation<M> {
    fn default() -> Self {
        Self {
            press: None,
            last: None,
        }
    }
}
impl<M: Clone + PartialEq> NodeActivation<M> {
    pub fn cancel(&mut self) {
        self.press = None;
        self.last = None;
    }
    pub fn press(&mut self, member: Option<M>, at: (f32, f32), modified: bool) {
        self.press = if modified {
            None
        } else {
            member.map(|m| (m, at))
        };
        if self.press.is_none() {
            self.last = None;
        }
    }
    pub fn moved(&mut self, at: (f32, f32)) {
        if self
            .press
            .as_ref()
            .is_some_and(|(_, p)| (p.0 - at.0).abs() > 4.0 || (p.1 - at.1).abs() > 4.0)
        {
            self.cancel();
        }
    }
    pub fn release(&mut self, member: Option<M>, at: (f32, f32), now_ms: f64) -> Option<M> {
        let Some((pressed, start)) = self.press.take() else {
            self.last = None;
            return None;
        };
        let near =
            |a: (f32, f32), b: (f32, f32)| (a.0 - b.0).abs() <= 4.0 && (a.1 - b.1).abs() <= 4.0;
        if member.as_ref() != Some(&pressed) || !near(start, at) || !now_ms.is_finite() {
            self.last = None;
            return None;
        }
        let doubled = self.last.as_ref().is_some_and(|(m, p, time)| {
            *m == pressed && near(*p, at) && (0.0..=500.0).contains(&(now_ms - time))
        });
        self.last = if doubled {
            None
        } else {
            Some((pressed.clone(), at, now_ms))
        };
        doubled.then_some(pressed)
    }
}

/// Closing a background reading keeps focus. Closing the current one chooses
/// its next neighbor, falling back to the previous reading at the end.
pub fn reading_after_close<M: Clone + PartialEq>(
    members: &[M],
    focus: Option<&M>,
    closing: &M,
) -> Option<M> {
    if let Some(focus) = focus.filter(|m| *m != closing && members.contains(m)) {
        return Some(focus.clone());
    }
    let index = members.iter().position(|m| m == closing)?;
    members
        .get(index + 1)
        .or_else(|| index.checked_sub(1).and_then(|i| members.get(i)))
        .cloned()
}

/// A close chooses another tab in its own tiled stack before crossing cells.
pub fn reading_after_close_in_stack<M: Clone + PartialEq>(
    members: &[M],
    stack: &[M],
    focus: Option<&M>,
    closing: &M,
) -> Option<M> {
    if let Some(focus) = focus.filter(|m| *m != closing && members.contains(m)) {
        return Some(focus.clone());
    }
    reading_after_close(stack, focus, closing)
        .or_else(|| reading_after_close(members, focus, closing))
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ScopeCamera {
    pub x: f32,
    pub y: f32,
    pub zoom: f32,
}
impl ScopeCamera {
    pub fn valid(self) -> bool {
        self.x.is_finite() && self.y.is_finite() && self.zoom.is_finite() && self.zoom > 0.0
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ScopeVisit<P, A> {
    pub place: P,
    pub camera: Option<ScopeCamera>,
    pub anchor: Option<A>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ScopeNavigation<P, A> {
    entries: Vec<ScopeVisit<P, A>>,
    cursor: usize,
    remembered: Vec<ScopeVisit<P, A>>,
}
impl<P, A> Default for ScopeNavigation<P, A> {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            cursor: 0,
            remembered: Vec::new(),
        }
    }
}
impl<P: Clone + PartialEq, A: Clone> ScopeNavigation<P, A> {
    pub fn current(&self) -> Option<&ScopeVisit<P, A>> {
        self.entries.get(self.cursor)
    }
    pub fn visits(&self) -> impl Iterator<Item = &ScopeVisit<P, A>> {
        self.entries.iter().chain(&self.remembered)
    }
    pub fn destination(&self, forward: bool) -> Option<&ScopeVisit<P, A>> {
        let index = if forward {
            self.cursor.checked_add(1)?
        } else {
            self.cursor.checked_sub(1)?
        };
        self.entries.get(index)
    }
    pub fn remember(&mut self, visit: ScopeVisit<P, A>) {
        self.remembered.retain(|v| v.place != visit.place);
        self.remembered.push(visit);
        if self.remembered.len() > 64 {
            self.remembered.remove(0);
        }
    }
    /// The host defines restoration identity. A context can share a document's
    /// durable anchor while retaining a different camera or contextual parent.
    pub fn saved_matching(&self, same_scope: impl Fn(&P) -> bool) -> Option<&ScopeVisit<P, A>> {
        self.remembered.iter().rev().find(|v| same_scope(&v.place))
    }
    pub fn capture(&mut self, visit: ScopeVisit<P, A>) {
        self.remember(visit.clone());
        if self.entries.is_empty() {
            self.entries.push(visit);
            self.cursor = 0;
        } else {
            self.entries[self.cursor] = visit;
        }
    }
    pub fn visit(&mut self, visit: ScopeVisit<P, A>) {
        if self.current().is_some_and(|v| v.place == visit.place) {
            self.capture(visit);
            return;
        }
        self.entries.truncate(self.cursor + 1);
        self.entries.push(visit.clone());
        if self.entries.len() > 128 {
            self.entries.remove(0);
        }
        self.cursor = self.entries.len() - 1;
        self.remember(visit);
    }
    /// Call only after the host admits and restores destination().
    pub fn step(&mut self, forward: bool) -> bool {
        if self.destination(forward).is_none() {
            return false;
        }
        if forward {
            self.cursor += 1;
        } else {
            self.cursor -= 1;
        }
        true
    }
    pub fn validate(
        &self,
        valid_place: impl Fn(&P) -> bool,
        valid_anchor: impl Fn(&A) -> bool,
    ) -> Result<(), String> {
        if self.entries.len() > 128
            || self.remembered.len() > 64
            || (self.entries.is_empty() && self.cursor != 0)
            || (!self.entries.is_empty() && self.cursor >= self.entries.len())
        {
            return Err("Invalid scope navigation history".into());
        }
        if self.entries.iter().chain(&self.remembered).any(|v| {
            v.camera.is_some_and(|c| !c.valid())
                || v.anchor.as_ref().is_some_and(|a| !valid_anchor(a))
                || !valid_place(&v.place)
        }) {
            return Err("Invalid scope navigation visit".into());
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScopeDirection {
    Back,
    Forward,
    Up,
    Down,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScopeCommand<T = ()> {
    Navigate(ScopeDirection),
    To(T),
    /// Edit the current graph location without recording a visit until accepted.
    EditLocation,
    Hint(Option<ScopeDirection>),
}
#[derive(Clone, Debug, Default)]
pub struct ScopeControls {
    pub back: Option<String>,
    pub forward: Option<String>,
    pub up: Option<String>,
    pub down: Option<String>,
    pub hint: Option<ScopeDirection>,
}
pub fn scope_controls<S: 'static, A: 'static, T: 'static, O: OptionalAction<A> + 'static>(
    controls: ScopeControls,
    handler: impl Fn(&mut S, ScopeCommand<T>) -> O + 'static,
) -> ShellSlot<S, A> {
    let handler = Rc::new(handler);
    let mut children = Vec::new();
    for (label, direction, destination) in [
        ("Back", ScopeDirection::Back, controls.back),
        ("Forward", ScopeDirection::Forward, controls.forward),
        ("Up", ScopeDirection::Up, controls.up),
        ("Down", ScopeDirection::Down, controls.down),
    ] {
        let name = destination
            .as_ref()
            .map(|d| format!("{label}: {d}"))
            .unwrap_or_else(|| label.into());
        let click = handler.clone();
        let hover = handler.clone();
        let view = el(
            "button",
            el("span", ())
                .attr("class", "scope-arrow")
                .attr("aria-hidden", "true"),
        )
        .attr("class", "shell-control")
        .attr("aria-label", name.clone())
        .attr(
            "aria-disabled",
            if destination.is_none() {
                "true"
            } else {
                "false"
            },
        );
        let focus = handler.clone();
        let button: ShellSlot<S, A> = if destination.is_none() {
            Box::new(view.attr("disabled", ""))
        } else {
            Box::new(on_focus(
                clickable(view, move |s, _| {
                    click(s, ScopeCommand::Navigate(direction))
                }),
                move |s: &mut S, event| {
                    focus(
                        s,
                        ScopeCommand::Hint(
                            (event.phase == FocusPhase::Gained).then_some(direction),
                        ),
                    )
                },
            ))
        };
        let mut content: Vec<ShellSlot<S, A>> = vec![button];
        if controls.hint == Some(direction) && destination.is_some() {
            content.push(Box::new(
                el("span", name)
                    .attr("class", "scope-tooltip")
                    .attr("role", "tooltip"),
            ));
        }
        children.push(Box::new(on_hover(
            el("span", content).attr(
                "class",
                format!("scope-control scope-{}", label.to_lowercase()),
            ),
            move |s: &mut S, event| {
                hover(
                    s,
                    ScopeCommand::Hint(match event.phase {
                        HoverPhase::Enter => Some(direction),
                        HoverPhase::Leave => None,
                        HoverPhase::Move => controls.hint,
                    }),
                )
            },
        )) as ShellSlot<S, A>);
    }
    Box::new(
        el("div", children)
            .attr("class", "cambium-scope-controls")
            .attr("role", "group")
            .attr("aria-label", "History and scope"),
    )
}

#[derive(Clone, Debug)]
pub struct ScopeCrumb<T = ()> {
    pub label: String,
    pub target: T,
    pub current: bool,
}
pub fn scope_trail<S: 'static, A: 'static, T: Clone + 'static, O: OptionalAction<A> + 'static>(
    crumbs: Vec<ScopeCrumb<T>>,
    handler: impl Fn(&mut S, ScopeCommand<T>) -> O + 'static,
) -> ShellSlot<S, A> {
    scope_trail_with_location_edit(crumbs, true, handler)
}

/// Hosts without a location editor keep the current crumb as plain text;
/// accepted ancestor destinations use the same contextual payloads.
pub fn scope_trail_with_location_edit<
    S: 'static,
    A: 'static,
    T: Clone + 'static,
    O: OptionalAction<A> + 'static,
>(
    crumbs: Vec<ScopeCrumb<T>>,
    editable: bool,
    handler: impl Fn(&mut S, ScopeCommand<T>) -> O + 'static,
) -> ShellSlot<S, A> {
    let handler = Rc::new(handler);
    let mut children: Vec<ShellSlot<S, A>> = Vec::new();
    for (index, crumb) in crumbs.into_iter().enumerate() {
        if index > 0 {
            children.push(Box::new(el("span", "›").attr("aria-hidden", "true")));
        }
        if crumb.current && !editable {
            children.push(Box::new(
                el("span", crumb.label)
                    .attr("class", "scope-location-current")
                    .attr("aria-current", "location"),
            ));
        } else if crumb.current {
            let click = handler.clone();
            children.push(Box::new(clickable(
                el("button", crumb.label.clone())
                    .attr("class", "shell-control scope-location-edit")
                    .attr("aria-current", "location")
                    .attr("aria-label", format!("Edit location: {}", crumb.label)),
                move |s, _| click(s, ScopeCommand::EditLocation),
            )));
        } else {
            let click = handler.clone();
            children.push(shell_control(
                crumb.label.clone(),
                format!("Scope: {}", crumb.label),
                Default::default(),
                move |s, _| click(s, ScopeCommand::To(crumb.target.clone())),
            ));
        }
    }
    Box::new(
        el("nav", children)
            .attr("class", "cambium-scope-trail")
            .attr("aria-label", "Scope trail"),
    )
}

/// One location slot: the quiet trail at rest, the host's existing editable
/// field while active. Text ownership and destination admission stay with hosts.
pub fn scope_location<
    S: 'static,
    A: 'static,
    T: Clone + 'static,
    O: OptionalAction<A> + 'static,
>(
    crumbs: Vec<ScopeCrumb<T>>,
    field: Option<ShellSlot<S, A>>,
    handler: impl Fn(&mut S, ScopeCommand<T>) -> O + 'static,
) -> ShellSlot<S, A> {
    Box::new(
        el("div", field.unwrap_or_else(|| scope_trail(crumbs, handler)))
            .attr("class", "cambium-scope-location"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn a_current_crumb_without_an_editor_has_no_inert_button_and_ancestors_still_navigate() {
        use crate::{DomHandle, GenetAppRunner, PointerClick};
        use genet_scripted_dom::ScriptedDom;
        use layout_dom_api::LayoutDom;
        use std::cell::RefCell;
        struct State {
            events: Vec<ScopeCommand<String>>,
        }
        fn view(_: &State) -> ShellSlot<State> {
            scope_trail_with_location_edit(
                vec![
                    ScopeCrumb {
                        label: "Set".into(),
                        target: "set-a".into(),
                        current: false,
                    },
                    ScopeCrumb {
                        label: "Measure".into(),
                        target: "measure-a".into(),
                        current: true,
                    },
                ],
                false,
                |state: &mut State, event| state.events.push(event),
            )
        }
        let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
        let mut runner = GenetAppRunner::new(
            dom.clone(),
            view as fn(&State) -> ShellSlot<State>,
            State { events: Vec::new() },
        );
        let root = runner.root();
        let buttons = dom.borrow().all_with_class(root, "shell-control");
        assert_eq!(buttons.len(), 1);
        assert_eq!(
            dom.borrow()
                .all_with_class(root, "scope-location-current")
                .len(),
            1
        );
        assert!(
            dom.borrow()
                .all_with_class(root, "scope-location-edit")
                .is_empty()
        );
        runner.dispatch_click(buttons[0], PointerClick::at((1., 1.)));
        assert_eq!(
            runner.state().events,
            vec![ScopeCommand::To("set-a".into())]
        );
    }
    #[test]
    fn double_activation_requires_two_unmodified_clicks_on_the_same_member() {
        let mut activation = NodeActivation::default();
        activation.press(Some(1), (4., 5.), false);
        assert_eq!(activation.release(Some(1), (4., 5.), 100.), None);
        activation.press(Some(1), (5., 5.), false);
        assert_eq!(activation.release(Some(1), (5., 5.), 300.), Some(1));
        activation.press(Some(1), (5., 5.), false);
        assert_eq!(
            activation.release(Some(1), (5., 5.), 350.),
            None,
            "a third click starts again"
        );
        activation.press(Some(2), (5., 5.), false);
        assert_eq!(activation.release(Some(2), (5., 5.), 400.), None);
        activation.press(Some(2), (5., 5.), true);
        assert_eq!(activation.release(Some(2), (5., 5.), 450.), None);
        activation.press(Some(2), (5., 5.), false);
        activation.moved((30., 5.));
        assert_eq!(
            activation.release(Some(2), (5., 5.), 500.),
            None,
            "dragging out and back is not a click"
        );
        activation.press(Some(2), (5., 5.), false);
        assert_eq!(activation.release(Some(2), (5., 5.), 600.), None);
        activation.press(Some(2), (5., 5.), false);
        assert_eq!(
            activation.release(Some(2), (5., 5.), 1200.),
            None,
            "slow clicks do not open"
        );
    }
    #[test]
    fn closing_readings_preserves_background_focus_and_selects_an_adjacent_view() {
        assert_eq!(reading_after_close(&[1, 2, 3], Some(&3), &1), Some(3));
        assert_eq!(reading_after_close(&[1, 2, 3], Some(&2), &2), Some(3));
        assert_eq!(reading_after_close(&[1, 2, 3], Some(&3), &3), Some(2));
        assert_eq!(reading_after_close(&[1], Some(&1), &1), None);
        assert_eq!(
            reading_after_close_in_stack(&[1, 2, 3], &[1, 2], Some(&2), &2),
            Some(1)
        );
        assert_eq!(
            reading_after_close_in_stack(&[1, 2, 3], &[1, 2], Some(&3), &1),
            Some(3)
        );
        assert_eq!(
            reading_after_close_in_stack(&[1, 2, 3], &[3], Some(&3), &3),
            Some(2)
        );
    }
    #[derive(Clone, Debug, PartialEq)]
    struct Place {
        route: Vec<&'static str>,
        focus: u8,
    }
    fn visit(route: &[&'static str], focus: u8) -> ScopeVisit<Place, u8> {
        ScopeVisit {
            place: Place {
                route: route.to_vec(),
                focus,
            },
            camera: None,
            anchor: Some(9),
        }
    }
    #[test]
    fn hosts_supply_nested_skipped_and_ambiguous_contexts_without_a_fixed_ladder() {
        let mut nav = ScopeNavigation::default();
        nav.visit(visit(&["concert", "set", "section", "measure"], 2));
        nav.visit(visit(&["concert", "section", "measure"], 2));
        nav.visit(visit(&["practice", "measure"], 2));
        assert_eq!(
            nav.destination(false).unwrap().place.route,
            ["concert", "section", "measure"]
        );
        assert!(nav.step(false));
        assert_eq!(nav.current().unwrap().place.focus, 2);
        assert!(nav.destination(true).is_some());
        assert!(
            nav.saved_matching(|p| p.route == ["practice", "measure"])
                .is_some()
        );
        assert_eq!(
            nav.saved_matching(|p| p.route == ["concert", "set", "section", "measure"])
                .unwrap()
                .anchor,
            Some(9)
        );
        assert!(nav.validate(|p| !p.route.is_empty(), |_| true).is_ok());
    }
    #[test]
    fn capture_restores_a_visit_without_accepting_or_advancing_another_one() {
        let mut nav = ScopeNavigation::default();
        nav.visit(visit(&["world", "region"], 1));
        nav.visit(visit(&["world", "region", "room"], 2));
        nav.capture(ScopeVisit {
            camera: Some(ScopeCamera {
                x: 4.,
                y: 8.,
                zoom: 2.,
            }),
            ..visit(&["world", "region", "room"], 3)
        });
        assert_eq!(nav.destination(false).unwrap().place.focus, 1);
        assert!(nav.step(false));
        assert_eq!(nav.destination(true).unwrap().camera.unwrap().zoom, 2.);
        nav.visit(visit(&["world", "elsewhere"], 4));
        assert!(nav.destination(true).is_none());
    }
    #[test]
    fn validation_belongs_to_the_host_and_camera_sanity_to_the_history() {
        let mut nav = ScopeNavigation::default();
        nav.visit(visit(&[], 1));
        assert!(nav.validate(|p| !p.route.is_empty(), |_| true).is_err());
        nav.capture(ScopeVisit {
            camera: Some(ScopeCamera {
                x: 0.,
                y: 0.,
                zoom: f32::NAN,
            }),
            ..visit(&["root"], 1)
        });
        assert!(nav.validate(|_| true, |_| true).is_err());
    }
    #[test]
    fn activation_can_bind_to_a_live_instance_independently_of_durable_document_identity() {
        let mut activation = NodeActivation::default();
        activation.press(Some((10, "same-document")), (0., 0.), false);
        assert_eq!(
            activation.release(Some((10, "same-document")), (0., 0.), 10.),
            None
        );
        activation.press(Some((11, "same-document")), (0., 0.), false);
        assert_eq!(
            activation.release(Some((11, "same-document")), (0., 0.), 100.),
            None
        );
    }
}
