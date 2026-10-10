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
//! Plan: `design_docs/mere_docs/implementation_strategy/2026-09-02_physics_catalog_plan.md`.

use std::collections::{HashMap, HashSet};

use kernel::graph::{Graph, NodeKey};
use seiche::{
    Anneal, BarnesHutRepulsion, Boids, Boundary, CounterDamping, DegreeRepulsion, Density,
    DepthGravity, DomainCluster, EdgeSpring, Force, Gravity, GravityLocus, GridSnap, Hold,
    HubGravity, Kuramoto, LinLogForce, MagneticSpring, NodeExclusion, ParticleLife, StressSpring,
};

use crate::canvas::seiche_bridge::visible_relation_edges;
use crate::canvas::{Canvas, SETTLE_TICKS};
use crate::signals::ChannelRegistry;
use crate::signals::physics::{self, PhysicsView, PhysicsViewKey};

/// The seed every seeded law (Kinds' rule matrix, Anneal's walk) starts from,
/// so a scene reopens to the same rules.
pub(crate) const LAW_SEED: u64 = 0x5EED_CA7A_1064;
/// The Skeleton overlay's tree-edge stiffness, against `EdgeSpring`'s 10.
const SKELETON_STIFFNESS: f32 = 60.0;
/// Charge's repulsion, calibrated so that at contact (a node diameter, 36)
/// the `1/d` push matches `NodeExclusion`'s inverse-square one
/// (`220_000 / 36² ≈ 170`): the seiche default of 2 400 left bodies
/// touching under the edge springs. (Physics catalog — the Charge receipt.)
pub(crate) const CHARGE_STRENGTH: f32 = 6_000.0;
/// Orbit's exclusion reaches two node diameters: a guard at contact, where
/// Springs' reach of 1,000 outweighed gravitation at every range inside it
/// and threw the graph apart. Orbit's centring is a weak well that holds what
/// the orbits spread. (Ruled 2026-10-04, "Frictionless orbits + centring".)
pub(crate) const ORBIT_EXCLUSION_REACH: f32 = 4.0 * seiche::NODE_BODY_RADIUS;
pub(crate) const ORBIT_CENTRING: f32 = 0.02;
/// Density's grid resolution on the CPU tier (cells per side): 64², ruled
/// 2026-10-02 (the same ranks as 128² at about a sixteenth of the cost).
pub const DENSITY_RESOLUTION: usize = 64;
/// Density's convergence test (ruled 2026-10-02): passes end once the mean
/// node shift stays under a twentieth of a spacing for three passes in a
/// row; the pass cap ends them if the test never does.
pub const DENSITY_STOP: seiche::DensityStop = seiche::DensityStop::Shift(0.05);
pub const DENSITY_PATIENCE: u32 = 3;
pub const DENSITY_MAX_PASSES: u32 = 120;
/// The fewest passes before the stop test may end them (ruled 2026-10-03,
/// "Min 60, bar: all >= 0.7").
pub const DENSITY_MIN_PASSES: u32 = 60;

/// The physics law: which dynamics the graph moves under. Ids are technical
/// (`family.method`), labels plain, as the arrangement catalog does it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PhysicsLaw {
    /// Exclusion + edge springs + a centering boundary: the force-directed default.
    Springs,
    /// Barnes–Hut charge repulsion + edge springs: every body repels every other,
    /// distant ones by their cell's centre of mass.
    Charge,
    /// Kamada–Kawai stress: a spring between every connected pair at its
    /// shortest-path distance (relation multiplicity shortens a hop), so the
    /// picture is a metric map of the graph.
    Stress,
    /// ForceAtlas2's force model: attraction linear in distance along edges,
    /// repulsion falling as `1/d`, degree-weighted, so communities separate
    /// and hubs sit central. LinLog proper is its attraction exponent `0`, a
    /// tuning; the id stays `energy.linlog`.
    Energy,
    /// Newtonian gravity with an orbital kick: hubs are suns, leaves circle them,
    /// and it never rests.
    Orbit,
    /// Particle life: nodes carry a kind, and a kind-by-kind rule matrix says who
    /// chases and who flees, and it never rests.
    Kinds,
    /// Boids: separation, alignment, cohesion, and a cruising speed; the graph
    /// moves as a flock.
    Flock,
    /// Kuramoto oscillators: each node has a phase, edges couple phases, and
    /// position follows phase on a ring, so communities become phase clusters.
    Sync,
    /// Magnetic springs: edges align to a field direction, so a directed graph
    /// reads as a flow.
    Flow,
    /// Davidson–Harel simulated annealing: a random walk over a layout energy,
    /// cooling to a minimum.
    Anneal,
    /// No law: bodies hold where the arrangement (or a hand) put them
    /// (velocity zeroed each tick, so contacts can only nudge).
    Still,
    /// Gastner–Newman density equalizing: nodes flow down the gradient of a
    /// diffused mass density until it is even, so room follows mass.
    Density,
}

