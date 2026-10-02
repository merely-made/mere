// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The accessibility mirror's plan: genet's neutral projection lowered to
//! WAI-ARIA, one element per projected node.
//!
//! The projection's roles and states were read from the tree's own ARIA and
//! HTML semantics, so this lowering is close to an identity inside ARIA's
//! vocabulary. A name goes where ARIA looks for it: a value control or an
//! author-named container carries it in `aria-label`, and anything named from
//! its content carries it as its own text. A leaf, which describes itself
//! only through AccessKit, contributes a role and a name through
//! [`LeafSemantics`].
//!
//! Pure, and compiled on every target, so it is tested without a browser.
//! `a11y` applies the plan to DOM elements.

use std::collections::HashMap;

use document_session_api::{
    DocumentA11yAction, DocumentA11yHasPopup, DocumentA11yLive, DocumentA11yNode,
    DocumentA11yNodeId, DocumentA11yOrientation, DocumentA11yProjection, DocumentA11yRole,
    DocumentA11yToggled,
};

/// The projection's nodes by id, built once per plan.
type Nodes<'a> = HashMap<DocumentA11yNodeId, &'a DocumentA11yNode>;

/// One mirror element: what the DOM element for one projected node carries.
#[derive(Clone, Debug, PartialEq)]
pub struct MirrorNode {
    /// The projected node's id, the DOM node's opaque id.
    pub id: u64,
    /// ARIA attributes, `role` first when there is one.
    pub attrs: Vec<(&'static str, String)>,
    /// The element's own text, ahead of its children.
    pub text: Option<String>,
    /// `[x, y, width, height]` in CSS pixels, relative to the nearest mirrored
    /// ancestor's box, or to the canvas at the top.
    pub rect: Option<[f32; 4]>,
    /// Whether a reader may move focus to it.
    pub focusable: bool,
    pub children: Vec<MirrorNode>,
}

/// What a leaf says of itself: an ARIA role and a name.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LeafSemantics {
    pub role: Option<&'static str>,
    pub name: Option<String>,
    /// What the leaf draws, each with its ARIA role, its name and where it is
    /// drawn (`[x, y, width, height]` in the leaf's layout pixels).
    pub children: Vec<LeafChild>,
    /// Whether this name replaces the author's. A producer speaks for its
    /// slot; a sprigging leaf's own label yields to an author's name.
    pub names_itself: bool,
}

/// One thing a leaf draws, as the mirror places it.
#[derive(Clone, Debug, PartialEq)]
pub struct LeafChild {
    pub role: &'static str,
    pub name: String,
    pub rect: [f32; 4],
}

impl LeafSemantics {
    /// A producer's own account of its slot.
    pub fn from_producer(semantics: cambium_rootstock::ProducerSemantics) -> Self {
        Self {
            role: semantics.role.map(cambium_rootstock::ProducerRole::aria),
            name: semantics.name,
            children: semantics
                .children
                .into_iter()
                .map(|child| LeafChild {
                    role: child.role.aria(),
                    name: child.name,
                    rect: child.rect,
                })
                .collect(),
            names_itself: true,
        }
    }
}

/// A leaf's AccessKit account of itself, in ARIA. A graphic keeps its role
/// from WAI-ARIA's Graphics Module, and its label becomes its name.
pub fn leaf_semantics(node: &accesskit::Node) -> LeafSemantics {
    use accesskit::Role;
    let role = match node.role() {
        Role::GraphicsDocument => Some("graphics-document"),
        Role::GraphicsObject => Some("graphics-object"),
        Role::GraphicsSymbol => Some("graphics-symbol"),
        Role::Image | Role::Canvas => Some("img"),
        _ => None,
    };
    LeafSemantics {
        role,
        name: node.label().map(str::to_owned),
        children: Vec::new(),
        names_itself: false,
    }
}

/// Lower `projection` to mirror elements. `scale` carries layout pixels into
/// CSS pixels. `leaf` answers for a node that is a leaf, given its projected
/// name, and returns `None` for every other node.
pub fn plan(
    projection: &DocumentA11yProjection,
    scale: f32,
    mut leaf: impl FnMut(u64, Option<&str>) -> Option<LeafSemantics>,
) -> Vec<MirrorNode> {
    let nodes: Nodes = projection
        .nodes()
        .iter()
        .map(|node| (node.id, node))
        .collect();
    let Some(root) = nodes.get(&projection.root()) else {
        return Vec::new();
    };
    // The root stands for the canvas, so its children sit at the top.
    lower_children(&nodes, root, [0.0, 0.0], scale, &mut leaf)
}

fn lower_children(
    nodes: &Nodes,
    parent: &DocumentA11yNode,
    origin: [f32; 2],
    scale: f32,
    leaf: &mut impl FnMut(u64, Option<&str>) -> Option<LeafSemantics>,
) -> Vec<MirrorNode> {
    parent
        .children
        .iter()
        .filter_map(|id| nodes.get(id))
        .filter(|node| !node.state.hidden)
        .map(|node| lower(nodes, node, origin, scale, leaf))
        .collect()
}

