// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! A self-driving scenario lane for applications on this host.
//!
//! A taproot [`Scenario`] names controls by role and label, asserts on a typed
//! snapshot, and captures frames. Running one needs the same machinery in every
//! application: resolve a selector against the host's live layout, queue the
//! pointer back through [`HostPointer`] so the receipt exercised the shipping
//! input path, arm a readback of the next presented frame, and write a receipt
//! once every step has run. The smoke example and woodshed each wrote that
//! themselves; [`ScenarioLane`] is the one copy.
//!
//! The application supplies only what is its own through [`LaneApp`]: its
//! stylesheet, a snapshot of its state, its semantic events, its `act` verbs,
//! any extra verbs, and whether work is still in flight. It builds a lane from
//! [`LaneConfig::from_env`] and calls [`ScenarioLane::drive`] from its
//! `after_frame` hook. With no scenario named, it builds no lane and runs as
//! usual.
//!
//! Selectors resolve through the host's own layout, as
//! [`Harness::resolve`](crate::Harness::resolve) does, never through taproot's
//! independent re-layout, whose font resolution can disagree with the one that
//! painted the window. A click lands where the control actually is.
//!
//! Evidence: each `capture <name>` writes `<name>.png` into the capture
//! directory when one is configured and records its size and digest. The
//! receipt starts `RESULT ok` or `RESULT fail`, then the scenario log, one line
//! per capture, and any failures. It is written to the configured receipt path,
//! or to `scenario.done` in the capture directory, and doubles as the sentinel a
//! harness waits for.

use std::cell::RefCell;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use cambium_rootstock::meristem_bounds::RootView;
use taproot::{
    Automatable, Driveable, Hit, Outcome, ProbeSurface, Progress, Scenario, Selector,
    SelectorTarget,
};

use crate::{AppCtx, Frame, HostPointer, WindowCommand, read_frame};

/// Re-exported so an application builds its snapshot against the same taproot
/// the lane drives, without naming taproot itself.
pub use taproot::ProbeSnapshot;

/// Frames a capture may take to land before the lane records it as lost.
const CAPTURE_PATIENCE: u32 = 120;

/// Where a lane reads its scenario and writes its evidence.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LaneConfig {
    pub scenario: PathBuf,
    pub capture_dir: Option<PathBuf>,
    pub receipt: Option<PathBuf>,
}

impl LaneConfig {
    /// Read `<PREFIX>_SCENARIO`, `<PREFIX>_CAPTURE_DIR` and `<PREFIX>_RECEIPT`.
    /// `None` when no scenario is named.
    pub fn from_env(prefix: &str) -> Option<Self> {
        let var = |suffix: &str| std::env::var_os(format!("{prefix}_{suffix}")).map(PathBuf::from);
        Some(Self {
            scenario: var("SCENARIO")?,
            capture_dir: var("CAPTURE_DIR"),
            receipt: var("RECEIPT"),
        })
    }

    /// The explicit receipt path, else `scenario.done` in the capture directory.
    pub fn receipt_path(&self) -> Option<PathBuf> {
        self.receipt.clone().or_else(|| {
            self.capture_dir
                .as_ref()
                .map(|dir| dir.join("scenario.done"))
        })
    }
}

