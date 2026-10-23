// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Same-device producers placed by the existing custom-leaf paint slot.
//!
//! The application owns scene state, rendering and picking. Rootstock owns
//! the image's document lifetime and stages changed output before rasterizing
//! the ordinary document scene. Producer keys share Sprigging's namespace;
//! local keys belong to their registry. Rootstock assigns independent physical
//! renderer identities so separate registries may use the same local keys.

use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

pub use netrender::SourceAlpha;

mod bridge;
mod child_lifetime;
pub use child_lifetime::{ProducerChildLifetime, ProducerChildToken};

/// Producer dispatch, not durable or asynchronous operation completion.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProducerActionDispatch {
    Dispatched,
    Declined,
    StaleTarget,
}
pub(crate) mod registry;
pub use paint_list_render::bundle::{PortablePaintCapture, ProducerSource};
pub use registry::PortableCaptureError;
pub use registry::{ProducerGpuRemapError, ProducerImageIdentity, ProducerRegistry};

/// Application-owned portable scene; its viewport is the same rounded logical
/// extent used by the producer's GPU callback. Identity changes on source rebind.
#[derive(Clone, Debug)]
pub struct ProducedScene {
    pub scene: netrender::Scene,
    pub source_identity: u64,
    pub source_revision: u64,
    pub alpha: SourceAlpha,
    pub encoding: SourceEncoding,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PortableProducerError {
    Unsupported,
    InvalidSource,
}

/// Colour values observed at the producer view's sampling boundary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceEncoding {
    /// Encoded sRGB RGB, sampled through an unorm view without hardware decode.
    /// Alpha is linear coverage. This is the existing external-image path.
    Srgb,
    /// Linear sRGB RGB. Explicitly rejected by this bridge: the current
    /// Netrender staging pass converts alpha but performs no colour transfer.
    LinearSrgb,
}

/// Only the properties the application declared at registration are read.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ResolvedAppearance {
    pub(crate) color: Option<[f32; 4]>,
    pub(crate) values: BTreeMap<String, String>,
}

impl ResolvedAppearance {
    /// Used CSS `color`: encoded sRGB RGB and straight alpha, independent of
    /// document/group opacity. None when `color` was not declared.
    pub fn color(&self) -> Option<[f32; 4]> {
        self.color
    }

    /// Resolved CSS serialization, including declared custom properties.
    pub fn css(&self, property: &str) -> Option<&str> {
        self.values.get(property).map(String::as_str)
    }
}

/// Presentation advances application time; export observes its current scene.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProducerFramePurpose {
    Presentation,
    Export,
}

/// The laid-out content extent for this producer invocation.
#[derive(Clone, Debug, PartialEq)]
pub struct ProducerFrameInfo {
    pub purpose: ProducerFramePurpose,
    pub logical_size: (f32, f32),
    pub physical_size: [u32; 2],
    /// Physical pixels per CSS pixel, including host UI zoom.
    pub layout_scale: f32,
    /// No prior image is usable: return a frame even if application data is
    /// unchanged. Set after resize, suspension, removal/recreation or a new device.
    pub needs_frame: bool,
    /// Host monotonic time for this draw. None preserves deterministic callers.
    pub timestamp: Option<std::time::Duration>,
    pub appearance: ResolvedAppearance,
}

/// Borrowed host GPU access. Producers must create and submit work on these
/// handles, and finish submission before returning a changed texture.
pub struct ProducerContext<'a> {
    pub device: &'a wgpu::Device,
    pub queue: &'a wgpu::Queue,
    pub frame: &'a ProducerFrameInfo,
    /// The host's render core, on the same device. A producer whose content
    /// is a netrender scene rasterizes it here under a key of its own
    /// (`RenderCore::rasterize_scaled_for`), sharing the host's renderer and
    /// glyph cache instead of booting another. `core.generation()` is the
    /// immutable device lifetime identity; use it rather than pointer identity
    /// for any producer-owned GPU cache. CPU source identity remains separate.
    pub core: &'a genet_render_host::RenderCore,
}

/// A full-size, single-layer, non-multisampled base-mip view. Its texture must
/// have TEXTURE_BINDING usage and exactly `frame.physical_size` dimensions.
#[derive(Clone, Debug)]
pub struct ProducedTexture {
    pub view: wgpu::TextureView,
    /// Advance when pixels in the same view change. A different view identity
    /// is independently detected, so replacing it cannot retain stale pixels.
    pub generation: u64,
    pub alpha: SourceAlpha,
    pub encoding: SourceEncoding,
}

