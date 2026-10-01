// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! One-time passwords as chatelaine items in a persona's item store.
//!
//! An OTP is an item holding one `Otp` credential (ruling 16): its account,
//! issuer, hash, code style and mode are chatelaine metadata, and its seed,
//! with an HOTP item's next counter, is the credential's sealed payload
//! (ruling 23). [`OtpItemStore`] imports `otpauth://` URIs and Steam Guard
//! secrets into that shape and reads the secret-free metadata back; no type
//! here holds or returns a seed.
//!
//! This is the storage seam below [`super::OtpReleaseGate`], which is the only
//! public path that can turn a sealed payload into a code-bearing tile.

use std::fmt;

use chatelaine::{
    Credential, CredentialId, CredentialKind, Item, ItemId, ItemState, OtpAlgorithm, OtpCodeStyle,
    OtpMode,
};
use personae::{IdentityError, PersonaId, SealedRecordStorage};

use super::credential::OtpFields;
use super::steam_guard::decode_shared_secret;
use super::{
    Otp, OtpCodeTile, OtpCredential, OtpError, OtpKind, OtpUriError, SteamGuard, SteamGuardError,
    parse_otpauth_uri,
};
use crate::items::{ItemStore, ItemStoreError, Payload};

/// The issuer every Steam Guard item is filed under.
const STEAM_ISSUER: &str = "Steam";
/// Valve's fixed Steam Guard step.
const STEAM_PERIOD_SECS: u64 = 30;

/// Failure while importing, reading or exercising an OTP credential.
#[derive(Debug)]
pub enum OtpItemError {
    /// The item store could not read, write or exercise a record.
    Store(ItemStoreError),
    /// The supplied provisioning URI was malformed or unsupported.
    Import(OtpUriError),
    /// Steam Guard compatibility material was malformed.
    SteamGuard(SteamGuardError),
    /// A stored generator could not produce a code.
    Generation(OtpError),
    /// This persona's store holds no OTP credential at this address.
    NotFound {
        /// The item asked for.
        item: ItemId,
        /// The credential asked for.
        credential: CredentialId,
    },
    /// The sealed payload does not fit its credential's metadata.
    PayloadMismatch,
    /// The final HOTP value cannot be released because it could not be
    /// advanced before the replacement record could be persisted.
    HotpCounterExhausted,
}

impl fmt::Display for OtpItemError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OtpItemError::Store(error) => write!(f, "OTP item store: {error}"),
            OtpItemError::Import(error) => write!(f, "OTP import: {error}"),
            OtpItemError::SteamGuard(error) => write!(f, "Steam Guard import: {error}"),
            OtpItemError::Generation(error) => write!(f, "OTP generation: {error}"),
            OtpItemError::NotFound { item, credential } => write!(
                f,
                "no OTP credential {credential} in item {item} for this persona"
            ),
            OtpItemError::PayloadMismatch => {
                f.write_str("the sealed OTP payload does not match its credential")
            },
            OtpItemError::HotpCounterExhausted => {
                f.write_str("the HOTP counter has no next durable value")
            },
        }
    }
}

impl std::error::Error for OtpItemError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            OtpItemError::Store(error) => Some(error),
            OtpItemError::Import(error) => Some(error),
            OtpItemError::SteamGuard(error) => Some(error),
            OtpItemError::Generation(error) => Some(error),
            OtpItemError::NotFound { .. }
            | OtpItemError::PayloadMismatch
            | OtpItemError::HotpCounterExhausted => None,
        }
    }
}

impl From<ItemStoreError> for OtpItemError {
    fn from(error: ItemStoreError) -> Self {
        Self::Store(error)
    }
}

impl From<IdentityError> for OtpItemError {
    fn from(error: IdentityError) -> Self {
        Self::Store(ItemStoreError::Storage(error))
    }
}

impl From<SteamGuardError> for OtpItemError {
    fn from(error: SteamGuardError) -> Self {
        Self::SteamGuard(error)
    }
}

impl From<OtpError> for OtpItemError {
    fn from(error: OtpError) -> Self {
        Self::Generation(error)
    }
}

