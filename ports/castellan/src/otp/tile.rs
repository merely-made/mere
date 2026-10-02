// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Embeddable, seed-free OTP code tiles.
//!
//! A tile is a short-lived presentation result from the resident authority.
//! It holds the code a host is allowed to show, ordinary item metadata, and
//! an absolute expiry for any renderer to draw the remaining-time ring and
//! stop presenting a stale TOTP. It deliberately has no serialization or
//! public constructor: carriers do not get a new code wire before an actual
//! carrier needs one.

use std::fmt;

use chatelaine::OtpMode;
use zeroize::Zeroizing;

use super::OtpCredential;

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
    pub(crate) fn new(credential: OtpCredential, code: String, unix_secs: u64) -> Self {
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
    use crate::otp::{OtpAlgorithm, OtpCodeStyle};
    use chatelaine::{Credential, CredentialId, CredentialKind, Item, ItemId, ItemState};

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
