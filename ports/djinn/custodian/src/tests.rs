// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The H1 negative controls against the core, on real Castellan vaults.
//! Peers are constructed here; `driver_tests` runs the same contract over
//! real sockets with OS-derived peers.

use std::collections::BTreeSet;
use std::time::{Duration, Instant};

use castellan::custody::bootstrap::{self, Unlock};
use custody_wire::contract::{Answer, FactCause, Outcome, Purpose, ServerMessage};
use custody_wire::secret::{Bytes, Secret};
use custody_wire::{ClientMessage, Operation, Peer};
use insigne::delegation::{CapabilityScope, DelegationCertificate, DelegationParent, SignedDelegationCertificate, delegation_signing_salt};
use personae::{IdentityProvider, ProfileId};

use crate::core::{Core, Decider, Effects, SessionId};
use crate::policy::Config;

pub(crate) const PASSPHRASE: &str = "correct horse battery staple";
pub(crate) const DJINN_UID: u32 = 1000;

pub(crate) fn djinn() -> Peer {
    Peer::Unix {
        uid: DJINN_UID,
        gid: DJINN_UID,
        pid: Some(4242),
    }
}

fn consent_helper() -> Peer {
    Peer::Signed {
        team: "TEAM".into(),
        identifier: "com.merely.djinn.consent".into(),
    }
}

fn admin() -> Peer {
    Peer::Unix { uid: 0, gid: 0, pid: Some(1) }
}

/// A real passphrase vault with one profile, `work`.
pub(crate) fn vault() -> (tempfile::TempDir, [u8; 32]) {
    let dir = tempfile::tempdir().unwrap();
    let opened = bootstrap::open_storage(dir.path(), Unlock::passphrase(PASSPHRASE)).unwrap();
    let (profile, _) = bootstrap::load_or_create_profile(&*opened.storage, &ProfileId("work".into())).unwrap();
    (dir, profile.master.public_key().to_bytes())
}

pub(crate) fn config(dir: &std::path::Path, consent: &str, release_consent: &str) -> Config {
    let json = serde_json::json!({
        "vault": {"directory": dir},
        "owner": {"unix_uid": DJINN_UID},
        "consent": consent,
        "development": release_consent == "none",
        "principals": [
            {"name": "djinn", "match": {"unix-uid": DJINN_UID}, "grants": [
                {"operation": "status"}, {"operation": "lock"}, {"operation": "unlock"},
                {"operation": "release", "profile": "current", "salts": [{"exact": "mesh-author"}, {"exact": "distillery-personal-mesh"}], "consent": release_consent},
                {"operation": "sign", "profile": "current", "purpose": {"purpose": "delegation", "audience": "mere.graphshell"}, "consent": "per-operation"}
            ]},
            {"name": "consent", "match": {"signed": {"team": "TEAM", "identifier": "com.merely.djinn.consent"}}, "consent_channel": true},
            {"name": "admin", "match": {"unix-uid": 0}, "grants": [{"operation": "revoke", "consent": "none"}, {"operation": "status"}]}
        ]
    });
    Config::parse(&serde_json::to_vec(&json).unwrap()).unwrap()
}

pub(crate) struct Harness {
    pub core: Core,
    pub now: Instant,
    next_id: u64,
}

impl Harness {
    pub fn new(config: Config) -> Self {
        Self {
            core: Core::start(config).unwrap(),
            now: Instant::now(),
            next_id: 1,
        }
    }

    pub fn open(&mut self, peer: Peer) -> SessionId {
        let hello = ClientMessage::Hello {
            version: custody_wire::VERSION,
            nonce: "n".into(),
        };
        self.core.open(peer, &hello).map(|(id, _)| id).unwrap()
    }

    pub fn request(&mut self, session: SessionId, operation: Operation) -> (u64, Effects) {
        let id = self.next_id;
        self.next_id += 1;
        let message = ClientMessage::Request {
            id,
            instance: self.core.instance().to_string(),
            generation: self.core.generation(),
            operation,
        };
        (id, self.core.handle(session, message, self.now))
    }

    pub fn outcome(&mut self, session: SessionId, operation: Operation) -> Outcome {
        let (id, effects) = self.request(session, operation);
        outcome_for(&effects, session, id).expect("an immediate outcome")
    }

    pub fn unlock(&mut self, session: SessionId) {
        let outcome = self.outcome(session, Operation::Unlock {
            passphrase: Secret::copy_from(PASSPHRASE.as_bytes()),
        });
        assert!(matches!(outcome, Outcome::Success { answer: Answer::Unlocked { .. } }), "{outcome:?}");
    }
}

