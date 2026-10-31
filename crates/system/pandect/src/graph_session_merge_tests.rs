// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Session merge-in: branches round-trip, picks and picks of picks merge back,
//! conflicts take each side, deletions merge three-way, and resource facets
//! with a rule merge as Compose merges them.

use super::*;
use kernel::graph::resource_content::RESOURCE_PROPERTIES;
use kernel::persistence::PersistedResourceRecord;
use muniment::MemoryBackend;

fn person() -> Author {
    Author::person("persona-a").via("turnstone")
}

fn other() -> Author {
    Author::person("persona-b").via("knot-editor")
}

fn id(n: u128) -> Uuid {
    Uuid::from_u128(n)
}

fn add(n: u128) -> CapturedDelta {
    CapturedDelta::ReplayAddNodeWithIdIfMissing {
        id: id(n).to_string(),
        url: format!("https://{n}.test/"),
        position: [0.0, 0.0],
    }
}

fn add_id(node: Uuid) -> CapturedDelta {
    CapturedDelta::ReplayAddNodeWithIdIfMissing {
        id: node.to_string(),
        url: format!("https://{node}.test/"),
        position: [0.0, 0.0],
    }
}

fn retitle(node: Uuid, title: &str) -> CapturedDelta {
    CapturedDelta::ReplaySetNodeTitleById {
        node_id: node.to_string(),
        title: title.to_string(),
    }
}

fn tag(node: Uuid, tag: &str) -> CapturedDelta {
    CapturedDelta::ReplayInsertNodeTagById {
        node_id: node.to_string(),
        tag: tag.to_string(),
    }
}

fn remove(node: Uuid) -> CapturedDelta {
    CapturedDelta::ReplayRemoveNodeById {
        node_id: node.to_string(),
    }
}

fn relate(from: Uuid, to: Uuid) -> CapturedDelta {
    CapturedDelta::ReplayAssertRelationByIds {
        from_id: from.to_string(),
        to_id: to.to_string(),
        assertion: kernel::graph::EdgeAssertion::Semantic {
            sub_kind: kernel::graph::SemanticSubKind::Cites,
            label: None,
        },
    }
}

fn title(graph: &Graph, node: Uuid) -> Option<String> {
    graph
        .get_node_by_id(node)
        .map(|(_, node)| node.title.clone())
}

async fn backend_snapshot(backend: &MemoryBackend) -> BTreeMap<String, Vec<u8>> {
    let mut keys = backend.list("").await.unwrap();
    keys.sort();
    let mut snapshot = BTreeMap::new();
    for key in keys {
        if let Some(value) = backend.get(&key).await.unwrap() {
            snapshot.insert(key, value);
        }
    }
    snapshot
}

fn related(graph: &Graph, from: Uuid, to: Uuid) -> bool {
    match (graph.get_node_by_id(from), graph.get_node_by_id(to)) {
        (Some((from, _)), Some((to, _))) => !graph.persisted_edges_between(from, to).is_empty(),
        _ => false,
    }
}

fn derivations(graph: &Graph, node: Uuid) -> Vec<NodeDerivation> {
    let (key, _) = graph.get_node_by_id(node).expect("node");
    graph.node_derivations(key).unwrap_or_default()
}

fn merged_from(graph: &Graph, node: Uuid) -> Vec<NodeDerivation> {
    derivations(graph, node)
        .into_iter()
        .filter(|derivation| derivation.sub_kind == ProvenanceSubKind::MergedFrom)
        .collect()
}

/// The picked node copied from `origin` in `container`.
fn copy_of(graph: &Graph, origin: Uuid, container: Uuid) -> Uuid {
    graph
        .nodes()
        .map(|(_, node)| node.id)
        .find(|node| copied_from(&[graph], *node, container) == Some(origin))
        .expect("a copy of the origin")
}

/// A session holding nodes 1 and 2 related, and a lone node 3.
async fn parent(mere: &MereSessions<MemoryBackend>) -> GraphSession<MemoryBackend> {
    let made = mere.mint(person(), Some("Reading".into())).await.unwrap();
    let mut parent = mere.open(made.session_id).await.unwrap();
    parent
        .apply(
            person(),
            vec![
                add(1),
                add(2),
                add(3),
                retitle(id(1), "One"),
                retitle(id(2), "Two"),
                relate(id(1), id(2)),
            ],
        )
        .await
        .unwrap();
    parent
}

async fn branch_of(
    mere: &MereSessions<MemoryBackend>,
    parent: &GraphSession<MemoryBackend>,
) -> GraphSession<MemoryBackend> {
    let made = mere
        .branch_at(parent, parent.journal().live_cursor(), person())
        .await
        .unwrap();
    mere.open(made.session_id).await.unwrap()
}

#[test]
fn a_branch_round_trips_into_its_parent_and_is_kept_merged() {
    pollster::block_on(async {
        let store = MemoryBackend::new();
        let mere = MereSessions::new(store.clone());
        let mut target = parent(&mere).await;
        let mut branch = branch_of(&mere, &target).await;
        assert_eq!(branch.manifest().lineage, Lineage::Branch);
        assert_eq!(branch.manifest().parent_session, Some(target.id()));
        assert_eq!(
            branch.manifest().display_name.as_deref(),
            Some("Reading (branch of Reading)")
        );
        assert_eq!(
            branch.changes()[0].kind,
            ChangeKind::Branched {
                from: target.id(),
                at: target.journal().live_cursor(),
            }
        );

        // The branch retitles 1, adds 4 related to 1, and deletes 3; the parent
        // meanwhile tags 1, a part the branch left alone.
        branch
            .apply(
                other(),
                vec![
                    retitle(id(1), "One, revised"),
                    add(4),
                    relate(id(1), id(4)),
                    remove(id(3)),
                ],
            )
            .await
            .unwrap();
        target
            .apply(person(), vec![tag(id(1), "kept")])
            .await
            .unwrap();
        let before = target.graph().clone();

        let plan = mere.merge_in(&target, &branch).await.unwrap();
        assert!(plan.is_clean(), "{:?}", plan.conflicts());
        assert_eq!(plan.matched(), MergeMatch::Branch);
        let through = branch.journal().live_cursor();
        mere.apply_merge(&mut target, &mut branch, plan, &BTreeMap::new(), person())
            .await
            .unwrap();

        let graph = target.graph();
        assert_eq!(title(graph, id(1)).as_deref(), Some("One, revised"));
        let (_, node) = graph.get_node_by_id(id(1)).unwrap();
        assert!(node.tags.contains("kept"), "the parent's own edit stays");
        assert!(related(graph, id(1), id(4)));
        assert!(graph.get_node_by_id(id(3)).is_none(), "the deletion merges");
        assert!(related(graph, id(1), id(2)));
        let branch_container = branch.manifest().root_graph_id.as_uuid().to_string();
        for node in [id(1), id(4)] {
            assert_eq!(
                merged_from(graph, node),
                vec![NodeDerivation {
                    sub_kind: ProvenanceSubKind::MergedFrom,
                    source_node: node.to_string(),
                    source_graph: Some(branch_container.clone()),
                }],
                "a merged node names where it came from"
            );
        }
        assert!(
            merged_from(graph, id(2)).is_empty(),
            "untouched nodes do not"
        );
        assert_eq!(
            target.changes().last().unwrap().kind,
            ChangeKind::MergedIn {
                from: branch.id(),
                through,
            }
        );

        // The source is kept, marked, read-only, with its history.
        let mark = branch.manifest().merged.clone().unwrap();
        assert_eq!((mark.into, mark.through), (target.id(), through.0));
        assert!(matches!(
            branch.apply(other(), vec![tag(id(1), "late")]).await,
            Err(SessionError::ReadOnly(_))
        ));
        assert!(matches!(
            mere.merge_in(&target, &branch).await,
            Err(SessionError::NotMergeable(_))
        ));

        // Both survive a reopen.
        let reopened = mere.open(target.id()).await.unwrap();
        assert_eq!(
            title(reopened.graph(), id(1)).as_deref(),
            Some("One, revised")
        );
        assert_eq!(merged_from(reopened.graph(), id(4)).len(), 1);
        let source = mere.open(branch.id()).await.unwrap();
        assert_eq!(source.manifest().merged, Some(mark));
        assert_eq!(source.journal().live_cursor(), through);
        assert_eq!(
            source.changes().last().unwrap().kind,
            ChangeKind::MergedInto {
                into: target.id(),
                through,
            }
        );
        assert_eq!(source.manifest().lineage, Lineage::Branch);

        // Undo takes the whole merge back in one step, and frees the source.
        target.undo(person()).await.unwrap().unwrap();
        let mut freed = mere.open(branch.id()).await.unwrap();
        assert_eq!(freed.manifest().merged, None, "undo clears the mark");
        assert!(
            mere.list()
                .await
                .unwrap()
                .iter()
                .all(|manifest| manifest.merged.is_none())
        );
        freed
            .apply(other(), vec![tag(id(1), "after")])
            .await
            .unwrap();
        assert!(
            mere.merge_in(&target, &freed).await.is_ok(),
            "mergeable again"
        );
        assert_eq!(title(target.graph(), id(1)).as_deref(), Some("One"));
        assert!(target.graph().get_node_by_id(id(4)).is_none());
        assert!(target.graph().get_node_by_id(id(3)).is_some());
        assert_eq!(
            target.graph().node_count(),
            before.node_count(),
            "the graph is as it stood before the merge"
        );
    });
}

