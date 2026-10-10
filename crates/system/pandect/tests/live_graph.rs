use kernel::graph::{Author, Graph, apply::add_node};
use muniment::MemoryBackend;
use pandect::{MereSessions, addressable_graph::GraphAddress, live_graph::*};
use uuid::Uuid;

#[test]
fn batch_can_edit_an_earlier_creation_without_resident_keys() {
    let mut session = MereSessions::new(MemoryBackend::new()).begin_recorded(Author::user(), None);
    let id = Uuid::from_u128(2000);
    let result = session
        .edit_live_now(
            Author::person("alice"),
            vec![
                ObservedCommand {
                    command: LiveCommand::CreateAddress {
                        id,
                        address: "https://live.test/new".into(),
                        title: " New ".into(),
                    },
                    checks: vec![LiveCheck::Absent(GraphAddress::Surface(id))],
                },
                ObservedCommand {
                    command: LiveCommand::SetPinned {
                        surface: id,
                        pinned: true,
                    },
                    checks: vec![LiveCheck::Exists(GraphAddress::Surface(id))],
                },
            ],
        )
        .unwrap();
    assert_eq!(result.created, vec![id]);
    assert_eq!(
        session
            .graph()
            .node_is_pinned(session.graph().get_node_key_by_id(id).unwrap()),
        Some(true)
    );
    assert!(result.applied.end > result.applied.first);
}

#[test]
fn later_missing_target_rejects_entire_batch_and_capture() {
    let mut session = MereSessions::new(MemoryBackend::new()).begin_recorded(Author::user(), None);
    let before = serde_json::to_vec(&session.graph().to_snapshot_at(0)).unwrap();
    let cursor = session.journal().live_cursor();
    let revision = session.revision();
    let id = Uuid::from_u128(2001);
    let missing = Uuid::from_u128(2002);
    assert!(matches!(session.edit_live_now(Author::user(), vec![
        ObservedCommand {
            command: LiveCommand::CreateAddress { id, address: "https://live.test/refused".into(), title: String::new() },
            checks: vec![LiveCheck::Absent(GraphAddress::Surface(id))],
        },
        ObservedCommand {
            command: LiveCommand::SetPinned { surface: missing, pinned: true },
            checks: vec![LiveCheck::Exists(GraphAddress::Surface(missing))],
        },
    ]), Err(LiveEditError::Missing(GraphAddress::Surface(value))) if value == missing));
    assert_eq!(
        serde_json::to_vec(&session.graph().to_snapshot_at(0)).unwrap(),
        before
    );
    assert_eq!(session.journal().live_cursor(), cursor);
    assert_eq!(session.revision(), revision);
    // A following valid edit must not collect the refused creation's captures.
    let result = session
        .edit_live_now(
            Author::user(),
            vec![ObservedCommand {
                command: LiveCommand::CreateAddress {
                    id,
                    address: "https://live.test/accepted".into(),
                    title: String::new(),
                },
                checks: vec![LiveCheck::Absent(GraphAddress::Surface(id))],
            }],
        )
        .unwrap();
    assert_eq!(result.created, vec![id]);
    assert_eq!(session.graph().node_count(), 1);
}

use kernel::graph::{
    ArrangementSubKind, ContainmentSubKind, EdgeAssertion, SemanticSubKind, Seq,
    apply::{GraphDelta, apply_graph_delta},
};
use pandect::addressable_graph::AddressableGraphStore;
use pandect::{ChangeKind, GraphId, GraphSession, GraphSessionManifest, SessionError, SessionId};
use serde_json::{Value, json};
use std::{
    collections::BTreeSet,
    sync::{Arc, Mutex},
};

