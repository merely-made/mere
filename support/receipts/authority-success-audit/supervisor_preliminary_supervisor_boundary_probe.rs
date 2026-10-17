//! Public local custody and supervisor-receipt observations.
use distillery::mesh_host::{deliver_inputs, HostConfig, ManualClock, MeshHost, NoCourier, Step, TransportBlobSpace};
use mesh::{BlobSource, DeterminismClass, JobControl, JobId, JobSpec, LeasePolicy, LeaseTerms, MeshEvent, MeshStore, ResourceId, SyncedMesh};
use personae::{IdentityProvider, InMemoryProvider};
use std::sync::Arc;
use std::time::Duration;
use transport::{BlobStore, P2pandaTransport};

fn spec(blob: mesh::BlobRef) -> JobSpec {
    JobSpec::simple(ResourceId::parse("mesh.blake3/v1").unwrap(), "payload", blob, "result", 32, DeterminismClass::Exact)
}
fn runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap()
}

async fn local_reuse(explicit_pin: bool) {
    let store = Arc::new(BlobStore::new_collecting(Duration::from_millis(10)));
    let a = TransportBlobSpace::for_mesh(store.clone(), [0xa1; 32]);
    let b = TransportBlobSpace::for_mesh(store.clone(), [0xb2; 32]);
    let blob = a.put(b"public synthetic cached job input").await.unwrap();
    assert!(b.has(&blob).await);
    let (_, control) = JobControl::new();
    deliver_inputs(&spec(blob.clone()), &b, &NoCourier, &[], &control).await;
    assert!(b.as_source_for_audit(&blob).await);
    if explicit_pin { assert_eq!(b.put(b"public synthetic cached job input").await.unwrap(), blob); }
    assert_eq!(a.release(std::slice::from_ref(&blob)).await.unwrap(), 1);
    if explicit_pin {
        for _ in 0..10 { tokio::time::sleep(Duration::from_millis(20)).await; assert!(b.has(&blob).await); }
        assert_eq!(b.release(std::slice::from_ref(&blob)).await.unwrap(), 1);
        println!("CONTROL: explicit second-mesh pin retains cached input after first mesh releases its claim");
    } else {
        let until = tokio::time::Instant::now() + Duration::from_secs(3);
        while b.has(&blob).await {
            assert!(tokio::time::Instant::now() < until, "unclaimed input was not collected");
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        assert!(b.fetch(&blob).await.unwrap().is_none());
        assert_eq!(b.release(std::slice::from_ref(&blob)).await.unwrap(), 0);
        println!("OBSERVATION: successful local-input delivery installed no second-mesh custody claim; first mesh release lets GC remove that input");
    }
}
// Keep the observation on the public BlobSource seam without reaching into tags.
trait AuditRead { async fn as_source_for_audit(&self, blob: &mesh::BlobRef) -> bool; }
impl AuditRead for TransportBlobSpace {
    async fn as_source_for_audit(&self, blob: &mesh::BlobRef) -> bool { self.fetch(blob).await.unwrap().is_some() }
}
#[test]
fn cached_input_admission_does_not_establish_this_meshs_retention() { runtime().block_on(local_reuse(false)); }
#[test]
fn explicit_mesh_retention_is_the_positive_control() { runtime().block_on(local_reuse(true)); }

async fn finished_receipt(expire: bool) {
    let identity = InMemoryProvider::from_seed([0x73; 32]);
    let key = identity.derive_keypair(mesh::MESH_AUTHOR_SALT).unwrap();
    let network = P2pandaTransport::builder_for(&key).gossip().bind().await.unwrap();
    let (endpoint, gossip) = network.sync_parts().unwrap();
    let synced = SyncedMesh::join(endpoint, gossip, MeshStore::in_memory(), [0xc3; 32]).await.unwrap();
    let space = Arc::new(mesh::MemoryBlobSpace::in_memory());
    let clock = Arc::new(ManualClock::at(1_000));
    let mut cfg = HostConfig::supervised(space.clone());
    cfg.clock = clock.clone(); cfg.lease = LeasePolicy { max_skew_ms: 0 };
    let mut host = MeshHost::new(synced, key.clone(), cfg);
    let input = space.put(b"public synthetic late completion").await.unwrap();
    let post = host.synced().author(&key, &MeshEvent::JobPostedV2 {
        spec: Box::new(spec(input).leased(LeaseTerms::new(60_000, 1_000))), nonce: 1, at_ms: 1_000,
    }).await.unwrap();
    let job = JobId(*post.hash.as_bytes());
    let mut started = false;
    for _ in 0..12 {
        let steps = host.tick().await.unwrap();
        if steps.iter().any(|s| matches!(s, Step::Started { job: j, .. } if *j == job)) { started = true; break; }
    }
    assert!(started, "supervisor never started the leased job");
    // Let the tiny in-memory job finish without another supervisor tick.
    // Step::Completed below proves the reaper actually observed its finished task.
    tokio::time::sleep(Duration::from_millis(50)).await;
    if expire { clock.set(1_000_000); }
    let steps = host.tick().await.unwrap();
    assert!(steps.iter().any(|s| matches!(s, Step::Completed { job: j, lease: Some(_), .. } if *j == job)), "{steps:?}");
    let board = host.synced().board().await.unwrap();
    let record = board.job(job).unwrap();
    if expire {
        assert!(!record.state.is_terminal(), "expired completion unexpectedly became terminal");
        println!("OBSERVATION: host emits Completed for expired lease, while its authoritative board remains nonterminal: {:?}; steps: {steps:?}", record.state);
    } else {
        assert!(record.state.is_terminal());
        println!("CONTROL: completion within the live lease is terminal in both host receipt and authoritative board");
    }
    host.shutdown().await.unwrap(); network.close().await.unwrap();
}
#[test]
fn expired_completion_receipt_disagrees_with_the_authoritative_board() { runtime().block_on(finished_receipt(true)); }
#[test]
fn live_lease_completion_is_the_positive_control() { runtime().block_on(finished_receipt(false)); }
