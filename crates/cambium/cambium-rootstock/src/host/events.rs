// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

/// An application-level close request. Native window chrome and an app's own
/// Close command deliberately use the same path.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CloseRequest {
    /// The operating system asked the window to close.
    Native,
    /// The application queued [`crate::WindowCommand::Close`].
    Command,
}

/// What the application decided to do with a [`CloseRequest`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CloseDisposition {
    /// Keep the window visible and keep the event loop alive.
    KeepVisible,
    /// Hide the window but keep the event loop and application state alive.
    Hide,
    /// End the event loop.
    Exit,
}

/// CPU-side timings from the most recent retained relayout.
///
/// This remains available to windowless harnesses, where no frame is
/// presented and [`FrameProfile`] is therefore not populated.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RelayoutProfile {
    pub layout_update_us: u64,
    pub layout_tick_us: u64,
    pub layout_apply_us: u64,
    pub layout_rebuild_us: u64,
    pub style_resolve_us: u64,
    pub layout_with_text_us: u64,
    pub content_extent_us: u64,
    pub layout_mutations: u64,
    pub layout_rebuilt: bool,
    pub leaf_boxes_us: u64,
    pub leaf_render_us: u64,
    pub leaf_repaints: u64,
}

/// CPU-side timings for the most recently presented frame.
///
/// These are host pipeline spans, not GPU timestamp queries. They answer which
/// retained/layout/paint/present stage consumed the caller thread and carry
/// netrender's own raster attribution when that backend published it. A host
/// can expose the value in a receipt without teaching the application about
/// wgpu, Vello, or a platform event loop.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FrameProfile {
    /// Additive, packet-bound UI/worker observations. Missing clocks stay null.
    pub native_work: crate::NativeWorkAttribution,
    pub producers: crate::ProducerFrameStats,
    pub total_us: u64,
    pub frame_hook_us: u64,
    pub relayout_us: u64,
    pub layout_update_us: u64,
    pub layout_tick_us: u64,
    pub layout_apply_us: u64,
    pub layout_rebuild_us: u64,
    pub style_resolve_us: u64,
    pub layout_with_text_us: u64,
    pub content_extent_us: u64,
    pub layout_mutations: u64,
    pub layout_rebuilt: bool,
    pub leaf_boxes_us: u64,
    pub leaf_render_us: u64,
    pub leaf_repaints: u64,
    pub leaf_fragments_us: u64,
    pub ime_us: u64,
    pub emit_scene_us: u64,
    pub shadows_us: u64,
    pub raster_us: u64,
    /// The complete raster was retained; no new checked raster was submitted.
    pub raster_reused: bool,
    /// Submission/empty collection attempts have no newly presented picture.
    pub native_pending: bool,
    /// UI preparation and encoding, without asynchronous qualification time.
    pub native_submit_us: u64,
    /// UI collection and publication, including surface acquisition.
    pub native_collect_us: u64,
    /// Preparation to presentation wall time, including worker qualification.
    pub native_latency_us: u64,
    /// A measured native timing partition is present, including zero spans.
    pub native_timing_available: bool,
    /// Preparation to scheduling the completion job.
    pub native_preseal_us: u64,
    /// Scheduling to completion-worker start; not host-thread work.
    pub native_queue_us: u64,
    /// Worker qualification wall time, including polls/scopes; not GPU time.
    pub native_worker_us: u64,
    /// Qualified result ready to successful channel receipt, including dispatch.
    pub native_ready_to_collect_us: u64,
    /// Successful channel receipt to completed publication, before host A11y.
    pub native_postcollect_us: u64,
    /// Sum of the five disjoint intervals; boundaries are rounded cumulatively.
    pub native_timeline_us: u64,
    pub acquire_us: u64,
    pub clear_us: u64,
    pub compose_us: u64,
    pub present_us: u64,
    pub capture_us: u64,
    pub pointer_us: u64,
    pub a11y_us: u64,
    /// Adapter tree synchronization, including a legacy combined adapter's drain.
    pub a11y_publish_us: u64,
    /// Independent action draining and host dispatch; no tree publication.
    pub a11y_actions_us: u64,
    pub raster_total_us: u64,
    pub tile_invalidate_us: u64,
    pub dirty_tile_rebuild_us: u64,
    pub master_compose_us: u64,
    pub vello_render_us: u64,
    pub dirty_tiles: u64,
}

