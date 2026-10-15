// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Merging a branch or a picked-off part back into a session (session
//! merge-in note, `design_docs/mere_docs/technical_architecture/2026-10-15_session_merge_in_note.md`,
//! rulings M1–M8).
//!
//! [`MereSessions::merge_in`] plans: it matches the source's nodes to the
//! target's, finds the base both started from, and compares base, source and
//! target part by part. What changed only in the source is taken, what changed
//! only in the target is kept, and a part both changed differently is a
//! conflict, unless the part's merge rule ([`crate::merge_rules`], shared with
//! codicil Compose) can join the two. [`MereSessions::apply_merge`] then writes
//! the plan, with a choice for each conflict, as one `MergedIn` change on the
//! target, and marks the source merged.
//!
//! Matching:
//! - a **branch** keeps its parent's node ids, so a node matches itself. Its
//!   base is its own baseline, the parent's graph where it branched. A branch
//!   of a branch already merged into the target matches the same way.
//! - a **picked-off part** matches through each node's `CopiedFrom`
//!   derivation into the target's container, walking a pick of a pick back
//!   through every part between (M3). Its base is the target's graph when the
//!   chain was first picked from it, and the part's own edits are replayed onto
//!   that base, so what a copy changed in passing (visit stamps, derivations,
//!   import provenance) never reads as an edit. A node the target never held is
//!   added under a new id.
//!
//! The comparison is undo's own (`kernel::graph::revert_change`): reverting
//! "base to source" against the target is exactly taking each source change
//! the target left alone. Deletions merge the same way (M7).

use std::collections::{BTreeMap, BTreeSet};

use kernel::graph::{
    Part, ProvenanceSubKind, part_node, part_of, part_value, recreate_node_edits,
    replay_captured_deltas_onto,
};
use kernel::persistence::PersistedResourceFacet;
use kernel::types::NodeDerivation;
use serde_json::Value;

use super::*;
use crate::manifest::MergedMark;
use crate::merge_rules::{FacetMerge, merge_resource_facet, resource_facet_has_rule};

/// How far a chain of picks or merged branches is followed before merge-in
/// gives up on it.
const MAX_LINEAGE_HOPS: usize = 64;

/// Which side a conflict takes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MergeSide {
    /// What the branch or picked-off part holds.
    Source,
    /// What the session being merged into holds.
    Target,
}

/// One part both sides changed differently since the base. Values are the
/// part as each graph holds it (`None`: absent), in the target's ids.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MergeConflict {
    pub part: Part,
    pub base: Option<Value>,
    pub source: Option<Value>,
    pub target: Option<Value>,
}

/// How a merge matched the source's nodes to the target's.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MergeMatch {
    /// By id: the source is a branch.
    Branch,
    /// By derivation provenance: the source is a picked-off part.
    Pick,
}

/// A planned merge-in: the edits that apply cleanly and the conflicts that
/// wait for a choice. Nothing is written until
/// [`MereSessions::apply_merge`].
#[derive(Clone)]
pub struct PendingMerge {
    target: SessionId,
    source: SessionId,
    matched: MergeMatch,
    through: Seq,
    planned_at: Seq,
    conflicts: Vec<MergeConflict>,
    /// The clean edits, rule merges included, in the target's ids.
    edits: Vec<CapturedDelta>,
    /// The source's edits since the base, in the target's ids.
    change: Vec<CapturedDelta>,
    /// The source as merge-in reads it, in the target's ids.
    source_graph: Graph,
    /// Each target id the source names, with the source's own id for it.
    origins: BTreeMap<Uuid, Uuid>,
    source_container: Uuid,
}

impl std::fmt::Debug for PendingMerge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PendingMerge")
            .field("target", &self.target)
            .field("source", &self.source)
            .field("matched", &self.matched)
            .field("through", &self.through)
            .field("conflicts", &self.conflicts)
            .field("edits", &self.edits.len())
            .finish_non_exhaustive()
    }
}

impl PendingMerge {
    pub fn target(&self) -> SessionId {
        self.target
    }

    pub fn source(&self) -> SessionId {
        self.source
    }

    pub fn matched(&self) -> MergeMatch {
        self.matched
    }

    /// The source's journal cursor this merge brings in.
    pub fn through(&self) -> Seq {
        self.through
    }

    /// Whether the merge applies with no choices to make.
    pub fn is_clean(&self) -> bool {
        self.conflicts.is_empty()
    }

    pub fn conflicts(&self) -> &[MergeConflict] {
        &self.conflicts
    }

    /// The edits the merge writes before any conflict's choice, in the
    /// target's ids.
    pub fn edits(&self) -> &[CapturedDelta] {
        &self.edits
    }
}