type Session = GraphSession<MemoryBackend>;
fn fixture() -> (Graph, Uuid, Uuid) {
    let mut graph = Graph::new();
    let a = Uuid::from_u128(3000);
    let ka = add_node(
        &mut graph,
        Some(a),
        "https://live.test/a".into(),
        Default::default(),
    );
    // Deliberate UUID overlap: the second Surface is the first Resource's UUID.
    let b = graph.shown_resource_id(ka).unwrap();
    let kb = add_node(
        &mut graph,
        Some(b),
        "https://live.test/b".into(),
        Default::default(),
    );
    graph.write_as(Author::person("alice"), |g| {
        apply_graph_delta(
            g,
            GraphDelta::InsertNodeTag {
                key: ka,
                tag: "shared".into(),
            },
        );
        apply_graph_delta(
            g,
            GraphDelta::InsertNodeTag {
                key: ka,
                tag: "remove".into(),
            },
        );
    });
    graph.write_as(Author::person("bob"), |g| {
        apply_graph_delta(
            g,
            GraphDelta::InsertNodeTag {
                key: ka,
                tag: "shared".into(),
            },
        );
    });
    apply_graph_delta(
        &mut graph,
        GraphDelta::BranchHistory {
            child: kb,
            parent: ka,
        },
    );
    let ra = graph.shown_resource_id(ka).unwrap();
    let rb = graph.shown_resource_id(kb).unwrap();
    for label in ["retained parallel one", "retained parallel two"] {
        graph
            .try_assert_semantic_statement_by_resource_ids(
                ra,
                rb,
                kernel::graph::SemanticStatementSpec {
                    predicate: kernel::graph::predicate_iri(SemanticSubKind::Cites).into(),
                    recognized_sub_kind: Some(SemanticSubKind::Cites),
                    label: Some(label.into()),
                    graph_scope: kernel::types::GraphScope::Custom("urn:scope:retained".into()),
                    provenance_iri: Some(Author::person("bob").asserter_iri()),
                    asserted_at_ms: Some(55),
                },
            )
            .unwrap();
    }
    (graph, a, b)
}
fn record_checks(graph: &Graph, id: Uuid, content: bool) -> Vec<LiveCheck> {
    let prepared = AddressableGraphStore::new(MemoryBackend::new(), Uuid::nil())
        .prepare_recorded_graph(graph)
        .unwrap();
    let address = GraphAddress::Surface(id);
    let mut checks = vec![LiveCheck::RecordUnchanged {
        address,
        digest: prepared.record_digest(address).unwrap().into(),
    }];
    if content {
        let resource = prepared.shown_resource(id).unwrap();
        checks.push(LiveCheck::ShownUnchanged {
            surface: id,
            resource,
        });
        checks.push(LiveCheck::RecordUnchanged {
            address: GraphAddress::Resource(resource),
            digest: prepared
                .record_digest(GraphAddress::Resource(resource))
                .unwrap()
                .into(),
        });
        checks.push(LiveCheck::ContentTagsUnchanged {
            surface: id,
            tags: graph
                .node_content_tags(graph.get_node_key_by_id(id).unwrap())
                .unwrap()
                .into_iter()
                .collect(),
        });
    }
    checks
}
fn whole(graph: &Graph) -> Value {
    json!({"snapshot": graph.to_snapshot_at(0), "facets": graph.facets()})
}
// Independent live executions mint different statement handles and creation
// clocks. Preserve original handles/clocks; give each new handle a consistent
// ordinal, preserving multiplicity and repeated references. Exact replay tests
// below do not use this normalization.
fn canonical(mut value: Value, baseline: &Graph, created: Option<Uuid>) -> Value {
    fn gather(value: &Value, ids: &mut BTreeSet<String>) {
        match value {
            Value::Object(object) => {
                for (name, value) in object {
                    if name == "statement_id" {
                        ids.insert(value.as_str().unwrap().into());
                    }
                    if name.ends_with("_json") && value.is_string() {
                        if let Ok(embedded) = serde_json::from_str::<Value>(value.as_str().unwrap())
                        {
                            gather(&embedded, ids);
                        }
                    } else {
                        gather(value, ids);
                    }
                }
            },
            Value::Array(values) => {
                for value in values {
                    gather(value, ids);
                }
            },
            _ => {},
        }
    }
    fn visit(
        value: &mut Value,
        original: &BTreeSet<String>,
        minted: &mut std::collections::BTreeMap<String, String>,
        created: Option<&str>,
        creation: bool,
    ) {
        match value {
            Value::Object(object) => {
                let new_statement = object
                    .get("statement_id")
                    .and_then(Value::as_str)
                    .is_some_and(|id| !original.contains(id));
                let creation = creation
                    || created.is_some_and(|id| {
                        object.get("node_id").and_then(Value::as_str) == Some(id)
                    });
                for (name, value) in object {
                    if name == "statement_id" && new_statement {
                        let next = format!("new-statement-{}", minted.len());
                        *value = Value::String(
                            minted
                                .entry(value.as_str().unwrap().into())
                                .or_insert(next)
                                .clone(),
                        );
                    } else if ((name == "asserted_at_ms" && new_statement)
                        || (creation
                            && matches!(name.as_str(), "last_visited_ms" | "timestamp_ms")))
                        && value.is_number()
                    {
                        *value = json!(0);
                    } else if name.ends_with("_json") && value.is_string() {
                        if let Ok(mut embedded) =
                            serde_json::from_str::<Value>(value.as_str().unwrap())
                        {
                            visit(&mut embedded, original, minted, created, creation);
                            *value = Value::String(serde_json::to_string(&embedded).unwrap());
                        }
                    } else {
                        visit(
                            value,
                            original,
                            minted,
                            created,
                            creation || created == Some(name.as_str()),
                        );
                    }
                }
            },
            Value::Array(values) => {
                for value in values {
                    visit(value, original, minted, created, creation);
                }
            },
            _ => {},
        }
    }
    let mut original = BTreeSet::new();
    gather(&whole(baseline), &mut original);
    let created = created.map(|id| id.to_string());
    visit(
        &mut value,
        &original,
        &mut Default::default(),
        created.as_deref(),
        false,
    );
    value
}
fn semantic(kind: SemanticSubKind, label: &str) -> EdgeAssertion {
    EdgeAssertion::Semantic {
        sub_kind: kind,
        label: Some(label.into()),
        decay_progress: None,
    }
}

