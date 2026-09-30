// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Unverified address claims, added without replacing local contact state.
//!
//! A resolver supplies claims; the host chooses the book and, if needed, a
//! fresh local id and petname. Only the primary handle selects a contact.
//! Aliases cannot merge people, select another record, or promote keys. This
//! first intake slice does not rotate keys, check proofs or raise trust.
//!
//! The human lookup helpers in `Handle` are intentionally separate from these
//! identity comparisons. Acct intake preserves account-name case; opaque
//! families compare exact typed values. The caller owns fetching and consent.
//!
//! ```
//! use gaz::{ContactBook, EndpointKind, HandleKind, LocalId, PersonaScope};
//! use gaz::intake::{AddressIntake, EndpointClaim, HandleClaim, NewLocalContact};
//! let mut book = ContactBook::new(PersonaScope::new("work"));
//! let claims = AddressIntake {
//!     handle: HandleClaim { kind: HandleKind::Acct, value: "acct:alice@example.org".into() },
//!     handles: vec![],
//!     endpoints: vec![EndpointClaim { kind: EndpointKind::Http, address: "https://example.org/alice".into() }],
//! };
//! let first = book.intake_addresses(&claims, Some(NewLocalContact {
//!     id: LocalId::from_random([1;16]), petname: "Alice".into(),
//! }))?;
//! assert!(first.created);
//! assert!(!book.intake_addresses(&claims, None)?.changed());
//! # Ok::<(), gaz::intake::IntakeError>(())
//! ```

use core::fmt;

use crate::{Anchor, EndpointKind, Handle, HandleKind, LocalId};

mod acct;
mod apply;

pub use acct::normalize_acct_handle;

/// A resolver's handle claim, carrying no local binding or trust conclusion.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HandleClaim {
    /// Handle family; the same text in a different family is a different claim.
    pub kind: HandleKind,
    /// Original handle spelling, retained when first recorded.
    pub value: String,
}

impl HandleClaim {
    /// Comparison key used by intake, distinct from human lookup normalization.
    pub fn comparison_key(&self) -> Result<String, IntakeError> {
        nonblank(&self.value, "handle")?;
        match &self.kind {
            HandleKind::Acct => normalize_acct_handle(&self.value),
            HandleKind::Other(kind) => {
                nonblank(kind, "handle kind")?;
                Ok(self.value.clone())
            },
            _ => Ok(self.value.clone()),
        }
    }

    pub(super) fn matches(&self, held: &Handle, comparison: &str) -> bool {
        if self.kind != held.kind {
            return false;
        }
        let claim = Self {
            kind: held.kind.clone(),
            value: held.value.clone(),
        };
        claim.comparison_key().is_ok_and(|key| key == comparison)
    }
}

/// A resolver's protocol address, carrying no trust or usage history.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EndpointClaim {
    /// Protocol family.
    pub kind: EndpointKind,
    /// Exact protocol address. Gaz does not interpret opaque address syntax.
    pub address: String,
}

/// Address claims for one primary handle. Secondary handles never select it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AddressIntake {
    /// The queried handle used to locate exactly one existing contact.
    pub handle: HandleClaim,
    /// Other advertised names, added only as unverified claims.
    pub handles: Vec<HandleClaim>,
    /// Advertised addresses, added only as unverified claims.
    pub endpoints: Vec<EndpointClaim>,
}

/// Host-selected local identity for a person not already in this book.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NewLocalContact {
    /// Caller-minted fresh local id; collisions refuse instead of overwriting.
    pub id: LocalId,
    /// The host/user's petname, never supplied implicitly by a resolver.
    pub petname: String,
}

/// The committed effect of a successful intake.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IntakeOutcome {
    /// The stable contact anchor; neither handles nor aliases replace it.
    pub anchor: Anchor,
    /// Whether a local, keyless contact was created.
    pub created: bool,
    /// Newly added handles, including the primary handle when absent.
    pub handles_added: usize,
    /// Newly added protocol addresses.
    pub endpoints_added: usize,
}

impl IntakeOutcome {
    /// Whether this intake changed the book. Replaying the same claims is false.
    pub fn changed(&self) -> bool {
        self.created || self.handles_added != 0 || self.endpoints_added != 0
    }
}

/// Refused intake. The book remains unchanged on every error.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IntakeError {
    /// Invalid or empty claim text; timestamp/trust fields are absent by type.
    InvalidClaim(&'static str),
    /// No contact matches the primary handle; the host must supply a local id.
    LocalContactRequired,
    /// More than one contact holds the primary typed handle.
    Ambiguous(Vec<Anchor>),
    /// An advertised secondary handle is already held by another contact.
    HandleConflict {
        /// The conflicting claim.
        handle: HandleClaim,
        /// Other contacts holding it, in anchor order.
        anchors: Vec<Anchor>,
    },
    /// The fallback local id belongs to an existing contact.
    LocalAnchorOccupied(Anchor),
}

impl fmt::Display for IntakeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidClaim(field) => write!(f, "invalid intake {field}"),
            Self::LocalContactRequired => {
                f.write_str("intake requires a host-selected local contact")
            },
            Self::Ambiguous(anchors) => {
                write!(f, "intake handle is held by {} contacts", anchors.len())
            },
            Self::HandleConflict { anchors, .. } => write!(
                f,
                "intake alias is held by {} other contacts",
                anchors.len()
            ),
            Self::LocalAnchorOccupied(anchor) => {
                write!(f, "intake local anchor already exists: {anchor}")
            },
        }
    }
}

impl std::error::Error for IntakeError {}

pub(super) fn nonblank(value: &str, field: &'static str) -> Result<(), IntakeError> {
    if value.trim().is_empty() || value.chars().any(char::is_control) {
        Err(IntakeError::InvalidClaim(field))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests;
