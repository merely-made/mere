// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Headed receipts for the host's one render core (stack seams P2). They open
//! a real window on a real device, so they are ignored by default; winit allows
//! one event loop per process, so run them one at a time:
//!
//! ```text
//! cargo test -p cambium-genet-winit-host --lib headed_tests::one_core -- --ignored --exact
//! cargo test -p cambium-genet-winit-host --lib headed_tests::control -- --ignored --exact
//! ```
//!
//! A desktop event loop never suspends, so the driver calls the host's own
//! `suspended` and `resumed` handlers, which is the code a mobile platform
//! would reach.

use std::cell::RefCell;
use std::rc::Rc;

use cambium::{AnyView, GenetCtx, GenetElement, el, text};

use super::*;

type Child = Box<dyn AnyView<(), (), GenetCtx, GenetElement>>;
type Logic = fn(&()) -> Child;

fn root(_: &()) -> Child {
    Box::new(el("div", text("one render core")))
}

/// What the driver read around one forced suspend and resume.
#[derive(Debug, Default)]
struct Readings {
    boots_before: u32,
    boots_after: u32,
    presents_before: u64,
    presents_after: u64,
    surface_dropped: bool,
    core_kept: bool,
    tenant_device_is_surface_device: bool,
}

struct Driver {
    host: WinitHost<(), Logic, Child>,
    /// The core as a tenant saw it through `AppCtx`, before the suspend.
    tenant: Rc<RefCell<Option<Arc<RenderCore>>>>,
    /// The control: forget the core between suspend and resume, as the host
    /// did before S2, so the instrument has to read a second boot.
    drop_core: bool,
    readings: Rc<RefCell<Option<Readings>>>,
}

impl ApplicationHandler<HostEvent> for Driver {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.host.resumed(event_loop);
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: HostEvent) {
        self.host.user_event(event_loop, event);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        self.host.window_event(event_loop, id, event);
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        self.host.about_to_wait(event_loop);
        if self.readings.borrow().is_some() {
            return;
        }
        // The first resume drew the first frame synchronously, so by the first
        // idle turn the core, a surface and the tenant's handle all exist.
        let before = self.host.s.render_core.clone().expect("a core was booted");
        let mut readings = Readings {
            boots_before: self.host.core_boots,
            presents_before: self.host.s.presentation_sequence,
            ..Readings::default()
        };
        self.host.suspended(event_loop);
        readings.surface_dropped = self.host.s.surface.is_none();
        if self.drop_core {
            self.host.s.render_core = None;
        }
        self.host.resumed(event_loop);
        let after = self.host.s.render_core.clone().expect("a core after resume");
        readings.boots_after = self.host.core_boots;
        readings.presents_after = self.host.s.presentation_sequence;
        readings.core_kept = Arc::ptr_eq(&before, &after);
        // Identity, not `==`: wgpu compares devices by a per-instance id, and
        // every core boot makes its own instance, so two devices from two
        // boots can compare equal. The control caught exactly that.
        readings.tenant_device_is_surface_device = self
            .tenant
            .borrow()
            .as_ref()
            .zip(self.host.s.surface.as_ref())
            .is_some_and(|(tenant, surface)| std::ptr::eq(tenant.device(), surface.device()));
        *self.readings.borrow_mut() = Some(readings);
        event_loop.exit();
    }
}

fn drive(drop_core: bool) -> Readings {
    let mut builder = EventLoop::<HostEvent>::with_user_event();
    #[cfg(target_os = "windows")]
    {
        use winit::platform::windows::EventLoopBuilderExtWindows;
        builder.with_any_thread(true);
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        use winit::platform::x11::EventLoopBuilderExtX11;
        builder.with_any_thread(true);
    }
    let event_loop = builder.build().expect("an event loop");
    let s = HostState::new();
    let proxy = event_loop.create_proxy();
    let wake = HostWake::new(
        s.wake_pending.clone(),
        Arc::new(move || {
            let _ = proxy.send_event(HostEvent::Wake);
        }),
    );
    let tenant = Rc::new(RefCell::new(None));
    let mut hooks: HostHooks<(), Logic, Child> = inert_hooks();
    hooks.frame = Box::new({
        let tenant = tenant.clone();
        move |ctx| {
            if tenant.borrow().is_none() {
                *tenant.borrow_mut() = ctx.render_core.cloned();
            }
            false
        }
    });
    let options = HostOptions {
        title: "cambium one render core".into(),
        ..Default::default()
    };
    let init = |_: &dyn HostWindow, _: &WindowCommands, _: &HostWake| Init {
        state: (),
        logic: root as Logic,
        sheet: String::new(),
        fonts: Vec::new(),
        images: Vec::new(),
    };
    let readings = Rc::new(RefCell::new(None));
    let mut driver = Driver {
        host: WinitHost::new(Host::new(options, Some(Box::new(init)), hooks, s, wake)),
        tenant,
        drop_core,
        readings: readings.clone(),
    };
    event_loop.run_app(&mut driver).expect("the event loop ran");
    let readings = readings.borrow_mut().take().expect("the driver read the host");
    eprintln!("[headed] drop_core={drop_core} {readings:?}");
    readings
}

/// One device for the host's life: a suspend drops the surface, the resume
/// makes a new one from the same core and presents through it, and a tenant
/// handed the core before the suspend shares the new surface's device.
#[test]
#[ignore = "headed: opens a window on a GPU"]
#[cfg(not(target_os = "macos"))]
fn one_core() {
    let r = drive(false);
    assert!(r.surface_dropped, "the suspend dropped the surface");
    assert_eq!(r.boots_before, 1, "one core booted for the first frame");
    assert_eq!(r.boots_after, 1, "the resume booted no core");
    assert!(r.core_kept, "the same core before and after");
    assert!(r.presents_after > r.presents_before, "the resumed surface presented");
    assert!(r.tenant_device_is_surface_device, "the tenant shares the device");
}

/// The control: a host that forgets its core across the suspend must read a
/// second boot and a tenant left on the old device, or the receipt above
/// proves nothing.
#[test]
#[ignore = "headed: opens a window on a GPU"]
#[cfg(not(target_os = "macos"))]
fn control() {
    let r = drive(true);
    assert_eq!(r.boots_after, 2, "forgetting the core boots another");
    assert!(!r.core_kept);
    assert!(!r.tenant_device_is_surface_device, "the tenant is on the old device");
    assert!(r.presents_after > r.presents_before);
}
