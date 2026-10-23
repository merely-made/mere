// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The physics composition form both hosts render: what it offers, what it
//! reads back from the canvas's spec, what it refuses, and that an edit
//! survives the round trip through the canvas.

use super::*;
use crate::canvas::physics_controls::{
    self, SPEED_PRESETS, TwoLawGrouping, TwoLawMix, TwoStageSchedule,
};
use crate::canvas::physics_form::{
    Command, Control, FieldId, FormAction, PhysicsForm, Readback, STOP_FRAMES, STOP_LAW, Section,
    Slot,
};
use crate::canvas::{PhysicsKindSource, PhysicsLaw, PhysicsOverlay, Role, StageStop};

fn canvas() -> Canvas {
    let mut canvas = Canvas::with_sample_graph();
    canvas.resize(800, 600);
    canvas
}

fn picked(canvas: &Canvas) -> PhysicsChoice {
    PhysicsChoice::live(canvas)
}

fn press(form: &PhysicsForm, command: Command, canvas: &mut Canvas) -> FormOutcome {
    form.action(command, &picked(canvas))
        .expect("the form holds a valid request")
        .run(canvas)
}

use crate::canvas::physics_form::FormOutcome;

#[test]
fn a_fresh_form_holds_the_defaults() {
    let canvas = canvas();
    let form = PhysicsForm::new(&canvas);
    assert_eq!(
        form.value(FieldId::MixLaw(Slot::First)),
        PhysicsLaw::ALL[0].id()
    );
    assert_eq!(
        form.value(FieldId::MixLaw(Slot::Second)),
        PhysicsLaw::ALL[1].id()
    );
    assert_eq!(form.value(FieldId::MixShare(Slot::First)), "1");
    assert_eq!(form.value(FieldId::GroupOuter), PhysicsLaw::Charge.id());
    assert_eq!(form.value(FieldId::GroupInner), PhysicsLaw::Springs.id());
    assert_eq!(
        form.value(FieldId::GroupOuterShare),
        crate::canvas::composition::CHARGE_BETWEEN_WEIGHT.to_string()
    );
    assert_eq!(form.value(FieldId::StageStop(Slot::First)), "rest");
    assert_eq!(form.value(FieldId::StageFrames(Slot::Second)), "60");
    assert_eq!(form.value(FieldId::StageCapture(Slot::First)), "none");
    assert_eq!(form.value(FieldId::Speed), "1");
    assert_eq!(
        form.value(FieldId::Role),
        canvas.arrangement_roles().default.id()
    );
    for overlay in PhysicsOverlay::ALL {
        assert_eq!(form.value(FieldId::ScheduleOverlay(overlay)), "false");
    }
}

#[test]
fn field_choices_are_the_catalogs_and_ids_are_unique() {
    let form = PhysicsForm::new(&canvas());
    let mut keys = Vec::new();
    for section in Section::ALL {
        for group in section.groups() {
            for field in form.fields(&group) {
                assert_eq!(field.label, field.id.label());
                keys.push(field.id.key());
                match field.control {
                    Control::Choice { options, current } => {
                        assert!(
                            options.iter().any(|(id, _)| *id == current),
                            "{}",
                            field.label
                        );
                        let ids: Vec<_> = options.iter().map(|(id, _)| *id).collect();
                        match field.id {
                            FieldId::MixLaw(_)
                            | FieldId::GroupOuter
                            | FieldId::GroupInner
                            | FieldId::StageLaw(_) => {
                                assert_eq!(ids, PhysicsLaw::ALL.map(PhysicsLaw::id).to_vec())
                            },
                            FieldId::GroupPartition => assert_eq!(
                                ids,
                                PhysicsKindSource::ALL.map(PhysicsKindSource::id).to_vec()
                            ),
                            FieldId::StageCapture(_) => assert_eq!(ids.len(), Role::ALL.len() + 1),
                            _ => assert_eq!(ids.len(), 3),
                        }
                    },
                    Control::Number { .. } | Control::Toggle(_) => {},
                }
            }
        }
    }
    for id in [FieldId::Role, FieldId::RoleGroups, FieldId::Speed] {
        keys.push(id.key());
        let Control::Choice { options, .. } = form.field(id).control else {
            panic!("{id:?} is a choice");
        };
        assert!(!options.is_empty());
    }
    let count = keys.len();
    keys.sort();
    keys.dedup();
    assert_eq!(keys.len(), count, "every field has its own id");
    let Control::Choice { options, .. } = form.field(FieldId::Speed).control else {
        unreachable!()
    };
    assert_eq!(options, SPEED_PRESETS.to_vec());
}

