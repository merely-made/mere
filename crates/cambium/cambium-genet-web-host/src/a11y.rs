// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The browser's accessibility: the tree mirrored into DOM elements with ARIA.
//!
//! Cambium presents onto a `<canvas>`, which a screen reader sees as one
//! graphic, so the browser needs what the desktop needs: a parallel tree kept
//! in step with the layout. Here it is made of DOM elements. Each sync lowers
//! genet's neutral projection to ARIA ([`crate::mirror`]) and writes only what
//! changed onto an element per node, positioned over the canvas where the
//! node painted. The elements are transparent and let the pointer through to
//! the canvas; they exist for a reader, whose Tab, virtual cursor and touch
//! exploration all find them where the controls are drawn.
//!
//! **Focus follows the tree.** When the application's focus moves while it
//! holds the page's focus, DOM focus moves to that node's element, so a
//! reader announces it. Keys that reach a focused element are handled as the
//! canvas's are, and the canvas hands focus on to the mirror when it gets it.
//!
//! **A reader's actions route at once.** `mount` listens on the mirror: a
//! click on an element activates its node and a focus on one focuses it,
//! through the host's own activation and focus paths, and the frame is drawn
//! and mirrored before the listener returns. So [`Accessibility::sync`]
//! drains nothing, and a page whose frame callback never runs, as in a
//! background tab, still answers.
//!
//! **A leaf describes itself through AccessKit.** For a node that is a leaf,
//! the leaf's own hook fills a scratch AccessKit node, and its role and label
//! are carried over into ARIA; an author's name wins over the leaf's.

use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use cambium_rootstock::{
    A11yRequest, Accessibility, LeafRegistry, NodeId, OwnedLayout, ScriptedDom, document_projection,
};
use layout_dom_api::{LayoutDom as _, LocalName, Namespace, NodeKind};
use wasm_bindgen::JsCast;
use web_sys::{Document, Element, HtmlCanvasElement, HtmlElement, Text};

use crate::mirror::{self, MirrorNode};

/// The attribute naming each element's node, which the listeners route by.
pub(crate) const NODE_ATTR: &str = "data-node";

/// What the mirror shares with the listeners `mount` installs on it.
#[derive(Clone)]
pub(crate) struct MirrorHandle {
    pub root: HtmlElement,
    /// Set while the mirror moves DOM focus itself, so the focus listener does
    /// not route the move back as a reader's request.
    pub moving: Rc<Cell<bool>>,
    shared: Rc<RefCell<Shared>>,
}

#[derive(Default)]
struct Shared {
    /// Each mirrored node's DOM node, as of the last sync.
    targets: HashMap<u64, NodeId>,
    /// The tree's focus as of the last sync, and its element when mirrored.
    focused: Option<HtmlElement>,
}

impl MirrorHandle {
    /// The DOM node behind the mirror element an event reached, if any.
    pub fn target_of(&self, event_target: Option<web_sys::EventTarget>) -> Option<NodeId> {
        let element = event_target?.dyn_into::<Element>().ok()?;
        let element = element.closest(&format!("[{NODE_ATTR}]")).ok()??;
        let id: u64 = element.get_attribute(NODE_ATTR)?.parse().ok()?;
        self.shared.borrow().targets.get(&id).copied()
    }

    /// Put DOM focus in the mirror: on the focused node's element, or on the
    /// mirror itself when the focus is on nothing mirrored.
    pub fn take_focus(&self) {
        let element = self
            .shared
            .borrow()
            .focused
            .clone()
            .unwrap_or_else(|| self.root.clone());
        self.moving.set(true);
        let _ = element.focus();
        self.moving.set(false);
    }
}

