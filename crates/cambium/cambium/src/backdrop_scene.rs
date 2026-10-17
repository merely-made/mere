// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Semantic controls and inspection for an independently embedded scene.
//! The stage and its input/renderer belong to the embedding host; this view
//! emits occurrence selection and never obtains source-edit authority.
use crate::{AnyView, GenetCtx, GenetElement, OptionalAction, View, button, el};
use scenomise::backdrops::BackdropScene;
use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BackdropSceneEvent {
    Select(String),
    Clear,
    DismissRetained,
}

pub fn backdrop_scene_reading<State, Action, OA, OnEvent>(
    reading: &BackdropScene,
    selected: Option<&str>,
    retained: &BTreeSet<String>,
    on_event: OnEvent,
) -> impl View<State, Action, GenetCtx, Element = GenetElement> + use<State, Action, OA, OnEvent>
where
    State: 'static,
    Action: 'static,
    OA: OptionalAction<Action> + 'static,
    OnEvent: Fn(&mut State, BackdropSceneEvent) -> OA + Clone + 'static,
{
    type Child<S, A> = Box<dyn AnyView<S, A, GenetCtx, GenetElement>>;
    let mut children: Vec<Child<State, Action>> = vec![Box::new(el("h3", "Selection"))];
    let selected_mark = reading
        .marks
        .iter()
        .find(|m| m.selectable && Some(m.key.as_str()) == selected);
    let selected_unplaced = reading
        .unplaced
        .iter()
        .find(|m| Some(m.key.as_str()) == selected);
    if let Some(mark) = selected_mark {
        let source =
            &reading.scene.sources[reading.scene.items[mark.instance.0 as usize].source.0 as usize];
        children.push(Box::new(
            el("p", mark.label.clone())
                .attr("class", "scene-selection")
                .attr("role", "status"),
        ));
        children.push(Box::new(el("p", mark.detail.clone())));
        for field in reading
            .fields
            .iter()
            .filter(|field| field.device.as_deref() == selected)
        {
            let tier = match field.tier {
                Some(scenomise::backdrops::CoverageTier::Fringe) => "Fringe",
                Some(scenomise::backdrops::CoverageTier::Usable) => "Usable",
                Some(scenomise::backdrops::CoverageTier::Near) => "Near",
                None => "Unmeasured",
            };
            children.push(Box::new(el(
                "p",
                format!("{tier} coverage: {}", field.basis),
            )));
        }
        children.push(Box::new(el(
            "p",
            format!(
                "Source: {} / {} · {}",
                source.adapter, source.id, reading.revision
            ),
        )));
        for relation in &reading.relations {
            let edge = &reading.scene.relations[relation.index];
            let other = if edge.from == mark.instance {
                Some(edge.to)
            } else if edge.to == mark.instance {
                Some(edge.from)
            } else {
                None
            };
            if let Some(other) = other.and_then(|id| {
                reading
                    .marks
                    .iter()
                    .find(|m| m.instance == id && m.selectable)
            }) {
                let key = other.key.clone();
                let emit = on_event.clone();
                children.push(Box::new(
                    button(
                        format!("{}: {}", relation.label, other.label),
                        move |state: &mut State, _| {
                            emit(state, BackdropSceneEvent::Select(key.clone()))
                        },
                    )
                    .attr("aria-label", format!("Related occurrence {}", other.key)),
                ));
                children.push(Box::new(el("p", relation.why.clone())));
            }
        }
    } else if let Some(unplaced) = selected_unplaced {
        children.push(Box::new(
            el("p", unplaced.label.clone())
                .attr("class", "scene-selection")
                .attr("role", "status"),
        ));
        children.push(Box::new(el("p", unplaced.reason.clone())));
    } else {
        children.push(Box::new(el("p", "Nothing selected").attr("role", "status")));
    }
    children.push(Box::new(
        el(
            "p",
            format!(
                "forward {} · retained {}",
                usize::from(selected.is_some()),
                retained.len()
            ),
        )
        .attr("data-scene-attention", "true"),
    ));
    for (label, event) in [
        ("Clear selection", BackdropSceneEvent::Clear),
        ("Dismiss retained", BackdropSceneEvent::DismissRetained),
    ] {
        let emit = on_event.clone();
        children.push(Box::new(button(label, move |state: &mut State, _| {
            emit(state, event.clone())
        })));
    }
    children.push(Box::new(el("h3", "Occurrences")));
    for mark in reading.marks.iter().filter(|m| m.selectable) {
        let key = mark.key.clone();
        let emit = on_event.clone();
        children.push(Box::new(
            button(
                format!("{} · {}", mark.label, mark.detail),
                move |state: &mut State, _| emit(state, BackdropSceneEvent::Select(key.clone())),
            )
            .attr(
                "aria-label",
                format!("Scene occurrence {}: {}", mark.key, mark.label),
            )
            .attr(
                "aria-pressed",
                (selected == Some(mark.key.as_str())).to_string(),
            )
            .attr("data-scene-occurrence", mark.key.clone()),
        ));
    }
    if !reading.unplaced.is_empty() {
        children.push(Box::new(el("h3", "Position unknown")));
    }
    for mark in &reading.unplaced {
        let key = mark.key.clone();
        let emit = on_event.clone();
        children.push(Box::new(
            button(mark.label.clone(), move |state: &mut State, _| {
                emit(state, BackdropSceneEvent::Select(key.clone()))
            })
            .attr(
                "aria-pressed",
                (selected == Some(mark.key.as_str())).to_string(),
            )
            .attr("data-scene-occurrence", mark.key.clone()),
        ));
    }
    for diagnostic in &reading.diagnostics {
        children.push(Box::new(
            el("p", diagnostic.clone())
                .attr("role", "status")
                .attr("class", "scene-diagnostic"),
        ));
    }
    el("aside", children)
        .attr("class", "backdrop-scene-reading")
        .attr("aria-label", "Scene selection and occurrences")
}