/// What an application supplies to its lane.
///
/// Every method but [`sheet`](Self::sheet) and [`snapshot`](Self::snapshot)
/// has a default, so an application starts with two and adds verbs as its
/// scenarios need them.
pub trait LaneApp<State: 'static, Logic, V>
where
    Logic: FnMut(&State) -> V,
    V: RootView<State>,
{
    /// The stylesheet the application lays out under, for the text queries
    /// taproot answers from the DOM.
    fn sheet(&self) -> &str;

    /// A typed read of application state for `assert snap`.
    fn snapshot(&self, ctx: &AppCtx<'_, State, Logic, V>) -> ProbeSnapshot;

    /// Semantic events since the last call, for `assert event`.
    fn drain_events(&mut self, _ctx: &mut AppCtx<'_, State, Logic, V>) -> Vec<String> {
        Vec::new()
    }

    /// Run one named command, the `act <label>` verb. `false` if unknown.
    fn act(&mut self, _ctx: &mut AppCtx<'_, State, Logic, V>, _label: &str) -> bool {
        false
    }

    /// Handle a verb neither taproot nor the lane recognizes.
    fn app_step(
        &mut self,
        _ctx: &mut AppCtx<'_, State, Logic, V>,
        line: &str,
    ) -> Result<(), String> {
        Err(format!("unknown verb: {line}"))
    }

    /// Whether work is in flight that the next step must not race. `None`
    /// means the application does not report it, and `wait` burns its cap.
    fn busy(&mut self, _ctx: &mut AppCtx<'_, State, Logic, V>) -> Option<bool> {
        None
    }

    /// Look at a captured frame before the lane keeps only its record.
    fn inspect(&mut self, _name: &str, _frame: &Frame) {}

    /// Extra receipt lines, after the capture lines.
    fn receipt_lines(&self) -> Vec<String> {
        Vec::new()
    }

    /// Claims the receipt must hold beyond the lane's own, as failure reasons.
    /// Empty passes.
    fn receipt_checks(&self, _captures: &[CaptureRecord]) -> Vec<String> {
        Vec::new()
    }
}

/// One frame the lane read back.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CaptureRecord {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub digest: u64,
    pub blank: bool,
    pub file: Option<PathBuf>,
}

struct PendingCapture {
    name: String,
    sink: Rc<RefCell<Option<Frame>>>,
    frames_waited: u32,
}

/// A running scenario, its evidence, and the application's half of it.
pub struct ScenarioLane<A> {
    app: A,
    scenario: Option<Scenario>,
    config: LaneConfig,
    captures: Vec<CaptureRecord>,
    errors: Vec<String>,
    pending: Option<PendingCapture>,
    finished: bool,
}

impl<A> ScenarioLane<A> {
    /// Read and parse the configured scenario.
    pub fn new(config: LaneConfig, app: A) -> Result<Self, String> {
        let text = std::fs::read_to_string(&config.scenario).map_err(|error| {
            format!("scenario {} unreadable: {error}", config.scenario.display())
        })?;
        let scenario = Scenario::parse(&text)
            .map_err(|error| format!("scenario {} rejected: {error}", config.scenario.display()))?;
        Ok(Self {
            app,
            scenario: Some(scenario),
            config,
            captures: Vec::new(),
            errors: Vec::new(),
            pending: None,
            finished: false,
        })
    }

    pub fn app(&self) -> &A {
        &self.app
    }

    pub fn app_mut(&mut self) -> &mut A {
        &mut self.app
    }

    pub fn captures(&self) -> &[CaptureRecord] {
        &self.captures
    }

    /// Whether the receipt has been written.
    pub fn finished(&self) -> bool {
        self.finished
    }

