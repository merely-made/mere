// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The resident Distillery board through Djinn's admitted Chronicle route.

mod common;

use std::sync::{Arc, mpsc};
use std::time::Duration;

use chirograph::{
    CapabilityProfile, Carrier, CarrierRequestBody, CarrierResponseBody, PresentationCapability,
    ProtocolVersion, ResumeReply, ResumeRequest, SessionOpen,
};
use common::{lane, lending, now_ms, open_vault, owner, profile, unlock, write_device_settings};
use distillery::ResidentReceipt;
use djinn::resident::DjinnResident;
use djinn::resident_distillery::ResidentDistillery;
use graphshell::admission::open_session;
use graphshell::carrier::projection_policy;
use graphshell::network_carrier::{
    CarrierRuntime, NetworkCarrier, dial_projection_session, projection_binding,
};
use mesh::ResourceId;
use mesh::spec::{DeterminismClass, JobSpec};
use notochord::{LocalNetworkPolicy, NetworkId, ProfileRef, TrafficClass, TrustedRoot};
use personae::delegation::{
    CapabilityScope, DelegationCertificate, DelegationParent, SignedDelegationCertificate,
};
use personae::{IdentityProvider, InMemoryProvider};
use tokio::sync::Notify;
use transport::PeerID;
use transport::memory::MemoryTransport;

const NETWORK: NetworkId = NetworkId([3; 32]);
const ROOT_AUTHORITY: [u8; 32] = [7; 32];
const NOW_MS: u64 = 50;

fn route_owner() -> InMemoryProvider {
    InMemoryProvider::from_seed([1; 32])
}

fn viewer() -> InMemoryProvider {
    InMemoryProvider::from_seed([4; 32])
}

fn profile_ref() -> ProfileRef {
    ProfileRef {
        id: "mere.base".into(),
        revision: 1,
    }
}

fn grant(subject: [u8; 32]) -> SignedDelegationCertificate {
    SignedDelegationCertificate::issue(
        &route_owner(),
        DelegationCertificate::new(
            DelegationParent::Root(ROOT_AUTHORITY),
            route_owner().master_public_key().to_bytes(),
            subject,
            CapabilityScope {
                domain: graphshell::admission::GRAPHSHELL_DOMAIN.into(),
                resource: NETWORK.0.to_vec(),
                path_prefix: graphshell::admission::PROJECTION_SERVICE.into(),
                actions: [graphshell::admission::CONNECT_ACTION.to_string()]
                    .into_iter()
                    .collect(),
            },
            5,
            10,
            Some(NOW_MS + 3_600_000),
            1,
            [1; 32],
        ),
    )
    .expect("issue route admission grant")
}

fn policy() -> LocalNetworkPolicy {
    projection_policy(
        NETWORK,
        vec![TrustedRoot {
            authority: ROOT_AUTHORITY,
            issuer: route_owner().master_public_key().to_bytes(),
        }],
        vec![profile_ref()],
        None,
    )
}

fn open_body() -> CarrierRequestBody {
    CarrierRequestBody::Open(Box::new(SessionOpen {
        version: ProtocolVersion::V1,
        capabilities: CapabilityProfile::new([PresentationCapability::PortableCard]),
    }))
}

