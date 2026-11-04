// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! **Titulus** — the inscription that says what a thing is.
//!
//! A titulus was the placard naming what hung beneath it, and the label on a
//! scroll or amphora identifying its contents. This crate is that label for a
//! projected resource: a small, portable, semantic description an endpoint
//! hands a host, plus the bounded contract for what may be done about it.
//!
//! Deliberately not a widget tree. A [`PortableCardV1`] carries a title,
//! labeled values, badges, and content addresses — enough for a host to render
//! something honest about a resource it does not own, and not enough to make
//! the host a rendering engine for somebody else's application truth.
//!
//! [`ActionFormV1`] is the other half: an endpoint-authored input contract
//! bounded to exact string choices, so a host can compose an action payload
//! without interpreting application meaning.
//!
//! Extracted from the projection protocol (now [`chirograph`]) 2026-08-14: the
//! card vocabulary is neutral, and the identity port wanted it without the
//! wire. See the mere design docs for the keeper founding that surfaced it.
//!
//! [`chirograph`]: https://crates.io/crates/chirograph

#![warn(missing_docs)]

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::str::FromStr;

use base64::Engine as _;
use serde::{Deserialize, Serialize};
use sha2::Digest as _;

/// A content address for a separately transferred resource.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ContentHash(pub [u8; 32]);

impl ContentHash {
    /// The address of these bytes (BLAKE3).
    pub fn of(bytes: &[u8]) -> Self {
        Self(*blake3::hash(bytes).as_bytes())
    }

    /// Parse the lowercase hexadecimal digest used by iroh's BLAKE3 address.
    pub fn from_hex(value: &str) -> Result<Self, ContentReferenceError> {
        parse_lower_hex_32(value)
            .map(Self)
            .ok_or(ContentReferenceError::InvalidBlake3)
    }

    /// The raw BLAKE3 digest bytes.
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl fmt::Display for ContentHash {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.0 {
            write!(formatter, "{byte:02x}")?;
        }
        Ok(())
    }
}

/// RFC 6920 Named Information URI carrying one full SHA-256 digest.
///
/// The canonical form is `ni:///sha-256;<base64url>` with no authority,
/// query, or padding. Restricting emission to that mandatory algorithm and
/// spelling gives Graphshell and Knot one interoperable portable identifier.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Sha256NamedInformation([u8; 32]);

impl Sha256NamedInformation {
    /// Canonical prefix for the supported RFC 6920 form.
    pub const PREFIX: &'static str = "ni:///sha-256;";

    /// Name these bytes portably.
    pub fn of(bytes: &[u8]) -> Self {
        Self(sha2::Sha256::digest(bytes).into())
    }

    /// Construct from the lowercase hexadecimal SHA-256 representation used
    /// by existing browser-file metadata.
    pub fn from_hex(value: &str) -> Result<Self, ContentReferenceError> {
        parse_lower_hex_32(value)
            .map(Self)
            .ok_or(ContentReferenceError::InvalidSha256Hex)
    }

    /// The named digest bytes.
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Verify bytes against the portable identity.
    pub fn verify(&self, bytes: &[u8]) -> Result<(), ContentReferenceError> {
        (Self::of(bytes) == *self)
            .then_some(())
            .ok_or(ContentReferenceError::Sha256Mismatch)
    }
}

impl fmt::Display for Sha256NamedInformation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(Self::PREFIX)?;
        formatter.write_str(&base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(self.0))
    }
}

impl FromStr for Sha256NamedInformation {
    type Err = ContentReferenceError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let encoded = value
            .strip_prefix(Self::PREFIX)
            .ok_or(ContentReferenceError::InvalidNamedInformation)?;
        if encoded.len() != 43 || encoded.contains('=') {
            return Err(ContentReferenceError::InvalidNamedInformation);
        }
        let decoded = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(encoded)
            .map_err(|_| ContentReferenceError::InvalidNamedInformation)?;
        let digest: [u8; 32] = decoded
            .try_into()
            .map_err(|_| ContentReferenceError::InvalidNamedInformation)?;
        Ok(Self(digest))
    }
}

impl Serialize for Sha256NamedInformation {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for Sha256NamedInformation {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map_err(serde::de::Error::custom)
    }
}

