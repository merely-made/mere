// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! A query editor that embeds through Cambium's ordinary `lens`. Owns only
//! draft controls and an outbox; hosts supply source bindings and completions.
//! No browser API, executor, store, Graphshell page or ambient graph is needed.
use cambium::{AnyView, GenetCtx, GenetElement, TextInput, button, el, lens, text_field_typed};
pub use mere::query_view::editor::{Completion, Editor, Effect, SaveState, Scope};
use mere::query_view::{AppearancePolicy, QueryViewSpec, StrokeCue};
use std::collections::VecDeque;
use uuid::Uuid;

pub const FIELD_LABELS: [&str; 5] = [
    "Query view title",
    "SELECT query",
    "Query member budget",
    "Query appearance budget",
    "Query relation budget",
];
pub type Element = Box<dyn AnyView<QueryEditor, (), GenetCtx, GenetElement>>;

pub struct QueryEditor {
    lifecycle: Editor,
    fields: [TextInput; 5],
    effects: VecDeque<Effect>,
    available: bool,
}
impl Default for QueryEditor {
    fn default() -> Self {
        Self::new(Uuid::new_v4())
    }
}
impl QueryEditor {
    pub fn new(instance: Uuid) -> Self {
        Self {
            lifecycle: Editor::new(instance),
            fields: std::array::from_fn(|_| TextInput::new("")),
            effects: VecDeque::new(),
            available: true,
        }
    }
    pub fn lifecycle(&self) -> &Editor {
        &self.lifecycle
    }
    pub fn field(&self, index: usize) -> &TextInput {
        &self.fields[index]
    }
    pub fn field_mut(&mut self, index: usize) -> &mut TextInput {
        &mut self.fields[index]
    }
    pub fn editable(&self) -> bool {
        self.available && self.lifecycle.editable()
    }
    /// Availability is a host capability, not proof of authority. The owner
    /// still validates every effect when it is executed.
    pub fn available(&self) -> bool {
        self.available
    }
    pub fn set_available(&mut self, available: bool) {
        self.available = available;
    }
    pub fn notice(&mut self, message: impl Into<String>) {
        self.lifecycle.notice(message);
    }
    pub fn open(&mut self, scope: Scope, owner: Option<Uuid>, spec: QueryViewSpec) {
        self.fields = [
            TextInput::new(&spec.title),
            TextInput::new(spec.query_text()),
            TextInput::new(spec.max_members.to_string()),
            TextInput::new(spec.max_appearances.to_string()),
            TextInput::new(spec.max_relations.to_string()),
        ];
        self.effects.clear();
        self.lifecycle.open(scope, owner, spec);
    }
    pub fn retire(&mut self) {
        self.lifecycle.retire();
        self.effects.clear();
        self.fields = std::array::from_fn(|_| TextInput::new(""));
    }
    pub fn has_effects(&self) -> bool {
        !self.effects.is_empty()
    }
    pub fn take_effect(&mut self) -> Option<Effect> {
        self.effects.pop_front()
    }
    pub fn complete(&mut self, completion: Completion) -> Option<Uuid> {
        self.lifecycle.complete(completion)
    }
    pub fn use_query(&mut self, query: &str) {
        if !self.editable() {
            return;
        }
        if let Some(draft) = self.lifecycle.draft_mut() {
            draft.definition = QueryViewSpec::select("", query).definition;
            self.fields[1] = TextInput::new(query);
        }
    }
    pub fn save(&mut self) {
        if !self.editable() {
            return;
        }
        let result = (|| {
            let mut spec = self
                .lifecycle
                .draft()
                .ok_or("Open a query draft first")?
                .clone();
            spec.title = self.fields[0].text().into();
            spec.set_query_text(self.fields[1].text())?;
            spec.max_members = self.fields[2]
                .text()
                .parse()
                .map_err(|_| "Enter a positive member budget")?;
            spec.max_appearances = self.fields[3]
                .text()
                .parse()
                .map_err(|_| "Enter a positive appearance budget")?;
            spec.max_relations = self.fields[4]
                .text()
                .parse()
                .map_err(|_| "Enter a positive relation budget")?;
            self.lifecycle.save(spec)
        })();
        match result {
            Ok(effect) => self.effects.push_back(effect),
            Err(error) => self.notice(error),
        }
    }
    pub fn retry(&mut self) {
        if let Some(effect) = self.lifecycle.retry() {
            self.effects.push_back(effect);
        }
    }
    fn cycle_stroke(&mut self, index: usize) {
        if !self.editable() {
            return;
        }
        let Some(draft) = self.lifecycle.draft_mut() else {
            return;
        };
        let styles = &mut draft.styles;
        let current = match index {
            0 => styles.resource_link,
            1 => styles.surface_link,
            _ => styles.cross_stratum_edge,
        };
        let start = StrokeCue::ALL
            .iter()
            .position(|cue| *cue == current)
            .unwrap();
        let next = StrokeCue::ALL
            .iter()
            .cycle()
            .skip(start + 1)
            .take(4)
            .copied()
            .find(|cue| {
                if index < 2 {
                    *cue != styles.cross_stratum_edge
                } else {
                    *cue != styles.resource_link && *cue != styles.surface_link
                }
            })
            .expect("four cues provide a distinct link and edge choice");
        match index {
            0 => styles.resource_link = next,
            1 => styles.surface_link = next,
            _ => styles.cross_stratum_edge = next,
        }
    }
}

