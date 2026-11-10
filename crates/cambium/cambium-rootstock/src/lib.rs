// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The stock a Cambium application host is grafted onto.
//!
//! A rootstock is the body a scion is fitted to: one root system, and tops that
//! can be exchanged without regrowing it. That is the relation here. The winit
//! event source is one scion, a browser's DOM will be another, and the living
//! machinery they are both fitted to does not change between them.
//!
//! `cambium-genet-winit-host` grew as one crate because there was one event
//! source. A browser is a second one, and the machinery it would need is the
//! same machinery: retained layout over the runner's DOM, logical-coordinate
//! hit testing, input routing, focus and spatial navigation, frame pacing, and
//! accessibility projection. None of that is about winit; the winit host
//! carries only 64 winit references across 3544 lines.
//!
//! This crate is that machinery's new home. It owns no event loop, no window,
//! and no surface. An event-source adapter (winit today, DOM next) converts
//! platform events into the vocabulary here and drives it.
//!
//! ## What lives here first
//!
//! The input vocabulary, because it is the seam every other piece crosses. The
//! winit host's [`KeyPress`] already existed for a version of this reason: a
//! `winit::event::KeyEvent` cannot be constructed outside winit, so a host that
//! routed one could never be driven from a test. A host that routes one also
//! cannot be driven from a browser. Same defect, wider blast radius.

// The types an event source implements the seams against, re-exported so a
// source names this crate rather than four.
pub use document_session_api::DocumentA11yProjection;
pub use genet_render_host::RasterizeFailure;
pub use genet_scripted_dom::{NodeId, ScriptedDom};
pub use sprigging::LeafRegistry;

use genet_render::VisualMovement;

#[cfg(all(test, not(target_arch = "wasm32")))]
mod native_receiving_control;
#[cfg(test)]
mod projection_tests;

mod a11y_publication;
mod work_attribution;
pub use work_attribution::{
    NativeGpuWork, NativePreparationWork, NativeWorkAttribution, NativeWorkCall,
    NativeWorkCallKind, NativeWorkPacket, NativeWorkSpan, NativeWorkerWork,
};
mod semantics;
pub use a11y_provenance::SemanticNodeIdentity;
pub use a11y_publication::{
    A11yAdapterState, A11yPublication, A11yPublicationOutcome, A11yPublicationRefusal,
    A11yPublicationSkip, A11yPublicationSnapshot, A11ySync,
};
pub use semantics::{
    ProducerSemanticIssue, PublicationBindings, PublicationIssue, SemanticProjection,
    project_semantics,
};

mod files;
pub use files::{FileAnswer, FileChooser};
mod presentation;
#[cfg(not(target_arch = "wasm32"))]
pub use presentation::NativePresentedImage;
pub use presentation::{
    ContextGeneration, FrameContentMode, NativePresentationFailure, PendingPresentationCapture,
    PortableMaterializer, PreparedFrame, PreparedPaint, PreparedPresentationObserver,
    PresentationBackend, PresentationChoice, PresentationDriver, PresentationFailure,
    PresentationLifetime, PresentationState, WgpuDriver, WgpuSurfaceFactory, backend_attempts,
};
mod owned_layout;
/// What [`HostOptions::vector_images`] resolves to.
pub use genet_livery::VectorImage;
pub use owned_layout::{HostVectors, OwnedLayout, ScrollAlign, ScrollTarget};
#[cfg(feature = "embedding")]
pub mod embedding;
pub mod producer;
mod scene_producer;
pub use producer::{
    PortableCaptureError, PortablePaintCapture, PortableProducerError, ProducedAction,
    ProducedScene, ProducedTexture, ProducerAction, ProducerActionDispatch, ProducerChildLifetime,
    ProducerChildToken, ProducerContext, ProducerError, ProducerFrameInfo, ProducerFramePurpose,
    ProducerFrameStats, ProducerGpuRemapError, ProducerImageIdentity, ProducerNode,
    ProducerRegistrationError, ProducerRegistry, ProducerRenderFailure, ProducerRole,
    ProducerSemantics, ProducerSource, ResolvedAppearance, SourceAlpha, SourceEncoding,
    TextureProducer,
};
pub use scene_producer::SceneProducer;

/// The host's clock.
///
/// Re-exported so every host and adapter reads time from one place. This is not
/// a stylistic preference: `std::time::Instant::now()` compiles for
/// `wasm32-unknown-unknown` and panics at runtime, so a browser host that used
/// std time would type-check, build, ship, and then die on its first animated
/// frame. Nothing in a compiler catches that.
pub use web_time::{Duration, Instant};

