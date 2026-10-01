// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Metadata values with a closed form, checked when made and when loaded.
//!
//! Each is text on the wire in every format, written as CXF writes it, and
//! a value that fails its check does not deserialize.

use core::fmt;
use core::str::FromStr;

use serde::de::{self, Deserializer};
use serde::{Deserialize, Serialize, Serializer};

/// Why text was not a metadata value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ValueError {
    /// Not exactly four ASCII digits.
    LastFour,
    /// Not two uppercase ASCII letters (ISO 3166-1 alpha-2).
    CountryCode,
    /// Not a country code, a hyphen and one to three uppercase ASCII letters
    /// or digits (ISO 3166-2).
    SubdivisionCode,
    /// Not a real calendar date written `YYYY-MM-DD`.
    Date,
    /// Not a year and month written `YYYY-MM`.
    YearMonth,
    /// Not `SHA256:` and 43 characters of unpadded base64.
    SshFingerprint,
}

impl fmt::Display for ValueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::LastFour => "last four is exactly four ASCII digits",
            Self::CountryCode => "a country code is two uppercase letters (ISO 3166-1 alpha-2)",
            Self::SubdivisionCode => {
                "a subdivision code is a country code, a hyphen and 1 to 3 letters or digits"
            },
            Self::Date => "a date is a real calendar date written YYYY-MM-DD",
            Self::YearMonth => "a year-month is written YYYY-MM",
            Self::SshFingerprint => {
                "an SSH fingerprint is SHA256: and 43 unpadded base64 characters"
            },
        })
    }
}

impl core::error::Error for ValueError {}

/// Text forms: `FromStr` checks, `Display` writes, serde goes through both.
macro_rules! text_value {
    ($name:ident) => {
        impl FromStr for $name {
            type Err = ValueError;

            fn from_str(text: &str) -> Result<Self, ValueError> {
                Self::parse(text)
            }
        }

        impl Serialize for $name {
            fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                serializer.collect_str(self)
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                let text = String::deserialize(deserializer)?;
                Self::parse(&text).map_err(de::Error::custom)
            }
        }
    };
}

/// A payment card's last four digits, derived by castellan at import.
///
/// chatelaine never sees the full number, so it offers no way to derive one.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LastFour([u8; 4]);

impl LastFour {
    /// Accept exactly four ASCII digits.
    pub fn parse(text: &str) -> Result<Self, ValueError> {
        let digits: [u8; 4] = text
            .as_bytes()
            .try_into()
            .map_err(|_| ValueError::LastFour)?;
        if digits.iter().all(u8::is_ascii_digit) {
            Ok(Self(digits))
        } else {
            Err(ValueError::LastFour)
        }
    }

    /// The four digits.
    pub fn as_str(&self) -> &str {
        core::str::from_utf8(&self.0).expect("checked ASCII digits")
    }
}

impl fmt::Display for LastFour {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

text_value!(LastFour);

/// An ISO 3166-1 alpha-2 country code, as CXF requires: two uppercase
/// letters. The form is checked, not membership of the ISO list.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CountryCode([u8; 2]);

impl CountryCode {
    /// Accept two uppercase ASCII letters.
    pub fn parse(text: &str) -> Result<Self, ValueError> {
        let letters: [u8; 2] = text
            .as_bytes()
            .try_into()
            .map_err(|_| ValueError::CountryCode)?;
        if letters.iter().all(u8::is_ascii_uppercase) {
            Ok(Self(letters))
        } else {
            Err(ValueError::CountryCode)
        }
    }

    /// The two letters.
    pub fn as_str(&self) -> &str {
        core::str::from_utf8(&self.0).expect("checked ASCII letters")
    }
}

impl fmt::Display for CountryCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

text_value!(CountryCode);

/// An ISO 3166-2 subdivision code such as `US-CA`: a country code, a
/// hyphen, and one to three uppercase letters or digits.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SubdivisionCode(String);

impl SubdivisionCode {
    /// Accept the ISO 3166-2 form.
    pub fn parse(text: &str) -> Result<Self, ValueError> {
        let (country, part) = text.split_once('-').ok_or(ValueError::SubdivisionCode)?;
        let part_ok = (1..=3).contains(&part.len())
            && part
                .bytes()
                .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit());
        if CountryCode::parse(country).is_ok() && part_ok {
            Ok(Self(text.to_string()))
        } else {
            Err(ValueError::SubdivisionCode)
        }
    }

