// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The lane lifecycle: tick per presented frame, arm and collect captures,
//! honour a frame limit, defer the close, write the receipt, set the exit code.

use std::{
    cell::{Cell, RefCell},
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
    rc::Rc,
};

use crate::{CaptureBackend, Readback, capture::NativeCapture};
use cambium_rootstock::{Duration, Instant, PresentedFrame};
use serde::Serialize;
use taproot::{Outcome, Progress, Scenario};

use crate::{
    Ctx, Product,
    cost::Costs,
    pixels::{PixelCheck, Viewport, create_parent, write_png},
    probe::Probe,
};

type PaintReadback = Rc<RefCell<Option<Result<Vec<u8>, String>>>>;
type SealReadback = Rc<RefCell<Option<Result<crate::pairing::SealedProjection, String>>>>;

enum PendingReadback {
    Legacy(Readback),
    Stamped(crate::StampedReadback),
}
struct PendingPairing {
    run: String,
    request: u64,
    seal: SealReadback,
}

struct PendingCapture {
    name: String,
    path: Option<PathBuf>,
    armed: u64,
    readback: PendingReadback,
    pairing: Option<PendingPairing>,
    viewport: Option<Viewport>,
    paint: Option<PaintReadback>,
}

/// One completed capture, as it appears in the receipt.
#[derive(Serialize)]
pub struct Capture {
    pub(crate) name: String,
    pub(crate) path: Option<PathBuf>,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) digest: String,
    pub(crate) fields: BTreeMap<String, String>,
    pub(crate) viewport: Option<Viewport>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) paint_path: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) pairing: Option<crate::CapturePairing>,
}
impl Capture {
    pub fn fields(&self) -> &BTreeMap<String, String> {
        &self.fields
    }
    pub fn pairing(&self) -> Option<&crate::CapturePairing> {
        self.pairing.as_ref()
    }
}

#[derive(Serialize)]
pub(crate) struct Diagnostics {
    /// Observation point in this lane, not the source's frame identity.
    sampled_at_lane_frame: u64,
    batch: crate::DiagnosticBatch,
}

/// The scenario lane, generic over one [`Product`].
pub struct Lane<P: Product> {
    pub(crate) product: P,
    scenario: Option<Scenario>,
    had_scenario: bool,
    outcome: Option<Outcome>,
    receipt: Option<PathBuf>,
    final_capture: Option<PathBuf>,
    final_armed: bool,
    pending: Option<PendingCapture>,
    pub(crate) captures: Vec<Capture>,
    pub(crate) pixel_checks: Vec<PixelCheck>,
    pub(crate) opacity: f32,
    pub(crate) checkpoints: BTreeMap<String, BTreeMap<String, String>>,
    pub(crate) errors: Vec<String>,
    pub(crate) misses: RefCell<Vec<String>>,
    pub(crate) clicks: crate::Clicks,
    exit_code: Rc<Cell<i32>>,
    frame_limit: Option<u32>,
    frames: u64,
    redraws: u64,
    unpresented_redraws: u64,
    last_frame_presentation: Option<PresentedFrame>,
    unpresented_since: Option<Instant>,
    presentation_timeout: Duration,
    finished: bool,
    pub(crate) costs: Costs,
    pub(crate) script_files: Option<crate::scenario::ScriptFiles>,
    records: Vec<crate::CaptureRecord>,
    capture_backend: Box<dyn CaptureBackend>,
    capture_paint: bool,
    pub(crate) diagnostics: Option<Diagnostics>,
    projection_limits: crate::CaptureProjectionLimits,
    projection_bytes: usize,
    paired_requests: usize,
    pairing_run: Option<String>,
    last_presentation: Option<cambium_rootstock::PresentedFrame>,
    next_capture_request: u64,
}

impl<P: Product> Lane<P> {
    /// A lane over `product`, optionally driving `scenario`, writing `receipt`
    /// and a last `final_capture`, and reporting through `exit_code`.
    pub fn new(
        product: P,
        scenario: Option<Scenario>,
        receipt: Option<PathBuf>,
        final_capture: Option<PathBuf>,
        exit_code: Rc<Cell<i32>>,
    ) -> Self {
        Self {
            product,
            had_scenario: scenario.is_some(),
            scenario,
            outcome: None,
            receipt,
            final_capture,
            final_armed: false,
            pending: None,
            captures: Vec::new(),
            pixel_checks: Vec::new(),
            opacity: 1.0,
            checkpoints: BTreeMap::new(),
            errors: Vec::new(),
            misses: RefCell::new(Vec::new()),
            clicks: crate::Clicks::default(),
            exit_code,
            frame_limit: None,
            frames: 0,
            redraws: 0,
            unpresented_redraws: 0,
            last_frame_presentation: None,
            unpresented_since: None,
            presentation_timeout: Duration::from_secs(10),
            finished: false,
            costs: Costs::default(),
            script_files: None,
            records: Vec::new(),
            capture_backend: Box::new(NativeCapture),
            capture_paint: std::env::var_os("MESQUITE_CAPTURE_PAINT")
                .is_some_and(|value| value == "1"),
            diagnostics: None,
            projection_limits: crate::CaptureProjectionLimits::default(),
            projection_bytes: 0,
            paired_requests: 0,
            pairing_run: None,
            last_presentation: None,
            next_capture_request: 1,
        }
    }