#[test]
fn frames_matter_only_for_a_stop_that_counts_them() {
    let mut form = PhysicsForm::new(&canvas());
    let frames = FieldId::StageFrames(Slot::First);
    assert!(!form.field(frames).relevant);
    assert!(form.choose(FieldId::StageStop(Slot::First), STOP_FRAMES));
    assert!(form.field(frames).relevant);
    assert!(form.enter(frames, "12000"));
    assert!(form.choose(FieldId::StageStop(Slot::First), STOP_LAW));
    assert!(!form.field(frames).relevant);
    assert_eq!(form.value(frames), "12000", "what was typed is kept");
}

#[test]
fn edits_that_name_nothing_the_field_offers_change_nothing() {
    let mut form = PhysicsForm::new(&canvas());
    let before = form.clone();
    assert!(!form.choose(FieldId::MixLaw(Slot::First), "plasma"));
    assert!(!form.choose(FieldId::Speed, "3"));
    assert!(!form.choose(FieldId::StageStop(Slot::Second), "never"));
    assert!(!form.choose(FieldId::MixShare(Slot::First), "rest"));
    assert!(!form.enter(FieldId::MixLaw(Slot::First), "1"));
    assert!(!form.tick(FieldId::Role, true));
    assert_eq!(form, before);
}

#[test]
fn a_mix_reads_back_what_the_form_applied() {
    let mut canvas = canvas();
    let mut form = PhysicsForm::new(&canvas);
    assert!(form.choose(FieldId::MixLaw(Slot::First), PhysicsLaw::Charge.id()));
    assert!(form.choose(FieldId::MixLaw(Slot::Second), PhysicsLaw::Springs.id()));
    assert!(form.enter(FieldId::MixShare(Slot::First), "0.30"));
    assert!(form.enter(FieldId::MixShare(Slot::Second), "2"));
    let outcome = press(&form, Command::ApplyMix, &mut canvas);
    assert!(outcome.applied, "{}", outcome.status);

    let readback = Readback::of(&canvas);
    assert_eq!(
        readback.mix,
        Ok(Some(TwoLawMix {
            first: PhysicsLaw::Charge,
            second: PhysicsLaw::Springs,
            first_share: 0.3,
            second_share: 2.0,
        }))
    );
    assert_eq!(readback.note(Section::Mix), "");
    assert!(readback.summary(Section::Mix).contains("0.3"));
    assert_eq!(readback.note(Section::Grouping), "No authored grouping");

    // A new form over the same canvas shows the authored values, the shares
    // as the canvas holds them.
    let reopened = PhysicsForm::new(&canvas);
    assert_eq!(reopened.value(FieldId::MixShare(Slot::First)), "0.3");
    assert_eq!(reopened.value(FieldId::MixShare(Slot::Second)), "2");
    assert_eq!(
        reopened.value(FieldId::MixLaw(Slot::First)),
        PhysicsLaw::Charge.id()
    );
    assert_eq!(reopened.mix(), form_after_roundtrip(&form).mix());
}

/// The form after its own request was applied and read back.
fn form_after_roundtrip(form: &PhysicsForm) -> PhysicsForm {
    let mut canvas = canvas();
    let _ = form
        .action(Command::ApplyMix, &picked(&canvas))
        .unwrap()
        .run(&mut canvas);
    PhysicsForm::new(&canvas)
}

#[test]
fn a_grouping_round_trips_through_the_canvas() {
    let mut canvas = canvas();
    let mut form = PhysicsForm::new(&canvas);
    assert!(form.choose(FieldId::GroupPartition, PhysicsKindSource::Component.id()));
    assert!(form.choose(FieldId::GroupOuter, PhysicsLaw::Charge.id()));
    assert!(form.choose(FieldId::GroupInner, PhysicsLaw::Springs.id()));
    assert!(form.enter(FieldId::GroupOuterShare, "4.5"));
    assert!(press(&form, Command::ApplyGrouping, &mut canvas).applied);
    assert_eq!(
        Readback::of(&canvas).grouping,
        Ok(Some(TwoLawGrouping {
            partition: PhysicsKindSource::Component,
            outer: PhysicsLaw::Charge,
            inner: PhysicsLaw::Springs,
            outer_share: 4.5,
        }))
    );
    assert_eq!(PhysicsForm::new(&canvas).grouping(), form.grouping());
}

