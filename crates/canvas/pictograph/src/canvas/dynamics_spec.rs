// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The dynamics spec over the canvas: its catalog of presets (G4a) and the
//! binding that makes a spec the canvas's record (G4b1).
//!
//! seiche holds the spec ([`seiche::spec`]); this module is its catalog of
//! presets: a law preset is a [`PhysicsLaw`] id, an overlay preset a
//! [`PhysicsOverlay`] id, the strings a saved scene already stores. A preset
//! is built over two nodes on two sites joined by an edge, which is enough
//! to read every law's declarations, Kinds' asymmetric matrix included
//! (over one site its matrix is 1 × 1 and reads as symmetric). Density's
//! word on overlays (F73) is the catalog's [`PhysicsLaw::admits`].
//!
//! **The binding** (G4b1; F146, F149, F156, F157). [`bind`] reads a spec
//! into what the canvas runs: derive first, then each channel slot resolved
//! host-side through [`Channel::parse`], family-checked, an id that does not
//! parse or that its slot cannot read refused by slot and id. The slots are
//! [`SLOTS`]: `kind` (Kinds' kinds), `groups` (Group pull's groups), `mass`
//! (Orbit's and Density's masses, the hub overlays' weights) and `depth`
//! (the Depth overlay). A grouping's partition and the target's role groups
//! read a `groups.*` channel the canvas resolves to a kind source, so
//! `groups.bridges` is refused there. The root runs as a law with overlays,
//! a mix or a grouping with overlays, or a schedule of those (the shapes
//! derive admits today).
//!
//! The canvas holds the spec as its record (`dynamics_record`,
//! [`Canvas::dynamics_spec`]): the
//! root and the channels are what it runs, a schedule as authored (F157, "the
//! spec is a recipe"), with the seed, bars, realization and the target's
//! arrangement carried from the spec it was given, and the target's roles
//! read live. A spec opened and not edited since reads back as it was
//! opened, so a re-save keeps it byte for byte.

use std::collections::{BTreeMap, HashMap};

use kernel::graph::NodeKey;
/// The signatures a spec's bars name.
pub use seiche::Observable;
pub use seiche::spec::*;
use seiche::{Admission, Force, Role};

use super::channels::Channel;
use super::composition::{GroupSource, PhysicsComposition, PhysicsGrouping};
use super::physics_board::PhysicsChoice;
use super::physics_catalog::{
    LawInputs, LawSources, PhysicsDepthSource, PhysicsKindSource, PhysicsLaw, PhysicsMassSource,
    PhysicsOverlay,
};
use super::schedule::{PhysicsStage, StageStop};

/// The canvas's catalog: its laws and overlays as presets.
pub struct CanvasCatalog;

impl CanvasCatalog {
    /// Two nodes on two sites, one edge between them.
    fn nominal() -> LawInputs<'static> {
        let (a, b) = (NodeKey::new(0), NodeKey::new(1));
        let sites = HashMap::from([(a, "a.test".to_string()), (b, "b.test".to_string())]);
        LawInputs::from_parts(vec![a, b], vec![(a, b)], sites)
    }
}

impl SpecCatalog for CanvasCatalog {
    fn preset(&self, id: &str, at: PresetAt) -> Option<Vec<Box<dyn Force>>> {
        let inputs = Self::nominal();
        match at {
            PresetAt::Law => Some(inputs.law_forces(PhysicsLaw::parse(id)?, LawSources::bare())),
            PresetAt::Overlay => {
                Some(vec![inputs.overlay_force(
                    PhysicsOverlay::parse(id)?,
                    LawSources::bare(),
                )])
            },
        }
    }

    fn admits(&self, law: &str, overlay: &str) -> Option<Admission> {
        Some(PhysicsLaw::parse(law)?.admits(PhysicsOverlay::parse(overlay)?))
    }
}

/// `spec`'s derived fields over the canvas's catalog, or why it is refused.
pub fn derive(spec: &DynamicsSpec) -> Result<Derived, SpecError> {
    spec.derive(&CanvasCatalog)
}

/// The slot Kinds reads its kinds through.
pub const SLOT_KIND: &str = "kind";
/// The slot Group pull reads its groups through.
pub const SLOT_GROUPS: &str = "groups";
/// The slot masses and hub weights read through.
pub const SLOT_MASS: &str = "mass";
/// The slot the Depth overlay reads depths through.
pub const SLOT_DEPTH: &str = "depth";
/// Every channel slot a preset reads.
pub const SLOTS: [&str; 4] = [SLOT_KIND, SLOT_GROUPS, SLOT_MASS, SLOT_DEPTH];

