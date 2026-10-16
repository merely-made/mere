// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! V3/V5 regression: one domain validator is installed by `open_with` before
//! serving and reused by daemon composition. Real process ownership changes
//! retain the valid graph; a forbidden complete batch changes no projection,
//! session file or journal. The test harness reads durable bytes as evidence;
//! the attached application uses only admitted carrier requests.

use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc::{Receiver, channel};
use std::time::Duration;

use castellan::authority::PersonaeHost;
use castellan::custody::{IdentityVault, InMemoryStorage, Profile};
use chirograph::{
    CarrierRequestBody, CarrierResponseBody, IntentInvocation, IntentResult, ProjectionSnapshot,
};
use djinn::door::app_broker::{AppEndpointCatalog, serve_app_broker};
use djinn::embedded_reservoir::{
    ApplicationReservoirOptions, ReservoirAttachment, ReservoirAttachmentOptions,
};
use djinn::resident_mere::MereRoutes;
use djinn::resident_reservoir::ResidentReservoir;
use graphshell_endpoint::door::app_admission::{AllowedAppRoutes, AppId, AppRouteGrants};
use graphshell_endpoint::door::app_client::AppBrokerClient;
use graphshell_endpoint::door::identity::VaultProtectionView;
use graphshell_endpoint::door::tasks::ResidentTasks;
use graphshell_host::endpoint_catalog::ResidentEndpointCatalog;
use mere_overmap::wire::{APPLY_EDITS_INTENT, ApplyEditsV1};
use pandect::{CapturedDelta, DomainId, GraphValidator, MereId, mere_dir};
use personae::{Ed25519Keypair, PersonaId, ProfileId};
use sceno::InstanceId;
use uuid::Uuid;

const CHILD: &str = "child_serves_the_registered_domain";
const DOMAIN: &str = "validated-fixture";
const ROOT: &str = "DJINN_V35_ROOT";
const ENDPOINT: &str = "DJINN_V35_ENDPOINT";
const MODE: &str = "DJINN_V35_MODE";
const PRODUCT: &str = "DJINN_V35_PRODUCT";
const PATIENCE: Duration = Duration::from_secs(30);

struct NoLocalPrompt;
impl djinn::startup_vault::Prompt for NoLocalPrompt {
    fn ask(
        &mut self,
        _: &djinn::startup_vault::Ask,
    ) -> Result<djinn::startup_vault::Answer, String> {
        panic!("live resident attachment must not prompt for local custody");
    }
}

fn persona() -> PersonaId {
    PersonaId::from_uuid(Uuid::from_u128(0x5633_5635))
}
fn endpoint() -> String {
    #[cfg(windows)]
    return format!(r"\\.\pipe\djinn-v35-{}", Uuid::new_v4());
    #[cfg(not(windows))]
    return format!("/tmp/dv35-{}.sock", Uuid::new_v4());
}
fn options(root: &Path, address: &str) -> ReservoirAttachmentOptions {
    ReservoirAttachmentOptions {
        endpoint: address.into(),
        application: AppId::new("validator-fixture"),
        applications: vec![AppId::new("validator-fixture")],
        shared_root: root.into(),
        persona: persona(),
        domain: DomainId::new(DOMAIN).unwrap(),
        session_duration: Duration::from_secs(60),
    }
}
fn identity() -> Arc<PersonaeHost<InMemoryStorage>> {
    Arc::new(PersonaeHost::new(
        IdentityVault::with_profile(
            InMemoryStorage::new(),
            Profile::new(
                ProfileId("v35-fixture".into()),
                "V3/V5 fixture",
                Ed25519Keypair::from_seed([0x35; 32]),
            ),
        ),
        None,
        VaultProtectionView::Ephemeral,
    ))
}
fn validator(calls: Arc<AtomicUsize>) -> GraphValidator {
    Arc::new(move |graph| {
        calls.fetch_add(1, Ordering::SeqCst);
        if graph.nodes().any(|(_, node)| node.title == "Forbidden") {
            Err("fixture forbids the complete candidate".into())
        } else {
            Ok(())
        }
    })
}
fn revision() -> String {
    let git = |arguments: &[&str]| {
        Command::new("git")
            .args(["-C", env!("CARGO_MANIFEST_DIR")])
            .args(arguments)
            .output()
            .unwrap()
            .stdout
    };
    let mut revision = String::from_utf8(git(&["rev-parse", "HEAD"]))
        .unwrap()
        .trim()
        .to_owned();
    if !git(&["status", "--porcelain"]).is_empty() {
        revision.push_str("+dirty-diagnostic");
    }
    revision
}

