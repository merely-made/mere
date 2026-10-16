// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Selection, tagging, semantic relations, and node-state setters.

use kernel::graph::Graph;
use kernel::graph::apply::{GraphDelta, GraphDeltaResult, apply_graph_delta};

use super::*;

/// Retract relations through the sanctioned delta path, unwrapping the count.
/// (Write-path migration, 2026-07-01.)
fn retract_via_delta(
    graph: &mut Graph,
    from: NodeKey,
    to: NodeKey,
    selector: RelationSelector,
) -> usize {
    match apply_graph_delta(graph, GraphDelta::RetractRelations { from, to, selector }) {
        GraphDeltaResult::EdgesRemoved(n) => n,
        _ => 0,
    }
}

impl Canvas {
    /// Replace the selection with just `key` (clearing any selected nodes/edges).
    pub(crate) fn select_only(&mut self, key: NodeKey) {
        self.sync_attention();
        self.selected.clear();
        self.selected_links.clear();
        self.selected_link_appearances.clear();
        self.selected.insert(key);
    }

    /// Select the existing node with `url` (URL identity), if present, without
    /// adding one. Returns whether a node was found and focused. The host calls
    /// this to restore the focused node from persisted view-intent.
    pub fn select_by_url(&mut self, url: &str) -> bool {
        if let Some((key, _)) = self.graph.get_node_by_url(url) {
            self.select_only(key);
            true
        } else {
            false
        }
    }

    /// Center the camera on the (first) selected node — the restore framing:
    /// a session switch or boot re-selects where the user was, and this puts
    /// that node at the viewport's center instead of leaving the camera at
    /// whatever origin happens to crop it. The node-keyed twin of
    /// [`Self::center_on_field`]. Returns `false` with nothing selected or no
    /// position yet.
    pub fn center_on_selected(&mut self) -> bool {
        let Some(&key) = self.selected.iter().next() else {
            return false;
        };
        let Some(pos) = self.view.position_of(key) else {
            return false;
        };
        // screen = world * zoom + offset; put the node at the viewport center.
        self.follow = false;
        self.camera.offset.0 = self.view_w as f32 / 2.0 - pos.x * self.camera.zoom;
        self.camera.offset.1 = self.view_h as f32 / 2.0 - pos.y * self.camera.zoom;
        true
    }

    /// Replace the selection with just `member` (member-id identity), if it is
    /// in the graph. Returns whether it was found. The host calls this on a
    /// right-click so node-scoped actions apply to whatever the pointer landed
    /// on. The member-keyed twin of [`Self::select_by_url`].
    pub fn select_member(&mut self, member: uuid::Uuid) -> bool {
        match self.graph.get_node_key_by_id(member) {
            Some(key) => {
                self.select_only(key);
                true
            },
            None => false,
        }
    }

    /// Toggle `member` in or out of the selection (a multi-select add), keeping the
    /// rest selected — the member-keyed twin of the canvas's Shift-click. Clears the
    /// edge selection (matching that gesture) so a mixed node+edge selection can't
    /// confuse the pairwise relate. Returns `false` if the member is not in the
    /// graph. Selection is read live at frame time, so no reconcile is needed.
    pub fn toggle_select_member(&mut self, member: uuid::Uuid) -> bool {
        let Some(key) = self.graph.get_node_key_by_id(member) else {
            return false;
        };
        self.sync_attention();
        if !self.selected.remove(&key) {
            self.selected.insert(key);
        }
        self.selected_links.clear();
        self.selected_link_appearances.clear();
        true
    }

    /// Remove the single focused node from the session graph (if exactly one is
    /// selected), returning its member id (the kernel node UUID) so the host can
    /// reap any activation for it. Clears the selection and reconciles the physics
    /// + node pool to the smaller graph. Returns `None` when zero or many nodes
    /// are selected, leaving the graph untouched. The host calls this on a
    /// delete-node gesture; deactivation (reaping the actor) is the host's job.
    pub fn remove_focused(&mut self) -> Option<uuid::Uuid> {
        if self.selected.len() != 1 {
            return None;
        }
        let key = *self.selected.iter().next()?;
        let id = self.graph.get_node(key)?.id;
        let _ = apply_graph_delta(&mut self.graph, GraphDelta::RemoveNode { key });
        self.pinned_nodes.remove(&key);
        self.sync_attention();
        self.selected.clear();
        self.selected_links.clear();
        self.selected_link_appearances.clear();
        self.reconcile_derived();
        Some(id)
    }

