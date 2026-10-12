// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Signed Gemot owners and real Eidetic/Cambium paths, with unsealed in-memory
//! storage and two independent semantic DOM readings. No renderer or network.

use cambium::{DomHandle, GenetAppRunner, PointerClick};
use eidetic::manifest::save_manifest;
use eidetic::{
    BlobManifest, BlobSource, ManifestId, PrivacyClass, ProvenanceRecord, SchemaRef, Timestamp,
    TrustEnvelope,
};
use gemot::moot::constitution::{CapabilityGrant, ConstitutionRules};
use gemot::moot::{
    AvailabilityPolicy, CollectionChange, CollectionId, CollectionRef, ContributionRef,
    ErasurePolicy, KeepBound, Moot, MootAccessLevel, MootId, MootMember, MootMembershipAction,
    MootOutboundOperation, MootRetentionSettings, PolicyRevision,
};
use genet_scripted_dom::{NodeId, ScriptedDom};
use layout_dom_api::{LayoutDom, LocalName, Namespace};
use moot::resource_coop::{ContentRead, HostingRead, Offer, Outcome, Pane, ResourceAction, Target};
use moot::resource_owner::{self as owner, Applied, ForkDestination};
use moot::resource_view::{Reading, View, view};
use muniment::{Backend, MemoryBackend};
use personae::{IdentityProvider, InMemoryProvider};
use proofs::Digest;
use servitor::{Cap, cap_path};
use std::{cell::RefCell, rc::Rc};

const MOOT: [u8; 32] = [71; 32];
const COLLECTION: [u8; 32] = [72; 32];
const BOB_COLLECTION_GRANT: [u8; 32] = [43; 32];

struct Fixture {
    alice: InMemoryProvider,
    bob: InMemoryProvider,
    moot: Moot<MemoryBackend>,
    peer: Moot<MemoryBackend>,
    blobs: MemoryBackend,
    manifest: BlobManifest,
    rules: ConstitutionRules,
    destination: ForkDestination,
}

fn retention() -> MootRetentionSettings {
    MootRetentionSettings {
        revision: PolicyRevision(Digest::blake3(b"resource-owner-receipt")),
        availability: AvailabilityPolicy {
            promised_floor: KeepBound::Forever,
        },
        erasure: ErasurePolicy {
            history_ceiling: KeepBound::UntilCheckpoint,
        },
    }
}

async fn replicate(
    source: &Moot<MemoryBackend>,
    peer: &Moot<MemoryBackend>,
    receipt: &gemot::moot::MootCommandReceipt,
) {
    match source.outbound(receipt).await.unwrap() {
        MootOutboundOperation::Membership(operation) => {
            peer.membership_store().accept(&operation).await.unwrap();
        },
        MootOutboundOperation::Object(operation) => {
            peer.object_store().accept(MOOT, &operation).await.unwrap();
        },
        _ => panic!("unexpected fixture lane"),
    }
}