#[test]
fn six_commands_match_complete_live_deltas_and_retain_exact_replay_history() {
    pollster::block_on(async {
        let (baseline, a, b) = fixture();
        let actor = Author::person("alice");
        for case in 0..10 {
            let sessions = MereSessions::new(MemoryBackend::new());
            let mut live = sessions.begin_recorded(actor.clone(), Some(baseline.clone()));
            let mut direct = MereSessions::new(MemoryBackend::new())
                .begin_recorded(actor.clone(), Some(baseline.clone()));
            let new = Uuid::from_u128(3100);
            let relation = match case {
                5 => Some((a, b, semantic(SemanticSubKind::Cites, "source"))),
                6 => Some((a, a, semantic(SemanticSubKind::Hyperlink, "self"))),
                7 => Some((a, b, semantic(SemanticSubKind::UserGrouped, "group"))),
                8 => Some((
                    a,
                    b,
                    EdgeAssertion::Containment {
                        sub_kind: ContainmentSubKind::CollectionMember,
                    },
                )),
                9 => Some((
                    a,
                    a,
                    EdgeAssertion::Arrangement {
                        sub_kind: ArrangementSubKind::FrameMember,
                    },
                )),
                _ => None,
            };
            let command = match case {
                0 => LiveCommand::CreateAddress {
                    id: new,
                    address: "https://live.test/new-table".into(),
                    title: " New title ".into(),
                },
                1 => LiveCommand::EditNode {
                    surface: a,
                    title: " Edited ".into(),
                    tags: BTreeSet::from([" added ".into(), "added".into(), "".into()]),
                },
                2 => LiveCommand::SetFacet {
                    surface: a,
                    facet: "test.reader".into(),
                    value: json!({"page":3}),
                },
                3 => LiveCommand::SetPinned {
                    surface: a,
                    pinned: true,
                },
                4 => LiveCommand::SetMimeHint {
                    surface: a,
                    mime_hint: Some("text/markdown".into()),
                },
                _ => {
                    let (from, to, assertion) = relation.clone().unwrap();
                    LiveCommand::AssertRelation {
                        from,
                        to,
                        assertion,
                    }
                },
            };
            let checks = if case == 0 {
                vec![LiveCheck::Absent(GraphAddress::Surface(new))]
            } else if let Some((from, to, _)) = &relation {
                let content = case == 5 || case == 6;
                let mut checks = record_checks(live.graph(), *from, content);
                checks.extend(record_checks(live.graph(), *to, content));
                checks
            } else {
                record_checks(live.graph(), a, case == 1)
            };
            let before = whole(live.graph());
            let result = live
                .edit_live_now(actor.clone(), vec![ObservedCommand { command, checks }])
                .unwrap();
            assert!(result.applied.end > result.applied.first, "case {case}");
            direct
                .edit_now(actor.clone(), |graph| {
                    let ka = graph.get_node_key_by_id(a).unwrap();
                    match case {
                        0 => {
                            apply_graph_delta(
                                graph,
                                GraphDelta::AddNode {
                                    id: Some(new),
                                    url: "https://live.test/new-table".into(),
                                    position: Default::default(),
                                },
                            );
                            let key = graph.get_node_key_by_id(new).unwrap();
                            apply_graph_delta(
                                graph,
                                GraphDelta::SetNodeTitle {
                                    key,
                                    title: "New title".into(),
                                },
                            );
                        },
                        1 => {
                            apply_graph_delta(
                                graph,
                                GraphDelta::SetNodeTitle {
                                    key: ka,
                                    title: "Edited".into(),
                                },
                            );
                            apply_graph_delta(
                                graph,
                                GraphDelta::RemoveNodeTag {
                                    key: ka,
                                    tag: "remove".into(),
                                },
                            );
                            apply_graph_delta(
                                graph,
                                GraphDelta::RemoveNodeTag {
                                    key: ka,
                                    tag: "shared".into(),
                                },
                            );
                            apply_graph_delta(
                                graph,
                                GraphDelta::InsertNodeTag {
                                    key: ka,
                                    tag: "added".into(),
                                },
                            );
                        },
                        2 => {
                            apply_graph_delta(
                                graph,
                                GraphDelta::SetNodeFacet {
                                    key: ka,
                                    facet: "test.reader".into(),
                                    value: json!({"page":3}),
                                },
                            );
                        },
                        3 => {
                            apply_graph_delta(
                                graph,
                                GraphDelta::SetNodePinned {
                                    key: ka,
                                    is_pinned: true,
                                },
                            );
                        },
                        4 => {
                            apply_graph_delta(
                                graph,
                                GraphDelta::SetNodeMimeHint {
                                    key: ka,
                                    mime_hint: Some("text/markdown".into()),
                                },
                            );
                        },
                        _ => {
                            let (from, to, assertion) = relation.clone().unwrap();
                            let from = graph.get_node_key_by_id(from).unwrap();
                            let to = graph.get_node_key_by_id(to).unwrap();
                            apply_graph_delta(
                                graph,
                                GraphDelta::AssertRelation {
                                    from,
                                    to,
                                    assertion,
                                    asserter_iri: actor.asserter_iri(),
                                },
                            );
                        },
                    }
                })
                .unwrap();
            assert_eq!(
                canonical(whole(live.graph()), &baseline, (case == 0).then_some(new)),
                canonical(whole(direct.graph()), &baseline, (case == 0).then_some(new)),
                "live parity case {case}"
            );
            let deltas = |session: &Session| {
                json!(
                    session
                        .journal()
                        .entries()
                        .iter()
                        .map(|entry| &entry.delta)
                        .collect::<Vec<_>>()
                )
            };
            assert_eq!(
                canonical(deltas(&live), &baseline, (case == 0).then_some(new)),
                canonical(deltas(&direct), &baseline, (case == 0).then_some(new)),
                "capture parity case {case}"
            );
            if case == 1 {
                let resource = live
                    .graph()
                    .shown_resource_id(live.graph().get_node_key_by_id(a).unwrap())
                    .unwrap();
                assert_eq!(
                    live.graph().resource_tag_labels(resource),
                    std::collections::HashSet::from(["shared".to_owned(), "added".to_owned()])
                );
                let shared = live
                    .graph()
                    .resource_edges()
                    .filter(|(from, to, _)| {
                        from.id() == resource
                            && live
                                .graph()
                                .resource_tag_concept(to.id())
                                .is_some_and(|tag| tag.label == "shared")
                    })
                    .flat_map(|(_, _, payload)| payload.semantic_statements())
                    .collect::<Vec<_>>();
                assert_eq!(shared.len(), 1);
                assert_eq!(
                    shared[0].provenance_iri,
                    Some(Author::person("bob").asserter_iri())
                );
            }
            let exact = whole(live.graph());
            let history = live.journal().entries().to_vec();
            live.flush(std::time::UNIX_EPOCH).await.unwrap();
            let mut reopened = sessions.open(live.id()).await.unwrap();
            assert_eq!(whole(reopened.graph()), exact, "exact reopen case {case}");
            assert_eq!(
                reopened.journal().entries(),
                history,
                "exact journal case {case}"
            );
            let undone = reopened.undo_now(actor.clone()).unwrap().unwrap();
            assert!(undone.applied.end > undone.applied.first);
            direct.undo_now(actor.clone()).unwrap().unwrap();
            assert_eq!(
                canonical(
                    whole(reopened.graph()),
                    &baseline,
                    (case == 0).then_some(new)
                ),
                canonical(whole(direct.graph()), &baseline, (case == 0).then_some(new)),
                "undo parity case {case}"
            );
            // Undo may retain unreferenced Resource records; full direct-live
            // undo, rather than manual deletion, is the authority for that state.
            assert_eq!(reopened.graph().node_count(), baseline.node_count());
            if case != 0 && case != 1 {
                assert_eq!(whole(reopened.graph()), before, "inverse case {case}");
            }
            reopened.flush(std::time::UNIX_EPOCH).await.unwrap();
            let after_undo = sessions.open(reopened.id()).await.unwrap();
            assert_eq!(
                whole(after_undo.graph()),
                whole(reopened.graph()),
                "exact undo reopen case {case}"
            );
            assert_eq!(
                after_undo.journal().entries(),
                reopened.journal().entries(),
                "exact undo journal case {case}"
            );
        }
    });
}