    /// Clear the node + edge selection (focus) without removing anything from
    /// the graph. The host calls this to drop focus — e.g. closing the last
    /// workbench tile returns to the graph with nothing focused, so the node
    /// deactivates instead of its Cartography preview re-activating it.
    /// (Card-system plan, Phase 1.)
    pub fn clear_selection(&mut self) {
        if self.selected.is_empty() && self.selected_links.is_empty() {
            return;
        }
        self.sync_attention();
        self.selected.clear();
        self.selected_links.clear();
        self.selected_link_appearances.clear();
        self.reconcile_derived();
    }

    /// The single focused node key, for layout strategies that center on a selection
    /// (radial). `None` when zero or many nodes are selected, so a focus-driven layout
    /// stays well-defined (it no-ops rather than picking an arbitrary node from a
    /// multi-selection). (Layout picker — radial.)
    pub fn focused_key(&self) -> Option<NodeKey> {
        match self.selected.len() {
            1 => self.selected.iter().copied().next(),
            _ => None,
        }
    }

    /// Assert a semantic relation of `sub_kind` between exactly two selected
    /// nodes — the user-initiated edge-creation gesture the rich kernel taxonomy
    /// always supported but the UI never reached. The pair is ordered by node
    /// UUID so a symmetric relation is reproducible; the edge is created or
    /// merged (idempotent per sub-kind) via [`Graph::assert_relation`]. Returns
    /// `true` when an edge was asserted, `false` for any selection that is not a
    /// clean pair. The springs / drawn edges refresh on the next reconcile.
    pub fn assert_selected_relation(&mut self, sub_kind: SemanticSubKind) -> bool {
        if self.selected.len() != 2 {
            return false;
        }
        let mut pair: Vec<NodeKey> = self.selected.iter().copied().collect();
        pair.sort_by_key(|k| self.graph.get_node(*k).map(|n| n.id));
        // `assert_relation` returns `None` for a no-op re-assert (the sub-kind is
        // already present), so we don't gate success on its return: for a clean
        // pair the relation is present afterwards either way, which is what
        // "relate these two" means. Reconcile rebuilds edges / springs.
        let asserter_iri = self.graph.write_author().asserter_iri();
        let _ = apply_graph_delta(
            &mut self.graph,
            GraphDelta::AssertRelation {
                asserter_iri,
                from: pair[0],
                to: pair[1],
                assertion: EdgeAssertion::Semantic {
                    sub_kind,
                    label: None,
                    decay_progress: None,
                },
            },
        );
        self.reconcile_derived();
        true
    }

    /// Assert a semantic relation between two stable graph members. This is the
    /// card/detail twin of [`assert_selected_relation`]: the roster Link Card has
    /// explicit endpoints, so it should not need to perturb selection first.
    pub fn assert_relation_between_members(
        &mut self,
        from_id: uuid::Uuid,
        to_id: uuid::Uuid,
        sub_kind: SemanticSubKind,
    ) -> bool {
        let Some(from) = self.graph.get_node_key_by_id(from_id) else {
            return false;
        };
        let Some(to) = self.graph.get_node_key_by_id(to_id) else {
            return false;
        };
        let asserter_iri = self.graph.write_author().asserter_iri();
        let _ = apply_graph_delta(
            &mut self.graph,
            GraphDelta::AssertRelation {
                asserter_iri,
                from,
                to,
                assertion: EdgeAssertion::Semantic {
                    sub_kind,
                    label: None,
                    decay_progress: None,
                },
            },
        );
        self.reconcile_derived();
        true
    }

    /// Insert `tag` on every selected node — the user-initiated tagging gesture
    /// (the context menu's "Add tag…"). Trims the tag; an empty tag or empty
    /// selection is a no-op. Returns how many nodes newly gained the tag (an
    /// already-tagged node counts 0). Tags are node truth the host persists; they
    /// do not affect layout, so no reconcile is needed.
    pub fn tag_selected(&mut self, tag: &str) -> usize {
        let tag = tag.trim();
        if tag.is_empty() {
            return 0;
        }
        let keys: Vec<NodeKey> = self.selected.iter().copied().collect();
        let mut tagged = 0;
        for key in keys {
            if matches!(
                apply_graph_delta(
                    &mut self.graph,
                    GraphDelta::InsertNodeTag {
                        key,
                        tag: tag.to_string()
                    },
                ),
                GraphDeltaResult::NodeMetadataUpdated(true)
            ) {
                tagged += 1;
            }
        }
        tagged
    }

