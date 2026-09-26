/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! The accessibility page in a browser, the test page for the one-tree plan's
//! phase 1: the host mirrors it into DOM elements with ARIA, and the browser's
//! own accessibility tree lists each control.
//!
//! ```text
//! RUSTFLAGS='--cfg getrandom_backend="wasm_js"' cargo build \
//!     -p cambium-genet-web-host --example a11y_page --target wasm32-unknown-unknown
//! wasm-bindgen --target web --no-typescript --out-dir <dir> \
//!     target/wasm32-unknown-unknown/debug/examples/a11y_page.wasm
//! ```
//!
//! Serve `<dir>` with this directory's `index.html` beside the bindings.
#![cfg(target_arch = "wasm32")]

mod page;

use cambium_genet_web_host::mount;
use cambium_rootstock::{CloseDisposition, HostFont, HostHooks, HostOptions, Init};
use wasm_bindgen::prelude::*;

use page::{Child, Page};

/// Mount the page onto the canvas with id `canvas_id`.
#[wasm_bindgen]
pub async fn start(canvas_id: String) -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    let canvas = web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.get_element_by_id(&canvas_id))
        .ok_or_else(|| JsValue::from_str(&format!("no element with id {canvas_id:?}")))?
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|_| JsValue::from_str(&format!("{canvas_id:?} is not a canvas")))?;
    let options = HostOptions {
        title: "Accessibility page".into(),
        ..Default::default()
    };
    mount(
        canvas,
        options,
        |_window, _commands, _wake| Init {
            state: Page::default(),
            logic: page::page as fn(&Page) -> Child,
            sheet: page::SHEET.to_string(),
            // A browser lends genet no system faces, so the page brings one.
            fonts: vec![HostFont {
                family: None,
                bytes: include_bytes!(
                    "../../../examples/genet_web_smoke/assets/Roboto-Regular.ttf"
                )
                .to_vec(),
            }],
            images: Vec::new(),
        },
        HostHooks {
            frame: Box::new(|_ctx| false),
            after_dispatch: Box::new(|_ctx| {}),
            after_frame: Box::new(|_ctx| {}),
            after_wake: Box::new(|_ctx| {}),
            close_request: Box::new(|_ctx, _request| CloseDisposition::KeepVisible),
            focused_text: Box::new(|_runner| None),
            key_intercept: Box::new(|_runner, _press| false),
        },
    )
    .await
    .map_err(|error| JsValue::from_str(&error))?;
    Ok(())
}
