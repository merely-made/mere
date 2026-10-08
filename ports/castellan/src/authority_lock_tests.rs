// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Vault lock, L2: castellan obeys (rulings 1, 10, 11, 13, 14, 27, 31).
//!
//! Every host here sits over a lockable temp vault, a sealed storage with a
//! passphrase enrolled, since `InMemoryStorage` cannot lock.

use std::path::Path;

use personae::{Ed25519Keypair, PersonaId, Profile, ProfileId, SealedProfileStorage};
use ssh_agent_lib::agent::Session;
use ssh_agent_lib::proto::SignRequest;
use tempfile::tempdir;

use super::*;
use crate::items::ItemStoreError;
use crate::otp::{OtpReleaseError, OtpReleaseGate, OtpReleaseParticipantClaim};
use crate::resident::{CastellanResident, CredentialSalts};

const PASSPHRASE: &[u8] = b"castellan lock";
const ROOT: [u8; 32] = [0x5d; 32];
const SALTS: CredentialSalts = CredentialSalts {
    record: b"castellan-test/records",
    freshness: b"castellan-test/freshness",
};
const TOTP: &str = "otpauth://totp/Example:mark?secret=GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ&issuer=Example";

/// A host over a lockable temp vault holding `research` (one Ed25519 key)
/// and `personal`, plus a credential authority registered as its holder.
struct Locked {
    host: PersonaeHost<SealedProfileStorage>,
    resident: CastellanResident,
    key: PrivateKey,
}

fn lockable(dir: &Path) -> Locked {
    let storage = SealedProfileStorage::open_with_key(dir.join("vault"), ROOT);
    storage.enroll_passphrase(PASSPHRASE).unwrap();
    let mut key = PrivateKey::random(&mut rand_core::OsRng, Algorithm::Ed25519).unwrap();
    key.set_comment("lock-receipt");
    let mut research = Profile::new(
        ProfileId("research".into()),
        "Research",
        Ed25519Keypair::from_seed([0x2b; 32]),
    );
    research.slots.insert(
        ssh_slot::protocol_key_for(&key),
        ssh_slot::slot_for(&key, UnlockTier::Session).unwrap(),
    );
    storage.save_profile(&research).unwrap();
    storage
        .save_profile(&Profile::new(
            ProfileId("personal".into()),
            "Personal",
            Ed25519Keypair::from_seed([0x2c; 32]),
        ))
        .unwrap();
    let host = PersonaeHost::with_decision_timeout(
        IdentityVault::open(storage, &ProfileId("research".into())).unwrap(),
        None,
        VaultProtectionView::OsProtected,
        Duration::from_secs(2),
    );
    let resident =
        CastellanResident::claim_derived(dir.join("records"), dir.join("freshness"), &host, SALTS)
            .unwrap();
    host.register_lock_holder(resident.lock_holder(SALTS));
    Locked {
        host,
        resident,
        key,
    }
}

fn unlock(host: &PersonaeHost<SealedProfileStorage>) {
    host.unlock_vault(UnlockMethod::Passphrase(PASSPHRASE))
        .unwrap();
}

fn participant() -> OtpReleaseParticipantClaim {
    OtpReleaseParticipantClaim::unverified("local:test", "lock:test").unwrap()
}

/// Every record file under `dir`, path and bytes, in order.
fn record_files(dir: &Path) -> Vec<(String, Vec<u8>)> {
    fn walk(dir: &Path, root: &Path, out: &mut Vec<(String, Vec<u8>)>) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                walk(&path, root, out);
            } else if path.extension().is_some_and(|e| e == "json") {
                let name = path.strip_prefix(root).unwrap().display().to_string();
                out.push((name, std::fs::read(&path).unwrap()));
            }
        }
    }
    let mut out = Vec::new();
    walk(dir, dir, &mut out);
    out.sort();
    out
}

// ─── Ruling 1: the holder hook drops castellan's keys ─────────────────────

