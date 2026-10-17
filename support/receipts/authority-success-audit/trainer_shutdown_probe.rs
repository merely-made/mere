//! Real CPU TrainerResource held at its shared library mutex; no model training.
use distillery::{LoraTrainerSettings, TrainRequest, TrainerResource, TrainerSettings, TrainerDevice, TRAINER_REQUEST_INPUT, TRAINER_RESOURCE};
use distillery::mesh_host::{HostConfig, ManualClock, MeshHost, Step};
use eidetic::{ManifestId, MemoryBackend};
use eidetic::models::EvalMetric;
use mesh::{DeterminismClass, JobSpec, MeshEvent, MeshStore, ResourceId, ResourceRegistry, SyncedMesh};
use personae::{IdentityProvider, InMemoryProvider};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use transport::P2pandaTransport;

fn request() -> TrainRequest {
    TrainRequest {
        base_model_ref: ManifestId::of_blob(b"synthetic absent model"),
        tokenizer_ref: ManifestId::of_blob(b"synthetic tokenizer"),
        corpus_ref: ManifestId::of_blob(b"synthetic corpus"),
        adapter_name: "audit-no-training".into(), prompt_template: "{{ prompt }}".into(),
        metric: EvalMetric::RankingAt { limit: 1 },
        settings: TrainerSettings::FiniteDifference(LoraTrainerSettings {
            rank: 1, alpha: 1.0, target_module: "v_proj".into(), steps: 1,
            initial_step_length: 1.0, minimum_step_length: 0.001, epsilon: 0.02,
        }), created_at: 1_000,
    }
}

async fn exercise(release_before_shutdown: bool) {
    let store = Arc::new(Mutex::new(MemoryBackend::default()));
    #[allow(deprecated)]
    let trainer = Arc::new(TrainerResource::new(store.clone(), TrainerDevice::ndarray()));
    assert_eq!(Arc::strong_count(&store), 2, "probe and real trainer own the store");
    let guard = store.lock().await;
    let identity = InMemoryProvider::from_seed([0x74; 32]);
    let key = identity.derive_keypair(mesh::MESH_AUTHOR_SALT).unwrap();
    let carrier_key = identity.derived_keypair(mesh::MESH_AUTHOR_SALT).unwrap();
    let network = P2pandaTransport::builder_for(&carrier_key).gossip().bind().await.unwrap();
    let (endpoint, gossip) = network.sync_parts().unwrap();
    let synced = SyncedMesh::join(endpoint, gossip, MeshStore::in_memory(), [0xd4; 32]).await.unwrap();
    let space = Arc::new(mesh::MemoryBlobSpace::in_memory());
    let mut registry = ResourceRegistry::new(); registry.register(trainer.clone()).unwrap();
    let mut cfg = HostConfig::supervised(space.clone()); cfg.registry = registry;
    cfg.clock = Arc::new(ManualClock::at(1_000));
    let mut host = MeshHost::new(synced, key.clone(), cfg);
    let bytes = serde_json::to_vec(&request()).unwrap();
    let input = space.put(&bytes).await.unwrap();
    host.synced().author(&key, &MeshEvent::JobPostedV2 {
        spec: Box::new(JobSpec::simple(ResourceId::parse(TRAINER_RESOURCE).unwrap(),
            TRAINER_REQUEST_INPUT, input, "receipt", 65536, DeterminismClass::Observed)),
        nonce: 1, at_ms: 1_000,
    }).await.unwrap();
    let mut started = false;
    for _ in 0..12 {
        if host.tick().await.unwrap().iter().any(|s| matches!(s, Step::Started { .. })) {
            started = true; break;
        }
    }
    assert!(started);
    let until = tokio::time::Instant::now() + Duration::from_secs(5);
    while Arc::strong_count(&store) == 2 {
        assert!(tokio::time::Instant::now() < until, "trainer blocking closure never took its store owner");
        tokio::task::yield_now().await;
    }
    assert_eq!(Arc::strong_count(&store), 3, "real blocking trainer owns the third reference");
    if release_before_shutdown {
        drop(guard);
        let until = tokio::time::Instant::now() + Duration::from_secs(5);
        while Arc::strong_count(&store) != 2 {
            assert!(tokio::time::Instant::now() < until);
            tokio::time::sleep(Duration::from_millis(2)).await;
        }
        host.shutdown().await.unwrap();
        assert_eq!(Arc::strong_count(&store), 2);
        println!("CONTROL: letting the real CPU worker drain first leaves only the probe/trainer store owners after host shutdown");
    } else {
        tokio::time::timeout(Duration::from_secs(5), host.shutdown()).await.unwrap().unwrap();
        assert_eq!(Arc::strong_count(&store), 3, "successful host shutdown left the CPU trainer's blocking closure alive");
        assert!(store.try_lock().is_err(), "probe still gates the worker before any model load");
        println!("OBSERVATION: host shutdown returned success while a real CPU trainer blocking closure retained the shared model-store owner");
        // Release only the audit-owned gate. The absent synthetic model then
        // makes the actual worker exit without reaching tensor training.
        drop(guard);
        let until = tokio::time::Instant::now() + Duration::from_secs(5);
        while Arc::strong_count(&store) != 2 {
            assert!(tokio::time::Instant::now() < until, "audit worker failed to drain after gate release");
            tokio::time::sleep(Duration::from_millis(2)).await;
        }
    }
    network.close().await.unwrap();
    drop(trainer); assert_eq!(Arc::strong_count(&store), 1);
}

#[test]
fn successful_host_shutdown_does_not_wait_for_a_cpu_trainer_store_owner() {
    tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(exercise(false));
}
#[test]
fn an_actually_drained_cpu_worker_is_the_positive_control() {
    tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(exercise(true));
}
