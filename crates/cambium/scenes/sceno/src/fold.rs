// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Folds: several scene instances drawn as one stand-in.
//!
//! A fold is a scene fact, beside [`crate::Region`], so a scene that is served,
//! traced, read remotely or frozen carries it as it is. It is not a source
//! fact: folding never adds, removes or rewrites a source, and a host that
//! keeps its own native fold record lowers that record into this one.
//!
//! The fact alone implies hiding. Every member except a member stand-in is
//! hidden; readers derive that with [`FoldEffect`] rather than reading a
//! `visible` flag, and the "+N" a stand-in shows is [`Fold::hidden_count`].
//! Members keep their own `visible`, transform and slot, so unfolding is
//! removing the fact and nothing else.
//!
//! Membership is the host's choice. The fact records how the host chose,
//! through an optional [`FoldRule`], so a reader can explain the fold and a
//! host can re-derive it; nothing here re-derives membership.
//!
//! Two active folds never share an instance. Nesting is deliberately not
//! modelled: a host that folds an enclosing set unfolds the inner fold first,
//! or folds the union, so "which fold hides this" always has one answer.

use std::collections::{HashMap, HashSet};
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::scene::InstanceId;

/// A direction through relations: from an instance along its outgoing
/// relations, or back along its incoming ones. For a boundary bundle it is
/// relative to the fold: `Outgoing` leaves the fold, `Incoming` enters it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum FoldDirection {
    Outgoing,
    Incoming,
}

/// What a reader draws in the fold's place.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StandIn {
    /// One of the members stays drawn, carrying the "+N" of the others (the
    /// site's root with its badge).
    Member(InstanceId),
    /// A synthetic body stands for every member (pictograph's summary). It
    /// has no instance of its own; a reader places it at the members'
    /// centroid, which folding never moves.
    Summary {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        label: Option<String>,
    },
}

/// How the host chose the members. Recorded, never re-run here.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FoldRule {
    /// The members are what someone selected.
    Selection,
    /// The members are `root` and every instance reachable from it through
    /// relations of `family`, followed in `direction`.
    Descendants {
        root: InstanceId,
        family: String,
        direction: FoldDirection,
    },
}

/// Relations crossing a fold's boundary, bundled by the instance outside,
/// their direction and their family.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BoundaryBundle {
    pub outside: InstanceId,
    pub direction: FoldDirection,
    /// The relations' kind, as [`crate::RoutedRelation::kind`] names it.
    pub family: Option<String>,
    pub count: u32,
}

/// Optional boundary accounting a host computed when it folded.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct FoldBoundary {
    /// Relations with both ends inside the fold.
    pub internal_relations: u32,
    pub bundles: Vec<BoundaryBundle>,
}

/// One fold over scene instances.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fold {
    /// At least two distinct live instances, in the host's order.
    pub members: Vec<InstanceId>,
    pub stand_in: StandIn,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rule: Option<FoldRule>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub boundary: Option<FoldBoundary>,
}

impl Fold {
    /// The member that stays drawn, when the stand-in is a member.
    pub fn stand_in_member(&self) -> Option<InstanceId> {
        match self.stand_in {
            StandIn::Member(member) => Some(member),
            StandIn::Summary { .. } => None,
        }
    }

    /// The members this fold hides: every member but a member stand-in. A
    /// fold never hides its own stand-in.
    pub fn hidden(&self) -> impl Iterator<Item = InstanceId> + '_ {
        let keep = self.stand_in_member();
        self.members
            .iter()
            .copied()
            .filter(move |member| Some(*member) != keep)
    }

    /// The "+N" the stand-in carries: how many members it hides.
    pub fn hidden_count(&self) -> usize {
        self.hidden().count()
    }

    /// Check this fold alone against the live instances.
    pub fn validate(&self, is_active: impl Fn(InstanceId) -> bool) -> Result<(), FoldError> {
        if self.members.len() < 2 {
            return Err(FoldError::TooFewMembers {
                count: self.members.len(),
            });
        }
        let mut seen = HashSet::new();
        for &member in &self.members {
            if !seen.insert(member) {
                return Err(FoldError::DuplicateMember(member));
            }
            if !is_active(member) {
                return Err(FoldError::AbsentMember(member));
            }
        }
        if let StandIn::Member(member) = self.stand_in
            && !seen.contains(&member)
        {
            return Err(FoldError::StandInNotMember(member));
        }
        if let Some(FoldRule::Descendants { root, .. }) = &self.rule
            && !seen.contains(root)
        {
            return Err(FoldError::RootNotMember(*root));
        }
        if let Some(boundary) = &self.boundary {
            for bundle in &boundary.bundles {
                if seen.contains(&bundle.outside) {
                    return Err(FoldError::BoundaryInside(bundle.outside));
                }
                if !is_active(bundle.outside) {
                    return Err(FoldError::BoundaryAbsent(bundle.outside));
                }
            }
        }
        Ok(())
    }
}