/// One portable byte identity beside the BLAKE3 address used for transfer.
///
/// Media type, source URI, and artifact role deliberately live outside this
/// type: they describe provenance and use, while these three fields identify
/// and bound the bytes themselves.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PortableContentRefV1 {
    /// RFC 6920 portable identity.
    pub portable_id: Sha256NamedInformation,
    /// BLAKE3 address used by the local and iroh content stores.
    #[serde(with = "blake3_transport_serde")]
    pub transport: ContentHash,
    /// Exact byte length, checked before either digest is trusted.
    pub byte_size: u64,
}

impl PortableContentRefV1 {
    /// Name bytes in both portable and transport domains.
    pub fn of(bytes: &[u8]) -> Self {
        Self {
            portable_id: Sha256NamedInformation::of(bytes),
            transport: ContentHash::of(bytes),
            byte_size: u64::try_from(bytes.len()).expect("slice length fits u64"),
        }
    }

    /// Verify length, iroh/BLAKE3 address, then portable SHA-256 identity.
    pub fn verify_bytes(&self, bytes: &[u8]) -> Result<(), ContentReferenceError> {
        if u64::try_from(bytes.len()).ok() != Some(self.byte_size) {
            return Err(ContentReferenceError::ByteSizeMismatch);
        }
        if ContentHash::of(bytes) != self.transport {
            return Err(ContentReferenceError::Blake3Mismatch);
        }
        self.portable_id.verify(bytes)
    }
}

/// A malformed or conflicting portable content reference.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContentReferenceError {
    /// The URI was not the canonical mandatory SHA-256 NI form.
    InvalidNamedInformation,
    /// An existing browser SHA-256 digest was not lowercase 32-byte hex.
    InvalidSha256Hex,
    /// A BLAKE3 transport address was not lowercase 32-byte hex.
    InvalidBlake3,
    /// The declared byte length differed.
    ByteSizeMismatch,
    /// The BLAKE3 transport address differed.
    Blake3Mismatch,
    /// The RFC 6920 SHA-256 identity differed.
    Sha256Mismatch,
}

impl fmt::Display for ContentReferenceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidNamedInformation => {
                "content name is not canonical ni:///sha-256;<base64url>"
            },
            Self::InvalidSha256Hex => "SHA-256 digest is not 64 lowercase hexadecimal characters",
            Self::InvalidBlake3 => {
                "BLAKE3 transport address is not blake3:<64 lowercase hexadecimal characters>"
            },
            Self::ByteSizeMismatch => "content byte length does not match its reference",
            Self::Blake3Mismatch => "content bytes do not match their BLAKE3 transport address",
            Self::Sha256Mismatch => "content bytes do not match their RFC 6920 SHA-256 identity",
        })
    }
}

impl std::error::Error for ContentReferenceError {}

fn parse_lower_hex_32(value: &str) -> Option<[u8; 32]> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return None;
    }
    let mut digest = [0u8; 32];
    for (index, pair) in value.as_bytes().chunks_exact(2).enumerate() {
        let hex = std::str::from_utf8(pair).ok()?;
        digest[index] = u8::from_str_radix(hex, 16).ok()?;
    }
    Some(digest)
}

mod blake3_transport_serde {
    use super::{ContentHash, ContentReferenceError};
    use serde::{Deserialize, Serializer};

    pub fn serialize<S>(hash: &ContentHash, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("blake3:{hash}"))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<ContentHash, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        let hex = value
            .strip_prefix("blake3:")
            .ok_or_else(|| serde::de::Error::custom(ContentReferenceError::InvalidBlake3))?;
        ContentHash::from_hex(hex).map_err(serde::de::Error::custom)
    }
}

/// A bounded, endpoint-authored input form for one advertised action.
///
/// Version one deliberately composes only a JSON object with a mandatory
/// `schema` string and named endpoint-supplied choices. It is enough for exact
/// selections such as saved-record IDs or digests, without turning Graphshell
/// into a generic JSON editor or asking a host to interpret application truth.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActionFormV1 {
    /// Must exactly equal the advertised action's `payload_schema`.
    pub schema: String,
    /// The named inputs, in the order a host should present them.
    pub fields: Vec<ActionFormFieldV1>,
}

/// One named input in an [`ActionFormV1`].
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActionFormFieldV1 {
    /// JSON property name. `schema` is reserved for the form schema marker.
    pub name: String,
    /// What a host shows beside the input.
    pub label: String,
    /// Longer guidance, or empty.
    pub description: String,
    /// Whether composing without this field is refused. Defaults to true, so
    /// an older endpoint's form does not silently become optional.
    #[serde(default = "required_action_form_field")]
    pub required: bool,
    /// The exact values the endpoint will accept. A host offers these and
    /// nothing else.
    pub choices: Vec<ActionFormChoiceV1>,
}

