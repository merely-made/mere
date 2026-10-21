// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Real retained host and Probe delivery, without an OS window or GPU.

use super::*;
use crate::{Ctx, LaneConfig, probe::Probe};
use cambium::{AnyView, GenetCtx, GenetElement, el};
use cambium_rootstock::{Hook, Host, HostHooks, HostOptions, HostState, HostWake, Runner};
use genet_scripted_dom::ScriptedDom;
use std::{
    cell::{Cell, RefCell},
    collections::BTreeMap,
    rc::Rc,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};
use taproot::{Automatable, Driveable, ProbeSnapshot};

const SCHEMA: &str = "retained-command-test/v1";
const SECRET: &[u8] = b"opaque argument that must not enter the audit";
const REFUSAL: &str = "αβγδεζη confirmation required";
type View = Box<dyn AnyView<State, (), GenetCtx, GenetElement>>;
type Logic = fn(&State) -> View;

#[derive(Clone, Default)]
struct State {
    dispatched: usize,
    applied: usize,
    legacy: usize,
    pending: bool,
    arguments: Vec<u8>,
}

fn view(_: &State) -> View {
    Box::new(el("div", "command host"))
}

struct Commands;
impl Product for Commands {
    type State = State;
    type Logic = Logic;
    type View = View;
    const KIND: &'static str = "retained-command-test";
    const SURFACE: &'static str = "local";
    const LOG_PREFIX: &'static str = "command-test";

    fn sheet(&self) -> &str {
        ""
    }

    fn snapshot(&self, ctx: &Ctx<'_, Self>, _: usize, _: f32) -> ProbeSnapshot {
        ProbeSnapshot::default().with_field("applied", ctx.runner.state().applied.to_string())
    }

    fn act(&mut self, ctx: &mut Ctx<'_, Self>, label: &str) -> bool {
        if label != "legacy" {
            return false;
        }
        ctx.runner.update(|state| state.legacy += 1);
        true
    }

    fn app_step(
        &mut self,
        ctx: &mut Ctx<'_, Self>,
        _: crate::Checkpoints<'_>,
        line: &str,
    ) -> Result<(), String> {
        if line != "host-fallback" {
            return Err("unknown test step".into());
        }
        ctx.runner.update(|state| state.legacy += 1);
        Ok(())
    }

    fn invoke(&mut self, ctx: &mut Ctx<'_, Self>, invocation: &Invocation) -> Option<Outcome> {
        ctx.runner.update(|state| state.dispatched += 1);
        if invocation.schema_id != SCHEMA
            || invocation.effect != EffectClass::Curation
            || invocation.arguments != SECRET
        {
            return Some(Outcome::Refused {
                reason: "invalid current command contract".into(),
            });
        }
        match invocation.id.as_str() {
            "physics:toggle" => {
                ctx.runner.update(|state| {
                    state.applied += 1;
                    state.arguments = invocation.arguments.clone();
                });
                Some(Outcome::Done)
            },
            "host:refuse" => Some(Outcome::Refused {
                reason: REFUSAL.into(),
            }),
            "host:stale" => Some(Outcome::Stale),
            "host:pending" => {
                ctx.runner.update(|state| state.pending = true);
                None
            },
            _ => Some(Outcome::Stale),
        }
    }
}

fn invocation(id: &str) -> Invocation {
    Invocation {
        id: id.into(),
        schema_id: SCHEMA.into(),
        effect: EffectClass::Curation,
        arguments: SECRET.into(),
    }
}

fn line(id: &str) -> String {
    format!("invoke {}", serde_json::to_string(&invocation(id)).unwrap())
}

struct Exercise {
    state: State,
    results: Vec<Result<(), String>>,
    receipt: serde_json::Value,
    written: String,
}

