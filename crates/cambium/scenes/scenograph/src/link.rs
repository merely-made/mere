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

pub use sceno::{LinkKey, LinkMarks, LinkStratum};

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

/// One exact, independently addressable relation assertion. Registered
/// relation kinds use their canonical predicate IRI; unfamiliar predicates
/// remain unchanged. `recognized_kind` remains the semantic sub-kind
/// compatibility field and is absent for other families.
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

/// View-local marks and default additive contribution of eligible held members.
/// The source reading itself is never mutated by this summary.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LinkRestSummary {
    pub marks: LinkMarks,
    pub strength: f32,
    pub held_assertions: usize,
    pub legacy_members: usize,
}

impl LinkReading {
    /// Summarize this particular displayed appearance. Resource links can have
    /// several appearances; original source endpoints remain in each bucket.
    /// Callers filter view-hidden members before supplying the reading.
    pub fn rest_summary(&self, displayed_from: &str, displayed_to: &str) -> LinkRestSummary {
        let mut summary = LinkRestSummary::default();
        let mut supports = false;
        let mut contradicts = false;
        let mut seen = std::collections::BTreeSet::new();
        for bucket in &self.buckets {
            let forward =
                bucket.displayed_from == displayed_from && bucket.displayed_to == displayed_to;
            let reverse =
                bucket.displayed_from == displayed_to && bucket.displayed_to == displayed_from;
            if !forward && !reverse {
                continue;
            }
            let mut eligible = false;
            for assertion in bucket.assertions.iter().filter(|a| a.withdrawn.is_none()) {
                eligible = true;
                supports |= assertion.predicate == "https://mere.computer/ns/rel#supports";
                contradicts |= assertion.predicate == "https://mere.computer/ns/rel#contradicts";
                if seen.insert((
                    assertion.id.clone(),
                    serde_json::to_string(&assertion.scope).expect("scope serializes"),
                )) {
                    summary.held_assertions += 1;
                }
            }
            for member in &bucket.legacy_members {
                eligible = true;
                summary.legacy_members += 1;
                supports |=
                    member.family == "Semantic" && member.sub_kind.as_deref() == Some("Supports");
                contradicts |= member.family == "Semantic"
                    && member.sub_kind.as_deref() == Some("Contradicts");
            }
            // legacy_summaries duplicate members for old inspector output and
            // never add a second strength contribution.
            summary.marks.forward |= eligible && forward;
            summary.marks.reverse |= eligible && reverse && !forward;
        }
        summary.marks.opposed = supports && contradicts;
        summary.strength = (summary.held_assertions + summary.legacy_members) as f32;
        summary
    }
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
        assert_eq!(
            serde_json::from_str::<LinkGraphScope>(&json).expect("reads"),
            scope
        );
    }
    #[test]
    fn rest_summary_preserves_independent_reasons_and_excludes_withdrawal() {
        let assertion = |id: &str, predicate: &str, withdrawn: bool| -> LinkAssertion {
            serde_json::from_value(serde_json::json!({
                "id":id,"predicate":predicate,"recognized_kind":null,"label":null,
                "scope":{"kind":"default"},"asserter":null,"asserted_at_ms":null,
                "withdrawn":if withdrawn {serde_json::json!({"at_ms":7,"by":null})} else {serde_json::Value::Null}
            })).unwrap()
        };
        let bucket = |from: &str, to: &str, assertions| LinkBucket {
            displayed_from: from.into(),
            displayed_to: to.into(),
            source_from: LinkEndpoint {
                id: format!("source:{from}"),
                address: None,
            },
            source_to: LinkEndpoint {
                id: format!("source:{to}"),
                address: None,
            },
            assertions,
            legacy_members: vec![],
            legacy_summaries: vec![],
        };
        let mut reading = LinkReading {
            key: LinkKey::new(LinkStratum::Resource, "urn:a", "urn:b"),
            buckets: vec![
                bucket(
                    "surface:a",
                    "surface:b",
                    vec![
                        assertion("one", "https://mere.computer/ns/rel#supports", false),
                        assertion("two", "https://mere.computer/ns/rel#supports", false),
                    ],
                ),
                bucket(
                    "surface:b",
                    "surface:a",
                    vec![assertion(
                        "ended",
                        "https://mere.computer/ns/rel#contradicts",
                        true,
                    )],
                ),
            ],
        };
        let before = reading.clone();
        let summary = reading.rest_summary("surface:a", "surface:b");
        assert_eq!(summary.held_assertions, 2);
        assert_eq!(summary.strength, 2.0);
        assert_eq!(
            summary.marks,
            LinkMarks {
                forward: true,
                reverse: false,
                opposed: false
            }
        );
        assert_eq!(
            reading, before,
            "rest drawing cannot change assertion history"
        );
        reading.buckets[1].assertions.push(assertion(
            "three",
            "https://mere.computer/ns/rel#contradicts",
            false,
        ));
        let summary = reading.rest_summary("surface:a", "surface:b");
        assert_eq!(summary.strength, 3.0);
        assert_eq!(
            summary.marks,
            LinkMarks {
                forward: true,
                reverse: true,
                opposed: true
            }
        );
        assert_eq!(
            reading.rest_summary("unrelated:a", "unrelated:b"),
            LinkRestSummary::default()
        );
        assert_eq!(
            reading.rest_summary("surface:b", "surface:a").marks,
            summary.marks.reversed()
        );
        reading.buckets[0].legacy_members.push(LinkLegacyMember {
            family: "Traversal".into(),
            sub_kind: None,
        });
        reading.buckets[0].legacy_summaries.push(LinkLegacyFamily {
            family: "Traversal".into(),
            data: serde_json::json!({}),
        });
        let summary = reading.rest_summary("surface:a", "surface:b");
        assert_eq!(
            (
                summary.held_assertions,
                summary.legacy_members,
                summary.strength
            ),
            (3, 1, 4.0)
        );
    }
}