    /// Opt into a resource-preserving `.paintlist` beside each saved PNG.
    /// Defaults to `MESQUITE_CAPTURE_PAINT=1`. This is a capture/replay aid,
    /// not a timing run: full font bytes are serialized without elision.
    /// External GPU images remain references, not portable pixel payloads.
    pub fn set_paint_capture(&mut self, enabled: bool) {
        self.capture_paint = enabled;
    }

    /// Read a scenario using the established environment/path conventions and
    /// text receipt format. The host supplies native file access; Mesquite owns
    /// scripted chooser answers, capture timing, and completion.
    pub fn from_config(
        config: crate::LaneConfig,
        product: P,
        reader: crate::scenario::FileReader,
    ) -> Result<Self, String> {
        let text = std::fs::read_to_string(&config.scenario).map_err(|error| {
            format!("scenario {} unreadable: {error}", config.scenario.display())
        })?;
        let scenario = Scenario::parse(&text)
            .map_err(|error| format!("scenario {} rejected: {error}", config.scenario.display()))?;
        let mut lane = Self::new(
            product,
            Some(scenario),
            config.receipt_path(),
            None,
            Rc::new(Cell::new(0)),
        );
        lane.script_files = Some(crate::scenario::ScriptFiles::new(config, reader));
        Ok(lane)
    }

    /// Use the host's readback mechanism with the shared capture lifecycle.
    pub fn with_capture_backend(mut self, backend: impl CaptureBackend + 'static) -> Self {
        self.capture_backend = Box::new(backend);
        self
    }

    /// Bound a continuous native presentation stall independently of frame
    /// budgets. Readback polling continues during the wait. Defaults to ten
    /// seconds; windowless explicit harness turns do not use this deadline.
    #[must_use]
    pub fn with_presentation_timeout(mut self, timeout: Duration) -> Self {
        self.presentation_timeout = timeout;
        self
    }

    /// Bound copied correlated metadata; a zero refuses requested pairing.
    pub fn with_capture_projection_limits(
        mut self,
        limits: crate::CaptureProjectionLimits,
    ) -> Self {
        self.projection_limits = limits;
        self
    }

    pub fn product(&self) -> &P {
        &self.product
    }
    pub fn product_mut(&mut self) -> &mut P {
        &mut self.product
    }
    pub fn captures(&self) -> &[crate::CaptureRecord] {
        &self.records
    }
    /// Immutable receipts; observing them does not consume another reader.
    pub fn capture_receipts(&self) -> &[Capture] {
        &self.captures
    }
    pub fn finished(&self) -> bool {
        self.finished
    }

    /// Bounds scenario work. An already requested readback/final capture may
    /// use the following frame before the host closes.
    #[must_use]
    pub fn with_frame_limit(mut self, limit: Option<u32>) -> Self {
        self.frame_limit = limit;
        self
    }

    /// Defer native close until the last requested frame and receipt are saved.
    pub fn request_close(&mut self) {
        if self.finished || self.outcome.is_some() {
            return;
        }
        self.outcome = Some(if let Some(scenario) = self.scenario.take() {
            self.errors
                .push("window closed before the scenario completed".into());
            scenario.finish()
        } else {
            Outcome {
                ok: true,
                log: Vec::new(),
            }
        });
    }

    /// Whether a capture readback is still outstanding.
    pub(crate) fn capture_pending(&self) -> bool {
        self.pending.is_some()
    }

    /// Whether `frames` has reached the configured limit. No limit never does.
    pub(crate) fn frame_limit_reached(&self, frames: u64) -> bool {
        self.frame_limit
            .is_some_and(|limit| frames >= u64::from(limit))
    }

    /// The receipt as it will be written. Separated from [`Self::finish`] so
    /// its shape is provable without a live host.
    pub(crate) fn receipt_value(
        &self,
        ok: bool,
        final_fields: &BTreeMap<String, String>,
    ) -> serde_json::Value {
        let log = self
            .outcome
            .as_ref()
            .map_or(&[] as &[String], |o| o.log.as_slice());
        let mut receipt = serde_json::json!({
            "ok": ok, "kind": P::KIND, "frames": self.frames,
            "redraws": self.redraws, "unpresented_redraws": self.unpresented_redraws,
            "scenario": self.had_scenario, "scenario_log": log,
            "errors": self.errors, "final": final_fields,
            "checkpoints": self.checkpoints, "captures": self.captures,
            "pixel_checks": self.pixel_checks,
            "cost": self.costs.report(),
            "product_log": self.product.receipt_lines(),
        });
        if let Some(diagnostics) = &self.diagnostics {
            receipt["diagnostics"] = serde_json::json!(diagnostics);
        }
        receipt
    }

