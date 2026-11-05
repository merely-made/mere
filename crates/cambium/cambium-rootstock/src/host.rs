// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The host proper: lifecycle, state, hooks, and the application-facing
//! context, with no event source of its own.
//!
//! What a winit window and a browser canvas have in common lives here. What
//! differs is grafted on: an event source converts platform events into this
//! crate's vocabulary and drives the methods below.

use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use cambium::{GenetAppRunner, TextInput};
use cambium_winit::ScrollbarFade;
use genet_render_host::RenderCore;
use genet_scripted_dom::NodeId;
use netrender::NetrenderOptions;

use crate::meristem_bounds::RootView;
use crate::wake::HostWake;
use crate::{Accessibility, HostWindow, Surface, WindowCommands, WindowGeometry};
use crate::{HostTree, OwnedLayout, ScrollAlign, ScrollTarget};
use crate::{KeyPress, Modifiers};

/// The runner shape this host drives: one state, one tree, unit actions.
pub type Runner<State, Logic, V> = GenetAppRunner<State, Logic, V, ()>;

fn next_host_identity() -> u64 {
    static NEXT: AtomicUsize = AtomicUsize::new(1);
    NEXT.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |next| {
        next.checked_add(1)
    })
    .expect("host identity exhausted") as u64
}

/// The browser zoom ladder, the rungs [`Host::step_ui_zoom`] steps between.
///
/// Borrowed rather than invented: every desktop browser offers these thirteen
/// stops, so Ctrl+plus in a Cambium application moves the interface by the
/// amount the person pressing it already expects.
pub const ZOOM_LADDER: [f32; 13] = [
    0.5, 0.67, 0.75, 0.8, 0.9, 1.0, 1.1, 1.25, 1.5, 1.75, 2.0, 2.5, 3.0,
];

/// Outer guards on the effective zoom. A fit factor is arithmetic on a window
/// size, so it is not bounded by the ladder; these keep a degenerate surface
/// (a one-pixel window mid-restore) from producing a zoom that divides the
/// logical size into nonsense.
const MIN_ZOOM: f32 = 0.1;
const MAX_ZOOM: f32 = 10.0;

/// The next rung of [`ZOOM_LADDER`] above or below `zoom`, clamped at the ends.
///
/// A value off the ladder — an application's own `ui_zoom`, or a fit offset —
/// steps to the next rung strictly past it rather than snapping first, so one
/// press always moves and never moves twice.
pub fn ladder_step(zoom: f32, up: bool) -> f32 {
    // Wide enough to absorb the f32 error in a ladder value that has been
    // through a multiply and back, narrow enough that two rungs never merge.
    const EPS: f32 = 1e-3;
    let ends = (ZOOM_LADDER[0], ZOOM_LADDER[ZOOM_LADDER.len() - 1]);
    if up {
        ZOOM_LADDER
            .iter()
            .copied()
            .find(|rung| *rung > zoom + EPS)
            .unwrap_or(ends.1)
    } else {
        ZOOM_LADDER
            .iter()
            .rev()
            .copied()
            .find(|rung| *rung < zoom - EPS)
            .unwrap_or(ends.0)
    }
}

/// The zoom at which `design` fits inside `available`, both in logical pixels:
/// the smaller of the two ratios, so the design fits whole with slack on the
/// axis that was not binding.
///
/// Public because a consumer that wants to know the number the host is about
/// to use — to place its own content against it, or to decide whether to ask
/// for a different design — must be able to compute the same one rather than
/// a near miss.
pub fn fit_zoom(design: (f32, f32), available: (f32, f32)) -> f32 {
    let ratio = |available: f32, design: f32| {
        if design > 0.0 {
            available / design
        } else {
            1.0
        }
    };
    ratio(available.0, design.0)
        .min(ratio(available.1, design.1))
        .clamp(MIN_ZOOM, MAX_ZOOM)
}

