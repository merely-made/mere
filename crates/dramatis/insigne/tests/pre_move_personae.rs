// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Proofs issued by personae before the 2026-09-24 move still load and check,
//! and each step of a check reports its own fault.
//!
//! The fixture was produced by personae at mere `3943874f`, before its
//! delegation and attestation types moved here: a certificate, a revocation
//! and an attestation from fixed seeds, with the certificate's id and the
//! result of each check. If a signing byte had moved, these would fail. The
//! fault tests take the same real statements and break one thing at a time.

use insigne::{
    CheckFault, DerivedKeyAttestation, SignedDelegationCertificate, SignedDelegationRevocation,
};
use serde_json::Value;

const FIXTURE: &str = include_str!("fixtures/pre_move_personae.json");

fn fixture() -> Value {
    serde_json::from_str(FIXTURE).unwrap()
}

fn round_trips<T: serde::Serialize + serde::de::DeserializeOwned>(value: &Value) -> T {
    let parsed: T = serde_json::from_value(value.clone()).unwrap();
    assert_eq!(
        &serde_json::to_value(&parsed).unwrap(),
        value,
        "re-serializing must reproduce the stored bytes"
    );
    parsed
}

fn certificate() -> SignedDelegationCertificate {
    serde_json::from_value(fixture()["certificate"].clone()).unwrap()
}

fn revocation() -> SignedDelegationRevocation {
    serde_json::from_value(fixture()["revocation"].clone()).unwrap()
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[test]
fn a_pre_move_certificate_loads_checks_and_keeps_its_id() {
    let fixture = fixture();
    assert_eq!(
        fixture["certificate_verifies"], true,
        "the fixture's own control"
    );
    let signed: SignedDelegationCertificate = round_trips(&fixture["certificate"]);
    let checked = signed.check().expect("a pre-move certificate checks");
    assert_eq!(
        hex(&checked.id().0),
        fixture["certificate_id"].as_str().unwrap()
    );
    assert_eq!(checked.certificate(), &signed.certificate);
    assert_eq!(checked.signer().master(), &signed.certificate.issuer);

    let mut tampered = signed.clone();
    tampered.certificate.scope.path_prefix = "moot/secret".into();
    assert_eq!(tampered.check(), Err(CheckFault::BadSignature));
}

#[test]
fn a_pre_move_revocation_loads_and_checks() {
    let fixture = fixture();
    assert_eq!(
        fixture["revocation_verifies"], true,
        "the fixture's own control"
    );
    let signed: SignedDelegationRevocation = round_trips(&fixture["revocation"]);
    let checked = signed.check().expect("a pre-move revocation checks");
    assert_eq!(checked.revocation(), &signed.revocation);

    let mut tampered = signed.clone();
    tampered.revocation.at_ms += 1;
    assert_eq!(tampered.check(), Err(CheckFault::BadSignature));
}

#[test]
fn a_pre_move_attestation_loads_and_checks_under_its_salt_only() {
    let fixture = fixture();
    assert_eq!(
        fixture["attestation_verifies"], true,
        "the fixture's own control"
    );
    let attestation: DerivedKeyAttestation = round_trips(&fixture["attestation"]);
    let salt = fixture["attestation_salt"].as_str().unwrap().as_bytes();
    let checked = attestation.check(salt).expect("checks under its own salt");
    assert_eq!(checked.derived(), attestation.derived());
    assert_eq!(
        attestation.check(b"another salt"),
        Err(CheckFault::BadSignature)
    );

    let mut derived = *attestation.derived();
    derived[0] ^= 1;
    let tampered = DerivedKeyAttestation::from_parts(
        *attestation.master(),
        derived,
        attestation.signature().to_vec(),
    );
    assert_eq!(tampered.check(salt), Err(CheckFault::BadSignature));
}

#[test]
fn each_step_of_a_certificate_check_reports_its_own_fault() {
    let mut malformed = certificate();
    malformed.certificate.subject = [0; 32];
    assert_eq!(malformed.check(), Err(CheckFault::Malformed));

    // The scope's domain and resource pick the signing key's salt, so a
    // different resource leaves the signer's attestation unchecked.
    let mut elsewhere = certificate();
    elsewhere.certificate.scope.resource.push(0);
    assert_eq!(elsewhere.check(), Err(CheckFault::BadAttestation));

    let mut claimed = certificate();
    claimed.certificate.issuer = [7; 32];
    assert_eq!(claimed.check(), Err(CheckFault::WrongIssuer));

    let mut altered = certificate();
    altered.certificate.nonce[0] ^= 1;
    assert_eq!(altered.check(), Err(CheckFault::BadSignature));
}

#[test]
fn each_step_of_a_revocation_check_reports_its_own_fault() {
    let mut malformed = revocation();
    malformed.revocation.scope.actions.clear();
    assert_eq!(malformed.check(), Err(CheckFault::Malformed));

    let mut elsewhere = revocation();
    elsewhere.revocation.scope.domain.push('x');
    assert_eq!(elsewhere.check(), Err(CheckFault::BadAttestation));

    let mut claimed = revocation();
    claimed.revocation.issuer = [7; 32];
    assert_eq!(claimed.check(), Err(CheckFault::WrongIssuer));

    let mut altered = revocation();
    altered.revocation.nonce[0] ^= 1;
    assert_eq!(altered.check(), Err(CheckFault::BadSignature));
}

#[test]
fn an_attestation_of_another_format_version_is_malformed() {
    let mut value = fixture()["attestation"].clone();
    value["format_version"] = 2.into();
    let attestation: DerivedKeyAttestation = serde_json::from_value(value).unwrap();
    let salt = fixture()["attestation_salt"].as_str().unwrap().to_owned();
    assert_eq!(
        attestation.check(salt.as_bytes()),
        Err(CheckFault::Malformed)
    );
}

/// personae's checks used ed25519-dalek's lax `verify`, which accepts a
/// small-order key that `verify_strict` refuses. This vector separates the two
/// modes: the identity point as the key and as `R`, with `s = 0`, satisfies the
/// lax equation for any message. insigne must accept it too, or the move would
/// have changed what verifies.
#[test]
fn checks_keep_the_lax_mode_personae_used() {
    use ed25519_dalek::{Signature, Verifier, VerifyingKey};

    let mut identity = [0u8; 32];
    identity[0] = 1;
    let mut signature = [0u8; 64];
    signature[0] = 1;
    let key = VerifyingKey::from_bytes(&identity).unwrap();
    let signature = Signature::from_bytes(&signature);
    assert!(
        key.verify(b"any message", &signature).is_ok(),
        "lax accepts"
    );
    assert!(
        key.verify_strict(b"any message", &signature).is_err(),
        "strict refuses: the vector separates the modes"
    );

    let attestation =
        DerivedKeyAttestation::from_parts(identity, identity, signature.to_bytes().to_vec());
    attestation
        .check(b"any salt")
        .expect("insigne's check is lax, as personae's was");
}