/// One step of a pick chain: the session picked from, its journal cursor then,
/// and the container its derivations name.
#[derive(Clone, Copy, Debug)]
struct Hop {
    session: SessionId,
    at: Seq,
    container: Uuid,
    /// A part picked before `picked_from` existed, read from its fork fields.
    legacy: bool,
}

/// What merge-in compares, once the source is matched to the target.
struct Basis {
    matched: MergeMatch,
    base: Graph,
    source_graph: Graph,
    change: Vec<CapturedDelta>,
    /// Source id to target id, for every source node.
    ids: BTreeMap<Uuid, Uuid>,
}

/// Rewrite every id `ids` names, wherever it appears in `edit` (node ids,
/// edge endpoints, import memberships, facet-store keys).
fn remap_edit(edit: &CapturedDelta, ids: &BTreeMap<String, String>) -> CapturedDelta {
    fn walk(value: &mut Value, ids: &BTreeMap<String, String>) {
        match value {
            Value::String(text) => {
                if let Some(mapped) = ids.get(text.as_str()) {
                    *text = mapped.clone();
                }
            },
            Value::Array(items) => items.iter_mut().for_each(|item| walk(item, ids)),
            Value::Object(map) => {
                let entries = std::mem::take(map);
                for (key, mut item) in entries {
                    walk(&mut item, ids);
                    map.insert(ids.get(&key).cloned().unwrap_or(key), item);
                }
            },
            _ => {},
        }
    }
    let mut value = serde_json::to_value(edit).expect("a captured edit encodes");
    walk(&mut value, ids);
    serde_json::from_value(value).expect("a remapped edit decodes")
}

/// The `CopiedFrom` derivation of node `id` into `container`, looked up in
/// each graph in turn.
fn copied_from(graphs: &[&Graph], id: Uuid, container: Uuid) -> Option<Uuid> {
    let container = container.to_string();
    graphs.iter().find_map(|graph| {
        let (key, _) = graph.get_node_by_id(id)?;
        graph
            .node_derivations(key)?
            .into_iter()
            .find(|derivation| {
                derivation.sub_kind == ProvenanceSubKind::CopiedFrom
                    && derivation.source_graph.as_deref() == Some(container.as_str())
            })
            .and_then(|derivation| Uuid::parse_str(&derivation.source_node).ok())
    })
}

fn node_ids(graph: &Graph) -> BTreeSet<Uuid> {
    graph.nodes().map(|(_, node)| node.id).collect()
}

impl<B: Backend + Clone> MereSessions<B> {
    async fn manifest_of(&self, id: SessionId) -> Result<GraphSessionManifest, SessionError> {
        read(&self.slots(), &Keys::new(id).at(MANIFEST))
            .await?
            .ok_or(SessionError::Missing(id))
    }

    /// Plan merging `source`, a branch or a picked-off part, into `target`.
    /// The plan is clean, or lists its conflicts; nothing is written.
    pub async fn merge_in(
        &self,
        target: &GraphSession<B>,
        source: &GraphSession<B>,
    ) -> Result<PendingMerge, SessionError> {
        if target.id() == source.id() {
            return Err(SessionError::NotMergeable(
                "a session cannot merge into itself".into(),
            ));
        }
        if let Some(mark) = &source.manifest().merged {
            return Err(SessionError::NotMergeable(format!(
                "already merged into {}",
                mark.into.as_uuid()
            )));
        }
        target.ensure_writable()?;
        let basis = if source.manifest().lineage == Lineage::Branch {
            self.branch_basis(target, source).await?
        } else {
            self.pick_basis(target, source).await?
        };
        let origins = basis
            .ids
            .iter()
            .map(|(source_id, target_id)| (*target_id, *source_id))
            .collect();
        let live = target.graph();
        let compared = revert_change(&basis.change, &basis.source_graph, &basis.base, live);
        let mut edits = compared.edits;
        let mut conflicts = Vec::new();
        let mut joined: BTreeMap<Uuid, Vec<(String, Value)>> = BTreeMap::new();
        let kept: BTreeSet<Part> = compared.kept.into_iter().collect();
        for part in kept {
            let (theirs, ours) = (
                part_value(&basis.source_graph, &part),
                part_value(live, &part),
            );
            if theirs == ours {
                continue;
            }
            if let (Part::ResourceFacet(resource, facet), Some(theirs), Some(ours)) =
                (&part, &theirs, &ours)
                && resource_facet_has_rule(facet)
                && let Some(record) = live.resource_record(*resource)
                && let Ok(FacetMerge::Merged(value)) =
                    merge_resource_facet(&record.canonical_iri, facet, ours, theirs)
            {
                joined
                    .entry(*resource)
                    .or_default()
                    .push((facet.clone(), value));
                continue;
            }
            conflicts.push(MergeConflict {
                base: part_value(&basis.base, &part),
                source: theirs,
                target: ours,
                part,
            });
        }
        if !joined.is_empty() {
            let mut scratch = live.clone();
            replay_captured_deltas_onto(&mut scratch, edits.iter().cloned());
            for (resource, facets) in joined {
                edits.extend(record_with(
                    &scratch,
                    resource,
                    facets
                        .into_iter()
                        .map(|(facet, value)| (facet, Some(value))),
                ));
            }
        }
        Ok(PendingMerge {
            target: target.id(),
            source: source.id(),
            matched: basis.matched,
            through: source.journal().live_cursor(),
            planned_at: target.journal().live_cursor(),
            conflicts,
            edits,
            change: basis.change,
            source_graph: basis.source_graph,
            origins,
            source_container: *source.manifest().root_graph_id.as_uuid(),
        })
    }