/// The last attributes, text and box written onto one element.
#[derive(Default, PartialEq)]
struct Written {
    attrs: Vec<(&'static str, String)>,
    text: Option<String>,
    rect: Option<[f32; 4]>,
    focusable: bool,
}

struct Mirrored {
    element: HtmlElement,
    text: Option<Text>,
    written: Written,
}

/// The browser accessibility seam: a DOM mirror of the tree, with ARIA.
pub struct DomAccessibility {
    canvas: HtmlCanvasElement,
    document: Document,
    handle: MirrorHandle,
    elements: HashMap<u64, Mirrored>,
    /// The focus id the last sync saw, to move DOM focus only on a change.
    focus: Option<u64>,
}

impl DomAccessibility {
    /// Attach to the canvas the application presents onto. `label` names the
    /// mirror's region, which stands for the application.
    pub fn new(canvas: HtmlCanvasElement, label: impl Into<String>) -> Result<Self, String> {
        let document = canvas
            .owner_document()
            .ok_or("the canvas has no document")?;
        let root: HtmlElement = document
            .create_element("div")
            .map_err(|_| "could not create the mirror")?
            .unchecked_into();
        let _ = root.set_attribute("role", "region");
        let _ = root.set_attribute("aria-label", &label.into());
        let _ = root.set_attribute("tabindex", "-1");
        let _ = root.set_attribute("data-cambium-mirror", "");
        // Transparent and pointer-blind: the canvas draws and takes the
        // pointer, the mirror is there for a reader.
        let _ = root.style().set_css_text(
            "position:absolute;margin:0;padding:0;border:0;overflow:hidden;\
             pointer-events:none;opacity:0;",
        );
        let parent = canvas
            .parent_node()
            .ok_or("the canvas is not in the page")?;
        parent
            .insert_before(&root, canvas.next_sibling().as_ref())
            .map_err(|_| "could not place the mirror")?;
        // The mirror speaks for the canvas now.
        let _ = canvas.set_attribute("aria-hidden", "true");
        Ok(Self {
            canvas,
            document,
            handle: MirrorHandle {
                root,
                moving: Rc::new(Cell::new(false)),
                shared: Rc::default(),
            },
            elements: HashMap::new(),
            focus: None,
        })
    }

    /// What the listeners on the mirror need.
    pub(crate) fn handle(&self) -> MirrorHandle {
        self.handle.clone()
    }

    /// Keep the mirror's box on the canvas's, wherever the page has put it.
    fn cover_canvas(&self) {
        let root = &self.handle.root;
        let canvas = self.canvas.get_bounding_client_rect();
        let here = root.get_bounding_client_rect();
        let style = root.style();
        let left = root.offset_left() as f64 + canvas.left() - here.left();
        let top = root.offset_top() as f64 + canvas.top() - here.top();
        let _ = style.set_property("left", &format!("{left}px"));
        let _ = style.set_property("top", &format!("{top}px"));
        let _ = style.set_property("width", &format!("{}px", canvas.width()));
        let _ = style.set_property("height", &format!("{}px", canvas.height()));
    }

    /// Write `nodes` as the children of `parent`, in order, after its text.
    fn write(&mut self, parent: &HtmlElement, nodes: &[MirrorNode], live: &mut HashSet<u64>) {
        let text_node = |element: &HtmlElement| {
            element
                .first_child()
                .filter(|child| child.node_type() == web_sys::Node::TEXT_NODE)
        };
        let mut cursor = match text_node(parent) {
            Some(text) => text.next_sibling(),
            None => parent.first_child(),
        };
        for node in nodes {
            live.insert(node.id);
            let mirrored = mirrored(&mut self.elements, &self.document, node.id);
            let element = mirrored.element.clone();
            let wanted = Written {
                attrs: node.attrs.clone(),
                text: node.text.clone(),
                rect: node.rect,
                focusable: node.focusable,
            };
            if mirrored.written != wanted {
                update(&self.document, mirrored, &wanted);
                mirrored.written = wanted;
            }
            let in_place = cursor
                .as_ref()
                .is_some_and(|at| at.is_same_node(Some(element.as_ref())));
            if !in_place {
                let _ = parent.insert_before(&element, cursor.as_ref());
            }
            cursor = element.next_sibling();
            self.write(&element, &node.children, live);
        }
    }
}

/// The element mirroring node `id`, made on first sight.
fn mirrored<'a>(
    elements: &'a mut HashMap<u64, Mirrored>,
    document: &Document,
    id: u64,
) -> &'a mut Mirrored {
    elements.entry(id).or_insert_with(|| {
        let element: HtmlElement = document
            .create_element("div")
            .expect("a document creates a div")
            .unchecked_into();
        let _ = element.set_attribute(NODE_ATTR, &id.to_string());
        let _ = element
            .style()
            .set_css_text("position:absolute;margin:0;padding:0;");
        Mirrored {
            element,
            text: None,
            written: Written::default(),
        }
    })
}