/// Why a spec does not bind: derive refused it, or a channel, a partition or
/// a role group names what the canvas cannot read, or an item id is not one.
#[derive(Clone, Debug, PartialEq)]
pub enum BindError {
    Refused(SpecError),
    Unbound { place: String, reason: String },
}

impl std::fmt::Display for BindError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BindError::Refused(error) => error.fmt(f),
            BindError::Unbound { place, reason } => write!(f, "at {place}: {reason}"),
        }
    }
}

impl std::error::Error for BindError {}

fn unbound(place: impl Into<String>, reason: impl Into<String>) -> BindError {
    BindError::Unbound {
        place: place.into(),
        reason: reason.into(),
    }
}

/// The four sources a spec's slots resolve to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BoundSources {
    pub kind: PhysicsKindSource,
    pub groups: PhysicsKindSource,
    pub mass: PhysicsMassSource,
    pub depth: PhysicsDepthSource,
}

impl Default for BoundSources {
    fn default() -> Self {
        Self {
            kind: PhysicsKindSource::Site,
            groups: PhysicsKindSource::Site,
            mass: PhysicsMassSource::Degree,
            depth: PhysicsDepthSource::Roots,
        }
    }
}

impl BoundSources {
    /// Every slot written, so the channels say what each slot reads.
    pub fn channels(&self) -> BTreeMap<String, String> {
        [
            (SLOT_KIND, Channel::Kind(self.kind)),
            (SLOT_GROUPS, Channel::Groups(self.groups)),
            (SLOT_MASS, Channel::Mass(self.mass)),
            (SLOT_DEPTH, Channel::Depth(self.depth)),
        ]
        .into_iter()
        .map(|(slot, channel)| (slot.to_string(), channel.id()))
        .collect()
    }
}

/// Resolve a spec's channel slots (F149): a slot it leaves out reads the
/// catalog's default.
pub fn resolve_channels(channels: &BTreeMap<String, String>) -> Result<BoundSources, BindError> {
    let mut sources = BoundSources::default();
    for (slot, id) in channels {
        let place = format!("channels.{slot}");
        if !SLOTS.contains(&slot.as_str()) {
            return Err(unbound(
                place,
                format!("no preset reads a slot named {slot}"),
            ));
        }
        let channel =
            Channel::parse(id).ok_or_else(|| unbound(&place, format!("unknown channel {id}")))?;
        match (slot.as_str(), channel) {
            (SLOT_KIND, Channel::Kind(source)) => sources.kind = source,
            (SLOT_GROUPS, Channel::Groups(source)) => sources.groups = source,
            (SLOT_MASS, Channel::Mass(source)) => sources.mass = source,
            (SLOT_DEPTH, Channel::Depth(source)) => sources.depth = source,
            _ => return Err(unbound(place, format!("the {slot} slot cannot read {id}"))),
        }
    }
    Ok(sources)
}

/// A `groups.*` channel the canvas resolves to a kind source: a grouping's
/// partition or the target's role groups.
fn groups_source(id: &str, place: &str) -> Result<PhysicsKindSource, BindError> {
    match Channel::parse(id) {
        Some(Channel::Groups(source)) => Ok(source),
        Some(_) => Err(unbound(
            place,
            format!("{id} is not a groups channel the canvas partitions by"),
        )),
        None => Err(unbound(place, format!("unknown channel {id}"))),
    }
}

/// What the root runs.
#[derive(Clone, Debug, PartialEq)]
pub enum BoundRoot {
    /// A law with its overlays.
    Law {
        law: PhysicsLaw,
        overlays: Vec<PhysicsOverlay>,
    },
    /// A mix or a grouping in the law slot, with overlays; `law` is the
    /// first law it runs, what a law picker shows (F157).
    Composed {
        law: PhysicsLaw,
        composition: PhysicsComposition,
        overlays: Vec<PhysicsOverlay>,
    },
    /// A schedule, run from its first stage.
    Schedule(Vec<PhysicsStage>),
}

/// The target as the canvas takes it.
#[derive(Clone, Debug, PartialEq)]
pub struct BoundTarget {
    pub arrangement: String,
    pub anchored_pull: f32,
    pub default_role: Role,
    pub group_source: PhysicsKindSource,
    pub groups: BTreeMap<String, Role>,
    pub items: Vec<(uuid::Uuid, Role)>,
}

/// A spec read into what the canvas runs.
#[derive(Clone, Debug, PartialEq)]
pub struct Bound {
    pub sources: BoundSources,
    pub seed: u64,
    pub root: BoundRoot,
    pub target: Option<BoundTarget>,
    pub damping: Option<f64>,
}

