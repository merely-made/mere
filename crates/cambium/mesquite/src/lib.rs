// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The generic scenario lane for a Cambium document host.
//!
//! One product's acceptance run grew the whole shape first: a
//! [`taproot::Scenario`] ticked once per *presented* frame inside
//! `after_frame`, a deferred window close that lets the last requested readback
//! land, capture arming through the host's own `read_frame`, PNG writing,
//! viewport-masked pixel comparison, bounded CPU cost accounting, and one JSON
//! receipt carrying captures, digests, snapshot fields, checkpoints, errors and
//! misses. None of that was about that product. This crate is that lane with
//! the product cut out of it.
//!
//! What stays the product's is everything that reads its own state: the typed
//! [`ProbeSnapshot`], its event stream, its `act` labels, which DOM element is
//! the viewport, where its default output directory is, and any verb its own
//! grammar adds. That is the [`Product`] trait — a small set of hooks a
//! consumer implements once, after which [`Lane`] runs the lifecycle.
//!
//! [`Lane::from_config`] also supports the established environment variables,
//! named PNG files, scripted file chooser and `RESULT ok` text sentinel. Native
//! file reading is supplied by the host. Both formats use the same frame pump,
//! deferred clicks, capture collection and product acceptance hooks. Text mode
//! preserves its 120-frame capture grace and accepts uniform nonblank frames;
//! the JSON constructor retains its eight-frame grace and detail checks.
//!
//! ```ignore
//! let mut lane = Lane::new(MyProduct, scenario, receipt, capture, exit_code)
//!     .with_frame_limit(Some(1800));
//! // in HostHooks::after_frame
//! lane.after_frame(ctx);
//! // in HostHooks::close_request
//! lane.request_close();
//! ```

use std::path::PathBuf;

use cambium_rootstock::{AppCtx, NodeId, meristem_bounds::RootView};
use taproot::ProbeSnapshot;

mod scenario;
pub use scenario::{CaptureRecord, LaneConfig};
mod checkpoints;
mod clicks;
pub use clicks::Clicks;
mod cost;
mod lane;
mod pixels;
mod probe;

pub use checkpoints::{Checkpoint, Checkpoints};
pub use cost::{CostObservation, Costs, Totals};
pub use lane::{Capture, Lane, capture_path};
pub use pixels::{PixelCheck, Viewport, ViewportTransform, create_parent, write_png};

/// The host context a [`Product`]'s hooks are handed, spelled once.
pub type Ctx<'a, P> =
    AppCtx<'a, <P as Product>::State, <P as Product>::Logic, <P as Product>::View>;

/// Everything the generic lane cannot know: the host's own types, and the
/// handful of readings that are about the product rather than about driving a
/// document host.
///
/// Implement `sheet` and `snapshot` to describe a surface. Other hooks have
/// defaults and can be added as the product's scenarios need them.
pub trait Product: Sized {
    /// The host application state the Cambium runner holds.
    type State: 'static;
    /// The view logic the runner diffs into a DOM.
    type Logic: FnMut(&Self::State) -> Self::View;
    /// The root view type that logic produces.
    type View: RootView<Self::State>;

    /// The receipt's `kind` field — what this lane is a receipt *of*.
    const KIND: &'static str;
    /// A stable surface name for hit attribution in probe's `Hit`.
    const SURFACE: &'static str;
    /// The prefix on every line the lane prints to stdout and stderr.
    const LOG_PREFIX: &'static str;

    /// The stylesheet the retained surface lays out under.
    fn sheet(&self) -> &str;