/// Why a fold, or a set of folds, was refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FoldError {
    /// A fold needs at least two members.
    TooFewMembers {
        count: usize,
    },
    DuplicateMember(InstanceId),
    /// A member is absent or tombstoned.
    AbsentMember(InstanceId),
    /// A member stand-in is not one of the members.
    StandInNotMember(InstanceId),
    /// A descendants rule's root is not one of the members.
    RootNotMember(InstanceId),
    /// A boundary bundle's outside instance is a member.
    BoundaryInside(InstanceId),
    /// A boundary bundle's outside instance is absent or tombstoned.
    BoundaryAbsent(InstanceId),
    /// Two active folds share an instance.
    Overlap {
        instance: InstanceId,
        first: u32,
        second: u32,
    },
}

impl fmt::Display for FoldError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooFewMembers { count } => {
                write!(f, "a fold needs at least two members, found {count}")
            },
            Self::DuplicateMember(id) => write!(f, "fold member {} is repeated", id.0),
            Self::AbsentMember(id) => write!(f, "fold member {} is absent or tombstoned", id.0),
            Self::StandInNotMember(id) => {
                write!(f, "fold stand-in {} is not one of its members", id.0)
            },
            Self::RootNotMember(id) => {
                write!(f, "fold rule root {} is not one of its members", id.0)
            },
            Self::BoundaryInside(id) => {
                write!(f, "fold boundary instance {} is a member", id.0)
            },
            Self::BoundaryAbsent(id) => {
                write!(f, "fold boundary instance {} is absent or tombstoned", id.0)
            },
            Self::Overlap {
                instance,
                first,
                second,
            } => write!(
                f,
                "instance {} is in folds {first} and {second}; folds do not nest",
                instance.0
            ),
        }
    }
}

impl std::error::Error for FoldError {}

/// Check a set of active folds, each with its table index: every fold alone,
/// then that no two share an instance.
pub fn validate_folds<'a>(
    folds: impl IntoIterator<Item = (u32, &'a Fold)>,
    is_active: impl Fn(InstanceId) -> bool,
) -> Result<(), FoldError> {
    let mut owner: HashMap<InstanceId, u32> = HashMap::new();
    for (index, fold) in folds {
        fold.validate(&is_active)?;
        for &member in &fold.members {
            if let Some(first) = owner.insert(member, index) {
                return Err(FoldError::Overlap {
                    instance: member,
                    first,
                    second: index,
                });
            }
        }
    }
    Ok(())
}

/// What a scene's folds hide, derived from the facts alone.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FoldEffect {
    /// Each hidden instance and the index of the fold hiding it.
    hidden: HashMap<InstanceId, u32>,
}

