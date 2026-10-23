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
//! Rootstock's composed host projection to ARIA ([`crate::mirror`]) and writes only what
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
//! **Contributions follow shared host policy.** Rootstock seeds leaf callbacks
//! with DOM semantics and composes producer children and action targets before
//! platform lowering. ARIA consumes that same AccessKit-shaped observation.

use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use cambium_rootstock::{A11yRequest, Accessibility, LeafRegistry, NodeId, OwnedLayout, WindowDom};
use wasm_bindgen::JsCast;
use web_sys::{Document, Element, HtmlCanvasElement, HtmlElement, Text};

use crate::mirror::{self, MirrorNode};
use cambium_rootstock::{
    A11yAdapterState, A11yPublication, A11yPublicationOutcome, A11yPublicationRefusal,
    A11yPublicationSkip, A11ySync,
};

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
    /// Frozen publication requests, shared with native/queue admission.
    requests: HashMap<u64, A11yRequest>,
    /// Exact proxy elements: a numeric attribute alone cannot impersonate one.
    elements: HashMap<u64, HtmlElement>,
    /// The tree's focus as of the last sync, and its element when mirrored.
    focused: Option<HtmlElement>,
}

impl MirrorHandle {
    /// What a reader's request on the mirror element an event reached lands
    /// on: its DOM node, or the drawn node's action its button stands for.
    pub fn request_of(
        &self,
        event_target: Option<web_sys::EventTarget>,
        action: cambium_rootstock::A11yAction,
    ) -> Option<A11yRequest> {
        let element = event_target?.dyn_into::<Element>().ok()?;
        let element = element.closest(&format!("[{NODE_ATTR}]")).ok()??;
        // Resolve only elements still belonging to this mirror. Detached or
        // foreign elements can carry a numeric ID that the current map reuses.
        if !self.root.contains(Some(element.as_ref())) {
            return None;
        }
        let id: u64 = element.get_attribute(NODE_ATTR)?.parse().ok()?;
        let shared = self.shared.borrow();
        if !shared
            .elements
            .get(&id)?
            .is_same_node(Some(element.as_ref()))
        {
            return None;
        }
        let mut request = shared.requests.get(&id)?.clone();
        request.action = action;
        Some(request)
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
        let options = web_sys::FocusOptions::new();
        options.set_prevent_scroll(true);
        let _ = element.focus_with_options(&options);
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
impl Drop for DomAccessibility {
    fn drop(&mut self) {
        self.handle.root.remove();
        let mut shared = self.handle.shared.borrow_mut();
        shared.requests.clear();
        shared.elements.clear();
        shared.focused = None;
    }
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
        root.set_attribute("role", "region")
            .map_err(|_| "could not configure the mirror")?;
        root.set_attribute("aria-label", &label.into())
            .map_err(|_| "could not configure the mirror")?;
        root.set_attribute("tabindex", "-1")
            .map_err(|_| "could not configure the mirror")?;
        root.set_attribute("data-cambium-mirror", "")
            .map_err(|_| "could not configure the mirror")?;
        // Transparent and pointer-blind: the canvas draws and takes the
        // pointer, the mirror is there for a reader. Clip without a scrolling
        // box: browser focus must not shift its controls away from the canvas.
        let _ = root.style().set_css_text(
            "position:absolute;margin:0;padding:0;border:0;overflow:clip;\
             pointer-events:none;opacity:0;",
        );
        let parent = canvas
            .parent_node()
            .ok_or("the canvas is not in the page")?;
        parent
            .insert_before(&root, canvas.next_sibling().as_ref())
            .map_err(|_| "could not place the mirror")?;
        // The mirror speaks for the canvas now.
        if canvas.set_attribute("aria-hidden", "true").is_err() {
            root.remove();
            return Err("could not configure the canvas accessibility".into());
        }
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
    fn clear_publication(&mut self) {
        for (_, old) in self.elements.drain() {
            old.element.remove();
        }
        let mut shared = self.handle.shared.borrow_mut();
        shared.requests.clear();
        shared.elements.clear();
        shared.focused = None;
        self.focus = None;
    }

    fn cover_canvas(&self) -> Result<(), wasm_bindgen::JsValue> {
        let root = &self.handle.root;
        let canvas = self.canvas.get_bounding_client_rect();
        let here = root.get_bounding_client_rect();
        let style = root.style();
        let left = root.offset_left() as f64 + canvas.left() - here.left();
        let top = root.offset_top() as f64 + canvas.top() - here.top();
        style.set_property("left", &format!("{left}px"))?;
        style.set_property("top", &format!("{top}px"))?;
        style.set_property("width", &format!("{}px", canvas.width()))?;
        style.set_property("height", &format!("{}px", canvas.height()))?;
        Ok(())
    }

    /// Write `nodes` as the children of `parent`, in order, after its text.
    fn write(
        &mut self,
        parent: &HtmlElement,
        nodes: &[MirrorNode],
        live: &mut HashSet<u64>,
    ) -> Result<(), wasm_bindgen::JsValue> {
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
            let mirrored = mirrored(&mut self.elements, &self.document, node.id)?;
            let element = mirrored.element.clone();
            let wanted = Written {
                attrs: node.attrs.clone(),
                text: node.text.clone(),
                rect: node.rect,
                focusable: node.focusable,
            };
            if mirrored.written != wanted {
                update(&self.document, mirrored, &wanted)?;
                mirrored.written = wanted;
            }
            let in_place = cursor
                .as_ref()
                .is_some_and(|at| at.is_same_node(Some(element.as_ref())));
            if !in_place {
                parent.insert_before(&element, cursor.as_ref())?;
            }
            cursor = element.next_sibling();
            self.write(&element, &node.children, live)?;
        }
        Ok(())
    }
}

/// The element mirroring node `id`, made on first sight.
fn mirrored<'a>(
    elements: &'a mut HashMap<u64, Mirrored>,
    document: &Document,
    id: u64,
) -> Result<&'a mut Mirrored, wasm_bindgen::JsValue> {
    if let std::collections::hash_map::Entry::Vacant(entry) = elements.entry(id) {
        let element: HtmlElement = document.create_element("div")?.unchecked_into();
        element.set_attribute(NODE_ATTR, &id.to_string())?;
        element
            .style()
            .set_css_text("position:absolute;margin:0;padding:0;");
        entry.insert(Mirrored {
            element,
            text: None,
            written: Written::default(),
        });
    }
    Ok(elements.get_mut(&id).expect("inserted mirror element"))
}

