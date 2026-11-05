// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Driver opt-in and capture ownership; no fabricated queue presentation.

use super::*;
use crate::presentation::wgpu::WgpuOutput;
use crate::{
    HostHooks, HostOptions, HostState, HostWake, NativePresentedImage, PendingPresentationCapture,
    PresentationDriver, Runner, ScriptedDom, WgpuDriver,
};
use ::wgpu;
use std::{
    cell::{Cell, RefCell},
    sync::Arc,
    time::{Duration, Instant},
};

struct CustomDriver {
    surface: Option<Box<dyn Surface>>,
    generation: ContextGeneration,
    calls: Rc<Cell<usize>>,
}
impl PresentationDriver for CustomDriver {
    fn backend(&self) -> PresentationBackend {
        PresentationBackend::Wgpu
    }
    fn generation(&self) -> ContextGeneration {
        self.generation
    }
    fn content_mode(&self) -> FrameContentMode {
        FrameContentMode::NativeTexture
    }
    fn wgpu_surface(&self) -> Option<&dyn Surface> {
        self.surface.as_deref()
    }
    fn resize(&mut self, _: u32, _: u32) -> Result<(), PresentationFailure> {
        Ok(())
    }
    fn present(
        &mut self,
        _: &PreparedFrame,
    ) -> Result<Option<crate::PresentedFrame>, PresentationFailure> {
        self.calls.set(self.calls.get() + 1);
        Ok(None)
    }
    fn lose(&mut self) {}
    fn restore(&mut self) -> Result<bool, PresentationFailure> {
        Ok(true)
    }
    fn begin_capture(
        &mut self,
        _: crate::PresentedFrame,
    ) -> Result<crate::CaptureFrame, CaptureRefusal> {
        Err(CaptureRefusal::HostDisposed)
    }
    fn dispose(&mut self) {}
}

#[test]
fn custom_driver_declaring_native_wgpu_does_not_implicitly_opt_in() {
    let driver = CustomDriver {
        surface: None,
        generation: ContextGeneration(1),
        calls: Rc::new(Cell::new(0)),
    };
    assert_eq!(driver.backend(), PresentationBackend::Wgpu);
    assert_eq!(driver.content_mode(), FrameContentMode::NativeTexture);
    assert!(!driver.supports_native_receiving());
}

struct Headless(Arc<genet_render_host::RenderCore>);
impl Surface for Headless {
    fn core(&self) -> &genet_render_host::RenderCore {
        &self.0
    }
    fn shared_core(&self) -> Option<Arc<genet_render_host::RenderCore>> {
        Some(self.0.clone())
    }
    fn format(&self) -> wgpu::TextureFormat {
        wgpu::TextureFormat::Rgba8Unorm
    }
    fn resize(&mut self, _: u32, _: u32) {}
    fn acquire(&self) -> Option<wgpu::SurfaceTexture> {
        None
    }
}
fn core() -> Arc<genet_render_host::RenderCore> {
    Arc::new(
        genet_render_host::RenderCore::boot(netrender::NetrenderOptions {
            enable_vello: true,
            tile_cache_size: Some(8),
            ..Default::default()
        })
        .expect("actual native adapter required"),
    )
}
fn checked_image(core: &genet_render_host::RenderCore, color: [f32; 4]) -> NativePresentedImage {
    let mut scene = netrender::Scene::new(8, 6);
    scene.push_rect(0., 0., 8., 6., color);
    let (texture, view) = core
        .try_rasterize_scaled_for(
            7193,
            &scene,
            8,
            6,
            netrender::ColorLoad::Clear(wgpu::Color::TRANSPARENT),
            1.,
        )
        .unwrap();
    NativePresentedImage {
        output: WgpuOutput {
            _texture: texture,
            view,
        },
    }
}
fn stamp(driver: &WgpuDriver, sequence: u64) -> crate::PresentedFrame {
    crate::PresentedFrame {
        context_generation: driver.generation(),
        host: 7193,
        sequence,
        width: 8,
        height: 6,
        layout_scale: 1.,
    }
}
fn collect(capture: &mut dyn PendingPresentationCapture) -> (crate::PresentedFrame, crate::Frame) {
    let deadline = Instant::now() + Duration::from_secs(6);
    loop {
        if let Some(result) = capture.poll() {
            return result.unwrap();
        }
        assert!(Instant::now() < deadline, "driver capture did not finish");
        std::thread::sleep(Duration::from_millis(1));
    }
}

