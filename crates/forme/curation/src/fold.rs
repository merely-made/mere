// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! View-local graph folds (mer3ly site plan, Rulings 153, 163 and 164).
//!
//! A fold is curation, not a source-graph group: it records which source
//! members one view draws as a single stand-in. Members are opaque strings,
//! so this crate stays serde-only (SE73); a host converts its own ids at its
//! edge. A host serving a scene lowers this record into `sceno::Fold`, which
//! has the same stand-in, rule and label.
//!
//! The wire is versioned. [`FOLD_RECORD_VERSION`] 1 was forme's UUID record
//! with no stand-in, rule or label; it still loads, migrated by
//! [`FoldRecordV1::migrate`] to a summary stand-in with neither. Any other
//! unknown version is refused rather than read with new semantics.

use std::fmt;

use serde::{Deserialize, Serialize};

/// The record's current wire version.
pub const FOLD_RECORD_VERSION: u16 = 2;

/// Stable identity of one fold, distinct from every source member. Opaque:
/// the host mints it (pictograph uses a UUID string).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FoldId(String);

impl FoldId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for FoldId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// How relations crossing a fold boundary read in the current projection.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum FoldBoundaryPolicy {
    /// Internal cells disappear. Boundary cells attach to the stand-in and
    /// bundle only with cells of the same relation family.
    #[default]
    BundleByRelationFamily,
}

/// A direction through relations, from the rule's root.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FoldDirection {
    Outgoing,
    Incoming,
}

/// What a view draws in the fold's place.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FoldStandIn {
    /// One member stays drawn, carrying the "+N" of the others.
    Member(String),
    /// A synthetic body stands for every member.
    Summary {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        label: Option<String>,
    },
}

/// How the host chose the members. Recorded, never re-run here.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FoldRule {
    Selection,
    /// `root` and everything reachable from it through relations of `family`,
    /// followed in `direction`.
    Descendants {
        root: String,
        family: String,
        direction: FoldDirection,
    },
}

/// One durable, view-local fold.
///
/// `members` are kept sorted and distinct so serialization, undo comparisons
/// and boundary accounting agree. `source_scope` is opaque: it lets a host
/// refuse a fold made against another graph, cursor or lens.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "FoldRecordWire")]
pub struct FoldRecord {
    pub version: u16,
    pub id: FoldId,
    pub source_scope: String,
    pub members: Vec<String>,
    pub stand_in: FoldStandIn,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rule: Option<FoldRule>,
    /// The host's own words for the fold. Never blank when present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    pub boundary_policy: FoldBoundaryPolicy,
}

/// The version 1 record, as forme wrote it (UUIDs read as strings).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FoldRecordV1 {
    pub version: u16,
    pub id: FoldId,
    pub source_scope: String,
    pub members: Vec<String>,
    pub boundary_policy: FoldBoundaryPolicy,
}

impl FoldRecordV1 {
    /// The version 2 record a version 1 one means (Ruling 164): the same id,
    /// scope and members, a summary stand-in, no rule and no label.
    pub fn migrate(self) -> FoldRecord {
        FoldRecord {
            version: FOLD_RECORD_VERSION,
            id: self.id,
            source_scope: self.source_scope,
            members: self.members,
            stand_in: FoldStandIn::Summary { label: None },
            rule: None,
            label: None,
            boundary_policy: self.boundary_policy,
        }
    }
}

/// Why a fold record was refused.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FoldRecordError {
    UnsupportedVersion {
        found: u16,
    },
    /// A version 2 record names no stand-in.
    MissingStandIn,
    /// A version 1 record carries a field version 1 never had.
    NotVersionOne,
    TooFewMembers {
        count: usize,
    },
    /// Members are not sorted and distinct.
    Unordered,
    StandInNotMember(String),
    RootNotMember(String),
    BlankLabel,
}

impl fmt::Display for FoldRecordError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedVersion { found } => {
                write!(f, "fold record version {found} is not supported")
            },
            Self::MissingStandIn => write!(f, "a version 2 fold record names no stand-in"),
            Self::NotVersionOne => write!(
                f,
                "a version 1 fold record carries a stand-in, rule or label"
            ),
            Self::TooFewMembers { count } => {
                write!(f, "a fold needs at least two members, found {count}")
            },
            Self::Unordered => write!(f, "fold members are not sorted and distinct"),
            Self::StandInNotMember(id) => write!(f, "fold stand-in {id} is not a member"),
            Self::RootNotMember(id) => write!(f, "fold rule root {id} is not a member"),
            Self::BlankLabel => write!(f, "a fold's label is blank"),
        }
    }
}

impl std::error::Error for FoldRecordError {}

