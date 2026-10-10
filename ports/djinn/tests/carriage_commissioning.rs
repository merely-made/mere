// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Graphshell's carriage commissioning through the real wallet.
//!
//! Moved from graphshell's `carriage_host_tests` in DR-C (dramatis repo plan,
//! D15): the wallet's seed and wrapping keys are castellan's custody, so the
//! resident loads them and hands them to the carriage host, and the story
//! that exercises both lives where castellan may be linked.

use std::path::PathBuf;

use graphshell::carriage::CarriageCeilings;
use graphshell::native::browser_host::now_ms;
use graphshell::native::carriage_host::{CarriageHost, CarriageHostConfig};
use personae::Ed25519Keypair;

/// The wrapping keys pairing retained, as the resident loads them.
fn wrapping_keys(carry_root: &std::path::Path) -> pandect::RemoteAuthWrappingKeyBridge {
    castellan::custody::wallet::load_remote_auth_wrapping_key_bridge(carry_root)
        .unwrap()
        .unwrap_or_default()
}

/// The commissioning story end to end, through the real machinery at every
/// step: a wallet pairs a device with a private epoch, leases it onto a
/// graph, publishes through the roster-driven path, and a peer replica
/// serves back a record the pairing key can actually open.
mod commissioning {
    use super::*;
    use castellan::custody::wallet::{
        ensure_wallet_state, issue_remote_auth_device_grant_from_pairing, load_identity_seed,
    };
    use pandect::{
        CarriagePolicy, DeviceExposure, DeviceId, DevicePublicKey, KeyEpochId,
        PairedRemoteAuthGrantSpec, PersonaId, PrivateEpochPlaintext, blinded_slot_id,
        decode_epoch_record, load_device_grant_set, load_device_roster, save_device_roster,
        unwrap_private_epoch_material,
    };
    use personae::InMemoryProvider;
    use personae::carry::derive_persona_chain_root;
    use uuid::Uuid;