    /// Insert `tag` on one stable graph member, if present. Returns whether the
    /// node newly gained it. Member addressing matters for background
    /// projections, where duplicate URL nodes must not make a first-match
    /// choice on the host's behalf.
    pub fn tag_node(&mut self, member: uuid::Uuid, tag: &str) -> bool {
        let Some(key) = self.graph.get_node_key_by_id(member) else {
            return false;
        };
        matches!(
            apply_graph_delta(
                &mut self.graph,
                GraphDelta::InsertNodeTag {
                    key,
                    tag: tag.to_string()
                },
            ),
            GraphDeltaResult::NodeMetadataUpdated(true)
        )
    }

    /// Insert `tag` on the node addressed by `url`, if present — a by-url tagging gesture (the
    /// Alembic "keep" one-click promote, distinct from the selection-based [`tag_selected`]).
    /// Returns whether the node newly gained the tag. Tags are node truth the host persists.
    pub fn tag_node_by_url(&mut self, url: &str, tag: &str) -> bool {
        let Some(member) = self.graph.get_node_by_url(url).map(|(_, node)| node.id) else {
            return false;
        };
        self.tag_node(member, tag)
    }

    /// Remove `tag` from one stable graph member, if present. Returns whether
    /// it was removed.
    pub fn untag_node(&mut self, member: uuid::Uuid, tag: &str) -> bool {
        let Some(key) = self.graph.get_node_key_by_id(member) else {
            return false;
        };
        matches!(
            apply_graph_delta(
                &mut self.graph,
                GraphDelta::RemoveNodeTag {
                    key,
                    tag: tag.to_string()
                },
            ),
            GraphDeltaResult::NodeMetadataUpdated(true)
        )
    }

    /// Remove `tag` from the node addressed by `url`, if present (the Alembic "release"
    /// one-click demote). Returns whether it was removed. A node still tagged otherwise stays
    /// long-term (Saved) — only this tag is touched.
    pub fn untag_node_by_url(&mut self, url: &str, tag: &str) -> bool {
        let Some(member) = self.graph.get_node_by_url(url).map(|(_, node)| node.id) else {
            return false;
        };
        self.untag_node(member, tag)
    }

    /// Retract the user-asserted semantic relation(s) on the selected edge(s) —
    /// a true removal, not the display-only [`hide_selected_edges`]. Scoped to the
    /// `Semantic` family, so navigation / provenance history on the same edge
    /// survives; an edge left with no families is garbage-collected by the kernel.
    /// Returns how many pairwise semantic rows were retracted. Selecting a
    /// link never performs a source mutation; member actions go through the
    /// source-bound inspector API.
    pub fn retract_selected_relation(&mut self) -> usize {
        let mut removed = 0;
        // Symmetric with `assert_selected_relation`: a two-node selection retracts
        // the relation between the pair (either stored direction), so `>unrelate`
        // mirrors `>relate` on the same gesture.
        if self.selected.len() == 2 {
            let mut pair: Vec<NodeKey> = self.selected.iter().copied().collect();
            pair.sort_by_key(|k| self.graph.get_node(*k).map(|n| n.id));
            removed += self.retract_semantic_between(pair[0], pair[1]);
            removed += self.retract_semantic_between(pair[1], pair[0]);
        }
        if removed > 0 {
            self.reconcile_derived();
        }
        removed
    }

    /// Retract every semantic relation on the directed edge `a -> b`. The `Family`
    /// selector is read-only (not retractable), so enumerate the edge's semantic
    /// sub-kinds and retract each — the user-meaning relations go, while traversal
    /// / provenance history on the same edge survives (an edge left with no
    /// families is garbage-collected by the kernel). Returns how many were removed.
    pub(crate) fn retract_semantic_between(&mut self, a: NodeKey, b: NodeKey) -> usize {
        let sub_kinds: Vec<SemanticSubKind> = self
            .graph
            .find_edge_key(a, b)
            .and_then(|k| self.graph.get_edge(k))
            .and_then(|p| p.semantic_data())
            .map(|s| s.sub_kinds.iter().copied().collect())
            .unwrap_or_default();
        let mut removed = 0;
        for sk in sub_kinds {
            removed += retract_via_delta(&mut self.graph, a, b, RelationSelector::Semantic(sk));
        }
        removed
    }