#[test]
fn a_branch_of_a_merged_branch_merges_into_the_same_target() {
    pollster::block_on(async {
        let mere = MereSessions::new(MemoryBackend::new());
        let mut target = parent(&mere).await;
        let mut first = branch_of(&mere, &target).await;
        first
            .apply(other(), vec![retitle(id(1), "First")])
            .await
            .unwrap();
        let plan = mere.merge_in(&target, &first).await.unwrap();
        mere.apply_merge(&mut target, &mut first, plan, &BTreeMap::new(), person())
            .await
            .unwrap();
        // An edit to the merged branch goes to a new branch of it.
        let mut second = branch_of(&mere, &first).await;
        second
            .apply(other(), vec![retitle(id(2), "Second")])
            .await
            .unwrap();
        target
            .apply(person(), vec![retitle(id(1), "Target's own")])
            .await
            .unwrap();
        let plan = mere.merge_in(&target, &second).await.unwrap();
        assert!(plan.is_clean(), "{:?}", plan.conflicts());
        mere.apply_merge(&mut target, &mut second, plan, &BTreeMap::new(), person())
            .await
            .unwrap();
        assert_eq!(title(target.graph(), id(2)).as_deref(), Some("Second"));
        assert_eq!(
            title(target.graph(), id(1)).as_deref(),
            Some("Target's own"),
            "the base is the merge point, so the first branch's title is not news"
        );
    });
}

#[test]
fn a_fork_never_merges_in() {
    pollster::block_on(async {
        let mere = MereSessions::new(MemoryBackend::new());
        let target = parent(&mere).await;
        let fork = mere
            .fork_at(&target, target.journal().live_cursor(), person())
            .await
            .unwrap();
        assert_eq!(fork.lineage, Lineage::Fork);
        let fork = mere.open(fork.session_id).await.unwrap();
        assert!(matches!(
            mere.merge_in(&target, &fork).await,
            Err(SessionError::NotMergeable(_))
        ));
        let unrelated = parent(&mere).await;
        let branch = branch_of(&mere, &unrelated).await;
        assert!(matches!(
            mere.merge_in(&target, &branch).await,
            Err(SessionError::NotMergeable(_))
        ));
    });
}

/// A branch and its parent that both retitled node 1 differently and both
/// retitled node 2 the same way.
async fn conflicted(
    mere: &MereSessions<MemoryBackend>,
) -> (GraphSession<MemoryBackend>, GraphSession<MemoryBackend>) {
    let mut target = parent(mere).await;
    let mut branch = branch_of(mere, &target).await;
    branch
        .apply(
            other(),
            vec![retitle(id(1), "Branch"), retitle(id(2), "Same")],
        )
        .await
        .unwrap();
    target
        .apply(
            person(),
            vec![retitle(id(1), "Target"), retitle(id(2), "Same")],
        )
        .await
        .unwrap();
    (target, branch)
}

#[test]
fn a_conflict_waits_for_a_choice_and_takes_either_side_or_a_hand_edit() {
    pollster::block_on(async {
        for (side, want) in [
            (MergeChoice::Source, "Branch"),
            (MergeChoice::Target, "Target"),
            (MergeChoice::Hand(Value::from("By hand")), "By hand"),
        ] {
            let mere = MereSessions::new(MemoryBackend::new());
            let (mut target, mut branch) = conflicted(&mere).await;
            let plan = mere.merge_in(&target, &branch).await.unwrap();
            assert_eq!(
                plan.conflicts(),
                [MergeConflict {
                    part: Part::Title(id(1)),
                    base: Some(Value::from("One")),
                    source: Some(Value::from("Branch")),
                    target: Some(Value::from("Target")),
                    add_as_new: false,
                }],
                "an equal change on both sides is no conflict"
            );
            assert!(matches!(
                mere.apply_merge(
                    &mut target,
                    &mut branch,
                    plan.clone(),
                    &BTreeMap::new(),
                    person()
                )
                .await,
                Err(SessionError::Unresolved(1))
            ));
            assert!(
                branch.manifest().merged.is_none(),
                "a refusal writes nothing"
            );
            let unfit = BTreeMap::from([(Part::Title(id(1)), MergeChoice::Hand(Value::from(7)))]);
            assert!(matches!(
                mere.apply_merge(&mut target, &mut branch, plan.clone(), &unfit, person())
                    .await,
                Err(SessionError::NotMergeable(_))
            ));
            let not_a_pick = BTreeMap::from([(Part::Title(id(1)), MergeChoice::AddAsNew)]);
            assert!(matches!(
                mere.apply_merge(
                    &mut target,
                    &mut branch,
                    plan.clone(),
                    &not_a_pick,
                    person()
                )
                .await,
                Err(SessionError::NotMergeable(_))
            ));
            let choices = BTreeMap::from([(Part::Title(id(1)), side)]);
            mere.apply_merge(&mut target, &mut branch, plan, &choices, person())
                .await
                .unwrap();
            assert_eq!(title(target.graph(), id(1)).as_deref(), Some(want));
            assert_eq!(title(target.graph(), id(2)).as_deref(), Some("Same"));
        }
    });
}

#[test]
fn a_plan_goes_stale_when_either_side_moves() {
    pollster::block_on(async {
        let mere = MereSessions::new(MemoryBackend::new());
        let (mut target, mut branch) = conflicted(&mere).await;
        let plan = mere.merge_in(&target, &branch).await.unwrap();
        target
            .apply(person(), vec![tag(id(2), "later")])
            .await
            .unwrap();
        let choices = BTreeMap::from([(Part::Title(id(1)), MergeChoice::Source)]);
        assert!(matches!(
            mere.apply_merge(&mut target, &mut branch, plan, &choices, person())
                .await,
            Err(SessionError::NotMergeable(_))
        ));
    });
}