fn exercise(lines: &[String], limits: Option<CommandAuditLimits>, text: bool) -> Exercise {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let path = std::env::temp_dir().join(format!(
        "mesquite-command-{}-{}.receipt",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let mut lane = Lane::new(
        Commands,
        None,
        Some(path.clone()),
        None,
        Rc::new(Cell::new(0)),
    );
    if text {
        lane.script_files = Some(crate::scenario::ScriptFiles::new(
            LaneConfig {
                scenario: "unused.scn".into(),
                capture_dir: None,
                receipt: Some(path.clone()),
            },
            |_| None,
        ));
    }
    if let Some(limits) = limits {
        lane.set_command_audit_limits(limits);
    }
    let lane = Rc::new(RefCell::new(lane));
    let results = Rc::new(RefCell::new(Vec::new()));
    let mut state = HostState::new();
    state.runner = Some(Runner::new(
        Rc::new(RefCell::new(ScriptedDom::new())),
        view as Logic,
        State::default(),
    ));
    let wake = HostWake::new(state.wake_pending.clone(), Arc::new(|| {}));
    let (driving, result, lines) = (lane.clone(), results.clone(), lines.to_vec());
    let mut hooks = HostHooks::inert();
    hooks.after_frame = Box::new(move |ctx| {
        let mut lane = driving.borrow_mut();
        {
            let mut probe = Probe {
                ctx,
                lane: &mut lane,
            };
            for line in &lines {
                let outcome = if let Some(label) = line.strip_prefix("act ") {
                    if probe.act(label) {
                        Ok(())
                    } else {
                        Err("legacy refused".into())
                    }
                } else {
                    probe.app_step(line)
                };
                if let Err(error) = &outcome {
                    probe.lane.errors.push(error.clone());
                }
                result.borrow_mut().push(outcome);
            }
        }
        lane.request_close();
        lane.after_frame(ctx);
    });
    let mut host = Host::new(HostOptions::default(), None, hooks, state, wake);
    host.with_ctx(Hook::AfterFrame);
    assert!(lane.borrow().finished());
    let written = std::fs::read_to_string(&path).unwrap();
    std::fs::remove_file(path).unwrap();
    let receipt = lane.borrow().receipt_value(false, &BTreeMap::new());
    let results = results.borrow().clone();
    Exercise {
        state: host.s.runner.as_ref().unwrap().state().clone(),
        results,
        receipt,
        written,
    }
}

#[test]
fn retained_invoke_preserves_terminal_and_pending_results_in_both_receipt_formats() {
    for text in [false, true] {
        let run = exercise(
            &[
                line("physics:toggle"),
                line("host:refuse"),
                line("host:stale"),
                line("host:pending"),
            ],
            None,
            text,
        );
        assert_eq!(run.state.dispatched, 4);
        assert_eq!(run.state.applied, 1);
        assert_eq!(run.state.arguments, SECRET);
        assert!(run.state.pending);
        assert_eq!(
            run.results.iter().map(Result::is_ok).collect::<Vec<_>>(),
            [true, false, false, true]
        );
        let audit = &run.receipt["commands"];
        assert_eq!(audit["schema"], "mesquite.commands/v1");
        let records = audit["records"].as_array().unwrap();
        assert_eq!(records[0]["id"], "physics:toggle");
        assert_eq!(records[0]["outcome"], "Done");
        assert_eq!(records[1]["outcome"]["Refused"]["reason"], REFUSAL);
        assert_eq!(records[2]["outcome"], "Stale");
        assert!(records[3]["outcome"].is_null());
        assert_eq!(records[3]["sequence"], 3);
        for record in records {
            assert!(record.get("arguments").is_none());
        }
        assert!(!run.written.contains(std::str::from_utf8(SECRET).unwrap()));
        let exported = if text {
            let line = run
                .written
                .lines()
                .find_map(|line| line.strip_prefix("commands "))
                .unwrap();
            serde_json::from_str::<serde_json::Value>(line).unwrap()
        } else {
            serde_json::from_str::<serde_json::Value>(&run.written).unwrap()["commands"].clone()
        };
        assert_eq!(exported, *audit);
    }
}

#[test]
fn malformed_payload_never_dispatches_and_legacy_hooks_keep_the_old_receipt_shape() {
    let run = exercise(
        &[
            "invoke".into(),
            "invoke {bad JSON}".into(),
            "invoke {\"arguments\":[\"secret contents\"]}".into(),
            "act legacy".into(),
            "host-fallback".into(),
        ],
        None,
        false,
    );
    assert_eq!(run.state.dispatched, 0);
    assert_eq!(run.state.applied, 0);
    assert_eq!(run.state.legacy, 2);
    assert_eq!(
        run.results.iter().map(Result::is_ok).collect::<Vec<_>>(),
        [false, false, false, true, true]
    );
    assert!(run.receipt.get("commands").is_none());
    assert!(!run.written.contains("secret contents"));
    assert!(parse("invoke-something {}").is_none());
}

#[test]
fn retention_reports_eviction_and_utf8_truncation_without_argument_export() {
    let run = exercise(
        &[
            line("physics:toggle"),
            line("host:refuse"),
            line("host:pending"),
        ],
        Some(CommandAuditLimits {
            max_records: 2,
            max_field_bytes: 5,
        }),
        false,
    );
    let audit = &run.receipt["commands"];
    assert_eq!(audit["dropped_records"], 1);
    assert_eq!(audit["truncated_records"], 3);
    assert_eq!(audit["records"][0]["sequence"], 1);
    assert_eq!(audit["records"][0]["outcome"]["Refused"]["reason"], "αβ");
    assert!(audit["records"][1]["outcome"].is_null());
    assert_eq!(audit["records"].as_array().unwrap().len(), 2);
    for record in audit["records"].as_array().unwrap() {
        assert!(record["truncated"].as_bool().unwrap());
        assert!(record["id"].as_str().unwrap().len() <= 5);
        assert!(record["schema_id"].as_str().unwrap().len() <= 5);
    }
    let empty = exercise(
        &[line("physics:toggle")],
        Some(CommandAuditLimits {
            max_records: 0,
            max_field_bytes: 0,
        }),
        false,
    );
    assert_eq!(
        empty.state.applied, 1,
        "receipt limits do not change enactment"
    );
    assert_eq!(empty.receipt["commands"]["dropped_records"], 1);
    assert!(
        empty.receipt["commands"]["records"]
            .as_array()
            .unwrap()
            .is_empty()
    );
}

#[test]
fn tightening_limits_reports_prior_record_loss_and_explicit_empty_audit() {
    let mut audit = CommandAudit::default();
    audit.record(&invocation("physics:toggle"), Some(&Outcome::Done));
    audit.record(
        &invocation("host:refuse"),
        Some(&Outcome::Refused {
            reason: REFUSAL.into(),
        }),
    );
    audit.set_limits(CommandAuditLimits {
        max_records: 1,
        max_field_bytes: 3,
    });
    let value = serde_json::to_value(&audit).unwrap();
    assert_eq!(value["dropped_records"], 1);
    assert_eq!(value["truncated_records"], 2);
    assert_eq!(value["records"][0]["outcome"]["Refused"]["reason"], "α");
    let empty = exercise(&[], Some(CommandAuditLimits::default()), false);
    assert!(
        empty.receipt["commands"]["records"]
            .as_array()
            .unwrap()
            .is_empty()
    );
}
