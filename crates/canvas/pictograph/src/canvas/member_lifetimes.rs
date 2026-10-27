// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Receive kernel membership changes before reconciling slot-keyed caches.
use super::*;
use kernel::graph::SurfaceNodeLifetime;

type Memberships = HashMap<NodeKey, (uuid::Uuid, SurfaceNodeLifetime)>;

impl Canvas {
    pub(super) fn reconcile_member_lifetimes(&mut self) -> Vec<NodeKey> {
        let live: Memberships = self
            .graph
            .nodes()
            .filter_map(|(key, node)| {
                Some((key, (node.id, self.graph.surface_node_lifetime(node.id)?)))
            })
            .collect();
        let retired: Memberships = self
            .cache_memberships
            .iter()
            .filter(|(key, member)| live.get(key) != Some(member))
            .map(|(&key, &member)| (key, member))
            .collect();
        self.retire_member_caches(&retired);
        self.cache_memberships = live;
        retired.keys().copied().collect()
    }

    pub(super) fn retire_member_caches(&mut self, retired: &Memberships) {
        if retired.is_empty() {
            return;
        }
        let keep = |key: &NodeKey| !retired.contains_key(key);
        let members: HashSet<_> = retired.values().map(|(id, _)| *id).collect();
        for &key in retired.keys() {
            // The synchronous read model must not paint a replacement at its
            // predecessor's position while waiting for a backend snapshot.
            self.view.set_position(key, Point2D::zero());
        }
        self.selected.retain(keep);
        self.pinned_nodes.retain(keep);
        self.forme_held.retain(keep);
        self.node_states.retain(|key, _| keep(key));
        self.node_shapes.retain(|key, _| keep(key));
        self.projection_representations.retain(|key, _| keep(key));
        self.grammar.retain(keep);
        self.mosaic.retain(keep);
        for positions in [&mut self.strategy_positions, &mut self.paused_positions]
            .into_iter()
            .flatten()
        {
            positions.retain(|(key, _)| keep(key));
        }
        for region in &mut self.strategy_regions {
            region.members.retain(keep);
        }
        // An exact UUID roster is authored view intent. A transient slot lens
        // must instead lose the retired members, even if those slots recur.
        if let Some(roster) = self.exact_scope.as_ref() {
            self.scope = Some(self.resolve_exact_scope(roster));
        } else if let Some(scope) = &mut self.scope {
            scope.retain(keep);
        }
        self.hidden_edges
            .retain(|cell| keep(&cell.from) && keep(&cell.to));
        let retire_appearances =
            |selected: &mut HashSet<LinkKey>,
             appearances: &mut HashMap<LinkKey, (NodeKey, NodeKey)>| {
                appearances.retain(|link, (a, b)| {
                    let live = keep(a) && keep(b);
                    if !live {
                        selected.remove(link);
                    }
                    live
                });
            };
        retire_appearances(
            &mut self.selected_links,
            &mut self.selected_link_appearances,
        );
        // Fold definitions are authored UUID rosters; only the saved runtime
        // selection/appearance caches must be prevented from reviving old slots.
        for state in self.fold_undo.iter_mut().chain(&mut self.fold_redo) {
            state.selected.retain(keep);
            retire_appearances(
                &mut state.selected_links,
                &mut state.selected_link_appearances,
            );
        }
        if self
            .self_link_press
            .as_ref()
            .is_some_and(|(link, _)| !keep(&link.display_from) || !keep(&link.display_to))
        {
            self.self_link_press = None;
        }
        if self
            .context_request
            .is_some_and(|request| request.node.is_some_and(|id| members.contains(&id)))
        {
            self.context_request = None;
        }
        let lost_drag = self.drag.is_some_and(|drag| !keep(&drag.node));
        if lost_drag {
            self.drag = None;
        }
        let lost_move = self.roles.retain_members(keep);
        if (lost_drag || lost_move)
            && self.drag.is_none()
            && self.roles.key_move.is_none()
            && self.field_drag.is_none()
        {
            // Do not run a release or Escape against the replacement body.
            self.physics.set_dragging(false);
        }
        // Keep the UUID-keyed override model (D1). A kernel-observed exit also
        // prunes removal/recreation hidden inside one ingest callback.
        self.node_faces.retain(|id, _| !members.contains(id));
        self.node_sizes.retain(|id, _| !members.contains(id));
        self.node_sprites.retain(|id, _| !members.contains(id));
        self.node_sprite_hulls.retain(|id, _| !members.contains(id));
        self.node_materials.retain(|id, _| !members.contains(id));
        self.node_rule_faces.retain(|id, _| !members.contains(id));
        if let Some(affinity) = &mut self.content_affinity {
            let before = affinity.len();
            affinity.retain(|(a, b, _)| keep(a) && keep(b));
            self.content_affinity_dirty |= before != affinity.len();
        }
        self.gloss_cache_inputs = None;
        self.gloss_positions = None;
        self.gloss_overlays.clear();
        self.projection_score = None;
        self.restored_score_hold = None;
        self.last_strategy_inputs = None;
        self.weighted_edges_cache = None;
        // Until the next current-graph install, no old force may address a
        // replacement body. Content rows above retain only surviving members.
        self.installed_affinity_revision = None;
        self.affinity_force_installed = false;
        self.physics.set_affinity_force(None);
    }
}
