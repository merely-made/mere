/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::{cell::RefCell, rc::Rc};

use genet_scripted_dom::{NodeId, ScriptedDom};
use layout_dom_api::{LayoutDom, LocalName, Namespace, NodeKind};
use mere_surface_api::settings::{
    SettingControl, SettingMovement, SettingMutability, SettingScope, SettingSecurity, SettingSpec,
    SettingValue,
};
use meristem::AnyView;

use super::*;
use crate::setting_row_session::{
    SettingRowAckDisposition, SettingRowFailureKind, SettingRowObservation, SettingRowPhase,
};
use crate::{DomHandle, GenetAppRunner};

#[derive(Default)]
struct App {
    session: Option<SettingRowSession>,
    events: Vec<SettingRowEvent>,
}

fn spec(control: SettingControl, value: SettingValue) -> SettingSpec {
    SettingSpec {
        id: "managed.setting".into(),
        label: "Managed setting".into(),
        scope: SettingScope::Application,
        movement: SettingMovement::LocalOnly,
        mutability: SettingMutability::Live,
        security: SettingSecurity::Ordinary,
        control,
        value,
    }
}

fn text_spec(value: &str) -> SettingSpec {
    spec(SettingControl::Text, SettingValue::Text(value.into()))
}

type AppView = Box<dyn AnyView<App, (), GenetCtx, GenetElement>>;
type AppLogic = fn(&App) -> AppView;
type Runner = GenetAppRunner<App, AppLogic, AppView, ()>;

fn view(app: &App) -> AppView {
    let session = app.session.as_ref().expect("managed session").clone();
    let event_session = session.clone();
    Box::new(setting_row_managed(
        &session,
        "Managed setting",
        move |app: &mut App, event| {
            match &event {
                SettingRowEvent::Discard => event_session.reset_to_observed(),
                SettingRowEvent::ReviewMine => {
                    let _ = event_session.review_mine();
                },
                SettingRowEvent::Apply {
                    edit_generation, ..
                }
                | SettingRowEvent::RetryMine {
                    edit_generation, ..
                } => {
                    assert!(event_session.begin_apply(*edit_generation));
                },
                _ => {},
            }
            app.events.push(event);
        },
    ))
}

fn find_attr(dom: &ScriptedDom, root: NodeId, name: &str, value: &str) -> Option<NodeId> {
    if dom.kind(root) == NodeKind::Element
        && dom.attribute(root, &Namespace::from(""), &LocalName::from(name)) == Some(value)
    {
        return Some(root);
    }
    dom.dom_children(root)
        .find_map(|child| find_attr(dom, child, name, value))
}

fn runner_for(initial: SettingSpec) -> (DomHandle, Runner) {
    let dom = Rc::new(RefCell::new(ScriptedDom::new()));
    let app = App {
        session: Some(SettingRowSession::new(initial)),
        events: Vec::new(),
    };
    let runner = GenetAppRunner::new(dom.clone(), view as AppLogic, app);
    (dom, runner)
}

fn type_text(runner: &mut Runner, input: NodeId, text: &str) {
    runner.set_focus(Some(input));
    for ch in text.chars() {
        runner.dispatch_key(crate::KeyEvent::new(crate::Key::Character(ch.to_string())));
    }
}

#[test]
fn managed_text_edit_is_component_local_and_session_tracks_edit_generation() {
    let (dom, mut runner) = runner_for(text_spec("a"));
    let input = find_attr(&dom.borrow(), runner.root(), "role", "textbox").unwrap();
    type_text(&mut runner, input, "x");
    let session = runner.state().session.as_ref().unwrap().clone();
    let snapshot = session.snapshot();
    assert_eq!(snapshot.draft_value, SettingValue::Text("ax".into()));
    assert_eq!(snapshot.phase, SettingRowPhase::Dirty);
    assert_eq!(snapshot.edit_generation, 1);
    assert!(
        matches!(runner.state().events.as_slice(), [SettingRowEvent::Edited { value: SettingValue::Text(v), edit_generation: 1 }] if v == "ax")
    );

    let current = session.snapshot().baseline_spec;
    let refreshed = SettingSpec {
        value: SettingValue::Text("external".into()),
        ..current
    };
    runner.update(|app| {
        app.session.as_ref().unwrap().observe(refreshed);
    });
    assert_eq!(
        session.snapshot().draft_value,
        SettingValue::Text("ax".into()),
        "a dirty draft survives an external value snapshot"
    );
    assert!(session.snapshot().apply_compatible);
    assert_eq!(session.snapshot().phase, SettingRowPhase::Dirty);
    assert_eq!(
        dom.borrow().attribute(
            input,
            &Namespace::from(""),
            &LocalName::from("data-cambium-text-value")
        ),
        Some("ax")
    );
}

