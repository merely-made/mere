// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Which persona the resident speaks as.
//!
//! The profile is the **family choice**, picked once beside the shared vault
//! and honoured by every Merely application. An explicit `--profile` beats
//! everything, and graphshell's `GRAPHSHELL_PROFILE` override sits above the
//! family ladder. Moved from graphshell's `profile` in DR-C (dramatis repo
//! plan, D5): reading the vault's roster is custody, so it happens here.

use std::path::Path;

use castellan::custody::IdentityStorage;
use castellan::custody::roster;
use personae::{IdentityError, ProfileId};

/// Resolve the persona: the explicit choice, then the environment override,
/// then the family ladder over the vault's roster.
pub fn resolve_selected_profile(
    storage: &dyn IdentityStorage,
    vault_dir: &Path,
    explicit: Option<&ProfileId>,
) -> Result<ProfileId, IdentityError> {
    if let Some(profile) = explicit {
        return Ok(profile.clone());
    }
    if let Some(profile) = graphshell::profile::env_profile() {
        return Ok(profile);
    }
    roster::resolve_profile(storage, vault_dir)
}

#[cfg(test)]
mod tests {
    use castellan::custody::{InMemoryStorage, Profile};
    use personae::Ed25519Keypair;

    use super::*;

    #[test]
    fn the_selected_profile_is_the_family_choice() {
        // djinn speaks as whoever the user is everywhere else. The rungs that
        // matter here: an explicit flag beats everything, and with no opinion
        // of its own the resident takes the family ladder — including the
        // sole-persona rung, so a vault holding one persona under another
        // name does not gain a second identity minted behind the user's back.
        let dir = tempfile::tempdir().unwrap();
        let storage = InMemoryStorage::new();

        let explicit = ProfileId("receipt".into());
        assert_eq!(
            resolve_selected_profile(&storage, dir.path(), Some(&explicit)).unwrap(),
            explicit
        );

        storage
            .save_profile(&Profile::new(
                ProfileId("stage-name".into()),
                "Stage Name",
                Ed25519Keypair::generate(),
            ))
            .unwrap();
        assert_eq!(
            resolve_selected_profile(&storage, dir.path(), None)
                .unwrap()
                .0,
            "stage-name",
            "the vault's sole persona wins over minting a default beside it"
        );
    }
}