#[test]
fn a_schedule_round_trips_through_the_canvas() {
    let mut canvas = canvas();
    let mut form = PhysicsForm::new(&canvas);
    assert!(form.choose(FieldId::StageLaw(Slot::First), PhysicsLaw::Springs.id()));
    assert!(form.choose(FieldId::StageStop(Slot::First), STOP_FRAMES));
    assert!(form.enter(FieldId::StageFrames(Slot::First), "12000"));
    assert!(form.choose(FieldId::StageLaw(Slot::Second), PhysicsLaw::Charge.id()));
    assert!(form.choose(FieldId::StageStop(Slot::Second), STOP_LAW));
    assert!(form.choose(FieldId::StageCapture(Slot::Second), Role::Pinned.id()));
    assert!(form.tick(FieldId::ScheduleOverlay(PhysicsOverlay::GravityLocus), true));
    let outcome = press(&form, Command::ApplySchedule, &mut canvas);
    assert!(outcome.applied, "{}", outcome.status);

    let Ok(Some(schedule)) = Readback::of(&canvas).schedule else {
        panic!("the canvas authored the schedule");
    };
    assert_eq!(schedule.first.stop, StageStop::Frames(12000));
    assert_eq!(schedule.second.stop, StageStop::LawDone);
    assert_eq!(schedule.second.capture, Some(Role::Pinned));
    assert_eq!(schedule.overlays, vec![PhysicsOverlay::GravityLocus]);

    let reopened = PhysicsForm::new(&canvas);
    assert_eq!(reopened.schedule(), form.schedule());
    assert_eq!(reopened.value(FieldId::StageFrames(Slot::First)), "12000");
    assert_eq!(
        reopened.value(FieldId::ScheduleOverlay(PhysicsOverlay::GravityLocus)),
        "true"
    );
    // Untick, and the overlay leaves the request.
    let mut form = reopened;
    assert!(form.tick(
        FieldId::ScheduleOverlay(PhysicsOverlay::GravityLocus),
        false
    ));
    assert_eq!(form.schedule().unwrap().overlays, Vec::new());
}

#[test]
fn role_role_groups_and_speed_follow_and_apply() {
    let mut canvas = canvas();
    let mut form = PhysicsForm::new(&canvas);
    assert!(form.choose(FieldId::Role, Role::Pinned.id()));
    assert!(form.choose(FieldId::RoleGroups, PhysicsKindSource::Component.id()));
    assert!(form.choose(FieldId::Speed, "5"));
    assert!(press(&form, Command::ApplyRole, &mut canvas).applied);
    assert!(press(&form, Command::ApplyRoleGroups, &mut canvas).applied);
    let outcome = form.speed_action().run(&mut canvas);
    assert_eq!(outcome.status, "Speed set to 5x");

    let readback = Readback::of(&canvas);
    assert_eq!(readback.role, Role::Pinned);
    assert_eq!(readback.role_groups, PhysicsKindSource::Component);
    assert_eq!(readback.speed_id(), "5");

    // Following the roles leaves the speed pick alone until asked.
    let mut other = PhysicsForm::new(&canvas);
    other.choose(FieldId::Speed, "10");
    other.follow_roles(&readback);
    assert_eq!(other.value(FieldId::Speed), "10");
    other.follow_speed(&readback);
    assert_eq!(other.value(FieldId::Speed), "5");
}

#[test]
fn an_unauthored_or_richer_spec_leaves_the_values_and_says_why() {
    let mut canvas = canvas();
    let mut form = PhysicsForm::new(&canvas);
    form.enter(FieldId::MixShare(Slot::First), "7");
    let readback = Readback::of(&canvas);
    form.follow_composition(&readback);
    assert_eq!(form.value(FieldId::MixShare(Slot::First)), "7");
    assert_eq!(
        readback.note(Section::Schedule),
        "No authored two-stage schedule"
    );

    // A mix with a per-part overlay is not the form's shape.
    let mut spec = canvas.dynamics_spec().unwrap();
    spec.root = crate::canvas::dynamics_spec::Node::Mix {
        parts: vec![
            crate::canvas::dynamics_spec::Node::preset(PhysicsLaw::Springs.id()),
            crate::canvas::dynamics_spec::Node::preset(PhysicsLaw::Charge.id()),
            crate::canvas::dynamics_spec::Node::preset(PhysicsLaw::Flow.id()),
        ],
        weight: 1.0,
        overlays: Vec::new(),
    };
    canvas.set_dynamics_spec(&spec).unwrap();
    let readback = Readback::of(&canvas);
    let note = readback.note(Section::Mix);
    assert!(
        note.starts_with("Unsupported authored mix shape · part count"),
        "{note}"
    );
    assert!(note.ends_with("editor values are not readback"));
    form.follow_composition(&readback);
    assert_eq!(form.value(FieldId::MixShare(Slot::First)), "7");
}

