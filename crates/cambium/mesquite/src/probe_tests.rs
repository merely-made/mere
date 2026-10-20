// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The actual Probe, retained host geometry and wheel callback, without an OS
//! window or GPU. The product records application effects, not driver guesses.

use super::*;
use apparatus::{Cursor, ObservationMetadata, ObservationStore, RetentionLimits};
use cambium::{AnyView, GenetCtx, GenetElement, el, on_wheel};
use cambium_rootstock::{Hook, Host, HostHooks, HostOptions, HostState, HostWake, Runner};
use genet_scripted_dom::ScriptedDom;
use std::{
    cell::{Cell, RefCell},
    collections::BTreeMap,
    rc::Rc,
    sync::Arc,
    time::Duration,
};

type Store = Rc<RefCell<ObservationStore<&'static str>>>;
struct State {
    delivered: u32,
    store: Store,
}
type View = Box<dyn AnyView<State, (), GenetCtx, GenetElement>>;
type Logic = fn(&State) -> View;
fn view(_: &State) -> View {
    Box::new(on_wheel(
        el("div", "wheel target").attr("class", "wheel"),
        |state: &mut State, event: cambium::WheelEvent| {
            event.prevent_default();
            state.delivered += 1;
            state
                .store
                .borrow_mut()
                .record(
                    "wheel-applied",
                    15,
                    ObservationMetadata {
                        operation: Some(format!("wheel:{}", state.delivered).into()),
                        ..ObservationMetadata::default()
                    },
                    Duration::ZERO,
                )
                .unwrap();
        },
    ))
}
struct WheelProduct {
    store: Store,
    receipt: Cursor,
}
impl Product for WheelProduct {
    type State = State;
    type Logic = Logic;
    type View = View;
    const KIND: &'static str = "wheel-evidence-test";
    const SURFACE: &'static str = "local";
    const LOG_PREFIX: &'static str = "wheel-test";
    fn sheet(&self) -> &str {
        ".wheel { display: block; width: 100px; height: 80px; }"
    }
    fn snapshot(&self, _: &Ctx<'_, Self>, _: usize, _: f32) -> ProbeSnapshot {
        ProbeSnapshot::default()
    }
    fn diagnostic_attachment(
        &mut self,
        _: &mut Ctx<'_, Self>,
    ) -> Result<Option<crate::DiagnosticBatch>, String> {
        self.store
            .borrow_mut()
            .read(&mut self.receipt, Duration::ZERO, 8)
            .map(|batch| Some(batch.map_payload(|phase| serde_json::json!({"phase": phase}))))
            .map_err(|error| error.to_string())
    }
}

#[test]
fn wheel_surface_refusal_and_owner_effect_survive_independent_diagnostic_reads() {
    let store = Rc::new(RefCell::new(ObservationStore::new(
        "ui-run".into(),
        "wheel-owner".into(),
        RetentionLimits {
            max_records: 8,
            max_bytes: 4096,
            max_age: Duration::from_secs(60),
        },
    )));
    let mut inspector = store.borrow().cursor();
    let product = WheelProduct {
        receipt: store.borrow().cursor(),
        store: store.clone(),
    };
    let lane = Rc::new(RefCell::new(Lane::new(
        product,
        None,
        None,
        None,
        Rc::new(Cell::new(0)),
    )));
    let command = Rc::new(RefCell::new(String::new()));
    let results = Rc::new(RefCell::new(Vec::new()));
    let mut state = HostState::<State, Logic, View>::new();
    state.shared.sheet = lane.borrow().product.sheet().into();
    state.runner = Some(Runner::new(
        Rc::new(RefCell::new(ScriptedDom::new())),
        view as Logic,
        State {
            delivered: 0,
            store: store.clone(),
        },
    ));
    let wake = HostWake::new(state.wake_pending.clone(), Arc::new(|| {}));
    let mut hooks = HostHooks::inert();
    let (driving, text, result) = (lane.clone(), command.clone(), results.clone());
    hooks.after_frame = Box::new(move |ctx| {
        let mut lane = driving.borrow_mut();
        let outcome = Probe {
            ctx,
            lane: &mut lane,
        }
        .app_step(&text.borrow());
        if let Err(error) = &outcome {
            lane.errors.push(error.clone());
        }
        result.borrow_mut().push(outcome);
    });
    let sampling = lane.clone();
    hooks.after_wake = Box::new(move |ctx| {
        let mut lane = sampling.borrow_mut();
        let batch = lane.product.diagnostic_attachment(ctx);
        lane.collect_diagnostics(batch);
    });
    let mut host = Host::new(HostOptions::default(), None, hooks, state, wake);
    host.set_surface_size(200.0, 200.0);
    host.relayout(200.0, 200.0);
    for text in [
        "wheel 0 1 on surface:foreign .wheel",
        "wheel 0 1 on .missing",
    ] {
        *command.borrow_mut() = text.into();
        host.with_ctx(Hook::AfterFrame);
        assert!(results.borrow().last().unwrap().is_err());
        assert_eq!(host.s.runner.as_ref().unwrap().state().delivered, 0);
        assert_eq!(store.borrow().stats().retained_records, 0);
    }
    *command.borrow_mut() = "wheel 0 1 on surface:local .wheel".into();
    host.with_ctx(Hook::AfterFrame);
    assert!(results.borrow().last().unwrap().is_ok());
    assert_eq!(host.s.runner.as_ref().unwrap().state().delivered, 1);
    let inspection = store
        .borrow_mut()
        .read(&mut inspector, Duration::ZERO, 8)
        .unwrap();
    assert_eq!(inspection.records.len(), 1);
    assert_eq!(inspection.records[0].payload, "wheel-applied");
    host.with_ctx(Hook::AfterWake);
    let receipt = lane.borrow().receipt_value(false, &BTreeMap::new());
    assert_eq!(receipt["ok"], false, "the rejected commands remain failed");
    assert_eq!(receipt["errors"].as_array().unwrap().len(), 2);
    let records = &receipt["diagnostics"]["batch"]["records"];
    assert_eq!(
        records.as_array().unwrap().len(),
        1,
        "inspection did not drain the receipt reader"
    );
    assert_eq!(records[0]["payload"]["phase"], "wheel-applied");
    assert_eq!(records[0]["envelope"]["metadata"]["operation"], "wheel:1");
    assert!(records[0]["envelope"]["metadata"]["presented_frame"].is_null());
}