pub(crate) fn outcome_for(effects: &Effects, session: SessionId, request: u64) -> Option<Outcome> {
    effects.messages.iter().find_map(|(to, message)| match message {
        ServerMessage::Outcome { request: r, outcome, .. } if *to == session && *r == request => Some(outcome.clone()),
        _ => None,
    })
}

fn petition_in(effects: &Effects, to: SessionId) -> Option<custody_wire::Petition> {
    effects.messages.iter().find_map(|(id, message)| match message {
        ServerMessage::Petition(petition) if *id == to => Some(petition.clone()),
        _ => None,
    })
}

fn release(salts: &[&str]) -> Operation {
    Operation::Release {
        profile: "work".into(),
        salts: salts.iter().map(|s| Bytes(s.as_bytes().to_vec())).collect(),
    }
}

fn code(outcome: &Outcome) -> &str {
    outcome.code().unwrap_or("success")
}

fn certificate(master: [u8; 32], domain: &str) -> DelegationCertificate {
    DelegationCertificate::new(
        DelegationParent::Root([7; 32]),
        master,
        [9; 32],
        CapabilityScope {
            domain: domain.into(),
            resource: b"guest-room".to_vec(),
            path_prefix: "/".into(),
            actions: BTreeSet::from(["visit".to_string()]),
        },
        1_000,
        1_000,
        Some(2_000),
        0,
        [3; 32],
    )
}

#[test]
fn the_policy_refuses_unattended_same_user_grants_and_a_desktop_consent_channel() {
    let (dir, _) = vault();
    let base = |principals: serde_json::Value, development: bool| {
        serde_json::to_vec(&serde_json::json!({
            "vault": {"directory": dir.path()}, "owner": {"unix_uid": 1000}, "consent": "channel",
            "development": development, "principals": principals
        }))
        .unwrap()
    };
    let unattended = serde_json::json!([{"name": "djinn", "match": {"unix-uid": 1000}, "grants": [
        {"operation": "release", "profile": "current", "salts": [{"exact": "mesh-author"}], "consent": "none"}]}]);
    assert!(Config::parse(&base(unattended.clone(), false)).is_err());
    assert!(Config::parse(&base(unattended, true)).is_ok(), "development mode may, and says so");
    let desktop_consent = serde_json::json!([{"name": "consent", "match": {"unix-uid": 1000}, "consent_channel": true}]);
    assert!(Config::parse(&base(desktop_consent, true)).is_err());
    let root_unattended = serde_json::json!([{"name": "svc", "match": {"unix-uid": 77}, "grants": [
        {"operation": "sign", "profile": "current", "purpose": {"purpose": "root"}, "consent": "per-generation"}]}]);
    assert!(Config::parse(&base(root_unattended, false)).is_err());
}

