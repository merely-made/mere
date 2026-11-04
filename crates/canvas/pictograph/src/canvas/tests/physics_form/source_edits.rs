// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

#[test]
fn readback_names_flat_charge_gas_and_composed_roots_custom() {
    let mut flat_canvas = canvas();
    let mut flat_spec = flat_canvas.dynamics_spec().unwrap();
    let mut choice = PhysicsChoice::live(&flat_canvas);
    choice.law = PhysicsLaw::Charge;
    choice.overlays.clear();
    choice.write_into(&mut flat_spec, &PhysicsChoice::live(&flat_canvas));
    flat_canvas.set_dynamics_spec(&flat_spec).unwrap();
    assert_eq!(Readback::of(&flat_canvas).physics_profile, "gas");
    assert_eq!(PhysicsForm::new(&flat_canvas).profile_id(), "gas");

    let mut mix_canvas = canvas();
    physics_controls::apply_mix(
        &mut mix_canvas,
        &TwoLawMix {
            first: PhysicsLaw::Charge,
            second: PhysicsLaw::Springs,
            first_share: 1.0,
            second_share: 1.0,
        },
    )
    .unwrap();
    let mix_readback = Readback::of(&mix_canvas);
    assert_eq!(mix_readback.physics.law, PhysicsLaw::Charge);
    assert_eq!(
        mix_readback.physics_profile,
        physics_controls::CUSTOM_PROFILE
    );

    let mut grouping_canvas = canvas();
    physics_controls::apply_grouping(
        &mut grouping_canvas,
        &TwoLawGrouping {
            partition: PhysicsKindSource::Site,
            outer: PhysicsLaw::Charge,
            inner: PhysicsLaw::Springs,
            outer_share: 1.0,
        },
    )
    .unwrap();
    assert_eq!(
        Readback::of(&grouping_canvas).physics_profile,
        physics_controls::CUSTOM_PROFILE
    );

    let mut schedule_canvas = canvas();
    physics_controls::apply_two_stage_schedule(
        &mut schedule_canvas,
        &TwoStageSchedule {
            first: StageDraft {
                law: PhysicsLaw::Charge,
                stop: StageStop::Rest,
                capture: None,
            },
            second: StageDraft {
                law: PhysicsLaw::Springs,
                stop: StageStop::Rest,
                capture: None,
            },
            overlays: vec![],
        },
    )
    .unwrap();
    assert_eq!(
        Readback::of(&schedule_canvas).physics_profile,
        physics_controls::CUSTOM_PROFILE
    );
}

#[test]
fn typed_source_only_edit_keeps_composed_root_and_law_edit_replaces_it() {
    let mut canvas = canvas();
    physics_controls::apply_mix(
        &mut canvas,
        &TwoLawMix {
            first: PhysicsLaw::Charge,
            second: PhysicsLaw::Springs,
            first_share: 0.4,
            second_share: 0.6,
        },
    )
    .unwrap();
    let original = canvas.dynamics_spec().unwrap();
    let mut form = PhysicsForm::new(&canvas);
    assert!(form.choose(FieldId::PhysicsKind, PhysicsKindSource::Cluster.id()));
    let choice = form.physics_choice();
    assert_eq!(choice.kind, PhysicsKindSource::Cluster);
    physics_controls::apply_physics(&mut canvas, &choice);
    let after_source = canvas.dynamics_spec().unwrap();
    assert_eq!(after_source.root, original.root);
    assert_eq!(after_source.seed, original.seed);
    assert_eq!(after_source.target, original.target);

    assert!(form.choose(FieldId::PhysicsLaw, PhysicsLaw::Springs.id()));
    physics_controls::apply_physics(&mut canvas, &form.physics_choice());
    let after_law = canvas.dynamics_spec().unwrap();
    assert_ne!(after_law.root, original.root);
    assert_eq!(after_law.seed, original.seed);
    assert_eq!(after_law.target, original.target);
}

#[test]
fn no_op_flat_picks_keep_custom_readback() {
    let mut mix_canvas = canvas();
    physics_controls::apply_mix(
        &mut mix_canvas,
        &TwoLawMix {
            first: PhysicsLaw::Charge,
            second: PhysicsLaw::Springs,
            first_share: 1.0,
            second_share: 1.0,
        },
    )
    .unwrap();
    let mut form = PhysicsForm::new(&mix_canvas);
    assert_eq!(form.profile_id(), physics_controls::CUSTOM_PROFILE);
    assert!(form.choose(FieldId::PhysicsLaw, PhysicsLaw::Charge.id()));
    assert!(form.tick(FieldId::PhysicsOverlay(PhysicsOverlay::GravityLocus), false));
    assert_eq!(form.profile_id(), physics_controls::CUSTOM_PROFILE);
}

