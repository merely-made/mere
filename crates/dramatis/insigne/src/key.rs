// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Public keys, typed by the family that minted them.
//!
//! A 32-byte Ed25519 key and a 32-byte Nostr key are not the same thing, so a
//! key carries its family. Each family keeps its own standard text form: the
//! multicodec families are written as `did:key` (base58btc multibase over a
//! multicodec prefix), and a Reticulum identity, which has no multicodec, is
//! written the way Reticulum's `rnid` exports one, as 128 lowercase hex
//! characters.

use core::fmt;
use core::str::FromStr;

use serde::de::{self, Deserializer, SeqAccess, Visitor};
use serde::{Deserialize, Serialize, Serializer};

use crate::encoding::{base58_decode, base58_encode, hex_decode, hex_encode};

const DID_KEY: &str = "did:key:";
const ED25519_CODEC: [u8; 2] = [0xed, 0x01];
const SECP256K1_CODEC: [u8; 2] = [0xe7, 0x01];
const P256_CODEC: [u8; 2] = [0x80, 0x24];

/// A public key and the family it belongs to.
///
/// insigne's lowest grade, the bare key: continuity, "same me as last time".
/// Plain data, like everything insigne's core holds. Nothing here verifies a
/// signature, so a keeper of keys such as gaz takes on no cryptography.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TypedKey {
    /// Ed25519: what `personae` mints, and what murm, iroh and the mesh sign
    /// with.
    Ed25519([u8; 32]),
    /// A compressed secp256k1 point: atproto signing and rotation keys, and
    /// Nostr keys lifted from x-only form.
    Secp256k1([u8; 33]),
    /// A compressed P-256 point: most atproto signing keys issued today.
    P256([u8; 33]),
    /// A whole Reticulum identity: the X25519 exchange half, then the Ed25519
    /// signing half. Reticulum fingerprints the two together, so they are
    /// never split.
    Reticulum([u8; 64]),
}

/// Which family a [`TypedKey`] belongs to.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum KeyAlgorithm {
    /// Ed25519.
    Ed25519,
    /// secp256k1, compressed.
    Secp256k1,
    /// P-256, compressed.
    P256,
    /// A Reticulum identity, X25519 then Ed25519.
    Reticulum,
}

impl KeyAlgorithm {
    /// insigne's own tag for the binary form. The text form carries the standard.
    const fn tag(self) -> u8 {
        match self {
            Self::Ed25519 => 1,
            Self::Secp256k1 => 2,
            Self::P256 => 3,
            Self::Reticulum => 4,
        }
    }
}

impl TypedKey {
    /// Wrap an Ed25519 public key.
    pub const fn ed25519(bytes: [u8; 32]) -> Self {
        Self::Ed25519(bytes)
    }

    /// Wrap a compressed secp256k1 point.
    pub fn secp256k1(bytes: [u8; 33]) -> Result<Self, KeyParseError> {
        compressed(&bytes)?;
        Ok(Self::Secp256k1(bytes))
    }

    /// Wrap a compressed P-256 point.
    pub fn p256(bytes: [u8; 33]) -> Result<Self, KeyParseError> {
        compressed(&bytes)?;
        Ok(Self::P256(bytes))
    }

    /// Wrap a whole Reticulum public identity, X25519 half first, as
    /// `RNS.Identity.get_public_key()` returns it.
    pub const fn reticulum(bytes: [u8; 64]) -> Self {
        Self::Reticulum(bytes)
    }

    /// Lift a Nostr x-only key (BIP-340) into compressed form.
    ///
    /// Lossless: BIP-340 fixes the point with the even Y, which is the `0x02`
    /// prefix.
    pub fn from_nostr_x_only(x: [u8; 32]) -> Self {
        let mut bytes = [0u8; 33];
        bytes[0] = 0x02;
        bytes[1..].copy_from_slice(&x);
        Self::Secp256k1(bytes)
    }

    /// Which family this key belongs to.
    pub const fn algorithm(&self) -> KeyAlgorithm {
        match self {
            Self::Ed25519(_) => KeyAlgorithm::Ed25519,
            Self::Secp256k1(_) => KeyAlgorithm::Secp256k1,
            Self::P256(_) => KeyAlgorithm::P256,
            Self::Reticulum(_) => KeyAlgorithm::Reticulum,
        }
    }