#[test]
fn deletions_merge_three_way_against_the_base() {
    pollster::block_on(async {
        // The branch deletes 2 while the parent retitles it: delete against
        // edit. And the parent deletes 3 while the branch tags it.
        for side in [MergeChoice::Source, MergeChoice::Target] {
            let mere = MereSessions::new(MemoryBackend::new());
            let mut target = parent(&mere).await;
            let mut branch = branch_of(&mere, &target).await;
            branch
                .apply(
                    other(),
                    vec![remove(id(2)), tag(id(3), "wanted"), relate(id(1), id(3))],
                )
                .await
                .unwrap();
            target
                .apply(person(), vec![retitle(id(2), "Edited"), remove(id(3))])
                .await
                .unwrap();
            let plan = mere.merge_in(&target, &branch).await.unwrap();
            let parts: Vec<&Part> = plan.conflicts().iter().map(|c| &c.part).collect();
            assert!(parts.contains(&&Part::Node(id(2))), "{parts:?}");
            assert!(parts.contains(&&Part::Node(id(3))), "{parts:?}");
            let deleted = plan
                .conflicts()
                .iter()
                .find(|conflict| conflict.part == Part::Node(id(2)))
                .unwrap();
            assert!(deleted.source.is_none() && deleted.target.is_some());
            assert_eq!(
                parts.len(),
                2,
                "relations of a node in conflict follow its choice: {parts:?}"
            );
            let choices: BTreeMap<Part, MergeChoice> = plan
                .conflicts()
                .iter()
                .map(|conflict| (conflict.part.clone(), side.clone()))
                .collect();
            mere.apply_merge(&mut target, &mut branch, plan, &choices, person())
                .await
                .unwrap();
            let graph = target.graph();
            match side {
                MergeChoice::Source => {
                    assert!(graph.get_node_by_id(id(2)).is_none(), "the deletion wins");
                    let (_, node) = graph.get_node_by_id(id(3)).expect("resurrected");
                    assert!(node.tags.contains("wanted"));
                    assert!(related(graph, id(1), id(3)), "with its relations");
                },
                MergeChoice::Target => {
                    assert_eq!(title(graph, id(2)).as_deref(), Some("Edited"));
                    assert!(
                        related(graph, id(1), id(2)),
                        "a node kept keeps its relations"
                    );
                    assert!(graph.get_node_by_id(id(3)).is_none(), "stays deleted");
                },
                _ => unreachable!(),
            }
        }

        // A deletion the target left alone merges without a question.
        let mere = MereSessions::new(MemoryBackend::new());
        let mut target = parent(&mere).await;
        let mut branch = branch_of(&mere, &target).await;
        branch.apply(other(), vec![remove(id(2))]).await.unwrap();
        let plan = mere.merge_in(&target, &branch).await.unwrap();
        assert!(plan.is_clean(), "{:?}", plan.conflicts());
        mere.apply_merge(&mut target, &mut branch, plan, &BTreeMap::new(), person())
            .await
            .unwrap();
        assert!(target.graph().get_node_by_id(id(2)).is_none());
        assert!(!related(target.graph(), id(1), id(2)));
    });
}

#[test]
fn a_picked_off_part_merges_back_by_derivation() {
    pollster::block_on(async {
        let mere = MereSessions::new(MemoryBackend::new());
        let mut target = parent(&mere).await;
        let container = *target.manifest().root_graph_id.as_uuid();
        let part = mere.fork_component(&target, id(1), person()).await.unwrap();
        let mut part = mere.open(part.session_id).await.unwrap();
        let (one, two) = (
            copy_of(part.graph(), id(1), container),
            copy_of(part.graph(), id(2), container),
        );
        let fresh = id(77);
        part.apply(
            other(),
            vec![
                retitle(one, "One, from the part"),
                add_id(fresh),
                relate(two, fresh),
            ],
        )
        .await
        .unwrap();
        let facets_before = target
            .graph()
            .facets()
            .facets_of(&id(1))
            .map(|facets| {
                facets
                    .iter()
                    .map(|(facet, value)| (facet.as_str().to_string(), value.clone()))
                    .collect::<BTreeMap<_, _>>()
            })
            .unwrap_or_default();

        let plan = mere.merge_in(&target, &part).await.unwrap();
        assert_eq!(plan.matched(), MergeMatch::Pick);
        assert!(plan.is_clean(), "{:?}", plan.conflicts());
        mere.apply_merge(&mut target, &mut part, plan, &BTreeMap::new(), person())
            .await
            .unwrap();

        let graph = target.graph();
        assert_eq!(title(graph, id(1)).as_deref(), Some("One, from the part"));
        let part_container = part.manifest().root_graph_id.as_uuid().to_string();
        assert_eq!(
            merged_from(graph, id(1)),
            vec![NodeDerivation {
                sub_kind: ProvenanceSubKind::MergedFrom,
                source_node: one.to_string(),
                source_graph: Some(part_container.clone()),
            }]
        );
        // The copy's own provenance and visit stamps are not news to the target.
        let facets_after: BTreeMap<String, Value> = graph
            .facets()
            .facets_of(&id(1))
            .unwrap()
            .iter()
            .filter(|(facet, _)| {
                facet.as_str() != kernel::graph::node_facets::PROVENANCE_DERIVATIONS
            })
            .map(|(facet, value)| (facet.as_str().to_string(), value.clone()))
            .collect();
        let mut facets_before = facets_before;
        facets_before.remove(kernel::graph::node_facets::PROVENANCE_DERIVATIONS);
        assert_eq!(facets_after, facets_before);
        assert!(
            derivations(graph, id(1))
                .iter()
                .all(|derivation| derivation.sub_kind != ProvenanceSubKind::CopiedFrom),
            "the target is not derived from its own copy"
        );
        // The part's new node arrives under a new id, linked back to it.
        let added = graph
            .nodes()
            .map(|(_, node)| node.id)
            .find(|node| {
                merged_from(graph, *node)
                    .iter()
                    .any(|derivation| derivation.source_node == fresh.to_string())
            })
            .expect("the part's new node");
        assert_ne!(added, fresh, "a picked part's new node gets a new id");
        assert!(related(graph, id(2), added));
        assert_eq!(graph.node_count(), 4, "1, 2, 3 and the new one");
        assert!(part.manifest().merged.is_some());
    });
}

#[test]
fn a_pick_of_a_pick_merges_into_the_original() {
    pollster::block_on(async {
        let mere = MereSessions::new(MemoryBackend::new());
        let mut original = parent(&mere).await;
        let container = *original.manifest().root_graph_id.as_uuid();
        let first = mere
            .fork_component(&original, id(1), person())
            .await
            .unwrap();
        let mut first = mere.open(first.session_id).await.unwrap();
        let first_container = *first.manifest().root_graph_id.as_uuid();
        let one = copy_of(first.graph(), id(1), container);
        // The part between changes what the second part will hold before it
        // is picked: a retitle and a node of its own.
        let made = id(55);
        first
            .apply(
                other(),
                vec![
                    retitle(copy_of(first.graph(), id(2), container), "Retitled between"),
                    add_id(made),
                    retitle(made, "Made between"),
                    relate(one, made),
                ],
            )
            .await
            .unwrap();
        let second = mere.fork_component(&first, one, person()).await.unwrap();
        let mut second = mere.open(second.session_id).await.unwrap();
        let one_again = copy_of(second.graph(), one, first_container);
        let two_again = copy_of(
            second.graph(),
            copy_of(first.graph(), id(2), container),
            first_container,
        );
        let _ = two_again;
        second
            .apply(other(), vec![retitle(one_again, "Twice picked")])
            .await
            .unwrap();
        original
            .apply(person(), vec![tag(id(1), "original's own")])
            .await
            .unwrap();

        let plan = mere.merge_in(&original, &second).await.unwrap();
        assert_eq!(plan.matched(), MergeMatch::Pick);
        assert!(plan.is_clean(), "{:?}", plan.conflicts());
        mere.apply_merge(&mut original, &mut second, plan, &BTreeMap::new(), person())
            .await
            .unwrap();
        let graph = original.graph();
        assert_eq!(title(graph, id(1)).as_deref(), Some("Twice picked"));
        assert_eq!(
            title(graph, id(2)).as_deref(),
            Some("Retitled between"),
            "everything since the first pick comes back"
        );
        let between = graph
            .nodes()
            .find(|(_, node)| node.title == "Made between")
            .map(|(_, node)| node.id)
            .expect("the node made between");
        assert_ne!(between, made);
        assert!(related(graph, id(1), between));
        assert!(
            graph
                .get_node_by_id(id(1))
                .unwrap()
                .1
                .tags
                .contains("original's own")
        );
        assert!(graph.get_node_by_id(id(3)).is_some());
        assert_eq!(
            merged_from(graph, id(1))[0].source_node,
            one_again.to_string()
        );
    });
}

