// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

#[test]
fn gpu_host_timestamps_and_visibility_reach_the_producer() {
    use crate::{Host, HostHooks, HostOptions, HostState, HostWake, HostWindow};
    use std::{cell::Cell, sync::Arc, time::Duration};

    struct Window(Rc<Cell<usize>>);
    impl HostWindow for Window {
        fn request_redraw(&self) {
            self.0.set(self.0.get() + 1);
        }
        fn inner_size(&self) -> (u32, u32) {
            (160, 120)
        }
        fn scale_factor(&self) -> f64 {
            1.0
        }
        fn set_ime_allowed(&self, _: bool) {}
        fn set_ime_cursor_area(&self, _: f64, _: f64, _: f64, _: f64) {}
    }

    let requests = Rc::new(Cell::new(0));
    let mut state: HostState<_, _, _> = HostState::new();
    state.shared.sheet = SHEET.into();
    state.set_resources(vec![], vec![]);
    state.window = Some(Box::new(Window(requests.clone())));
    state.surface = Some(Box::new(surface()));
    state.runner = Some(crate::Runner::new(
        Rc::new(RefCell::new(ScriptedDom::new())),
        |_: &()| cambium::custom_leaf::<(), ()>(7, 8, 6),
        (),
    ));
    let producer = Rc::new(RefCell::new(Producer::default()));
    state
        .shared
        .producers
        .register(7, producer.clone(), &["color"])
        .unwrap();
    let wake = HostWake::new(state.wake_pending.clone(), Arc::new(|| {}));
    let hooks = HostHooks {
        frame: Box::new(|_| false),
        after_dispatch: Box::new(|_| {}),
        after_frame: Box::new(|_| {}),
        prepare_presentation: Box::new(|_, _| None),
        after_wake: Box::new(|_| {}),
        close_request: Box::new(|_, _| crate::CloseDisposition::Exit),
        focused_text: Box::new(|_| None),
        key_intercept: Box::new(|_, _| false),
    };
    let mut host = Host::new(HostOptions::default(), None, hooks, state, wake);
    host.redraw_at(Duration::from_millis(125));
    assert_eq!(
        producer.borrow().frames.last().unwrap().timestamp,
        Some(Duration::from_millis(125))
    );
    host.redraw();
    assert_eq!(producer.borrow().frames.last().unwrap().timestamp, None);
    host.set_hidden(true);
    host.set_hidden(true);
    assert_eq!(producer.borrow().suspended, 1);
    let before = requests.get();
    host.set_hidden(false);
    assert_eq!(requests.get(), before + 1);
    host.redraw_at(Duration::from_secs(5));
    assert!(producer.borrow().frames.last().unwrap().needs_frame);
}