    /// The raw key bytes, for handing to a verifier that does own crypto.
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            Self::Ed25519(bytes) => bytes,
            Self::Secp256k1(bytes) | Self::P256(bytes) => bytes,
            Self::Reticulum(bytes) => bytes,
        }
    }

    /// The family's standard text form: a `did:key`, or `rnid`'s hex.
    pub fn to_text(&self) -> String {
        match self {
            Self::Ed25519(bytes) => did_key(ED25519_CODEC, bytes),
            Self::Secp256k1(bytes) => did_key(SECP256K1_CODEC, bytes),
            Self::P256(bytes) => did_key(P256_CODEC, bytes),
            Self::Reticulum(bytes) => hex_encode(bytes),
        }
    }

    /// The first eight hex characters of the key bytes, for logs and compact
    /// UI. A prefix is never an identity: compare whole keys.
    pub fn short(&self) -> String {
        hex_encode(&self.as_bytes()[..4])
    }

    /// Read the binary form: one tag byte, then the key bytes.
    fn from_tagged(tagged: &[u8]) -> Result<Self, KeyParseError> {
        let Some((&tag, key)) = tagged.split_first() else {
            return Err(KeyParseError::Length {
                expected: 1,
                found: 0,
            });
        };
        match tag {
            1 => exact(key).map(Self::Ed25519),
            2 => Self::secp256k1(exact(key)?),
            3 => Self::p256(exact(key)?),
            4 => exact(key).map(Self::Reticulum),
            _ => Err(KeyParseError::Codec),
        }
    }

    fn to_tagged(self) -> Vec<u8> {
        let mut tagged = Vec::with_capacity(1 + self.as_bytes().len());
        tagged.push(self.algorithm().tag());
        tagged.extend_from_slice(self.as_bytes());
        tagged
    }
}

fn did_key(codec: [u8; 2], key: &[u8]) -> String {
    let mut payload = Vec::with_capacity(codec.len() + key.len());
    payload.extend_from_slice(&codec);
    payload.extend_from_slice(key);
    format!("{DID_KEY}z{}", base58_encode(&payload))
}

fn from_did_key(multibase: &str) -> Result<TypedKey, KeyParseError> {
    let encoded = multibase
        .strip_prefix('z')
        .ok_or(KeyParseError::Multibase)?;
    let payload = base58_decode(encoded).ok_or(KeyParseError::Base58)?;
    let (codec, key) = payload.split_at_checked(2).ok_or(KeyParseError::Codec)?;
    match codec {
        [0xed, 0x01] => exact(key).map(TypedKey::Ed25519),
        [0xe7, 0x01] => TypedKey::secp256k1(exact(key)?),
        [0x80, 0x24] => TypedKey::p256(exact(key)?),
        _ => Err(KeyParseError::Codec),
    }
}

fn exact<const N: usize>(bytes: &[u8]) -> Result<[u8; N], KeyParseError> {
    bytes.try_into().map_err(|_| KeyParseError::Length {
        expected: N,
        found: bytes.len(),
    })
}

fn compressed(bytes: &[u8; 33]) -> Result<(), KeyParseError> {
    match bytes[0] {
        0x02 | 0x03 => Ok(()),
        _ => Err(KeyParseError::NotCompressed),
    }
}

impl FromStr for TypedKey {
    type Err = KeyParseError;

    /// Read a `did:key`, or a Reticulum identity as `rnid` writes it (either
    /// case is accepted; insigne writes lowercase).
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        if let Some(multibase) = text.strip_prefix(DID_KEY) {
            return from_did_key(multibase);
        }
        if text.len() == 128 {
            return hex_decode::<64>(text)
                .map(Self::Reticulum)
                .ok_or(KeyParseError::Hex);
        }
        Err(KeyParseError::Form)
    }
}

/// The standard text form, so a key round-trips through `to_string` and
/// `parse`.
impl fmt::Display for TypedKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_text())
    }
}

/// Abbreviated, so a log line stays readable.
impl fmt::Debug for TypedKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}({})", self.algorithm(), self.short())
    }
}

/// The text form in human-readable formats, so a stored book stays readable
/// and a key can serve as a JSON string. Binary formats get the tag and the
/// raw bytes.
impl Serialize for TypedKey {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if serializer.is_human_readable() {
            serializer.serialize_str(&self.to_text())
        } else {
            serializer.serialize_bytes(&self.to_tagged())
        }
    }
}

impl<'de> Deserialize<'de> for TypedKey {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        if deserializer.is_human_readable() {
            deserializer.deserialize_str(TextVisitor)
        } else {
            deserializer.deserialize_bytes(TaggedVisitor)
        }
    }
}

struct TextVisitor;

impl Visitor<'_> for TextVisitor {
    type Value = TypedKey;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("a did:key, or a Reticulum identity in hex")
    }

    fn visit_str<E: de::Error>(self, text: &str) -> Result<TypedKey, E> {
        text.parse().map_err(E::custom)
    }
}

