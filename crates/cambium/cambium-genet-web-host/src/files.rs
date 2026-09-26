// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The browser's file chooser: a file input the host keeps in the page.
//!
//! A request clicks the input, which opens the browser's chooser while the
//! press that asked still counts as a user gesture. The input's `change` reads
//! the chosen files and answers the request, and its `cancel` answers it with
//! nothing chosen. The input stays out of a reader's way, since the view's own
//! control is what a reader operates, but it stays in the page, where a tool
//! can set files on it without the dialog.

use std::cell::RefCell;
use std::rc::Rc;

use cambium::{FileEvent, FileRequest, OpenedFile};
use cambium_rootstock::{FileAnswer, FileChooser};
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::Closure;
use web_sys::{HtmlCanvasElement, HtmlInputElement};

/// Opens files through a file input kept beside the canvas.
pub struct WebFileChooser {
    input: HtmlInputElement,
    pending: Rc<RefCell<Option<FileAnswer>>>,
}

impl WebFileChooser {
    /// Put the chooser's input in the page beside `canvas`.
    pub fn new(canvas: &HtmlCanvasElement) -> Result<Self, String> {
        let document = canvas
            .owner_document()
            .ok_or("the canvas has no document")?;
        let input: HtmlInputElement = document
            .create_element("input")
            .map_err(|_| "could not create the file input")?
            .unchecked_into();
        input.set_type("file");
        input.set_id("cambium-file-input");
        let _ = input.set_attribute("aria-hidden", "true");
        let _ = input.set_attribute("tabindex", "-1");
        input.style().set_css_text(
            "position:absolute;width:1px;height:1px;opacity:0;overflow:hidden;pointer-events:none;",
        );
        let parent = canvas
            .parent_node()
            .ok_or("the canvas is not in the page")?;
        parent
            .insert_before(&input, canvas.next_sibling().as_ref())
            .map_err(|_| "could not place the file input")?;

        let pending: Rc<RefCell<Option<FileAnswer>>> = Rc::default();
        let chosen = {
            let input = input.clone();
            let pending = pending.clone();
            Closure::<dyn FnMut(web_sys::Event)>::new(move |_: web_sys::Event| {
                let Some(answer) = pending.borrow_mut().take() else {
                    return;
                };
                let files: Vec<web_sys::File> = input
                    .files()
                    .map(|list| (0..list.length()).filter_map(|i| list.item(i)).collect())
                    .unwrap_or_default();
                // Choosing the same file again must fire `change` again.
                input.set_value("");
                wasm_bindgen_futures::spawn_local(async move {
                    let mut opened = Vec::with_capacity(files.len());
                    for file in files {
                        if let Some(file) = read(file).await {
                            opened.push(file);
                        }
                    }
                    answer.send(FileEvent { files: opened });
                });
            })
        };
        input
            .add_event_listener_with_callback("change", chosen.as_ref().unchecked_ref())
            .map_err(|_| "could not listen for a chosen file")?;
        chosen.forget();
        let cancelled = {
            let pending = pending.clone();
            Closure::<dyn FnMut(web_sys::Event)>::new(move |_: web_sys::Event| {
                if let Some(answer) = pending.borrow_mut().take() {
                    answer.send(FileEvent::default());
                }
            })
        };
        input
            .add_event_listener_with_callback("cancel", cancelled.as_ref().unchecked_ref())
            .map_err(|_| "could not listen for a cancelled choice")?;
        cancelled.forget();
        Ok(Self { input, pending })
    }
}

impl FileChooser for WebFileChooser {
    fn open(&mut self, request: &FileRequest, answer: FileAnswer) {
        // A request still waiting is answered with nothing chosen.
        if let Some(earlier) = self.pending.borrow_mut().replace(answer) {
            earlier.send(FileEvent::default());
        }
        let accept: Vec<String> = request
            .filter
            .extensions
            .iter()
            .map(|extension| format!(".{extension}"))
            .collect();
        self.input.set_accept(&accept.join(","));
        self.input.set_multiple(request.filter.multiple);
        self.input.click();
    }
}

/// One chosen file's bytes and what the browser says of it.
async fn read(file: web_sys::File) -> Option<OpenedFile> {
    let buffer = wasm_bindgen_futures::JsFuture::from(file.array_buffer())
        .await
        .ok()?;
    let media_type = file.type_();
    Some(OpenedFile {
        name: file.name(),
        media_type: (!media_type.is_empty()).then_some(media_type),
        last_modified_ms: Some(file.last_modified() as u64),
        bytes: js_sys::Uint8Array::new(&buffer).to_vec(),
    })
}