/// The OTP credentials of one persona's item store.
#[derive(Clone)]
pub struct OtpItemStore {
    items: ItemStore,
}

impl OtpItemStore {
    /// Open one persona's OTP credentials over an unlocked record store.
    pub fn new(storage: SealedRecordStorage, persona: PersonaId) -> Self {
        Self::over(ItemStore::new(storage, persona))
    }

    pub(crate) fn over(items: ItemStore) -> Self {
        Self { items }
    }

    /// The persona whose items this store serves.
    pub fn persona(&self) -> PersonaId {
        self.items.persona()
    }

    /// The item store these OTP credentials live in.
    pub fn items(&self) -> &ItemStore {
        &self.items
    }

    /// Parse and seal a provisioning URI as a new item.
    pub fn import_otpauth_uri(&self, uri: &str) -> Result<OtpCredential, OtpItemError> {
        let (otp, imported) = parse_otpauth_uri(uri).map_err(OtpItemError::Import)?;
        let (mode, counter) = match otp.kind {
            OtpKind::Totp { period, t0 } => (OtpMode::Totp { period, t0 }, None),
            OtpKind::Hotp { counter } => (OtpMode::Hotp, Some(counter)),
        };
        let payload = Payload::Otp {
            secret: otp.secret.to_vec(),
            counter,
        };
        let titles = titles(&imported.account, imported.issuer.as_deref());
        let metadata = CredentialKind::Otp {
            account: imported.account,
            issuer: imported.issuer,
            algorithm: otp.algorithm,
            code_style: OtpCodeStyle::Decimal { digits: otp.digits },
            mode,
        };
        self.insert(titles, metadata, payload)
    }

    /// Seal one Valve Steam Guard mobile authenticator `shared_secret`.
    ///
    /// The base64 value comes from a Steam authenticator file. It is not an
    /// `otpauth://` extension and is always presented as a five-character
    /// Steam Guard code under the fixed `Steam` issuer.
    pub fn import_steam_guard(
        &self,
        account: &str,
        shared_secret: &str,
    ) -> Result<OtpCredential, OtpItemError> {
        if account.is_empty()
            || account != account.trim()
            || account.len() > 256
            || account.chars().any(char::is_control)
        {
            return Err(SteamGuardError::InvalidAccount.into());
        }
        let secret = decode_shared_secret(shared_secret)?;
        let payload = Payload::Otp {
            secret: secret.to_vec(),
            counter: None,
        };
        let metadata = CredentialKind::Otp {
            account: account.to_string(),
            issuer: Some(STEAM_ISSUER.to_string()),
            algorithm: OtpAlgorithm::Sha1,
            code_style: OtpCodeStyle::SteamGuard,
            mode: OtpMode::Totp {
                period: STEAM_PERIOD_SECS,
                t0: 0,
            },
        };
        self.insert(titles(account, Some(STEAM_ISSUER)), metadata, payload)
    }

    /// Read one OTP credential's secret-free metadata, or `None` when this
    /// persona holds no OTP credential at that address.
    pub fn get(
        &self,
        item: ItemId,
        credential: CredentialId,
    ) -> Result<Option<OtpCredential>, OtpItemError> {
        Ok(self
            .items
            .get(item)?
            .and_then(|item| OtpCredential::from_item(item, credential)))
    }

    /// Exercise one credential after a participant-gated approval.
    ///
    /// TOTP payloads are unchanged. HOTP advances its sealed counter, in the
    /// payload record alone, before returning a code.
    pub(crate) fn release_tile_at_unix_time(
        &self,
        item: ItemId,
        credential: CredentialId,
        unix_secs: u64,
    ) -> Result<OtpCodeTile, OtpItemError> {
        self.items
            .exercise(item, credential, |stored, held, payload| {
                let fields =
                    OtpFields::of(held).ok_or(OtpItemError::NotFound { item, credential })?;
                let Payload::Otp { secret, counter } = payload;
                let code = match (fields.mode, counter.as_mut()) {
                    (OtpMode::Hotp, Some(counter)) => {
                        if *counter == u64::MAX {
                            return Err(OtpItemError::HotpCounterExhausted);
                        }
                        let code = code_at(fields, secret, Some(*counter), unix_secs)?;
                        *counter += 1;
                        code
                    },
                    (OtpMode::Totp { .. }, None) => code_at(fields, secret, None, unix_secs)?,
                    _ => return Err(OtpItemError::PayloadMismatch),
                };
                let changed = fields.mode == OtpMode::Hotp;
                let shown = OtpCredential::from_item(stored.clone(), credential)
                    .expect("the exercised credential is an Otp credential of this item");
                Ok((OtpCodeTile::new(shown, code, unix_secs), changed))
            })
    }

