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
//! resident kernel or the lagged upload. Density takes only Hub room, Centre
//! and Tide, the overlays that hold its bar converted (F73), and refuses the
//! rest ([`PhysicsLaw::refuses`]).
//!
//! A composition runs in the law slot instead of the picked law, overlays
//! still after it. The pickers do not express one yet (G4's `PhysicsChoice`
//! fields), so a pick replaces it. A grouping reads its partition from a
//! `groups.*` channel through the law inputs (`groups.meaning` among them),
//! or takes one the host hands in, as the separation receipt hands in its
//! topics (F70), and between groups an outer law acts by its repulsion alone
//! (F71).

use std::collections::HashMap;

use kernel::graph::NodeKey;
use seiche::{Admission, Currency, Force, Grouped, Partition, Weighted, compose};

use super::Canvas;
use super::physics_catalog::{
    DENSITY_ADMITS, DENSITY_REFUSAL, LawInputs, LawSources, PhysicsKindSource, PhysicsLaw,
    PhysicsOverlay,
};

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

    /// What the law does with `overlay`: compose it, convert it, or refuse
    /// it. Density takes only the overlays that hold its bar converted (F73).
    pub fn admits(self, overlay: PhysicsOverlay) -> Admission {
        if self == PhysicsLaw::Density && !DENSITY_ADMITS.contains(&overlay) {
            return Admission::Refuse(DENSITY_REFUSAL);
        }
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

/// The outer weight of the first instance, Charge between groups (ruled
/// 2026-10-06, F71, "Repulsion only, weight 16").
pub const CHARGE_BETWEEN_WEIGHT: f32 = 16.0;

/// Where a grouping's partition comes from.
#[derive(Clone, Debug, PartialEq)]
pub enum GroupSource {
    /// A `groups.*` channel, read through the law inputs at each rebuild, so
    /// the partition follows the channel (F70).
    Channel(PhysicsKindSource),
    /// A partition the host hands in, a group per node: the separation
    /// receipt's topics, its ground truth.
    Given(Vec<(NodeKey, u32)>),
}

/// Groups: Charge between them and Springs within is the first instance
/// ([`Self::charge_between`]).
#[derive(Clone, Debug, PartialEq)]
pub struct PhysicsGrouping {
    /// Each node's group: a channel's, or the host's.
    pub groups: GroupSource,
    /// The law whose repulsion acts between groups, over their centroids:
    /// its repulsion alone, the rest of the law left out (F71).
    pub outer: PhysicsLaw,
    /// The law within each group, under its membership mask.
    pub inner: PhysicsLaw,
    /// The outer repulsion's weight; 1 is the law as calibrated.
    pub outer_weight: f32,
}

impl PhysicsGrouping {
    /// Charge's repulsion between groups at weight 16, Springs within (F71).
    pub fn charge_between(groups: GroupSource) -> Self {
        Self {
            groups,
            outer: PhysicsLaw::Charge,
            inner: PhysicsLaw::Springs,
            outer_weight: CHARGE_BETWEEN_WEIGHT,
        }
    }
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

/// A law's repulsion terms alone, each its own force: a single-term force
/// as it is, a term of a bundle by its isolation. Every force law in the
/// catalog carries one (`NodeExclusion` at least).
fn repulsion_terms(forces: Vec<Box<dyn Force>>) -> Vec<Box<dyn Force>> {
    let is_repulsion = |t: &seiche::Term| matches!(t.kernel, seiche::Kernel::Repulsion { .. });
    let mut out = Vec::new();
    for force in forces {
        let terms = force.terms();
        if terms.len() == 1 {
            if is_repulsion(&terms[0]) {
                out.push(force);
            }
            continue;
        }
        for (t, term) in terms.iter().enumerate() {
            if is_repulsion(term)
                && let Some(alone) = force.isolate(t)
            {
                out.push(alone);
            }
        }
    }
    out
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

    /// The grouped composition over these inputs: the outer law's repulsion
    /// built over the groups' graph at the outer weight (F71), an inner law
    /// built over each group's members and the edges inside it.
    pub(crate) fn grouped_force(
        &self,
        grouping: &PhysicsGrouping,
        sources: LawSources,
    ) -> Box<dyn Force> {
        let groups = match &grouping.groups {
            GroupSource::Channel(source) => self.groups(*source),
            GroupSource::Given(groups) => groups.clone(),
        };
        let partition = Partition::new(groups, self.edges());
        let (keys, edges) = partition.outer_graph();
        let outer = repulsion_terms(
            LawInputs::from_parts(keys, edges, HashMap::new()).law_forces(grouping.outer, sources),
        )
        .into_iter()
        .map(|force| {
            Box::new(
                Weighted::new(force, grouping.outer_weight).expect("a force law, checked on set"),
            ) as Box<dyn Force>
        })
        .collect();
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

    /// Run `composition` in the law slot instead of the law (`None` returns
    /// to the law), with one rebuild and one settle. Refused, with nothing
    /// changed, if it holds a law that is not a force law. Crate-internal: a
    /// host runs a composition through its spec (F162); this is the way in
    /// for a partition the host hands in, which has no spec (F157); only the
    /// separation receipts hand one in.
    #[cfg(test)]
    pub(crate) fn set_physics_composition(
        &mut self,
        composition: Option<PhysicsComposition>,
    ) -> Result<(), CompositionRefusal> {
        if let Some(refusal) = composition.as_ref().and_then(PhysicsComposition::refusal) {
            return Err(refusal);
        }
        self.physics_composition = composition;
        // A composition picked takes over from a schedule, run or recorded.
        self.schedule = None;
        self.dynamics.schedule = None;
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
        inputs.composed(
            self.physics_law,
            self.physics_composition.as_ref(),
            &self.physics_overlays,
            sources,
        )
    }
}

impl LawInputs<'_> {
    /// The law slot's forces, `law` or `composition` when one is set, then
    /// the overlays: what a canvas runs, and a board mirroring its stage.
    pub(crate) fn composed(
        &self,
        law: PhysicsLaw,
        composition: Option<&PhysicsComposition>,
        overlays: &[PhysicsOverlay],
        sources: LawSources,
    ) -> Vec<Box<dyn Force>> {
        let mut forces = match composition {
            None => self.law_forces_taking(law, sources, overlays),
            Some(PhysicsComposition::Mix(laws)) => laws
                .iter()
                .flat_map(|(law, weight)| self.weighted_law(*law, *weight, sources))
                .collect(),
            Some(PhysicsComposition::Grouped(grouping)) => {
                vec![self.grouped_force(grouping, sources)]
            },
        };
        forces.extend(
            overlays
                .iter()
                .map(|overlay| self.overlay_force(*overlay, sources)),
        );
        forces
    }
}
