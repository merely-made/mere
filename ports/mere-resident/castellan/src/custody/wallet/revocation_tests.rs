// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! pandect's `wallet_grant::revocation` tests. They issue grants with the
//! wallet's seed, custody since the dramatis repo plan's DR-B, so they moved
//! with it.

mod tests {
    use super::super::test_support::*;
    use super::super::*;

    const AT_MS: u64 = 1_750_000_000_000;

    fn seeded_root(tag: &str) -> (std::path::PathBuf, [u8; 32]) {
        let root = temp_data_root(tag);
        let seed =
            crate::custody::wallet::ensure_wallet_state(&root, fixture_persona(), "Studio PC")
                .unwrap();
        (root, seed)
    }

    #[test]
    fn an_unwritten_ledger_loads_empty() {
        let root = tempfile::tempdir().unwrap();
        assert!(load_revocation_ledger(root.path()).unwrap().is_empty());
    }

    #[test]
    fn revoking_a_device_withdraws_every_certificate_it_held() {
        let (root, seed) = seeded_root("m4-revoke");
        let spec = sample_remote_auth_spec();
        let set = issue_remote_auth_device_grant(&root, &spec).unwrap();

        let statements = revoke_device_certificates(&root, seed, spec.device_id, AT_MS).unwrap();
        assert_eq!(statements.len(), set.certificates().count());
        assert!(device_is_fully_revoked(&root, spec.device_id).unwrap());
    }

    #[test]
    fn the_ledger_survives_a_round_trip_through_disk() {
        let (root, seed) = seeded_root("m4-persist");
        let spec = sample_remote_auth_spec();
        issue_remote_auth_device_grant(&root, &spec).unwrap();
        revoke_device_certificates(&root, seed, spec.device_id, AT_MS).unwrap();

        let reloaded = load_revocation_ledger(&root).unwrap();
        assert!(!reloaded.is_empty());
        assert!(device_is_fully_revoked(&root, spec.device_id).unwrap());
    }

    /// A statement is worth its signature and nothing else. This is what lets
    /// revocations travel: a peer folds what verifies and ignores the rest.
    #[test]
    fn a_tampered_statement_is_refused_by_the_fold() {
        let (root, seed) = seeded_root("m4-tamper");
        let spec = sample_remote_auth_spec();
        issue_remote_auth_device_grant(&root, &spec).unwrap();
        let set = load_device_grant_set(&root, spec.device_id).unwrap();
        let mut statements = personae::carry::revoke_device_grant_set(seed, &set, AT_MS).unwrap();

        statements[0].revocation.at_ms = AT_MS + 1;
        let accepted = fold_revocations(&root, &statements[..1]).unwrap();

        assert_eq!(accepted, 0, "an altered statement must not fold");
        assert!(!device_is_fully_revoked(&root, spec.device_id).unwrap());
    }

    /// Partial revocation must stay visible. Withdrawing one persona's
    /// authority is exactly the operation the old single-signature grant
    /// could not express.
    #[test]
    fn withdrawing_one_certificate_does_not_revoke_the_device() {
        let (root, seed) = seeded_root("m4-partial");
        let mut spec = sample_remote_auth_spec();
        spec.scopes = vec!["transport.egress".into(), "identity.act".into()];
        issue_remote_auth_device_grant(&root, &spec).unwrap();
        let set = load_device_grant_set(&root, spec.device_id).unwrap();
        assert!(set.device.is_some() && !set.personas.is_empty());

        let statements = personae::carry::revoke_device_grant_set(seed, &set, AT_MS).unwrap();
        let persona_only: Vec<_> = statements
            .into_iter()
            .filter(|statement| {
                set.personas
                    .values()
                    .any(|c| c.certificate.id() == statement.revocation.certificate)
            })
            .collect();
        assert_eq!(fold_revocations(&root, &persona_only).unwrap(), 1);

        assert_eq!(revoked_certificate_count(&root, &set).unwrap(), 1);
        assert!(
            !device_is_fully_revoked(&root, spec.device_id).unwrap(),
            "transport authority still stands, so the device is not revoked"
        );
    }
}
