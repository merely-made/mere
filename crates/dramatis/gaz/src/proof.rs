// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Retained evidence for a key, rather than a serialized checking result.

use insigne::{DerivedKeyAttestation, SignedDelegationCertificate, TypedKey};
use serde::{Deserialize, Serialize};

use crate::ProofMethod;

/// The artifact a caller used when recording a key.
///
/// Holding this is not a successful check. Gaz compares the keys named by
/// typed artifacts to the record, but never verifies signatures or authority.
/// After reload, a caller checks the artifact again and applies its current
/// policy, including delegation chains, expiry and revocation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyProof {
    /// A root's signed binding of a derived key and the exact derivation salt.
    /// The contact's display scope need not be that salt.
    Attestation {
        /// The signed statement, kept unchanged.
        attestation: DerivedKeyAttestation,
        /// The context committed by the signature, needed to check it again.
        salt: Vec<u8>,
    },
    /// An issuer's signed delegation to a subject key. Its signing context is
    /// already inside the certificate. A capability grant alone does not
    /// establish that its subject and issuer are the same person; that
    /// interpretation remains the caller's responsibility.
    Delegation(Box<SignedDelegationCertificate>),
    /// Evidence owned by another protocol, such as a PLC operation, or a
    /// human's out-of-band confirmation. Gaz preserves the exact bytes and
    /// format identifier for the caller's decoder and checker.
    Other {
        /// How the caller established the binding.
        method: ProofMethod,
        /// Caller-owned format identifier, including any required version.
        format: String,
        /// The artifact and any context its checker requires, encoded in that
        /// format. Gaz neither decodes it nor interprets its key bindings.
        bytes: Vec<u8>,
    },
}

impl KeyProof {
    /// How the retained evidence was checked according to the caller.
    pub fn method(&self) -> ProofMethod {
        match self {
            Self::Attestation { .. } | Self::Delegation(_) => ProofMethod::Signature,
            Self::Other { method, .. } => method.clone(),
        }
    }

    /// Structural agreement only. This does not establish authenticity.
    pub(crate) fn binds(&self, key: &TypedKey, root: Option<&TypedKey>) -> bool {
        match self {
            Self::Attestation { attestation, .. } => {
                *key == attestation.derived_key() && root == Some(&attestation.master_key())
            },
            Self::Delegation(signed) => {
                *key == TypedKey::ed25519(signed.certificate.subject)
                    && root == Some(&TypedKey::ed25519(signed.certificate.issuer))
            },
            Self::Other { .. } => true,
        }
    }
}

#[cfg(test)]
#[path = "proof_tests.rs"]
mod tests;