#[test]
fn a_pinned_windows_image_can_be_the_consent_channel_but_only_with_a_status_grant() {
    let (dir, _) = vault();
    const SID: &str = "S-1-5-21-1-2-3-1001";
    const IMAGE: &str = r"C:\Program Files\Merely\djinn-custodian\djinn-custodian.exe";
    let parse = |principals: serde_json::Value| {
        Config::parse(&serde_json::to_vec(&serde_json::json!({
            "vault": {"directory": dir.path()}, "owner": {"windows_sid": SID}, "consent": "channel", "principals": principals
        }))
        .unwrap())
    };
    let channel = |image: &str, grants: serde_json::Value| serde_json::json!({
        "name": "consent", "match": {"windows-image": {"sid": SID, "image": image}}, "consent_channel": true, "grants": grants
    });
    let djinn = serde_json::json!({"name": "djinn", "match": {"windows-sid": SID}, "grants": [
        {"operation": "release", "profile": "current", "salts": [{"exact": "mesh-author"}], "consent": "per-generation"}]});
    let status = serde_json::json!([{"operation": "status"}]);

    assert!(parse(serde_json::json!([channel(IMAGE, status.clone()), djinn.clone()])).is_ok());
    assert!(parse(serde_json::json!([{"name": "consent", "match": {"windows-sid": SID}, "consent_channel": true}])).is_err(), "the account alone is never the channel");
    assert!(parse(serde_json::json!([channel(IMAGE, serde_json::json!([{"operation": "status"}, {"operation": "lock"}]))])).is_err(), "a pinned channel decides and does nothing else");
    for relative in ["djinn-custodian.exe", "", r"..\djinn-custodian.exe"] {
        assert!(parse(serde_json::json!([channel(relative, status.clone())])).is_err(), "{relative:?} is not an absolute image");
    }

    let config = parse(serde_json::json!([channel(IMAGE, status.clone()), djinn.clone()])).unwrap();
    let peer = |image: Option<&str>| Peer::Windows { sid: SID.into(), pid: 7, image: image.map(Into::into) };
    let name = |peer: &Peer| config.principal(peer).map(|p| (p.name, p.consent_channel, p.same_user));
    assert_eq!(name(&peer(Some(IMAGE))), Some(("consent".into(), true, true)));
    assert_eq!(name(&peer(Some(&format!(r"\\?\{}", IMAGE.to_uppercase().replace('\\', "/"))))), Some(("consent".into(), true, true)), "case, separator and the verbatim prefix do not matter");
    assert_eq!(name(&peer(Some(r"C:\Users\mark\djinn-custodian.exe"))), Some(("djinn".into(), false, true)), "another executable of the same user is only Djinn's requester");
    assert_eq!(name(&peer(None)), Some(("djinn".into(), false, true)), "a peer whose image could not be read is never the channel");
    let other_user = Peer::Windows { sid: "S-1-5-21-1-2-3-1002".into(), pid: 7, image: Some(IMAGE.into()) };
    assert_eq!(name(&other_user), None, "the pinned image of another account is nobody");
    // Rules are matched in order, which is why the installer lists the channel first.
    let reversed = parse(serde_json::json!([djinn, channel(IMAGE, status)])).unwrap();
    assert_eq!(reversed.principal(&peer(Some(IMAGE))).map(|p| p.name), Some("djinn".into()));
}

#[test]
fn the_principal_comes_from_the_os_peer_and_every_start_is_locked() {
    let (dir, _) = vault();
    let mut h = Harness::new(config(dir.path(), "channel", "per-generation"));
    let hello = ClientMessage::Hello {
        version: custody_wire::VERSION,
        nonce: "n".into(),
    };
    let stranger = Peer::Unix { uid: 4321, gid: 4321, pid: None };
    assert!(matches!(h.core.open(stranger, &hello), Err(ServerMessage::Refused { code }) if code == "principal-unknown"));
    let wrong_version = ClientMessage::Hello { version: 99, nonce: "n".into() };
    assert!(h.core.open(djinn(), &wrong_version).is_err());
    let session = h.open(djinn());
    assert!(h.core.status().locked);
    assert_eq!(code(&h.outcome(session, release(&["mesh-author"]))), "locked");
    let wrong = h.outcome(session, Operation::Unlock {
        passphrase: Secret::copy_from(b"wrong"),
    });
    assert_eq!(code(&wrong), "unlock-refused");
    assert!(h.core.status().locked);
    let admin = h.open(admin());
    let no_grant = h.outcome(admin, Operation::Unlock {
        passphrase: Secret::copy_from(PASSPHRASE.as_bytes()),
    });
    assert_eq!(code(&no_grant), "operation-denied");
}

#[test]
fn wrong_profile_and_adjacent_namespaces_are_refused() {
    let (dir, master) = vault();
    let mut h = Harness::new(config(dir.path(), "channel", "none"));
    let session = h.open(djinn());
    h.unlock(session);
    let other_profile = Operation::Release {
        profile: "home".into(),
        salts: vec![Bytes(b"mesh-author".to_vec())],
    };
    assert_eq!(code(&h.outcome(session, other_profile)), "operation-or-scope");
    assert_eq!(code(&h.outcome(session, release(&["mesh-author", "graphshell/endpoint-session/x"]))), "operation-or-scope");
    assert_eq!(code(&h.outcome(session, release(&[""]))), "operation-or-scope");

    let statement = certificate(master, "mere.denizen");
    let adjacent = Operation::Sign {
        profile: "work".into(),
        purpose: Purpose::Delegation { audience: "mere.denizen".into() },
        salt: Bytes(delegation_signing_salt(&statement.scope)),
        message: Bytes(statement.signing_bytes()),
    };
    assert_eq!(code(&h.outcome(session, adjacent)), "operation-or-scope");
    let mislabelled = Operation::Sign {
        profile: "work".into(),
        purpose: Purpose::Delegation { audience: "mere.graphshell".into() },
        salt: Bytes(delegation_signing_salt(&statement.scope)),
        message: Bytes(statement.signing_bytes()),
    };
    assert_eq!(code(&h.outcome(session, mislabelled)), "audience-denied");
    let oracle = Operation::Sign {
        profile: "work".into(),
        purpose: Purpose::Delegation { audience: "mere.graphshell".into() },
        salt: Bytes(b"anything".to_vec()),
        message: Bytes(b"arbitrary bytes".to_vec()),
    };
    assert_eq!(code(&h.outcome(session, oracle)), "not-a-delegation");
    let root = Operation::Sign {
        profile: "work".into(),
        purpose: Purpose::Root {},
        salt: Bytes(Vec::new()),
        message: Bytes(b"x".to_vec()),
    };
    assert_eq!(code(&h.outcome(session, root)), "operation-or-scope");
}