/// The surface's containing frame, as the neutral machinery needs it.
///
/// Deliberately five methods, not a window abstraction. The host asks a window
/// for fifteen things, and the split between them is not arbitrary: these five
/// have honest browser analogues, and the other ten are desktop window
/// management, which lives with the adapter that has a window to manage.
///
/// | here | winit | browser |
/// |---|---|---|
/// | `request_redraw` | `Window::request_redraw` | `requestAnimationFrame` |
/// | `inner_size` | `Window::inner_size` | the canvas's client size |
/// | `scale_factor` | `Window::scale_factor` | `devicePixelRatio` |
/// | `set_ime_allowed` | `Window::set_ime_allowed` | focus on the editing host |
/// | `set_ime_cursor_area` | `Window::set_ime_cursor_area` | the composition rect |
///
/// Maximize, minimize, drag, close, the window menu and visibility are absent
/// on purpose. A browser tab does not maximize itself, and pretending otherwise
/// would put a method on this trait that one implementation could only answer
/// by lying.
pub trait HostWindow {
    /// Ask for another frame.
    fn request_redraw(&self);

    /// The drawable size in physical pixels.
    fn inner_size(&self) -> (u32, u32);

    /// Physical pixels per logical pixel.
    fn scale_factor(&self) -> f64;

    /// Whether the platform should offer text input.
    fn set_ime_allowed(&self, allowed: bool);

    /// Where the composition window should sit, in logical coordinates.
    fn set_ime_cursor_area(&self, x: f64, y: f64, width: f64, height: f64);

    /// What the platform keeps for itself across the window's top edge, in
    /// logical pixels.
    ///
    /// Unlike maximize, both implementations can answer this honestly: a PWA
    /// in a browser window has exactly this concept, and it is the same one --
    /// Window Controls Overlay's `env(titlebar-area-*)` names the complement,
    /// the part of the title bar the page may draw into. So it belongs here
    /// rather than on the winit side alone.
    ///
    /// The default reserves nothing, which is the truth on a platform where
    /// the host draws every control itself (Windows and Linux CSD). macOS is
    /// the case that is not zero: its traffic lights stay system-drawn even
    /// under a full-size content view.
    fn titlebar_insets(&self) -> TitlebarInsets {
        TitlebarInsets::default()
    }
}

/// What the platform reserved along the top edge, in logical pixels.
///
/// Insets rather than the spec's rect because that is the part the platform
/// actually knows. What the page gets is the window's width minus these, which
/// needs a width the window has and the platform layer does not -- so the
/// neutral layer computes the published values from this.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TitlebarInsets {
    /// Reserved at the leading edge — macOS's traffic lights.
    pub left: f32,
    /// Reserved at the trailing edge — a platform that puts its controls right.
    pub right: f32,
    /// How tall the reserved strip is. Zero means nothing is reserved at all,
    /// so a stylesheet reserving `--titlebar-area-height` gets a no-op rather
    /// than a mystery gap.
    pub height: f32,
}

impl TitlebarInsets {
    /// Nothing reserved.
    pub const NONE: Self = Self {
        left: 0.0,
        right: 0.0,
        height: 0.0,
    };

    /// The four Window-Controls-Overlay values for a window `width` logical
    /// pixels across: the run of title bar the page may draw into.
    ///
    /// Clamped at zero so a window narrower than its own controls publishes an
    /// empty area rather than a negative width that would lay out inside out.
    pub fn titlebar_area(&self, width: f32) -> (f32, f32, f32, f32) {
        let available = (width - self.left - self.right).max(0.0);
        (self.left, 0.0, available, self.height)
    }

    /// The published declarations, in the order x, y, width, height.
    ///
    /// Custom properties rather than `env()` because genet has no `env()` yet,
    /// exactly as `--app-region` stands in for the `app-region` longhand. A
    /// stylesheet reads `var(--titlebar-area-height)` today and `env(...)` the
    /// day livery grows it; the host publishing does not change either way.
    pub fn declarations(&self, width: f32) -> String {
        let (x, y, w, h) = self.titlebar_area(width);
        format!(
            "--titlebar-area-x: {x}px; --titlebar-area-y: {y}px;              --titlebar-area-width: {w}px; --titlebar-area-height: {h}px;"
        )
    }
}

