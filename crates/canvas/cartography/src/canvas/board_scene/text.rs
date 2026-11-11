// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

impl BoardText {
    pub fn new() -> Self {
        Self {
            text: genet_livery::TextSystem::new(),
            generation: 0,
            font_roles: String::new(),
        }
    }

    /// Declarations for the document's root custom properties, such as
    /// `--tabard-font-interface: "IBM Plex Sans";`, so the titles' `var()`
    /// resolves. Empty (the default) leaves the sheets as authored; a changed
    /// value applies on the next paint.
    pub fn set_font_roles(&mut self, declarations: &str) {
        self.font_roles = declarations.to_owned();
    }

    /// `sheet` under the host's root declarations. Each label is its own
    /// document root (no parent element), so `:root` reaches them.
    pub(crate) fn rooted(&self, sheet: &str) -> String {
        if self.font_roles.is_empty() {
            sheet.to_owned()
        } else {
            format!(":root {{ {} }}\n{sheet}", self.font_roles)
        }
    }

    /// Register one face, by its bytes.
    pub fn register_font(&mut self, bytes: Vec<u8>) {
        self.text.register_font_bytes(bytes);
    }

    /// Lay the titles out over their card rectangles and paint them.
    pub(super) fn titles(
        &mut self,
        cards: &[(String, String, BoardRect)],
        width: u32,
        height: u32,
    ) -> Option<genet_livery::LiveryPaintList> {
        self.labels(cards, width, height, TITLE_SHEET)
    }

    /// Shared label shaping for independent surfaces; appearance belongs to the host.
    pub(crate) fn labels(
        &mut self,
        cards: &[(String, String, BoardRect)],
        width: u32,
        height: u32,
        sheet: &str,
    ) -> Option<genet_livery::LiveryPaintList> {
        use layout_dom_api::{LayoutDom, LayoutDomMut};
        if cards.iter().all(|(_, title, _)| title.is_empty()) {
            return None;
        }
        let mut dom = genet_scripted_dom::ScriptedDom::new();
        let root = dom.document();
        for (_, title, rect) in cards.iter().filter(|(_, title, _)| !title.is_empty()) {
            let line = dom.create_element(qual("div"));
            dom.set_attribute(line, qual("class"), "board-title");
            dom.set_attribute(
                line,
                qual("style"),
                &format!(
                    "left: {:.1}px; top: {:.1}px; width: {:.1}px;",
                    rect.x,
                    rect.y + (rect.height - TITLE_LINE) * 0.5,
                    rect.width
                ),
            );
            let words = dom.create_text(title);
            dom.append_child(line, words);
            dom.append_child(root, line);
        }
        let sheet = self.rooted(sheet);
        let styles = genet_livery::resolve_styles(
            &dom,
            &genet_livery::StyleSet::cambium(&[sheet.as_str()]),
            &genet_livery::Device::screen(width as f32, height as f32),
            &genet_livery::InteractionStates::default(),
        );
        let (styles, layout) = genet_livery::layout_with_text_system(
            &dom,
            &styles,
            width as f32,
            height as f32,
            genet_livery::ViewportSizes::uniform(width as f32, height as f32),
            &mut self.text,
            &std::collections::HashMap::new(),
        )
        .ok()?;
        self.generation += 1;
        Some(genet_livery::emit_paint_list_with_text_system(
            &dom,
            &styles,
            &layout,
            DeviceIntSize::new(width as i32, height as i32),
            self.generation,
            &mut self.text,
        ))
    }
}
