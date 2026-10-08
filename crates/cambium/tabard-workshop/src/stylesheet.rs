// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! A separate real HTML document for authored appearance sheets. Its cascade
//! never sees the workshop editor, and exact author CSS is parsed by Livery.

use genet_livery::{Device, LiveryDocument, StyleSet};
use genet_scripted_dom::{NodeId, ScriptedDom};
use layout_dom_api::{LayoutDom, LocalName, Namespace, NodeKind};
use livery::values::ColorScheme;
use netrender::Scene;
use tabard::theme::registry::Mode;

use crate::ReaderSpecimen;

pub const STYLESHEET_LEAF_KEY: u64 = 0x7461_6263;
pub const APPLICATION_SOURCE: &str = include_str!("../fixtures/application.html");
pub const APPLICATION_STRUCTURE: &str = include_str!("../fixtures/application.css");

/// Portable scenes share the existing same-device host adapter. Ownership of
/// CSS, layout and paint stays with each existing document component.
pub trait PreviewScene {
    fn frame(&mut self, width: u32, height: u32) -> Scene;
    fn revision(&self) -> u64;
    fn accessible_name(&self) -> &str;
}

impl PreviewScene for ReaderSpecimen {
    fn frame(&mut self, width: u32, height: u32) -> Scene {
        ReaderSpecimen::frame(self, width, height)
    }
    fn revision(&self) -> u64 {
        ReaderSpecimen::revision(self)
    }
    fn accessible_name(&self) -> &str {
        ReaderSpecimen::accessible_name(self)
    }
}

/// The fixed application content has its own DOM, cascade, media device and
/// retained Livery layout. It is a read-only appearance, with no scripts or
/// resource loading. Host chrome and typed reader/graph paints are separate.
pub struct StylesheetSpecimen {
    document: LiveryDocument<ScriptedDom>,
    mode: Mode,
    rules: Vec<String>,
    accessible_name: String,
    revision: u64,
    last_error: Option<String>,
}

impl Default for StylesheetSpecimen {
    fn default() -> Self {
        Self::new(&Mode::Light, &[])
    }
}

impl StylesheetSpecimen {
    pub fn new(mode: &Mode, rules: &[String]) -> Self {
        let document = make_document(mode, rules);
        let mut text = Vec::new();
        if let Some(body) = node_with_id(document.dom(), "application-body") {
            collect_text(document.dom(), body, &mut text);
        }
        let accessible_name = format!(
            "Read-only application stylesheet preview. {}",
            text.join(" ")
        );
        Self {
            document,
            mode: mode.clone(),
            rules: rules.to_vec(),
            accessible_name,
            revision: 1,
            last_error: None,
        }
    }

    /// A stylesheet replaces the derived sheet; only the fixed fixture's
    /// structure and variable consumers precede it. Repeated inputs retain
    /// the document.
    pub fn set_appearance(&mut self, mode: &Mode, rules: &[String]) -> bool {
        if &self.mode == mode && self.rules == rules {
            return false;
        }
        self.document = make_document(mode, rules);
        self.mode = mode.clone();
        self.rules = rules.to_vec();
        self.last_error = None;
        self.revision = self
            .revision
            .checked_add(1)
            .expect("stylesheet revision exhausted");
        true
    }

    pub fn source(&self) -> &'static str {
        APPLICATION_SOURCE
    }

    pub fn rules(&self) -> &[String] {
        &self.rules
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }

    pub fn accessible_name(&self) -> &str {
        &self.accessible_name
    }

    /// Real CSS recovery diagnostics from the shared parser/cascade. Rendering
    /// failures are included after a frame, while author text remains intact.
    pub fn diagnostics(&self) -> Vec<String> {
        let mut diagnostics: Vec<String> = self
            .document
            .style_set()
            .diagnostics()
            .iter()
            .map(|diagnostic| format!("{}: {}", diagnostic.prelude, diagnostic.message))
            .collect();
        diagnostics.extend(self.last_error.iter().cloned());
        diagnostics
    }

    /// Inspect the actual isolated document's resolved longhands/custom
    /// properties, including var() and media rules, using Livery's CSSOM seam.
    pub fn computed_style(&self, id: &str, property: &str) -> Option<String> {
        let node = node_with_id(self.document.dom(), id)?;
        self.document.computed_style(node, property)
    }

    pub fn frame(&mut self, width: u32, height: u32) -> Scene {
        let (width, height) = (width.max(1), height.max(1));
        match self.document.frame(width, height) {
            Ok(list) => {
                self.last_error = None;
                paint_list_render::translate_paint_list(&list)
            },
            Err(error) => {
                self.last_error = Some(error.to_string());
                Scene::new(width, height)
            },
        }
    }
}

impl PreviewScene for StylesheetSpecimen {
    fn frame(&mut self, width: u32, height: u32) -> Scene {
        StylesheetSpecimen::frame(self, width, height)
    }
    fn revision(&self) -> u64 {
        StylesheetSpecimen::revision(self)
    }
    fn accessible_name(&self) -> &str {
        StylesheetSpecimen::accessible_name(self)
    }
}

fn make_document(mode: &Mode, rules: &[String]) -> LiveryDocument<ScriptedDom> {
    let dom = ScriptedDom::from_serialized_document(APPLICATION_SOURCE);
    let sheets: Vec<&str> = std::iter::once(APPLICATION_STRUCTURE)
        .chain(rules.iter().map(String::as_str))
        .collect();
    let mut document = LiveryDocument::new(
        dom,
        StyleSet::cambium(&sheets),
        Device::screen(420.0, 320.0),
    );
    document.set_preferred_color_scheme(if mode.dark() {
        ColorScheme::Dark
    } else {
        ColorScheme::Light
    });
    document
}

fn node_with_id(dom: &ScriptedDom, id: &str) -> Option<NodeId> {
    fn walk(dom: &ScriptedDom, node: NodeId, id: &str) -> Option<NodeId> {
        if dom
            .attribute(node, &Namespace::default(), &LocalName::from("id"))
            .is_some_and(|value| value == id)
        {
            return Some(node);
        }
        dom.dom_children(node)
            .find_map(|child| walk(dom, child, id))
    }
    walk(dom, dom.document(), id)
}

fn collect_text(dom: &ScriptedDom, node: NodeId, output: &mut Vec<String>) {
    if matches!(dom.kind(node), NodeKind::Text)
        && let Some(text) = dom.text(node)
        && !text.trim().is_empty()
    {
        output.push(text.trim().to_owned());
    }
    for child in dom.dom_children(node) {
        collect_text(dom, child, output);
    }
}