    const GRAPH: [u8; 32] = [0x91; 32];

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn a_paired_leased_device_recovers_its_epoch_through_a_peer() {
        let directory = tempfile::tempdir().unwrap();
        let carry_root = directory.path().join("wallet-root");
        let persona = PersonaId::new();
        ensure_wallet_state(&carry_root, persona, "Graphshell workstation").unwrap();

        // Pair a device carrying one private epoch. The pairing path is the
        // one that retains a wrapping key, which is what makes the slot
        // addressable at publish time.
        let device_id = DeviceId::new();
        let epoch = KeyEpochId(Uuid::from_u128(0x92));
        let delegatee = Ed25519Keypair::from_seed([0x93; 32]);
        let now = now_ms();
        let (_grant, pairing) = issue_remote_auth_device_grant_from_pairing(
            &carry_root,
            &PairedRemoteAuthGrantSpec {
                device_id,
                delegatee_pubkey: DevicePublicKey::from(delegatee.public_key()),
                label: "Pocket relay".into(),
                exposure: DeviceExposure::HiddenClient,
                issued_at_ms: now,
                expires_at_ms: Some(now + 7 * 24 * 60 * 60 * 1000),
                personas: vec![persona],
                scopes: vec!["identity.act".into(), "private.read".into()],
                attenuations: vec!["no-subdelegation".into()],
                pairing_secret: b"qr-code-derived-shared-secret".to_vec(),
                private_epochs: vec![PrivateEpochPlaintext {
                    persona_id: persona,
                    epoch_id: epoch,
                    epoch_secret: b"commissioned-epoch-secret".to_vec(),
                }],
            },
        )
        .unwrap();

        // Lease the device onto this graph: the ruled layering's "whether",
        // decided on the roster where the other posture fields live.
        let mut roster = load_device_roster(&carry_root).unwrap().unwrap();
        let record = roster
            .devices
            .iter_mut()
            .find(|record| record.device_id == device_id)
            .unwrap();
        record.carriage = CarriagePolicy::Leased {
            max_ttl_ms: 60_000,
            graph: GRAPH,
        };
        save_device_roster(&carry_root, &roster).unwrap();

        // Verifiers hold one trusted root per persona: its chain root.
        let master_seed = load_identity_seed(&carry_root).unwrap().unwrap();
        let chain_root = derive_persona_chain_root(master_seed, persona).unwrap();
        let trusted = vec![chain_root.0];

        let wallet_device = InMemoryProvider::from_seed([0x94; 32]);
        let replica_device = InMemoryProvider::from_seed([0x95; 32]);
        let host_config = |path: PathBuf, tickets: Vec<String>| CarriageHostConfig {
            graph: GRAPH,
            store_path: path,
            trusted_roots: trusted.clone(),
            ceilings: CarriageCeilings::default(),
            peer_tickets: tickets,
            paired_nodes: Vec::new(),
        };
        let wallet_host = CarriageHost::open(
            &wallet_device,
            host_config(directory.path().join("wallet.redb"), Vec::new()),
        )
        .await
        .unwrap();
        let replica_host = CarriageHost::open(
            &replica_device,
            host_config(
                directory.path().join("replica.redb"),
                vec![wallet_host.ticket().await.unwrap()],
            ),
        )
        .await
        .unwrap();

        // The roster-driven publish. One persona certificate carries epoch
        // material, so exactly one slot goes onto the lane.
        let report = wallet_host
            .publish_grant_carriage(
                &carry_root,
                &InMemoryProvider::from_seed(master_seed),
                &wrapping_keys(&carry_root),
            )
            .await
            .unwrap();
        assert_eq!(report.published, vec![(device_id, persona)]);
        assert!(report.skipped_no_wrapping_key.is_empty());
        assert_eq!(report.skipped_no_record, 0);

        // The device's side of the story: it knows its certificate and its
        // pairing key, so it can compute its slot and ask a peer.
        let set = load_device_grant_set(&carry_root, device_id).unwrap();
        let certificate_id = set.personas.get(&persona).unwrap().certificate.id();
        let slot = blinded_slot_id(certificate_id, pairing.wrapping_key);

        let mut recovered = None;
        for _ in 0..100 {
            recovered = replica_host.recover(slot).await.unwrap();
            if recovered.is_some() {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
        let bytes = recovered.expect("the replica must serve the commissioned slot");

        // The recovered record is not just bytes: the pairing key opens it,
        // which is recovery without re-pairing, the lane's whole point.
        let record = decode_epoch_record(&bytes).unwrap();
        assert_eq!(record.certificate, certificate_id);
        let secret =
            unwrap_private_epoch_material(&record.epochs[0], persona, epoch, pairing.wrapping_key)
                .unwrap();
        assert_eq!(secret, b"commissioned-epoch-secret".to_vec());

        wallet_host.close().await.unwrap();
        replica_host.close().await.unwrap();
    }
}

/// The revocation fast path, end to end through real revocation: material a
/// peer holds is destroyed by retraction now, not at expiry.
mod retraction {
    use super::*;
    use castellan::custody::wallet::{
        ensure_wallet_state, issue_remote_auth_device_grant_from_pairing, load_identity_seed,
        revoke_remote_auth_device,
    };
    use pandect::{
        CarriagePolicy, DeviceExposure, DeviceId, DevicePublicKey, KeyEpochId,
        PairedRemoteAuthGrantSpec, PersonaId, PrivateEpochPlaintext, blinded_slot_id,
        decode_epoch_record, load_device_grant_set, load_device_roster, save_device_roster,
    };
    use personae::InMemoryProvider;
    use personae::carry::derive_persona_chain_root;
    use uuid::Uuid;

    const GRAPH: [u8; 32] = [0xA1; 32];

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn revoking_a_device_destroys_its_carriage_on_a_peer_before_expiry() {
        let directory = tempfile::tempdir().unwrap();
        let carry_root = directory.path().join("wallet-root");
        let persona = PersonaId::new();
        ensure_wallet_state(&carry_root, persona, "Graphshell workstation").unwrap();

        let device_id = DeviceId::new();
        let now = now_ms();
        let (_grant, pairing) = issue_remote_auth_device_grant_from_pairing(
            &carry_root,
            &PairedRemoteAuthGrantSpec {
                device_id,
                delegatee_pubkey: DevicePublicKey::from(
                    Ed25519Keypair::from_seed([0xA2; 32]).public_key(),
                ),
                label: "Doomed relay".into(),
                exposure: DeviceExposure::HiddenClient,
                issued_at_ms: now,
                expires_at_ms: Some(now + 7 * 24 * 60 * 60 * 1000),
                personas: vec![persona],
                scopes: vec!["identity.act".into(), "private.read".into()],
                attenuations: vec!["no-subdelegation".into()],
                pairing_secret: b"qr-code-derived-shared-secret".to_vec(),
                private_epochs: vec![PrivateEpochPlaintext {
                    persona_id: persona,
                    epoch_id: KeyEpochId(Uuid::from_u128(0xA3)),
                    epoch_secret: b"soon-to-be-retracted".to_vec(),
                }],
            },
        )
        .unwrap();
        let mut roster = load_device_roster(&carry_root).unwrap().unwrap();
        roster
            .devices
            .iter_mut()
            .find(|record| record.device_id == device_id)
            .unwrap()
            .carriage = CarriagePolicy::Leased {
            // A long lease on purpose: if expiry did the destroying, this
            // test could not tell retraction from waiting.
            max_ttl_ms: 60 * 60 * 1000,
            graph: GRAPH,
        };
        save_device_roster(&carry_root, &roster).unwrap();

        let master_seed = load_identity_seed(&carry_root).unwrap().unwrap();
        let trusted = vec![derive_persona_chain_root(master_seed, persona).unwrap().0];
        let host_config = |path: PathBuf, tickets: Vec<String>| CarriageHostConfig {
            graph: GRAPH,
            store_path: path,
            trusted_roots: trusted.clone(),
            ceilings: CarriageCeilings::default(),
            peer_tickets: tickets,
            paired_nodes: Vec::new(),
        };
        let wallet_host = CarriageHost::open(
            &InMemoryProvider::from_seed([0xA4; 32]),
            host_config(directory.path().join("wallet.redb"), Vec::new()),
        )
        .await
        .unwrap();
        let replica_host = CarriageHost::open(
            &InMemoryProvider::from_seed([0xA5; 32]),
            host_config(
                directory.path().join("replica.redb"),
                vec![wallet_host.ticket().await.unwrap()],
            ),
        )
        .await
        .unwrap();

        // Slot computed while the wrapping key exists; revocation deletes it.
        let set = load_device_grant_set(&carry_root, device_id).unwrap();
        let certificate_id = set.personas.get(&persona).unwrap().certificate.id();
        let slot = blinded_slot_id(certificate_id, pairing.wrapping_key);

        let report = wallet_host
            .publish_grant_carriage(
                &carry_root,
                &InMemoryProvider::from_seed(master_seed),
                &wrapping_keys(&carry_root),
            )
            .await
            .unwrap();
        assert_eq!(report.published.len(), 1);
        let mut held = None;
        for _ in 0..100 {
            held = replica_host.recover(slot).await.unwrap();
            if held.is_some() {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
        let before = decode_epoch_record(&held.expect("replica must hold the slot")).unwrap();
        assert!(!before.epochs.is_empty(), "the lease carries real material");

        // The real revocation, wrapping-key deletion and all. Then the fast
        // path, addressable only because the index was written at publish.
        revoke_remote_auth_device(&carry_root, device_id).unwrap();
        let retract = wallet_host
            .retract_device_carriage(device_id, &InMemoryProvider::from_seed(master_seed))
            .await
            .unwrap();
        assert_eq!(retract.retracted, vec![slot.0]);
        assert!(retract.skipped_not_held.is_empty());

        // The peer's copy is destroyed by supersession, with the lease still
        // hours from expiry: the recovered record is now the empty shell.
        let mut after = None;
        for _ in 0..100 {
            if let Some(bytes) = replica_host.recover(slot).await.unwrap() {
                let record = decode_epoch_record(&bytes).unwrap();
                if record.epochs.is_empty() {
                    after = Some(record);
                    break;
                }
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
        let shell = after.expect("the retraction must reach the peer");
        assert_eq!(shell.certificate, certificate_id);

        // Retraction is once: the index is consumed with it.
        let again = wallet_host
            .retract_device_carriage(device_id, &InMemoryProvider::from_seed(master_seed))
            .await
            .unwrap();
        assert!(again.retracted.is_empty());

        wallet_host.close().await.unwrap();
        replica_host.close().await.unwrap();
    }
}