    /// Pump one step. Call from the `after_frame` hook, so every step reads a
    /// state that was actually rendered. Asks the host to close once the
    /// receipt is written.
    pub fn drive<State: 'static, Logic, V>(&mut self, ctx: &mut AppCtx<'_, State, Logic, V>)
    where
        A: LaneApp<State, Logic, V>,
        Logic: FnMut(&State) -> V,
        V: RootView<State>,
    {
        if self.finished {
            return;
        }
        self.collect_capture::<State, Logic, V>();
        // Hold the steps while a capture is in flight, so the frame read back
        // is the state the scenario captured and a second capture cannot
        // replace the first.
        if self.pending.is_none()
            && let Some(mut scenario) = self.scenario.take()
        {
            let progress = scenario.tick(&mut Probe { ctx, lane: self });
            if progress == Progress::Done && self.pending.is_none() {
                self.finish::<State, Logic, V>(scenario.finish());
                *ctx.close = true;
            }
            self.scenario = Some(scenario);
        }
        // Every step is pumped by a frame, so an idle application would stall
        // the run rather than finish it.
        if let Some(window) = ctx.window {
            window.request_redraw();
        }
    }

    fn collect_capture<State: 'static, Logic, V>(&mut self)
    where
        A: LaneApp<State, Logic, V>,
        Logic: FnMut(&State) -> V,
        V: RootView<State>,
    {
        let Some(mut pending) = self.pending.take() else {
            return;
        };
        let frame = pending.sink.borrow_mut().take();
        let Some(frame) = frame else {
            pending.frames_waited += 1;
            if pending.frames_waited > CAPTURE_PATIENCE {
                self.errors.push(format!(
                    "capture {} never landed after {CAPTURE_PATIENCE} frames",
                    pending.name
                ));
            } else {
                self.pending = Some(pending);
            }
            return;
        };
        self.app.inspect(&pending.name, &frame);
        self.record(pending.name, frame);
    }

    fn record(&mut self, name: String, frame: Frame) {
        let file = self.config.capture_dir.as_ref().map(|dir| {
            let safe: String = name
                .chars()
                .map(|ch| {
                    if ch.is_ascii_alphanumeric() || ch == '-' {
                        ch
                    } else {
                        '_'
                    }
                })
                .collect();
            dir.join(format!("{safe}.png"))
        });
        if let Some(path) = &file
            && let Err(error) = write_png(path, &frame)
        {
            self.errors
                .push(format!("could not write {}: {error}", path.display()));
        }
        self.captures.push(CaptureRecord {
            name,
            width: frame.width,
            height: frame.height,
            digest: frame.digest(),
            blank: frame.is_blank(),
            file,
        });
    }

    fn finish<State: 'static, Logic, V>(&mut self, outcome: Outcome)
    where
        A: LaneApp<State, Logic, V>,
        Logic: FnMut(&State) -> V,
        V: RootView<State>,
    {
        self.finished = true;
        let blanks = self.captures.iter().filter(|capture| capture.blank).count();
        let distinct: BTreeSet<u64> = self.captures.iter().map(|capture| capture.digest).collect();
        let mut failures = self.errors.clone();
        if blanks > 0 {
            failures.push(format!("{blanks} captured frames were blank"));
        }
        failures.extend(self.app.receipt_checks(&self.captures));
        let ok = outcome.ok && failures.is_empty();

        let mut lines = vec![if ok { "RESULT ok" } else { "RESULT fail" }.to_string()];
        lines.extend(outcome.log);
        for capture in &self.captures {
            lines.push(format!(
                "capture {} {}x{} digest={:016x}{}{}",
                capture.name,
                capture.width,
                capture.height,
                capture.digest,
                capture
                    .file
                    .as_ref()
                    .map(|file| format!(" file={}", file.display()))
                    .unwrap_or_default(),
                if capture.blank { " BLANK" } else { "" },
            ));
        }
        lines.push(format!(
            "frames: {} captured, {blanks} blank, {} distinct digests",
            self.captures.len(),
            distinct.len(),
        ));
        lines.extend(self.app.receipt_lines());
        lines.extend(failures.iter().map(|failure| format!("FAIL: {failure}")));

        let text = lines.join("\n");
        eprintln!("[scenario] {text}");
        if let Some(path) = self.config.receipt_path() {
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            if let Err(error) = std::fs::write(&path, format!("{text}\n")) {
                eprintln!("[scenario] could not write {}: {error}", path.display());
            }
        }
    }
}

/// The lane's view of the application for one tick: the hook's context, held
/// only as long as the driver needs it.
struct Probe<'a, 'c, A, State: 'static, Logic, V>
where
    Logic: FnMut(&State) -> V,
    V: RootView<State>,
{
    ctx: &'a mut AppCtx<'c, State, Logic, V>,
    lane: &'a mut ScenarioLane<A>,
}

