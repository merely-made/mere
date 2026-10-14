// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! A context owned by one browser appearance, independently of wgpu.
use cambium_rootstock::{
    CaptureRefusal, ContextGeneration, FrameContentMode, PendingPresentationCapture, PreparedFrame,
    PreparedPaint, PresentationBackend, PresentationDriver, PresentationFailure,
    PresentationLifetime, PresentationState, PresentedFrame, Surface,
};
use netrender::vello_backends::{SparseResourceLimits, WebGlSession, WebGlSessionInit};
use std::{cell::Cell, rc::Rc};
use wasm_bindgen::JsCast;
use web_sys::HtmlCanvasElement;

pub struct WebGlPresentation {
    canvas: HtmlCanvasElement,
    gl: web_sys::WebGl2RenderingContext,
    session: Option<WebGlSession>,
    rebuilding: Option<WebGlSessionInit>,
    lifetime: PresentationLifetime,
    limits: SparseResourceLimits,
    readbacks: Vec<super::readback::PendingBuffers>,
    readback_bytes: Rc<Cell<usize>>,
    capture_turn: Rc<Cell<Option<PresentedFrame>>>,
}
impl WebGlPresentation {
    pub async fn boot(
        canvas: HtmlCanvasElement,
        limits: SparseResourceLimits,
    ) -> Result<Self, String> {
        let mut init = WebGlSession::start(&canvas, limits).map_err(|e| e.to_string())?;
        let gl = canvas
            .get_context("webgl2")
            .map_err(|e| format!("owned context: {e:?}"))?
            .ok_or("initializer has no context")?
            .dyn_into::<web_sys::WebGl2RenderingContext>()
            .map_err(|_| "initializer context has the wrong type")?;
        let mut ownership = InitialContext(Some(gl));
        loop {
            if !canvas.is_connected() {
                return Err("appearance boot cancelled".into());
            }
            if let Some(session) = init.try_finish().map_err(|e| e.to_string())? {
                return Ok(Self {
                    canvas,
                    gl: ownership.0.take().expect("initializer owns its context"),
                    session: Some(session),
                    rebuilding: None,
                    lifetime: PresentationLifetime::new(ContextGeneration(1)),
                    limits,
                    readbacks: Vec::new(),
                    readback_bytes: Rc::new(Cell::new(0)),
                    capture_turn: Rc::new(Cell::new(None)),
                });
            }
            super::boot::next_frame().await?;
        }
    }
    fn failure(&self, message: impl ToString, producer_key: Option<u64>) -> PresentationFailure {
        PresentationFailure {
            backend: self.backend(),
            generation: self.generation(),
            producer_key,
            message: message.to_string(),
            native: None,
        }
    }
}
impl PresentationDriver for WebGlPresentation {
    fn backend(&self) -> PresentationBackend {
        PresentationBackend::WebGl2
    }
    fn generation(&self) -> ContextGeneration {
        self.lifetime.generation()
    }
    fn state(&self) -> PresentationState {
        self.lifetime.state()
    }
    fn content_mode(&self) -> FrameContentMode {
        FrameContentMode::PortableScene
    }
    fn wgpu_surface(&self) -> Option<&dyn Surface> {
        None
    }
    fn resize(&mut self, _: u32, _: u32) -> Result<(), PresentationFailure> {
        // Target dimensions are committed only after full recording admission.
        Ok(())
    }
    fn present(
        &mut self,
        frame: &PreparedFrame,
    ) -> Result<Option<PresentedFrame>, PresentationFailure> {
        if !self.lifetime.accepts(frame.generation) {
            return Ok(None);
        }
        let PreparedPaint::Portable(capture) = &frame.paint else {
            return Err(self.failure("WebGL requires portable paint", None));
        };
        if self.canvas.has_attribute("data-presentation-diagnostics") {
            let rows = capture
                .producers
                .iter()
                .map(|source| {
                    let mut counts = std::collections::BTreeMap::new();
                    for op in &source.scene.ops {
                        use netrender::SceneOp::*;
                        let name = match op {
                            Rect(_) => "Rect",
                            Stroke(_) => "Stroke",
                            Gradient(_) => "Gradient",
                            Image(_) => "Image",
                            Pattern(_) => "Pattern",
                            Shape(_) => "Shape",
                            GlyphRun(_) => "GlyphRun",
                            PushLayer(_) => "PushLayer",
                            PopLayer => "PopLayer",
                            Fragment(_) => "Fragment",
                        };
                        *counts.entry(name).or_insert(0usize) += 1;
                    }
                    let ops = counts
                        .iter()
                        .map(|(name, count)| format!("\"{name}\":{count}"))
                        .collect::<Vec<_>>()
                        .join(",");
                    format!(
                        "{{\"key\":{},\"revision\":{},\"images\":{},\"ops\":{{{ops}}}}}",
                        source.texture_key,
                        source.source_revision,
                        source.scene.image_sources.len()
                    )
                })
                .collect::<Vec<_>>()
                .join(",");
            let _ = self
                .canvas
                .set_attribute("data-portable-producer-inventory", &format!("[{rows}]"));
            let _ = self.canvas.set_attribute(
                "data-portable-paint-command-count",
                &capture.envelope.commands.len().to_string(),
            );
        }
        let scene = crate::portable::materialize(
            capture,
            frame.width,
            frame.height,
            frame.scale,
            self.limits,
        )
        .map_err(|e| self.failure(e.message, e.source_key))?;
        let generation = self.generation();
        let result = (|| {
            let session = self.session.as_mut().expect("ready context owns session");
            let recording = session.prepare(&scene, generation.0, frame.clear)?;
            if self.canvas.width() != frame.width {
                self.canvas.set_width(frame.width);
            }
            if self.canvas.height() != frame.height {
                self.canvas.set_height(frame.height);
            }
            recording.present()
        })();
        if let Err(error) = result {
            if matches!(
                error,
                netrender::vello_backends::WebGlSessionError::ContextLost
            ) {
                self.lose();
            }
            return Err(self.failure(error, None));
        }
        let stamp = frame.presentation_stamp();
        self.capture_turn.set(Some(stamp));
        let turn = self.capture_turn.clone();
        wasm_bindgen_futures::spawn_local(async move {
            if turn.get() == Some(stamp) {
                turn.set(None);
            }
        });
        Ok(Some(stamp))
    }
    fn lose(&mut self) {
        self.capture_turn.set(None);
        super::readback::cancel(&mut self.readbacks);
        self.lifetime.lose();
        self.rebuilding = None;
        self.session = None;
    }
    fn restore(&mut self) -> Result<bool, PresentationFailure> {
        match self.state() {
            PresentationState::Ready => return Ok(true),
            PresentationState::Disposed => return Ok(false),
            PresentationState::Lost => {
                // The host calls restoration only after the browser restores it.
                let generation = self
                    .generation()
                    .0
                    .checked_add(1)
                    .ok_or_else(|| self.failure("context generation exhausted", None))?;
                self.lifetime
                    .begin_restore()
                    .map_err(|e| self.failure(e, None))?;
                match WebGlSession::start_with_generation(&self.canvas, self.limits, generation) {
                    Ok(init) => self.rebuilding = Some(init),
                    Err(error) => {
                        self.lifetime.lose();
                        return Err(self.failure(error, None));
                    },
                }
            },
            PresentationState::Restoring => {},
        }
        let result = self
            .rebuilding
            .as_mut()
            .expect("restoring context owns initializer")
            .try_finish();
        match result {
            Ok(Some(session)) => {
                self.lifetime
                    .finish_restore()
                    .map_err(|e| self.failure(e, None))?;
                self.session = Some(session);
                self.rebuilding = None;
                Ok(true)
            },
            Ok(None) => Ok(false),
            Err(error) => {
                self.lose();
                Err(self.failure(error, None))
            },
        }
    }
    fn begin_capture(
        &mut self,
        stamp: PresentedFrame,
    ) -> Result<Box<dyn PendingPresentationCapture>, CaptureRefusal> {
        if !self.lifetime.accepts(stamp.context_generation)
            || self.capture_turn.get() != Some(stamp)
        {
            return Err(CaptureRefusal::PresentationError {
                backend: self.backend(),
                generation: self.generation(),
                source_key: None,
                error: "capture requires this context's current presentation turn".into(),
            });
        }
        let (capture, weak) = super::readback::start(
            &self.gl,
            stamp,
            self.readback_bytes.clone(),
            self.limits.max_cached_bytes,
        )?;
        self.readbacks.retain(|job| job.strong_count() > 0);
        self.readbacks.push(weak);
        Ok(capture)
    }
    fn dispose(&mut self) {
        if self.state() == PresentationState::Disposed {
            return;
        }
        self.capture_turn.set(None);
        super::readback::cancel(&mut self.readbacks);
        self.session = None;
        self.rebuilding = None;
        self.lifetime.dispose();
        release_context(&self.gl);
    }
}

impl Drop for WebGlPresentation {
    fn drop(&mut self) {
        self.dispose();
    }
}

/// Own the exclusive context before a driver exists, including rejected rAFs.
struct InitialContext(Option<web_sys::WebGl2RenderingContext>);
impl Drop for InitialContext {
    fn drop(&mut self) {
        if let Some(gl) = &self.0 {
            release_context(gl);
        }
    }
}
fn release_context(gl: &web_sys::WebGl2RenderingContext) {
    if let Ok(Some(extension)) = gl.get_extension("WEBGL_lose_context") {
        extension
            .unchecked_into::<web_sys::WebglLoseContext>()
            .lose_context();
    }
}
