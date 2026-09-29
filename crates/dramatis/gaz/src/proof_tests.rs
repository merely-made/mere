// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use insigne::{CapabilityScope, CheckFault, DelegationCertificate, DelegationParent};
use personae::delegation::Issue;
use personae::{IdentityProvider, InMemoryProvider};

use super::*;
use crate::{AttestError, Contact, ContactBook, ContactError, PersonaScope};

fn provider(seed: u8) -> InMemoryProvider {
    InMemoryProvider::from_seed([seed; 32])
}

fn root(provider: &InMemoryProvider) -> TypedKey {
    TypedKey::ed25519(provider.master_public_key().to_bytes())
}

fn attestation(provider: &InMemoryProvider, salt: &[u8]) -> KeyProof {
    KeyProof::Attestation {
        attestation: provider.attest_derived_key(salt).unwrap(),
        salt: salt.to_vec(),
    }
}

fn derived(proof: &KeyProof) -> TypedKey {
    match proof {
        KeyProof::Attestation { attestation, .. } => attestation.derived_key(),
        _ => panic!("expected an attestation"),
    }
}

fn check_attestation(proof: &KeyProof, expected_key: &TypedKey, expected_root: &TypedKey) {
    let KeyProof::Attestation { attestation, salt } = proof else {
        panic!("expected an attestation");
    };
    let checked = attestation.check(salt).unwrap();
    assert_eq!(checked.derived_key(), *expected_key);
    assert_eq!(checked.master_key(), *expected_root);
}

fn reloads(book: &ContactBook) -> [ContactBook; 2] {
    [
        serde_json::from_slice(&serde_json::to_vec(book).unwrap()).unwrap(),
        postcard::from_bytes(&postcard::to_allocvec(book).unwrap()).unwrap(),
    ]
}

#[test]
fn a_book_reloads_root_and_attested_artifacts_and_checks_them_again() {
    let issuer = provider(1);
    let rotation_salt = b"root-rotation\0\xff";
    let rotation = attestation(&issuer, rotation_salt);
    let next = InMemoryProvider::from_seed(issuer.derive_keypair(rotation_salt).unwrap().to_seed());
    let device = attestation(&next, b"protocol/device\0\xfe");
    let mut contact = Contact::new("Alice", root(&issuer));
    contact.rotate_to(root(&next), Some(rotation)).unwrap();
    contact
        .attest(derived(&device), "display scope", root(&next), Some(device))
        .unwrap();
    let mut book = ContactBook::new(PersonaScope::new("work"));
    book.insert(contact.clone());

    for loaded in reloads(&book) {
        assert_eq!(loaded, book);
        let contact = loaded.get(contact.anchor()).unwrap();
        let rotation = &contact.root_line()[1];
        check_attestation(
            rotation.proof.as_ref().unwrap(),
            &rotation.key,
            &root(&issuer),
        );
        let device = &contact.attested()[0];
        check_attestation(device.proof.as_ref().unwrap(), &device.key, &device.root);
        assert_eq!(device.scope, "display scope");
    }
}

fn delegation(issuer: &InMemoryProvider, subject: &InMemoryProvider) -> KeyProof {
    let certificate = DelegationCertificate::new(
        DelegationParent::Root([9; 32]),
        issuer.master_public_key().to_bytes(),
        subject.master_public_key().to_bytes(),
        CapabilityScope {
            domain: "gaz-test".into(),
            resource: vec![7],
            path_prefix: "/".into(),
            actions: ["speak".into()].into(),
        },
        10,
        10,
        Some(100),
        0,
        [8; 32],
    );
    KeyProof::Delegation(Box::new(
        SignedDelegationCertificate::issue(issuer, certificate).unwrap(),
    ))
}

#[test]
fn delegation_artifacts_survive_both_key_roles_and_both_codecs() {
    let issuer = provider(1);
    let subject = provider(2);
    for as_rotation in [true, false] {
        let evidence = delegation(&issuer, &subject);
        let mut contact = Contact::new("Alice", root(&issuer));
        if as_rotation {
            contact.rotate_to(root(&subject), Some(evidence)).unwrap();
        } else {
            contact
                .attest(root(&subject), "station", root(&issuer), Some(evidence))
                .unwrap();
        }
        let mut book = ContactBook::new(PersonaScope::new("work"));
        book.insert(contact.clone());
        for loaded in reloads(&book) {
            assert_eq!(loaded, book);
            let contact = loaded.get(contact.anchor()).unwrap();
            let proof = if as_rotation {
                contact.root_line()[1].proof.as_ref().unwrap()
            } else {
                contact.attested()[0].proof.as_ref().unwrap()
            };
            assert_eq!(proof.method(), ProofMethod::Signature);
            let KeyProof::Delegation(signed) = proof else {
                panic!("lost delegation");
            };
            assert_eq!(
                signed.check().unwrap().certificate().subject,
                subject.master_public_key().to_bytes()
            );
        }
    }
}

#[test]
fn opaque_protocol_evidence_keeps_its_format_method_and_exact_bytes() {
    let evidence = KeyProof::Other {
        method: ProofMethod::DidAuth,
        format: "caller-owned-operation/v1".into(),
        bytes: vec![0, 255, 13, 10, 128],
    };
    let mut contact = Contact::new("Alice", root(&provider(1)));
    contact
        .rotate_to(root(&provider(2)), Some(evidence.clone()))
        .unwrap();
    contact
        .attest(
            root(&provider(3)),
            "device",
            root(&provider(2)),
            Some(evidence.clone()),
        )
        .unwrap();
    let mut book = ContactBook::new(PersonaScope::new("work"));
    book.insert(contact.clone());
    for loaded in reloads(&book) {
        let contact = loaded.get(contact.anchor()).unwrap();
        assert_eq!(contact.root_line()[1].proof, Some(evidence.clone()));
        assert_eq!(contact.attested()[0].proof, Some(evidence.clone()));
        assert_eq!(evidence.method(), ProofMethod::DidAuth);
    }
}

