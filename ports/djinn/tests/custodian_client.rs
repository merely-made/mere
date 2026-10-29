// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Custody isolation from Djinn's side (H1 and H2): Djinn's custody is a
//! client of a real custodian over a socket, and nothing falls back to a
//! vault in Djinn when the custodian refuses, is locked or is gone. Every
//! key or signature is checked against the custodian's vault opened
//! directly, so an answer that checks came from the custodian.

#![cfg(unix)]

mod custodian_fixture;

use std::collections::BTreeSet;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use custodian_fixture::{TestCustodian, grants_with, manage_grants};
use djinn::custodian::InstalledCustodian;
use djinn::custody_host::remote::RemoteHost;
use djinn::custody_host::{LockHolder, ResidentCustody};
use djinn::door::resident_identity::ResidentIdentity;
use djinn::keeper::Keeper;
use dramatis::intents::CreateProfileIntentV1;
use graphshell_endpoint::door::app_admission::AppId;
use graphshell_endpoint::door::custody::{CustodyAnswer, CustodyCall, CustodyRefusal, KeySource};
use graphshell_endpoint::door::identity::VaultLockView;
use insigne::delegation::{CapabilityScope, DelegationCertificate, DelegationParent, delegation_signing_salt};
use personae::{IdentityError, IdentityProvider, ProfileId};
use serde_json::json;

const PASSPHRASE: &str = "custodian client passphrase";
const WORK: [u8; 32] = [0x31; 32];
const HOME: [u8; 32] = [0x32; 32];

fn mesh_grants() -> serde_json::Value {
    let mut grants = manage_grants();
    let list = grants.as_array_mut().unwrap();
    list.push(json!({"operation": "release", "profile": "current", "salts": [{"exact": "mesh-author"}, {"exact": "distillery-personal-mesh"}, {"prefix": "holder/"}], "consent": "none"}));
    list.push(json!({"operation": "sign", "profile": "current", "purpose": {"purpose": "delegation", "audience": "mere.graphshell"}, "consent": "none"}));
    grants
}

fn custodian() -> TestCustodian {
    TestCustodian::start(PASSPHRASE, &[("work", WORK), ("home", HOME)], mesh_grants())
}

fn unlocked(custodian: &TestCustodian) -> Arc<RemoteHost> {
    let remote = custodian.remote();
    remote.unlock_with_passphrase(PASSPHRASE.as_bytes()).unwrap();
    remote
}

fn delegation(master: [u8; 32]) -> DelegationCertificate {
    DelegationCertificate::new(
        DelegationParent::Root([8; 32]),
        master,
        [6; 32],
        CapabilityScope {
            domain: "mere.graphshell".into(),
            resource: b"guest".to_vec(),
            path_prefix: "/".into(),
            actions: BTreeSet::from(["visit".to_string()]),
        },
        10,
        10,
        None,
        0,
        [2; 32],
    )
}