/// A pointer event an application asks the host to deliver to itself, in
/// logical window coordinates.
///
/// The host owns hit testing, capture, and the dispatch order, so an
/// application that drives itself — a `taproot` scenario clicking a
/// resolved element, a demo replaying a gesture — must not re-roll that
/// routing. It queues one of these instead and the host runs it through the
/// same path a real mouse takes, so a self-driven receipt exercises the
/// production code rather than a parallel one.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum HostPointer {
    /// Move the cursor (hover restyle, Enter/Leave, drag tracking).
    Moved(f32, f32),
    /// Press the left button at this point.
    Press(f32, f32),
    /// Release the left button at this point.
    Release(f32, f32),
    /// Press the right button at this point.
    ///
    /// Secondary capture is opt-in through the retained pointer handler.
    SecondaryPress(f32, f32),
    /// Release the right button through the host's qualified capture route.
    SecondaryRelease(f32, f32),
    /// Cancel the current capture without delivering an activating release.
    Cancel,
    /// Turn the wheel with the cursor at this point and these modifiers held.
    ///
    /// The notch runs through [`Host::wheel`](crate::Host::wheel): the nearest
    /// `on_wheel` ancestor of the hit node gets a genuine
    /// [`WheelEvent`](cambium::WheelEvent) (both axes, the cursor-local
    /// position, the modifiers), and what no handler took falls to the host's
    /// scroll and Ctrl-zoom defaults. The modifiers are held for this notch
    /// only; the host's own modifier state is restored after it.
    Wheel {
        /// Where the cursor is, in logical window coordinates.
        x: f32,
        y: f32,
        /// The scroll delta in the direction the content moves, as
        /// [`Host::wheel`](crate::Host::wheel) takes it: a positive `dy`
        /// advances toward the end of the document.
        dx: f32,
        dy: f32,
        /// Whether `dx` and `dy` are logical pixels or wheel lines.
        unit: WheelUnit,
        /// The modifiers held while it turns.
        modifiers: Modifiers,
    },
}

/// What a [`HostPointer::Wheel`] delta counts.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum WheelUnit {
    /// Logical pixels: a trackpad's pixel delta.
    #[default]
    Pixels,
    /// Wheel lines: a mouse wheel's notches, each [`HOST_WHEEL_LINE_PX`] logical
    /// pixels, as the desktop host converts a winit `LineDelta`.
    Lines,
}

/// A request to bring one retained DOM node into view, queued by
/// [`AppCtx::scroll_into_view`] and resolved on the host's next layout.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScrollIntoView {
    /// The node to reveal, by the identity [`AppCtx::painted_rect`] takes.
    pub node: NodeId,
    /// Where it lands in its scroll area.
    pub align: ScrollAlign,
}

/// A deferred scroll request resolved against the next retained layout.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrollRequest {
    /// Reveal an element's painted box.
    IntoView(ScrollIntoView),
    /// Reveal a byte position in a node's text without moving an editor caret.
    Text {
        node: NodeId,
        byte: usize,
        align: ScrollAlign,
    },
}

/// What the event loop should do on an idle turn.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IdlePolicy {
    /// Nothing is pending; sleep until the next event.
    Wait,
    /// Something time-based is live (an overlay scrollbar mid-fade): repaint
    /// and come back after this long.
    Animate(std::time::Duration),
    /// A screen reader acted while the app was idle: repaint so the queued
    /// accessibility action is drained.
    A11yWake,
}

impl FrameProfile {
    /// One compact line suitable for a headed receipt's diagnostic log.
    pub fn summary(self) -> String {
        format!(
            "native-pending={} native-submit={}us native-collect={}us native-latency={}us total={}us hook={}us relayout={}us layout-update={}us tick={}us apply={}us layout-rebuild={}us style-resolve={}us layout-text={}us content-extent={}us mutations={} layout-rebuilt={} leaf-boxes={}us leaf-render={}us leaf-repaints={} fragments={}us emit={}us raster={}us raster-reused={} acquire={}us clear={}us compose={}us capture={}us present={}us a11y={}us raster-inner={}us invalidate={}us rebuild={}us master={}us vello={}us dirty-tiles={} producer={}us staging={}us producer-calls={} stages={} work=[{}]",
            self.native_pending,
            self.native_submit_us,
            self.native_collect_us,
            self.native_latency_us,
            self.total_us,
            self.frame_hook_us,
            self.relayout_us,
            self.layout_update_us,
            self.layout_tick_us,
            self.layout_apply_us,
            self.layout_rebuild_us,
            self.style_resolve_us,
            self.layout_with_text_us,
            self.content_extent_us,
            self.layout_mutations,
            self.layout_rebuilt,
            self.leaf_boxes_us,
            self.leaf_render_us,
            self.leaf_repaints,
            self.leaf_fragments_us,
            self.emit_scene_us,
            self.raster_us,
            self.raster_reused,
            self.acquire_us,
            self.clear_us,
            self.compose_us,
            self.capture_us,
            self.present_us,
            self.a11y_us,
            self.raster_total_us,
            self.tile_invalidate_us,
            self.dirty_tile_rebuild_us,
            self.master_compose_us,
            self.vello_render_us,
            self.dirty_tiles,
            self.producers.render_us,
            self.producers.stage_us,
            self.producers.render_calls,
            self.producers.stages,
            self.native_work.summary(),
        )
    }
}

impl From<ScrollIntoView> for ScrollRequest {
    fn from(request: ScrollIntoView) -> Self {
        Self::IntoView(request)
    }
}
