// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Quarantine state changes under the persona transaction, without release.
use super::items::Payload;
use super::items::records::StoredPayload;
use super::items::{ItemStore, ItemStoreError};
use chatelaine::{CredentialKind as K, Item, ItemId, ItemState, OtpCodeStyle};

/// Identifies the precise metadata reviewed by the owner.
pub fn revision(item: &Item) -> String {
    blake3::hash(&serde_json::to_vec(item).expect("public metadata serializes"))
        .to_hex()
        .to_string()
}
/// Exact public metadata of the whole older item namespace being assigned.
pub fn namespace_revision(items: &[Item]) -> String {
    blake3::hash(&serde_json::to_vec(items).expect("public metadata serializes"))
        .to_hex()
        .to_string()
}
/// A reason that acceptance would leave an unusable credential active.
/// This reads only metadata. Acceptance separately authenticates payloads.
pub fn accept_blocked(item: &Item) -> Option<String> {
    for c in &item.credentials {
        let reason = match &c.kind {
            K::Unknown { .. } => Some("This credential type is not supported."),
            K::RefusedCxf { .. } => Some("This credential needs a corrected import."),
            K::Passkey { .. } => Some("Passkey custody is not available."),
            K::File { .. } => Some("File custody is not available."),
            K::Otp {
                code_style: OtpCodeStyle::Decimal { digits },
                ..
            } if !(6..=10).contains(digits) => {
                Some("OTP digit counts outside 6–10 remain quarantined.")
            },
            K::Address
            | K::ApiKey { .. }
            | K::BasicAuth { .. }
            | K::CreditCard { .. }
            | K::CustomFields { .. }
            | K::DriversLicense { .. }
            | K::GeneratedPassword
            | K::IdentityDocument { .. }
            | K::ItemReference(_)
            | K::Note
            | K::Passport { .. }
            | K::PersonName
            | K::Otp { .. }
            | K::Wifi { .. }
            | K::Secret { .. } => None,
            // Future kinds require an explicit custody decision before activation.
            _ => Some("This credential type is not supported."),
        };
        if let Some(reason) = reason {
            return Some(reason.into());
        }
    }
    None
}
impl ItemStore {
    /// Atomically change the exact quarantined item named by resident consent.
    /// Caller holds vault authority; neither approval nor secrets come from UI.
    pub fn review(&self, id: ItemId, expected: &str, accept: bool) -> Result<(), ItemStoreError> {
        let transaction = self.begin();
        let mut item = transaction
            .get(id)?
            .ok_or(ItemStoreError::ItemNotFound(id))?;
        if item.state != ItemState::Quarantined || revision(&item) != expected {
            return Err(ItemStoreError::InvalidItem("item changed; review it again"));
        }
        if !accept {
            return transaction.delete(id);
        }
        if accept_blocked(&item).is_some() {
            return Err(ItemStoreError::InvalidItem(
                "credential remains unsupported",
            ));
        }
        for credential in &item.credentials {
            if matches!(credential.kind, K::ItemReference(_)) {
                continue;
            }
            let record: StoredPayload = self
                .storage
                .load_record(self.payload_path(credential.id))?
                .ok_or(ItemStoreError::InvalidItem("credential payload is missing"))?;
            self.check_payload(id, credential.id, &record)?;
            match (&credential.kind, &record.payload) {
                (K::Otp { .. }, Payload::Otp { secret, .. }) if !secret.is_empty() => {},
                (K::Secret { .. }, Payload::Secret { .. }) => {},
                (_, Payload::Secret { bytes }) => {
                    let typed: credential_exchange_format::Credential =
                        serde_json::from_slice(bytes).map_err(|_| {
                            ItemStoreError::InvalidItem("credential needs a corrected import")
                        })?;
                    use credential_exchange_format::Credential as C;
                    let matching = matches!(
                        (&credential.kind, &typed),
                        (K::Address, C::Address(_))
                            | (K::ApiKey { .. }, C::ApiKey(_))
                            | (K::BasicAuth { .. }, C::BasicAuth(_))
                            | (K::CreditCard { .. }, C::CreditCard(_))
                            | (K::CustomFields { .. }, C::CustomFields(_))
                            | (K::DriversLicense { .. }, C::DriversLicense(_))
                            | (K::GeneratedPassword, C::GeneratedPassword(_))
                            | (K::IdentityDocument { .. }, C::IdentityDocument(_))
                            | (K::Note, C::Note(_))
                            | (K::Passport { .. }, C::Passport(_))
                            | (K::PersonName, C::PersonName(_))
                            | (K::Wifi { .. }, C::Wifi(_))
                    );
                    let malformed = !matching;
                    use crate::cxf::erase::Erase;
                    typed.erase();
                    if malformed {
                        return Err(ItemStoreError::InvalidItem(
                            "credential needs a corrected import",
                        ));
                    }
                },
                _ => {
                    return Err(ItemStoreError::InvalidItem(
                        "credential payload does not match its kind",
                    ));
                },
            }
        }
        item.state = ItemState::Vault;
        transaction.save_metadata(item)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        otp::{OtpItemStore, OtpReleaseGate, OtpReleaseParticipantClaim},
        resident::CastellanResident,
    };
    use personae::PersonaId;
    #[test]
    fn supported_acceptance_opens_the_existing_exercise_gate_without_replacing_payload() {
        let dir = tempfile::tempdir().unwrap();
        let resident = CastellanResident::claim(
            dir.path().join("records"),
            [1; 32],
            dir.path().join("freshness"),
            [2; 32],
        )
        .unwrap();
        let store = resident.items(PersonaId::new());
        let otp = OtpItemStore::over(store.clone())
            .import_otpauth_uri(
                "otpauth://hotp/Receipt:owner?secret=GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ&counter=0",
            )
            .unwrap();
        let id = otp.item_id();
        let mut item = store.get(id).unwrap().unwrap();
        item.state = ItemState::Quarantined;
        store.begin().save_metadata(item.clone()).unwrap();
        let gate = OtpReleaseGate::new(OtpItemStore::over(store.clone()));
        let participant =
            || OtpReleaseParticipantClaim::unverified("local:receipt", "review:receipt").unwrap();
        assert!(
            gate.petition(id, otp.credential_id(), participant())
                .is_err()
        );
        let path = store.payload_path(otp.credential_id());
        let sealed = std::fs::read(dir.path().join("records").join(&path)).unwrap();
        store.review(id, &revision(&item), true).unwrap();
        assert_eq!(
            sealed,
            std::fs::read(dir.path().join("records").join(&path)).unwrap()
        );
        let petition = gate
            .petition(id, otp.credential_id(), participant())
            .unwrap();
        assert_eq!(
            gate.approve(petition.id)
                .unwrap()
                .tile()
                .code_at_unix_time(0),
            Some("755224")
        );
    }
}