    /// A branch's basis: ids match themselves, and the base is the branch's
    /// own baseline. The branch's parent must be the target, or a branch
    /// already merged into it (an edit to a merged branch makes a new branch
    /// from it, M6).
    async fn branch_basis(
        &self,
        target: &GraphSession<B>,
        source: &GraphSession<B>,
    ) -> Result<Basis, SessionError> {
        let mut parent = source
            .manifest()
            .parent_session
            .ok_or_else(|| SessionError::NotMergeable("a branch with no parent session".into()))?;
        let mut hops = 0;
        while parent != target.id() {
            hops += 1;
            let into = self
                .manifest_of(parent)
                .await?
                .merged
                .map(|mark| mark.into)
                .filter(|_| hops < MAX_LINEAGE_HOPS);
            parent = into.ok_or_else(|| {
                SessionError::NotMergeable(format!(
                    "{} is not a branch of {}",
                    source.id().as_uuid(),
                    target.id().as_uuid()
                ))
            })?;
        }
        let base = source
            .graph_at(Seq(0))
            .ok_or_else(|| SessionError::Corrupt("a branch's baseline does not load".into()))?;
        let source_graph = source.graph().clone();
        let live = source.journal().live_cursor();
        let change = source.edits_in(Seq(0), live);
        let ids = node_ids(&base)
            .union(&node_ids(&source_graph))
            .map(|id| (*id, *id))
            .collect();
        Ok(Basis {
            matched: MergeMatch::Branch,
            base,
            source_graph,
            change,
            ids,
        })
    }

    /// The step a part was picked by: its `picked_from`, or, for a part
    /// picked before that record, its fork fields.
    async fn pick_hop(
        &self,
        manifest: &GraphSessionManifest,
        target: &GraphSession<B>,
    ) -> Result<Option<Hop>, SessionError> {
        if let Some(picked) = &manifest.picked_from {
            return Ok(Some(Hop {
                session: picked.session,
                at: Seq(picked.at),
                container: picked.container,
                legacy: false,
            }));
        }
        let (Some(parent), Some(at)) = (manifest.parent_session, manifest.forked_at) else {
            return Ok(None);
        };
        let container = if parent == target.id() {
            *target.manifest().root_graph_id.as_uuid()
        } else {
            *self.manifest_of(parent).await?.root_graph_id.as_uuid()
        };
        Ok(Some(Hop {
            session: parent,
            at: Seq(at),
            container,
            legacy: true,
        }))
    }

