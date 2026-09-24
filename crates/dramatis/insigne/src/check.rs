// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Signature checks, behind the `verify` feature.

use ed25519_dalek::{Signature, Verifier, VerifyingKey};

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
