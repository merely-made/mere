// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The physics catalog: `laws × overlays`, and the named profiles over them.
//!
//! A **law** is which dynamics the graph moves under — springs and charges,
//! stress over hop distances, gravity and orbits, particle-life kinds, a
//! flock, coupled oscillators, a magnetic field, annealing, or none. Not a
//! tuning of one force-directed rule but a different rule. An **overlay** is
//! one more pull or push composed onto any law (hub room, group pull, depth,
//! grid, a centre, a tide, the skeleton). A **profile** is a named (law,
//! overlays) pair; the donor Graphshell's ten presets return as the first
//! ten, and a bare profile per law puts every law one pick away.
//!
//! A **source** is where a law or overlay reads a node attribute from: the
//! Kinds law's kind and Group pull's groups (site, cluster, colouring, island,
//! degree, meaning), Orbit's mass and the hub overlays' weight (degree,
//! PageRank), the Depth overlay's depth (roots, layers, the focus). Sources
//! are tunables, not laws; each is a channel of the registry in
//! [`channels`](crate::canvas::channels).
//!
//! The canvas builds the seiche force set from the chosen law + overlays
//! against the current graph and hands it to the physics backend wholesale —
//! the coupling / affinity / anchor slots are separate and untouched. The
//! attribute builders run over a petgraph view of the *visible* edges
//! ([`LawInputs`]), so hidden relations relax the physics as they relax the
//! springs. The choice rides the saved scene by id. Sibling to the
//! arrangement catalog in [`cartography_scene`](crate::canvas::cartography_scene):
//! that is *where* nodes go, this is *how* they move.
//!
//! Plan: `design_docs/archive_docs/2026-10-28/2026-09-02_physics_catalog_plan.md`.

use std::collections::HashMap;

use kernel::graph::NodeKey;

use crate::canvas::law_inputs::{
    law_inputs_without_relation_edges, with_link_hides, with_meaning, with_occurrence_edges,
    with_registry,
};
use crate::canvas::{Canvas, SETTLE_TICKS};
use crate::signals::physics::PhysicsViewKey;

pub(crate) use seiche::catalog::{LawInputs, LawSources};
#[cfg(test)]
pub(crate) use seiche::catalog::{
    CHARGE_STRENGTH, LAW_SEED, ORBIT_CENTRING, ORBIT_EXCLUSION_REACH, density_law,
};
pub use seiche::catalog::{
    DENSITY_MAX_PASSES, DENSITY_MIN_PASSES, DENSITY_PATIENCE, DENSITY_RESOLUTION, DENSITY_STOP,
};

pub use seiche::{
    DENSITY_ADMITS, DENSITY_REFUSAL, OverlayRefusal, PhysicsDepthSource, PhysicsKindSource,
    PhysicsLaw, PhysicsMassSource, PhysicsOverlay,
};

/// What the layout looks like right now, in numbers a receipt can assert
/// on: the laws' signatures. (Physics catalog — P2.)
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LayoutStats {
    /// The node bodies' kinetic energy (live inline; the last snapshot's
    /// figure offloaded).
    pub energy: f32,
    /// Root-mean-square distance of the nodes from their centroid.
    pub spread: f32,
    /// Node pairs closer than a node body's diameter.
    pub overlaps: usize,
    /// Canvas distance between the two graph-farthest connected nodes,
    /// divided by the mean visible edge length — a metric layout (Stress)
    /// reads near the diameter in hops, a local one well under it. Zero
    /// without edges.
    pub stretch: f32,
    /// Spearman rank correlation between each node's mass (the mass source)
    /// and its area share (discrete Voronoi cell): Density's signature,
    /// near one when room follows mass. Zero under uniform mass.
    pub mass_area_rank: f32,
    /// Coefficient of variation of `mass / area share` over the nodes: how
    /// uneven the layout's density is, zero when perfectly even.
    pub density_cv: f32,
}

/// A named (law, overlays) pair: what a picker offers as one choice.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PhysicsProfile {
    pub id: &'static str,
    pub label: &'static str,
    pub law: PhysicsLaw,
    pub overlays: &'static [PhysicsOverlay],
}

/// The law catalog for the picker: `(id, label)`, every law.
pub const CANVAS_PHYSICS_LAWS: &[(&str, &str)] = &[
    ("spring.rapier", "Springs"),
    ("charge.barnes-hut", "Charge"),
    ("stress.kamada-kawai", "Stress"),
    ("energy.linlog", "Energy"),
    ("orbit.gravity", "Orbit"),
    ("kinds.particle-life", "Kinds"),
    ("flock.boids", "Flock"),
    ("sync.kuramoto", "Sync"),
    ("flow.magnetic", "Flow"),
    ("anneal.davidson-harel", "Anneal"),
    ("still.default", "Still"),
    ("density.gastner-newman", "Density"),
];