#[test]
fn navigation_is_not_an_edit_and_reverting_to_baseline_becomes_clean() {
    let (dom, mut runner) = runner_for(text_spec("a"));
    let session = runner.state().session.as_ref().unwrap().clone();
    let input = find_attr(&dom.borrow(), runner.root(), "role", "textbox").unwrap();
    runner.set_focus(Some(input));
    runner.dispatch_key(crate::KeyEvent::new(crate::Key::Named(
        crate::NamedKey::ArrowLeft,
    )));
    assert_eq!(session.snapshot().edit_generation, 0);
    assert_eq!(session.snapshot().phase, SettingRowPhase::Clean);

    runner.dispatch_key(crate::KeyEvent::new(crate::Key::Character("x".into())));
    assert_eq!(session.snapshot().phase, SettingRowPhase::Dirty);
    runner.dispatch_key(crate::KeyEvent::new(crate::Key::Named(
        crate::NamedKey::Backspace,
    )));
    assert_eq!(
        session.snapshot().draft_value,
        SettingValue::Text("a".into())
    );
    assert_eq!(session.snapshot().phase, SettingRowPhase::Clean);
    assert_eq!(session.snapshot().edit_generation, 2);
}

#[test]
fn clean_refresh_adopts_value_and_control_replacement_waits_for_explicit_reset() {
    let (dom, mut runner) = runner_for(text_spec("before"));
    let session = runner.state().session.as_ref().unwrap().clone();
    let mut clean_next = text_spec("after");
    clean_next.label = "Renamed".into();
    runner.update(|app| {
        assert_eq!(
            app.session.as_ref().unwrap().observe(clean_next),
            SettingRowObservation::AdoptedClean
        );
    });
    let input = find_attr(&dom.borrow(), runner.root(), "role", "textbox").unwrap();
    assert_eq!(
        session.snapshot().draft_value,
        SettingValue::Text("after".into())
    );
    assert_eq!(
        dom.borrow().attribute(
            input,
            &Namespace::from(""),
            &LocalName::from("data-cambium-text-value")
        ),
        Some("after")
    );

    runner.set_focus(Some(input));
    runner.dispatch_key(crate::KeyEvent::new(crate::Key::Character("!".into())));
    let incompatible = spec(SettingControl::Toggle, SettingValue::Boolean(false));
    runner.update(|app| {
        assert_eq!(
            app.session.as_ref().unwrap().observe(incompatible),
            SettingRowObservation::IncompatibleDraft
        );
    });
    assert_eq!(
        session.snapshot().draft_value,
        SettingValue::Text("after!".into())
    );
    assert!(!session.snapshot().apply_compatible);
    assert!(find_attr(&dom.borrow(), runner.root(), "role", "textbox").is_some());

    runner.update(|app| app.session.as_ref().unwrap().reset_to_observed());
    assert_eq!(session.snapshot().phase, SettingRowPhase::Clean);
    assert_eq!(session.snapshot().draft_value, SettingValue::Boolean(false));
    assert!(find_attr(&dom.borrow(), runner.root(), "role", "switch").is_some());
}

#[test]
fn canonical_ack_keeps_a_newer_edit_dirty() {
    let (dom, mut runner) = runner_for(text_spec("a"));
    let session = runner.state().session.as_ref().unwrap().clone();
    let input = find_attr(&dom.borrow(), runner.root(), "role", "textbox").unwrap();
    type_text(&mut runner, input, "x");
    let apply = dom.borrow().all_with_class(runner.root(), "setting-apply")[0];
    runner.dispatch_click(apply, crate::PointerClick::at((1.0, 1.0)));
    let generation = session.snapshot().pending_generation.unwrap();
    assert!(
        matches!(runner.state().events.last(), Some(SettingRowEvent::Apply { edit_generation, .. }) if *edit_generation == generation)
    );

    type_text(&mut runner, input, "y");
    let mut canonical = text_spec("ax");
    canonical.label = "Canonical".into();
    assert_eq!(
        session.acknowledge(generation, canonical),
        Some(SettingRowAckDisposition::KeepNewerDraft)
    );
    assert_eq!(
        session.snapshot().draft_value,
        SettingValue::Text("axy".into())
    );
    assert_eq!(session.snapshot().phase, SettingRowPhase::Dirty);
    assert_eq!(
        session.snapshot().baseline_spec.value,
        SettingValue::Text("ax".into())
    );
}