#[test]
fn a_part_picked_before_the_pick_record_merges_as_a_pick() {
    pollster::block_on(async {
        let store = MemoryBackend::new();
        let mere = MereSessions::new(store.clone());
        let mut target = parent(&mere).await;
        let container = *target.manifest().root_graph_id.as_uuid();
        let part = mere.fork_component(&target, id(1), person()).await.unwrap();
        // Rewrite its manifest as parts were written before `picked_from`.
        let mut legacy = part.clone();
        let picked = legacy.picked_from.take().unwrap();
        legacy.parent_session = Some(picked.session);
        legacy.forked_at = Some(picked.at);
        store
            .apply(&[pretty(Keys::new(legacy.session_id).at(MANIFEST), &legacy).unwrap()])
            .await
            .unwrap();
        let mut part = mere.open(part.session_id).await.unwrap();
        assert_eq!(part.manifest().picked_from, None);
        let one = copy_of(part.graph(), id(1), container);
        part.apply(other(), vec![retitle(one, "Legacy part")])
            .await
            .unwrap();
        let plan = mere.merge_in(&target, &part).await.unwrap();
        assert_eq!(plan.matched(), MergeMatch::Pick);
        mere.apply_merge(&mut target, &mut part, plan, &BTreeMap::new(), person())
            .await
            .unwrap();
        assert_eq!(title(target.graph(), id(1)).as_deref(), Some("Legacy part"));
    });
}

#[test]
fn merge_in_and_compose_join_resource_facets_by_one_rule() {
    // The same two records: Compose unions the properties and refuses the
    // rule-less facet; merge-in (above) joins the properties and asks.
    let iri = "https://resource.test/a";
    let record = |handles: &[(&str, &str)], review: &str| {
        let CapturedDelta::ReplaySetResourceRecordById {
            record: Some(record),
            ..
        } = properties_record(
            iri,
            &[
                (
                    RESOURCE_PROPERTIES,
                    Value::from(
                        handles
                            .iter()
                            .map(|(handle, value)| property(handle, value))
                            .collect::<Vec<_>>(),
                    ),
                ),
                ("test.review", serde_json::json!({ "status": review })),
            ],
        )
        else {
            unreachable!()
        };
        record
    };
    let mut ours = record(&[("p1", "a"), ("p3", "c")], "open");
    let theirs = record(&[("p1", "a"), ("p2", "b")], "open");
    crate::merge_rules::layer_resource_record(&mut ours, &theirs).unwrap();
    let joined: Value = serde_json::from_str(
        &ours
            .facets
            .iter()
            .find(|facet| facet.facet == RESOURCE_PROPERTIES)
            .unwrap()
            .value_json,
    )
    .unwrap();
    assert_eq!(
        joined
            .as_array()
            .unwrap()
            .iter()
            .map(|property| property["statement_id"].as_str().unwrap())
            .collect::<Vec<_>>(),
        ["p1", "p2", "p3"]
    );
    let mut ours = record(&[("p1", "a")], "accepted");
    let error =
        crate::merge_rules::layer_resource_record(&mut ours, &record(&[], "rejected")).unwrap_err();
    assert!(error.0.contains("test.review") && error.0.contains("conflicting"));
}

#[test]
fn redoing_an_undone_merge_marks_the_source_again() {
    pollster::block_on(async {
        let mere = MereSessions::new(MemoryBackend::new());
        let mut target = parent(&mere).await;
        let mut branch = branch_of(&mere, &target).await;
        branch
            .apply(other(), vec![retitle(id(1), "Branch")])
            .await
            .unwrap();
        let plan = mere.merge_in(&target, &branch).await.unwrap();
        mere.apply_merge(&mut target, &mut branch, plan, &BTreeMap::new(), person())
            .await
            .unwrap();
        // A host that writes its own batches: undo_now, then pending/stored.
        target.undo_now(person()).unwrap().unwrap();
        let batch = target.pending(wall_clock_now()).unwrap();
        assert!(
            batch
                .ops()
                .iter()
                .any(|op| matches!(op, WriteOp::Delete { key } if key.ends_with("/merged.json")))
        );
        mere.backend.apply(batch.ops()).await.unwrap();
        target.stored(batch);
        assert!(!target.has_unstored());
        assert_eq!(
            mere.open(branch.id()).await.unwrap().manifest().merged,
            None
        );
        target.redo(person()).await.unwrap().unwrap();
        assert_eq!(title(target.graph(), id(1)).as_deref(), Some("Branch"));
        let mark = mere
            .open(branch.id())
            .await
            .unwrap()
            .manifest()
            .merged
            .clone();
        assert_eq!(mark.map(|mark| mark.into), Some(target.id()));
    });
}

// Exercise the transaction boundary, not just the held handle's cached
// `manifest.merged`: target redo persists merged.json without advancing the
// source logs, so a handle opened after undo must refuse atomically.
#[test]
fn source_transaction_refuses_after_another_handle_redoes_merge() {
    pollster::block_on(async {
        let store = MemoryBackend::new();
        let mere = MereSessions::new(store.clone());
        let mut target = parent(&mere).await;
        let mut branch = branch_of(&mere, &target).await;
        branch
            .apply(other(), vec![retitle(id(1), "Branch")])
            .await
            .unwrap();
        // `branch_at` currently creates an unqualified copy regardless of
        // its parent's placement. Reopen the durable source, qualify its full
        // journal through the supported legacy receipt path, and prove the
        // fixture can prepare an addressable publication before merging it.
        let mut branch = mere.open(branch.id()).await.unwrap();
        let cutoff = branch.journal().live_cursor();
        branch.qualify_legacy(cutoff, vec![]).await.unwrap();
        assert!(branch.prepare_resident_publication().is_ok());
        let plan = mere.merge_in(&target, &branch).await.unwrap();
        mere.apply_merge(&mut target, &mut branch, plan, &BTreeMap::new(), person())
            .await
            .unwrap();

        // Persist target undo, which removes merged.json from the source.
        target.undo_now(person()).unwrap().unwrap();
        let undone = target.pending(wall_clock_now()).unwrap();
        assert!(
            undone
                .ops()
                .iter()
                .any(|op| matches!(op, WriteOp::Delete { key } if key.ends_with("/merged.json")))
        );
        mere.backend.apply(undone.ops()).await.unwrap();
        target.stored(undone);
        let mut held = mere.open(branch.id()).await.unwrap();
        assert!(held.manifest().merged.is_none());
        let source_title_before = title(held.graph(), id(1));
        let journal_before = held.journal().live_cursor();
        let changes_before = held.changes().len();

        // Redo writes the marker to the backend, without updating `held`.
        target.redo(person()).await.unwrap().unwrap();
        let reopened_after_redo = mere.open(branch.id()).await.unwrap();
        assert_eq!(
            reopened_after_redo
                .manifest()
                .merged
                .as_ref()
                .map(|mark| mark.into),
            Some(target.id())
        );
        assert!(held.manifest().merged.is_none());
        let durable_before_refusal = backend_snapshot(&store).await;

        let key = held.graph().get_node_by_id(id(1)).unwrap().0;
        let result = held
            .transact_deltas(
                person(),
                vec![GraphDelta::SetNodeTitle {
                    key,
                    title: "stale write".into(),
                }],
                wall_clock_now(),
            )
            .await;
        assert!(
            matches!(result, Err(SessionError::ReadOnly(session)) if session == held.id()),
            "a durably merged source must reject transact_deltas; got {result:?}"
        );
        assert_eq!(
            backend_snapshot(&store).await,
            durable_before_refusal,
            "refused stale write must leave every persisted key and byte unchanged"
        );
        let published_result = held
            .transact_deltas_published(
                person(),
                vec![GraphDelta::SetNodeTitle {
                    key,
                    title: "stale published write".into(),
                }],
                wall_clock_now(),
            )
            .await;
        assert!(
            matches!(published_result, Err(SessionError::ReadOnly(session)) if session == held.id()),
            "a durably merged source must reject published transactions; got {published_result:?}"
        );
        assert_eq!(
            backend_snapshot(&store).await,
            durable_before_refusal,
            "refused published transaction must leave every persisted key and byte unchanged"
        );

        // Refusal is atomic from both the caller's handle and durable reopen:
        // no candidate graph or history entries become visible.
        assert_eq!(title(held.graph(), id(1)), source_title_before);
        assert_eq!(held.journal().live_cursor(), journal_before);
        assert_eq!(held.changes().len(), changes_before);
        let reopened_after_refusal = mere.open(branch.id()).await.unwrap();
        assert_eq!(
            title(reopened_after_refusal.graph(), id(1)),
            source_title_before
        );
        assert_eq!(
            reopened_after_refusal.journal().live_cursor(),
            journal_before
        );
        assert_eq!(reopened_after_refusal.changes().len(), changes_before);
        assert_eq!(
            reopened_after_refusal
                .manifest()
                .merged
                .as_ref()
                .map(|mark| mark.into),
            Some(target.id())
        );
    });
}

