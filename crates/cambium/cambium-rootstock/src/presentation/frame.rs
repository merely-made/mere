// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! CPU frame packets and the host-owned presentation boundary.
use super::{ContextGeneration, PresentationBackend};

pub enum PreparedPaint {
    Native(paint_list_api::PaintEnvelope),
    Portable(crate::PortablePaintCapture),
}

pub struct PreparedFrame {
    pub paint: PreparedPaint,
    pub host: u64,
    /// Proposed sequence; the host commits it only after presentation succeeds.
    pub sequence: u64,
    pub width: u32,
    pub height: u32,
    pub scale: f32,
    pub generation: ContextGeneration,
    pub clear: [f32; 4],
    pub(crate) profile: crate::FrameProfile,
    pub(crate) started: crate::Instant,
    pub(crate) animating: bool,
}
impl PreparedFrame {
    pub fn presentation_stamp(&self) -> crate::PresentedFrame {
        crate::PresentedFrame {
            host: self.host,
            sequence: self.sequence,
            width: self.width,
            height: self.height,
            layout_scale: self.scale,
            context_generation: self.generation,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrameContentMode {
    NativeTexture,
    PortableScene,
}

/// Preserve the installed native provider's attributed refusal or acquisition cause.
#[derive(Clone, Debug)]
pub enum NativePresentationFailure {
    Rasterize(genet_render_host::RasterizeFailure),
    DeviceLost(genet_render_host::DeviceLossNotice),
    SurfaceLost,
}

#[derive(Clone, Debug)]
pub struct PresentationFailure {
    pub backend: PresentationBackend,
    pub generation: ContextGeneration,
    pub producer_key: Option<u64>,
    pub message: String,
    pub native: Option<NativePresentationFailure>,
}
impl std::fmt::Display for PresentationFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?} context {} source {:?}: {}",
            self.backend, self.generation.0, self.producer_key, self.message
        )
    }
}
impl std::error::Error for PresentationFailure {}

pub trait PendingPresentationCapture {
    fn poll(
        &mut self,
    ) -> Option<Result<(crate::PresentedFrame, crate::Frame), crate::CaptureRefusal>>;
}

pub trait PresentationDriver {
    fn take_profile(&mut self) -> Option<crate::FrameProfile> {
        None
    }
    fn backend(&self) -> PresentationBackend;
    fn generation(&self) -> ContextGeneration;
    fn state(&self) -> super::PresentationState {
        super::PresentationState::Ready
    }
    fn content_mode(&self) -> FrameContentMode;
    fn wgpu_surface(&self) -> Option<&dyn crate::Surface>;
    fn resize(&mut self, width: u32, height: u32) -> Result<(), PresentationFailure>;
    fn present(
        &mut self,
        frame: &PreparedFrame,
    ) -> Result<Option<crate::PresentedFrame>, PresentationFailure>;
    fn lose(&mut self);
    fn restore(&mut self) -> Result<bool, PresentationFailure>;
    fn begin_capture(
        &mut self,
        stamp: crate::PresentedFrame,
    ) -> Result<Box<dyn PendingPresentationCapture>, crate::CaptureRefusal>;
    fn dispose(&mut self);
}