#[test]
fn clean_same_value_apply_remains_an_explicit_request() {
    let (dom, mut runner) = runner_for(text_spec("same value"));
    let session = runner.state().session.as_ref().unwrap().clone();
    let apply = dom.borrow().all_with_class(runner.root(), "setting-apply")[0];
    runner.dispatch_click(apply, crate::PointerClick::at((1.0, 1.0)));

    assert_eq!(session.snapshot().phase, SettingRowPhase::Applying);
    assert!(matches!(
        runner.state().events.last(),
        Some(SettingRowEvent::Apply {
            value: SettingValue::Text(value),
            edit_generation: 0,
        }) if value == "same value"
    ));
}

#[test]
fn one_pending_write_blocks_second_apply_and_reset_invalidates_late_ack() {
    let session = SettingRowSession::new(text_spec("base"));
    let first = session
        .record_edit(SettingValue::Text("one".into()))
        .unwrap();
    let second = session
        .record_edit(SettingValue::Text("two".into()))
        .unwrap();
    assert!(session.begin_apply(second));
    assert!(
        !session.begin_apply(second),
        "the same pending write is single-use"
    );

    session.reset(text_spec("two"));
    let reset_generation = session.snapshot().edit_generation;
    assert!(reset_generation > second);
    assert!(
        session.begin_apply(reset_generation),
        "clean Apply is explicit"
    );
    assert_eq!(
        session.acknowledge(second, text_spec("canonical old")),
        None,
        "an old completion cannot match a post-reset same-value Apply"
    );
    assert_eq!(
        session.acknowledge(reset_generation, text_spec("canonical new")),
        Some(SettingRowAckDisposition::Clean)
    );
    let _ = first;
}

#[test]
fn unavailable_session_recovers_only_when_clean_and_same_target_returns() {
    let clean = SettingRowSession::new(text_spec("old"));
    clean.set_unavailable("source offline");
    assert_eq!(
        clean.observe(text_spec("latest")),
        SettingRowObservation::AdoptedClean
    );
    assert_eq!(clean.snapshot().phase, SettingRowPhase::Clean);
    assert_eq!(
        clean.snapshot().draft_value,
        SettingValue::Text("latest".into())
    );

    let dirty = SettingRowSession::new(text_spec("old"));
    let _ = dirty.record_edit(SettingValue::Text("mine".into()));
    dirty.set_unavailable("source offline");
    assert_eq!(
        dirty.observe(text_spec("latest")),
        SettingRowObservation::PreservedDraft
    );
    assert_eq!(dirty.snapshot().phase, SettingRowPhase::Unavailable);
    assert_eq!(
        dirty.snapshot().draft_value,
        SettingValue::Text("mine".into())
    );
}

#[test]
fn conflict_requires_review_then_explicit_apply_or_use_current() {
    let (dom, mut runner) = runner_for(text_spec("a"));
    let session = runner.state().session.as_ref().unwrap().clone();
    let input = find_attr(&dom.borrow(), runner.root(), "role", "textbox").unwrap();
    type_text(&mut runner, input, "x");
    let mut observed = text_spec("external");
    observed.id = "replacement-setting".into();
    runner.update(|app| {
        app.session.as_ref().unwrap().observe(observed);
        app.session
            .as_ref()
            .unwrap()
            .fail(SettingRowFailureKind::Conflict, "Revision changed");
    });
    assert_eq!(
        session.snapshot().draft_value,
        SettingValue::Text("ax".into())
    );
    assert!(!session.snapshot().apply_compatible);

    // A conflict from a changed id cannot be retried through the old row.
    let use_current = dom
        .borrow()
        .all_with_class(runner.root(), "setting-use-current")[0];
    runner.dispatch_click(use_current, crate::PointerClick::at((1.0, 1.0)));
    assert_eq!(session.snapshot().baseline_spec.id, "replacement-setting");
    assert_eq!(
        session.snapshot().draft_value,
        SettingValue::Text("external".into())
    );
    assert!(matches!(
        runner.state().events.last(),
        Some(SettingRowEvent::Discard)
    ));
}

