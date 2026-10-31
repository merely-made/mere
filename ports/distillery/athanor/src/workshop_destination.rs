// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Owner-gated evidence brief admission into the existing GraphSession writer.
//! The operation record, journal and addressable publication share one storage
//! transaction. This adapter supplies no custody or human authority of its own.

use crate::workshop::{Error, revalidate_acceptance, validate_proposal};
use alembic::workshop::{Acceptance, Owner, Passage, Proposal, Receipt, Source};
use kernel::graph::{Author, Graph, apply::GraphDelta};
use kernel::persistence::{PersistedResourceFacet, PersistedResourceRecord};
use pandect::graph_session::{GraphSession, LegacyTransactionBackend, OperationReceipt};
use servitor::AuthorityProvider;
use std::time::SystemTime;
use uuid::Uuid;

fn encoded<T: serde::Serialize>(value: &T) -> Result<Vec<u8>, Error> {
    serde_json::to_vec(value).map_err(|_| Error::Invalid("destination record encoding"))
}
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
fn failure(error: impl std::fmt::Display) -> Error {
    Error::Store(error.to_string())
}

/// Stable durable graph and change-log head, including across reopening.
pub fn revision<B: LegacyTransactionBackend + Clone>(
    session: &GraphSession<B>,
) -> Result<String, Error> {
    session.operation_revision().map_err(failure)
}

fn request(
    graph_id: &str,
    owner: &Owner,
    acceptance: &Acceptance,
    proposal: &Proposal,
) -> Result<Vec<u8>, Error> {
    validate_proposal(proposal)?;
    let operation = blake3::hash(&encoded(&(
        "alembic-acceptance-v1",
        owner,
        acceptance.installation,
        acceptance.run,
        proposal.digest,
    ))?);
    if acceptance.operation != *operation.as_bytes()
        || acceptance.destination.owner != *owner
        || acceptance.destination.graph != graph_id
        || acceptance.proposal != proposal.digest
        || acceptance.installation != proposal.installation
        || acceptance.run != proposal.run
        || proposal.sources.iter().any(|p| p.source.owner != *owner)
    {
        return Err(Error::Invalid("acceptance identity"));
    }
    encoded(&(acceptance, proposal))
}

pub(crate) fn identifiers(acceptance: &Acceptance) -> (Uuid, String, String) {
    let iri = format!(
        "https://mere.computer/notes/evidence-brief/{}",
        hex(&acceptance.operation)
    );
    let resource = chartulary::resource_id_from_canonical_iri(&chartulary::canonical_url(&iri));
    let hash =
        blake3::hash(format!("alembic-note-surface-v1/{}", hex(&acceptance.operation)).as_bytes());
    let mut bytes = [0; 16];
    bytes.copy_from_slice(&hash.as_bytes()[..16]);
    (Uuid::from_bytes(bytes), iri, resource.to_string())
}

/// The original created note surface. Other surfaces showing its Resource do
/// not replace this identity when routing an operation's result.
pub fn result_surface(acceptance: &Acceptance) -> Uuid {
    identifiers(acceptance).0
}

pub(crate) fn ledger_identity(acceptance: &Acceptance) -> (String, Uuid) {
    ledger_identity_for(&acceptance.operation)
}

pub(crate) fn ledger_identity_for(operation: &[u8; 32]) -> (String, Uuid) {
    let iri = format!(
        "https://mere.computer/operations/evidence-brief/{}",
        hex(operation)
    );
    let iri = chartulary::canonical_url(&iri);
    let id = chartulary::resource_id_from_canonical_iri(&iri);
    (iri, id)
}

fn recover(
    record: OperationReceipt,
    session: &str,
    request: &[u8],
    acceptance: &Acceptance,
    resource: String,
) -> Result<Receipt, Error> {
    if record.session != session
        || record.request != request
        || record.result != resource.as_bytes()
    {
        return Err(Error::Conflict);
    }
    Ok(Receipt {
        operation: acceptance.operation,
        proposal: acceptance.proposal,
        destination: acceptance.destination.clone(),
        resource,
        storage_witness: record.generation,
    })
}

