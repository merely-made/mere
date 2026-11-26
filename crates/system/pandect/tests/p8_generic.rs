// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! pandect's transaction, operation and page-claims families over a graph
//! with no kernel (engine split D257): the minimal surface of D245, publishing
//! through its own hook and writing resource records through the engine's one
//! recorded write.

mod support;

use std::time::{SystemTime, UNIX_EPOCH};

use mere::ContextCache;
use mere::acquire::page_claims::{PageClaims, interpret};
use mere::model::Author;
use muniment::{Backend, MemoryBackend};
use oxrdf::NamedNode;
use pandect::{GraphSession, MereSessions, SessionError, SessionId};
use support::{PUBLICATION, Surface, UNPUBLISHABLE, generation_of, head_key, show_edits};
use uuid::Uuid;

type Sessions = MereSessions<MemoryBackend, Surface>;

async fn minted(sessions: &Sessions) -> (SessionId, GraphSession<MemoryBackend, Surface>) {
    let id = sessions
        .mint(Author::person("p8-generic"), None)
        .await
        .unwrap()
        .session_id;
    (id, sessions.open_with(id).await.unwrap())
}

async fn stored(backend: &MemoryBackend) -> Vec<(String, Vec<u8>)> {
    let mut out = Vec::new();
    for key in backend.list("").await.unwrap() {
        let value = backend.get(&key).await.unwrap().unwrap();
        out.push((key, value));
    }
    out.sort();
    out
}

#[test]
fn a_published_transaction_writes_the_hooks_publication_with_its_edit() {
    pollster::block_on(async {
        let backend = MemoryBackend::new();
        let sessions = Sessions::new_with(backend.clone()).with_publication(&PUBLICATION);
        let (id, mut session) = minted(&sessions).await;
        let container = *session.manifest().root_graph_id.as_uuid();
        let author = Author::person("p8-generic");
        let (resource, edits) = show_edits(Uuid::from_u128(0xa1), "https://p8.test/published");

        let (applied, generation) = session
            .transact_deltas_published(author.clone(), edits, SystemTime::now())
            .await
            .unwrap();
        assert_eq!((applied.first.0, applied.end.0), (0, 2));
        assert_eq!(session.graph().shown(Uuid::from_u128(0xa1)), Some(resource));
        // The hook named the edited graph, and its write landed with the edit.
        assert_eq!(generation, generation_of(session.graph()));
        assert_eq!(
            backend.get(&head_key(container)).await.unwrap(),
            Some(generation.clone().into_bytes())
        );
        drop(session);
        let reopened = sessions.open_with(id).await.unwrap();
        assert_eq!(reopened.journal().live_cursor().0, 2);
        assert_eq!(generation_of(reopened.graph()), generation);

        // Positive control: the same surface with no hook installed refuses
        // to publish, and the store does not move.
        let bare = Sessions::new_with(backend.clone());
        let mut unhooked = bare.open_with(id).await.unwrap();
        let before = stored(&backend).await;
        let (_, more) = show_edits(Uuid::from_u128(0xa2), "https://p8.test/refused");
        let refused = unhooked
            .transact_deltas_published(author.clone(), more.clone(), SystemTime::now())
            .await;
        assert!(
            matches!(&refused, Err(SessionError::Validation(why)) if why.contains("no publication hook")),
            "{refused:?}"
        );
        assert_eq!(stored(&backend).await, before);
        assert_eq!(unhooked.journal().live_cursor().0, 2);
        // The unpublished path still writes without a hook.
        unhooked
            .transact_deltas(author.clone(), more, SystemTime::now())
            .await
            .unwrap();
        assert_eq!(unhooked.journal().live_cursor().0, 4);
        assert_eq!(
            backend.get(&head_key(container)).await.unwrap(),
            Some(generation.clone().into_bytes()),
            "an unpublished write leaves the publication"
        );
        drop(unhooked);

        // And the hook's own refusal leaves graph, journal and store as they were.
        let mut hooked = sessions.open_with(id).await.unwrap();
        let before = stored(&backend).await;
        let (_, bad) = show_edits(UNPUBLISHABLE, "https://p8.test/unpublishable");
        let refused = hooked
            .transact_deltas_published(author, bad, SystemTime::now())
            .await;
        assert!(
            matches!(&refused, Err(SessionError::Validation(why)) if why == "surface refuses publication"),
            "{refused:?}"
        );
        assert_eq!(stored(&backend).await, before);
        assert_eq!(hooked.graph().shown(UNPUBLISHABLE), None);
        assert_eq!(hooked.journal().live_cursor().0, 4);
    });
}