/// The overlay catalog for the toggles: `(id, label)`.
pub const CANVAS_PHYSICS_OVERLAYS: &[(&str, &str)] = &[
    ("degree-repulsion", "Hub room"),
    ("domain-cluster", "Group pull"),
    ("hub-gravity", "Hub pull"),
    ("depth-gravity", "Depth"),
    ("grid-snap", "Grid"),
    ("gravity-locus", "Centre"),
    ("tide", "Tide"),
    ("skeleton", "Skeleton"),
];

/// The kind-source catalog: `(id, label)`.
pub const CANVAS_PHYSICS_KIND_SOURCES: &[(&str, &str)] = &[
    ("site", "By site"),
    ("cluster", "By cluster"),
    ("coloring", "By colouring"),
    ("component", "By island"),
    ("degree", "By degree"),
    ("meaning", "By meaning"),
];

/// The mass-source catalog: `(id, label)`.
pub const CANVAS_PHYSICS_MASS_SOURCES: &[(&str, &str)] =
    &[("degree", "By degree"), ("pagerank", "By rank")];

/// The depth-source catalog: `(id, label)`.
pub const CANVAS_PHYSICS_DEPTH_SOURCES: &[(&str, &str)] = &[
    ("roots", "From roots"),
    ("layers", "By layer"),
    ("focus", "From focus"),
];

/// The profile catalog: the donor's ten presets first (each a law + overlays,
/// under the donor's own name), then one bare profile per law the ten do not
/// already offer bare — Gas *is* Charge alone, Magnet Flow alone, Void Still
/// alone, so those three are not repeated: every (law, overlays) pair names
/// exactly one profile, which is what lets the picker show the live choice.
pub const CANVAS_PHYSICS_PROFILES: &[PhysicsProfile] = &[
    PhysicsProfile {
        id: "liquid",
        label: "Liquid",
        law: PhysicsLaw::Springs,
        overlays: &[PhysicsOverlay::GravityLocus],
    },
    PhysicsProfile {
        id: "gas",
        label: "Gas",
        law: PhysicsLaw::Charge,
        overlays: &[],
    },
    PhysicsProfile {
        id: "solid",
        label: "Solid",
        law: PhysicsLaw::Springs,
        overlays: &[
            PhysicsOverlay::DomainCluster,
            PhysicsOverlay::DegreeRepulsion,
        ],
    },
    PhysicsProfile {
        id: "archipelago",
        label: "Archipelago",
        law: PhysicsLaw::Energy,
        overlays: &[
            PhysicsOverlay::DomainCluster,
            PhysicsOverlay::DegreeRepulsion,
        ],
    },
    PhysicsProfile {
        id: "constellation",
        label: "Constellation",
        law: PhysicsLaw::Springs,
        overlays: &[PhysicsOverlay::DegreeRepulsion, PhysicsOverlay::HubGravity],
    },
    PhysicsProfile {
        id: "crystal",
        label: "Crystal",
        law: PhysicsLaw::Stress,
        overlays: &[PhysicsOverlay::GridSnap],
    },
    PhysicsProfile {
        id: "tide",
        label: "Tide",
        law: PhysicsLaw::Springs,
        overlays: &[PhysicsOverlay::Tide],
    },
    PhysicsProfile {
        id: "sediment",
        label: "Sediment",
        law: PhysicsLaw::Springs,
        overlays: &[PhysicsOverlay::DepthGravity],
    },
    PhysicsProfile {
        id: "magnet",
        label: "Magnet",
        law: PhysicsLaw::Flow,
        overlays: &[],
    },
    PhysicsProfile {
        id: "void",
        label: "Void",
        law: PhysicsLaw::Still,
        overlays: &[],
    },
    PhysicsProfile {
        id: "law.springs",
        label: "Springs",
        law: PhysicsLaw::Springs,
        overlays: &[],
    },
    PhysicsProfile {
        id: "law.stress",
        label: "Stress",
        law: PhysicsLaw::Stress,
        overlays: &[],
    },
    PhysicsProfile {
        id: "law.energy",
        label: "Energy",
        law: PhysicsLaw::Energy,
        overlays: &[],
    },
    PhysicsProfile {
        id: "law.orbit",
        label: "Orbit",
        law: PhysicsLaw::Orbit,
        overlays: &[],
    },
    PhysicsProfile {
        id: "law.kinds",
        label: "Kinds",
        law: PhysicsLaw::Kinds,
        overlays: &[],
    },
    PhysicsProfile {
        id: "law.flock",
        label: "Flock",
        law: PhysicsLaw::Flock,
        overlays: &[],
    },
    PhysicsProfile {
        id: "law.sync",
        label: "Sync",
        law: PhysicsLaw::Sync,
        overlays: &[],
    },
    PhysicsProfile {
        id: "law.anneal",
        label: "Anneal",
        law: PhysicsLaw::Anneal,
        overlays: &[],
    },
    PhysicsProfile {
        id: "skeleton",
        label: "Skeleton",
        law: PhysicsLaw::Charge,
        overlays: &[PhysicsOverlay::Skeleton],
    },
    PhysicsProfile {
        id: "law.density",
        label: "Density",
        law: PhysicsLaw::Density,
        overlays: &[],
    },
];

