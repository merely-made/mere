// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::{CaptureRefusal, HostHooks, HostOptions, HostState, HostWake, Runner, ScriptedDom};
use cambium::{AnyView, GenetCtx, GenetElement, custom_leaf};
use paint_list_api::{
    CommonPlacement, LayoutPoint, LayoutRect, PaintCmd, PaintList as _, items::ExternalTextureItem,
};
use std::{cell::RefCell, rc::Rc, sync::Arc};

struct TestSurface(genet_render_host::RenderCore);
impl crate::Surface for TestSurface {
    fn core(&self) -> &genet_render_host::RenderCore {
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

struct TestWindow;
impl crate::HostWindow for TestWindow {
    fn request_redraw(&self) {}
    fn inner_size(&self) -> (u32, u32) {
        (8, 6)
    }
    fn scale_factor(&self) -> f64 {
        1.0
    }
    fn set_ime_allowed(&self, _: bool) {}
    fn set_ime_cursor_area(&self, _: f64, _: f64, _: f64, _: f64) {}
}

struct BorderLeaf(Rc<std::cell::Cell<bool>>);
impl sprigging::Leaf for BorderLeaf {
    fn measure(&mut self, _: sprigging::SizeHint, _: sprigging::SizeHint) -> sprigging::Size {
        sprigging::Size {
            width: 8.0,
            height: 6.0,
        }
    }
    fn paint(&mut self, cx: &mut sprigging::PaintCx<'_>) {
        let bounds = LayoutRect::new(LayoutPoint::new(0.0, 0.0), LayoutPoint::new(8.0, 6.0));
        if self.0.get() {
            cx.emit(PaintCmd::DrawBorder(paint_list_api::items::BorderItem {
                placement: CommonPlacement::new(bounds),
                widths: paint_list_api::LayoutSideOffsets::new(1.0, 1.0, 1.0, 1.0),
                details: paint_list_api::items::BorderDetails::Normal(
                    paint_list_api::NormalBorder {
                        do_aa: false,
                        ..Default::default()
                    },
                ),
            }));
        } else {
            cx.emit(PaintCmd::DrawRect(paint_list_api::items::RectItem {
                placement: CommonPlacement::new(bounds),
                color: paint_list_api::ColorF::new(1.0, 0.0, 0.0, 1.0),
            }));
        }
    }
    fn paint_dirty(&self) -> bool {
        true
    }
}

#[test]
fn rejected_leaf_epoch_refuses_presentation_and_recovers_without_replacing_old_fragment() {
    let bad = Rc::new(std::cell::Cell::new(false));
    let mut state: HostState<App, Logic, Child> = HostState::new();
    state.shared.sheet = "custom-leaf { display:block;width:8px;height:6px; }".into();
    state
        .shared
        .leaves
        .insert(7, Box::new(BorderLeaf(bad.clone())));
    state.surface = Some(Box::new(TestSurface(
        genet_render_host::RenderCore::boot(netrender::NetrenderOptions {
            enable_vello: true,
            tile_cache_size: Some(32),
            ..Default::default()
        })
        .expect("retained leaf test needs a headless adapter"),
    )));
    state.window = Some(Box::new(TestWindow));
    state.runner = Some(Runner::new(
        Rc::new(RefCell::new(ScriptedDom::new())),
        root as Logic,
        App {
            draft: String::new(),
        },
    ));
    let wake = HostWake::new(state.wake_pending.clone(), Arc::new(|| {}));
    let mut host = Host::new(
        HostOptions::default(),
        None,
        HostHooks::inert(),
        state,
        wake,
    );

    assert!(host.prepare_presentation_frame().unwrap().is_some());
    let original = host.s.shared.leaf_fragments[&7];
    bad.set(true);
    let refusal = Rc::new(RefCell::new(None));
    let out = refusal.clone();
    host.s.pending_capture = Some(Box::new(move |result| {
        *out.borrow_mut() = Some(result.err().expect("capture refused"));
    }));
    host.redraw();
    assert_eq!(host.s.shared.leaf_fragments[&7], original);
    assert!(matches!(
        refusal.borrow().as_ref(),
        Some(CaptureRefusal::PresentationError { error, .. })
            if error.contains("non-antialiased border geometry")
    ));
    assert!(host.s.last_redraw_presentation.is_none());

    bad.set(false);
    assert!(host.prepare_presentation_frame().unwrap().is_some());
    let corrected = host.s.shared.leaf_fragments[&7];
    assert_eq!(
        corrected.0, original.0,
        "the old id is updated after recovery"
    );
    assert!(corrected.1 > original.1, "only an admitted epoch is cached");
}

struct App {
    draft: String,
}
type Child = Box<dyn AnyView<App, (), GenetCtx, GenetElement>>;
type Logic = fn(&App) -> Child;
fn root(_: &App) -> Child {
    Box::new(custom_leaf::<App, ()>(7, 8, 6))
}
struct UnresolvedLeaf;
impl sprigging::Leaf for UnresolvedLeaf {
    fn measure(&mut self, _: sprigging::SizeHint, _: sprigging::SizeHint) -> sprigging::Size {
        sprigging::Size {
            width: 8.0,
            height: 6.0,
        }
    }
    fn paint(&mut self, cx: &mut sprigging::PaintCx<'_>) {
        cx.emit(PaintCmd::DrawExternalTexture(ExternalTextureItem {
            placement: CommonPlacement::new(LayoutRect::new(
                LayoutPoint::new(0.0, 0.0),
                LayoutPoint::new(8.0, 6.0),
            )),
            texture_key: 7,
            content_generation: Some(1),
            opacity: 0.73,
        }));
    }
    fn paint_dirty(&self) -> bool {
        false
    }
}

#[test]
fn actual_host_missing_gpu_binding_refuses_all_capture_forms_once_and_keeps_authority() {
    let mut state: HostState<App, Logic, Child> = HostState::new();
    state.shared.sheet = "custom-leaf { display:block;width:8px;height:6px; }".into();
    state.shared.leaves.insert(7, Box::new(UnresolvedLeaf));
    state.runner = Some(Runner::new(
        Rc::new(RefCell::new(ScriptedDom::new())),
        root as Logic,
        App {
            draft: "application-owned draft".into(),
        },
    ));
    let wake = HostWake::new(state.wake_pending.clone(), Arc::new(|| {}));
    let mut host: Host<App, Logic, Child> = Host::new(
        HostOptions::default(),
        None,
        HostHooks::inert(),
        state,
        wake,
    );
    host.relayout(8.0, 6.0);
    let list = host.emit_paint_list(8.0, 6.0, None).unwrap();
    assert!(list.commands().iter().any(|command| matches!(command,
        PaintCmd::DrawExternalTexture(item) if item.texture_key == 7 && item.content_generation == Some(1))));
    let refusals = Rc::new(RefCell::new(Vec::new()));
    fn record(result: CaptureRefusal, name: &'static str, out: &Rc<RefCell<Vec<&'static str>>>) {
        assert!(matches!(
            result,
            CaptureRefusal::ProducerResource {
                generation: 0,
                error: crate::ProducerGpuRemapError::MissingProducer { local_key: 7 }
            }
        ));
        out.borrow_mut().push(name);
    }
    let out = refusals.clone();
    host.s.pending_capture = Some(Box::new(move |result| {
        record(result.err().unwrap(), "image", &out)
    }));
    let out = refusals.clone();
    host.s.pending_stamped_capture = Some(Box::new(move |result| {
        record(result.err().unwrap(), "stamped", &out)
    }));
    let out = refusals.clone();
    host.s.pending_paint_capture = Some(Box::new(move |result| {
        record(result.err().unwrap(), "paint", &out)
    }));
    host.s.pending_presentation_observer = Some(Box::new(|_, _| {
        panic!("refused frame must never be presented")
    }));
    assert!(host.emit_native_scene(8.0, 6.0).unwrap().is_none());
    assert!(host.emit_native_scene(8.0, 6.0).unwrap().is_none());
    assert_eq!(*refusals.borrow(), ["image", "stamped", "paint"]);
    assert!(host.s.pending_presentation_observer.is_none());
    assert!(host.s.last_redraw_presentation.is_none());
    assert_eq!(
        host.s.runner.as_ref().unwrap().state().draft,
        "application-owned draft"
    );
    assert_eq!(
        host.s.shared.producers.gpu_remap_failure(),
        Some(crate::ProducerGpuRemapError::MissingProducer { local_key: 7 })
    );
    let after = host.emit_paint_list(8.0, 6.0, None).unwrap();
    assert_eq!(
        format!("{:?}", after.commands()),
        format!("{:?}", list.commands())
    );
}
