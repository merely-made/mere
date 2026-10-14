// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! Owned DOM event routing for the retained host.
use crate::input::{
    CompositionKind, composition_from_dom, key_press_from_dom, wheel_delta_from_dom,
};
use crate::{
    WHEEL_LINE_PX, WHEEL_PAGE_PX, a11y::MirrorHandle, lifetime::Appearance, mount::draw,
    surface::WebWindow,
};
use cambium_rootstock::{A11yAction, A11yRequest, Host, HostWindow, meristem_bounds::RootView};
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::prelude::Closure;
use web_sys::HtmlCanvasElement;
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
pub(crate) fn install_listeners<State, Logic, V>(
    canvas: &HtmlCanvasElement,
    mirror: &MirrorHandle,
    host: &Rc<RefCell<Host<State, Logic, V>>>,
    window: &WebWindow,
    appearance: &Rc<Appearance>,
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
            let alive = appearance.alive.clone();
            #[allow(unused_mut)]
            let mut handler = $handler;
            let closure = Closure::<dyn FnMut($ty)>::new(move |event: $ty| {
                if alive.get() {
                    handler(event);
                }
            });
            appearance.listen($target.as_ref(), $name, closure)?;
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

    let targets: [&web_sys::EventTarget; 2] = [canvas.as_ref(), root.as_ref()];
    for target in targets {
        for (name, kind) in [
            ("compositionstart", CompositionKind::Start),
            ("compositionupdate", CompositionKind::Update),
            ("compositionend", CompositionKind::End),
        ] {
            let h = host.clone();
            let w = window.clone();
            let alive = appearance.alive.clone();
            let closure = Closure::<dyn FnMut(web_sys::CompositionEvent)>::new(
                move |e: web_sys::CompositionEvent| {
                    if !alive.get() {
                        return;
                    }
                    h.borrow_mut()
                        .ime(composition_from_dom(kind, e.data().unwrap_or_default()));
                    w.request_redraw();
                },
            );
            appearance.listen(target, name, closure)?;
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
    let alive = appearance.alive.clone();
    let focus_listener =
        Closure::<dyn Fn(web_sys::FocusEvent)>::new(move |e: web_sys::FocusEvent| {
            // The mirror moving focus to follow the tree is not a reader asking.
            if !alive.get() || m.moving.get() {
                return;
            }
            if let Some(target) = m.request_target_of(e.target()) {
                let action = A11yAction::Focus;
                reader_act(&h, A11yRequest { action, target });
            }
        });
    appearance.listen(root.as_ref(), "focusin", focus_listener)?;

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
