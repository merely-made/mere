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
//! The comparison is undo's own ([`SessionGraph::revert`]): reverting
//! "base to source" against the target is exactly taking each source change
//! the target left alone. Deletions merge the same way (M7).
//!
//! Merge runs over any graph that implements [`MergeGraph`] (engine split
//! D264); the kernel graph's is the kernel's revert and capture calls.

use std::collections::{BTreeMap, BTreeSet};

use mere::model::{PersistedResourceFacet, ResourceDelta};
use mere::session::{MergeGraph, MergePart};
use serde_json::Value;

use super::*;
use crate::manifest::MergedMark;
use crate::merge_rules::{FacetMerge, merge_resource_facet, resource_facet_has_rule};

/// How far a chain of picks or merged branches is followed before merge-in
/// gives up on it.
const MAX_LINEAGE_HOPS: usize = 64;

/// What a conflict takes (M5).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "choice", content = "value", rename_all = "snake_case")]
pub enum MergeChoice {
    /// What the branch or picked-off part holds. For a node, its relations
    /// come with it.
    Source,
    /// What the session being merged into holds.
    Target,
    /// A value supplied by hand, in the conflict's own form (`null`: absent).
    Hand(Value),
    /// For a picked node the target has since deleted: add the source node
    /// as a new node, leaving the deletion as it is.
    AddAsNew,
}

/// One part both sides changed differently since the base. Values are the
/// part as each graph holds it (`None`: absent), in the target's ids.
#[derive(Serialize, Deserialize)]
#[serde(bound = "")]
pub struct MergeConflict<G: MergeGraph = Graph> {
    pub part: G::Part,
    pub base: Option<Value>,
    pub source: Option<Value>,
    pub target: Option<Value>,
    /// Whether [`MergeChoice::AddAsNew`] applies: a picked node whose target
    /// node has since been deleted (the vanished source).
    #[serde(default)]
    pub add_as_new: bool,
}

impl<G: MergeGraph> Clone for MergeConflict<G> {
    fn clone(&self) -> Self {
        Self {
            part: self.part.clone(),
            base: self.base.clone(),
            source: self.source.clone(),
            target: self.target.clone(),
            add_as_new: self.add_as_new,
        }
    }
}

impl<G: MergeGraph> std::fmt::Debug for MergeConflict<G> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MergeConflict")
            .field("part", &self.part)
            .field("base", &self.base)
            .field("source", &self.source)
            .field("target", &self.target)
            .field("add_as_new", &self.add_as_new)
            .finish()
    }
}

impl<G: MergeGraph> PartialEq for MergeConflict<G> {
    fn eq(&self, other: &Self) -> bool {
        (
            &self.part,
            &self.base,
            &self.source,
            &self.target,
            self.add_as_new,
        ) == (
            &other.part,
            &other.base,
            &other.source,
            &other.target,
            other.add_as_new,
        )
    }
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
pub struct PendingMerge<G: MergeGraph = Graph> {
    target: SessionId,
    source: SessionId,
    matched: MergeMatch,
    through: Seq,
    planned_at: Seq,
    conflicts: Vec<MergeConflict<G>>,
    /// The clean edits, rule merges included, in the target's ids.
    edits: Vec<G::Delta>,
    /// The source's edits since the base, in the target's ids.
    change: Vec<G::Delta>,
    /// The source as merge-in reads it, in the target's ids.
    source_graph: G,
    /// Each target id the source names, with the source's own id for it.
    origins: BTreeMap<Uuid, Uuid>,
    source_container: Uuid,
}

impl<G: MergeGraph> Clone for PendingMerge<G> {
    fn clone(&self) -> Self {
        Self {
            target: self.target,
            source: self.source,
            matched: self.matched,
            through: self.through,
            planned_at: self.planned_at,
            conflicts: self.conflicts.clone(),
            edits: self.edits.clone(),
            change: self.change.clone(),
            source_graph: self.source_graph.clone(),
            origins: self.origins.clone(),
            source_container: self.source_container,
        }
    }
}

impl<G: MergeGraph> std::fmt::Debug for PendingMerge<G> {
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

impl<G: MergeGraph> PendingMerge<G> {
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

    pub fn conflicts(&self) -> &[MergeConflict<G>] {
        &self.conflicts
    }