impl<A, State: 'static, Logic, V> Automatable for Probe<'_, '_, A, State, Logic, V>
where
    A: LaneApp<State, Logic, V>,
    Logic: FnMut(&State) -> V,
    V: RootView<State>,
{
    fn with_surfaces<R>(&self, f: impl FnOnce(&[ProbeSurface<'_>]) -> R) -> R {
        let dom = self.ctx.runner.dom();
        let dom_ref = dom.borrow();
        let (width, height) = self.ctx.logical_size;
        f(&[ProbeSurface {
            name: "app",
            dom: &dom_ref,
            rect: [0.0, 0.0, width, height],
            sheet: self.lane.app.sheet(),
        }])
    }

    fn selector_target(&self, selector: &Selector) -> SelectorTarget {
        let nodes = {
            let dom = self.ctx.runner.dom();
            let dom_ref = dom.borrow();
            taproot::matching(&dom_ref, selector)
        };
        nodes
            .into_iter()
            .find_map(|node| self.ctx.painted_rect(node))
            .map_or(SelectorTarget::Miss, |(x, y, width, height)| {
                SelectorTarget::Hit(Hit {
                    surface: "app",
                    point: (x + width / 2.0, y + height / 2.0),
                })
            })
    }

    fn snapshot(&self) -> ProbeSnapshot {
        self.lane
            .app
            .snapshot(self.ctx)
            .with_field("captures", self.lane.captures.len().to_string())
    }

    fn drain_events(&mut self) -> Vec<String> {
        self.lane.app.drain_events(self.ctx)
    }

    fn act(&mut self, label: &str) -> bool {
        self.lane.app.act(self.ctx, label)
    }

    fn press(&mut self, x: f32, y: f32) {
        self.ctx.pointer.push(HostPointer::Press(x, y));
    }

    fn moved(&mut self, x: f32, y: f32) {
        self.ctx.pointer.push(HostPointer::Moved(x, y));
    }

    fn release(&mut self, x: f32, y: f32) {
        self.ctx.pointer.push(HostPointer::Release(x, y));
    }

    fn busy(&mut self) -> Option<bool> {
        self.lane.app.busy(self.ctx)
    }
}

impl<A, State: 'static, Logic, V> Driveable for Probe<'_, '_, A, State, Logic, V>
where
    A: LaneApp<State, Logic, V>,
    Logic: FnMut(&State) -> V,
    V: RootView<State>,
{
    fn capture(&mut self, name: &str) -> bool {
        let sink = Rc::new(RefCell::new(None::<Frame>));
        let out = sink.clone();
        *self.ctx.capture = Some(Box::new(move |surface, view, width, height| {
            *out.borrow_mut() = read_frame(surface, view, width, height);
        }));
        self.lane.pending = Some(PendingCapture {
            name: name.to_string(),
            sink,
            frames_waited: 0,
        });
        true
    }

    /// `resize <width> <height>` in logical px, through the host's window-verb
    /// queue like every other window verb. Anything else is the application's.
    fn app_step(&mut self, line: &str) -> Result<(), String> {
        let mut parts = line.split_whitespace();
        if parts.next() == Some("resize") {
            let mut size = || parts.next().and_then(|value| value.parse::<f64>().ok());
            let (Some(width), Some(height)) = (size(), size()) else {
                return Err("resize wants a width and a height".to_string());
            };
            self.ctx
                .window_commands
                .push(WindowCommand::Resize(width, height));
            return Ok(());
        }
        self.lane.app.app_step(self.ctx, line)
    }
}

fn write_png(path: &Path, frame: &Frame) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let file = std::fs::File::create(path).map_err(|error| error.to_string())?;
    let mut encoder = png::Encoder::new(std::io::BufWriter::new(file), frame.width, frame.height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().map_err(|error| error.to_string())?;
    writer
        .write_image_data(&frame.rgba)
        .map_err(|error| error.to_string())?;
    writer.finish().map_err(|error| error.to_string())
}