#[test]
fn readbacks_differ_only_when_the_composition_does() {
    let mut canvas = canvas();
    let before = Readback::of(&canvas);
    physics_controls::apply_role(&mut canvas, Role::Pinned).unwrap();
    let after_role = Readback::of(&canvas);
    assert!(before.same_composition(&after_role));
    let form = PhysicsForm::new(&canvas);
    press(&form, Command::ApplyMix, &mut canvas);
    assert!(!after_role.same_composition(&Readback::of(&canvas)));
}

#[test]
fn the_form_refuses_what_it_cannot_make_into_a_request() {
    let mut form = PhysicsForm::new(&canvas());
    let flat = picked(&canvas());
    form.enter(FieldId::MixShare(Slot::Second), "lots");
    assert_eq!(
        form.action(Command::ApplyMix, &flat),
        Err("Mix shares must be numbers".into())
    );
    form.enter(FieldId::GroupOuterShare, "");
    assert_eq!(
        form.action(Command::ApplyGrouping, &flat),
        Err("Grouping outer share must be a number".into())
    );
    form.choose(FieldId::StageStop(Slot::First), STOP_FRAMES);
    for bad in ["0", "-3", "12.5", "many", ""] {
        form.enter(FieldId::StageFrames(Slot::First), bad);
        assert_eq!(
            form.action(Command::ApplySchedule, &flat),
            Err("Schedule refused · First stage frames must be a positive integer".into()),
            "{bad:?}"
        );
    }
    form.enter(FieldId::StageFrames(Slot::First), "30");
    form.choose(FieldId::StageStop(Slot::Second), STOP_FRAMES);
    form.enter(FieldId::StageFrames(Slot::Second), "0");
    assert_eq!(
        form.action(Command::ApplySchedule, &flat),
        Err("Schedule refused · Second stage frames must be a positive integer".into())
    );
}

#[test]
fn the_canvas_refusing_a_request_is_a_worded_status_and_changes_nothing() {
    let mut canvas = canvas();
    let before = canvas.dynamics_spec().unwrap();
    let mut form = PhysicsForm::new(&canvas);
    form.choose(FieldId::MixLaw(Slot::First), PhysicsLaw::Anneal.id());
    let outcome = press(&form, Command::ApplyMix, &mut canvas);
    assert!(!outcome.applied);
    assert!(
        outcome.status.starts_with("Mix refused · "),
        "{}",
        outcome.status
    );
    assert_eq!(canvas.dynamics_spec().unwrap(), before);

    form.choose(FieldId::MixLaw(Slot::First), PhysicsLaw::Springs.id());
    form.enter(FieldId::MixShare(Slot::First), "NaN");
    let outcome = press(&form, Command::ApplyMix, &mut canvas);
    assert!(
        outcome.status.starts_with("Mix refused · "),
        "{}",
        outcome.status
    );

    // An unknown speed id is refused by name.
    let outcome = FormAction::Speed("3").run(&mut canvas);
    assert_eq!(outcome.status, "Speed refused · unknown speed 3");
}

#[test]
fn use_single_law_replaces_the_composition_with_the_picked_choice() {
    let mut canvas = canvas();
    let form = PhysicsForm::new(&canvas);
    press(&form, Command::ApplyMix, &mut canvas);
    assert!(matches!(Readback::of(&canvas).mix, Ok(Some(_))));
    let outcome = press(&form, Command::UseSingleLaw, &mut canvas);
    assert!(outcome.applied, "{}", outcome.status);
    assert_eq!(Readback::of(&canvas).mix, Ok(None));
    assert!(FormAction::SingleLaw(picked(&canvas)).changes_law());
    assert!(!FormAction::Role(Role::Pinned).changes_law());
}

#[test]
fn the_requests_the_form_makes_are_the_typed_inputs() {
    let form = PhysicsForm::new(&canvas());
    assert_eq!(
        form.mix(),
        Ok(TwoLawMix {
            first: PhysicsLaw::ALL[0],
            second: PhysicsLaw::ALL[1],
            first_share: 1.0,
            second_share: 1.0,
        })
    );
    assert_eq!(
        form.schedule(),
        Ok(TwoStageSchedule {
            first: physics_controls::StageDraft {
                law: PhysicsLaw::ALL[0],
                stop: StageStop::Rest,
                capture: None,
            },
            second: physics_controls::StageDraft {
                law: PhysicsLaw::ALL[1],
                stop: StageStop::Rest,
                capture: None,
            },
            overlays: Vec::new(),
        })
    );
}