#[test]
fn mandatory_observations_and_changed_hidden_incidents_refuse_without_effects() {
    let (baseline, a, b) = fixture();
    let mut session =
        MereSessions::new(MemoryBackend::new()).begin_recorded(Author::user(), Some(baseline));
    let original = record_checks(session.graph(), a, true);
    for omitted in 0..original.len() {
        let mut checks = original.clone();
        checks.remove(omitted);
        let before = whole(session.graph());
        let cursor = session.journal().live_cursor();
        let revision = session.revision();
        assert!(matches!(
            session.edit_live_now(
                Author::person("alice"),
                vec![ObservedCommand {
                    command: LiveCommand::EditNode {
                        surface: a,
                        title: "bad".into(),
                        tags: BTreeSet::new()
                    },
                    checks,
                }]
            ),
            Err(LiveEditError::MissingObservation(_))
        ));
        assert_eq!(whole(session.graph()), before);
        assert_eq!(session.journal().live_cursor(), cursor);
        assert_eq!(session.revision(), revision);
    }
    session
        .edit_now(Author::person("hidden-author"), |graph| {
            let from = graph.get_node_key_by_id(a).unwrap();
            let to = graph.get_node_key_by_id(b).unwrap();
            apply_graph_delta(
                graph,
                GraphDelta::AssertRelation {
                    from,
                    to,
                    assertion: semantic(SemanticSubKind::Cites, "hidden incident"),
                    asserter_iri: Author::person("hidden-author").asserter_iri(),
                },
            );
        })
        .unwrap();
    let before = whole(session.graph());
    let cursor = session.journal().live_cursor();
    assert!(matches!(
        session.edit_live_now(
            Author::person("alice"),
            vec![ObservedCommand {
                command: LiveCommand::EditNode {
                    surface: a,
                    title: "stale".into(),
                    tags: BTreeSet::new()
                },
                checks: original,
            }]
        ),
        Err(LiveEditError::Stale(GraphAddress::Resource(_)))
    ));
    assert_eq!(whole(session.graph()), before);
    assert_eq!(session.journal().live_cursor(), cursor);
}

