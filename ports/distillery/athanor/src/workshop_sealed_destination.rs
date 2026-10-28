// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Private evidence briefs in the graph owner's existing snapshot transaction.
//! The owner must serialize graph writes and custody changes through the save;
//! these synchronous gates alone do not provide a lock against an external
//! custodian. Prepared deltas are never a durability acknowledgement.

use crate::workshop::{Error, revalidate_acceptance, validate_private_proposal};
use crate::workshop_destination::{
    graph_revision, identifiers, ledger_identity, ledger_identity_for, require_complete,
};
use alembic::workshop::{Acceptance, Owner, Proposal, Receipt};
use kernel::graph::{Graph, apply::GraphDelta};
use kernel::persistence::{PersistedResourceFacet, PersistedResourceRecord};
use servitor::AuthorityProvider;

pub(crate) const OPERATION_FACET: &str = "mere.alembic.evidence-brief.sealed-operation/v1";
const NOTE_FACET: &str = "mere.alembic.evidence-brief.sealed-note/v1";
const PLACEHOLDER: &str = "Private evidence brief. Open through its owner's authorized reader.";

pub use crate::workshop_seal::Protection;

fn encoded<T: serde::Serialize>(value: &T) -> Result<Vec<u8>, Error> {
    serde_json::to_vec(value).map_err(|_| Error::Invalid("private destination encoding"))
}
fn request(
    graph_id: &str,
    owner: &Owner,
    acceptance: &Acceptance,
    proposal: &Proposal,
) -> Result<Vec<u8>, Error> {
    validate_private_proposal(proposal)?;
    let id = blake3::hash(&encoded(&(
        "alembic-acceptance-v1",
        owner,
        acceptance.installation,
        acceptance.run,
        proposal.digest,
    ))?);
    if acceptance.operation != *id.as_bytes()
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
// Routing contains only stable namespace/correlation identifiers. It lets the
// human's retained note outlive the helper's private journal. These identifiers
// are authenticated by the encryption context; changing one cannot reroute bytes.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Routing {
    schema: u8,
    owner: Owner,
    operation: [u8; 32],
    installation: [u8; 16],
    run: u64,
}
impl Routing {
    fn new(owner: &Owner, acceptance: &Acceptance) -> Self {
        Self {
            schema: 1,
            owner: owner.clone(),
            operation: acceptance.operation,
            installation: acceptance.installation,
            run: acceptance.run,
        }
    }
    fn context(&self, kind: &str) -> Result<Vec<u8>, Error> {
        encoded(&(
            "alembic-private-destination-v1",
            kind,
            &self.owner,
            self.operation,
            self.installation,
            self.run,
        ))
    }
}
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Envelope {
    routing: Routing,
    ciphertext: Vec<u8>,
}
fn seal_entry(
    owner: &Owner,
    acceptance: &Acceptance,
    kind: &str,
    clear: &[u8],
    protection: &(impl Protection + ?Sized),
) -> Result<Vec<u8>, Error> {
    let routing = Routing::new(owner, acceptance);
    let ciphertext = protection.seal(&routing.context(kind)?, clear)?;
    encoded(&Envelope {
        routing,
        ciphertext,
    })
}
fn envelope(bytes: &[u8]) -> Result<Envelope, Error> {
    if bytes.len() > 4 * 1024 * 1024 {
        return Err(Error::Conflict);
    }
    let value: Envelope = serde_json::from_slice(bytes).map_err(|_| Error::Conflict)?;
    if value.routing.schema != 1 {
        return Err(Error::Conflict);
    }
    Ok(value)
}
fn open_entry(
    owner: &Owner,
    acceptance: &Acceptance,
    kind: &str,
    bytes: &[u8],
    protection: &(impl Protection + ?Sized),
) -> Result<Vec<u8>, Error> {
    let value = envelope(bytes)?;
    if value.routing != Routing::new(owner, acceptance) {
        return Err(Error::Conflict);
    }
    protection.open(&value.routing.context(kind)?, &value.ciphertext)
}
fn gate(p: &(impl Protection + ?Sized)) -> Result<u64, Error> {
    p.epoch().ok_or(Error::Unauthorized)
}
fn same_epoch(p: &(impl Protection + ?Sized), epoch: u64) -> Result<(), Error> {
    if p.epoch() == Some(epoch) {
        Ok(())
    } else {
        Err(Error::Unauthorized)
    }
}

