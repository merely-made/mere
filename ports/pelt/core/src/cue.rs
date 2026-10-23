// SPDX-License-Identifier: MPL-2.0
//! Pelt's retained controller as a host-device shared surface. No window,
//! second device, fetcher or replacement document session is constructed.
use crate::PeltController;
use inker::webcue::*;
use std::time::Duration;
use wgpu_cue::{RenderContext, Rendered, TextureSource};
use wgpu_cue_netrender::{Scene, SceneRasterizer, WgpuHandles};

/// Renderer state is separate from retained custody so a placement pool can
/// lend its current controller without constructing a replacement session.
pub struct PeltSceneRenderer {
    rasterizer: SceneRasterizer,
    submitted_scope: Option<Scope>,
    pending: bool,
}
impl PeltSceneRenderer {
    pub fn new(handles: WgpuHandles) -> Result<Self> {
        Ok(Self {
            rasterizer: SceneRasterizer::new(handles)?,
            submitted_scope: None,
            pending: false,
        })
    }
    pub fn suspend(&mut self) {
        self.rasterizer.suspend();
        self.pending = false;
    }
    pub fn pending(&self) -> bool {
        self.pending
    }
    pub fn bind<'a>(
        &'a mut self,
        controller: &'a mut PeltController<Scene>,
    ) -> PeltSceneSource<'a> {
        PeltSceneSource {
            controller,
            renderer: self,
        }
    }
}
pub struct PeltSceneSource<'a> {
    controller: &'a mut PeltController<Scene>,
    renderer: &'a mut PeltSceneRenderer,
}
impl Surface for PeltSceneSource<'_> {
    fn scope(&self) -> Scope {
        self.controller.cue_surface().scope()
    }
    fn capabilities(&self) -> &CapabilityReport {
        self.controller.cue_surface().capabilities()
    }
    fn set_viewport(&mut self, v: Viewport) -> Result<()> {
        self.controller.cue_surface_mut().set_viewport(v)
    }
    fn set_visible(&mut self, visible: bool) -> Result<()> {
        if !visible {
            self.renderer.suspend();
        }
        self.controller.cue_surface_mut().set_visible(visible)
    }
    fn input(&mut self, input: Input) -> Result<()> {
        self.controller.cue_surface_mut().input(input)
    }
    fn request(&mut self, id: RequestId, command: Command) -> Result<RequestKey> {
        self.controller.cue_surface_mut().request(id, command)
    }
    fn semantic_action(&mut self, action: SemanticAction) -> Result<()> {
        self.controller.cue_surface_mut().semantic_action(action)
    }
    fn advance(&mut self, now: Duration) -> Result<()> {
        self.controller.cue_surface_mut().advance(now)
    }
    fn poll_event(&mut self) -> Option<Envelope> {
        self.controller.poll_cue_event()
    }
    fn retire(&mut self) -> Result<()> {
        self.renderer.suspend();
        self.controller.cue_surface_mut().retire()
    }
}
impl TextureSource for PeltSceneSource<'_> {
    fn render(&mut self, cx: &RenderContext<'_>, _needs_frame: bool) -> Result<Rendered> {
        if self
            .renderer
            .submitted_scope
            .is_some_and(|scope| scope != self.scope())
        {
            self.renderer.suspend();
        }
        self.set_viewport(cx.viewport)?;
        if !self.controller.cue_surface().visible() {
            return Ok(Rendered::Pending);
        }
        // Preparing a new scene while an unchanged frame is pending would
        // continually supersede it. Poll the submitted frame before new layout.
        if let Some(rendered) = self.renderer.rasterizer.poll(cx)? {
            self.renderer.pending = matches!(rendered, Rendered::Pending);
            if matches!(rendered, Rendered::Changed(_)) {
                self.controller.cue_surface_mut().observe_frame()?;
            }
            return Ok(rendered);
        }
        let width = (cx.viewport.width as f32 / cx.viewport.scale)
            .ceil()
            .max(1.) as u32;
        let height = (cx.viewport.height as f32 / cx.viewport.scale)
            .ceil()
            .max(1.) as u32;
        let scene = self
            .controller
            .try_frame(width, height)
            .map_err(Error::Backend)?;
        let revision = self.controller.cue_surface_mut().next_frame_revision()?;
        self.renderer.submitted_scope = Some(self.scope());
        let rendered = self.renderer.rasterizer.render(
            cx,
            &scene,
            FrameStamp {
                scope: self.scope(),
                allocation_epoch: 0,
                content_revision: revision,
            },
        )?;
        self.renderer.pending = matches!(rendered, Rendered::Pending);
        Ok(rendered)
    }
}

