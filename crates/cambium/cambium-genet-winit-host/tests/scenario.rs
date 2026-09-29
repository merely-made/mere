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

use mesquite::LaneConfig;
use std::cell::Cell;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use taproot::ProbeSnapshot;

use cambium::{
    AnyView, FileEvent, FileFilter, GenetCtx, GenetElement, PointerClick, button, clickable, el,
    focusable, open_file, text,
};
use cambium_genet_winit_host::{AppCtx, Harness, HostHooks, Init, WindowCommand, inert_hooks};

#[derive(Default)]
struct App {
    count: usize,
    events: Vec<String>,
    asking: bool,
    opened: String,
    target_calls: Cell<usize>,
    target_point: Cell<Option<(f32, f32)>>,
}

type Child = Box<dyn AnyView<App, (), GenetCtx, GenetElement>>;
type Logic = fn(&App) -> Child;

fn root(state: &App) -> Child {
    Box::new(el(
        "main",
        (
            focusable(clickable(
                el("button", text(format!("Count {}", state.count))).attr("class", "count"),
                |s: &mut App, _| {
                    s.count += 1;
                    let n = s.count;
                    s.events.push(format!("count {n}"));
                },
            )),
            open_file(
                button("Open", |s: &mut App, _: PointerClick| s.asking = true)
                    .attr("class", "open"),
                state.asking,
                FileFilter::extensions(["txt"]),
                |s: &mut App, event: FileEvent| {
                    s.asking = false;
                    s.opened = event
                        .files
                        .first()
                        .map_or_else(|| "nothing".to_string(), |file| file.name.clone());
                },
            ),
            el(
                "div",
                (
                    el("div", ()).attr("class", "spacer"),
                    button("Far", |s: &mut App, _| s.count += 1),
                ),
            )
            .attr("class", "scroller"),
        ),
    ))
}

const SHEET: &str = "button { display: block; width: 120px; height: 32px; }
    .scroller { height: 64px; overflow: auto; } .spacer { height: 300px; }";

struct TestLane;

impl mesquite::Product for TestLane {
    type State = App;
    type Logic = Logic;
    type View = Child;
    const KIND: &'static str = "scenario-test";
    const SURFACE: &'static str = "app";
    const LOG_PREFIX: &'static str = "scenario-test";

    fn sheet(&self) -> &str {
        SHEET
    }

    fn app_step_with_clicks(
        &mut self,
        ctx: &mut mesquite::Ctx<'_, Self>,
        _: mesquite::Checkpoints<'_>,
        clicks: &mut mesquite::Clicks,
        line: &str,
    ) -> Result<(), String> {
        if line != "select-far" {
            return Err(format!("unknown scenario step: {line}"));
        }
        let selector = taproot::Selector::role("button").containing("Far");
        if clicks.click(ctx, &selector, |_, _, r| {
            (r[0] + r[2] * 0.5, r[1] + r[3] * 0.5)
        }) {
            Ok(())
        } else {
            Err("Far target missing".into())
        }
    }