/// Bring one element to `wanted`, touching only what differs.
fn update(
    document: &Document,
    mirrored: &mut Mirrored,
    wanted: &Written,
) -> Result<(), wasm_bindgen::JsValue> {
    let element = &mirrored.element;
    let old = &mirrored.written;
    for (name, _) in &old.attrs {
        if !wanted.attrs.iter().any(|(key, _)| key == name) {
            element.remove_attribute(name)?;
        }
    }
    for (name, value) in &wanted.attrs {
        if !old
            .attrs
            .iter()
            .any(|(key, old)| key == name && old == value)
        {
            element.set_attribute(name, value)?;
        }
    }
    if old.text != wanted.text {
        match (&mirrored.text, &wanted.text) {
            (Some(node), Some(text)) => node.set_data(text),
            (None, Some(text)) => {
                let node = document.create_text_node(text);
                element.insert_before(&node, element.first_child().as_ref())?;
                mirrored.text = Some(node);
            },
            (Some(node), None) => {
                element.remove_child(node)?;
                mirrored.text = None;
            },
            (None, None) => {},
        }
    }
    if old.rect != wanted.rect {
        let [x, y, width, height] = wanted.rect.unwrap_or_default();
        let style = element.style();
        style.set_property("left", &format!("{x}px"))?;
        style.set_property("top", &format!("{y}px"))?;
        style.set_property("width", &format!("{width}px"))?;
        style.set_property("height", &format!("{height}px"))?;
    }
    if old.focusable != wanted.focusable {
        if wanted.focusable {
            element.set_attribute("tabindex", "-1")?;
        } else {
            element.remove_attribute("tabindex")?;
        }
    }
    Ok(())
}

impl Accessibility for DomAccessibility {
    fn adapter_state(&self) -> A11yAdapterState {
        if self.canvas.is_connected()
            && self.handle.root.is_connected()
            && self
                .canvas
                .parent_node()
                .zip(self.handle.root.parent_node())
                .is_some_and(|(canvas, root)| canvas.is_same_node(Some(&root)))
        {
            A11yAdapterState::Installed
        } else {
            A11yAdapterState::Detached
        }
    }

