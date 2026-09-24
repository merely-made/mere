// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! What a record is filed under.
//!
//! An anchor never moves. It is whatever the peer's own system says stays
//! fixed: a root key, an atproto `did:plc`, or, for someone who has shown you
//! no key yet, an id gaz keeps for them. Every anchor has a text form that is
//! a standard identifier (`did:key`, `did:plc`, `urn:uuid`) or, for a
//! Reticulum identity, `rnid`'s hex.

use core::fmt;
use core::str::FromStr;

use insigne::{KeyParseError, TypedKey};
use serde::de::{self, Deserializer, Visitor};
use serde::{Deserialize, Serialize, Serializer};
use uuid::Uuid;

const PLC: &str = "did:plc:";
const URN_UUID: &str = "urn:uuid:";

/// An atproto account's `did:plc`.
///
/// Self-certifying: the identifier is a hash of the account's signed genesis
/// operation. What it binds is that operation's rotation keys, usually the
/// hosting server's, so a `did:plc` anchor trusts whoever holds them.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PlcDid(String);

impl PlcDid {
    /// Accept `did:plc:` followed by 24 characters of lowercase base32.
    pub fn parse(text: &str) -> Result<Self, AnchorParseError> {
        let id = text.strip_prefix(PLC).ok_or(AnchorParseError::Plc)?;
        let valid = id.len() == 24
            && id
                .bytes()
                .all(|c| c.is_ascii_lowercase() || (b'2'..=b'7').contains(&c));
        if valid {
            Ok(Self(text.to_string()))
        } else {
            Err(AnchorParseError::Plc)
        }
    }

    /// The DID as written.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for PlcDid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PlcDid({})", self.0)
    }
}

/// An id gaz keeps for someone who has shown you no key yet: a UUID, written
/// as a `urn:uuid`, which is also JSContact's recommended `uid` form.
///
/// gaz reads no random source, as it reads no clock: the caller supplies the
/// bytes. Any UUID version reads back, since an imported card may carry one.
/// Built on the `uuid` crate, the stack's UUID type (personae's `PersonaId`
/// wraps the same).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LocalId(Uuid);

impl LocalId {
    /// Mint a version-4 id from 16 random bytes, setting the version and
    /// variant bits RFC 9562 requires.
    pub fn from_random(bytes: [u8; 16]) -> Self {
        Self(uuid::Builder::from_random_bytes(bytes).into_uuid())
    }

    /// Accept `urn:uuid:` and a hyphenated UUID, either case.
    pub fn parse(text: &str) -> Result<Self, AnchorParseError> {
        let (prefix, hyphenated) = text
            .split_at_checked(URN_UUID.len())
            .ok_or(AnchorParseError::Uuid)?;
        // Thirty-six characters is the hyphenated form and no other.
        if !prefix.eq_ignore_ascii_case(URN_UUID) || hyphenated.len() != 36 {
            return Err(AnchorParseError::Uuid);
        }
        Uuid::try_parse(hyphenated)
            .map(Self)
            .map_err(|_| AnchorParseError::Uuid)
    }

    /// The raw UUID bytes.
    pub const fn as_bytes(&self) -> &[u8; 16] {
        self.0.as_bytes()
    }

    /// The `urn:uuid:` form, lowercase.
    pub fn to_urn(&self) -> String {
        self.0.urn().to_string()
    }
}

impl fmt::Debug for LocalId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "LocalId({})", self.to_urn())
    }
}

/// What a contact is filed under, fixed for the life of the record.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum Anchor {
    /// A root key: the first key of the contact's root line. For a personae
    /// peer this is the master key its attestations name, not a derived key.
    Key(TypedKey),
    /// An atproto account. Its signing keys rotate beneath it.
    Plc(PlcDid),
    /// Someone who has shown you no key yet.
    Local(LocalId),
}

impl Anchor {
    /// The anchor key, if this anchor is one.
    pub fn as_key(&self) -> Option<&TypedKey> {
        match self {
            Self::Key(key) => Some(key),
            Self::Plc(_) | Self::Local(_) => None,
        }
    }
}

