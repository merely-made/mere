// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

#![cfg(feature = "inference-stages")]
#[path = "../../../crates/intel/esp/tests/support/decoder.rs"]
mod fixture;

use distillery::inference_stage::*;
use distillery::mesh_host::{HostConfig, ManualClock, MeshHost, ObservedConditions, Step};
use eidetic::{ManifestId, PrivacyClass, ProvenanceRecord, Timestamp, TrustEnvelope};
use esp::infer::decoder::{DecoderDevice, WeightFormat};
use esp::infer::library::{InferenceModelRow, resolve_library_session};
use esp::infer::{GenerationRequest, InferenceProvider};
use mesh::{
    DevicePolicy, JobId, LeaseId, LeasePolicy, LeaseTerms, MESH_AUTHOR_SALT, MeshEvent, MeshStore,
    RemoteSessionClaim, ResourceRegistry, SyncedMesh,
};
use personae::{IdentityProvider, InMemoryProvider};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use transport::{P2pandaTransport, Transport};

const MESH: [u8; 32] = [0x18; 32];
const NOW: u64 = 10_000;
type Library = Arc<tokio::sync::Mutex<muniment::RedbBackend>>;

#[test]
fn actor_lane_records_offline_streaming_placement_in_its_sealed_journal() {
    let temp = tempfile::tempdir().unwrap();
    let (library, row) = tiny_library(temp.path());
    let journal = Arc::new(distillery::inference::durable::PersonaRunJournal::new(
        muniment::MemoryBackend::new(),
        [33; 32],
        &personae::Ed25519Keypair::from_seed([34; 32]),
        "coordinator".into(),
        [35; 32],
    ));
    let lane = open_staged_library_lane_with_journal(
        library,
        row,
        6,
        StagedLaneDevice::Cpu,
        Arc::new(|_| {
            Err(esp::infer::InferError::StageUnavailable(
                "workers offline".into(),
            ))
        }),
        "coordinator".into(),
        Some(journal.clone()),
    )
    .unwrap();
    lane.ask(distillery::inference::AskRequestV1 {
        run: "offline".into(),
        question: "t3 t7".into(),
    })
    .unwrap();
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    loop {
        let runs = lane.runs();
        if let Some(run) = runs.runs.first().filter(|r| r.state.is_terminal()) {
            assert_eq!(run.state, distillery::inference::RunStateV1::Finished);
            assert_eq!(run.tokens, 6);
            break;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "offline stage lane did not finish: {runs:?}"
        );
        std::thread::sleep(Duration::from_millis(5));
    }
    lane.close().unwrap();
    let evidence = journal.retained_record("offline").unwrap();
    assert_eq!(evidence.weight_format.as_deref(), Some("f32"));
    assert!(
        evidence.continuations.iter().any(|c| c.reason
            == "stage plan unavailable; local layer streaming"
            && c.at_token == 0)
    );
    let bytes = journal
        .resolve_evidence_blob(evidence.checkpoint_blob.unwrap())
        .unwrap()
        .unwrap();
    let checkpoint: esp::infer::provider::GenerationCheckpoint =
        serde_json::from_slice(&bytes).unwrap();
    assert_eq!(checkpoint.placements.len(), 1);
    assert!(checkpoint.complete);
}