    /// Retract a specific relation selector between two stable graph members.
    /// Used by the roster Link Card; unlike `retract_selected_relation`, it can
    /// target one semantic cell without first selecting an edge on the canvas.
    pub fn retract_relation_between_members(
        &mut self,
        from_id: uuid::Uuid,
        to_id: uuid::Uuid,
        selector: RelationSelector,
    ) -> usize {
        let Some(from) = self.graph.get_node_key_by_id(from_id) else {
            return 0;
        };
        let Some(to) = self.graph.get_node_key_by_id(to_id) else {
            return 0;
        };
        let removed = retract_via_delta(&mut self.graph, from, to, selector);
        if removed > 0 {
            self.reconcile_derived();
        }
        removed
    }

    /// Whether any canonical link is currently selected (the host routes
    /// inspection to the link, or node actions when no link is selected).
    pub fn has_selected_edges(&self) -> bool {
        !self.selected_links.is_empty()
    }

    /// Compatibility projection for hosts that only support Surface pairs.
    /// Resource links require `selected_link_keys` plus `link_appearance_pair`.
    pub fn selected_link_pairs(&self) -> Vec<(uuid::Uuid, uuid::Uuid)> {
        self.selected_links
            .iter()
            .filter(|key| key.stratum == scenograph::link::LinkStratum::Surface)
            .filter_map(|key| {
                let a = key.first.parse::<uuid::Uuid>().ok()?;
                let b = key.second.parse::<uuid::Uuid>().ok()?;
                Some((a.min(b), a.max(b)))
            })
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect()
    }

    /// Source identities and marks of the current resting stroke appearances.
    /// Geometry, picking and host-accessible alternates read this same projection.
    pub fn visible_link_appearances(&self) -> Vec<crate::canvas::LinkAppearance> {
        crate::canvas::edge_cells::visible_link_appearances_with_assertion_hides(
            &self.graph,
            &self.view,
            &self.hidden_links,
            &self.hidden_edges,
            &self.hidden_assertions,
            |key| self.node_size(key),
        )
        .into_iter()
        .map(|segment| segment.appearance)
        .collect()
    }

    /// Canonical identities hit on the canvas. This is the only closed-link
    /// selection key; displayed Resource appearances can share one key.
    pub fn selected_link_keys(&self) -> Vec<scenograph::link::LinkKey> {
        let mut keys: Vec<_> = self.selected_links.iter().cloned().collect();
        keys.sort();
        keys
    }

    /// Resolve a canonical selected link to the displayed Surface appearance
    /// that was hit. If no appearance was recorded (for example, a restored
    /// key), choose the first stable current appearance for that key.
    pub fn link_appearance_pair(
        &self,
        key: &scenograph::link::LinkKey,
    ) -> Option<(uuid::Uuid, uuid::Uuid)> {
        let pair = self
            .selected_link_appearances
            .get(key)
            .copied()
            .or_else(|| {
                crate::canvas::edge_cells::visible_link_appearances_with_assertion_hides(
                    &self.graph,
                    &self.view,
                    &self.hidden_links,
                    &self.hidden_edges,
                    &self.hidden_assertions,
                    |key| self.node_size(key),
                )
                .into_iter()
                .find(|segment| &segment.appearance.key == key)
                .map(|segment| {
                    (
                        segment.appearance.display_from,
                        segment.appearance.display_to,
                    )
                })
            })?;
        Some((
            self.graph.get_node(pair.0)?.id,
            self.graph.get_node(pair.1)?.id,
        ))
    }

    /// Hide selected links as whole canonical targets and clear their selection.
    pub fn hide_selected_links(&mut self) -> usize {
        let before = self.hidden_links.len();
        self.hidden_links.extend(self.selected_links.drain());
        let hidden = self.hidden_links.len() - before;
        self.selected_link_appearances.clear();
        if hidden > 0 {
            self.resync_edge_springs();
        }
        hidden
    }

    /// Replace the canvas' view-local closed-link hides from a host intent.
    /// Exact legacy directed-cell eligibility filters stay independent.
    pub fn set_hidden_link_keys(
        &mut self,
        keys: impl IntoIterator<Item = scenograph::link::LinkKey>,
    ) -> bool {
        let replacement: std::collections::HashSet<_> = keys.into_iter().collect();
        if replacement == self.hidden_links {
            return false;
        }
        self.hidden_links = replacement;
        self.resync_edge_springs();
        true
    }

    /// Return a stable ordered snapshot for host view-intent persistence.
    pub fn hidden_link_keys(&self) -> Vec<scenograph::link::LinkKey> {
        let mut keys: Vec<_> = self.hidden_links.iter().cloned().collect();
        keys.sort();
        keys
    }