    fn snapshot(
        &self,
        ctx: &AppCtx<'_, App, Logic, Child>,
        _captures: usize,
        _: f32,
    ) -> ProbeSnapshot {
        ProbeSnapshot::default()
            .with_field("count", ctx.runner.state().count.to_string())
            .with_field("opened", ctx.runner.state().opened.clone())
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
    run_in(&scratch(test), scenario)
}

/// [`run`], in a directory the test has already put files in.
fn run_in(dir: &Path, scenario: &str) -> (String, Harness<App, Logic, Child>) {
    run_lane(dir, scenario, SHEET, false)
}

struct TestProduct(&'static str);

impl mesquite::Product for TestProduct {
    type State = App;
    type Logic = Logic;
    type View = Child;
    const KIND: &'static str = "scenario-test";
    const SURFACE: &'static str = "app";
    const LOG_PREFIX: &'static str = "scenario-test";

    fn sheet(&self) -> &'static str {
        self.0
    }
    fn snapshot(&self, ctx: &mesquite::Ctx<'_, Self>, _: usize, _: f32) -> ProbeSnapshot {
        TestLane.snapshot(ctx, 0, 1.0)
    }
    fn drain_events(&mut self, ctx: &mut mesquite::Ctx<'_, Self>) -> Vec<String> {
        TestLane.drain_events(ctx)
    }
    fn default_capture_path(&self) -> PathBuf {
        PathBuf::from("unused.png")
    }
    fn target_point(
        &self,
        ctx: &mesquite::Ctx<'_, Self>,
        _: cambium_rootstock::NodeId,
        rect: [f32; 4],
    ) -> (f32, f32) {
        let calls = &ctx.runner.state().target_calls;
        calls.set(calls.get() + 1);
        // Deliberately use an off-centre point inside the button.
        let point = (rect[0] + rect[2] * 0.25, rect[1] + rect[3] * 0.25);
        ctx.runner.state().target_point.set(Some(point));
        point
    }
}

fn run_lane(
    dir: &Path,
    scenario: &str,
    sheet: &'static str,
    use_mesquite: bool,
) -> (String, Harness<App, Logic, Child>) {
    let dir = dir.to_path_buf();
    let path = dir.join("test.scn");
    std::fs::write(&path, scenario).expect("scenario file");
    let config = LaneConfig {
        scenario: path,
        capture_dir: Some(dir.clone()),
        receipt: None,
    };
    let receipt = config.receipt_path().expect("receipt path");
    let hooks: HostHooks<App, Logic, Child> = if use_mesquite {
        let mut lane = mesquite::Lane::new(
            TestProduct(sheet),
            Some(taproot::Scenario::parse(scenario).unwrap()),
            Some(receipt.clone()),
            None,
            Rc::new(Cell::new(0)),
        );
        HostHooks {
            after_frame: Box::new(move |ctx| lane.after_frame(ctx)),
            ..inert_hooks()
        }
    } else {
        let mut lane =
            mesquite::Lane::from_config(config, TestLane, cambium_genet_winit_host::read_file)
                .expect("scenario parses");
        HostHooks {
            after_frame: Box::new(move |ctx| lane.after_frame(ctx)),
            ..inert_hooks()
        }
    };
    let mut h = Harness::with_hooks(
        Init {
            state: App::default(),
            logic: root as Logic,
            sheet: sheet.into(),
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
    assert!(h.close_requested(), "the lane never wrote its receipt");
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
fn a_click_scrolls_a_below_the_fold_button_before_the_next_assertion() {
    let (receipt, h) = run("far", "click role:button Far\nassert snap count == 1\n");
    assert!(receipt.starts_with("RESULT ok"), "{receipt}");
    assert_eq!(h.state().count, 1);
}

#[test]
fn product_click_verbs_share_the_lanes_scroll_and_dispatch_wait() {
    let (receipt, h) = run("product-far", "select-far\nassert snap count == 1\n");
    assert!(receipt.starts_with("RESULT ok"), "{receipt}");
    assert_eq!(h.state().count, 1);
}

#[test]
fn both_receipt_modes_click_the_visible_part_of_an_oversized_button() {
    for mesquite in [false, true] {
        let (receipt, h) = run_lane(
            &scratch(&format!("tall-{mesquite}")),
            "click role:button Far\nassert snap count == 1\n",
            "button { display:block; width:120px; height:32px; }
             .scroller { height:64px; overflow:auto; }
             .scroller button { height:300px; }",
            mesquite,
        );
        assert_eq!(h.state().count, 1, "{receipt}");
        assert!(!receipt.contains("FAIL:"), "{receipt}");
    }
}

#[test]
fn both_receipt_modes_fail_an_unrevealable_target_even_without_an_assertion() {
    for mesquite in [false, true] {
        let (receipt, h) = run_lane(
            &scratch(&format!("clipped-{mesquite}")),
            "click role:button Far\n",
            "button { display:block; width:120px; height:32px; }
             .scroller { height:0; overflow:hidden; }",
            mesquite,
        );
        assert_eq!(h.state().count, 0);
        assert!(receipt.contains("never came into view"), "{receipt}");
        assert!(
            receipt.starts_with("RESULT fail") || receipt.contains("\"ok\": false"),
            "{receipt}"
        );
    }
}

#[test]
fn mesquite_scrolls_then_uses_the_product_target_point_before_advancing() {
    let (receipt, h) = run_lane(
        &scratch("mesquite-far"),
        "click role:button Count\nassert snap count == 1\nclick role:button Far\nassert snap count == 2\n",
        SHEET,
        true,
    );
    assert!(receipt.contains("\"ok\": true"), "{receipt}");
    assert_eq!(h.state().count, 2);
    assert_eq!(h.state().target_calls.get(), 2);
    assert_eq!(Some(h.cursor()), h.state().target_point.get());
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
    assert!(
        receipt.contains("unknown scenario step: frobnicate"),
        "{receipt}"
    );
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
fn a_scenario_supplies_a_file_through_the_same_event_without_a_dialog() {
    // The file sits beside the scenario, whose directory the path is read from.
    let dir = scratch("file");
    std::fs::write(dir.join("hello.txt"), "hi").expect("fixture file");
    let (receipt, h) = run_in(
        &dir,
        "click role:button Open\nsettle 1\nfile hello.txt\nsettle 1\nassert snap opened == hello.txt\n",
    );
    assert!(receipt.starts_with("RESULT ok"), "{receipt}");
    assert_eq!(h.state().opened, "hello.txt");
    assert!(!h.state().asking);
}

#[test]
fn a_cancelled_file_request_answers_with_nothing() {
    let (receipt, h) = run(
        "file-cancel",
        "click role:button Open\nsettle 1\nfile cancel\nsettle 1\nassert snap opened == nothing\n",
    );
    assert!(receipt.starts_with("RESULT ok"), "{receipt}");
    assert!(!h.state().asking);
}

#[test]
fn a_file_with_no_request_waiting_is_loud() {
    let (receipt, _) = run("file-unasked", "file hello.txt\n");
    assert!(receipt.starts_with("RESULT fail"), "{receipt}");
    assert!(receipt.contains("no file request is waiting"), "{receipt}");
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

#[test]
fn text_and_json_runs_use_the_same_checkpoint_commands() {
    for json in [false, true] {
        let (receipt, h) = run_lane(
            &scratch(&format!("checkpoint-{json}")),
            "remember before\nclick role:button Count\nassert snap count == 1\nmore before count\n",
            SHEET,
            json,
        );
        if json {
            let value: serde_json::Value = serde_json::from_str(&receipt).unwrap();
            assert_eq!(value["ok"], true, "{receipt}");
            assert_eq!(value["checkpoints"]["before"]["count"], "0");
        } else {
            assert!(receipt.starts_with("RESULT ok"), "{receipt}");
        }
        assert_eq!(h.state().count, 1);
    }
}

struct RefusingProduct;
impl mesquite::Product for RefusingProduct {
    type State = App;
    type Logic = Logic;
    type View = Child;
    const KIND: &'static str = "refusing-test";
    const SURFACE: &'static str = "app";
    const LOG_PREFIX: &'static str = "refusing-test";
    fn sheet(&self) -> &str {
        SHEET
    }
    fn snapshot(&self, _: &mesquite::Ctx<'_, Self>, _: usize, _: f32) -> ProbeSnapshot {
        ProbeSnapshot::default()
    }
    fn receipt_checks(&self, _: &[mesquite::CaptureRecord]) -> Vec<String> {
        vec!["product acceptance failed".into()]
    }
    fn receipt_lines(&self) -> Vec<String> {
        vec!["product diagnostic".into()]
    }
}

#[test]
fn product_acceptance_can_fail_either_receipt_format() {
    for json in [false, true] {
        let dir = scratch(&format!("acceptance-{json}"));
        let scenario_path = dir.join("run.scn");
        std::fs::write(&scenario_path, "log done\n").unwrap();
        let receipt = dir.join("receipt");
        let mut lane = if json {
            mesquite::Lane::new(
                RefusingProduct,
                Some(taproot::Scenario::parse("log done\n").unwrap()),
                Some(receipt.clone()),
                None,
                Rc::new(Cell::new(0)),
            )
        } else {
            mesquite::Lane::from_config(
                LaneConfig {
                    scenario: scenario_path,
                    capture_dir: None,
                    receipt: Some(receipt.clone()),
                },
                RefusingProduct,
                cambium_genet_winit_host::read_file,
            )
            .unwrap()
        };
        let mut h = Harness::with_hooks(
            Init {
                state: App::default(),
                logic: root as Logic,
                sheet: SHEET.into(),
                fonts: vec![],
                images: vec![],
            },
            HostHooks {
                after_frame: Box::new(move |ctx| lane.after_frame(ctx)),
                ..inert_hooks()
            },
        );
        for _ in 0..10 {
            h.layout_at(300.0, 200.0);
            h.after_frame();
            if h.close_requested() {
                break;
            }
        }
        assert!(h.close_requested());
        let text = std::fs::read_to_string(receipt).unwrap();
        if json {
            let value: serde_json::Value = serde_json::from_str(&text).unwrap();
            assert_eq!(value["ok"], false);
            assert_eq!(value["errors"][0], "product acceptance failed");
            assert_eq!(value["product_log"][0], "product diagnostic");
        } else {
            assert!(text.starts_with("RESULT fail"), "{text}");
            assert!(text.contains("FAIL: product acceptance failed"), "{text}");
            assert!(text.contains("product diagnostic"), "{text}");
        }
    }
}

/// Completion receives capture failures as well as script failures, and can
/// retain an interactive trial without continuing to drive it.
#[test]
fn product_completion_runs_once_and_preserves_failures_when_kept_open() {
    struct Completion {
        calls: Rc<Cell<usize>>,
        saw_failure: Rc<Cell<bool>>,
        reject: bool,
    }
    impl mesquite::Product for Completion {
        type State = App;
        type Logic = Logic;
        type View = Child;
        const KIND: &'static str = "completion-test";
        const SURFACE: &'static str = "app";
        const LOG_PREFIX: &'static str = "completion-test";
        fn sheet(&self) -> &str {
            SHEET
        }
        fn snapshot(&self, ctx: &mesquite::Ctx<'_, Self>, _: usize, _: f32) -> ProbeSnapshot {
            TestLane.snapshot(ctx, 0, 1.0)
        }
        fn complete(
            &mut self,
            _: &mut mesquite::Ctx<'_, Self>,
            outcome: &taproot::Outcome,
        ) -> Result<(), String> {
            self.calls.set(self.calls.get() + 1);
            self.saw_failure
                .set(!outcome.ok && outcome.log.iter().any(|line| line.contains("FAIL")));
            if self.reject {
                Err("durable receipt unavailable".into())
            } else {
                Ok(())
            }
        }
        fn close_on_completion(&self) -> bool {
            false
        }
    }
    for (name, script, reject, expected_failure) in [
        ("complete-ok", "settle 1\n", false, false),
        ("complete-capture-failure", "capture lost\n", false, true),
        ("complete-product-failure", "settle 1\n", true, false),
    ] {
        let dir = scratch(name);
        let path = dir.join("test.scn");
        std::fs::write(&path, script).unwrap();
        let calls = Rc::new(Cell::new(0));
        let saw_failure = Rc::new(Cell::new(false));
        let mut lane = mesquite::Lane::from_config(
            LaneConfig {
                scenario: path,
                capture_dir: Some(dir.clone()),
                receipt: None,
            },
            Completion {
                calls: calls.clone(),
                saw_failure: saw_failure.clone(),
                reject,
            },
            cambium_genet_winit_host::read_file,
        )
        .unwrap();
        let mut h = Harness::with_hooks(
            Init {
                state: App::default(),
                logic: root as Logic,
                sheet: SHEET.into(),
                fonts: vec![],
                images: vec![],
            },
            HostHooks {
                after_frame: Box::new(move |ctx| lane.after_frame(ctx)),
                ..inert_hooks()
            },
        );
        for _ in 0..160 {
            h.layout_at(300.0, 200.0);
            h.after_frame();
        }
        assert_eq!(calls.get(), 1);
        assert_eq!(saw_failure.get(), expected_failure);
        assert!(!h.close_requested(), "interactive trial was closed");
        let receipt = std::fs::read_to_string(dir.join("scenario.done")).unwrap();
        assert!(
            receipt.starts_with(if reject || expected_failure {
                "RESULT fail"
            } else {
                "RESULT ok"
            }),
            "{receipt}"
        );
        if reject {
            assert!(receipt.contains("durable receipt unavailable"));
        }
    }
}

#[test]
fn scenario_keys_use_retained_focus_and_button_activation() {
    let (receipt, h) = run(
        "keys",
        "click role:button Count\nkey Enter\nkey Space\nassert snap count == 3\n",
    );
    assert!(receipt.starts_with("RESULT ok"), "{receipt}");
    assert_eq!(h.state().count, 3);
}
