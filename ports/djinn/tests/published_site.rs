// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Public-service receipts for Djinn's profile-owned Gemini publication.
//!
//! These tests intentionally use only the saved Knot snapshot handoff and
//! [`PublishedSiteService`]'s public lifecycle API. They exercise a listener
//! collision, durable stop/reopen state, and two profile roots sharing one
//! physical custody store without reaching into descriptor files or service
//! internals.

use std::net::TcpListener;

use djinn::resident_blobs::ResidentBlobCustody;
use djinn::resident_site::{GeminiListenerPolicyV1, PublishedSiteService, PublishedSiteStateV1};
use djinn::settings::ResidentContentSettings;
use knot_site::{PublishedSnapshotV1, Site, SiteFormat};
use transport::BlobScope;

fn scope(value: u8) -> BlobScope {
    BlobScope::new([value; 32])
}

async fn snapshot(root: &std::path::Path, name: &str, body: &[u8]) -> PublishedSnapshotV1 {
    std::fs::create_dir_all(root).expect("create the profile's saved-site root");
    let site = Site::create_for(&root.join(name), SiteFormat::Gemini)
        .expect("create a saved Gemini site for the handoff");
    let mut snapshot = site
        .publication()
        .expect("read the saved publication")
        .to_snapshot_v1()
        .expect("create the immutable snapshot");
    snapshot
        .pages
        .get_mut("/index.gmi")
        .expect("default Gemini index page")
        .source = body.to_vec();
    snapshot.validate().expect("keep the test snapshot valid");
    snapshot
}

fn resuming_policy() -> GeminiListenerPolicyV1 {
    GeminiListenerPolicyV1 {
        resume_on_restart: true,
        ..Default::default()
    }
}

