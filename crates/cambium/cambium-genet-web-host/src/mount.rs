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

use crate::MountLifetime;
use crate::a11y::DomAccessibility;
use crate::events::install_listeners;
use crate::files::WebFileChooser;
use crate::presentation::boot::{WebGpuFactory, default_factory};
use crate::recovery::{self, WakeBridge};
use crate::surface::{WebSurface, WebWindow};

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
    lifetime: MountLifetime,
}

impl<State, Logic, V> Mounted<State, Logic, V>
where
    State: 'static,
    Logic: FnMut(&State) -> V + 'static,
    V: RootView<State>,
{
    /// The host, for a caller that wants to drive it directly.
    pub fn host(&self) -> &Rc<RefCell<Host<State, Logic, V>>> {
        &self.host
    }

    /// Explicit lifetime authority. Dropping Mounted still leaves it running.
    pub fn lifetime(&self) -> &MountLifetime {
        &self.lifetime
    }
    /// Request idempotent disposal. Observe lifetime.status() for completion.
    pub fn dispose(&self) -> bool {
        self.lifetime.dispose()
    }
    /// Disposal has been requested; callbacks and frames no longer run.
    pub fn is_disposed(&self) -> bool {
        !self.lifetime.admitted()
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
    mount_with_lifetime(canvas, options, MountLifetime::new(), init, hooks)
}

/// Mount an independent application onto another canvas on the same device.
/// Application, DOM, resources and producer registry remain per mount; the
/// existing core's recovery owner controls replacement for all its mounts.
pub fn mount_with_render_core<State, Logic, V>(
    canvas: HtmlCanvasElement,
    options: HostOptions,
    core: std::sync::Arc<genet_render_host::RenderCore>,
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
    mount_with_render_core_and_lifetime(canvas, options, core, MountLifetime::new(), init, hooks)
}

/// Mount with explicit startup cancellation and active disposal ownership.
/// Create the lifetime before launching this future. Cancellation drops Rust
/// acquisition state; JavaScript GPU requests themselves cannot be cancelled.
pub fn mount_with_lifetime<State, Logic, V>(
    canvas: HtmlCanvasElement,
    options: HostOptions,
    lifetime: MountLifetime,
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
    Box::pin(async move {
        lifetime
            .startup(mount_inner(
                canvas,
                options,
                None,
                default_factory(),
                init,
                hooks,
                lifetime.clone(),
            ))
            .await?
    })
}

/// Explicit lifetime for an independent mount on an existing shared core.
pub fn mount_with_render_core_and_lifetime<State, Logic, V>(
    canvas: HtmlCanvasElement,
    options: HostOptions,
    core: std::sync::Arc<genet_render_host::RenderCore>,
    lifetime: MountLifetime,
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
    mount_with_core_future_and_lifetime(
        canvas,
        options,
        lifetime,
        async move { Ok(core) },
        init,
        hooks,
    )
}

/// Supply an actual core acquisition future through the same admission and
/// activation path as ordinary mounting. A caller may impose its own finite
/// acquisition deadline or hold a real candidate for lifecycle qualification.
/// Disposal drops this future before init, listeners or drawing can activate.
pub fn mount_with_core_future_and_lifetime<State, Logic, V>(
    canvas: HtmlCanvasElement,
    options: HostOptions,
    lifetime: MountLifetime,
    acquire: impl std::future::Future<
        Output = Result<std::sync::Arc<genet_render_host::RenderCore>, String>,
    > + 'static,
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
    Box::pin(async move {
        lifetime
            .startup(mount_inner(
                canvas,
                options,
                Some(Box::pin(acquire)),
                default_factory(),
                init,
                hooks,
                lifetime.clone(),
            ))
            .await?
    })
}

/// Mount with a caller-supplied WebGPU boot attempt. Backend policy still
/// controls attempts, and application initialization follows selection once.
/// The lifetime cancels startup and owns disposal as for `mount_with_lifetime`.
pub fn mount_with_webgpu_factory<State, Logic, V>(
    canvas: HtmlCanvasElement,
    options: HostOptions,
    lifetime: MountLifetime,
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
    Box::pin(async move {
        lifetime
            .startup(mount_inner(
                canvas,
                options,
                None,
                factory,
                init,
                hooks,
                lifetime.clone(),
            ))
            .await?
    })
}

async fn mount_inner<State, Logic, V>(
    canvas: HtmlCanvasElement,
    options: HostOptions,
    acquire: Option<
        std::pin::Pin<
            Box<
                dyn std::future::Future<
                        Output = Result<std::sync::Arc<genet_render_host::RenderCore>, String>,
                    >,
            >,
        >,
    >,
    mut factory: WebGpuFactory,
    init: impl FnOnce(
        &dyn HostWindow,
        &cambium_rootstock::WindowCommands,
        &HostWake,
    ) -> Init<State, Logic>,
    hooks: HostHooks<State, Logic, V>,
    lifetime: MountLifetime,
) -> Result<Mounted<State, Logic, V>, String>
where
    State: 'static,
    Logic: FnMut(&State) -> V + 'static,
    V: RootView<State>,
{
    lifetime.check()?;
    // A caller-supplied core is a shared WebGPU device: it presents through
    // the host surface and the group recovery owner, never a private driver.
    // Otherwise backend policy selects and boots a private presentation route.
    let (canvas, surface, driver, core) = match acquire {
        Some(acquire) => {
            let window = WebWindow::new(canvas.clone());
            let (width, height) = window.physical_size();
            // The backing store is sized in physical pixels so a fragment lands
            // on a device pixel, the contract a desktop surface is configured under.
            canvas.set_width(width.max(1));
            canvas.set_height(height.max(1));
            let core = acquire.await?;
            // Caller-provided futures/cores can share a device and must never
            // be destroyed here.
            lifetime.check()?;
            let surface = WebSurface::from_core(canvas.clone(), core, width, height)?;
            (canvas, Some(surface), None, None)
        },
        None => {
            let mut selected = crate::presentation::boot::select(
                canvas,
                options.presentation_choice,
                &*options.netrender,
                &mut factory,
                options.wgpu_portable_scene,
            )
            .await?;
            // A selected route is a fresh private device or context, not yet
            // lent to a host. Only that ownership permits explicit destruction.
            let refused = lifetime.check().err().or_else(|| {
                (!selected.canvas.is_connected()).then(|| "appearance boot cancelled".into())
            });
            if let Some(error) = refused {
                if let Some(driver) = selected.driver.as_mut() {
                    driver.dispose();
                }
                if let Some(core) = selected.core.take() {
                    core.device().destroy();
                }
                return Err(error);
            }
            (selected.canvas, None, selected.driver, selected.core)
        },
    };
    let window = WebWindow::new(canvas.clone());
    let label = options.title.clone();
    // The wake flag the host reads, and the signal that gets a frame drawn.
    // Both are atomics: the host's wake handle is `Send + Sync` because a
    // desktop worker thread may wake the UI, and that bound holds regardless
    // of how many threads a browser has.
    let wake_pending = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let bridge = WakeBridge::new();
    let wake = HostWake::new(wake_pending.clone(), bridge.signal());

    let mut s = HostState::new();
    s.wake_pending = wake_pending;
    let Init {
        state,
        logic,
        sheet,
        fonts,
        images,
    } = init(&window, &s.commands.clone(), &wake);
    lifetime.check()?;
    let dom = Rc::new(RefCell::new(ScriptedDom::new()));

    s.shared.sheet = sheet;
    s.set_resources(fonts, images);
    s.runner = Some(Runner::new(dom, logic, state));
    let a11y = DomAccessibility::new(canvas.clone(), label)?;
    let mirror = a11y.handle();
    let cleanup_mirror = mirror.root.clone();
    lifetime.on_dispose(move || cleanup_mirror.remove());
    s.a11y = Some(Box::new(a11y));
    s.files = Some(Box::new(WebFileChooser::new_with_lifetime(
        &canvas, &lifetime,
    )?));
    s.window = Some(Box::new(window.clone()));
    if let Some(surface) = surface {
        s.shared.render_core = Some(surface.shared_core());
        s.surface = Some(Box::new(surface));
    }
    if let Some(core) = core {
        s.shared.render_core = Some(core);
    }
    s.presentation_driver = driver;

    let host = Rc::new(RefCell::new(Host::new(options, None, hooks, s, wake)));
    let cleanup_host = host.clone();
    let cleanup_mirror = mirror.root.clone();
    lifetime.on_dispose(move || {
        let (guard, a11y, files, window) = {
            let mut host = cleanup_host.borrow_mut();
            (
                host.take_disposal(),
                host.s.a11y.take(),
                host.s.files.take(),
                host.s.window.take(),
            )
        };
        drop(guard); // producer retirement + capture refusals outside RefMut
        cleanup_mirror.remove();
        drop((a11y, files, window));
    });
    let cleanup_bridge = bridge.clone();
    lifetime.on_dispose(move || drop(cleanup_bridge));
    bridge.attach(&host, &window, &lifetime);
    recovery::drain(&host, &window);

    // A direct WebGL2 context reports loss and restoration on its canvas. Loss
    // invalidates owned captures at once; the frame loop suspends GPU work and
    // rebuilds at a new context generation after restoration.
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
        let current = generation.clone();
        let lost_lifetime = lifetime.clone();
        let lost = Closure::<dyn FnMut(web_sys::Event)>::new(move |event: web_sys::Event| {
            if !lost_lifetime.admitted() {
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
        let callback = lost.as_ref().clone();
        lifetime.listen(canvas.as_ref(), "webglcontextlost", &callback, move || {
            drop(lost)
        })?;
        let pending = restore_pending.clone();
        let restored_lifetime = lifetime.clone();
        let restored = Closure::<dyn FnMut(web_sys::Event)>::new(move |_| {
            if restored_lifetime.admitted() {
                pending.set(true);
            }
        });
        let callback = restored.as_ref().clone();
        lifetime.listen(
            canvas.as_ref(),
            "webglcontextrestored",
            &callback,
            move || drop(restored),
        )?;
    }

    let document = web_sys::window()
        .and_then(|w| w.document())
        .ok_or("no document")?;
    host.borrow_mut().set_hidden(document.hidden());
    let visibility_host = host.clone();
    let visibility_document = document.clone();
    let visibility_window = window.clone();
    let visibility_bridge = bridge.clone();
    let visibility_lifetime = lifetime.clone();
    let visibility = Closure::<dyn FnMut(web_sys::Event)>::new(move |_| {
        if !visibility_lifetime.admitted() {
            return;
        }
        let _keep_bridge = &visibility_bridge;
        visibility_host
            .borrow_mut()
            .set_hidden(visibility_document.hidden());
        recovery::drain(&visibility_host, &visibility_window);
    });
    let callback = visibility.as_ref().clone();
    lifetime.listen(
        document.as_ref(),
        "visibilitychange",
        &callback,
        move || drop(visibility),
    )?;

    // Visible mounts paint immediately. Hidden mounts still lay out and publish
    // their accessibility mirror, without advancing texture producers.
    lifetime.check()?;
    draw(&mut host.borrow_mut());
    lifetime.check()?;

    install_listeners(&canvas, &mirror, &host, &window, &lifetime)?;
    schedule_frames(
        &host,
        &window,
        bridge,
        &lifetime,
        lost_pending,
        restore_pending,
        generation,
    )?;
    lifetime.activate()?;

    Ok(Mounted {
        host,
        window,
        lifetime,
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
    bridge: Rc<WakeBridge>,
    lifetime: &MountLifetime,
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
    let frame: Rc<RefCell<Option<Closure<dyn FnMut(f64)>>>> = Rc::new(RefCell::new(None));
    let next = frame.clone();
    let request = Rc::new(std::cell::Cell::new(None::<i32>));
    let cleanup_request = request.clone();
    let cleanup_frame = frame.clone();
    let cleanup_browser = browser.clone();
    let cleanup_lifetime = lifetime.clone();
    lifetime.on_dispose(move || {
        if let Some(id) = cleanup_request.take() {
            let _ = cleanup_browser.cancel_animation_frame(id);
        }
        cleanup_lifetime.raf_pending(false);
        drop(cleanup_frame.borrow_mut().take());
    });
    let initial_request = request.clone();
    let frame_lifetime = lifetime.clone();
    let host = host.clone();
    let window = window.clone();
    let mut last_size = window.physical_size();
    let mut attempts = 0u64;

    *next.borrow_mut() = Some(Closure::new(move |timestamp_ms: f64| {
        request.set(None);
        frame_lifetime.raf_pending(false);
        if !frame_lifetime.admitted() {
            return;
        }
        // Keep the wake bridge alive for the same listener/RAF lifetime as
        // the mounted application, even after Mounted itself is dropped.
        let _keep_bridge = &bridge;
        recovery::drain(&host, &window);
        if let Ok(mut host) = host.try_borrow_mut() {
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
            // A CSS-driven resize has no event of its own that also covers zoom
            // and device-pixel-ratio changes, so the frame loop watches the one
            // number that captures all three. A zero-size canvas defers.
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
                    // The WebGL2 session records its complete frame before it
                    // resizes the canvas it owns.
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
                        &host
                            .s
                            .last_presentation_failure
                            .as_ref()
                            .map_or_else(|| "None".to_owned(), ToString::to_string),
                    );
                    let _ = window.canvas().set_attribute(
                        "data-last-producer-profile",
                        &format!("{:?}", host.frame_profile().map(|p| p.producers)),
                    );
                }
            }
        }
        if let Some(browser) = web_sys::window()
            && let Some(callback) = frame.borrow().as_ref()
        {
            if frame_lifetime.admitted() {
                if let Ok(id) = browser.request_animation_frame(callback.as_ref().unchecked_ref()) {
                    request.set(Some(id));
                    frame_lifetime.raf_pending(true);
                }
            }
        }
    }));

    let callback = next.borrow();
    let callback = callback.as_ref().ok_or("frame closure")?;
    let id = browser
        .request_animation_frame(callback.as_ref().unchecked_ref())
        .map_err(|_| "could not schedule frames".to_string())?;
    initial_request.set(Some(id));
    lifetime.raf_pending(true);
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