    /// Return seconds before a time-based code rolls over, or `None` for
    /// HOTP. Reads metadata only.
    pub fn seconds_remaining_at(
        &self,
        item: ItemId,
        credential: CredentialId,
        unix_secs: u64,
    ) -> Result<Option<u64>, OtpItemError> {
        let otp = self
            .get(item, credential)?
            .ok_or(OtpItemError::NotFound { item, credential })?;
        match otp.mode() {
            OtpMode::Totp { period: 0, .. } => Err(OtpError::ZeroPeriod.into()),
            OtpMode::Totp { period, t0 } => Ok(unix_secs
                .checked_sub(t0)
                .map(|elapsed| period - (elapsed % period))),
            OtpMode::Hotp => Ok(None),
        }
    }

    /// Remove one item, its OTP credential's payload included.
    ///
    /// Deleting an absent item succeeds, matching Personae's record-store
    /// deletion semantics.
    pub fn delete(&self, item: ItemId) -> Result<(), OtpItemError> {
        Ok(self.items.delete(item)?)
    }

    fn insert(
        &self,
        (title, subtitle): (String, Option<String>),
        metadata: CredentialKind,
        payload: Payload,
    ) -> Result<OtpCredential, OtpItemError> {
        let credential = CredentialId::from_random(random_id_bytes());
        let item = Item {
            id: ItemId::from_random(random_id_bytes()),
            source_id: None,
            title,
            subtitle,
            scope: None,
            tags: Vec::new(),
            favorite: false,
            created_at: None,
            modified_at: None,
            credentials: vec![Credential {
                id: credential,
                kind: metadata,
            }],
            state: ItemState::Vault,
        };
        let item = self.items.insert(item, vec![(credential, payload)])?;
        Ok(OtpCredential::from_item(item, credential)
            .expect("an imported OTP item holds its Otp credential"))
    }
}

/// An imported item's title and subtitle: the issuer with the account under
/// it, or the account alone (ruling 42).
fn titles(account: &str, issuer: Option<&str>) -> (String, Option<String>) {
    match issuer {
        Some(issuer) => (issuer.to_string(), Some(account.to_string())),
        None => (account.to_string(), None),
    }
}

/// The code for one exercise, from metadata and the sealed seed.
fn code_at(
    fields: OtpFields<'_>,
    secret: &[u8],
    counter: Option<u64>,
    unix_secs: u64,
) -> Result<String, OtpItemError> {
    if fields.code_style == OtpCodeStyle::SteamGuard {
        return Ok(SteamGuard::from_secret_bytes(secret)?.code_at_unix_time(unix_secs));
    }
    let OtpCodeStyle::Decimal { digits } = fields.code_style else {
        return Err(OtpItemError::PayloadMismatch);
    };
    let otp = match (fields.mode, counter) {
        (OtpMode::Totp { period, t0 }, None) => {
            let mut otp = Otp::totp(secret.to_vec())?.with_period(period)?;
            otp.kind = OtpKind::Totp { period, t0 };
            otp
        },
        (OtpMode::Hotp, Some(counter)) => Otp::hotp(secret.to_vec(), counter)?,
        _ => return Err(OtpItemError::PayloadMismatch),
    };
    let otp = otp.with_digits(digits)?.with_algorithm(fields.algorithm);
    Ok(otp.code_at_unix_time(unix_secs)?)
}

/// Sixteen random bytes for a new item or credential id.
fn random_id_bytes() -> [u8; 16] {
    uuid::Uuid::new_v4().into_bytes()
}

#[cfg(test)]
#[path = "item_tests.rs"]
mod tests;
