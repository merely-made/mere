// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! View-local expansion of an explicitly disclosed containment forest.
//! The caller names the parent-to-member relationship kind. Dependencies do
//! not establish containment. Bundles retain their source relationship ids;
//! this projection neither invents source assertions nor edits the input.

use crate::projection::{DisclosedRelationship, ProjectionDataset, relationship_disclosure_issues};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// View choices, independent of source membership and coordinates.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GroupViewState {
    pub expanded: BTreeSet<String>,
    pub entered: Option<String>,
}

/// A visible edge with its original witnesses, including parallel assertions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GroupedRelation {
    pub from: String,
    pub to: String,
    pub kind: String,
    pub relationship_ids: Vec<String>,
}

/// A displayed subset and exact relationship accounting for one view.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GroupProjection {
    pub visible: BTreeSet<String>,
    pub boundary: BTreeSet<String>,
    pub relations: Vec<GroupedRelation>,
    /// Dependencies inside a visible summary, rather than invented self loops.
    pub internal_relationships: BTreeMap<String, Vec<String>>,
    pub breadcrumbs: Vec<String>,
}

/// Validated forest over occurrence identities. Ambiguous parents and cycles
/// refuse instead of choosing an owner or duplicating a component.
#[derive(Clone, Debug)]
pub struct GroupHierarchy {
    kind: String,
    occurrences: BTreeSet<String>,
    parents: BTreeMap<String, String>,
    children: BTreeMap<String, BTreeSet<String>>,
    relationships: Vec<DisclosedRelationship>,
}

impl GroupHierarchy {
    pub fn new(
        dataset: &ProjectionDataset,
        relationships: &[DisclosedRelationship],
        membership_kind: &str,
    ) -> Result<Self, String> {
        if membership_kind.trim().is_empty() {
            return Err("Grouping needs an explicit membership relationship kind".into());
        }
        let issues = relationship_disclosure_issues(dataset, relationships);
        if !issues.is_empty() {
            return Err(issues
                .iter()
                .map(|i| format!("{}: {}", i.field, i.message))
                .collect::<Vec<_>>()
                .join("; "));
        }
        let occurrences = dataset
            .occurrences
            .iter()
            .map(|o| o.occurrence_id.clone())
            .collect();
        let mut parents = BTreeMap::new();
        let mut children: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        for relation in relationships.iter().filter(|r| r.kind == membership_kind) {
            if let Some(previous) = parents.insert(
                relation.to_occurrence.clone(),
                relation.from_occurrence.clone(),
            ) {
                if previous != relation.from_occurrence {
                    return Err(format!(
                        "{} has multiple grouping parents",
                        relation.to_occurrence
                    ));
                }
            }
            children
                .entry(relation.from_occurrence.clone())
                .or_default()
                .insert(relation.to_occurrence.clone());
        }
        if children.is_empty() {
            return Err(format!(
                "No membership relationships of kind {membership_kind} are disclosed"
            ));
        }
        let mut checked = BTreeSet::new();
        for start in parents.keys() {
            let mut path = BTreeSet::new();
            let mut cursor = start.as_str();
            while !checked.contains(cursor) {
                if !path.insert(cursor.to_owned()) {
                    return Err(format!("Grouping membership contains a cycle at {cursor}"));
                }
                let Some(parent) = parents.get(cursor) else {
                    break;
                };
                cursor = parent;
            }
            checked.extend(path);
        }
        Ok(Self {
            kind: membership_kind.into(),
            occurrences,
            parents,
            children,
            relationships: relationships.to_vec(),
        })
    }

    pub fn groups(&self) -> impl Iterator<Item = &str> {
        self.children.keys().map(String::as_str)
    }
    pub fn children(&self, group: &str) -> Option<&BTreeSet<String>> {
        self.children.get(group)
    }

    pub fn breadcrumbs(&self, group: &str) -> Vec<String> {
        let mut path = vec![group.to_owned()];
        let mut cursor = group;
        while let Some(parent) = self.parents.get(cursor) {
            path.push(parent.clone());
            cursor = parent;
        }
        path.reverse();
        path
    }

    /// Iterative traversal supports deep hierarchies without stack recursion.
    fn representatives(
        &self,
        roots: Vec<String>,
        expanded: &BTreeSet<String>,
    ) -> BTreeMap<String, String> {
        let mut representatives = BTreeMap::new();
        let mut pending: Vec<_> = roots.into_iter().map(|id| (id, None::<String>)).collect();
        while let Some((id, hidden_by)) = pending.pop() {
            let representative = hidden_by.clone().unwrap_or_else(|| id.clone());
            representatives.insert(id.clone(), representative.clone());
            let child_hidden_by = if hidden_by.is_none() && expanded.contains(&id) {
                None
            } else {
                Some(representative)
            };
            if let Some(children) = self.children.get(&id) {
                pending.extend(
                    children
                        .iter()
                        .cloned()
                        .map(|child| (child, child_hidden_by.clone())),
                );
            }
        }
        representatives
    }

    pub fn project(&self, state: &GroupViewState) -> Result<GroupProjection, String> {
        for group in state.expanded.iter().chain(state.entered.iter()) {
            if !self.children.contains_key(group) {
                return Err(format!("Unknown or non-group occurrence {group}"));
            }
        }
        let roots = self
            .occurrences
            .iter()
            .filter(|id| !self.parents.contains_key(*id))
            .cloned()
            .collect();
        let global = self.representatives(roots, &state.expanded);
        let (representatives, breadcrumbs) = if let Some(group) = &state.entered {
            let mut expanded = state.expanded.clone();
            expanded.insert(group.clone());
            (
                self.representatives(vec![group.clone()], &expanded),
                self.breadcrumbs(group),
            )
        } else {
            (global.clone(), Vec::new())
        };
        let mut visible: BTreeSet<_> = representatives.values().cloned().collect();
        let mut boundary = BTreeSet::new();
        let mut bundles: BTreeMap<(String, String, String), Vec<String>> = BTreeMap::new();
        let mut internal_relationships: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for relation in &self.relationships {
            let inside_from = representatives.get(&relation.from_occurrence);
            let inside_to = representatives.get(&relation.to_occurrence);
            if inside_from.is_none() && inside_to.is_none() {
                continue;
            }
            // Membership is shown only between visible original endpoints.
            if relation.kind == self.kind
                && (inside_from != Some(&relation.from_occurrence)
                    || inside_to != Some(&relation.to_occurrence))
            {
                continue;
            }
            let from = inside_from
                .unwrap_or(&global[&relation.from_occurrence])
                .clone();
            let to = inside_to
                .unwrap_or(&global[&relation.to_occurrence])
                .clone();
            if from == to {
                internal_relationships
                    .entry(from)
                    .or_default()
                    .push(relation.id.clone());
                continue;
            }
            for (inside, representative) in [(inside_from, &from), (inside_to, &to)] {
                if inside.is_none() {
                    boundary.insert(representative.clone());
                    visible.insert(representative.clone());
                }
            }
            bundles
                .entry((from, to, relation.kind.clone()))
                .or_default()
                .push(relation.id.clone());
        }
        for ids in bundles
            .values_mut()
            .chain(internal_relationships.values_mut())
        {
            ids.sort();
        }
        let relations = bundles
            .into_iter()
            .map(|((from, to, kind), relationship_ids)| GroupedRelation {
                from,
                to,
                kind,
                relationship_ids,
            })
            .collect();
        Ok(GroupProjection {
            visible,
            boundary,
            relations,
            internal_relationships,
            breadcrumbs,
        })
    }
}

#[cfg(test)]
#[path = "grouping_tests.rs"]
mod tests;