#[cfg(test)]
#[path = "host/zoom_tests.rs"]
mod zoom_tests;

#[cfg(test)]
#[path = "host/window_frame_tests.rs"]
mod window_frame_tests;

/// The focused text field, as the application maps it: which DOM node carries
/// the caret, and how to reach the [`TextInput`] inside the state. Returning
/// `Some` enables IME, the caret/selection overlay, visual caret movement,
/// and drag selection for that node.
pub struct FocusedTextSlot<State> {
    /// The focused `<input>`-carrying node.
    pub node: NodeId,
    /// Borrow the field's [`TextInput`] from the state.
    pub get: Box<dyn Fn(&State) -> &TextInput>,
    /// Borrow it mutably.
    pub get_mut: Box<dyn Fn(&mut State) -> &mut TextInput>,
}

/// Logical pixels a wheel line is worth under the desktop host. It restates
/// `genet_winit_host::WHEEL_LINE_PX`, which this crate does not depend on; a
/// test in `cambium-genet-winit-host` holds the two equal.
pub const HOST_WHEEL_LINE_PX: f32 = 48.0;

/// What the application sees inside a hook. One shape for every hook so the
/// application-side plumbing stays boring.
pub struct AppCtx<'a, State: 'static, Logic, V, T = Runner<State, Logic, V>>
where
    Logic: FnMut(&State) -> V,
    V: RootView<State>,
{
    /// The runner: state access, updates, and dispatch. A single-window
    /// host's [`Runner`]; under a multi-window host, this window's projection.
    pub runner: &'a mut T,
    /// The host-owned retained layout for read-only geometry queries.
    ///
    /// Kept private to the crate so an application cannot couple itself to
    /// the layout engine. Geometry and declared appearance are exposed through
    /// read-only methods keyed by DOM node identity.
    pub(crate) layout: Option<&'a OwnedLayout>,
    /// The native window (chrome requests, redraws, cursor, IME area), when
    /// there is one. `None` under `cambium_genet_winit_host::Harness`, the
    /// windowless test host — an application that asks for window chrome must
    /// tolerate its absence rather than assume a window exists.
    /// The frame, behind the neutral seam rather than as a winit handle.
    ///
    /// Applications ask it for redraws, size and scale, all of which a browser
    /// answers. Naming `&Window` here would have made the hook signature itself
    /// undeliverable on a second event source.
    pub window: Option<&'a dyn HostWindow>,
    /// The logical (DPI-independent) size of the surface being laid out — the
    /// coordinate space the layout, the cursor, and [`HostPointer`] all use.
    ///
    /// Under a UI zoom this is the *post-zoom* size, `window / zoom`: zoom
    /// changes how much of the interface fits, so an application that lays out
    /// against this number needs no zoom arithmetic of its own.
    pub logical_size: (f32, f32),
    /// The effective interface zoom this context was built at — the device
    /// scale is *not* in it. An application persists this, or lays a
    /// zoom-independent overlay out against it.
    pub ui_zoom: f32,
    /// Whether [`ui_zoom`](Self::ui_zoom) has moved since the last hook context
    /// the application was given.
    ///
    /// True in exactly one hook per change, the first one built after it — the
    /// edge, not the level, so an application that persists the preference
    /// writes once rather than every frame.
    pub zoom_changed: bool,
    /// Set to change the interface zoom; the host relayouts under it, exactly
    /// as [`set_sheet`](Self::set_sheet) relayouts under a new stylesheet.
    ///
    /// With [`HostOptions::fit_design`] set this is the user offset on the fit
    /// factor rather than the whole zoom, matching what the keyboard steps.
    pub set_ui_zoom: &'a mut Option<f32>,
    /// The custom-paint leaf registry the paint pass renders from.
    pub leaves: &'a mut sprigging::LeafRegistry<u64>,
    /// The file chooser that answers views' file requests. A hook may put its
    /// own in, as a scenario lane does to answer from its script.
    pub files: &'a mut Option<Box<dyn crate::FileChooser>>,
    /// Same-device viewport producers, keyed to existing custom-leaf slots.
    pub producers: &'a mut crate::ProducerRegistry,
    /// Set to swap the stylesheet; the host relayouts under the new sheet.
    pub set_sheet: &'a mut Option<String>,
    /// Set to end the application after this event.
    pub close: &'a mut bool,
    /// The cross-thread wake handle for application-owned workers. It has the
    /// same callback shape Armillary consumes, without making this host depend
    /// on Armillary or any task runtime.
    pub wake: &'a HostWake,
    /// Arm a capture of the next presented frame.
    pub capture: &'a mut Option<CaptureFn>,
    /// Optional one-shot state seal for the next successful presentation.
    pub presentation_observer: &'a mut Option<PresentationObserver<State, Logic, V, T>>,
    /// Optional branded readback of the same presentation as the state seal.
    pub capture_stamped: &'a mut Option<StampedCaptureFn>,
    /// Arm a one-shot paint-envelope capture. When armed with `capture`, both
    /// callbacks receive the same presented frame. Failed presentation keeps
    /// the request pending; no envelope is cloned while this is unarmed.
    pub capture_paint: &'a mut Option<PaintCaptureFn>,
    /// Pointer events for the host to deliver to itself once this hook
    /// returns, in order. See [`HostPointer`].
    pub pointer: &'a mut Vec<HostPointer>,
    pub(crate) a11y: &'a mut Vec<crate::QueuedA11y>,
    pub(crate) a11y_owner: &'a Rc<std::cell::Cell<bool>>,
    pub(crate) a11y_publication: crate::A11yPublicationSnapshot,
    /// Scroll requests for the next layout; queued through
    /// [`scroll_into_view`](Self::scroll_into_view).
    pub(crate) scroll: &'a mut Vec<ScrollRequest>,
    /// The window-verb seam, for a hook that wants to minimize/maximize/close
    /// without routing through application state. The same handle `init`
    /// received.
    pub window_commands: &'a WindowCommands,
    /// The render core every surface of this host draws through: one wgpu
    /// device and renderer, booted once and kept across suspend and resume.
    /// Hand it to a same-device tenant (a compute or import pass) so what it
    /// renders lands on the device the host presents with. `None` under
    /// `cambium_genet_winit_host::Harness` and before the first surface exists.
    pub render_core: Option<&'a Arc<RenderCore>>,
    /// Where the window is now, for persisting across launches. `None` under
    /// `cambium_genet_winit_host::Harness`, which has no window.
    pub geometry: Option<WindowGeometry>,
    /// CPU-side attribution for the frame that just completed. Present in
    /// `after_frame`; other hooks see the last completed frame, if any.
    pub frame_profile: Option<FrameProfile>,
    /// Successful presentation from the most recent redraw attempt. A failed
    /// acquisition clears this value; it never repeats an older presentation
    /// as if the attempted frame reached the queue. Windowless layout has none.
    pub presentation: Option<PresentedFrame>,
}