impl From<TypedKey> for Anchor {
    fn from(key: TypedKey) -> Self {
        Self::Key(key)
    }
}

impl fmt::Display for Anchor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Key(key) => fmt::Display::fmt(key, f),
            Self::Plc(did) => f.write_str(did.as_str()),
            Self::Local(id) => f.write_str(&id.to_urn()),
        }
    }
}

impl FromStr for Anchor {
    type Err = AnchorParseError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        if text.starts_with(PLC) {
            return PlcDid::parse(text).map(Self::Plc);
        }
        let urn = text
            .get(..URN_UUID.len())
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case(URN_UUID));
        if urn {
            return LocalId::parse(text).map(Self::Local);
        }
        text.parse().map(Self::Key).map_err(AnchorParseError::Key)
    }
}

/// The binary form: a plain tagged enum. Variant order is the wire format.
#[derive(Serialize)]
enum WireRef<'a> {
    Key(&'a TypedKey),
    Plc(&'a str),
    Local(&'a [u8; 16]),
}

#[derive(Deserialize)]
enum Wire {
    Key(TypedKey),
    Plc(String),
    Local([u8; 16]),
}

/// The text form in human-readable formats, so an anchor reads as the
/// identifier it is. Binary formats get a tagged enum.
impl Serialize for Anchor {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if serializer.is_human_readable() {
            return serializer.serialize_str(&self.to_string());
        }
        match self {
            Self::Key(key) => WireRef::Key(key),
            Self::Plc(did) => WireRef::Plc(did.as_str()),
            Self::Local(id) => WireRef::Local(id.as_bytes()),
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Anchor {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        if deserializer.is_human_readable() {
            return deserializer.deserialize_str(TextVisitor);
        }
        match Wire::deserialize(deserializer)? {
            Wire::Key(key) => Ok(Self::Key(key)),
            Wire::Plc(text) => PlcDid::parse(&text)
                .map(Self::Plc)
                .map_err(de::Error::custom),
            Wire::Local(bytes) => Ok(Self::Local(LocalId(Uuid::from_bytes(bytes)))),
        }
    }
}

struct TextVisitor;

impl Visitor<'_> for TextVisitor {
    type Value = Anchor;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("a did:key, a did:plc, a urn:uuid, or a Reticulum identity")
    }

    fn visit_str<E: de::Error>(self, text: &str) -> Result<Anchor, E> {
        text.parse().map_err(E::custom)
    }
}

/// Why text was not an anchor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnchorParseError {
    /// Not a key gaz can read.
    Key(KeyParseError),
    /// Not `did:plc:` followed by 24 characters of lowercase base32.
    Plc,
    /// Not `urn:uuid:` followed by a hyphenated UUID.
    Uuid,
}

impl fmt::Display for AnchorParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Key(error) => write!(f, "not an anchor key: {error}"),
            Self::Plc => f.write_str("a did:plc is did:plc: and 24 lowercase base32 characters"),
            Self::Uuid => f.write_str("a local anchor is urn:uuid: and a hyphenated UUID"),
        }
    }
}

impl core::error::Error for AnchorParseError {}

#[cfg(test)]
mod tests {
    use super::*;

    const BSKY: &str = "did:plc:z72i7hdynmk6r22z27h6tvur";
    const ATPROTO: &str = "did:plc:ewvi7nxzyoun6zhxrhs64oiz";
    // RFC 9553's own uid example: a version-1 UUID.
    const RFC_UID: &str = "urn:uuid:f81d4fae-7dec-11d0-a765-00a0c91e6bf6";
    const SPEC_ED25519: &str = "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK";
    const RNS_IDENTITY: &str = "0faa684ed28867b97f4a6a2dee5df8ce974e76b7018e3f22a1c4cf2678570f20d04ab232742bb4ab3a1368bd4615e4e6d0224ab71a016baf8520a332c9778737";