// Positive control: after undo removes the durable marker, the held source
// handle is legitimately writable and a real transaction persists its edit.
#[test]
fn source_transaction_after_undo_without_redo_is_allowed() {
    pollster::block_on(async {
        let mere = MereSessions::new(MemoryBackend::new());
        let mut target = parent(&mere).await;
        let mut branch = branch_of(&mere, &target).await;
        branch
            .apply(other(), vec![retitle(id(1), "Branch")])
            .await
            .unwrap();
        let plan = mere.merge_in(&target, &branch).await.unwrap();
        mere.apply_merge(&mut target, &mut branch, plan, &BTreeMap::new(), person())
            .await
            .unwrap();

        target.undo_now(person()).unwrap().unwrap();
        let undone = target.pending(wall_clock_now()).unwrap();
        mere.backend.apply(undone.ops()).await.unwrap();
        target.stored(undone);
        let mut held = mere.open(branch.id()).await.unwrap();
        assert!(held.manifest().merged.is_none());
        let key = held.graph().get_node_by_id(id(1)).unwrap().0;
        held.transact_deltas(
            person(),
            vec![GraphDelta::SetNodeTitle {
                key,
                title: "undo control write".into(),
            }],
            wall_clock_now(),
        )
        .await
        .unwrap();
        let reopened = mere.open(branch.id()).await.unwrap();
        assert_eq!(
            title(reopened.graph(), id(1)).as_deref(),
            Some("undo control write")
        );
        assert!(reopened.manifest().merged.is_none());
    });
}

// A fresh open sees the marker written by redo and refuses through the same
// actual transaction API; this separates stale-handle admission from normal
// read-only enforcement.
#[test]
fn freshly_opened_source_transaction_after_redo_is_refused() {
    pollster::block_on(async {
        let mere = MereSessions::new(MemoryBackend::new());
        let mut target = parent(&mere).await;
        let mut branch = branch_of(&mere, &target).await;
        branch
            .apply(other(), vec![retitle(id(1), "Branch")])
            .await
            .unwrap();
        let plan = mere.merge_in(&target, &branch).await.unwrap();
        mere.apply_merge(&mut target, &mut branch, plan, &BTreeMap::new(), person())
            .await
            .unwrap();

        target.undo_now(person()).unwrap().unwrap();
        let undone = target.pending(wall_clock_now()).unwrap();
        mere.backend.apply(undone.ops()).await.unwrap();
        target.stored(undone);
        target.redo(person()).await.unwrap().unwrap();

        let mut fresh = mere.open(branch.id()).await.unwrap();
        assert_eq!(
            fresh.manifest().merged.as_ref().map(|mark| mark.into),
            Some(target.id())
        );
        let key = fresh.graph().get_node_by_id(id(1)).unwrap().0;
        assert!(matches!(
            fresh
                .transact_deltas(
                    person(),
                    vec![GraphDelta::SetNodeTitle {
                        key,
                        title: "fresh stale write".into(),
                    }],
                    wall_clock_now(),
                )
                .await,
            Err(SessionError::ReadOnly(session)) if session == fresh.id()
        ));
    });
}

// Reload control for a long-lived source handle: an explicit marker refresh
// restores the existing cached read-only policy after target redo.
#[test]
fn reloading_source_merged_mark_after_redo_restores_read_only_policy() {
    pollster::block_on(async {
        let mere = MereSessions::new(MemoryBackend::new());
        let mut target = parent(&mere).await;
        let mut branch = branch_of(&mere, &target).await;
        branch
            .apply(other(), vec![retitle(id(1), "Branch")])
            .await
            .unwrap();
        let plan = mere.merge_in(&target, &branch).await.unwrap();
        mere.apply_merge(&mut target, &mut branch, plan, &BTreeMap::new(), person())
            .await
            .unwrap();

        target.undo_now(person()).unwrap().unwrap();
        let undone = target.pending(wall_clock_now()).unwrap();
        mere.backend.apply(undone.ops()).await.unwrap();
        target.stored(undone);
        let mut held = mere.open(branch.id()).await.unwrap();
        target.redo(person()).await.unwrap().unwrap();
        assert!(held.manifest().merged.is_none());
        held.reload_merged_mark().await.unwrap();
        assert_eq!(
            held.manifest().merged.as_ref().map(|mark| mark.into),
            Some(target.id())
        );
        assert!(held.ensure_writable().is_err());
    });
}

#[test]
fn a_picked_node_the_target_deleted_can_be_added_as_new() {
    pollster::block_on(async {
        for choice in [
            MergeChoice::AddAsNew,
            MergeChoice::Source,
            MergeChoice::Target,
        ] {
            let mere = MereSessions::new(MemoryBackend::new());
            let mut target = parent(&mere).await;
            let container = *target.manifest().root_graph_id.as_uuid();
            let part = mere.fork_component(&target, id(1), person()).await.unwrap();
            let mut part = mere.open(part.session_id).await.unwrap();
            let two = copy_of(part.graph(), id(2), container);
            part.apply(other(), vec![retitle(two, "Kept in the part")])
                .await
                .unwrap();
            target.apply(person(), vec![remove(id(2))]).await.unwrap();
            let plan = mere.merge_in(&target, &part).await.unwrap();
            assert_eq!(plan.conflicts().len(), 1, "{:?}", plan.conflicts());
            let conflict = &plan.conflicts()[0];
            assert_eq!(conflict.part, Part::Node(id(2)));
            assert!(conflict.add_as_new, "the vanished source offers add as new");
            let choices = BTreeMap::from([(Part::Node(id(2)), choice.clone())]);
            mere.apply_merge(&mut target, &mut part, plan, &choices, person())
                .await
                .unwrap();
            let graph = target.graph();
            let titled = graph
                .nodes()
                .find(|(_, node)| node.title == "Kept in the part")
                .map(|(_, node)| node.id);
            match choice {
                MergeChoice::AddAsNew => {
                    let fresh = titled.expect("added as new");
                    assert_ne!(fresh, id(2));
                    assert!(graph.get_node_by_id(id(2)).is_none(), "the deletion stands");
                    assert!(related(graph, id(1), fresh), "with its relations");
                    assert_eq!(merged_from(graph, fresh)[0].source_node, two.to_string());
                },
                MergeChoice::Source => {
                    assert_eq!(titled, Some(id(2)), "resurrected under its own id");
                    assert!(related(graph, id(1), id(2)), "with its relations");
                },
                _ => assert_eq!(titled, None, "the deletion stands"),
            }
        }
    });
}