#[test]
fn an_operation_is_retained_with_its_publication_and_reconciles() {
    pollster::block_on(async {
        let backend = MemoryBackend::new();
        let sessions = Sessions::new_with(backend.clone()).with_publication(&PUBLICATION);
        let (id, mut session) = minted(&sessions).await;
        let author = Author::person("p8-generic");
        let operation = [7u8; 32];
        let (_, edits) = show_edits(Uuid::from_u128(0xb1), "https://p8.test/operation");

        let receipt = session
            .transact_operation_published(
                operation,
                b"request".to_vec(),
                b"result".to_vec(),
                author.clone(),
                edits.clone(),
                SystemTime::now(),
            )
            .await
            .unwrap();
        assert_eq!(receipt.generation, generation_of(session.graph()));
        assert_eq!(
            session.operation_receipt(operation).await.unwrap(),
            Some(receipt.clone())
        );
        let revision = session.operation_revision().unwrap();
        let cursor = session.journal().live_cursor();

        // The same request is answered by its receipt, editing nothing twice;
        // another request under the identity is refused.
        let again = session
            .transact_operation_published(
                operation,
                b"request".to_vec(),
                b"result".to_vec(),
                author.clone(),
                edits.clone(),
                SystemTime::now(),
            )
            .await
            .unwrap();
        assert_eq!(again, receipt);
        assert_eq!(session.journal().live_cursor(), cursor);
        assert!(
            session
                .transact_operation_published(
                    operation,
                    b"other".to_vec(),
                    b"result".to_vec(),
                    author,
                    edits,
                    SystemTime::now(),
                )
                .await
                .is_err()
        );

        // Reconciliation reopens through the session's hooks and reads the
        // generation the store holds.
        let mut stale = sessions.open_with(id).await.unwrap();
        assert_eq!(stale.operation_revision().unwrap(), revision);
        let (held, generation) = stale.reconcile_operation(operation).await.unwrap().unwrap();
        assert_eq!(held, receipt);
        assert_eq!(generation, receipt.generation);
        assert_eq!(stale.journal().live_cursor(), cursor);
        assert!(
            stale
                .reconcile_operation([8u8; 32])
                .await
                .unwrap()
                .is_none()
        );

        // Positive control: an unhooked handle has no revision to name.
        let bare = Sessions::new_with(backend).open_with(id).await.unwrap();
        assert!(bare.operation_revision().is_err());
    });
}

