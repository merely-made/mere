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

use std::cell::RefCell;
use std::rc::Rc;

use cambium_rootstock::{
    A11yAction, A11yRequest, Host, HostHooks, HostOptions, HostState, HostWake, HostWindow, Init,
    Runner, ScriptedDom, meristem_bounds::RootView,
};
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::Closure;
use web_sys::HtmlCanvasElement;

use crate::a11y::{DomAccessibility, MirrorHandle};
use crate::files::WebFileChooser;
use crate::input::{
    CompositionKind, composition_from_dom, key_press_from_dom, wheel_delta_from_dom,
};
use crate::recovery::{self, WakeBridge};
use crate::surface::{WebSurface, WebWindow};
use crate::{MountLifetime, WHEEL_LINE_PX, WHEEL_PAGE_PX};

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
    let window = WebWindow::new(canvas.clone());
    let (width, height) = window.physical_size();
    // The backing store is sized in physical pixels so a fragment lands on a
    // device pixel, the same contract a desktop surface is configured under.
    canvas.set_width(width);
    canvas.set_height(height);

    let private_acquisition = acquire.is_none();
    let core = match acquire {
        Some(acquire) => acquire.await?,
        None => std::sync::Arc::new(
            genet_render_host::RenderCore::boot_async((options.netrender)()).await?,
        ),
    };
    // A default acquisition is a fresh private device, not yet lent to a
    // host. Only that ownership permits explicit destruction. Caller-provided
    // futures/cores can share a device and must never be destroyed here.
    if let Err(error) = lifetime.check() {
        if private_acquisition {
            core.renderer().wgpu_device.core.device.destroy();
        }
        drop(core);
        return Err(error);
    }
    let surface = WebSurface::from_core(canvas.clone(), core, width, height)?;
    lifetime.check()?;
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
    s.shared.render_core = Some(surface.shared_core());
    s.surface = Some(Box::new(surface));

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
    schedule_frames(&host, &window, bridge, &lifetime)?;
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
        // A CSS-driven resize has no event of its own that also covers zoom
        // and device-pixel-ratio changes, so the frame loop watches the one
        // number that captures all three.
        let size = window.physical_size();
        if size != last_size {
            last_size = size;
            window.canvas().set_width(size.0);
            window.canvas().set_height(size.1);
            let mut h = host.borrow_mut();
            if let Some(surface) = h.s.surface.as_mut() {
                surface.resize(size.0, size.1);
            }
            let (lw, lh) = h.logical_size();
            h.relayout(lw, lh);
            window.request_redraw();
        }
        if !host.borrow().s.hidden && window.take_pending_frame() {
            draw_at(&mut host.borrow_mut(), timestamp_ms);
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
fn draw<State, Logic, V>(host: &mut Host<State, Logic, V>)
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

/// Carry out what a reader asked of a mirror element, and draw the result
/// before returning, so the reader hears it without waiting on a frame.
fn reader_act<State, Logic, V>(host: &Rc<RefCell<Host<State, Logic, V>>>, request: A11yRequest)
where
    State: 'static,
    Logic: FnMut(&State) -> V + 'static,
    V: RootView<State>,
{
    let mut host = host.borrow_mut();
    host.apply_a11y_requests(&[request]);
    draw(&mut host);
}

/// A key, from the canvas or from a mirror element a reader focused.
fn on_key<State, Logic, V>(
    host: &Rc<RefCell<Host<State, Logic, V>>>,
    window: &WebWindow,
) -> impl FnMut(web_sys::KeyboardEvent) + 'static
where
    State: 'static,
    Logic: FnMut(&State) -> V + 'static,
    V: RootView<State>,
{
    let h = host.clone();
    let w = window.clone();
    move |e: web_sys::KeyboardEvent| {
        let press = key_press_from_dom(&e);
        // Tab moves focus inside the application, not out of it; the arrows
        // and Space scroll the page unless claimed.
        if !press.modifiers.is_command_chord() {
            e.prevent_default();
        }
        h.borrow_mut().key(&press);
        w.request_redraw();
    }
}

/// Attach the DOM listeners that drive the host.
///
/// Every one of these is the same shape: read the event, say what happened in
/// the host's vocabulary, let the host decide. The ordering rules that used to
/// live in each event source are inside `pointer_moved` now, so this cannot
/// get them subtly wrong.
fn install_listeners<State, Logic, V>(
    canvas: &HtmlCanvasElement,
    mirror: &MirrorHandle,
    host: &Rc<RefCell<Host<State, Logic, V>>>,
    window: &WebWindow,
    lifetime: &MountLifetime,
) -> Result<(), String>
where
    State: 'static,
    Logic: FnMut(&State) -> V + 'static,
    V: RootView<State>,
{
    /// Where a pointer event landed, in the layout's own coordinates.
    ///
    /// The browser reports CSS pixels relative to the viewport; the canvas
    /// rect makes them canvas-relative, and `zoom` carries them the rest of
    /// the way into the space the layout was built in. The desktop source
    /// divides physical pixels by the whole layout scale for the same reason —
    /// here the device half is already gone, so only the zoom is left.
    fn local(canvas: &HtmlCanvasElement, x: i32, y: i32, zoom: f32) -> (f32, f32) {
        let rect = canvas.get_bounding_client_rect();
        (
            ((x as f64 - rect.x()) as f32) / zoom,
            ((y as f64 - rect.y()) as f32) / zoom,
        )
    }

    macro_rules! listen {
        ($target:expr, $name:literal, $ty:ty, $handler:expr) => {{
            let listener_lifetime = lifetime.clone();
            let mut handler = $handler;
            let closure = Closure::<dyn FnMut($ty)>::new(move |event| {
                if listener_lifetime.admitted() {
                    handler(event);
                }
            });
            let callback = closure.as_ref().clone();
            lifetime.listen($target.as_ref(), $name, &callback, move || drop(closure))?;
        }};
    }

    let h = host.clone();
    let c = canvas.clone();
    let w = window.clone();
    listen!(
        canvas,
        "pointerdown",
        web_sys::PointerEvent,
        move |e: web_sys::PointerEvent| {
            let mut host = h.borrow_mut();
            host.s.pointer_id = e.pointer_id();
            host.s.modifiers = cambium_rootstock::Modifiers {
                shift: e.shift_key(),
                ctrl: e.ctrl_key(),
                alt: e.alt_key(),
                meta: e.meta_key(),
            };
            let (x, y) = local(&c, e.client_x(), e.client_y(), host.ui_zoom());
            host.pointer_moved(x, y);
            match e.button() {
                0 => host.click(),
                1 => host.pointer_press(cambium::PointerButton::Auxiliary),
                2 => host.secondary_press(),
                _ => return,
            }
            let _ = c.set_pointer_capture(e.pointer_id());
            w.request_redraw();
        }
    );

    let h = host.clone();
    let c = canvas.clone();
    let w = window.clone();
    listen!(
        canvas,
        "pointermove",
        web_sys::PointerEvent,
        move |e: web_sys::PointerEvent| {
            let mut host = h.borrow_mut();
            host.s.pointer_id = e.pointer_id();
            host.s.modifiers = cambium_rootstock::Modifiers {
                shift: e.shift_key(),
                ctrl: e.ctrl_key(),
                alt: e.alt_key(),
                meta: e.meta_key(),
            };
            let (x, y) = local(&c, e.client_x(), e.client_y(), host.ui_zoom());
            host.pointer_moved(x, y);
            w.request_redraw();
        }
    );

    let h = host.clone();
    let c = canvas.clone();
    let w = window.clone();
    listen!(
        canvas,
        "pointerup",
        web_sys::PointerEvent,
        move |e: web_sys::PointerEvent| {
            let mut host = h.borrow_mut();
            host.s.pointer_id = e.pointer_id();
            host.s.modifiers = cambium_rootstock::Modifiers {
                shift: e.shift_key(),
                ctrl: e.ctrl_key(),
                alt: e.alt_key(),
                meta: e.meta_key(),
            };
            let (x, y) = local(&c, e.client_x(), e.client_y(), host.ui_zoom());
            host.pointer_moved(x, y);
            match e.button() {
                0 => host.release(),
                1 => host.release_button(cambium::PointerButton::Auxiliary),
                2 => host.release_button(cambium::PointerButton::Secondary),
                _ => return,
            }
            let _ = c.release_pointer_capture(e.pointer_id());
            w.request_redraw();
        }
    );

    let h = host.clone();
    let w = window.clone();
    listen!(
        canvas,
        "pointercancel",
        web_sys::PointerEvent,
        move |e: web_sys::PointerEvent| {
            let mut host = h.borrow_mut();
            if host
                .s
                .runner
                .as_ref()
                .and_then(|r| r.pointer_capture_owner())
                .is_some_and(|(id, _)| id == e.pointer_id())
            {
                host.cancel_pointer();
                w.request_redraw();
            }
        }
    );

    let h = host.clone();
    let w = window.clone();
    listen!(
        canvas,
        "wheel",
        web_sys::WheelEvent,
        move |e: web_sys::WheelEvent| {
            // The page must not scroll underneath an application that handles the
            // notch itself.
            e.prevent_default();
            let (dx, dy) = wheel_delta_from_dom(&e, WHEEL_LINE_PX, WHEEL_PAGE_PX);
            let mut host = h.borrow_mut();
            host.s.modifiers = cambium_rootstock::Modifiers {
                shift: e.shift_key(),
                ctrl: e.ctrl_key(),
                alt: e.alt_key(),
                meta: e.meta_key(),
            };
            host.wheel(dx, dy);
            w.request_redraw();
        }
    );

    // Keys reach the host from the canvas, or from the mirror element that
    // holds focus for a reader.
    let root = &mirror.root;
    listen!(
        canvas,
        "keydown",
        web_sys::KeyboardEvent,
        on_key(host, window)
    );
    listen!(
        root,
        "keydown",
        web_sys::KeyboardEvent,
        on_key(host, window)
    );

    // A Tab tap must leave held-Tab spatial navigation. Own these listeners
    // in the same phase-admitted lifetime as keydown, including the mirror
    // that receives keyboard focus after accessibility routing.
    let key_targets: [&web_sys::EventTarget; 2] = [canvas.as_ref(), root.as_ref()];
    for target in key_targets {
        let h = host.clone();
        let release_lifetime = lifetime.clone();
        let release =
            Closure::<dyn FnMut(web_sys::KeyboardEvent)>::new(move |e: web_sys::KeyboardEvent| {
                if !release_lifetime.admitted() {
                    return;
                }
                let key = key_press_from_dom(&e).key;
                h.borrow_mut().key_released(&key);
            });
        let callback = release.as_ref().clone();
        lifetime.listen(target, "keyup", &callback, move || drop(release))?;
    }

    let targets: [&web_sys::EventTarget; 2] = [canvas.as_ref(), root.as_ref()];
    for target in targets {
        for (name, kind) in [
            ("compositionstart", CompositionKind::Start),
            ("compositionupdate", CompositionKind::Update),
            ("compositionend", CompositionKind::End),
        ] {
            let h = host.clone();
            let w = window.clone();
            let listener_lifetime = lifetime.clone();
            let closure = Closure::<dyn FnMut(web_sys::CompositionEvent)>::new(
                move |e: web_sys::CompositionEvent| {
                    if !listener_lifetime.admitted() {
                        return;
                    }
                    h.borrow_mut()
                        .ime(composition_from_dom(kind, e.data().unwrap_or_default()));
                    w.request_redraw();
                },
            );
            let callback = closure.as_ref().clone();
            lifetime.listen(target, name, &callback, move || drop(closure))?;
        }
    }

    // A reader's click on a mirror element activates its node, and a
    // reader's focus on one focuses it.
    let h = host.clone();
    let m = mirror.clone();
    listen!(
        root,
        "click",
        web_sys::MouseEvent,
        move |e: web_sys::MouseEvent| {
            if let Some(target) = m.request_target_of(e.target()) {
                let action = A11yAction::Click;
                reader_act(&h, A11yRequest { action, target });
            }
        }
    );
    let h = host.clone();
    let m = mirror.clone();
    // Following host focus can synchronously emit another focusin. The guard
    // below answers that nested event; an FnMut trampoline refuses it before
    // the guard can run.
    let focus_lifetime = lifetime.clone();
    let focus_listener =
        Closure::<dyn Fn(web_sys::FocusEvent)>::new(move |e: web_sys::FocusEvent| {
            if !focus_lifetime.admitted() {
                return;
            }
            // The mirror moving focus to follow the tree is not a reader asking.
            if m.moving.get() {
                return;
            }
            if let Some(target) = m.request_target_of(e.target()) {
                let action = A11yAction::Focus;
                reader_act(&h, A11yRequest { action, target });
            }
        });
    let callback = focus_listener.as_ref().clone();
    lifetime.listen(root.as_ref(), "focusin", &callback, move || {
        drop(focus_listener)
    })?;

    // The canvas takes focus from Tab or a press, and hands it to the mirror,
    // where a reader hears what has it.
    let m = mirror.clone();
    listen!(
        canvas,
        "focus",
        web_sys::FocusEvent,
        move |_: web_sys::FocusEvent| m.take_focus()
    );

    // The canvas has to be focusable to receive keys at all.
    let _ = canvas.set_attribute("tabindex", "0");
    Ok(())
}
