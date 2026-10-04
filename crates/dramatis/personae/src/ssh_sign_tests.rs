// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Every signature is checked by `ssh-key`'s public verification, which
//! for RSA is the `rsa` crate's public operation: independent of ring.

use super::*;
use signature::Verifier;
use ssh_key::EcdsaCurve;
use ssh_key::private::PrivateKey;
use ssh_key::public::PublicKey;

const MESSAGE: &[u8] = b"personae p4a session blob";

/// Fixtures are test-only keys made by `ssh-keygen` (OpenSSH for Windows
/// 9.5p2), unencrypted, comment `personae-p4a-fixture-<name>`.
pub(crate) fn fixture(name: &str) -> PrivateKey {
    let text = match name {
        "ed25519" => include_str!("../tests/fixtures/ssh/ed25519"),
        "rsa1024" => include_str!("../tests/fixtures/ssh/rsa1024"),
        "rsa2048" => include_str!("../tests/fixtures/ssh/rsa2048"),
        "rsa3072" => include_str!("../tests/fixtures/ssh/rsa3072"),
        "rsa4096" => include_str!("../tests/fixtures/ssh/rsa4096"),
        "ecdsa256" => include_str!("../tests/fixtures/ssh/ecdsa256"),
        "ecdsa384" => include_str!("../tests/fixtures/ssh/ecdsa384"),
        "ecdsa521" => include_str!("../tests/fixtures/ssh/ecdsa521"),
        other => panic!("no fixture {other}"),
    };
    PrivateKey::from_openssh(text).expect("fixture parses")
}

fn verifies(key: &PrivateKey, data: &[u8], signature: &Signature) -> bool {
    PublicKey::from(key)
        .key_data()
        .verify(data, signature)
        .is_ok()
}

#[test]
fn rsa_signs_sha2_256_and_512_per_flag_and_verifies_independently() {
    for (name, modulus_bytes) in [("rsa2048", 256), ("rsa3072", 384), ("rsa4096", 512)] {
        let key = fixture(name);
        for (flags, hash) in [
            (SSH_AGENT_RSA_SHA2_256, HashAlg::Sha256),
            (SSH_AGENT_RSA_SHA2_512, HashAlg::Sha512),
        ] {
            let signature = sign(key.key_data(), MESSAGE, flags).unwrap();
            assert_eq!(
                signature.algorithm(),
                Algorithm::Rsa { hash: Some(hash) },
                "{name}"
            );
            assert_eq!(signature.as_bytes().len(), modulus_bytes, "{name}");
            assert!(verifies(&key, MESSAGE, &signature), "{name} {hash:?}");
            assert!(
                !verifies(&key, b"other data", &signature),
                "{name}: verifier must refuse"
            );
        }
    }
}

#[test]
fn rsa_signature_claims_only_the_hash_it_used() {
    let key = fixture("rsa3072");
    let sha256 = sign(key.key_data(), MESSAGE, SSH_AGENT_RSA_SHA2_256).unwrap();
    let relabelled = Signature::new(
        Algorithm::Rsa {
            hash: Some(HashAlg::Sha512),
        },
        sha256.as_bytes(),
    )
    .unwrap();
    assert!(!verifies(&key, MESSAGE, &relabelled));
}

#[test]
fn rsa_with_both_flags_signs_sha2_256_as_openssh_does() {
    let key = fixture("rsa2048");
    let both = SSH_AGENT_RSA_SHA2_256 | SSH_AGENT_RSA_SHA2_512;
    let signature = sign(key.key_data(), MESSAGE, both).unwrap();
    assert_eq!(
        signature.algorithm(),
        Algorithm::Rsa {
            hash: Some(HashAlg::Sha256)
        }
    );
}

/// The `rsa` crate's signer (ssh-key's `RsaKeypair::try_sign`) answers a
/// flagless request with SHA-512, so this also fails if RSA ever routes
/// back through it.
#[test]
fn rsa_without_a_sha2_flag_is_refused() {
    let key = fixture("rsa2048");
    for flags in [0, 0x01, 0x08] {
        assert!(matches!(
            sign(key.key_data(), MESSAGE, flags),
            Err(SshSignError::RsaSha1Refused)
        ));
    }
}

#[test]
fn rsa_outside_rings_bounds_is_refused_by_name() {
    let error = sign(
        fixture("rsa1024").key_data(),
        MESSAGE,
        SSH_AGENT_RSA_SHA2_256,
    )
    .unwrap_err();
    assert!(matches!(error, SshSignError::RsaKeyRejected(_)), "{error}");
}

#[test]
fn rsa_signatures_are_deterministic_pkcs1() {
    let key = fixture("rsa2048");
    let first = sign(key.key_data(), MESSAGE, SSH_AGENT_RSA_SHA2_512).unwrap();
    let second = sign(key.key_data(), MESSAGE, SSH_AGENT_RSA_SHA2_512).unwrap();
    assert_eq!(first, second);
}

#[test]
fn ecdsa_signs_on_every_curve_and_ignores_flags() {
    for (name, curve) in [
        ("ecdsa256", EcdsaCurve::NistP256),
        ("ecdsa384", EcdsaCurve::NistP384),
        ("ecdsa521", EcdsaCurve::NistP521),
    ] {
        let key = fixture(name);
        for flags in [0, SSH_AGENT_RSA_SHA2_256, SSH_AGENT_RSA_SHA2_512] {
            let signature = sign(key.key_data(), MESSAGE, flags).unwrap();
            assert_eq!(signature.algorithm(), Algorithm::Ecdsa { curve }, "{name}");
            assert!(verifies(&key, MESSAGE, &signature), "{name}");
            assert!(
                !verifies(&key, b"other data", &signature),
                "{name}: verifier must refuse"
            );
        }
    }
}

/// Captured at `d0d8372b`, before P4a, from the unmodified agent signing
/// [`BASELINE_MESSAGE`] with the `ed25519` fixture (flags 0, 2 and 4 alike).
pub(crate) const BASELINE_MESSAGE: &[u8] = b"personae p4a ed25519 baseline";
pub(crate) const BASELINE_SIGNATURE: &str = "73d06858d19d2464bc757cef01fcea96cd6d6a36283171508d474da39d86301311b3f4a1564b57759681f05cf9b97d5366ccc12e5cf88a428adbb95e8484f707";

#[test]
fn ed25519_signs_exactly_as_before() {
    let key = fixture("ed25519");
    for flags in [0, SSH_AGENT_RSA_SHA2_256, SSH_AGENT_RSA_SHA2_512] {
        let signature = sign(key.key_data(), BASELINE_MESSAGE, flags).unwrap();
        assert_eq!(signature.algorithm(), Algorithm::Ed25519);
        assert_eq!(hex::encode(signature.as_bytes()), BASELINE_SIGNATURE);
        assert_eq!(
            signature,
            key.try_sign(BASELINE_MESSAGE).unwrap(),
            "the pre-P4a path"
        );
        assert!(verifies(&key, BASELINE_MESSAGE, &signature));
    }
}