fn serving_identity(state: PublishedSiteStateV1) -> ([u8; 32], [u8; 32]) {
    let PublishedSiteStateV1::Serving {
        snapshot_digest,
        certificate_sha256,
        ..
    } = state
    else {
        panic!("expected the saved publication to be serving")
    };
    (snapshot_digest, certificate_sha256)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn occupied_replacement_keeps_the_prior_saved_snapshot_and_certificate() {
    let temp = tempfile::tempdir().expect("temporary resident root");
    let custody = ResidentBlobCustody::open(
        &temp.path().join("data"),
        &ResidentContentSettings::default(),
    )
    .await
    .expect("open shared custody");
    let mut service = PublishedSiteService::open(temp.path(), scope(0x31), custody.clone())
        .await
        .expect("open the profile publication service");

    let original = snapshot(temp.path(), "original", b"# Original saved revision\r\n").await;
    let replacement = snapshot(temp.path(), "replacement", b"# Replacement revision\r\n").await;
    assert_ne!(
        original.digest().expect("original digest"),
        replacement.digest().expect("replacement digest"),
        "the failed operation must be a genuine replacement attempt"
    );
    let (original_digest, original_certificate) = serving_identity(
        service
            .publish(original, resuming_policy())
            .await
            .expect("serve original"),
    );

    let blocker = TcpListener::bind(("127.0.0.1", 0)).expect("reserve a loopback port");
    let collision_port = blocker.local_addr().expect("read the reserved port").port();
    let failure = service
        .publish(
            replacement,
            GeminiListenerPolicyV1 {
                port: collision_port,
                ..resuming_policy()
            },
        )
        .await
        .expect_err("occupied replacement port must refuse publication");
    assert!(
        failure.contains("bind") || failure.contains("address"),
        "the failure must describe the listener collision: {failure}"
    );
    assert_eq!(
        serving_identity(service.state()),
        (original_digest, original_certificate),
        "a failed replacement restores the previous in-memory snapshot and identity"
    );
    assert!(
        service.status().resume_on_restart,
        "the prior durable restart policy remains installed"
    );

    drop(blocker);
    service.shutdown();
    let mut reopened = PublishedSiteService::open(temp.path(), scope(0x31), custody.clone())
        .await
        .expect("reopen the same profile");
    assert_eq!(
        serving_identity(reopened.state()),
        (original_digest, original_certificate),
        "the persisted descriptor still names the original snapshot and certificate"
    );
    reopened.remove().await.expect("remove test publication");
    drop(reopened);
    custody.shutdown().await.expect("close shared custody");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn stop_is_persisted_and_same_profile_can_resume_with_its_saved_identity() {
    let temp = tempfile::tempdir().expect("temporary resident root");
    let custody = ResidentBlobCustody::open(
        &temp.path().join("data"),
        &ResidentContentSettings::default(),
    )
    .await
    .expect("open shared custody");
    let publication = snapshot(temp.path(), "site", b"# Saved site\r\n").await;
    let expected_digest = publication.digest().expect("saved snapshot digest");
    let mut service = PublishedSiteService::open(temp.path(), scope(0x32), custody.clone())
        .await
        .expect("open the profile publication service");
    let (_, certificate) = serving_identity(
        service
            .publish(publication.clone(), resuming_policy())
            .await
            .expect("serve saved publication"),
    );
    assert!(matches!(
        service.stop().expect("stop the saved publication"),
        PublishedSiteStateV1::Stopped { snapshot_digest } if snapshot_digest == expected_digest
    ));
    service.shutdown();

    let mut reopened = PublishedSiteService::open(temp.path(), scope(0x32), custody.clone())
        .await
        .expect("reopen the same profile after stop");
    assert!(matches!(
        reopened.state(),
        PublishedSiteStateV1::Stopped { snapshot_digest } if snapshot_digest == expected_digest
    ));
    assert!(
        reopened.status().resume_on_restart,
        "stop changes current state, not the owner's restart policy"
    );

    let (resumed_digest, resumed_certificate) = serving_identity(
        reopened
            .publish(publication, resuming_policy())
            .await
            .expect("start the saved publication again"),
    );
    assert_eq!(resumed_digest, expected_digest);
    assert_eq!(
        resumed_certificate, certificate,
        "the same profile must retain its generated certificate across stop and restart"
    );
    reopened.shutdown();

    let mut restored = PublishedSiteService::open(temp.path(), scope(0x32), custody.clone())
        .await
        .expect("reopen the explicitly resumed profile");
    assert_eq!(
        serving_identity(restored.state()),
        (expected_digest, certificate),
        "the restarted same-profile publication persists across a process reopen"
    );
    restored.remove().await.expect("remove test publication");
    drop(restored);
    custody.shutdown().await.expect("close shared custody");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn profile_roots_and_custody_scopes_keep_published_sites_separate() {
    let temp = tempfile::tempdir().expect("temporary resident root");
    let custody = ResidentBlobCustody::open(
        &temp.path().join("data"),
        &ResidentContentSettings::default(),
    )
    .await
    .expect("open shared custody");
    let personal_root = temp.path().join("profiles").join("personal");
    let work_root = temp.path().join("profiles").join("work");
    let personal_scope = scope(0x41);
    let work_scope = scope(0x42);
    let personal_snapshot = snapshot(&personal_root, "site", b"# Personal revision\r\n").await;
    let work_snapshot = snapshot(&work_root, "site", b"# Work revision\r\n").await;

    let mut personal = PublishedSiteService::open(&personal_root, personal_scope, custody.clone())
        .await
        .expect("open personal profile publication service");
    let mut work = PublishedSiteService::open(&work_root, work_scope, custody.clone())
        .await
        .expect("open work profile publication service");
    let (personal_digest, _) = serving_identity(
        personal
            .publish(personal_snapshot, resuming_policy())
            .await
            .expect("serve personal snapshot"),
    );
    let (work_digest, _) = serving_identity(
        work.publish(work_snapshot, resuming_policy())
            .await
            .expect("serve work snapshot"),
    );
    assert_ne!(personal_digest, work_digest);
    assert!(
        matches!(personal.state(), PublishedSiteStateV1::Serving { snapshot_digest, .. } if snapshot_digest == personal_digest),
        "personal service must retain its own descriptor"
    );
    assert!(
        matches!(work.state(), PublishedSiteStateV1::Serving { snapshot_digest, .. } if snapshot_digest == work_digest),
        "work service must retain its own descriptor"
    );
    assert_eq!(
        custody
            .blobs()
            .leased_hashes(personal_scope)
            .await
            .expect("read personal custody scope")
            .len(),
        1,
        "the personal scope holds only its publication"
    );
    assert_eq!(
        custody
            .blobs()
            .leased_hashes(work_scope)
            .await
            .expect("read work custody scope")
            .len(),
        1,
        "the work scope holds only its publication"
    );

    personal
        .remove()
        .await
        .expect("remove personal publication");
    work.remove().await.expect("remove work publication");
    drop(personal);
    drop(work);
    custody.shutdown().await.expect("close shared custody");
}