impl Fixture {
    async fn new() -> Self {
        let alice = InMemoryProvider::from_seed([11; 32]);
        let bob = InMemoryProvider::from_seed([12; 32]);
        let a = alice.master_public_key().to_bytes();
        let b = bob.master_public_key().to_bytes();
        let mut rules = ConstitutionRules::founder_only(a);
        rules.admission = mien::Policy::MembersOnly {
            rate_limit: 100,
            rate_window_ms: 1_000,
        };
        for (id, subject, scope) in [
            ([41; 32], a, "moot"),
            ([42; 32], b, "moot/fauna"),
            (BOB_COLLECTION_GRANT, b, "moot/collection"),
        ] {
            rules.grant(CapabilityGrant {
                id,
                subject,
                path_prefix: cap_path(&Cap::scope(scope).unwrap()),
                not_before_ms: 1,
                expires_at_ms: None,
                delegation_depth: 0,
            });
        }
        let moot = Moot::in_memory(MootId(MOOT), a, retention());
        let peer = Moot::in_memory(MootId(MOOT), a, retention());
        for instance in [&moot, &peer] {
            instance
                .found(
                    alice.master_keypair().to_seed(),
                    None,
                    None,
                    rules.clone(),
                    1,
                )
                .await
                .unwrap();
        }
        let receipt = moot
            .update_membership_for_identity(
                &alice,
                MootMembershipAction::Create {
                    initial_members: vec![
                        MootMember {
                            member: a,
                            access: MootAccessLevel::Manage,
                        },
                        MootMember {
                            member: b,
                            access: MootAccessLevel::Write,
                        },
                    ],
                },
            )
            .await
            .unwrap();
        replicate(&moot, &peer, &receipt).await;
        let blobs = MemoryBackend::new();
        let body = b"A notebook we can discuss together.";
        let id = ManifestId::of_blob(body);
        blobs.put("blob:public-notebook", body).await.unwrap();
        let manifest = BlobManifest {
            id,
            schema: SchemaRef::from_id(ManifestId::of_blob(b"notebook schema")),
            content_hash: id.0,
            byte_size: body.len() as u64,
            created_at: Timestamp::ZERO,
            last_accessed: None,
            sources: vec![BlobSource::Local {
                key: "blob:public-notebook".into(),
            }],
            privacy: PrivacyClass::PublicPortable,
            provenance: ProvenanceRecord::self_imported("fixture notebook"),
            trust: TrustEnvelope::self_asserted(),
            schema_metadata: serde_json::json!({}),
            manifest_version: BlobManifest::CURRENT_VERSION,
        };
        save_manifest(&mut blobs.clone(), &manifest).await.unwrap();
        let mut contributions = Vec::new();
        for identity in [&alice, &bob] {
            let receipt = moot
                .share_authorized_for_identity(
                    identity,
                    *id.0.as_bytes(),
                    "example.notebook.v1".into(),
                    "Shared notebook".into(),
                    10,
                )
                .await
                .unwrap();
            contributions.push(receipt.operation);
            replicate(&moot, &peer, &receipt).await;
        }
        let receipt = moot
            .declare_collection_for_identity(
                &alice,
                CollectionId(COLLECTION),
                "Notebooks".into(),
                None,
                11,
            )
            .await
            .unwrap();
        replicate(&moot, &peer, &receipt).await;
        for contribution in contributions {
            let heads = moot
                .authorized_collection(CollectionId(COLLECTION), 12)
                .await
                .unwrap()
                .unwrap()
                .heads;
            let receipt = moot
                .set_collection_membership_for_identity(
                    &alice,
                    CollectionRef {
                        moot_id: MOOT,
                        collection_id: CollectionId(COLLECTION),
                    },
                    heads,
                    CollectionChange::SetMembership {
                        contribution: ContributionRef {
                            moot_id: MOOT,
                            share: contribution,
                        },
                        included: true,
                    },
                    12,
                )
                .await
                .unwrap();
            replicate(&moot, &peer, &receipt).await;
        }
        Self {
            alice,
            bob,
            moot,
            peer,
            blobs,
            manifest,
            rules,
            destination: ForkDestination {
                id: [73; 32],
                name: "Our notebook collection".into(),
            },
        }
    }
    async fn read(&self, identity: &InMemoryProvider) -> moot::resource_coop::Snapshot {
        owner::snapshot(
            &self.moot,
            identity.master_public_key().to_bytes(),
            COLLECTION,
            Some(&self.destination),
            vec![],
            &mut self.blobs.clone(),
            20,
        )
        .await
        .unwrap()
    }
}

fn find(dom: &ScriptedDom, node: NodeId, id: &str) -> Option<NodeId> {
    if dom.attribute(node, &Namespace::from(""), &LocalName::from("id")) == Some(id) {
        return Some(node);
    }
    dom.dom_children(node)
        .find_map(|child| find(dom, child, id))
}