    /// The edits the merge writes before any conflict's choice, in the
    /// target's ids.
    pub fn edits(&self) -> &[G::Delta] {
        &self.edits
    }

    /// Carry `choices`, made on an earlier plan of this merge, over to this
    /// one: a choice carries when this plan still has a conflict on its part
    /// that takes it. The rest are dropped, each with the reason, so the
    /// person can see what to choose again.
    pub fn carry_choices(
        &self,
        choices: &BTreeMap<G::Part, MergeChoice>,
    ) -> (BTreeMap<G::Part, MergeChoice>, Vec<DroppedChoice<G>>) {
        let mut carried = BTreeMap::new();
        let mut dropped = Vec::new();
        for (part, choice) in choices {
            let reason = match self
                .conflicts
                .iter()
                .find(|conflict| &conflict.part == part)
            {
                None => Some("no longer in conflict"),
                Some(conflict) if *choice == MergeChoice::AddAsNew && !conflict.add_as_new => {
                    Some("adding as new no longer applies")
                },
                Some(_) => match choice {
                    MergeChoice::Hand(value)
                        if !matches!(G::part_kind(part), MergePart::ResourceFacet(..))
                            && self
                                .source_graph
                                .part_edits(part, (!value.is_null()).then_some(value))
                                .is_err() =>
                    {
                        Some("the value given by hand does not fit")
                    },
                    _ => None,
                },
            };
            match reason {
                None => {
                    carried.insert(part.clone(), choice.clone());
                },
                Some(reason) => dropped.push(DroppedChoice {
                    part: part.clone(),
                    choice: choice.clone(),
                    reason: reason.to_string(),
                }),
            }
        }
        (carried, dropped)
    }
}

/// What a merge-in wrote, and the edits admission refused (engine-split D169).
pub struct Merged<G: MergeGraph = Graph> {
    pub applied: Applied,
    /// The edits left out while the rest applied, in order.
    pub refused: Vec<Refused<G>>,
}

/// An edit a merge-in could not apply because admission refused it.
#[derive(Serialize, Deserialize)]
#[serde(bound = "")]
pub struct Refused<G: MergeGraph = Graph> {
    /// Its position among the edits the merge wrote: the plan's clean edits,
    /// then each choice's, then the derivation records.
    pub position: usize,
    /// The part it writes; `None` for an edit with no part, such as a
    /// derivation record.
    pub part: Option<G::Part>,
    /// Why admission refused it, naming the handle for a collision.
    pub reason: String,
}

/// A choice made on an earlier plan that a new plan of the same merge could
/// not take, and why.
#[derive(Serialize, Deserialize)]
#[serde(bound = "")]
pub struct DroppedChoice<G: MergeGraph = Graph> {
    pub part: G::Part,
    pub choice: MergeChoice,
    pub reason: String,
}

// By hand, so they ask nothing of the graph type itself (D244).
impl<G: MergeGraph> Clone for Merged<G> {
    fn clone(&self) -> Self {
        Self {
            applied: self.applied.clone(),
            refused: self.refused.clone(),
        }
    }
}

impl<G: MergeGraph> std::fmt::Debug for Merged<G> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Merged")
            .field("applied", &self.applied)
            .field("refused", &self.refused)
            .finish()
    }
}

impl<G: MergeGraph> PartialEq for Merged<G> {
    fn eq(&self, other: &Self) -> bool {
        self.applied == other.applied && self.refused == other.refused
    }
}

impl<G: MergeGraph> Clone for Refused<G> {
    fn clone(&self) -> Self {
        Self {
            position: self.position,
            part: self.part.clone(),
            reason: self.reason.clone(),
        }
    }
}

impl<G: MergeGraph> std::fmt::Debug for Refused<G> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Refused")
            .field("position", &self.position)
            .field("part", &self.part)
            .field("reason", &self.reason)
            .finish()
    }
}

impl<G: MergeGraph> PartialEq for Refused<G> {
    fn eq(&self, other: &Self) -> bool {
        (self.position, &self.part, &self.reason) == (other.position, &other.part, &other.reason)
    }
}

impl<G: MergeGraph> Clone for DroppedChoice<G> {
    fn clone(&self) -> Self {
        Self {
            part: self.part.clone(),
            choice: self.choice.clone(),
            reason: self.reason.clone(),
        }
    }
}