#[test]
fn readback_and_source_only_edit_preserve_authored_overlay_run_order() {
    let authored_overlays = [
        PhysicsOverlay::DomainCluster,
        PhysicsOverlay::DegreeRepulsion,
    ];
    let mut canvas = canvas();
    let mut spec = canvas.dynamics_spec().unwrap();
    let running = PhysicsChoice::live(&canvas);
    let mut flat = running.clone();
    flat.law = PhysicsLaw::Springs;
    flat.overlays = authored_overlays.to_vec();
    flat.write_into(&mut spec, &running);
    canvas.set_dynamics_spec(&spec).unwrap();

    physics_controls::apply_mix(
        &mut canvas,
        &TwoLawMix {
            first: PhysicsLaw::Charge,
            second: PhysicsLaw::Springs,
            first_share: 0.4,
            second_share: 0.6,
        },
    )
    .unwrap();
    let before = canvas.dynamics_spec().unwrap();
    let mut form = PhysicsForm::new(&canvas);
    assert_eq!(form.physics_choice().overlays, authored_overlays.to_vec());
    assert_eq!(form.physics_choice(), Readback::of(&canvas).physics);
    assert_eq!(form.profile_id(), physics_controls::CUSTOM_PROFILE);

    assert!(form.choose(FieldId::PhysicsKind, PhysicsKindSource::Cluster.id()));
    physics_controls::apply_physics(&mut canvas, &form.physics_choice());
    let after = canvas.dynamics_spec().unwrap();
    assert_eq!(after.root, before.root);
    assert_eq!(after.seed, before.seed);
    assert_eq!(after.target, before.target);
    assert_eq!(
        PhysicsChoice::live(&canvas).overlays,
        authored_overlays.to_vec()
    );
    assert_eq!(
        PhysicsForm::new(&canvas).profile_id(),
        physics_controls::CUSTOM_PROFILE
    );
}

#[test]
fn source_only_edits_keep_mix_grouping_and_schedule_roots_and_overlay_order() {
    let authored_overlays = [
        PhysicsOverlay::DomainCluster,
        PhysicsOverlay::DegreeRepulsion,
    ];
    let mut mix_canvas = canvas();
    install_authored_overlay_order(&mut mix_canvas, &authored_overlays);
    physics_controls::apply_mix(
        &mut mix_canvas,
        &TwoLawMix {
            first: PhysicsLaw::Charge,
            second: PhysicsLaw::Springs,
            first_share: 0.4,
            second_share: 0.6,
        },
    )
    .unwrap();
    assert_source_only_keeps_root_and_order(&mut mix_canvas, &authored_overlays);

    let mut grouping_canvas = canvas();
    install_authored_overlay_order(&mut grouping_canvas, &authored_overlays);
    physics_controls::apply_grouping(
        &mut grouping_canvas,
        &TwoLawGrouping {
            partition: PhysicsKindSource::Site,
            outer: PhysicsLaw::Charge,
            inner: PhysicsLaw::Springs,
            outer_share: 0.5,
        },
    )
    .unwrap();
    assert_source_only_keeps_root_and_order(&mut grouping_canvas, &authored_overlays);

    let mut schedule_canvas = canvas();
    let schedule_overlays = [
        PhysicsOverlay::DegreeRepulsion,
        PhysicsOverlay::DomainCluster,
    ];
    install_authored_overlay_order(&mut schedule_canvas, &schedule_overlays);
    physics_controls::apply_two_stage_schedule(
        &mut schedule_canvas,
        &TwoStageSchedule {
            first: StageDraft {
                law: PhysicsLaw::Charge,
                stop: StageStop::Rest,
                capture: None,
            },
            second: StageDraft {
                law: PhysicsLaw::Springs,
                stop: StageStop::Rest,
                capture: None,
            },
            overlays: schedule_overlays.to_vec(),
        },
    )
    .unwrap();
    assert_source_only_keeps_root_and_order(&mut schedule_canvas, &schedule_overlays);
}

#[test]
fn matching_profile_keeps_composition_until_single_law_is_explicit() {
    let mut canvas = canvas();
    let mut flat = PhysicsChoice::live(&canvas);
    flat.overlays.clear();
    physics_controls::apply_physics(&mut canvas, &flat);
    physics_controls::apply_mix(
        &mut canvas,
        &TwoLawMix {
            first: PhysicsLaw::Charge,
            second: PhysicsLaw::Springs,
            first_share: 1.0,
            second_share: 1.0,
        },
    )
    .unwrap();
    let before = canvas.dynamics_spec().unwrap();
    let mut form = PhysicsForm::new(&canvas);
    assert!(form.choose(FieldId::PhysicsProfile, "gas"));
    assert_eq!(
        canvas.dynamics_spec().unwrap(),
        before,
        "a form pick is staged"
    );
    physics_controls::apply_physics(&mut canvas, &form.physics_choice());
    assert_eq!(canvas.dynamics_spec().unwrap().root, before.root);
    assert_eq!(
        Readback::of(&canvas).physics_profile,
        physics_controls::CUSTOM_PROFILE
    );

    form.action(Command::UseSingleLaw).unwrap().run(&mut canvas);
    assert_eq!(physics_controls::read_two_law_mix(&canvas), Ok(None));
    assert_eq!(Readback::of(&canvas).physics_profile, "gas");
}