#[test]
fn unrelated_edits_preserve_valid_observations_and_url_hits_do_not_alias_requested_ids() {
    let (baseline, a, b) = fixture();
    let mut session =
        MereSessions::new(MemoryBackend::new()).begin_recorded(Author::user(), Some(baseline));
    let checks = record_checks(session.graph(), a, false);
    session
        .edit_now(Author::user(), |graph| {
            let key = graph.get_node_key_by_id(b).unwrap();
            apply_graph_delta(
                graph,
                GraphDelta::SetNodeTitle {
                    key,
                    title: "unrelated".into(),
                },
            );
        })
        .unwrap();
    session
        .edit_live_now(
            Author::user(),
            vec![ObservedCommand {
                command: LiveCommand::SetPinned {
                    surface: a,
                    pinned: true,
                },
                checks,
            }],
        )
        .unwrap();
    let unused = Uuid::from_u128(3200);
    for requested in [a, unused] {
        let result = session
            .edit_live_now(
                Author::user(),
                vec![ObservedCommand {
                    command: LiveCommand::CreateAddress {
                        id: requested,
                        address: "https://live.test/a".into(),
                        title: "ignored".into(),
                    },
                    checks: vec![LiveCheck::Absent(GraphAddress::Surface(requested))],
                }],
            )
            .unwrap();
        assert_eq!(result.created, vec![a]);
        assert_eq!(result.applied.first, result.applied.end);
    }
    assert!(matches!(session.edit_live_now(Author::user(), vec![
        ObservedCommand { command: LiveCommand::CreateAddress { id: unused, address: "https://live.test/a".into(), title: String::new() }, checks: vec![] },
        ObservedCommand { command: LiveCommand::SetPinned { surface: unused, pinned: true }, checks: vec![LiveCheck::Exists(GraphAddress::Surface(unused))] },
    ]), Err(LiveEditError::Missing(GraphAddress::Surface(id))) if id == unused));
    assert_eq!(session.graph().node_count(), 2);
}

