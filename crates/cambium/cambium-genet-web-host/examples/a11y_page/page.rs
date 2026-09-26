/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! The test page for the one-tree plan's phases 1 and 2: one of each control
//! phase 1 names, a button that opens a file, and a status line that says what
//! the last action did. The browser example mounts it, and the mirror's native
//! tests lay it out.

use cambium::{
    AnyView, FileEvent, FileFilter, GenetCtx, GenetElement, PointerClick, SelectState, TabStrip,
    TextInput, button, checkbox, el, lens, open_file, select, tab_strip, text_field,
};

pub const COLOURS: [&str; 3] = ["Red", "Green", "Blue"];
pub const TABS: [&str; 3] = ["One", "Two", "Three"];

pub struct Page {
    pub presses: u32,
    pub name: TextInput,
    pub subscribed: bool,
    pub colour: SelectState,
    pub tabs: TabStrip,
    /// Whether the page is asking its host for a file.
    pub asking: bool,
    /// The last file opened, with its size.
    pub opened: Option<String>,
}

impl Default for Page {
    fn default() -> Self {
        let mut tabs = TabStrip::new(0);
        tabs.label = "Sections".into();
        Self {
            presses: 0,
            name: TextInput::new(""),
            subscribed: false,
            colour: SelectState::new(0).with_label("Colour"),
            tabs,
            asking: false,
            opened: None,
        }
    }
}

pub type Child = Box<dyn AnyView<Page, (), GenetCtx, GenetElement>>;

pub const SHEET: &str = "\
    :root { font-family: Roboto, sans-serif; font-size: 16px; color: #1d1d1f; background: #fafaf7; } \
    main { display: flex; flex-direction: column; gap: 10px; padding: 16px; } \
    h1 { font-size: 20px; margin: 0; } \
    label { display: flex; gap: 8px; align-items: center; } \
    input { width: 200px; height: 24px; border: 1px solid #888; } \
    ul { margin: 0; } \
    .select-list { display: flex; flex-direction: column; }";

pub fn page(page: &Page) -> Child {
    Box::new(el(
        "main",
        (
            el("h1", "Accessibility page"),
            button("Press", |page: &mut Page, _: PointerClick| {
                page.presses += 1
            })
            .attr("id", "press"),
            open_file(
                button("Open a file", |page: &mut Page, _: PointerClick| {
                    page.asking = true
                })
                .attr("id", "open-file"),
                page.asking,
                FileFilter::default(),
                |page: &mut Page, event: FileEvent| {
                    page.asking = false;
                    page.opened = event
                        .files
                        .first()
                        .map(|file| format!("{} ({} bytes)", file.name, file.bytes.len()));
                },
            ),
            el(
                "label",
                (
                    "Name",
                    lens(
                        |input: &mut TextInput| text_field(input),
                        |page: &mut Page| &mut page.name,
                    ),
                ),
            ),
            lens(
                |on: &mut bool| checkbox(*on).attr("aria-label", "Subscribe"),
                |page: &mut Page| &mut page.subscribed,
            ),
            lens(
                |colour: &mut SelectState| select(colour, &COLOURS),
                |page: &mut Page| &mut page.colour,
            ),
            lens(
                |tabs: &mut TabStrip| tab_strip::<()>(tabs, &TABS),
                |page: &mut Page| &mut page.tabs,
            ),
            el(
                "ul",
                (el("li", "Alpha"), el("li", "Beta"), el("li", "Gamma")),
            ),
            el("p", status(page)).attr("role", "status"),
        ),
    ))
}

/// What the page says of itself, so a reader's action shows in the mirror.
pub fn status(page: &Page) -> String {
    format!(
        "Pressed {} times. Subscribed: {}. Colour: {}. Tab: {}. File: {}.",
        page.presses,
        if page.subscribed { "yes" } else { "no" },
        COLOURS.get(page.colour.selected).unwrap_or(&""),
        TABS.get(page.tabs.selected).unwrap_or(&""),
        page.opened.as_deref().unwrap_or("none"),
    )
}