pub trait TextureProducer {
    /// Called after layout, before document paint. None keeps the last usable
    /// image; it leaves an empty slot when `needs_frame` is true. Cheaply skip
    /// unchanged world/appearance inputs here. Depth remains producer-owned.
    fn render(&mut self, cx: &ProducerContext<'_>) -> Option<ProducedTexture>;

    /// A changed frame, reuse, or explicit rendering refusal. Existing GPU
    /// producers retain their contract; scene adapters override this method.
    fn render_checked(
        &mut self,
        cx: &ProducerContext<'_>,
    ) -> Result<Option<ProducedTexture>, genet_render_host::RasterizeFailure> {
        Ok(self.render(cx))
    }

    /// Export the currently bound application scene without acquiring a device.
    /// GPU-only producers explicitly refuse; a capture never substitutes pixels
    /// or removes their authored external-texture command.
    fn portable_scene(
        &mut self,
        _frame: &ProducerFrameInfo,
    ) -> Result<ProducedScene, PortableProducerError> {
        Err(PortableProducerError::Unsupported)
    }

    /// Once on entering a non-painted or surface-suspended state. Releasing
    /// transient targets is optional; retained geometry can stay cached.
    fn suspend(&mut self) {}

    /// A DOM owner was removed/replaced or the registration was removed.
    /// Release image and renderer resources; a retained application handle
    /// may be registered again and lazily initialize on the next render.
    fn retire(&mut self) {
        self.suspend();
    }

    /// What the slot means to an assistive technology: a role and name for
    /// the slot itself, and the things drawn in it as child nodes with their
    /// rectangles. Read for host semantic publication and current-target action
    /// admission; separate reads may occur between frames. Describe current
    /// state without rendering, allocating identities, rebuilding action routes
    /// or advancing application state. `None`, the default,
    /// leaves the slot's DOM/Sprigging semantics alone. Reading and activation
    /// are separate calls, without a snapshot or transaction guarantee.
    fn semantics(&self) -> Option<ProducerSemantics> {
        None
    }

    /// Observe an authored child life when numeric keys can recur. `None`
    /// retains the nonreused-key contract of `ProducerNode::key`; it does not
    /// prove arbitrary key reuse safe. No identity allocation on this read.
    fn child_lifetime(&self, _key: u64) -> Option<ProducerChildToken> {
        None
    }

    /// A reader invoked action `id` of the drawn node `key` (a
    /// [`ProducerNode`]'s own [`ProducerAction`]). Returns whether the producer
    /// dispatched it; `false`, the default, for one it does not know. This is
    /// current-context ingress; the host uses `act_checked` for admitted requests.
    fn act(&mut self, key: u64, id: &str) -> bool {
        let _ = (key, id);
        false
    }

    /// Validate the admitted child's life at activation. Producers whose model
    /// is behind another shared handle must override this and hold that model's
    /// mutable borrow across validation and dispatch. This default covers a
    /// directly owned model; it cannot make arbitrary interior state atomic.
    fn act_checked(
        &mut self,
        key: u64,
        id: &str,
        expected: Option<&ProducerChildToken>,
    ) -> ProducerActionDispatch {
        if !ProducerChildToken::matches_current(expected, self.child_lifetime(key).as_ref()) {
            return ProducerActionDispatch::StaleTarget;
        }
        if self.act(key, id) {
            ProducerActionDispatch::Dispatched
        } else {
            ProducerActionDispatch::Declined
        }
    }
}

/// A producer's accessible description of its slot. See
/// [`TextureProducer::semantics`].
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ProducerSemantics {
    /// The slot's role, or `None` to keep its DOM role.
    pub role: Option<ProducerRole>,
    /// The slot's name, or `None` to keep its DOM name.
    pub name: Option<String>,
    /// What is drawn in the slot, in reading order.
    pub children: Vec<ProducerNode>,
}

/// One thing drawn in a producer's slot.
#[derive(Clone, Debug, PartialEq)]
pub struct ProducerNode {
    /// The producer's stable name for the node, so a reader's request
    /// reaches it however the list is ordered when the reader acts.
    /// Keys must be unique within one `ProducerSemantics` description and must
    /// not be reused for a different child life within a live registration.
    /// Assign them at authored model transitions, including removal/restoration
    /// between semantic reads. Reading a description must not allocate keys or
    /// reconcile their lifetimes. Ordinary value, order and geometry updates
    /// keep the key. The host cannot infer an unobserved removal from snapshots.
    pub key: u64,
    pub role: ProducerRole,
    pub name: String,
    /// Where it is drawn, `[x, y, width, height]` in the slot's own logical
    /// (layout) pixels, from the content-box top-left corner, matching
    /// `ProducerFrameInfo::logical_size` and content-local pointer coordinates.
    /// The host currently publishes untransformed, scroll-adjusted bounds;
    /// forward CSS transform/clip and flattened-ancestor placement remain
    /// unqualified.
    pub rect: [f32; 4],
    /// What a reader may do to it, each lowered to a button of its own
    /// ([`TextureProducer::act`] carries one out).
    pub actions: Vec<ProducerAction>,
}

/// One action a drawn node offers a reader. Neutral: a host maps its own
/// action model onto it, and the id comes back through
/// [`TextureProducer::act`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProducerAction {
    /// Unique within this child's actions. Other child keys may use the same
    /// action ID. Ambiguous contributions are refused by shared host composition.
    pub id: String,
    /// The button's name.
    pub label: String,
    /// What the action does, a reader's description of the button.
    pub description: String,
}