#[test]
fn stale_shown_binding_and_unsupported_unbound_content_refuse() {
    let (baseline, a, b) = fixture();
    let mut session =
        MereSessions::new(MemoryBackend::new()).begin_recorded(Author::user(), Some(baseline));
    let stale = record_checks(session.graph(), a, true);
    let resource_b = session
        .graph()
        .shown_resource_id(session.graph().get_node_key_by_id(b).unwrap())
        .unwrap();
    session
        .edit_now(Author::user(), |graph| {
            apply_graph_delta(
                graph,
                GraphDelta::ReplaySetShownResourceById {
                    surface_id: a,
                    resource_id: Some(resource_b),
                },
            );
        })
        .unwrap();
    let before = whole(session.graph());
    assert!(
        matches!(session.edit_live_now(Author::user(), vec![ObservedCommand {
        command: LiveCommand::EditNode { surface: a, title: String::new(), tags: BTreeSet::new() }, checks: stale,
    }]), Err(LiveEditError::Stale(GraphAddress::Surface(id))) if id == a)
    );
    assert_eq!(whole(session.graph()), before);
    session
        .edit_now(Author::user(), |graph| {
            apply_graph_delta(
                graph,
                GraphDelta::ReplaySetShownResourceById {
                    surface_id: a,
                    resource_id: None,
                },
            );
        })
        .unwrap();
    let before = whole(session.graph());
    let checks = record_checks(session.graph(), a, false);
    assert!(matches!(
        session.edit_live_now(
            Author::user(),
            vec![ObservedCommand {
                command: LiveCommand::EditNode {
                    surface: a,
                    title: String::new(),
                    tags: BTreeSet::new()
                },
                checks,
            }]
        ),
        Err(LiveEditError::NeedsCompleteOperation)
    ));
    assert_eq!(whole(session.graph()), before);
}