/// The parent, a part picked off node 1, and the part's copy of node 1.
async fn picked_one(
    mere: &MereSessions<MemoryBackend>,
) -> (
    GraphSession<MemoryBackend>,
    GraphSession<MemoryBackend>,
    Uuid,
) {
    let target = parent(mere).await;
    let container = *target.manifest().root_graph_id.as_uuid();
    let part = mere.fork_component(&target, id(1), person()).await.unwrap();
    let part = mere.open(part.session_id).await.unwrap();
    let copy = copy_of(part.graph(), id(1), container);
    (target, part, copy)
}

async fn merge_clean(
    mere: &MereSessions<MemoryBackend>,
    target: &mut GraphSession<MemoryBackend>,
    part: &mut GraphSession<MemoryBackend>,
) {
    let plan = mere.merge_in(target, part).await.unwrap();
    assert!(plan.is_clean(), "{:?}", plan.conflicts());
    mere.apply_merge(target, part, plan, &BTreeMap::new(), person())
        .await
        .unwrap();
}

#[test]
fn merging_a_pick_keeps_a_title_that_equals_a_node_id() {
    pollster::block_on(async {
        let mere = MereSessions::new(MemoryBackend::new());
        let (mut target, mut part, copy) = picked_one(&mere).await;
        // Text that reads as the id the merge maps the copy to its original from.
        let text = copy.to_string();
        part.apply(other(), vec![retitle(copy, &text)])
            .await
            .unwrap();

        merge_clean(&mere, &mut target, &mut part).await;

        assert_eq!(title(target.graph(), id(1)).as_deref(), Some(text.as_str()));
        assert_eq!(title(part.graph(), copy).as_deref(), Some(text.as_str()));
    });
}

#[test]
fn a_facet_value_naming_an_id_is_carried_verbatim_through_a_pick_merge() {
    pollster::block_on(async {
        let mere = MereSessions::new(MemoryBackend::new());
        let (mut target, mut part, copy) = picked_one(&mere).await;
        let facet = FacetId::new("extension.note");
        let value = serde_json::json!({ "about": copy.to_string() });
        part.apply(
            other(),
            vec![CapturedDelta::ReplaySetNodeFacetById {
                node_id: copy.to_string(),
                facet: facet.as_str().to_string(),
                value_json: value.to_string(),
            }],
        )
        .await
        .unwrap();

        merge_clean(&mere, &mut target, &mut part).await;

        assert_eq!(target.graph().facets().get(&id(1), &facet), Some(&value));
        assert_eq!(part.graph().facets().get(&copy, &facet), Some(&value));
    });
}

fn properties_record(iri: &str, facets: &[(&str, Value)]) -> CapturedDelta {
    CapturedDelta::ReplaySetResourceRecordById {
        resource_id: chartulary::resource_id(iri).to_string(),
        record: Some(PersistedResourceRecord {
            canonical_iri: chartulary::canonical_url(iri),
            facets: facets
                .iter()
                .map(|(facet, value)| PersistedResourceFacet {
                    facet: (*facet).into(),
                    value_json: value.to_string(),
                })
                .collect(),
        }),
    }
}

fn property(handle: &str, value: &str) -> Value {
    serde_json::json!({
        "statement_id": handle,
        "predicate": "https://schema.org/name",
        "value": value,
    })
}

#[test]
fn resource_facets_merge_by_the_rules_compose_uses() {
    pollster::block_on(async {
        let iri = "https://resource.test/a";
        let resource = chartulary::resource_id(iri);
        let mere = MereSessions::new(MemoryBackend::new());
        let mut target = parent(&mere).await;
        target
            .apply(
                person(),
                vec![properties_record(
                    iri,
                    &[
                        (RESOURCE_PROPERTIES, Value::from(vec![property("p1", "a")])),
                        ("test.review", serde_json::json!({"status": "open"})),
                    ],
                )],
            )
            .await
            .unwrap();
        let mut branch = branch_of(&mere, &target).await;
        branch
            .apply(
                other(),
                vec![properties_record(
                    iri,
                    &[
                        (
                            RESOURCE_PROPERTIES,
                            Value::from(vec![property("p1", "a"), property("p2", "b")]),
                        ),
                        ("test.review", serde_json::json!({"status": "accepted"})),
                    ],
                )],
            )
            .await
            .unwrap();
        target
            .apply(
                person(),
                vec![properties_record(
                    iri,
                    &[
                        (
                            RESOURCE_PROPERTIES,
                            Value::from(vec![property("p1", "a"), property("p3", "c")]),
                        ),
                        ("test.review", serde_json::json!({"status": "rejected"})),
                    ],
                )],
            )
            .await
            .unwrap();
        let plan = mere.merge_in(&target, &branch).await.unwrap();
        assert_eq!(
            plan.conflicts()
                .iter()
                .map(|conflict| conflict.part.clone())
                .collect::<Vec<_>>(),
            [Part::ResourceFacet(resource, "test.review".into())],
            "properties join by their rule; a facet with none conflicts"
        );
        let choices = BTreeMap::from([(
            Part::ResourceFacet(resource, "test.review".into()),
            MergeChoice::Source,
        )]);
        mere.apply_merge(&mut target, &mut branch, plan, &choices, person())
            .await
            .unwrap();
        let record = target.graph().resource_record(resource).unwrap();
        let facet = |name: &str| -> Value {
            serde_json::from_str(
                &record
                    .facets
                    .iter()
                    .find(|facet| facet.facet == name)
                    .unwrap()
                    .value_json,
            )
            .unwrap()
        };
        let handles: Vec<String> = facet(RESOURCE_PROPERTIES)
            .as_array()
            .unwrap()
            .iter()
            .map(|property| property["statement_id"].as_str().unwrap().to_string())
            .collect();
        assert_eq!(handles, ["p1", "p2", "p3"]);
        assert_eq!(
            facet("test.review"),
            serde_json::json!({"status": "accepted"})
        );
    });
}

#[test]
fn a_new_plan_carries_the_choices_that_still_apply_and_says_why_others_dropped() {
    pollster::block_on(async {
        let mere = MereSessions::new(MemoryBackend::new());
        let mut target = parent(&mere).await;
        let mut branch = branch_of(&mere, &target).await;
        branch
            .apply(
                other(),
                vec![
                    retitle(id(1), "Branch"),
                    retitle(id(2), "B2"),
                    retitle(id(3), "B3"),
                ],
            )
            .await
            .unwrap();
        target
            .apply(
                person(),
                vec![
                    retitle(id(1), "Target"),
                    retitle(id(2), "T2"),
                    retitle(id(3), "T3"),
                ],
            )
            .await
            .unwrap();
        let plan = mere.merge_in(&target, &branch).await.unwrap();
        assert_eq!(plan.conflicts().len(), 3);
        let choices = BTreeMap::from([
            (Part::Title(id(1)), MergeChoice::Source),
            (Part::Title(id(2)), MergeChoice::Target),
            (Part::Title(id(3)), MergeChoice::Hand(Value::from(7))),
            (Part::Title(id(4)), MergeChoice::AddAsNew),
        ]);
        // The target moves before Apply: it now agrees with the branch on 2.
        target
            .apply(person(), vec![retitle(id(2), "B2")])
            .await
            .unwrap();
        assert!(matches!(
            mere.apply_merge(&mut target, &mut branch, plan, &choices, person())
                .await,
            Err(SessionError::NotMergeable(_))
        ));
        let again = mere.merge_in(&target, &branch).await.unwrap();
        let (carried, dropped) = again.carry_choices(&choices);
        assert_eq!(
            carried,
            BTreeMap::from([(Part::Title(id(1)), MergeChoice::Source)])
        );
        let why: BTreeMap<Part, &str> = dropped
            .iter()
            .map(|dropped| (dropped.part.clone(), dropped.reason.as_str()))
            .collect();
        assert_eq!(
            why,
            BTreeMap::from([
                (Part::Title(id(2)), "no longer in conflict"),
                (Part::Title(id(3)), "the value given by hand does not fit"),
                (Part::Title(id(4)), "no longer in conflict"),
            ])
        );
        // With the carried choice and a fresh one, the new plan applies.
        let mut choices = carried;
        choices.insert(
            Part::Title(id(3)),
            MergeChoice::Hand(Value::from("By hand")),
        );
        mere.apply_merge(&mut target, &mut branch, again, &choices, person())
            .await
            .unwrap();
        assert_eq!(title(target.graph(), id(1)).as_deref(), Some("Branch"));
        assert_eq!(title(target.graph(), id(3)).as_deref(), Some("By hand"));
    });
}