/// What the host presents onto.
///
/// The present core beneath this is already windowing-neutral: a
/// [`RenderCore`](genet_render_host::RenderCore) boots wgpu and owns the
/// renderer, and a surface is created against whatever the platform hands it.
/// `SurfaceHost` is that pair for a winit window; the browser presenter
/// assembles the same pair against an `HtmlCanvasElement`, differing in one
/// line, the surface target.
///
/// So this trait exists to name the pair, not to abstract a difference. Only
/// four methods are required; the rest reach the core, which every
/// implementation has.
pub trait Surface {
    /// The present core: wgpu device, queue, and the netrender renderer.
    fn core(&self) -> &genet_render_host::RenderCore;

    /// Native hosts opt into nonblocking, shared-core frame submission.
    /// Custom surfaces without an owned Core keep the checked legacy route.
    #[cfg(not(target_arch = "wasm32"))]
    fn shared_core(&self) -> Option<std::sync::Arc<genet_render_host::RenderCore>> {
        None
    }

    /// The surface's texture format.
    fn format(&self) -> wgpu::TextureFormat;

    /// Reconfigure for a new size.
    fn resize(&mut self, width: u32, height: u32);

    /// Acquire the next backbuffer, or `None` when the surface is not
    /// presentable this frame.
    fn acquire(&self) -> Option<wgpu::SurfaceTexture>;
    /// Typed native acquisition. Existing custom/headless surfaces retain
    /// their unavailable behavior; event-source surfaces distinguish loss.
    fn acquire_status(&self) -> genet_render_host::SurfaceAcquire {
        if let Some(loss) = self.core().device_failure() {
            return genet_render_host::SurfaceAcquire::DeviceLost(loss);
        }
        match self.acquire() {
            Some(frame) => genet_render_host::SurfaceAcquire::Frame(frame),
            None => genet_render_host::SurfaceAcquire::Unavailable {
                reason: "custom surface unavailable".into(),
            },
        }
    }

    /// The native netrender renderer, for one operation. Browser callers use
    /// [`try_renderer`](Self::try_renderer) so a busy or lost Core is refused.
    #[cfg(not(target_arch = "wasm32"))]
    fn renderer(&self) -> RendererAccess<'_> {
        self.core().renderer()
    }

    /// The netrender renderer for one operation, or the attributed refusal
    /// (device loss, or another owner of the browser frame gate).
    fn try_renderer(&self) -> Result<RendererAccess<'_>, genet_render_host::RasterizeFailure> {
        try_renderer(self.core())
    }

    /// The wgpu device backing the renderer.
    fn device(&self) -> &wgpu::Device {
        self.core().device()
    }

    /// The wgpu queue backing the renderer.
    fn queue(&self) -> &wgpu::Queue {
        self.core().queue()
    }
}

/// One operation's access to a core's renderer. In the browser it is Genet's
/// guard over the shared frame gate, so it must not outlive the operation;
/// native hosts borrow the renderer directly.
#[cfg(target_arch = "wasm32")]
pub type RendererAccess<'a> = genet_render_host::RendererAccess<'a>;
/// One operation's access to a core's renderer (see the browser definition).
#[cfg(not(target_arch = "wasm32"))]
pub type RendererAccess<'a> = &'a netrender::Renderer;

/// Checked access to `core`'s renderer for one operation.
pub fn try_renderer(
    core: &genet_render_host::RenderCore,
) -> Result<RendererAccess<'_>, genet_render_host::RasterizeFailure> {
    #[cfg(target_arch = "wasm32")]
    return core.try_renderer();
    #[cfg(not(target_arch = "wasm32"))]
    return Ok(core.renderer());
}

/// Retire an external image this host registered. Valid after device loss
/// and without the browser frame gate, unlike live renderer access.
pub(crate) fn retire_external_image(core: &genet_render_host::RenderCore, key: u64) {
    #[cfg(target_arch = "wasm32")]
    core.retire_owned_external_image(key);
    #[cfg(not(target_arch = "wasm32"))]
    core.renderer().unregister_external_image(key);
}

/// Retire a retained fragment this host registered; valid after device loss.
pub(crate) fn retire_fragment(core: &genet_render_host::RenderCore, id: u64) {
    #[cfg(target_arch = "wasm32")]
    core.retire_owned_fragment(id);
    #[cfg(not(target_arch = "wasm32"))]
    let _ = core.renderer().remove_fragment(id);
}

/// What a screen reader asked for, kept as the action it actually requested.
///
/// `Click` and `Focus` stay apart because collapsing them lies to the reader:
/// navigating a list with a virtual cursor issues `Focus`, and turning that into
/// a click activates every control the reader merely moves across. The host
/// routes `Click` through its activation path and `Focus` through `set_focus`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum A11yAction {
    /// Activate the element: the reader's equivalent of a pointer click.
    Click,
    /// Move focus to the element, without activating it.
    Focus,
    /// Set an accessible numeric control's value without implying a click.
    SetValue(f64),
}

