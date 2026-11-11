// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

impl Canvas {
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
                },
            },
        );
        self.reconcile_derived();
        true
    }

    /// Retract the semantic relations between the two selected nodes that the
    /// installed host rule ([`set_withdraw_policy`](Self::set_withdraw_policy))
    /// allows, or [`default_withdraw_policy`](Self::default_withdraw_policy)
    /// without one, in both strata and both stored directions. Returns how many
    /// statements it
    /// withdrew; [`withdraw_selected_relation`](Self::withdraw_selected_relation)
    /// also says how many it left. A true removal, not the display-only
    /// [`hide_selected_edges`]. Scoped to the `Semantic` family, so navigation /
    /// provenance history on the same edge survives. Selecting a link never
    /// performs a source mutation; member actions go through the source-bound
    /// inspector API.
    pub fn retract_selected_relation(&mut self) -> usize {
        let Some(policy) = self.withdraw_policy.take() else {
            return self
                .withdraw_selected_relation(Self::default_withdraw_policy)
                .withdrawn;
        };
        let done = self.withdraw_selected_relation(&*policy);
        self.withdraw_policy = Some(policy);
        done.withdrawn
    }

    /// Install the host's rule for what the pair gesture may withdraw; `None`
    /// restores the canvas's own default.
    pub fn set_withdraw_policy(&mut self, policy: Option<WithdrawPolicy>) {
        self.withdraw_policy = policy;
    }

    /// Whether a canvas without a host rule may withdraw `statement`: it is in
    /// the user's own scope (the unnamed graph the relate gesture writes into,
    /// or `User`) and no method inferred it. Imported, agent- and method-backed
    /// statements stay.
    pub fn default_withdraw_policy(statement: &SemanticStatement) -> bool {
        matches!(
            statement.graph_scope,
            GraphScope::Default | GraphScope::User
        ) && statement.basis.method.is_none()
    }

    /// Withdraw, for the two selected nodes, each semantic statement `allow`
    /// accepts, in both strata and both stored directions: symmetric with
    /// [`assert_selected_relation`], so `>unrelate` mirrors `>relate` on the same
    /// gesture. The kernel asks `allow` per statement; a refused statement stays
    /// held and is counted in [`Retraction::refused`].
    pub fn withdraw_selected_relation(
        &mut self,
        allow: impl Fn(&SemanticStatement) -> bool,
    ) -> Retraction {
        let mut done = Retraction::default();
        if self.selected.len() == 2 {
            let mut pair: Vec<NodeKey> = self.selected.iter().copied().collect();
            pair.sort_by_key(|k| self.graph.get_node(*k).map(|n| n.id));
            let semantic = RelationSelector::Family(EdgeFamily::Semantic);
            for (from, to) in [(pair[0], pair[1]), (pair[1], pair[0])] {
                let part = self
                    .graph
                    .retract_relations_where(from, to, semantic, &allow);
                done.withdrawn += part.withdrawn;
                done.refused += part.refused;
            }
        }
        if done.withdrawn > 0 {
            self.reconcile_derived();
        }
        done
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
}
