// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Receipt-only live Distillery Chronicle host.
//!
//! This binary is intentionally separate from `djinn`: it creates a temporary
//! Personae vault, composes a real `DjinnResident`, posts one real
//! `mesh.blake3/v1` job, pauses after its first authority fold, and serves the
//! resident-owned `distillery.chronicle` observer through Graphshell's WebRTC
//! door. It is a headed receipt host,
//! not a production deployment path. In particular, its invitation and
//! release reference live only for this process and are generated for the
//! receipt run.
//!
//! Run:
//!
//! ```text
//! cargo run -p djinn --features distillery-chronicle-receipt-host --bin \
//!     distillery_chronicle_receipt_host -- --bind 192.168.4.36 \
//!     --advertise 192.168.4.36
//! ```
//!
//! Paste the printed invite fragment into the Distillery Chronicle page and
//! point its signaling URL at the printed `/offer` endpoint. The browser
//! should see one `Job ...` card at a revision greater than one.

use std::collections::BTreeMap;
use std::net::{IpAddr, SocketAddr};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use chirograph::ProjectionSession;
use distillery::{
    ChronicleEndpoint, ChronicleObserver, ChronicleRevision, ResidentReceipt,
    chronicle_shelfmark_input, verify_chronicle_shelfmark_input,
};
use djinn::resident::DjinnResident;
use djinn::resident_distillery::ResidentDistillery;
use djinn::settings::{DistilleryLaneSettings, OwnerSettings};
use graphshell::carrier::projection_policy;
use graphshell::native::projection_host::{ResidentProjectionHost, admit_webrtc_catalog};
use graphshell::projection_editor::{
    ProjectionInputBinding, PublicSourceRevision, RevisionEvidence, SourceBinding,
    chronicle_definition, chronicle_era_bands_off_variant,
};
use graphshell::webrtc_door::{InviteTerms, issue_invite};
use graphshell::webrtc_session::HostedInvite;
use incipit::ShelfmarkV1;
use mesh::spec::{DeterminismClass, JobSpec};
use mesh::{Job, JobBoard, JobBoardSnapshot, ResourceId};
use notochord::{NetworkId, ProfileRef, TrustedRoot};
use pandect::{DeviceSettings, MeshLendingSettings, StatedConditionSettings};
use personae::bootstrap::{self, Unlock};
use personae::{IdentityProvider, IdentityVault, InMemoryProvider, ProfileId};
use rand_core::{OsRng, RngCore};
use serde::Deserialize;
use serde_json::json;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Mutex, Notify};
use webrtc_carrier::ReleaseRefV1;
use webrtc_carrier::native::{Answerer, AnswererConfig, CarrierConfig};

const NETWORK: NetworkId = NetworkId([3; 32]);
const ROOT_AUTHORITY: [u8; 32] = [7; 32];
const PROFILE: &str = "works";
const PASSPHRASE: &[u8] = b"mere-distillery-chronicle-receipt-host";
const DELEGATION_TTL_MS: u64 = 60 * 60 * 1000;
const CHRONICLE_BINDING_RECEIPT_SCHEMA: &str = "mere.chronicle-binding-receipt/1";
const DISTILLERY_BOARD_FIXTURE: &str =
    include_str!("../../../distillery/tests/fixtures/chronicle/distillery_board.json");

#[derive(Deserialize)]
struct ChronicleFixture {
    jobs: Vec<Job>,
}

struct Args {
    signal_port: u16,
    bind: IpAddr,
    advertise: Vec<IpAddr>,
    udp_port: u16,
    uses: u32,
    invite_ttl_ms: u64,
}

