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

#[test]
fn selector_tests_probe_routes_generated_actions_and_observes_declines() {
    use cambium_rootstock::{
        ProducedTexture, ProducerAction, ProducerContext, ProducerNode, ProducerRole,
        ProducerSemantics, TextureProducer,
    };
    type V = Box<dyn AnyView<u32, (), GenetCtx, GenetElement>>;
    fn view(state: &u32) -> V {
        Box::new(el(
            "main",
            (
                cambium::clickable(
                    el("button", "Pin selected in workbench"),
                    |s: &mut u32, _| *s += 1,
                ),
                cambium::custom_leaf::<u32, ()>(7, 80, 30)
                    .attr("aria-hidden", (*state > 0).to_string()),
                cambium::custom_leaf::<u32, ()>(8, 80, 30),
            ),
        ))
    }
    struct ProductFixture;
    impl Product for ProductFixture {
        type State = u32;
        type Logic = fn(&u32) -> V;
        type View = V;
        const KIND: &'static str = "semantic-receiving";
        const SURFACE: &'static str = "local";
        const LOG_PREFIX: &'static str = "semantic-test";
        fn sheet(&self) -> &str {
            "main,button,custom-leaf {display:block;width:80px;height:30px}"
        }
        fn snapshot(&self, _: &Ctx<'_, Self>, _: usize, _: f32) -> ProbeSnapshot {
            ProbeSnapshot::default()
        }
    }
    struct Board {
        calls: Rc<Cell<u32>>,
        accepted: Rc<Cell<bool>>,
        offered: Rc<Cell<bool>>,
    }
    impl TextureProducer for Board {
        fn render(&mut self, _: &ProducerContext<'_>) -> Option<ProducedTexture> {
            None
        }
        fn semantics(&self) -> Option<ProducerSemantics> {
            Some(ProducerSemantics {
                children: vec![ProducerNode {
                    key: 41,
                    role: ProducerRole::ListItem,
                    name: "Card".into(),
                    rect: [0., 0., 40., 20.],
                    actions: if self.offered.get() {
                        vec![ProducerAction {
                            id: "pin".into(),
                            label: "Pin selected".into(),
                            description: "Pin".into(),
                        }]
                    } else {
                        vec![]
                    },
                }],
                ..Default::default()
            })
        }
        fn act(&mut self, key: u64, id: &str) -> bool {
            assert_eq!((key, id), (41, "pin"));
            self.calls.set(self.calls.get() + 1);
            self.accepted.get()
        }
    }
    let calls = Rc::new(Cell::new(0));
    let accepted = Rc::new(Cell::new(true));
    let offered = Rc::new(Cell::new(true));
    let mut state: HostState<u32, fn(&u32) -> V, V> = HostState::new();
    let lane = Rc::new(RefCell::new(Lane::new(
        ProductFixture,
        None,
        None,
        None,
        Rc::new(Cell::new(0)),
    )));
    state.shared.sheet = lane.borrow().product.sheet().into();
    state
        .shared
        .producers
        .register(
            7,
            Board {
                calls: calls.clone(),
                accepted: accepted.clone(),
                offered: offered.clone(),
            },
            &[],
        )
        .unwrap();
    // A leaf-only slider; its DOM seed has no slider role.
    state.shared.leaves.insert(
        8,
        Box::new(sprigging::Knob::new(sprigging::Size {
            width: 80.,
            height: 30.,
        })),
    );
    state.runner = Some(Runner::new(
        Rc::new(RefCell::new(ScriptedDom::new())),
        view as fn(&u32) -> V,
        0,
    ));
    let stage = Rc::new(Cell::new(0));
    let results = Rc::new(RefCell::new(Vec::new()));
    let (driving, step, saved, current_offer) = (
        lane.clone(),
        stage.clone(),
        results.clone(),
        offered.clone(),
    );
    let mut hooks = HostHooks::inert();
    hooks.after_frame = Box::new(move |ctx| {
        let mut lane = driving.borrow_mut();
        let selector = Selector::role("button").containing("Pin selected");
        match step.get() {
            0 | 2 | 5 => {
                let mut probe = Probe {
                    ctx,
                    lane: &mut lane,
                };
                let before = probe.lane.misses.borrow().len();
                assert!(
                    probe
                        .resolve(&Selector::role("button").containing("absent target"))
                        .is_none()
                );
                assert_eq!(
                    probe.lane.misses.borrow().len(),
                    before,
                    "read-only point misses are not failed interactions"
                );
                assert!(matches!(
                    probe.selector_target(&Selector::role("slider")),
                    SelectorTarget::Hit(_)
                ));
                assert!(
                    probe
                        .resolve(&Selector::role("slider").on_surface("foreign"))
                        .is_none()
                );
                assert!(probe.click(&selector.clone().on_surface("local")));
                assert!(
                    probe.ctx.pointer.is_empty(),
                    "virtual buttons use semantic routing"
                );
                if step.get() == 5 {
                    current_offer.set(false);
                }
            },
            1 | 3 | 6 => saved.borrow_mut().push(
                lane.clicks
                    .after_frame(ctx, |_, _, _| panic!("no pointer for virtual actions")),
            ),
            4 => {
                let mut probe = Probe {
                    ctx,
                    lane: &mut lane,
                };
                assert_eq!(probe.click_target(&selector), Some(true));
            },
            _ => unreachable!(),
        }
    });
    let wake = HostWake::new(state.wake_pending.clone(), Arc::new(|| {}));
    let mut host = Host::new(HostOptions::default(), None, hooks, state, wake);
    host.set_surface_size(320., 240.);
    host.relayout(320., 240.);
    host.with_ctx(Hook::AfterFrame);
    assert_eq!(calls.get(), 1);
    assert_eq!(*host.s.runner.as_ref().unwrap().state(), 0);
    stage.set(1);
    host.with_ctx(Hook::AfterFrame);
    assert!(matches!(results.borrow()[0], Ok(true)));
    accepted.set(false);
    stage.set(2);
    host.with_ctx(Hook::AfterFrame);
    assert_eq!(calls.get(), 2);
    stage.set(3);
    host.with_ctx(Hook::AfterFrame);
    assert!(
        results.borrow()[1]
            .as_ref()
            .unwrap_err()
            .contains("ProducerDeclined")
    );
    offered.set(false);
    stage.set(4);
    host.with_ctx(Hook::AfterFrame);
    assert_eq!(
        calls.get(),
        2,
        "a removed action cannot reuse its old route"
    );
    assert_eq!(
        *host.s.runner.as_ref().unwrap().state(),
        1,
        "remaining prefix is the native DOM button"
    );
    // Admission must re-read an offer withdrawn after matching but before drain.
    offered.set(true);
    accepted.set(true);
    host.s.runner.as_mut().unwrap().update(|state| *state = 0);
    host.relayout(320., 240.);
    stage.set(5);
    host.with_ctx(Hook::AfterFrame);
    assert_eq!(
        calls.get(),
        2,
        "withdrawn queued action must not reach the producer"
    );
    stage.set(6);
    host.with_ctx(Hook::AfterFrame);
    assert!(
        results.borrow()[2]
            .as_ref()
            .unwrap_err()
            .contains("Unsupported")
    );
    // Scoped virtual clicks must not be recorded as pointer misses. Both
    // accepted and declined last actions pass through the actual scenario lane.
    for accepts in [true, false] {
        offered.set(true);
        accepted.set(accepts);
        host.s.close_requested = false;
        host.s.runner.as_mut().unwrap().update(|state| *state = 0);
        host.relayout(320., 240.);
        let exit = Rc::new(Cell::new(0));
        let completion = Rc::new(RefCell::new(Lane::new(
            ProductFixture,
            Some(
                taproot::Scenario::parse("click surface:local role:button Pin selected\n").unwrap(),
            ),
            None,
            None,
            exit.clone(),
        )));
        let drive = completion.clone();
        host.hooks.after_frame = Box::new(move |ctx| drive.borrow_mut().after_frame(ctx));
        host.with_ctx(Hook::AfterFrame);
        assert!(!host.s.close_requested);
        host.with_ctx(Hook::AfterFrame);
        assert!(!host.s.close_requested);
        host.with_ctx(Hook::AfterFrame);
        assert!(host.s.close_requested);
        assert_eq!(exit.get(), if accepts { 0 } else { 1 });
        if accepts {
            assert!(completion.borrow().errors.is_empty());
        } else {
            assert!(
                completion
                    .borrow()
                    .errors
                    .iter()
                    .any(|e| e.contains("ProducerDeclined"))
            );
        }
    }
}