#[test]
fn the_distillery_mesh_author_is_released_by_the_custodian_for_exactly_its_salts() {
    let custodian = custodian();
    let remote = unlocked(&custodian);
    let salts = distillery::InstalledAuthority::release_salts();
    let released = remote.retained_keys(&salts).unwrap();
    let direct = custodian.direct("work");
    assert_eq!(released.master_public_key(), direct.master_public_key());
    for salt in &salts {
        assert_eq!(released.derive_keypair(salt).unwrap().to_seed(), direct.derive_keypair(salt).unwrap().to_seed());
    }
    assert!(released.derive_keypair(b"mesh-author-adjacent").is_err(), "only what was released");
    let other = [b"graphshell/endpoint-session/x".to_vec()];
    assert!(matches!(remote.release(KeySource::Persona, None, &other), Err(CustodyRefusal::NotReleasable)));
    let home = ProfileId("home".into());
    assert!(
        matches!(remote.release(KeySource::Persona, Some(&home), &salts), Err(CustodyRefusal::NotReleasable)),
        "a profile the grant does not cover is refused"
    );
    assert!(remote.derive_keypair(b"").is_err(), "the master's place is never released");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_graphshell_delegation_is_signed_by_the_custodian_not_by_djinn() {
    let custodian = custodian();
    let keeper = Arc::new(Keeper::new(unlocked(&custodian)));
    let master = custodian.direct("work").master_public_key().to_bytes();
    let statement = delegation(master);
    let salt = delegation_signing_salt(&statement.scope);
    let call = CustodyCall::Sign {
        source: KeySource::Persona,
        salt: salt.clone(),
        message: statement.signing_bytes(),
    };
    let answer = keeper.clone().custody(AppId::new("graphshell"), call).await.unwrap();
    let CustodyAnswer::Signature { public_key, signature } = answer else {
        panic!("not a signature")
    };
    let direct = custodian.direct("work");
    assert_eq!(public_key, direct.derive_keypair(&salt).unwrap().public_key().to_bytes(), "the custodian's scope key");
    let signature: [u8; 64] = signature.try_into().unwrap();
    assert!(personae::Ed25519PublicKey::from_bytes(&public_key).unwrap().verify(&statement.signing_bytes(), &personae::Ed25519Signature::from_bytes(&signature)));

    // A statement another identity issued is not the custodian's to sign.
    let foreign = delegation([9; 32]);
    let call = CustodyCall::Sign {
        source: KeySource::Persona,
        salt: delegation_signing_salt(&foreign.scope),
        message: foreign.signing_bytes(),
    };
    let refused = keeper.clone().custody(AppId::new("graphshell"), call).await;
    assert!(matches!(refused, Err(CustodyRefusal::Refused { reason }) if reason.contains("issuer-not-profile")));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_locked_or_lost_custodian_leaves_the_identity_pending_with_no_fallback() {
    let custodian = custodian();
    let remote = unlocked(&custodian);
    let keeper = Arc::new(Keeper::new(remote.clone()));
    let statement = delegation(custodian.direct("work").master_public_key().to_bytes());
    let call = || CustodyCall::Sign {
        source: KeySource::Persona,
        salt: delegation_signing_salt(&statement.scope),
        message: statement.signing_bytes(),
    };
    remote.lock_vault().unwrap();
    assert!(keeper.is_locked());
    assert!(matches!(keeper.clone().custody(AppId::new("graphshell"), call()).await, Err(CustodyRefusal::Locked)));
    let salts = distillery::InstalledAuthority::release_salts();
    assert!(matches!(remote.retained_keys(&salts), Err(IdentityError::Locked)));

    // No custodian at all: the start fails; nothing opens a vault instead.
    let gone = tempfile::tempdir().unwrap();
    let file = gone.path().join("client.json");
    std::fs::write(&file, br#"{"endpoint": "/nonexistent/custody.sock", "server": {"unix_uids": [0]}}"#).unwrap();
    let refused = RemoteHost::connect(InstalledCustodian::from_path(&file, false).unwrap());
    assert!(refused.is_err_and(|e| e.contains("unavailable")), "never served in-process");
}

#[test]
fn a_client_file_the_desktop_user_could_write_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("client.json");
    std::fs::write(&file, br#"{"endpoint": "/run/djinn-custodian/x.sock", "server": {"unix_uids": [0]}}"#).unwrap();
    let refused = InstalledCustodian::from_path(&file, true);
    assert!(refused.is_err_and(|e| e.contains("root-owned")));
}

/// A holder that keeps one key derived through custody.
struct Holder {
    key: Mutex<Option<[u8; 32]>>,
    locks: AtomicUsize,
}

impl LockHolder for Holder {
    fn name(&self) -> &str {
        "test holder"
    }
    fn lock(&self) {
        self.locks.fetch_add(1, Ordering::SeqCst);
        *self.key.lock().unwrap() = None;
    }
    fn unlock(&self, vault: &dyn IdentityProvider) -> Result<(), IdentityError> {
        *self.key.lock().unwrap() = Some(vault.derive_keypair(b"holder/records")?.to_seed());
        Ok(())
    }
}

fn wait_for(mut done: impl FnMut() -> bool) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while !done() {
        assert!(std::time::Instant::now() < deadline, "timed out");
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
}

#[test]
fn holders_forget_on_the_custodians_lock_and_rederive_through_it_on_unlock() {
    let custodian = custodian();
    let remote = custodian.remote();
    let holder = Arc::new(Holder {
        key: Mutex::new(None),
        locks: AtomicUsize::new(0),
    });
    remote.register_lock_holder(holder.clone());
    assert_eq!(*remote.lock_state().borrow(), VaultLockView::Locked);
    remote.unlock_with_passphrase(PASSPHRASE.as_bytes()).unwrap();
    let expected = custodian.direct("work").derive_keypair(b"holder/records").unwrap().to_seed();
    assert_eq!(*holder.key.lock().unwrap(), Some(expected));
    assert_eq!(*remote.lock_state().borrow(), VaultLockView::Unlocked);

    // A lock taken elsewhere (the custodian's own idle or session lock)
    // reaches the holder as a fact, without Djinn asking.
    let locks = holder.locks.load(Ordering::SeqCst);
    custodian.service.lock_now();
    wait_for(|| holder.key.lock().unwrap().is_none());
    assert!(holder.locks.load(Ordering::SeqCst) > locks);
    wait_for(|| *remote.lock_state().borrow() == VaultLockView::Locked);
    assert!(remote.is_locked());
}

#[test]
fn profiles_snapshot_and_intents_are_answered_by_the_custodian() {
    let custodian = custodian();
    let remote = custodian.remote();
    let locked = remote.snapshot().unwrap();
    assert_eq!(locked.vault.lock, VaultLockView::Locked);
    remote.unlock_with_passphrase(PASSPHRASE.as_bytes()).unwrap();
    assert_eq!(remote.current_profile_id(), ProfileId("work".into()));
    assert_eq!(remote.master_public_key(), custodian.direct("work").master_public_key());
    assert!(remote.has_profile(&ProfileId("home".into())).unwrap());
    assert!(!remote.has_profile(&ProfileId("absent".into())).unwrap());
    let snapshot = remote.snapshot().unwrap();
    assert_eq!(snapshot.vault.lock, VaultLockView::Unlocked);
    remote
        .create_profile(CreateProfileIntentV1 {
            id: "studio".into(),
            display_name: "Studio".into(),
        })
        .unwrap();
    let after = remote.snapshot().unwrap();
    assert!(after.profiles.iter().any(|p| p.id == "studio" && p.display_name == "Studio"));
    assert_eq!(
        remote.root_public_key(KeySource::Persona, None).unwrap(),
        custodian.direct("work").master_public_key().to_bytes()
    );
    assert!(
        matches!(remote.root_public_key(KeySource::Persona, Some(&ProfileId("home".into()))), Err(CustodyRefusal::NotReleasable)),
        "another profile's root only under a grant naming it"
    );
    // Signing decisions are the custodian's consent channel, never an intent.
    assert!(remote.apply_intent(dramatis::intents::SIGNING_DENY_INTENT, b"{}").is_err());
}

#[tokio::test(flavor = "multi_thread")]
async fn the_wallets_persona_index_is_the_custodians_to_disclose_and_djinn_never_lists_a_file() {
    use djinn::custody_host::{CustodyPersonas, resolve_persona};
    let custodian = custodian();
    let remote = custodian.remote();
    let (first, second) = (personae::PersonaId::new(), personae::PersonaId::new());
    let index = |host: Arc<RemoteHost>| -> Arc<dyn djinn::custody_host::PersonaIndex> {
        // A root that holds no wallet: only the custodian's own is read.
        Arc::new(CustodyPersonas { custody: host, wallet_root: custodian.dir.path().join("djinns-data-root-has-no-wallet") })
    };
    let nothing = remote.wallet_public(custodian.dir.path()).unwrap();
    assert!(nothing.personas.is_empty() && nothing.roster.is_none());
    assert!(resolve_persona(index(Arc::clone(&remote)), None).await.unwrap_err().contains("no persona wallet"));

    castellan::custody::wallet::ensure_wallet_state(&custodian.data_root, first, "fixture").unwrap();
    // Answered while locked, and without the wallet root Djinn names.
    let locked = remote.wallet_public(custodian.dir.path()).unwrap();
    assert_eq!(locked.personas, vec![first]);
    assert_eq!(locked.roster, pandect::wallet_store::load_device_roster(&custodian.data_root).unwrap());
    assert!(locked.roster.is_some());
    assert_eq!(resolve_persona(index(Arc::clone(&remote)), None).await.unwrap(), first);
    assert_eq!(resolve_persona(index(Arc::clone(&remote)), Some(second)).await.unwrap(), second, "a named persona is never second-guessed");

    castellan::custody::wallet::ensure_wallet_state(&custodian.data_root, second, "fixture").unwrap();
    let several = resolve_persona(index(Arc::clone(&remote)), None).await.unwrap_err();
    assert!(several.contains("several personas") && several.contains(&first.as_uuid().to_string()) && several.contains(&second.as_uuid().to_string()), "{several}");
}

#[test]
fn a_custodian_without_a_grant_refuses_every_release() {
    let custodian = TestCustodian::start(PASSPHRASE, &[("work", WORK)], manage_grants());
    let remote = unlocked(&custodian);
    let salts = distillery::InstalledAuthority::release_salts();
    assert!(matches!(remote.release(KeySource::Persona, None, &salts), Err(CustodyRefusal::NotReleasable)));
    assert!(matches!(remote.sign(KeySource::Persona, None, b"holder/x", b"m"), Err(CustodyRefusal::NotReleasable)));
    // With a grant, a derived signature is made where the key is.
    let custodian = TestCustodian::start(PASSPHRASE, &[("work", WORK)], grants_with(json!([{"prefix": "app/"}])));
    let remote = unlocked(&custodian);
    let (public_key, _) = remote.sign(KeySource::Persona, None, b"app/act", b"message").unwrap();
    assert_eq!(public_key, custodian.direct("work").derive_keypair(b"app/act").unwrap().public_key().to_bytes());
}

#[test]
fn the_custodians_door_rule_names_exactly_djinns_door_keys() {
    for master in [[0x11; 32], [0x5c; 32]] {
        let custodian = djinn_custodian::policy::door_salts(&master).to_vec();
        assert_eq!(custodian, djinn::door::local_session::door_salts(master));
    }
}

#[test]
fn real_custodian_fences_private_payload_to_authoritative_generation() {
    use eidetic::Hash;
    use graphshell_endpoint::door::custody_payload::{PayloadBytes, PayloadOperation, PayloadRequest, PayloadResult};
    let custodian = TestCustodian::start(PASSPHRASE, &[("work", WORK)], manage_grants());
    let persona = personae::PersonaId::new();
    castellan::custody::wallet::ensure_wallet_state(&custodian.data_root, persona, "isolated fixture").unwrap();
    let remote = unlocked(&custodian);
    let original = remote.custody_epoch().unwrap().expect("custodian exposes an epoch");
    let master = remote.master_public_key().to_bytes();
    assert_eq!(master, custodian.direct("work").master_public_key().to_bytes());
    let clear = b"PRIVATE-EPOCH-CANARY-381";
    let request = PayloadRequest {
        profile: ProfileId("work".into()), master, content_hash: Hash::of(clear),
        operation: PayloadOperation::Seal { bytes: PayloadBytes(clear.to_vec()) },
        custody_epoch: Some(original.clone()),
    };
    let scope = b"isolated-test-scope".to_vec();
    let PayloadResult::Sealed { bytes, marker } = remote.payload(custodian.dir.path(), persona, request.clone(), scope.clone()).expect("current custodian epoch seals") else { panic!("expected sealed payload") };
    assert_ne!(bytes.0, clear);
    remote.lock_vault().unwrap();
    remote.unlock_with_passphrase(PASSPHRASE.as_bytes()).unwrap();
    let fresh = remote.custody_epoch().unwrap().expect("fresh epoch");
    assert_ne!(fresh, original, "lock and unlock change authoritative epoch");
    assert!(remote.payload(custodian.dir.path(), persona, request, scope.clone()).is_err(), "an old binding cannot write after lock and unlock");
    let open = PayloadRequest {
        profile: ProfileId("work".into()), master, content_hash: Hash::of(clear),
        operation: PayloadOperation::Unseal { bytes, marker }, custody_epoch: Some(fresh),
    };
    assert!(matches!(remote.payload(custodian.dir.path(), persona, open, scope), Ok(PayloadResult::Clear { bytes }) if bytes.0 == clear));
}

#[tokio::test(flavor = "multi_thread")]
async fn real_custodian_generation_reaches_the_app_payload_route() {
    use djinn::door::app_broker::{AppEndpointCatalog, serve_app_broker};
    use eidetic::{Hash, PayloadSealer};
    use graphshell_endpoint::door::app_admission::{AllowedAppRoutes, AppRouteGrants};
    use graphshell_endpoint::door::custody_client::CustodyClient;
    use graphshell_endpoint::door::custody_identity::CustodyIdentity;
    use std::time::Duration;

    let custodian = TestCustodian::start(PASSPHRASE, &[("work", WORK)], manage_grants());
    let persona = personae::PersonaId::new();
    castellan::custody::wallet::ensure_wallet_state(&custodian.data_root, persona, "isolated route fixture").unwrap();
    let remote = unlocked(&custodian);
    let keeper = Arc::new(Keeper::new(remote.clone()));
    keeper.configure_payload_custody(ProfileId("work".into()), persona, custodian.data_root.clone()).unwrap();
    let grants = AppRouteGrants::new(AllowedAppRoutes::none());
    djinn::custody::grant(&grants).unwrap();
    let endpoint = custodian.dir.path().join("app.sock").to_string_lossy().to_string();
    let server = tokio::spawn({ let endpoint = endpoint.clone(); async move {
        serve_app_broker(&endpoint, keeper, grants, 60_000, None, AppEndpointCatalog::default()).await
    }});
    for _ in 0..100 {
        if CustodyClient::open_at(&endpoint, AppId::new("turnstone")).await.is_ok() { break; }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    let open = |endpoint: String| tokio::task::spawn_blocking(move || CustodyIdentity::connect_at_timeout(&endpoint, "turnstone", Duration::from_secs(3)));
    let old = open(endpoint.clone()).await.unwrap().expect("application custody connects");
    let epoch = old.connected_status().custody_epoch.clone().expect("authoritative generation forwarded");
    let clear = b"PRIVATE-BROKER-EPOCH-CANARY-417";
    let (bytes, marker) = old.seal(&Hash::of(clear), clear).expect("bound private seal through actual route");
    assert_ne!(bytes, clear);
    remote.lock_vault().unwrap();
    remote.unlock_with_passphrase(PASSPHRASE.as_bytes()).unwrap();
    assert!(old.unseal(&Hash::of(clear), &marker, &bytes).is_err(), "old application binding cannot survive a lock cycle");
    let fresh = open(endpoint).await.unwrap().expect("fresh application binding");
    assert_ne!(fresh.connected_status().custody_epoch.as_ref(), Some(&epoch));
    assert_eq!(fresh.unseal(&Hash::of(clear), &marker, &bytes).unwrap(), clear);
    drop(old);
    drop(fresh);
    server.abort();
}
