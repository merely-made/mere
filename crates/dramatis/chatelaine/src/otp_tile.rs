// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The keychain's secret-free OTP display: which credential a code is for,
//! the tile that shows one code, and its remaining-time ring.
//!
//! Moved from castellan's OTP gate (dramatis repo plan, ruling D29), which
//! re-exports these. The trust-bearing types, the release participant
//! claim and the release request, stay in castellan. A tile holds the code a
//! host may show and an absolute expiry for any renderer to draw the ring
//! and stop presenting a stale TOTP. It deliberately has no serialization:
//! carriers get no new code wire before a real carrier needs one.

use std::fmt;

use zeroize::Zeroizing;

use crate::{
    Credential, CredentialId, CredentialKind, Item, ItemId, OtpAlgorithm, OtpCodeStyle, OtpMode,
};

/// An item and the `Otp` credential in it that a petition or tile is about.
///
/// Built by [`OtpCredential::from_item`], which checks that the credential
/// is in the item and is an `Otp` credential. Every field may be shown.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OtpCredential {
    item: Item,
    credential: CredentialId,
}

/// The display fields of an `Otp` credential, borrowed from it.
#[derive(Clone, Copy)]
pub struct OtpFields<'a> {
    /// The account the codes are for.
    pub account: &'a str,
    /// The issuing service, when the import named one.
    pub issuer: Option<&'a str>,
    /// The HMAC hash behind each code.
    pub algorithm: OtpAlgorithm,
    /// How each code is written.
    pub code_style: OtpCodeStyle,
    /// Time-based with its period, or counter-based.
    pub mode: OtpMode,
}

impl<'a> OtpFields<'a> {
    /// The fields, when `credential` is an `Otp` credential.
    pub fn of(credential: &'a Credential) -> Option<Self> {
        match &credential.kind {
            CredentialKind::Otp {
                account,
                issuer,
                algorithm,
                code_style,
                mode,
            } => Some(Self {
                account,
                issuer: issuer.as_deref(),
                algorithm: *algorithm,
                code_style: *code_style,
                mode: *mode,
            }),
            _ => None,
        }
    }
}

impl OtpCredential {
    /// Pair an item with one of its credentials, if that credential is OTP.
    ///
    /// Public since the dramatis repo plan's ruling D29 moved it here from
    /// castellan; it still refuses a credential that is not the item's or not
    /// OTP.
    pub fn from_item(item: Item, credential: CredentialId) -> Option<Self> {
        let held = item
            .credentials
            .iter()
            .find(|candidate| candidate.id == credential)?;
        OtpFields::of(held)?;
        Some(Self { item, credential })
    }

    /// The whole item, every credential's metadata included.
    pub fn item(&self) -> &Item {
        &self.item
    }

    /// The item's id.
    pub fn item_id(&self) -> ItemId {
        self.item.id
    }

    /// The exercised credential's id.
    pub fn credential_id(&self) -> CredentialId {
        self.credential
    }

    /// The account the codes are for.
    pub fn account(&self) -> &str {
        self.fields().account
    }

    /// The issuing service, when the import named one.
    pub fn issuer(&self) -> Option<&str> {
        self.fields().issuer
    }

    /// The HMAC hash behind each code.
    pub fn algorithm(&self) -> OtpAlgorithm {
        self.fields().algorithm
    }

    /// How each code is written.
    pub fn code_style(&self) -> OtpCodeStyle {
        self.fields().code_style
    }

    /// Time-based with its period, or counter-based. The counter itself is
    /// sealed, never shown.
    pub fn mode(&self) -> OtpMode {
        self.fields().mode
    }

    fn fields(&self) -> OtpFields<'_> {
        self.item
            .credentials
            .iter()
            .find(|candidate| candidate.id == self.credential)
            .and_then(OtpFields::of)
            .expect("an OtpCredential is built only around an Otp credential it holds")
    }
}

/// One code prepared for an admitted host to show.
///
/// The code is intentionally not part of [`fmt::Debug`]. It is not a seed,
/// but it is still a credential valid for a short interval and belongs on a
/// screen, not in diagnostic output.
pub struct OtpCodeTile {
    credential: OtpCredential,
    code: Zeroizing<String>,
    time_ring: Option<OtpTimeRing>,
}

impl fmt::Debug for OtpCodeTile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OtpCodeTile")
            .field("credential", &self.credential)
            .field("code", &"<redacted>")
            .field("time_ring", &self.time_ring)
            .finish()
    }
}

impl OtpCodeTile {
    /// A tile for `code`, generated at `unix_secs`.
    ///
    /// castellan's OTP gate is what generates codes and builds tiles. Public
    /// since ruling D29 moved the tile here: a tile built anywhere else only
    /// fools its own builder's display, and grants nothing.
    pub fn new(credential: OtpCredential, code: String, unix_secs: u64) -> Self {
        let time_ring = match credential.mode() {
            OtpMode::Totp { period, t0 } => {
                let elapsed = unix_secs.saturating_sub(t0);
                let seconds_remaining = period - (elapsed % period);
                Some(OtpTimeRing {
                    period_seconds: period,
                    expires_at_unix_secs: unix_secs.saturating_add(seconds_remaining),
                })
            },
            OtpMode::Hotp => None,
        };
        Self {
            credential,
            code: Zeroizing::new(code),
            time_ring,
        }
    }