/// A real resident fold reaches the registered admitted route and replaces the
/// empty opening snapshot with the one-job Chronicle snapshot at a later
/// revision. The update is performed by `ResidentDistillery::run_until`, not
/// by the test or an endpoint-local observer call.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn resident_board_fold_advances_the_admitted_chronicle_route() {
    let directory = tempfile::tempdir().expect("temporary resident root");
    let data_root = directory.path().join("data");
    let (vault_dir, identity) = open_vault(directory.path());
    write_device_settings(&data_root, Some(lending()));

    let mut resident = DjinnResident::open(
        &identity,
        &data_root,
        &profile(),
        owner(Some(lane())),
        &vault_dir,
        unlock(),
    )
    .await
    .expect("compose Djinn with its resident Distillery lane");
    let observer = resident
        .distillery_chronicle_observer()
        .expect("the composed lane retains Chronicle source state");
    let mut host = ResidentDistillery::chronicle_projection_host(observer, policy())
        .expect("build the production Chronicle route host");
    let mut works = resident
        .take_distillery()
        .expect("the composed resident Distillery lane");

    // The resident observer opens before this post, so its first admitted
    // snapshot is genuinely empty. The only way it receives this job is the
    // production run loop's post-tick folded-board callback.
    let input = works
        .space()
        .put(b"Djinn Chronicle route receipt")
        .await
        .expect("stage job input");
    works
        .post_job(
            JobSpec::simple(
                ResourceId::parse("mesh.blake3/v1").expect("resource id"),
                "payload",
                input,
                "result",
                32,
                DeterminismClass::Exact,
            ),
            41,
            now_ms(),
        )
        .await
        .expect("post a real resident job");

    let viewer = viewer();
    let subject = viewer.master_public_key().to_bytes();
    let client_peer = PeerID::from_bytes(&subject).expect("client peer");
    let server_peer =
        PeerID::from_bytes(&route_owner().master_public_key().to_bytes()).expect("server peer");
    let (server, client) = MemoryTransport::pair(server_peer, client_peer);
    let (mounted_tx, mounted_rx) = mpsc::channel();
    let handle = tokio::runtime::Handle::current();
    let client_task = tokio::task::spawn_blocking(move || {
        let hello = open_session(
            &viewer,
            NETWORK,
            profile_ref(),
            TrafficClass::Interactive,
            [5; 32],
            &projection_binding(client_peer),
            vec![grant(subject)],
        )
        .expect("issue admission hello");
        let stream = handle
            .block_on(dial_projection_session(
                &client,
                server_peer,
                &hello,
                &policy().limits,
            ))
            .expect("dial resident host")
            .expect("viewer admitted");
        let mut carrier = NetworkCarrier::over(stream, CarrierRuntime::borrowed(handle));

        let opened = match carrier.request(open_body()).expect("open Chronicle") {
            CarrierResponseBody::Opened(opened) => opened,
            other => panic!("expected Chronicle endpoint, got {other:?}"),
        };
        assert_eq!(opened.descriptor.label, "Distillery Chronicle");
        let request = opened.descriptor.projections[0].request.clone();
        let session = request.session.clone();
        let initial = match carrier
            .request(CarrierRequestBody::Snapshot(request))
            .expect("read opening Chronicle snapshot")
        {
            CarrierResponseBody::Snapshot(snapshot) => *snapshot,
            other => panic!("expected Chronicle snapshot, got {other:?}"),
        };
        assert_eq!(initial.scene.revision.0, 1);
        assert!(
            initial.presentation.bindings.is_empty(),
            "the job was posted after the resident's opening Chronicle fold"
        );
        mounted_tx
            .send(session.clone())
            .expect("initial snapshot mounted");

        let notice = carrier.wait_for_notice().expect("resident revision notice");
        assert_eq!(notice.session, session);
        assert!(notice.revision.0 > initial.scene.revision.0);
        let resumed = match carrier
            .request(CarrierRequestBody::Resume(ResumeRequest {
                session: session.clone(),
                epoch: initial.scene.epoch,
                revision: initial.scene.revision,
            }))
            .expect("resume after resident board fold")
        {
            CarrierResponseBody::Resume(reply) => reply,
            other => panic!("expected Chronicle resume, got {other:?}"),
        };
        let snapshot = match resumed {
            ResumeReply::Snapshot(snapshot) => snapshot,
            other => panic!("a newly folded job changes Chronicle topology, got {other:?}"),
        };
        assert!(snapshot.scene.revision.0 > initial.scene.revision.0);
        assert_eq!(snapshot.presentation.bindings.len(), 1);
        carrier.request(CarrierRequestBody::Close).expect("close");
        carrier.shutdown().expect("shutdown");
        session
    });

    let served = host
        .accept_one(&server, || NOW_MS)
        .await
        .expect("admission host accepts")
        .expect("viewer is admitted");
    let mounted = mounted_rx
        .recv()
        .expect("client mounted its opening snapshot");
    assert_eq!(served.session(), &mounted);

    let stop = Arc::new(Notify::new());
    let signal = Arc::clone(&stop);
    let (tick_tx, tick_rx) = mpsc::channel();
    let run = tokio::spawn(async move {
        let result = works
            .run_until(async move { signal.notified().await }, |receipt| {
                if matches!(receipt, ResidentReceipt::Tick { .. }) {
                    let _ = tick_tx.send(());
                }
            })
            .await;
        (works, result)
    });
    tick_rx
        .recv_timeout(Duration::from_secs(20))
        .expect("the resident ran a supervisor tick that folded the posted job");

    let client_session = client_task.await.expect("client task");
    assert_eq!(client_session, mounted);
    let summary = served.finished().await.expect("join").expect("served");
    assert_eq!(summary.answered, 4, "open, empty snapshot, resume, close");

    stop.notify_one();
    let (works, result) = run.await.expect("resident run task");
    result.expect("resident run stopped cleanly");
    resident.restore_distillery(Some(works));
    resident
        .shutdown()
        .await
        .expect("ordered resident shutdown");
}

