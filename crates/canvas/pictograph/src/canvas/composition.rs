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

pub use seiche::{
    CHARGE_BETWEEN_WEIGHT, CompositionRefusal, GroupSource, PhysicsComposition, PhysicsGrouping,
};

use super::Canvas;
use super::physics_catalog::{LawInputs, LawSources};
use seiche::Force;

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
    #[cfg_attr(not(feature = "physics-engine"), allow(dead_code))]
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

/// The law currency and overlay admission tables in `seiche-model` against what
/// the built forces declare: the tables are only safe while this holds.
#[cfg(test)]
mod law_facts_drift {
    use std::collections::HashMap;

    use seiche::{Admission, Force, compose};

    use super::*;
    use crate::canvas::physics_catalog::{
        DENSITY_ADMITS, DENSITY_REFUSAL, PhysicsLaw, PhysicsOverlay,
    };

    fn bare_forces(law: PhysicsLaw) -> Vec<Box<dyn Force>> {
        LawInputs::from_parts(Vec::new(), Vec::new(), HashMap::new())
            .law_forces(law, LawSources::bare())
    }

    #[test]
    fn the_law_currency_table_matches_the_declared_terms() {
        for law in PhysicsLaw::ALL {
            assert_eq!(
                law.currency(),
                compose::currency_of(&bare_forces(law)),
                "{law:?}"
            );
        }
    }

    #[test]
    fn the_overlay_admission_table_matches_the_composer() {
        let inputs = LawInputs::from_parts(Vec::new(), Vec::new(), HashMap::new());
        for law in PhysicsLaw::ALL {
            for overlay in PhysicsOverlay::ALL {
                let expected = if law == PhysicsLaw::Density && !DENSITY_ADMITS.contains(&overlay)
                {
                    Admission::Refuse(DENSITY_REFUSAL)
                } else {
                    let force = inputs.overlay_force(overlay, LawSources::bare());
                    compose::admit_force(compose::currency_of(&bare_forces(law)), force.as_ref())
                };
                assert_eq!(law.admits(overlay), expected, "{law:?} with {overlay:?}");
            }
        }
    }
}
