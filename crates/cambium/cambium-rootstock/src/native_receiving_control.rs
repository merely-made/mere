// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Common R2/R3 control for the redraw return boundary. No swapchain is acquired.

use crate::{
    Host, HostHooks, HostOptions, HostState, HostWake, HostWindow, Runner, ScriptedDom, Surface,
};
use cambium::{AnyView, GenetCtx, GenetElement, el};
use genet_render_host::RenderCore;
use std::{cell::RefCell, rc::Rc, sync::Arc};

type Child = Box<dyn AnyView<(), (), GenetCtx, GenetElement>>;
type Logic = fn(&()) -> Child;

fn root(_: &()) -> Child {
    Box::new(el("div", "").attr("style", "width:8px;height:6px;background:rgb(120,30,80);"))
}

struct HeadlessSurface(Arc<RenderCore>);
impl Surface for HeadlessSurface {
    fn core(&self) -> &RenderCore {
        &self.0
    }
    // R3_SHARED_CORE_ADAPTER_BEGIN
    fn shared_core(&self) -> Option<Arc<RenderCore>> {
        Some(self.0.clone())
    }
    // R3_SHARED_CORE_ADAPTER_END
    fn format(&self) -> wgpu::TextureFormat {
        wgpu::TextureFormat::Rgba8Unorm
    }
    fn resize(&mut self, _: u32, _: u32) {}
    fn acquire(&self) -> Option<wgpu::SurfaceTexture> {
        None
    }
}

struct Window;
impl HostWindow for Window {
    fn request_redraw(&self) {}
    fn inner_size(&self) -> (u32, u32) {
        (16, 16)
    }
    fn scale_factor(&self) -> f64 {
        1.0
    }
    fn set_ime_allowed(&self, _: bool) {}
    fn set_ime_cursor_area(&self, _: f64, _: f64, _: f64, _: f64) {}
}

#[test]
#[ignore = "actual GPU; serialized common pre-fix/post-fix redraw boundary control"]
fn redraw_returns_while_the_native_frame_is_receiving() {
    let core = Arc::new(
        RenderCore::boot(netrender::NetrenderOptions {
            enable_vello: true,
            tile_cache_size: Some(16),
            ..Default::default()
        })
        .expect("the control requires an actual headless adapter"),
    );
    let mut state = HostState::<(), Logic, Child>::new();
    state.shared.render_core = Some(core.clone());
    state.surface = Some(Box::new(HeadlessSurface(core.clone())));
    state.window = Some(Box::new(Window));
    state.runner = Some(Runner::new(
        Rc::new(RefCell::new(ScriptedDom::new())),
        root as Logic,
        (),
    ));
    let wake = HostWake::new(state.wake_pending.clone(), Arc::new(|| {}));
    let mut host = Host::new(
        HostOptions::default(),
        None,
        HostHooks::inert(),
        state,
        wake,
    );

    host.redraw();

    assert!(
        core.native_frame_busy(),
        "redraw must return with native acceptance still pending; the synchronous R2 path returns idle"
    );
    drop(host.take_disposal());
}
