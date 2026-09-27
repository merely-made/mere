// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The self-driving scenario lane, one copy for every host.
//!
//! A taproot [`Scenario`] names controls by role and label, asserts on a typed
//! snapshot, and captures frames. Running one needs the same machinery in every
//! application: resolve a selector against the host's live layout, queue the
//! pointer back through [`HostPointer`] so the receipt exercised the shipping
//! input path, arm a readback of the next presented frame, and publish a
//! receipt once every step has run. [`ScenarioLane`] is that machinery.
//!
//! Two halves plug into it. The application supplies what is its own through
//! [`LaneApp`]: its stylesheet, a snapshot of its state, its semantic events,
//! its `act` verbs, any extra verbs, and whether work is still in flight. The
//! host supplies what differs by platform through [`LaneHost`]: how a frame is
//! read back, where evidence is kept, where a `file` step's file comes from,
//! and where the receipt goes. The desktop host reads frames at once and writes
//! files; a browser reads them back a few frames later and hands them to the
//! page.
//!
//! Selectors resolve through the host's own layout, never through taproot's
//! independent re-layout, whose font resolution can disagree with the one that
//! painted the frame. A click lands where the control actually is.
//!
//! The receipt starts `RESULT ok` or `RESULT fail`, then the scenario log, one
//! line per capture, the frame count, the application's lines, and any
//! failures.

use std::cell::RefCell;
use std::collections::BTreeSet;
use std::path::PathBuf;
use std::rc::Rc;

use cambium::{FileEvent, FileRequest, OpenedFile};
use taproot::{
    Automatable, Driveable, Hit, Outcome, ProbeSurface, Progress, Scenario, Selector,
    SelectorTarget,
};

use crate::meristem_bounds::RootView;
use crate::{AppCtx, CaptureFn, FileAnswer, FileChooser, Frame, HostPointer, WindowCommand};

/// Re-exported so an application builds its snapshot against the same taproot
/// the lane drives, without naming taproot itself.
pub use taproot::ProbeSnapshot;

/// Frames a capture may take to land before the lane records it as lost.
const CAPTURE_PATIENCE: u32 = 120;

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

/// What a host supplies to its lane: the parts that differ by platform.
pub trait LaneHost {
    /// Arm a readback of the next presented frame on `capture`, and return a
    /// poll that yields the frame once it has landed.
    fn arm_capture(&mut self, capture: &mut Option<CaptureFn>)
    -> Box<dyn FnMut() -> Option<Frame>>;

    /// Keep a captured frame as evidence, returning the file it went to, if
    /// the host keeps files.
    fn keep(&mut self, name: &str, frame: &Frame) -> Result<Option<PathBuf>, String>;

    /// The file a `file <argument>` step supplies to a waiting request.
    fn open_file(&mut self, argument: &str) -> Result<OpenedFile, String>;

    /// Publish the finished receipt.
    fn publish(&mut self, ok: bool, text: &str);

