// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0
#![cfg(feature = "query-editor")]
//! A separate, windowless native host: its own graph, write acknowledgements
//! and placement. No Graphshell state, storage flags or frame hook is used.
use cambium::{el, lens};
use cambium_genet_winit_host::Harness;
use mere::kernel::graph::Graph;
use mere::query_view::editor::{Completion, Effect, SaveRequest, SaveResult, SaveState, Scope};
use mere::query_view::{self, QueryViewSpec};
use mere_view::{
    MereViewElement,
    query_editor::{self, QueryEditor},
};
use taproot::Selector;
use uuid::Uuid;

struct Host {
    editor: QueryEditor,
    graph: Graph,
    pending: Option<(SaveRequest, Uuid)>,
    created: usize,
}
impl Host {
    fn new() -> Self {
        let mut editor = QueryEditor::new(Uuid::from_u128(10));
        editor.open(
            Scope {
                session: Uuid::from_u128(1),
                persona: "host:test".into(),
            },
            None,
            QueryViewSpec::select("Host-owned query", "SELECT ?member WHERE { FILTER(false) }"),
        );
        Self {
            editor,
            graph: Graph::new(),
            pending: None,
            created: 0,
        }
    }
    fn accept(&mut self) {
        let effect = self
            .editor
            .take_effect()
            .expect("a real control emitted an effect");
        match effect {
            Effect::Save(request) => {
                let owner = query_view::save_spec(
                    &mut self.graph,
                    request.owner,
                    request.expected.as_ref(),
                    Uuid::from_u128(100),
                    &request.spec,
                )
                .unwrap();
                self.created += 1;
                self.pending = Some((request, owner));
            },
            Effect::Retry { id, scope } => {
                let (request, _) = self.pending.as_mut().unwrap();
                assert!(id.same_operation(request.id));
                assert_eq!(id.attempt, request.id.attempt + 1);
                request.id = id;
                assert_eq!(scope, request.scope);
            },
        }
    }
    fn acknowledge(&mut self, success: bool) {
        let (request, owner) = self.pending.as_ref().unwrap();
        let result = if success {
            SaveResult::Committed {
                owner: *owner,
                access: None,
            }
        } else {
            SaveResult::Unconfirmed {
                reason: "fixture store failure".into(),
            }
        };
        self.editor.complete(Completion {
            id: request.id,
            scope: request.scope.clone(),
            result,
        });
        if success {
            self.pending = None;
        }
    }
}
fn root(_: &Host) -> MereViewElement<Host, ()> {
    Box::new(el(
        "main",
        lens(
            |editor: &mut QueryEditor| query_editor::view(editor),
            |host: &mut Host| &mut host.editor,
        ),
    ))
}

#[test]
fn native_host_routes_the_editor_and_retries_without_recreating_source() {
    let mut host = Harness::new(
        "main {display:flex;flex-direction:column;padding:10px} label {display:block} button {display:block;padding:6px} [data-cambium-text-value] {display:block;min-height:24px}",
        Host::new(),
        root as fn(&Host) -> MereViewElement<Host, ()>,
    );
    host.layout_at(800.0, 900.0);
    assert!(host.click_on(&Selector::role("button").containing("Save query view")));
    assert!(matches!(
        host.state().editor.lifecycle().state(),
        SaveState::Saving(_)
    ));
    host.update(|host| {
        host.accept();
        host.acknowledge(false);
    });
    assert!(matches!(
        host.state().editor.lifecycle().state(),
        SaveState::Unconfirmed(_)
    ));
    assert!(host.click_on(&Selector::role("button").containing("Retry query save")));
    host.update(|host| {
        host.accept();
        host.acknowledge(true);
    });
    assert_eq!(host.state().created, 1);
    assert_eq!(host.state().editor.lifecycle().status(), "Query view saved");
    let owner = host.state().editor.lifecycle().owner().unwrap();
    assert_eq!(
        query_view::read_spec(&host.state().graph, owner)
            .unwrap()
            .title,
        "Host-owned query"
    );
}

#[test]
fn removal_retires_ui_interest_without_cancelling_the_owners_write() {
    let mut host = Host::new();
    host.editor.save();
    host.accept();
    host.editor.retire();
    host.acknowledge(true);
    assert_eq!(host.created, 1);
    assert_eq!(host.graph.resource_nodes().count(), 1);
    assert!(host.editor.lifecycle().owner().is_none());
    assert!(host.editor.lifecycle().draft().is_none());
}