    /// The secret-free item and credential, suitable for the tile label.
    pub fn credential(&self) -> &OtpCredential {
        &self.credential
    }

    /// The code when it is still current at `unix_secs`.
    ///
    /// HOTP values have no clock expiry and are always returned. TOTP values
    /// disappear at the absolute step boundary carried by their time ring.
    pub fn code_at_unix_time(&self, unix_secs: u64) -> Option<&str> {
        if self
            .time_ring
            .is_some_and(|ring| ring.is_expired_at(unix_secs))
        {
            None
        } else {
            Some(self.code.as_str())
        }
    }

    /// Remaining-time facts for a TOTP code, or `None` for HOTP.
    pub fn time_ring(&self) -> Option<OtpTimeRing> {
        self.time_ring
    }
}

/// Integer facts for a TOTP tile's remaining-time ring.
///
/// Renderers choose their own geometry and motion. The absolute expiry keeps
/// carrier delay and redraw cadence from extending the code's presentation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OtpTimeRing {
    /// The TOTP period selected by the issuer.
    pub period_seconds: u64,
    /// Unix second at which this code is replaced and must stop being shown.
    pub expires_at_unix_secs: u64,
}

impl OtpTimeRing {
    /// Whole seconds remaining at the supplied Unix time.
    pub fn seconds_remaining_at(self, unix_secs: u64) -> u64 {
        self.expires_at_unix_secs
            .saturating_sub(unix_secs)
            .min(self.period_seconds)
    }

    /// Whether the code has reached its absolute step boundary.
    pub fn is_expired_at(self, unix_secs: u64) -> bool {
        unix_secs >= self.expires_at_unix_secs
    }

    /// Whole seconds elapsed within this code's period.
    pub fn elapsed_seconds_at(self, unix_secs: u64) -> u64 {
        self.period_seconds
            .saturating_sub(self.seconds_remaining_at(unix_secs))
    }

    /// Completed ring fraction, quantized to 0 through 1000.
    pub fn completed_per_mille_at(self, unix_secs: u64) -> u16 {
        if self.period_seconds == 0 {
            return 0;
        }
        let completed = self
            .elapsed_seconds_at(unix_secs)
            .saturating_mul(1_000)
            .checked_div(self.period_seconds)
            .unwrap_or(0);
        completed.min(1_000) as u16
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Credential, CredentialId, CredentialKind, Item, ItemId, ItemState};

    fn item(mode: OtpMode) -> OtpCredential {
        let credential = CredentialId::from_bytes([0x45; 16]);
        let item = Item {
            id: ItemId::from_bytes([0x44; 16]),
            source_id: None,
            title: "Merely".to_string(),
            subtitle: Some("mark".to_string()),
            scope: None,
            tags: Vec::new(),
            favorite: false,
            created_at: None,
            modified_at: None,
            credentials: vec![Credential {
                id: credential,
                kind: CredentialKind::Otp {
                    account: "mark".to_string(),
                    issuer: Some("Merely".to_string()),
                    algorithm: OtpAlgorithm::Sha1,
                    code_style: OtpCodeStyle::Decimal { digits: 6 },
                    mode,
                },
            }],
            state: ItemState::Vault,
        };
        OtpCredential::from_item(item, credential).unwrap()
    }

    #[test]
    fn totp_tile_describes_the_remaining_time_ring_without_exposing_its_code_in_debug() {
        let tile = OtpCodeTile::new(
            item(OtpMode::Totp { period: 30, t0: 0 }),
            "123456".to_string(),
            29,
        );

        assert_eq!(tile.code_at_unix_time(29), Some("123456"));
        assert_eq!(
            tile.time_ring(),
            Some(OtpTimeRing {
                period_seconds: 30,
                expires_at_unix_secs: 30,
            })
        );
        assert_eq!(tile.time_ring().unwrap().seconds_remaining_at(29), 1);
        assert_eq!(tile.time_ring().unwrap().elapsed_seconds_at(29), 29);
        assert_eq!(tile.time_ring().unwrap().completed_per_mille_at(29), 966);
        assert_eq!(tile.code_at_unix_time(30), None);
        assert!(!format!("{tile:?}").contains("123456"));
    }

    #[test]
    fn hotp_tile_has_no_countdown_ring() {
        let tile = OtpCodeTile::new(item(OtpMode::Hotp), "123456".to_string(), 59);

        assert_eq!(tile.time_ring(), None);
        assert_eq!(tile.code_at_unix_time(u64::MAX), Some("123456"));
    }
}