    // Explicit windowless turns are a test clock. A native turn counts only
    // when the host reports a new successful presentation identity.
    fn admit_frame(
        &mut self,
        native: bool,
        presentation: Option<PresentedFrame>,
        now: Instant,
    ) -> bool {
        self.redraws += 1;
        let presented = !native
            || presentation.is_some_and(|frame| {
                self.last_frame_presentation
                    .is_none_or(|last| last.host != frame.host || frame.sequence > last.sequence)
            });
        if presented {
            self.frames += 1;
            self.last_frame_presentation = presentation;
            self.unpresented_since = None;
        } else {
            self.unpresented_redraws += 1;
            if self.scenario.is_some()
                || self.pending.is_some()
                || self.frame_limit.is_some()
                || self.outcome.is_some()
            {
                self.unpresented_since.get_or_insert(now);
            }
        }
        presented
    }

    /// Call after every redraw attempt. Successful native presentations advance
    /// the scenario and frame budgets; failed attempts still poll readbacks.
    pub fn after_frame(&mut self, ctx: &mut Ctx<'_, P>) {
        self.after_frame_at(ctx, Instant::now());
    }

    fn after_frame_at(&mut self, ctx: &mut Ctx<'_, P>, now: Instant) {
        if self.finished {
            return;
        }
        if let Some(files) = &mut self.script_files {
            files.install(ctx.files);
        }
        let presented = self.admit_frame(ctx.window.is_some(), ctx.presentation, now);
        if presented {
            let observation = self.product.cost_observation(ctx);
            if let Err(why) = self.costs.observe(
                self.frames,
                ctx.frame_profile,
                observation.totals,
                self.pending.is_some(),
                observation.valid,
                observation.populated,
            ) {
                self.errors.push(why);
            }
        }
        self.collect_capture(ctx);
        if self
            .unpresented_since
            .is_some_and(|since| now.duration_since(since) >= self.presentation_timeout)
        {
            self.errors.push(format!(
                "native presentation stalled for {} ms ({} redraws, {} presented frames)",
                self.presentation_timeout.as_millis(),
                self.redraws,
                self.frames
            ));
            // These callbacks belong to this lane's pending request. Cancel
            // them before finishing, so a late wake cannot manufacture a receipt.
            if let Some(pending) = self.pending.take() {
                match pending.readback {
                    PendingReadback::Legacy(_) => *ctx.capture = None,
                    PendingReadback::Stamped(_) => {
                        *ctx.capture_stamped = None;
                        *ctx.presentation_observer = None;
                    },
                }
                if pending.paint.is_some() {
                    *ctx.capture_paint = None;
                }
            }
            self.outcome = Some(self.scenario.take().map_or(
                Outcome {
                    ok: false,
                    log: Vec::new(),
                },
                |scenario| scenario.finish(),
            ));
            self.final_armed = true;
        } else if presented {
            let clicked = match self.clicks.after_frame(ctx, |ctx, node, rect| {
                self.product.target_point(ctx, node, rect)
            }) {
                Ok(clicked) => clicked,
                Err(error) => {
                    self.errors.push(error);
                    true
                },
            };
            if !clicked
                && self.pending.is_none()
                && let Some(mut scenario) = self.scenario.take()
            {
                let progress = scenario.tick(&mut Probe { ctx, lane: self });
                if progress == Progress::Done {
                    self.outcome = Some(scenario.finish());
                } else {
                    self.scenario = Some(scenario);
                }
            }
        }
        if self.outcome.is_none() && self.frame_limit_reached(self.frames) {
            if self.had_scenario {
                self.errors
                    .push("frame limit ran out before the scenario completed".into());
            }
            self.outcome = Some(self.scenario.take().map_or(
                Outcome {
                    ok: true,
                    log: Vec::new(),
                },
                |scenario| scenario.finish(),
            ));
        }
        if self.outcome.is_some() && self.pending.is_none() {
            if !self.final_armed {
                self.final_armed = true;
                if let Some(path) = self.final_capture.clone()
                    && let Err(why) = self.arm_capture(ctx, "final".into(), Some(path))
                {
                    self.errors.push(why);
                }
            }
            if self.pending.is_none() {
                self.finish(ctx);
            }
        }
        if !self.finished
            && (self.scenario.is_some()
                || self.pending.is_some()
                || self.frame_limit.is_some()
                || self.outcome.is_some())
            && let Some(window) = ctx.window
        {
            window.request_redraw();
        }
    }

