// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Portable reading of a derived link and its source assertions.
//!
//! These values describe a read projection only. They do not mint a pair
//! entity or authorize a write to the source.

use serde::{Deserialize, Serialize};

use crate::{PublicSourceRevision, SourceBinding};

/// The store whose assertions form a link.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LinkStratum {
    Resource,
    Surface,
}

/// Stable endpoint identity within a stratum. Resource endpoints use their
/// canonical IRI; surface endpoints use their stable source identity.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct LinkKey {
    pub stratum: LinkStratum,
    pub first: String,
    pub second: String,
}

impl LinkKey {
    pub fn new(stratum: LinkStratum, a: impl Into<String>, b: impl Into<String>) -> Self {
        let (a, b) = (a.into(), b.into());
        if a <= b {
            Self {
                stratum,
                first: a,
                second: b,
            }
        } else {
            Self {
                stratum,
                first: b,
                second: a,
            }
        }
    }
}

/// Public identity and revision of the store that disclosed this reading.
/// A missing source is explicit; process-local graph generations are not
/// public revisions.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum LinkSource {
    Available {
        binding: SourceBinding,
        revision: LinkSourceRevision,
    },
    Unavailable {
        reason: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum LinkSourceRevision {
    Available { revision: PublicSourceRevision },
    Unavailable { reason: String },
}

/// Portable equivalent of a kernel graph scope.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind", content = "value")]
pub enum LinkGraphScope {
    Default,
    Source,
    User,
    Agent,
    Moot,
    Custom(String),
}

/// One endpoint in the source bucket, with its source-local stable identity
/// and optional canonical address for display or cross-store resolution.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct LinkEndpoint {
    pub id: String,
    pub address: Option<String>,
}

/// One exact, independently addressable semantic assertion.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LinkAssertion {
    pub id: String,
    pub predicate: String,
    pub recognized_kind: Option<String>,
    pub label: Option<String>,
    pub scope: LinkGraphScope,
    pub asserter: Option<String>,
    /// First recorded assertion time. Absence stays undated.
    #[serde(default)]
    pub first_asserted_ms: Option<u64>,
    /// Most recent confirmation time; kept under the P1 field name.
    pub asserted_at_ms: Option<u64>,
    /// A retained withdrawal tombstone. Held-only reads omit withdrawn assertions.
    #[serde(default)]
    pub withdrawn: Option<LinkWithdrawal>,
    #[serde(default)]
    pub basis: LinkAssertionBasis,
}

/// Record-time information for an assertion withdrawal.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LinkWithdrawal {
    pub at_ms: Option<u64>,
    pub by: Option<String>,
}

/// Portable read-only projection of why an assertion exists.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LinkAssertionBasis {
    #[serde(default)]
    pub note: Option<String>,
    #[serde(default)]
    pub method: Option<LinkAssertionMethod>,
    /// Stored order is preserved; reason-key comparison belongs to admission.
    #[serde(default)]
    pub evidence: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LinkAssertionMethod {
    pub iri: String,
    pub version: Option<String>,
}

/// A currently stored family sidecar, retained for P1 reads without
/// manufacturing an assertion identity or write handle.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LinkLegacyFamily {
    pub family: String,
    pub data: serde_json::Value,
}

/// A read-only legacy sidecar member. It has no assertion ID and cannot be
/// addressed for a source mutation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LinkLegacyMember {
    pub family: String,
    pub sub_kind: Option<String>,
}

/// One directed source bucket contributing to a link. Distinct parallel
/// buckets remain distinct entries, even when their members have equal data.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LinkBucket {
    pub displayed_from: String,
    pub displayed_to: String,
    pub source_from: LinkEndpoint,
    pub source_to: LinkEndpoint,
    pub assertions: Vec<LinkAssertion>,
    pub legacy_members: Vec<LinkLegacyMember>,
    /// Transitional raw summaries retained only for Graphshell's current
    /// inspector wire/output compatibility; not the portable member model.
    pub legacy_summaries: Vec<LinkLegacyFamily>,
}

/// Link reading over the currently available source snapshot.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LinkReading {
    pub key: LinkKey,
    pub buckets: Vec<LinkBucket>,
}

/// One displayed endpoint pair may resolve to multiple stratum links. Source
/// and coverage describe the shared disclosure operation.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LinkPairReading {
    pub source: LinkSource,
    pub links: Vec<LinkReading>,
    pub coverage: LinkCoverage,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LinkCoverageLayer {
    Possession,
    Residency,
    Disclosure,
    Synchronization,
    Projection,
}

/// A known limit on this supplied reading, not a claim of world completeness.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LinkCoverageLimit {
    pub layer: LinkCoverageLayer,
    pub reason: String,
    pub resources: Vec<String>,
    pub surfaces: Vec<String>,
    pub count: Option<usize>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LinkCoverage {
    pub limits: Vec<LinkCoverageLimit>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_are_unordered_within_their_stratum() {
        assert_eq!(
            LinkKey::new(LinkStratum::Resource, "urn:z", "urn:a"),
            LinkKey::new(LinkStratum::Resource, "urn:a", "urn:z")
        );
        assert_ne!(
            LinkKey::new(LinkStratum::Resource, "urn:a", "urn:z"),
            LinkKey::new(LinkStratum::Surface, "urn:a", "urn:z")
        );
    }

    #[test]
    fn source_and_revision_availability_round_trip_explicitly() {
        let states = [
            LinkSource::Unavailable {
                reason: "no owning authority".into(),
            },
            LinkSource::Available {
                binding: SourceBinding {
                    authority: "mere:test".into(),
                    domain: "graph".into(),
                    resource: "g1".into(),
                },
                revision: LinkSourceRevision::Unavailable {
                    reason: "no public revision".into(),
                },
            },
            LinkSource::Available {
                binding: SourceBinding {
                    authority: "mere:test".into(),
                    domain: "graph".into(),
                    resource: "g1".into(),
                },
                revision: LinkSourceRevision::Available {
                    revision: PublicSourceRevision::new("r7"),
                },
            },
        ];
        for state in states {
            let encoded = serde_json::to_vec(&state).unwrap();
            assert_eq!(
                serde_json::from_slice::<LinkSource>(&encoded).unwrap(),
                state
            );
        }
    }

    #[test]
    fn prior_p1_assertion_json_defaults_new_lifecycle_and_basis_fields() {
        let old = serde_json::json!({
            "id": "urn:assertion:old",
            "predicate": "https://example.test/cites",
            "recognized_kind": "cites",
            "label": "an existing claim",
            "scope": {"kind": "default"},
            "asserter": "urn:agent:alice",
            "asserted_at_ms": 42
        });

        let decoded: LinkAssertion = serde_json::from_value(old).unwrap();
        assert_eq!(decoded.asserted_at_ms, Some(42));
        assert_eq!(decoded.first_asserted_ms, None);
        assert_eq!(decoded.withdrawn, None);
        assert_eq!(decoded.basis, LinkAssertionBasis::default());
    }
}