impl Args {
    fn parse() -> Result<Self, String> {
        let mut args = Self {
            signal_port: 8788,
            bind: IpAddr::from([0, 0, 0, 0]),
            advertise: Vec::new(),
            udp_port: 0,
            uses: 8,
            invite_ttl_ms: DELEGATION_TTL_MS,
        };
        let mut argv = std::env::args().skip(1);
        while let Some(flag) = argv.next() {
            let mut value = || argv.next().ok_or_else(|| format!("{flag} needs a value"));
            match flag.as_str() {
                "--signal-port" => {
                    args.signal_port = value()?.parse().map_err(|_| "bad --signal-port")?
                },
                "--bind" => args.bind = value()?.parse().map_err(|_| "bad --bind address")?,
                "--advertise" => args
                    .advertise
                    .push(value()?.parse().map_err(|_| "bad --advertise address")?),
                "--udp-port" => args.udp_port = value()?.parse().map_err(|_| "bad --udp-port")?,
                "--uses" => args.uses = value()?.parse().map_err(|_| "bad --uses")?,
                "--invite-ttl-ms" => {
                    args.invite_ttl_ms = value()?.parse().map_err(|_| "bad --invite-ttl-ms")?
                },
                "--help" | "-h" => return Err(USAGE.to_string()),
                other => return Err(format!("unknown flag {other}\n\n{USAGE}")),
            }
        }
        Ok(args)
    }
}

const USAGE: &str = "distillery_chronicle_receipt_host \
    [--signal-port 8788] [--bind 0.0.0.0] [--advertise IP]... \
    [--udp-port 0] [--uses 8] [--invite-ttl-ms 3600000]";

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("the clock is after 1970")
        .as_millis() as u64
}

