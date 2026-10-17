// Audit-only. Complete unchanged production modules; no cfg(test) substitution.
#![allow(dead_code, unused_imports)]
mod constitution {
    #[path = "C:/Users/mark_/Code/repos/mere/crates/moot/gemot/src/moot/constitution/event.rs"]
    mod event;
    #[path = "C:/Users/mark_/Code/repos/mere/crates/moot/gemot/src/moot/constitution/fold.rs"]
    mod fold;
    #[path = "C:/Users/mark_/Code/repos/mere/crates/moot/gemot/src/moot/constitution/wire.rs"]
    mod wire;
    #[path = "C:/Users/mark_/Code/repos/mere/crates/moot/gemot/src/moot/constitution/store.rs"]
    mod store;
    pub use event::*;
    pub use fold::*;
    pub use wire::*;
    pub use store::*;
}
use constitution::*;
use p2panda_core::{Operation, SigningKey};
use proofs::Digest;
use std::collections::BTreeSet;
use std::future::Future;
use std::task::{Context, Poll, Waker};

fn ready<F: Future>(future: F) -> F::Output {
    let mut future = std::pin::pin!(future);
    let mut context = Context::from_waker(Waker::noop());
    // In-memory sequential storage only. A bounded executor also catches an
    // unexpected dependency on a timer, spawned task or external I/O runtime.
    for _ in 0..100_000 {
        if let Poll::Ready(value) = future.as_mut().poll(&mut context) { return value; }
        std::thread::yield_now();
    }
    panic!("in-memory diagnostic did not become ready");
}
const MOOT: [u8; 32] = [0xa6; 32];
fn id(seed: u8) -> [u8; 32] { *SigningKey::from_bytes(&[seed; 32]).verifying_key().as_bytes() }
fn founder_genesis(rules: ConstitutionRules) -> Operation<ConstitutionExt> {
    to_operation_seed([1; 32], MOOT,
        &ConstitutionEvent::genesis(MOOT, id(1), None, None, rules, 10), 0, None)
}
fn revision(operation: &Operation<ConstitutionExt>) -> Digest {
    Digest::p2panda_operation(*operation.hash.as_bytes())
}
async fn sequential_control() {
    let a = ConstitutionStore::in_memory(MOOT, id(1));
    let b = ConstitutionStore::in_memory(MOOT, id(1));
    let genesis = a.author_genesis([1; 32], None, None, ConstitutionRules::founder_only(id(1)), 10).await.unwrap();
    assert!(b.accept(&genesis).await.unwrap());
    let mut next = ConstitutionRules::founder_only(id(1));
    next.checkpoint_signers.insert(id(9));
    let amended = a.author_amendment([1; 32], next.clone(), 11).await.unwrap();
    assert!(b.accept(&amended).await.unwrap());
    assert_eq!(a.constitution().await.unwrap(), b.constitution().await.unwrap());
    assert_eq!(b.constitution().await.unwrap().unwrap().rules, next);
    println!("PASS sequential_control");
}
async fn competing_eligible_members() {
    // Independent eligible authors, each at sequence zero: no same-signer log fork.
    let mut rules = ConstitutionRules::founder_only(id(1));
    rules.amendment = AmendmentRule::MemberQuorum {
        electorate: BTreeSet::from([id(2), id(3)]), threshold: 1,
    };
    let genesis = founder_genesis(rules.clone());
    let mut left_rules = rules.clone(); left_rules.checkpoint_signers = BTreeSet::from([id(7)]);
    let mut right_rules = rules; right_rules.checkpoint_signers = BTreeSet::from([id(8)]);
    let left = to_operation_seed([2; 32], MOOT, &ConstitutionEvent::amended(revision(&genesis), left_rules.clone(), 11), 0, None);
    let right = to_operation_seed([3; 32], MOOT, &ConstitutionEvent::amended(revision(&genesis), right_rules.clone(), 11), 0, None);
    assert!(verify(&genesis) && verify(&left) && verify(&right));
    let pure_left = Constitution::fold(MOOT, id(1), [&genesis, &left, &right]).unwrap();
    let pure_right = Constitution::fold(MOOT, id(1), [&right, &left, &genesis]).unwrap();
    assert_eq!(pure_left, pure_right, "complete-corpus fold is deterministic");
    let a = ConstitutionStore::in_memory(MOOT, id(1));
    let b = ConstitutionStore::in_memory(MOOT, id(1));
    assert!(a.accept(&genesis).await.unwrap() && b.accept(&genesis).await.unwrap());
    assert!(a.accept(&left).await.unwrap() && b.accept(&right).await.unwrap());
    assert!(matches!(a.accept(&right).await, Err(ConstitutionStoreError::Constitution(ConstitutionError::StaleRevision))));
    assert!(matches!(b.accept(&left).await, Err(ConstitutionStoreError::Constitution(ConstitutionError::StaleRevision))));
    let state_a = a.constitution().await.unwrap().unwrap();
    let state_b = b.constitution().await.unwrap().unwrap();
    assert_eq!(state_a.rules, left_rules); assert_eq!(state_b.rules, right_rules);
    assert_ne!(state_a.revision, state_b.revision);
    assert!(!a.sync_store().has_operation(&right.hash).await.unwrap());
    assert!(!b.sync_store().has_operation(&left.hash).await.unwrap());
    // Repeating the rejected deliveries cannot make either complete corpus.
    assert!(a.accept(&right).await.is_err() && b.accept(&left).await.is_err());
    println!("PASS competing_eligible_members: deterministic full fold; stored laws differ; cross-deliveries refused before retention");
}
async fn missing_parent_retry_control() {
    let genesis = founder_genesis(ConstitutionRules::founder_only(id(1)));
    let amended = to_operation_seed([1; 32], MOOT,
        &ConstitutionEvent::amended(revision(&genesis), ConstitutionRules::founder_only(id(1)), 11),
        1, Some(*genesis.hash.as_bytes()));
    let store = ConstitutionStore::in_memory(MOOT, id(1));
    assert!(matches!(store.accept(&amended).await, Err(ConstitutionStoreError::NotFounded)));
    assert!(store.accept(&genesis).await.unwrap());
    assert!(store.accept(&amended).await.unwrap());
    assert_eq!(store.constitution().await.unwrap().unwrap().revision, revision(&amended));
    println!("PASS missing_parent_retry_control");
}
fn main() {
    ready(sequential_control());
    ready(competing_eligible_members());
    ready(missing_parent_retry_control());
}