fn preset_law(node: &Node, place: &str) -> Result<(PhysicsLaw, f64), BindError> {
    match node {
        Node::Preset { id, weight, .. } => PhysicsLaw::parse(id)
            .map(|law| (law, *weight))
            .ok_or_else(|| unbound(place, format!("unknown law preset {id}"))),
        _ => Err(unbound(place, "the canvas runs a law preset here")),
    }
}

/// A node's overlays as the canvas runs them, duplicates collapsed as
/// [`Canvas::set_physics_overlays`] collapses them.
fn bound_overlays(node: &Node, place: &str) -> Result<Vec<PhysicsOverlay>, BindError> {
    let mut out: Vec<PhysicsOverlay> = Vec::new();
    for (i, overlay) in node.overlays().iter().enumerate() {
        let here = format!("{place}.overlays[{i}]");
        let Node::Preset { id, .. } = overlay else {
            return Err(unbound(here, "the canvas runs an overlay preset here"));
        };
        let overlay = PhysicsOverlay::parse(id)
            .ok_or_else(|| unbound(&here, format!("unknown overlay preset {id}")))?;
        if !out.contains(&overlay) {
            out.push(overlay);
        }
    }
    Ok(out)
}

/// A root or a stage: a law, or a mix or grouping, with overlays.
fn bind_node(
    node: &Node,
    place: &str,
) -> Result<(PhysicsLaw, Option<PhysicsComposition>, Vec<PhysicsOverlay>), BindError> {
    let overlays = bound_overlays(node, place)?;
    match node {
        Node::Preset { .. } => Ok((preset_law(node, place)?.0, None, overlays)),
        Node::Mix { parts, .. } => {
            let laws = parts
                .iter()
                .enumerate()
                .map(|(i, part)| {
                    preset_law(part, &format!("{place}.parts[{i}]"))
                        .map(|(law, weight)| (law, weight as f32))
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok((laws[0].0, Some(PhysicsComposition::Mix(laws)), overlays))
        },
        Node::Grouped {
            partition,
            outer,
            inner,
            ..
        } => {
            let groups = groups_source(partition, &format!("{place}.partition"))?;
            let (outer, outer_weight) = preset_law(outer, &format!("{place}.outer"))?;
            let (inner, _) = preset_law(inner, &format!("{place}.inner"))?;
            Ok((
                outer,
                Some(PhysicsComposition::Grouped(PhysicsGrouping {
                    groups: GroupSource::Channel(groups),
                    outer,
                    inner,
                    outer_weight: outer_weight as f32,
                })),
                overlays,
            ))
        },
        Node::Raw { .. } | Node::Schedule { .. } => Err(unbound(
            place,
            "the canvas runs a law, a mix or a grouping here",
        )),
    }
}

fn bind_stop(stop: Stop) -> StageStop {
    match stop {
        Stop::Rest => StageStop::Rest,
        Stop::Frames(n) => StageStop::Frames(n),
        Stop::LawDone => StageStop::LawDone,
    }
}

fn spec_stop(stop: StageStop) -> Stop {
    match stop {
        StageStop::Rest => Stop::Rest,
        StageStop::Frames(n) => Stop::Frames(n),
        StageStop::LawDone => Stop::LawDone,
    }
}

fn bind_target(target: &Target) -> Result<BoundTarget, BindError> {
    let (group_source, groups) = match &target.groups {
        Some(groups) => (
            groups_source(&groups.channel, "target.groups.channel")?,
            groups.roles.clone(),
        ),
        None => (PhysicsKindSource::Site, BTreeMap::new()),
    };
    let items = target
        .items
        .iter()
        .map(|(id, role)| {
            uuid::Uuid::parse_str(id)
                .map(|id| (id, *role))
                .map_err(|_| {
                    unbound(
                        format!("target.items.{id}"),
                        format!("{id} is not an item id"),
                    )
                })
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(BoundTarget {
        arrangement: target.arrangement.clone(),
        anchored_pull: target.anchored_pull as f32,
        default_role: target.default_role,
        group_source,
        groups,
        items,
    })
}

/// Read `spec` into what the canvas runs (see the module docs); graph-free,
/// so a host checks a scene's spec before it opens anything.
pub fn bind(spec: &DynamicsSpec) -> Result<Bound, BindError> {
    derive(spec).map_err(BindError::Refused)?;
    bind_shape(spec)
}

/// [`bind`] past derive: the channels, the root's shape and the target.
pub(crate) fn bind_shape(spec: &DynamicsSpec) -> Result<Bound, BindError> {
    let sources = resolve_channels(&spec.channels)?;
    let root = match &spec.root {
        Node::Schedule { stages, .. } => BoundRoot::Schedule(
            stages
                .iter()
                .enumerate()
                .map(|(i, stage)| {
                    let (law, composition, overlays) =
                        bind_node(&stage.node, &format!("root.stages[{i}].node"))?;
                    Ok(PhysicsStage {
                        law,
                        overlays,
                        composition,
                        stop: bind_stop(stage.stop),
                        capture: stage.capture,
                    })
                })
                .collect::<Result<_, BindError>>()?,
        ),
        node => match bind_node(node, "root")? {
            (law, None, overlays) => BoundRoot::Law { law, overlays },
            (law, Some(composition), overlays) => BoundRoot::Composed {
                law,
                composition,
                overlays,
            },
        },
    };
    let Realization::Integrate { damping } = spec.realization;
    Ok(Bound {
        sources,
        seed: spec.seed,
        root,
        target: spec.target.as_ref().map(bind_target).transpose()?,
        damping,
    })
}

/// A law and its overlays as a node.
fn law_node(law: PhysicsLaw, overlays: &[PhysicsOverlay]) -> Node {
    let mut node = Node::preset(law.id());
    node.overlays_mut()
        .extend(overlays.iter().map(|o| Node::preset(o.id())));
    node
}

/// What a canvas runs as a node: a law, or a composition, with overlays. A
/// grouping on a partition the host handed in has no channel id, so it has
/// no node (F157).
pub fn running_node(
    law: PhysicsLaw,
    composition: Option<&PhysicsComposition>,
    overlays: &[PhysicsOverlay],
) -> Result<Node, String> {
    let mut node = match composition {
        None => return Ok(law_node(law, overlays)),
        Some(PhysicsComposition::Mix(laws)) => Node::Mix {
            parts: laws
                .iter()
                .map(|(law, weight)| {
                    let mut part = Node::preset(law.id());
                    if let Node::Preset { weight: w, .. } = &mut part {
                        *w = f64::from(*weight);
                    }
                    part
                })
                .collect(),
            weight: 1.0,
            overlays: Vec::new(),
        },
        Some(PhysicsComposition::Grouped(grouping)) => {
            let GroupSource::Channel(source) = grouping.groups else {
                return Err(
                    "a grouping on a partition the host handed in has no channel id, so \
                            it cannot be saved"
                        .into(),
                );
            };
            let mut outer = Node::preset(grouping.outer.id());
            if let Node::Preset { weight, .. } = &mut outer {
                *weight = f64::from(grouping.outer_weight);
            }
            Node::Grouped {
                partition: Channel::Groups(source).id(),
                outer: Box::new(outer),
                inner: Box::new(Node::preset(grouping.inner.id())),
                weight: 1.0,
                overlays: Vec::new(),
            }
        },
    };
    node.overlays_mut()
        .extend(overlays.iter().map(|o| Node::preset(o.id())));
    Ok(node)
}

/// A schedule as a node.
pub fn schedule_node(stages: &[PhysicsStage]) -> Result<Node, String> {
    Ok(Node::Schedule {
        stages: stages
            .iter()
            .map(|stage| {
                Ok(Stage {
                    node: running_node(stage.law, stage.composition.as_ref(), &stage.overlays)?,
                    stop: spec_stop(stage.stop),
                    capture: stage.capture,
                })
            })
            .collect::<Result<_, String>>()?,
        weight: 1.0,
        overlays: Vec::new(),
    })
}

impl PhysicsChoice {
    /// The choice as a spec: its law and overlays at the root, every slot
    /// written, the default seed, no target (F146).
    pub fn into_spec(&self) -> DynamicsSpec {
        let mut spec = DynamicsSpec::new(law_node(self.law, &self.overlays));
        spec.channels = self.sources().channels();
        spec
    }

    /// The flat view of a spec: its sources, and the law and overlays its
    /// root (or its first stage) runs; a composition's first law (F157).
    pub fn from_spec(spec: &DynamicsSpec) -> Result<Self, BindError> {
        Ok(Self::flat(bind(spec)?))
    }

    /// The flat view of a bound spec.
    pub(crate) fn flat(bound: Bound) -> Self {
        let (law, overlays) = match bound.root {
            BoundRoot::Law { law, overlays } | BoundRoot::Composed { law, overlays, .. } => {
                (law, overlays)
            },
            BoundRoot::Schedule(stages) => (stages[0].law, stages[0].overlays.clone()),
        };
        Self::of(law, overlays, bound.sources)
    }

    fn of(law: PhysicsLaw, overlays: Vec<PhysicsOverlay>, sources: BoundSources) -> Self {
        Self {
            law,
            overlays,
            kind: sources.kind,
            groups: sources.groups,
            mass: sources.mass,
            depth: sources.depth,
        }
    }

    /// The choice's four sources.
    pub fn sources(&self) -> BoundSources {
        BoundSources {
            kind: self.kind,
            groups: self.groups,
            mass: self.mass,
            depth: self.depth,
        }
    }
}
