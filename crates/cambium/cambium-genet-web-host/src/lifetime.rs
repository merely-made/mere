// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! DOM and frame callbacks owned by one appearance.
use std::{
    any::Any,
    cell::{Cell, RefCell},
    rc::Rc,
};
use wasm_bindgen::{JsCast, JsValue, prelude::Closure};
use web_sys::{Element, EventTarget};

pub(crate) type FrameCallback = Closure<dyn FnMut(f64)>;
struct Listener {
    target: EventTarget,
    name: String,
    callback: JsValue,
    _closure: Box<dyn Any>,
}
impl Listener {
    fn detach(&self) {
        let _ = self
            .target
            .remove_event_listener_with_callback(&self.name, self.callback.unchecked_ref());
    }
}
impl Drop for Listener {
    fn drop(&mut self) {
        self.detach();
    }
}

pub(crate) struct Appearance {
    pub alive: Rc<Cell<bool>>,
    pub frame: RefCell<Option<FrameCallback>>,
    pub frame_id: Cell<Option<i32>>,
    listeners: RefCell<Vec<Listener>>,
    nodes: RefCell<Vec<Element>>,
    captures: cambium_rootstock::CaptureScope,
    cleanup: RefCell<Option<Box<dyn FnMut() -> bool>>>,
}
impl Appearance {
    pub fn new(captures: cambium_rootstock::CaptureScope) -> Rc<Self> {
        Rc::new(Self {
            alive: Rc::new(Cell::new(true)),
            frame: RefCell::new(None),
            frame_id: Cell::new(None),
            listeners: RefCell::new(Vec::new()),
            nodes: RefCell::new(Vec::new()),
            captures,
            cleanup: RefCell::new(None),
        })
    }
    pub fn own_node(&self, node: Element) {
        self.nodes.borrow_mut().push(node);
    }
    pub fn cleanup_with(&self, cleanup: impl FnMut() -> bool + 'static) {
        *self.cleanup.borrow_mut() = Some(Box::new(cleanup));
    }
    pub fn listen<C: AsRef<JsValue> + 'static>(
        &self,
        target: &EventTarget,
        name: &str,
        closure: C,
    ) -> Result<(), String> {
        let callback = closure.as_ref().clone();
        target
            .add_event_listener_with_callback(name, callback.unchecked_ref())
            .map_err(|_| format!("could not attach {name}"))?;
        self.listeners.borrow_mut().push(Listener {
            target: target.clone(),
            name: name.into(),
            callback,
            _closure: Box::new(closure),
        });
        Ok(())
    }
    pub fn dispose(self: &Rc<Self>) {
        if !self.alive.replace(false) {
            return;
        }
        self.captures.dispose();
        if let Some(id) = self.frame_id.take()
            && let Some(window) = web_sys::window()
        {
            let _ = window.cancel_animation_frame(id);
        }
        for listener in self.listeners.borrow().iter() {
            listener.detach();
        }
        for node in self.nodes.borrow_mut().drain(..) {
            node.remove();
        }
        let scope = self.clone();
        // Destroying an executing closure or borrowing its Host synchronously
        // is unsafe. Liveness and capture cancellation already took effect.
        wasm_bindgen_futures::spawn_local(async move {
            loop {
                let done = scope
                    .cleanup
                    .borrow_mut()
                    .as_mut()
                    .is_none_or(|cleanup| cleanup());
                if done {
                    break;
                }
                if next_turn().await.is_err() {
                    break;
                }
            }
            scope.cleanup.borrow_mut().take();
            scope.listeners.borrow_mut().clear();
            scope.frame.borrow_mut().take();
        });
    }
}
async fn next_turn() -> Result<(), JsValue> {
    let promise = js_sys::Promise::new(&mut |resolve, reject| {
        let callback = Closure::once_into_js(move || {
            let _ = resolve.call0(&JsValue::NULL);
        });
        match web_sys::window()
            .ok_or_else(|| JsValue::from_str("no window"))
            .and_then(|window| {
                window.set_timeout_with_callback_and_timeout_and_arguments_0(
                    callback.unchecked_ref(),
                    0,
                )
            }) {
            Ok(_) => {},
            Err(error) => {
                let _ = reject.call1(&JsValue::NULL, &error);
            },
        }
    });
    wasm_bindgen_futures::JsFuture::from(promise)
        .await
        .map(|_| ())
}
