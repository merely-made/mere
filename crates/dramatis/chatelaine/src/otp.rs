// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! What a one-time password looks like, without the seed that makes one.
//!
//! [`OtpAlgorithm`] and [`OtpCodeStyle`] moved here from castellan, which
//! re-exports them. [`OtpKind`] is the shape alone: an HOTP counter is
//! mutable, freshness-critical state, so it stays with castellan's sealed
//! record.

use core::fmt;

use serde::{Deserialize, Serialize};

/// The HMAC hash behind a code.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OtpAlgorithm {
    /// SHA-1. The RFC 4226 original and what essentially every issuer uses.
    #[default]
    Sha1,
    /// SHA-256.
    Sha256,
    /// SHA-512.
    Sha512,
}

impl OtpAlgorithm {
    /// The spelling used in an `otpauth://` URI.
    pub fn as_uri_str(self) -> &'static str {
        match self {
            OtpAlgorithm::Sha1 => "SHA1",
            OtpAlgorithm::Sha256 => "SHA256",
            OtpAlgorithm::Sha512 => "SHA512",
        }
    }
}

impl fmt::Display for OtpAlgorithm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_uri_str())
    }
}

/// Characters used to present a generated code.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum OtpCodeStyle {
    /// RFC-style decimal output with an explicit width.
    Decimal {
        /// Number of decimal digits in the code.
        digits: u32,
    },
    /// Valve's five-character Steam Guard compatibility alphabet.
    SteamGuard,
}

impl OtpCodeStyle {
    /// Number of visible characters in a code of this style.
    pub fn character_count(self) -> u32 {
        match self {
            Self::Decimal { digits } => digits,
            Self::SteamGuard => 5,
        }
    }
}

/// Whether codes advance on a clock or on a counter.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OtpKind {
    /// Time-based (RFC 6238), stepping every `period` seconds from `t0`.
    Totp {
        /// Seconds per step.
        period: u64,
        /// The epoch the step count is measured from. Zero in practice.
        t0: u64,
    },
    /// Counter-based (RFC 4226). The counter itself is castellan's.
    Hotp,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn algorithms_keep_their_otpauth_spelling() {
        assert_eq!(OtpAlgorithm::default(), OtpAlgorithm::Sha1);
        let spelled: Vec<String> = [
            OtpAlgorithm::Sha1,
            OtpAlgorithm::Sha256,
            OtpAlgorithm::Sha512,
        ]
        .iter()
        .map(ToString::to_string)
        .collect();
        assert_eq!(spelled, ["SHA1", "SHA256", "SHA512"]);
    }

    #[test]
    fn a_steam_guard_code_is_five_characters() {
        assert_eq!(OtpCodeStyle::SteamGuard.character_count(), 5);
        assert_eq!(OtpCodeStyle::Decimal { digits: 8 }.character_count(), 8);
    }

    #[test]
    fn the_otp_enums_have_stable_text_names() {
        // CXF's own spelling for the hash.
        assert_eq!(
            serde_json::to_string(&OtpAlgorithm::Sha256).unwrap(),
            "\"sha256\""
        );
        assert_eq!(
            serde_json::to_string(&OtpCodeStyle::SteamGuard).unwrap(),
            "\"steam-guard\""
        );
        assert_eq!(
            serde_json::to_string(&OtpKind::Totp { period: 30, t0: 0 }).unwrap(),
            r#"{"totp":{"period":30,"t0":0}}"#
        );
        assert_eq!(serde_json::to_string(&OtpKind::Hotp).unwrap(), "\"hotp\"");
    }
}