#[test]
fn compatible_conflict_requires_review_before_retry_event() {
    let (dom, mut runner) = runner_for(text_spec("a"));
    let session = runner.state().session.as_ref().unwrap().clone();
    let input = find_attr(&dom.borrow(), runner.root(), "role", "textbox").unwrap();
    type_text(&mut runner, input, "x");
    let mut observed = text_spec("external");
    observed.label = "Current value".into();
    runner.update(|app| {
        app.session.as_ref().unwrap().observe(observed);
        app.session
            .as_ref()
            .unwrap()
            .fail(SettingRowFailureKind::Conflict, "Revision changed");
    });
    assert!(session.snapshot().apply_compatible);

    let review = dom
        .borrow()
        .all_with_class(runner.root(), "setting-review-mine")[0];
    runner.dispatch_click(review, crate::PointerClick::at((1.0, 1.0)));
    assert_eq!(session.snapshot().phase, SettingRowPhase::ReviewingMine);
    let retry = dom
        .borrow()
        .all_with_class(runner.root(), "setting-retry-mine")[0];
    runner.dispatch_click(retry, crate::PointerClick::at((1.0, 1.0)));
    assert!(matches!(
        runner.state().events.last(),
        Some(SettingRowEvent::RetryMine {
            value: SettingValue::Text(value),
            ..
        }) if value == "ax"
    ));
    assert_eq!(session.snapshot().phase, SettingRowPhase::Applying);
}

#[test]
fn independent_sessions_do_not_share_draft_or_debug_values() {
    let a = SettingRowSession::new(text_spec("secret-a"));
    let b = SettingRowSession::new(text_spec("secret-b"));
    let _ = a.record_edit(SettingValue::Text("draft-a".into()));
    assert_eq!(
        b.snapshot().draft_value,
        SettingValue::Text("secret-b".into())
    );
    assert!(!format!("{a:?}").contains("draft-a"));
    assert!(!format!("{:?}", a.snapshot()).contains("draft-a"));
}

#[test]
fn persistence_failure_preserves_draft_and_write_baseline() {
    let session = SettingRowSession::new(text_spec("baseline"));
    let generation = session
        .record_edit(SettingValue::Text("draft".into()))
        .unwrap();
    assert!(session.begin_apply(generation));
    session.fail(SettingRowFailureKind::Persistence, "save failed");

    let snapshot = session.snapshot();
    assert_eq!(snapshot.draft_value, SettingValue::Text("draft".into()));
    assert_eq!(
        snapshot.baseline_spec.value,
        SettingValue::Text("baseline".into())
    );
    assert_eq!(snapshot.phase, SettingRowPhase::Failed);
    assert_eq!(snapshot.pending_generation, None);
}

#[test]
fn same_owner_resume_preserves_dirty_baseline_and_generation() {
    let session = SettingRowSession::new(text_spec("baseline"));
    let generation = session
        .record_edit(SettingValue::Text("draft".into()))
        .unwrap();
    session.set_unavailable("temporarily offline");
    assert_eq!(
        session.observe(text_spec("external")),
        SettingRowObservation::PreservedDraft
    );
    // The product calls resume only after confirming this is the same owner
    // incarnation. The helper itself does not adopt the observed value.
    let before = session.snapshot();
    assert_eq!(before.phase, SettingRowPhase::Unavailable);
    assert!(session.resume_available());
    let resumed = session.snapshot();
    assert_eq!(resumed.phase, SettingRowPhase::Dirty);
    assert_eq!(
        resumed.baseline_spec.value,
        SettingValue::Text("baseline".into())
    );
    assert_eq!(
        resumed.observed_spec.value,
        SettingValue::Text("external".into())
    );
    assert_eq!(resumed.draft_value, SettingValue::Text("draft".into()));
    assert_eq!(resumed.edit_generation, generation);
    assert_eq!(resumed.reset_generation, before.reset_generation);
}

