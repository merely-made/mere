// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0
use super::*;
use muniment::MemoryBackend;
#[path = "../../examples/moot_resource_handlers/support.rs"]
mod support;
async fn service() -> Moot<MemoryBackend> {
    let service = Moot::in_memory(
        support::ID,
        support::founder().master_public_key().to_bytes(),
        support::retention_settings(),
    );
    support::initialize(&service).await.unwrap();
    service
}
async fn source() -> MemoryBackend {
    let mut source = MemoryBackend::default();
    eidetic::manifest::save_manifest(&mut source, &support::manifest())
        .await
        .unwrap();
    source
}
fn request(snapshot: Snapshot, action: ResourceAction) -> Request {
    let r = snapshot.resources[0].reference;
    let mut pane = Pane::new(snapshot).unwrap();
    pane.select_resource(r).unwrap();
    pane.prepare_resource(action).unwrap()
}
#[tokio::test]
async fn actual_handlers_retain_sign_refresh_and_serve_after_source_disappears() {
    let service = service().await;
    let id = support::reader();
    let mut encounter = source().await;
    let mut custody = MemoryBackend::default();
    let config = support::config(&service).await.unwrap();
    let mut owner = Owner {
        moot: &service,
        identity: &id,
        encounter: &mut encounter,
        custody: &mut custody,
        sealer: None,
        config,
        selected_collection: None,
    };
    let initial = owner.read(10).await.unwrap();
    assert_eq!(initial.resources.len(), 2);
    let reference = initial.resources[0].reference;
    let copy = request(initial.clone(), ResourceAction::RetainCopy);
    assert!(matches!(
        owner.dispatch(&copy, 10).await.0,
        Outcome::Accepted { .. }
    ));
    // The old read is unchanged; only a fresh actual store read reports custody.
    assert_eq!(initial.resources[0].retention, RetentionRead::Unknown);
    let after = owner.read(11).await.unwrap();
    assert!(matches!(
        after.resources[0].retention,
        RetentionRead::Held { .. }
    ));
    let host = request(after, ResourceAction::OfferHosting);
    assert!(matches!(
        owner.dispatch(&host, 11).await.0,
        Outcome::Accepted { .. }
    ));
    assert!(matches!(
        owner.read(12).await.unwrap().resources[0].hosting,
        HostingRead::Promise { .. }
    ));
    drop(owner);
    encounter = MemoryBackend::default();
    let mut owner = Owner {
        moot: &service,
        identity: &id,
        encounter: &mut encounter,
        custody: &mut custody,
        sealer: None,
        config: support::config(&service).await.unwrap(),
        selected_collection: None,
    };
    assert_eq!(
        owner.serve(reference, "public", 20).await.unwrap(),
        support::BODY
    );
    assert!(owner.serve(reference, "friends", 20).await.is_err());
    assert!(owner.serve(reference, "public", 600_011).await.is_err());
}
#[tokio::test]
async fn settings_and_real_grant_revocation_refuse_pending_writes() {
    let service = service().await;
    let id = support::reader();
    let mut encounter = source().await;
    let mut custody = MemoryBackend::default();
    let mut owner = Owner {
        moot: &service,
        identity: &id,
        encounter: &mut encounter,
        custody: &mut custody,
        sealer: None,
        config: support::config(&service).await.unwrap(),
        selected_collection: None,
    };
    let pending = request(owner.read(10).await.unwrap(), ResourceAction::RetainCopy);
    owner.config.retention_limits.max_stored_bytes = 0;
    assert!(matches!(
        owner.dispatch(&pending, 11).await.0,
        Outcome::Refused { .. }
    ));
    assert!(owner.custody.list("").await.unwrap().is_empty());
    owner.config = support::config(&service).await.unwrap();
    let pending = request(owner.read(12).await.unwrap(), ResourceAction::OfferHosting);
    let mut rules = service.snapshot().await.unwrap().governance.rules;
    rules.revoke_grant(&[4; 32]);
    service.amend([1; 32], rules, 13).await.unwrap();
    assert!(matches!(
        owner.dispatch(&pending, 14).await.0,
        Outcome::Refused { .. }
    ));
    assert!(owner.custody.list("").await.unwrap().is_empty());
    assert_eq!(service.standing_store().len().await.unwrap(), 0);
}
#[tokio::test]
async fn foreign_reader_quota_and_private_export_refuse_without_writes() {
    let service = service().await;
    let id = support::reader();
    let mut encounter = source().await;
    let mut custody = MemoryBackend::default();
    let mut owner = Owner {
        moot: &service,
        identity: &id,
        encounter: &mut encounter,
        custody: &mut custody,
        sealer: None,
        config: support::config(&service).await.unwrap(),
        selected_collection: None,
    };
    let initial = owner.read(10).await.unwrap();
    owner.config.public_export_manifests.clear();
    assert!(matches!(
        owner.read(10).await.unwrap().resources[0].offers[&ResourceAction::OfferHosting],
        Offer::Withheld { .. }
    ));
    owner.config = support::config(&service).await.unwrap();
    let mut foreign = request(initial.clone(), ResourceAction::RetainCopy);
    foreign.reader = [99; 32];
    assert!(matches!(
        owner.dispatch(&foreign, 10).await.0,
        Outcome::Refused { .. }
    ));
    owner.config.retention_limits.max_stored_bytes = 1;
    let overquota = request(owner.read(11).await.unwrap(), ResourceAction::RetainCopy);
    assert!(matches!(
        owner.dispatch(&overquota, 11).await.0,
        Outcome::Refused { .. }
    ));
    owner.config = support::config(&service).await.unwrap();
    let mut private = support::manifest();
    private.privacy = eidetic::PrivacyClass::LocalOnly;
    eidetic::manifest::save_manifest(owner.encounter, &private)
        .await
        .unwrap();
    let current = owner.read(12).await.unwrap();
    assert!(matches!(
        current.resources[0].offers[&ResourceAction::OfferHosting],
        Offer::Withheld { .. }
    ));
    let copy = request(current, ResourceAction::RetainCopy);
    assert!(matches!(
        owner.dispatch(&copy, 12).await.0,
        Outcome::Refused { .. }
    ));
    assert!(owner.custody.list("").await.unwrap().is_empty());
    assert_eq!(service.standing_store().len().await.unwrap(), 0);
}