/// Host routing result, not a promise that application work completed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum A11yDispatch {
    Dispatched,
    Rejected(A11yRejection),
    /// The current producer advertised the action but declined to handle it.
    ProducerDeclined,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum A11yRejection {
    Unavailable,
    TargetAbsent,
    /// The queued request's host, document, mount or producer was replaced.
    StaleTarget,
    /// Its owning queue was disposed before the host could route the request.
    Cancelled,
    Disabled,
    Unsupported,
    InvalidValue,
}

/// Routing acknowledgement for one queued semantic request. Only the host
/// writes this result; `Dispatched` does not claim application completion.
/// Dropping or disposing its pending queue completes it as `Cancelled`.
#[derive(Clone)]
pub struct A11yReceipt(std::rc::Rc<std::cell::Cell<Option<A11yDispatch>>>);
impl A11yReceipt {
    pub fn result(&self) -> Option<A11yDispatch> {
        self.0.get()
    }
}
mod a11y_owner;
pub use a11y_owner::A11yOwner;
mod a11y_provenance;
mod a11y_queue;
pub(crate) use a11y_queue::QueuedA11y;

/// One drained screen-reader request: which action, on what.
#[derive(Clone, Debug, PartialEq)]
pub struct A11yRequest {
    pub action: A11yAction,
    pub target: A11yTarget,
    provenance: Option<a11y_provenance::A11yProvenance>,
}

/// What a reader's request lands on.
#[derive(Clone, Debug, PartialEq)]
pub enum A11yTarget {
    /// A DOM node of the application's tree.
    Node(NodeId),
    /// One action of a node a texture producer draws, as the producer's own
    /// button for it presents it ([`ProducerNode::actions`]).
    Produced(ProducedAction),
}

/// How a host publishes its accessible tree and collects what a reader asked
/// for.
///
/// This is the seam where the two event sources differ most, and the only one
/// that is not a translation. AccessKit builds a *parallel* tree that the
/// platform queries through a handle bound to a native window. A browser has no
/// such handle, but needs a parallel tree all the same: Cambium presents onto a
/// canvas, which a reader sees as one graphic, so the browser host lowers
/// [`project_semantics`] into DOM elements carrying ARIA.
///
/// The signature is what both can honestly implement. Notably absent is the
/// window: the winit implementation holds its own, and a browser has none to
/// hold.
///
/// Returning requests rather than dispatching them keeps the routing decision
/// with the host, which owns the activation and focus paths, rather than
/// splitting it across two implementations that would then have to agree.
pub trait Accessibility {
    /// Install only the native window root before revealing a receiving
    /// window. This is adapter bootstrap, not accepted DOM/producer content.
    fn prepare_native_window(&mut self) {}

    /// Current physical adapter/mirror attachment, independent of tree acceptance.
    fn adapter_state(&self) -> A11yAdapterState;

    /// Drain only requests bound to the last submitted tree, without reading
    /// contributors or publishing another projection. `Some(empty)` is an
    /// implemented drain with no work. `None` keeps the combined `sync` path
    /// for adapters that have not opted into independent action service.
    fn drain_actions(&mut self, _owner: &A11yOwner) -> Option<Vec<A11yRequest>> {
        None
    }

    /// Publish the current tree and drain what the reader asked for since the
    /// last call. Report the publisher outcome separately from action requests;
    /// neither the adapter state nor this outcome certifies actor completion.
    ///
    /// `focus` is the focused DOM node's opaque id, used as the tree's focus
    /// only when that node is really in the tree, so a stale id never points
    /// the reader at nothing.
    ///
    /// `layout_scale` is [`Host::layout_scale`] — device scale times UI zoom —
    /// and is what the projected boxes ride into the platform's physical client
    /// coordinates. It is passed rather than read off a window because the zoom
    /// half of it is the host's, not the window's.
    ///
    /// `producers` is asked for each texture-producer slot's own semantics
    /// ([`TextureProducer::semantics`]), which a host writes under the slot.
    ///
    /// `owner` is the publishing host lifetime. Freeze it with publication
    /// requests; never replace an old request's owner at event dispatch.
    ///
    /// `dom` is this window's subtree of the document: the whole document for
    /// a single window, its window-root's subtree under a forest.
    #[allow(clippy::too_many_arguments)]
    fn sync(
        &mut self,
        owner: &A11yOwner,
        dom: &WindowDom<'_>,
        layout: &OwnedLayout,
        leaves: &LeafRegistry<u64>,
        producers: &ProducerRegistry,
        focus: Option<u64>,
        layout_scale: f64,
    ) -> A11ySync;
}