/// What `init` hands back once the window exists.
pub struct Init<State, Logic> {
    /// The application state the runner owns.
    pub state: State,
    /// The view logic (`fn(&State) -> V` or a closure).
    pub logic: Logic,
    /// The stylesheet the layout runs under.
    pub sheet: String,
    /// Font faces the application bundles, registered into every text system
    /// the host builds. Empty means system faces only.
    pub fonts: Vec<HostFont>,
    /// Images the application bundles, resolved against the `url()` the sheet
    /// or the DOM names. Empty means no host-supplied images.
    pub images: Vec<HostImage>,
}

/// One host-supplied font face.
///
/// `family` pins the face to a CSS family name, the way `@font-face` does, so
/// a sheet can name `'IBM Plex Sans'` without the face declaring it. `None`
/// registers the face under whatever family its own name table carries.
/// `sans-serif`, `serif`, `monospace`, and `system-ui` explicitly bind the
/// corresponding CSS generic to this face in addition to retaining the quoted
/// named-family alias. No global/system fallback is installed.
#[derive(Clone, Debug)]
pub struct HostFont {
    pub family: Option<String>,
    pub bytes: Vec<u8>,
}

/// One host-supplied image, keyed by the URL a sheet or the DOM names. The
/// bytes are the encoded file (PNG, JPEG, …); Livery decodes them itself.
#[derive(Clone, Debug)]
pub struct HostImage {
    pub url: String,
    pub bytes: Vec<u8>,
}

