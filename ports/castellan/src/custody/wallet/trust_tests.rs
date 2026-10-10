// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! pandect's `wallet_grant::trust` tests. They issue grants with the
//! wallet's seed, custody since the dramatis repo plan's DR-B, so they moved
//! with it.

mod tests {
    use personae::IdentityProvider;

    use super::super::test_support::*;
    use super::super::*;
    use notochord::{ChainFault, TrustedRoot, validate_chain};

    /// pandect's `wallet_grant::trust` depth for a device grant.
    const DEVICE_GRANT_DEPTH: u16 = 1;

    const NOW_MS: u64 = 1_750_000_000_000;

    fn seeded(tag: &str) -> (std::path::PathBuf, [u8; 32], [u8; 32]) {
        let root = temp_data_root(tag);
        let seed =
            crate::custody::wallet::ensure_wallet_state(&root, fixture_persona(), "Studio PC")
                .unwrap();
        let master = personae::InMemoryProvider::from_seed(seed)
            .master_public_key()
            .to_bytes();
        (root, seed, master)
    }

    fn holder() -> DevicePublicKey {
        DevicePublicKey::from(delegatee().public_key())
    }

    /// The headline consequence: a verifier needs one root per persona, not
    /// one master key. This asserts the set actually grows with them.
    #[test]
    fn the_trusted_set_carries_one_root_per_persona() {
        let (root, _seed, master) = seeded("m5-roots");
        let roots = wallet_trusted_roots(&root, master).unwrap();

        assert!(roots.iter().any(|r| r.authority == master));
        assert_eq!(roots.len(), 2, "the master root plus one persona");

        crate::custody::wallet::ensure_wallet_state(&root, second_persona(), "Studio PC").unwrap();
        let roots = wallet_trusted_roots(&root, master).unwrap();
        assert_eq!(roots.len(), 3, "a second persona adds a third root");
    }

    #[test]
    fn a_fresh_grant_stands_on_every_certificate() {
        let (root, _seed, master) = seeded("m5-fresh");
        let mut spec = sample_remote_auth_spec();
        spec.scopes = vec!["transport.egress".into(), "identity.act".into()];
        spec.issued_at_ms = NOW_MS;
        spec.expires_at_ms = Some(NOW_MS + 3_600_000);
        issue_remote_auth_device_grant(&root, &spec).unwrap();

        let standing =
            assess_device_grant(&root, spec.device_id, holder(), master, NOW_MS + 1_000).unwrap();

        assert!(standing.is_wholly_valid(), "{standing:?}");
        assert!(standing.device.is_some());
        assert_eq!(standing.personas.len(), 1);
    }

    #[test]
    fn a_revoked_grant_reports_revoked_rather_than_merely_failing() {
        let (root, seed, master) = seeded("m5-revoked");
        let mut spec = sample_remote_auth_spec();
        spec.issued_at_ms = NOW_MS;
        spec.expires_at_ms = Some(NOW_MS + 3_600_000);
        issue_remote_auth_device_grant(&root, &spec).unwrap();
        revoke_device_certificates(&root, seed, spec.device_id, NOW_MS + 10).unwrap();

        let standing =
            assess_device_grant(&root, spec.device_id, holder(), master, NOW_MS + 1_000).unwrap();

        assert!(!standing.holds_any_authority());
        assert_eq!(standing.standing_faults(), vec![ChainFault::Revoked]);
    }

    #[test]
    fn an_expired_grant_does_not_stand() {
        let (root, _seed, master) = seeded("m5-expired");
        let mut spec = sample_remote_auth_spec();
        spec.issued_at_ms = NOW_MS;
        spec.expires_at_ms = Some(NOW_MS + 1_000);
        issue_remote_auth_device_grant(&root, &spec).unwrap();

        let standing =
            assess_device_grant(&root, spec.device_id, holder(), master, NOW_MS + 5_000).unwrap();

        assert!(!standing.holds_any_authority());
    }

    /// A certificate is bound to the device that holds it. Presenting one from
    /// a different holder must fail even though its signature is perfectly
    /// good.
    #[test]
    fn a_certificate_does_not_travel_to_another_holder() {
        let (root, _seed, master) = seeded("m5-holder");
        let mut spec = sample_remote_auth_spec();
        spec.issued_at_ms = NOW_MS;
        spec.expires_at_ms = Some(NOW_MS + 3_600_000);
        issue_remote_auth_device_grant(&root, &spec).unwrap();

        let standing = assess_device_grant(
            &root,
            spec.device_id,
            DevicePublicKey([0x99; 32]),
            master,
            NOW_MS + 1_000,
        )
        .unwrap();

        assert!(!standing.holds_any_authority());
    }

    /// Without the persona's root in the trusted set, its certificate is a
    /// well-signed statement from a stranger. This is the failure mode the
    /// per-persona root exists to make possible.
    #[test]
    fn a_persona_certificate_needs_its_own_root_to_be_trusted() {
        let (root, _seed, master) = seeded("m5-untrusted");
        let mut spec = sample_remote_auth_spec();
        spec.scopes = vec!["identity.act".into()];
        spec.issued_at_ms = NOW_MS;
        spec.expires_at_ms = Some(NOW_MS + 3_600_000);
        issue_remote_auth_device_grant(&root, &spec).unwrap();

        let set = load_device_grant_set(&root, spec.device_id).unwrap();
        let certificate = set.personas.values().next().expect("a persona certificate");
        let ledger = load_revocation_ledger(&root).unwrap();
        let master_only = [TrustedRoot {
            authority: master,
            issuer: master,
        }];

        let verdict = validate_chain(
            std::slice::from_ref(certificate),
            holder().0,
            &master_only,
            &ledger,
            DEVICE_GRANT_DEPTH,
            NOW_MS + 1_000,
        );

        assert_eq!(verdict, Err(ChainFault::UntrustedRoot));
    }
}