fn tiny_library(directory: &std::path::Path) -> (Library, ManifestId) {
    std::fs::create_dir_all(directory).unwrap();
    let mut config: serde_json::Value = serde_json::from_str(fixture::CONFIG_JSON).unwrap();
    config["max_position_embeddings"] = serde_json::json!(256);
    std::fs::write(
        directory.join("config.json"),
        serde_json::to_vec(&config).unwrap(),
    )
    .unwrap();
    std::fs::write(directory.join("tokenizer.json"), fixture::tokenizer_json()).unwrap();
    std::fs::write(
        directory.join("model.safetensors"),
        fixture::weights_bytes(),
    )
    .unwrap();
    let mut store = muniment::RedbBackend::open(directory.join("model.redb")).unwrap();
    let row = pollster::block_on(async {
        let model = esp::infer::layer_library::import_layered_model(
            &mut store,
            directory,
            "fixture/stages",
            "MPL-2.0",
            Default::default(),
        )
        .await
        .unwrap();
        let mut template = esp::infer::prompt::PromptTemplate::from_bytes(include_bytes!(
            "../../../crates/intel/esp/templates/smollm2-instruct-v1.json"
        ))
        .unwrap();
        template.bos.clear();
        template.eos.clear();
        template.system_prompt.clear();
        template.user.prefix.clear();
        template.user.suffix = " ".into();
        template.assistant.prefix.clear();
        template.assistant.suffix.clear();
        template.stop.clear();
        let template = eidetic::typed::save_typed(
            &mut store,
            &template,
            vec![],
            PrivacyClass::LocalOnly,
            ProvenanceRecord::self_imported("local"),
            TrustEnvelope::self_asserted(),
            Timestamp::ZERO,
        )
        .await
        .unwrap();
        eidetic::typed::save_typed(
            &mut store,
            &InferenceModelRow { model, template },
            vec![],
            PrivacyClass::LocalOnly,
            ProvenanceRecord::self_imported("local"),
            TrustEnvelope::self_asserted(),
            Timestamp::ZERO,
        )
        .await
        .unwrap()
    });
    (Arc::new(tokio::sync::Mutex::new(store)), row)
}

#[cfg(feature = "inference-gpu")]
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "pinned SmolLM2 source and I7 qualified f16 row; native GPU"]
async fn real_smollm2_f16_stages_match_and_recover_under_supervised_leases() {
    let work = std::path::PathBuf::from(std::env::var_os("ESP_STAGE_WORK_DIR").unwrap());
    let source = std::path::PathBuf::from(std::env::var_os("ESP_SMOLLM2_DIR").unwrap());
    let formats = std::path::PathBuf::from(std::env::var_os("ESP_FORMAT_WORK_DIR").unwrap());
    std::fs::create_dir_all(&work).unwrap();
    let import_work = work.clone();
    let (libraries, row) = tokio::task::spawn_blocking(move || {
        let receipt: serde_json::Value =
            serde_json::from_slice(&std::fs::read(formats.join("f16-burn-wgpu.json")).unwrap())
                .unwrap();
        let mut prior = muniment::RedbBackend::open(formats.join("model-formats.redb")).unwrap();
        let row =
            ManifestId(eidetic::Hash::parse(receipt["qualified_row"].as_str().unwrap()).unwrap());
        let (row_record, model, index) = pollster::block_on(async {
            let r = eidetic::typed::load_typed::<InferenceModelRow>(
                &mut prior,
                &mut eidetic::NoFetcher,
                row,
            )
            .await
            .unwrap()
            .unwrap();
            let m = eidetic::ModelLibrary::load_model(&mut prior, r.model)
                .await
                .unwrap()
                .unwrap();
            let i = eidetic::typed::load_typed::<
                esp::infer::layer_library::FormattedLayerWeightIndex,
            >(&mut prior, &mut eidetic::NoFetcher, m.weight_blob)
            .await
            .unwrap()
            .unwrap();
            (r, m, i)
        });
        drop(prior);
        let save = |store: &mut dyn eidetic::Store, value: &InferenceModelRow| {
            pollster::block_on(eidetic::typed::save_typed(
                store,
                value,
                vec![],
                PrivacyClass::LocalOnly,
                ProvenanceRecord::self_imported("local"),
                TrustEnvelope::self_asserted(),
                Timestamp::ZERO,
            ))
            .unwrap()
        };
        let mut libraries = Vec::new();
        for name in ["coordinator", "worker-a", "worker-b"] {
            let directory = import_work.join(name);
            std::fs::create_dir_all(&directory).unwrap();
            let mut store = muniment::RedbBackend::open(directory.join("model.redb")).unwrap();
            let candidate = pollster::block_on(esp::infer::layer_library::import_formatted_model(
                &mut store,
                &source,
                "HuggingFaceTB/SmolLM2-360M-Instruct",
                "Apache-2.0",
                Default::default(),
                WeightFormat::F16,
            ))
            .unwrap();
            assert_eq!(
                candidate.to_string(),
                receipt["candidate_model"].as_str().unwrap()
            );
            // Replicate the previously qualified immutable index and model,
            // rather than creating another measurement or qualification.
            let copied = pollster::block_on(eidetic::typed::save_typed(
                &mut store,
                &index,
                vec![],
                PrivacyClass::LocalOnly,
                ProvenanceRecord::self_imported("local"),
                TrustEnvelope::self_asserted(),
                Timestamp::ZERO,
            ))
            .unwrap();
            assert_eq!(copied, model.weight_blob);
            assert_eq!(
                pollster::block_on(eidetic::ModelLibrary::save_model(&mut store, model.clone()))
                    .unwrap(),
                row_record.model
            );
            let template = esp::infer::prompt::PromptTemplate::from_bytes(include_bytes!(
                "../../../crates/intel/esp/templates/smollm2-instruct-v1.json"
            ))
            .unwrap();
            let template = pollster::block_on(eidetic::typed::save_typed(
                &mut store,
                &template,
                vec![],
                PrivacyClass::LocalOnly,
                ProvenanceRecord::self_imported("local"),
                TrustEnvelope::self_asserted(),
                Timestamp::ZERO,
            ))
            .unwrap();
            assert_eq!(template, row_record.template);
            assert_eq!(save(&mut store, &row_record), row);
            libraries.push(Arc::new(tokio::sync::Mutex::new(store)));
        }
        (libraries.try_into().ok().unwrap(), row)
    })
    .await
    .unwrap();
    exercise(
        libraries,
        row,
        work.join("checkpoint.json"),
        Profile {
            departure: Departure::Owner,
            loader: "burn-wgpu",
            format: WeightFormat::F16,
            width: 960,
            vocab: 49152,
            context: 8192,
            layers: 32,
            prompt: "What is the capital of France?".into(),
            max_tokens: 16,
            temperature: 0.0,
        },
    )
    .await;
}