mod document_projection;
pub use document_projection::document_projection;

/// How much an off-axis offset costs relative to on-axis distance. Above 1 so a
/// control that is roughly in line wins over a nearer one well off to the side —
/// which is what "down" means to a person looking at a column.
const OFF_AXIS_COST: f32 = 2.0;

/// The penalty for a candidate that does not overlap the current element's band
/// at all. Large enough that any aligned candidate beats any unaligned one,
/// finite so that a lone diagonal neighbour is still reachable rather than
/// stranding focus.
const NO_OVERLAP_PENALTY: f32 = 10_000.0;

/// Score `candidate` as a move from `from` in `dir`, or `None` when it is not in
/// that direction at all.
///
/// The rule is the ordinary spatial-navigation heuristic: travel along the axis,
/// penalized by how far off-axis you have to go, and penalized much harder for
/// leaving the current element's band entirely. Lower is better.
pub fn score(from: Box2, candidate: Box2, dir: Direction) -> Option<f32> {
    let (fx, fy) = from.centre();
    let (cx, cy) = candidate.centre();
    // A hair of tolerance, so two controls on the same visual row do not count
    // as being above or below each other through rounding.
    const EPS: f32 = 0.5;
    let (along, off, overlaps) = match dir {
        Direction::Right => (
            cx - fx,
            (cy - fy).abs(),
            candidate.y < from.y + from.h && from.y < candidate.y + candidate.h,
        ),
        Direction::Left => (
            fx - cx,
            (cy - fy).abs(),
            candidate.y < from.y + from.h && from.y < candidate.y + candidate.h,
        ),
        Direction::Down => (
            cy - fy,
            (cx - fx).abs(),
            candidate.x < from.x + from.w && from.x < candidate.x + candidate.w,
        ),
        Direction::Up => (
            fy - cy,
            (cx - fx).abs(),
            candidate.x < from.x + from.w && from.x < candidate.x + candidate.w,
        ),
    };
    if along <= EPS {
        return None;
    }
    let penalty = if overlaps { 0.0 } else { NO_OVERLAP_PENALTY };
    Some(along + off * OFF_AXIS_COST + penalty)
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;

#[cfg(test)]
#[path = "spatial_tests.rs"]
mod spatial_tests;

#[cfg(test)]
mod accessibility_tests;
mod capture;
mod frame;
mod host;
#[cfg(test)]
mod hover_tests;
mod input;
#[cfg(test)]
mod mount_probe_tests;
mod multi_host;
#[cfg(test)]
mod multi_host_tests;
#[cfg(all(test, not(target_arch = "wasm32")))]
mod native_a11y_tail_control;
mod recovery;
mod spatial;
mod tree;
mod wake;
mod window_dom;
mod window_verbs;

/// Bounds the host's view type without naming meristem at every use site.
#[path = "meristem_bounds.rs"]
pub mod meristem_bounds;

pub use capture::{
    CaptureOwner, CaptureScope, Frame, PendingFrame, read_frame, start_frame_readback,
    start_presentation_readback,
};
pub use host::{
    AppCtx, AppFrameInsets, AppHook, AppShared, CaptureFn, CaptureFrame, CaptureRefusal,
    CloseDisposition, CloseRequest, CloseRequestHook, FocusedTextHook, FocusedTextSlot, FrameHook,
    FrameProfile, HOST_WHEEL_LINE_PX, Hook, Host, HostDisposal, HostFont, HostHooks, HostImage,
    HostOptions, HostPointer, HostState, IdlePolicy, Init, KeyInterceptHook, PaintCaptureFn,
    PresentationObserver, PresentedFrame, RelayoutProfile, Runner, ScrollIntoView, ScrollRequest,
    StampedCaptureFn, StampedCaptureFrame, VectorImageSource, WheelUnit, WindowFrame, ZOOM_LADDER,
    env_size, fit_zoom, ladder_step,
};
pub use multi_host::{MultiHost, MultiRunner, WindowRequests, WindowSlot, WindowTree};
pub use tree::HostTree;
pub use wake::HostWake;
pub use window_dom::WindowDom;
pub use window_verbs::{AppRegion, WindowCommand, WindowCommands, WindowGeometry};

#[path = "input_types.rs"]
mod input_types;

pub use input_types::{Box2, Direction, Key, KeyPress, Modifiers, NamedKey};

#[path = "a11y_request.rs"]
mod a11y_request;