#[test]
fn retaining_or_loading_an_artifact_does_not_make_its_signature_valid() {
    let issuer = provider(1);
    for wrong_context in [true, false] {
        let mut proof = attestation(&issuer, b"correct salt");
        let KeyProof::Attestation { attestation, salt } = &mut proof else {
            unreachable!()
        };
        if wrong_context {
            *salt = b"wrong salt".to_vec();
        } else {
            let mut signature = attestation.signature().to_vec();
            signature[0] ^= 1;
            *attestation = DerivedKeyAttestation::from_parts(
                *attestation.master(),
                *attestation.derived(),
                signature,
            );
        }
        let mut contact = Contact::new("Alice", root(&issuer));
        contact
            .attest(derived(&proof), "device", root(&issuer), Some(proof))
            .unwrap();
        let mut book = ContactBook::new(PersonaScope::new("work"));
        book.insert(contact.clone());
        for loaded in reloads(&book) {
            let proof = loaded.get(contact.anchor()).unwrap().attested()[0]
                .proof
                .as_ref()
                .unwrap();
            let KeyProof::Attestation { attestation, salt } = proof else {
                panic!("lost artifact");
            };
            assert_eq!(attestation.check(salt), Err(CheckFault::BadSignature));
        }
    }
}

#[test]
fn typed_evidence_must_name_the_recorded_key_and_root_before_insertion() {
    let issuer = provider(1);
    let stranger = provider(2);
    for proof in [
        attestation(&issuer, b"device"),
        delegation(&issuer, &stranger),
    ] {
        let key = match &proof {
            KeyProof::Attestation { .. } => derived(&proof),
            KeyProof::Delegation(_) => root(&stranger),
            _ => unreachable!(),
        };
        let mut contact = Contact::new("Alice", root(&issuer));
        let before = contact.clone();
        assert_eq!(
            contact.rotate_to(root(&provider(3)), Some(proof.clone())),
            Err(ContactError::ProofMismatch)
        );
        assert_eq!(
            contact.attest(
                root(&provider(3)),
                "device",
                root(&issuer),
                Some(proof.clone())
            ),
            Err(AttestError::ProofMismatch)
        );
        assert_eq!(contact, before);

        // A proof from an old root cannot justify a later rotation. Both
        // roots are known, so this exercises the preceding-root requirement.
        contact.rotate_to(root(&provider(4)), None).unwrap();
        let before = contact.clone();
        assert_eq!(
            contact.rotate_to(key, Some(proof.clone())),
            Err(ContactError::ProofMismatch)
        );
        assert_eq!(
            contact.attest(key, "device", root(&provider(4)), Some(proof)),
            Err(AttestError::ProofMismatch)
        );
        assert_eq!(contact, before);
    }
}

#[test]
fn a_reloaded_record_refuses_artifacts_attached_to_other_keys_or_roots() {
    let issuer = provider(1);
    for as_rotation in [true, false] {
        for evidence in [
            attestation(&issuer, b"device"),
            delegation(&issuer, &provider(2)),
        ] {
            let key = match &evidence {
                KeyProof::Attestation { .. } => derived(&evidence),
                KeyProof::Delegation(_) => root(&provider(2)),
                _ => unreachable!(),
            };
            let mut contact = Contact::new("Alice", root(&issuer));
            if as_rotation {
                contact.rotate_to(key, Some(evidence)).unwrap();
            } else {
                contact
                    .attest(key, "device", root(&issuer), Some(evidence))
                    .unwrap();
            }
            for wrong_root in [true, false] {
                let mut record = serde_json::to_value(&contact).unwrap();
                let wrong = serde_json::to_value(root(&provider(3))).unwrap();
                if wrong_root && as_rotation {
                    record["anchor"] = wrong.clone();
                    record["root_line"][0]["key"] = wrong;
                } else if wrong_root {
                    record["root_line"]
                        .as_array_mut()
                        .unwrap()
                        .push(serde_json::json!({"key": wrong.clone(), "proof": null}));
                    record["attested"][0]["root"] = wrong;
                } else if as_rotation {
                    record["root_line"][1]["key"] = wrong;
                } else {
                    record["attested"][0]["key"] = wrong;
                }
                let error = serde_json::from_value::<Contact>(record).unwrap_err();
                assert!(
                    error
                        .to_string()
                        .contains("proof names a different key or root"),
                    "{error}"
                );
            }
        }
    }
}

#[test]
fn replaying_a_key_preserves_its_original_evidence() {
    let issuer = provider(1);
    for as_rotation in [true, false] {
        let evidence = attestation(&issuer, b"device");
        let key = derived(&evidence);
        let mut contact = Contact::new("Alice", root(&issuer));
        if as_rotation {
            contact.rotate_to(key, Some(evidence)).unwrap();
            let before = contact.clone();
            assert!(!contact.rotate_to(key, None).unwrap());
            assert_eq!(contact, before);
        } else {
            contact
                .attest(key, "device", root(&issuer), Some(evidence))
                .unwrap();
            let before = contact.clone();
            assert!(
                !contact
                    .attest(key, "changed scope", root(&issuer), None)
                    .unwrap()
            );
            assert_eq!(contact, before);
        }
    }
}
