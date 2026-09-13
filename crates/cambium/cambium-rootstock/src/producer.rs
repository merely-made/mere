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
//! register a key in only one registry.

use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

pub use netrender::SourceAlpha;

mod bridge;
mod registry;
pub use registry::ProducerRegistry;

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

/// The laid-out content extent for this producer invocation.
#[derive(Clone, Debug, PartialEq)]
pub struct ProducerFrameInfo {
    pub logical_size: (f32, f32),
    pub physical_size: [u32; 2],
    /// Physical pixels per CSS pixel, including host UI zoom.
    pub layout_scale: f32,
    /// No prior image is usable: return a frame even if application data is
    /// unchanged. Set after resize, suspension, removal/recreation or a new device.
    pub needs_frame: bool,
    pub appearance: ResolvedAppearance,
}

/// Borrowed host GPU access. Producers must create and submit work on these
/// handles, and finish submission before returning a changed texture.
pub struct ProducerContext<'a> {
    pub device: &'a wgpu::Device,
    pub queue: &'a wgpu::Queue,
    pub frame: &'a ProducerFrameInfo,
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

    /// Once on entering a non-painted or surface-suspended state. Releasing
    /// transient targets is optional; retained geometry can stay cached.
    fn suspend(&mut self) {}

    /// A DOM owner was removed/replaced or the registration was removed.
    /// Release image and renderer resources; a retained application handle
    /// may be registered again and lazily initialize on the next render.
    fn retire(&mut self) {
        self.suspend();
    }
}

impl<P: TextureProducer> TextureProducer for Rc<RefCell<P>> {
    fn render(&mut self, cx: &ProducerContext<'_>) -> Option<ProducedTexture> {
        self.borrow_mut().render(cx)
    }
    fn suspend(&mut self) {
        self.borrow_mut().suspend();
    }
    fn retire(&mut self) {
        self.borrow_mut().retire();
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProducerRegistrationError {
    KeyOutOfRange,
    AlreadyRegistered,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProducerError {
    InvalidExtent,
    InvalidTexture,
    UnsupportedEncoding,
    DuplicateDomKey,
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