#[test]
fn source_unavailable_observation_does_not_settle_pending_apply() {
    let session = SettingRowSession::new(text_spec("baseline"));
    let generation = session
        .record_edit(SettingValue::Text("draft".into()))
        .unwrap();
    assert!(session.begin_apply(generation));

    session.set_unavailable("source temporarily unavailable");
    let unavailable = session.snapshot();
    assert_eq!(unavailable.phase, SettingRowPhase::Unavailable);
    assert_eq!(unavailable.pending_generation, Some(generation));
    assert!(!session.begin_apply(generation));

    // A completed provider outcome, unlike a source observation, settles it.
    session.fail(SettingRowFailureKind::Unavailable, "request failed");
    assert_eq!(session.snapshot().pending_generation, None);
}

#[test]
fn choice_label_only_refresh_and_ack_keep_typed_editor_compatible() {
    let choice = |first_label: &str, second_label: &str| SettingSpec {
        control: SettingControl::Choice {
            options: vec![
                mere_surface_api::settings::SettingOption {
                    value: "a".into(),
                    label: first_label.into(),
                },
                mere_surface_api::settings::SettingOption {
                    value: "b".into(),
                    label: second_label.into(),
                },
            ],
        },
        ..spec(
            SettingControl::Choice { options: vec![] },
            SettingValue::Text("a".into()),
        )
    };
    let session = SettingRowSession::new(choice("Alpha", "Beta"));
    assert_eq!(session.record_edit(SettingValue::Text("b".into())), Some(1));
    let reset_generation = session.snapshot().reset_generation;
    let mut observed = choice("First", "Second");
    observed.value = SettingValue::Text("a".into());
    assert_eq!(
        session.observe(observed),
        SettingRowObservation::PreservedDraft
    );
    assert!(session.snapshot().apply_compatible);
    assert_eq!(session.snapshot().reset_generation, reset_generation);
    assert!(session.begin_apply(1));

    let mut canonical = choice("Canonical A", "Canonical B");
    canonical.value = SettingValue::Text("b".into());
    assert_eq!(
        session.acknowledge(1, canonical),
        Some(SettingRowAckDisposition::Clean)
    );
    let after = session.snapshot();
    assert_eq!(after.phase, SettingRowPhase::Clean);
    assert_eq!(after.draft_value, SettingValue::Text("b".into()));
    assert!(after.apply_compatible);
    assert_eq!(after.reset_generation, reset_generation);
}

#[derive(Default)]
struct PairApp {
    sessions: Option<[SettingRowSession; 2]>,
    events: Vec<(usize, SettingRowEvent)>,
}

fn pair_view(app: &PairApp) -> Box<dyn AnyView<PairApp, (), GenetCtx, GenetElement>> {
    let sessions = app.sessions.as_ref().unwrap();
    let first = sessions[0].clone();
    let second = sessions[1].clone();
    let a = setting_row_managed(&first, "First", |app: &mut PairApp, event| {
        app.events.push((0, event));
    });
    let b = setting_row_managed(&second, "Second", |app: &mut PairApp, event| {
        app.events.push((1, event));
    });
    Box::new(crate::el::<_, PairApp, ()>("div", (a, b)))
}

#[test]
fn adjacent_managed_controls_keep_distinct_session_identity() {
    let initial = spec(SettingControl::Toggle, SettingValue::Boolean(false));
    let sessions = [
        SettingRowSession::new(initial.clone()),
        SettingRowSession::new(initial),
    ];
    let app = PairApp {
        sessions: Some(sessions.clone()),
        events: Vec::new(),
    };
    let dom = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut runner = GenetAppRunner::new(dom.clone(), pair_view, app);
    let toggles = dom.borrow().all_with_class(runner.root(), "toggle");
    runner.dispatch_click(toggles[0], crate::PointerClick::at((1.0, 1.0)));

    assert_eq!(
        sessions[0].snapshot().draft_value,
        SettingValue::Boolean(true)
    );
    assert_eq!(
        sessions[1].snapshot().draft_value,
        SettingValue::Boolean(false)
    );
    assert!(matches!(
        runner.state().events.as_slice(),
        [(
            0,
            SettingRowEvent::Edited {
                value: SettingValue::Boolean(true),
                ..
            }
        )]
    ));
}

#[cfg(test)]
#[path = "setting_row_managed_review_tests.rs"]
mod review_tests;