fn lower(
    nodes: &Nodes,
    node: &DocumentA11yNode,
    origin: [f32; 2],
    scale: f32,
    leaf: &mut impl FnMut(u64, Option<&str>) -> Option<LeafSemantics>,
) -> MirrorNode {
    let id = node.id.get();
    let mut semantics = leaf(id, node.name.as_deref());
    let drawn = semantics
        .as_mut()
        .map(|semantics| std::mem::take(&mut semantics.children))
        .unwrap_or_default();
    let role = semantics
        .as_ref()
        .and_then(|semantics| semantics.role)
        .or_else(|| aria_role(node.role));
    // A producer names its slot; otherwise the author's name wins.
    let names_itself = semantics
        .as_ref()
        .is_some_and(|semantics| semantics.names_itself);
    let name = if !names_itself {
        node.name
            .clone()
            .or_else(|| semantics.and_then(|semantics| semantics.name))
    } else {
        semantics
            .and_then(|semantics| semantics.name)
            .or_else(|| node.name.clone())
    }
    .filter(|name| !name.trim().is_empty());

    let mut attrs = Vec::new();
    if let Some(role) = role {
        attrs.push(("role", role.to_string()));
    }
    let mut text = None;
    match (placement(node.role, role), name) {
        (Placement::Label | Placement::Value, Some(name)) => attrs.push(("aria-label", name)),
        (Placement::Content, Some(name)) => text = Some(name),
        (_, None) => {},
    }
    // A description says more than the name: what a control does, say.
    if let Some(description) = node
        .description
        .clone()
        .filter(|description| !description.trim().is_empty())
    {
        attrs.push(("aria-description", description));
    }
    // A text control's content is its value, which is where a reader reads it.
    if matches!(
        node.role,
        DocumentA11yRole::TextField | DocumentA11yRole::ComboBox
    ) {
        text = node.value.clone().filter(|value| !value.is_empty());
    }
    attrs.extend(states(node));

    let (rect, child_origin) = match node.bounds {
        Some(bounds) => (
            Some([
                (bounds.x - origin[0]) * scale,
                (bounds.y - origin[1]) * scale,
                bounds.width * scale,
                bounds.height * scale,
            ]),
            [bounds.x, bounds.y],
        ),
        None => (None, origin),
    };
    let mut children = lower_children(nodes, node, child_origin, scale, leaf);
    children.extend(
        drawn
            .into_iter()
            .enumerate()
            .map(|(ordinal, child)| drawn_node(id, ordinal, child, scale)),
    );
    MirrorNode {
        id,
        attrs,
        text,
        rect,
        focusable: node.actions.contains(&DocumentA11yAction::Focus),
        children,
    }
}

/// One thing a leaf draws, as a mirror element over where it is drawn. Its
/// id sits in a range a DOM node's opaque id never reaches.
fn drawn_node(leaf: u64, ordinal: usize, child: LeafChild, scale: f32) -> MirrorNode {
    let [x, y, width, height] = child.rect;
    let mut attrs = vec![("role", child.role.to_string())];
    let text = if child.role == "listitem" {
        Some(child.name)
    } else {
        attrs.push(("aria-label", child.name));
        None
    };
    MirrorNode {
        id: (1 << 63) | (leaf << 20) | (ordinal as u64 & 0xF_FFFF),
        attrs,
        text,
        rect: Some([x * scale, y * scale, width * scale, height * scale]),
        focusable: false,
        children: Vec::new(),
    }
}

/// Where ARIA looks for a role's name.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Placement {
    /// Named by the author, never from content: `aria-label`.
    Label,
    /// Named from content: the element's own text.
    Content,
    /// A control whose content is its value: `aria-label` for the name.
    Value,
}

fn placement(role: DocumentA11yRole, aria: Option<&str>) -> Placement {
    use DocumentA11yRole as R;
    if matches!(
        aria,
        Some("graphics-document" | "graphics-object" | "graphics-symbol" | "img")
    ) {
        return Placement::Label;
    }
    match role {
        R::TextField | R::ComboBox | R::Slider | R::SpinButton | R::Splitter => Placement::Value,
        R::Document
        | R::Article
        | R::Region
        | R::Group
        | R::Navigation
        | R::Main
        | R::Form
        | R::Dialog
        | R::List
        | R::ListBox
        | R::Table
        | R::Row
        | R::TabList
        | R::TabPanel
        | R::Tree
        | R::Menu
        | R::Toolbar
        | R::RadioGroup
        | R::ProgressIndicator
        | R::Image => Placement::Label,
        _ => Placement::Content,
    }
}

