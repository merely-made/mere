// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Cartography geometry — the canvas's settled-layout read surface.
//!
//! Moved from platen's `projection_geometry` in the 2026-07-09 platen
//! decomposition: this is graph-scene material (member-keyed world positions
//! plus the per-member representation overrides), so it lives with the canvas;
//! platen keeps the Tree (pane) geometry. The host persists it as the
//! `arrangement.*` facet family (pandect `arrangement_facets`, in
//! `facets.json`) — the planned bespoke sidecar file was obviated by the facet
//! convergence before any host wired it.

use forme::GraphMemberId;
use serde::{Deserialize, Serialize};
/// The geometry of the **Cartography** projection (the canvas): each member's world
/// position, member-keyed. The cartography counterpart of [`TreeGeometry`] for the
/// Tree projection — "canonical world positions (cartography)" per the composition
/// spine (§9). Positions are *world* coordinates (the semantic geometry), not
/// pixels, so they render responsively at any zoom.
///
/// The save-time snapshot of the canvas's *settled* layout: the live positions
/// live in the seiche read model and the kernel graph carries none at all, so
/// without persisting this a session's layout is lost on reload. The host
/// persists it family-by-family as `arrangement.*` facets (pandect
/// `arrangement_facets`), one facet id per field group below.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct CartographyGeometry {
    positions: Vec<(GraphMemberId, (f32, f32))>,
    /// Per-member face-size overrides (px) — the sizing counterpart of the positions, for
    /// members the user sized explicitly. A member absent here falls back to size-by-degree
    /// or the default at render time, so only deliberate overrides persist. Serde-defaulted,
    /// so a pre-sizing sidecar still loads. (Node-rep — size persistence.)
    #[serde(default)]
    sizes: Vec<(GraphMemberId, f32)>,
    /// Whether the scene's size-by-degree mode (faces grow with undirected degree) was on at
    /// save time. Serde-defaulted to `false`. (Node-rep — size persistence.)
    #[serde(default)]
    size_by_degree: bool,
    /// Whether the scene's **size-by-importance** mode (faces grow with the graph-signals
    /// importance) was on at save time. Serde-defaulted to `false`. (Graph signals.)
    #[serde(default)]
    size_by_importance: bool,
    /// The importance **metric** code (`degree` / `betweenness`) size-by-importance read at save
    /// time, so a betweenness-sized scene re-opens betweenness-sized. Stored as a string to keep
    /// this crate signals-free; the canvas maps it to/from `ImportanceMetric`. Serde-defaulted to
    /// the empty string, which the canvas's `from_code` reads as `degree` (a pre-metric sidecar
    /// loads as degree). (Graph signals — metric persistence.)
    #[serde(default)]
    importance_metric: String,
    /// Per-member sprite faces: the imported image as a PNG data-URI, for members the user
    /// gave a custom face. A member absent here has no sprite (its content-sensitive default
    /// applies). Serde-defaulted, so a pre-sprite sidecar still loads. The data-URIs
    /// add bulk to the sidecar — acceptable for a handful of faces (a future optimization
    /// could externalize the blobs). (Node-rep — sprite persistence.)
    #[serde(default)]
    sprites: Vec<(GraphMemberId, String)>,
    /// Per-member sprite collider hulls: the opaque region as a face-normalized convex
    /// polygon ([-0.5, 0.5]), persisted beside the sprite so the traced-to-image collider
    /// survives a reload without re-decoding the image. Serde-defaulted. (Node-rep — sprite hull.)
    #[serde(default)]
    sprite_hulls: Vec<(GraphMemberId, Vec<(f32, f32)>)>,
    /// Per-member physical material overrides `(restitution, friction, density)` on the Body
    /// axis, so a node tuned heavier / bouncier / grippier re-opens that way. Stored as a plain
    /// tuple to keep this crate seiche-free; the canvas maps it to/from `seiche::NodeMaterial`.
    /// Serde-defaulted. (Node body & face — material.)
    #[serde(default)]
    materials: Vec<(GraphMemberId, (f32, f32, f32))>,
    /// Per-member **face** overrides on the Face axis, as a string code (`favicon` / `derived` /
    /// `sprite` / `bare`), so a node's chosen texture re-opens that way. Stored as a string to keep this
    /// crate canvas-free; the canvas maps it to/from `crate::canvas::Face`. Serde-defaulted. (Node body
    /// & face — face persistence.)
    #[serde(default)]
    faces: Vec<(GraphMemberId, String)>,
    /// The saved seed (graph grammar, *Reset and seed*): `None` resets to the
    /// recipe seed. Serde-defaulted.
    #[serde(default)]
    seed: Option<Vec<(GraphMemberId, (f32, f32))>>,
    /// The regions the active arrangement drew (Partition, Sets, Mosaic), so a
    /// restored scene draws them without re-projecting. Serde-defaulted.
    #[serde(default)]
    regions: Vec<SavedRegion>,
    /// The person's arrangement of the Mosaic, if they made one. Serde-defaulted.
    #[serde(default)]
    mosaic: Option<SavedMosaic>,
}