#[test]
fn page_claims_are_kept_through_the_engines_recorded_resource_write() {
    pollster::block_on(async {
        let backend = MemoryBackend::new();
        let sessions = Sessions::new_with(backend.clone());
        let (id, mut session) = minted(&sessions).await;
        let author = Author::person("p8-generic");
        let source = NamedNode::new_unchecked("urn:fixture:p8-kept-capture");
        let block = r##"{"@id":"#river","http://example.test/source":"surface claim"}"##;
        let claims = interpret(
            &ContextCache::new(),
            "https://example.test/notes",
            [(5, block)],
        );
        let before = session.journal().live_cursor();
        assert!(session.kept_page_claims(&source).unwrap().is_none());

        let kept = session
            .admit_page_claims(author.clone(), &source, &claims, UNIX_EPOCH)
            .await
            .unwrap();
        let cursor = session.journal().live_cursor();
        // One recorded resource write, journaled as the engine's own delta.
        assert_eq!(cursor.0, before.0 + 1);
        assert!(matches!(
            session.journal().entries()[before.index()].delta,
            mere::model::JournalDelta::Resource(
                mere::model::ResourceDelta::ReplaySetResourceRecordById { .. }
            )
        ));
        assert!(
            session
                .pin(before)
                .unwrap()
                .kept_page_claims(&source)
                .unwrap()
                .is_none()
        );
        // An identical retry edits nothing; different claims are refused.
        let retry = session
            .admit_page_claims(author.clone(), &source, &claims, UNIX_EPOCH)
            .await
            .unwrap();
        assert_eq!(retry, kept);
        assert_eq!(session.journal().live_cursor(), cursor);
        assert!(
            session
                .admit_page_claims(author, &source, &PageClaims::default(), UNIX_EPOCH)
                .await
                .is_err()
        );
        assert_eq!(session.journal().live_cursor(), cursor);
        drop(session);

        let reopened = sessions.open_with(id).await.unwrap();
        let read = reopened
            .pin(cursor)
            .unwrap()
            .kept_page_claims(&source)
            .unwrap()
            .unwrap();
        assert_eq!(read, kept);
        assert_eq!(read.claims.blocks[0].order, 5);
    });
}

/// The personal graph's protocol folding into the surface through its host
/// fold (engine split D259).
#[cfg(feature = "personal-sync")]
mod personal_sync {
    use eidetic::PrivacyClass;
    use identity::{IdentityProvider, InMemoryProvider};
    use mere::model::{JournalDelta, ResourceNode};
    use mere::session::SessionGraph;
    use muniment::MemoryBackend;
    use pandect::access::{
        ACCESS_HISTORY_FACET, AccessAction, AccessHistory, AccessRecord, AccessTransition,
    };
    use pandect::personal_sync::{
        KernelFold, PersonalFold, PersonalGraphError, PersonalGraphEvent, PersonalGraphReplica,
        SyncRoster, SyncSelection,
    };
    use pandect::{AcceptAll, FacetId};
    use uuid::Uuid;

    use super::support::{Surface, SurfaceDelta};

    const GRAPH: [u8; 32] = [0x8b; 32];
    const A: Uuid = Uuid::from_u128(0xc1);
    const B: Uuid = Uuid::from_u128(0xc2);

    /// Nodes become surfaces showing their address; access history rides as
    /// the surface's facet. It folds nothing else.
    struct SurfaceFold;

    impl PersonalFold<Surface> for SurfaceFold {
        fn apply(
            &self,
            graph: &mut Surface,
            _asserter_iri: &str,
            _fallback_id: &str,
            event: &PersonalGraphEvent,
        ) -> Result<(), PersonalGraphError> {
            match event {
                PersonalGraphEvent::AddNode { id, address, .. } => {
                    graph.show(*id, address);
                    Ok(())
                },
                PersonalGraphEvent::RemoveNode { id } => graph
                    .apply(JournalDelta::Surface(SurfaceDelta::HideSurface {
                        surface_id: *id,
                    }))
                    .map_err(PersonalGraphError::Excluded),
                other => Err(PersonalGraphError::Excluded(format!(
                    "the surface folds no {other:?}"
                ))),
            }
        }

        fn set_access_history(&self, graph: &mut Surface, node: Uuid, history: AccessHistory) {
            let mut facets = graph.facets().clone();
            facets
                .set(
                    node,
                    FacetId::new(ACCESS_HISTORY_FACET),
                    serde_json::to_value(history).unwrap(),
                    &AcceptAll,
                )
                .unwrap();
            graph.install_facets(facets).unwrap();
        }
    }

    fn access(id: u128) -> AccessRecord {
        AccessRecord {
            record_id: Uuid::from_u128(id),
            container_id: A,
            address: "https://p8.test/sync-a".into(),
            action: AccessAction::Examine,
            persona: "personae://profile/shared".into(),
            device: "p8".into(),
            application: "p8".into(),
            at_ms: 1,
            handler: "p8.inspect".into(),
            dwell_ms: None,
            referring_container_id: None,
            referring_address: None,
            transition: AccessTransition::Unknown,
            capture_source: "p8.generic".into(),
            source_event_id: None,
            privacy: PrivacyClass::TrustedPeersOnly,
        }
    }

