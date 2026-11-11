// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The dynamics spec over the catalog (G4a): every law and overlay as a
//! preset, and every raw kind against the force the catalog builds, row by
//! row (F112).

use std::collections::HashSet;

use seiche::spec::raw::*;
use seiche::{Class, Currency, Force, Term, compose};

use super::*;
use crate::canvas::dynamics_spec::{self as spec, CanvasCatalog};
use crate::canvas::physics_catalog::{
    CHARGE_STRENGTH, DENSITY_REFUSAL, LAW_SEED, LawInputs, LawSources, ORBIT_CENTRING,
    ORBIT_EXCLUSION_REACH, PhysicsLaw, PhysicsOverlay,
};
use spec::{DynamicsSpec, Node, PresetAt, SpecCatalog, SpecError};

/// Twelve nodes on three sites, a ring with chords: the catalog builds
/// every law over it as a host would.
fn fixture() -> LawInputs<'static> {
    let keys: Vec<NodeKey> = (0..12).map(NodeKey::new).collect();
    let mut edges: Vec<(NodeKey, NodeKey)> =
        (0..12).map(|i| (keys[i], keys[(i + 1) % 12])).collect();
    edges.extend((0..12).step_by(3).map(|i| (keys[i], keys[(i + 6) % 12])));
    let sites = keys
        .iter()
        .enumerate()
        .map(|(i, &k)| (k, format!("s{}.test", i % 3)))
        .collect();
    LawInputs::from_parts(keys, edges, sites)
}

fn ch() -> Option<String> {
    Some("test.channel".into())
}

/// Each law's forces as raw terms with the catalog's own settings, in the
/// catalog's order (`physics_catalog.rs`, `law_forces`).
fn law_as_raw(law: PhysicsLaw) -> Vec<RawTerm> {
    let exclusion = || RawTerm::NodeExclusion(Default::default());
    let spring = || RawTerm::EdgeSpring(Default::default());
    let boundary = || RawTerm::Boundary(Default::default());
    match law {
        PhysicsLaw::Springs => vec![exclusion(), spring(), boundary()],
        PhysicsLaw::Charge => vec![
            RawTerm::BarnesHutRepulsion(BarnesHutRepulsionParams {
                strength: f64::from(CHARGE_STRENGTH),
                ..Default::default()
            }),
            spring(),
            boundary(),
        ],
        PhysicsLaw::Stress => vec![
            exclusion(),
            RawTerm::StressSpring(StressSpringParams {
                pairs: ch(),
                unit_length: Some(170.0),
                ..Default::default()
            }),
            boundary(),
        ],
        PhysicsLaw::Energy => vec![exclusion(), RawTerm::Linlog(Default::default())],
        PhysicsLaw::Orbit => vec![
            RawTerm::NodeExclusion(NodeExclusionParams {
                cutoff: f64::from(ORBIT_EXCLUSION_REACH),
                ..Default::default()
            }),
            RawTerm::Gravity(GravityParams {
                masses: ch(),
                counter_damping: Some(CounterDampingParam::Tangential),
                ..Default::default()
            }),
            RawTerm::Boundary(BoundaryParams {
                strength: f64::from(ORBIT_CENTRING),
            }),
        ],
        PhysicsLaw::Kinds => vec![
            exclusion(),
            RawTerm::ParticleLife(ParticleLifeParams {
                kinds: ch(),
                ..Default::default()
            }),
        ],
        PhysicsLaw::Flock => vec![exclusion(), RawTerm::Boids(Default::default())],
        PhysicsLaw::Sync => vec![
            exclusion(),
            RawTerm::Kuramoto(KuramotoParams {
                default_radius: 240.0,
                ..Default::default()
            }),
        ],
        PhysicsLaw::Flow => vec![
            exclusion(),
            RawTerm::MagneticSpring(Default::default()),
            boundary(),
        ],
        PhysicsLaw::Anneal => vec![RawTerm::Anneal(Default::default())],
        PhysicsLaw::Still => vec![RawTerm::Hold(HoldParams {})],
        PhysicsLaw::Density => vec![RawTerm::Density(DensityParams {
            masses: ch(),
            resolution: Some(64),
            seconds: 1.0,
            initial_blur: 0.25,
            stop: DensityStopParam::Shift(0.05),
            patience: 3,
            max_passes: 120,
            min_passes: 60,
            ..Default::default()
        })],
    }
}