#[tokio::test]
async fn real_owner_reads_preserve_duplicate_contributions_and_do_not_write_content() {
    let f = Fixture::new().await;
    let count = f.blobs.len();
    let read = f.read(&f.alice).await;
    assert_eq!(read.resources.len(), 2);
    assert_eq!(
        read.resources[0].reference.manifest,
        read.resources[1].reference.manifest
    );
    assert_ne!(
        read.resources[0].reference.contribution,
        read.resources[1].reference.contribution
    );
    assert!(
        read.resources
            .iter()
            .all(|r| r.content == ContentRead::Resolved && r.hosting == HostingRead::Unknown)
    );
    assert_eq!(read.collection.as_ref().unwrap().fork, Some(Offer::Offered));
    assert!(matches!(
        read.resources[0].offers[&ResourceAction::RetainCopy],
        Offer::Unsupported { .. }
    ));
    assert_eq!(f.blobs.len(), count);
}

#[tokio::test]
async fn cambium_clicks_route_two_readings_to_real_open_and_reference_fork_owners() {
    let f = Fixture::new().await;
    let left_dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let right_dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut left = GenetAppRunner::new(
        left_dom.clone(),
        view as fn(&Reading) -> View,
        Reading::new("left", f.read(&f.alice).await).unwrap(),
    );
    let mut right = GenetAppRunner::new(
        right_dom.clone(),
        view as fn(&Reading) -> View,
        Reading::new("right", f.read(&f.bob).await).unwrap(),
    );
    let reference = left.state().pane().snapshot().resources[0].reference;
    let hex: String = reference
        .contribution
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    let node = find(
        &left_dom.borrow(),
        left.root(),
        &format!("moot-left-inspect-{hex}"),
    )
    .unwrap();
    left.dispatch_click(node, PointerClick::at((1., 1.)));
    assert_eq!(left.state().pane().selected_resource(), Some(reference));
    assert_eq!(right.state().pane().selected_resource(), None);
    let node = find(&left_dom.borrow(), left.root(), "moot-left-open").unwrap();
    left.dispatch_click(node, PointerClick::at((1., 1.)));
    let mut request = None;
    left.update(|state| request = state.take_request());
    let request = request.unwrap();
    let applied = owner::dispatch(&f.moot, &f.alice, &mut f.blobs.clone(), &request, None, 21)
        .await
        .unwrap();
    assert!(
        matches!(&applied,Applied::Opened{bytes,..} if bytes==b"A notebook we can discuss together.")
    );
    right.update(|state| assert!(!state.complete(&request, applied.outcome())));
    left.update(|state| assert!(state.complete(&request, applied.outcome())));
    let node = find(
        &right_dom.borrow(),
        right.root(),
        "moot-right-fork-collection",
    )
    .unwrap();
    right.dispatch_click(node, PointerClick::at((1., 1.)));
    let mut request = None;
    right.update(|state| request = state.take_request());
    let request = request.unwrap();
    let count = f.blobs.len();
    let applied = owner::dispatch(
        &f.moot,
        &f.bob,
        &mut f.blobs.clone(),
        &request,
        Some(&f.destination),
        22,
    )
    .await
    .unwrap();
    let Applied::CollectionForked { operation, .. } = applied else {
        panic!("expected collection fork")
    };
    let operation = f
        .moot
        .object_store()
        .operation(&p2panda_core::Hash::from_bytes(operation))
        .await
        .unwrap()
        .unwrap();
    f.peer
        .object_store()
        .accept(MOOT, &operation)
        .await
        .unwrap();
    let child = f
        .peer
        .authorized_collection(CollectionId(f.destination.id), 22)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(child.selected.len(), 2);
    assert_eq!(
        child.fork.unwrap().parent.collection.collection_id.0,
        COLLECTION
    );
    let empty = MemoryBackend::new();
    let peer_read = owner::snapshot(
        &f.peer,
        f.bob.master_public_key().to_bytes(),
        f.destination.id,
        None,
        vec![],
        &mut empty.clone(),
        23,
    )
    .await
    .unwrap();
    assert!(
        peer_read
            .resources
            .iter()
            .all(|r| matches!(r.content, ContentRead::Unavailable { .. })
                && r.hosting == HostingRead::Unknown)
    );
    assert_eq!(empty.len(), 0);
    assert_eq!(f.blobs.len(), count);
}

