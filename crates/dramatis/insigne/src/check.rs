// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Signature checks, behind the `verify` feature.

use std::fmt;

use ed25519_dalek::{Signature, Verifier, VerifyingKey};

/// Which step of a check failed.
///
/// A check runs its steps in a fixed order and reports the first that fails.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum CheckFault {
    /// The statement's fields or format version are out of range.
    Malformed,
    /// The signer's attestation does not check under the statement's scope.
    BadAttestation,
    /// The attesting master is not the statement's declared issuer.
    WrongIssuer,
    /// The signature does not check under the key that should have made it.
    BadSignature,
}

impl fmt::Display for CheckFault {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Malformed => "statement is malformed",
            Self::BadAttestation => "signer's attestation does not check",
            Self::WrongIssuer => "signer is not the statement's issuer",
            Self::BadSignature => "signature does not check",
        })
    }
}

impl std::error::Error for CheckFault {}

/// Check an Ed25519 signature in ed25519-dalek's default, lax mode: the mode
/// personae's `Ed25519PublicKey::verify` used before the move, so moving the
/// checks here changes nothing about what verifies.
pub(crate) fn ed25519(public_key: &[u8; 32], signature: &[u8], message: &[u8]) -> bool {
    let Ok(key) = VerifyingKey::from_bytes(public_key) else {
        return false;
    };
    let Ok(signature) = <[u8; 64]>::try_from(signature) else {
        return false;
    };
    key.verify(message, &Signature::from_bytes(&signature))
        .is_ok()
}