type InitFn<State, Logic> =
    Box<dyn FnOnce(&dyn HostWindow, &WindowCommands, &HostWake) -> Init<State, Logic>>;

/// A logical window dimension from the environment.
pub fn env_size(key: &str) -> Option<f64> {
    std::env::var(key)
        .ok()?
        .parse::<f64>()
        .ok()
        .filter(|v| *v > 0.0)
}

/// What every window of an application shares: its resources, its
/// custom-paint leaves and texture producers, and the render core they draw
/// through (stack seams S29). A single-window host keeps it in its one
/// [`HostState`]; a multi-window host lends it to whichever window is taking
/// its turn, so each window paints the leaves its own layout holds.
pub struct AppShared {
    /// The render core every surface is made from, held here so it outlives
    /// the surfaces: a suspend drops a surface and keeps the device, and the
    /// resume makes a new surface from this same core (stack seams S2). The
    /// event source sets it when it boots the core.
    pub render_core: Option<Arc<RenderCore>>,
    /// The single replacement owner of the core shared by this application.
    pub render_recovery: Option<Arc<genet_render_host::SharedRenderCore>>,
    /// The application's stylesheet.
    pub sheet: String,
    /// Host-supplied faces. Held here, not in the layout, so a rebuilt text
    /// system is registered again from the same list.
    pub fonts: Vec<HostFont>,
    /// Host-supplied image bytes by URL, handed to layout and paint each frame.
    pub images: std::collections::HashMap<String, Vec<u8>>,
    pub leaves: sprigging::LeafRegistry<u64>,
    pub producers: crate::ProducerRegistry,
    pub rendered: sprigging::RenderedLeaves,
    /// Netrender roadmap E4 — leaf key → (retained `FragmentId`, epoch it was
    /// translated at). Synced against `rendered` each redraw while a surface
    /// exists. The fragments live in the render core's renderer, which outlives
    /// any surface, so a suspend keeps them. The emitter does not place them
    /// yet: `emit_scene` passes no fragment lookup until a retained fragment
    /// has a clip and layer in the renderer, so every leaf still paints its
    /// commands inline.
    pub leaf_fragments: std::collections::HashMap<u64, (u64, u64)>,
    /// Counts sheet swaps, so a window that did not swap the sheet notices it
    /// changed and lays out afresh.
    pub(crate) sheet_generation: u64,
    /// The leaf keys other windows' layouts hold, set before each window's
    /// turn: a producer whose key another window lays out is that window's,
    /// not absent. Empty for a single window.
    pub(crate) held_elsewhere: std::collections::HashSet<u64>,
}

/// Resources detached from a disposed independent host. Drop outside any host
/// borrow: producer retirement, subscription wakes and capture callbacks may
/// enter application code. Only this independent host's physical producer keys
/// and retained fragment IDs are retired. Shared-forest disposal is not qualified.
#[must_use = "drop the disposal guard outside the host borrow"]
pub struct HostDisposal {
    #[cfg(not(target_arch = "wasm32"))]
    pending_native: Option<crate::presentation::native::PendingFrame>,
    #[cfg(not(target_arch = "wasm32"))]
    native_waiter: Option<genet_render_host::NativeFrameWaiter>,
    producers: Option<crate::ProducerRegistry>,
    fragments: std::collections::HashMap<u64, (u64, u64)>,
    driver: Option<Box<dyn crate::PresentationDriver>>,
    surface: Option<Box<dyn Surface>>,
    core: Option<Arc<RenderCore>>,
    subscription: Option<genet_render_host::RecoverySubscription>,
    owner: Option<Arc<genet_render_host::SharedRenderCore>>,
    capture: Option<CaptureFn>,
    stamped: Option<StampedCaptureFn>,
    paint: Option<PaintCaptureFn>,
    a11y_observer: Option<Box<dyn FnMut(crate::A11yPublicationSnapshot)>>,
    a11y_notice: Option<crate::A11yPublicationSnapshot>,
}