    #[test]
    fn a_host_fold_materializes_the_personal_graph_into_its_own_graph() {
        pollster::block_on(async {
            let identity = InMemoryProvider::from_seed([0x8c; 32]);
            let roster = SyncRoster::new([identity.master_public_key().to_bytes()]);
            let mut replica = PersonalGraphReplica::for_identity(
                MemoryBackend::new(),
                GRAPH,
                &identity,
                roster,
                SyncSelection::default().with_access_records(true),
            )
            .unwrap();
            let add = |id, address: &str| PersonalGraphEvent::AddNode {
                id,
                address: address.into(),
                title: String::new(),
            };
            replica
                .author(vec![
                    add(A, "https://p8.test/sync-a"),
                    add(B, "https://p8.test/sync-b"),
                ])
                .await
                .unwrap();
            replica
                .author(vec![
                    PersonalGraphEvent::RemoveNode { id: B },
                    PersonalGraphEvent::AppendAccess {
                        record: access(0xacc),
                    },
                ])
                .await
                .unwrap();

            let projection = replica.projection_with(&SurfaceFold).await.unwrap();
            let graph = &projection.graph;
            assert_eq!(
                graph.shown(A),
                Some(ResourceNode::for_term("https://p8.test/sync-a").id())
            );
            assert_eq!(graph.shown(B), None);
            // The protocol's bookkeeping stays pandect's; the fold holds it.
            assert_eq!(projection.access_records.len(), 1);
            let held = graph
                .facets()
                .get(&A, &FacetId::new(ACCESS_HISTORY_FACET))
                .expect("access history on the surface");
            assert_eq!(
                held["records"][0]["record_id"],
                access(0xacc).record_id.to_string()
            );
            assert_eq!(projection.writers.len(), 2);

            // Control: the kernel's fold of the same operations holds the
            // same nodes in its own graph.
            let kernel = replica.projection_with(&KernelFold).await.unwrap();
            let ids: Vec<Uuid> = kernel.graph.nodes().map(|(_, node)| node.id).collect();
            assert_eq!(ids, [A]);

            // Control: a fold that refuses an event refuses the projection,
            // so every graph event reached the host's fold.
            replica
                .author(vec![PersonalGraphEvent::SetTitle {
                    node: A,
                    title: "titled".into(),
                }])
                .await
                .unwrap();
            assert!(matches!(
                replica.projection_with(&SurfaceFold).await,
                Err(PersonalGraphError::Excluded(why)) if why.contains("SetTitle")
            ));
            assert!(replica.projection().await.is_ok());
        });
    }
}

#[test]
fn a_snapshot_materializes_under_its_placement_over_any_graph() {
    use mere::session::SessionGraph;
    use pandect::graph_placement::{PlacementProfile, materialize_snapshot_with};
    use pandect::product_codicil::profiled_codicil_graph_with;

    let mut graph = Surface::default();
    graph.show(Uuid::from_u128(0xd1), "https://p8.test/codicil");
    let snapshot = graph.to_snapshot();
    for placement in [None, Some(PlacementProfile::RecordedStrataV1)] {
        let read: Surface = materialize_snapshot_with(&snapshot, placement).unwrap();
        assert_eq!(read.to_snapshot(), snapshot);
        let read: Surface =
            profiled_codicil_graph_with(snapshot.clone(), graph.facets().clone(), placement)
                .unwrap();
        assert_eq!(read.to_snapshot(), snapshot);
    }
    // Control: a legacy placement is refused without qualified replay.
    assert!(
        materialize_snapshot_with::<Surface>(&snapshot, Some(PlacementProfile::LegacySurfaceV1))
            .is_err()
    );
}

