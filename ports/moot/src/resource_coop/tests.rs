// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::coop::{Access, Membership, Verdict};

const MOOT: [u8; 32] = [1; 32];
const READER: [u8; 32] = [2; 32];

fn resource(contribution: u8, manifest: u8) -> ResourceRead {
    ResourceRead {
        shared_by: None,
        reference: ResourceRef {
            moot: MOOT,
            contribution: [contribution; 32],
            manifest: [manifest; 32],
        },
        title: format!("Notebook contribution {contribution}"),
        schema: "fixture/notebook/v1".into(),
        content: ContentRead::AddressOnly,
        retention: RetentionRead::Unknown,
        terms: BTreeMap::new(),
        hosting: HostingRead::Unknown,
        offers: [(ResourceAction::Open, Offer::Offered)].into(),
    }
}

fn snapshot() -> Snapshot {
    let first = resource(3, 9);
    let second = resource(4, 9);
    let mut lifecycle = LifecycleReport::new(Verdict::Joined, None, 100);
    lifecycle.membership = Some(Membership {
        members: 7,
        access: Access::Manage,
    });
    Snapshot {
        moot: MOOT,
        reader: READER,
        collection: Some(CollectionRead {
            id: [5; 32],
            name: "Shared notebooks".into(),
            heads: vec![[6; 32], [7; 32]],
            membership_commitment: [8; 32],
            fork: Some(Offer::Offered),
            fork_destination: Some(ForkDestination {
                id: [19; 32],
                name: "Our collection copy".into(),
            }),
        }),
        activities: vec![ActivityRead {
            reference: ActivityRef {
                moot: MOOT,
                owner: "knot".into(),
                activity: "session/notebook-review".into(),
            },
            title: "Review together".into(),
            lifecycle,
            resources: vec![first.reference],
            offers: [(ActivityAction::Join, Offer::Offered)].into(),
        }],
        resources: vec![first, second],
    }
}

#[test]
fn duplicate_payloads_keep_contributions_and_new_revisions_do_not_retarget_attention() {
    let read = snapshot();
    let original = read.resources[0].reference;
    assert_eq!(
        read.resources[0].reference.manifest,
        read.resources[1].reference.manifest
    );
    let mut pane = Pane::new(read.clone()).unwrap();
    pane.select_resource(original).unwrap();
    let mut updated = read;
    updated.resources.push(resource(10, 11));
    pane.refresh(updated).unwrap();
    assert_eq!(pane.snapshot().resources.len(), 3);
    assert_eq!(pane.selected_resource(), Some(original));
    let request = pane.prepare_resource(ResourceAction::Open).unwrap();
    assert_eq!(
        request.target,
        Target::Resource {
            reference: original,
            action: ResourceAction::Open,
            observed_terms: None,
        }
    );
}

#[test]
fn a_collection_fork_offers_no_copy_or_content_fork_right() {
    let read = snapshot();
    let expected_collection = read.collection.clone().unwrap();
    let selected = read.resources[0].reference;
    let mut pane = Pane::new(read).unwrap();
    pane.select_resource(selected).unwrap();
    let request = pane.prepare_collection_fork().unwrap();
    assert_eq!(
        request.target,
        Target::CollectionFork {
            moot: MOOT,
            observed: expected_collection
        }
    );
    assert_eq!(
        pane.prepare_resource(ResourceAction::RetainCopy)
            .unwrap_err(),
        Refusal::UnknownOffer
    );
    assert_eq!(
        pane.prepare_resource(ResourceAction::ForkContent)
            .unwrap_err(),
        Refusal::UnknownOffer
    );
    assert_eq!(
        pane.snapshot().resources[0].content,
        ContentRead::AddressOnly
    );
}