impl PhysicsLaw {
    pub const ALL: [PhysicsLaw; 12] = [
        PhysicsLaw::Springs,
        PhysicsLaw::Charge,
        PhysicsLaw::Stress,
        PhysicsLaw::Energy,
        PhysicsLaw::Orbit,
        PhysicsLaw::Kinds,
        PhysicsLaw::Flock,
        PhysicsLaw::Sync,
        PhysicsLaw::Flow,
        PhysicsLaw::Anneal,
        PhysicsLaw::Still,
        PhysicsLaw::Density,
    ];

    pub fn id(self) -> &'static str {
        match self {
            PhysicsLaw::Springs => "spring.rapier",
            PhysicsLaw::Charge => "charge.barnes-hut",
            PhysicsLaw::Stress => "stress.kamada-kawai",
            PhysicsLaw::Energy => "energy.linlog",
            PhysicsLaw::Orbit => "orbit.gravity",
            PhysicsLaw::Kinds => "kinds.particle-life",
            PhysicsLaw::Flock => "flock.boids",
            PhysicsLaw::Sync => "sync.kuramoto",
            PhysicsLaw::Flow => "flow.magnetic",
            PhysicsLaw::Anneal => "anneal.davidson-harel",
            PhysicsLaw::Still => "still.default",
            PhysicsLaw::Density => "density.gastner-newman",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            PhysicsLaw::Springs => "Springs",
            PhysicsLaw::Charge => "Charge",
            PhysicsLaw::Stress => "Stress",
            PhysicsLaw::Energy => "Energy",
            PhysicsLaw::Orbit => "Orbit",
            PhysicsLaw::Kinds => "Kinds",
            PhysicsLaw::Flock => "Flock",
            PhysicsLaw::Sync => "Sync",
            PhysicsLaw::Flow => "Flow",
            PhysicsLaw::Anneal => "Anneal",
            PhysicsLaw::Still => "Still",
            PhysicsLaw::Density => "Density",
        }
    }

    pub fn parse(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|law| law.id() == id)
    }

    /// Whether the law is a living display that never comes to rest (Orbit,
    /// Kinds, Flock, Sync), so the host keeps ticking rather than settling.
    /// Kinds joined on the P2 fixture's figures: kinetic energy about 18 300
    /// at 6 s and 140 500 at 30 s under continuous ticking, against the floor
    /// of 1 (dynamics grammar plan, G1, F10).
    pub fn never_rests(self) -> bool {
        matches!(
            self,
            PhysicsLaw::Orbit | PhysicsLaw::Kinds | PhysicsLaw::Flock | PhysicsLaw::Sync
        )
    }

    /// Whether the law snapshots graph structure at build (and so is rebuilt on
    /// a topology change): Stress's distances, Orbit's and Density's masses,
    /// Kinds' kinds.
    pub fn graph_bound(self) -> bool {
        matches!(
            self,
            PhysicsLaw::Stress | PhysicsLaw::Orbit | PhysicsLaw::Kinds | PhysicsLaw::Density
        )
    }

    /// Whether the law reads the mass source (and so is rebuilt when it changes).
    pub fn weighted(self) -> bool {
        matches!(self, PhysicsLaw::Orbit | PhysicsLaw::Density)
    }

    /// Why the law refuses `overlay`, if it does: what its currency refuses
    /// ([`Self::admits`]), and on Density every overlay but the two it takes
    /// converted (F73).
    pub fn refuses(self, overlay: PhysicsOverlay) -> Option<&'static str> {
        self.admits(overlay).refusal()
    }
}

/// The overlays Density takes, converted into its flow: the three F73
/// named ("Hub room, Centre, Tide if they hold"), against Density's own bar
/// ("Min 60, bar: all >= 0.7") at all sixteen dealt starts of both bar
/// graphs. Hub room's lowest start reads 0.808 (gen-50) and 0.765 (gen-200),
/// Centre's 0.704 and 0.735, and Tide's 0.699 and 0.736, admitted with them
/// (F81, "Admit Tide too").
pub(crate) const DENSITY_ADMITS: &[PhysicsOverlay] = &[
    PhysicsOverlay::DegreeRepulsion,
    PhysicsOverlay::GravityLocus,
    PhysicsOverlay::Tide,
];

/// Why Density refuses the rest.
pub(crate) const DENSITY_REFUSAL: &str = "Density takes only Hub room, Centre and Tide: converted \
                                          into its flow, the other overlays keep room from \
                                          following mass.";

