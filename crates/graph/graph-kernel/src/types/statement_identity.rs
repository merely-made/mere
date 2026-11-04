// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

/// Seed the statement-id minter's per-process salt. The kernel's identity
/// doctrine: OS randomness is native-only, wasm hosts supply theirs — a wasm
/// host should call this once at boot with 64 host-random bits (unseeded wasm
/// falls back to `0`, i.e. the pre-salt time+nonce behaviour, and loses the
/// cross-device guarantee). Native self-seeds from `Uuid::new_v4()` on first
/// mint; a later seed call is a no-op.
pub fn seed_statement_minter(salt: u64) {
    let _ = STATEMENT_MINTER_SALT.set(salt);
}

pub(super) fn statement_minter_salt() -> u64 {
    *STATEMENT_MINTER_SALT.get_or_init(|| {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let bytes = uuid::Uuid::new_v4();
            u64::from_le_bytes(bytes.as_bytes()[..8].try_into().expect("8 bytes"))
        }
        #[cfg(target_arch = "wasm32")]
        {
            0
        }
    })
}

/// Mint a [`SemanticStatement`](crate::graph::SemanticStatement) id — the
/// FEDERATION-SAFE story the petgraph-RDF plan's Phase 1 required (the fact
/// handle reification, precise retract, snapshot migration, and federation
/// tombstones point at):
///
/// `{unix_ms:012x}-{process_salt:016x}-{counter:016x}`
///
/// - Device-safe: two devices collide only on a 64-bit salt collision
///   (2^-64 per process pair), independent of clocks and counters.
/// - Time-sortable by prefix (12 hex ms digits reach year ~10889).
/// - Opaque to consumers: dedup happens on statement CONTENT, never by
///   parsing this id, so legacy `{ts:016x}-{nonce:016x}` ids already in
///   snapshots remain valid handles forever.
pub(crate) fn mint_local_statement_id() -> String {
    let timestamp_ms = crate::time::unix_epoch_millis();
    let salt = statement_minter_salt();
    let nonce = NEXT_LOCAL_STATEMENT_NONCE.fetch_add(1, Ordering::Relaxed);
    format!("{timestamp_ms:012x}-{salt:016x}-{nonce:016x}")
}