/// Recover an exact previously committed operation, independent of later graph
/// changes or an elapsed deadline. A conflicting identity is refused.
pub async fn query<B: LegacyTransactionBackend + Clone>(
    session: &GraphSession<B>,
    owner: &Owner,
    acceptance: &Acceptance,
    proposal: &Proposal,
) -> Result<Option<Receipt>, Error> {
    if owner.session != session.id().as_uuid().to_string() {
        return Err(Error::Invalid("destination session"));
    }
    let graph_id = session.manifest().root_graph_id.as_uuid().to_string();
    let request = request(&graph_id, owner, acceptance, proposal)?;
    let (_, _, resource) = identifiers(acceptance);
    session
        .operation_receipt(acceptance.operation)
        .await
        .map_err(failure)?
        .map(|record| recover(record, &owner.session, &request, acceptance, resource))
        .transpose()
}

/// Commit a new accepted note after the host's current custody decision. Exact
/// replays recover first; the caller must supply a fresh custody observation and
/// re-enter this gate after any intervening authority or custody change.
pub async fn commit<B: LegacyTransactionBackend + Clone>(
    session: &mut GraphSession<B>,
    owner: &Owner,
    acceptance: &Acceptance,
    proposal: &Proposal,
    human: [u8; 32],
    authority: &impl AuthorityProvider,
    now_ms: u64,
    custody_open: bool,
) -> Result<Receipt, Error> {
    if owner.session != session.id().as_uuid().to_string() {
        return Err(Error::Invalid("destination session"));
    }
    let graph_id = session.manifest().root_graph_id.as_uuid().to_string();
    let request = request(&graph_id, owner, acceptance, proposal)?;
    if human != acceptance.human {
        return Err(Error::Unauthorized);
    }
    let (_, _, resource) = identifiers(acceptance);
    if let Some(record) = session
        .operation_receipt(acceptance.operation)
        .await
        .map_err(failure)?
    {
        return recover(record, &owner.session, &request, acceptance, resource);
    }
    let prepared = prepare_graph(
        session.graph(),
        owner,
        &graph_id,
        &revision(session)?,
        acceptance,
        proposal,
        human,
        authority,
        now_ms,
        custody_open,
    )?;
    let GraphPreparation::New { deltas, .. } = prepared else {
        // GraphSession's independent durable ledger is authoritative here.
        return Err(Error::Conflict);
    };
    let verify_owner = owner.clone();
    let verify_acceptance = acceptance.clone();
    let verify_proposal = proposal.clone();
    let verify_graph_id = graph_id.clone();
    let record = session
        .transact_operation_published_checked(
            acceptance.operation,
            request.clone(),
            resource.as_bytes().to_vec(),
            Author::person(hex(&human)).via("alembic"),
            deltas,
            SystemTime::now(),
            move |graph| {
                verify_graph_effect(
                    graph,
                    &verify_owner,
                    &verify_graph_id,
                    &verify_acceptance,
                    &verify_proposal,
                )
                .map_err(|e| e.to_string())
            },
        )
        .await
        .map_err(failure)?;
    recover(record, &owner.session, &request, acceptance, resource)
}

const OPERATION_FACET: &str = "mere.alembic.evidence-brief.operation/v1";