#[tokio::test]
async fn signed_revocation_rechecks_destination_authority_at_dispatch() {
    let mut f = Fixture::new().await;
    let mut pane = Pane::new(f.read(&f.bob).await).unwrap();
    let request = pane.prepare_collection_fork().unwrap();
    f.rules.capability_grants.remove(&BOB_COLLECTION_GRANT);
    f.moot
        .amend(f.alice.master_keypair().to_seed(), f.rules.clone(), 30)
        .await
        .unwrap();
    let count = f.moot.snapshot().await.unwrap().roster.collections.len();
    let error = owner::dispatch(
        &f.moot,
        &f.bob,
        &mut f.blobs.clone(),
        &request,
        Some(&f.destination),
        31,
    )
    .await
    .unwrap_err();
    assert!(error.to_string().contains("authority refused"));
    assert_eq!(
        f.moot.snapshot().await.unwrap().roster.collections.len(),
        count
    );
    let current = f.read(&f.bob).await;
    assert_eq!(current.resources.len(), 2);
    assert!(matches!(
        current.collection.unwrap().fork,
        Some(Offer::Withheld { .. })
    ));
}

#[tokio::test]
async fn withdrawn_contribution_stops_opening_and_does_not_erase_held_bytes() {
    let f = Fixture::new().await;
    let read = f.read(&f.alice).await;
    let reference = read
        .resources
        .iter()
        .find(|r| f.moot.moot_id().0 == r.reference.moot)
        .unwrap()
        .reference;
    let share = f
        .moot
        .authorized_fauna(20)
        .await
        .unwrap()
        .into_iter()
        .find(|s| s.op_hash == reference.contribution)
        .unwrap();
    let identity = if share.shared_by == f.alice.master_public_key().to_bytes() {
        &f.alice
    } else {
        &f.bob
    };
    let mut pane = Pane::new(read).unwrap();
    pane.select_resource(reference).unwrap();
    let request = pane.prepare_resource(ResourceAction::Open).unwrap();
    let receipt = f
        .moot
        .withdraw_share_for_identity(identity, reference.contribution, 30)
        .await
        .unwrap();
    replicate(&f.moot, &f.peer, &receipt).await;
    assert!(
        owner::dispatch(&f.moot, &f.alice, &mut f.blobs.clone(), &request, None, 31)
            .await
            .is_err()
    );
    let refreshed = f.read(&f.alice).await;
    assert_eq!(refreshed.resources.len(), 1);
    assert!(matches!(
        refreshed.collection.unwrap().fork,
        Some(Offer::Withheld { .. })
    ));
    assert_eq!(
        f.blobs.get("blob:public-notebook").await.unwrap().unwrap(),
        b"A notebook we can discuss together."
    );
}

#[tokio::test]
async fn private_changed_and_future_manifests_are_refused_without_mutating_the_store() {
    let mut f = Fixture::new().await;
    let read = f.read(&f.alice).await;
    let mut pane = Pane::new(read).unwrap();
    pane.select_resource(pane.snapshot().resources[0].reference)
        .unwrap();
    let request = pane.prepare_resource(ResourceAction::Open).unwrap();
    for mode in 0..4 {
        f.manifest.privacy = if mode == 0 {
            PrivacyClass::MootScoped
        } else {
            PrivacyClass::PublicPortable
        };
        f.manifest.manifest_version = if mode == 1 { 2 } else { 1 };
        f.manifest.byte_size = if mode == 2 { 5 * 1024 * 1024 } else { 35 };
        save_manifest(&mut f.blobs.clone(), &f.manifest)
            .await
            .unwrap();
        if mode == 3 {
            f.blobs
                .put("blob:public-notebook", b"corrupt backing bytes")
                .await
                .unwrap();
        }
        let count = f.blobs.len();
        let error = owner::dispatch(&f.moot, &f.alice, &mut f.blobs.clone(), &request, None, 21)
            .await
            .unwrap_err();
        if mode < 3 {
            assert!(matches!(error.outcome(), Outcome::Unsupported { .. }));
        } else {
            assert!(matches!(error.outcome(), Outcome::Refused { .. }));
            assert!(error.to_string().contains("hash mismatch"));
        }
        assert_eq!(f.blobs.len(), count);
    }
}