    #[test]
    fn live_plc_dids_parse() {
        for did in [BSKY, ATPROTO] {
            assert_eq!(PlcDid::parse(did).unwrap().as_str(), did);
        }
    }

    #[test]
    fn malformed_plc_dids_are_refused() {
        for bad in [
            "did:plc:z72i7hdynmk6r22z27h6tvu",   // 23 characters
            "did:plc:z72i7hdynmk6r22z27h6tvurr", // 25 characters
            "did:plc:Z72i7hdynmk6r22z27h6tvur",  // uppercase
            "did:plc:072i7hdynmk6r22z27h6tvur",  // 0 is not base32
            "did:plc:z82i7hdynmk6r22z27h6tvur",  // nor is 8
            "did:web:example.org",
        ] {
            assert_eq!(PlcDid::parse(bad), Err(AnchorParseError::Plc), "{bad}");
        }
    }

    #[test]
    fn a_minted_local_id_is_version_4() {
        let id = LocalId::from_random([0xff; 16]);
        assert_eq!(id.as_bytes()[6] >> 4, 4, "version nibble");
        assert_eq!(id.as_bytes()[8] >> 6, 0b10, "RFC 9562 variant");
        assert_eq!(id.to_urn(), "urn:uuid:ffffffff-ffff-4fff-bfff-ffffffffffff");
    }

    #[test]
    fn a_foreign_uuid_reads_back_unchanged() {
        let id = LocalId::parse(RFC_UID).unwrap();
        assert_eq!(id.to_urn(), RFC_UID);
        let shouted = LocalId::parse(&RFC_UID.to_uppercase()).unwrap();
        assert_eq!(shouted, id, "UUID input is case-insensitive");
    }

    #[test]
    fn malformed_uuids_are_refused() {
        for bad in [
            "urn:uuid:f81d4fae7dec11d0a76500a0c91e6bf6",
            "urn:uuid:f81d4fae-7dec-11d0-a765-00a0c91e6bf",
            "urn:uuid:g81d4fae-7dec-11d0-a765-00a0c91e6bf6",
            "urn:uuid:",
            "urn:uu",
        ] {
            assert_eq!(LocalId::parse(bad), Err(AnchorParseError::Uuid), "{bad}");
        }
    }

    #[test]
    fn every_anchor_round_trips_through_its_text_form() {
        for text in [SPEC_ED25519, RNS_IDENTITY, BSKY, RFC_UID] {
            let anchor: Anchor = text.parse().unwrap();
            assert_eq!(anchor.to_string(), text);
        }
        assert!(matches!(BSKY.parse::<Anchor>(), Ok(Anchor::Plc(_))));
        assert!(matches!(RFC_UID.parse::<Anchor>(), Ok(Anchor::Local(_))));
        assert!(matches!(SPEC_ED25519.parse::<Anchor>(), Ok(Anchor::Key(_))));
    }

    #[test]
    fn did_web_is_not_an_anchor() {
        assert!(matches!(
            "did:web:example.org".parse::<Anchor>(),
            Err(AnchorParseError::Key(KeyParseError::Form))
        ));
    }

    #[test]
    fn anchors_serialize_as_text_in_json_and_tagged_in_binary() {
        let anchors: Vec<Anchor> = [SPEC_ED25519, RNS_IDENTITY, BSKY, RFC_UID]
            .iter()
            .map(|text| text.parse().unwrap())
            .collect();
        for anchor in &anchors {
            let json = serde_json::to_string(anchor).unwrap();
            assert_eq!(json, format!("\"{anchor}\""));
            assert_eq!(&serde_json::from_str::<Anchor>(&json).unwrap(), anchor);

            let bytes = postcard::to_allocvec(anchor).unwrap();
            assert_eq!(&postcard::from_bytes::<Anchor>(&bytes).unwrap(), anchor);
        }
    }

    #[test]
    fn a_binary_plc_did_is_validated_on_load() {
        let bad = postcard::to_allocvec(&WireRef::Plc("did:plc:NOT-VALID")).unwrap();
        assert!(postcard::from_bytes::<Anchor>(&bad).is_err());
    }
}
