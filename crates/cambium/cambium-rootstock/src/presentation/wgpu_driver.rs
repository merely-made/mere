// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! Owned surface/output adapter reusing the existing same-device presenter.
use super::*;
use crate::{CaptureFrame, CaptureRefusal, FrameProfile, PresentedFrame, Surface};
use genet_render_host::{RecoveryStatus, RenderCore, SharedRenderCore};
use std::sync::Arc;
pub type WgpuSurfaceFactory = Box<dyn Fn(Arc<RenderCore>) -> Result<Box<dyn Surface>, String>>;
/// Expands a complete portable capture into one physical-pixel scene at the
/// given extent and scale, composing producer scenes in painter order.
pub type PortableMaterializer = Box<
    dyn Fn(
        &crate::PortablePaintCapture,
        u32,
        u32,
        f32,
    ) -> Result<netrender::Scene, (Option<u64>, String)>,
>;
pub struct WgpuDriver {
    surface: Option<Box<dyn Surface>>,
    portable: Option<PortableMaterializer>,
    owner: Arc<SharedRenderCore>,
    factory: WgpuSurfaceFactory,
    lifetime: PresentationLifetime,
    output: Option<(PresentedFrame, super::wgpu::WgpuOutput)>,
    profile: Option<FrameProfile>,
    captures: crate::CaptureOwner,
}
impl WgpuDriver {
    pub fn new(
        surface: Box<dyn Surface>,
        owner: Arc<SharedRenderCore>,
        factory: WgpuSurfaceFactory,
    ) -> Self {
        let generation = ContextGeneration(surface.core().generation());
        Self {
            surface: Some(surface),
            portable: None,
            owner,
            factory,
            lifetime: PresentationLifetime::new(generation),
            output: None,
            profile: None,
            captures: crate::CaptureOwner::default(),
        }
    }
    /// Present the same portable scene a direct WebGL2 route draws: producer
    /// scenes are composed in the page pass, never rasterized to an
    /// intermediate texture and resampled. Same-device texture producers are
    /// not called on this route.
    pub fn with_portable_scene(mut self, materialize: PortableMaterializer) -> Self {
        self.portable = Some(materialize);
        self
    }
    fn failure(&self, message: impl ToString) -> PresentationFailure {
        PresentationFailure {
            backend: self.backend(),
            generation: self.generation(),
            producer_key: None,
            message: message.to_string(),
            native: None,
        }
    }
}
impl PresentationDriver for WgpuDriver {
    fn backend(&self) -> PresentationBackend {
        PresentationBackend::Wgpu
    }
    fn generation(&self) -> ContextGeneration {
        self.lifetime.generation()
    }
    fn state(&self) -> PresentationState {
        self.lifetime.state()
    }
    fn content_mode(&self) -> FrameContentMode {
        if self.portable.is_some() {
            FrameContentMode::PortableScene
        } else {
            FrameContentMode::NativeTexture
        }
    }
    fn wgpu_surface(&self) -> Option<&dyn Surface> {
        self.surface.as_deref()
    }
    fn resize(&mut self, width: u32, height: u32) -> Result<(), PresentationFailure> {
        if width > 0
            && height > 0
            && let Some(surface) = self.surface.as_mut()
        {
            surface.resize(width, height);
        }
        Ok(())
    }
    fn present(
        &mut self,
        frame: &PreparedFrame,
    ) -> Result<Option<PresentedFrame>, PresentationFailure> {
        self.output = None;
        if !self.lifetime.accepts(frame.generation) {
            return Ok(None);
        }
        let Some(surface) = self.surface.as_deref() else {
            return Ok(None);
        };
        let mut profile = frame.profile;
        let presenter = super::wgpu::WgpuPresentation::new(surface);
        let result = match (&frame.paint, &self.portable) {
            (PreparedPaint::Portable(capture), Some(materialize)) => {
                let phase = crate::Instant::now();
                let scene = materialize(capture, frame.width, frame.height, frame.scale).map_err(
                    |(producer_key, message)| {
                        let mut failure = self.failure(message);
                        failure.producer_key = producer_key;
                        failure
                    },
                )?;
                profile.emit_scene_us = profile
                    .emit_scene_us
                    .saturating_add(phase.elapsed().as_micros().min(u64::MAX as u128) as u64);
                presenter.present_scene(frame, &scene, &[], 1.0, &mut profile, None)
            },
            (PreparedPaint::Portable(_), None) => {
                return Err(self.failure("portable paint without a portable WebGPU route"));
            },
            (PreparedPaint::Native(_), _) => presenter.present(frame, &mut profile, None),
        };
        self.profile = Some(profile);
        let Some(output) = result? else {
            return Ok(None);
        };
        let stamp = frame.presentation_stamp();
        let _ = self
            .captures
            .resume(self.backend(), stamp.context_generation);
        self.output = Some((stamp, output));
        Ok(Some(stamp))
    }
    fn take_profile(&mut self) -> Option<FrameProfile> {
        self.profile.take()
    }
    fn lose(&mut self) {
        self.captures
            .scope()
            .invalidate(CaptureRefusal::ContextLost {
                backend: self.backend(),
                generation: self.generation(),
            });
        self.output = None;
        self.surface = None;
        self.lifetime.lose();
    }
    fn restore(&mut self) -> Result<bool, PresentationFailure> {
        match self.state() {
            PresentationState::Ready => return Ok(true),
            PresentationState::Disposed => return Ok(false),
            _ => {},
        }
        if !matches!(self.owner.status(), RecoveryStatus::Ready { .. }) {
            return Ok(false);
        }
        let core = self.owner.current().map_err(|status| {
            let mut failure = self.failure(format!("shared core unavailable: {status:?}"));
            failure.native = status
                .loss()
                .cloned()
                .map(NativePresentationFailure::DeviceLost);
            failure
        })?;
        let next = (self.factory)(core).map_err(|e| self.failure(e))?;
        if self.state() == PresentationState::Lost {
            self.lifetime.begin_restore().map_err(|e| self.failure(e))?;
        }
        self.lifetime
            .finish_restore()
            .map_err(|e| self.failure(e))?;
        self.surface = Some(next);
        Ok(true)
    }
    fn begin_capture(&mut self, stamp: PresentedFrame) -> Result<CaptureFrame, CaptureRefusal> {
        let refuse = || CaptureRefusal::ContextLost {
            backend: PresentationBackend::Wgpu,
            generation: stamp.context_generation,
        };
        if !self.lifetime.accepts(stamp.context_generation) {
            return Err(refuse());
        }
        let Some((current, output)) = &self.output else {
            return Err(refuse());
        };
        if *current != stamp {
            return Err(refuse());
        }
        crate::start_presentation_readback(
            self.surface.as_deref().ok_or_else(refuse)?,
            &output.view,
            stamp,
        )
        .and_then(|capture| self.captures.wrap(self.backend(), stamp, capture))
    }
    fn dispose(&mut self) {
        self.captures.scope().dispose();
        self.output = None;
        self.surface = None;
        self.lifetime.dispose();
    }
}