#[tokio::test]
async fn reviewed_destination_and_current_membership_are_bound_to_the_request() {
    let f = Fixture::new().await;
    let mut pane = Pane::new(f.read(&f.bob).await).unwrap();
    let request = pane.prepare_collection_fork().unwrap();
    let changed = ForkDestination {
        id: [74; 32],
        name: "Another collection".into(),
    };
    assert!(
        owner::dispatch(
            &f.moot,
            &f.bob,
            &mut f.blobs.clone(),
            &request,
            Some(&changed),
            21
        )
        .await
        .unwrap_err()
        .to_string()
        .contains("destination changed")
    );
    f.moot
        .update_membership_for_identity(
            &f.alice,
            MootMembershipAction::Remove {
                member: f.bob.master_public_key().to_bytes(),
            },
        )
        .await
        .unwrap();
    assert!(
        owner::dispatch(
            &f.moot,
            &f.bob,
            &mut f.blobs.clone(),
            &request,
            Some(&f.destination),
            22
        )
        .await
        .is_err()
    );
    assert!(
        owner::snapshot(
            &f.moot,
            f.bob.master_public_key().to_bytes(),
            COLLECTION,
            None,
            vec![],
            &mut f.blobs.clone(),
            22
        )
        .await
        .is_err()
    );
}

#[tokio::test]
async fn a_second_fork_cannot_redeclare_the_same_destination() {
    let f = Fixture::new().await;
    let mut pane = Pane::new(f.read(&f.bob).await).unwrap();
    let request = pane.prepare_collection_fork().unwrap();
    owner::dispatch(
        &f.moot,
        &f.bob,
        &mut f.blobs.clone(),
        &request,
        Some(&f.destination),
        21,
    )
    .await
    .unwrap();
    let count = f.moot.snapshot().await.unwrap().roster.collections.len();
    assert!(
        owner::dispatch(
            &f.moot,
            &f.bob,
            &mut f.blobs.clone(),
            &request,
            Some(&f.destination),
            22
        )
        .await
        .unwrap_err()
        .to_string()
        .contains("already has history")
    );
    assert_eq!(
        f.moot.snapshot().await.unwrap().roster.collections.len(),
        count
    );
}

#[tokio::test]
async fn refresh_clears_queued_controls_and_stale_feedback_without_changing_another_reading() {
    let f = Fixture::new().await;
    let read = f.read(&f.alice).await;
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut runner = GenetAppRunner::new(
        dom.clone(),
        view as fn(&Reading) -> View,
        Reading::new("fresh", read.clone()).unwrap(),
    );
    let node = find(&dom.borrow(), runner.root(), "moot-fresh-fork-collection").unwrap();
    runner.dispatch_click(node, PointerClick::at((1., 1.)));
    let mut request = None;
    runner.update(|state| request = state.take_request());
    let request = request.unwrap();
    let mut other = Reading::new("other", read.clone()).unwrap();
    runner.update(|state| state.refresh(read).unwrap());
    runner.update(|state| {
        assert!(!state.complete(
            &request,
            Outcome::Accepted {
                owner_receipt: "late".into()
            }
        ))
    });
    assert!(!other.complete(
        &request,
        Outcome::Refused {
            reason: "foreign reading".into()
        }
    ));
    assert!(other.take_request().is_none());
    assert!(matches!(request.target, Target::CollectionFork { .. }));
}

#[tokio::test]
async fn unavailable_content_controls_cannot_dispatch_a_copy() {
    let f = Fixture::new().await;
    let read = f.read(&f.alice).await;
    let reference = read.resources[0].reference;
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut runner = GenetAppRunner::new(
        dom.clone(),
        view as fn(&Reading) -> View,
        Reading::new("denied", read).unwrap(),
    );
    let hex: String = reference
        .contribution
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    let node = find(
        &dom.borrow(),
        runner.root(),
        &format!("moot-denied-inspect-{hex}"),
    )
    .unwrap();
    runner.dispatch_click(node, PointerClick::at((1., 1.)));
    let node = find(&dom.borrow(), runner.root(), "moot-denied-retain").unwrap();
    assert_eq!(
        dom.borrow()
            .attribute(node, &Namespace::from(""), &LocalName::from("disabled")),
        Some("")
    );
    runner.dispatch_click(node, PointerClick::at((1., 1.)));
    runner.update(|state| assert!(state.take_request().is_none()));
    assert_eq!(runner.state().pane().selected_resource(), Some(reference));
}

