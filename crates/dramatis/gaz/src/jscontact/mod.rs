// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! JSContact 1.x exchange, with separate peer intake and private restoration.
//!
//! [`JsContactFormat::import`] makes unverified contact claims and preserves the
//! source [`Card`], including fields Gaz cannot represent. It never changes a
//! book. [`JsContactFormat::restore_contact`] is an explicit private-backup path
//! that restores local trust, notes, tier and recency. Do not use it for peers.
//! [`PublicCard`] contains only explicitly selected publishable fields.
//!
//! This is a projection of RFC 9553, not a complete standards validator or a
//! vCard converter. Gaz validates the fields it maps; other fields remain in the
//! source Card. No network, clock, entropy, cryptography, or persona dependency
//! is added. The host supplies its own controlled extension domain.
//!
//! ```
//! use gaz::{Contact, TypedKey};
//! use gaz::jscontact::JsContactFormat;
//! let format = JsContactFormat::new("example.org")?; // Use your own domain.
//! let contact = Contact::new("Alice", TypedKey::ed25519([1; 32]));
//! let backup = format.export_contact(&contact)?; // Includes PRIVATE local state.
//! assert_eq!(format.restore_contact(&backup)?, contact);
//! let peer = format.import(&backup, None)?;
//! assert_eq!(peer.contact.petname, "Alice");
//! # Ok::<(), gaz::jscontact::ExchangeError>(())
//! ```

use core::fmt;

use serde::{Deserialize, Serialize};

use crate::{Anchor, AttestedKey, Contact, EndpointKind, HandleKind, RootKey, TypedKey};

mod card;
mod exchange;
mod strict_json;

pub use card::Card;

/// A refused exchange. No partial contact or implicit book update is returned.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExchangeError {
    /// Invalid JSON, mapped Card field, or conflicting public/private data.
    Invalid(String),
    /// A foreign uid needs a local anchor chosen by the host.
    LocalAnchorRequired,
    /// The host's domain is not a valid, non-IETF extension prefix.
    ExtensionDomain,
}

impl fmt::Display for ExchangeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Invalid(message) => write!(f, "JSContact: {message}"),
            Self::LocalAnchorRequired => {
                f.write_str("JSContact uid requires a caller-supplied local anchor")
            },
            Self::ExtensionDomain => {
                f.write_str("JSContact extensions require a host-controlled domain prefix")
            },
        }
    }
}

impl std::error::Error for ExchangeError {}

pub(super) fn invalid(message: impl Into<String>) -> ExchangeError {
    ExchangeError::Invalid(message.into())
}

/// Public identity claims. Holding these artifacts never means they checked.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IdentityClaim {
    /// Stable anchor, matching the Card uid.
    pub anchor: Anchor,
    /// Root history, oldest first, with retained rotation evidence.
    pub root_line: Vec<RootKey>,
    /// Concurrent keys with retained artifacts and their signed context.
    pub attested: Vec<AttestedKey>,
}

/// A handle explicitly selected for publication, without local trust state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublicHandle {
    /// Handle family.
    pub kind: HandleKind,
    /// Exact advertised value.
    pub value: String,
}

/// An endpoint explicitly selected for publication, without trust or recency.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublicEndpoint {
    /// Protocol family.
    pub kind: EndpointKind,
    /// Exact advertised address.
    pub address: String,
}

/// The selected fields of a persona's public card.
///
/// There is intentionally no conversion from Contact: publication cannot
/// accidentally copy a private petname, note, kin tier, trust or usage history.
/// Hosts construct this from the persona and addresses they intend to publish.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublicCard {
    /// The persona's chosen public display name.
    pub name: String,
    /// The selected public root and device/protocol claims.
    pub identity: IdentityClaim,
    /// Advertised handles, in caller-selected order.
    pub handles: Vec<PublicHandle>,
    /// Advertised endpoints, in caller-selected order.
    pub endpoints: Vec<PublicEndpoint>,
}

/// Peer intake together with its original exchange object.
#[derive(Clone, Debug, PartialEq)]
pub struct ImportedCard {
    /// Unverified claims with default kith tier and no private note or history.
    pub contact: Contact,
    /// Preserve this when re-exchanging unmapped fields or original map ids.
    pub source: Card,
    /// Recognized keys not assigned a root/device role by a Gaz identity claim.
    /// These remain unbound; import does not invent rotation or attestations.
    pub unbound_keys: Vec<TypedKey>,
}

/// A JSContact exchange vocabulary under a domain controlled by the host.
///
/// Both Gaz peers configure the same domain to interpret Gaz-specific claims.
/// Other domains' extensions are retained as opaque data. Prefix configuration
/// neither authenticates a card nor authorizes its sender.
#[derive(Clone, Debug)]
pub struct JsContactFormat {
    domain: String,
}

impl JsContactFormat {
    /// Use the host's ASCII domain (IDNs use their ASCII/Punycode spelling).
    /// Reserved IETF domains and invalid labels are refused; ownership is the
    /// host's responsibility, not something Gaz infers from a DNS lookup.
    pub fn new(domain: &str) -> Result<Self, ExchangeError> {
        let domain = domain.to_ascii_lowercase();
        if domain == "ietf.org"
            || domain.ends_with(".ietf.org")
            || !domain.contains('.')
            || !domain.split('.').all(|label| {
                let bytes = label.as_bytes();
                !bytes.is_empty()
                    && bytes[0].is_ascii_alphanumeric()
                    && bytes[bytes.len() - 1].is_ascii_alphanumeric()
                    && bytes
                        .iter()
                        .all(|b| b.is_ascii_alphanumeric() || *b == b'-')
            })
        {
            return Err(ExchangeError::ExtensionDomain);
        }
        Ok(Self { domain })
    }

    pub(super) fn property(&self, name: &str) -> String {
        format!("{}:{name}", self.domain)
    }
}

#[cfg(test)]
mod tests;