impl FoldEffect {
    /// Derive the effect of the active folds, each with its table index.
    pub fn of<'a>(folds: impl IntoIterator<Item = (u32, &'a Fold)>) -> Self {
        let mut hidden = HashMap::new();
        for (index, fold) in folds {
            for member in fold.hidden() {
                hidden.entry(member).or_insert(index);
            }
        }
        Self { hidden }
    }

    /// Whether a fold hides `instance`.
    pub fn is_hidden(&self, instance: InstanceId) -> bool {
        self.hidden.contains_key(&instance)
    }

    /// The index of the fold hiding `instance`.
    pub fn folded_by(&self, instance: InstanceId) -> Option<u32> {
        self.hidden.get(&instance).copied()
    }

    /// How many instances the folds hide in all.
    pub fn hidden_count(&self) -> usize {
        self.hidden.len()
    }

    /// Whether an item is drawn: its own flag, and no fold hiding it.
    pub fn is_shown(&self, instance: InstanceId, visible: bool) -> bool {
        visible && !self.is_hidden(instance)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ids(values: &[u32]) -> Vec<InstanceId> {
        values.iter().copied().map(InstanceId).collect()
    }

    fn rooted(members: &[u32]) -> Fold {
        Fold {
            members: ids(members),
            stand_in: StandIn::Member(InstanceId(members[0])),
            rule: Some(FoldRule::Descendants {
                root: InstanceId(members[0]),
                family: "depends_on".into(),
                direction: FoldDirection::Outgoing,
            }),
            boundary: None,
        }
    }

    fn live(count: u32) -> impl Fn(InstanceId) -> bool {
        move |id| id.0 < count
    }

    #[test]
    fn a_member_stand_in_stays_drawn_and_counts_the_rest() {
        let fold = rooted(&[0, 1, 2, 3]);
        assert_eq!(fold.hidden().collect::<Vec<_>>(), ids(&[1, 2, 3]));
        assert_eq!(fold.hidden_count(), 3, "the +N");
        let effect = FoldEffect::of([(0, &fold)]);
        assert!(
            effect.is_shown(InstanceId(0), true),
            "never hides its stand-in"
        );
        assert!(!effect.is_shown(InstanceId(2), true));
        assert_eq!(effect.folded_by(InstanceId(2)), Some(0));
        assert!(
            !effect.is_shown(InstanceId(9), false),
            "own flag still counts"
        );
    }

    #[test]
    fn a_summary_stand_in_hides_every_member() {
        let fold = Fold {
            members: ids(&[4, 5]),
            stand_in: StandIn::Summary {
                label: Some("two".into()),
            },
            rule: Some(FoldRule::Selection),
            boundary: None,
        };
        assert_eq!(fold.hidden_count(), 2);
        assert!(fold.validate(live(6)).is_ok());
    }

    #[test]
    fn invalid_folds_are_refused() {
        assert_eq!(
            rooted(&[0]).validate(live(4)),
            Err(FoldError::TooFewMembers { count: 1 })
        );
        assert_eq!(
            rooted(&[0, 1, 1]).validate(live(4)),
            Err(FoldError::DuplicateMember(InstanceId(1)))
        );
        assert_eq!(
            rooted(&[0, 7]).validate(live(4)),
            Err(FoldError::AbsentMember(InstanceId(7)))
        );
        let mut stray = rooted(&[0, 1]);
        stray.stand_in = StandIn::Member(InstanceId(2));
        assert_eq!(
            stray.validate(live(4)),
            Err(FoldError::StandInNotMember(InstanceId(2)))
        );
        let mut rootless = rooted(&[0, 1]);
        rootless.rule = Some(FoldRule::Descendants {
            root: InstanceId(3),
            family: "depends_on".into(),
            direction: FoldDirection::Incoming,
        });
        assert_eq!(
            rootless.validate(live(4)),
            Err(FoldError::RootNotMember(InstanceId(3)))
        );
        let mut inside = rooted(&[0, 1]);
        inside.boundary = Some(FoldBoundary {
            internal_relations: 1,
            bundles: vec![BoundaryBundle {
                outside: InstanceId(1),
                direction: FoldDirection::Outgoing,
                family: None,
                count: 1,
            }],
        });
        assert_eq!(
            inside.validate(live(4)),
            Err(FoldError::BoundaryInside(InstanceId(1)))
        );
    }

    #[test]
    fn two_folds_never_share_an_instance() {
        let a = rooted(&[0, 1]);
        let b = rooted(&[2, 1]);
        assert_eq!(
            validate_folds([(0, &a), (3, &b)], live(4)),
            Err(FoldError::Overlap {
                instance: InstanceId(1),
                first: 0,
                second: 3
            })
        );
        let c = rooted(&[2, 3]);
        assert!(validate_folds([(0, &a), (1, &c)], live(4)).is_ok());
    }

    #[test]
    fn optional_detail_stays_off_the_wire_when_absent() {
        let bare = Fold {
            members: ids(&[0, 1]),
            stand_in: StandIn::Summary { label: None },
            rule: None,
            boundary: None,
        };
        let wire = serde_json::to_string(&bare).unwrap();
        assert_eq!(
            wire, r#"{"members":[0,1],"stand_in":{"Summary":{}}}"#,
            "absent rule, boundary and label are not written"
        );
        assert_eq!(serde_json::from_str::<Fold>(&wire).unwrap(), bare);
        let full = rooted(&[0, 1]);
        let wire = serde_json::to_string(&full).unwrap();
        assert_eq!(serde_json::from_str::<Fold>(&wire).unwrap(), full);
    }
}
