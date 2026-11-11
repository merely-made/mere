// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

impl Canvas {
    /// A node's render color, matching the shared node accent palette: amber when
    /// selected, else green open / red closed / blue idle. The host colors the canvas
    /// element's chrome-DOM gnodes with it so they carry node identity without the
    /// in-scene gnode layer. (Canvas-as-element — Phase 2.)
    pub fn node_color(&self, key: NodeKey) -> &'static str {
        palette::bg_hex(
            self.selected.contains(&key),
            self.node_states
                .get(&key)
                .copied()
                .unwrap_or(NodeState::Idle),
        )
    }

    /// A node's activation-state color WITHOUT the selection override — green open /
    /// red closed / blue idle. The chrome-DOM gnode colors its face with this and shows
    /// selection as a ring + lift instead, so the color channel stays free to carry
    /// activation state. The in-scene gnode path still uses [`node_color`](Self::node_color).
    pub fn node_state_color(&self, key: NodeKey) -> &'static str {
        palette::bg_hex(
            false,
            self.node_states
                .get(&key)
                .copied()
                .unwrap_or(NodeState::Idle),
        )
    }

    /// Theme the canvas's surfaces: the content-surface `backdrop` and the `edge`
    /// stroke color, as straight `[r, g, b, a]` (0..1). The host pushes these from
    /// the active theme so the graph re-themes with the chrome. Node *state* colors
    /// (open / closed / idle / selected) stay semantic and are not rethemed here.
    pub fn set_palette(&mut self, backdrop: [f32; 4], edge: [f32; 4]) {
        self.backdrop = ColorF::new(backdrop[0], backdrop[1], backdrop[2], backdrop[3]);
        self.style.edge_color = ColorF::new(edge[0], edge[1], edge[2], edge[3]);
    }

    /// Replace the live palette used to decode every derived face. Face bytes stay cached and
    /// unchanged; the next frame resolves their palette slots through these colors.
    pub fn set_derived_face_palette(&mut self, palette: DerivedFacePalette) {
        self.derived_face_palette = palette;
    }

    /// The palette currently used to decode derived faces.
    pub fn derived_face_palette(&self) -> DerivedFacePalette {
        self.derived_face_palette
    }

    /// A node's physical material (the Body axis): its per-node override if set, else the
    /// default [`seiche::NodeMaterial`] (the spawn restitution / friction / density).
    /// (Node body & face — material.)
    pub fn node_material(&self, key: NodeKey) -> seiche::NodeMaterial {
        self.member_of(key)
            .and_then(|id| self.node_materials.get(&id).copied())
            .unwrap_or_default()
    }

    /// Set a node's physical material (restitution / friction / density) and push it to the
    /// live body, so the node feels heavier / bouncier / grippier at once. A material equal to
    /// the default is stored too (an explicit override), so the facet shows it as set. Keyed by
    /// node UUID; a no-op for an unknown id. Persisted in the cartography sidecar. (Node body
    /// & face — material.)
    pub fn set_node_material(&mut self, id: uuid::Uuid, material: seiche::NodeMaterial) {
        if let Some(key) = self.graph.get_node_key_by_id(id) {
            self.node_materials.insert(id, material);
            self.physics.set_node_materials(vec![(key, material)]);
        }
    }

    /// Reset a node's physical material to the default (drop its override) and push the default
    /// back to the live body. Keyed by node UUID; a no-op for an unknown id. (Node body & face.)
    pub fn clear_node_material(&mut self, id: uuid::Uuid) {
        if self.node_materials.remove(&id).is_some()
            && let Some(key) = self.graph.get_node_key_by_id(id)
        {
            self.physics
                .set_node_materials(vec![(key, seiche::NodeMaterial::default())]);
        }
    }

    /// A node's face footprint (px): a per-node override if set, else size-by-degree
    /// (the face grows with the node's undirected degree, capped) when that mode is on,
    /// else the uniform default. The gnode applies the selection lift on top, and uses the
    /// same value to center the face on the seiche collider. (P0 resize.)
    pub fn node_size(&self, key: NodeKey) -> f32 {
        const DEFAULT: f32 = 36.0;
        const MAX: f32 = 88.0;
        if let Some(&s) = self.member_of(key).and_then(|id| self.node_sizes.get(&id)) {
            return s;
        }
        // Size by importance (the signal-driven channel) wins over size-by-degree: normalized
        // importance (0..=1) maps to DEFAULT..=MAX, so the most important node hits the cap and
        // the rest scale relative to it. An un-scored node reads 0 -> DEFAULT. (Graph signals.)
        if self.size_by_importance {
            let importance = self
                .channels
                .importance_held(self.importance_metric)
                .and_then(|weights| weights.get(&key).copied())
                .unwrap_or(0.0);
            return DEFAULT + importance * (MAX - DEFAULT);
        }
        // Recency (temporal channel, the registry's `order.recency` values): newest reads at the
        // cap, oldest at the default. An unscored node reads 0 -> DEFAULT. (Projection proofs —
        // P3, recency; G2b, one recency.)
        if self.size_by_recency {
            let recency = self
                .channels
                .recency_held()
                .and_then(|recency| recency.values.get(&key).copied())
                .unwrap_or(0.0);
            return DEFAULT + recency * (MAX - DEFAULT);
        }
        if self.size_by_degree {
            let degree = self.graph.projected_neighbors(key).count();
            return (DEFAULT + 8.0 * degree as f32).min(MAX);
        }
        DEFAULT
    }

    /// Override a single node's face footprint (px), keyed by node UUID; clamped to a sane
    /// range. Wins over size-by-degree / the default until [`clear_node_size`]; a no-op for
    /// an unknown id. (P0 resize.)
    pub fn set_node_size(&mut self, id: uuid::Uuid, size: f32) {
        if self.graph.get_node_key_by_id(id).is_some() {
            self.node_sizes.insert(id, size.clamp(16.0, 160.0));
            self.resettle_for_size();
        }
    }

    /// Clear a node's footprint override, reverting it to size-by-degree / the default.
    /// Keyed by node UUID; a no-op for an unknown id. (P0 resize.)
    pub fn clear_node_size(&mut self, id: uuid::Uuid) {
        if self.graph.get_node_key_by_id(id).is_some() {
            self.node_sizes.remove(&id);
            self.resettle_for_size();
        }
    }
}