    pub(crate) fn arm_capture(
        &mut self,
        ctx: &mut Ctx<'_, P>,
        name: String,
        path: Option<PathBuf>,
    ) -> Result<(), String> {
        if self.pending.is_some()
            || ctx.capture.is_some()
            || ctx.capture_stamped.is_some()
            || ctx.presentation_observer.is_some()
        {
            return Err("another capture is still pending".into());
        }
        if self.capture_paint && ctx.capture_paint.is_some() {
            return Err("another paint capture is still pending".into());
        }
        if self.capture_paint && path.is_none() {
            return Err("paint capture requires a saved PNG path".into());
        }
        if self.capture_paint
            && path
                .as_ref()
                .and_then(|path| path.extension())
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| extension.eq_ignore_ascii_case("paintlist"))
        {
            return Err("PNG and paint capture paths must differ".into());
        }
        if let Some(path) = &path
            && self
                .records
                .iter()
                .any(|capture| capture.file.as_ref() == Some(path))
        {
            return Err(format!("capture path already used: {}", path.display()));
        }
        let observer = self.product.capture_observer();
        let viewport = if observer.is_none() {
            self.product.viewport(ctx)
        } else {
            None
        };
        let (readback, pairing) = if let Some(observer) = observer {
            let limits = self.projection_limits;
            if limits.max_projection_bytes == 0
                || limits.max_total_bytes == 0
                || self.paired_requests >= limits.max_captures
            {
                return Err("capture projection retention limit exceeded".into());
            }
            if observer.run.is_empty() || observer.run.len() > 128 {
                return Err("capture pairing requires a bounded fresh product run identity".into());
            }
            if self
                .pairing_run
                .as_ref()
                .is_some_and(|run| run != &observer.run)
            {
                return Err("capture pairing run changed inside a lane".into());
            }
            let request = self.next_capture_request;
            let next_request = request
                .checked_add(1)
                .ok_or("capture request sequence exhausted")?;
            let readback = match self.capture_backend.arm_stamped(
                ctx.capture_stamped,
                crate::CaptureRequest {
                    run: observer.run.clone(),
                    request,
                },
            ) {
                Ok(readback) => readback,
                Err(error) => {
                    *ctx.capture_stamped = None;
                    return Err(error);
                },
            };
            let seal: SealReadback = Rc::new(RefCell::new(None));
            let sink = seal.clone();
            let run = observer.run;
            let measured_run = run.clone();
            *ctx.presentation_observer = Some(Box::new(move |ctx, presentation| {
                *sink.borrow_mut() = Some((observer.observe)(ctx, presentation).and_then(
                    |projection| {
                        // Include run/request/frame identity as well as the frozen
                        // projection in admission accounting.
                        #[derive(Serialize)]
                        struct Admission<'a> {
                            run: &'a str,
                            request: u64,
                            presentation: crate::Presentation,
                            projection: &'a crate::CaptureProjection,
                        }
                        let metadata = Admission {
                            run: &measured_run,
                            request,
                            presentation: presentation.into(),
                            projection: &projection,
                        };
                        let encoded_bytes =
                            crate::pairing::encoded_size(&metadata, limits.max_projection_bytes)?;
                        Ok(crate::pairing::SealedProjection {
                            presentation,
                            projection,
                            encoded_bytes,
                        })
                    },
                ));
            }));
            self.next_capture_request = next_request;
            self.paired_requests += 1;
            self.pairing_run = Some(run.clone());
            (
                PendingReadback::Stamped(readback),
                Some(PendingPairing { run, request, seal }),
            )
        } else {
            (
                PendingReadback::Legacy(self.capture_backend.arm(ctx.capture)),
                None,
            )
        };
        let paint = self.capture_paint.then(|| {
            let sink: PaintReadback = Rc::new(RefCell::new(None));
            let callback_sink = sink.clone();
            *ctx.capture_paint = Some(Box::new(move |envelope| {
                *callback_sink.borrow_mut() =
                    Some(postcard::to_allocvec(&envelope).map_err(|error| error.to_string()));
            }));
            sink
        });
        self.pending = Some(PendingCapture {
            name,
            path,
            armed: self.frames,
            readback,
            pairing,
            viewport,
            paint,
        });
        Ok(())
    }

    fn collect_capture(&mut self, ctx: &Ctx<'_, P>) {
        let Some(mut pending) = self.pending.take() else {
            return;
        };
        let result = match &mut pending.readback {
            PendingReadback::Legacy(readback) => {
                readback().map(|result| result.map(|pixels| (pixels, None)))
            },
            PendingReadback::Stamped(readback) => readback().map(|result| {
                result.map(|frame| (frame.pixels, Some((frame.presentation, frame.identity))))
            }),
        };
        let Some(result) = result else {
            let patience = if self.script_files.is_some() {
                120
            } else {
                self.capture_backend.patience()
            };
            if self.frames.saturating_sub(pending.armed) > patience {
                self.errors.push(if self.script_files.is_some() {
                    format!(
                        "capture {} never landed after {patience} presented frames",
                        pending.name
                    )
                } else {
                    format!("capture {} never reached a presented frame", pending.name)
                });
            } else {
                self.pending = Some(pending);
            }
            return;
        };
        let (frame, presentation) = match result {
            Ok(frame) => frame,
            Err(why) => {
                self.errors.push(format!("capture {}: {why}", pending.name));
                return;
            },
        };
        if frame.width == 0
            || frame.height == 0
            || frame.rgba.len() != frame.width as usize * frame.height as usize * 4
        {
            self.errors.push(format!(
                "capture {} has invalid pixel storage",
                pending.name
            ));
            return;
        }
        let sealed = match self.collect_pairing(&pending, presentation, &frame) {
            Ok(sealed) => sealed,
            Err(error) => {
                self.errors
                    .push(format!("capture {}: {error}", pending.name));
                return;
            },
        };
        self.product.inspect(&pending.name, &frame);
        let paint_path = match write_paint_sidecar(pending.path.as_deref(), pending.paint.as_ref())
        {
            Ok(path) => path,
            Err(why) => {
                self.errors
                    .push(format!("capture {} paint: {why}", pending.name));
                return;
            },
        };
        if let Some(path) = &pending.path
            && let Err(why) = write_png(path, &frame)
        {
            self.errors.push(format!("capture {}: {why}", pending.name));
            return;
        }
        if frame.is_blank()
            || (self.script_files.is_none()
                && !frame
                    .rgba
                    .chunks_exact(4)
                    .any(|pixel| pixel != &frame.rgba[..4]))
        {
            self.errors.push(format!(
                "capture {} contains no visible {} detail",
                pending.name,
                P::LOG_PREFIX
            ));
        }
        let (fields, viewport, pairing) = if let Some(sealed) = sealed {
            self.projection_bytes += sealed.encoded_bytes;
            self.last_presentation = Some(sealed.presentation);
            let identity = pending.pairing.as_ref().expect("validated pairing");
            (
                sealed.projection.fields,
                sealed.projection.viewport,
                Some(crate::CapturePairing {
                    run: identity.run.clone(),
                    request: identity.request,
                    presentation: sealed.presentation.into(),
                    product_projection: sealed.projection.product,
                }),
            )
        } else {
            (
                self.product
                    .snapshot(ctx, self.captures.len() + 1, self.opacity)
                    .fields,
                pending.viewport,
                None,
            )
        };
        self.records.push(crate::CaptureRecord {
            name: pending.name.clone(),
            width: frame.width,
            height: frame.height,
            digest: frame.digest(),
            blank: frame.is_blank(),
            file: pending.path.clone(),
        });
        self.captures.push(Capture {
            name: pending.name,
            path: pending.path,
            width: frame.width,
            height: frame.height,
            digest: format!("{:016x}", frame.digest()),
            fields,
            viewport,
            paint_path,
            pairing,
        });
    }

    fn collect_pairing(
        &self,
        pending: &PendingCapture,
        frame_id: Option<(cambium_rootstock::PresentedFrame, crate::CaptureRequest)>,
        pixels: &cambium_rootstock::Frame,
    ) -> Result<Option<crate::pairing::SealedProjection>, String> {
        let Some(pairing) = &pending.pairing else {
            return if frame_id.is_none() {
                Ok(None)
            } else {
                Err("unexpected stamped readback".into())
            };
        };
        let sealed = pairing
            .seal
            .borrow_mut()
            .take()
            .ok_or("presented product projection is missing")??;
        let (frame_id, identity) = frame_id.ok_or("presented readback identity is missing")?;
        if identity.run != pairing.run || identity.request != pairing.request {
            return Err("readback run or request identity does not match capture".into());
        }
        if frame_id != sealed.presentation
            || frame_id.sequence == 0
            || frame_id.host == 0
            || frame_id.width != pixels.width
            || frame_id.height != pixels.height
            || !frame_id.layout_scale.is_finite()
            || frame_id.layout_scale <= 0.0
        {
            return Err("readback does not match sealed presentation".into());
        }
        if self
            .last_presentation
            .is_some_and(|last| last.host != frame_id.host || last.sequence >= frame_id.sequence)
        {
            return Err("stale or foreign presentation readback".into());
        }
        if self
            .projection_bytes
            .checked_add(sealed.encoded_bytes)
            .is_none_or(|total| total > self.projection_limits.max_total_bytes)
        {
            return Err("capture projection total byte limit exceeded".into());
        }
        Ok(Some(sealed))
    }

    pub(crate) fn named_capture(&self, name: &str) -> Option<PathBuf> {
        if !self.capture_backend.writes_files() {
            return None;
        }
        if let Some(files) = &self.script_files {
            return files.capture_path(name);
        }
        Some(capture_path(
            name,
            self.final_capture.as_deref(),
            self.receipt.as_deref(),
            &self.product.default_capture_path(),
        ))
    }

    fn text_receipt(&self, ok: bool) -> String {
        let mut lines = vec![if ok { "RESULT ok" } else { "RESULT fail" }.to_string()];
        lines.extend(
            self.outcome
                .as_ref()
                .expect("completion requested")
                .log
                .iter()
                .cloned(),
        );
        for c in &self.records {
            lines.push(format!(
                "capture {} {}x{} digest={:016x}{}{}",
                c.name,
                c.width,
                c.height,
                c.digest,
                c.file
                    .as_ref()
                    .map(|p| format!(" file={}", p.display()))
                    .unwrap_or_default(),
                if c.blank { " BLANK" } else { "" }
            ));
        }
        for capture in &self.captures {
            if let Some(pairing) = &capture.pairing {
                lines.push(format!(
                    "capture-pairing {} {}",
                    capture.name,
                    serde_json::json!({
                        "pairing": pairing, "fields": capture.fields, "viewport": capture.viewport,
                        "digest": capture.digest, "width": capture.width, "height": capture.height
                    })
                ));
            }
        }
        let blanks = self.records.iter().filter(|c| c.blank).count();
        let distinct: BTreeSet<_> = self.records.iter().map(|c| c.digest).collect();
        lines.push(format!(
            "frames: {} captured, {blanks} blank, {} distinct digests",
            self.records.len(),
            distinct.len()
        ));
        lines.push(format!(
            "presentation: {} frames, {} redraws, {} unpresented redraws",
            self.frames, self.redraws, self.unpresented_redraws
        ));
        lines.extend(self.product.receipt_lines());
        if let Some(diagnostics) = &self.diagnostics {
            // Value payloads and integer timing fields have a JSON encoding;
            // the shared write path reports actual export/I/O failures.
            lines.push(format!("diagnostics {}", serde_json::json!(diagnostics)));
        }
        lines.extend(self.errors.iter().map(|e| format!("FAIL: {e}")));
        lines.join("\n") + "\n"
    }

    fn finish(&mut self, ctx: &mut Ctx<'_, P>) {
        self.finished = true;
        if let Err(why) = self.costs.finish() {
            self.errors.push(why);
        }
        self.errors.extend(self.misses.borrow_mut().drain(..));
        self.errors
            .extend(self.product.receipt_checks(&self.records));
        let final_state = self
            .product
            .snapshot(ctx, self.captures.len(), self.opacity);
        if self.script_files.is_none()
            && let Some(error) = final_state.field("error").filter(|value| *value != "none")
        {
            self.errors.push(format!("scene producer: {error}"));
        }
        let outcome = self.outcome.as_ref().expect("completion requested");
        let completion = Outcome {
            ok: outcome.ok && self.errors.is_empty() && self.exit_code.get() == 0,
            log: outcome
                .log
                .iter()
                .cloned()
                .chain(self.errors.iter().map(|error| format!("FAIL: {error}")))
                .collect(),
        };
        if let Err(error) = self.product.complete(ctx, &completion) {
            self.errors.push(format!("product completion: {error}"));
        }
        let attachment = self.product.diagnostic_attachment(ctx);
        self.collect_diagnostics(attachment);
        let outcome = self.outcome.as_ref().expect("completion requested");
        let mut ok = outcome.ok && self.errors.is_empty() && self.exit_code.get() == 0;
        let receipt = if self.script_files.is_some() {
            Ok(self.text_receipt(ok).into_bytes())
        } else {
            serde_json::to_vec_pretty(&self.receipt_value(ok, &final_state.fields))
                .map_err(|e| e.to_string())
        };
        if let Some(path) = &self.receipt {
            let result = create_parent(path)
                .and_then(|()| std::fs::write(path, receipt?).map_err(|e| e.to_string()));
            if let Err(why) = result {
                ok = false;
                self.errors
                    .push(format!("receipt {}: {why}", path.display()));
            }
        }
        let tag = P::LOG_PREFIX;
        for line in &outcome.log {
            println!("{tag}: {line}");
        }
        for error in &self.errors {
            eprintln!("{tag}: FAIL: {error}");
        }
        println!(
            "{tag}: RESULT {} ({} frames, {} captures)",
            if ok { "ok" } else { "fail" },
            self.frames,
            self.captures.len()
        );
        if !ok {
            self.exit_code.set(1);
        }
        *ctx.close |= self.product.close_on_completion();
    }

    pub(crate) fn collect_diagnostics(
        &mut self,
        attachment: Result<Option<crate::DiagnosticBatch>, String>,
    ) {
        match attachment {
            Ok(Some(batch)) => {
                self.diagnostics = Some(Diagnostics {
                    sampled_at_lane_frame: self.frames,
                    batch,
                });
            },
            Ok(None) => {},
            Err(error) => self.errors.push(format!("diagnostic attachment: {error}")),
        }
    }
}