#[test]
fn released_keys_are_exactly_the_in_process_derivations_and_rebuild_as_retained_keys() {
    let (dir, master) = vault();
    let mut h = Harness::new(config(dir.path(), "channel", "none"));
    let session = h.open(djinn());
    h.unlock(session);
    let Outcome::Success { answer: Answer::Released { master_public_key, keys } } = h.outcome(session, release(&["mesh-author", "distillery-personal-mesh"])) else {
        panic!("release refused")
    };
    assert_eq!(master_public_key, master);
    let opened = bootstrap::open_storage(dir.path(), Unlock::passphrase(PASSPHRASE)).unwrap();
    let direct = castellan::custody::IdentityVault::open(opened.storage, &ProfileId("work".into())).unwrap();
    let mut rebuilt = Vec::new();
    for key in keys {
        assert_eq!(key.seed.expose(), direct.derive_keypair(&key.salt.0).unwrap().to_seed());
        let attestation = insigne::DerivedKeyAttestation::from_parts(key.attestation.master, key.attestation.derived, key.attestation.signature.0.clone());
        let seed: [u8; 32] = key.seed.expose().try_into().unwrap();
        rebuilt.push((key.salt.0.clone(), personae::Ed25519Keypair::from_seed(seed), attestation));
    }
    let retained = personae::RetainedKeys::from_released(personae::Ed25519PublicKey::from_bytes(&master).unwrap(), rebuilt).unwrap();
    assert!(retained.derive_keypair(b"mesh-author").is_ok());
    assert!(retained.derive_keypair(b"anything-else").is_err(), "nothing but the release");
}

#[test]
fn a_delegation_is_signed_inside_custody_only_after_trusted_consent_bound_to_its_digest() {
    let (dir, master) = vault();
    let mut h = Harness::new(config(dir.path(), "channel", "per-generation"));
    let session = h.open(djinn());
    let helper = h.open(consent_helper());
    h.unlock(session);
    let statement = certificate(master, "mere.graphshell");
    let sign = Operation::Sign {
        profile: "work".into(),
        purpose: Purpose::Delegation { audience: "mere.graphshell".into() },
        salt: Bytes(delegation_signing_salt(&statement.scope)),
        message: Bytes(statement.signing_bytes()),
    };
    let (id, effects) = h.request(session, sign);
    assert!(outcome_for(&effects, session, id).is_none(), "waits for consent");
    let petition = petition_in(&effects, helper).expect("the consent channel is petitioned");
    assert!(petition_in(&effects, session).is_none(), "the requester is never asked to approve itself");
    assert_eq!(petition.principal, "djinn");
    assert_eq!(petition.namespace, "delegation:mere.graphshell");

    // Djinn cannot decide its own petition; bindings must match exactly.
    let mine = h.core.handle(session, ClientMessage::Decide { request: id, digest: petition.digest.clone(), generation: petition.generation, accept: true }, h.now);
    assert!(matches!(&mine.messages[..], [(_, ServerMessage::Decided { recorded: false, code: Some(c), .. })] if c == "not-consent-channel"));
    for (request, digest, generation) in [(id + 1, petition.digest.clone(), petition.generation), (id, "0".repeat(64), petition.generation), (id, petition.digest.clone(), petition.generation + 1)] {
        let (recorded, _) = h.core.decide(Decider::Session(helper), request, &digest, generation, true, h.now);
        assert!(recorded.is_err());
    }
    let (recorded, effects) = h.core.decide(Decider::Session(helper), id, &petition.digest, petition.generation, true, h.now);
    recorded.unwrap();
    let Some(Outcome::Success { answer: Answer::Signature { signature, attestation: Some(attestation), .. } }) = outcome_for(&effects, session, id) else {
        panic!("no signature")
    };
    let signer = insigne::DerivedKeyAttestation::from_parts(attestation.master, attestation.derived, attestation.signature.0);
    let signed = SignedDelegationCertificate::from_parts(statement, signer, signature.to_vec());
    signed.check().expect("a valid delegation under the profile's scope key");
    let (again, _) = h.core.decide(Decider::Session(helper), id, &petition.digest, petition.generation, true, h.now);
    assert!(again.is_err(), "one decision per petition");
}