    /// A picked-off part's basis: ids match through `CopiedFrom` derivations,
    /// hop by hop to the target; the base is the target where the chain was
    /// picked; the source is that base with the part's own edits replayed.
    async fn pick_basis(
        &self,
        target: &GraphSession<B>,
        source: &GraphSession<B>,
    ) -> Result<Basis, SessionError> {
        let not_a_pick = || {
            SessionError::NotMergeable(format!(
                "{} is neither a branch nor a part picked from {}",
                source.id().as_uuid(),
                target.id().as_uuid()
            ))
        };
        let baseline = source
            .graph_at(Seq(0))
            .ok_or_else(|| SessionError::Corrupt("a part's baseline does not load".into()))?;
        // Each source node's id at the current hop.
        let mut current: BTreeMap<Uuid, Uuid> = node_ids(&baseline)
            .union(&node_ids(source.graph()))
            .map(|id| (*id, *id))
            .collect();
        let mut manifest = source.manifest().clone();
        let mut level: Vec<Graph> = vec![baseline.clone(), source.graph().clone()];
        let mut root_at = None;
        for hop_index in 0..MAX_LINEAGE_HOPS {
            let hop = self
                .pick_hop(&manifest, target)
                .await?
                .ok_or_else(not_a_pick)?;
            let graphs: Vec<&Graph> = level.iter().collect();
            current = current
                .into_iter()
                .filter_map(|(origin, id)| {
                    copied_from(&graphs, id, hop.container).map(|copied| (origin, copied))
                })
                .collect();
            if hop.legacy && hop_index == 0 && current.is_empty() {
                // A fork carries no derivations into its parent: not a pick.
                return Err(not_a_pick());
            }
            if hop.session == target.id() {
                root_at = Some(hop.at);
                break;
            }
            let parent = self.open(hop.session).await?;
            manifest = parent.manifest().clone();
            level = parent
                .graph_at(hop.at)
                .into_iter()
                .chain([parent.graph().clone()])
                .collect();
        }
        let root_at = root_at.ok_or_else(not_a_pick)?;
        let base = target
            .graph_at(root_at)
            .ok_or(SessionError::CursorOutOfRange {
                cursor: root_at,
                live: target.journal().live_cursor(),
            })?;
        // A node the target never held is added under a new id, stable for
        // this target and source node so a plan can be made again.
        let target_ns = *target.id().as_uuid();
        let mut ids = current;
        for id in node_ids(&baseline).union(&node_ids(source.graph())) {
            ids.entry(*id)
                .or_insert_with(|| Uuid::new_v5(&target_ns, id.as_bytes()));
        }
        let names: BTreeMap<String, String> = ids
            .iter()
            .map(|(from, to)| (from.to_string(), to.to_string()))
            .collect();
        // Baseline nodes with no match (made in a part between) arrive whole,
        // with their relations; then the part's own edits.
        let mut change = Vec::new();
        let unmatched: Vec<Uuid> = node_ids(&baseline)
            .into_iter()
            .filter(|id| base.get_node_by_id(ids[id]).is_none())
            .collect();
        for id in &unmatched {
            if let Some(made) = recreate_node_edits(&baseline, *id) {
                change.extend(made.iter().map(|edit| remap_edit(edit, &names)));
            }
        }
        let mut pairs = BTreeSet::new();
        for id in &unmatched {
            let Some((key, _)) = baseline.get_node_by_id(*id) else {
                continue;
            };
            for (other_key, other) in baseline.nodes() {
                for (from, to) in [
                    ((key, *id), (other_key, other.id)),
                    ((other_key, other.id), (key, *id)),
                ] {
                    if !pairs.insert((from.1, to.1)) {
                        continue;
                    }
                    let edges = baseline.persisted_edges_between(from.0, to.0);
                    if !edges.is_empty() {
                        change.push(remap_edit(
                            &CapturedDelta::ReplaySetEdgesByIds {
                                from_id: from.1.to_string(),
                                to_id: to.1.to_string(),
                                edges,
                            },
                            &names,
                        ));
                    }
                }
            }
        }
        let live = source.journal().live_cursor();
        let own = source.edits_in(Seq(0), live);
        change.extend(own.iter().map(|edit| remap_edit(edit, &names)));
        let mut source_graph = base.clone();
        replay_captured_deltas_onto(&mut source_graph, change.iter().cloned());
        Ok(Basis {
            matched: MergeMatch::Pick,
            base,
            source_graph,
            change,
            ids,
        })
    }