    /// Read-only exact assertion hides for consumers that must preserve one
    /// spring contribution per independently hidden source record.
    pub fn hidden_assertions(&self) -> &HashSet<String> {
        &self.hidden_assertions
    }

    /// Return a stable ordered snapshot for host view-intent persistence.
    pub fn hidden_assertion_ids(&self) -> Vec<String> {
        let mut ids: Vec<_> = self.hidden_assertions.iter().cloned().collect();
        ids.sort();
        ids
    }

    /// Replace exact source assertion hides restored from a host view intent.
    pub fn set_hidden_assertion_ids(&mut self, ids: impl IntoIterator<Item = String>) -> bool {
        let replacement: HashSet<_> = ids.into_iter().collect();
        if replacement == self.hidden_assertions {
            return false;
        }
        self.hidden_assertions = replacement;
        self.resync_edge_springs();
        true
    }

    /// Clear only exact assertion hides, preserving whole-link and legacy
    /// selector visibility state.
    pub fn show_all_assertions(&mut self) -> usize {
        let count = self.hidden_assertions.len();
        if count > 0 {
            self.hidden_assertions.clear();
            self.resync_edge_springs();
        }
        count
    }

    /// Reveal every closed-link hide and every legacy cell eligibility filter.
    pub fn show_all_links(&mut self) -> usize {
        let link_count = self.hidden_links.len();
        let legacy_count = self.hidden_edges.len();
        let count = link_count + legacy_count;
        self.hidden_links.clear();
        self.hidden_edges.clear();
        if legacy_count > 0 || link_count > 0 {
            self.resync_edge_springs();
        }
        count
    }

    /// Compatibility name for host controls. Selection is link-keyed, so this
    /// now hides the whole selected canonical link.
    pub fn hide_selected_edges(&mut self) -> usize {
        self.hide_selected_links()
    }

    /// Whether every live relation cell in the endpoint bundle is hidden.
    pub fn edge_between_members_hidden(&self, from_id: uuid::Uuid, to_id: uuid::Uuid) -> bool {
        self.edge_cells_between_members(from_id, to_id)
            .is_some_and(|cells| {
                !cells.is_empty() && cells.iter().all(|cell| self.hidden_edges.contains(cell))
            })
    }

    /// Whether one directed relation cell between two graph members is hidden.
    pub fn relation_between_members_hidden(
        &self,
        from_id: uuid::Uuid,
        to_id: uuid::Uuid,
        selector: RelationSelector,
    ) -> bool {
        self.edge_cell_between_members(from_id, to_id, selector)
            .is_some_and(|cell| self.hidden_edges.contains(&cell))
    }

    /// Hide every live relation cell in the endpoint bundle between two graph members.
    pub fn hide_edge_between_members(&mut self, from_id: uuid::Uuid, to_id: uuid::Uuid) -> bool {
        let Some(cells) = self.edge_cells_between_members(from_id, to_id) else {
            return false;
        };
        let changed = cells.into_iter().fold(false, |changed, cell| {
            self.hidden_edges.insert(cell) || changed
        });
        if changed {
            self.resync_edge_springs();
        }
        changed
    }

    /// Hide one directed relation cell between two graph members.
    pub fn hide_relation_between_members(
        &mut self,
        from_id: uuid::Uuid,
        to_id: uuid::Uuid,
        selector: RelationSelector,
    ) -> bool {
        let changed = self
            .edge_cell_between_members(from_id, to_id, selector)
            .is_some_and(|cell| self.hidden_edges.insert(cell));
        if changed {
            self.resync_edge_springs();
        }
        changed
    }

    /// Reveal every hidden relation cell in the endpoint bundle between two graph members.
    pub fn show_edge_between_members(&mut self, from_id: uuid::Uuid, to_id: uuid::Uuid) -> bool {
        let Some(pair) = self.edge_pair_between_members(from_id, to_id) else {
            return false;
        };
        let before = self.hidden_edges.len();
        self.hidden_edges
            .retain(|cell| cell.endpoint_pair() != pair);
        let changed = self.hidden_edges.len() != before;
        if changed {
            self.resync_edge_springs();
        }
        changed
    }

    /// Reveal one directed relation cell between two graph members.
    pub fn show_relation_between_members(
        &mut self,
        from_id: uuid::Uuid,
        to_id: uuid::Uuid,
        selector: RelationSelector,
    ) -> bool {
        let (Some(from), Some(to)) = (
            self.graph.get_node_key_by_id(from_id),
            self.graph.get_node_key_by_id(to_id),
        ) else {
            return false;
        };
        let changed = self.hidden_edges.remove(&EdgeCell { from, to, selector });
        if changed {
            self.resync_edge_springs();
        }
        changed
    }

