// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Ids for items, credentials and collections.
//!
//! Each is a UUID built from bytes the caller supplies: chatelaine reads no
//! random source, as it reads no clock. Any UUID version reads back.

use core::fmt;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

macro_rules! uuid_id {
    ($(#[$doc:meta])* $name:ident) => {
        $(#[$doc])*
        ///
        /// Hyphenated text in human-readable formats, sixteen bytes in binary
        /// ones.
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(Uuid);

        impl $name {
            /// Mint a version-4 id from 16 random bytes, setting the version
            /// and variant bits RFC 9562 requires.
            pub fn from_random(bytes: [u8; 16]) -> Self {
                Self(uuid::Builder::from_random_bytes(bytes).into_uuid())
            }

            /// Read back an id saved as raw bytes, whatever its version.
            pub const fn from_bytes(bytes: [u8; 16]) -> Self {
                Self(Uuid::from_bytes(bytes))
            }

            /// Accept a hyphenated UUID, either case.
            pub fn parse(text: &str) -> Result<Self, IdParseError> {
                // Thirty-six characters is the hyphenated form and no other.
                if text.len() != 36 {
                    return Err(IdParseError);
                }
                Uuid::try_parse(text).map(Self).map_err(|_| IdParseError)
            }

            /// The raw UUID bytes.
            pub const fn as_bytes(&self) -> &[u8; 16] {
                self.0.as_bytes()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                fmt::Display::fmt(&self.0.hyphenated(), f)
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, concat!(stringify!($name), "({})"), self.0.hyphenated())
            }
        }
    };
}

uuid_id!(
    /// An item: a titled container of credentials.
    ItemId
);
uuid_id!(
    /// One credential within an item.
    CredentialId
);
uuid_id!(
    /// A collection of items.
    CollectionId
);

/// Text that was not a hyphenated UUID.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IdParseError;

impl fmt::Display for IdParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("an id is a hyphenated UUID")
    }
}

impl core::error::Error for IdParseError {}

#[cfg(test)]
mod tests {
    use super::*;

    // RFC 9562's own example: a version-1 UUID.
    const FOREIGN: &str = "f81d4fae-7dec-11d0-a765-00a0c91e6bf6";

    #[test]
    fn a_minted_id_is_version_4() {
        let id = ItemId::from_random([0xff; 16]);
        assert_eq!(id.as_bytes()[6] >> 4, 4, "version nibble");
        assert_eq!(id.as_bytes()[8] >> 6, 0b10, "RFC 9562 variant");
        assert_eq!(id.to_string(), "ffffffff-ffff-4fff-bfff-ffffffffffff");
    }

    #[test]
    fn a_foreign_uuid_reads_back_unchanged() {
        let id = CredentialId::parse(FOREIGN).unwrap();
        assert_eq!(id.to_string(), FOREIGN);
        assert_eq!(CredentialId::parse(&FOREIGN.to_uppercase()), Ok(id));
        assert_eq!(CredentialId::from_bytes(*id.as_bytes()), id);
    }

    #[test]
    fn only_the_hyphenated_form_parses() {
        for bad in [
            "f81d4fae7dec11d0a76500a0c91e6bf6",
            "urn:uuid:f81d4fae-7dec-11d0-a765-00a0c91e6bf6",
            "{f81d4fae-7dec-11d0-a765-00a0c91e6bf6}",
            "f81d4fae-7dec-11d0-a765-00a0c91e6bf",
            "g81d4fae-7dec-11d0-a765-00a0c91e6bf6",
            "",
        ] {
            assert_eq!(CollectionId::parse(bad), Err(IdParseError), "{bad}");
        }
    }

    #[test]
    fn ids_are_text_in_json_and_bytes_in_postcard() {
        let id = ItemId::parse(FOREIGN).unwrap();
        let json = serde_json::to_string(&id).unwrap();
        assert_eq!(json, format!("\"{FOREIGN}\""));
        assert_eq!(serde_json::from_str::<ItemId>(&json).unwrap(), id);

        let bytes = postcard::to_allocvec(&id).unwrap();
        assert_eq!(bytes.len(), 17, "a length byte and sixteen id bytes");
        assert_eq!(postcard::from_bytes::<ItemId>(&bytes).unwrap(), id);
    }
}
