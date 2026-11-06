// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Common submission-tail witness; the prior-picture proof is CPU supplied.
//! The new candidate is an actual native batch; no swapchain is fabricated.

use crate::{
    A11yAdapterState, A11yOwner, A11yPublication, A11yPublicationOutcome, A11yRequest, A11ySync,
    Accessibility, ContextGeneration, Host, HostHooks, HostOptions, HostState, HostTree as _,
    HostWake, HostWindow, OwnedLayout, PresentedFrame, ProducerRegistry, Runner, ScriptedDom,
    Surface, WindowDom,
};
use cambium::{AnyView, GenetCtx, GenetElement, el};
use genet_render_host::RenderCore;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    sync::Arc,
};

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
    fn shared_core(&self) -> Option<Arc<RenderCore>> {
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
struct Window;
impl HostWindow for Window {
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
struct Adapter(Rc<Cell<usize>>);
impl Accessibility for Adapter {
    fn adapter_state(&self) -> A11yAdapterState {
        A11yAdapterState::Installed
    }
    // A11Y_DRAIN_ADAPTER_BEGIN
    fn drain_actions(&mut self, _: &A11yOwner) -> Option<Vec<A11yRequest>> {
        Some(Vec::new())
    }
    // A11Y_DRAIN_ADAPTER_END
    fn sync(
        &mut self,
        _: &A11yOwner,
        _: &WindowDom<'_>,
        _: &OwnedLayout,
        _: &sprigging::LeafRegistry<u64>,
        _: &ProducerRegistry,
        _: Option<u64>,
        _: f64,
    ) -> A11ySync {
        self.0.set(self.0.get() + 1);
        A11ySync {
            requests: Vec::new(),
            publication: A11yPublication {
                adapter: self.adapter_state(),
                outcome: A11yPublicationOutcome::Submitted {
                    nodes: 1,
                    contribution_issues: 0,
                },
            },
        }
    }
}

// COMMON_TEST_BODY_BEGIN
#[test]
#[ignore = "actual GPU; serialized native accessibility submission-tail control"]
fn pending_submission_does_not_reproject_the_prior_accessibility_tree() {
    let core = Arc::new(
        RenderCore::boot(netrender::NetrenderOptions {
            enable_vello: true,
            tile_cache_size: Some(16),
            ..Default::default()
        })
        .expect("the control requires an actual headless adapter"),
    );
    let count = Rc::new(Cell::new(0));
    let mut state = HostState::<(), Logic, Child>::new();
    state.shared.render_core = Some(core.clone());
    state.surface = Some(Box::new(HeadlessSurface(core.clone())));
    state.window = Some(Box::new(Window));
    state.runner = Some(Runner::new(
        Rc::new(RefCell::new(ScriptedDom::new())),
        root as Logic,
        (),
    ));
    state.a11y = Some(Box::new(Adapter(count.clone())));
    let wake = HostWake::new(state.wake_pending.clone(), Arc::new(|| {}));
    let mut host = Host::new(
        HostOptions::default(),
        None,
        HostHooks::inert(),
        state,
        wake,
    );
    host.relayout(16., 16.);
    // A controlled prior-picture proof, not a claim of physical presentation.
    let input = {
        let runner = host.s.runner.as_ref().unwrap();
        let dom = runner.dom();
        let dom = dom.borrow();
        Rc::new(
            host.s
                .layout
                .as_ref()
                .unwrap()
                .presented_input(&WindowDom::new(&dom, runner.mount())),
        )
    };
    host.s.native_receiving = true;
    host.s.displayed_input = Some(input);
    host.s.displayed_presentation = Some(PresentedFrame {
        host: host.s.presentation_host,
        sequence: 1,
        width: 16,
        height: 16,
        layout_scale: 1.,
        context_generation: ContextGeneration(0),
    });
    host.s.last_redraw_presentation = host.s.displayed_presentation;
    host.sync_a11y();
    assert_eq!(count.get(), 1);
    host.redraw();
    assert!(core.native_frame_busy(), "candidate is still receiving");
    host.sync_a11y();
    assert_eq!(
        count.get(),
        1,
        "submission must drain old actions without another projection"
    );
    drop(host.take_disposal());
}