impl FoldRecord {
    /// A fold over `members`, sorted and deduplicated. Refused when fewer
    /// than two distinct members remain, or when the stand-in, the rule's
    /// root or the label does not hold.
    pub fn new(
        id: FoldId,
        source_scope: impl Into<String>,
        members: impl IntoIterator<Item = impl Into<String>>,
        stand_in: FoldStandIn,
        rule: Option<FoldRule>,
        label: Option<String>,
    ) -> Result<Self, FoldRecordError> {
        let mut members: Vec<String> = members.into_iter().map(Into::into).collect();
        members.sort();
        members.dedup();
        let record = Self {
            version: FOLD_RECORD_VERSION,
            id,
            source_scope: source_scope.into(),
            members,
            stand_in,
            rule,
            label,
            boundary_policy: FoldBoundaryPolicy::BundleByRelationFamily,
        };
        record.validate()?;
        Ok(record)
    }

    /// A selection fold drawn as an unlabelled summary.
    pub fn selection(
        id: FoldId,
        source_scope: impl Into<String>,
        members: impl IntoIterator<Item = impl Into<String>>,
    ) -> Result<Self, FoldRecordError> {
        Self::new(
            id,
            source_scope,
            members,
            FoldStandIn::Summary { label: None },
            Some(FoldRule::Selection),
            None,
        )
    }

    /// Check the record alone; whether its members exist is the host's check.
    pub fn validate(&self) -> Result<(), FoldRecordError> {
        if self.version != FOLD_RECORD_VERSION {
            return Err(FoldRecordError::UnsupportedVersion {
                found: self.version,
            });
        }
        if self.members.len() < 2 {
            return Err(FoldRecordError::TooFewMembers {
                count: self.members.len(),
            });
        }
        if self.members.windows(2).any(|pair| pair[0] >= pair[1]) {
            return Err(FoldRecordError::Unordered);
        }
        if let FoldStandIn::Member(member) = &self.stand_in
            && !self.contains(member)
        {
            return Err(FoldRecordError::StandInNotMember(member.clone()));
        }
        if let Some(FoldRule::Descendants { root, .. }) = &self.rule
            && !self.contains(root)
        {
            return Err(FoldRecordError::RootNotMember(root.clone()));
        }
        if self
            .label
            .as_ref()
            .is_some_and(|label| label.trim().is_empty())
        {
            return Err(FoldRecordError::BlankLabel);
        }
        Ok(())
    }

    /// Whether the fold has `member` among its members.
    pub fn contains(&self, member: &str) -> bool {
        self.members
            .binary_search_by(|probe| probe.as_str().cmp(member))
            .is_ok()
    }

    /// The member that stays drawn, when the stand-in is a member.
    pub fn stand_in_member(&self) -> Option<&str> {
        match &self.stand_in {
            FoldStandIn::Member(member) => Some(member),
            FoldStandIn::Summary { .. } => None,
        }
    }

    /// The members the fold hides: all but a member stand-in.
    pub fn hidden(&self) -> impl Iterator<Item = &str> {
        let keep = self.stand_in_member();
        self.members
            .iter()
            .map(String::as_str)
            .filter(move |member| Some(*member) != keep)
    }

    /// The "+N" the stand-in carries.
    pub fn hidden_count(&self) -> usize {
        self.hidden().count()
    }

    /// Whether two folds share a member. Active folds never do.
    pub fn overlaps(&self, other: &FoldRecord) -> bool {
        self.members.iter().any(|member| other.contains(member))
    }
}

/// Every version's fields, read before the version picks a meaning.
#[derive(Deserialize)]
struct FoldRecordWire {
    version: u16,
    id: FoldId,
    source_scope: String,
    members: Vec<String>,
    #[serde(default)]
    stand_in: Option<FoldStandIn>,
    #[serde(default)]
    rule: Option<FoldRule>,
    #[serde(default)]
    label: Option<String>,
    boundary_policy: FoldBoundaryPolicy,
}

impl TryFrom<FoldRecordWire> for FoldRecord {
    type Error = FoldRecordError;