/// The ARIA role a projected role lowers to. `None` leaves a generic element.
pub fn aria_role(role: DocumentA11yRole) -> Option<&'static str> {
    use DocumentA11yRole as R;
    Some(match role {
        R::Document => "document",
        R::Article => "article",
        R::Region => "region",
        R::Group => "group",
        R::Navigation => "navigation",
        R::Main => "main",
        R::Heading { .. } => "heading",
        R::Paragraph => "paragraph",
        R::Link => "link",
        R::Button => "button",
        R::TextField => "textbox",
        R::CheckBox => "checkbox",
        R::RadioButton => "radio",
        R::RadioGroup => "radiogroup",
        R::Switch => "switch",
        R::ComboBox => "combobox",
        R::List => "list",
        R::ListItem => "listitem",
        R::ListBox => "listbox",
        R::ListBoxOption => "option",
        R::Table => "table",
        R::Row => "row",
        R::Cell => "cell",
        R::Image => "img",
        R::Form => "form",
        R::Dialog => "dialog",
        R::Alert => "alert",
        R::Menu => "menu",
        R::MenuItem => "menuitem",
        R::MenuItemCheckBox => "menuitemcheckbox",
        R::MenuItemRadio => "menuitemradio",
        R::TabList => "tablist",
        R::Tab => "tab",
        R::TabPanel => "tabpanel",
        R::Tree => "tree",
        R::TreeItem => "treeitem",
        R::Slider => "slider",
        R::SpinButton => "spinbutton",
        R::Splitter => "separator",
        R::Toolbar => "toolbar",
        R::ProgressIndicator => "progressbar",
        R::Status => "status",
        R::Log => "log",
        R::Note => "note",
        R::Window | R::StaticText | R::Label | R::Unknown => return None,
    })
}

/// A node's states as ARIA attributes, each only where the projection says
/// something.
fn states(node: &DocumentA11yNode) -> Vec<(&'static str, String)> {
    let state = &node.state;
    let flag = |on: bool| on.then(|| "true".to_string());
    let bool_text = |value: bool| value.to_string();
    let mut attrs = Vec::new();
    let mut push = |name: &'static str, value: Option<String>| {
        if let Some(value) = value {
            attrs.push((name, value));
        }
    };
    if let DocumentA11yRole::Heading { level } = node.role
        && level > 0
    {
        push("aria-level", Some(level.to_string()));
    }
    push("aria-disabled", flag(state.disabled));
    push("aria-selected", state.selected.map(bool_text));
    push("aria-expanded", state.expanded.map(bool_text));
    // A toggle button is pressed; anything else that toggles is checked.
    let toggled = state.toggled.map(|toggled| {
        match toggled {
            DocumentA11yToggled::On => "true",
            DocumentA11yToggled::Off => "false",
            DocumentA11yToggled::Mixed => "mixed",
        }
        .to_string()
    });
    match (node.role, state.checked, toggled) {
        (DocumentA11yRole::Button, _, Some(pressed)) => push("aria-pressed", Some(pressed)),
        (_, Some(checked), _) => push("aria-checked", Some(checked.to_string())),
        (_, None, toggled) => push("aria-checked", toggled),
    }
    if node.role == DocumentA11yRole::TextField {
        push("aria-multiline", flag(state.multiline));
    }
    push("aria-readonly", flag(state.read_only));
    push("aria-required", flag(state.required));
    push(
        "aria-live",
        state.live.map(|live| {
            match live {
                DocumentA11yLive::Off => "off",
                DocumentA11yLive::Polite => "polite",
                DocumentA11yLive::Assertive => "assertive",
            }
            .to_string()
        }),
    );
    push(
        "aria-orientation",
        state.orientation.map(|orientation| {
            match orientation {
                DocumentA11yOrientation::Horizontal => "horizontal",
                DocumentA11yOrientation::Vertical => "vertical",
            }
            .to_string()
        }),
    );
    push(
        "aria-haspopup",
        state.has_popup.map(|popup| {
            match popup {
                DocumentA11yHasPopup::Menu => "menu",
                DocumentA11yHasPopup::ListBox => "listbox",
                DocumentA11yHasPopup::Tree => "tree",
                DocumentA11yHasPopup::Grid => "grid",
                DocumentA11yHasPopup::Dialog => "dialog",
            }
            .to_string()
        }),
    );
    push(
        "aria-valuenow",
        node.numeric_value.map(|value| value.to_string()),
    );
    push(
        "aria-valuemin",
        node.numeric_minimum.map(|value| value.to_string()),
    );
    push(
        "aria-valuemax",
        node.numeric_maximum.map(|value| value.to_string()),
    );
    attrs
}

/// Every id in `nodes` and their descendants, parents first.
pub fn ids(nodes: &[MirrorNode]) -> Vec<u64> {
    let mut out = Vec::new();
    let mut stack: Vec<&MirrorNode> = nodes.iter().rev().collect();
    while let Some(node) = stack.pop() {
        out.push(node.id);
        stack.extend(node.children.iter().rev());
    }
    out
}

#[cfg(test)]
mod tests;