    /// The code as written.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The country the subdivision belongs to.
    pub fn country(&self) -> CountryCode {
        CountryCode::parse(&self.0[..2]).expect("checked on parse")
    }
}

impl fmt::Display for SubdivisionCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

text_value!(SubdivisionCode);

/// The ASCII digits at `range` of `text`, as a number.
fn digits(text: &str, range: core::ops::Range<usize>) -> Option<u16> {
    let field = text.get(range)?;
    let valid = field.bytes().all(|c| c.is_ascii_digit());
    valid.then(|| field.parse().ok()).flatten()
}

/// Whether `text` is `len` bytes with hyphens exactly at `hyphens`.
fn shaped(text: &str, len: usize, hyphens: &[usize]) -> bool {
    text.len() == len && hyphens.iter().all(|&at| text.as_bytes()[at] == b'-')
}

/// A calendar date, CXF's `date` field type: RFC 3339 `full-date`.
///
/// Ordered by time, so expiry dates compare as dates.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Date {
    year: u16,
    month: u8,
    day: u8,
}

impl Date {
    /// A date, if the day exists in that month of that year.
    pub fn new(year: u16, month: u8, day: u8) -> Result<Self, ValueError> {
        let valid =
            year <= 9999 && (1..=12).contains(&month) && (1..=days_in(year, month)).contains(&day);
        if valid {
            Ok(Self { year, month, day })
        } else {
            Err(ValueError::Date)
        }
    }

    /// Accept `YYYY-MM-DD`.
    pub fn parse(text: &str) -> Result<Self, ValueError> {
        if !shaped(text, 10, &[4, 7]) {
            return Err(ValueError::Date);
        }
        let field = |range| digits(text, range).ok_or(ValueError::Date);
        let (year, month, day) = (field(0..4)?, field(5..7)?, field(8..10)?);
        Self::new(year, month as u8, day as u8)
    }

    /// The year.
    pub const fn year(&self) -> u16 {
        self.year
    }

    /// The month, 1 to 12.
    pub const fn month(&self) -> u8 {
        self.month
    }

    /// The day of the month.
    pub const fn day(&self) -> u8 {
        self.day
    }
}

fn days_in(year: u16, month: u8) -> u8 {
    match month {
        4 | 6 | 9 | 11 => 30,
        2 if year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400)) => {
            29
        },
        2 => 28,
        _ => 31,
    }
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

text_value!(Date);

/// A year and month, CXF's `year-month` field type: how cards write expiry.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct YearMonth {
    year: u16,
    month: u8,
}

impl YearMonth {
    /// A year-month, if the month is 1 to 12.
    pub fn new(year: u16, month: u8) -> Result<Self, ValueError> {
        if year <= 9999 && (1..=12).contains(&month) {
            Ok(Self { year, month })
        } else {
            Err(ValueError::YearMonth)
        }
    }

    /// Accept `YYYY-MM`.
    pub fn parse(text: &str) -> Result<Self, ValueError> {
        if !shaped(text, 7, &[4]) {
            return Err(ValueError::YearMonth);
        }
        let field = |range| digits(text, range).ok_or(ValueError::YearMonth);
        let (year, month) = (field(0..4)?, field(5..7)?);
        Self::new(year, month as u8)
    }

    /// The year.
    pub const fn year(&self) -> u16 {
        self.year
    }

    /// The month, 1 to 12.
    pub const fn month(&self) -> u8 {
        self.month
    }
}

impl fmt::Display for YearMonth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}-{:02}", self.year, self.month)
    }
}

text_value!(YearMonth);

const SHA256_PREFIX: &str = "SHA256:";

/// An SSH public key's SHA-256 fingerprint in the `SHA256:...` form ssh
/// tools print, the key personae files SSH slots under.
///
/// castellan derives it from the key at import; chatelaine does no hashing.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SshFingerprint(String);

impl SshFingerprint {
    /// Accept `SHA256:` and 43 characters of the standard base64 alphabet,
    /// unpadded: the length of a 32-byte digest.
    pub fn parse(text: &str) -> Result<Self, ValueError> {
        let digest = text
            .strip_prefix(SHA256_PREFIX)
            .ok_or(ValueError::SshFingerprint)?;
        let valid = digest.len() == 43
            && digest
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || c == b'+' || c == b'/');
        if valid {
            Ok(Self(text.to_string()))
        } else {
            Err(ValueError::SshFingerprint)
        }
    }

    /// The fingerprint as written.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SshFingerprint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

text_value!(SshFingerprint);

#[cfg(test)]
#[path = "value_tests.rs"]
mod tests;