/// One window's host state, with the application's [`AppShared`] part in
/// `shared`.
pub struct HostState<State: 'static, Logic, V, T = Runner<State, Logic, V>>
where
    Logic: FnMut(&State) -> V,
    V: RootView<State>,
{
    /// The frame behind the neutral seam: redraw, size, scale, IME. A browser
    /// event source supplies a canvas here.
    pub window: Option<Box<dyn HostWindow>>,
    /// The last published titlebar area and the width it was computed for, so
    /// the sheet is rebuilt only when one of them actually moves.
    pub titlebar_published: Option<(crate::TitlebarInsets, f32)>,
    /// The `:root` rule carrying the published custom properties. Empty until
    /// the first publish, which is a valid stylesheet.
    pub titlebar_sheet: String,
    /// The presentation surface behind the neutral seam. A browser event
    /// source supplies the same pair against a canvas.
    pub surface: Option<Box<dyn Surface>>,
    /// A browser presenter with optional same-device services.
    pub presentation_driver: Option<Box<dyn crate::PresentationDriver>>,
    pub recovery_status: Option<genet_render_host::RecoveryStatus>,
    /// Current checked raster refusal; cleared at the next redraw admission.
    /// Most recent refused presentation, including portable backend failures.
    pub last_presentation_failure: Option<crate::PresentationFailure>,
    pub last_raster_failure: Option<genet_render_host::RasterizeFailure>,
    pub(crate) detached_generation: Option<u64>,
    pub needs_recovery_surface: bool,
    /// The tree this window drives.
    pub runner: Option<T>,
    /// Retained Livery/Buckram session in logical coordinates.
    pub layout: Option<OwnedLayout>,
    pub layout_size: (f32, f32),
    /// The user's own zoom: [`HostOptions::ui_zoom`] as the runtime setter and
    /// the keyboard ladder have since moved it. Multiplied by
    /// [`fit_factor`](Self::fit_factor) to give the effective zoom.
    pub(crate) user_zoom: f32,
    /// The zoom `fit_design` currently asks for, recomputed on every resize.
    /// `1.0` when no design was named, so the multiply is unconditional.
    pub(crate) fit_factor: f32,
    /// The effective zoom the application was last told about, so a change is
    /// reported to a hook once rather than every frame.
    pub(crate) zoom_seen: f32,
    /// The surface in device-logical pixels when there is no window: how a
    /// windowless host ([`Harness`](crate::Harness)) states what `fit_design`
    /// is measured against. `None` asks the window instead.
    pub(crate) surface_size: Option<(f32, f32)>,
    pub(crate) last_layout_update_us: u64,
    pub(crate) last_layout_tick_us: u64,
    pub(crate) last_layout_apply_us: u64,
    pub(crate) last_layout_rebuild_us: u64,
    pub(crate) last_style_resolve_us: u64,
    pub(crate) last_layout_with_text_us: u64,
    pub(crate) last_content_extent_us: u64,
    pub(crate) last_layout_mutations: u64,
    pub(crate) last_layout_rebuilt: bool,
    pub(crate) last_leaf_boxes_us: u64,
    pub(crate) last_leaf_render_us: u64,
    pub(crate) last_leaf_repaints: u64,
    // Retire all pending/output GPU ownership before the detach subscription.
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) pending_native: Option<crate::presentation::native::PendingFrame>,
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) native_waiter: Option<genet_render_host::NativeFrameWaiter>,
    pub(crate) raster_cache: crate::presentation::wgpu::RasterCache,
    /// What this window shares with the application's other windows.
    pub shared: AppShared,
    /// Declared after shared GPU resources so a closing host releases them
    /// before its subscription opens the remaining hosts' detach barrier.
    pub(crate) recovery_subscription: Option<genet_render_host::RecoverySubscription>,
    /// The shared sheet's generation this window's layout was built against.
    pub(crate) layout_sheet_generation: u64,
    /// The custom-leaf keys this window's last layout held.
    pub(crate) leaf_keys: Vec<u64>,
    /// Cursor position in logical coordinates.
    pub cursor: (f32, f32),
    /// Current native pointer identity (mouse defaults to zero).
    pub pointer_id: i32,
    /// Live modifier state, in the neutral vocabulary. Winit's own state is
    /// converted once, where it arrives, so nothing downstream reads it.
    pub modifiers: Modifiers,
    /// The node whose text field anchors an active drag selection.
    pub text_drag: Option<NodeId>,
    /// Opaque ids for `:hover` / `:focus` restyles on target change.
    pub last_hover: Option<u64>,
    pub last_focus: Option<u64>,
    /// The resolved hover handler, for `on_hover` Enter/Leave/Move routing.
    pub last_hover_hit: Option<NodeId>,
    /// Monotonic base for the CSS-transition animation clock.
    pub anim_base: crate::Instant,
    /// The accessibility seam, behind the neutral trait rather than named as
    /// AccessKit. `HostState` holds no winit or AccessKit type through it, so
    /// this field is ready to move with the struct; a browser event source
    /// supplies a projection onto the live document instead.
    pub a11y: Option<Box<dyn Accessibility>>,
    /// Opt-in observer. Products choose retention, redaction and export policy.
    /// Sync callbacks run in the host call and must not reenter it. Disposal
    /// notification runs when its guard is dropped outside the host borrow.
    /// Never invoked by semantic reads.
    pub a11y_publication_observer: Option<Box<dyn FnMut(crate::A11yPublicationSnapshot)>>,
    pub(crate) a11y_publication: crate::A11yPublication,
    pub(crate) a11y_publication_sequence: u64,
    pub(crate) a11y_wake: Arc<AtomicBool>,
    /// The platform's file chooser, which answers a view's file request.
    pub files: Option<Box<dyn crate::FileChooser>>,
    /// File answers waiting for the next frame.
    pub(crate) file_answers: crate::files::FileAnswers,
    /// Whether presentation is suspended: a close policy hid the root window,
    /// or, on a future browser source, the tab went background. Portable fact;
    /// only *how* a host hides (set_visible, document.hidden) is platform.
    pub hidden: bool,
    /// The application's end of the window-verb seam. The queue is plain data
    /// (`Rc<RefCell<Vec<WindowCommand>>>`, no platform type in it), so it lives
    /// with the host and every event source shares it; *draining* it is the
    /// event source's job, since honouring a verb needs a real window.
    pub commands: WindowCommands,
    /// Where the window is and how big, refreshed by the event source. A
    /// snapshot rather than a live query so the host can hand it to hooks
    /// without asking a window it does not own.
    pub geometry: Option<WindowGeometry>,
    /// Set by [`HostWake`] until the event loop gives the application one drain
    /// turn. Kept in host state so the harness can exercise the same coalescing
    /// contract without a native event loop.
    pub wake_pending: Arc<AtomicBool>,
    pub scrollbar_fade: ScrollbarFade<ScrollTarget>,
    pub close_requested: bool,
    pub pending_sheet: Option<String>,
    /// A zoom an application hook asked for, applied once its borrows end —
    /// the same deferral `pending_sheet` gets, and for the same reason.
    pub pending_ui_zoom: Option<f32>,
    pub pending_capture: Option<CaptureFn>,
    pub capture_owner: crate::CaptureOwner,
    pub pending_presentation_observer: Option<PresentationObserver<State, Logic, V, T>>,
    pub pending_stamped_capture: Option<StampedCaptureFn>,
    pub presentation_host: u64,
    pub presentation_sequence: u64,
    pub(crate) native_receiving: bool,
    pub(crate) displayed_input: Option<Rc<crate::owned_layout::PresentedInput>>,
    pub(crate) displayed_presentation: Option<PresentedFrame>,
    pub(crate) native_completion_pending: Arc<AtomicBool>,
    pub(crate) native_dirty: bool,
    /// The current redraw's successful presentation, cleared at the start of
    /// each attempt. Separate from the monotonic sequence, which survives a
    /// failed acquisition and a surface suspension.
    pub last_redraw_presentation: Option<PresentedFrame>,
    pub pending_paint_capture: Option<PaintCaptureFn>,
    /// Pointer events an application hook asked the host to deliver to itself,
    /// drained through the real input path once the hook returns.
    pub pending_pointer: Vec<HostPointer>,
    pub(crate) pending_a11y: Vec<crate::QueuedA11y>,
    pub(crate) a11y_owner: Rc<std::cell::Cell<bool>>,
    draining_a11y: bool,
    /// Scroll requests an application hook queued, resolved in order by the
    /// next [`relayout`](Host::relayout) once its layout is current.
    pub pending_scroll: Vec<ScrollRequest>,
    /// The focused field and caret byte the last relayout scrolled into view.
    /// A caret that has not moved since is left where the user scrolled it.
    pub caret_followed: Option<(NodeId, usize)>,
    /// The last frame's host-owned phase attribution.
    pub last_frame_profile: Option<FrameProfile>,
    /// Tab is being held: the arrow keys steer focus instead of reaching the
    /// focused element. Set by Tab's first key-repeat, cleared on its release.
    pub tab_held: bool,
}

