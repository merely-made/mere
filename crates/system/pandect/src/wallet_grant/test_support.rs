// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Shared fixtures for the grant modulesّ test suites.

use identity::{Ed25519Keypair, IdentityProvider, InMemoryProvider, PersonaId};
use uuid::Uuid;

use super::*;

pub(super) fn fixture_device() -> DeviceId {
    DeviceId::from_uuid(Uuid::from_u128(0xaaa1))
}

pub(super) fn fixture_persona() -> PersonaId {
    PersonaId::from_uuid(Uuid::from_u128(0xaaa2))
}

pub(super) fn fixture_epoch() -> KeyEpochId {
    KeyEpochId(Uuid::from_u128(0xaaa3))
}

pub(super) fn second_epoch() -> KeyEpochId {
    KeyEpochId(Uuid::from_u128(0xaaa5))
}

pub(super) fn second_persona() -> PersonaId {
    PersonaId::from_uuid(Uuid::from_u128(0xaaa4))
}

pub(super) fn delegator() -> Ed25519Keypair {
    InMemoryProvider::from_seed([3; 32])
        .derive_keypair(b"wallet-grant-delegator")
        .unwrap()
}

pub(super) fn delegatee() -> Ed25519Keypair {
    InMemoryProvider::from_seed([4; 32])
        .derive_keypair(b"wallet-grant-delegatee")
        .unwrap()
}

pub(super) fn sample_pairing_ticket_request() -> RemoteAuthPairingTicketRequest {
    RemoteAuthPairingTicketRequest {
        issued_at_ms: 1_700_000_001,
        // Far-future (2100-01-01), like the grant fixtures below: the
        // ticket-issue path checks expiry against the real clock, and the
        // old seconds-scale 1_800_000_001 in this ms field reads as 1970,
        // so every ticket-consuming test failed as "expired". The
        // deliberately-expired test overrides this to Some(1) itself.
        expires_at_ms: Some(4_102_444_800_000),
        personas: vec![fixture_persona()],
        scopes: vec!["identity.act".into(), "private.read".into()],
        attenuations: vec!["no-subdelegation".into()],
    }
}