struct TaggedVisitor;

impl<'de> Visitor<'de> for TaggedVisitor {
    type Value = TypedKey;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("a key tag followed by the key bytes")
    }

    fn visit_bytes<E: de::Error>(self, tagged: &[u8]) -> Result<TypedKey, E> {
        TypedKey::from_tagged(tagged).map_err(E::custom)
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<TypedKey, A::Error> {
        let mut tagged = Vec::with_capacity(65);
        while let Some(byte) = seq.next_element::<u8>()? {
            tagged.push(byte);
        }
        TypedKey::from_tagged(&tagged).map_err(de::Error::custom)
    }
}

/// Why text or bytes were not a key.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyParseError {
    /// Neither a `did:key` nor a 128-character Reticulum identity.
    Form,
    /// A `did:key` whose multibase is not base58btc (`z`).
    Multibase,
    /// A character outside the base58btc alphabet.
    Base58,
    /// A multicodec prefix, or a binary tag, insigne does not hold.
    Codec,
    /// The right family, the wrong number of bytes.
    Length {
        /// How many bytes the family takes.
        expected: usize,
        /// How many were supplied.
        found: usize,
    },
    /// A secp256k1 or P-256 key that is not a compressed point.
    NotCompressed,
    /// A Reticulum identity with a character outside `[0-9a-fA-F]`.
    Hex,
}

impl fmt::Display for KeyParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Form => f.write_str("a key is a did:key or a 128-character Reticulum identity"),
            Self::Multibase => f.write_str("a did:key is base58btc multibase, starting with z"),
            Self::Base58 => f.write_str("a did:key holds base58btc characters only"),
            Self::Codec => f.write_str("not a key family insigne holds"),
            Self::Length { expected, found } => {
                write!(f, "this key family takes {expected} bytes, found {found}")
            },
            Self::NotCompressed => {
                f.write_str("a secp256k1 or P-256 key must be a compressed point")
            },
            Self::Hex => f.write_str("a Reticulum identity holds hex digits only"),
        }
    }
}

impl core::error::Error for KeyParseError {}

#[cfg(test)]
mod tests {
    use super::*;

    // The did:key spec's own worked example.
    const SPEC_ED25519: &str = "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK";
    const SPEC_ED25519_HEX: &str =
        "2e6fcce36701dc791488e0d0b1745cc1e33a4c1c9fcc41c63bd343dbbe0970e6";
    // bsky.app's live atproto signing key, from its PLC document.
    const BSKY_SECP256K1: &str = "did:key:zQ3shQo6TF2moaqMTrUZEM1jeuYRQXeHEx4evX9751y2qPqRA";
    const BSKY_SECP256K1_HEX: &str =
        "023249d921a1da482dc7117e9451bf2ae48ef641dc87bd9c9ea3648f3e81cce249";
    // A live P-256 signing key from a 2026-08-01 PLC export sample.
    const PLC_P256: &str = "did:key:zDnaeyxJYdUvhr4FR6YwqWXutZp6YzQP4wisXUFaftZpyB4wY";
    const PLC_P256_HEX: &str = "03f217243a9bb95f6c851b406f50c5a167b9fe81de8d470abbee15801120449cb7";
    // prns's public identity for the RNS 1.4.2 fixture, as rnid would export it.
    const RNS_IDENTITY: &str = "0faa684ed28867b97f4a6a2dee5df8ce974e76b7018e3f22a1c4cf2678570f20d04ab232742bb4ab3a1368bd4615e4e6d0224ab71a016baf8520a332c9778737";

    fn from_hex<const N: usize>(text: &str) -> [u8; N] {
        hex_decode(text).unwrap()
    }

    #[test]
    fn live_did_keys_decode_to_the_right_family_and_bytes() {
        let cases = [
            (SPEC_ED25519, KeyAlgorithm::Ed25519, SPEC_ED25519_HEX),
            (BSKY_SECP256K1, KeyAlgorithm::Secp256k1, BSKY_SECP256K1_HEX),
            (PLC_P256, KeyAlgorithm::P256, PLC_P256_HEX),
        ];
        for (text, algorithm, hex) in cases {
            let key: TypedKey = text.parse().unwrap();
            assert_eq!(key.algorithm(), algorithm, "{text}");
            assert_eq!(hex_encode(key.as_bytes()), hex, "{text}");
            assert_eq!(key.to_text(), text, "{text} must re-encode byte for byte");
        }
    }