const fn required_action_form_field() -> bool {
    true
}

/// One exact selectable value supplied by the endpoint.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActionFormChoiceV1 {
    /// Opaque payload value. Hosts display the label and return this exact value.
    pub value: String,
    /// What the host shows in place of the value.
    pub label: String,
    /// Longer guidance for this one choice.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Reasons a host must not compose an action payload.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ActionFormError {
    /// Values were supplied for an action that advertised no form. Refused
    /// rather than guessing an encoding the endpoint never described.
    NoInputForm,
    /// The form's own schema string is blank.
    EmptyFormSchema,
    /// The form belongs to a different action than the one being composed.
    SchemaMismatch {
        /// The schema the action advertised.
        expected: String,
        /// The schema the form carries.
        form: String,
    },
    /// A field has no JSON property name.
    EmptyFieldName,
    /// A field claimed the name `schema`, which the form writes itself.
    ReservedFieldName,
    /// Two fields share one name.
    DuplicateField(String),
    /// The named field has nothing to show beside its input.
    EmptyFieldLabel(String),
    /// The named field offers nothing to choose, so it can never be satisfied.
    EmptyChoices(String),
    /// A choice in the named field carries no value.
    EmptyChoiceValue(String),
    /// A choice in the named field has nothing to show.
    EmptyChoiceLabel(String),
    /// Two choices in one field carry the same value.
    DuplicateChoiceValue {
        /// The field holding the repeat.
        field: String,
        /// The value offered twice.
        value: String,
    },
    /// A required field had no value.
    MissingField(String),
    /// A value was supplied under a name the form never advertised.
    UnknownField(String),
    /// A value was supplied that the endpoint did not offer. This is the
    /// bound that keeps a host from inventing payloads.
    InvalidChoice {
        /// The field the value was offered for.
        field: String,
        /// The value that was not on offer.
        value: String,
    },
    /// The composed object would not serialize.
    Encode(String),
}

impl fmt::Display for ActionFormError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoInputForm => write!(formatter, "action has no advertised input form"),
            Self::EmptyFormSchema => write!(formatter, "action form schema is empty"),
            Self::SchemaMismatch { expected, form } => {
                write!(
                    formatter,
                    "action schema {expected} does not match form schema {form}"
                )
            },
            Self::EmptyFieldName => write!(formatter, "action form field name is empty"),
            Self::ReservedFieldName => {
                write!(formatter, "action form field name schema is reserved")
            },
            Self::DuplicateField(field) => write!(formatter, "action form repeats field {field}"),
            Self::EmptyFieldLabel(field) => {
                write!(formatter, "action form field {field} has no label")
            },
            Self::EmptyChoices(field) => {
                write!(formatter, "action form field {field} has no choices")
            },
            Self::EmptyChoiceValue(field) => {
                write!(
                    formatter,
                    "action form field {field} has an empty choice value"
                )
            },
            Self::EmptyChoiceLabel(field) => {
                write!(
                    formatter,
                    "action form field {field} has an empty choice label"
                )
            },
            Self::DuplicateChoiceValue { field, value } => {
                write!(
                    formatter,
                    "action form field {field} repeats choice value {value}"
                )
            },
            Self::MissingField(field) => write!(formatter, "action form requires field {field}"),
            Self::UnknownField(field) => {
                write!(formatter, "action form does not define field {field}")
            },
            Self::InvalidChoice { field, value } => {
                write!(
                    formatter,
                    "action form field {field} does not offer choice {value}"
                )
            },
            Self::Encode(error) => {
                write!(formatter, "could not encode action form payload: {error}")
            },
        }
    }
}

impl std::error::Error for ActionFormError {}

/// One labeled value in a portable card.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CardValueV1 {
    /// What the value is called.
    pub label: String,
    /// The value itself, already rendered to text by the endpoint.
    pub value: String,
}

/// A deliberately small semantic card, not a serialized widget tree.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PortableCardV1 {
    /// What the card is about.
    pub title: String,
    /// Labeled facts, in the order the endpoint wants them read.
    pub values: Vec<CardValueV1>,
    /// Short status words a host may render as chips.
    pub badges: Vec<String>,
    /// Addresses of separately transferred resources. The bytes are fetched
    /// through the carrier; the card carries only where to ask.
    pub media: Vec<ContentHash>,
}

#[cfg(test)]
#[path = "titulus/tests.rs"]
mod tests;

#[path = "titulus/forms.rs"]
mod forms;
