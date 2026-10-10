// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Graphshell's identity endpoint over the real keeper.
//!
//! These ran in graphshell against castellan's `PersonaeHost` until DR-C
//! (dramatis repo plan, D15). Graphshell now tests its endpoint over a fixed
//! authority, and the keeper's half, real private key bytes and a real SSH
//! key generated through an intent, is proved here, where castellan may be
//! linked.

use std::sync::Arc;

use castellan::authority::PersonaeHost;
use castellan::custody::ssh_slot::{protocol_key_for, slot_for};
use castellan::custody::{IdentityVault, InMemoryStorage, Profile};
use chirograph::{IntentInvocation, IntentResult, ResourceRequest};
use djinn::keeper::Keeper;
use graphshell::identity::VaultProtectionView;
use graphshell::identity_endpoint::IdentityEndpoint;
use graphshell::identity_projection::{
    GenerateSshKeyIntentV1, SSH_GENERATE_INTENT, SshUnlockPolicyIntentV1,
};
use graphshell_endpoint::{IntentSink, PresentationSource, ProjectionSource};
use personae::{Ed25519Keypair, ProfileId, UnlockTier};
use ssh_key::{Algorithm, LineEnding};

type Kept = Keeper<InMemoryStorage>;

fn endpoint_with_private_sentinel() -> (IdentityEndpoint<Kept>, String) {
    let mut private =
        ssh_key::PrivateKey::random(&mut rand_core::OsRng, Algorithm::Ed25519).unwrap();
    private.set_comment("endpoint-receipt");
    let private_openssh = private.to_openssh(LineEnding::LF).unwrap().to_string();
    let mut profile = Profile::new(
        ProfileId("research".to_string()),
        "Research",
        Ed25519Keypair::from_seed([0x6b; 32]),
    );
    profile.slots.insert(
        protocol_key_for(&private),
        slot_for(&private, UnlockTier::PerUse).unwrap(),
    );
    let host = Arc::new(PersonaeHost::new(
        IdentityVault::with_profile(InMemoryStorage::new(), profile),
        None,
        VaultProtectionView::Ephemeral,
    ));
    (
        IdentityEndpoint::new(Arc::new(Keeper::new(host))),
        private_openssh,
    )
}

#[test]
fn the_keepers_cards_carry_no_private_key_bytes() {
    let (mut endpoint, private_openssh) = endpoint_with_private_sentinel();
    let snapshot = endpoint.snapshot(endpoint.request()).unwrap();
    let resources = snapshot
        .presentation
        .offers
        .values()
        .flatten()
        .map(|offer| offer.resource)
        .collect::<Vec<_>>();
    assert!(!resources.is_empty(), "the keeper's cards are on offer");
    for resource in resources {
        let response = endpoint
            .resource(ResourceRequest {
                session: snapshot.session.clone(),
                resource,
            })
            .unwrap();
        let text = String::from_utf8(response.bytes).unwrap();
        assert!(!text.contains(&private_openssh));
        assert!(!text.contains("BEGIN OPENSSH PRIVATE KEY"));
    }
}

#[test]
fn an_advertised_generate_intent_reaches_the_keeper() {
    let (mut endpoint, _) = endpoint_with_private_sentinel();
    let snapshot = endpoint.snapshot(endpoint.request()).unwrap();
    let vault = snapshot
        .presentation
        .bindings
        .iter()
        .find(|binding| {
            snapshot.presentation.offers.get(&binding.key).unwrap()[0]
                .semantics
                .actions
                .iter()
                .any(|action| action.intent.0 == SSH_GENERATE_INTENT)
        })
        .unwrap()
        .instance;
    let accepted = endpoint
        .invoke(IntentInvocation {
            session: snapshot.session,
            target: vault,
            observed_epoch: snapshot.scene.epoch,
            observed_revision: snapshot.scene.revision,
            intent: SSH_GENERATE_INTENT.to_string(),
            payload: serde_json::to_vec(&GenerateSshKeyIntentV1 {
                comment: "generated through endpoint".to_string(),
                unlock_policy: SshUnlockPolicyIntentV1::Session,
            })
            .unwrap(),
        })
        .unwrap();
    assert_eq!(accepted, IntentResult::Accepted);
    assert_eq!(endpoint.host().snapshot().unwrap().ssh_keys.len(), 2);
}