#[test]
fn publication_collector_receives_host_transitions_and_independent_receipt_reads() {
    use cambium_rootstock::{
        A11yAdapterState as Adapter, A11yPublication, A11yPublicationOutcome as Phase,
        A11yPublicationRefusal as Refusal, A11yPublicationSnapshot, A11ySync, Accessibility,
    };
    struct Publisher(Rc<Cell<A11yPublication>>);
    impl Accessibility for Publisher {
        fn adapter_state(&self) -> Adapter {
            self.0.get().adapter
        }

        fn sync(
            &mut self,
            _: &cambium_rootstock::A11yOwner,
            _: &cambium_rootstock::WindowDom<'_>,
            _: &cambium_rootstock::OwnedLayout,
            _: &sprigging::LeafRegistry<u64>,
            _: &cambium_rootstock::ProducerRegistry,
            _: Option<u64>,
            _: f64,
        ) -> A11ySync {
            A11ySync {
                requests: vec![],
                publication: self.0.get(),
            }
        }
    }
    let history = Rc::new(RefCell::new(
        ObservationStore::<A11yPublicationSnapshot>::new(
            "publication-run".into(),
            "ui-publisher".into(),
            RetentionLimits {
                max_records: 8,
                max_bytes: 16384,
                max_age: Duration::from_secs(60),
            },
        ),
    ));
    let mut inspector = history.borrow().cursor();
    let mut receipt_cursor = history.borrow().cursor();
    let application = Rc::new(RefCell::new(ObservationStore::new(
        "application-run".into(),
        "wheel".into(),
        RetentionLimits {
            max_records: 8,
            max_bytes: 4096,
            max_age: Duration::from_secs(60),
        },
    )));
    let product = WheelProduct {
        receipt: application.borrow().cursor(),
        store: application.clone(),
    };
    let lane = Rc::new(RefCell::new(Lane::new(
        product,
        None,
        None,
        None,
        Rc::new(Cell::new(0)),
    )));
    let mut state = HostState::<State, Logic, View>::new();
    state.shared.sheet = lane.borrow().product.sheet().into();
    state.runner = Some(Runner::new(
        Rc::new(RefCell::new(ScriptedDom::new())),
        view as Logic,
        State {
            delivered: 0,
            store: application,
        },
    ));
    let reported = Rc::new(Cell::new(A11yPublication {
        adapter: Adapter::Installed,
        outcome: Phase::Submitted {
            nodes: 4,
            contribution_issues: 1,
        },
    }));
    state.a11y = Some(Box::new(Publisher(reported.clone())));
    let sink = history.clone();
    state.a11y_publication_observer = Some(Box::new(move |snapshot| {
        let bytes = serde_json::to_vec(&snapshot).unwrap().len();
        sink.borrow_mut()
            .record(
                snapshot,
                bytes,
                ObservationMetadata::default(),
                Duration::ZERO,
            )
            .unwrap();
    }));
    let wake = HostWake::new(state.wake_pending.clone(), Arc::new(|| {}));
    let mut hooks = HostHooks::inert();
    let collecting = lane.clone();
    hooks.after_frame = Box::new(move |ctx| collecting.borrow_mut().after_frame(ctx));
    let mut host = Host::new(HostOptions::default(), None, hooks, state, wake);
    host.set_surface_size(200.0, 200.0);
    host.relayout(200.0, 200.0);
    host.sync_a11y();
    reported.set(A11yPublication::refused(
        Adapter::Installed,
        Refusal::InvalidTree,
    ));
    host.sync_a11y();
    let inspection = history
        .borrow_mut()
        .read(&mut inspector, Duration::ZERO, 8)
        .unwrap();
    assert_eq!(inspection.records.len(), 2);
    let exported = history
        .borrow_mut()
        .read(&mut receipt_cursor, Duration::ZERO, 8)
        .unwrap();
    assert_eq!(
        exported.records.len(),
        2,
        "inspection never drains receipt history"
    );
    lane.borrow_mut().collect_diagnostics(Ok(Some(
        exported.map_payload(|value| serde_json::to_value(value).unwrap()),
    )));
    lane.borrow_mut().request_close();
    host.with_ctx(Hook::AfterFrame); // real completion path automatically samples AppCtx
    let value = lane.borrow().receipt_value(false, &BTreeMap::new());
    assert_eq!(
        value["a11y_publication"]["snapshot"]["publication"]["outcome"]["phase"],
        "refused"
    );
    assert_eq!(value["a11y_publication"]["snapshot"]["sequence"], 2);
    assert_eq!(host.s.runner.as_ref().unwrap().state().delivered, 0);
    assert!(
        value.get("diagnostics").is_some(),
        "product completion sampled its separate diagnostics"
    );
    reported.set(A11yPublication {
        adapter: Adapter::Installed,
        outcome: Phase::Submitted {
            nodes: 5,
            contribution_issues: 0,
        },
    });
    host.sync_a11y();
    drop(host.take_disposal());
    let later = history
        .borrow_mut()
        .read(&mut inspector, Duration::ZERO, 8)
        .unwrap();
    assert_eq!(later.records.len(), 2);
    assert!(matches!(
        later.records[1].payload.publication.outcome,
        Phase::NotAttempted {
            reason: cambium_rootstock::A11yPublicationSkip::OwnerEnded
        }
    ));
    assert!(
        later
            .records
            .iter()
            .all(|record| record.envelope.metadata.presented_frame.is_none())
    );
}