/// The person's Mosaic arrangement by member: tiles moved out of their site's
/// group, the group order, the member order and the size factors.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct SavedMosaic {
    pub group_of: Vec<(GraphMemberId, String)>,
    pub group_order: Vec<String>,
    pub member_order: Vec<GraphMemberId>,
    pub scale: Vec<(GraphMemberId, f32)>,
}

/// One region an arrangement drew, in world coordinates, its members by id.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct SavedRegion {
    pub contour: Vec<(f32, f32)>,
    pub label: Option<String>,
    pub members: Vec<GraphMemberId>,
}

impl CartographyGeometry {
    /// Collect a member-keyed position set (e.g. the canvas's live node positions).
    pub fn from_positions(
        positions: impl IntoIterator<Item = (GraphMemberId, (f32, f32))>,
    ) -> Self {
        Self {
            positions: positions.into_iter().collect(),
            ..Self::default()
        }
    }

    /// Attach per-member size overrides (chainable after [`from_positions`]).
    /// (Node-rep — size persistence.)
    pub fn with_sizes(mut self, sizes: impl IntoIterator<Item = (GraphMemberId, f32)>) -> Self {
        self.sizes = sizes.into_iter().collect();
        self
    }

    /// Set the scene's size-by-importance flag (chainable). (Graph signals.)
    pub fn with_size_by_importance(mut self, on: bool) -> Self {
        self.size_by_importance = on;
        self
    }

    /// Set the scene's importance-metric code (`degree` / `betweenness`) (chainable).
    /// (Graph signals — metric persistence.)
    pub fn with_importance_metric(mut self, code: impl Into<String>) -> Self {
        self.importance_metric = code.into();
        self
    }

    /// Set the scene's size-by-degree flag (chainable). (Node-rep — size persistence.)
    pub fn with_size_by_degree(mut self, on: bool) -> Self {
        self.size_by_degree = on;
        self
    }

    /// Attach per-member sprite faces (chainable). (Node-rep — sprite persistence.)
    pub fn with_sprites(
        mut self,
        sprites: impl IntoIterator<Item = (GraphMemberId, String)>,
    ) -> Self {
        self.sprites = sprites.into_iter().collect();
        self
    }

    /// Attach per-member sprite collider hulls (chainable). (Node-rep — sprite hull.)
    pub fn with_sprite_hulls(
        mut self,
        hulls: impl IntoIterator<Item = (GraphMemberId, Vec<(f32, f32)>)>,
    ) -> Self {
        self.sprite_hulls = hulls.into_iter().collect();
        self
    }

    /// Attach per-member physical material overrides `(restitution, friction, density)`
    /// (chainable). (Node body & face — material.)
    pub fn with_materials(
        mut self,
        materials: impl IntoIterator<Item = (GraphMemberId, (f32, f32, f32))>,
    ) -> Self {
        self.materials = materials.into_iter().collect();
        self
    }

    /// Attach per-member face overrides (string codes `favicon` / `derived` / `sprite` / `bare`)
    /// (chainable). (Node body & face — face persistence.)
    pub fn with_faces(mut self, faces: impl IntoIterator<Item = (GraphMemberId, String)>) -> Self {
        self.faces = faces.into_iter().collect();
        self
    }

