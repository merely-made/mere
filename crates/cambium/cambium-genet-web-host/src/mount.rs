// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Putting an application on a canvas, and keeping it fed.
//!
//! The browser's answer to the winit source's `run`. It differs in shape for
//! one reason worth stating: `run` owns the thread until the window closes,
//! and nothing may own a browser's thread. So this returns as soon as the
//! application is mounted, and the listeners and frame callback it installed
//! keep it alive.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use cambium_rootstock::{
    Host, HostHooks, HostOptions, HostState, HostWake, HostWindow, Init, Runner, ScriptedDom,
    meristem_bounds::RootView,
};
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::Closure;
use web_sys::HtmlCanvasElement;

use crate::a11y::DomAccessibility;
use crate::files::WebFileChooser;

use crate::events::install_listeners;
use crate::lifetime::Appearance;
use crate::presentation::boot::{WebGpuFactory, default_factory};
use crate::surface::WebWindow;

/// A mounted application, kept alive by the closures it installed.
///
/// Dropping this does not stop it: the listeners hold their own references,
/// which is how a browser application stays running with nothing on the stack.
pub struct Mounted<State: 'static, Logic, V>
where
    Logic: FnMut(&State) -> V,
    V: RootView<State>,
{
    host: Rc<RefCell<Host<State, Logic, V>>>,
    window: WebWindow,
    appearance: Rc<Appearance>,
}

impl<State, Logic, V> Mounted<State, Logic, V>
where
    State: 'static,
    Logic: FnMut(&State) -> V + 'static,
    V: RootView<State>,
{
    /// Stop this appearance and cancel its owned graphics/DOM work.
    pub fn dispose(&self) {
        self.appearance.dispose();
    }
    pub fn is_disposed(&self) -> bool {
        !self.appearance.alive.get()
    }

    /// The host, for a caller that wants to drive it directly.
    pub fn host(&self) -> &Rc<RefCell<Host<State, Logic, V>>> {
        &self.host
    }

    /// The canvas frame.
    pub fn window(&self) -> &WebWindow {
        &self.window
    }
}

/// Mount an application onto a canvas.
///
/// Mirrors what the winit source does on `resumed`: boot a surface, run the
/// application's init against the frame it will present on, build the runner,
/// attach accessibility, then draw once so the first frame exists before any
/// event arrives.
#[inline(never)]
pub fn mount<State, Logic, V>(
    canvas: HtmlCanvasElement,
    options: HostOptions,
    init: impl FnOnce(
        &dyn HostWindow,
        &cambium_rootstock::WindowCommands,
        &HostWake,
    ) -> Init<State, Logic>,
    hooks: HostHooks<State, Logic, V>,
) -> std::pin::Pin<Box<impl std::future::Future<Output = Result<Mounted<State, Logic, V>, String>>>>
where
    State: 'static,
    Logic: FnMut(&State) -> V + 'static,
    V: RootView<State>,
{
    Box::pin(mount_inner(canvas, options, init, hooks, default_factory()))
}

/// Mount with a caller-supplied WebGPU boot attempt. Backend policy still
/// controls attempts, and application initialization follows selection once.
pub fn mount_with_webgpu_factory<State, Logic, V>(
    canvas: HtmlCanvasElement,
    options: HostOptions,
    init: impl FnOnce(
        &dyn HostWindow,
        &cambium_rootstock::WindowCommands,
        &HostWake,
    ) -> Init<State, Logic>,
    hooks: HostHooks<State, Logic, V>,
    factory: WebGpuFactory,
) -> std::pin::Pin<Box<impl std::future::Future<Output = Result<Mounted<State, Logic, V>, String>>>>
where
    State: 'static,
    Logic: FnMut(&State) -> V + 'static,
    V: RootView<State>,
{
    Box::pin(mount_inner(canvas, options, init, hooks, factory))
}