#[tokio::test]
async fn coop_controls_keep_the_exact_application_owner_separate_from_resource_attention() {
    use moot::coop::{LifecycleReport, Verdict};
    use moot::resource_coop::{ActivityAction, ActivityRead, ActivityRef};
    let f = Fixture::new().await;
    let mut read = f.read(&f.alice).await;
    let resource = read.resources[0].reference;
    let activity = ActivityRef {
        moot: MOOT,
        owner: "knot".into(),
        activity: "notebook-review".into(),
    };
    // This is disclosed fixture activity data, not a live application session.
    read.activities.push(ActivityRead {
        reference: activity.clone(),
        title: "Notebook review".into(),
        lifecycle: LifecycleReport::new(
            Verdict::NotJoined,
            Some("An invitation is available".into()),
            20,
        ),
        resources: vec![resource],
        offers: [(ActivityAction::Join, Offer::Offered)].into(),
    });
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut runner = GenetAppRunner::new(
        dom.clone(),
        view as fn(&Reading) -> View,
        Reading::new("coop", read).unwrap(),
    );
    let hex: String = resource
        .contribution
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    let node = find(
        &dom.borrow(),
        runner.root(),
        &format!("moot-coop-inspect-{hex}"),
    )
    .unwrap();
    runner.dispatch_click(node, PointerClick::at((1., 1.)));
    fn label(dom: &ScriptedDom, node: NodeId) -> Option<NodeId> {
        if dom.attribute(node, &Namespace::from(""), &LocalName::from("aria-label"))
            == Some("Inspect Notebook review")
        {
            return Some(node);
        }
        dom.dom_children(node).find_map(|child| label(dom, child))
    }
    let node = label(&dom.borrow(), runner.root()).unwrap();
    runner.dispatch_click(node, PointerClick::at((1., 1.)));
    assert_eq!(runner.state().pane().selected_resource(), Some(resource));
    assert_eq!(runner.state().pane().selected_activity(), Some(&activity));
    let node = find(&dom.borrow(), runner.root(), "moot-coop-join").unwrap();
    runner.dispatch_click(node, PointerClick::at((1., 1.)));
    let mut request = None;
    runner.update(|state| request = state.take_request());
    let request = request.unwrap();
    assert_eq!(
        request.target,
        Target::Activity {
            reference: activity,
            action: ActivityAction::Join
        }
    );
    assert!(
        owner::dispatch(&f.moot, &f.alice, &mut f.blobs.clone(), &request, None, 21)
            .await
            .unwrap_err()
            .to_string()
            .contains("exact application owner")
    );
    assert_eq!(
        runner.state().pane().snapshot().activities[0]
            .lifecycle
            .verdict,
        Verdict::NotJoined
    );
}

#[tokio::test]
async fn a_current_admission_floor_is_not_bypassed_by_a_still_installed_grant() {
    let mut f = Fixture::new().await;
    let read = f.read(&f.alice).await;
    let selected = read.resources[0].reference;
    let mut pane = Pane::new(read).unwrap();
    pane.select_resource(selected).unwrap();
    let request = pane.prepare_resource(ResourceAction::Open).unwrap();
    f.rules.admission = mien::Policy::OpenWithFloor(mien::GateConfig::default());
    f.moot
        .amend(f.alice.master_keypair().to_seed(), f.rules, 30)
        .await
        .unwrap();
    let count = f.blobs.len();
    assert!(
        owner::dispatch(&f.moot, &f.alice, &mut f.blobs.clone(), &request, None, 31)
            .await
            .unwrap_err()
            .to_string()
            .contains("no longer authorized")
    );
    assert_eq!(f.blobs.len(), count);
}