#[test]
fn operation_receipt_reconciles_duplicates_and_refuses_changed_request() {
    pollster::block_on(async {
        let mere = MereSessions::new(MemoryBackend::new());
        let mut first = parent(&mere).await;
        let mut competing = mere.open(first.id()).await.unwrap();
        let before = first.graph().node_count();
        let operation = [17; 32];
        let make_deltas = || {
            vec![
                GraphDelta::AddNode {
                    id: Some(id(77)),
                    url: "https://brief.test/77".into(),
                    position: euclid::point2(0.0, 0.0),
                },
                GraphDelta::ReplaySetNodeBodyById {
                    node_id: id(77),
                    body: Some("brief".into()),
                },
            ]
        };
        let saved = first
            .transact_operation_published(
                operation,
                b"request".to_vec(),
                b"note".to_vec(),
                person(),
                make_deltas(),
                wall_clock_now(),
            )
            .await
            .unwrap();
        let duplicate = competing
            .transact_operation_published(
                operation,
                b"request".to_vec(),
                b"note".to_vec(),
                person(),
                make_deltas(),
                wall_clock_now(),
            )
            .await
            .unwrap();
        assert_eq!(saved, duplicate);
        assert_eq!(
            competing.graph().node_count(),
            before,
            "duplicate must not install a speculative candidate"
        );
        assert!(
            competing
                .transact_operation_published(
                    operation,
                    b"changed".to_vec(),
                    vec![],
                    person(),
                    make_deltas(),
                    wall_clock_now()
                )
                .await
                .is_err()
        );
        assert!(
            competing
                .transact_operation_published(
                    operation,
                    b"request".to_vec(),
                    b"other result".to_vec(),
                    person(),
                    make_deltas(),
                    wall_clock_now()
                )
                .await
                .is_err()
        );
        let reopened = mere.open(first.id()).await.unwrap();
        assert_eq!(
            reopened.operation_receipt(operation).await.unwrap(),
            Some(saved)
        );
        let (key, _) = reopened.graph().get_node_by_id(id(77)).unwrap();
        let snapshot = reopened.graph().to_snapshot();
        let node = snapshot
            .nodes
            .iter()
            .find(|n| n.node_id == id(77).to_string())
            .unwrap();
        assert_eq!(node.body.as_deref(), Some("brief"));
        assert!(reopened.graph().shown_resource_id(key).is_some());
        assert_eq!(reopened.graph().node_count(), before + 1);
    });
}

