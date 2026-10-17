// Audit-only public consumer of the unchanged current Mien owner.
use mien_audit::*;
use identity::{IdentityProvider, InMemoryProvider};
use std::future::Future;
use std::task::{Context, Poll, Waker};

const A: [u8; 32] = [0x61; 32];
const B: [u8; 32] = [0x62; 32];
fn person(seed: u8) -> InMemoryProvider { InMemoryProvider::from_seed([seed; 32]) }
fn root(seed: u8) -> ChainRoot { ChainRoot(person(seed).master_public_key().to_bytes()) }
fn ready<F: Future>(future: F) -> F::Output {
    let mut future = std::pin::pin!(future);
    let mut context = Context::from_waker(Waker::noop());
    for _ in 0..100_000 {
        if let Poll::Ready(value) = future.as_mut().poll(&mut context) { return value; }
        std::thread::yield_now();
    }
    panic!("sequential in-memory diagnostic needs unexpected external progress");
}
async fn score(store: &StandingStore<muniment::MemoryBackend>, moot: [u8;32], who: ChainRoot) -> i64 {
    store.fold_moot(moot, StandingConfig::default()).await.unwrap().score(&who, 1_000)
}
async fn judgement_without_role() {
    let store = StandingStore::in_memory();
    let outsider = person(2);
    let op = store.author_for_identity(&outsider, A, &StandingEvent::Censure {
        by: root(2), target: root(1), weight: 30, at_ms: 1,
    }).await.unwrap();
    assert!(verify(&op)); assert_eq!(stable_author(&op).unwrap(), root(2));
    assert_eq!(score(&store,A,root(1)).await,-30);
    store.author_for_identity(&outsider,A,&StandingEvent::Pardon {
        by:root(2),target:root(1),weight:30,at_ms:2,
    }).await.unwrap();
    assert_eq!(score(&store,A,root(1)).await,0);
    println!("PASS judgement_without_role: authenticated unrelated signer censures and pardons; no governance provider exists on this path");
}
async fn unsigned_weight_sign() {
    for (pardon,weight,wanted) in [(true,30,30),(false,30,-30),(true,u64::MAX,-1),(false,u64::MAX,1)] {
        let store = StandingStore::in_memory();
        let event = if pardon { StandingEvent::Pardon{by:root(2),target:root(1),weight,at_ms:1} }
            else { StandingEvent::Censure{by:root(2),target:root(1),weight,at_ms:1} };
        let op=store.author_for_identity(&person(2),A,&event).await.unwrap();
        assert!(verify(&op)); assert_eq!(score(&store,A,root(1)).await,wanted);
    }
    println!("PASS unsigned_weight_sign: ordinary +/-30 control; max unsigned pardon lowers by1 and censure raises by1");
}
async fn asserted_time_vs_log_order() {
    for (completion_time,wanted) in [(101,10),(99,-20)] {
        let store=StandingStore::in_memory();
        let by=root(1);let identity=person(1);let commitment=CommitmentId([7;32]);
        let made=StandingEvent::CommitmentMade{by,commitment,scope:Scope("notes".into()),cadence_ms:10,duration_ms:None,at_ms:100};
        let fulfilled=StandingEvent::CommitmentFulfilled{by,commitment,at_ms:completion_time};
        let first=store.author_for_identity(&identity,A,&made).await.unwrap();
        let second=store.author_for_identity(&identity,A,&fulfilled).await.unwrap();
        assert_eq!(second.header.seq_num,1);
        assert!(verify(&first)&&verify(&second));
        let causal=Ledger::from_events(StandingConfig::default(),&[made,fulfilled]);
        assert_eq!(causal.score(&by,1000),10);
        assert_eq!(score(&store,A,by).await,wanted);
    }
    println!("PASS asserted_time_vs_log_order: forward-time control +10; backwards authored time sorts valid completion before creation and yields -20");
}
async fn repeat_and_self_vouch() {
    let store=StandingStore::in_memory();let identity=person(1);
    store.author_for_identity(&identity,A,&StandingEvent::GovernanceParticipation{by:root(1),at_ms:0}).await.unwrap();
    for at_ms in [1,2] {
        store.author_for_identity(&identity,A,&StandingEvent::Vouch {
            voucher:root(1),newcomer:root(2),fraction_bp:10_000,at_ms,
        }).await.unwrap();
    }
    assert_eq!(score(&store,A,root(1)).await,1);
    assert_eq!(score(&store,A,root(2)).await,2);
    store.author_for_identity(&identity,A,&StandingEvent::Vouch {
        voucher:root(1),newcomer:root(1),fraction_bp:10_000,at_ms:3,
    }).await.unwrap();
    assert_eq!(score(&store,A,root(1)).await,2);
    println!("PASS repeat_and_self_vouch: two fresh same-edge vouches double newcomer boost; self-vouch then doubles voucher accrual");
}
async fn out_of_range_basis_points() {
    for (fraction_bp,wanted) in [(10_000,1),(u16::MAX,6)] {
        let store=StandingStore::in_memory();let identity=person(1);
        store.author_for_identity(&identity,A,&StandingEvent::GovernanceParticipation{by:root(1),at_ms:0}).await.unwrap();
        let op=store.author_for_identity(&identity,A,&StandingEvent::Vouch {
            voucher:root(1),newcomer:root(2),fraction_bp,at_ms:1,
        }).await.unwrap();
        assert!(verify(&op));assert_eq!(score(&store,A,root(2)).await,wanted);
    }
    println!("PASS out_of_range_basis_points: documented10000 limit control yields1; signed65535 accepted and yields6");
}
async fn shared_store_moot_isolation() {
    let store=StandingStore::in_memory();let by=root(1);
    let first=store.author_seed([1;32],A,&StandingEvent::GovernanceParticipation{by,at_ms:1}).await.unwrap();
    let second=store.author_seed([1;32],B,&StandingEvent::GovernanceParticipation{by,at_ms:2}).await.unwrap();
    assert_eq!(first.header.seq_num,0);assert_eq!(second.header.seq_num,1);
    assert!(verify(&first)&&verify(&second));
    let a=score(&store,A,by).await;let b=score(&store,B,by).await;
    assert_eq!((a,b),(2,2));
    let separate=StandingStore::in_memory();
    separate.author_seed([1;32],A,&StandingEvent::GovernanceParticipation{by,at_ms:1}).await.unwrap();
    assert_eq!(score(&separate,A,by).await,1);assert_eq!(score(&separate,B,by).await,0);
    println!("PASS shared_store_moot_isolation: A and B share raw-author/log0; both score2 from one own event; separate-store control is1/0");
}
fn main() {
    ready(judgement_without_role());ready(unsigned_weight_sign());ready(asserted_time_vs_log_order());
    ready(repeat_and_self_vouch());ready(out_of_range_basis_points());ready(shared_store_moot_isolation());
}