#[test]
fn per_generation_consent_lasts_until_the_next_lock() {
    let (dir, _) = vault();
    let mut h = Harness::new(config(dir.path(), "channel", "per-generation"));
    let session = h.open(djinn());
    let helper = h.open(consent_helper());
    h.unlock(session);
    let (id, effects) = h.request(session, release(&["mesh-author"]));
    let petition = petition_in(&effects, helper).unwrap();
    let (_, effects) = h.core.decide(Decider::Session(helper), id, &petition.digest, petition.generation, true, h.now);
    assert!(matches!(outcome_for(&effects, session, id), Some(Outcome::Success { .. })));
    assert!(matches!(h.outcome(session, release(&["distillery-personal-mesh"])), Outcome::Success { .. }), "same grant, same generation");
    assert!(matches!(h.outcome(session, Operation::Lock {}), Outcome::Success { .. }));
    h.unlock(session);
    let (id, effects) = h.request(session, release(&["mesh-author"]));
    assert!(outcome_for(&effects, session, id).is_none() && petition_in(&effects, helper).is_some(), "a new generation asks again");
}

#[test]
fn a_lock_during_a_pending_operation_invalidates_it_and_reports_three_facts() {
    let (dir, master) = vault();
    let mut h = Harness::new(config(dir.path(), "channel", "per-generation"));
    let session = h.open(djinn());
    let helper = h.open(consent_helper());
    h.unlock(session);
    let statement = certificate(master, "mere.graphshell");
    let (id, effects) = h.request(session, Operation::Sign {
        profile: "work".into(),
        purpose: Purpose::Delegation { audience: "mere.graphshell".into() },
        salt: Bytes(delegation_signing_salt(&statement.scope)),
        message: Bytes(statement.signing_bytes()),
    });
    let petition = petition_in(&effects, helper).unwrap();
    let before = h.core.generation();
    let (lock_id, effects) = h.request(session, Operation::Lock {});
    assert_eq!(code(&outcome_for(&effects, session, id).unwrap()), "invalidated");
    let Some(Outcome::Success { answer: Answer::Lock { report } }) = outcome_for(&effects, session, lock_id) else {
        panic!("lock refused")
    };
    assert!(report.memory_locked && report.drained && report.durable);
    assert_eq!(report.generation, before + 1);
    let facts: Vec<_> = effects.messages.iter().filter_map(|(to, m)| match m {
        ServerMessage::Fact(f) => Some((*to, f.clone())),
        _ => None,
    }).collect();
    assert!(facts.iter().any(|(to, f)| *to == helper && f.cause == FactCause::Lock && f.locked && f.generation == before + 1));
    let (late, _) = h.core.decide(Decider::Session(helper), id, &petition.digest, petition.generation, true, h.now);
    assert!(late.is_err(), "a decision cannot revive an invalidated petition");
}

#[test]
fn a_lock_whose_barrier_fails_is_degraded_but_memory_still_locks() {
    let (dir, _) = vault();
    let mut h = Harness::new(config(dir.path(), "channel", "none"));
    let session = h.open(djinn());
    h.unlock(session);
    std::fs::create_dir(dir.path().join(castellan::custody::PERSISTED_LOCK_FILE)).unwrap();
    std::fs::write(dir.path().join(castellan::custody::PERSISTED_LOCK_FILE).join("x"), b"x").unwrap();
    let outcome = h.outcome(session, Operation::Lock {});
    let Outcome::Degraded { code: degraded, answer: Some(Answer::Lock { report }) } = outcome else {
        panic!("expected a degraded lock, got {outcome:?}")
    };
    assert_eq!(degraded, "restart-barrier-not-durable");
    assert!(report.memory_locked && !report.durable);
    assert!(h.core.status().locked);
    assert_eq!(code(&h.outcome(session, release(&["mesh-author"]))), "locked");
}