#[test]
#[ignore = "actual GPU; execute the serialized native driver capture gate"]
fn driver_keeps_capture_a_pixels_and_stamp_after_receiving_b() {
    let core = core();
    let owner = genet_render_host::SharedRenderCore::with_factory(
        core.clone(),
        Arc::new(|| Err("replacement is outside the capture control".into())),
    );
    let mut driver = WgpuDriver::new(
        Box::new(Headless(core.clone())),
        owner,
        Box::new(|core| Ok(Box::new(Headless(core)))),
    );
    assert!(driver.supports_native_receiving());
    // Premise: checked offscreen images supplied to the host-only receiving
    // hook. This test performs no surface acquisition or queue presentation.
    let a = stamp(&driver, 1);
    driver.native_presented(a, checked_image(&core, [1., 0., 0., 1.]));
    let mut capture_a = driver.begin_capture(a).unwrap();
    let b = stamp(&driver, 2);
    driver.native_presented(b, checked_image(&core, [0., 1., 0., 1.]));
    let mut capture_b = driver.begin_capture(b).unwrap();
    assert!(matches!(
        driver.begin_capture(a),
        Err(CaptureRefusal::ContextLost { .. })
    ));
    let mut wrong = b;
    wrong.host += 1;
    assert!(matches!(
        driver.begin_capture(wrong),
        Err(CaptureRefusal::ContextLost { .. })
    ));
    wrong = b;
    wrong.context_generation.0 += 1;
    assert!(matches!(
        driver.begin_capture(wrong),
        Err(CaptureRefusal::ContextLost { .. })
    ));
    driver.native_presented(wrong, checked_image(&core, [0., 0., 1., 1.]));
    let mut still_b = driver.begin_capture(b).unwrap();

    let (captured_a, red) = collect(&mut *capture_a);
    let (captured_b, green) = collect(&mut *capture_b);
    let (latest, unchanged) = collect(&mut *still_b);
    assert_eq!(captured_a, a);
    assert_eq!(captured_b, b);
    assert_eq!(latest, b);
    assert_eq!((red.width, red.height), (8, 6));
    assert!(red.rgba.chunks_exact(4).all(|p| p == [255, 0, 0, 255]));
    assert!(green.rgba.chunks_exact(4).all(|p| p == [0, 255, 0, 255]));
    assert_eq!(unchanged.rgba, green.rgba);
    assert!(capture_a.poll().is_none());
    assert!(capture_b.poll().is_none());
    let mut cancelled = driver.begin_capture(b).unwrap();
    driver.lose();
    assert!(matches!(
        cancelled.poll(),
        Some(Err(CaptureRefusal::ContextLost { .. }))
    ));
    assert!(cancelled.poll().is_none());
    assert!(matches!(
        driver.begin_capture(b),
        Err(CaptureRefusal::ContextLost { .. })
    ));
}

#[test]
#[ignore = "actual GPU; execute the serialized native driver policy gate"]
fn host_preserves_custom_driver_policy_even_with_a_shared_native_surface() {
    use cambium::{AnyView, GenetCtx, GenetElement, el};
    type Child = Box<dyn AnyView<(), (), GenetCtx, GenetElement>>;
    type Logic = fn(&()) -> Child;
    fn root(_: &()) -> Child {
        Box::new(el("div", "custom policy"))
    }
    struct Window;
    impl crate::HostWindow for Window {
        fn request_redraw(&self) {}
        fn inner_size(&self) -> (u32, u32) {
            (16, 16)
        }
        fn scale_factor(&self) -> f64 {
            1.
        }
        fn set_ime_allowed(&self, _: bool) {}
        fn set_ime_cursor_area(&self, _: f64, _: f64, _: f64, _: f64) {}
    }
    let core = core();
    let calls = Rc::new(Cell::new(0));
    let mut state = HostState::<(), Logic, Child>::new();
    state.shared.render_core = Some(core.clone());
    state.window = Some(Box::new(Window));
    state.runner = Some(Runner::new(
        Rc::new(RefCell::new(ScriptedDom::new())),
        root as Logic,
        (),
    ));
    state.presentation_driver = Some(Box::new(CustomDriver {
        surface: Some(Box::new(Headless(core.clone()))),
        generation: ContextGeneration(core.generation()),
        calls: calls.clone(),
    }));
    let wake = HostWake::new(state.wake_pending.clone(), Arc::new(|| {}));
    let mut host = Host::new(
        HostOptions::default(),
        None,
        HostHooks::inert(),
        state,
        wake,
    );
    host.redraw();
    assert_eq!(calls.get(), 1, "the custom present policy must run");
    assert!(host.s.pending_native.is_none());
    assert!(!host.s.native_receiving);
    assert!(!core.native_frame_busy());
}