/// A prepared edit is not a storage acknowledgement. The owner installs the
/// candidate and returns its receipt only after its atomic save succeeds.
pub enum GraphPreparation {
    Existing(Receipt),
    New {
        deltas: Vec<GraphDelta>,
        receipt: Receipt,
    },
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct GraphOperation {
    request: Vec<u8>,
    resource: String,
    witness: String,
}

pub(crate) fn require_complete(graph: &Graph) -> Result<(), Error> {
    if graph
        .known_coverage()
        .limits
        .iter()
        .any(|limit| limit.layer == kernel::graph::CoverageLayer::Residency)
    {
        return Err(Error::Invalid("partial destination graph"));
    }
    Ok(())
}

/// A complete snapshot content token; the owner must serialize all mutations
/// between observing this token and atomically replacing its graph snapshot.
pub fn graph_revision(graph: &Graph) -> Result<String, Error> {
    require_complete(graph)?;
    // Snapshot reconstruction synthesizes visit times and imports legacy
    // defaults. Replace those with the exact authoritative facet sidecar.
    let mut normalized =
        Graph::try_from_recorded_snapshot(&graph.to_snapshot_at(0)).map_err(failure)?;
    normalized
        .install_facets(graph.facets().clone())
        .map_err(failure)?;
    Ok(blake3::hash(&encoded(&(
        normalized.to_snapshot_at(0),
        normalized.facets(),
    ))?)
    .to_hex()
    .to_string())
}

/// Inspect the separate operation Resource in a saved owner graph. Ordinary
/// note deletion leaves it intact. The owner must retain the ledger Resource
/// against explicit removal or replacement while reconciliation is required.
pub fn query_graph(
    graph: &Graph,
    owner: &Owner,
    graph_id: &str,
    acceptance: &Acceptance,
    proposal: &Proposal,
) -> Result<Option<Receipt>, Error> {
    require_complete(graph)?;
    let request = request(graph_id, owner, acceptance, proposal)?;
    let (_, _, resource) = identifiers(acceptance);
    let (ledger_iri, _) = ledger_identity(acceptance);
    let snapshot = graph.to_snapshot_at(0);
    let Some(held) = snapshot
        .resources
        .iter()
        .find(|r| r.canonical_iri == ledger_iri)
    else {
        return Ok(None);
    };
    let Some(facet) = held.facets.iter().find(|f| f.facet == OPERATION_FACET) else {
        return Err(Error::Conflict);
    };
    let record: GraphOperation =
        serde_json::from_str(&facet.value_json).map_err(|_| Error::Conflict)?;
    let witness = blake3::hash(&encoded(&(
        "alembic-graph-operation-v1",
        &request,
        &resource,
    ))?)
    .to_hex()
    .to_string();
    if record.request != request || record.resource != resource || record.witness != witness {
        return Err(Error::Conflict);
    }
    Ok(Some(Receipt {
        operation: acceptance.operation,
        proposal: acceptance.proposal,
        destination: acceptance.destination.clone(),
        resource,
        storage_witness: witness,
    }))
}

/// Refuse deletion or replacement of existing operation evidence. A graph-only
/// owner must call this on every candidate (including undo and GC), with the
/// comparison serialized against its atomic snapshot replacement. This helper
/// does not install a kernel-wide retention policy or protect other writers.
pub fn preserve_graph_operations(before: &Graph, after: &Graph) -> Result<(), Error> {
    require_complete(before)?;
    require_complete(after)?;
    let previous = before.to_snapshot_at(0);
    let next = after.to_snapshot_at(0);
    for resource in &previous.resources {
        for operation in resource.facets.iter().filter(|f| {
            f.facet == OPERATION_FACET
                || f.facet == "mere.alembic.evidence-brief.sealed-operation/v1"
        }) {
            let retained = next
                .resources
                .iter()
                .find(|r| r.canonical_iri == resource.canonical_iri)
                .and_then(|r| r.facets.iter().find(|f| f.facet == operation.facet));
            if retained != Some(operation) {
                return Err(Error::Conflict);
            }
        }
    }
    Ok(())
}

/// Whether the created Resource still has its original shown note surface.
/// This is a current graph observation, separate from the durable receipt.
pub fn graph_result_present(graph: &Graph, acceptance: &Acceptance) -> bool {
    let (surface, _, resource) = identifiers(acceptance);
    graph
        .get_node_by_id(surface)
        .and_then(|(key, _)| graph.shown_resource_id(key))
        .is_some_and(|id| id.to_string() == resource)
}

/// Validate and stage the shared note effect for an existing graph owner.
/// The caller supplies its actual graph and session identities and serializes
/// the authority/custody decision with the eventual durable save.
pub fn prepare_graph(
    graph: &Graph,
    owner: &Owner,
    graph_id: &str,
    current_revision: &str,
    acceptance: &Acceptance,
    proposal: &Proposal,
    human: [u8; 32],
    authority: &impl AuthorityProvider,
    now_ms: u64,
    custody_open: bool,
) -> Result<GraphPreparation, Error> {
    let request = request(graph_id, owner, acceptance, proposal)?;
    if human != acceptance.human {
        return Err(Error::Unauthorized);
    }
    if let Some(receipt) = query_graph(graph, owner, graph_id, acceptance, proposal)? {
        return Ok(GraphPreparation::Existing(receipt));
    }
    if !custody_open {
        return Err(Error::Unauthorized);
    }
    revalidate_acceptance(acceptance, human, authority, now_ms)?;
    if acceptance.destination.revision != current_revision {
        return Err(Error::Stale);
    }
    let (node_id, iri, resource) = identifiers(acceptance);
    let (ledger_iri, ledger_id) = ledger_identity(acceptance);
    if graph.get_node_by_id(node_id).is_some()
        || graph.resource_nodes().any(|r| {
            r.canonical_iri() == chartulary::canonical_url(&iri) || r.canonical_iri() == ledger_iri
        })
    {
        return Err(Error::Conflict);
    }
    let witness = blake3::hash(&encoded(&(
        "alembic-graph-operation-v1",
        &request,
        &resource,
    ))?)
    .to_hex()
    .to_string();
    let record = GraphOperation {
        request,
        resource: resource.clone(),
        witness: witness.clone(),
    };
    let record = PersistedResourceRecord {
        canonical_iri: ledger_iri,
        facets: vec![PersistedResourceFacet {
            facet: OPERATION_FACET.into(),
            value_json: String::from_utf8(encoded(&record)?)
                .map_err(|_| Error::Invalid("operation encoding"))?,
        }],
    };
    let deltas = vec![
        GraphDelta::AddNode {
            id: Some(node_id),
            url: iri,
            position: euclid::point2(0.0, 0.0),
        },
        GraphDelta::ReplaySetNodeTitleById {
            node_id,
            title: "Evidence brief".into(),
        },
        GraphDelta::ReplaySetNodeBodyById {
            node_id,
            body: Some(proposal.djot.clone()),
        },
        GraphDelta::ReplaySetNodeMimeHintById {
            node_id,
            mime_hint: Some("text/djot".into()),
        },
        GraphDelta::ReplaySetResourceRecordById {
            resource_id: ledger_id,
            record: Some(record),
        },
    ];
    let receipt = Receipt {
        operation: acceptance.operation,
        proposal: acceptance.proposal,
        destination: acceptance.destination.clone(),
        resource,
        storage_witness: witness,
    };
    Ok(GraphPreparation::New { deltas, receipt })
}

/// Check the newly staged Resource, shown surface and retained Djot before the
/// owner commits a prepared effect. This is not a persistence acknowledgement.
pub fn verify_graph_effect(
    graph: &Graph,
    owner: &Owner,
    graph_id: &str,
    acceptance: &Acceptance,
    proposal: &Proposal,
) -> Result<(), Error> {
    if query_graph(graph, owner, graph_id, acceptance, proposal)?.is_none()
        || !graph_result_present(graph, acceptance)
    {
        return Err(Error::Conflict);
    }
    let (surface, _, resource) = identifiers(acceptance);
    let (key, _) = graph.get_node_by_id(surface).ok_or(Error::Conflict)?;
    if graph
        .shown_resource_id(key)
        .map(|id| id.to_string())
        .as_deref()
        != Some(resource.as_str())
    {
        return Err(Error::Conflict);
    }
    let snapshot = graph.to_snapshot_at(0);
    let node = snapshot
        .nodes
        .iter()
        .find(|n| n.node_id == surface.to_string())
        .ok_or(Error::Conflict)?;
    if node.body.as_deref() != Some(proposal.djot.as_str()) {
        return Err(Error::Conflict);
    }
    Ok(())
}

/// Read an actual retained body only after the owner authorizes its exact surface
/// and Resource as public. Presentation metadata alone is not that authority.
pub fn capture_public_source<B: LegacyTransactionBackend + Clone>(
    session: &GraphSession<B>,
    owner: &Owner,
    surface: Uuid,
    start: u64,
    end: u64,
    authorize_public: impl FnOnce(Uuid, Uuid) -> bool,
) -> Result<Passage, Error> {
    if owner.session != session.id().as_uuid().to_string() {
        return Err(Error::Invalid("source session"));
    }
    capture_graph_public_source(
        session.graph(),
        owner,
        surface,
        start,
        end,
        authorize_public,
    )
}

/// Capture an owner-authorized public retained body from a graph without a
/// GraphSession. The caller must verify that owner names its live workspace.
pub fn capture_graph_public_source(
    graph: &Graph,
    owner: &Owner,
    surface: Uuid,
    start: u64,
    end: u64,
    authorize_public: impl FnOnce(Uuid, Uuid) -> bool,
) -> Result<Passage, Error> {
    let (key, _) = graph.get_node_by_id(surface).ok_or(Error::Missing)?;
    let resource = graph.shown_resource_id(key).ok_or(Error::Missing)?;
    if !authorize_public(surface, resource) {
        return Err(Error::Unauthorized);
    }
    let snapshot = graph.to_snapshot_at(0);
    let node = snapshot
        .nodes
        .iter()
        .find(|n| n.node_id == surface.to_string())
        .ok_or(Error::Missing)?;
    let body = node.body.as_deref().ok_or(Error::Missing)?;
    let version = blake3::hash(body.as_bytes()).to_hex().to_string();
    let source = Source {
        owner: owner.clone(),
        resource: resource.to_string(),
        version,
        content_digest: [0; 32],
        start,
        end,
        title: node.title.clone(),
    };
    crate::workshop::capture(source, body, true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workshop::{Workshop, evaluate};
    use alembic::workshop::Destination;
    use muniment::MemoryBackend;
    use pandect::graph_session::MereSessions;
    use servitor::{Cap, Mode, Subject};

    struct Authority(bool);
    impl AuthorityProvider for Authority {
        fn covers(&self, _: Subject, _: &Cap, _: Mode) -> bool {
            self.0
        }
    }

    #[test]
    fn operation_snapshot_revision_binds_equal_count_graph_content() {
        use kernel::graph::apply::apply_graph_delta;
        let mut a = Graph::default();
        let mut b = Graph::default();
        let id = Uuid::from_u128(55);
        for (graph, title) in [(&mut a, "first"), (&mut b, "second")] {
            apply_graph_delta(
                graph,
                GraphDelta::AddNode {
                    id: Some(id),
                    url: "https://example.test/same".into(),
                    position: euclid::point2(0.0, 0.0),
                },
            );
            apply_graph_delta(
                graph,
                GraphDelta::ReplaySetNodeTitleById {
                    node_id: id,
                    title: title.into(),
                },
            );
        }
        assert_eq!(a.node_count(), b.node_count());
        assert_ne!(graph_revision(&a).unwrap(), graph_revision(&b).unwrap());
        // A recorded graph is the snapshot plus its facet sidecar. Snapshot
        // reconstruction alone assigns a new wall-clock visit timestamp.
        let facet_id = chartulary::FacetId::new(kernel::graph::node_facets::VISIT_HISTORY);
        let sentinel = serde_json::json!({"last_visited_ms": 1, "last_session_visited": 7});
        let mut facets = a.facets().clone();
        facets
            .set(
                id,
                facet_id.clone(),
                sentinel.clone(),
                &chartulary::AcceptAll,
            )
            .unwrap();
        a.install_facets(facets.clone()).unwrap();
        let mut restored = Graph::try_from_recorded_snapshot(&a.to_snapshot_at(0)).unwrap();
        assert_ne!(restored.facets().get(&id, &facet_id), Some(&sentinel));
        restored.install_facets(facets).unwrap();
        assert_eq!(restored.facets().get(&id, &facet_id), Some(&sentinel));
        assert_eq!(
            graph_revision(&a).unwrap(),
            graph_revision(&restored).unwrap()
        );

        // A missing visit facet must not become a new clock-dependent value
        // just because revision preparation reconstructs the snapshot.
        let mut absent = a.facets().clone();
        absent.remove(&id, &facet_id);
        a.install_facets(absent.clone()).unwrap();
        let first = graph_revision(&a).unwrap();
        #[cfg(not(target_arch = "wasm32"))]
        std::thread::sleep(std::time::Duration::from_millis(5));
        assert_eq!(first, graph_revision(&a).unwrap());
        let mut restored_absent = Graph::try_from_recorded_snapshot(&a.to_snapshot_at(0)).unwrap();
        restored_absent.install_facets(absent).unwrap();
        assert_eq!(first, graph_revision(&restored_absent).unwrap());
    }

    #[test]
    fn operation_public_graph_to_durable_note_and_reconciliation() {
        pollster::block_on(async {
            let store = MemoryBackend::new();
            let mere = MereSessions::new(store.clone());
            let manifest = mere
                .mint(Author::user(), Some("Research".into()))
                .await
                .unwrap();
            let mut session = mere.open(manifest.session_id).await.unwrap();
            let owner = Owner {
                host: "fixture".into(),
                workspace: "public".into(),
                session: session.id().as_uuid().to_string(),
            };
            let first = Uuid::from_u128(1);
            let second = Uuid::from_u128(2);
            let texts = [
                (first, "One", "First source."),
                (second, "Two", "A café source."),
            ];
            let mut deltas = Vec::new();
            for (id, title, body) in texts {
                deltas.push(GraphDelta::AddNode {
                    id: Some(id),
                    url: format!("https://example.test/{id}"),
                    position: euclid::point2(0.0, 0.0),
                });
                deltas.push(GraphDelta::ReplaySetNodeTitleById {
                    node_id: id,
                    title: title.into(),
                });
                deltas.push(GraphDelta::ReplaySetNodeBodyById {
                    node_id: id,
                    body: Some(body.into()),
                });
            }
            session
                .transact_deltas_published(Author::user(), deltas, SystemTime::now())
                .await
                .unwrap();
            assert!(capture_public_source(&session, &owner, first, 0, 3, |_, _| false).is_err());
            let sources = texts
                .into_iter()
                .map(|(id, _, body)| {
                    capture_public_source(&session, &owner, id, 0, body.len() as u64, |_, _| true)
                        .unwrap()
                })
                .collect();
            let shop = Workshop::new(store, owner.clone(), [3; 16]);
            shop.install([7; 32]).await.unwrap();
            shop.begin(1, sources, &Authority(true), 10).await.unwrap();
            for i in 0..3 {
                let dispatch = shop
                    .dispatch(1, &Authority(true), 11 + 2 * i)
                    .await
                    .unwrap();
                shop.observe(evaluate(dispatch).unwrap(), &Authority(true), 12 + 2 * i)
                    .await
                    .unwrap();
            }
            let proposal = shop.view(Some(1)).await.unwrap().proposal.unwrap();
            let target = Destination {
                owner: owner.clone(),
                graph: manifest.root_graph_id.as_uuid().to_string(),
                revision: revision(&session).unwrap(),
            };
            let acceptance = shop
                .accept(1, target, [8; 32], &Authority(true), 20)
                .await
                .unwrap();
            assert!(
                commit(
                    &mut session,
                    &owner,
                    &acceptance,
                    &proposal,
                    [8; 32],
                    &Authority(false),
                    21,
                    true
                )
                .await
                .is_err()
            );
            assert!(
                query(&session, &owner, &acceptance, &proposal)
                    .await
                    .unwrap()
                    .is_none()
            );
            let receipt = commit(
                &mut session,
                &owner,
                &acceptance,
                &proposal,
                [8; 32],
                &Authority(true),
                21,
                true,
            )
            .await
            .unwrap();
            let reopened = mere.open(manifest.session_id).await.unwrap();
            assert_eq!(
                query(&reopened, &owner, &acceptance, &proposal)
                    .await
                    .unwrap(),
                Some(receipt.clone())
            );
            let graph_id = manifest.root_graph_id.as_uuid().to_string();
            verify_graph_effect(reopened.graph(), &owner, &graph_id, &acceptance, &proposal)
                .unwrap();
            let embedded = query_graph(reopened.graph(), &owner, &graph_id, &acceptance, &proposal)
                .unwrap()
                .unwrap();
            assert_eq!(embedded.resource, receipt.resource);
            let snapshot = reopened.graph().to_snapshot_at(0);
            assert!(
                snapshot
                    .nodes
                    .iter()
                    .any(|node| node.body.as_deref() == Some(proposal.djot.as_str()))
            );
            let mut reopened = reopened;
            let recovered = commit(
                &mut reopened,
                &owner,
                &acceptance,
                &proposal,
                [8; 32],
                &Authority(false),
                u64::MAX,
                false,
            )
            .await
            .unwrap();
            assert_eq!(receipt, recovered);
            let before_delete =
                Graph::try_from_recorded_snapshot(&reopened.graph().to_snapshot_at(0)).unwrap();
            let (node_id, _, _) = identifiers(&acceptance);
            let key = reopened.graph().get_node_by_id(node_id).unwrap().0;
            reopened
                .transact_deltas_published(
                    Author::user(),
                    vec![GraphDelta::RemoveNode { key }],
                    SystemTime::now(),
                )
                .await
                .unwrap();
            let after_delete = mere.open(manifest.session_id).await.unwrap();
            assert!(!graph_result_present(after_delete.graph(), &acceptance));
            preserve_graph_operations(&before_delete, after_delete.graph()).unwrap();
            let mut gc_candidate =
                Graph::try_from_recorded_snapshot(&after_delete.graph().to_snapshot_at(0)).unwrap();
            let (_, ledger_id) = ledger_identity(&acceptance);
            kernel::graph::apply::apply_graph_delta(
                &mut gc_candidate,
                GraphDelta::ReplaySetResourceRecordById {
                    resource_id: ledger_id,
                    record: None,
                },
            );
            assert_eq!(
                preserve_graph_operations(after_delete.graph(), &gc_candidate),
                Err(Error::Conflict)
            );
            assert_eq!(
                query_graph(
                    after_delete.graph(),
                    &owner,
                    &graph_id,
                    &acceptance,
                    &proposal
                )
                .unwrap(),
                Some(embedded.clone())
            );
            let duplicate = prepare_graph(
                after_delete.graph(),
                &owner,
                &graph_id,
                "later revision",
                &acceptance,
                &proposal,
                [8; 32],
                &Authority(false),
                u64::MAX,
                false,
            )
            .unwrap();
            assert!(matches!(duplicate, GraphPreparation::Existing(found) if found == embedded));
            let mut changed = acceptance.clone();
            changed.human = [9; 32];
            assert_eq!(
                query(&reopened, &owner, &changed, &proposal).await,
                Err(Error::Conflict)
            );
        });
    }
}