#[test]
fn a_lock_drops_the_credential_keys_so_items_and_the_otp_gate_say_locked() {
    let dir = tempdir().unwrap();
    let fixture = lockable(dir.path());
    let persona = PersonaId::new();
    let otp = fixture.resident.otp_items(persona);
    let item = otp.import_otpauth_uri(TOTP).unwrap();
    let gate = OtpReleaseGate::new(fixture.resident.otp_items(persona));
    let pending = gate
        .petition(item.item_id(), item.credential_id(), participant())
        .unwrap();
    let listed = serde_json::to_vec(&fixture.resident.items(persona).list().unwrap()).unwrap();
    let tile = otp
        .release_tile_at_unix_time(item.item_id(), item.credential_id(), 59)
        .unwrap();
    let on_disk = record_files(&dir.path().join("records"));

    fixture.host.lock_vault().unwrap();
    assert!(fixture.resident.is_locked(), "the hook ran before lock returned");
    assert!(matches!(
        fixture.resident.items(persona).list(),
        Err(ItemStoreError::Locked)
    ));
    assert!(matches!(
        gate.petition(item.item_id(), item.credential_id(), participant()),
        Err(OtpReleaseError::Locked)
    ));
    assert!(matches!(gate.approve(pending.id), Err(OtpReleaseError::Locked)));

    unlock(&fixture.host);
    assert!(!fixture.resident.is_locked(), "re-derived on unlock");
    let again = serde_json::to_vec(&fixture.resident.items(persona).list().unwrap()).unwrap();
    assert_eq!(again, listed, "items read back byte-identical");
    assert_eq!(
        otp.release_tile_at_unix_time(item.item_id(), item.credential_id(), 59)
            .unwrap()
            .code_at_unix_time(59),
        tile.code_at_unix_time(59)
    );
    assert_eq!(
        record_files(&dir.path().join("records")),
        on_disk,
        "the lock rewrote nothing"
    );
    assert!(gate.approve(pending.id).is_ok(), "the petition survived");
}

/// A holder that cannot re-derive leaves nothing half unlocked.
#[test]
fn a_holder_that_cannot_rederive_relocks_everything() {
    struct Refuses;
    impl VaultLockHolder for Refuses {
        fn name(&self) -> &str {
            "refuses"
        }
        fn lock(&self) {}
        fn unlock(&self, _: &dyn IdentityProvider) -> Result<(), IdentityError> {
            Err(IdentityError::Backend("refused".into()))
        }
    }
    let dir = tempdir().unwrap();
    let fixture = lockable(dir.path());
    fixture.host.register_lock_holder(Arc::new(Refuses));
    fixture.host.lock_vault().unwrap();
    assert!(
        fixture
            .host
            .unlock_vault(UnlockMethod::Passphrase(PASSPHRASE))
            .is_err()
    );
    assert!(fixture.host.is_locked());
    assert!(fixture.resident.is_locked());
    assert_eq!(*fixture.host.lock_state().borrow(), VaultLockView::Locked);
}

// ─── Ruling 27: no lock nobody can undo ───────────────────────────────────

#[test]
fn a_refused_lock_leaves_the_holders_their_keys() {
    let dir = tempdir().unwrap();
    let host = PersonaeHost::new(
        IdentityVault::with_profile(
            personae::InMemoryStorage::new(),
            Profile::new(
                ProfileId("research".into()),
                "Research",
                Ed25519Keypair::from_seed([0x2d; 32]),
            ),
        ),
        None,
        VaultProtectionView::Ephemeral,
    );
    let resident =
        CastellanResident::claim_derived(dir.path().join("r"), dir.path().join("f"), &host, SALTS)
            .unwrap();
    host.register_lock_holder(resident.lock_holder(SALTS));
    assert!(host.lock_vault().is_err());
    assert!(!host.is_locked());
    assert!(!resident.is_locked());
    assert_eq!(*host.lock_state().borrow(), VaultLockView::Unlocked);
}

// ─── Ruling 31: the watch channel ─────────────────────────────────────────

#[test]
fn observers_see_every_change() {
    let dir = tempdir().unwrap();
    let fixture = lockable(dir.path());
    let mut state = fixture.host.lock_state();
    assert_eq!(*state.borrow_and_update(), VaultLockView::Unlocked);
    fixture.host.lock_vault().unwrap();
    assert!(state.has_changed().unwrap());
    assert_eq!(*state.borrow_and_update(), VaultLockView::Locked);
    unlock(&fixture.host);
    assert_eq!(*state.borrow_and_update(), VaultLockView::Unlocked);
}

/// A holder registered while locked drops its keys at once.
#[test]
fn a_holder_joining_a_locked_vault_locks() {
    let dir = tempdir().unwrap();
    let fixture = lockable(dir.path());
    fixture.host.lock_vault().unwrap();
    let late = CastellanResident::claim(
        dir.path().join("late-records"),
        [0x41; 32],
        dir.path().join("late-freshness"),
        [0x42; 32],
    )
    .unwrap();
    fixture.host.register_lock_holder(late.lock_holder(SALTS));
    assert!(late.is_locked());
}