async fn mount_inner<State, Logic, V>(
    canvas: HtmlCanvasElement,
    options: HostOptions,
    init: impl FnOnce(
        &dyn HostWindow,
        &cambium_rootstock::WindowCommands,
        &HostWake,
    ) -> Init<State, Logic>,
    hooks: HostHooks<State, Logic, V>,
    mut factory: WebGpuFactory,
) -> Result<Mounted<State, Logic, V>, String>
where
    State: 'static,
    Logic: FnMut(&State) -> V + 'static,
    V: RootView<State>,
{
    let mut selected = crate::presentation::boot::select(
        canvas,
        options.presentation_choice,
        &*options.netrender,
        &mut factory,
    )
    .await?;
    if !selected.canvas.is_connected() {
        if let Some(driver) = selected.driver.as_mut() {
            driver.dispose();
        }
        return Err("appearance boot cancelled".into());
    }
    let canvas = selected.canvas;
    let window = WebWindow::new(canvas.clone());
    let label = options.title.clone();
    // The wake flag the host reads, and the signal that gets a frame drawn.
    // Both are atomics: the host's wake handle is `Send + Sync` because a
    // desktop worker thread may wake the UI, and that bound holds regardless
    // of how many threads a browser has.
    let wake_pending = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let wake = HostWake::new(wake_pending.clone(), {
        let pending = window.pending_frame_flag();
        std::sync::Arc::new(move || pending.store(true, std::sync::atomic::Ordering::Relaxed))
    });

    let mut s = HostState::new();
    let appearance = Appearance::new(s.capture_owner.scope());
    s.wake_pending = wake_pending;
    let Init {
        state,
        logic,
        sheet,
        fonts,
        images,
    } = init(&window, &s.commands.clone(), &wake);
    let dom = Rc::new(RefCell::new(ScriptedDom::new()));

    s.shared.sheet = sheet;
    s.set_resources(fonts, images);
    s.runner = Some(Runner::new(dom, logic, state));
    let a11y = DomAccessibility::new(canvas.clone(), label)?;
    let mirror = a11y.handle();
    appearance.own_node(mirror.root.clone().unchecked_into());
    s.a11y = Some(Box::new(a11y));
    let chooser = WebFileChooser::new_scoped(&canvas, appearance.alive.clone())?;
    appearance.own_node(chooser.input().clone().unchecked_into());
    s.files = Some(Box::new(chooser));
    s.window = Some(Box::new(window.clone()));
    if let Some(surface) = selected.surface {
        s.shared.render_core = Some(surface.shared_core());
        s.surface = Some(Box::new(surface));
    }
    if let Some(core) = selected.core {
        s.shared.render_core = Some(core);
    }
    s.presentation_driver = selected.driver;

    let host = Rc::new(RefCell::new(Host::new(options, None, hooks, s, wake)));
    let restore_pending = Rc::new(Cell::new(false));
    let lost_pending = Rc::new(Cell::new(false));
    let generation = Rc::new(Cell::new(
        host.borrow()
            .s
            .presentation_driver
            .as_ref()
            .map_or(cambium_rootstock::ContextGeneration(0), |driver| {
                driver.generation()
            }),
    ));
    if host
        .borrow()
        .s
        .presentation_driver
        .as_ref()
        .is_some_and(|driver| driver.backend() == cambium_rootstock::PresentationBackend::WebGl2)
    {
        let captures = host.borrow().s.capture_owner.scope();
        let pending = lost_pending.clone();
        let restore = restore_pending.clone();
        let alive = appearance.alive.clone();
        let current = generation.clone();
        let lost = Closure::<dyn FnMut(web_sys::Event)>::new(move |event: web_sys::Event| {
            if !alive.get() {
                return;
            }
            event.prevent_default();
            restore.set(false);
            pending.set(true);
            captures.invalidate(cambium_rootstock::CaptureRefusal::ContextLost {
                backend: cambium_rootstock::PresentationBackend::WebGl2,
                generation: current.get(),
            });
        });
        appearance.listen(canvas.as_ref(), "webglcontextlost", lost)?;
        let alive = appearance.alive.clone();
        let pending = restore_pending.clone();
        let restored = Closure::<dyn FnMut(web_sys::Event)>::new(move |_| {
            if alive.get() {
                pending.set(true);
            }
        });
        appearance.listen(canvas.as_ref(), "webglcontextrestored", restored)?;
    }
    let cleanup_host = host.clone();
    appearance.cleanup_with(move || {
        let Ok(mut host) = cleanup_host.try_borrow_mut() else {
            return false;
        };
        host.dispose_presentation();
        true
    });

    let document = web_sys::window()
        .and_then(|w| w.document())
        .ok_or("no document")?;
    host.borrow_mut().set_hidden(document.hidden());
    let visibility_host = host.clone();
    let visibility_document = document.clone();
    let alive = appearance.alive.clone();
    let visibility = Closure::<dyn FnMut(web_sys::Event)>::new(move |_| {
        if !alive.get() {
            return;
        }
        visibility_host
            .borrow_mut()
            .set_hidden(visibility_document.hidden());
    });
    appearance.listen(document.as_ref(), "visibilitychange", visibility)?;

    // Visible mounts paint immediately. Hidden mounts still lay out and publish
    // their accessibility mirror, without advancing texture producers.
    draw(&mut host.borrow_mut());

    if let Err(error) =
        install_listeners(&canvas, &mirror, &host, &window, &appearance).and_then(|_| {
            schedule_frames(
                &host,
                &window,
                &appearance,
                lost_pending,
                restore_pending,
                generation,
            )
        })
    {
        appearance.dispose();
        return Err(error);
    }
    Ok(Mounted {
        host,
        window,
        appearance,
    })
}

