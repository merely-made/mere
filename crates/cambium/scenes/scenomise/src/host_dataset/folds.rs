// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Host membership adapted into portable scene folds (site Ruling 161).
//!
//! Visibility is derived only from `Scene::fold_effect`. Nested source
//! membership produces a non-overlapping frontier of active `sceno::Fold`
//! facts, never nested hiding rules. Entering filters focus and retains
//! dependency context; it leaves source instances and placement intact.
//! The caller names the parent-to-member relationship kind. Dependencies do
//! not establish containment. Bundles retain their source relationship ids;
//! this projection neither invents source assertions nor edits the input.

use crate::projection::{
    DisclosedRelationship, ProjectionDataset, ProjectionValue, relationship_disclosure_issues,
};
use sceno::{
    BoundaryBundle, Fold, FoldBoundary, FoldDirection, FoldRule, InstanceId, Scene, StandIn,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// View choices, independent of source membership and coordinates.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FoldViewState {
    pub expanded: BTreeSet<String>,
    pub entered: Option<String>,
}

/// A visible edge with its original witnesses, including parallel assertions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FoldedRelation {
    pub from: String,
    pub to: String,
    pub kind: String,
    pub relationship_ids: Vec<String>,
}

/// A folded scene and exact relationship accounting for one view.
#[derive(Clone, Debug, PartialEq)]
pub struct FoldProjection {
    /// Complete source placement with portable Fold facts; no items are removed.
    pub scene: Scene,
    pub visible: BTreeSet<String>,
    pub boundary: BTreeSet<String>,
    pub relations: Vec<FoldedRelation>,
    /// Dependencies inside a visible summary, rather than invented self loops.
    pub internal_relationships: BTreeMap<String, Vec<String>>,
    pub breadcrumbs: Vec<String>,
}

/// Validated forest over occurrence identities. Ambiguous parents and cycles
/// refuse instead of choosing an owner or duplicating a component.
#[derive(Clone, Debug)]
pub struct HostDatasetFolds {
    kind: String,
    occurrences: BTreeSet<String>,
    parents: BTreeMap<String, String>,
    children: BTreeMap<String, BTreeSet<String>>,
    relationships: Vec<DisclosedRelationship>,
    scene: Scene,
    instances: BTreeMap<String, InstanceId>,
    labels: BTreeMap<String, String>,
}

