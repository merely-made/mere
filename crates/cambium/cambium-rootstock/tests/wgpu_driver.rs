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