    fn try_from(wire: FoldRecordWire) -> Result<Self, Self::Error> {
        match wire.version {
            1 => {
                if wire.stand_in.is_some() || wire.rule.is_some() || wire.label.is_some() {
                    return Err(FoldRecordError::NotVersionOne);
                }
                Ok(FoldRecordV1 {
                    version: 1,
                    id: wire.id,
                    source_scope: wire.source_scope,
                    members: wire.members,
                    boundary_policy: wire.boundary_policy,
                }
                .migrate())
            },
            FOLD_RECORD_VERSION => Ok(Self {
                version: FOLD_RECORD_VERSION,
                id: wire.id,
                source_scope: wire.source_scope,
                members: wire.members,
                stand_in: wire.stand_in.ok_or(FoldRecordError::MissingStandIn)?,
                rule: wire.rule,
                label: wire.label,
                boundary_policy: wire.boundary_policy,
            }),
            found => Err(FoldRecordError::UnsupportedVersion { found }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const A: &str = "00000000-0000-0000-0000-000000000001";
    const B: &str = "00000000-0000-0000-0000-000000000002";
    const C: &str = "00000000-0000-0000-0000-000000000003";

    fn id() -> FoldId {
        FoldId::new("00000000-0000-0000-0000-0000000000f0")
    }

    #[test]
    fn a_selection_canonicalizes_its_members() {
        let fold = FoldRecord::selection(id(), "graph:local", [C, A, C]).expect("two members");
        assert_eq!(fold.version, FOLD_RECORD_VERSION);
        assert_eq!(fold.members, vec![A, C]);
        assert!(fold.contains(A) && !fold.contains(B));
        assert_eq!(fold.rule, Some(FoldRule::Selection));
        assert_eq!(fold.hidden_count(), 2, "a summary hides every member");
        assert_eq!(
            FoldRecord::selection(id(), "graph:local", [A, A]),
            Err(FoldRecordError::TooFewMembers { count: 1 })
        );
    }

    #[test]
    fn a_member_stand_in_counts_the_others() {
        let fold = FoldRecord::new(
            id(),
            "graph:local",
            [A, B, C],
            FoldStandIn::Member(A.into()),
            Some(FoldRule::Descendants {
                root: A.into(),
                family: "containment".into(),
                direction: FoldDirection::Outgoing,
            }),
            Some("A's contents".into()),
        )
        .expect("valid");
        assert_eq!(fold.hidden().collect::<Vec<_>>(), vec![B, C]);
        assert_eq!(fold.hidden_count(), 2);
        let other = FoldRecord::selection(id(), "graph:local", [C, "z"]).unwrap();
        assert!(fold.overlaps(&other));
    }

    #[test]
    fn invalid_records_are_refused() {
        let stray = FoldRecord::new(id(), "s", [A, B], FoldStandIn::Member(C.into()), None, None);
        assert_eq!(stray, Err(FoldRecordError::StandInNotMember(C.into())));
        let rootless = FoldRecord::new(
            id(),
            "s",
            [A, B],
            FoldStandIn::Summary { label: None },
            Some(FoldRule::Descendants {
                root: C.into(),
                family: "containment".into(),
                direction: FoldDirection::Incoming,
            }),
            None,
        );
        assert_eq!(rootless, Err(FoldRecordError::RootNotMember(C.into())));
        let blank = FoldRecord::new(
            id(),
            "s",
            [A, B],
            FoldStandIn::Summary { label: None },
            None,
            Some(" ".into()),
        );
        assert_eq!(blank, Err(FoldRecordError::BlankLabel));
        let mut unordered = FoldRecord::selection(id(), "s", [A, B]).unwrap();
        unordered.members.reverse();
        assert_eq!(unordered.validate(), Err(FoldRecordError::Unordered));
    }

    #[test]
    fn version_two_round_trips_with_its_wire_pinned() {
        let fold = FoldRecord::selection(id(), "graph:local", [A, B]).unwrap();
        let wire = serde_json::to_string(&fold).unwrap();
        assert_eq!(
            wire,
            format!(
                r#"{{"version":2,"id":"{}","source_scope":"graph:local","members":["{A}","{B}"],"stand_in":{{"Summary":{{}}}},"rule":"Selection","boundary_policy":"BundleByRelationFamily"}}"#,
                id()
            ),
            "absent label stays off the wire"
        );
        assert_eq!(serde_json::from_str::<FoldRecord>(&wire).unwrap(), fold);
    }

    /// Ruling 164: a saved version 1 record loads as a summary with no rule
    /// and no label, keeping its id and members, and writes back as version 2.
    #[test]
    fn a_saved_version_one_record_migrates_on_load() {
        let v1 = format!(
            r#"{{"version":1,"id":"{}","source_scope":"session:alpha/graph:current","members":["{A}","{C}"],"boundary_policy":"BundleByRelationFamily"}}"#,
            id()
        );
        let fold: FoldRecord = serde_json::from_str(&v1).expect("v1 loads");
        assert_eq!(
            fold,
            FoldRecord {
                version: 2,
                id: id(),
                source_scope: "session:alpha/graph:current".into(),
                members: vec![A.into(), C.into()],
                stand_in: FoldStandIn::Summary { label: None },
                rule: None,
                label: None,
                boundary_policy: FoldBoundaryPolicy::BundleByRelationFamily,
            }
        );
        assert!(fold.validate().is_ok());
        let written = serde_json::to_string(&fold).unwrap();
        assert!(written.starts_with(r#"{"version":2,"#), "{written}");
        let explicit: FoldRecordV1 = serde_json::from_str(&v1).unwrap();
        assert_eq!(explicit.migrate(), fold);
    }

    #[test]
    fn unknown_or_mixed_versions_are_refused() {
        let base = format!(
            r#""id":"{}","source_scope":"s","members":["{A}","{B}"],"boundary_policy":"BundleByRelationFamily""#,
            id()
        );
        let read = |json: String| serde_json::from_str::<FoldRecord>(&json).map(|_| ());
        let error = read(format!(r#"{{"version":3,{base}}}"#)).unwrap_err();
        assert!(error.to_string().contains("version 3"), "{error}");
        let error = read(format!(r#"{{"version":2,{base}}}"#)).unwrap_err();
        assert!(error.to_string().contains("no stand-in"), "{error}");
        let error = read(format!(
            r#"{{"version":1,{base},"stand_in":{{"Summary":{{}}}}}}"#
        ))
        .unwrap_err();
        assert!(error.to_string().contains("version 1"), "{error}");
    }
}