/// Overlays a law refused: which, and why. The law itself was applied.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OverlayRefusal {
    pub law: PhysicsLaw,
    pub refused: Vec<PhysicsOverlay>,
    pub reason: &'static str,
}

impl std::fmt::Display for OverlayRefusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.reason)
    }
}

/// An overlay: one extra force composed onto any law.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PhysicsOverlay {
    /// Hubs push their surroundings apart, by weight.
    DegreeRepulsion,
    /// Nodes drift toward the centroid of their group (by the groups channel,
    /// site by default).
    DomainCluster,
    /// Everything is drawn toward the hubs, by weight.
    HubGravity,
    /// Depth drives the vertical: roots up, leaves down.
    DepthGravity,
    /// A spring to the nearest grid point.
    GridSnap,
    /// A gentle pull toward the canvas centre.
    GravityLocus,
    /// A centre that rides a slow sine, so the graph never fully settles.
    Tide,
    /// The spanning tree's edges held stiff, so the backbone shows under any law.
    Skeleton,
}

impl PhysicsOverlay {
    pub const ALL: [PhysicsOverlay; 8] = [
        PhysicsOverlay::DegreeRepulsion,
        PhysicsOverlay::DomainCluster,
        PhysicsOverlay::HubGravity,
        PhysicsOverlay::DepthGravity,
        PhysicsOverlay::GridSnap,
        PhysicsOverlay::GravityLocus,
        PhysicsOverlay::Tide,
        PhysicsOverlay::Skeleton,
    ];

    pub fn id(self) -> &'static str {
        match self {
            PhysicsOverlay::DegreeRepulsion => "degree-repulsion",
            PhysicsOverlay::DomainCluster => "domain-cluster",
            PhysicsOverlay::HubGravity => "hub-gravity",
            PhysicsOverlay::DepthGravity => "depth-gravity",
            PhysicsOverlay::GridSnap => "grid-snap",
            PhysicsOverlay::GravityLocus => "gravity-locus",
            PhysicsOverlay::Tide => "tide",
            PhysicsOverlay::Skeleton => "skeleton",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            PhysicsOverlay::DegreeRepulsion => "Hub room",
            PhysicsOverlay::DomainCluster => "Group pull",
            PhysicsOverlay::HubGravity => "Hub pull",
            PhysicsOverlay::DepthGravity => "Depth",
            PhysicsOverlay::GridSnap => "Grid",
            PhysicsOverlay::GravityLocus => "Centre",
            PhysicsOverlay::Tide => "Tide",
            PhysicsOverlay::Skeleton => "Skeleton",
        }
    }

    pub fn parse(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|overlay| overlay.id() == id)
    }

    /// Whether the overlay snapshots graph structure at build (the grouping,
    /// the depths, the tree), and so is rebuilt on a topology change.
    pub fn graph_bound(self) -> bool {
        matches!(
            self,
            PhysicsOverlay::DomainCluster | PhysicsOverlay::DepthGravity | PhysicsOverlay::Skeleton
        )
    }

    /// Whether the overlay reads the mass source's weights (and so is
    /// graph-bound whenever that source is computed from the topology).
    pub fn weighted(self) -> bool {
        matches!(
            self,
            PhysicsOverlay::DegreeRepulsion | PhysicsOverlay::HubGravity
        )
    }

    /// Whether the overlay keeps the graph moving on its own (the tide).
    pub fn never_rests(self) -> bool {
        matches!(self, PhysicsOverlay::Tide)
    }
}

/// Where the Kinds law reads a node's kind from, and Group pull its groups —
/// the host's choice per scene, one selector each.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PhysicsKindSource {
    /// The URL host: every site a kind.
    Site,
    /// The Louvain community: every cluster a kind.
    Cluster,
    /// A proper colouring (DSATUR): a kind never touches its own kind, so the
    /// rule matrix plays out between neighbours.
    Coloring,
    /// The connected component: every island a kind.
    Component,
    /// Degree bands: isolated, leaf, connected, hub.
    Degree,
    /// What the nodes say: their titles embedded, and the clusters of the
    /// similarity pairs between them (the Meaning channel, dynamics grammar
    /// plan, G2). Site until the first snapshot lands.
    Meaning,
}

impl PhysicsKindSource {
    pub const ALL: [PhysicsKindSource; 6] = [
        PhysicsKindSource::Site,
        PhysicsKindSource::Cluster,
        PhysicsKindSource::Coloring,
        PhysicsKindSource::Component,
        PhysicsKindSource::Degree,
        PhysicsKindSource::Meaning,
    ];