impl<G: MergeGraph> std::fmt::Debug for DroppedChoice<G> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DroppedChoice")
            .field("part", &self.part)
            .field("choice", &self.choice)
            .field("reason", &self.reason)
            .finish()
    }
}

impl<G: MergeGraph> PartialEq for DroppedChoice<G> {
    fn eq(&self, other: &Self) -> bool {
        (&self.part, &self.choice, &self.reason) == (&other.part, &other.choice, &other.reason)
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
struct Basis<G: MergeGraph> {
    matched: MergeMatch,
    base: G,
    source_graph: G,
    change: Vec<G::Delta>,
    /// Source id to target id, for every source node.
    ids: BTreeMap<Uuid, Uuid>,
}

/// The `CopiedFrom` derivation of node `id` into `container`, looked up in
/// each graph in turn.
fn copied_from<G: MergeGraph>(graphs: &[&G], id: Uuid, container: Uuid) -> Option<Uuid> {
    let container = container.to_string();
    graphs
        .iter()
        .find_map(|graph| graph.copied_from(id, &container))
}

impl<B: Backend + Clone, G: MergeGraph> MereSessions<B, G> {
    async fn manifest_of(&self, id: SessionId) -> Result<GraphSessionManifest, SessionError> {
        let slots = self.slots();
        let mut manifest: GraphSessionManifest = read(&slots, &Keys::new(id).at(MANIFEST))
            .await?
            .ok_or(SessionError::Missing(id))?;
        manifest.merged = read(&slots, &Keys::new(id).at(MERGED)).await?;
        Ok(manifest)
    }

    /// Plan merging `source`, a branch or a picked-off part, into `target`.
    /// The plan is clean, or lists its conflicts; nothing is written.
    pub async fn merge_in(
        &self,
        target: &GraphSession<B, G>,
        source: &GraphSession<B, G>,
    ) -> Result<PendingMerge<G>, SessionError> {
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
        let (mut edits, kept) = G::revert(&basis.change, &basis.source_graph, &basis.base, live);
        let mut conflicts = Vec::new();
        let mut joined: BTreeMap<Uuid, Vec<(String, Value)>> = BTreeMap::new();
        let kept: BTreeSet<G::Part> = kept.into_iter().collect();
        for part in kept {
            let (theirs, ours) = (basis.source_graph.part_value(&part), live.part_value(&part));
            if theirs == ours {
                continue;
            }
            if let (MergePart::ResourceFacet(resource, facet), Some(theirs), Some(ours)) =
                (G::part_kind(&part), &theirs, &ours)
                && resource_facet_has_rule(&facet)
                && let Some(record) = live.resources().resource_record(resource)
                && let Ok(FacetMerge::Merged(value)) =
                    merge_resource_facet(&record.canonical_iri, &facet, ours, theirs)
            {
                joined.entry(resource).or_default().push((facet, value));
                continue;
            }
            conflicts.push(MergeConflict {
                base: basis.base.part_value(&part),
                add_as_new: basis.matched == MergeMatch::Pick
                    && matches!(G::part_kind(&part), MergePart::Node(_))
                    && theirs.is_some()
                    && ours.is_none(),
                source: theirs,
                target: ours,
                part,
            });
        }
        // A node in conflict decides its relations: they are asked about
        // only when neither end is, and wait for the node's choice otherwise.
        let disputed: BTreeSet<Uuid> = conflicts
            .iter()
            .filter_map(|conflict| match G::part_kind(&conflict.part) {
                MergePart::Node(id) => Some(id),
                _ => None,
            })
            .collect();
        let follows = |part: &G::Part| matches!(G::part_kind(part), MergePart::Edges(from, to) if disputed.contains(&from) || disputed.contains(&to));
        conflicts.retain(|conflict| !follows(&conflict.part));
        edits.retain(|edit| !G::part_of(edit).is_some_and(|part| follows(&part)));
        if !joined.is_empty() {
            let mut scratch = live.clone();
            scratch.replay_scratch(&edits);
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
        target: &GraphSession<B, G>,
        source: &GraphSession<B, G>,
    ) -> Result<Basis<G>, SessionError> {
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
        let ids = base
            .node_ids()
            .union(&source_graph.node_ids())
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
        target: &GraphSession<B, G>,
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

    /// A picked-off part's basis. Ids match through `CopiedFrom` derivations,
    /// hop by hop to the target (M3); a node made in a part along the way
    /// gets a new id, derived from the target and that node. The base is the
    /// target where the chain was first picked. The source is that base with
    /// everything since replayed: each part between's edits to what the source
    /// holds, up to where the next part was picked from it, then the source's
    /// own edits.
    async fn pick_basis(
        &self,
        target: &GraphSession<B, G>,
        source: &GraphSession<B, G>,
    ) -> Result<Basis<G>, SessionError> {
        let not_a_pick = || {
            SessionError::NotMergeable(format!(
                "{} is neither a branch nor a part picked from {}",
                source.id().as_uuid(),
                target.id().as_uuid()
            ))
        };
        // The chain: hops[i] goes from level i (0: the source) to level i + 1
        // (the last: the target); middles hold the parts between, opened.
        let mut hops = Vec::new();
        let mut middles: Vec<GraphSession<B, G>> = Vec::new();
        let mut manifest = source.manifest().clone();
        loop {
            if hops.len() >= MAX_LINEAGE_HOPS {
                return Err(not_a_pick());
            }
            let hop = self
                .pick_hop(&manifest, target)
                .await?
                .ok_or_else(not_a_pick)?;
            hops.push(hop);
            if hop.session == target.id() {
                break;
            }
            // Opened with these sessions' hooks: the kernel's for its own.
            let parent = self.open_with(hop.session).await?;
            manifest = parent.manifest().clone();
            middles.push(parent);
        }
        let levels: Vec<&GraphSession<B, G>> = std::iter::once(source).chain(&middles).collect();
        // Every graph a level's nodes are read from: its baseline, where the
        // level below was picked from it, and now.
        let graphs_of = |level: usize| -> Vec<G> {
            let session = levels[level];
            let mut graphs: Vec<G> = session.graph_at(Seq(0)).into_iter().collect();
            if level > 0 {
                graphs.extend(session.graph_at(hops[level - 1].at));
            }
            graphs.push(session.graph().clone());
            graphs
        };
        let target_ns = *target.id().as_uuid();
        let mint = |id: &Uuid| Uuid::new_v5(&target_ns, id.as_bytes());
        // Each level's ids to target ids, from the top down.
        let mut maps: Vec<BTreeMap<Uuid, Uuid>> = vec![BTreeMap::new(); levels.len()];
        for level in (0..levels.len()).rev() {
            let graphs = graphs_of(level);
            let refs: Vec<&G> = graphs.iter().collect();
            let ids: BTreeSet<Uuid> = graphs.iter().flat_map(G::node_ids).collect();
            let mut derived = 0;
            let mut map = BTreeMap::new();
            for id in ids {
                let above = copied_from(&refs, id, hops[level].container);
                derived += usize::from(above.is_some());
                let mapped = above.and_then(|above| {
                    if level + 1 == levels.len() {
                        Some(above)
                    } else {
                        maps[level + 1].get(&above).copied()
                    }
                });
                map.insert(id, mapped.unwrap_or_else(|| mint(&id)));
            }
            if hops[level].legacy && derived == 0 {
                // A fork carries no derivations into its parent: not a pick.
                return Err(not_a_pick());
            }
            maps[level] = map;
        }
        let root_at = hops.last().expect("a chain has a hop").at;
        let base = target
            .graph_at(root_at)
            .ok_or(SessionError::CursorOutOfRange {
                cursor: root_at,
                live: target.journal().live_cursor(),
            })?;
        let names = |map: &BTreeMap<Uuid, Uuid>| -> BTreeMap<String, String> {
            map.iter()
                .map(|(from, to)| (from.to_string(), to.to_string()))
                .collect()
        };
        // What the source holds, in target ids: the parts between carry only
        // their edits to it.
        let held: BTreeSet<Uuid> = maps[0].values().copied().collect();
        let mut change = Vec::new();
        for level in (1..levels.len()).rev() {
            let map = &maps[level];
            let within = |id: &Uuid| map.get(id).is_some_and(|mapped| held.contains(mapped));
            let names = names(map);
            for edit in levels[level].edits_in(Seq(0), hops[level - 1].at) {
                let touched = G::touched(std::slice::from_ref(&edit));
                if touched.session_level
                    || !touched.nodes.iter().all(&within)
                    || !touched
                        .edges
                        .iter()
                        .all(|(from, to)| within(from) && within(to))
                    || !touched.shown_surfaces.iter().all(&within)
                {
                    continue;
                }
                change.push(G::remap_ids(&edit, &names));
            }
        }
        // A node of the source's baseline the replay did not bring (a part
        // between with no journal for it) arrives whole, with its relations.
        let names = names(&maps[0]);
        let baseline = source
            .graph_at(Seq(0))
            .ok_or_else(|| SessionError::Corrupt("a part's baseline does not load".into()))?;
        let mut rebuilt = base.clone();
        rebuilt.replay_scratch(&change);
        let missing: Vec<Uuid> = baseline
            .node_ids()
            .into_iter()
            .filter(|id| !rebuilt.has_node(maps[0][id]))
            .collect();
        for id in &missing {
            if let Some(made) = baseline.recreate_node_edits(*id) {
                change.extend(made.iter().map(|edit| G::remap_ids(edit, &names)));
            }
        }
        change.extend(
            baseline
                .incident_edge_edits(&missing)
                .iter()
                .map(|edit| G::remap_ids(edit, &names)),
        );
        let live = source.journal().live_cursor();
        change.extend(
            source
                .edits_in(Seq(0), live)
                .iter()
                .map(|edit| G::remap_ids(edit, &names)),
        );
        let mut source_graph = base.clone();
        source_graph.replay_scratch(&change);
        Ok(Basis {
            matched: MergeMatch::Pick,
            base,
            source_graph,
            change,
            ids: maps.swap_remove(0),
        })
    }

    /// Write `pending` into `target` as one `MergedIn` change, taking each
    /// conflict's choice from `choices`, and mark `source` merged: kept, with
    /// its history, and read-only until the merge is undone (M6). Every merged
    /// node records a `MergedFrom` derivation naming the source node (M4).
    /// Both sessions are stored in one batch. Refused if either session
    /// changed since the plan, a conflict has no choice, or a choice does not
    /// fit its conflict. An edit admission refuses is left out and reported
    /// in [`Merged::refused`] while the rest applies (engine-split D169).
    pub async fn apply_merge(
        &self,
        target: &mut GraphSession<B, G>,
        source: &mut GraphSession<B, G>,
        pending: PendingMerge<G>,
        choices: &BTreeMap<G::Part, MergeChoice>,
        author: Author,
    ) -> Result<Merged<G>, SessionError> {
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
        let mut origins = pending.origins.clone();
        let mut edits = pending.edits.clone();
        let mut scratch = target.graph().clone();
        scratch.replay_scratch(&edits);
        let add = |scratch: &mut G, more: Vec<G::Delta>, edits: &mut Vec<_>| {
            scratch.replay_scratch(&more);
            edits.extend(more);
        };

        // Source: the edits that make each taken part read as the source
        // does. A node brought back or removed takes its relations with it.
        let taken: BTreeSet<&G::Part> = pending
            .conflicts
            .iter()
            .map(|conflict| &conflict.part)
            .filter(|part| choices.get(*part) == Some(&MergeChoice::Source))
            .collect();
        if !taken.is_empty() {
            let (forced, _) = G::revert(&pending.change, &pending.source_graph, &scratch, &scratch);
            let whole: BTreeSet<Uuid> = taken
                .iter()
                .filter_map(|part| match G::part_kind(part) {
                    MergePart::Node(id) => Some(id),
                    _ => None,
                })
                .collect();
            let chosen: Vec<G::Delta> = forced
                .into_iter()
                .filter(|edit| {
                    G::part_of(edit).is_some_and(|part| {
                        taken.contains(&part)
                            || G::part_node(&part).is_some_and(|node| whole.contains(&node))
                            || matches!(G::part_kind(&part), MergePart::Edges(from, to)
                                if whole.contains(&from) || whole.contains(&to))
                    })
                })
                .collect();
            add(&mut scratch, chosen, &mut edits);
            let mut facets: BTreeMap<Uuid, Vec<(String, Option<Value>)>> = BTreeMap::new();
            for part in &taken {
                if let MergePart::ResourceFacet(resource, facet) = G::part_kind(part) {
                    facets
                        .entry(resource)
                        .or_default()
                        .push((facet, pending.source_graph.part_value(part)));
                }
            }
            for (resource, facets) in facets {
                let record = record_with(&scratch, resource, facets);
                add(&mut scratch, record, &mut edits);
            }
        }

        for conflict in &pending.conflicts {
            match choices.get(&conflict.part) {
                // Edit by hand: the part as supplied, `null` for absent.
                Some(MergeChoice::Hand(value)) => {
                    let value = (!value.is_null()).then_some(value);
                    let made = scratch.part_edits(&conflict.part, value).map_err(|why| {
                        SessionError::NotMergeable(format!(
                            "a hand edit does not fit {:?}: {why}",
                            conflict.part
                        ))
                    })?;
                    add(&mut scratch, made, &mut edits);
                },
                // Add as new: the source node, under a new id, with its
                // relations, beside the target's choice to have deleted it.
                Some(MergeChoice::AddAsNew) => {
                    let MergePart::Node(node) = G::part_kind(&conflict.part) else {
                        return Err(SessionError::NotMergeable(
                            "only a node can be added as new".into(),
                        ));
                    };
                    if !conflict.add_as_new {
                        return Err(SessionError::NotMergeable(
                            "add as new is for a picked node its target deleted".into(),
                        ));
                    }
                    let fresh = Uuid::new_v5(
                        pending.target.as_uuid(),
                        format!("add-as-new:{node}").as_bytes(),
                    );
                    let names = BTreeMap::from([(node.to_string(), fresh.to_string())]);
                    let mut made = pending
                        .source_graph
                        .recreate_node_edits(node)
                        .unwrap_or_default();
                    made.extend(pending.source_graph.incident_edge_edits(&[node]));
                    let made = made.iter().map(|edit| G::remap_ids(edit, &names)).collect();
                    if let Some(origin) = origins.get(&node).copied() {
                        origins.insert(fresh, origin);
                    }
                    add(&mut scratch, made, &mut edits);
                },
                _ => {},
            }
        }

        // Every node the merge made or changed records where it came from.
        let container = Some(pending.source_container.to_string());
        for node in G::touched(&edits).nodes {
            if !scratch.has_node(node) {
                continue;
            }
            let Some(origin) = origins.get(&node) else {
                continue;
            };
            edits.extend(G::merged_from_edit(node, *origin, container.clone()));
        }
        let deltas = edits
            .iter()
            .map(|edit| {
                G::edit_of(edit, &author)
                    .map_err(|why| SessionError::NotReplayable(format!("{why}: {edit:?}")))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let kind = ChangeKind::MergedIn {
            from: source.id(),
            through: pending.through,
        };
        let mut refused = Vec::new();
        let (_, applied) = target.edit_as(author.clone(), kind, |graph| {
            for (position, (edit, delta)) in edits.iter().zip(deltas).enumerate() {
                if let Err(reason) = graph.apply(delta) {
                    refused.push(Refused {
                        position,
                        part: G::part_of(edit),
                        reason,
                    });
                }
            }
        })?;
        let at = wall_clock_now();
        let mark = MergedMark {
            into: target.id(),
            through: pending.through.0,
            at,
        };
        source.manifest.merged = Some(mark.clone());
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
        let mut ops: Vec<WriteOp> = ours.ops().iter().chain(theirs.ops()).cloned().collect();
        ops.push(pretty(Keys::new(source.id()).at(MERGED), &mark)?);
        self.backend.apply(&ops).await?;
        target.stored(ours);
        source.stored(theirs);
        Ok(Merged { applied, refused })
    }
}

/// The edit that sets `resource`'s record as `graph` holds it, with each
/// named facet set to its value or, for `None`, removed. Empty when the graph
/// holds no such resource.
fn record_with<G: MergeGraph>(
    graph: &G,
    resource: Uuid,
    facets: impl IntoIterator<Item = (String, Option<Value>)>,
) -> Vec<G::Delta> {
    let Some(mut record) = graph.resources().resource_record(resource) else {
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
    vec![G::Delta::from(ResourceDelta::ReplaySetResourceRecordById {
        resource_id: resource.to_string(),
        record: Some(record),
    })]
}

#[cfg(test)]
#[path = "graph_session_merge_tests.rs"]
mod tests;
