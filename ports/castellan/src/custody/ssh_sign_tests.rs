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

/// Fixtures are test-only, unencrypted keys made by `ssh-keygen` (OpenSSH
/// for Windows 9.5p2), comment `personae-p4a-fixture-<name>`, except
/// `rsa2048e3` (e = 3), made by Python `cryptography` 50, which ssh-keygen
/// cannot do.
pub(crate) fn fixture(name: &str) -> PrivateKey {
    let text = match name {
        "ed25519" => include_str!("../../tests/fixtures/ssh/ed25519"),
        "rsa1024" => include_str!("../../tests/fixtures/ssh/rsa1024"),
        "rsa2048" => include_str!("../../tests/fixtures/ssh/rsa2048"),
        "rsa2048e3" => include_str!("../../tests/fixtures/ssh/rsa2048e3"),
        "rsa2560" => include_str!("../../tests/fixtures/ssh/rsa2560"),
        "rsa3072" => include_str!("../../tests/fixtures/ssh/rsa3072"),
        "rsa4096" => include_str!("../../tests/fixtures/ssh/rsa4096"),
        "rsa8192" => include_str!("../../tests/fixtures/ssh/rsa8192"),
        "ecdsa256" => include_str!("../../tests/fixtures/ssh/ecdsa256"),
        "ecdsa384" => include_str!("../../tests/fixtures/ssh/ecdsa384"),
        "ecdsa521" => include_str!("../../tests/fixtures/ssh/ecdsa521"),
        "dsa" => include_str!("../../tests/fixtures/ssh/dsa"),
        other => panic!("no fixture {other}"),
    };
    PrivateKey::from_openssh(text).expect("fixture parses")
}

/// Security-key types carry a hardware handle, not a private key; built
/// from public parts here, since making one needs a FIDO device.
pub(crate) fn security_keys() -> Vec<KeypairData> {
    use ssh_key::private::{SkEcdsaSha2NistP256, SkEd25519};
    let ed = fixture("ed25519");
    let ec = fixture("ecdsa256");
    let ssh_key::public::KeyData::Ed25519(ed_public) = PublicKey::from(&ed).key_data().clone()
    else {
        unreachable!()
    };
    let ssh_key::public::KeyData::Ecdsa(ssh_key::public::EcdsaPublicKey::NistP256(ec_point)) =
        PublicKey::from(&ec).key_data().clone()
    else {
        unreachable!()
    };
    vec![
        KeypairData::SkEd25519(
            SkEd25519::new(
                ssh_key::public::SkEd25519::new(ed_public, "ssh:"),
                0x01,
                vec![1, 2, 3, 4],
            )
            .unwrap(),
        ),
        KeypairData::SkEcdsaSha2NistP256(
            SkEcdsaSha2NistP256::new(
                ssh_key::public::SkEcdsaSha2NistP256::new(ec_point, "ssh:"),
                0x01,
                vec![1, 2, 3, 4],
            )
            .unwrap(),
        ),
    ]
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
fn ecdsa_signs_on_p256_and_p384_and_ignores_flags() {
    for (name, curve) in [
        ("ecdsa256", EcdsaCurve::NistP256),
        ("ecdsa384", EcdsaCurve::NistP384),
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

/// Ruling 55: refused at the door and at signing, even for a P-521 key the
/// decoder happened to accept.
#[test]
fn p521_is_refused_until_upstream_decodes_every_key() {
    let key = fixture("ecdsa521");
    assert!(matches!(
        check_signable(key.key_data()),
        Err(SshSignError::P521Refused)
    ));
    assert!(matches!(
        sign(key.key_data(), MESSAGE, 0),
        Err(SshSignError::P521Refused)
    ));
}

#[test]
fn every_signable_key_passes_the_door() {
    for name in [
        "ed25519", "rsa2048", "rsa3072", "rsa4096", "ecdsa256", "ecdsa384",
    ] {
        check_signable(fixture(name).key_data()).unwrap_or_else(|error| panic!("{name}: {error}"));
    }
}

/// Ruling 56: each refusal names why.
#[test]
fn unsignable_keys_are_refused_with_the_reason() {
    for name in ["rsa1024", "rsa2560", "rsa8192", "rsa2048e3"] {
        let error = check_signable(fixture(name).key_data()).unwrap_err();
        println!("{name}: {error}");
        assert!(
            matches!(error, SshSignError::RsaKeyRejected(_)),
            "{name}: {error}"
        );
        assert!(
            error.to_string().contains("2048 to 4096 bits"),
            "{name}: {error}"
        );
    }
    let error = check_signable(fixture("dsa").key_data()).unwrap_err();
    assert_eq!(error.to_string(), "the agent does not sign ssh-dss keys");
    for key in security_keys() {
        let error = check_signable(&key).unwrap_err();
        assert!(matches!(error, SshSignError::Unsupported(_)), "{error}");
        assert!(error.to_string().contains("sk-"), "{error}");
    }
}

/// Ruling 59 tripwire. ring's and the rsa crate's PKCS#1 v1.5 signatures are
/// byte-identical, so no output test can tell who signed; this keeps the
/// rsa crate to building the key, and ring the only RSA signer.
#[test]
fn the_rsa_crate_only_builds_the_key() {
    const SOURCE: &str = include_str!("ssh_sign.rs");
    let rsa_uses: Vec<&str> = SOURCE
        .match_indices("rsa::")
        .filter(|(at, _)| !SOURCE[..*at].ends_with("ring::"))
        .map(|(at, _)| {
            let rest = &SOURCE[at..];
            &rest[..rest
                .find(|c: char| !(c.is_alphanumeric() || c == ':' || c == '_'))
                .unwrap()]
        })
        .collect();
    for path in &rsa_uses {
        assert!(
            [
                "rsa::pkcs8::EncodePrivateKey",
                "rsa::BigUint::from_bytes_be",
                "rsa::RsaPrivateKey::from_components",
            ]
            .contains(path),
            "the rsa crate may build the key, never sign: found {path}"
        );
    }
    assert_eq!(rsa_uses.len(), 3);
    let compact: String = SOURCE.split_whitespace().collect();
    assert_eq!(compact.matches("ring_key.sign(").count(), 1);
    assert_eq!(
        compact.matches(".sign(").count(),
        1,
        "one RSA signer, ring's"
    );
}