/// Each overlay's force as a raw term (`overlay_force`).
fn overlay_as_raw(overlay: PhysicsOverlay) -> RawTerm {
    let locus = |oscillation| {
        RawTerm::GravityLocus(GravityLocusParams {
            target: Some((0.0, 0.0)),
            oscillation,
            ..Default::default()
        })
    };
    match overlay {
        PhysicsOverlay::DegreeRepulsion => RawTerm::DegreeRepulsion(Default::default()),
        PhysicsOverlay::DomainCluster => RawTerm::DomainCluster(DomainClusterParams {
            groups: ch(),
            ..Default::default()
        }),
        PhysicsOverlay::HubGravity => RawTerm::HubGravity(Default::default()),
        PhysicsOverlay::DepthGravity => RawTerm::DepthGravity(DepthGravityParams {
            depths: ch(),
            ..Default::default()
        }),
        PhysicsOverlay::GridSnap => RawTerm::GridSnap(Default::default()),
        PhysicsOverlay::GravityLocus => locus(None),
        PhysicsOverlay::Tide => locus(Some((240.0, 24.0))),
        PhysicsOverlay::Skeleton => RawTerm::StressSpring(StressSpringParams {
            pairs: ch(),
            unit_length: Some(170.0),
            // The catalog's `SKELETON_STIFFNESS`; a spring's rows do not read it.
            stiffness: 60.0,
        }),
    }
}

fn rows(forces: &[Box<dyn Force>]) -> Vec<Term> {
    forces.iter().flat_map(|f| f.terms()).collect()
}

/// F112: every raw kind's declared rows match the force the catalog builds
/// where it uses that type, law by law and overlay by overlay; all 20 types
/// are reached.
#[test]
fn every_raw_kind_matches_the_catalog_built_force_row_by_row() {
    let inputs = fixture();
    let mut reached = HashSet::new();
    for law in PhysicsLaw::ALL {
        let built = inputs.law_forces(law, LawSources::bare());
        let raw = law_as_raw(law);
        assert_eq!(
            built.len(),
            raw.len(),
            "{}: one raw term per force",
            law.id()
        );
        for (k, (force, term)) in built.iter().zip(&raw).enumerate() {
            let from_raw = term.build("root", LAW_SEED).expect("filled");
            assert_eq!(
                from_raw.terms(),
                force.terms(),
                "{}[{k}] as {}",
                law.id(),
                term.kind()
            );
            reached.insert(term.kind());
        }
    }
    for overlay in PhysicsOverlay::ALL {
        let force = inputs.overlay_force(overlay, LawSources::bare());
        let term = overlay_as_raw(overlay);
        let from_raw = term.build("root", LAW_SEED).expect("filled");
        assert_eq!(
            from_raw.terms(),
            force.terms(),
            "{} as {}",
            overlay.id(),
            term.kind()
        );
        reached.insert(term.kind());
    }
    let mut reached: Vec<_> = reached.into_iter().collect();
    reached.sort();
    let mut kinds = RawTerm::KINDS.to_vec();
    kinds.sort();
    assert_eq!(reached, kinds, "the catalog builds all 20 types");
}