    pub fn id(self) -> &'static str {
        match self {
            PhysicsKindSource::Site => "site",
            PhysicsKindSource::Cluster => "cluster",
            PhysicsKindSource::Coloring => "coloring",
            PhysicsKindSource::Component => "component",
            PhysicsKindSource::Degree => "degree",
            PhysicsKindSource::Meaning => "meaning",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            PhysicsKindSource::Site => "By site",
            PhysicsKindSource::Cluster => "By cluster",
            PhysicsKindSource::Coloring => "By colouring",
            PhysicsKindSource::Component => "By island",
            PhysicsKindSource::Degree => "By degree",
            PhysicsKindSource::Meaning => "By meaning",
        }
    }

    pub fn parse(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|source| source.id() == id)
    }
}

/// Where Orbit's masses and the hub overlays' weights come from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PhysicsMassSource {
    /// Degree: the most-connected node is the heaviest.
    Degree,
    /// PageRank: the most linked-to node is the heaviest, links from heavy
    /// nodes counting for more.
    PageRank,
}

impl PhysicsMassSource {
    pub const ALL: [PhysicsMassSource; 2] =
        [PhysicsMassSource::Degree, PhysicsMassSource::PageRank];

    pub fn id(self) -> &'static str {
        match self {
            PhysicsMassSource::Degree => "degree",
            PhysicsMassSource::PageRank => "pagerank",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            PhysicsMassSource::Degree => "By degree",
            PhysicsMassSource::PageRank => "By rank",
        }
    }

    pub fn parse(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|source| source.id() == id)
    }
}

/// Where the Depth overlay reads a node's depth from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PhysicsDepthSource {
    /// Breadth-first from the nodes nothing points at.
    Roots,
    /// Sugiyama's layer step: cut a feedback arc set, then longest-path layers
    /// over the remaining order — works on cycles.
    Layers,
    /// Dominator-tree depth from the focused node: what you must pass through
    /// to reach a node from the selection. Roots when nothing is focused.
    Focus,
}

impl PhysicsDepthSource {
    pub const ALL: [PhysicsDepthSource; 3] = [
        PhysicsDepthSource::Roots,
        PhysicsDepthSource::Layers,
        PhysicsDepthSource::Focus,
    ];

    pub fn id(self) -> &'static str {
        match self {
            PhysicsDepthSource::Roots => "roots",
            PhysicsDepthSource::Layers => "layers",
            PhysicsDepthSource::Focus => "focus",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            PhysicsDepthSource::Roots => "From roots",
            PhysicsDepthSource::Layers => "By layer",
            PhysicsDepthSource::Focus => "From focus",
        }
    }

    pub fn parse(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|source| source.id() == id)
    }
}

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

/// Density at the catalog's defaults (ruled 2026-10-02 and 2026-10-03): 64²,
/// one-second passes, blur 0.25 spacings, at least 60 passes, then passes
/// repeated until the shift test stops them, the pass cap as the fallback.
pub(crate) fn density_law(masses: Vec<(NodeKey, f32)>) -> Density {
    let mut law = Density::new(masses, DENSITY_RESOLUTION);
    law.seconds = 1.0;
    law.initial_blur = 0.25;
    law.stop = DENSITY_STOP;
    law.patience = DENSITY_PATIENCE;
    law.max_passes = DENSITY_MAX_PASSES;
    law.min_passes = DENSITY_MIN_PASSES;
    law
}

/// The sources a build reads attributes through.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct LawSources {
    pub kind: PhysicsKindSource,
    /// Group pull's groups: any kind channel, unfolded.
    pub groups: PhysicsKindSource,
    pub mass: PhysicsMassSource,
    pub depth: PhysicsDepthSource,
    /// The focused node, for the Focus depth source.
    pub focus: Option<NodeKey>,
    /// The seed Kinds' matrix and Anneal's walk draw from: the spec's
    /// (F156), [`LAW_SEED`] by default, so no default run moves.
    pub seed: u64,
}

impl LawSources {
    /// The default sources and no focus: enough to build any law over no
    /// graph, to read its declared terms.
    pub(crate) fn bare() -> Self {
        Self {
            kind: PhysicsKindSource::Site,
            groups: PhysicsKindSource::Site,
            mass: PhysicsMassSource::Degree,
            depth: PhysicsDepthSource::Roots,
            focus: None,
            seed: LAW_SEED,
        }
    }
}