#[test]
fn independent_readings_keep_resource_and_activity_attention_and_completions_separate() {
    let read = snapshot();
    let activity = read.activities[0].reference.clone();
    let first = read.resources[0].reference;
    let second = read.resources[1].reference;
    let mut left = Pane::new(read.clone()).unwrap();
    let mut right = Pane::new(read).unwrap();
    left.select_resource(first).unwrap();
    left.select_activity(&activity).unwrap();
    right.select_resource(second).unwrap();
    let request = left.prepare_activity(ActivityAction::Join).unwrap();
    assert!(!right.complete(
        &request,
        Outcome::Accepted {
            owner_receipt: "wrong pane".into()
        }
    ));
    assert_eq!(right.selected_resource(), Some(second));
    assert_eq!(right.selected_activity(), None);
    assert_eq!(right.outcome(), None);
    assert_eq!(left.selected_resource(), Some(first));
}

#[test]
fn refreshed_authority_invalidates_the_old_request_and_preserves_exact_resource_attention() {
    let mut read = snapshot();
    let selected = read.resources[0].reference;
    let mut pane = Pane::new(read.clone()).unwrap();
    pane.select_resource(selected).unwrap();
    let request = pane.prepare_resource(ResourceAction::Open).unwrap();
    let denied = Offer::Withheld {
        reason: "current owner withdrew access".into(),
    };
    read.resources[0]
        .offers
        .insert(ResourceAction::Open, denied.clone());
    pane.refresh(read).unwrap();
    assert!(!pane.complete(
        &request,
        Outcome::Accepted {
            owner_receipt: "old read".into()
        }
    ));
    assert_eq!(
        pane.prepare_resource(ResourceAction::Open).unwrap_err(),
        Refusal::OwnerOffer(denied)
    );
    assert_eq!(pane.selected_resource(), Some(selected));
    assert_eq!(pane.outcome(), None);
}

#[test]
fn owner_acceptance_changes_feedback_only_and_cannot_invent_retained_bytes_or_hosting() {
    let mut read = snapshot();
    read.resources[0]
        .offers
        .insert(ResourceAction::RetainCopy, Offer::Offered);
    let selected = read.resources[0].reference;
    let mut pane = Pane::new(read.clone()).unwrap();
    pane.select_resource(selected).unwrap();
    let old = pane.prepare_resource(ResourceAction::RetainCopy).unwrap();
    let latest = pane.prepare_resource(ResourceAction::RetainCopy).unwrap();
    assert!(!pane.complete(
        &old,
        Outcome::Accepted {
            owner_receipt: "old preparation".into()
        }
    ));
    assert!(pane.complete(
        &latest,
        Outcome::Accepted {
            owner_receipt: "owner/local-operation".into()
        }
    ));
    assert_eq!(pane.snapshot(), &read);
    assert_eq!(pane.snapshot().resources[0].hosting, HostingRead::Unknown);
    assert_eq!(
        pane.snapshot().resources[0].content,
        ContentRead::AddressOnly
    );
}

#[test]
fn withdrawal_clears_only_missing_attention_and_does_not_fabricate_content_for_activity_links() {
    let read = snapshot();
    let selected = read.resources[0].reference;
    let activity = read.activities[0].reference.clone();
    let mut pane = Pane::new(read.clone()).unwrap();
    pane.select_resource(selected).unwrap();
    pane.select_activity(&activity).unwrap();
    let request = pane.prepare_resource(ResourceAction::Open).unwrap();
    let mut withdrawn = read;
    withdrawn.resources.remove(0);
    pane.refresh(withdrawn).unwrap();
    assert_eq!(pane.selected_resource(), None);
    assert_eq!(pane.selected_activity(), Some(&activity));
    assert_eq!(pane.snapshot().activities[0].resources, vec![selected]);
    assert!(!pane.complete(
        &request,
        Outcome::Accepted {
            owner_receipt: "late".into()
        }
    ));
}

#[test]
fn membership_and_a_hosting_promise_never_supply_action_offers_or_live_presence() {
    let mut read = snapshot();
    read.resources[0].hosting = HostingRead::Promise {
        owner_record: "standing/promise".into(),
    };
    read.activities[0].offers.clear();
    let selected = read.resources[0].reference;
    let activity = read.activities[0].reference.clone();
    let mut pane = Pane::new(read).unwrap();
    pane.select_resource(selected).unwrap();
    pane.select_activity(&activity).unwrap();
    assert_eq!(
        pane.prepare_resource(ResourceAction::OfferHosting)
            .unwrap_err(),
        Refusal::UnknownOffer
    );
    assert_eq!(
        pane.prepare_activity(ActivityAction::Join).unwrap_err(),
        Refusal::UnknownOffer
    );
    assert_eq!(
        pane.snapshot().resources[0].content,
        ContentRead::AddressOnly
    );
    assert_eq!(
        pane.snapshot().activities[0]
            .lifecycle
            .membership
            .unwrap()
            .members,
        7
    );
}

