// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Portable, signed capability delegation: the statements, and their checks.
//!
//! The statements are plain data: what a delegation grants, to whom and for
//! how long, and the canonical bytes a signature covers. Issuing needs a
//! persona's keys and stays in `personae`. Certificate ids and attenuation
//! hash with BLAKE3 behind the `digest` feature; signature checks sit behind
//! `verify`. Moved here from `personae::delegation` on 2026-09-24 with every
//! domain string and format unchanged, so what was issued before still checks.
//!
//! It owns the grammar, not an application's grant ledger or policy: Murm may
//! keep session grants in memory, while Gemot or Kith may fold the same
//! statements through durable replicated stores.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::attestation::DerivedKeyAttestation;

const CERTIFICATE_VERSION: u16 = 1;
const REVOCATION_VERSION: u16 = 1;
const CERTIFICATE_DOMAIN: &[u8] = b"personae/delegation-certificate/v1";
const REVOCATION_DOMAIN: &[u8] = b"personae/delegation-revocation/v1";
const SIGNING_KEY_DOMAIN: &[u8] = b"personae/delegation-signing-key/v1";

/// Stable content identifier for one delegation certificate.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct DelegationId(pub [u8; 32]);

/// The authority above a delegation certificate.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DelegationParent {
    /// Application-owned root authority, such as a constitutional grant.
    Root([u8; 32]),
    /// Another independently signed delegation certificate.
    Certificate(DelegationId),
}

/// A capability target shared across application-specific authority ledgers.
///
/// `domain` separates applications (`moot`, `murm.coop`, `mesh`); `resource`
/// identifies one space within that application; `path_prefix` selects a
/// structural subset; and `actions` remains an application-owned vocabulary.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityScope {
    /// Application or protocol family which owns the action vocabulary.
    pub domain: String,
    /// Opaque application-owned identity of the governed space.
    pub resource: Vec<u8>,
    /// Inclusive structural path prefix within the resource.
    pub path_prefix: String,
    /// Application-owned actions allowed under this scope.
    pub actions: BTreeSet<String>,
}

impl CapabilityScope {
    /// Whether this scope is a valid narrowing of `parent`.
    pub fn attenuates(&self, parent: &Self) -> bool {
        self.is_well_formed()
            && parent.is_well_formed()
            && self.domain == parent.domain
            && self.resource == parent.resource
            && path_covers(&parent.path_prefix, &self.path_prefix)
            && self.actions.is_subset(&parent.actions)
    }

    fn is_well_formed(&self) -> bool {
        !self.domain.is_empty()
            && !self.resource.is_empty()
            && !self.path_prefix.is_empty()
            && !self.actions.is_empty()
            && self.actions.iter().all(|action| !action.is_empty())
    }
}

/// The signed content of an independently delegated capability.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DelegationCertificate {
    version: u16,
    /// Constitutional, session, mesh, or preceding-certificate authority.
    pub parent: DelegationParent,
    /// Personae master public key which currently holds the parent authority.
    pub issuer: [u8; 32],
    /// Personae master public key receiving this capability.
    pub subject: [u8; 32],
    /// Application/resource/path/action subset carried by this certificate.
    pub scope: CapabilityScope,
    /// First millisecond at which the certificate may be used.
    pub not_before_ms: u64,
    /// Issuer-asserted creation time committed by the signature.
    pub issued_at_ms: u64,
    /// Last usable millisecond, or unbounded when the parent is also unbounded.
    pub expires_at_ms: Option<u64>,
    /// Further delegation steps available to the subject. Zero forbids it.
    pub remaining_delegation_depth: u16,
    /// Issuer-chosen uniqueness, allowing two otherwise identical grants.
    pub nonce: [u8; 32],
}