#[test]
fn stale_instance_and_generation_are_refused() {
    let (dir, _) = vault();
    let mut h = Harness::new(config(dir.path(), "channel", "none"));
    let session = h.open(djinn());
    h.unlock(session);
    let old = h.core.generation();
    assert!(matches!(h.outcome(session, Operation::Lock {}), Outcome::Success { .. }));
    h.unlock(session);
    let stale = ClientMessage::Request { id: 90, instance: h.core.instance().into(), generation: old, operation: release(&["mesh-author"]) };
    let effects = h.core.handle(session, stale, h.now);
    assert_eq!(code(&outcome_for(&effects, session, 90).unwrap()), "stale-generation");
    let other = ClientMessage::Request { id: 91, instance: "an-earlier-custodian".into(), generation: h.core.generation(), operation: Operation::Status {} };
    let effects = h.core.handle(session, other, h.now);
    assert_eq!(code(&outcome_for(&effects, session, 91).unwrap()), "stale-instance");
}

#[test]
fn revocation_advances_the_generation_closes_sessions_and_refuses_readmission() {
    let (dir, _) = vault();
    let mut h = Harness::new(config(dir.path(), "channel", "none"));
    let session = h.open(djinn());
    let admin = h.open(admin());
    h.unlock(session);
    let before = h.core.generation();
    let (id, effects) = h.request(admin, Operation::Revoke { principal: "djinn".into() });
    assert!(matches!(outcome_for(&effects, admin, id), Some(Outcome::Success { answer: Answer::Revoked { generation } }) if generation == before + 1));
    assert!(effects.close.contains(&session));
    assert!(effects.messages.iter().any(|(to, m)| *to == session && matches!(m, ServerMessage::Fact(f) if f.cause == FactCause::Revoke && f.revoked.as_deref() == Some("djinn"))));
    h.core.close(session);
    let hello = ClientMessage::Hello { version: custody_wire::VERSION, nonce: "n".into() };
    assert!(matches!(h.core.open(djinn(), &hello), Err(ServerMessage::Refused { code }) if code == "revoked"));
}

#[test]
fn petitions_expire_and_withdraw_with_their_session() {
    let (dir, _) = vault();
    let mut h = Harness::new(config(dir.path(), "channel", "per-generation"));
    let session = h.open(djinn());
    let helper = h.open(consent_helper());
    h.unlock(session);
    let (id, effects) = h.request(session, release(&["mesh-author"]));
    assert!(petition_in(&effects, helper).is_some());
    let effects = h.core.tick(h.now + custody_wire::limits::CONSENT + Duration::from_secs(1));
    assert_eq!(code(&outcome_for(&effects, session, id).unwrap()), "consent-expired");

    let (_, effects) = h.request(session, release(&["mesh-author"]));
    let petition = petition_in(&effects, helper).unwrap();
    h.core.close(session);
    let (recorded, _) = h.core.decide(Decider::Session(helper), petition.request, &petition.digest, petition.generation, true, h.now);
    assert!(recorded.is_err(), "a withdrawn petition cannot be approved");
}

#[test]
fn without_a_consent_channel_petitions_are_refused_and_connections_are_bounded() {
    let (dir, _) = vault();
    let mut h = Harness::new(config(dir.path(), "channel", "per-generation"));
    let session = h.open(djinn());
    h.unlock(session);
    assert_eq!(code(&h.outcome(session, release(&["mesh-author"]))), "consent-unavailable");
    let _second = h.open(djinn());
    let hello = ClientMessage::Hello { version: custody_wire::VERSION, nonce: "n".into() };
    assert!(matches!(h.core.open(djinn(), &hello), Err(ServerMessage::Refused { code }) if code == "principal-connection-capacity"));
}

#[test]
fn an_absent_vault_is_never_created_and_a_dpapi_vault_needs_migration() {
    let empty = tempfile::tempdir().unwrap();
    let mut json = serde_json::to_value(serde_json::json!({
        "vault": {"directory": empty.path()}, "owner": {"unix_uid": 1}, "consent": "deny", "principals": []
    })).unwrap();
    let config = Config::parse(&serde_json::to_vec(&json).unwrap()).unwrap();
    assert_eq!(Core::start(config).err(), Some(crate::Refusal("vault-missing")));
    assert!(!empty.path().join(bootstrap::PASSPHRASE_VAULT_FILE).exists());
    std::fs::write(empty.path().join(castellan::custody::AUTO_UNLOCK_ROOT_FILE), b"{}").unwrap();
    json["vault"]["directory"] = serde_json::json!(empty.path());
    let config = Config::parse(&serde_json::to_vec(&json).unwrap()).unwrap();
    assert_eq!(Core::start(config).err(), Some(crate::Refusal("os-sealed-vault-needs-migration")));
}

mod driver_tests;
mod h2_tests;
