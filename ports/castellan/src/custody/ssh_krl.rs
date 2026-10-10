// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The revocation ledger as a profile slot: the custody half of personae's
//! `ssh_krl`, moved with the vault (dramatis repo plan, DR-B).

use personae::IdentityError;
use personae::ssh_krl::{REVOCATION_MOD_ID, RevocationLedger, ledger_key};

use crate::custody::vault::{CredentialLineage, IdentitySlot, Profile, SecretBytes, UnlockTier};

/// Read a profile's revocation ledger, empty when it has none.
pub fn load_ledger(profile: &Profile) -> Result<RevocationLedger, IdentityError> {
    let Some(IdentitySlot::Direct { payload, .. }) = profile.slots.get(&ledger_key()) else {
        return Ok(RevocationLedger::new());
    };
    serde_json::from_slice(payload.as_slice())
        .map_err(|err| IdentityError::Backend(format!("decode revocation ledger: {err}")))
}

/// Write a profile's revocation ledger.
pub fn store_ledger(profile: &mut Profile, ledger: &RevocationLedger) -> Result<(), IdentityError> {
    let encoded = serde_json::to_vec(ledger)
        .map_err(|err| IdentityError::Backend(format!("encode revocation ledger: {err}")))?;
    profile.slots.insert(
        ledger_key(),
        IdentitySlot::Direct {
            kind: REVOCATION_MOD_ID.to_string(),
            payload: SecretBytes::new(encoded),
            lineage: CredentialLineage::LocallyDerived,
            unlock_tier: UnlockTier::Session,
        },
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use insigne::delegation::{DelegationId, DelegationRevocation, SignedDelegationRevocation};
    use personae::carry::{ACTION_SSH_LOGIN, DeviceId, device_capability_scope};
    use personae::delegation::Issue;
    use personae::vault::ProfileId;
    use personae::{Ed25519Keypair, IdentityProvider, InMemoryProvider};

    const NOW_MS: u64 = 1_760_000_000_000;

    fn device(n: u128) -> DeviceId {
        DeviceId::from_uuid(uuid::Uuid::from_u128(n))
    }

    fn revocation(
        provider: &InMemoryProvider,
        device: DeviceId,
        grant: DelegationId,
    ) -> SignedDelegationRevocation {
        SignedDelegationRevocation::issue(
            provider,
            DelegationRevocation::new(
                grant,
                provider.master_public_key().to_bytes(),
                device_capability_scope(device, [ACTION_SSH_LOGIN]),
                NOW_MS,
                [1; 32],
            ),
        )
        .unwrap()
    }

    #[test]
    fn a_ledger_round_trips_through_a_slot() {
        let provider = InMemoryProvider::from_seed([1; 32]);
        let mut ledger = RevocationLedger::new();
        ledger.fold(
            &revocation(&provider, device(3), DelegationId([2; 32])),
            "imac",
        );

        let mut profile = Profile::new(
            ProfileId("t".into()),
            "t",
            Ed25519Keypair::from_seed([5; 32]),
        );
        store_ledger(&mut profile, &ledger).unwrap();
        assert_eq!(load_ledger(&profile).unwrap(), ledger);
    }

    #[test]
    fn an_absent_ledger_reads_as_empty_rather_than_failing() {
        let profile = Profile::new(
            ProfileId("t".into()),
            "t",
            Ed25519Keypair::from_seed([5; 32]),
        );
        assert!(load_ledger(&profile).unwrap().is_empty());
    }
}