impl DelegationCertificate {
    /// Construct a versioned certificate payload.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        parent: DelegationParent,
        issuer: [u8; 32],
        subject: [u8; 32],
        scope: CapabilityScope,
        issued_at_ms: u64,
        not_before_ms: u64,
        expires_at_ms: Option<u64>,
        remaining_delegation_depth: u16,
        nonce: [u8; 32],
    ) -> Self {
        Self {
            version: CERTIFICATE_VERSION,
            parent,
            issuer,
            subject,
            scope,
            issued_at_ms,
            not_before_ms,
            expires_at_ms,
            remaining_delegation_depth,
            nonce,
        }
    }

    /// Stable content id of this certificate payload: BLAKE3 over its signing
    /// bytes.
    #[cfg(feature = "digest")]
    pub fn id(&self) -> DelegationId {
        DelegationId(*blake3::hash(&self.signing_bytes()).as_bytes())
    }

    /// Whether this certificate is structurally valid on its own.
    pub fn is_well_formed(&self) -> bool {
        self.version == CERTIFICATE_VERSION
            && self.issuer != [0; 32]
            && self.subject != [0; 32]
            && self.scope.is_well_formed()
            && self.issued_at_ms <= self.not_before_ms
            && self
                .expires_at_ms
                .is_none_or(|expires| expires >= self.not_before_ms)
    }

    /// Whether this certificate narrows an already-valid parent certificate.
    ///
    /// Behind `digest`, since naming the parent means hashing it.
    #[cfg(feature = "digest")]
    pub fn attenuates(&self, parent: &Self) -> bool {
        self.is_well_formed()
            && parent.is_well_formed()
            && self.parent == DelegationParent::Certificate(parent.id())
            && self.issuer == parent.subject
            && self.scope.attenuates(&parent.scope)
            && self.not_before_ms >= parent.not_before_ms
            && expiry_within(self.expires_at_ms, parent.expires_at_ms)
            && parent.remaining_delegation_depth > 0
            && self.remaining_delegation_depth < parent.remaining_delegation_depth
    }

    /// Whether the certificate covers one action at the evaluation time.
    pub fn covers(&self, path: &str, action: &str, at_ms: u64) -> bool {
        self.is_well_formed()
            && at_ms >= self.not_before_ms
            && self.expires_at_ms.is_none_or(|expires| at_ms <= expires)
            && path_covers(&self.scope.path_prefix, path)
            && self.scope.actions.contains(action)
    }

    /// The canonical bytes an issuer signs and a checker verifies.
    pub fn signing_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        push_bytes(&mut bytes, CERTIFICATE_DOMAIN);
        bytes.extend_from_slice(&self.version.to_le_bytes());
        match self.parent {
            DelegationParent::Root(id) => {
                bytes.push(0);
                bytes.extend_from_slice(&id);
            },
            DelegationParent::Certificate(id) => {
                bytes.push(1);
                bytes.extend_from_slice(&id.0);
            },
        }
        bytes.extend_from_slice(&self.issuer);
        bytes.extend_from_slice(&self.subject);
        push_scope(&mut bytes, &self.scope);
        bytes.extend_from_slice(&self.issued_at_ms.to_le_bytes());
        bytes.extend_from_slice(&self.not_before_ms.to_le_bytes());
        push_expiry(&mut bytes, self.expires_at_ms);
        bytes.extend_from_slice(&self.remaining_delegation_depth.to_le_bytes());
        bytes.extend_from_slice(&self.nonce);
        bytes
    }
}

/// A certificate plus proof that its signing key belongs to the issuer root.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignedDelegationCertificate {
    /// Capability statement committed by the signature.
    pub certificate: DelegationCertificate,
    /// Master-signed proof of the resource-scoped derived signing key.
    pub signer: DerivedKeyAttestation,
    signature: Vec<u8>,
}

impl SignedDelegationCertificate {
    /// Assemble a signed certificate from its parts. An issuer calls this
    /// after signing [`DelegationCertificate::signing_bytes`]; nothing is
    /// checked here.
    pub fn from_parts(
        certificate: DelegationCertificate,
        signer: DerivedKeyAttestation,
        signature: Vec<u8>,
    ) -> Self {
        Self {
            certificate,
            signer,
            signature,
        }
    }

    /// The signature bytes.
    pub fn signature(&self) -> &[u8] {
        &self.signature
    }

    /// Verify payload, issuer binding, derived-key attestation, and signature.
    #[cfg(feature = "verify")]
    pub fn verify(&self) -> bool {
        self.certificate.is_well_formed()
            && self
                .signer
                .verify(&delegation_signing_salt(&self.certificate.scope))
            && self.signer.master() == &self.certificate.issuer
            && crate::check::ed25519(
                self.signer.derived(),
                &self.signature,
                &self.certificate.signing_bytes(),
            )
    }
}

/// Signed removal of one certificate by the identity which issued it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DelegationRevocation {
    version: u16,
    /// Certificate withdrawn by this statement.
    pub certificate: DelegationId,
    /// Identity root which originally issued the certificate.
    pub issuer: [u8; 32],
    /// Original certificate scope, binding the same derived signing key.
    pub scope: CapabilityScope,
    /// Issuer-chosen uniqueness for this revocation statement.
    pub nonce: [u8; 32],
    /// Issuer-asserted revocation time committed by the signature.
    pub at_ms: u64,
}