// ─── Rulings 10, 11, 14: the surfaces ─────────────────────────────────────

#[test]
fn the_snapshot_reports_locked_with_the_view_kept_at_lock_time() {
    let dir = tempdir().unwrap();
    let fixture = lockable(dir.path());
    let before = fixture.host.snapshot().unwrap();
    assert_eq!(before.vault.lock, VaultLockView::Unlocked);
    fixture.host.lock_vault().unwrap();
    let locked = fixture.host.snapshot().unwrap();
    assert_eq!(locked.vault.lock, VaultLockView::Locked);
    assert_eq!(locked.profiles, before.profiles);
    assert_eq!(locked.ssh_keys, before.ssh_keys);
    assert_eq!(locked.ssh_keys.len(), 1);
    let json = locked.to_public_json().unwrap();
    assert!(!json.contains("BEGIN OPENSSH PRIVATE KEY"));
}

#[test]
fn the_lock_intent_works_from_any_surface_and_unlock_is_native_only() {
    let dir = tempdir().unwrap();
    let fixture = lockable(dir.path());
    let outcome = fixture
        .host
        .apply_intent(VAULT_LOCK_INTENT, b"{}")
        .unwrap();
    assert_eq!(outcome, IdentityIntentOutcome::VaultLocked);
    assert!(fixture.host.is_locked());
    // Locking a locked vault through the intent is harmless.
    fixture.host.apply_intent(VAULT_LOCK_INTENT, b"{}").unwrap();

    let passphrase = serde_json::to_vec(&serde_json::json!({ "passphrase": "castellan lock" }))
        .unwrap();
    for payload in [&b"{}"[..], &passphrase] {
        assert!(matches!(
            fixture.host.apply_intent(VAULT_UNLOCK_INTENT, payload),
            Err(IdentityIntentError::UnlockNativeOnly)
        ));
    }
    assert!(fixture.host.is_locked());
    assert!(
        fixture
            .host
            .unlock_vault(UnlockMethod::Passphrase(b"wrong"))
            .is_err()
    );
    assert!(fixture.host.is_locked());
    unlock(&fixture.host);
    assert!(!fixture.host.is_locked());
}

#[test]
fn a_switch_is_refused_while_locked() {
    let dir = tempdir().unwrap();
    let fixture = lockable(dir.path());
    fixture.host.lock_vault().unwrap();
    let payload = serde_json::to_vec(&SwitchProfileIntentV1 {
        profile: "personal".into(),
    })
    .unwrap();
    assert!(matches!(
        fixture.host.apply_intent(PROFILE_SWITCH_INTENT, &payload),
        Err(IdentityIntentError::Locked)
    ));
    unlock(&fixture.host);
    let selected: Vec<String> = fixture
        .host
        .snapshot()
        .unwrap()
        .profiles
        .into_iter()
        .filter(|p| p.selected)
        .map(|p| p.id)
        .collect();
    assert_eq!(selected, ["research"]);
}

#[test]
fn key_mutations_and_creation_are_refused_while_locked() {
    let dir = tempdir().unwrap();
    let fixture = lockable(dir.path());
    fixture.host.lock_vault().unwrap();
    let generate = serde_json::to_vec(&GenerateSshKeyIntentV1 {
        comment: "while locked".into(),
        unlock_policy: SshUnlockPolicyIntentV1::Session,
    })
    .unwrap();
    assert!(matches!(
        fixture.host.apply_intent(SSH_GENERATE_INTENT, &generate),
        Err(IdentityIntentError::Locked)
    ));
    let create = serde_json::to_vec(&CreateProfileIntentV1 {
        id: "late".into(),
        display_name: "Late".into(),
    })
    .unwrap();
    assert!(matches!(
        fixture.host.apply_intent(PROFILE_CREATE_INTENT, &create),
        Err(IdentityIntentError::Locked)
    ));
    let remove = serde_json::to_vec(&RemoveSshKeyIntentV1 {
        fingerprint: PublicKey::from(&fixture.key)
            .fingerprint(ssh_key::HashAlg::Sha256)
            .to_string(),
        confirmed: true,
    })
    .unwrap();
    assert!(matches!(
        fixture.host.apply_intent(SSH_REMOVE_INTENT, &remove),
        Err(IdentityIntentError::Locked)
    ));
    unlock(&fixture.host);
    assert_eq!(fixture.host.snapshot().unwrap().ssh_keys.len(), 1);
}