#[test]
fn placement_readonly_validator_and_invalid_commands_do_not_leak_capture() {
    let (baseline, a, b) = fixture();
    let sessions = MereSessions::new(MemoryBackend::new());
    let mut unqualified = sessions.begin(Author::user(), Some(baseline.clone()));
    assert!(matches!(
        unqualified.edit_live_now(Author::user(), vec![]),
        Err(LiveEditError::Session(SessionError::Validation(_)))
    ));
    let mut manifest = GraphSessionManifest::new(SessionId::new(), GraphId::new());
    manifest.codicil_read_only = true;
    let mut readonly = GraphSession::new_recorded(
        MemoryBackend::new(),
        manifest,
        Some(baseline.clone()),
        Author::user(),
        ChangeKind::Minted,
    );
    assert!(matches!(
        readonly.edit_live_now(Author::user(), vec![]),
        Err(LiveEditError::Session(SessionError::ReadOnly(_)))
    ));
    let mut session = sessions
        .begin_recorded(Author::user(), Some(baseline))
        .with_validator(Arc::new(move |graph| {
            let key = graph.get_node_key_by_id(a).unwrap();
            if graph.node_is_pinned(key) == Some(true) {
                Err("pin rejected".into())
            } else {
                Ok(())
            }
        }))
        .unwrap();
    let seen = Arc::new(Mutex::new(Vec::new()));
    let observer = seen.clone();
    kernel::graph::set_captured_delta_hook(Some(Arc::new(move |delta| {
        observer.lock().unwrap().push(delta.clone())
    })));
    let before = whole(session.graph());
    let cursor = session.journal().live_cursor();
    let revision = session.revision();
    let checks = record_checks(session.graph(), a, false);
    let refused = session.edit_live_now(
        Author::user(),
        vec![ObservedCommand {
            command: LiveCommand::SetPinned {
                surface: a,
                pinned: true,
            },
            checks,
        }],
    );
    let mut checks = record_checks(session.graph(), a, true);
    checks.extend(record_checks(session.graph(), b, true));
    let invalid = session.edit_live_now(
        Author::user(),
        vec![ObservedCommand {
            command: LiveCommand::AssertRelation {
                from: a,
                to: b,
                assertion: EdgeAssertion::Semantic {
                    sub_kind: SemanticSubKind::Cites,
                    label: None,
                    decay_progress: Some(f32::NAN),
                },
            },
            checks,
        }],
    );
    kernel::graph::set_captured_delta_hook(None);
    assert!(matches!(
        refused,
        Err(LiveEditError::Session(SessionError::Validation(_)))
    ));
    assert!(matches!(invalid, Err(LiveEditError::InvalidCommand(_))));
    assert!(seen.lock().unwrap().is_empty());
    assert_eq!(whole(session.graph()), before);
    assert_eq!(session.journal().live_cursor(), cursor);
    assert_eq!(session.revision(), revision);
    let checks = record_checks(session.graph(), b, false);
    session
        .edit_live_now(
            Author::user(),
            vec![ObservedCommand {
                command: LiveCommand::SetMimeHint {
                    surface: b,
                    mime_hint: Some("application/pdf".into()),
                },
                checks,
            }],
        )
        .unwrap();
    assert_eq!(
        session.journal().entries().len(),
        1,
        "no refused scratch captures"
    );
}

#[test]
fn qualified_legacy_session_accepts_live_edit_reopen_and_undo() {
    pollster::block_on(async {
        let backend = MemoryBackend::new();
        let sessions = MereSessions::new(backend);
        let mut graph = Graph::new();
        let id = Uuid::from_u128(3300);
        add_node(
            &mut graph,
            Some(id),
            "https://legacy.test/".into(),
            Default::default(),
        );
        let mut session = sessions.begin(Author::user(), Some(graph));
        session.flush(std::time::UNIX_EPOCH).await.unwrap();
        let mut session = sessions.open(session.id()).await.unwrap();
        session.qualify_legacy(Seq(0), vec![]).await.unwrap();
        let before = whole(session.graph());
        let checks = record_checks(session.graph(), id, false);
        session
            .edit_live_now(
                Author::person("alice"),
                vec![ObservedCommand {
                    command: LiveCommand::SetPinned {
                        surface: id,
                        pinned: true,
                    },
                    checks,
                }],
            )
            .unwrap();
        let exact = whole(session.graph());
        session.flush(std::time::UNIX_EPOCH).await.unwrap();
        let mut reopened = sessions.open(session.id()).await.unwrap();
        assert_eq!(whole(reopened.graph()), exact);
        reopened.undo_now(Author::person("alice")).unwrap().unwrap();
        assert_eq!(whole(reopened.graph()), before);
    });
}