fn chronicle_binding_receipt(observer: &ChronicleObserver) -> Result<String, String> {
    let fixture: ChronicleFixture = serde_json::from_str(DISTILLERY_BOARD_FIXTURE)
        .map_err(|error| format!("read Distillery W0 fixture: {error}"))?;
    let fixture_board = JobBoard::fold_from_snapshot(
        [0xd1; 32],
        &JobBoardSnapshot { jobs: fixture.jobs },
        std::iter::empty(),
    );
    let distillery = ChronicleEndpoint::new_at_tick(
        ProjectionSession("receipt:w2-distillery".into()),
        &fixture_board,
        6,
        ChronicleRevision::new(41, 1, 1),
    );
    let djinn = observer.endpoint(ProjectionSession("receipt:w2-djinn".into()));
    if distillery.score().items.is_empty() || djinn.score().items.is_empty() {
        return Err("both Chronicle datasets must produce at least one item".into());
    }

    let distillery_source = SourceBinding {
        authority: "distillery.fixture".into(),
        domain: "mesh.job-board".into(),
        resource: "fixtures/chronicle/distillery_board.json".into(),
    };
    let djinn_source = SourceBinding {
        authority: "djinn.resident".into(),
        domain: "mesh.job-board".into(),
        resource: "distillery.chronicle".into(),
    };
    let distillery_generation = distillery.dataset_generation();
    let djinn_generation = djinn.dataset_generation();
    let definition = chronicle_definition(
        BTreeMap::from([
            (
                "distillery".into(),
                ProjectionInputBinding {
                    source: distillery_source.clone(),
                    expects_generation: Some(PublicSourceRevision::from(
                        distillery_generation.clone(),
                    )),
                    revision_evidence: RevisionEvidence::PublicGeneration,
                },
            ),
            (
                "djinn".into(),
                ProjectionInputBinding {
                    source: djinn_source.clone(),
                    expects_generation: Some(PublicSourceRevision::from(djinn_generation.clone())),
                    revision_evidence: RevisionEvidence::PublicGeneration,
                },
            ),
        ]),
        "Distillery W2 receipt host",
        "chronicle-definition-v1",
    );
    definition
        .validate()
        .map_err(|error| format!("validate authored Chronicle definition: {error}"))?;
    definition
        .bind("distillery", None)
        .map_err(|error| format!("bind Distillery fixture: {error}"))?;
    definition
        .bind("djinn", None)
        .map_err(|error| format!("bind Djinn resident: {error}"))?;

    let mut shelfmark = ShelfmarkV1::new(definition.id.clone());
    let distillery_input = chronicle_shelfmark_input(
        serde_json::to_string(&distillery_source)
            .map_err(|error| format!("serialize Distillery authority record: {error}"))?,
        distillery_generation.clone(),
    );
    let djinn_input = chronicle_shelfmark_input(
        serde_json::to_string(&djinn_source)
            .map_err(|error| format!("serialize Djinn authority record: {error}"))?,
        djinn_generation.clone(),
    );
    verify_chronicle_shelfmark_input(&distillery_input, &distillery_generation)
        .map_err(|error| error.to_string())?;
    verify_chronicle_shelfmark_input(&djinn_input, &djinn_generation)
        .map_err(|error| error.to_string())?;
    shelfmark
        .inputs
        .insert("distillery".into(), distillery_input);
    shelfmark.inputs.insert("djinn".into(), djinn_input);
    shelfmark
        .validate()
        .map_err(|error| format!("validate combined Chronicle shelfmark: {error:?}"))?;
    let shelfmark_wire = serde_json::to_string(&shelfmark)
        .map_err(|error| format!("serialize combined Chronicle shelfmark: {error}"))?;
    let shelfmark_round_trip = serde_json::from_str::<ShelfmarkV1>(&shelfmark_wire)
        .map_err(|error| format!("decode combined Chronicle shelfmark: {error}"))?
        == shelfmark;
    let variant = chronicle_era_bands_off_variant();
    definition
        .bind("djinn", Some(&variant))
        .map_err(|error| format!("bind Chronicle variant: {error}"))?;

    serde_json::to_string_pretty(&json!({
        "schema": CHRONICLE_BINDING_RECEIPT_SCHEMA,
        "definition": definition,
        "bindings": [
            {
                "role": "distillery",
                "definition_id": "distillery.chronicle",
                "source": distillery_source,
                "generation": distillery_generation,
                "read": true
            },
            {
                "role": "djinn",
                "definition_id": "distillery.chronicle",
                "source": djinn_source,
                "generation": djinn_generation,
                "read": true
            }
        ],
        "variant": variant,
        "shelfmark": shelfmark,
        "shelfmark_round_trip": shelfmark_round_trip
    }))
    .map_err(|error| format!("serialize Chronicle W2 binding receipt: {error}"))
}

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse().map_err(|error| -> Box<dyn std::error::Error> { error.into() })?;
    let root = temporary_root()?;
    let (vault_dir, identity) = open_vault(&root)?;
    let data_root = root.join("data");
    write_device_settings(&data_root)?;

    let profile = ProfileId(PROFILE.into());
    let owner = OwnerSettings {
        distillery: Some(DistilleryLaneSettings {
            tick_every_ms: 25,
            maintenance_every_ms: None,
            blob_gc_every_ms: 1_000,
            collect_after_checkpoint: true,
            retention_revision: "3f".repeat(32),
            promised_floor: "forever".into(),
            privacy_ceiling: "until-checkpoint".into(),
            erase_terminal_at_checkpoint: true,
            max_skew_ms: 0,
            trainer: None,
        }),
        ..OwnerSettings::default()
    };
    let mut resident = DjinnResident::open(
        &identity,
        &data_root,
        &profile,
        owner,
        &vault_dir,
        Unlock::passphrase(PASSPHRASE),
    )
    .await
    .map_err(|error| format!("compose temporary Djinn resident: {error}"))?;

    let observer = resident
        .distillery_chronicle_observer()
        .ok_or("temporary resident did not compose Distillery")?;
    let works = resident
        .take_distillery()
        .ok_or("temporary resident did not retain Distillery works")?;
    let input = works
        .space()
        .put(b"Distillery Chronicle live receipt")
        .await
        .map_err(|error| format!("stage receipt job input: {error}"))?;
    works
        .post_job(
            JobSpec::simple(
                ResourceId::parse("mesh.blake3/v1").expect("mesh.blake3 resource id"),
                "payload",
                input,
                "result",
                32,
                DeterminismClass::Exact,
            ),
            1,
            now_ms(),
        )
        .await
        .map_err(|error| format!("post receipt job: {error}"))?;

    // The works task owns the mutable resident lane. It folds the post through
    // the actual authority and advances the observer retained beside it.
    let stop = Arc::new(Notify::new());
    let works_stop = Arc::clone(&stop);
    let folded = Arc::new(Notify::new());
    let folded_signal = Arc::clone(&folded);
    let works_task = tokio::spawn(async move {
        let mut works = works;
        let result = works
            .run_until(works_stop.notified(), |receipt| {
                if matches!(receipt, ResidentReceipt::Tick { .. }) {
                    println!("[resident] folded a Distillery tick");
                    folded_signal.notify_one();
                }
            })
            .await;
        (works, result)
    });
    // Do not accept a browser until the posted job has crossed one real
    // authority tick. The live receipt then starts at the resident's later
    // one-card revision instead of racing the first fold.
    folded.notified().await;
    // Hold the W2 authority generation stable while the browser reads its
    // shelfmark and mounts the matching scene. The observation came through a
    // real Djinn resident tick; stopping at that boundary makes the receipt
    // reproducible instead of racing the resident's next clock tick.
    stop.notify_waiters();
    let (works, result) = works_task
        .await
        .map_err(|error| format!("resident task: {error}"))?;
    result?;
    let binding_receipt = Arc::new(chronicle_binding_receipt(&observer)?);

    let mut provider_seed = [0u8; 32];
    OsRng.fill_bytes(&mut provider_seed);
    let provider = Arc::new(InMemoryProvider::from_seed(provider_seed));
    let host_key = provider.master_public_key().to_bytes();
    let policy = projection_policy(
        NETWORK,
        vec![TrustedRoot {
            authority: ROOT_AUTHORITY,
            issuer: host_key,
        }],
        vec![ProfileRef {
            id: "mere.base".into(),
            revision: 1,
        }],
        None,
    );
    let host = Arc::new(Mutex::new(ResidentDistillery::chronicle_projection_host(
        observer, policy,
    )?));

    // This reference is derived for this receipt host. It is not a copied
    // product release identity and is never persisted or used by `djinn`.
    let release = ReleaseRefV1 {
        manifest_blake3: *blake3::hash(b"mere.distillery-chronicle-receipt-host/v1").as_bytes(),
        publisher_key_id: *blake3::hash(&host_key).as_bytes(),
    };
    let issue = issue_invite(
        provider.as_ref(),
        &InviteTerms::projection(
            NETWORK,
            ProfileRef {
                id: "mere.base".into(),
                revision: 1,
            },
            now_ms() + args.invite_ttl_ms,
            args.uses,
            release,
        ),
    )?;
    let fragment = issue.descriptor.invite.to_fragment();
    let hosted = Arc::new(Mutex::new(HostedInvite {
        invite: issue.descriptor.invite,
        redemption: issue.redemption,
    }));

    let signal_addr = SocketAddr::new(IpAddr::from([127, 0, 0, 1]), args.signal_port);
    let listener = TcpListener::bind(signal_addr).await?;
    println!("distillery Chronicle receipt host (temporary, receipt-only)");
    println!("  temporary root {}", root.display());
    println!("  live route  distillery.chronicle");
    println!("  job         mesh.blake3/v1, nonce 1");
    println!("  offer       POST http://{signal_addr}/offer");
    println!("  invite      GET  http://{signal_addr}/invite");
    println!("  health      GET  http://{signal_addr}/health");
    println!("  W2 binding GET  http://{signal_addr}/chronicle-binding");
    println!("  carrier     {}:{}", args.bind, args.udp_port);
    println!("  release     receipt-only, generated for this run");
    if args.advertise.is_empty() {
        println!(
            "  advertising discovered addresses; pass --advertise with the browser-reachable LAN address"
        );
    } else {
        println!(
            "  advertising {}",
            args.advertise
                .iter()
                .map(IpAddr::to_string)
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    println!();
    println!("INVITE FRAGMENT (paste into the browser page):");
    println!("{fragment}");
    println!();
    println!("READY — waiting for a browser offer. Ctrl-C to stop.");

    let mut session = 0u64;
    loop {
        tokio::select! {
            accepted = listener.accept() => {
                let (stream, _peer) = accepted?;
                session += 1;
                let id = session;
                let hosted = Arc::clone(&hosted);
                let host = Arc::clone(&host);
                let provider = Arc::clone(&provider);
                let binding_receipt = Arc::clone(&binding_receipt);
                let fragment = fragment.clone();
                let bind = SocketAddr::new(args.bind, args.udp_port);
                let advertise = args.advertise.clone();
                tokio::spawn(async move {
                    if let Err(error) = serve_request(
                        stream, id, hosted, host, provider, binding_receipt, fragment, bind,
                        advertise,
                    )
                    .await
                    {
                        eprintln!("[signal {id}] {error}");
                    }
                });
            },
            _ = tokio::signal::ctrl_c() => break,
        }
    }
    resident.restore_distillery(Some(works));
    resident.shutdown().await?;
    std::fs::remove_dir_all(&root)?;
    Ok(())
}

fn temporary_root() -> Result<PathBuf, std::io::Error> {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "mere-distillery-chronicle-receipt-{}-{stamp}",
        std::process::id()
    ));
    std::fs::create_dir_all(&root)?;
    Ok(root)
}