/// Where a scenario's `capture <name>` lands.
///
/// An absolute name, or one carrying a separator, is taken literally. Anything
/// else is a suffix on the run's own artifact stem: the explicit final capture,
/// else the receipt, else the product's default path.
pub fn capture_path(
    name: &str,
    final_capture: Option<&Path>,
    receipt: Option<&Path>,
    default: &Path,
) -> PathBuf {
    let path = Path::new(name);
    if path.is_absolute() || name.contains('/') || name.contains('\\') {
        return path.to_path_buf();
    }
    let base = final_capture.or(receipt).unwrap_or(default);
    let fallback = default
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("capture");
    let stem = base
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(fallback);
    let name: String = name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect();
    base.with_file_name(format!("{stem}-{name}.png"))
}

fn write_paint_sidecar(
    png_path: Option<&Path>,
    readback: Option<&PaintReadback>,
) -> Result<Option<PathBuf>, String> {
    use std::io::Write;

    let Some(readback) = readback else {
        return Ok(None);
    };
    let bytes = readback
        .borrow_mut()
        .take()
        .ok_or_else(|| "presented frame has no paired paint envelope".to_owned())??;
    let path = png_path
        .ok_or("paint capture requires a saved PNG path")?
        .with_extension("paintlist");
    create_parent(&path)?;
    // A failed/repeated run must not silently replace a previously captured
    // resource packet. Its PNG is written only after this succeeds.
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .map_err(|error| format!("{}: {error}", path.display()))?;
    file.write_all(&bytes)
        .map_err(|error| format!("{}: {error}", path.display()))?;
    Ok(Some(path))
}