struct Process {
    child: Child,
    lines: Receiver<String>,
}
impl Process {
    fn start(root: &Path, address: &str, mode: &str) -> Self {
        Self::start_mode(root, address, mode, false)
    }
    fn start_mode(root: &Path, address: &str, mode: &str, product: bool) -> Self {
        let mut child = Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                CHILD,
                "--ignored",
                "--nocapture",
                "--test-threads",
                "1",
            ])
            .env(ROOT, root)
            .env(ENDPOINT, address)
            .env(MODE, mode)
            .env(PRODUCT, if product { "divination" } else { "fixture" })
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .unwrap();
        let (send, lines) = channel();
        let stdout = child.stdout.take().unwrap();
        std::thread::spawn(move || {
            for line in BufReader::new(stdout).lines().map_while(Result::ok) {
                let _ = send.send(line);
            }
        });
        let process = Self { child, lines };
        process.wait_for("READY");
        process
    }
    fn wait_for(&self, marker: &str) {
        loop {
            let line = self.lines.recv_timeout(PATIENCE).unwrap();
            if line.contains(marker) {
                println!("{line}");
                break;
            }
        }
    }
    fn stop(mut self) {
        writeln!(self.child.stdin.as_mut().unwrap(), "stop").unwrap();
        self.wait_for("RELEASED");
        assert!(self.child.wait().unwrap().success());
    }
}
impl Drop for Process {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

async fn snapshots(client: &mut AppBrokerClient) -> Vec<ProjectionSnapshot> {
    let opened = client.open_session().await.unwrap();
    assert_eq!(
        opened.descriptor.projections.len(),
        3,
        "sessions, graph and V3 archive"
    );
    let mut snapshots = Vec::new();
    for offer in opened.descriptor.projections {
        snapshots.push(client.snapshot(offer.request).await.unwrap());
    }
    snapshots
}
fn node(id: u128) -> CapturedDelta {
    CapturedDelta::ReplayAddNodeWithIdIfMissing {
        id: Uuid::from_u128(id).to_string(),
        url: format!("https://{id}.validator.test/"),
        position: [0.0, 0.0],
    }
}
fn title(id: u128, title: &str) -> CapturedDelta {
    CapturedDelta::ReplaySetNodeTitleById {
        node_id: Uuid::from_u128(id).to_string(),
        title: title.into(),
    }
}
async fn apply(
    client: &mut AppBrokerClient,
    snapshot: &ProjectionSnapshot,
    edits: Vec<CapturedDelta>,
) -> IntentResult {
    let response = client
        .request_body(CarrierRequestBody::Intent(IntentInvocation {
            session: snapshot.session.clone(),
            target: InstanceId(0),
            observed_epoch: snapshot.scene.epoch,
            observed_revision: snapshot.scene.revision,
            intent: APPLY_EDITS_INTENT.into(),
            payload: serde_json::to_vec(&ApplyEditsV1::new(edits)).unwrap(),
        }))
        .await
        .unwrap();
    let CarrierResponseBody::Intent(result) = response else {
        panic!("{response:?}");
    };
    result
}
fn durable_files(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    durable_files_for(root, DOMAIN)
}
fn durable_files_for(root: &Path, domain: &str) -> BTreeMap<PathBuf, Vec<u8>> {
    fn collect(base: &Path, at: &Path, files: &mut BTreeMap<PathBuf, Vec<u8>>) {
        for entry in std::fs::read_dir(at).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                collect(base, &path, files);
            } else {
                files.insert(
                    path.strip_prefix(base).unwrap().to_owned(),
                    std::fs::read(path).unwrap(),
                );
            }
        }
    }
    let domain = DomainId::new(domain).unwrap();
    let sessions = mere_dir(root, persona(), MereId::derive(persona(), &domain)).join("sessions");
    let mut files = BTreeMap::new();
    collect(&sessions, &sessions, &mut files);
    files
}