/// Embed with `lens(|editor: &mut QueryEditor| view(editor), host_editor)`.
/// Hosts drain effects after dispatch and feed back explicit completions.
pub fn view(editor: &QueryEditor) -> Element {
    let mut children: Vec<Element> = vec![];
    if editor.editable() {
        for index in 0..FIELD_LABELS.len() {
            children.push(Box::new(el(
                "label",
                (
                    FIELD_LABELS[index],
                    lens(
                        move |input: &mut TextInput| {
                            text_field_typed(input).attr("aria-label", FIELD_LABELS[index])
                        },
                        move |editor: &mut QueryEditor| &mut editor.fields[index],
                    ),
                ),
            )));
        }
        let draft = editor.lifecycle.draft().unwrap();
        children.push(Box::new(button(
            match draft.appearances {
                AppearancePolicy::AllHeldAccesses => "Show Resources and held appearances",
                AppearancePolicy::ResourcesOnly => "Show Resources only",
            },
            |editor: &mut QueryEditor, _| {
                if !editor.editable() {
                    return;
                }
                if let Some(draft) = editor.lifecycle.draft_mut() {
                    draft.appearances = match draft.appearances {
                        AppearancePolicy::AllHeldAccesses => AppearancePolicy::ResourcesOnly,
                        AppearancePolicy::ResourcesOnly => AppearancePolicy::AllHeldAccesses,
                    };
                }
            },
        )));
        children.push(Box::new(button(
            format!("Tag context: {}", draft.include_tag_concepts),
            |editor: &mut QueryEditor, _| {
                if !editor.editable() {
                    return;
                }
                if let Some(draft) = editor.lifecycle.draft_mut() {
                    draft.include_tag_concepts = !draft.include_tag_concepts;
                }
            },
        )));
        for (index, cue) in [
            draft.styles.resource_link,
            draft.styles.surface_link,
            draft.styles.cross_stratum_edge,
        ]
        .into_iter()
        .enumerate()
        {
            children.push(Box::new(button(
                format!(
                    "{} stroke: {}",
                    ["Resource link", "Surface link", "Cross-stratum edge"][index],
                    cue.css()
                ),
                move |editor: &mut QueryEditor, _| editor.cycle_stroke(index),
            )));
        }
        children.push(Box::new(button(
            "Save query view",
            |editor: &mut QueryEditor, _| editor.save(),
        )));
    }
    if matches!(editor.lifecycle.state(), SaveState::Unconfirmed(_)) {
        children.push(Box::new(button(
            "Retry query save",
            |editor: &mut QueryEditor, _| editor.retry(),
        )));
    }
    children.push(Box::new(
        el("p", editor.lifecycle.status().to_owned()).attr("role", "status"),
    ));
    Box::new(
        el("div", children)
            .attr("class", "query-editor")
            .attr("aria-label", "Query editor"),
    )
}