/// A later authority fold that keeps the same job card must be resumable by
/// the one contiguous diff retained by Chronicle. This drives both folds
/// through the resident loop; the test never reaches through to
/// `ChronicleObserver::observe`.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn resident_board_state_transition_resumes_by_contiguous_diff() {
    let directory = tempfile::tempdir().expect("temporary resident root");
    let data_root = directory.path().join("data");
    let (vault_dir, identity) = open_vault(directory.path());
    write_device_settings(&data_root, Some(lending()));

    let mut resident_lane = lane();
    // Each run loop gets its immediate first tick, then the client has a full
    // interval to acknowledge the retained diff before the next authority
    // fold can advance Chronicle again.
    resident_lane.tick_every_ms = 60_000;

    let mut resident = DjinnResident::open(
        &identity,
        &data_root,
        &profile(),
        owner(Some(resident_lane)),
        &vault_dir,
        unlock(),
    )
    .await
    .expect("compose Djinn with its resident Distillery lane");
    let observer = resident
        .distillery_chronicle_observer()
        .expect("the composed lane retains Chronicle source state");
    let mut host = ResidentDistillery::chronicle_projection_host(observer, policy())
        .expect("build the production Chronicle route host");
    let mut works = resident
        .take_distillery()
        .expect("the composed resident Distillery lane");

    let input = works
        .space()
        .put(b"Djinn Chronicle stable topology receipt")
        .await
        .expect("stage job input");
    works
        .post_job(
            JobSpec::simple(
                ResourceId::parse("mesh.blake3/v1").expect("resource id"),
                "payload",
                input,
                "result",
                32,
                DeterminismClass::Exact,
            ),
            42,
            now_ms(),
        )
        .await
        .expect("post a real resident job");

    // The opening observer predates the post. Drive one real authority tick,
    // then let the resident stop cleanly. The stop receipt also crosses the
    // same production callback, so the opening snapshot below is the current
    // folded board rather than a test-authored observer state.
    let first_stop = Arc::new(Notify::new());
    let first_signal = Arc::clone(&first_stop);
    let (first_tick_tx, first_tick_rx) = mpsc::channel();
    let first_run = tokio::spawn(async move {
        let result = works
            .run_until(async move { first_signal.notified().await }, |receipt| {
                if matches!(receipt, ResidentReceipt::Tick { .. }) {
                    let _ = first_tick_tx.send(());
                }
            })
            .await;
        (works, result)
    });
    first_tick_rx
        .recv_timeout(Duration::from_secs(20))
        .expect("the resident folded the posted job through its authority");
    first_stop.notify_one();
    let (mut works, first_result) = first_run.await.expect("join first resident run");
    first_result.expect("first resident run stopped cleanly");

    let viewer = viewer();
    let subject = viewer.master_public_key().to_bytes();
    let client_peer = PeerID::from_bytes(&subject).expect("client peer");
    let server_peer =
        PeerID::from_bytes(&route_owner().master_public_key().to_bytes()).expect("server peer");
    let (server, client) = MemoryTransport::pair(server_peer, client_peer);
    let (mounted_tx, mounted_rx) = mpsc::channel();
    let handle = tokio::runtime::Handle::current();
    let client_task = tokio::task::spawn_blocking(move || {
        let hello = open_session(
            &viewer,
            NETWORK,
            profile_ref(),
            TrafficClass::Interactive,
            [6; 32],
            &projection_binding(client_peer),
            vec![grant(subject)],
        )
        .expect("issue admission hello");
        let stream = handle
            .block_on(dial_projection_session(
                &client,
                server_peer,
                &hello,
                &policy().limits,
            ))
            .expect("dial resident host")
            .expect("viewer admitted");
        let mut carrier = NetworkCarrier::over(stream, CarrierRuntime::borrowed(handle));

        let opened = match carrier.request(open_body()).expect("open Chronicle") {
            CarrierResponseBody::Opened(opened) => opened,
            other => panic!("expected Chronicle endpoint, got {other:?}"),
        };
        let request = opened.descriptor.projections[0].request.clone();
        let session = request.session.clone();
        let initial = match carrier
            .request(CarrierRequestBody::Snapshot(request))
            .expect("read current Chronicle snapshot")
        {
            CarrierResponseBody::Snapshot(snapshot) => *snapshot,
            other => panic!("expected Chronicle snapshot, got {other:?}"),
        };
        assert_eq!(initial.presentation.bindings.len(), 1);
        assert_eq!(initial.scene.tables.items.len(), 1);
        let initial_revision = initial.scene.revision.0;
        mounted_tx
            .send(session.clone())
            .expect("initial snapshot mounted");

        let notice = loop {
            let notice = carrier
                .wait_for_notice()
                .expect("later resident revision notice");
            assert_eq!(notice.session, session);
            if notice.revision.0 > initial_revision {
                break notice;
            }
        };
        assert_eq!(notice.revision.0, initial_revision + 1);
        let resumed = match carrier
            .request(CarrierRequestBody::Resume(ResumeRequest {
                session: session.clone(),
                epoch: initial.scene.epoch,
                revision: initial.scene.revision,
            }))
            .expect("resume after the later resident fold")
        {
            CarrierResponseBody::Resume(reply) => reply,
            other => panic!("expected Chronicle resume, got {other:?}"),
        };
        let diffs = match resumed {
            ResumeReply::Diffs(diffs) => diffs,
            other => {
                panic!("stable Chronicle topology must resume by contiguous diff, got {other:?}")
            },
        };
        assert_eq!(diffs.len(), 1);
        let diff = &diffs[0];
        assert_eq!(diff.scene.base, initial.scene.revision);
        assert_eq!(diff.scene.revision.0, initial_revision + 1);
        assert_eq!(diff.scene.operations.len(), 1);
        assert!(matches!(
            &diff.scene.operations[0],
            scenotime::SceneOp::SetGeneration { .. }
        ));
        assert!(
            !diff.presentation.is_empty(),
            "the same card's live state or observation tick changed"
        );

        carrier.request(CarrierRequestBody::Close).expect("close");
        carrier.shutdown().expect("shutdown");
        session
    });

    let served = host
        .accept_one(&server, || NOW_MS)
        .await
        .expect("admission host accepts")
        .expect("viewer is admitted");
    let mounted = mounted_rx
        .recv()
        .expect("client mounted the current Chronicle snapshot");
    assert_eq!(served.session(), &mounted);

    // Start the next authority tick only after the client has acknowledged its
    // opening revision. That makes the retained history's base exact and
    // prevents an unrelated extra tick from forcing a snapshot fallback.
    let second_stop = Arc::new(Notify::new());
    let second_signal = Arc::clone(&second_stop);
    let run = tokio::spawn(async move {
        let result = works
            .run_until(async move { second_signal.notified().await }, |_| {})
            .await;
        (works, result)
    });

    let client_session = client_task.await.expect("client task");
    assert_eq!(client_session, mounted);
    let summary = served.finished().await.expect("join").expect("served");
    assert_eq!(summary.answered, 4, "open, snapshot, resume, close");

    second_stop.notify_one();
    let (works, second_result) = run.await.expect("join second resident run");
    second_result.expect("second resident run stopped cleanly");
    resident.restore_distillery(Some(works));
    resident
        .shutdown()
        .await
        .expect("ordered resident shutdown");
}