/// Bring one element to `wanted`, touching only what differs.
fn update(document: &Document, mirrored: &mut Mirrored, wanted: &Written) {
    let element = &mirrored.element;
    let old = &mirrored.written;
    for (name, _) in &old.attrs {
        if !wanted.attrs.iter().any(|(key, _)| key == name) {
            let _ = element.remove_attribute(name);
        }
    }
    for (name, value) in &wanted.attrs {
        if !old
            .attrs
            .iter()
            .any(|(key, old)| key == name && old == value)
        {
            let _ = element.set_attribute(name, value);
        }
    }
    if old.text != wanted.text {
        match (&mirrored.text, &wanted.text) {
            (Some(node), Some(text)) => node.set_data(text),
            (None, Some(text)) => {
                let node = document.create_text_node(text);
                let _ = element.insert_before(&node, element.first_child().as_ref());
                mirrored.text = Some(node);
            },
            (Some(node), None) => {
                let _ = element.remove_child(node);
                mirrored.text = None;
            },
            (None, None) => {},
        }
    }
    if old.rect != wanted.rect {
        let [x, y, width, height] = wanted.rect.unwrap_or_default();
        let style = element.style();
        let _ = style.set_property("left", &format!("{x}px"));
        let _ = style.set_property("top", &format!("{y}px"));
        let _ = style.set_property("width", &format!("{width}px"));
        let _ = style.set_property("height", &format!("{height}px"));
    }
    if old.focusable != wanted.focusable {
        if wanted.focusable {
            let _ = element.set_attribute("tabindex", "-1");
        } else {
            let _ = element.remove_attribute("tabindex");
        }
    }
}

/// Every DOM node by its opaque id, and each leaf's key.
fn walk(
    dom: &ScriptedDom,
    node: NodeId,
    targets: &mut HashMap<u64, NodeId>,
    leaves: &mut HashMap<u64, u64>,
) {
    let id = dom.opaque_id(node);
    targets.insert(id, node);
    if dom.kind(node) == NodeKind::Element
        && dom
            .element_name(node)
            .is_some_and(|name| matches!(name.local.as_ref(), "custom-leaf" | "chisel-leaf"))
        && let Some(key) = dom
            .attribute(node, &Namespace::default(), &LocalName::from("key"))
            .and_then(|key| key.parse().ok())
    {
        leaves.insert(id, key);
    }
    for child in dom.dom_children(node) {
        walk(dom, child, targets, leaves);
    }
}

impl Accessibility for DomAccessibility {
    fn sync(
        &mut self,
        dom: &ScriptedDom,
        layout: &OwnedLayout,
        leaves: &mut LeafRegistry<u64>,
        focus: Option<u64>,
        layout_scale: f64,
    ) -> Vec<A11yRequest> {
        let mut targets = HashMap::new();
        let mut leaf_keys = HashMap::new();
        walk(dom, dom.document(), &mut targets, &mut leaf_keys);

        // Layout pixels ride the UI zoom into CSS pixels; the device half of
        // the layout scale is the browser's own.
        let device = web_sys::window().map_or(1.0, |window| window.device_pixel_ratio());
        let scale = (layout_scale / device) as f32;
        let projection = document_projection(dom, layout, focus);
        let plan = mirror::plan(&projection, scale, |id, name| {
            let key = leaf_keys.get(&id)?;
            let leaf = leaves.get_mut(key)?;
            let mut scratch = accesskit::Node::new(accesskit::Role::Unknown);
            if let Some(name) = name {
                scratch.set_label(name);
            }
            leaf.accessibility(&mut scratch);
            Some(mirror::leaf_semantics(&scratch))
        });

        self.cover_canvas();
        let root = self.handle.root.clone();
        let mut live = HashSet::new();
        self.write(&root, &plan, &mut live);
        self.elements.retain(|id, mirrored| {
            let keep = live.contains(id);
            if !keep {
                mirrored.element.remove();
            }
            keep
        });

        let focused = focus.and_then(|id| self.elements.get(&id).map(|m| m.element.clone()));
        {
            let mut shared = self.handle.shared.borrow_mut();
            shared.targets = targets;
            shared.focused = focused;
        }
        if focus != self.focus {
            self.focus = focus;
            // Only while the application holds the page's focus: a reader
            // elsewhere on the page is not pulled back into it.
            let active = self.document.active_element();
            let holds = active.as_ref().is_some_and(|active| {
                active.is_same_node(Some(self.canvas.as_ref()))
                    || self.handle.root.contains(Some(active.as_ref()))
            });
            if holds {
                self.handle.take_focus();
            }
        }
        Vec::new()
    }
}