#[cfg(test)]
mod presentation_tests {
    use super::*;
    use crate::tests::Headless;
    use cambium::{AnyView, GenetCtx, GenetElement, el};
    use cambium_rootstock::{
        Hook, Host, HostHooks, HostOptions, HostState, HostWake, HostWindow, Runner,
    };
    use genet_scripted_dom::ScriptedDom;
    use std::sync::Arc;

    type View = Box<dyn AnyView<(), (), GenetCtx, GenetElement>>;
    type Logic = fn(&()) -> View;
    fn view(_: &()) -> View {
        Box::new(el("main", "presentation test"))
    }
    struct Window;
    impl HostWindow for Window {
        fn request_redraw(&self) {}
        fn inner_size(&self) -> (u32, u32) {
            (100, 100)
        }
        fn scale_factor(&self) -> f64 {
            1.0
        }
        fn set_ime_allowed(&self, _: bool) {}
        fn set_ime_cursor_area(&self, _: f64, _: f64, _: f64, _: f64) {}
    }
    fn frame(sequence: u64) -> PresentedFrame {
        PresentedFrame {
            host: 1,
            sequence,
            width: 100,
            height: 100,
            layout_scale: 1.0,
        }
    }
    fn host(lane: Rc<RefCell<Lane<Headless>>>, clock: Rc<Cell<Instant>>) -> Host<(), Logic, View> {
        let mut state = HostState::new();
        state.window = Some(Box::new(Window));
        state.runner = Some(Runner::new(
            Rc::new(RefCell::new(ScriptedDom::new())),
            view as Logic,
            (),
        ));
        let wake = HostWake::new(state.wake_pending.clone(), Arc::new(|| {}));
        let hooks = HostHooks {
            after_frame: Box::new(move |ctx| lane.borrow_mut().after_frame_at(ctx, clock.get())),
            ..HostHooks::inert()
        };
        Host::new(HostOptions::default(), None, hooks, state, wake)
    }
    fn lane(scenario: &str) -> Lane<Headless> {
        Lane::new(
            Headless,
            Some(Scenario::parse(scenario).unwrap()),
            None,
            None,
            Rc::new(Cell::new(0)),
        )
    }

