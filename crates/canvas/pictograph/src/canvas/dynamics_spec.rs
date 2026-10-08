// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The dynamics spec over the canvas's catalog (dynamics grammar plan, G4a).
//!
//! seiche holds the spec ([`seiche::spec`]); this module is its catalog of
//! presets: a law preset is a [`PhysicsLaw`] id, an overlay preset a
//! [`PhysicsOverlay`] id, the strings a saved scene already stores. A preset
//! is built over two nodes on two sites joined by an edge, which is enough
//! to read every law's declarations, Kinds' asymmetric matrix included
//! (over one site its matrix is 1 × 1 and reads as symmetric). Density's
//! word on overlays (F73) is the catalog's [`PhysicsLaw::admits`].
//!
//! Nothing here binds a spec to the canvas: G4b's binding does (F100).

use std::collections::HashMap;

use kernel::graph::NodeKey;
/// The signatures a spec's bars name.
pub use seiche::Observable;
pub use seiche::spec::*;
use seiche::{Admission, Force};

use super::physics_catalog::{LawInputs, LawSources, PhysicsLaw, PhysicsOverlay};

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