/// The graph inputs a law or overlay snapshots at build: the node set, the
/// visible spring edges, and (on demand) the site grouping, the Louvain
/// partition, the Meaning snapshot and the physics view's channels.
///
/// The physics channels come from the producers in
/// [`signals::physics`](crate::signals::physics). Over the physics view,
/// handed a registry and the view's key ([`Self::with_registry`]), they are
/// the registry's, one computation per key (G2c; F178, F179); over anything
/// else (a grouping's subgraphs, the board's items, the catalog's nominal)
/// they are computed here.
pub(crate) struct LawInputs<'a> {
    nodes: Vec<NodeKey>,
    edges: Vec<(NodeKey, NodeKey)>,
    /// Each node's site (the URL host for a graph node; whatever grouping a
    /// board's host names), the Kinds law's and the group overlay's default:
    /// the canvas's registry's `groups.site` when it hands them in.
    sites: std::borrow::Cow<'a, HashMap<NodeKey, String>>,
    clusters: Option<&'a crate::signals::ClusterSet>,
    meaning: Option<&'a crate::canvas::meaning::MeaningSnapshot>,
    /// The registry serving the physics view, and the view's key.
    registry: Option<(&'a ChannelRegistry, PhysicsViewKey)>,
}

impl<'a> LawInputs<'a> {
    /// The inputs over `graph`, reading `sites` when the caller's registry
    /// holds them (computing them here otherwise, as a stats reader does).
    pub(crate) fn new(
        graph: &Graph,
        hidden_edges: &HashSet<crate::canvas::EdgeCell>,
        clusters: Option<&'a crate::signals::ClusterSet>,
        sites: Option<&'a HashMap<NodeKey, String>>,
    ) -> Self {
        let mut inputs = Self::from_parts(
            graph.nodes().map(|(key, _)| key).collect(),
            visible_relation_edges(graph, hidden_edges),
            HashMap::new(),
        );
        inputs.sites = match sites {
            Some(sites) => std::borrow::Cow::Borrowed(sites),
            None => std::borrow::Cow::Owned(
                graph
                    .nodes()
                    .map(|(key, node)| (key, crate::canvas::channels::site_of(node).to_string()))
                    .collect(),
            ),
        };
        inputs.clusters = clusters;
        inputs
    }

    /// Graph-free inputs: a node list, an edge list, and a site per node —
    /// what a [`PhysicsBoard`](crate::canvas::PhysicsBoard) has for a scene's items.
    pub(crate) fn from_parts(
        mut nodes: Vec<NodeKey>,
        edges: Vec<(NodeKey, NodeKey)>,
        sites: HashMap<NodeKey, String>,
    ) -> Self {
        nodes.sort_by_key(|key| key.index());
        Self {
            nodes,
            edges,
            sites: std::borrow::Cow::Owned(sites),
            clusters: None,
            meaning: None,
            registry: None,
        }
    }

    /// Read Meaning groups from `snapshot` (site until one exists).
    pub(crate) fn with_meaning(
        mut self,
        snapshot: Option<&'a crate::canvas::meaning::MeaningSnapshot>,
    ) -> Self {
        self.meaning = snapshot;
        self
    }

    /// Read the physics channels from `registry`, these inputs being the
    /// physics view at `key` (F178).
    pub(crate) fn with_registry(
        mut self,
        registry: &'a ChannelRegistry,
        key: PhysicsViewKey,
    ) -> Self {
        self.registry = Some((registry, key));
        self
    }

    /// The registry and the view it serves, when these inputs are the view.
    fn served(&self) -> Option<(&'a ChannelRegistry, PhysicsView<'_>)> {
        self.registry.map(|(registry, key)| {
            (
                registry,
                PhysicsView {
                    key,
                    nodes: &self.nodes,
                    edges: &self.edges,
                },
            )
        })
    }

    /// The visible spring edges the forces pull along.
    pub(crate) fn edges(&self) -> &[(NodeKey, NodeKey)] {
        &self.edges
    }

    /// Each node's site, for inputs built over a subset of the nodes.
    pub(crate) fn sites(&self) -> &HashMap<NodeKey, String> {
        &self.sites
    }

    /// Every node's group by site, as a dense id in first-seen order.
    fn site_groups(&self) -> Vec<(NodeKey, u32)> {
        let mut ids: HashMap<String, u32> = HashMap::new();
        self.nodes
            .iter()
            .map(|&key| {
                let site = self.sites.get(&key).cloned().unwrap_or_default();
                let next = ids.len() as u32;
                (key, *ids.entry(site).or_insert(next))
            })
            .collect()
    }

    /// Every node's Louvain community index (`None` without a partition).
    fn cluster_groups(&self) -> Option<Vec<(NodeKey, u32)>> {
        let clusters = self.clusters?;
        let mut groups = Vec::new();
        for (i, cluster) in clusters.clusters.iter().enumerate() {
            for &member in &cluster.members {
                groups.push((member, i as u32));
            }
        }
        Some(groups)
    }