    #[test]
    fn unavailable_and_duplicate_native_turns_do_not_spend_scenario_frames() {
        let now = Instant::now();
        let clock = Rc::new(Cell::new(now));
        let lane = Rc::new(RefCell::new(lane("settle 2\nlog completed\n")));
        let mut host = host(lane.clone(), clock.clone());
        for _ in 0..150 {
            host.with_ctx(Hook::AfterFrame);
        }
        assert_eq!(lane.borrow().frames, 0);
        assert!(lane.borrow().outcome.is_none());
        host.s.last_redraw_presentation = Some(frame(1));
        host.with_ctx(Hook::AfterFrame);
        host.with_ctx(Hook::AfterFrame); // same presentation cannot satisfy settle
        assert_eq!(lane.borrow().frames, 1);
        assert!(lane.borrow().outcome.is_none());
        for sequence in 2..=5 {
            host.s.last_redraw_presentation = Some(frame(sequence));
            host.with_ctx(Hook::AfterFrame);
        }
        assert!(lane.borrow().finished);
        assert!(lane.borrow().errors.is_empty());
        assert_eq!(lane.borrow().unpresented_redraws, 151);
        assert_eq!(lane.borrow().redraws, lane.borrow().frames + 151);
    }

    struct Delayed;
    impl CaptureBackend for Delayed {
        fn arm(&mut self, _: &mut Option<cambium_rootstock::CaptureFn>) -> Readback {
            let mut polls = 0;
            Box::new(move || {
                polls += 1;
                (polls == 3).then(|| {
                    Ok(cambium_rootstock::Frame {
                        width: 2,
                        height: 1,
                        rgba: vec![1, 2, 3, 255, 4, 5, 6, 255],
                    })
                })
            })
        }
        fn writes_files(&self) -> bool {
            false
        }
    }
    #[test]
    fn asynchronous_readback_is_polled_while_native_presentation_is_unavailable() {
        let clock = Rc::new(Cell::new(Instant::now()));
        let lane = Rc::new(RefCell::new(
            lane("capture asynchronous\n").with_capture_backend(Delayed),
        ));
        let mut host = host(lane.clone(), clock);
        host.s.last_redraw_presentation = Some(frame(1));
        host.with_ctx(Hook::AfterFrame); // arm
        host.s.last_redraw_presentation = None;
        for _ in 0..3 {
            host.with_ctx(Hook::AfterFrame);
        }
        assert_eq!(lane.borrow().captures.len(), 1);
        assert_eq!(lane.borrow().frames, 1);
        assert!(lane.borrow().errors.is_empty());
        host.s.last_redraw_presentation = Some(frame(2));
        host.with_ctx(Hook::AfterFrame);
        assert!(lane.borrow().finished);
    }

