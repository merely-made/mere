// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Structural scope and temporal history are independent. Hosts resolve actual
//! graph membership, reading anchors and destinations before accepting a visit.
use crate::{
    FocusPhase, HoverPhase, OptionalAction, ShellSlot, clickable, el, on_focus, on_hover,
    shell_control,
};
use serde::{Deserialize, Serialize};
use std::rc::Rc;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScopeLevel {
    Document,
    Neighborhood,
    #[default]
    Graph,
    Overmap,
}
impl ScopeLevel {
    pub fn label(self) -> &'static str {
        match self {
            Self::Document => "Document",
            Self::Neighborhood => "Neighborhood",
            Self::Graph => "Graph",
            Self::Overmap => "Meres",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScopePlace<G, M> {
    pub level: ScopeLevel,
    pub graph: G,
    /// The root of the chosen neighborhood, independent of its selected child.
    pub root: Option<M>,
    pub focus: Option<M>,
}
impl<G: Clone + PartialEq, M: Clone + PartialEq> ScopePlace<G, M> {
    pub fn up(&self) -> Option<Self> {
        let level = match self.level {
            ScopeLevel::Document => ScopeLevel::Neighborhood,
            ScopeLevel::Neighborhood => ScopeLevel::Graph,
            ScopeLevel::Graph => ScopeLevel::Overmap,
            ScopeLevel::Overmap => return None,
        };
        Some(Self {
            level,
            ..self.clone()
        })
    }
    pub fn down(&self) -> Option<Self> {
        let level = match self.level {
            ScopeLevel::Overmap => ScopeLevel::Graph,
            ScopeLevel::Graph if self.root.is_some() || self.focus.is_some() => {
                ScopeLevel::Neighborhood
            },
            ScopeLevel::Neighborhood if self.focus.is_some() => ScopeLevel::Document,
            _ => return None,
        };
        let root = self.root.clone().or_else(|| self.focus.clone());
        Some(Self {
            level,
            root,
            ..self.clone()
        })
    }
    /// A deliberate new graph selection names a new neighborhood to enter.
    pub fn select(&mut self, focus: M) {
        if self.focus.as_ref() != Some(&focus) && self.level == ScopeLevel::Graph {
            self.root = Some(focus.clone());
        }
        self.focus = Some(focus);
    }
    pub fn same_scope(&self, other: &Self) -> bool {
        self.level == other.level
            && self.graph == other.graph
            && match self.level {
                ScopeLevel::Document => self.focus == other.focus,
                ScopeLevel::Neighborhood => self.root == other.root,
                _ => true,
            }
    }
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
pub struct ScopeVisit<G, M, A> {
    pub place: ScopePlace<G, M>,
    pub camera: Option<ScopeCamera>,
    pub anchor: Option<A>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ScopeNavigation<G, M, A> {
    entries: Vec<ScopeVisit<G, M, A>>,
    cursor: usize,
    remembered: Vec<ScopeVisit<G, M, A>>,
}
impl<G, M, A> Default for ScopeNavigation<G, M, A> {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            cursor: 0,
            remembered: Vec::new(),
        }
    }
}
impl<G: Clone + PartialEq, M: Clone + PartialEq, A: Clone> ScopeNavigation<G, M, A> {
    pub fn current(&self) -> Option<&ScopeVisit<G, M, A>> {
        self.entries.get(self.cursor)
    }
    pub fn visits(&self) -> impl Iterator<Item = &ScopeVisit<G, M, A>> {
        self.entries.iter().chain(&self.remembered)
    }
    pub fn destination(&self, forward: bool) -> Option<&ScopeVisit<G, M, A>> {
        let index = if forward {
            self.cursor.checked_add(1)?
        } else {
            self.cursor.checked_sub(1)?
        };
        self.entries.get(index)
    }
    pub fn remember(&mut self, visit: ScopeVisit<G, M, A>) {
        self.remembered
            .retain(|v| !v.place.same_scope(&visit.place));
        self.remembered.push(visit);
        if self.remembered.len() > 64 {
            self.remembered.remove(0);
        }
    }
    pub fn saved(&self, place: &ScopePlace<G, M>) -> Option<&ScopeVisit<G, M, A>> {
        self.remembered
            .iter()
            .rev()
            .find(|v| v.place.same_scope(place))
    }
    pub fn capture(&mut self, visit: ScopeVisit<G, M, A>) {
        self.remember(visit.clone());
        if self.entries.is_empty() {
            self.entries.push(visit);
            self.cursor = 0;
        } else {
            self.entries[self.cursor] = visit;
        }
    }
    pub fn visit(&mut self, visit: ScopeVisit<G, M, A>) {
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
    pub fn validate(&self, valid_anchor: impl Fn(&A) -> bool) -> Result<(), String> {
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
                || (matches!(
                    v.place.level,
                    ScopeLevel::Document | ScopeLevel::Neighborhood
                ) && v.place.focus.is_none())
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
pub enum ScopeCommand {
    Navigate(ScopeDirection),
    To(ScopeLevel),
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
pub fn scope_controls<S: 'static, A: 'static, O: OptionalAction<A> + 'static>(
    controls: ScopeControls,
    handler: impl Fn(&mut S, ScopeCommand) -> O + 'static,
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
pub struct ScopeCrumb {
    pub label: String,
    pub level: ScopeLevel,
    pub current: bool,
}
pub fn scope_trail<S: 'static, A: 'static, O: OptionalAction<A> + 'static>(
    crumbs: Vec<ScopeCrumb>,
    handler: impl Fn(&mut S, ScopeCommand) -> O + 'static,
) -> ShellSlot<S, A> {
    let handler = Rc::new(handler);
    let mut children: Vec<ShellSlot<S, A>> = Vec::new();
    for (index, crumb) in crumbs.into_iter().enumerate() {
        if index > 0 {
            children.push(Box::new(el("span", "›").attr("aria-hidden", "true")));
        }
        if crumb.current {
            children.push(Box::new(
                el("span", crumb.label).attr("aria-current", "location"),
            ));
        } else {
            let click = handler.clone();
            children.push(shell_control(
                crumb.label.clone(),
                format!("Scope: {}", crumb.label),
                Default::default(),
                move |s, _| click(s, ScopeCommand::To(crumb.level)),
            ));
        }
    }
    Box::new(
        el("nav", children)
            .attr("class", "cambium-scope-trail")
            .attr("aria-label", "Scope trail"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    fn visit(level: ScopeLevel, focus: u8) -> ScopeVisit<u8, u8, u8> {
        ScopeVisit {
            place: ScopePlace {
                level,
                graph: 1,
                root: Some(2),
                focus: Some(focus),
            },
            camera: None,
            anchor: Some(9),
        }
    }
    #[test]
    fn down_enters_focus_and_history_can_go_outward() {
        let mut nav = ScopeNavigation::default();
        nav.visit(visit(ScopeLevel::Document, 2));
        nav.visit(visit(ScopeLevel::Neighborhood, 2));
        nav.visit(visit(ScopeLevel::Graph, 2));
        let mut place = nav.current().unwrap().place.clone();
        place.select(3);
        let neighborhood = place.down().unwrap();
        assert_eq!(neighborhood.root, Some(3));
        nav.visit(ScopeVisit {
            place: neighborhood,
            ..visit(ScopeLevel::Neighborhood, 3)
        });
        assert_eq!(
            nav.destination(false).unwrap().place.level,
            ScopeLevel::Graph
        );
        assert_eq!(
            nav.current().unwrap().place.down().unwrap().level,
            ScopeLevel::Document
        );
        assert_eq!(nav.current().unwrap().place.down().unwrap().focus, Some(3));
        assert_eq!(nav.current().unwrap().anchor, Some(9));
    }
    #[test]
    fn route_keeps_context_when_reading_another_member() {
        let place = visit(ScopeLevel::Neighborhood, 3).place.down().unwrap();
        assert_eq!(place.up().unwrap().root, Some(2));
        assert!(place.down().is_none());
        assert!(visit(ScopeLevel::Overmap, 2).place.up().is_none());
    }
    #[test]
    fn new_visit_cuts_forward_history_but_keeps_scope_camera() {
        let mut nav = ScopeNavigation::default();
        let mut graph = visit(ScopeLevel::Graph, 2);
        graph.camera = Some(ScopeCamera {
            x: 120.0,
            y: 10.0,
            zoom: 2.0,
        });
        nav.visit(graph.clone());
        nav.visit(visit(ScopeLevel::Neighborhood, 2));
        assert!(nav.step(false));
        nav.visit(visit(ScopeLevel::Document, 3));
        assert!(nav.destination(true).is_none());
        assert_eq!(nav.saved(&graph.place).unwrap().camera, graph.camera);
        assert!(nav.validate(|_| true).is_ok());
    }
    #[test]
    fn compass_keeps_keyboard_focus_and_exposes_destination_hints() {
        use crate::{DomHandle, GenetAppRunner, Key, KeyEvent, NamedKey, PointerClick};
        use genet_scripted_dom::ScriptedDom;
        use layout_dom_api::{LayoutDom, LocalName, Namespace};
        use std::cell::RefCell;
        #[derive(Default)]
        struct State {
            hint: Option<ScopeDirection>,
            commands: Vec<ScopeDirection>,
        }
        let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
        let mut runner = GenetAppRunner::<_, _, _, ()>::new(
            dom.clone(),
            |s: &State| {
                scope_controls(
                    ScopeControls {
                        up: Some("Neighborhood of notes".into()),
                        hint: s.hint,
                        ..Default::default()
                    },
                    |s: &mut State, c| match c {
                        ScopeCommand::Hint(h) => s.hint = h,
                        ScopeCommand::Navigate(d) => s.commands.push(d),
                        _ => {},
                    },
                )
            },
            State::default(),
        );
        let buttons = dom.borrow().all_with_class(runner.root(), "shell-control");
        let name = |node| {
            dom.borrow()
                .attribute(node, &Namespace::from(""), &LocalName::from("aria-label"))
                .map(|s| s.to_string())
        };
        let up = buttons
            .iter()
            .copied()
            .find(|n| name(*n).as_deref() == Some("Up: Neighborhood of notes"))
            .unwrap();
        let down = buttons
            .iter()
            .copied()
            .find(|n| name(*n).as_deref() == Some("Down"))
            .unwrap();
        runner.set_focus(Some(up));
        assert_eq!(runner.state().hint, Some(ScopeDirection::Up));
        assert_eq!(
            dom.borrow()
                .all_with_class(runner.root(), "scope-tooltip")
                .len(),
            1
        );
        runner.dispatch_key(KeyEvent::new(Key::Named(NamedKey::Enter)));
        assert_eq!(runner.state().commands, vec![ScopeDirection::Up]);
        assert_eq!(runner.focus(), Some(up));
        runner.dispatch_click(down, PointerClick::at((0.0, 0.0)));
        assert_eq!(runner.state().commands.len(), 1);
    }
}
