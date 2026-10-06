// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Compositions in the catalog (dynamics grammar plan, G3): every law's
//! currency and what it does with an overlay, a weighted mix of force laws,
//! and groups (an outer law between groups, an inner law within each).
//!
//! Currencies are read from the laws' declared terms ([`seiche::compose`]):
//! a force law composes overlays freely, a kinematic law (Still, Anneal,
//! Density) takes them converted, and a resident law takes only those with a
//! resident kernel or the lagged upload. Density's interim refusal stands
//! ([`PhysicsLaw::overlay_refusal`]).
//!
//! A composition runs in the law slot instead of the picked law, overlays
//! still after it. The pickers do not express one yet (G4's `PhysicsChoice`
//! fields), so a pick replaces it. A grouping takes its partition from the
//! host until the group source is ruled: G2's `groups.*` channels are not on
//! main.

use std::collections::HashMap;

use kernel::graph::NodeKey;
use seiche::{Admission, Currency, Force, Grouped, Partition, Weighted, compose};

use super::Canvas;
use super::physics_catalog::{LawInputs, LawSources, PhysicsLaw, PhysicsOverlay};

impl PhysicsLaw {
    /// The law's forces over no graph: enough to read its declared terms.
    fn bare_forces(self) -> Vec<Box<dyn Force>> {
        LawInputs::from_parts(Vec::new(), Vec::new(), HashMap::new())
            .law_forces(self, LawSources::bare())
    }

    /// How the law's motion enters the step, from its declared terms.
    pub fn currency(self) -> Currency {
        compose::currency_of(&self.bare_forces())
    }

    /// What the law does with `overlay`: compose it, convert it, or refuse it.
    pub fn admits(self, overlay: PhysicsOverlay) -> Admission {
        let inputs = LawInputs::from_parts(Vec::new(), Vec::new(), HashMap::new());
        let force = inputs.overlay_force(overlay, LawSources::bare());
        compose::admit_force(self.currency(), force.as_ref())
    }
}

/// What runs in the law slot instead of the picked law.
#[derive(Clone, Debug, PartialEq)]
pub enum PhysicsComposition {
    /// Force laws at weights; 1 is the law as calibrated, 0 leaves it out.
    Mix(Vec<(PhysicsLaw, f32)>),
    /// An outer law between groups and an inner law within each.
    Grouped(PhysicsGrouping),
}

impl PhysicsComposition {
    /// Every law the composition runs.
    pub fn laws(&self) -> Vec<PhysicsLaw> {
        match self {
            PhysicsComposition::Mix(laws) => laws.iter().map(|(law, _)| *law).collect(),
            PhysicsComposition::Grouped(g) => vec![g.outer, g.inner],
        }
    }
}

/// Groups: Charge between them and Springs within is the first instance.
#[derive(Clone, Debug, PartialEq)]
pub struct PhysicsGrouping {
    /// Each node's group, from the host (see the module docs).
    pub groups: Vec<(NodeKey, u32)>,
    /// The law between groups, over their centroids.
    pub outer: PhysicsLaw,
    /// The law within each group, under its membership mask.
    pub inner: PhysicsLaw,
    /// The outer law's weight; 1 is the law as calibrated.
    pub outer_weight: f32,
}

/// Why a composition was refused, and which law refused it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompositionRefusal {
    pub law: PhysicsLaw,
    pub reason: &'static str,
}

impl std::fmt::Display for CompositionRefusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.law.label(), self.reason)
    }
}

impl PhysicsComposition {
    /// The first law the composition refuses: a weight scales forces, and
    /// groups share the outer law's force among members, so both take force
    /// laws only.
    pub fn refusal(&self) -> Option<CompositionRefusal> {
        self.laws()
            .into_iter()
            .find(|law| law.currency() != Currency::Force)
            .map(|law| CompositionRefusal {
                law,
                reason: compose::UNWEIGHTED,
            })
    }
}

impl LawInputs<'_> {
    /// `law`'s forces, each at `weight`.
    fn weighted_law(
        &self,
        law: PhysicsLaw,
        weight: f32,
        sources: LawSources,
    ) -> Vec<Box<dyn Force>> {
        self.law_forces(law, sources)
            .into_iter()
            .map(|force| {
                Box::new(Weighted::new(force, weight).expect("a force law, checked on set"))
                    as Box<dyn Force>
            })
            .collect()
    }

    /// The grouped composition over these inputs: the outer law built over
    /// the groups' graph, an inner law built over each group's members and
    /// the edges inside it.
    pub(crate) fn grouped_force(
        &self,
        grouping: &PhysicsGrouping,
        sources: LawSources,
    ) -> Box<dyn Force> {
        let partition = Partition::new(grouping.groups.iter().copied(), self.edges());
        let (keys, edges) = partition.outer_graph();
        let outer = LawInputs::from_parts(keys, edges, HashMap::new()).weighted_law(
            grouping.outer,
            grouping.outer_weight,
            sources,
        );
        let inner = (0..partition.len())
            .map(|g| {
                let (members, edges) = partition.group(g);
                let sites = members
                    .iter()
                    .filter_map(|k| Some((*k, self.sites().get(k)?.clone())))
                    .collect();
                LawInputs::from_parts(members.to_vec(), edges.to_vec(), sites)
                    .law_forces(grouping.inner, sources)
            })
            .collect();
        Box::new(Grouped::new(partition, outer, inner))
    }
}

impl Canvas {
    /// The composition running in the law slot, if any.
    pub fn physics_composition(&self) -> Option<&PhysicsComposition> {
        self.physics_composition.as_ref()
    }

    /// Run `composition` in the law slot instead of the picked law (`None`
    /// returns to the law), with one rebuild and one settle. Refused, with
    /// nothing changed, if it holds a law that is not a force law.
    pub fn set_physics_composition(
        &mut self,
        composition: Option<PhysicsComposition>,
    ) -> Result<(), CompositionRefusal> {
        if let Some(refusal) = composition.as_ref().and_then(PhysicsComposition::refusal) {
            return Err(refusal);
        }
        self.physics_composition = composition;
        self.rebuild_law_forces();
        self.settle_physics(if self.physics_never_rests() {
            u32::MAX
        } else {
            super::SETTLE_TICKS
        });
        Ok(())
    }

    /// The law slot's forces (the law, or the composition), then the overlays.
    pub(crate) fn composed_forces(
        &self,
        inputs: &LawInputs<'_>,
        sources: LawSources,
    ) -> Vec<Box<dyn Force>> {
        let mut forces = match &self.physics_composition {
            None => inputs.law_forces(self.physics_law, sources),
            Some(PhysicsComposition::Mix(laws)) => laws
                .iter()
                .flat_map(|(law, weight)| inputs.weighted_law(*law, *weight, sources))
                .collect(),
            Some(PhysicsComposition::Grouped(grouping)) => {
                vec![inputs.grouped_force(grouping, sources)]
            },
        };
        forces.extend(
            self.physics_overlays
                .iter()
                .map(|overlay| inputs.overlay_force(*overlay, sources)),
        );
        forces
    }
}