    /// Hidden display-only endpoint bundles as stable graph member ids.
    pub fn hidden_edge_member_pairs(&self) -> Vec<(uuid::Uuid, uuid::Uuid)> {
        self.hidden_edges
            .iter()
            .filter_map(|cell| {
                let (a, b) = cell.endpoint_pair();
                Some((self.graph.get_node(a)?.id, self.graph.get_node(b)?.id))
            })
            .collect()
    }

    /// Whether the pair has at least one visible live relation cell.
    pub(crate) fn edge_pair_has_visible_relation(&self, a: NodeKey, b: NodeKey) -> bool {
        self.edge_cells_between_pair(a, b)
            .into_iter()
            .any(|cell| !self.hidden_edges.contains(&cell))
    }

    fn edge_pair_between_members(
        &self,
        from_id: uuid::Uuid,
        to_id: uuid::Uuid,
    ) -> Option<(NodeKey, NodeKey)> {
        let from = self.graph.get_node_key_by_id(from_id)?;
        let to = self.graph.get_node_key_by_id(to_id)?;
        Some(if from <= to { (from, to) } else { (to, from) })
    }

    fn edge_cell_between_members(
        &self,
        from_id: uuid::Uuid,
        to_id: uuid::Uuid,
        selector: RelationSelector,
    ) -> Option<EdgeCell> {
        let from = self.graph.get_node_key_by_id(from_id)?;
        let to = self.graph.get_node_key_by_id(to_id)?;
        let cell = EdgeCell { from, to, selector };
        self.graph
            .projected_relations()
            .any(|(_, relation)| {
                relation.from == from
                    && relation.to == to
                    && crate::canvas::edge_cells::selector_for_relation_kind(relation.kind)
                        == selector
            })
            .then_some(cell)
    }

    fn edge_cells_between_members(
        &self,
        from_id: uuid::Uuid,
        to_id: uuid::Uuid,
    ) -> Option<Vec<EdgeCell>> {
        let from = self.graph.get_node_key_by_id(from_id)?;
        let to = self.graph.get_node_key_by_id(to_id)?;
        Some(self.edge_cells_between_pair(from, to))
    }

    fn edge_cells_between_pair(&self, a: NodeKey, b: NodeKey) -> Vec<EdgeCell> {
        let pair = if a <= b { (a, b) } else { (b, a) };
        self.graph
            .projected_relations()
            .filter_map(|(_, relation)| {
                let rel_pair = if relation.from <= relation.to {
                    (relation.from, relation.to)
                } else {
                    (relation.to, relation.from)
                };
                (rel_pair == pair).then_some(crate::canvas::edge_cells::edge_cell_for_relation(
                    relation.from,
                    relation.to,
                    relation.kind,
                ))
            })
            .collect()
    }

    /// Reveal every hidden edge. Returns how many were shown.
    pub fn show_all_edges(&mut self) -> usize {
        self.show_all_links()
    }

    /// Re-sync the physics spring topology to the current visible relation-cell set and
    /// give it a quick settle nudge. Hiding/showing a cell should relax/restore its own
    /// spring in this instance right away, not wait for an unrelated graph mutation to
    /// reconcile it. (Swatch-primitive P5 — hiding relaxes the spring in that instance
    /// only; graph truth, membership, and every other instance are unaffected.)
    fn resync_edge_springs(&mut self) {
        self.physics_view_revision += 1;
        self.physics
            .sync_edges(visible_relation_edges_with_assertion_hides(
                &self.graph,
                &self.hidden_edges,
                &self.hidden_links,
                &self.hidden_assertions,
            ));
        self.settle_physics(SETTLE_TICKS / 3);
    }

    /// Set the per-node activation states the canvas colors its on-screen nodes
    /// by, keyed by node UUID (the host's member id); the canvas resolves each to
    /// its `NodeKey`. The host recomputes + pushes this as the actor pool / content
    /// cache change; a node absent from `states` colors as [`NodeState::Idle`].
    pub fn set_node_states(&mut self, states: HashMap<uuid::Uuid, NodeState>) {
        self.node_states = states
            .into_iter()
            .filter_map(|(id, state)| self.graph.get_node_by_id(id).map(|(key, _)| (key, state)))
            .collect();
    }
}
