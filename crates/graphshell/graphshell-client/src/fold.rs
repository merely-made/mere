// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Reading a scene's folds the way every Graphshell reader does.
//!
//! A fold is a scene fact (`sceno::Fold`), and the fact alone hides its
//! members: no item's `visible` changes when a host folds. The remote reader
//! and the frozen reader both read it here, so they agree on which members are
//! hidden, what stands in for them, the "+N" the stand-in carries, and how the
//! rule reads as text (site canvas plan, Rulings 149 to 151). A host's own
//! label, when the fold carries one, is read in preference to that generic
//! wording (Ruling 155).
//!
//! Reversing a fold is not a client edit. Like every other change to a remote
//! scene it arrives as a diff from the endpoint, here a
//! `SceneOp::TombstoneFold`, after the host's own authority accepted the
//! person's intent. The client never rewrites the fact on its own.

use sceno::{Fold, FoldBoundary, FoldDirection, FoldRule, InstanceId, StandIn, Vec2};
use scenotime::{FoldId, SceneSnapshot, SceneTables};

/// One active fold, as a reader presents it.
#[derive(Clone, Debug, PartialEq)]
pub struct FoldReading {
    pub fold: FoldId,
    /// The member drawn in the fold's place, or `None` for a summary.
    pub stand_in: Option<InstanceId>,
    /// A summary stand-in's own label, when the host gave one.
    pub summary_label: Option<String>,
    /// Every member, in the host's order.
    pub members: Vec<InstanceId>,
    /// The members the fold hides: every member but a member stand-in.
    pub hidden: Vec<InstanceId>,
    pub rule: Option<FoldRule>,
    /// The host's own words for the fold, preferred to the rule's wording.
    pub label: Option<String>,
    pub boundary: Option<FoldBoundary>,
    /// Where a summary stand-in is drawn: the centroid of its members' world
    /// positions, which folding never moves. `None` for a member stand-in.
    pub summary_at: Option<Vec2>,
}

impl FoldReading {
    /// Read one fold fact. `summary_at` stays `None`: placing a summary needs
    /// the scene's geometry, which [`read_folds`] supplies.
    pub fn new(fold: FoldId, value: &Fold) -> Self {
        Self {
            fold,
            stand_in: value.stand_in_member(),
            summary_label: match &value.stand_in {
                StandIn::Summary { label } => label.clone(),
                StandIn::Member(_) => None,
            },
            members: value.members.clone(),
            hidden: value.hidden().collect(),
            rule: value.rule.clone(),
            label: value.label.clone(),
            boundary: value.boundary.clone(),
            summary_at: None,
        }
    }

    fn of(tables: &SceneTables, fold: FoldId, value: &Fold) -> Self {
        let mut reading = Self::new(fold, value);
        if reading.stand_in.is_none() {
            reading.summary_at = centroid(tables, &value.members);
        }
        reading
    }

    /// The count the stand-in carries, as drawn: `+N`.
    pub fn badge(&self) -> String {
        format!("+{}", self.hidden.len())
    }

    /// What the stand-in is called: the member's name, the summary's label,
    /// or a count when the host gave a summary no label.
    pub fn stand_in_name(&self, name_of: impl Fn(InstanceId) -> String) -> String {
        match (self.stand_in, &self.summary_label) {
            (Some(member), _) => name_of(member),
            (None, Some(label)) => label.clone(),
            (None, None) => format!("{} folded items", self.members.len()),
        }
    }

    /// What a reader is told the fold is: the host's label when it gave one
    /// (Ruling 155), else the rule as a sentence.
    pub fn description(&self, name_of: impl Fn(InstanceId) -> String) -> Option<String> {
        match &self.label {
            Some(label) => Some(label.clone()),
            None => self.rule_text(name_of),
        }
    }

    /// The rule as a sentence, when the fold records one.
    pub fn rule_text(&self, name_of: impl Fn(InstanceId) -> String) -> Option<String> {
        Some(match self.rule.as_ref()? {
            FoldRule::Selection => "chosen by selection".to_owned(),
            FoldRule::Descendants {
                root,
                family,
                direction,
            } => {
                let family = family.replace(['_', '-'], " ");
                match direction {
                    FoldDirection::Outgoing => {
                        format!("{} and everything it reaches by {family}", name_of(*root))
                    },
                    FoldDirection::Incoming => format!(
                        "{} and everything that reaches it by {family}",
                        name_of(*root)
                    ),
                }
            },
        })
    }
}

/// Every active fold of a snapshot, in table order.
pub fn read_folds(snapshot: &SceneSnapshot) -> Vec<FoldReading> {
    snapshot
        .active_folds()
        .into_iter()
        .map(|(id, fold)| FoldReading::of(&snapshot.tables, id, fold))
        .collect()
}

fn centroid(tables: &SceneTables, members: &[InstanceId]) -> Option<Vec2> {
    let mut sum = Vec2::ZERO;
    for member in members {
        let item = tables.items.get(member.0 as usize)?.as_ref()?;
        let at = tables
            .space_to_world(item.space)?
            .apply(item.transform.translate);
        sum = Vec2::new(sum.x + at.x, sum.y + at.y);
    }
    let count = members.len() as f32;
    (count > 0.0).then(|| Vec2::new(sum.x / count, sum.y / count))
}