impl HostDatasetFolds {
    pub fn new(
        dataset: &ProjectionDataset,
        relationships: &[DisclosedRelationship],
        membership_kind: &str,
        scene: Scene,
        instances: BTreeMap<String, InstanceId>,
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
        let occurrences: BTreeSet<String> = dataset
            .occurrences
            .iter()
            .map(|o| o.occurrence_id.clone())
            .collect();
        if !scene.folds.is_empty() {
            return Err("Host membership cannot replace existing scene folds".into());
        }
        let live: BTreeSet<_> = instances.values().map(|id| id.0).collect();
        if instances.keys().cloned().collect::<BTreeSet<_>>() != occurrences
            || live.len() != instances.len()
            || live.len() != scene.items.len()
            || live.iter().any(|id| *id as usize >= scene.items.len())
        {
            return Err("Fold membership needs a complete occurrence-to-instance mapping".into());
        }
        if dataset.occurrences.iter().any(|occurrence| {
            let item = &scene.items[instances[&occurrence.occurrence_id].0 as usize];
            scene.sources.get(item.source.0 as usize) != Some(&occurrence.source)
        }) {
            return Err("Fold instance mapping disagrees with its source identity".into());
        }
        let labels = dataset
            .occurrences
            .iter()
            .map(|item| {
                let label = match item.values.get("label") {
                    Some(ProjectionValue::Text(label)) if !label.trim().is_empty() => label.clone(),
                    _ => item.occurrence_id.clone(),
                };
                (item.occurrence_id.clone(), label)
            })
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
            scene,
            instances,
            labels,
        })
    }

    pub fn instances(&self) -> &BTreeMap<String, InstanceId> {
        &self.instances
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

    /// Descendants are source membership, never a second visibility mechanism.
    fn descendants(&self, root: &str) -> BTreeSet<String> {
        let mut members = BTreeSet::new();
        let mut pending = vec![root.to_owned()];
        while let Some(id) = pending.pop() {
            if let Some(children) = self.children.get(&id) {
                pending.extend(children.iter().cloned());
            }
            members.insert(id);
        }
        members
    }

    /// Closed ancestors fold the union of their descendants. Their inner folds
    /// disappear until the ancestor opens, keeping active facts disjoint while
    /// remembering the reader's choices at every level.
    pub fn scene(&self, state: &FoldViewState) -> Result<Scene, String> {
        for group in state.expanded.iter().chain(state.entered.iter()) {
            if !self.children.contains_key(group) {
                return Err(format!("Unknown or non-group occurrence {group}"));
            }
        }
        let mut expanded = state.expanded.clone();
        if let Some(entered) = &state.entered {
            expanded.extend(self.breadcrumbs(entered));
        }
        let mut scene = self.scene.clone();
        let mut pending: Vec<_> = self
            .occurrences
            .iter()
            .filter(|id| !self.parents.contains_key(*id))
            .cloned()
            .collect();
        while let Some(root) = pending.pop() {
            let Some(children) = self.children.get(&root) else {
                continue;
            };
            if expanded.contains(&root) {
                pending.extend(children.iter().cloned());
                continue;
            }
            let members = self.descendants(&root);
            let mut bundles: BTreeMap<(u32, FoldDirection, String), u32> = BTreeMap::new();
            let mut internal_relations = 0;
            for relation in &self.relationships {
                match (
                    members.contains(&relation.from_occurrence),
                    members.contains(&relation.to_occurrence),
                ) {
                    (true, true) => internal_relations += 1,
                    (true, false) => {
                        *bundles
                            .entry((
                                self.instances[&relation.to_occurrence].0,
                                FoldDirection::Outgoing,
                                relation.kind.clone(),
                            ))
                            .or_default() += 1
                    },
                    (false, true) => {
                        *bundles
                            .entry((
                                self.instances[&relation.from_occurrence].0,
                                FoldDirection::Incoming,
                                relation.kind.clone(),
                            ))
                            .or_default() += 1
                    },
                    _ => {},
                }
            }
            scene.folds.push(Fold {
                members: members.iter().map(|id| self.instances[id]).collect(),
                stand_in: StandIn::Member(self.instances[&root]),
                rule: Some(FoldRule::Descendants {
                    root: self.instances[&root],
                    family: self.kind.clone(),
                    direction: FoldDirection::Outgoing,
                }),
                label: Some(self.labels[&root].clone()),
                boundary: Some(FoldBoundary {
                    internal_relations,
                    bundles: bundles
                        .into_iter()
                        .map(|((outside, direction, family), count)| BoundaryBundle {
                            outside: InstanceId(outside),
                            direction,
                            family: Some(family),
                            count,
                        })
                        .collect(),
                }),
            });
        }
        scene.validate_folds().map_err(|error| error.to_string())?;
        Ok(scene)
    }

    pub fn project(&self, state: &FoldViewState) -> Result<FoldProjection, String> {
        let scene = self.scene(state)?;
        let effect = scene.fold_effect();
        let occurrence_by_instance: BTreeMap<_, _> = self
            .instances
            .iter()
            .map(|(id, instance)| (instance.0, id))
            .collect();
        let global: BTreeMap<_, _> = self
            .instances
            .iter()
            .map(|(id, instance)| {
                let representative = effect
                    .folded_by(*instance)
                    .map(|fold| {
                        let stand_in = scene.folds[fold as usize]
                            .stand_in_member()
                            .expect("host membership folds use a member stand-in");
                        occurrence_by_instance[&stand_in.0].clone()
                    })
                    .unwrap_or_else(|| id.clone());
                (id.clone(), representative)
            })
            .collect();
        let (representatives, breadcrumbs) = if let Some(group) = &state.entered {
            let members = self.descendants(group);
            (
                global
                    .iter()
                    .filter(|(id, _)| members.contains(*id))
                    .map(|(id, representative)| (id.clone(), representative.clone()))
                    .collect(),
                self.breadcrumbs(group),
            )
        } else {
            (global.clone(), Vec::new())
        };
        let shown = |id: &String| {
            let instance = self.instances[id];
            effect.is_shown(instance, scene.items[instance.0 as usize].visible)
        };
        let mut visible: BTreeSet<_> = representatives
            .values()
            .filter(|id| shown(id))
            .cloned()
            .collect();
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
            if !shown(&from) || !shown(&to) {
                continue;
            }
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
            .map(|((from, to, kind), relationship_ids)| FoldedRelation {
                from,
                to,
                kind,
                relationship_ids,
            })
            .collect();
        Ok(FoldProjection {
            scene,
            visible,
            boundary,
            relations,
            internal_relationships,
            breadcrumbs,
        })
    }
}

#[cfg(test)]
#[path = "fold_tests.rs"]
mod tests;