/// The profile with this id, if any.
pub fn physics_profile(id: &str) -> Option<&'static PhysicsProfile> {
    CANVAS_PHYSICS_PROFILES
        .iter()
        .find(|profile| profile.id == id)
}

impl Canvas {
    pub(crate) fn controller_topology_reading(
        &self,
    ) -> Option<super::seiche_bridge::VisibleRelationTopology> {
        self.occurrence_edges.is_none().then(|| {
            super::seiche_bridge::visible_relation_topology_with_assertion_hides(
                &self.graph,
                &self.hidden_edges,
                &self.hidden_links,
                &self.hidden_assertions,
            )
        })
    }

    /// Exact current topology input submitted to this controller. Occurrence input
    /// is explicit transient controller state; otherwise rows come from the
    /// shared hide-filtered graph source reader.
    pub(crate) fn controller_topology(&self) -> seiche::EdgeTopology {
        self.controller_topology_reading()
            .map(|reading| reading.edge_topology())
            .unwrap_or_else(|| {
                self.occurrence_edges
                    .as_ref()
                    .expect("occurrence topology selected when no relation reading exists")
                    .iter()
                    .copied()
                    .collect()
            })
    }

    pub(crate) fn controller_edges(&self) -> Vec<(NodeKey, NodeKey)> {
        self.controller_topology().iter().copied().collect()
    }
    /// Whether the live law or an overlay snapshots graph structure, so a
    /// topology change must rebuild it.
    pub(crate) fn physics_forces_are_graph_bound(&self) -> bool {
        self.physics_composition.is_some()
            || self.physics_law.graph_bound()
            || self.physics_overlays.iter().any(|o| o.graph_bound())
            || (self.physics_mass_source == PhysicsMassSource::PageRank
                && self.physics_overlays.iter().any(|o| o.weighted()))
    }

    /// Whether the live law or an overlay keeps the graph moving on its own.
    pub fn physics_never_rests(&self) -> bool {
        if self.roles.encoded == seiche::Axes::BOTH {
            return false;
        }
        let laws = self.physics_composition.as_ref().map_or_else(
            || vec![self.physics_law],
            crate::canvas::composition::PhysicsComposition::laws,
        );
        laws.iter().any(|law| law.never_rests())
            || self.physics_overlays.iter().any(|o| o.never_rests())
    }

    /// The sources the next build reads through.
    fn law_sources(&self) -> LawSources {
        LawSources {
            kind: self.physics_kind_source,
            groups: self.physics_group_source,
            mass: self.physics_mass_source,
            depth: self.physics_depth_source,
            focus: self.focused_key(),
            seed: self.dynamics.seed,
        }
    }