/// The host: options, hooks, and everything the donor's `App` owned that was
/// not application policy.
pub struct Host<State: 'static, Logic, V, T = Runner<State, Logic, V>>
where
    Logic: FnMut(&State) -> V,
    V: RootView<State>,
{
    pub options: HostOptions,
    pub init: Option<InitFn<State, Logic>>,
    pub hooks: HostHooks<State, Logic, V, T>,
    pub s: HostState<State, Logic, V, T>,
    pub wake: HostWake,
}

pub enum Hook {
    AfterDispatch,
    AfterFrame,
    AfterWake,
}

#[cfg(test)]
#[path = "host/disposal_controls.rs"]
mod disposal_controls;

#[path = "host/options.rs"]
mod options;

pub use options::{AppFrameInsets, HostOptions, WindowFrame};

#[path = "host/capture_types.rs"]
mod capture_types;

pub use capture_types::{
    CaptureFn, CaptureFrame, CaptureRefusal, PaintCaptureFn, PresentationObserver, PresentedFrame,
    StampedCaptureFn, StampedCaptureFrame,
};

#[path = "host/events.rs"]
mod events;

pub use events::{
    CloseDisposition, CloseRequest, FrameProfile, HostPointer, IdlePolicy, RelayoutProfile,
    ScrollIntoView, ScrollRequest, WheelUnit,
};

#[path = "host/context.rs"]
mod context;

#[path = "host/state_ops.rs"]
mod state_ops;

#[path = "host/startup.rs"]
mod startup;

#[path = "host/commands.rs"]
mod commands;

#[path = "host/hooks.rs"]
mod hooks;

#[path = "host/presentation_hooks.rs"]
mod presentation_hooks;

pub use hooks::{
    AppHook, CloseRequestHook, FocusedTextHook, FrameHook, HostHooks, KeyInterceptHook,
};
