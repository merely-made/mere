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

/// The layout selected when a link reading is opened in a swatch.
///
/// These are compositions of existing arrangements, not new arrangement
/// families. List uses `Stack`; Timeline uses the existing numeric Timeline
/// axis and places disclosed lanes as rows within it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind", content = "config")]
pub enum OpenedLinkLayout {
    List {
        group_by: LinkListGroup,
        stack: sceno::Stack,
    },
    Timeline {
        lane_by: LinkTimelineLane,
        timeline: sceno::Timeline,
    },
}

impl OpenedLinkLayout {
    /// Choose Timeline only when dated exact assertions are a majority of all
    /// exact and legacy members. Legacy members have no first-time and count
    /// as undated. Empty and mostly undated readings use List by family.
    pub fn default_for(reading: &LinkPairReading) -> Self {
        let mut dated = 0usize;
        let mut members = 0usize;
        for link in &reading.links {
            for bucket in &link.buckets {
                for assertion in &bucket.assertions {
                    members += 1;
                    dated += usize::from(assertion.first_asserted_ms.is_some());
                }
                members += bucket.legacy_members.len() + bucket.legacy_summaries.len();
            }
        }
        if dated > members / 2 {
            Self::Timeline {
                lane_by: LinkTimelineLane::Family,
                timeline: sceno::Timeline::default(),
            }
        } else {
            Self::List {
                group_by: LinkListGroup::Family,
                stack: sceno::Stack::default(),
            }
        }
    }
}

/// The grouping key shown above rows in the accessible list arrangement.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LinkListGroup {
    Family,
    /// Owning stratum and source endpoint pair; row direction remains visible.
    Source,
}

/// The grouping key shown as stacked rows inside the existing Timeline.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LinkTimelineLane {
    Family,
    Asserter,
    Direction,
}

/// Caller-supplied, view-local values used to complete an opened-link layout.
/// No current time is invented by the shared compiler.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct OpenedLinkOptions {
    /// Explicit clock for currently held intervals. `None` leaves their end
    /// unknown; the compiler never samples wall-clock time.
    pub now_ms: Option<u64>,
}

/// A reference into the owned source reading retained by a compiled view.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OpenedLinkEntryRef {
    pub link_index: usize,
    pub bucket_index: usize,
    pub member: OpenedLinkMemberRef,
}

/// The exact member kind and its index in its original source bucket.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind", content = "index")]
pub enum OpenedLinkMemberRef {
    Assertion(usize),
    LegacyMember(usize),
    LegacySummary(usize),
}

/// Resolve an opened entry to its original exact assertion or legacy source.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum OpenedLinkMember<'a> {
    Assertion(&'a LinkAssertion),
    LegacyMember(&'a LinkLegacyMember),
    LegacySummary(&'a LinkLegacyFamily),
}

impl OpenedLinkEntryRef {
    pub fn resolve<'a>(&self, reading: &'a LinkPairReading) -> Option<OpenedLinkMember<'a>> {
        let bucket = reading
            .links
            .get(self.link_index)?
            .buckets
            .get(self.bucket_index)?;
        match self.member {
            OpenedLinkMemberRef::Assertion(index) => bucket
                .assertions
                .get(index)
                .map(OpenedLinkMember::Assertion),
            OpenedLinkMemberRef::LegacyMember(index) => bucket
                .legacy_members
                .get(index)
                .map(OpenedLinkMember::LegacyMember),
            OpenedLinkMemberRef::LegacySummary(index) => bucket
                .legacy_summaries
                .get(index)
                .map(OpenedLinkMember::LegacySummary),
        }
    }
}

/// End-state of an assertion span. Missing withdrawal time and missing
/// caller clock remain distinct from a known open/closed interval.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind", content = "at_ms")]
pub enum LinkSpanEnd {
    WithdrawalAt(u64),
    WithdrawalTimeUnknown,
    OpenAt(u64),
    OpenTimeUnknown,
}

/// A dated interval in scene coordinates. Undated entries have no span but
/// still occupy an explicit Timeline lane and scene item.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct LinkSpanGeometry {
    pub start_ms: u64,
    pub end: LinkSpanEnd,
    pub x_start: f32,
    pub x_end: Option<f32>,
    pub y: f32,
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

    #[test]
    fn custom_assertion_scope_round_trips_without_normalization() {
        let scope = LinkGraphScope::Custom("urn:scope:review".into());
        let json = serde_json::to_string(&scope).expect("writes");
        assert_eq!(serde_json::from_str::<LinkGraphScope>(&json).expect("reads"), scope);
    }
}
