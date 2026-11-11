// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

impl Canvas {
    /// Inject a host-computed **content-affinity** signal — semantic similarity from node
    /// embeddings, as `(a, b, weight)` triples (`weight` in `0..=1`) — to drive the affinity force,
    /// superseding the internal structural-Jaccard signal while set. `None` reverts to structural.
    ///
    /// The host owns the embedding provider (so burn stays out of the canvas), recomputes this when
    /// node *content* changes, and re-injects; the canvas installs it on the next [`frame`](Self::frame)
    /// under the [`cluster_by_affinity`](Self::set_cluster_by_affinity) toggle, with a settle so the
    /// new clustering takes. `Some(empty)` is authoritative-but-inert (the host ran embeddings and
    /// found no pairs above its threshold) — it clears the force rather than falling back to
    /// structural; pass `None` to opt back into structural. (burn brief Lane 5 — P4.)
    pub fn set_content_affinity(&mut self, pairs: Option<Vec<(NodeKey, NodeKey, f32)>>) {
        self.content_affinity = pairs;
        self.content_affinity_dirty = true;
    }

    /// Whether a host content-affinity signal is currently the affinity source (vs the internal
    /// structural one). `Some(empty)` still counts as content-sourced. (burn brief Lane 5 — P4.)
    pub fn has_content_affinity(&self) -> bool {
        self.content_affinity.is_some()
    }

    /// Set how the structural and content affinity signals combine ([`AffinityBlend`]). Default is
    /// [`Blend`](AffinityBlend::Blend) (noisy-OR of both). Forces a rebuild of the affinity force on
    /// the next [`frame`](Self::frame) so the mode change takes. (burn brief Lane 5 — P6.)
    pub fn set_affinity_blend(&mut self, mode: AffinityBlend) {
        if self.affinity_blend == mode {
            return;
        }
        self.affinity_blend = mode;
        // The contributing source set changed; re-arm both gates so the next sync reinstalls.
        self.installed_affinity_revision = None;
        self.content_affinity_dirty = true;
    }

    /// How the structural and content affinity signals currently combine. (burn brief Lane 5 — P6.)
    pub fn affinity_blend(&self) -> AffinityBlend {
        self.affinity_blend
    }

    /// The number of affinity pairs in the live affinity force (`0` when off). Test introspection
    /// for the P4 wiring (inline backend). (Graph signals — P4.)
    #[cfg(test)]
    pub(crate) fn affinity_pair_count(&self) -> usize {
        self.physics.affinity_pair_count()
    }

    /// Recompute the affinity signal if stale for the current graph revision. Jaccard is cheap (like
    /// betweenness at current scale), so this stays inline; the revision gate avoids the per-frame
    /// redo. (Graph signals — P4.)
    pub(crate) fn ensure_affinity_fresh(&mut self) {
        self.channels
            .structural_affinity(&self.graph, AFFINITY_MIN_SIMILARITY);
    }

    /// Install / refresh / clear the affinity force to match the toggle + the current signal, once
    /// per real change (not per frame). Called from [`frame`](Self::frame). When on, recompute the
    /// signal if stale and (re)install the force only when the graph revision moved since the live
    /// force was built; when off, clear it once. Each install/clear is followed by a settle so the
    /// new equilibrium takes. (Graph signals — P4.)
    pub(crate) fn sync_affinity_force(&mut self) {
        if self.cluster_by_affinity {
            let revision = self.graph.structural_input_key();
            let has_content = self.content_affinity.is_some();
            // Which sources feed the force under the current blend mode. `ContentOnly` with no
            // injected signal falls back to structural, so the toggle always does something.
            let (use_structural, use_content) = match self.affinity_blend {
                AffinityBlend::StructuralOnly => (true, false),
                AffinityBlend::ContentOnly => (!has_content, has_content),
                AffinityBlend::Blend => (true, has_content),
            };
            // Structural is revision-gated (recompute the Jaccard memo if stale); content is
            // host-fresh (dirty-gated). Reinstall when a *contributing* source changed, or when
            // nothing is installed yet.
            if use_structural {
                self.ensure_affinity_fresh();
            }
            let structural_changed =
                use_structural && self.installed_affinity_revision != Some(revision);
            let content_changed = use_content && self.content_affinity_dirty;
            if self.affinity_force_installed && !structural_changed && !content_changed {
                // Nothing that feeds the live force moved; leave it (no per-frame rebuild).
            } else {
                let structural = use_structural
                    .then(|| {
                        self.channels
                            .structural_affinity_fresh(&self.graph, AFFINITY_MIN_SIMILARITY)
                    })
                    .flatten();
                let content = use_content
                    .then(|| self.content_affinity.as_deref())
                    .flatten();
                let pairs = blend_affinity_pairs(structural, content);
                let force = (!pairs.is_empty()).then(|| AffinitySpring::new(pairs));
                self.affinity_force_installed = force.is_some();
                self.physics.set_affinity_force(force);
                self.physics.settle(SETTLE_TICKS / 2);
                // Structural is what the revision gate tracks; content-only installs leave it `None`
                // so a later switch back to structural/blend reinstalls from the current revision.
                self.installed_affinity_revision = use_structural.then_some(revision);
                self.content_affinity_dirty = false;
            }
        } else if self.affinity_force_installed {
            self.physics.set_affinity_force(None);
            self.physics.settle(SETTLE_TICKS / 2);
            self.affinity_force_installed = false;
            self.installed_affinity_revision = None;
            // Re-arm the content signal so re-enabling the toggle reinstalls it (the dirty flag was
            // consumed at install; without this a toggle off→on would leave content uninstalled).
            self.content_affinity_dirty = true;
        }
    }
}