    /// Rebuild the law + overlay force set against the current graph and hand it
    /// to the physics backend, as boxed forces in-thread and as a descriptor
    /// otherwise. Position-preserving.
    pub(crate) fn rebuild_law_forces(&mut self) {
        #[cfg(test)]
        {
            self.law_rebuilds += 1;
        }
        if self.roles.encoded == seiche::Axes::BOTH {
            // F28: two directly encoded axes admit contact separation only.
            self.physics
                .set_forces(vec![Box::new(seiche::NodeExclusion {
                    cutoff: 2.0 * seiche::NODE_BODY_RADIUS,
                    ..Default::default()
                })]);
            return;
        }
        let wants_clusters = self.physics_reads(PhysicsKindSource::Cluster);
        if wants_clusters {
            self.ensure_community_fresh();
        }
        if self.physics_reads(PhysicsKindSource::Meaning) {
            self.refresh_meaning();
        }
        let sources = self.law_sources();
        // The site channel from the registry, once per structure and URL grouping.
        self.channels.sites(&self.graph);
        // An in-thread backend builds the forces here, reading the physics
        // channels through the registry, which caches one computation per view
        // key. Any other backend is sent the law as data and computes the
        // channels on its own side, uncached: a descriptor would give up the
        // cache on the in-thread path, so that path stays on boxed forces
        // (Graphshell host plan, T7, step 2b milestone 3).
        let inline = self.physics.runs_inline();
        {
            let inputs = law_inputs_without_relation_edges(
                &self.graph,
                if wants_clusters {
                    self.channels.community_held()
                } else {
                    None
                },
                self.channels.sites_fresh(&self.graph),
            );
            let inputs = with_link_hides(
                inputs,
                &self.graph,
                &self.hidden_edges,
                &self.hidden_links,
                &self.hidden_assertions,
            );
            let inputs = with_occurrence_edges(inputs, self.occurrence_edges.as_deref());
            let inputs = with_meaning(inputs, self.meaning.snapshot());
            if inline {
                let inputs = with_registry(inputs, &self.channels, self.physics_view_key());
                let forces = self.composed_forces(&inputs, sources);
                self.physics.set_forces(forces);
            } else {
                let descriptor = inputs.describe(
                    self.physics_law,
                    self.physics_composition.as_ref(),
                    &self.physics_overlays,
                    sources,
                );
                self.physics.set_force_set(descriptor);
            }
        }
        #[cfg(test)]
        if self.physics_reads(PhysicsKindSource::Meaning) {
            self.meaning.built_from = self.meaning.snapshot().map(|s| s.run);
        }
    }

    /// Whether the live law or overlays read `source` through the kind or the
    /// groups channel.
    pub(crate) fn physics_reads(&self, source: PhysicsKindSource) -> bool {
        use super::composition::{GroupSource, PhysicsComposition};
        (self.physics_law == PhysicsLaw::Kinds && self.physics_kind_source == source)
            || (self.physics_group_source == source
                && self
                    .physics_overlays
                    .contains(&PhysicsOverlay::DomainCluster))
            || matches!(
                &self.physics_composition,
                Some(PhysicsComposition::Grouped(grouping))
                    if grouping.groups == GroupSource::Channel(source)
            )
    }

    /// The settle a law switch earns: a living law runs until paused, the rest
    /// settle for the usual budget.
    pub(crate) fn settle_for_law(&mut self) {
        if self.physics_never_rests() {
            self.settle_physics(u32::MAX);
        } else {
            self.settle_physics(SETTLE_TICKS);
        }
    }

    /// The node bodies' kinetic energy right now (inline backend; zero offloaded).
    pub fn physics_energy(&self) -> f32 {
        self.physics.kinetic_energy()
    }

    /// The layout's signature numbers: energy, spread, overlaps, stretch.
    pub fn layout_stats(&self) -> LayoutStats {
        let positions: Vec<(NodeKey, euclid::default::Point2D<f32>)> =
            self.view.positions().collect();
        let n = positions.len();
        if n == 0 {
            return LayoutStats {
                energy: self.physics_energy(),
                ..LayoutStats::default()
            };
        }
        let centroid = positions
            .iter()
            .fold(euclid::default::Vector2D::<f32>::zero(), |acc, (_, p)| {
                acc + p.to_vector()
            })
            / n as f32;
        let spread = (positions
            .iter()
            .map(|(_, p)| (p.to_vector() - centroid).square_length())
            .sum::<f32>()
            / n as f32)
            .sqrt();
        let diameter = 2.0 * crate::canvas::NODE_HALF;
        let mut overlaps = 0;
        for i in 0..n {
            for j in (i + 1)..n {
                if (positions[i].1 - positions[j].1).length() < diameter {
                    overlaps += 1;
                }
            }
        }
        let (mass_area_rank, density_cv) = {
            let masses: HashMap<NodeKey, f32> = self
                .law_inputs_now()
                .masses(self.physics_mass_source)
                .into_iter()
                .collect();
            let points: Vec<(f32, f32)> = positions.iter().map(|(_, p)| (p.x, p.y)).collect();
            let weights: Vec<f32> = positions
                .iter()
                .map(|(k, _)| masses.get(k).copied().unwrap_or(1.0))
                .collect();
            crate::canvas::area_share::mass_area_stats(&points, &weights)
        };
        let at: HashMap<NodeKey, euclid::default::Point2D<f32>> = positions.into_iter().collect();
        let edges = self.controller_edges();
        let lengths: Vec<f32> = edges
            .iter()
            .filter_map(|(a, b)| Some((*at.get(a)? - *at.get(b)?).length()))
            .collect();
        let stretch = if lengths.is_empty() {
            0.0
        } else {
            let mean_edge = lengths.iter().sum::<f32>() / lengths.len() as f32;
            let keys: Vec<NodeKey> = at.keys().copied().collect();
            // The graph-farthest connected pair, by BFS from every node.
            let mut farthest: Option<(NodeKey, NodeKey, u32)> = None;
            for (a, b, hops) in seiche::graph_distances(keys.iter().copied(), &edges) {
                if farthest.is_none_or(|(_, _, best)| hops > best) {
                    farthest = Some((a, b, hops));
                }
            }
            match farthest {
                Some((a, b, _)) if mean_edge > 0.0 => (at[&a] - at[&b]).length() / mean_edge,
                _ => 0.0,
            }
        };
        LayoutStats {
            energy: self.physics_energy(),
            spread,
            overlaps,
            stretch,
            mass_area_rank,
            density_cv,
        }
    }