/// A drawn node's action, as a reader's request names it: the producer's
/// slot (its registry key), the node's [`ProducerNode::key`] and the action's
/// [`ProducerAction::id`].
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ProducedAction {
    pub slot: u64,
    pub key: u64,
    pub id: String,
}

/// The roles a producer's slot and its nodes can take. A small, neutral set
/// that each host lowers to its own vocabulary (ARIA, AccessKit).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProducerRole {
    /// A collection whose items are its children.
    List,
    ListItem,
    /// A generic grouping.
    Group,
    /// A picture with a name.
    Image,
    /// A drawn object inside a graphic (a node on a canvas).
    GraphicsObject,
}

impl ProducerRole {
    /// The WAI-ARIA role this lowers to.
    pub fn aria(self) -> &'static str {
        match self {
            ProducerRole::List => "list",
            ProducerRole::ListItem => "listitem",
            ProducerRole::Group => "group",
            ProducerRole::Image => "img",
            ProducerRole::GraphicsObject => "graphics-object",
        }
    }
}

impl<P: TextureProducer> TextureProducer for Rc<RefCell<P>> {
    fn render(&mut self, cx: &ProducerContext<'_>) -> Option<ProducedTexture> {
        self.borrow_mut().render(cx)
    }
    fn render_checked(
        &mut self,
        cx: &ProducerContext<'_>,
    ) -> Result<Option<ProducedTexture>, genet_render_host::RasterizeFailure> {
        self.borrow_mut().render_checked(cx)
    }
    fn portable_scene(
        &mut self,
        frame: &ProducerFrameInfo,
    ) -> Result<ProducedScene, PortableProducerError> {
        self.borrow_mut().portable_scene(frame)
    }
    fn suspend(&mut self) {
        self.borrow_mut().suspend();
    }
    fn retire(&mut self) {
        self.borrow_mut().retire();
    }
    fn semantics(&self) -> Option<ProducerSemantics> {
        self.borrow().semantics()
    }
    fn child_lifetime(&self, key: u64) -> Option<ProducerChildToken> {
        self.borrow().child_lifetime(key)
    }
    fn act(&mut self, key: u64, id: &str) -> bool {
        self.borrow_mut().act(key, id)
    }
    fn act_checked(
        &mut self,
        key: u64,
        id: &str,
        expected: Option<&ProducerChildToken>,
    ) -> ProducerActionDispatch {
        self.borrow_mut().act_checked(key, id, expected)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProducerRegistrationError {
    KeyOutOfRange,
    AlreadyRegistered,
    /// The nonreused physical producer identity range has been exhausted.
    IdentityExhausted,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProducerError {
    InvalidExtent,
    InvalidTexture,
    UnsupportedEncoding,
    DuplicateDomKey,
    RenderRefused,
}

/// The source whose checked render refused the current host frame.
#[derive(Clone, Debug)]
pub struct ProducerRenderFailure {
    pub source_key: u64,
    pub failure: genet_render_host::RasterizeFailure,
}

/// Per-frame CPU attribution, separate from document raster and present wait.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ProducerFrameStats {
    pub render_us: u64,
    pub stage_us: u64,
    pub render_calls: u64,
    pub stages: u64,
    pub suspensions: u64,
    pub retirements: u64,
    pub invalid_frames: u64,
}

#[cfg(test)]
mod tests;
