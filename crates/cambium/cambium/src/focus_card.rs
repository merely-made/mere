// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A source-neutral focus card. Hosts resolve targets and previews, admit all
//! actions, and own navigation/history. Keeping a card changes only this deck.
use crate::{OptionalAction, ShellControl, ShellSlot, el, shell_control};
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq)]
pub struct FocusCardDeck<T> {
    pub peek: Option<T>,
    pub kept: Vec<T>,
}
impl<T> Default for FocusCardDeck<T> {
    fn default() -> Self {
        Self {
            peek: None,
            kept: Vec::new(),
        }
    }
}
impl<T: Clone + PartialEq> FocusCardDeck<T> {
    pub fn show(&mut self, target: T) {
        self.peek = (!self.kept.contains(&target)).then_some(target);
    }
    pub fn keep(&mut self, target: &T) {
        if self.peek.as_ref() != Some(target) {
            return;
        }
        if !self.kept.contains(target) {
            self.kept.push(target.clone());
        }
        self.peek = None;
    }
    pub fn close(&mut self, target: &T) {
        if self.peek.as_ref() == Some(target) {
            self.peek = None;
        }
        self.kept.retain(|t| t != target);
    }
    pub fn retain(&mut self, valid: impl Fn(&T) -> bool) {
        if self.peek.as_ref().is_some_and(|t| !valid(t)) {
            self.peek = None;
        }
        self.kept.retain(valid);
    }
    pub fn cards(&self) -> Vec<(T, bool)> {
        let mut cards: Vec<_> = self.kept.iter().cloned().map(|t| (t, true)).collect();
        if let Some(target) = &self.peek {
            if !self.kept.contains(target) {
                cards.push((target.clone(), false));
            }
        }
        cards
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct FocusCardInfo {
    pub title: String,
    pub subtitle: String,
    pub context: String,
    /// An honest host-provided retained excerpt, never implicitly fetched.
    pub excerpt: Option<String>,
    pub preview_status: String,
    pub primary: Option<String>,
    pub secondary: Option<String>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FocusCardCommand {
    KeepOpen,
    Close,
    Primary,
    Secondary,
}

/// A small independent arrangement for a scoped graph preview. Membership and
/// edges come from the host's resolved scope; this never moves its main canvas.
pub fn context_graph_swatch<Id: Clone + PartialEq>(
    leaf: u64,
    mut members: Vec<(Id, String, String)>,
    edges: Vec<(Id, Id)>,
    focus: Option<Id>,
    label: String,
) -> crate::GraphCanvasSwatch<Id, ()> {
    if let Some(index) = members
        .iter()
        .position(|(id, _, _)| Some(id) == focus.as_ref())
    {
        members.swap(0, index);
    }
    let count = members.len().saturating_sub(1).max(1) as f32;
    let nodes = members
        .into_iter()
        .enumerate()
        .map(|(i, (id, label, key))| {
            let angle = (i.saturating_sub(1) as f32 / count) * std::f32::consts::TAU;
            crate::GraphCanvasNode {
                id,
                kind: (),
                label,
                key: Some(key),
                position: if i == 0 {
                    (0.5, 0.5)
                } else {
                    (0.5 + 0.4 * angle.cos(), 0.5 + 0.4 * angle.sin())
                },
            }
        })
        .collect();
    let mut swatch = crate::GraphCanvasSwatch::new(
        leaf,
        crate::GraphCanvasSubgraph {
            nodes,
            edges: edges
                .into_iter()
                .map(|(from, to)| crate::GraphCanvasEdge { from, to })
                .collect(),
        },
    )
    .with_size(300, 164)
    .with_label(label)
    .with_expand(false)
    .with_node_labels(true);
    swatch.selected = focus;
    swatch
}

pub fn focus_card<State: 'static, Action: 'static, OA: OptionalAction<Action> + 'static>(
    info: FocusCardInfo,
    kept: bool,
    related: Vec<ShellSlot<State, Action>>,
    handler: impl Fn(&mut State, FocusCardCommand) -> OA + 'static,
) -> ShellSlot<State, Action> {
    let body: ShellSlot<State, Action> = Box::new(el(
        "div",
        info.excerpt
            .clone()
            .map(|text| el("p", text).attr("class", "focus-card-excerpt")),
    ));
    focus_card_with_body(info, kept, body, related, handler)
}

/// A context card hosts the destination's actual projection. The body may be a
/// reading, graph swatch, scene or application control surface; Cambium neither
/// resolves the scope nor creates another navigation or content owner.
pub fn focus_card_with_body<
    State: 'static,
    Action: 'static,
    OA: OptionalAction<Action> + 'static,
>(
    info: FocusCardInfo,
    kept: bool,
    body: ShellSlot<State, Action>,
    related: Vec<ShellSlot<State, Action>>,
    handler: impl Fn(&mut State, FocusCardCommand) -> OA + 'static,
) -> ShellSlot<State, Action> {
    let handler = Rc::new(handler);
    let close = handler.clone();
    let title = info.title.clone();
    let header = el(
        "div",
        (
            el("strong", title.clone()).attr("class", "focus-card-title"),
            el("span", if kept { "Kept open" } else { "Peek" }).attr("class", "focus-card-mode"),
            shell_control(
                "×",
                format!("Close focus card: {title}"),
                ShellControl::default(),
                move |state, _| close(state, FocusCardCommand::Close),
            ),
        ),
    )
    .attr("class", "focus-card-header");
    let mut actions: Vec<ShellSlot<State, Action>> = Vec::new();
    for (label, command) in [
        (info.primary.clone(), FocusCardCommand::Primary),
        (info.secondary.clone(), FocusCardCommand::Secondary),
        (
            (!kept).then(|| "Keep open".into()),
            FocusCardCommand::KeepOpen,
        ),
    ] {
        if let Some(label) = label {
            let handler = handler.clone();
            let name = if command == FocusCardCommand::KeepOpen {
                format!("Keep focus card open: {title}")
            } else {
                format!("{label}: {title}")
            };
            actions.push(shell_control(
                label,
                name,
                ShellControl::default(),
                move |state, _| handler(state, command),
            ));
        }
    }
    Box::new(crate::request_focus(
        crate::focusable_if(
            el(
                "section",
                (
                    header,
                    el("p", info.context).attr("class", "focus-card-context"),
                    el("p", info.subtitle).attr("class", "focus-card-subtitle"),
                    el("p", info.preview_status).attr("class", "focus-card-status"),
                    body,
                    el("div", related).attr("class", "focus-card-related"),
                    el("div", actions).attr("class", "focus-card-actions"),
                ),
            )
            .attr("class", "cambium-focus-card")
            .attr("role", "region")
            .attr("aria-label", format!("Focus card: {title}"))
            .attr("data-kept", if kept { "true" } else { "false" }),
            true,
        ),
        !kept,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_context_card_hosts_the_projection_instead_of_repeating_an_excerpt() {
        use crate::{DomHandle, GenetAppRunner};
        use genet_scripted_dom::ScriptedDom;
        use layout_dom_api::LayoutDom;
        use std::{cell::RefCell, rc::Rc};
        fn view(_: &()) -> ShellSlot<(), ()> {
            focus_card_with_body(
                FocusCardInfo {
                    title: "Lower scope".into(),
                    subtitle: "2 objects".into(),
                    context: "Scope below".into(),
                    preview_status: "Live projection".into(),
                    excerpt: Some("fallback excerpt".into()),
                    primary: Some("Enter".into()),
                    secondary: None,
                },
                false,
                Box::new(el("p", "Actual scope contents")),
                Vec::new(),
                |_: &mut (), _| (),
            )
        }
        let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
        let runner = GenetAppRunner::new(dom.clone(), view, ());
        let html = dom.borrow().outer_html(runner.root());
        assert!(html.contains("Actual scope contents"));
        assert!(!html.contains("fallback excerpt"));
        assert!(html.contains("Enter: Lower scope"));
    }

    #[test]
    fn context_swatch_arranges_only_supplied_scope_and_keeps_its_edges() {
        let swatch = context_graph_swatch(
            91,
            vec![
                (1, "One".into(), "one:".into()),
                (2, "Two".into(), "two:".into()),
            ],
            vec![(1, 2)],
            Some(2),
            "Scope below".into(),
        );
        assert_eq!(swatch.graph.nodes[0].id, 2);
        assert_eq!(swatch.graph.nodes[0].position, (0.5, 0.5));
        assert_eq!(
            swatch.graph.edges,
            vec![crate::GraphCanvasEdge { from: 1, to: 2 }]
        );
        assert!(
            !swatch.show_expand,
            "Enter belongs to the card's admitted destination"
        );
        let empty =
            context_graph_swatch::<u8>(92, Vec::new(), Vec::new(), None, "Empty graph".into());
        assert!(empty.graph.nodes.is_empty());
    }
    #[test]
    fn actual_card_focus_and_typed_controls_keep_preview_and_activation_separate() {
        use crate::{AnyView, DomHandle, GenetAppRunner, GenetCtx, GenetElement, PointerClick};
        use genet_scripted_dom::ScriptedDom;
        use layout_dom_api::{LayoutDom, LocalName, Namespace};
        use std::{cell::RefCell, rc::Rc};
        #[derive(Default)]
        struct Host {
            kept: bool,
            events: Vec<FocusCardCommand>,
        }
        fn view(s: &Host) -> Box<dyn AnyView<Host, (), GenetCtx, GenetElement>> {
            focus_card(
                FocusCardInfo {
                    title: "Phrase".into(),
                    subtitle: "Set / Section".into(),
                    context: "Referenced material".into(),
                    excerpt: Some("Retained phrase".into()),
                    preview_status: "Retained excerpt".into(),
                    primary: Some("Rehearse".into()),
                    secondary: None,
                },
                s.kept,
                Vec::new(),
                |s: &mut Host, c| {
                    s.events.push(c);
                    if c == FocusCardCommand::KeepOpen {
                        s.kept = true;
                    }
                },
            )
        }
        let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
        let mut runner = GenetAppRunner::new(dom.clone(), view, Host::default());
        let root = runner.root();
        let card = dom.borrow().all_with_class(root, "cambium-focus-card")[0];
        assert_eq!(runner.focus(), Some(card));
        let find = |label: &str| {
            dom.borrow()
                .all_with_class(root, "shell-control")
                .into_iter()
                .find(|node| {
                    dom.borrow().attribute(
                        *node,
                        &Namespace::from(""),
                        &LocalName::from("aria-label"),
                    ) == Some(label)
                })
        };
        let primary = find("Rehearse: Phrase").unwrap();
        runner.set_focus(Some(primary));
        let keep = find("Keep focus card open: Phrase").unwrap();
        runner.dispatch_click(keep, PointerClick::at((0., 0.)));
        assert_eq!(runner.state().events, vec![FocusCardCommand::KeepOpen]);
        assert!(find("Keep focus card open: Phrase").is_none());
        runner.set_focus(Some(primary));
        runner.update(|_| {});
        assert_eq!(
            runner.focus(),
            Some(primary),
            "a kept card must not take focus on refresh"
        );
        runner.dispatch_click(primary, PointerClick::at((0., 0.)));
        assert_eq!(
            runner.state().events,
            vec![FocusCardCommand::KeepOpen, FocusCardCommand::Primary]
        );
    }

    #[test]
    fn independent_kept_contexts_survive_new_peeks_and_late_keep_is_refused() {
        let mut deck = FocusCardDeck::default();
        let a = ("score", "phrase", "rehearsal");
        let b = ("score", "phrase", "analysis");
        deck.show(a);
        deck.keep(&a);
        deck.show(a);
        assert!(
            deck.peek.is_none(),
            "an already-kept card must not create an invisible peek"
        );
        deck.show(b);
        deck.keep(&a);
        assert_eq!(deck.cards(), vec![(a, true), (b, false)]);
        deck.close(&b);
        assert_eq!(deck.cards(), vec![(a, true)]);
        deck.show(b);
        deck.retain(|t| *t != a);
        assert_eq!(deck.cards(), vec![(b, false)]);
    }
}