    #[test]
    fn an_ed25519_key_encodes_to_its_did_key() {
        let key = TypedKey::ed25519(from_hex(SPEC_ED25519_HEX));
        assert_eq!(key.to_string(), SPEC_ED25519);
    }

    #[test]
    fn a_reticulum_identity_reads_and_writes_as_rnid_hex() {
        let key: TypedKey = RNS_IDENTITY.parse().unwrap();
        assert_eq!(key, TypedKey::reticulum(from_hex(RNS_IDENTITY)));
        assert_eq!(key.to_text(), RNS_IDENTITY);

        let upper: TypedKey = RNS_IDENTITY.to_uppercase().parse().unwrap();
        assert_eq!(upper, key, "rnid input is case-insensitive");
        assert_eq!(upper.to_text(), RNS_IDENTITY, "insigne writes lowercase");
    }

    #[test]
    fn a_nostr_key_lifts_to_an_even_y_compressed_point() {
        let x = [7u8; 32];
        let key = TypedKey::from_nostr_x_only(x);
        assert_eq!(key.algorithm(), KeyAlgorithm::Secp256k1);
        assert_eq!(key.as_bytes()[0], 0x02);
        assert_eq!(&key.as_bytes()[1..], &x);
    }

    #[test]
    fn same_bytes_in_different_families_are_different_keys() {
        let ed = TypedKey::ed25519([7u8; 32]);
        let nostr = TypedKey::from_nostr_x_only([7u8; 32]);
        assert_ne!(ed, nostr);
    }

    #[test]
    fn malformed_text_is_refused_by_name() {
        let cases = [
            ("alice", KeyParseError::Form),
            ("did:key:f00", KeyParseError::Multibase),
            ("did:key:z0OIl", KeyParseError::Base58),
            // A valid base58 payload whose prefix is no family insigne holds.
            ("did:key:z2NEpo7TZRRrLZSi2U", KeyParseError::Codec),
        ];
        for (text, error) in cases {
            assert_eq!(text.parse::<TypedKey>(), Err(error), "{text}");
        }
        let short = format!("did:key:z{}", base58_encode(&[0xed, 0x01, 1, 2, 3]));
        assert_eq!(
            short.parse::<TypedKey>(),
            Err(KeyParseError::Length {
                expected: 32,
                found: 3
            })
        );
        assert_eq!("g".repeat(128).parse::<TypedKey>(), Err(KeyParseError::Hex));
    }

    #[test]
    fn an_uncompressed_prefix_is_refused() {
        let mut bytes = from_hex::<33>(BSKY_SECP256K1_HEX);
        bytes[0] = 0x04;
        assert_eq!(
            TypedKey::secp256k1(bytes),
            Err(KeyParseError::NotCompressed)
        );
        let text = did_key(SECP256K1_CODEC, &bytes);
        assert_eq!(text.parse::<TypedKey>(), Err(KeyParseError::NotCompressed));
    }

    #[test]
    fn debug_is_short_and_names_the_family() {
        let key: TypedKey = SPEC_ED25519.parse().unwrap();
        assert_eq!(format!("{key:?}"), "Ed25519(2e6fcce3)");
    }

    #[test]
    fn json_carries_the_text_form() {
        for text in [SPEC_ED25519, BSKY_SECP256K1, PLC_P256, RNS_IDENTITY] {
            let key: TypedKey = text.parse().unwrap();
            let json = serde_json::to_string(&key).unwrap();
            assert_eq!(json, format!("\"{text}\""));
            assert_eq!(serde_json::from_str::<TypedKey>(&json).unwrap(), key);
        }
    }

    #[test]
    fn binary_carries_the_tag_and_raw_bytes() {
        for text in [SPEC_ED25519, BSKY_SECP256K1, PLC_P256, RNS_IDENTITY] {
            let key: TypedKey = text.parse().unwrap();
            let bytes = postcard::to_allocvec(&key).unwrap();
            // A length prefix, the tag, then the key: no text form inside.
            assert_eq!(bytes.len(), 2 + key.as_bytes().len(), "{text}");
            assert_eq!(bytes[1], key.algorithm().tag());
            assert_eq!(postcard::from_bytes::<TypedKey>(&bytes).unwrap(), key);
        }
    }

    #[test]
    fn a_bad_binary_tag_fails_to_deserialize() {
        // postcard bytes: length 3, then tag 9 (no family), then two bytes.
        let error = postcard::from_bytes::<TypedKey>(&[3, 9, 1, 2]).unwrap_err();
        assert!(
            matches!(error, postcard::Error::SerdeDeCustom),
            "must fail on the tag, not on framing: {error:?}"
        );
    }
}
