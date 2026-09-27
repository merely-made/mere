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

use cambium_rootstock::{Frame, read_frame};
use serde::Serialize;
use taproot::{Outcome, Progress, Scenario};

use crate::{
    Ctx, Product,
    cost::Costs,
    pixels::{PixelCheck, Viewport, create_parent, write_png},
    probe::Probe,
};

type Readback = Rc<RefCell<Option<Result<Frame, String>>>>;

struct PendingCapture {
    name: String,
    path: Option<PathBuf>,
    armed: u64,
    readback: Readback,
    viewport: Option<Viewport>,
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
    finished: bool,
    pub(crate) costs: Costs,
    pub(crate) script_files: Option<crate::scenario::ScriptFiles>,
    records: Vec<crate::CaptureRecord>,
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
            finished: false,
            costs: Costs::default(),
            script_files: None,
            records: Vec::new(),
        }
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

    pub fn product(&self) -> &P {
        &self.product
    }
    pub fn product_mut(&mut self) -> &mut P {
        &mut self.product
    }
    pub fn captures(&self) -> &[crate::CaptureRecord] {
        &self.records
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
        serde_json::json!({
            "ok": ok, "kind": P::KIND, "frames": self.frames,
            "scenario": self.had_scenario, "scenario_log": log,
            "errors": self.errors, "final": final_fields,
            "checkpoints": self.checkpoints, "captures": self.captures,
            "pixel_checks": self.pixel_checks,
            "cost": self.costs.report(),
            "product_log": self.product.receipt_lines(),
        })
    }

    /// Call once per presented frame, from the host's `after_frame` hook.
    pub fn after_frame(&mut self, ctx: &mut Ctx<'_, P>) {
        if self.finished {
            return;
        }
        if let Some(files) = &mut self.script_files {
            files.install(ctx.files);
        }
        self.frames += 1;
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
        self.collect_capture(ctx);
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
        if self.pending.is_some() || ctx.capture.is_some() {
            return Err("another native capture is still pending".into());
        }
        if let Some(path) = &path
            && self
                .records
                .iter()
                .any(|capture| capture.file.as_ref() == Some(path))
        {
            return Err(format!("capture path already used: {}", path.display()));
        }
        let viewport = self.product.viewport(ctx);
        let readback = Rc::new(RefCell::new(None));
        let sink = readback.clone();
        *ctx.capture = Some(Box::new(move |surface, view, width, height| {
            *sink.borrow_mut() = Some(
                read_frame(surface, view, width, height)
                    .ok_or_else(|| "native frame readback failed".to_owned()),
            );
        }));
        self.pending = Some(PendingCapture {
            name,
            path,
            armed: self.frames,
            readback,
            viewport,
        });
        Ok(())
    }

    fn collect_capture(&mut self, ctx: &Ctx<'_, P>) {
        let Some(pending) = self.pending.take() else {
            return;
        };
        let result = pending.readback.borrow_mut().take();
        let Some(result) = result else {
            let patience = if self.script_files.is_some() { 120 } else { 8 };
            if self.frames.saturating_sub(pending.armed) > patience {
                self.errors.push(if self.script_files.is_some() {
                    format!(
                        "capture {} never landed after {patience} frames",
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
        let frame = match result {
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
        self.product.inspect(&pending.name, &frame);
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
        let fields = self
            .product
            .snapshot(ctx, self.captures.len() + 1, self.opacity)
            .fields;
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
            viewport: pending.viewport,
        });
    }

    pub(crate) fn named_capture(&self, name: &str) -> Option<PathBuf> {
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
        let blanks = self.records.iter().filter(|c| c.blank).count();
        let distinct: BTreeSet<_> = self.records.iter().map(|c| c.digest).collect();
        lines.push(format!(
            "frames: {} captured, {blanks} blank, {} distinct digests",
            self.records.len(),
            distinct.len()
        ));
        lines.extend(self.product.receipt_lines());
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
        *ctx.close = true;
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