fn facet(graph: &Graph, iri: &str, name: &str) -> Result<Option<Vec<u8>>, Error> {
    let snapshot = graph.to_snapshot_at(0);
    let Some(resource) = snapshot.resources.iter().find(|r| r.canonical_iri == iri) else {
        return Ok(None);
    };
    let facet = resource
        .facets
        .iter()
        .find(|f| f.facet == name)
        .ok_or(Error::Conflict)?;
    serde_json::from_str(&facet.value_json)
        .map(Some)
        .map_err(|_| Error::Conflict)
}
fn record(iri: String, facet: &str, ciphertext: &[u8]) -> Result<PersistedResourceRecord, Error> {
    Ok(PersistedResourceRecord {
        canonical_iri: iri,
        facets: vec![PersistedResourceFacet {
            facet: facet.into(),
            value_json: String::from_utf8(encoded(&ciphertext)?)
                .map_err(|_| Error::Invalid("private ciphertext encoding"))?,
        }],
    })
}
fn witness(request: &[u8], resource: &str) -> Result<String, Error> {
    Ok(blake3::hash(&encoded(&(
        "alembic-private-operation-v1",
        request,
        resource,
    ))?)
    .to_hex()
    .to_string())
}

/// Query only after fresh custody admission. A removed note does not remove its
/// operation receipt; an exact conflicting request never receives that receipt.
pub fn query_graph(
    graph: &Graph,
    owner: &Owner,
    graph_id: &str,
    acceptance: &Acceptance,
    proposal: &Proposal,
    protection: &(impl Protection + ?Sized),
) -> Result<Option<Receipt>, Error> {
    require_complete(graph)?;
    let epoch = gate(protection)?;
    let request = request(graph_id, owner, acceptance, proposal)?;
    let (_, _, resource) = identifiers(acceptance);
    let (ledger_iri, _) = ledger_identity(acceptance);
    let Some(cipher) = facet(graph, &ledger_iri, OPERATION_FACET)? else {
        same_epoch(protection, epoch)?;
        return Ok(None);
    };
    let opened = open_entry(owner, acceptance, "operation", &cipher, protection)?;
    same_epoch(protection, epoch)?;
    if opened != request {
        return Err(Error::Conflict);
    }
    Ok(Some(Receipt {
        operation: acceptance.operation,
        proposal: acceptance.proposal,
        destination: acceptance.destination.clone(),
        storage_witness: witness(&request, &resource)?,
        resource,
    }))
}

pub enum Preparation {
    Existing {
        receipt: Receipt,
        epoch: u64,
    },
    New {
        deltas: Vec<GraphDelta>,
        receipt: Receipt,
        epoch: u64,
    },
}