    /// Every node's Meaning cluster (`None` without a snapshot). A node the
    /// snapshot does not know yet (added since it was taken) joins one extra
    /// group until the next snapshot lands.
    fn meaning_groups(&self) -> Option<Vec<(NodeKey, u32)>> {
        let snapshot = self.meaning?;
        let of: HashMap<NodeKey, u32> = snapshot.groups.iter().copied().collect();
        let unknown = snapshot.clusters.clusters.len() as u32;
        Some(
            self.nodes
                .iter()
                .map(|&key| (key, of.get(&key).copied().unwrap_or(unknown)))
                .collect(),
        )
    }

    /// Degree bands: isolated, leaf, connected, hub.
    fn degree_groups(&self) -> Vec<(NodeKey, u32)> {
        match self.served() {
            Some((registry, view)) => registry.groups_degree(view),
            None => physics::degree_bands(&self.nodes, &self.edges),
        }
    }

    /// The groups channel: every node's group from any kind channel, as a
    /// dense id, unfolded. Cluster and Meaning read site until their
    /// partition exists.
    pub(crate) fn groups(&self, source: PhysicsKindSource) -> Vec<(NodeKey, u32)> {
        match source {
            PhysicsKindSource::Site => self.site_groups(),
            PhysicsKindSource::Cluster => {
                self.cluster_groups().unwrap_or_else(|| self.site_groups())
            },
            PhysicsKindSource::Coloring => self.coloring_groups(),
            PhysicsKindSource::Component => self.component_groups(),
            PhysicsKindSource::Degree => self.degree_groups(),
            PhysicsKindSource::Meaning => {
                self.meaning_groups().unwrap_or_else(|| self.site_groups())
            },
        }
    }

    /// A proper colouring of the visible graph (DSATUR): adjacent nodes never
    /// share a group.
    pub(crate) fn coloring_groups(&self) -> Vec<(NodeKey, u32)> {
        match self.served() {
            Some((registry, view)) => registry.groups_coloring(view),
            None => physics::coloring_groups(&self.nodes, &self.edges),
        }
    }

    /// Every node's connected component (island), as a dense id.
    pub(crate) fn component_groups(&self) -> Vec<(NodeKey, u32)> {
        match self.served() {
            Some((registry, view)) => registry.groups_component(view),
            None => physics::component_groups(&self.nodes, &self.edges),
        }
    }

    /// PageRank over the directed view, scaled so the mean weight is one
    /// (comparable to log degree on a sparse graph).
    pub(crate) fn page_rank_weights(&self) -> Vec<(NodeKey, f32)> {
        match self.served() {
            Some((registry, view)) => registry.mass_pagerank(view),
            None => physics::page_rank_weights(&self.nodes, &self.edges),
        }
    }

    /// The mass channel's values before any reader's transform: degree, or
    /// PageRank with the mean rank one. Orbit reads `1 +` these; the hub
    /// overlays read `ln(1 + degree)` (seiche's default) or the rank itself.
    pub(crate) fn mass_values(&self, source: PhysicsMassSource) -> Vec<(NodeKey, f32)> {
        match source {
            PhysicsMassSource::Degree => match self.served() {
                Some((registry, view)) => registry.mass_degree(view),
                None => physics::mass_degree(&self.nodes, &self.edges),
            },
            PhysicsMassSource::PageRank => self.page_rank_weights(),
        }
    }

    /// The depth channel's values.
    pub(crate) fn depth_values(
        &self,
        source: PhysicsDepthSource,
        focus: Option<NodeKey>,
    ) -> Vec<(NodeKey, u32)> {
        self.depths(source, focus)
    }

    /// Masses for Orbit and Density: `1 + degree`, or `1 + rank` with the
    /// mean rank one.
    pub(crate) fn masses(&self, source: PhysicsMassSource) -> Vec<(NodeKey, f32)> {
        self.mass_values(source)
            .into_iter()
            .map(|(key, value)| (key, 1.0 + value))
            .collect()
    }

    /// A kind per node for the Kinds law, from the chosen source, plus how many
    /// kinds there are.
    pub(crate) fn kinds(&self, source: PhysicsKindSource) -> (Vec<(NodeKey, u8)>, usize) {
        let groups = self.groups(source);
        // Particle life reads best with a handful of kinds; fold a long tail of
        // sites into eight, keeping a small catalog its own size.
        let distinct = groups
            .iter()
            .map(|(_, g)| *g)
            .max()
            .map_or(0, |g| g as usize + 1);
        let kind_count = distinct.clamp(1, 8);
        let kinds = groups
            .into_iter()
            .map(|(key, g)| (key, (g as usize % kind_count) as u8))
            .collect();
        (kinds, kind_count)
    }

