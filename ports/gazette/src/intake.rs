// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Supplied WebFinger results become unverified Gaz address claims.
//!
//! The queried account selects the contact. Returned aliases never select or
//! refile a contact, and DID strings remain names rather than key assertions.
//! This bounded adapter refuses account migration expressed only by a changed
//! JRD subject. RFC 7033 permits such subjects; admitting a migration needs a
//! separate checked identity path.

use std::fmt;

use gaz::intake::{AddressIntake, EndpointClaim, HandleClaim, IntakeError, normalize_acct_handle};
use gaz::{EndpointKind, HandleKind};
use iri_string::types::UriStr;

use crate::WebFingerImport;

/// An account-bound projection together with the resolver result it came from.
///
/// Gaz stores address claims, not resolver provenance. A host that retains the
/// resolution must keep this source alongside its own intake receipt. The
/// existing resolver's classified representation is preserved unchanged;
/// this is not a lossless archive of the original JRD response.
#[derive(Clone, Debug)]
pub struct WebFingerIntake {
    resource: String,
    source: WebFingerImport,
    addresses: AddressIntake,
}

/// Why a supplied resolution cannot be attached to the queried account.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WebFingerIntakeError {
    /// The query or returned subject is not a valid account handle.
    InvalidAccount(IntakeError),
    /// The returned JRD subject is not an explicit `acct:` URI.
    SubjectNotAccount,
    /// The returned subject identifies a different account.
    SubjectMismatch {
        /// The normalized account requested by the host.
        requested: String,
        /// The normalized account named by the resolver.
        returned: String,
    },
}

impl fmt::Display for WebFingerIntakeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidAccount(error) => write!(f, "invalid WebFinger intake account: {error}"),
            Self::SubjectNotAccount => f.write_str("WebFinger intake subject must be an acct: URI"),
            Self::SubjectMismatch {
                requested,
                returned,
            } => write!(
                f,
                "WebFinger subject {returned} differs from queried account {requested}"
            ),
        }
    }
}

impl std::error::Error for WebFingerIntakeError {}

impl WebFingerIntake {
    /// Project an already-resolved account without fetching or mutating a book.
    ///
    /// Only valid account/DID aliases and URI endpoint targets are projected.
    /// Unmapped or malformed targets remain in [`Self::source`]. All projected
    /// fields are claims: this adapter supplies neither keys nor trust state.
    pub fn from_import(
        resource: &str,
        source: WebFingerImport,
    ) -> Result<Self, WebFingerIntakeError> {
        let account =
            normalize_acct_handle(resource).map_err(WebFingerIntakeError::InvalidAccount)?;
        if !is_acct_uri(&source.subject) {
            return Err(WebFingerIntakeError::SubjectNotAccount);
        }
        let subject =
            normalize_acct_handle(&source.subject).map_err(WebFingerIntakeError::InvalidAccount)?;
        if account != subject {
            return Err(WebFingerIntakeError::SubjectMismatch {
                requested: account,
                returned: subject,
            });
        }

        let resource = account;
        let mut addresses = AddressIntake {
            handle: HandleClaim {
                kind: HandleKind::Acct,
                value: resource.clone(),
            },
            handles: Vec::new(),
            endpoints: Vec::new(),
        };
        for alias in &source.aliases {
            if let Some(account) = is_acct_uri(alias)
                .then(|| normalize_acct_handle(alias).ok())
                .flatten()
            {
                addresses.handles.push(HandleClaim {
                    kind: HandleKind::Acct,
                    value: account,
                });
            } else if valid_did(alias) {
                addresses.handles.push(HandleClaim {
                    kind: HandleKind::Did,
                    value: alias.clone(),
                });
            }
        }
        for (kind, values) in [
            (EndpointKind::Http, &source.profile_pages),
            (EndpointKind::Gemini, &source.gemini_capsules),
            (EndpointKind::Gopher, &source.gopher_resources),
            (EndpointKind::Misfin, &source.misfin_mailboxes),
            (EndpointKind::ActivityPub, &source.activitypub_actors),
        ] {
            for target in values {
                if valid_endpoint(&kind, target) {
                    addresses.endpoints.push(EndpointClaim {
                        kind: kind.clone(),
                        address: target.clone(),
                    });
                }
            }
        }
        for endpoint in &source.other_endpoints {
            if !endpoint.rel.trim().is_empty() && valid_uri(&endpoint.href) {
                addresses.endpoints.push(EndpointClaim {
                    kind: EndpointKind::Other(endpoint.rel.clone()),
                    address: endpoint.href.clone(),
                });
            }
        }
        Ok(Self {
            resource,
            source,
            addresses,
        })
    }

    /// The normalized account query that selected these claims.
    pub fn resource(&self) -> &str {
        &self.resource
    }

    /// The resolver result, including aliases and catch-all link metadata.
    pub fn source(&self) -> &WebFingerImport {
        &self.source
    }

    /// Unverified claims for [`gaz::ContactBook::intake_addresses`].
    pub fn addresses(&self) -> &AddressIntake {
        &self.addresses
    }
}

fn is_acct_uri(value: &str) -> bool {
    value
        .get(..5)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("acct:"))
        && value.trim() == value
}

fn valid_did(value: &str) -> bool {
    let Some((method, id)) = value
        .strip_prefix("did:")
        .and_then(|did| did.split_once(':'))
    else {
        return false;
    };
    !method.is_empty()
        && method
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
        && !id.is_empty()
        && id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"._:%-".contains(&byte))
        && !id.ends_with(':')
        && valid_uri(value)
}

fn valid_uri(value: &str) -> bool {
    if UriStr::new(value).is_err() {
        return false;
    }
    let Ok(uri) = url::Url::parse(value) else {
        return false;
    };
    match uri.scheme() {
        "http" | "https" | "gemini" | "gopher" | "misfin" => uri.host_str().is_some(),
        _ => true,
    }
}

fn valid_endpoint(kind: &EndpointKind, value: &str) -> bool {
    if !valid_uri(value) {
        return false;
    }
    let uri = url::Url::parse(value).expect("valid_uri checked URL parsing");
    match kind {
        EndpointKind::Http | EndpointKind::ActivityPub => matches!(uri.scheme(), "http" | "https"),
        EndpointKind::Gemini => uri.scheme() == "gemini",
        EndpointKind::Gopher => uri.scheme() == "gopher",
        EndpointKind::Misfin => uri.scheme() == "misfin",
        _ => true,
    }
}

#[cfg(test)]
mod tests;
