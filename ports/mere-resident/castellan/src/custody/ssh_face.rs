// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! A face's SSH policy as a profile slot: the custody half of personae's
//! `ssh_face`, moved with the vault (dramatis repo plan, DR-B).

use personae::IdentityError;
use personae::ssh_face::{FacePolicy, SSH_FACE_MOD_ID, policy_key};

use crate::custody::vault::{CredentialLineage, IdentitySlot, Profile, SecretBytes, UnlockTier};

/// Read a profile's face policy.
///
/// Returns `None` when the face has never been given one; callers should
/// fall back to [`FacePolicy::work`] under the profile's own name rather
/// than inventing a wider default.
pub fn load_policy(profile: &Profile) -> Result<Option<FacePolicy>, IdentityError> {
    let Some(IdentitySlot::Direct { payload, .. }) = profile.slots.get(&policy_key()) else {
        return Ok(None);
    };
    serde_json::from_slice(payload.as_slice())
        .map(Some)
        .map_err(|err| IdentityError::Backend(format!("decode ssh face policy: {err}")))
}

/// Write a profile's face policy, replacing any previous one.
pub fn store_policy(profile: &mut Profile, policy: &FacePolicy) -> Result<(), IdentityError> {
    let encoded = serde_json::to_vec(policy)
        .map_err(|err| IdentityError::Backend(format!("encode ssh face policy: {err}")))?;
    profile.slots.insert(
        policy_key(),
        IdentitySlot::Direct {
            kind: SSH_FACE_MOD_ID.to_string(),
            payload: SecretBytes::new(encoded),
            // The policy is derived from a decision, not from key material:
            // losing it costs a re-declaration, not a re-registration.
            lineage: CredentialLineage::LocallyDerived,
            unlock_tier: UnlockTier::Session,
        },
    );
    Ok(())
}

/// The policy in force for a profile: its stored one, or a work face named
/// for the profile itself.
pub fn effective_policy(profile: &Profile) -> Result<FacePolicy, IdentityError> {
    Ok(load_policy(profile)?.unwrap_or_else(|| FacePolicy::work(profile.id.0.clone())))
}

#[cfg(test)]
mod tests {
    use super::*;
    use personae::Ed25519Keypair;
    use personae::vault::ProfileId;

    fn profile(id: &str) -> Profile {
        Profile::new(ProfileId(id.into()), id, Ed25519Keypair::from_seed([5; 32]))
    }

    #[test]
    fn a_policy_round_trips_through_a_slot() {
        let mut profile = profile("work");
        let policy = FacePolicy::research("markik");
        store_policy(&mut profile, &policy).unwrap();
        assert_eq!(load_policy(&profile).unwrap(), Some(policy));
    }

    #[test]
    fn storing_twice_replaces_rather_than_accumulates() {
        let mut profile = profile("work");
        store_policy(&mut profile, &FacePolicy::work("markik")).unwrap();
        store_policy(&mut profile, &FacePolicy::burner("burner", "uptime")).unwrap();
        assert_eq!(profile.slots.len(), 1);
        assert_eq!(
            load_policy(&profile).unwrap().unwrap().principals,
            vec!["burner".to_string()]
        );
    }

    /// An absent policy must not silently mean "full reach for any name".
    #[test]
    fn the_default_face_is_named_for_its_profile() {
        let policy = effective_policy(&profile("research")).unwrap();
        assert_eq!(policy.principals, vec!["research".to_string()]);
    }
}