    #[test]
    fn continuous_native_stall_has_an_independent_clock_deadline_and_honest_receipt() {
        let now = Instant::now();
        let clock = Rc::new(Cell::new(now));
        let lane = Rc::new(RefCell::new(
            lane("capture never\n").with_presentation_timeout(Duration::from_secs(1)),
        ));
        let mut host = host(lane.clone(), clock.clone());
        host.s.last_redraw_presentation = Some(frame(1));
        // A separate paint reader can coexist with this lane's legacy pixels.
        host.s.pending_paint_capture = Some(Box::new(|_| {}));
        host.with_ctx(Hook::AfterFrame); // arm native callbacks
        assert!(host.s.pending_capture.is_some());
        host.s.last_redraw_presentation = None;
        for _ in 0..150 {
            host.with_ctx(Hook::AfterFrame);
        }
        assert!(
            lane.borrow().errors.is_empty(),
            "failed acquisitions do not spend capture grace"
        );
        assert_eq!(lane.borrow().frames, 1);
        clock.set(now + Duration::from_secs(1));
        host.with_ctx(Hook::AfterFrame);
        let lane = lane.borrow();
        assert!(lane.finished);
        assert_eq!(lane.frames, 1);
        assert_eq!(lane.captures.len(), 0);
        assert!(
            lane.errors
                .iter()
                .any(|error| error.contains("native presentation stalled for 1000 ms"))
        );
        assert!(host.s.pending_capture.is_none());
        assert!(
            host.s.pending_paint_capture.is_some(),
            "do not cancel another reader's paint request"
        );
        assert!(host.s.close_requested);
        let receipt = lane.receipt_value(false, &BTreeMap::new());
        assert_eq!(receipt["frames"], 1);
        assert_eq!(receipt["redraws"], 152);
        assert_eq!(receipt["unpresented_redraws"], 151);
    }

    #[test]
    fn a_stall_before_the_first_presentation_finishes_without_arming_a_final_capture() {
        let now = Instant::now();
        let clock = Rc::new(Cell::new(now));
        let mut run = lane("log must wait for presentation\n")
            .with_presentation_timeout(Duration::from_secs(1));
        run.final_capture = Some(PathBuf::from("unpresented-final.png"));
        let lane = Rc::new(RefCell::new(run));
        let mut host = host(lane.clone(), clock.clone());
        host.with_ctx(Hook::AfterFrame);
        clock.set(now + Duration::from_secs(1));
        host.with_ctx(Hook::AfterFrame);
        let lane = lane.borrow();
        assert!(lane.finished);
        assert_eq!(lane.frames, 0);
        assert_eq!(lane.redraws, 2);
        assert_eq!(lane.unpresented_redraws, 2);
        assert!(lane.captures.is_empty());
        assert!(host.s.pending_capture.is_none());
        assert!(host.s.close_requested);
        assert!(
            !lane
                .outcome
                .as_ref()
                .unwrap()
                .log
                .iter()
                .any(|entry| entry.contains("must wait"))
        );
    }

    #[test]
    fn failed_host_redraw_clears_the_previous_presentation_identity() {
        let clock = Rc::new(Cell::new(Instant::now()));
        let lane = Rc::new(RefCell::new(lane("log pending\n")));
        let mut host = host(lane, clock);
        host.s.last_redraw_presentation = Some(frame(7));
        // No surface/render core: this attempt cannot possibly present.
        host.redraw();
        assert!(host.s.last_redraw_presentation.is_none());
    }
}
