// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The scenario lane, driven headless: the harness stands in for the event
//! loop, calling `after_frame` once per frame, then delivering the pointer the
//! lane queued and running the window verbs, as the headed host does.
//!
//! A headless run presents no frames, so no capture can land here. That makes
//! the lost-capture path testable, and leaves real captures to the headed smoke.

use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use cambium::{AnyView, GenetCtx, GenetElement, clickable, el, focusable, text};
use cambium_genet_winit_host::{
    AppCtx, Harness, HostHooks, Init, LaneApp, LaneConfig, ProbeSnapshot, ScenarioLane,
    WindowCommand, inert_hooks,
};

#[derive(Default)]
struct App {
    count: usize,
    events: Vec<String>,
}

type Child = Box<dyn AnyView<App, (), GenetCtx, GenetElement>>;
type Logic = fn(&App) -> Child;

fn root(state: &App) -> Child {
    Box::new(focusable(clickable(
        el("button", text(format!("Count {}", state.count))).attr("class", "count"),
        |s: &mut App, _| {
            s.count += 1;
            let n = s.count;
            s.events.push(format!("count {n}"));
        },
    )))
}

const SHEET: &str = ".count { display: block; width: 120px; height: 32px; }";

struct TestLane;

impl LaneApp<App, Logic, Child> for TestLane {
    fn sheet(&self) -> &str {
        SHEET
    }

    fn snapshot(&self, ctx: &AppCtx<'_, App, Logic, Child>) -> ProbeSnapshot {
        ProbeSnapshot::default().with_field("count", ctx.runner.state().count.to_string())
    }

    fn drain_events(&mut self, ctx: &mut AppCtx<'_, App, Logic, Child>) -> Vec<String> {
        let mut drained = Vec::new();
        ctx.runner
            .update(|s| drained = std::mem::take(&mut s.events));
        drained
    }
}

/// A scratch directory per test, so parallel tests never share a receipt.
fn scratch(test: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "cambium-scenario-lane-{}-{test}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("scratch directory");
    dir
}

/// Run `scenario` to its receipt and return the receipt text, with the harness
/// for the assertions the receipt cannot make.
fn run(test: &str, scenario: &str) -> (String, Harness<App, Logic, Child>) {
    let dir = scratch(test);
    let path = dir.join("test.scn");
    std::fs::write(&path, scenario).expect("scenario file");
    let config = LaneConfig {
        scenario: path,
        capture_dir: Some(dir.clone()),
        receipt: None,
    };
    let receipt = config.receipt_path().expect("receipt path");
    let lane = Rc::new(RefCell::new(
        ScenarioLane::new(config, TestLane).expect("scenario parses"),
    ));
    let driven = lane.clone();
    let hooks: HostHooks<App, Logic, Child> = HostHooks {
        after_frame: Box::new(move |ctx| driven.borrow_mut().drive(ctx)),
        ..inert_hooks()
    };
    let mut h = Harness::with_hooks(
        Init {
            state: App::default(),
            logic: root as Logic,
            sheet: SHEET.into(),
            fonts: Vec::new(),
            images: Vec::new(),
        },
        hooks,
    );
    for _ in 0..400 {
        h.layout_at(300.0, 200.0);
        h.after_frame();
        h.drain_pointer();
        h.after_dispatch();
        if h.close_requested() {
            break;
        }
    }
    assert!(lane.borrow().finished(), "the lane never wrote its receipt");
    let text = std::fs::read_to_string(&receipt).expect("receipt written");
    (text, h)
}

#[test]
fn a_click_by_label_reaches_the_app_and_the_receipt_says_ok() {
    let (receipt, h) = run(
        "click",
        "click role:button Count\nsettle 1\nassert snap count == 1\nassert event count 1\n",
    );
    assert!(receipt.starts_with("RESULT ok"), "{receipt}");
    assert!(receipt.contains("frames: 0 captured"), "{receipt}");
    assert_eq!(h.state().count, 1);
    assert!(
        h.close_requested(),
        "a finished lane asks the host to close"
    );
}

#[test]
fn a_failed_assertion_fails_the_receipt() {
    let (receipt, _) = run("assert", "assert snap count == 7\n");
    assert!(receipt.starts_with("RESULT fail"), "{receipt}");
}

#[test]
fn an_unknown_verb_is_loud() {
    let (receipt, _) = run("verb", "frobnicate the widget\n");
    assert!(receipt.starts_with("RESULT fail"), "{receipt}");
    assert!(receipt.contains("unknown verb: frobnicate"), "{receipt}");
}

#[test]
fn resize_goes_through_the_window_verb_queue() {
    let (receipt, h) = run("resize", "resize 320 240\n");
    assert!(receipt.starts_with("RESULT ok"), "{receipt}");
    assert!(
        h.performed().iter().any(
            |command| matches!(command, WindowCommand::Resize(w, h) if *w == 320.0 && *h == 240.0)
        ),
        "{:?}",
        h.performed()
    );
}

#[test]
fn a_capture_that_never_lands_fails_rather_than_hangs() {
    let (receipt, _) = run("capture", "capture never\n");
    assert!(receipt.starts_with("RESULT fail"), "{receipt}");
    assert!(receipt.contains("capture never never landed"), "{receipt}");
}

#[test]
fn the_receipt_goes_to_its_own_path_or_the_capture_directory() {
    let dir = PathBuf::from("captures");
    let into_dir = LaneConfig {
        scenario: PathBuf::from("a.scn"),
        capture_dir: Some(dir.clone()),
        receipt: None,
    };
    assert_eq!(into_dir.receipt_path(), Some(dir.join("scenario.done")));
    let explicit = LaneConfig {
        receipt: Some(PathBuf::from("receipt.txt")),
        ..into_dir
    };
    assert_eq!(explicit.receipt_path(), Some(PathBuf::from("receipt.txt")));
    let neither = LaneConfig {
        scenario: PathBuf::from("a.scn"),
        capture_dir: None,
        receipt: None,
    };
    assert_eq!(neither.receipt_path(), None);
}