#[test]
fn every_law_and_overlay_is_a_preset_whose_rows_are_the_catalogs() {
    let inputs = fixture();
    for law in PhysicsLaw::ALL {
        let derived = spec::derive(&DynamicsSpec::new(Node::preset(law.id())))
            .unwrap_or_else(|e| panic!("{}: {e}", law.id()));
        let derived_rows: Vec<Term> = derived.leaves[0].terms.iter().map(|t| t.term).collect();
        assert_eq!(
            derived_rows,
            rows(&inputs.law_forces(law, LawSources::bare())),
            "{}",
            law.id()
        );
        assert_eq!(derived.leaves[0].currency, law.currency(), "{}", law.id());
        let names: Vec<_> = derived_rows.iter().map(|t| t.name).collect();
        let distinct: HashSet<_> = names.iter().collect();
        assert_eq!(
            distinct.len(),
            names.len(),
            "{}: term names are distinct, so an override by name is unambiguous",
            law.id()
        );
    }
    for overlay in PhysicsOverlay::ALL {
        let mut root = Node::preset(PhysicsLaw::Springs.id());
        root.overlays_mut().push(Node::preset(overlay.id()));
        let derived = spec::derive(&DynamicsSpec::new(root))
            .unwrap_or_else(|e| panic!("{}: {e}", overlay.id()));
        assert_eq!(derived.leaves[1].path, "root.overlays[0]");
    }
    // Kinds reads its asymmetric matrix, as the catalog builds it for a host.
    let kinds = CanvasCatalog
        .preset(PhysicsLaw::Kinds.id(), PresetAt::Law)
        .unwrap();
    assert!(
        rows(&kinds)
            .iter()
            .any(|t| t.name == "kind matrix" && t.class == Class::N)
    );
    assert_eq!(
        LAW_SEED,
        spec::DEFAULT_SEED,
        "a spec without a seed runs on the catalog's"
    );
}

#[test]
fn ids_fail_where_the_catalog_does_not_know_them() {
    let unknown = spec::derive(&DynamicsSpec::new(Node::preset("charge.coulomb"))).unwrap_err();
    assert_eq!(
        unknown,
        SpecError::UnknownPreset {
            path: "root".into(),
            id: "charge.coulomb".into(),
            at: PresetAt::Law,
        }
    );
    let mut root = Node::preset(PhysicsLaw::Springs.id());
    root.overlays_mut()
        .push(Node::preset(PhysicsLaw::Charge.id()));
    assert!(matches!(
        spec::derive(&DynamicsSpec::new(root)),
        Err(SpecError::UnknownPreset {
            at: PresetAt::Overlay,
            ..
        })
    ));
}

#[test]
fn density_takes_the_overlays_it_admits_and_refuses_the_rest() {
    let density = |overlay: PhysicsOverlay| {
        let mut root = Node::preset(PhysicsLaw::Density.id());
        root.overlays_mut().push(Node::preset(overlay.id()));
        spec::derive(&DynamicsSpec::new(root))
    };
    assert!(density(PhysicsOverlay::GravityLocus).is_ok());
    assert!(density(PhysicsOverlay::Tide).is_ok());
    assert_eq!(
        density(PhysicsOverlay::DomainCluster).unwrap_err(),
        SpecError::Currency {
            path: "root.overlays[0]".into(),
            reason: DENSITY_REFUSAL,
        }
    );
}

#[test]
fn the_catalogs_compositions_derive_and_a_writing_law_in_a_mix_is_refused() {
    let between = Node::Grouped {
        partition: "groups.meaning".into(),
        outer: Box::new(Node::Preset {
            id: PhysicsLaw::Charge.id().into(),
            weight: f64::from(crate::canvas::composition::CHARGE_BETWEEN_WEIGHT),
            terms: Default::default(),
            rung: None,
            overlays: Vec::new(),
        }),
        inner: Box::new(Node::preset(PhysicsLaw::Springs.id())),
        weight: 1.0,
        overlays: Vec::new(),
    };
    assert!(spec::derive(&DynamicsSpec::new(between)).is_ok());
    for law in PhysicsLaw::ALL {
        let mix = Node::Mix {
            parts: vec![
                Node::preset(PhysicsLaw::Springs.id()),
                Node::preset(law.id()),
            ],
            weight: 1.0,
            overlays: Vec::new(),
        };
        let derived = spec::derive(&DynamicsSpec::new(mix));
        if law.currency() == Currency::Force {
            assert!(derived.is_ok(), "{}: {derived:?}", law.id());
        } else {
            assert_eq!(
                derived.unwrap_err(),
                SpecError::Currency {
                    path: "root.parts[1]".into(),
                    reason: compose::UNWEIGHTED,
                },
                "{}",
                law.id()
            );
        }
    }
}