/// Prepare a single snapshot mutation. The owner validates the candidate and
/// atomically saves it under the same exclusive graph and custody admission.
pub fn prepare_graph(
    graph: &Graph,
    owner: &Owner,
    graph_id: &str,
    acceptance: &Acceptance,
    proposal: &Proposal,
    human: [u8; 32],
    authority: &impl AuthorityProvider,
    now_ms: u64,
    protection: &(impl Protection + ?Sized),
) -> Result<Preparation, Error> {
    let epoch = gate(protection)?;
    let request = request(graph_id, owner, acceptance, proposal)?;
    if human != acceptance.human {
        return Err(Error::Unauthorized);
    }
    if let Some(receipt) = query_graph(graph, owner, graph_id, acceptance, proposal, protection)? {
        same_epoch(protection, epoch)?;
        return Ok(Preparation::Existing { receipt, epoch });
    }
    same_epoch(protection, epoch)?;
    revalidate_acceptance(acceptance, human, authority, now_ms)?;
    if acceptance.destination.revision != graph_revision(graph)? {
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
    let sealed_request = seal_entry(owner, acceptance, "operation", &request, protection)?;
    same_epoch(protection, epoch)?;
    let sealed_note = seal_entry(
        owner,
        acceptance,
        "note",
        proposal.djot.as_bytes(),
        protection,
    )?;
    same_epoch(protection, epoch)?;
    let resource_id = chartulary::resource_id_from_canonical_iri(&chartulary::canonical_url(&iri));
    let deltas = vec![
        GraphDelta::AddNode {
            id: Some(node_id),
            url: iri.clone(),
            position: euclid::point2(0.0, 0.0),
        },
        GraphDelta::ReplaySetNodeTitleById {
            node_id,
            title: "Private evidence brief".into(),
        },
        GraphDelta::ReplaySetNodeBodyById {
            node_id,
            body: Some(PLACEHOLDER.into()),
        },
        GraphDelta::ReplaySetResourceRecordById {
            resource_id,
            record: Some(record(
                chartulary::canonical_url(&iri),
                NOTE_FACET,
                &sealed_note,
            )?),
        },
        GraphDelta::ReplaySetResourceRecordById {
            resource_id: ledger_id,
            record: Some(record(ledger_iri, OPERATION_FACET, &sealed_request)?),
        },
    ];
    let receipt = Receipt {
        operation: acceptance.operation,
        proposal: acceptance.proposal,
        destination: acceptance.destination.clone(),
        storage_witness: witness(&request, &resource)?,
        resource,
    };
    same_epoch(protection, epoch)?;
    Ok(Preparation::New {
        deltas,
        receipt,
        epoch,
    })
}

/// Read the retained note only after the exact operation and its owner have
/// been admitted. A deleted surface is not recreated from operation evidence.
pub fn open_note(
    graph: &Graph,
    owner: &Owner,
    graph_id: &str,
    acceptance: &Acceptance,
    proposal: &Proposal,
    protection: &(impl Protection + ?Sized),
) -> Result<String, Error> {
    let epoch = gate(protection)?;
    if query_graph(graph, owner, graph_id, acceptance, proposal, protection)?.is_none() {
        return Err(Error::Missing);
    }
    let (node, iri, resource) = identifiers(acceptance);
    let key = graph.get_node_by_id(node).ok_or(Error::Missing)?.0;
    if graph
        .shown_resource_id(key)
        .map(|v| v.to_string())
        .as_deref()
        != Some(resource.as_str())
    {
        return Err(Error::Conflict);
    }
    let bytes =
        facet(graph, &chartulary::canonical_url(&iri), NOTE_FACET)?.ok_or(Error::Missing)?;
    let clear = open_entry(owner, acceptance, "note", &bytes, protection)?;
    same_epoch(protection, epoch)?;
    if clear != proposal.djot.as_bytes() {
        return Err(Error::Conflict);
    }
    String::from_utf8(clear).map_err(|_| Error::Conflict)
}

/// Read a human-owned result using only the retained graph and operation ID.
/// No helper installation, run journal, or live helper is required. Custody and
/// exact graph/owner admission are still required, and a deleted note stays gone.
pub fn open_retained_note(
    graph: &Graph,
    owner: &Owner,
    graph_id: &str,
    operation: &[u8; 32],
    protection: &(impl Protection + ?Sized),
) -> Result<(Receipt, String), Error> {
    require_complete(graph)?;
    let epoch = gate(protection)?;
    let (ledger, _) = ledger_identity_for(operation);
    let bytes = facet(graph, &ledger, OPERATION_FACET)?.ok_or(Error::Missing)?;
    let value = envelope(&bytes)?;
    if value.routing.owner != *owner || value.routing.operation != *operation {
        return Err(Error::Conflict);
    }
    let clear = protection.open(&value.routing.context("operation")?, &value.ciphertext)?;
    same_epoch(protection, epoch)?;
    let (acceptance, proposal): (Acceptance, Proposal) =
        serde_json::from_slice(&clear).map_err(|_| Error::Conflict)?;
    if value.routing != Routing::new(owner, &acceptance)
        || request(graph_id, owner, &acceptance, &proposal)? != clear
    {
        return Err(Error::Conflict);
    }
    let receipt = query_graph(graph, owner, graph_id, &acceptance, &proposal, protection)?
        .ok_or(Error::Missing)?;
    let note = open_note(graph, owner, graph_id, &acceptance, &proposal, protection)?;
    same_epoch(protection, epoch)?;
    Ok((receipt, note))
}

/// Open a retained result by its Resource identity, independently of helper
/// state. Routing is only a lookup hint until both encrypted records and the
/// original result surface are checked by the owner's private reader.
pub fn open_result_resource(
    graph: &Graph,
    owner: &Owner,
    graph_id: &str,
    resource: &str,
    protection: &(impl Protection + ?Sized),
) -> Result<(Receipt, String), Error> {
    require_complete(graph)?;
    let epoch = gate(protection)?;
    let snapshot = graph.to_snapshot_at(0);
    let record = snapshot
        .resources
        .iter()
        .find(|record| {
            chartulary::resource_id_from_canonical_iri(&record.canonical_iri).to_string()
                == resource
        })
        .ok_or(Error::Missing)?;
    let stored = record
        .facets
        .iter()
        .find(|facet| facet.facet == NOTE_FACET)
        .ok_or(Error::Missing)?;
    let bytes: Vec<u8> = serde_json::from_str(&stored.value_json).map_err(|_| Error::Conflict)?;
    let routing = envelope(&bytes)?.routing;
    if routing.owner != *owner {
        return Err(Error::Conflict);
    }
    let result = open_retained_note(graph, owner, graph_id, &routing.operation, protection)?;
    same_epoch(protection, epoch)?;
    if result.0.resource != resource {
        return Err(Error::Conflict);
    }
    Ok(result)
}

/// Check both ledger and retained note in the staged candidate before the
/// owner persists it. This is a structural check, not a storage receipt.
pub fn verify_graph_effect(
    graph: &Graph,
    owner: &Owner,
    graph_id: &str,
    acceptance: &Acceptance,
    proposal: &Proposal,
    protection: &(impl Protection + ?Sized),
) -> Result<(), Error> {
    open_note(graph, owner, graph_id, acceptance, proposal, protection)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use alembic::workshop::{Destination, Passage, Source};
    use kernel::graph::apply::{GraphDeltaResult, apply_graph_delta};
    use servitor::{Cap, Mode, Subject};
    use std::sync::atomic::{AtomicBool, Ordering};

    // Deterministic test double, not a cryptographic implementation.
    struct TestProtection {
        open: AtomicBool,
    }
    impl Protection for TestProtection {
        fn epoch(&self) -> Option<u64> {
            self.open.load(Ordering::SeqCst).then_some(1)
        }
        fn seal(&self, context: &[u8], clear: &[u8]) -> Result<Vec<u8>, Error> {
            gate(self)?;
            let mut value = blake3::hash(&[context, clear].concat()).as_bytes().to_vec();
            value.extend(clear.iter().map(|b| b ^ 0xa5));
            Ok(value)
        }
        fn open(&self, context: &[u8], sealed: &[u8]) -> Result<Vec<u8>, Error> {
            gate(self)?;
            if sealed.len() < 32 {
                return Err(Error::Conflict);
            }
            let clear: Vec<_> = sealed[32..].iter().map(|b| b ^ 0xa5).collect();
            if sealed[..32] != *blake3::hash(&[context, &clear].concat()).as_bytes() {
                return Err(Error::Conflict);
            }
            Ok(clear)
        }
    }
    struct Authority(bool);
    impl AuthorityProvider for Authority {
        fn covers(&self, _: Subject, _: &Cap, _: Mode) -> bool {
            self.0
        }
    }
    fn fixtures(graph: &Graph) -> (Owner, Proposal, Acceptance) {
        let owner = Owner {
            host: "turnstone".into(),
            workspace: "workspace".into(),
            session: "session".into(),
        };
        let text = "PRIVATE-CANARY-λ-381";
        let passages = (0..2)
            .map(|i| Passage {
                source: Source {
                    owner: owner.clone(),
                    resource: format!("source-{i}"),
                    version: "v1".into(),
                    content_digest: *blake3::hash(text.as_bytes()).as_bytes(),
                    start: 0,
                    end: text.len() as u64,
                    title: format!("source {i}"),
                },
                text: text.into(),
                text_digest: *blake3::hash(text.as_bytes()).as_bytes(),
                public: i == 0,
            })
            .collect::<Vec<_>>();
        let proposal = crate::workshop::proposal([3; 16], 1, &passages).unwrap();
        let operation = *blake3::hash(
            &encoded(&(
                "alembic-acceptance-v1",
                &owner,
                proposal.installation,
                proposal.run,
                proposal.digest,
            ))
            .unwrap(),
        )
        .as_bytes();
        let acceptance = Acceptance {
            operation,
            proposal: proposal.digest,
            destination: Destination {
                owner: owner.clone(),
                graph: "graph".into(),
                revision: graph_revision(graph).unwrap(),
            },
            human: [8; 32],
            installation: proposal.installation,
            run: proposal.run,
            proposing_subject: [7; 32],
            requested_at_ms: 10,
            deadline_at_ms: 30_010,
        };
        (owner, proposal, acceptance)
    }
    fn apply(graph: &mut Graph, deltas: Vec<GraphDelta>) {
        for delta in deltas {
            assert!(!matches!(
                apply_graph_delta(graph, delta),
                GraphDeltaResult::Refused(_)
            ));
        }
    }
    #[test]
    fn custody_epoch_change_during_preparation_refuses_before_effect() {
        struct ChangesEpoch(std::sync::atomic::AtomicU64);
        impl Protection for ChangesEpoch {
            fn epoch(&self) -> Option<u64> {
                Some(self.0.load(Ordering::SeqCst))
            }
            fn seal(&self, _: &[u8], _: &[u8]) -> Result<Vec<u8>, Error> {
                self.0.fetch_add(1, Ordering::SeqCst);
                Ok(vec![1])
            }
            fn open(&self, _: &[u8], _: &[u8]) -> Result<Vec<u8>, Error> {
                Err(Error::Conflict)
            }
        }
        let graph = Graph::default();
        let (owner, proposal, acceptance) = fixtures(&graph);
        let protection = ChangesEpoch(std::sync::atomic::AtomicU64::new(1));
        assert!(matches!(
            prepare_graph(
                &graph,
                &owner,
                "graph",
                &acceptance,
                &proposal,
                [8; 32],
                &Authority(true),
                11,
                &protection
            ),
            Err(Error::Unauthorized)
        ));
        assert_eq!(graph.node_count(), 0);
    }
    #[test]
    fn mixed_private_commit_reopens_without_cleartext_and_recovers_after_deletion() {
        let mut graph = Graph::default();
        let (owner, proposal, acceptance) = fixtures(&graph);
        let protection = TestProtection {
            open: AtomicBool::new(true),
        };
        let Preparation::New {
            deltas, receipt, ..
        } = prepare_graph(
            &graph,
            &owner,
            "graph",
            &acceptance,
            &proposal,
            [8; 32],
            &Authority(true),
            11,
            &protection,
        )
        .unwrap()
        else {
            panic!("new operation expected")
        };
        apply(&mut graph, deltas);
        let snapshot = graph.to_snapshot_at(0);
        let bytes = encoded(&(snapshot.clone(), graph.facets())).unwrap();
        assert!(
            !bytes
                .windows("PRIVATE-CANARY-λ-381".len())
                .any(|w| w == "PRIVATE-CANARY-λ-381".as_bytes())
        );
        let mut reopened = Graph::try_from_recorded_snapshot(&snapshot).unwrap();
        reopened.install_facets(graph.facets().clone()).unwrap();
        assert_eq!(
            query_graph(
                &reopened,
                &owner,
                "graph",
                &acceptance,
                &proposal,
                &protection
            )
            .unwrap(),
            Some(receipt.clone())
        );
        assert_eq!(
            open_note(
                &reopened,
                &owner,
                "graph",
                &acceptance,
                &proposal,
                &protection
            )
            .unwrap(),
            proposal.djot
        );
        assert_eq!(
            open_retained_note(
                &reopened,
                &owner,
                "graph",
                &acceptance.operation,
                &protection
            )
            .unwrap(),
            (receipt.clone(), proposal.djot.clone())
        );
        assert_eq!(
            open_result_resource(&reopened, &owner, "graph", &receipt.resource, &protection)
                .unwrap(),
            (receipt.clone(), proposal.djot.clone())
        );
        assert!(
            open_result_resource(&reopened, &owner, "graph", "unknown resource", &protection)
                .is_err()
        );
        let mut other_owner = owner.clone();
        other_owner.session = "different session".into();
        assert!(
            open_retained_note(
                &reopened,
                &other_owner,
                "graph",
                &acceptance.operation,
                &protection
            )
            .is_err()
        );
        assert!(
            open_retained_note(
                &reopened,
                &owner,
                "another graph",
                &acceptance.operation,
                &protection
            )
            .is_err()
        );
        assert!(
            matches!(prepare_graph(&reopened, &owner, "graph", &acceptance, &proposal, [8; 32], &Authority(false), u64::MAX, &protection).unwrap(), Preparation::Existing { receipt: r, .. } if r == receipt)
        );
        let before = reopened.clone();
        let node = identifiers(&acceptance).0;
        let key = reopened.get_node_by_id(node).unwrap().0;
        apply(&mut reopened, vec![GraphDelta::RemoveNode { key }]);
        crate::workshop_destination::preserve_graph_operations(&before, &reopened).unwrap();
        assert_eq!(
            query_graph(
                &reopened,
                &owner,
                "graph",
                &acceptance,
                &proposal,
                &protection
            )
            .unwrap(),
            Some(receipt)
        );
        assert!(matches!(
            open_note(
                &reopened,
                &owner,
                "graph",
                &acceptance,
                &proposal,
                &protection
            ),
            Err(Error::Missing)
        ));
        assert!(matches!(
            open_retained_note(
                &reopened,
                &owner,
                "graph",
                &acceptance.operation,
                &protection
            ),
            Err(Error::Missing)
        ));
        let (_, ledger_id) = ledger_identity(&acceptance);
        let saved = reopened.clone();
        apply(
            &mut reopened,
            vec![GraphDelta::ReplaySetResourceRecordById {
                resource_id: ledger_id,
                record: None,
            }],
        );
        assert_eq!(
            crate::workshop_destination::preserve_graph_operations(&saved, &reopened),
            Err(Error::Conflict)
        );
    }
    #[test]
    fn custody_context_conflict_and_tampering_refuse() {
        let mut graph = Graph::default();
        let (owner, proposal, acceptance) = fixtures(&graph);
        let protection = TestProtection {
            open: AtomicBool::new(true),
        };
        let Preparation::New { deltas, .. } = prepare_graph(
            &graph,
            &owner,
            "graph",
            &acceptance,
            &proposal,
            [8; 32],
            &Authority(true),
            11,
            &protection,
        )
        .unwrap() else {
            panic!("new operation expected")
        };
        apply(&mut graph, deltas);
        let (ledger_iri, ledger_id) = ledger_identity(&acceptance);
        let mut moved = graph.clone();
        let mut routed = envelope(
            &facet(&graph, &ledger_iri, OPERATION_FACET)
                .unwrap()
                .unwrap(),
        )
        .unwrap();
        routed.routing.run += 1;
        apply(
            &mut moved,
            vec![GraphDelta::ReplaySetResourceRecordById {
                resource_id: ledger_id,
                record: Some(
                    record(ledger_iri, OPERATION_FACET, &encoded(&routed).unwrap()).unwrap(),
                ),
            }],
        );
        assert!(
            open_retained_note(&moved, &owner, "graph", &acceptance.operation, &protection)
                .is_err()
        );
        let mut changed = acceptance.clone();
        changed.human = [9; 32];
        assert!(query_graph(&graph, &owner, "graph", &changed, &proposal, &protection).is_err());
        let mut wrong_installation = acceptance.clone();
        wrong_installation.installation = [4; 16];
        assert!(
            query_graph(
                &graph,
                &owner,
                "graph",
                &wrong_installation,
                &proposal,
                &protection
            )
            .is_err()
        );
        let mut wrong_owner = owner.clone();
        wrong_owner.session = "other".into();
        assert!(
            query_graph(
                &graph,
                &wrong_owner,
                "graph",
                &acceptance,
                &proposal,
                &protection
            )
            .is_err()
        );
        assert!(
            protection
                .open(
                    b"different context",
                    &protection.seal(b"correct context", b"secret").unwrap()
                )
                .is_err()
        );
        protection.open.store(false, Ordering::SeqCst);
        assert!(matches!(
            query_graph(&graph, &owner, "graph", &acceptance, &proposal, &protection),
            Err(Error::Unauthorized)
        ));
        protection.open.store(true, Ordering::SeqCst);
        let (_, note_iri, _) = identifiers(&acceptance);
        let note_iri = chartulary::canonical_url(&note_iri);
        let note_id = chartulary::resource_id_from_canonical_iri(&note_iri);
        let mut note_tampered = graph.clone();
        let mut note = facet(&graph, &note_iri, NOTE_FACET).unwrap().unwrap();
        note[0] ^= 1;
        apply(
            &mut note_tampered,
            vec![GraphDelta::ReplaySetResourceRecordById {
                resource_id: note_id,
                record: Some(record(note_iri, NOTE_FACET, &note).unwrap()),
            }],
        );
        assert!(
            open_note(
                &note_tampered,
                &owner,
                "graph",
                &acceptance,
                &proposal,
                &protection
            )
            .is_err()
        );
        let (iri, ledger_id) = ledger_identity(&acceptance);
        let mut sealed = facet(&graph, &iri, OPERATION_FACET).unwrap().unwrap();
        sealed[0] ^= 1;
        apply(
            &mut graph,
            vec![GraphDelta::ReplaySetResourceRecordById {
                resource_id: ledger_id,
                record: Some(record(iri, OPERATION_FACET, &sealed).unwrap()),
            }],
        );
        assert!(query_graph(&graph, &owner, "graph", &acceptance, &proposal, &protection).is_err());
    }
}
