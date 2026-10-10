// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The wallet's codicil sealer for a persona's current private epoch: the
//! loader half of pandect's `WalletEpochSealer`, moved with the epoch bridge
//! (dramatis repo plan, DR-B, D8). pandect keeps the sealer itself, built from
//! an epoch its caller holds.

use std::io;
use std::path::Path;

use pandect::{PersonaId, WalletEpochSealer};

use super::load_current_private_epoch;

/// Build a sealer from a persona's current private epoch, or `None` when the
/// persona has no staged epoch yet (nothing to seal under, so writes stay
/// cleartext: the host's degraded-but-honest posture).
pub fn epoch_sealer_for_persona(
    data_root: &Path,
    persona: PersonaId,
) -> io::Result<Option<WalletEpochSealer>> {
    let Some(epoch) = load_current_private_epoch(data_root, persona)? else {
        return Ok(None);
    };
    Ok(Some(WalletEpochSealer::from_epoch(
        persona,
        epoch.epoch_id,
        &epoch.epoch_secret,
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use eidetic::{Hash, PayloadSealer};

    #[test]
    fn for_persona_is_none_without_a_staged_epoch() {
        // No wallet state on this path, so there is no epoch to seal under and
        // the host stays in the cleartext lane rather than erroring.
        let dir = std::env::temp_dir().join("mere-codicil-seal-none-probe");
        let _ = std::fs::remove_dir_all(&dir);
        let sealer = epoch_sealer_for_persona(&dir, PersonaId::new()).unwrap();
        assert!(sealer.is_none());
    }

    #[test]
    fn for_persona_builds_a_working_sealer_from_a_staged_epoch() {
        // The path the meerkat wiring relies on: real wallet state -> for_persona
        // -> a sealer that actually seals and unseals.
        use crate::custody::wallet::{
            ensure_wallet_state, load_persona_wallet, stage_persona_private_epoch,
        };

        let dir = std::env::temp_dir().join(format!(
            "mere-codicil-seal-forpersona-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        let persona = PersonaId::new();
        ensure_wallet_state(&dir, persona, "Test PC").unwrap();
        let head = load_persona_wallet(&dir, persona)
            .unwrap()
            .unwrap()
            .private_epoch_head;
        stage_persona_private_epoch(&dir, persona, head, b"staged-epoch-secret").unwrap();

        let sealer = epoch_sealer_for_persona(&dir, persona)
            .unwrap()
            .expect("a staged epoch yields a sealer");
        let cleartext = b"round-trip via for_persona";
        let hash = Hash::of(cleartext);
        let (sealed, marker) = sealer.seal(&hash, cleartext).unwrap();
        assert_ne!(sealed.as_slice(), cleartext);
        assert_eq!(sealer.unseal(&hash, &marker, &sealed).unwrap(), cleartext);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
