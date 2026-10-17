// Audit-only public consumer of the freshly compiled, unchanged complete Gemot owner.
use gemot_audit::moot::*;
use gemot_audit::moot::constitution::{AmendmentRule, ConstitutionRules};
use identity::{IdentityProvider, InMemoryProvider};
use mooting::{ElectorateSnapshot, RecognitionContext, RecognitionPolicy};
use std::collections::BTreeSet;
use std::future::Future;
use std::task::{Context, Poll, Waker};

const ID: MootId = MootId([0x4e; 32]);
fn ready<F: Future>(future: F) -> F::Output {
    let mut future = std::pin::pin!(future);
    let mut context = Context::from_waker(Waker::noop());
    for _ in 0..100_000 {
        if let Poll::Ready(value) = future.as_mut().poll(&mut context) { return value; }
        std::thread::yield_now();
    }
    panic!("sequential in-memory diagnostic needs unexpected external progress");
}
fn person(seed: u8) -> InMemoryProvider { InMemoryProvider::from_seed([seed; 32]) }
fn root(seed: u8) -> [u8; 32] { person(seed).master_public_key().to_bytes() }
fn retention() -> MootRetentionSettings {
    MootRetentionSettings {
        revision: PolicyRevision(proofs::Digest::blake3(b"audit retention")),
        availability: AvailabilityPolicy { promised_floor: KeepBound::Forever },
        erasure: ErasurePolicy { history_ceiling: KeepBound::UntilCheckpoint },
    }
}
fn recognition() -> RecognitionContext {
    RecognitionContext::new(RecognitionPolicy::AnyEligible,
        ElectorateSnapshot::new(ID.0, [0x55; 32], [root(2)]))
}
async fn proposal(store: &TulpaStore<muniment::MemoryBackend>, id: u8, version: TulpaVersion, artifact: ArtifactRef, at_ms: u64) {
    store.author_seed([2; 32], ID.0, &TulpaEvent::Proposed {
        proposal: TulpaProposalId([id; 32]),
        action: TulpaProposal::Adopt { tulpa: TulpaId([3; 32]), version, artifact },
        recognition: recognition(), at_ms,
    }).await.unwrap();
    store.author_seed([2; 32], ID.0, &TulpaEvent::Endorsed {
        proposal: TulpaProposalId([id; 32]), at_ms: at_ms+1,
    }).await.unwrap();
}
async fn artifact_case(conflict: bool) {
    let store = TulpaStore::in_memory();
    let old = ArtifactRef::blake3(b"old bytes");
    let new = ArtifactRef::blake3(b"new bytes");
    let version = TulpaVersion([4; 32]);
    proposal(&store, 1, version, old.clone(), 10).await;
    proposal(&store, 2, if conflict { version } else { TulpaVersion([5; 32]) }, new.clone(), 20).await;
    let result = store.fold_moot(ID.0).await.unwrap();
    assert_eq!(result.facts.len(), 4);
    assert!(result.decisions.values().all(|decision| decision.accepted));
    let adopted = result.adopted.get(&TulpaId([3; 32])).unwrap();
    assert_eq!(adopted.proposal, TulpaProposalId([2; 32]));
    let op = store.get(&p2panda_core::Hash::from_bytes(adopted.operation)).await.unwrap().unwrap();
    assert!(gemot_audit::moot::tulpa::verify(&op));
    let TulpaEvent::Proposed { action: TulpaProposal::Adopt { artifact: cited, .. }, .. } =
        gemot_audit::moot::tulpa::from_operation(&op).unwrap() else { panic!("wrong cited event") };
    assert_eq!(cited, new);
    if conflict {
        assert_eq!(adopted.artifact, old);
        assert_ne!(adopted.artifact, cited);
        println!("PASS same_version_conflict: effective citation names new artifact; projected artifact is old");
    } else {
        assert_eq!(adopted.artifact, cited);
        println!("PASS distinct_version_control");
    }
}
async fn self_elected_aggregate() {
    let service = Moot::in_memory(ID, root(1), retention());
    service.found([1; 32], None, None, ConstitutionRules::founder_only(root(1)), 1).await.unwrap();
    assert!(service.snapshot().await.unwrap().membership.members.is_empty());
    service.record_tulpa([2; 32], TulpaEvent::Proposed {
        proposal: TulpaProposalId([6; 32]),
        action: TulpaProposal::Adopt { tulpa: TulpaId([7; 32]), version: TulpaVersion([8; 32]), artifact: ArtifactRef::blake3(b"outsider choice") },
        recognition: recognition(), at_ms: 2,
    }).await.unwrap();
    let result = service.record_tulpa([2; 32], TulpaEvent::Endorsed {
        proposal: TulpaProposalId([6; 32]), at_ms: 3,
    }).await.unwrap();
    assert!(result.snapshot.membership.members.is_empty());
    assert_eq!(result.snapshot.governance.founder, root(1));
    assert_eq!(result.snapshot.tulpa.adopted.len(), 1);
    assert!(result.snapshot.tulpa.decisions[&TulpaProposalId([6; 32])].accepted);
    assert_eq!(result.snapshot.tulpa.facts.len(), 2);
    println!("PASS self_elected_aggregate: stranger chooses electorate and self-endorses; adopted snapshot despite zero members");
}
async fn pending_quorum_observation() {
    let service = Moot::in_memory(ID, root(1), retention());
    let mut rules = ConstitutionRules::founder_only(root(1));
    rules.amendment = AmendmentRule::MemberQuorum {
        electorate: BTreeSet::from([root(2), root(3)]), threshold: 2,
    };
    let before = service.found([1; 32], None, None, rules.clone(), 1).await.unwrap();
    let mut desired = rules.clone(); desired.checkpoint_signers = BTreeSet::from([root(9)]);
    let returned = service.amend([2; 32], desired, 2).await.unwrap();
    assert_eq!(returned.governance.revision, before.governance.revision);
    assert_eq!(returned.governance.rules, rules);
    println!("PASS pending_quorum_observation: aggregate amend returns Ok with unchanged accepted law");
}
fn main() {
    ready(artifact_case(false)); ready(artifact_case(true));
    ready(self_elected_aggregate()); ready(pending_quorum_observation());
}