/// Merge over the surface (engine split D264): a branch's changes come in
/// where the target left a part alone, the target's own changes stay, and a
/// part both changed is a conflict that waits for a choice.
#[test]
fn a_branch_merges_back_over_a_host_graph() {
    use std::collections::BTreeMap;

    use mere::model::ResourceNode;
    use pandect::{MergeChoice, MergeMatch};
    use support::Part;

    pollster::block_on(async {
        let backend = MemoryBackend::new();
        let sessions = Sessions::new_with(backend.clone());
        let author = Author::person("p8-merge");
        let (_, mut target) = minted(&sessions).await;
        let (a, b, c) = (
            Uuid::from_u128(0xe1),
            Uuid::from_u128(0xe2),
            Uuid::from_u128(0xe3),
        );
        let resource = |iri: &str| ResourceNode::for_term(iri).id();
        target
            .edit_now(author.clone(), |graph| {
                graph.show(a, "https://p8.test/a1");
                graph.show(b, "https://p8.test/b1");
            })
            .unwrap();
        target.flush(UNIX_EPOCH).await.unwrap();
        let at = target.journal().live_cursor();
        let branch = sessions
            .branch_at(&target, at, author.clone())
            .await
            .unwrap()
            .session_id;
        let mut source = sessions.open_with(branch).await.unwrap();
        source
            .edit_now(author.clone(), |graph| {
                graph.show(a, "https://p8.test/a2");
                graph.show(c, "https://p8.test/c1");
            })
            .unwrap();
        source.flush(UNIX_EPOCH).await.unwrap();
        target
            .edit_now(author.clone(), |graph| {
                graph.show(b, "https://p8.test/b2");
            })
            .unwrap();
        target.flush(UNIX_EPOCH).await.unwrap();

        let plan = sessions.merge_in(&target, &source).await.unwrap();
        assert_eq!(plan.matched(), MergeMatch::Branch);
        assert!(plan.is_clean(), "{:?}", plan.conflicts());
        let merged = sessions
            .apply_merge(
                &mut target,
                &mut source,
                plan,
                &BTreeMap::new(),
                author.clone(),
            )
            .await
            .unwrap();
        assert!(merged.refused.is_empty());
        let graph = target.graph();
        assert_eq!(graph.shown(a), Some(resource("https://p8.test/a2")));
        assert_eq!(graph.shown(b), Some(resource("https://p8.test/b2")));
        assert_eq!(graph.shown(c), Some(resource("https://p8.test/c1")));
        assert!(source.manifest().merged.is_some());

        // Control: a part both sides changed is a conflict, refused without a
        // choice and taken from the source with one.
        let at = target.journal().live_cursor();
        let branch = sessions
            .branch_at(&target, at, author.clone())
            .await
            .unwrap()
            .session_id;
        let mut source = sessions.open_with(branch).await.unwrap();
        source
            .edit_now(author.clone(), |graph| {
                graph.show(a, "https://p8.test/a-source");
            })
            .unwrap();
        source.flush(UNIX_EPOCH).await.unwrap();
        target
            .edit_now(author.clone(), |graph| {
                graph.show(a, "https://p8.test/a-target");
            })
            .unwrap();
        target.flush(UNIX_EPOCH).await.unwrap();
        let plan = sessions.merge_in(&target, &source).await.unwrap();
        let parts: Vec<&Part> = plan.conflicts().iter().map(|c| &c.part).collect();
        assert_eq!(parts, [&Part::Surface(a)]);
        assert!(matches!(
            sessions
                .apply_merge(
                    &mut target,
                    &mut source,
                    plan.clone(),
                    &BTreeMap::new(),
                    author.clone(),
                )
                .await,
            Err(SessionError::Unresolved(1))
        ));
        assert_eq!(
            target.graph().shown(a),
            Some(resource("https://p8.test/a-target"))
        );
        let choices = BTreeMap::from([(Part::Surface(a), MergeChoice::Source)]);
        sessions
            .apply_merge(&mut target, &mut source, plan, &choices, author)
            .await
            .unwrap();
        assert_eq!(
            target.graph().shown(a),
            Some(resource("https://p8.test/a-source"))
        );
    });
}
