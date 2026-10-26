// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Custody isolation H1 from Djinn's side: its two consumers reach a real
//! custodian over a socket, and nothing falls back to Djinn's own vault when
//! the custodian refuses, is locked or is gone. Djinn's own host here holds
//! a different identity than the custodian's vault, so a signature or key
//! that checks against the custodian's vault came from the custodian.

#![cfg(unix)]

use std::collections::BTreeSet;
use std::sync::Arc;

use castellan::authority::PersonaeHost;
use castellan::custody::bootstrap::{self, Unlock};
use castellan::custody::{IdentityVault, InMemoryStorage, Profile};
use dramatis::view::VaultProtectionView;
use djinn::custodian::{CustodianLink, InstalledCustodian};
use djinn::door::resident_identity::ResidentIdentity;
use djinn::keeper::Keeper;
use graphshell_endpoint::door::app_admission::AppId;
use graphshell_endpoint::door::custody::{CustodyAnswer, CustodyCall, CustodyRefusal, KeySource};
use insigne::delegation::{CapabilityScope, DelegationCertificate, DelegationParent, delegation_signing_salt};
use personae::{Ed25519Keypair, IdentityProvider, ProfileId};

const PASSPHRASE: &str = "h1 custodian passphrase";

struct Custodian {
    _dir: tempfile::TempDir,
    vault: std::path::PathBuf,
    client_file: std::path::PathBuf,
    master: [u8; 32],
}

fn custodian() -> Custodian {
    let dir = tempfile::tempdir().unwrap();
    let vault = dir.path().join("vault");
    let master = {
        let opened = bootstrap::open_storage(&vault, Unlock::passphrase(PASSPHRASE)).unwrap();
        let (profile, _) = bootstrap::load_or_create_profile(&*opened.storage, &ProfileId("work".into())).unwrap();
        profile.master.public_key().to_bytes()
    };
    let uid = custody_wire::peer::effective_uid();
    let config = serde_json::json!({
        "vault": {"directory": vault}, "owner": {"unix_uid": uid}, "consent": "deny", "development": true,
        "principals": [{"name": "djinn", "match": {"unix-uid": uid}, "grants": [
            {"operation": "status"}, {"operation": "lock"}, {"operation": "unlock"},
            {"operation": "release", "profile": "current", "salts": [{"exact": "mesh-author"}, {"exact": "distillery-personal-mesh"}], "consent": "none"},
            {"operation": "sign", "profile": "current", "purpose": {"purpose": "delegation", "audience": "mere.graphshell"}, "consent": "none"}
        ]}]
    });
    let config = djinn_custodian::policy::Config::parse(&serde_json::to_vec(&config).unwrap()).unwrap();
    let run = dir.path().join("run");
    std::fs::create_dir(&run).unwrap();
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(&run, std::fs::Permissions::from_mode(0o700)).unwrap();
    let socket = run.join("custody.sock");
    let listener = custody_wire::transport::UnixSocketListener::bind(&socket).unwrap();
    let service = djinn_custodian::driver::Service::new(djinn_custodian::core::Core::start(config).unwrap(), None);
    std::thread::spawn(move || service.serve(&listener));
    let client_file = dir.path().join("client.json");
    std::fs::write(
        &client_file,
        serde_json::to_vec(&serde_json::json!({"endpoint": socket, "server": {"unix_uids": [uid]}})).unwrap(),
    )
    .unwrap();
    Custodian {
        _dir: dir,
        vault,
        client_file,
        master,
    }
}

fn link(custodian: &Custodian) -> Arc<CustodianLink> {
    Arc::new(CustodianLink::new(InstalledCustodian::from_path(&custodian.client_file, false).unwrap()))
}