fn device(loader: &str) -> DecoderDevice {
    match loader {
        #[cfg(feature = "inference-gpu")]
        "burn-wgpu" => DecoderDevice::wgpu(esp::infer::decoder::DecoderGpuKind::DiscreteGpu(0)),
        "burn-flex" => DecoderDevice::flex(),
        _ => DecoderDevice::ndarray(),
    }
}

#[derive(Clone, Copy, Debug)]
enum Departure {
    Owner,
    Lock,
    Reader,
    Idle,
}

#[derive(Clone)]
struct Profile {
    departure: Departure,
    loader: &'static str,
    format: WeightFormat,
    width: usize,
    vocab: usize,
    context: usize,
    layers: usize,
    prompt: String,
    max_tokens: usize,
    temperature: f32,
}

fn streaming(
    library: Library,
    row: ManifestId,
    loader: &str,
    max_tokens: usize,
) -> Box<dyn InferenceProvider> {
    let resolved = {
        let mut store = library.blocking_lock();
        pollster::block_on(resolve_library_session(&mut *store, row)).unwrap()
    };
    let artifacts = resolved.layer_artifacts().unwrap();
    let source = move |i: usize, limit: usize| {
        let mut store = library.blocking_lock();
        pollster::block_on(artifacts[i].read(&mut *store, limit))
    };
    Box::new(
        resolved
            .load_with_layer_source(&device(loader), loader, max_tokens, Some(Box::new(source)))
            .unwrap(),
    )
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn leased_disjoint_iroh_stages_match_and_owner_reclaim_streams_from_the_committed_boundary() {
    tiny_exercise(Departure::Owner).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn worker_vault_lock_drains_private_state_and_the_ask_streams_elsewhere() {
    tiny_exercise(Departure::Lock).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn retiring_a_reader_drains_its_stage_and_the_ask_streams_elsewhere() {
    tiny_exercise(Departure::Reader).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn idle_stages_close_their_leases_and_the_ask_streams_elsewhere() {
    tiny_exercise(Departure::Idle).await;
}

async fn tiny_exercise(departure: Departure) {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().to_path_buf();
    let (libraries, row) = tokio::task::spawn_blocking(move || {
        let (a, row) = tiny_library(&root.join("coordinator"));
        let (b, row_b) = tiny_library(&root.join("worker-a"));
        let (c, row_c) = tiny_library(&root.join("worker-b"));
        assert_eq!(row, row_b);
        assert_eq!(row, row_c);
        ([a, b, c], row)
    })
    .await
    .unwrap();
    exercise(
        libraries,
        row,
        temp.path().join("checkpoint.json"),
        Profile {
            departure,
            loader: "burn-ndarray",
            format: WeightFormat::F32,
            width: 8,
            vocab: 32,
            context: 256,
            layers: 2,
            prompt: "t3 t7".into(),
            max_tokens: 6,
            temperature: 0.7,
        },
    )
    .await;
}

async fn exercise(
    libraries: [Library; 3],
    row: ManifestId,
    checkpoint_path: std::path::PathBuf,
    profile: Profile,
) {
    let seeds = if profile.width > 8 {
        [131, 132, 133]
    } else {
        match profile.departure {
            Departure::Owner => [91, 92, 93],
            Departure::Lock => [101, 102, 103],
            Departure::Reader => [111, 112, 113],
            Departure::Idle => [141, 142, 143],
        }
    };
    let identities: Vec<_> = seeds
        .into_iter()
        .map(|s| InMemoryProvider::from_seed([s; 32]))
        .collect();
    let keys: Vec<_> = identities
        .iter()
        .map(|p| p.derive_keypair(MESH_AUTHOR_SALT).unwrap())
        .collect();
    let mut transports = Vec::new();
    for identity in &identities {
        transports.push(Arc::new(
            P2pandaTransport::builder_for(&identity.derived_keypair(MESH_AUTHOR_SALT).unwrap())
                .gossip()
                .bind()
                .await
                .unwrap(),
        ));
    }
    for (i, a) in transports.iter().enumerate() {
        for (j, b) in transports.iter().enumerate() {
            if i != j {
                a.add_peer(b.endpoint_addr().await.unwrap()).await.unwrap();
                a.set_topics(b.local_peer_id(), &[transport::sync_overlay_topic(MESH)])
                    .await
                    .unwrap();
            }
        }
    }
    let mut synced = Vec::new();
    for transport in &transports {
        let (endpoint, gossip) = transport.sync_parts().unwrap();
        synced.push(
            SyncedMesh::join(endpoint, gossip, MeshStore::in_memory(), MESH)
                .await
                .unwrap(),
        );
    }
    for i in 0..3 {
        synced[i]
            .author(
                &keys[i],
                &MeshEvent::DeviceAttested {
                    attestation: Box::new(
                        identities[i].attest_derived_key(MESH_AUTHOR_SALT).unwrap(),
                    ),
                },
            )
            .await
            .unwrap();
    }
    let resolved = {
        let library = libraries[0].clone();
        tokio::task::spawn_blocking(move || {
            pollster::block_on(resolve_library_session(&mut *library.blocking_lock(), row)).unwrap()
        })
        .await
        .unwrap()
    };
    let session = resolved.session(profile.loader).id().unwrap().to_string();
    let settings = StageSettings {
        max_hidden_bytes: 16 << 20,
        call_timeout: if matches!(profile.departure, Departure::Idle) {
            Duration::from_secs(1)
        } else {
            Duration::from_secs(60)
        },
        poll_every: Duration::from_millis(5),
        requires: mesh::ResourceRequirements {
            memory_mib: 1024,
            compute: if profile.loader == "burn-wgpu" {
                mesh::ComputeClass::Gpu
            } else {
                mesh::ComputeClass::Cpu
            },
        },
    };
    let costs = resolved.stage_layer_costs().unwrap();
    let capacity = costs[..profile.layers / 2].iter().sum();
    let assignments = esp::infer::decoder::stage::assign_ranges(
        &costs,
        &[
            esp::infer::decoder::stage::StageCapacity {
                device: transports[1].local_peer_id().to_bytes(),
                layer_bytes: capacity,
            },
            esp::infer::decoder::stage::StageCapacity {
                device: transports[2].local_peer_id().to_bytes(),
                layer_bytes: capacity,
            },
        ],
    )
    .unwrap();
    assert_eq!(assignments.len(), 2);
    let worker_access: Vec<_> = (0..2)
        .map(|_| distillery::gpu::GpuAccess::default())
        .collect();
    let clock = Arc::new(ManualClock::at(NOW));
    let keyed = Arc::new(std::sync::atomic::AtomicBool::new(true));
    let roots: std::collections::BTreeSet<_> = identities
        .iter()
        .map(|id| id.master_keypair().public_key().to_bytes())
        .collect();
    let roots = Arc::new(std::sync::RwLock::new(roots));
    // Root checks below use the exact verified directory roots; the live bit
    // models the graph key owner retiring a seat, not transport reachability.
    let mut services = Vec::new();
    for i in 1..3 {
        let keyed = keyed.clone();
        let roots = roots.clone();
        let readers: Arc<StageReaders> = Arc::new(move |root| {
            keyed.load(std::sync::atomic::Ordering::SeqCst) && roots.read().unwrap().contains(&root)
        });
        services.push(
            StageService::mount(
                transports[i].clone(),
                MESH,
                keys[i].public_key().to_bytes(),
                clock.clone(),
                LeasePolicy { max_skew_ms: 0 },
                settings.clone(),
                library_stage_factory(
                    libraries[i].clone(),
                    device(profile.loader),
                    profile.loader.into(),
                    if profile.loader == "burn-wgpu" {
                        Some(worker_access[i - 1].clone())
                    } else {
                        None
                    },
                    capacity,
                ),
                readers,
            )
            .await
            .unwrap(),
        );
    }
    let mut hosts = Vec::new();
    let mut conditions = Vec::new();
    let mut jobs = Vec::new();
    let mut requests = Vec::new();
    let poster = synced.remove(0);
    for (i, sync) in synced.into_iter().enumerate() {
        let request = StageRequest {
            row,
            session: session.clone(),
            loader: profile.loader.into(),
            range: assignments[i].range,
        };
        let space = Arc::new(mesh::MemoryBlobSpace::in_memory());
        let input = space
            .put(&serde_json::to_vec(&request).unwrap())
            .await
            .unwrap();
        let spec = mesh::JobSpec::simple(
            stage_resource(services[i].server_peer()),
            "stage",
            input,
            "receipt",
            256,
            mesh::DeterminismClass::Observed,
        )
        .leased(LeaseTerms::new(600_000, 60_000));
        let post = poster
            .author(
                &keys[0],
                &MeshEvent::JobPostedV2 {
                    spec: Box::new(spec),
                    nonce: i as u64,
                    at_ms: NOW,
                },
            )
            .await
            .unwrap();
        jobs.push(JobId(*post.hash.as_bytes()));
        let mut registry = ResourceRegistry::new();
        registry.register(services[i].resource()).unwrap();
        let condition = Arc::new(ObservedConditions::spare());
        let mut config = HostConfig::supervised(space);
        config.registry = registry;
        config.clock = clock.clone();
        config.conditions = condition.clone();
        config.policy = DevicePolicy::conservative();
        config.lease = LeasePolicy { max_skew_ms: 0 };
        config.facts = mesh::HostFacts {
            memory_mib: 2048,
            gpu: profile.loader == "burn-wgpu",
        };
        let mut host = MeshHost::new(sync, keys[i + 1].clone(), config);
        host.set_gpu_access(worker_access[i].clone());
        hosts.push(host);
        conditions.push(condition);
        requests.push(request);
    }
    let steps = Arc::new(Mutex::new(Vec::new()));
    let (stop, mut stopped) = tokio::sync::oneshot::channel();
    let pump_services = services.clone();
    let pump_steps = steps.clone();
    let pump = tokio::spawn(async move {
        loop {
            for (host, service) in hosts.iter_mut().zip(&pump_services) {
                service.refresh(host.synced().board().await.unwrap());
                let tick = host.tick().await.unwrap();
                pump_steps.lock().unwrap().extend(tick);
                service.refresh(host.synced().board().await.unwrap());
            }
            tokio::select! { _=&mut stopped=>break, _=tokio::time::sleep(Duration::from_millis(10))=>{} }
        }
        for host in hosts {
            host.shutdown().await.unwrap();
        }
    });
    let leases: Vec<LeaseId> = tokio::time::timeout(Duration::from_secs(60), async {
        loop {
            let current = steps.lock().unwrap().clone();
            let leases: Vec<_> = jobs
                .iter()
                .filter_map(|job| {
                    current.iter().find_map(|step| match step {
                        Step::Granted { job: j, lease, .. } if j == job => Some(*lease),
                        _ => None,
                    })
                })
                .collect();
            if leases.len() == 2
                && services
                    .iter()
                    .enumerate()
                    .all(|(i, s)| s.is_active(jobs[i], leases[i]))
            {
                break leases;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("supervisors claim, grant and load both ranges");
    let peers: Vec<_> = (0..2)
        .map(|i| StagePeer {
            peer: transports[i + 1].local_peer_id(),
            credential: RemoteSessionClaim::signed(
                &keys[0],
                MESH,
                jobs[i],
                leases[i],
                0,
                services[i].server_peer(),
                0,
            )
            .encode()
            .unwrap(),
            request: requests[i].clone(),
        })
        .collect();
    let network = transports[0].clone();
    let handle = tokio::runtime::Handle::current();
    let key_gate = keyed.clone();
    let peer_roots = Arc::new(
        transports
            .iter()
            .zip(&identities)
            .map(|(transport, identity)| {
                (
                    transport.local_peer_id().to_bytes(),
                    identity.master_keypair().public_key().to_bytes(),
                )
            })
            .collect::<std::collections::BTreeMap<_, _>>(),
    );
    let coordinator_library = libraries[0].clone();
    let retiring_root = identities[2].master_keypair().public_key().to_bytes();
    let retiring_roots = roots.clone();
    let departure = profile.departure;
    let leaving = conditions[1].clone();
    let departed = services[1].clone();
    let departed_job = jobs[1];
    let work = tokio::task::spawn_blocking(move || {
        let make_chain = || StageChain::new(network.clone(),handle.clone(),peers.clone(),settings.clone(),profile.format,StageShape{hidden:profile.width,vocab:profile.vocab,context:profile.context,layers:profile.layers},{let gate=key_gate.clone();let roots=retiring_roots.clone();let peers=peer_roots.clone();Arc::new(move |peer:transport::PeerID|gate.load(std::sync::atomic::Ordering::SeqCst) && peers.get(&peer.to_bytes()).is_some_and(|root|roots.read().unwrap().contains(root)))}).unwrap();
        let request = GenerationRequest{prompt:profile.prompt.clone(),max_tokens:profile.max_tokens,temperature:profile.temperature,top_p:Some(0.9),seed:Some(19),..Default::default()};
        let reference_start = std::time::Instant::now();
        let reference = streaming(coordinator_library.clone(),row,profile.loader,profile.max_tokens).generate_checkpointed(&request,None,&mut |_|Ok(()),&mut ||false).unwrap();
        let reference_seconds = reference_start.elapsed().as_secs_f64();
        let chain = make_chain(); chain.wait_ready().unwrap(); let traffic = chain.traffic();
        let provider = resolved.load_with_stages(&device(profile.loader),profile.loader,profile.max_tokens,Box::new(chain)).unwrap();
        let staged_start = std::time::Instant::now();
        let staged = provider.generate_checkpointed(&request,None,&mut |_|Ok(()),&mut ||false).unwrap();
        let staged_seconds = staged_start.elapsed().as_secs_f64();
        let normal_traffic = traffic.lock().unwrap().clone();
        assert_eq!(staged.token_ids,reference.token_ids); assert_eq!(staged.draws,reference.draws);
        assert!(traffic.lock().unwrap().payload_bytes_sent > 0);
        let library = coordinator_library.clone();
        let fallback_profile = profile.clone();
        let fallback = StageFallbackProvider::new(Box::new(provider),Arc::new(move ||Ok(streaming(library.clone(),row,fallback_profile.loader,fallback_profile.max_tokens))),"coordinator".into()).unwrap();
        let mut reclaimed = false;
        let recovery_start = std::time::Instant::now();
        let continued = fallback.generate_checkpointed(&request,None,&mut |cp| {
            use std::io::Write;
            let mut file = std::fs::File::create(&checkpoint_path).unwrap();
            file.write_all(&serde_json::to_vec(cp).unwrap()).unwrap(); file.sync_all().unwrap();
            if cp.token_ids.len()==3 && !reclaimed {
                reclaimed = true; match profile.departure {Departure::Owner=>leaving.in_use(),Departure::Lock=>departed.set_private_locked(true),Departure::Reader=>{retiring_roots.write().unwrap().remove(&retiring_root);},Departure::Idle=>{}}
                let deadline = std::time::Instant::now()+Duration::from_secs(5);
                while departed.is_active(jobs[1],leases[1]) && std::time::Instant::now()<deadline {std::thread::sleep(Duration::from_millis(5));}
                assert!(!departed.is_active(jobs[1],leases[1]));
            }
            Ok(())
        },&mut ||false).unwrap();
        let recovery_seconds = recovery_start.elapsed().as_secs_f64();
        assert!(reclaimed); assert!(continued.complete);
        assert_eq!(continued.token_ids,reference.token_ids); assert_eq!(continued.draws,reference.draws);
        assert_eq!(continued.placements.len(),1); assert_eq!(continued.placements[0].at_token,3);
        let reopened:esp::infer::provider::GenerationCheckpoint = serde_json::from_slice(&std::fs::read(&checkpoint_path).unwrap()).unwrap();
        assert_eq!(reopened,continued);
        make_chain().finish();
        key_gate.store(false,std::sync::atomic::Ordering::SeqCst);
        let chain = make_chain();
        let refused = resolved.load_with_stages(&device(profile.loader),profile.loader,profile.max_tokens,Box::new(chain)).unwrap().generate_checkpointed(&request,None,&mut |_|Ok(()),&mut ||false);
        assert!(matches!(refused,Err(esp::infer::InferError::StageUnavailable(_))));
        let server_chain=StageChain::new(network.clone(),handle.clone(),peers.clone(),settings.clone(),profile.format,StageShape{hidden:profile.width,vocab:profile.vocab,context:profile.context,layers:profile.layers},Arc::new(|_|true)).unwrap();
        let server_refused=resolved.load_with_stages(&device(profile.loader),profile.loader,profile.max_tokens,Box::new(server_chain)).unwrap().generate_checkpointed(&request,None,&mut |_|Ok(()),&mut ||false);
        assert!(matches!(server_refused,Err(esp::infer::InferError::StageUnavailable(_))));
        let result = serde_json::json!({"row":row.to_string(),"departure":format!("{:?}",profile.departure),"loader":profile.loader,"format":profile.format,"devices":peers.iter().map(|p|serde_json::json!({"peer":p.peer.to_bytes(),"range":p.request.range})).collect::<Vec<_>>(),"reference_seconds":reference_seconds,"staged_seconds":staged_seconds,"staged_seconds_per_token":staged_seconds/staged.token_ids.len() as f64,"recovery_seconds":recovery_seconds,"normal_traffic":normal_traffic,"reference":reference.token_ids,"staged":staged.token_ids,"continued":continued.token_ids,"placements":continued.placements,"traffic":*traffic.lock().unwrap()});
        std::fs::write(checkpoint_path.with_file_name("stage-result.json"),serde_json::to_vec_pretty(&result).unwrap()).unwrap();
        result
    }).await.unwrap();
    eprintln!("I8 staged fixture: {}", work);
    if matches!(departure, Departure::Owner) {
        assert!(
            steps
                .lock()
                .unwrap()
                .iter()
                .any(|step| matches!(step, Step::Reclaimed { .. }))
        );
    }

    tokio::time::timeout(Duration::from_secs(3), async {
        loop {
            if steps
                .lock()
                .unwrap()
                .iter()
                .any(|s| matches!(s, Step::Completed { lease: Some(_), .. }))
            {
                break;
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .expect("healthy stage commits its lease-bound completion");
    assert_eq!(
        steps
            .lock()
            .unwrap()
            .iter()
            .filter(|step| matches!(step,Step::Granted{job,..} if *job==departed_job))
            .count(),
        1,
        "an unavailable stage must not be repeatedly granted its old job"
    );
    assert!(services[1].private_drained());
    for service in &services {
        service.shutdown().await.unwrap();
    }
    let _ = stop.send(());
    pump.await.unwrap();
    drop(poster);
    for transport in &transports {
        transport.close().await.unwrap();
    }
}