/// The frame loop.
///
/// `requestAnimationFrame` unconditionally, but a frame is only *drawn* when
/// one was asked for. That keeps a static application at zero GPU work while
/// still noticing a resize, which the desktop gets from the window manager and
/// the browser must observe for itself.
fn schedule_frames<State, Logic, V>(
    host: &Rc<RefCell<Host<State, Logic, V>>>,
    window: &WebWindow,
    appearance: &Rc<Appearance>,
    lost_pending: Rc<Cell<bool>>,
    restore_pending: Rc<Cell<bool>>,
    generation: Rc<Cell<cambium_rootstock::ContextGeneration>>,
) -> Result<(), String>
where
    State: 'static,
    Logic: FnMut(&State) -> V + 'static,
    V: RootView<State>,
{
    let browser = web_sys::window().ok_or("no window")?;
    let scope = appearance.clone();
    let h = host.clone();
    let window = window.clone();
    let mut last_size = window.physical_size();
    let mut attempts = 0u64;
    *appearance.frame.borrow_mut() = Some(Closure::new(move |timestamp_ms: f64| {
        scope.frame_id.set(None);
        if !scope.alive.get() {
            return;
        }
        if let Ok(mut host) = h.try_borrow_mut() {
            if lost_pending.replace(false) {
                host.lose_presentation_context();
            }
            if restore_pending.get()
                && let Some(driver) = host.s.presentation_driver.as_mut()
            {
                match driver.restore() {
                    Ok(true) => {
                        generation.set(driver.generation());
                        restore_pending.set(false);
                        window.request_redraw();
                    },
                    Ok(false) => {},
                    Err(error) => {
                        restore_pending.set(false);
                        host.refuse_presentation(error);
                    },
                }
            }
            let size = window.physical_size();
            if size != last_size {
                last_size = size;
                if size.0 > 0
                    && size.1 > 0
                    && let Some(surface) = host.s.surface.as_mut()
                {
                    window.canvas().set_width(size.0);
                    window.canvas().set_height(size.1);
                    surface.resize(size.0, size.1);
                }
                if size.0 > 0
                    && size.1 > 0
                    && let Some(driver) = host.s.presentation_driver.as_mut()
                {
                    if driver.backend() == cambium_rootstock::PresentationBackend::Wgpu {
                        window.canvas().set_width(size.0);
                        window.canvas().set_height(size.1);
                    }
                    if let Err(error) = driver.resize(size.0, size.1) {
                        host.refuse_presentation(error);
                    }
                }
                window.request_redraw();
            }
            if !host.s.hidden && window.take_pending_frame() {
                draw_at(&mut host, timestamp_ms);
                if window
                    .canvas()
                    .has_attribute("data-presentation-diagnostics")
                {
                    attempts = attempts.saturating_add(1);
                    let _ = window
                        .canvas()
                        .set_attribute("data-redraw-attempts", &attempts.to_string());
                    let _ = window.canvas().set_attribute(
                        "data-last-presentation-failure",
                        &format!("{:?}", host.s.last_raster_failure),
                    );
                    let _ = window.canvas().set_attribute(
                        "data-last-producer-profile",
                        &format!("{:?}", host.frame_profile().map(|p| p.producers)),
                    );
                }
            }
        }
        if scope.alive.get()
            && let Some(browser) = web_sys::window()
            && let Some(callback) = scope.frame.borrow().as_ref()
        {
            if let Ok(id) = browser.request_animation_frame(callback.as_ref().unchecked_ref()) {
                scope.frame_id.set(Some(id));
            }
        }
    }));
    let frame = appearance.frame.borrow();
    let callback = frame.as_ref().ok_or("frame closure")?;
    let id = browser
        .request_animation_frame(callback.as_ref().unchecked_ref())
        .map_err(|_| "could not schedule frames".to_string())?;
    appearance.frame_id.set(Some(id));
    Ok(())
}

/// Draw a frame and mirror it for a reader.
pub(crate) fn draw<State, Logic, V>(host: &mut Host<State, Logic, V>)
where
    State: 'static,
    Logic: FnMut(&State) -> V + 'static,
    V: RootView<State>,
{
    let timestamp_ms = web_sys::window()
        .and_then(|w| w.performance())
        .map_or(0.0, |performance| performance.now());
    draw_at(host, timestamp_ms);
}

fn draw_at<State, Logic, V>(host: &mut Host<State, Logic, V>, timestamp_ms: f64)
where
    State: 'static,
    Logic: FnMut(&State) -> V + 'static,
    V: RootView<State>,
{
    if host.s.hidden {
        let (width, height) = host.logical_size();
        host.relayout(width, height);
        host.sync_a11y();
        return;
    }
    let timestamp =
        std::time::Duration::try_from_secs_f64(timestamp_ms / 1000.0).unwrap_or_default();
    host.redraw_at(timestamp);
    host.sync_a11y();
    host.with_ctx(cambium_rootstock::Hook::AfterFrame);
}