    /// The `(member, (x, y))` pairs, in insertion order.
    pub fn iter(&self) -> impl Iterator<Item = (GraphMemberId, (f32, f32))> + '_ {
        self.positions.iter().copied()
    }

    /// The `(member, size)` overrides, in insertion order. (Node-rep — size persistence.)
    pub fn size_iter(&self) -> impl Iterator<Item = (GraphMemberId, f32)> + '_ {
        self.sizes.iter().copied()
    }

    /// Whether size-by-degree was on at save time. (Node-rep — size persistence.)
    pub fn size_by_degree(&self) -> bool {
        self.size_by_degree
    }

    /// Whether size-by-importance was on at save time. (Graph signals.)
    pub fn size_by_importance(&self) -> bool {
        self.size_by_importance
    }

    /// The importance-metric code (`degree` / `betweenness`, or empty for a pre-metric sidecar)
    /// at save time. (Graph signals — metric persistence.)
    pub fn importance_metric(&self) -> &str {
        &self.importance_metric
    }

    /// The `(member, data-URI)` sprite faces, in insertion order. (Node-rep — sprite persistence.)
    pub fn sprite_iter(&self) -> impl Iterator<Item = (GraphMemberId, &str)> + '_ {
        self.sprites.iter().map(|(m, uri)| (*m, uri.as_str()))
    }

    /// The `(member, hull)` sprite collider hulls, in insertion order (owned clones, for the
    /// host to apply via `apply_cartography_sprite_hulls`). (Node-rep — sprite hull.)
    pub fn sprite_hull_iter(&self) -> impl Iterator<Item = (GraphMemberId, Vec<(f32, f32)>)> + '_ {
        self.sprite_hulls.iter().map(|(m, h)| (*m, h.clone()))
    }

    /// The `(member, (restitution, friction, density))` material overrides, in insertion order
    /// (for the host to apply via `apply_cartography_materials`). (Node body & face — material.)
    pub fn material_iter(&self) -> impl Iterator<Item = (GraphMemberId, (f32, f32, f32))> + '_ {
        self.materials.iter().map(|(m, mat)| (*m, *mat))
    }

    /// The `(member, code)` face overrides, in insertion order (for the host to apply via
    /// `apply_cartography_faces`). (Node body & face — face persistence.)
    pub fn face_iter(&self) -> impl Iterator<Item = (GraphMemberId, &str)> + '_ {
        self.faces.iter().map(|(m, code)| (*m, code.as_str()))
    }

    /// Attach the saved seed (chainable); `None` is the recipe seed.
    pub fn with_seed(
        mut self,
        seed: Option<impl IntoIterator<Item = (GraphMemberId, (f32, f32))>>,
    ) -> Self {
        self.seed = seed.map(|seed| seed.into_iter().collect());
        self
    }

    pub fn seed(&self) -> Option<&[(GraphMemberId, (f32, f32))]> {
        self.seed.as_deref()
    }

    /// Attach the arrangement's regions (chainable).
    pub fn with_regions(mut self, regions: impl IntoIterator<Item = SavedRegion>) -> Self {
        self.regions = regions.into_iter().collect();
        self
    }

    pub fn regions(&self) -> &[SavedRegion] {
        &self.regions
    }

    /// Attach the Mosaic arrangement (chainable).
    pub fn with_mosaic(mut self, mosaic: Option<SavedMosaic>) -> Self {
        self.mosaic = mosaic;
        self
    }

    pub fn mosaic(&self) -> Option<&SavedMosaic> {
        self.mosaic.as_ref()
    }

    /// [`Self::retain_members`], chainable.
    pub fn retaining_members(mut self, keep: impl Fn(&GraphMemberId) -> bool) -> Self {
        self.retain_members(keep);
        self
    }

    /// Keep only what belongs to members `keep` admits, across every
    /// member-keyed field; a region keeps its contour with its kept members.
    pub fn retain_members(&mut self, keep: impl Fn(&GraphMemberId) -> bool) {
        self.positions.retain(|(id, _)| keep(id));
        self.sizes.retain(|(id, _)| keep(id));
        self.sprites.retain(|(id, _)| keep(id));
        self.sprite_hulls.retain(|(id, _)| keep(id));
        self.materials.retain(|(id, _)| keep(id));
        self.faces.retain(|(id, _)| keep(id));
        if let Some(seed) = self.seed.as_mut() {
            seed.retain(|(id, _)| keep(id));
        }
        for region in &mut self.regions {
            region.members.retain(|id| keep(id));
        }
        if let Some(mosaic) = self.mosaic.as_mut() {
            mosaic.group_of.retain(|(id, _)| keep(id));
            mosaic.member_order.retain(|id| keep(id));
            mosaic.scale.retain(|(id, _)| keep(id));
        }
    }

    pub fn is_empty(&self) -> bool {
        self.positions.is_empty()
    }

    /// The members carrying a position.
    pub fn members(&self) -> Vec<GraphMemberId> {
        self.positions.iter().map(|(m, _)| *m).collect()
    }

    // `to_persisted_json` / `from_persisted_json` — the bespoke-sidecar half —
    // left with the facet convergence: no host ever wired the cartography.json
    // sidecar, and the durable arrangement now persists as `arrangement.*`
    // facets (pandect `arrangement_facets`). This type remains as the
    // canvas's save-time read surface (`Canvas::cartography_geometry`).
}