#[test]
fn operation_refuses_stale_graph_without_installing_candidate() {
    pollster::block_on(async {
        let mere = MereSessions::new(MemoryBackend::new());
        let mut first = parent(&mere).await;
        let mut stale = mere.open(first.id()).await.unwrap();
        let key = first.graph().get_node_by_id(id(1)).unwrap().0;
        first
            .transact_deltas_published(
                person(),
                vec![GraphDelta::SetNodeTitle {
                    key,
                    title: "new head".into(),
                }],
                wall_clock_now(),
            )
            .await
            .unwrap();
        let stale_key = stale.graph().get_node_by_id(id(1)).unwrap().0;
        let result = stale
            .transact_operation_published(
                [18; 32],
                b"request".to_vec(),
                b"result".to_vec(),
                person(),
                vec![GraphDelta::SetNodeTitle {
                    key: stale_key,
                    title: "refused".into(),
                }],
                wall_clock_now(),
            )
            .await;
        assert!(result.is_err());
        assert_eq!(title(stale.graph(), id(1)).as_deref(), Some("One"));
        assert!(stale.operation_receipt([18; 32]).await.unwrap().is_none());
        assert_eq!(
            title(mere.open(first.id()).await.unwrap().graph(), id(1)).as_deref(),
            Some("new head")
        );
    });
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn concurrent_operation_requests_share_one_receipt() {
    use std::sync::{Arc, Barrier};
    #[derive(Clone)]
    struct RacingBackend {
        store: MemoryBackend,
        barrier: Arc<Barrier>,
    }
    #[async_trait::async_trait]
    impl Backend for RacingBackend {
        async fn get(&self, key: &str) -> Result<Option<Vec<u8>>, StoreError> {
            let value = self.store.get(key).await?;
            if key.starts_with("graph-operations/") {
                self.barrier.wait();
            }
            Ok(value)
        }
        async fn put(&self, key: &str, bytes: &[u8]) -> Result<(), StoreError> {
            self.store.put(key, bytes).await
        }
        async fn delete(&self, key: &str) -> Result<(), StoreError> {
            self.store.delete(key).await
        }
        async fn list(&self, prefix: &str) -> Result<Vec<String>, StoreError> {
            self.store.list(prefix).await
        }
        async fn scan(&self, start: &str, end: &str) -> Result<Vec<String>, StoreError> {
            self.store.scan(start, end).await
        }
        async fn apply(&self, ops: &[WriteOp]) -> Result<(), StoreError> {
            self.store.apply(ops).await
        }
        async fn transact(&self, transaction: muniment::TransactFn) -> Result<(), StoreError> {
            self.store.transact(transaction).await
        }
    }
    let store = MemoryBackend::new();
    let mere = MereSessions::new(store.clone());
    let session = pollster::block_on(parent(&mere)).id();
    let backend = RacingBackend {
        store: store.clone(),
        barrier: Arc::new(Barrier::new(2)),
    };
    let runners: Vec<_> = (0..2)
        .map(|_| {
            let backend = backend.clone();
            std::thread::spawn(move || {
                pollster::block_on(async move {
                    let mut writer = GraphSession::open(backend, session).await.unwrap();
                    writer
                        .transact_operation_published(
                            [19; 32],
                            b"same".to_vec(),
                            b"result".to_vec(),
                            person(),
                            vec![
                                GraphDelta::AddNode {
                                    id: Some(id(78)),
                                    url: "https://brief.test/78".into(),
                                    position: euclid::point2(0.0, 0.0),
                                },
                                GraphDelta::ReplaySetNodeBodyById {
                                    node_id: id(78),
                                    body: Some("brief".into()),
                                },
                            ],
                            wall_clock_now(),
                        )
                        .await
                        .unwrap()
                })
            })
        })
        .collect();
    let receipts: Vec<_> = runners.into_iter().map(|r| r.join().unwrap()).collect();
    assert_eq!(receipts[0], receipts[1]);
    let reopened = pollster::block_on(mere.open(session)).unwrap();
    assert_eq!(reopened.graph().node_count(), 4);
    assert!(reopened.graph().get_node_by_id(id(78)).is_some());
}

#[test]
fn operation_revision_binds_content_and_survives_reopen() {
    pollster::block_on(async {
        let one_store = MemoryBackend::new();
        let one = MereSessions::new(one_store.clone());
        let mut a = parent(&one).await;
        let two_store = MemoryBackend::new();
        for (key, value) in backend_snapshot(&one_store).await {
            two_store.put(&key, &value).await.unwrap();
        }
        let two = MereSessions::new(two_store);
        let mut b = two.open(a.id()).await.unwrap();
        assert_eq!(
            a.operation_revision().unwrap(),
            b.operation_revision().unwrap()
        );
        let akey = a.graph().get_node_by_id(id(1)).unwrap().0;
        let bkey = b.graph().get_node_by_id(id(1)).unwrap().0;
        a.transact_deltas_published(
            person(),
            vec![GraphDelta::SetNodeTitle {
                key: akey,
                title: "alpha".into(),
            }],
            wall_clock_now(),
        )
        .await
        .unwrap();
        b.transact_deltas_published(
            person(),
            vec![GraphDelta::SetNodeTitle {
                key: bkey,
                title: "beta".into(),
            }],
            wall_clock_now(),
        )
        .await
        .unwrap();
        assert_eq!(a.journal().live_cursor(), b.journal().live_cursor());
        assert_eq!(a.changes().len(), b.changes().len());
        assert_ne!(
            a.operation_revision().unwrap(),
            b.operation_revision().unwrap()
        );
        assert_eq!(
            a.operation_revision().unwrap(),
            one.open(a.id())
                .await
                .unwrap()
                .operation_revision()
                .unwrap()
        );
    });
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn operation_storage_refusal_and_lost_ack_reconcile() {
    use std::sync::{
        Arc,
        atomic::{AtomicU8, Ordering},
    };
    #[derive(Clone)]
    struct FaultBackend {
        store: MemoryBackend,
        fault: Arc<AtomicU8>,
    }
    #[async_trait::async_trait]
    impl Backend for FaultBackend {
        async fn get(&self, key: &str) -> Result<Option<Vec<u8>>, StoreError> {
            self.store.get(key).await
        }
        async fn put(&self, key: &str, bytes: &[u8]) -> Result<(), StoreError> {
            self.store.put(key, bytes).await
        }
        async fn delete(&self, key: &str) -> Result<(), StoreError> {
            self.store.delete(key).await
        }
        async fn list(&self, prefix: &str) -> Result<Vec<String>, StoreError> {
            self.store.list(prefix).await
        }
        async fn scan(&self, start: &str, end: &str) -> Result<Vec<String>, StoreError> {
            self.store.scan(start, end).await
        }
        async fn apply(&self, ops: &[WriteOp]) -> Result<(), StoreError> {
            self.store.apply(ops).await
        }
        async fn transact(&self, transaction: muniment::TransactFn) -> Result<(), StoreError> {
            let fault = self.fault.swap(0, Ordering::SeqCst);
            if fault == 1 {
                return Err(StoreError::NotTransactional);
            }
            self.store.transact(transaction).await?;
            if fault == 2 {
                return Err(StoreError::NotTransactional);
            }
            Ok(())
        }
    }
    pollster::block_on(async {
        let store = MemoryBackend::new();
        let mere = MereSessions::new(store.clone());
        let session = parent(&mere).await.id();
        let fault = Arc::new(AtomicU8::new(1));
        let backend = FaultBackend {
            store,
            fault: fault.clone(),
        };
        let mut writer = GraphSession::open(backend.clone(), session).await.unwrap();
        let mut pending = GraphSession::open(backend.clone(), session).await.unwrap();
        let mut rejected = GraphSession::open(backend.clone(), session)
            .await
            .unwrap()
            .with_validator(Arc::new(|graph| {
                if graph.get_node_by_id(id(79)).is_some() {
                    Err("replayed graph rejected".into())
                } else {
                    Ok(())
                }
            }))
            .unwrap();
        let make_deltas = || {
            vec![GraphDelta::AddNode {
                id: Some(id(79)),
                url: "https://brief.test/79".into(),
                position: euclid::point2(0.0, 0.0),
            }]
        };
        assert!(
            writer
                .transact_operation_published(
                    [20; 32],
                    b"same".to_vec(),
                    b"result".to_vec(),
                    person(),
                    make_deltas(),
                    wall_clock_now()
                )
                .await
                .is_err()
        );
        assert!(writer.graph().get_node_by_id(id(79)).is_none());
        assert!(writer.operation_receipt([20; 32]).await.unwrap().is_none());
        assert!(
            writer
                .reconcile_operation([20; 32])
                .await
                .unwrap()
                .is_none()
        );
        fault.store(2, Ordering::SeqCst);
        assert!(
            writer
                .transact_operation_published(
                    [20; 32],
                    b"same".to_vec(),
                    b"result".to_vec(),
                    person(),
                    make_deltas(),
                    wall_clock_now()
                )
                .await
                .is_err()
        );
        assert!(writer.graph().get_node_by_id(id(79)).is_none());
        let old_cursor = writer.journal().live_cursor();

        // A reload must never discard local unsaved work, even when the receipt
        // is present in durable storage.
        let key = pending.graph().get_node_by_id(id(1)).unwrap().0;
        pending
            .edit_now(person(), |graph| {
                apply_graph_delta(
                    graph,
                    GraphDelta::SetNodeTitle {
                        key,
                        title: "unsaved".into(),
                    },
                );
            })
            .unwrap();
        assert!(pending.reconcile_operation([20; 32]).await.is_err());
        assert_eq!(title(pending.graph(), id(1)).as_deref(), Some("unsaved"));
        assert!(pending.graph().get_node_by_id(id(79)).is_none());

        // Product validation rejects a reconstructed graph without replacing the
        // previously held graph, history or settings.
        assert!(rejected.reconcile_operation([20; 32]).await.is_err());
        assert!(rejected.graph().get_node_by_id(id(79)).is_none());
        assert_eq!(rejected.journal().live_cursor(), old_cursor);

        writer.set_current_session(43);
        writer.set_checkpoint_interval(37);
        let validation: GraphValidator = Arc::new(|_| Ok(()));
        let admission: GraphAdmissionValidator =
            Arc::new(|_, _| Err("new admission refused".into()));
        writer = writer
            .with_validator(validation.clone())
            .unwrap()
            .with_admission_validator(admission.clone());
        let (reconciled, current_generation) =
            writer.reconcile_operation([20; 32]).await.unwrap().unwrap();
        assert_eq!(writer.graph().node_count(), 4);
        assert!(writer.graph().get_node_by_id(id(79)).is_some());
        assert!(writer.journal().live_cursor() > old_cursor);
        assert_eq!(writer.graph().current_session(), 43);
        assert_eq!(writer.checkpoint_interval, 37);
        assert!(Arc::ptr_eq(writer.validator.as_ref().unwrap(), &validation));
        assert!(Arc::ptr_eq(
            writer.admission_validator.as_ref().unwrap(),
            &admission
        ));
        assert_eq!(current_generation, reconciled.generation);
        // Reconciliation does not consume a second operation or repeat a graph edit.
        let current_cursor = writer.journal().live_cursor();
        assert_eq!(
            writer
                .reconcile_operation([20; 32])
                .await
                .unwrap()
                .unwrap()
                .0,
            reconciled
        );
        assert_eq!(writer.journal().live_cursor(), current_cursor);

        let mut reopened = GraphSession::open(backend, session).await.unwrap();
        let receipt = reopened.operation_receipt([20; 32]).await.unwrap().unwrap();
        assert_eq!(receipt, reconciled);
        assert!(reopened.graph().get_node_by_id(id(79)).is_some());
        let retry = reopened
            .transact_operation_published(
                [20; 32],
                b"same".to_vec(),
                b"result".to_vec(),
                person(),
                make_deltas(),
                wall_clock_now(),
            )
            .await
            .unwrap();
        assert_eq!(receipt, retry);
        assert_eq!(reopened.graph().node_count(), 4);
    });
}