fn open_vault(
    root: &Path,
) -> Result<(PathBuf, IdentityVault<Box<dyn personae::IdentityStorage>>), Box<dyn std::error::Error>>
{
    let vault_dir = root.join("vault");
    let opened = bootstrap::open_storage(&vault_dir, Unlock::passphrase(PASSPHRASE))?;
    let profile = ProfileId(PROFILE.into());
    let (record, _created) = bootstrap::load_or_create_profile(&*opened.storage, &profile)?;
    Ok((
        vault_dir,
        IdentityVault::with_profile(opened.storage, record),
    ))
}

fn write_device_settings(data_root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    std::fs::create_dir_all(data_root)?;
    pandect::save_device_settings(
        data_root,
        &DeviceSettings {
            mesh_lending: Some(MeshLendingSettings {
                min_idle_ms: 1,
                min_battery_pct: 0,
                max_thermal_c: 0,
                min_network: "offline".into(),
                max_bandwidth_in_use_kbps: 0,
                quiet_hours: None,
                max_concurrent_jobs: 1,
                allowed_resources: vec!["mesh.blake3/v1".into()],
                accepted_checkpoints: vec!["restart".into()],
                reclaim_grace_ms: 0,
                supervises_leases: true,
                stated: StatedConditionSettings {
                    idle_ms: Some(600_000),
                    battery_pct: None,
                    on_mains: None,
                    thermal_c: None,
                    network: Some("wired".into()),
                    bandwidth_in_use_kbps: None,
                },
            }),
            ..DeviceSettings::default()
        },
    )?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn serve_request(
    mut stream: TcpStream,
    id: u64,
    hosted: Arc<Mutex<HostedInvite>>,
    host: Arc<Mutex<ResidentProjectionHost>>,
    provider: Arc<InMemoryProvider>,
    binding_receipt: Arc<String>,
    fragment: String,
    bind: SocketAddr,
    advertise: Vec<IpAddr>,
) -> Result<(), String> {
    let (read, mut write) = stream.split();
    let mut reader = BufReader::new(read);
    let mut request_line = String::new();
    reader
        .read_line(&mut request_line)
        .await
        .map_err(|error| format!("request line: {error}"))?;
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or_default().to_string();
    let path = parts.next().unwrap_or_default().to_string();
    let mut content_length = 0usize;
    loop {
        let mut header = String::new();
        reader
            .read_line(&mut header)
            .await
            .map_err(|error| format!("header: {error}"))?;
        let header = header.trim_end();
        if header.is_empty() {
            break;
        }
        if let Some(value) = header
            .to_ascii_lowercase()
            .strip_prefix("content-length:")
            .map(str::trim)
            .and_then(|value| value.parse::<usize>().ok())
        {
            content_length = value;
        }
    }
    match (method.as_str(), path.as_str()) {
        ("OPTIONS", _) => respond(&mut write, "204 No Content", "").await,
        ("GET", "/health") => respond(&mut write, "200 OK", "ok").await,
        ("GET", "/invite") => respond(&mut write, "200 OK", &fragment).await,
        ("GET", "/chronicle-binding") => {
            respond_typed(
                &mut write,
                "200 OK",
                "application/json; charset=utf-8",
                binding_receipt.as_str(),
            )
            .await
        },
        ("POST", "/offer") => {
            let mut body = vec![0u8; content_length];
            if content_length > 0 {
                reader
                    .read_exact(&mut body)
                    .await
                    .map_err(|error| format!("body: {error}"))?;
            }
            let offer = String::from_utf8_lossy(&body).into_owned();
            if offer.trim().is_empty() {
                return respond(&mut write, "400 Bad Request", "empty offer").await;
            }
            println!("[session {id}] offer received, {} bytes", offer.len());
            let mut answerer = Answerer::bind(AnswererConfig {
                bind,
                advertise,
                carrier: CarrierConfig::default(),
            })
            .await
            .map_err(|error| format!("bind: {error}"))?;
            let answer = match answerer.answer(&offer) {
                Ok(answer) => answer,
                Err(error) => {
                    return respond(&mut write, "400 Bad Request", &error.to_string()).await;
                },
            };
            respond(&mut write, "200 OK", &answer).await?;
            tokio::spawn(async move {
                if let Err(error) = admit_and_serve(id, answerer, hosted, host, provider).await {
                    println!("[session {id}] {error}");
                }
            });
            Ok(())
        },
        _ => {
            respond(
                &mut write,
                "404 Not Found",
                "try POST /offer, GET /invite, GET /health, GET /chronicle-binding",
            )
            .await
        },
    }
}

async fn admit_and_serve(
    id: u64,
    answerer: Answerer,
    hosted: Arc<Mutex<HostedInvite>>,
    host: Arc<Mutex<ResidentProjectionHost>>,
    provider: Arc<InMemoryProvider>,
) -> Result<(), String> {
    let carrier = answerer
        .accept()
        .await
        .map_err(|error| format!("the data channel never opened: {error}"))?;
    let served = {
        let mut hosted = hosted.lock().await;
        admit_webrtc_catalog(
            &host,
            carrier,
            provider.as_ref(),
            &mut hosted,
            ROOT_AUTHORITY,
            DELEGATION_TTL_MS,
            now_ms,
        )
        .await
        .map_err(|error| format!("join or catalog route refused: {error}"))?
    };
    println!(
        "[session {id}] admitted live Distillery Chronicle session {}",
        served.projection().session().0
    );
    let summary = served
        .finish()
        .await
        .map_err(|error| format!("the served projection failed: {error}"))?;
    println!("[session {id}] served {} request(s)", summary.answered);
    Ok(())
}

async fn respond(
    write: &mut (impl AsyncWriteExt + Unpin),
    status: &str,
    body: &str,
) -> Result<(), String> {
    respond_typed(write, status, "text/plain; charset=utf-8", body).await
}

async fn respond_typed(
    write: &mut (impl AsyncWriteExt + Unpin),
    status: &str,
    content_type: &str,
    body: &str,
) -> Result<(), String> {
    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Headers: content-type\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    write
        .write_all(response.as_bytes())
        .await
        .map_err(|error| format!("response: {error}"))?;
    write
        .flush()
        .await
        .map_err(|error| format!("flush: {error}"))
}