    /// BFS depth from the roots — the nodes with no incoming visible edge, or
    /// every node when the graph is all cycles.
    pub(crate) fn root_depths(&self) -> Vec<(NodeKey, u32)> {
        match self.served() {
            Some((registry, view)) => registry.depth_roots(view),
            None => physics::root_depths(&self.nodes, &self.edges),
        }
    }

    /// Sugiyama's layer step: cut a greedy feedback arc set so the directed
    /// view is acyclic, then longest-path layering in topological order.
    pub(crate) fn layer_depths(&self) -> Vec<(NodeKey, u32)> {
        match self.served() {
            Some((registry, view)) => registry.depth_layers(view),
            None => physics::layer_depths(&self.nodes, &self.edges),
        }
    }

    /// Dominator-tree depth from `focus`: the number of nodes every path from
    /// the focus must pass through. Unreachable nodes sit one level below the
    /// deepest reachable one; without a focus, the roots.
    pub(crate) fn focus_depths(&self, focus: Option<NodeKey>) -> Vec<(NodeKey, u32)> {
        let Some(focus) = focus else {
            return self.root_depths();
        };
        match self.served() {
            Some((registry, view)) => registry.depth_focus(view, focus),
            None => physics::focus_depths(&self.nodes, &self.edges, focus),
        }
    }

    fn depths(&self, source: PhysicsDepthSource, focus: Option<NodeKey>) -> Vec<(NodeKey, u32)> {
        match source {
            PhysicsDepthSource::Roots => self.root_depths(),
            PhysicsDepthSource::Layers => self.layer_depths(),
            PhysicsDepthSource::Focus => self.focus_depths(focus),
        }
    }

    /// The minimum spanning tree's edges over the undirected view (a pair
    /// with more relations is a shorter edge, so the tree prefers it):
    /// `edges.spanning`, Stress's skeleton.
    pub(crate) fn skeleton_edges(&self) -> Vec<(NodeKey, NodeKey)> {
        match self.served() {
            Some((registry, view)) => registry.edges_spanning(view),
            None => physics::spanning_edges(&self.nodes, &self.edges),
        }
    }

    /// Shortest-path distances in hops over the undirected view, each hop
    /// costing `1 / multiplicity`, every connected pair once.
    pub(crate) fn weighted_distances(&self) -> Vec<(NodeKey, NodeKey, f32)> {
        match self.served() {
            Some((registry, view)) => registry.distances_hops(view),
            None => physics::hop_distances(&self.nodes, &self.edges),
        }
    }

    /// The law's forces when `overlays` join it: Density's conversion is on
    /// once it takes an overlay, and off alone (F73).
    pub(crate) fn law_forces_taking(
        &self,
        law: PhysicsLaw,
        sources: LawSources,
        overlays: &[PhysicsOverlay],
    ) -> Vec<Box<dyn Force>> {
        if law == PhysicsLaw::Density && !overlays.is_empty() {
            let mut density = density_law(self.masses(sources.mass));
            density.converts = true;
            return vec![Box::new(density)];
        }
        self.law_forces(law, sources)
    }

    /// Build the law's own forces against these inputs.
    pub(crate) fn law_forces(&self, law: PhysicsLaw, sources: LawSources) -> Vec<Box<dyn Force>> {
        match law {
            PhysicsLaw::Springs => vec![
                Box::new(NodeExclusion::default()),
                Box::new(EdgeSpring::default()),
                Box::new(Boundary::default()),
            ],
            PhysicsLaw::Charge => vec![
                Box::new(BarnesHutRepulsion {
                    strength: CHARGE_STRENGTH,
                    ..BarnesHutRepulsion::default()
                }),
                Box::new(EdgeSpring::default()),
                Box::new(Boundary::default()),
            ],
            PhysicsLaw::Stress => vec![
                Box::new(NodeExclusion::default()),
                Box::new(StressSpring::from_weighted_distances(
                    self.weighted_distances(),
                    EdgeSpring::default().rest_length,
                )),
                Box::new(Boundary::default()),
            ],
            PhysicsLaw::Energy => vec![
                Box::new(NodeExclusion::default()),
                Box::new(LinLogForce::default()),
            ],
            PhysicsLaw::Orbit => vec![
                Box::new(NodeExclusion {
                    cutoff: ORBIT_EXCLUSION_REACH,
                    ..NodeExclusion::default()
                }),
                Box::new(Gravity::new(
                    self.masses(sources.mass),
                    CounterDamping::Tangential,
                )),
                Box::new(Boundary {
                    strength: ORBIT_CENTRING,
                }),
            ],
            PhysicsLaw::Kinds => {
                let (kinds, kind_count) = self.kinds(sources.kind);
                vec![
                    Box::new(NodeExclusion::default()),
                    Box::new(ParticleLife::seeded(kinds, kind_count, sources.seed)),
                ]
            },
            PhysicsLaw::Flock => vec![
                Box::new(NodeExclusion::default()),
                Box::new(Boids::default()),
            ],
            PhysicsLaw::Sync => {
                // Every node on one ring; communities become arcs of it.
                let radii = self.nodes.iter().map(|&key| (key, 240.0));
                vec![
                    Box::new(NodeExclusion::default()),
                    Box::new(Kuramoto::new(radii)),
                ]
            },
            PhysicsLaw::Flow => vec![
                Box::new(NodeExclusion::default()),
                Box::new(MagneticSpring::default()),
                Box::new(Boundary::default()),
            ],
            PhysicsLaw::Anneal => vec![Box::new(Anneal::seeded(sources.seed))],
            // Held, not empty: with no force at all rapier's contact solver
            // blasts an overlapping seed apart (the Still receipt found it).
            PhysicsLaw::Still => vec![Box::new(Hold)],
            PhysicsLaw::Density => vec![Box::new(density_law(self.masses(sources.mass)))],
        }
    }