impl DelegationRevocation {
    /// Construct a versioned revocation payload.
    pub fn new(
        certificate: DelegationId,
        issuer: [u8; 32],
        scope: CapabilityScope,
        at_ms: u64,
        nonce: [u8; 32],
    ) -> Self {
        Self {
            version: REVOCATION_VERSION,
            certificate,
            issuer,
            scope,
            at_ms,
            nonce,
        }
    }

    /// Whether this revocation is structurally valid on its own.
    pub fn is_well_formed(&self) -> bool {
        self.version == REVOCATION_VERSION && self.scope.is_well_formed()
    }

    /// The canonical bytes an issuer signs and a checker verifies.
    pub fn signing_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        push_bytes(&mut bytes, REVOCATION_DOMAIN);
        bytes.extend_from_slice(&self.version.to_le_bytes());
        bytes.extend_from_slice(&self.certificate.0);
        bytes.extend_from_slice(&self.issuer);
        push_scope(&mut bytes, &self.scope);
        bytes.extend_from_slice(&self.at_ms.to_le_bytes());
        bytes.extend_from_slice(&self.nonce);
        bytes
    }
}

/// A revocation any application-owned grant ledger can fold once checked.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignedDelegationRevocation {
    /// Revocation statement committed by the signature.
    pub revocation: DelegationRevocation,
    /// Master-signed proof of the resource-scoped derived signing key.
    pub signer: DerivedKeyAttestation,
    signature: Vec<u8>,
}

impl SignedDelegationRevocation {
    /// Assemble a signed revocation from its parts. An issuer calls this
    /// after signing [`DelegationRevocation::signing_bytes`]; nothing is
    /// checked here.
    pub fn from_parts(
        revocation: DelegationRevocation,
        signer: DerivedKeyAttestation,
        signature: Vec<u8>,
    ) -> Self {
        Self {
            revocation,
            signer,
            signature,
        }
    }

    /// The signature bytes.
    pub fn signature(&self) -> &[u8] {
        &self.signature
    }

    /// Verify payload, issuer binding, derived-key attestation, and signature.
    #[cfg(feature = "verify")]
    pub fn verify(&self) -> bool {
        self.revocation.is_well_formed()
            && self
                .signer
                .verify(&delegation_signing_salt(&self.revocation.scope))
            && self.signer.master() == &self.revocation.issuer
            && crate::check::ed25519(
                self.signer.derived(),
                &self.signature,
                &self.revocation.signing_bytes(),
            )
    }
}

/// Deterministic salt for the signing key assigned to one authority resource.
///
/// Hosts use this when the same derived key signs an outer transport envelope.
pub fn delegation_signing_salt(scope: &CapabilityScope) -> Vec<u8> {
    let mut bytes = Vec::new();
    push_bytes(&mut bytes, SIGNING_KEY_DOMAIN);
    push_str(&mut bytes, &scope.domain);
    push_bytes(&mut bytes, &scope.resource);
    bytes
}

/// Whether `prefix` selects `path` at a structural slash boundary.
///
/// A scope for `/publications/a` reaches `/publications/a/version`, but not
/// `/publications/a-private`. Application service routing uses the same rule
/// when a base service owns resource-specific children.
pub fn path_covers(prefix: &str, path: &str) -> bool {
    path == prefix
        || path
            .strip_prefix(prefix)
            .is_some_and(|suffix| suffix.starts_with('/'))
}

#[cfg(feature = "digest")]
fn expiry_within(child: Option<u64>, parent: Option<u64>) -> bool {
    match (child, parent) {
        (_, None) => true,
        (Some(child), Some(parent)) => child <= parent,
        (None, Some(_)) => false,
    }
}

fn push_scope(bytes: &mut Vec<u8>, scope: &CapabilityScope) {
    push_str(bytes, &scope.domain);
    push_bytes(bytes, &scope.resource);
    push_str(bytes, &scope.path_prefix);
    bytes.extend_from_slice(&(scope.actions.len() as u64).to_le_bytes());
    for action in &scope.actions {
        push_str(bytes, action);
    }
}

fn push_str(bytes: &mut Vec<u8>, value: &str) {
    push_bytes(bytes, value.as_bytes());
}

fn push_bytes(bytes: &mut Vec<u8>, value: &[u8]) {
    bytes.extend_from_slice(&(value.len() as u64).to_le_bytes());
    bytes.extend_from_slice(value);
}

fn push_expiry(bytes: &mut Vec<u8>, value: Option<u64>) {
    match value {
        Some(value) => {
            bytes.push(1);
            bytes.extend_from_slice(&value.to_le_bytes());
        },
        None => bytes.push(0),
    }
}