/// Djinn's own host: profile `work`, but a different master than the
/// custodian's vault.
fn djinn_keeper(link: Option<Arc<CustodianLink>>) -> Arc<Keeper<InMemoryStorage>> {
    let profile = Profile::new(ProfileId("work".into()), "Work", Ed25519Keypair::from_seed([0x5a; 32]));
    let host = Arc::new(PersonaeHost::new(
        IdentityVault::with_profile(InMemoryStorage::new(), profile),
        None,
        VaultProtectionView::Ephemeral,
    ));
    let keeper = Keeper::new(host);
    if let Some(link) = link {
        keeper.attach_custodian(link).unwrap();
    }
    Arc::new(keeper)
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
    let link = link(&custodian);
    link.unlock(PASSPHRASE.as_bytes()).unwrap();
    let salts = distillery::InstalledAuthority::release_salts();
    let released = link.release("work", &salts).unwrap();
    let opened = bootstrap::open_storage(&custodian.vault, Unlock::passphrase(PASSPHRASE)).unwrap();
    let direct = IdentityVault::open(opened.storage, &ProfileId("work".into())).unwrap();
    assert_eq!(released.master_public_key().to_bytes(), custodian.master);
    for salt in &salts {
        assert_eq!(released.derive_keypair(salt).unwrap().to_seed(), direct.derive_keypair(salt).unwrap().to_seed());
    }
    assert!(released.derive_keypair(b"mesh-author-adjacent").is_err(), "only what was released");
    assert!(matches!(link.release("work", &[b"graphshell/endpoint-session/x".to_vec()]), Err(CustodyRefusal::Refused { .. })));
    assert!(matches!(link.release("home", &salts), Err(CustodyRefusal::Refused { .. })), "a wrong profile is refused");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_graphshell_delegation_is_signed_by_the_custodian_not_by_djinn() {
    let custodian = custodian();
    let link = link(&custodian);
    link.unlock(PASSPHRASE.as_bytes()).unwrap();
    let keeper = djinn_keeper(Some(Arc::clone(&link)));
    let statement = delegation(custodian.master);
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
    let opened = bootstrap::open_storage(&custodian.vault, Unlock::passphrase(PASSPHRASE)).unwrap();
    let direct = IdentityVault::open(opened.storage, &ProfileId("work".into())).unwrap();
    assert_eq!(public_key, direct.derive_keypair(&salt).unwrap().public_key().to_bytes(), "the custodian's scope key, not Djinn's");
    let signature: [u8; 64] = signature.try_into().unwrap();
    assert!(personae::Ed25519PublicKey::from_bytes(&public_key).unwrap().verify(&statement.signing_bytes(), &personae::Ed25519Signature::from_bytes(&signature)));

    // A statement Djinn's own identity issued is not the custodian's to sign.
    let foreign = delegation(keeper.host().master_public_key().to_bytes());
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
    let link = link(&custodian);
    let keeper = djinn_keeper(Some(Arc::clone(&link)));
    let statement = delegation(custodian.master);
    let call = || CustodyCall::Sign {
        source: KeySource::Persona,
        salt: delegation_signing_salt(&statement.scope),
        message: statement.signing_bytes(),
    };
    // Djinn's own host is unlocked; the custodian is not. Locked wins.
    assert!(!keeper.is_locked());
    assert!(matches!(keeper.clone().custody(AppId::new("graphshell"), call()).await, Err(CustodyRefusal::Locked)));

    let gone = tempfile::tempdir().unwrap();
    let file = gone.path().join("client.json");
    std::fs::write(&file, br#"{"endpoint": "/nonexistent/custody.sock", "server": {"unix_uids": [0]}}"#).unwrap();
    let orphan = djinn_keeper(Some(Arc::new(CustodianLink::new(InstalledCustodian::from_path(&file, false).unwrap()))));
    let refused = orphan.clone().custody(AppId::new("graphshell"), call()).await;
    assert!(matches!(refused, Err(CustodyRefusal::Failed { reason }) if reason.contains("unavailable")), "never signed in-process");
}

#[test]
fn a_client_file_the_desktop_user_could_write_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("client.json");
    std::fs::write(&file, br#"{"endpoint": "/run/djinn-custodian/x.sock", "server": {"unix_uids": [0]}}"#).unwrap();
    let refused = InstalledCustodian::from_path(&file, true);
    assert!(refused.is_err_and(|e| e.contains("root-owned")));
}