    /// A typed read of product state for assertions the DOM cannot express.
    /// `captures` is how many captures have completed; `opacity` is the value
    /// the `opacity` verb last set.
    fn snapshot(&self, ctx: &Ctx<'_, Self>, captures: usize, opacity: f32) -> ProbeSnapshot;

    /// Drain the semantic events emitted since the last call.
    fn drain_events(&mut self, _ctx: &mut Ctx<'_, Self>) -> Vec<String> {
        Vec::new()
    }

    /// Where captures land when a scenario names one without a path. Its file
    /// stem is also the fallback stem for generated capture names.
    fn default_capture_path(&self) -> PathBuf {
        PathBuf::from("capture.png")
    }

    /// Run one product-named command (the `act <label>` verb). `false` when no
    /// such command exists, so the driver fails loudly. The default refuses
    /// every label: a host whose controls are exercised *as controls*, through
    /// pointer delivery, wants exactly that.
    fn act(&mut self, ctx: &mut Ctx<'_, Self>, label: &str) -> bool {
        let _ = (ctx, label);
        false
    }

    /// Whether the product still has work the next scenario step must not
    /// race. `capture_pending` is the lane's own outstanding readback, which is
    /// always part of the answer. `None` means this product does not report
    /// quiescence; see [`taproot::Automatable::busy`].
    fn busy(&self, ctx: &Ctx<'_, Self>, capture_pending: bool) -> Option<bool> {
        let _ = (ctx, capture_pending);
        None
    }

    /// Mutable quiescence hook for products that advance their own pending work.
    /// Existing read-only implementations continue through `busy`.
    fn busy_mut(&mut self, ctx: &mut Ctx<'_, Self>, capture_pending: bool) -> Option<bool> {
        self.busy(ctx, capture_pending)
    }

    /// Inspect pixels before the lane releases the native readback.
    fn inspect(&mut self, _name: &str, _frame: &cambium_rootstock::Frame) {}

    /// Product-specific acceptance checks, applied to either receipt format.
    fn receipt_checks(&self, _captures: &[CaptureRecord]) -> Vec<String> {
        Vec::new()
    }

    /// Product diagnostics included in the text receipt and JSON `product_log`.
    fn receipt_lines(&self) -> Vec<String> {
        Vec::new()
    }

    /// Write product-specific receipts once, after all captures and acceptance
    /// checks complete. The outcome includes lane failures, not just script
    /// assertions. An error here also fails the shared receipt and exit code.
    fn complete(
        &mut self,
        _ctx: &mut Ctx<'_, Self>,
        _outcome: &taproot::Outcome,
    ) -> Result<(), String> {
        Ok(())
    }

    /// Whether completion closes the window. Interactive trials can retain
    /// their finished state while the lane stops driving it.
    fn close_on_completion(&self) -> bool {
        true
    }

    /// The product's rendered viewport, as a pixel mask for capture
    /// comparison. `None` disables pixel checks for that capture.
    fn viewport(&self, ctx: &Ctx<'_, Self>) -> Option<Viewport> {
        let _ = ctx;
        None
    }

    /// Where a click on a matched element should land, given its rect
    /// `[x, y, w, h]`. After scrolling, this is the visible portion of the
    /// element. The default is the rect's centre; a product whose viewport is
    /// CSS-transformed maps the centre through that transform.
    fn target_point(&self, ctx: &Ctx<'_, Self>, node: NodeId, rect: [f32; 4]) -> (f32, f32) {
        let _ = (ctx, node);
        (rect[0] + rect[2] * 0.5, rect[1] + rect[3] * 0.5)
    }

    /// The stylesheet to install for an `opacity <value>` verb. `None` records
    /// the value in the snapshot without restyling anything.
    fn opacity_sheet(&self, opacity: f32) -> Option<String> {
        let _ = opacity;
        None
    }

    /// This frame's cumulative redraw and upload totals, plus whether the frame
    /// is eligible for cost summaries and whether its scene is populated.
    fn cost_observation(&self, ctx: &Ctx<'_, Self>) -> CostObservation {
        let _ = ctx;
        CostObservation::default()
    }

    /// Handle a scenario verb the generic grammar did not recognize.
    /// `checkpoints` is the read-only view of what `remember` has taken, so a
    /// product verb compares against the same checkpoints the shared verbs do.
    fn app_step(
        &mut self,
        ctx: &mut Ctx<'_, Self>,
        checkpoints: Checkpoints<'_>,
        line: &str,
    ) -> Result<(), String> {
        let _ = (ctx, checkpoints);
        Err(format!("unknown scenario step: {line}"))
    }
}

#[cfg(test)]
mod tests;
