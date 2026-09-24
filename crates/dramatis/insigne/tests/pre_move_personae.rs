// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Proofs issued by personae before the 2026-09-24 move still load and check.
//!
//! The fixture was produced by personae at mere `3943874f`, before its
//! delegation and attestation types moved here: a certificate, a revocation
//! and an attestation from fixed seeds, with the certificate's id and the
//! result of each check. If a signing byte had moved, these would fail.

use insigne::{DerivedKeyAttestation, SignedDelegationCertificate, SignedDelegationRevocation};
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

#[test]
fn a_pre_move_certificate_loads_checks_and_keeps_its_id() {
    let fixture = fixture();
    assert_eq!(
        fixture["certificate_verifies"], true,
        "the fixture's own control"
    );
    let certificate: SignedDelegationCertificate = round_trips(&fixture["certificate"]);
    assert!(certificate.verify());
    let id: String = certificate
        .certificate
        .id()
        .0
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    assert_eq!(id, fixture["certificate_id"].as_str().unwrap());

    let mut tampered = certificate;
    tampered.certificate.scope.path_prefix = "moot/secret".into();
    assert!(!tampered.verify());
}

#[test]
fn a_pre_move_revocation_loads_and_checks() {
    let fixture = fixture();
    assert_eq!(
        fixture["revocation_verifies"], true,
        "the fixture's own control"
    );
    let revocation: SignedDelegationRevocation = round_trips(&fixture["revocation"]);
    assert!(revocation.verify());

    let mut tampered = revocation;
    tampered.revocation.at_ms += 1;
    assert!(!tampered.verify());
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
    assert!(attestation.verify(salt));
    assert!(!attestation.verify(b"another salt"));

    let mut derived = *attestation.derived();
    derived[0] ^= 1;
    let tampered = DerivedKeyAttestation::from_parts(
        *attestation.master(),
        derived,
        attestation.signature().to_vec(),
    );
    assert!(!tampered.verify(salt));
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
    assert!(
        attestation.verify(b"any salt"),
        "insigne's check is lax, as personae's was"
    );
}
