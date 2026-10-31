// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Base64 byte fields: [`Bytes`] for public bytes, [`Secret`] for a
//! passphrase or released seed (§5.5). A secret decodes straight from the
//! frame's borrowed string into a clearing buffer of exact size and encodes
//! through a clearing string; its `Debug` is redacted.

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use serde::de::{Error as _, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use zeroize::Zeroizing;

/// Public bytes, base64 on the wire.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Bytes(pub Vec<u8>);

impl Serialize for Bytes {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&STANDARD.encode(&self.0))
    }
}

impl<'de> Deserialize<'de> for Bytes {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let decoded = deserializer.deserialize_str(Base64Visitor)?;
        Ok(Self(decoded.to_vec()))
    }
}

impl From<&[u8]> for Bytes {
    fn from(bytes: &[u8]) -> Self {
        Self(bytes.to_vec())
    }
}

/// Secret bytes that clear on drop, with no stray copy made on decode.
#[derive(Clone, PartialEq, Eq)]
pub struct Secret(Zeroizing<Vec<u8>>);

impl Secret {
    /// Take ownership of already-clearing bytes.
    pub fn new(bytes: Zeroizing<Vec<u8>>) -> Self {
        Self(bytes)
    }

    /// Copy from a slice into a clearing buffer of exact size.
    pub fn copy_from(bytes: &[u8]) -> Self {
        let mut owned = Zeroizing::new(Vec::with_capacity(bytes.len()));
        owned.extend_from_slice(bytes);
        Self(owned)
    }

    /// The bytes.
    pub fn expose(&self) -> &[u8] {
        &self.0
    }

    /// The clearing buffer itself.
    pub fn into_inner(self) -> Zeroizing<Vec<u8>> {
        self.0
    }
}

impl std::fmt::Debug for Secret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Secret([redacted; {}])", self.0.len())
    }
}

impl Serialize for Secret {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut encoded = Zeroizing::new(String::with_capacity(
            base64::encoded_len(self.0.len(), true).unwrap_or(0),
        ));
        STANDARD.encode_string(&*self.0, &mut encoded);
        serializer.serialize_str(&encoded)
    }
}

impl<'de> Deserialize<'de> for Secret {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_str(Base64Visitor).map(Self)
    }
}

struct Base64Visitor;

impl Visitor<'_> for Base64Visitor {
    type Value = Zeroizing<Vec<u8>>;

    fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("base64 bytes")
    }

    fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<Self::Value, E> {
        let mut out = Zeroizing::new(Vec::with_capacity(value.len() / 4 * 3 + 3));
        STANDARD
            .decode_vec(value, &mut out)
            .map_err(|_| E::custom("invalid base64"))?;
        Ok(out)
    }
}

/// `[u8; N]` as base64, for public keys and signatures.
pub mod fixed {
    use super::*;

    pub fn serialize<S: Serializer, const N: usize>(
        bytes: &[u8; N],
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&STANDARD.encode(bytes))
    }

    pub fn deserialize<'de, D: Deserializer<'de>, const N: usize>(
        deserializer: D,
    ) -> Result<[u8; N], D::Error> {
        let decoded = deserializer.deserialize_str(Base64Visitor)?;
        <[u8; N]>::try_from(decoded.as_slice()).map_err(|_| D::Error::custom("wrong length"))
    }
}

/// `Option<[u8; N]>` as optional base64.
pub mod fixed_opt {
    use super::*;

    pub fn serialize<S: Serializer, const N: usize>(
        bytes: &Option<[u8; N]>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        match bytes {
            Some(bytes) => serializer.serialize_some(&STANDARD.encode(bytes)),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D: Deserializer<'de>, const N: usize>(
        deserializer: D,
    ) -> Result<Option<[u8; N]>, D::Error> {
        let decoded: Option<Bytes> = Option::deserialize(deserializer)?;
        decoded
            .map(|bytes| <[u8; N]>::try_from(bytes.0.as_slice()))
            .transpose()
            .map_err(|_| D::Error::custom("wrong length"))
    }
}
