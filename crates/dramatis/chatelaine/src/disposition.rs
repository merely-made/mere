// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! What import does with each kind: rulings 10 to 15 and 25.

use serde::{Deserialize, Serialize};

use crate::kind::CredentialKind;

/// The ruled import treatment of a credential kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Disposition {
    /// Sealed into the vault as a chatelaine kind, exercisable at once.
    Stored,
    /// Sealed, never exercised, autofilled or shown, until the user accepts
    /// the item (invariant 12).
    Quarantined,
    /// Handed to castellan's native SSH import, into personae's SSH slots,
    /// so SSH has one home.
    RoutedToSshImport,
    /// Kept as a link between items, not stored as an item; a dangling one
    /// is reported.
    KeptAsLink,
}

/// The ruled import treatment for `kind`.
///
/// A Secret Service secret is never imported from CXF; it is
/// [`Disposition::Stored`] because the Secret Service keeps its items in the
/// vault (ruling 17).
pub fn disposition(kind: &CredentialKind) -> Disposition {
    use CredentialKind as K;
    match kind {
        // Ruling 10; totp lands on castellan's RFC 6238 items.
        K::BasicAuth { .. }
        | K::GeneratedPassword
        | K::Totp { .. }
        | K::ApiKey { .. }
        | K::Wifi { .. } => Disposition::Stored,
        // Ruling 10, every algorithm since ruling 25.
        K::SshKey { .. } => Disposition::RoutedToSshImport,
        // Ruling 12: autofill kinds, exercised only by filling forms.
        K::Address | K::PersonName => Disposition::Stored,
        // Ruling 14.
        K::Note | K::CustomFields { .. } => Disposition::Stored,
        K::ItemReference(_) => Disposition::KeptAsLink,
        // Ruling 11.
        K::Passport { .. }
        | K::DriversLicense { .. }
        | K::IdentityDocument { .. }
        | K::CreditCard { .. } => Disposition::Quarantined,
        // Ruling 13: until a passkey provider and blob custody exist.
        K::Passkey { .. } | K::File { .. } => Disposition::Quarantined,
        // Ruling 15, fields preserved verbatim by castellan.
        K::Unknown { .. } => Disposition::Quarantined,
        // Ruling 17.
        K::Secret { .. } => Disposition::Stored,
    }
}

/// A test per row of the tier architecture's §7 import table.
#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;
    use crate::{CXF_V1_TYPES, samples};

    const CORE: &str = "basic-auth, generated-password, totp, api-key, wifi";
    const SSH: &str = "ssh-key";
    const AUTOFILL: &str = "address, person-name";
    const FREEFORM: &str = "note, custom-fields";
    const REFERENCE: &str = "item-reference";
    const DOCUMENTS: &str = "passport, drivers-license, identity-document, credit-card";
    const UNUSABLE: &str = "passkey, file";
    const NEWER: &str = "types newer than CXF v1.0";

    fn ssh_key(key_type: &str) -> CredentialKind {
        let mut kind = samples::ssh_key();
        if let CredentialKind::SshKey { key_type: held, .. } = &mut kind {
            *held = key_type.to_string();
        }
        kind
    }

    /// The table: each row's label, a sample of each kind in it, and its
    /// ruled treatment.
    fn table() -> Vec<(&'static str, Vec<CredentialKind>, Disposition)> {
        use Disposition::*;
        vec![
            (
                CORE,
                vec![
                    samples::basic_auth(),
                    samples::generated_password(),
                    samples::totp(),
                    samples::hotp(),
                    samples::steam_guard(),
                    samples::api_key(),
                    samples::wifi(),
                ],
                Stored,
            ),
            (
                SSH,
                vec![
                    ssh_key("ssh-ed25519"),
                    ssh_key("ssh-rsa"),
                    ssh_key("ecdsa-sha2-nistp256"),
                ],
                RoutedToSshImport,
            ),
            (
                AUTOFILL,
                vec![samples::address(), samples::person_name()],
                Stored,
            ),
            (
                FREEFORM,
                vec![samples::note(), samples::custom_fields()],
                Stored,
            ),
            (REFERENCE, vec![samples::item_reference()], KeptAsLink),
            (
                DOCUMENTS,
                vec![
                    samples::passport(),
                    samples::drivers_license(),
                    samples::identity_document(),
                    samples::credit_card(),
                ],
                Quarantined,
            ),
            (
                UNUSABLE,
                vec![samples::passkey(), samples::file()],
                Quarantined,
            ),
            (NEWER, vec![samples::unknown()], Quarantined),
        ]
    }

    fn row(label: &str) {
        let (_, kinds, expected) = table().into_iter().find(|(row, ..)| *row == label).unwrap();
        for kind in &kinds {
            assert_eq!(disposition(kind), expected, "{kind:?}");
        }
    }

    #[test]
    fn core_credentials_are_stored() {
        row(CORE);
    }

    #[test]
    fn every_ssh_key_routes_to_the_ssh_import() {
        row(SSH);
    }

    #[test]
    fn autofill_kinds_are_stored() {
        row(AUTOFILL);
    }

    #[test]
    fn notes_and_custom_fields_are_stored() {
        row(FREEFORM);
    }

    #[test]
    fn item_references_are_kept_as_links() {
        row(REFERENCE);
    }

    #[test]
    fn identity_documents_and_cards_are_quarantined() {
        row(DOCUMENTS);
    }

    #[test]
    fn passkeys_and_files_are_quarantined() {
        row(UNUSABLE);
    }

    #[test]
    fn types_newer_than_v1_are_quarantined() {
        row(NEWER);
    }

    #[test]
    fn a_secret_service_secret_is_stored() {
        assert_eq!(disposition(&samples::secret()), Disposition::Stored);
    }

    #[test]
    fn the_rows_cover_every_cxf_v1_type_once() {
        let mut seen = BTreeSet::new();
        for (label, kinds, _) in table().into_iter().filter(|(row, ..)| *row != NEWER) {
            let row: BTreeSet<&str> = kinds.iter().map(|k| k.cxf_type().unwrap()).collect();
            assert_eq!(
                row,
                label.split(", ").collect(),
                "row {label} lists its kinds"
            );
            for cxf_type in row {
                assert!(
                    seen.insert(cxf_type.to_string()),
                    "{cxf_type} is in two rows"
                );
            }
        }
        assert_eq!(seen, CXF_V1_TYPES.iter().map(|t| t.to_string()).collect());
    }
}