#[test]
fn gpu_producer_refresh_resize_suspend_remove_and_recreate_use_fresh_images() {
    let surface = surface();
    let (mut dom, mut layout, node) = fixture(SHEET);
    let producer = Rc::new(RefCell::new(Producer::default()));
    let mut registry = ProducerRegistry::new();
    registry.register(7, producer.clone(), &["color"]).unwrap();
    registry.timestamp = Some(std::time::Duration::from_millis(125));
    assert_eq!(registry.prepare(&surface, &layout, &dom, 1.0).stages, 1);
    assert_eq!(
        producer.borrow().frames.last().unwrap().timestamp,
        registry.timestamp
    );
    registry.timestamp = None;
    assert_eq!(
        pixel(&surface, &registry, &dom, &mut layout),
        [64, 128, 192, 255]
    );
    assert_eq!(registry.prepare(&surface, &layout, &dom, 1.0).stages, 0);
    assert!(!producer.borrow().frames.last().unwrap().needs_frame);
    let generation = producer.borrow().generation;
    producer.borrow_mut().replace_view = true;
    producer.borrow_mut().pixel_override = Some([200, 32, 80, 255]);
    assert_eq!(registry.prepare(&surface, &layout, &dom, 1.0).stages, 1);
    assert_eq!(
        producer.borrow().generation,
        generation,
        "source identity protects a reused generation"
    );
    assert_eq!(
        pixel(&surface, &registry, &dom, &mut layout),
        [200, 32, 80, 255],
        "a reused generation still invalidates the document's old pixels"
    );

    let decoration = SHEET.replace(
        "color:rgb",
        "box-shadow:1px 1px 2px black;border-color:red;color:rgb",
    );
    layout = OwnedLayout::new(&dom, &[&decoration], 160.0, 120.0, &[], &Default::default());
    assert_eq!(registry.prepare(&surface, &layout, &dom, 1.0).stages, 0);
    producer.borrow_mut().pixel_override = None;
    layout = OwnedLayout::new(
        &dom,
        &[&SHEET.replace("64,128,192", "0,255,0")],
        160.0,
        120.0,
        &[],
        &Default::default(),
    );
    assert_eq!(registry.prepare(&surface, &layout, &dom, 1.0).stages, 1);
    assert_eq!(
        pixel(&surface, &registry, &dom, &mut layout),
        [0, 255, 0, 255]
    );

    assert_eq!(registry.prepare(&surface, &layout, &dom, 1.5).stages, 1);
    assert_eq!(
        producer.borrow().frames.last().unwrap().physical_size,
        [12, 9]
    );
    assert!(producer.borrow().frames.last().unwrap().needs_frame);
    layout = OwnedLayout::new(
        &dom,
        &["custom-leaf { display:none; }"],
        160.0,
        120.0,
        &[],
        &Default::default(),
    );
    assert_eq!(
        registry.prepare(&surface, &layout, &dom, 1.0).suspensions,
        1
    );
    assert_eq!(
        registry.prepare(&surface, &layout, &dom, 1.0).render_calls,
        0
    );
    assert_eq!(producer.borrow().suspended, 1);
    assert!(registry.contains(7));
    assert!(registry.commands(7).unwrap().is_empty());
    layout = OwnedLayout::new(&dom, &[SHEET], 160.0, 120.0, &[], &Default::default());
    assert_eq!(registry.prepare(&surface, &layout, &dom, 1.0).stages, 1);
    registry.suspend_all(Some(surface.core()));
    assert!(registry.commands(7).unwrap().is_empty());
    assert_eq!(registry.prepare(&surface, &layout, &dom, 1.0).stages, 1);

    let removed_renderer_key = registry.image_identity(7).unwrap().renderer_key;
    dom.remove(node);
    layout = OwnedLayout::new(&dom, &[SHEET], 160.0, 120.0, &[], &Default::default());
    assert_eq!(
        registry.prepare(&surface, &layout, &dom, 1.0).retirements,
        1
    );
    assert!(!registry.contains(7));
    assert_eq!(producer.borrow().retired, 1);
    assert!(
        !surface
            .renderer()
            .unregister_external_image(removed_renderer_key)
    );
    let new_node = dom.create_element(name("custom-leaf"));
    dom.set_attribute(new_node, name("key"), "7");
    dom.append_child(dom.document(), new_node);
    layout = OwnedLayout::new(&dom, &[SHEET], 160.0, 120.0, &[], &Default::default());
    registry.register(7, producer.clone(), &["color"]).unwrap();
    assert_eq!(registry.prepare(&surface, &layout, &dom, 1.0).stages, 1);
    assert_eq!(
        pixel(&surface, &registry, &dom, &mut layout),
        [64, 128, 192, 255]
    );
    registry.remove(7);
    assert_eq!(
        registry.prepare(&surface, &layout, &dom, 1.0).retirements,
        1
    );
    assert_eq!(producer.borrow().retired, 2);
}

#[test]
fn gpu_unsupported_encoding_and_duplicate_dom_key_refuse_stale_pixels() {
    let surface = surface();
    let (mut dom, mut layout, _) = fixture(SHEET);
    let producer = Rc::new(RefCell::new(Producer::default()));
    let mut registry = ProducerRegistry::new();
    registry.register(7, producer.clone(), &["color"]).unwrap();
    registry.prepare(&surface, &layout, &dom, 1.0);
    producer.borrow_mut().linear = true;
    assert_eq!(
        registry
            .prepare(&surface, &layout, &dom, 1.0)
            .invalid_frames,
        1
    );
    assert_eq!(registry.error(7), Some(ProducerError::UnsupportedEncoding));
    assert!(registry.commands(7).unwrap().is_empty());
    producer.borrow_mut().linear = false;
    assert_eq!(registry.prepare(&surface, &layout, &dom, 1.0).stages, 1);
    let duplicate = dom.create_element(name("custom-leaf"));
    dom.set_attribute(duplicate, name("key"), "7");
    dom.append_child(dom.document(), duplicate);
    layout = OwnedLayout::new(&dom, &[SHEET], 160.0, 120.0, &[], &Default::default());
    let stats = registry.prepare(&surface, &layout, &dom, 1.0);
    assert_eq!((stats.render_calls, stats.invalid_frames), (0, 1));
    assert_eq!(registry.error(7), Some(ProducerError::DuplicateDomKey));
    assert!(registry.commands(7).unwrap().is_empty());
}