    /// Whether the application closes once the receipt is published.
    fn closes_on_finish(&self) -> bool {
        true
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
    poll: Box<dyn FnMut() -> Option<Frame>>,
    frames_waited: u32,
}

/// A running scenario, its evidence, and the application's and host's halves
/// of it.
pub struct ScenarioLane<A, H> {
    app: A,
    host: H,
    scenario: Option<Scenario>,
    captures: Vec<CaptureRecord>,
    errors: Vec<String>,
    pending: Option<PendingCapture>,
    finished: bool,
    /// A file request waiting for the script's `file` verb.
    parked_file: Rc<RefCell<Option<FileAnswer>>>,
    files_installed: bool,
}

/// The lane's file chooser: it parks a request until a `file` verb answers
/// it, so a scenario supplies a file without a dialog. A second request
/// answers the first with nothing chosen.
struct ParkedFiles(Rc<RefCell<Option<FileAnswer>>>);

impl FileChooser for ParkedFiles {
    fn open(&mut self, _request: &FileRequest, answer: FileAnswer) {
        if let Some(earlier) = self.0.borrow_mut().replace(answer) {
            earlier.send(FileEvent::default());
        }
    }
}

impl<A, H: LaneHost> ScenarioLane<A, H> {
    /// Parse `text` as a scenario.
    pub fn new(text: &str, app: A, host: H) -> Result<Self, String> {
        let scenario = Scenario::parse(text)?;
        Ok(Self {
            app,
            host,
            scenario: Some(scenario),
            captures: Vec::new(),
            parked_file: Rc::default(),
            files_installed: false,
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

    pub fn host(&self) -> &H {
        &self.host
    }

    pub fn host_mut(&mut self) -> &mut H {
        &mut self.host
    }

    pub fn captures(&self) -> &[CaptureRecord] {
        &self.captures
    }

    /// Whether the receipt has been published.
    pub fn finished(&self) -> bool {
        self.finished
    }

    /// Pump one step. Call from the `after_frame` hook, so every step reads a
    /// state that was actually rendered. Asks the host to close once the
    /// receipt is published, if the host closes then.
    pub fn drive<State: 'static, Logic, V>(&mut self, ctx: &mut AppCtx<'_, State, Logic, V>)
    where
        A: LaneApp<State, Logic, V>,
        Logic: FnMut(&State) -> V,
        V: RootView<State>,
    {
        if self.finished {
            return;
        }
        if !self.files_installed {
            *ctx.files = Some(Box::new(ParkedFiles(self.parked_file.clone())));
            self.files_installed = true;
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
                if self.host.closes_on_finish() {
                    *ctx.close = true;
                }
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
        let Some(frame) = (pending.poll)() else {
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
        let file = match self.host.keep(&name, &frame) {
            Ok(file) => file,
            Err(error) => {
                self.errors.push(error);
                None
            },
        };
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
        self.host.publish(ok, &lines.join("\n"));
    }

    /// Answer the file request waiting, with the host's file for `argument` or
    /// with nothing when it is `cancel`.
    fn answer_file(&mut self, argument: &str) -> Result<(), String> {
        let answer = self
            .parked_file
            .borrow_mut()
            .take()
            .ok_or_else(|| "no file request is waiting".to_string())?;
        if argument == "cancel" {
            answer.send(FileEvent::default());
            return Ok(());
        }
        let file = self.host.open_file(argument)?;
        answer.send(FileEvent { files: vec![file] });
        Ok(())
    }
}

/// The lane's view of the application for one tick: the hook's context, held
/// only as long as the driver needs it.
struct Probe<'a, 'c, A, H, State: 'static, Logic, V>
where
    Logic: FnMut(&State) -> V,
    V: RootView<State>,
{
    ctx: &'a mut AppCtx<'c, State, Logic, V>,
    lane: &'a mut ScenarioLane<A, H>,
}

impl<A, H, State: 'static, Logic, V> Automatable for Probe<'_, '_, A, H, State, Logic, V>
where
    A: LaneApp<State, Logic, V>,
    H: LaneHost,
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

impl<A, H, State: 'static, Logic, V> Driveable for Probe<'_, '_, A, H, State, Logic, V>
where
    A: LaneApp<State, Logic, V>,
    H: LaneHost,
    Logic: FnMut(&State) -> V,
    V: RootView<State>,
{
    fn capture(&mut self, name: &str) -> bool {
        let poll = self.lane.host.arm_capture(self.ctx.capture);
        self.lane.pending = Some(PendingCapture {
            name: name.to_string(),
            poll,
            frames_waited: 0,
        });
        true
    }

    /// `resize <width> <height>` in logical px, through the host's window-verb
    /// queue like every other window verb. `file <argument>` answers the file
    /// request waiting with the host's file for it, and `file cancel` answers
    /// it with nothing chosen. Anything else is the application's.
    fn app_step(&mut self, line: &str) -> Result<(), String> {
        if let Some(argument) = line.strip_prefix("file ") {
            return self.lane.answer_file(argument.trim());
        }
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