    /// The attribute builders over the current graph and visible edges,
    /// the physics channels read from the registry.
    fn law_inputs_now(&self) -> LawInputs<'_> {
        let inputs = law_inputs_without_relation_edges(&self.graph, None, None);
        let inputs = with_link_hides(
            inputs,
            &self.graph,
            &self.hidden_edges,
            &self.hidden_links,
            &self.hidden_assertions,
        );
        let inputs = with_occurrence_edges(inputs, self.occurrence_edges.as_deref());
        with_registry(inputs, &self.channels, self.physics_view_key())
    }

    /// The physics view's key: the graph's structure and the view revision
    /// (F178).
    pub(crate) fn physics_view_key(&self) -> PhysicsViewKey {
        PhysicsViewKey {
            structure: self.graph.structural_input_key(),
            view: self.physics_view_revision,
        }
    }

    /// The number of forces in the live law slot (inline backend only). Test introspection.
    #[cfg(test)]
    pub(crate) fn law_force_count(&self) -> usize {
        self.physics.force_count()
    }

    /// How many force-set rebuilds have run. Test introspection.
    #[cfg(test)]
    pub(crate) fn law_rebuilds(&self) -> usize {
        self.law_rebuilds
    }

    /// The attribute builders over the current graph. Test introspection.
    #[cfg(test)]
    pub(crate) fn law_inputs(&self) -> LawInputs<'_> {
        self.law_inputs_now()
    }

    /// [`Self::layout_stats`] without `stretch` (zero here), for graphs too
    /// big for its all-pairs passes: overlaps by the same definition, found
    /// through a uniform grid of diameter-wide cells, so the cost is linear.
    pub fn layout_stats_without_stretch(&self) -> LayoutStats {
        let positions: Vec<euclid::default::Point2D<f32>> =
            self.view.positions().map(|(_, p)| p).collect();
        let n = positions.len();
        if n == 0 {
            return LayoutStats {
                energy: self.physics_energy(),
                ..LayoutStats::default()
            };
        }
        let centroid = positions
            .iter()
            .fold(euclid::default::Vector2D::<f32>::zero(), |acc, p| {
                acc + p.to_vector()
            })
            / n as f32;
        let spread = (positions
            .iter()
            .map(|p| (p.to_vector() - centroid).square_length())
            .sum::<f32>()
            / n as f32)
            .sqrt();
        let diameter = 2.0 * crate::canvas::NODE_HALF;
        let cell_of = |p: &euclid::default::Point2D<f32>| {
            (
                (p.x / diameter).floor() as i64,
                (p.y / diameter).floor() as i64,
            )
        };
        let mut grid: HashMap<(i64, i64), Vec<usize>> = HashMap::new();
        for (i, p) in positions.iter().enumerate() {
            grid.entry(cell_of(p)).or_default().push(i);
        }
        let mut overlaps = 0;
        for (i, p) in positions.iter().enumerate() {
            let (cx, cy) = cell_of(p);
            for dx in -1..=1 {
                for dy in -1..=1 {
                    for &j in grid.get(&(cx + dx, cy + dy)).into_iter().flatten() {
                        if j > i && (positions[j] - *p).length() < diameter {
                            overlaps += 1;
                        }
                    }
                }
            }
        }
        LayoutStats {
            energy: self.physics_energy(),
            spread,
            overlaps,
            stretch: 0.0,
            // Mass against area is all-pairs as well: zero here.
            ..LayoutStats::default()
        }
    }
}