    /// Build one overlay's force against these inputs.
    pub(crate) fn overlay_force(
        &self,
        overlay: PhysicsOverlay,
        sources: LawSources,
    ) -> Box<dyn Force> {
        match overlay {
            PhysicsOverlay::DegreeRepulsion => match sources.mass {
                PhysicsMassSource::Degree => Box::new(DegreeRepulsion::default()),
                PhysicsMassSource::PageRank => {
                    Box::new(DegreeRepulsion::default().with_weights(self.page_rank_weights()))
                },
            },
            PhysicsOverlay::DomainCluster => {
                Box::new(DomainCluster::new(self.groups(sources.groups)))
            },
            PhysicsOverlay::HubGravity => match sources.mass {
                PhysicsMassSource::Degree => Box::new(HubGravity::default()),
                PhysicsMassSource::PageRank => {
                    Box::new(HubGravity::default().with_weights(self.page_rank_weights()))
                },
            },
            PhysicsOverlay::DepthGravity => {
                Box::new(DepthGravity::new(self.depths(sources.depth, sources.focus)))
            },
            PhysicsOverlay::GridSnap => Box::new(GridSnap::default()),
            PhysicsOverlay::GravityLocus => Box::new(GravityLocus::at((0.0, 0.0))),
            PhysicsOverlay::Tide => Box::new(GravityLocus::tidal((0.0, 0.0), 240.0, 24.0)),
            PhysicsOverlay::Skeleton => Box::new(
                StressSpring::from_distances(
                    self.skeleton_edges().into_iter().map(|(a, b)| (a, b, 1)),
                    EdgeSpring::default().rest_length,
                )
                .with_stiffness(SKELETON_STIFFNESS),
            ),
        }
    }
}

impl Canvas {
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
        if self.roles.encoded == seiche::Axes::BOTH { return false; }
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
    /// to the physics backend. Position-preserving.
    pub(crate) fn rebuild_law_forces(&mut self) {
        #[cfg(test)]
        {
            self.law_rebuilds += 1;
        }
        if self.roles.encoded == seiche::Axes::BOTH {
            // F28: two directly encoded axes admit contact separation only.
            self.physics.set_forces(vec![Box::new(seiche::NodeExclusion {
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
        let forces = {
            let inputs = LawInputs::new(
                &self.graph,
                &self.hidden_edges,
                if wants_clusters {
                    self.channels.community_held()
                } else {
                    None
                },
                self.channels.sites_fresh(&self.graph),
            )
            .with_meaning(self.meaning.snapshot())
            .with_registry(&self.channels, self.physics_view_key());
            self.composed_forces(&inputs, sources)
        };
        #[cfg(test)]
        if self.physics_reads(PhysicsKindSource::Meaning) {
            self.meaning.built_from = self.meaning.snapshot().map(|s| s.run);
        }
        self.physics.set_forces(forces);
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
        let edges = visible_relation_edges(&self.graph, &self.hidden_edges);
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
        LawInputs::new(&self.graph, &self.hidden_edges, None, None)
            .with_registry(&self.channels, self.physics_view_key())
    }

    /// The physics view's key: the graph's structure and the view revision
    /// (F178).
    pub(crate) fn physics_view_key(&self) -> PhysicsViewKey {
        PhysicsViewKey {
            structure: self.graph.revision(),
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