#[tokio::test(flavor = "multi_thread")]
async fn published_divination_authority_survives_three_real_resident_owner_processes() {
    let root = tempfile::tempdir().unwrap();
    let unusable = root.path().join("no-client-store");
    std::fs::write(&unusable, b"client cannot open a private store").unwrap();
    for (phase, mode) in ["embedded", "daemon", "embedded"].into_iter().enumerate() {
        let address = endpoint();
        let process = Process::start_mode(root.path(), &address, mode, true);
        let mut attached = ReservoirAttachment::open_application(
            ApplicationReservoirOptions {
                endpoint: address,
                application: AppId::new("validator-fixture"),
                domain: DomainId::new("divination").unwrap(),
                shared_root: unusable.clone(),
                vault_dir: unusable.clone(),
                profile: None,
                persona: Some(persona()),
                session_duration: Duration::from_secs(60),
            },
            &mut NoLocalPrompt,
            |_| async {
                panic!("client must not compose local authority");
                #[allow(unreachable_code)]
                Ok::<MereRoutes, String>(unreachable!())
            },
        )
        .await
        .unwrap();
        let initial = snapshots(attached.client_mut()).await;
        if phase == 0 {
            assert_eq!(
                apply(attached.client_mut(), &initial[1], vec![node(1)]).await,
                IntentResult::Accepted
            );
        }
        let before = snapshots(attached.client_mut()).await;
        let bytes = durable_files_for(root.path(), "divination");
        let invalid = CapturedDelta::ReplaySetNodeFacetById {
            node_id: Uuid::from_u128(1).to_string(),
            facet: "cleromancy.reading/v999".into(),
            value_json: "{}".into(),
        };
        assert!(
            matches!(apply(attached.client_mut(), &before[1], vec![node(2), invalid]).await, IntentResult::Rejected { reason } if reason.contains("unsupported domain facet"))
        );
        let after = snapshots(attached.client_mut()).await;
        assert_eq!(
            serde_json::to_value(&after).unwrap(),
            serde_json::to_value(&before).unwrap()
        );
        assert_eq!(durable_files_for(root.path(), "divination"), bytes);
        assert_eq!(after[1].scene.active_items_in_order().len(), 2);
        attached.close().await.unwrap();
        println!(
            "RECEIPT DIVINATION mode={mode} phase={phase} published-product-validator=true complete-batch-refused=true projection-and-durable-bytes-unchanged=true"
        );
        process.stop();
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn registered_domain_validator_survives_ownership_and_refuses_complete_batches() {
    let root = tempfile::tempdir().unwrap();
    let unusable = root.path().join("not-a-storage-directory");
    std::fs::write(&unusable, b"attached clients cannot open stores here").unwrap();
    let forbidden_identity = || async {
        panic!("the client must not open local identity");
        #[allow(unreachable_code)]
        Ok::<_, String>(identity())
    };
    let mut original_session = None;
    for (phase, mode) in ["embedded", "daemon", "embedded"].into_iter().enumerate() {
        let address = endpoint();
        let process = Process::start(root.path(), &address, mode);
        assert_ne!(process.child.id(), std::process::id());
        let mut capsule = ReservoirAttachment::open_application(
            ApplicationReservoirOptions {
                endpoint: address.clone(),
                application: AppId::new("validator-fixture"),
                domain: DomainId::new(DOMAIN).unwrap(),
                shared_root: unusable.clone(),
                vault_dir: unusable.clone(),
                profile: None,
                persona: None,
                session_duration: Duration::from_secs(60),
            },
            &mut NoLocalPrompt,
            |_| async {
                panic!("client capsule must not compose local routes");
                #[allow(unreachable_code)]
                Ok::<MereRoutes, String>(unreachable!())
            },
        )
        .await
        .unwrap();
        assert!(!capsule.is_embedded());
        snapshots(capsule.client_mut()).await;
        capsule.close().await.unwrap();
        let mut attached =
            ReservoirAttachment::open(options(&unusable, &address), forbidden_identity)
                .await
                .unwrap();
        assert!(!attached.is_embedded());
        let start = snapshots(attached.client_mut()).await;
        if phase == 0 {
            // A temporary forbidden intermediate is allowed: domain authority
            // checks the complete final candidate, not each individual delta.
            assert_eq!(
                apply(
                    attached.client_mut(),
                    &start[1],
                    vec![node(1), title(1, "Forbidden"), title(1, "Allowed")]
                )
                .await,
                IntentResult::Accepted
            );
        }
        let before = snapshots(attached.client_mut()).await;
        assert_eq!(
            before[1].scene.active_items_in_order().len(),
            2,
            "one valid node plus session item"
        );
        let sessions: Vec<_> = before[0]
            .scene
            .tables
            .sources
            .iter()
            .flatten()
            .filter(|source| source.adapter == mere_overmap::wire::SESSIONS_SOURCE)
            .filter(|source| Uuid::parse_str(&source.id).is_ok())
            .map(|source| source.id.clone())
            .collect();
        assert_eq!(sessions.len(), 1);
        match &original_session {
            None => original_session = Some(sessions),
            Some(original) => assert_eq!(&sessions, original),
        }
        let bytes = durable_files(root.path());
        assert!(
            matches!(apply(attached.client_mut(), &before[1], vec![node(2), title(1,"Forbidden")]).await,
            IntentResult::Rejected { reason } if reason.contains("fixture forbids"))
        );
        let after = snapshots(attached.client_mut()).await;
        assert_eq!(
            serde_json::to_value(&after).unwrap(),
            serde_json::to_value(&before).unwrap(),
            "rejection changes no graph, sessions or archive projection"
        );
        assert_eq!(
            durable_files(root.path()),
            bytes,
            "rejection changes no durable session/checkpoint/journal/view bytes"
        );
        if phase == 0 {
            // The callback is bound to one domain, not every mere on the door.
            let mut control = options(&unusable, &address);
            control.domain = DomainId::new("unvalidated-control").unwrap();
            let mut control = ReservoirAttachment::open(control, forbidden_identity)
                .await
                .unwrap();
            let control_graph = snapshots(control.client_mut()).await;
            assert_eq!(
                apply(
                    control.client_mut(),
                    &control_graph[1],
                    vec![node(1), title(1, "Forbidden")]
                )
                .await,
                IntentResult::Accepted
            );
            control.close().await.unwrap();
        }
        attached.close().await.unwrap();
        println!(
            "RECEIPT VALIDATOR revision={} mode={mode} phase={phase} retained-session=true whole-batch-refused=true all-projections-and-session-bytes-unchanged=true",
            revision()
        );
        process.stop();
    }
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "runs only as the real process owner launched by the regression"]
async fn child_serves_the_registered_domain() {
    let (Ok(root), Ok(address), Ok(mode)) = (
        std::env::var(ROOT),
        std::env::var(ENDPOINT),
        std::env::var(MODE),
    ) else {
        return;
    };
    let product = std::env::var(PRODUCT).as_deref() == Ok("divination");
    let mut config = options(Path::new(&root), &address);
    if product {
        config.domain = DomainId::new("divination").unwrap();
    }
    let calls = Arc::new(AtomicUsize::new(0));
    let registered = Arc::new(AtomicBool::new(false));
    let mut daemon = None;
    let mut attached = if mode == "embedded" {
        let (calls, registered) = (calls.clone(), registered.clone());
        ReservoirAttachment::open_with(
            config.clone(),
            || async { Ok(identity()) },
            move |routes| async move {
                registered.store(true, Ordering::SeqCst);
                Ok(if product {
                    routes.with_divination_authority()
                } else {
                    routes.with_domain_validator(DOMAIN, validator(calls))
                })
            },
        )
        .await
        .unwrap()
    } else {
        let reservoir = ResidentReservoir::open(&config.shared_root, Some(config.persona))
            .await
            .unwrap();
        let catalog = AppEndpointCatalog::new(ResidentEndpointCatalog::new());
        let grants = AppRouteGrants::new(AllowedAppRoutes::new([(
            config.application.clone(),
            ResidentReservoir::route(),
        )]));
        let routes = MereRoutes::new(
            config.shared_root.clone(),
            config.persona,
            catalog.clone(),
            grants.clone(),
            config.applications.clone(),
        );
        let routes = if product {
            routes.with_divination_authority()
        } else {
            routes.with_domain_validator(DOMAIN, validator(calls.clone()))
        };
        registered.store(true, Ordering::SeqCst);
        routes.load_access(&reservoir).await.unwrap();
        for mere in reservoir.meres().await {
            routes.serve(&mere).await.unwrap();
        }
        catalog
            .update(|catalog| reservoir.register(catalog, Some(routes)))
            .await
            .unwrap();
        let tasks = ResidentTasks::new();
        let (scope, door, address) = (tasks.clone(), catalog.clone(), address.clone());
        let server = tokio::spawn(async move {
            scope
                .scope(serve_app_broker(
                    &address,
                    std::sync::Arc::new(djinn::keeper::Keeper::new(identity())),
                    grants,
                    60_000,
                    None,
                    door,
                ))
                .await
        });
        for _ in 0..100 {
            if graphshell_endpoint::door::local_endpoint::connect_local(&config.endpoint)
                .await
                .is_ok()
            {
                break;
            }
            assert!(!server.is_finished());
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        daemon = Some((server, tasks, catalog, reservoir));
        ReservoirAttachment::open(config.clone(), || async {
            panic!("live daemon must not request identity");
            #[allow(unreachable_code)]
            Ok::<_, String>(identity())
        })
        .await
        .unwrap()
    };
    assert_eq!(attached.is_embedded(), mode == "embedded");
    snapshots(attached.client_mut()).await;
    assert!(registered.load(Ordering::SeqCst));
    assert!(
        product || calls.load(Ordering::SeqCst) > 0,
        "validator installed before first graph served"
    );
    println!(
        "READY mode={mode} pid={} validator-installed=true",
        std::process::id()
    );
    std::io::stdout().flush().unwrap();
    let mut line = String::new();
    std::io::stdin().read_line(&mut line).unwrap();
    attached.close().await.unwrap();
    if let Some((server, tasks, catalog, reservoir)) = daemon {
        server.abort();
        let _ = server.await;
        tasks.cancel_and_join().await;
        catalog
            .update(|catalog| *catalog = ResidentEndpointCatalog::new())
            .await;
        drop(reservoir);
    }
    ResidentReservoir::open(&config.shared_root, Some(config.persona))
        .await
        .unwrap();
    println!(
        "RELEASED mode={mode} pid={} validator-calls={}",
        std::process::id(),
        calls.load(Ordering::SeqCst)
    );
}

mod published_replay_regression {
    use super::*;
    use cleromancy_domain::{CleromancyHost, Consultation, ContextDraft, SelectionMode};
    use mere_overmap::wire::{self as session_item, SessionGraphV1};
    use muniment::MemoryBackend;
    use serde_json::Value;
    use std::sync::Mutex;

    struct PublicEntropy(u64);
    impl cleromancy_domain::moirai::clotho::EntropySource for PublicEntropy {
        fn next_u64(&mut self) -> Result<u64, cleromancy_domain::ReadingError> {
            self.0 += 1;
            Ok(self.0)
        }
    }

    // The historical published package exposes graph() but not graph_mut().
    // Use the existing thread-local observer only around this synchronous,
    // memory-only product fixture. RAII clears it before any owner I/O or replay.
    // This uses public APIs supported by both 2851 and the later portable C.
    struct ClearCapture;
    impl Drop for ClearCapture {
        fn drop(&mut self) {
            kernel::graph::set_captured_delta_hook(None);
        }
    }

    fn product_fixture(
        typed_decks: bool,
        ambient: bool,
        shuffled: bool,
        situation: bool,
        inference: bool,
    ) -> (Vec<CapturedDelta>, Value, cleromancy_domain::Reading) {
        let captured = Arc::new(Mutex::new(Vec::new()));
        let observed = captured.clone();
        kernel::graph::set_captured_delta_hook(Some(Arc::new(move |delta| {
            observed.lock().unwrap().push(delta.clone());
        })));
        let clear = ClearCapture;
        let (graph, reading) = pollster::block_on(async {
            // Historical 2851 persists only to this transient MemoryBackend.
            // No snapshot is installed in the resident: only captured edits cross its door.
            let mut consultation = Consultation::new(CleromancyHost::empty(MemoryBackend::new()));
            let field = if typed_decks {
                let fields = consultation.install_typed_tarot_at(1).await.unwrap();
                assert_eq!(fields.len(), 2);
                fields[0].clone()
            } else {
                consultation.install_builtin_tarot_at(1).await.unwrap()
            };
            let context = consultation
                .save_context_at(
                    ContextDraft::new(
                        "Published owner replay",
                        "Which boundary must survive a new owner?",
                        "ownership",
                    ),
                    2,
                )
                .await
                .unwrap();
            let detail = consultation
                .read_at_with_entropy(
                    &context,
                    &field,
                    SelectionMode::Calculated,
                    3_000,
                    3,
                    &mut PublicEntropy(0xc1),
                )
                .await
                .unwrap();
            assert_eq!(detail.readings.len(), 1);
            assert_eq!(detail.readings[0].receipt.mode, SelectionMode::Calculated);
            let mut host = consultation.into_host();
            if ambient {
                let context = host.ambient_context(&detail.session.id, "focus").unwrap();
                let item = context
                    .data
                    .items
                    .iter()
                    .find(|i| i.id.starts_with("card:"))
                    .unwrap()
                    .id
                    .clone();
                host.examine_ambient(&detail.session.id, "focus", &item, false, 4000)
                    .unwrap();
                host.examine_ambient(&detail.session.id, "focus", &item, true, 5000)
                    .unwrap();
                let mut view = context.view;
                view.snapshot.recipe.definition.arrangement.kind = "grid.ordered".into();
                view.snapshot.selected_occurrence = Some(item.clone());
                view.roles.insert(
                    item.clone(),
                    serde_json::from_value(serde_json::json!("pinned")).unwrap(),
                );
                view.foreground.insert(item.clone());
                view.positions.insert(item, [123000, 456000]);
                view.reduced_motion = true;
                host.insert_ambient_view(&view).unwrap();
            }
            let reading = if shuffled {
                let context = host.context_for_digest(&context).unwrap();
                let field = host.field_for_digest(&field).unwrap();
                let drawn = host
                    .record_three_card_spread_derived_at_with_entropy(
                        &context,
                        &field,
                        "C5 owner public seed",
                        "cleromancy.owner/shuffle",
                        if situation { 1_792_149_302_000 } else { 6000 },
                        None,
                        &mut PublicEntropy(0xc5),
                    )
                    .unwrap();
                if situation {
                    use cleromancy_domain::situation::*;
                    let place = ReadingPlace::new(51_476_900, 0).unwrap();
                    // Public synthetic response; live network provenance has
                    // a separate product-native receipt.
                    let body = r#"{"latitude":51.472404,"longitude":0.011566162,"utc_offset_seconds":0,"current_units":{"time":"unixtime","interval":"seconds","temperature_2m":"°C","relative_humidity_2m":"%","precipitation":"mm","weather_code":"wmo code","wind_speed_10m":"m/s"},"current":{"time":1792149300,"interval":900,"temperature_2m":12.8,"relative_humidity_2m":70,"precipitation":0,"weather_code":3,"wind_speed_10m":4.1}}"#;
                    let weather = WeatherObservation::from_response(
                        &place,
                        1_792_149_301_000,
                        weather_url(&place).unwrap(),
                        body.into(),
                    )
                    .unwrap();
                    let snapshot = ReadingSituation::capture(
                        &drawn.0,
                        &SituationPreferences {
                            place: Some(place),
                            weather: true,
                        },
                        Some(weather),
                    )
                    .unwrap();
                    host.insert_reading_situation(&snapshot).unwrap();
                }
                drawn.2[0].clone()
            } else {
                detail.readings[0].clone()
            };
            if inference {
                use cleromancy_domain::inference::*;
                let session = host
                    .sessions()
                    .unwrap()
                    .into_iter()
                    .max_by_key(|s| s.created_at_ms)
                    .unwrap();
                let inputs = host.inference_inputs(&session.id).unwrap();
                host.insert_generated_inference(&GeneratedInference::formula(inputs.clone()))
                    .unwrap();
                // Deliberately synthetic: this fixture qualifies stored model
                // records and receiving authority, not model execution.
                let model = ModelIdentity {
                    model_id: "fixture-only/receiving-record".into(),
                    revision: "contract-v1".into(),
                    loader: "esp-burn-ndarray".into(),
                    artifacts: ["config.json", "tokenizer.json", "model.safetensors"]
                        .map(|name| (name.into(), "a".repeat(64)))
                        .into_iter()
                        .collect(),
                };
                let record = GeneratedInference::model(
                    inputs,
                    model,
                    "Synthetic receiving fixture output; no model was executed by this fixture."
                        .into(),
                );
                host.insert_generated_inference(&record).unwrap();
                host.keep_generated_inference(&record.digest()).unwrap();
            }
            (host.graph().clone(), reading)
        });
        drop(clear);
        let edits = std::mem::take(&mut *captured.lock().unwrap());
        assert!(!edits.is_empty(), "the real product emitted captured edits");
        let replay = kernel::graph::replay_captured_deltas(edits.clone());
        assert_eq!(replay.node_count(), graph.node_count());
        assert_eq!(
            serde_json::to_value(replay.facets()).unwrap(),
            serde_json::to_value(graph.facets()).unwrap(),
            "the captured batch retains every product facet"
        );
        // A control for the fixture, not the owner assertion: the real admitted
        // transaction below independently invokes this same published supplier.
        cleromancy_domain::host::validate_divination_graph(&replay).unwrap();
        (
            edits,
            serde_json::to_value(graph.facets()).unwrap(),
            reading,
        )
    }

    async fn disclosed(
        client: &mut AppBrokerClient,
        snapshot: &ProjectionSnapshot,
    ) -> SessionGraphV1 {
        let offer = &snapshot.presentation.offers_for(InstanceId(0)).unwrap()[0];
        let bytes = client
            .resource(snapshot.session.clone(), offer.resource)
            .await
            .unwrap();
        assert_eq!(chirograph::ContentHash::of(&bytes), offer.resource);
        let card: chirograph::PortableCardV1 = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(
            card.media.len(),
            1,
            "session card advertises its complete graph"
        );
        let media = card.media[0];
        let bytes = client
            .resource(snapshot.session.clone(), media)
            .await
            .unwrap();
        assert_eq!(chirograph::ContentHash::of(&bytes), media);
        let envelope: SessionGraphV1 = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(envelope.schema, session_item::SESSION_GRAPH_SCHEMA);
        assert_eq!(envelope.application, "validator-fixture");
        assert_eq!(envelope.epoch, snapshot.scene.epoch);
        assert_eq!(envelope.revision, snapshot.scene.revision);
        assert!(!envelope.read_only);
        envelope
    }

    // Fixture seeding may contain several complete decks. Each bounded
    // transaction remains a fully valid candidate; no frame cap or refusal
    // guard is widened to fit a combined acceptance setup packet.
    async fn apply_bounded_fixture(client: &mut AppBrokerClient, edits: &[CapturedDelta]) {
        let mut accepted: Vec<CapturedDelta> = Vec::new();
        let mut offset = 0;
        while offset < edits.len() {
            let current = snapshots(client).await;
            let mut choice = None;
            for end in (offset + 1..=edits.len()).rev() {
                let batch = edits[offset..end].to_vec();
                let payload =
                    serde_json::to_vec(&ApplyEditsV1::checked(current[1].scene.revision, batch))
                        .unwrap();
                if serde_json::to_vec(&payload).unwrap().len() > 900_000 {
                    continue;
                }
                let candidate = kernel::graph::replay_captured_deltas(
                    accepted
                        .iter()
                        .cloned()
                        .chain(edits[offset..end].iter().cloned()),
                );
                if cleromancy_domain::host::validate_divination_graph(&candidate).is_ok() {
                    choice = Some(end);
                    break;
                }
            }
            let end =
                choice.expect("fixture must have a replay-valid prefix below the carrier limit");
            assert_eq!(
                apply_checked(client, &current[1], edits[offset..end].to_vec()).await,
                IntentResult::Accepted
            );
            println!(
                "BOUNDED PRODUCT FIXTURE deltas={} replay-valid-candidate=true",
                end - offset
            );
            accepted.extend_from_slice(&edits[offset..end]);
            offset = end;
        }
    }

    async fn apply_checked(
        client: &mut AppBrokerClient,
        snapshot: &ProjectionSnapshot,
        edits: Vec<CapturedDelta>,
    ) -> IntentResult {
        let response = client
            .request_body(CarrierRequestBody::Intent(IntentInvocation {
                session: snapshot.session.clone(),
                target: InstanceId(0),
                observed_epoch: snapshot.scene.epoch,
                observed_revision: snapshot.scene.revision,
                intent: APPLY_EDITS_INTENT.into(),
                payload: serde_json::to_vec(&ApplyEditsV1::checked(snapshot.scene.revision, edits))
                    .unwrap(),
            }))
            .await
            .unwrap();
        let CarrierResponseBody::Intent(result) = response else {
            panic!("unexpected owner response: {response:?}");
        };
        result
    }

    fn journal_and_history(files: &BTreeMap<PathBuf, Vec<u8>>) -> BTreeMap<PathBuf, Vec<u8>> {
        let logs: BTreeMap<_, _> = files
            .iter()
            .filter(|(path, _)| {
                matches!(
                    path.file_name().and_then(|name| name.to_str()),
                    Some("journal.jsonl" | "changes.jsonl")
                )
            })
            .map(|(path, bytes)| (path.clone(), bytes.clone()))
            .collect();
        for name in ["journal.jsonl", "changes.jsonl"] {
            assert!(
                logs.iter().any(
                    |(path, bytes)| path.file_name().and_then(|name| name.to_str()) == Some(name)
                        && !bytes.is_empty()
                ),
                "accepted product batch must retain nonempty {name}"
            );
        }
        logs
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn published_reading_replay_and_same_schema_tamper_survive_three_real_owners() {
        owner_replay(false, false, false, false, false).await;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn typed_decks_and_reading_survive_three_owners_and_refuse_metadata_tamper() {
        owner_replay(true, false, false, false, false).await;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn ambient_view_and_kept_memory_survive_three_owners_and_refuse_source_tamper() {
        owner_replay(true, true, false, false, false).await;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn shuffled_deck_survives_three_owners_and_refuses_orientation_tamper() {
        owner_replay(true, false, true, false, false).await;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn recorded_situation_and_weather_survive_three_owners_and_refuse_tamper() {
        owner_replay(true, false, true, true, false).await;
    }
    #[tokio::test(flavor = "multi_thread")]
    async fn generated_inferences_survive_three_owners_and_refuse_input_and_output_tamper() {
        owner_replay(true, false, true, false, true).await;
    }
    async fn owner_replay(
        typed_decks: bool,
        ambient: bool,
        shuffled: bool,
        situation: bool,
        inference: bool,
    ) {
        let root = tempfile::tempdir().unwrap();
        let unusable = root.path().join("no-attached-client-store");
        std::fs::write(
            &unusable,
            b"the live client cannot open local custody or storage",
        )
        .unwrap();
        let (edits, expected_facets, reading) =
            product_fixture(typed_decks, ambient, shuffled, situation, inference);
        let reading_url = format!("cleromancy://reading/{}", reading.id);
        let reading_id = kernel::graph::Graph::node_namespace_id(&reading_url);
        let mut retained_session = None;
        let mut retained_graph = None;
        let mut retained_logs = None;
        for (phase, mode) in ["embedded", "daemon", "embedded"].into_iter().enumerate() {
            let address = endpoint();
            let process = Process::start_mode(root.path(), &address, mode, true);
            assert_ne!(process.child.id(), std::process::id());
            let mut attached = ReservoirAttachment::open_application(
                ApplicationReservoirOptions {
                    endpoint: address,
                    application: AppId::new("validator-fixture"),
                    domain: DomainId::new("divination").unwrap(),
                    shared_root: unusable.clone(),
                    vault_dir: unusable.clone(),
                    profile: None,
                    persona: Some(persona()),
                    session_duration: Duration::from_secs(60),
                },
                &mut NoLocalPrompt,
                |_| async {
                    panic!("an attached client must not compose local authority");
                    #[allow(unreachable_code)]
                    Ok::<MereRoutes, String>(unreachable!())
                },
            )
            .await
            .unwrap();
            assert!(!attached.is_embedded());
            let initial = snapshots(attached.client_mut()).await;
            if phase == 0 {
                let empty = disclosed(attached.client_mut(), &initial[1])
                    .await
                    .into_graph()
                    .unwrap();
                assert_eq!(
                    empty.node_count(),
                    0,
                    "the owner has no fixture-seeded graph"
                );
                if shuffled || typed_decks {
                    apply_bounded_fixture(attached.client_mut(), &edits).await;
                } else {
                    assert_eq!(
                        apply_checked(attached.client_mut(), &initial[1], edits.clone()).await,
                        IntentResult::Accepted,
                        "real published replay authority accepts the complete product batch"
                    );
                }
            }
            let before = snapshots(attached.client_mut()).await;
            let envelope = disclosed(attached.client_mut(), &before[1]).await;
            let session = envelope.session;
            let graph = envelope.clone().into_graph().unwrap();
            assert_eq!(
                serde_json::to_value(graph.facets()).unwrap(),
                expected_facets
            );
            let stored = graph
                .facets()
                .get(
                    &reading_id,
                    &pandect::facet_store::FacetId::new(cleromancy_domain::host::READING_FACET),
                )
                .unwrap();
            assert_eq!(
                serde_json::from_value::<cleromancy_domain::Reading>(stored.clone()).unwrap(),
                reading
            );
            let graph_state = serde_json::to_value(graph.to_snapshot_at(0)).unwrap();
            let durable = durable_files_for(root.path(), "divination");
            let logs = journal_and_history(&durable);
            if typed_decks {
                let fields: Vec<_> = graph
                    .nodes()
                    .filter(|(_, node)| node.url().starts_with("cleromancy://field/"))
                    .collect();
                assert_eq!(fields.len(), 2);
                for (_, field_node) in &fields {
                    let stored = graph
                        .facets()
                        .get(
                            &field_node.id,
                            &pandect::facet_store::FacetId::new(
                                cleromancy_domain::host::FIELD_FACET,
                            ),
                        )
                        .unwrap();
                    let field: cleromancy_domain::Field =
                        serde_json::from_value(stored.clone()).unwrap();
                    assert_eq!(field.tarot.as_ref().unwrap().cards.len(), 78);
                    assert!(field.position_meanings.is_some());
                }
                let (_, field_node) = fields[0];
                let mut forged = graph
                    .facets()
                    .get(
                        &field_node.id,
                        &pandect::facet_store::FacetId::new(cleromancy_domain::host::FIELD_FACET),
                    )
                    .unwrap()
                    .clone();
                forged["tarot"]["cards"][0]["title"] = serde_json::json!("Forged typed card title");
                let result = apply_checked(
                    attached.client_mut(),
                    &before[1],
                    vec![
                        node(0xc300 + phase as u128),
                        CapturedDelta::ReplaySetNodeFacetById {
                            node_id: field_node.id.to_string(),
                            facet: cleromancy_domain::host::FIELD_FACET.into(),
                            value_json: forged.to_string(),
                        },
                    ],
                )
                .await;
                assert!(
                    matches!(result, IntentResult::Rejected { .. }),
                    "forged deck metadata must be refused: {result:?}"
                );
                let after = snapshots(attached.client_mut()).await;
                assert_eq!(
                    serde_json::to_value(&after).unwrap(),
                    serde_json::to_value(&before).unwrap()
                );
                assert_eq!(durable_files_for(root.path(), "divination"), durable);
                println!(
                    "TYPED DECK REPLAY mode={mode} phase={phase} decks=2 cards-per-deck=78 content-revisions-retained=true metadata-tamper-refused=true durable-bytes-unchanged=true"
                );
            }
            if ambient {
                let host = cleromancy_domain::CleromancyHost::candidate(graph.clone()).unwrap();
                let views = host.ambient_views().unwrap();
                let memories = host.ambient_memories().unwrap();
                assert_eq!(views.len(), 1);
                assert!(views[0].reduced_motion);
                assert_eq!(
                    views[0].snapshot.recipe.definition.arrangement.kind,
                    "grid.ordered"
                );
                assert_eq!(memories.len(), 1);
                assert!(memories[0].kept);
                assert_eq!(views[0].roles[&memories[0].memory.item.id].id(), "pinned");
                assert!(views[0].foreground.contains(&memories[0].memory.item.id));
                for facet in [
                    cleromancy_domain::host::AMBIENT_VIEW_FACET,
                    cleromancy_domain::host::AMBIENT_MEMORY_FACET,
                ] {
                    let (node_id, mut forged) = graph
                        .facets()
                        .iter()
                        .find_map(|(id, facets)| {
                            facets
                                .iter()
                                .find(|(name, _)| name.as_str() == facet)
                                .map(|(_, v)| (*id, v.clone()))
                        })
                        .unwrap();
                    if facet == cleromancy_domain::host::AMBIENT_VIEW_FACET {
                        forged["snapshot"]["selected_occurrence"] =
                            serde_json::json!("undisclosed");
                    } else {
                        forged["item"]["body"] = serde_json::json!("not the saved engine result");
                    }
                    let result = apply_checked(
                        attached.client_mut(),
                        &before[1],
                        vec![CapturedDelta::ReplaySetNodeFacetById {
                            node_id: node_id.to_string(),
                            facet: facet.into(),
                            value_json: forged.to_string(),
                        }],
                    )
                    .await;
                    assert!(
                        matches!(result, IntentResult::Rejected { .. }),
                        "C4 source/curation tamper accepted: {result:?}"
                    );
                    assert_eq!(
                        serde_json::to_value(snapshots(attached.client_mut()).await).unwrap(),
                        serde_json::to_value(&before).unwrap()
                    );
                    assert_eq!(durable_files_for(root.path(), "divination"), durable);
                }
                println!(
                    "AMBIENT REPLAY mode={mode} phase={phase} focus-roles-foreground-and-spec-retained=true saved-memory=true source-tamper-refused=true durable-bytes-unchanged=true"
                );
            }
            if shuffled {
                let draw = reading.receipt.shuffle.as_ref().unwrap();
                assert_eq!(draw.count, 3);
                let plan = draw
                    .shuffle
                    .replay(
                        &cleromancy_domain::CleromancyHost::candidate(graph.clone())
                            .unwrap()
                            .context_for_digest(&reading.receipt.context_digest)
                            .unwrap(),
                        &cleromancy_domain::CleromancyHost::candidate(graph.clone())
                            .unwrap()
                            .field_for_digest(&reading.receipt.field_digest)
                            .unwrap(),
                    )
                    .unwrap();
                assert_eq!(plan.len(), 78);
                let mut forged = stored.clone();
                forged["receipt"]["shuffle"]["orientation"] = serde_json::json!(if reading
                    .orientation()
                    == cleromancy_domain::Orientation::Reversed
                {
                    "upright"
                } else {
                    "reversed"
                });
                let result = apply_checked(
                    attached.client_mut(),
                    &before[1],
                    vec![
                        node(0xc500 + phase as u128),
                        CapturedDelta::ReplaySetNodeFacetById {
                            node_id: reading_id.to_string(),
                            facet: cleromancy_domain::host::READING_FACET.into(),
                            value_json: forged.to_string(),
                        },
                    ],
                )
                .await;
                assert!(matches!(result, IntentResult::Rejected { .. }));
                assert_eq!(
                    serde_json::to_value(snapshots(attached.client_mut()).await).unwrap(),
                    serde_json::to_value(&before).unwrap()
                );
                assert_eq!(durable_files_for(root.path(), "divination"), durable);
                println!(
                    "SHUFFLED DECK REPLAY mode={mode} phase={phase} full-deck=78 prefix=3 orientation-tamper-refused=true durable-bytes-unchanged=true"
                );
            }
            if situation {
                use cleromancy_domain::situation::*;
                let host = cleromancy_domain::CleromancyHost::candidate(graph.clone()).unwrap();
                let records = host.reading_situations().unwrap();
                assert_eq!(records.len(), 1);
                assert_eq!(
                    records[0]
                        .weather
                        .as_ref()
                        .unwrap()
                        .values
                        .temperature_millicelsius,
                    12_800
                );
                let facet = SITUATION_FACET;
                let (node_id, mut forged) = graph
                    .facets()
                    .iter()
                    .find_map(|(id, facets)| {
                        facets
                            .iter()
                            .find(|(name, _)| name.as_str() == facet)
                            .map(|(_, v)| (*id, v.clone()))
                    })
                    .unwrap();
                forged["weather"]["values"]["temperature_millicelsius"] = serde_json::json!(99_999);
                let result = apply_checked(
                    attached.client_mut(),
                    &before[1],
                    vec![
                        node(0xc600 + phase as u128),
                        CapturedDelta::ReplaySetNodeFacetById {
                            node_id: node_id.to_string(),
                            facet: facet.into(),
                            value_json: forged.to_string(),
                        },
                    ],
                )
                .await;
                assert!(matches!(result, IntentResult::Rejected { .. }));
                assert_eq!(
                    serde_json::to_value(snapshots(attached.client_mut()).await).unwrap(),
                    serde_json::to_value(&before).unwrap()
                );
                assert_eq!(durable_files_for(root.path(), "divination"), durable);
                println!(
                    "SITUATION REPLAY mode={mode} phase={phase} saved-response-exact=true network-free-replay=true weather-tamper-refused=true durable-bytes-unchanged=true"
                );
            }
            if inference {
                use cleromancy_domain::inference::*;
                let host = cleromancy_domain::CleromancyHost::candidate(graph.clone()).unwrap();
                let records = host.generated_inferences().unwrap();
                assert_eq!(records.len(), 2);
                for record in records {
                    let key = graph
                        .get_node_by_url(&format!("cleromancy://inference/{}", record.digest()))
                        .unwrap()
                        .0;
                    let id = graph.get_node(key).unwrap().id;
                    match record.generation {
                        Generation::Formula { .. } => {
                            assert_eq!(record.output, record.inputs.formula_output())
                        },
                        Generation::Model { .. } => {
                            assert_eq!(record.label, LABEL_MODEL);
                            assert!(
                                record
                                    .output
                                    .starts_with("Synthetic receiving fixture output")
                            );
                            assert!(
                                graph
                                    .node_content_tags(key)
                                    .is_some_and(|tags| tags.contains("kept"))
                            );
                        },
                    }
                    for case in 0..3 {
                        let mut forged = serde_json::to_value(&record).unwrap();
                        if case == 0 {
                            forged["inputs"]["context_text"] =
                                serde_json::json!("fabricated source input");
                        } else if case == 1 {
                            forged["output"] = serde_json::json!("fabricated output");
                        } else {
                            forged["label"] = serde_json::json!("Unlabelled invented output");
                        }
                        let result = apply_checked(
                            attached.client_mut(),
                            &before[1],
                            vec![
                                node(0xc700 + phase as u128 * 6 + case),
                                CapturedDelta::ReplaySetNodeFacetById {
                                    node_id: id.to_string(),
                                    facet: INFERENCE_FACET.into(),
                                    value_json: forged.to_string(),
                                },
                            ],
                        )
                        .await;
                        assert!(matches!(result, IntentResult::Rejected { .. }));
                        assert_eq!(
                            serde_json::to_value(snapshots(attached.client_mut()).await).unwrap(),
                            serde_json::to_value(&before).unwrap()
                        );
                        assert_eq!(durable_files_for(root.path(), "divination"), durable);
                    }
                }
                println!(
                    "INFERENCE REPLAY mode={mode} phase={phase} formula-exact=true stored-model-label-retained=true explicit-keep-retained=true input-and-disclosure-tamper-refused=true durable-bytes-unchanged=true"
                );
            }
            if let Some(id) = retained_session {
                assert_eq!(session, id, "the same owner-held session reopens");
                assert_eq!(retained_graph.as_ref().unwrap(), &graph_state);
                assert_eq!(retained_logs.as_ref().unwrap(), &logs);
            } else {
                retained_session = Some(session);
                retained_graph = Some(graph_state.clone());
                retained_logs = Some(logs.clone());
            }
            for (case, label) in ["candidate_id", "receipt_total_weight"]
                .into_iter()
                .enumerate()
            {
                let mut tampered = stored.clone();
                if case == 0 {
                    tampered["candidate_id"] = serde_json::json!("invented-c1-candidate");
                } else {
                    tampered["receipt"]["total_weight"] =
                        serde_json::json!(reading.receipt.total_weight + 1);
                }
                let typed: cleromancy_domain::Reading =
                    serde_json::from_value(tampered.clone()).unwrap();
                assert_eq!(typed.schema, reading.schema);
                assert_eq!(typed.receipt.schema, reading.receipt.schema);
                assert_eq!(typed.id, reading.id);
                assert!(typed.schema.ends_with(if shuffled { "/v3" } else { "/v1" }));
                assert!(
                    typed
                        .receipt
                        .schema
                        .ends_with(if shuffled { "/v5" } else { "/v1" })
                );
                let result = apply_checked(
                    attached.client_mut(),
                    &before[1],
                    vec![
                        node(0xc100 + phase as u128 * 2 + case as u128),
                        CapturedDelta::ReplaySetNodeFacetById {
                            node_id: reading_id.to_string(),
                            facet: cleromancy_domain::host::READING_FACET.into(),
                            value_json: tampered.to_string(),
                        },
                    ],
                )
                .await;
                let IntentResult::Rejected { reason } = result else {
                    panic!("owner must refuse supported-schema {label} tamper: {result:?}");
                };
                assert!(
                    reason.contains("receipt"),
                    "expected replay mismatch, got {reason}"
                );
                assert!(
                    !reason.contains("unsupported domain facet"),
                    "schema rejection is not this proof"
                );
                let after = snapshots(attached.client_mut()).await;
                assert_eq!(
                    serde_json::to_value(&after).unwrap(),
                    serde_json::to_value(&before).unwrap()
                );
                let after_envelope = disclosed(attached.client_mut(), &after[1]).await;
                assert_eq!(
                    serde_json::to_value(after_envelope).unwrap(),
                    serde_json::to_value(&envelope).unwrap()
                );
                let after_bytes = durable_files_for(root.path(), "divination");
                assert_eq!(journal_and_history(&after_bytes), logs);
                assert_eq!(
                    after_bytes, durable,
                    "rejected whole batch changes no durable session state"
                );
                println!(
                    "RECEIPT REPLAY mode={mode} phase={phase} case={label} same-v1-schema=true replay-refused=true projection-revision-and-journal-history-and-durable-bytes-unchanged=true"
                );
            }
            attached.close().await.unwrap();
            process.stop();
        }
    }
}
