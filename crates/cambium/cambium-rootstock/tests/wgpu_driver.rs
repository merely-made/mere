// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
use cambium_rootstock::{
    PresentationBackend, PresentationDriver, PresentationState, Surface, WgpuDriver,
};
use genet_render_host::RenderCore;
use std::{cell::Cell, rc::Rc, sync::Arc};
struct Unavailable(Arc<RenderCore>);
impl Surface for Unavailable {
    fn core(&self) -> &RenderCore {
        &self.0
    }
    fn format(&self) -> wgpu::TextureFormat {
        wgpu::TextureFormat::Rgba8Unorm
    }
    fn resize(&mut self, _: u32, _: u32) {}
    fn acquire(&self) -> Option<wgpu::SurfaceTexture> {
        None
    }
}
#[test]
fn surface_replacement_reuses_shared_device_and_terminal_disposal_cannot_reboot() {
    let core = Arc::new(
        RenderCore::boot(netrender::NetrenderOptions {
            enable_vello: true,
            tile_cache_size: Some(128),
            ..Default::default()
        })
        .expect("native test core"),
    );
    let owner = core.shared_recovery();
    let original = core.clone();
    let calls = Rc::new(Cell::new(0));
    let count = calls.clone();
    let mut driver = WgpuDriver::new(
        Box::new(Unavailable(core)),
        owner,
        Box::new(move |core| {
            assert!(
                Arc::ptr_eq(&core, &original),
                "surface-only replacement must reuse the device"
            );
            count.set(count.get() + 1);
            Ok(Box::new(Unavailable(core)))
        }),
    );
    assert_eq!(driver.backend(), PresentationBackend::Wgpu);
    let old = driver.generation();
    assert!(driver.wgpu_surface().is_some());
    driver.lose();
    assert_eq!(driver.state(), PresentationState::Lost);
    assert!(driver.wgpu_surface().is_none());
    assert_eq!(calls.get(), 0);
    assert!(driver.restore().unwrap());
    assert_eq!(calls.get(), 1);
    assert!(driver.generation().0 > old.0);
    assert!(driver.wgpu_surface().is_some());
    driver.dispose();
    assert_eq!(driver.state(), PresentationState::Disposed);
    assert!(driver.wgpu_surface().is_none());
    assert!(!driver.restore().unwrap());
    assert_eq!(calls.get(), 1);
}

type Child = Box<dyn cambium::AnyView<(), (), cambium::GenetCtx, cambium::GenetElement>>;
type Logic = fn(&()) -> Child;
fn view(_: &()) -> Child {
    Box::new(cambium::el("div", "portable WebGPU host"))
}
struct Window;
impl cambium_rootstock::HostWindow for Window {
    fn request_redraw(&self) {}
    fn inner_size(&self) -> (u32, u32) {
        (320, 240)
    }
    fn scale_factor(&self) -> f64 {
        1.25
    }
    fn set_ime_allowed(&self, _: bool) {}
    fn set_ime_cursor_area(&self, _: f64, _: f64, _: f64, _: f64) {}
}

#[test]
fn portable_webgpu_route_prepares_portable_paint_and_attributes_materialization_refusal() {
    use cambium_rootstock::*;
    let core = Arc::new(
        RenderCore::boot(netrender::NetrenderOptions {
            enable_vello: true,
            tile_cache_size: Some(128),
            ..Default::default()
        })
        .expect("native test core"),
    );
    let owner = core.shared_recovery();
    let texture = WgpuDriver::new(
        Box::new(Unavailable(core.clone())),
        owner.clone(),
        Box::new(|core| Ok(Box::new(Unavailable(core)))),
    );
    assert_eq!(texture.content_mode(), FrameContentMode::NativeTexture);

    let calls = Rc::new(std::cell::RefCell::new(Vec::new()));
    let seen = calls.clone();
    let driver = WgpuDriver::new(
        Box::new(Unavailable(core)),
        owner,
        Box::new(|core| Ok(Box::new(Unavailable(core)))),
    )
    .with_portable_scene(Box::new(move |capture, width, height, scale| {
        seen.borrow_mut()
            .push((capture.producers.len(), width, height, scale));
        Err((Some(7), "materialization refused".into()))
    }));
    assert_eq!(driver.content_mode(), FrameContentMode::PortableScene);
    let generation = driver.generation();

    let mut state = HostState::<(), Logic, Child>::new();
    state.runner = Some(cambium::GenetAppRunner::new(
        Rc::new(std::cell::RefCell::new(ScriptedDom::new())),
        view as Logic,
        (),
    ));
    state.window = Some(Box::new(Window));
    state.presentation_driver = Some(Box::new(driver));
    let wake = HostWake::new(state.wake_pending.clone(), Arc::new(|| {}));
    let mut host = Host::new(
        HostOptions::default(),
        None,
        HostHooks::inert(),
        state,
        wake,
    );
    let frame = host
        .prepare_presentation_frame()
        .unwrap()
        .expect("visible portable frame");
    assert!(matches!(frame.paint, PreparedPaint::Portable(_)));
    let failure = host
        .s
        .presentation_driver
        .as_mut()
        .unwrap()
        .present(&frame)
        .expect_err("materializer refusal reaches the host");
    assert_eq!(*calls.borrow(), [(0, 320, 240, 1.25)]);
    assert_eq!(failure.backend, PresentationBackend::Wgpu);
    assert_eq!(failure.generation, generation);
    assert_eq!(failure.producer_key, Some(7));
    assert_eq!(failure.message, "materialization refused");
}
