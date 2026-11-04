// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

/// What a view has selected, as data rather than host state.
///
/// Selection was host-only state until a scene could be shared: mer3ly puts a
/// selected id on its shared wire, so opening someone else's link restores what
/// they had selected on a second device. That is the forcing consumer, and it
/// forces exactly this much.
///
/// `source` names the view that produced the selection. It is required even
/// with one view, because the entire point of the record is that a second view
/// can tell whose selection it is reading, and a field added later would be
/// absent on every link already in circulation.
///
/// A single [`Selection`] remains the backwards-compatible one-view record.
/// When several views contribute clauses, [`CoordinatedSelection`] owns the
/// declared rule that combines them. Keeping the wrapper separate means links
/// already carrying this exact shape continue to decode without an inferred
/// multi-view policy.
///
/// This is a noun, not an intent. Freeze ruling D1 stands: the protocol owns
/// the intent triple, and selecting something is not invoking anything.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Selection {
    /// The view that produced this selection.
    pub source: String,
    /// What is selected. Empty means a cleared selection, which is a fact worth
    /// carrying rather than an absence to infer.
    pub targets: Vec<SelectionTarget>,
}

/// One selected thing, in the shape a shipped consumer already uses.
///
/// Kind and id are opaque strings on purpose. The protocol does not learn a
/// product's taxonomy, the same way a score carries opaque [`sceno::SourceRef`]s
/// rather than source truth.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SelectionTarget {
    /// The product's word for what this is: `node`, `edge`, a region name.
    pub kind: String,
    pub id: String,
}

/// What work one view's selection clause performs.
///
/// The role is carried because focus, filtering, and a brush are not
/// interchangeable host gestures even when they happen to name the same
/// source targets.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SelectionRole {
    Focus,
    Filter,
    Brush,
}

/// The combination rule a coordinated set of views has actually forced.
///
/// `Single` preserves the meaning of the pre-coordination record.
/// `Crossfilter` is the first multi-view rule: each consuming view applies
/// every foreign clause and excludes its own. Union and intersection remain
/// absent until a consumer needs their distinct behavior.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SelectionResolution {
    Single,
    Crossfilter,
}

/// One source view's named contribution to coordinated selection.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SelectionClause {
    pub role: SelectionRole,
    pub targets: Vec<SelectionTarget>,
}

/// Selections contributed by several views, with their combination rule as
/// data rather than host convention.
///
/// The ordered map makes the serialized clause order deterministic and makes
/// the producing view an unavoidable part of every clause. Removing the last
/// foreign clause makes [`Self::targets_for`] return `None`, the explicit
/// unfiltered state.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoordinatedSelection {
    pub resolution: SelectionResolution,
    pub clauses: BTreeMap<String, SelectionClause>,
}

impl Selection {
    /// A selection of one thing, the case every shipping consumer has today.
    pub fn one(source: impl Into<String>, kind: impl Into<String>, id: impl Into<String>) -> Self {
        Self {
            source: source.into(),
            targets: vec![SelectionTarget {
                kind: kind.into(),
                id: id.into(),
            }],
        }
    }

    /// A cleared selection, distinct from never having selected anything.
    pub fn cleared(source: impl Into<String>) -> Self {
        Self {
            source: source.into(),
            targets: Vec::new(),
        }
    }

    pub fn is_cleared(&self) -> bool {
        self.targets.is_empty()
    }

    /// True when `other` came from a different view.
    ///
    /// The one question a resolution strategy will need answered when it
    /// arrives, and the reason `source` is mandatory now: crossfilter is
    /// defined as "every brush but this view's own", which is unanswerable
    /// without it.
    pub fn is_foreign_to(&self, other: &Selection) -> bool {
        self.source != other.source
    }
}

impl CoordinatedSelection {
    pub fn new(resolution: SelectionResolution) -> Self {
        Self {
            resolution,
            clauses: BTreeMap::new(),
        }
    }

    /// Insert or replace the clause produced by `selection.source`.
    pub fn set(&mut self, role: SelectionRole, selection: Selection) {
        self.clauses.insert(
            selection.source,
            SelectionClause {
                role,
                targets: selection.targets,
            },
        );
    }

    /// Remove one producing view's clause. Returns whether one existed.
    pub fn remove(&mut self, source: &str) -> bool {
        self.clauses.remove(source).is_some()
    }

    /// Resolve the targets that should constrain `consumer`.
    ///
    /// `None` means unfiltered. An empty set means the declared clauses were
    /// applied and matched nothing. Crossfilter intersects foreign clauses:
    /// every foreign brush constrains the view, while the view's own clause is
    /// deliberately excluded.
    pub fn targets_for(&self, consumer: &str) -> Option<BTreeSet<SelectionTarget>> {
        match self.resolution {
            SelectionResolution::Single => self
                .clauses
                .values()
                .next()
                .map(|clause| clause.targets.iter().cloned().collect()),
            SelectionResolution::Crossfilter => {
                let mut foreign = self
                    .clauses
                    .iter()
                    .filter(|(source, _)| source.as_str() != consumer)
                    .map(|(_, clause)| clause.targets.iter().cloned().collect::<BTreeSet<_>>());
                let mut resolved = foreign.next()?;
                for targets in foreign {
                    resolved.retain(|target| targets.contains(target));
                }
                Some(resolved)
            },
        }
    }
}