    fn sync(
        &mut self,
        owner: &cambium_rootstock::A11yOwner,
        dom: &WindowDom<'_>,
        layout: &OwnedLayout,
        leaves: &LeafRegistry<u64>,
        producers: &cambium_rootstock::ProducerRegistry,
        focus: Option<u64>,
        layout_scale: f64,
    ) -> A11ySync {
        let attached = self.adapter_state() == A11yAdapterState::Installed;
        if !owner.is_live() {
            self.clear_publication();
            return A11ySync {
                requests: Vec::new(),
                publication: A11yPublication::skipped(
                    if attached {
                        A11yAdapterState::Installed
                    } else {
                        A11yAdapterState::Detached
                    },
                    A11yPublicationSkip::OwnerEnded,
                ),
            };
        }
        if !attached {
            self.clear_publication();
            return A11ySync {
                requests: Vec::new(),
                publication: A11yPublication::refused(
                    A11yAdapterState::Detached,
                    A11yPublicationRefusal::DetachedMirror,
                ),
            };
        }
        let projection =
            cambium_rootstock::project_semantics(dom, layout, leaves, producers, focus);
        let requests = match projection.publication_bindings(owner, dom, leaves, producers) {
            Ok(bindings) => bindings.requests,
            Err(_) => {
                self.clear_publication();
                return A11ySync {
                    requests: Vec::new(),
                    publication: A11yPublication::refused(
                        A11yAdapterState::Installed,
                        A11yPublicationRefusal::InvalidBindings,
                    ),
                };
            },
        };
        let device = web_sys::window().map_or(1.0, |window| window.device_pixel_ratio());
        let scale = (layout_scale / device) as f32;
        let plan = mirror::plan(&projection, scale);

        if self.cover_canvas().is_err() {
            self.clear_publication();
            return A11ySync {
                requests: Vec::new(),
                publication: A11yPublication::refused(
                    A11yAdapterState::Installed,
                    A11yPublicationRefusal::DomWrite,
                ),
            };
        }
        let root = self.handle.root.clone();
        // Removing a focused proxy (for example a dismissed search field)
        // makes Chrome focus body. Remember ownership before that mutation,
        // while still leaving genuinely external page focus alone.
        let active_before = self.document.active_element();
        let held_before = active_before.as_ref().is_some_and(|active| {
            active.is_same_node(Some(self.canvas.as_ref())) || root.contains(Some(active.as_ref()))
        });
        // Reusing an element would let an old event handle acquire the new
        // contributor's authority. Ordinary updates retain the same element.
        let mut focused_proxy_replaced = false;
        {
            let shared = self.handle.shared.borrow();
            for (id, old) in &shared.requests {
                if requests
                    .get(id)
                    .is_none_or(|new| !old.same_publication_target(new))
                {
                    if let Some(old) = self.elements.remove(id) {
                        focused_proxy_replaced |= shared
                            .focused
                            .as_ref()
                            .is_some_and(|focused| old.element.contains(Some(focused.as_ref())));
                        old.element.remove();
                    }
                }
            }
        }
        let mut live = HashSet::new();
        if self.write(&root, &plan, &mut live).is_err() {
            self.clear_publication();
            return A11ySync {
                requests: Vec::new(),
                publication: A11yPublication::refused(
                    A11yAdapterState::Installed,
                    A11yPublicationRefusal::DomWrite,
                ),
            };
        }
        self.elements.retain(|id, mirrored| {
            let keep = live.contains(id);
            if !keep {
                mirrored.element.remove();
            }
            keep
        });

        let focus = focus.map(|_| projection.tree.focus.0);
        let focused = focus.and_then(|id| self.elements.get(&id).map(|m| m.element.clone()));
        {
            let mut shared = self.handle.shared.borrow_mut();
            shared.elements = requests
                .keys()
                .filter_map(|id| {
                    self.elements
                        .get(id)
                        .map(|element| (*id, element.element.clone()))
                })
                .collect();
            shared.requests = requests;
            shared.focused = focused;
        }
        if focus != self.focus || focused_proxy_replaced {
            self.focus = focus;
            // Only while the application holds the page's focus: a reader
            // elsewhere on the page is not pulled back into it.
            let active = self.document.active_element();
            let holds = active.as_ref().is_some_and(|active| {
                active.is_same_node(Some(self.canvas.as_ref()))
                    || self.handle.root.contains(Some(active.as_ref()))
            });
            if held_before || holds {
                self.handle.take_focus();
            }
        }
        A11ySync {
            requests: Vec::new(),
            publication: A11yPublication {
                adapter: A11yAdapterState::Installed,
                outcome: A11yPublicationOutcome::Mirrored {
                    nodes: live.len(),
                    contribution_issues: projection.producer_issues.len(),
                },
            },
        }
    }
}