pub struct PeltTextureSource {
    controller: PeltController<Scene>,
    renderer: PeltSceneRenderer,
}
impl PeltTextureSource {
    pub fn new(controller: PeltController<Scene>, handles: WgpuHandles) -> Result<Self> {
        Ok(Self {
            controller,
            renderer: PeltSceneRenderer::new(handles)?,
        })
    }
    pub fn rebind_device(&mut self, handles: WgpuHandles) -> Result<()> {
        self.renderer = PeltSceneRenderer::new(handles)?;
        Ok(())
    }
    pub fn controller(&self) -> &PeltController<Scene> {
        &self.controller
    }
    pub fn controller_mut(&mut self) -> &mut PeltController<Scene> {
        self.renderer.suspend();
        &mut self.controller
    }
}
impl Surface for PeltTextureSource {
    fn scope(&self) -> Scope {
        self.controller.cue_surface().scope()
    }
    fn capabilities(&self) -> &CapabilityReport {
        self.controller.cue_surface().capabilities()
    }
    fn set_viewport(&mut self, v: Viewport) -> Result<()> {
        self.controller.cue_surface_mut().set_viewport(v)
    }
    fn set_visible(&mut self, visible: bool) -> Result<()> {
        if !visible {
            self.renderer.suspend();
        }
        self.controller.cue_surface_mut().set_visible(visible)
    }
    fn input(&mut self, input: Input) -> Result<()> {
        self.controller.cue_surface_mut().input(input)
    }
    fn request(&mut self, id: RequestId, command: Command) -> Result<RequestKey> {
        self.controller.cue_surface_mut().request(id, command)
    }
    fn semantic_action(&mut self, action: SemanticAction) -> Result<()> {
        self.controller.cue_surface_mut().semantic_action(action)
    }
    fn advance(&mut self, now: Duration) -> Result<()> {
        self.controller.cue_surface_mut().advance(now)
    }
    fn poll_event(&mut self) -> Option<Envelope> {
        self.controller.poll_cue_event()
    }
    fn retire(&mut self) -> Result<()> {
        self.renderer.suspend();
        self.controller.cue_surface_mut().retire()
    }
}
impl TextureSource for PeltTextureSource {
    fn render(&mut self, cx: &RenderContext<'_>, needs_frame: bool) -> Result<Rendered> {
        self.renderer
            .bind(&mut self.controller)
            .render(cx, needs_frame)
    }
}

/// Both presentation directions behind one shared control object. Routing,
/// history and held resources come from the content that the host already opened.
pub struct PeltSharedContent {
    route: crate::PeltRoute,
    producer: SharedProducer,
}
enum SharedProducer {
    Retained(Box<PeltTextureSource>),
    Browser(Box<inker::cue_browser::BrowserSurface>),
}
impl PeltSharedContent {
    /// Embed an already-bound native browser with its concrete capability
    /// report and scope. Routing policy stays with the caller.
    pub fn from_browser(
        route: crate::PeltRoute,
        producer: inker::cue_browser::BrowserSurface,
    ) -> Self {
        Self {
            route,
            producer: SharedProducer::Browser(Box::new(producer)),
        }
    }
    /// A browser lane requires its concrete non-blocking frame binding. A
    /// retained Genet lane uses the supplied host device and ignores that option.
    pub fn from_content(
        content: crate::PeltContent<Scene>,
        handles: WgpuHandles,
        native: Option<(PresentationMode, inker::cue_browser::FrameReader)>,
    ) -> Result<Self> {
        let route = content.route().clone();
        let producer = match content.into_cue_lane() {
            Ok(controller) => {
                SharedProducer::Retained(Box::new(PeltTextureSource::new(controller, handles)?))
            },
            Err((producer, viewport, owner)) => match producer {
                crate::content::PeltBrowser::Shared(producer) => {
                    SharedProducer::Browser(Box::new(producer))
                },
                crate::content::PeltBrowser::Platform(producer) => {
                    let (mode, frames) = native.ok_or_else(|| {
                        Error::Unsupported(
                            "browser lane requires an explicit owned frame binding".into(),
                        )
                    })?;
                    SharedProducer::Browser(Box::new(inker::cue_browser::BrowserSurface::new(
                        producer,
                        Scope {
                            owner: owner.into(),
                            generation: 1,
                        },
                        Viewport {
                            width: viewport.0.max(1),
                            height: viewport.1.max(1),
                            scale: 1.,
                        },
                        mode,
                        frames,
                    )?))
                },
            },
        };
        Ok(Self { route, producer })
    }
    pub fn rebind_device(&mut self, handles: WgpuHandles) -> Result<()> {
        match &mut self.producer {
            SharedProducer::Retained(producer) => producer.rebind_device(handles),
            SharedProducer::Browser(_) => Err(Error::Unsupported(
                "native importer reconstruction belongs to its engine/host frame binding".into(),
            )),
        }
    }
    pub fn route(&self) -> &crate::PeltRoute {
        &self.route
    }
    pub fn source(&mut self) -> wgpu_cue::Source<'_> {
        match &mut self.producer {
            SharedProducer::Retained(producer) => wgpu_cue::Source::HostDevice(producer.as_mut()),
            SharedProducer::Browser(producer) => wgpu_cue::Source::Native(producer.as_mut()),
        }
    }
    fn surface(&self) -> &dyn Surface {
        match &self.producer {
            SharedProducer::Retained(producer) => producer.as_ref(),
            SharedProducer::Browser(producer) => producer.as_ref(),
        }
    }
    fn surface_mut(&mut self) -> &mut dyn Surface {
        match &mut self.producer {
            SharedProducer::Retained(producer) => producer.as_mut(),
            SharedProducer::Browser(producer) => producer.as_mut(),
        }
    }
}
impl Surface for PeltSharedContent {
    fn scope(&self) -> Scope {
        self.surface().scope()
    }
    fn capabilities(&self) -> &CapabilityReport {
        self.surface().capabilities()
    }
    fn set_viewport(&mut self, v: Viewport) -> Result<()> {
        self.surface_mut().set_viewport(v)
    }
    fn set_visible(&mut self, v: bool) -> Result<()> {
        self.surface_mut().set_visible(v)
    }
    fn input(&mut self, v: Input) -> Result<()> {
        self.surface_mut().input(v)
    }
    fn request(&mut self, id: RequestId, command: Command) -> Result<RequestKey> {
        self.surface_mut().request(id, command)
    }
    fn semantic_action(&mut self, v: SemanticAction) -> Result<()> {
        self.surface_mut().semantic_action(v)
    }
    fn advance(&mut self, now: Duration) -> Result<()> {
        self.surface_mut().advance(now)
    }
    fn poll_event(&mut self) -> Option<Envelope> {
        self.surface_mut().poll_event()
    }
    fn retire(&mut self) -> Result<()> {
        self.surface_mut().retire()
    }
}