pub const BACKDROP_SCENE_CSS: &str = "\
 .backdrop-scene-workspace { display:flex; flex-direction:column; flex:1 1 auto; min-width:0; } \
 .backdrop-scene-bar { display:flex; flex-wrap:wrap; gap:6px; align-items:center; padding:8px 12px; border-bottom:1px solid var(--workspace-rule, #444d3e); } \
 .backdrop-scene-body { display:flex; flex-direction:row; min-width:0; } \
 .backdrop-scene-stage { min-width:0; max-width:100%; flex:0 0 auto; overflow-x:auto; } \
 .backdrop-scene-stages { display:flex; flex-direction:column; min-width:0; flex:1 1 auto; } \
 .backdrop-scene-reading { width:240px; flex:none; padding:12px; border-left:1px solid var(--workspace-rule, #444d3e); overflow-wrap:anywhere; } \
 .backdrop-scene-reading h3 { font-size:13px; font-weight:500; margin:4px 0 8px; } \
 .backdrop-scene-reading p { font-size:12px; margin:6px 0; } \
 .backdrop-scene-reading button { display:block; width:100%; text-align:left; margin:4px 0; padding:5px; background:transparent; color:inherit; border:1px solid var(--workspace-rule, #444d3e); } \
 .backdrop-scene-reading .scene-selection { font-family:var(--tabard-font-reading, Roboto),sans-serif; font-size:18px; } \
 .backdrop-scene-reading .scene-diagnostic { color:var(--tabard-color-danger, #dca494); } \
 @media(max-width:720px) { .backdrop-scene-body { flex-direction:column; } .backdrop-scene-reading { width:100%; border-left:0; border-top:1px solid var(--workspace-rule, #444d3e); } }";

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DomHandle, GenetAppRunner, PointerClick};
    use genet_scripted_dom::ScriptedDom;
    use layout_dom_api::{LayoutDom, LocalName, Namespace};
    use std::{cell::RefCell, rc::Rc};
    struct State {
        reading: BackdropScene,
        events: Vec<BackdropSceneEvent>,
    }
    type Child = Box<dyn AnyView<State, (), GenetCtx, GenetElement>>;
    fn view(state: &State) -> Child {
        Box::new(backdrop_scene_reading(
            &state.reading,
            None,
            &BTreeSet::new(),
            |state: &mut State, event| state.events.push(event),
        ))
    }
    fn descendants(
        dom: &ScriptedDom,
        node: genet_scripted_dom::NodeId,
    ) -> Vec<genet_scripted_dom::NodeId> {
        let mut all = vec![node];
        for child in dom.dom_children(node) {
            all.extend(descendants(dom, child));
        }
        all
    }
    #[test]
    fn semantic_actions_keep_distinct_occurrences_and_never_mutate_source() {
        let input = scenomise::backdrops::SceneInput::parse(include_str!(
            "../../../../support/design-studies/backdrops/fretboard.json"
        ))
        .unwrap();
        let reading = input.compile().unwrap();
        let original = reading.clone();
        let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
        let mut runner = GenetAppRunner::new(
            dom.clone(),
            view as fn(&State) -> Child,
            State {
                reading,
                events: vec![],
            },
        );
        let buttons = {
            let d = dom.borrow();
            descendants(&d, d.document())
                .into_iter()
                .filter_map(|node| {
                    d.attribute(
                        node,
                        &Namespace::from(""),
                        &LocalName::from("data-scene-occurrence"),
                    )
                    .filter(|key| *key == "string-2-fret-1" || *key == "string-5-fret-3")
                    .map(|key| (node, key.to_owned()))
                })
                .collect::<Vec<_>>()
        };
        assert_eq!(buttons.len(), 2);
        for (node, _) in &buttons {
            runner.dispatch_click(*node, PointerClick::at((1.0, 1.0)));
        }
        assert_eq!(
            runner.state().events,
            buttons
                .into_iter()
                .map(|(_, key)| BackdropSceneEvent::Select(key))
                .collect::<Vec<_>>()
        );
        assert_eq!(runner.state().reading, original);
    }
}