/// pandect's wallet is not relocked yet (ruling 15 is a fork), so castellan
/// refuses to reach it while locked.
#[test]
fn revocation_and_the_wallet_sealer_are_refused_while_locked() {
    let dir = tempdir().unwrap();
    let fixture = lockable(dir.path());
    fixture.host.lock_vault().unwrap();
    assert!(matches!(
        fixture.host.revoke_device(RevokeDeviceIntentV1 {
            device_id: Uuid::new_v4(),
            confirmed: true,
        }),
        Err(IdentityIntentError::Locked)
    ));
    assert!(matches!(
        fixture.host.payload_sealer(PersonaId::new()),
        Err(error) if error.kind() == io::ErrorKind::PermissionDenied
    ));
}

// ─── Rulings 8 and 9: the agent, in process ───────────────────────────────

#[tokio::test]
async fn the_agent_locks_the_whole_resident_and_refuses_to_unlock_it() {
    let dir = tempdir().unwrap();
    let fixture = lockable(dir.path());
    let mut agent = fixture.host.agent_session();
    agent.lock(String::from("pw")).await.unwrap();
    assert!(fixture.host.is_locked());
    assert!(fixture.resident.is_locked(), "the agent's lock is the host's");
    assert_eq!(*fixture.host.lock_state().borrow(), VaultLockView::Locked);
    assert!(agent.unlock(String::from("castellan lock")).await.is_err());
    assert!(fixture.host.is_locked());
}

/// The wire receipt: lock by message 22 over an isolated pipe, message 23
/// refused, then a native unlock restores the same identities, a verifying
/// signature, and byte-identical items.
#[cfg(windows)]
#[tokio::test]
async fn over_an_isolated_pipe_the_agent_locks_by_the_wire_and_unlocks_only_natively() {
    use signature::Verifier;
    use ssh_agent_lib::client::Client;
    use tokio::net::windows::named_pipe::ClientOptions;

    let dir = tempdir().unwrap();
    let fixture = lockable(dir.path());
    let persona = PersonaId::new();
    fixture
        .resident
        .otp_items(persona)
        .import_otpauth_uri(TOTP)
        .unwrap();
    let items = || serde_json::to_vec(&fixture.resident.items(persona).list().unwrap()).unwrap();
    let items_before = items();

    let endpoint = format!(r"\\.\pipe\castellan-lock-receipt-{}", Uuid::new_v4());
    let listener = fixture.host.bind_receipt_listener(&endpoint).unwrap();
    let server = tokio::spawn(ssh_agent_lib::agent::listen(
        listener,
        fixture.host.agent_session(),
    ));
    let mut client = Client::new(ClientOptions::new().open(&endpoint).unwrap());
    let public = PublicKey::from(&fixture.key);
    let listed = client.request_identities().await.unwrap();
    assert_eq!(listed.len(), 2);
    sign_over(&mut client, &public).await.unwrap();

    client.lock(String::from("pw")).await.unwrap();
    assert!(fixture.host.is_locked() && fixture.resident.is_locked());
    assert!(client.request_identities().await.unwrap().is_empty());
    assert!(sign_over(&mut client, &public).await.is_err());
    assert!(client.unlock(String::from("pw")).await.is_err());
    assert!(client.lock(String::from("pw")).await.is_err(), "already locked");
    assert!(fixture.host.is_locked());

    unlock(&fixture.host);
    let again = client.request_identities().await.unwrap();
    assert_eq!(
        again.iter().map(|i| i.credential.key_data()).collect::<Vec<_>>(),
        listed.iter().map(|i| i.credential.key_data()).collect::<Vec<_>>()
    );
    let signature = sign_over(&mut client, &public).await.unwrap();
    public
        .key_data()
        .verify(b"lock receipt", &signature)
        .unwrap();
    assert_eq!(items(), items_before);

    server.abort();
    let _ = server.await;
}

#[cfg(windows)]
async fn sign_over(
    client: &mut impl Session,
    public: &PublicKey,
) -> Result<ssh_key::Signature, ssh_agent_lib::error::AgentError> {
    client
        .sign(SignRequest {
            credential: public.key_data().clone().into(),
            data: b"lock receipt".to_vec(),
            flags: 0,
        })
        .await
}