#[test]
fn bad_reads_are_refused_without_replacing_the_current_snapshot() {
    let read = snapshot();
    let mut pane = Pane::new(read.clone()).unwrap();
    let mut foreign = read.clone();
    foreign.moot = [99; 32];
    assert_eq!(pane.refresh(foreign).unwrap_err(), Refusal::ForeignMoot);
    let mut reader = read.clone();
    reader.reader = [99; 32];
    assert_eq!(pane.refresh(reader).unwrap_err(), Refusal::DifferentReader);
    let mut conflicting = read.clone();
    conflicting.resources[0].reference.manifest = [99; 32];
    assert_eq!(
        pane.refresh(conflicting).unwrap_err(),
        Refusal::ChangedContribution
    );
    assert_eq!(pane.snapshot(), &read);
    let mut duplicate = read.clone();
    duplicate.resources.push(duplicate.resources[0].clone());
    assert!(matches!(
        Pane::new(duplicate),
        Err(Refusal::DuplicateContribution)
    ));
}

#[test]
fn requests_cannot_be_relabelled_to_attach_a_completion_to_another_resource() {
    let read = snapshot();
    let mut pane = Pane::new(read.clone()).unwrap();
    pane.select_resource(read.resources[0].reference).unwrap();
    let request = pane.prepare_resource(ResourceAction::Open).unwrap();
    let mut relabelled = request.clone();
    relabelled.target = Target::Resource {
        reference: read.resources[1].reference,
        action: ResourceAction::Open,
        observed_terms: None,
    };
    assert!(!pane.complete(
        &relabelled,
        Outcome::Accepted {
            owner_receipt: "wrong target".into()
        }
    ));
    assert!(pane.complete(
        &request,
        Outcome::Refused {
            reason: "owner rechecked and refused".into()
        }
    ));
    assert_eq!(pane.selected_resource(), Some(read.resources[0].reference));
}

#[test]
fn view_json_round_trips_without_becoming_a_capability_and_future_reports_are_refused() {
    let read = snapshot();
    let bytes = serde_json::to_vec(&read).unwrap();
    let decoded: Snapshot = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(decoded, read);
    let mut future = decoded;
    future.activities[0].lifecycle.version += 1;
    assert!(matches!(
        Pane::new(future),
        Err(Refusal::InvalidLifecycle(_))
    ));
    let mut unknown_owner = read;
    unknown_owner.activities[0].reference.owner.clear();
    assert!(matches!(
        Pane::new(unknown_owner),
        Err(Refusal::MissingActivityOwner)
    ));
}

#[test]
fn offered_collection_fork_requires_a_named_different_reviewed_destination() {
    for destination in [
        None,
        Some(ForkDestination {
            id: [5; 32],
            name: "Same source".into(),
        }),
        Some(ForkDestination {
            id: [19; 32],
            name: " ".into(),
        }),
    ] {
        let mut read = snapshot();
        read.collection.as_mut().unwrap().fork_destination = destination;
        let mut pane = Pane::new(read).unwrap();
        assert_eq!(
            pane.prepare_collection_fork().unwrap_err(),
            Refusal::InvalidForkDestination
        );
    }
}

#[test]
fn a_disclosed_sharing_root_is_immutable_without_inventing_content_authorship() {
    let mut read = snapshot();
    read.resources[0].shared_by = Some([17; 32]);
    let mut pane = Pane::new(read.clone()).unwrap();
    read.resources[0].shared_by = Some([18; 32]);
    assert_eq!(
        pane.refresh(read).unwrap_err(),
        Refusal::ChangedContribution
    );
    assert_eq!(pane.snapshot().resources[0].shared_by, Some([17; 32]));
}