    /// Write `pending` into `target` as one `MergedIn` change, taking each
    /// conflict's side from `choices`, and mark `source` merged: kept, with its
    /// history, and read-only from then on (M6). Every merged node records a
    /// `MergedFrom` derivation naming the source node (M4). Both sessions are
    /// stored in one batch. Refused if either session changed since the plan,
    /// or a conflict has no choice.
    pub async fn apply_merge(
        &self,
        target: &mut GraphSession<B>,
        source: &mut GraphSession<B>,
        pending: PendingMerge,
        choices: &BTreeMap<Part, MergeSide>,
        author: Author,
    ) -> Result<Applied, SessionError> {
        if target.id() != pending.target || source.id() != pending.source {
            return Err(SessionError::NotMergeable(
                "the plan names other sessions".into(),
            ));
        }
        if target.journal().live_cursor() != pending.planned_at
            || source.journal().live_cursor() != pending.through
        {
            return Err(SessionError::NotMergeable(
                "a session changed since the merge was planned; plan it again".into(),
            ));
        }
        if source.manifest().merged.is_some() {
            return Err(SessionError::NotMergeable(
                "the source is already merged".into(),
            ));
        }
        let unresolved = pending
            .conflicts
            .iter()
            .filter(|conflict| !choices.contains_key(&conflict.part))
            .count();
        if unresolved > 0 {
            return Err(SessionError::Unresolved(unresolved));
        }
        let mut edits = pending.edits.clone();
        let mut scratch = target.graph().clone();
        replay_captured_deltas_onto(&mut scratch, edits.iter().cloned());
        let taken: BTreeSet<&Part> = pending
            .conflicts
            .iter()
            .map(|conflict| &conflict.part)
            .filter(|part| choices.get(*part) == Some(&MergeSide::Source))
            .collect();
        if !taken.is_empty() {
            // The edits that make each taken part read as the source does.
            let forced = revert_change(&pending.change, &pending.source_graph, &scratch, &scratch);
            let whole: BTreeSet<Uuid> = taken
                .iter()
                .filter_map(|part| match part {
                    Part::Node(id) => Some(*id),
                    _ => None,
                })
                .collect();
            let chosen: Vec<CapturedDelta> = forced
                .edits
                .into_iter()
                .filter(|edit| {
                    part_of(edit).is_some_and(|part| {
                        taken.contains(&part)
                            || part_node(&part).is_some_and(|node| whole.contains(&node))
                    })
                })
                .collect();
            let mut facets: BTreeMap<Uuid, Vec<(String, Option<Value>)>> = BTreeMap::new();
            for part in &taken {
                if let Part::ResourceFacet(resource, facet) = part {
                    facets
                        .entry(*resource)
                        .or_default()
                        .push((facet.clone(), part_value(&pending.source_graph, part)));
                }
            }
            replay_captured_deltas_onto(&mut scratch, chosen.iter().cloned());
            edits.extend(chosen);
            for (resource, facets) in facets {
                let record = record_with(&scratch, resource, facets);
                replay_captured_deltas_onto(&mut scratch, record.iter().cloned());
                edits.extend(record);
            }
        }
        // Every node the merge made or changed records where it came from.
        let container = Some(pending.source_container.to_string());
        let merged_nodes = Touched::of(&edits).nodes;
        for node in merged_nodes {
            if scratch.get_node_by_id(node).is_none() {
                continue;
            }
            let Some(origin) = pending.origins.get(&node) else {
                continue;
            };
            edits.push(CapturedDelta::ReplayRecordNodeDerivationById {
                node_id: node.to_string(),
                derivation: NodeDerivation {
                    sub_kind: ProvenanceSubKind::MergedFrom,
                    source_node: origin.to_string(),
                    source_graph: container.clone(),
                },
            });
        }
        let deltas = edits
            .iter()
            .map(|edit| {
                edit.replay_delta_as(&author)
                    .ok_or_else(|| SessionError::NotReplayable(format!("{edit:?}")))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let kind = ChangeKind::MergedIn {
            from: source.id(),
            through: pending.through,
        };
        let (_, applied) = target.edit_as(author.clone(), kind, |graph| {
            for delta in deltas {
                let _ = apply_graph_delta(graph, delta);
            }
        })?;
        let at = wall_clock_now();
        source.manifest.merged = Some(MergedMark {
            into: target.id(),
            through: pending.through.0,
            at,
        });
        let cursor = source.journal.live_cursor();
        source.changes.append(Change {
            author,
            kind: ChangeKind::MergedInto {
                into: target.id(),
                through: pending.through,
            },
            first: cursor,
            end: cursor,
        });
        source.revision += 1;
        let ours = target.pending(at)?;
        let theirs = source.pending(at)?;
        let ops: Vec<WriteOp> = ours.ops().iter().chain(theirs.ops()).cloned().collect();
        self.backend.apply(&ops).await?;
        target.stored(ours);
        source.stored(theirs);
        Ok(applied)
    }
}

/// The edit that sets `resource`'s record as `graph` holds it, with each
/// named facet set to its value or, for `None`, removed. Empty when the graph
/// holds no such resource.
fn record_with(
    graph: &Graph,
    resource: Uuid,
    facets: impl IntoIterator<Item = (String, Option<Value>)>,
) -> Vec<CapturedDelta> {
    let Some(mut record) = graph.resource_record(resource) else {
        return Vec::new();
    };
    for (facet, value) in facets {
        record.facets.retain(|held| held.facet != facet);
        if let Some(value) = value {
            record.facets.push(PersistedResourceFacet {
                facet,
                value_json: value.to_string(),
            });
        }
    }
    vec![CapturedDelta::ReplaySetResourceRecordById {
        resource_id: resource.to_string(),
        record: Some(record),
    }]
}

#[cfg(test)]
#[path = "graph_session_merge_tests.rs"]
mod tests;
