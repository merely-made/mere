extern crate cambium;
use std::{collections::HashMap,rc::Rc,cell::RefCell,sync::Arc};
use accesskit::{Action,ActionData,Affine,NodeId as A11yNodeId,Rect,Role,TreeUpdate};
use cambium_rootstock::{A11yAction,A11yRequest,ProducedAction,ProducerRegistry,ProducerRole,ProducerSemantics,OwnedLayout,WindowDom,document_projection,Host,HostState,HostHooks,HostOptions,HostWake,HostWindow,Runner};
use genet_scripted_dom::{ScriptedDom,NodeId};
use layout_dom_api::{LayoutDom,LocalName,Namespace,NodeKind};
use sprigging::LeafRegistry;
use cambium::{AnyView,GenetCtx,GenetElement,el,button,custom_leaf};
struct Mapping {action_map:HashMap<A11yNodeId,NodeId>,produced_map:HashMap<A11yNodeId,ProducedAction>}

impl Mapping {
    fn map_parts(
        &self,
        action: Action,
        data: Option<ActionData>,
        target: A11yNodeId,
    ) -> Option<A11yRequest> {
        let action = match action {
            Action::Click => A11yAction::Click,
            Action::Focus => A11yAction::Focus,
            Action::SetValue => match data {
                Some(ActionData::NumericValue(value)) if value.is_finite() => {
                    A11yAction::SetValue(value)
                },
                _ => return None,
            },
            _ => return None,
        };
        if let Some(node) = self.action_map.get(&target) {
            return Some(A11yRequest::node(action, *node));
        }
        let produced = self.produced_map.get(&target)?;
        Some(A11yRequest::produced(action, produced.clone()))
    }
}
pub fn project_tree<D: LayoutDom<NodeId = NodeId>>(
    dom: &D,
    layout: &cambium_rootstock::OwnedLayout,
    leaves: &mut LeafRegistry<u64>,
    producers: &mut ProducerRegistry,
    focus: Option<u64>,
) -> (TreeUpdate, HashMap<A11yNodeId, NodeId>) {
    let (tree, nodes, _) = project_tree_with_actions(dom, layout, leaves, producers, focus);
    (tree, nodes)
}

/// [`project_tree`], also returning which drawn node's action each produced
/// button stands for, the map a reader's click on one resolves through.
pub fn project_tree_with_actions<D: LayoutDom<NodeId = NodeId>>(
    dom: &D,
    layout: &cambium_rootstock::OwnedLayout,
    leaves: &mut LeafRegistry<u64>,
    producers: &mut ProducerRegistry,
    focus: Option<u64>,
) -> (
    TreeUpdate,
    HashMap<A11yNodeId, NodeId>,
    HashMap<A11yNodeId, ProducedAction>,
) {
    let root = dom.document();
    let id_of = |d: &D, n: NodeId| A11yNodeId(d.opaque_id(n));
    let focused = focus.and_then(|opaque| find_opaque(dom, root, opaque));
    let mut tree = genet_render::accesskit_tree_with_style(
        dom,
        layout.fragments(),
        focused,
        &genet_render::A11yStyleQueries {
            generated: &|node| layout.generated_text(dom, node),
            rendered: &|node| layout.rendered_visible(dom, node),
        },
    );
    let mut action_map = HashMap::new();
    let mut produced = Vec::new();
    let mut app_textboxes = Vec::new();
    walk(dom, root, &mut |node| {
        let id = id_of(dom, node);
        action_map.insert(id, node);
        if let Some(key) = custom_leaf_key(dom, node)
            && let Some(leaf) = leaves.get_mut(&key)
            && let Some((_, access)) = tree
                .nodes
                .iter_mut()
                .find(|(candidate, _)| *candidate == id)
        {
            leaf.accessibility(access);
        }
        if let Some(key) = custom_leaf_key(dom, node)
            && let Some(semantics) = producers.semantics(key)
        {
            produced.push((id, key, semantics));
        }
        if dom
            .attribute(node, &Namespace::default(), &LocalName::from("role"))
            .is_some_and(|role| role.trim().eq_ignore_ascii_case("textbox"))
            && let Some(value) = dom.attribute(
                node,
                &Namespace::default(),
                &LocalName::from("data-cambium-text-value"),
            )
            && let Some((_, access)) = tree
                .nodes
                .iter_mut()
                .find(|(candidate, _)| *candidate == id)
        {
            app_textboxes.push(id);
            // Cambium paints committed text alongside ephemeral preedit,
            // ghost and caret nodes. The owner marker carries only the model
            // value, so platform accessibility never mistakes painted
            // decoration for the field's editable contents.
            let multiline = dom.attribute(
                node,
                &Namespace::default(),
                &LocalName::from("aria-multiline"),
            ) == Some("true");
            access.set_role(if multiline {
                Role::MultilineTextInput
            } else {
                Role::TextInput
            });
            access.set_value(value.to_owned());
        }
        if dom.attribute(
            node,
            &Namespace::default(),
            &LocalName::from("data-cambium-set-value"),
        ) == Some("true")
            && let Some((_, access)) = tree
                .nodes
                .iter_mut()
                .find(|(candidate, _)| *candidate == id)
        {
            access.add_action(Action::SetValue);
        }
    });
    let mut produced_map = HashMap::new();
    for (slot, key, semantics) in produced {
        add_producer_semantics(&mut tree, slot, key, semantics, &mut produced_map);
    }
    prune_app_textbox_descendants(
        dom,
        &mut tree,
        &mut action_map,
        &mut produced_map,
        &app_textboxes,
    );
    (tree, action_map, produced_map)
}

/// App textboxes expose their committed value as a leaf. Decoration and
/// painted text remain in the drawing DOM, while their accessibility rows and
/// routes are removed after all producer semantics have been added.
fn prune_app_textbox_descendants(
    dom: &impl LayoutDom<NodeId = NodeId>,
    tree: &mut TreeUpdate,
    action_map: &mut HashMap<A11yNodeId, NodeId>,
    produced_map: &mut HashMap<A11yNodeId, ProducedAction>,
    textboxes: &[A11yNodeId],
) {
    let children: HashMap<_, Vec<_>> = tree
        .nodes
        .iter()
        .map(|(id, node)| (*id, node.children().to_vec()))
        .collect();
    let mut hidden = HashMap::new();
    for textbox in textboxes {
        if let Some(owner) = find_opaque(dom, dom.document(), textbox.0) {
            let mut pending: Vec<_> = dom.dom_children(owner).collect();
            while let Some(child) = pending.pop() {
                hidden
                    .entry(A11yNodeId(dom.opaque_id(child)))
                    .or_insert(*textbox);
                pending.extend(dom.dom_children(child));
            }
        }
        let mut pending = children.get(textbox).cloned().unwrap_or_default();
        while let Some(child) = pending.pop() {
            hidden.entry(child).or_insert(*textbox);
            pending.extend(children.get(&child).into_iter().flatten().copied());
        }
        if let Some((_, node)) = tree.nodes.iter_mut().find(|(id, _)| id == textbox) {
            node.set_children(Vec::new());
        }
    }
    if let Some(owner) = hidden.get(&tree.focus) {
        tree.focus = *owner;
    }
    let hidden_ids: std::collections::HashSet<_> = hidden.keys().copied().collect();
    tree.nodes.retain(|(id, _)| !hidden_ids.contains(id));
    action_map.retain(|id, _| !hidden_ids.contains(id));
    produced_map.retain(|id, _| !hidden_ids.contains(id));
}

/// Write a producer's own semantics onto its slot's node, and what it draws
/// as child nodes placed where it draws them. A drawn node's actions become
/// its Button children, each taking Click (`producer` is the slot's registry
/// key, which a reader's click on one names in `produced_map`).
fn add_producer_semantics(
    tree: &mut TreeUpdate,
    slot: A11yNodeId,
    producer: u64,
    semantics: ProducerSemantics,
    produced_map: &mut HashMap<A11yNodeId, ProducedAction>,
) {
    let Some(index) = tree.nodes.iter().position(|(id, _)| *id == slot) else {
        return;
    };
    let origin = {
        let access = &mut tree.nodes[index].1;
        if let Some(role) = semantics.role {
            access.set_role(accesskit_role(role));
        }
        if let Some(name) = semantics.name {
            access.set_label(name);
        }
        access
            .bounds()
            .map_or((0.0, 0.0), |bounds| (bounds.x0, bounds.y0))
    };
    let mut children = Vec::new();
    let mut buttons = Vec::new();
    for child in semantics.children {
        let id = produced_node_id(slot, child.key, None);
        let mut access = accesskit::Node::new(accesskit_role(child.role));
        access.set_label(child.name);
        let [x, y, width, height] = child.rect.map(f64::from);
        let bounds = Rect {
            x0: origin.0 + x,
            y0: origin.1 + y,
            x1: origin.0 + x + width,
            y1: origin.1 + y + height,
        };
        access.set_bounds(bounds);
        for action in child.actions {
            let button_id = produced_node_id(slot, child.key, Some(&action.id));
            let mut button = accesskit::Node::new(Role::Button);
            button.set_label(action.label);
            button.set_description(action.description);
            button.set_bounds(bounds);
            button.add_action(Action::Click);
            button.add_action(Action::Focus);
            access.push_child(button_id);
            produced_map.insert(
                button_id,
                ProducedAction {
                    slot: producer,
                    key: child.key,
                    id: action.id,
                },
            );
            buttons.push((button_id, button));
        }
        children.push((id, access));
    }
    let ids: Vec<A11yNodeId> = children.iter().map(|(id, _)| *id).collect();
    for id in &ids {
        tree.nodes[index].1.push_child(*id);
    }
    tree.nodes.extend(children);
    tree.nodes.extend(buttons);
}

/// A drawn node's id, or its action button's: stable for the slot, the
/// node's key and the action, however the producer orders its nodes, in a
/// range DOM ids never reach.
fn produced_node_id(slot: A11yNodeId, key: u64, action: Option<&str>) -> A11yNodeId {
    // FNV-1a: fixed, so an id is the same on every frame and every run.
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    let mut eat = |bytes: &[u8]| {
        for byte in bytes {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(0x0100_0000_01b3);
        }
    };
    eat(&slot.0.to_le_bytes());
    eat(&key.to_le_bytes());
    if let Some(action) = action {
        eat(&[1]);
        eat(action.as_bytes());
    }
    A11yNodeId((1 << 63) | (hash >> 1))
}

fn accesskit_role(role: ProducerRole) -> Role {
    match role {
        ProducerRole::List => Role::List,
        ProducerRole::ListItem => Role::ListItem,
        ProducerRole::Group => Role::Group,
        ProducerRole::Image => Role::Image,
        ProducerRole::GraphicsObject => Role::GraphicsObject,
    }
}

/// Stamp the host's layout scale on the tree root.
///
/// Layout bounds are layout-space CSS pixels, and AccessKit expects the final
/// transformed coordinates to be the platform's physical client pixels. The
/// Windows adapter applies no DPI conversion of its own, so without this a
/// screen reader or UI Automation client at 125% is told every control sits
/// at four fifths of its true position.
///
/// `layout_scale` is the device scale **times the UI zoom**, because both
/// separate the two spaces and a reader has no way to know about either. Pass
/// [`Host::layout_scale`](cambium_rootstock::Host::layout_scale).
pub fn scale_tree_to_window<D: LayoutDom<NodeId = NodeId>>(
    tree: &mut TreeUpdate,
    dom: &D,
    layout_scale: f64,
) {
    let root = A11yNodeId(dom.opaque_id(dom.document()));
    if let Some((_, node)) = tree.nodes.iter_mut().find(|(id, _)| *id == root) {
        node.set_transform(Affine::scale(layout_scale));
    }
}

fn walk<D: LayoutDom<NodeId = NodeId>>(dom: &D, node: NodeId, visit: &mut impl FnMut(NodeId)) {
    visit(node);
    for child in dom.dom_children(node) {
        walk(dom, child, visit);
    }
}

fn find_opaque<D: LayoutDom<NodeId = NodeId>>(
    dom: &D,
    node: NodeId,
    opaque: u64,
) -> Option<NodeId> {
    if dom.opaque_id(node) == opaque {
        return Some(node);
    }
    dom.dom_children(node)
        .find_map(|child| find_opaque(dom, child, opaque))
}

fn custom_leaf_key<D: LayoutDom<NodeId = NodeId>>(dom: &D, node: NodeId) -> Option<u64> {
    if dom.kind(node) != NodeKind::Element
        || !matches!(
            dom.element_name(node)?.local.as_ref(),
            "custom-leaf" | "chisel-leaf"
        )
    {
        return None;
    }
    dom.attribute(node, &Namespace::default(), &LocalName::from("key"))?
        .parse()
        .ok()
}

mod web_mirror {
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
    /// For a drawn node's action button, the action a reader's click on it
    /// names (dynamics grammar plan, F65 and F66).
    pub action: Option<cambium_rootstock::ProducedAction>,
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
    /// The producer's registry key, which a click on one of its drawn
    /// nodes' action buttons names; `None` for a sprigging leaf.
    pub slot: Option<u64>,
}

/// One thing a leaf draws, as the mirror places it.
#[derive(Clone, Debug, PartialEq)]
pub struct LeafChild {
    /// The producer's stable name for it.
    pub key: u64,
    pub role: &'static str,
    pub name: String,
    pub rect: [f32; 4],
    /// Each lowered to a button child.
    pub actions: Vec<cambium_rootstock::ProducerAction>,
}

impl LeafSemantics {
    /// A producer's own account of its slot, the producer registered under
    /// `slot`.
    pub fn from_producer(slot: u64, semantics: cambium_rootstock::ProducerSemantics) -> Self {
        Self {
            role: semantics.role.map(cambium_rootstock::ProducerRole::aria),
            name: semantics.name,
            children: semantics
                .children
                .into_iter()
                .map(|child| LeafChild {
                    key: child.key,
                    role: child.role.aria(),
                    name: child.name,
                    rect: child.rect,
                    actions: child.actions,
                })
                .collect(),
            names_itself: true,
            slot: Some(slot),
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
        slot: None,
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
    let slot = semantics.as_ref().and_then(|semantics| semantics.slot);
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
            .map(|child| drawn_node(id, slot, child, scale)),
    );
    MirrorNode {
        id,
        attrs,
        text,
        rect,
        focusable: node.actions.contains(&DocumentA11yAction::Focus),
        action: None,
        children,
    }
}

/// One thing a leaf draws, as a mirror element over where it is drawn, its
/// actions as button children over the same box (F65). Ids are stable for
/// the leaf, the node's key and the action, and sit in a range a DOM node's
/// opaque id never reaches.
fn drawn_node(leaf: u64, slot: Option<u64>, child: LeafChild, scale: f32) -> MirrorNode {
    let [x, y, width, height] = child.rect;
    let mut attrs = vec![("role", child.role.to_string())];
    let text = if child.role == "listitem" && child.actions.is_empty() {
        Some(child.name)
    } else {
        attrs.push(("aria-label", child.name));
        None
    };
    let key = child.key;
    let buttons = child
        .actions
        .into_iter()
        .map(|action| {
            let mut attrs = vec![("role", "button".to_string())];
            if !action.description.trim().is_empty() {
                attrs.push(("aria-description", action.description));
            }
            MirrorNode {
                id: drawn_id(leaf, key, Some(&action.id)),
                attrs,
                text: Some(action.label),
                rect: Some([0.0, 0.0, width * scale, height * scale]),
                focusable: true,
                action: slot.map(|slot| cambium_rootstock::ProducedAction {
                    slot,
                    key,
                    id: action.id,
                }),
                children: Vec::new(),
            }
        })
        .collect();
    MirrorNode {
        id: drawn_id(leaf, key, None),
        attrs,
        text,
        rect: Some([x * scale, y * scale, width * scale, height * scale]),
        focusable: false,
        action: None,
        children: buttons,
    }
}

/// A drawn node's mirror id, or its action button's: FNV-1a over the leaf,
/// the key and the action, fixed so an element keeps its id frame to frame.
fn drawn_id(leaf: u64, key: u64, action: Option<&str>) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    let mut eat = |bytes: &[u8]| {
        for byte in bytes {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(0x0100_0000_01b3);
        }
    };
    eat(&leaf.to_le_bytes());
    eat(&key.to_le_bytes());
    if let Some(action) = action {
        eat(&[1]);
        eat(action.as_bytes());
    }
    (1 << 63) | (hash >> 1)
}

/// Every drawn node's action button in `nodes` and their descendants: its
/// mirror id and the action a click on it names.
pub fn produced_actions(nodes: &[MirrorNode]) -> Vec<(u64, cambium_rootstock::ProducedAction)> {
    let mut out = Vec::new();
    let mut stack: Vec<&MirrorNode> = nodes.iter().collect();
    while let Some(node) = stack.pop() {
        if let Some(action) = &node.action {
            out.push((node.id, action.clone()));
        }
        stack.extend(node.children.iter());
    }
    out
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
        | R::ListBoxOption
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

}

#[derive(Default)]struct State{pane:bool,hidden:bool,clicks:usize}
type Child=Box<dyn AnyView<State,(),GenetCtx,GenetElement>>;
type Logic=fn(&State)->Child;
fn view(s:&State)->Child{
    let target=button("Target",|s:&mut State,_|s.clicks+=1).attr("id","target").attr("style",if s.hidden{"display:none;"}else{"display:block;width:100px;height:40px;"});
    if s.pane {Box::new(el("div",(el("div",()).attr("style","height:300px;"),target)).attr("id","pane").attr("style","display:block;width:280px;height:120px;overflow:auto;"))}
    else {Box::new(el("div",(el("div",()).attr("style","height:300px;"),target,el("div",()).attr("style","height:500px;"))))}
}
struct Window;
impl HostWindow for Window{
    fn request_redraw(&self){} fn inner_size(&self)->(u32,u32){(400,200)} fn scale_factor(&self)->f64{1.}
    fn set_ime_allowed(&self,_:bool){} fn set_ime_cursor_area(&self,_:f64,_:f64,_:f64,_:f64){}
}
fn host(pane:bool)->Host<State,Logic,Child>{
    let mut s=HostState::new();s.shared.sheet="body { margin:0; }".into();s.window=Some(Box::new(Window));
    s.runner=Some(Runner::new(Rc::new(RefCell::new(ScriptedDom::new())),view as Logic,State{pane,..Default::default()}));
    let wake=HostWake::new(s.wake_pending.clone(),Arc::new(||{}));
    let mut host=Host::new(HostOptions::default(),None,HostHooks::inert(),s,wake);
    host.set_surface_size(400.,200.);host.relayout(400.,200.);host
}
fn by_id(dom:&ScriptedDom,n:NodeId,id:&str)->Option<NodeId>{
    if dom.attribute(n,&Namespace::from(""),&LocalName::from("id"))==Some(id){return Some(n)}
    dom.dom_children(n).find_map(|c|by_id(dom,c,id))
}
fn compare_scroll(pane:bool){
    let mut h=host(pane);h.pointer_moved(20.,20.);h.wheel(0.,150.);
    let dom=h.s.runner.as_ref().unwrap().dom();let dom=dom.borrow();let n=by_id(&dom,dom.document(),"target").unwrap();
    let layout=h.s.layout.as_ref().unwrap();let painted=layout.painted_rect(&*dom,n).unwrap();
    let neutral=document_projection(&WindowDom::document(&dom),layout,None);
    let neutral=neutral.nodes().iter().find(|row|row.id.get()==dom.opaque_id(n)).unwrap().bounds.unwrap();
    let (native,_,_)=project_tree_with_actions(&*dom,layout,&mut LeafRegistry::new(),&mut ProducerRegistry::new(),None);
    let native=native.nodes.iter().find(|(id,_)|id.0==dom.opaque_id(n)).unwrap().1.bounds().unwrap();
    println!("pane={pane} painted={painted:?} neutral={neutral:?} native={native:?}");
    assert!((painted.1-150.).abs()<0.01);assert!((neutral.y-painted.1).abs()<0.01);
    assert!((native.y0-300.).abs()<0.01);assert!((native.y0-f64::from(painted.1)).abs()>149.);
}
#[test]fn viewport_scroll_native_and_neutral_bounds_disagree(){compare_scroll(false)}
#[test]fn nested_scroll_native_and_neutral_bounds_disagree(){compare_scroll(true)}
#[test]fn hidden_node_missing_from_tree_remains_mappable_and_activates(){
    let mut h=host(false);h.s.runner.as_mut().unwrap().update(|s|s.hidden=true);h.relayout(400.,200.);
    let request={
        let dom=h.s.runner.as_ref().unwrap().dom();let dom=dom.borrow();let n=by_id(&dom,dom.document(),"target").unwrap();let id=A11yNodeId(dom.opaque_id(n));
        let (tree,action_map,produced_map)=project_tree_with_actions(&*dom,h.s.layout.as_ref().unwrap(),&mut LeafRegistry::new(),&mut ProducerRegistry::new(),None);
        assert!(!tree.nodes.iter().any(|(candidate,_)|*candidate==id));assert!(action_map.contains_key(&id));
        Mapping{action_map,produced_map}.map_parts(Action::Click,None,id).unwrap()
    };
    h.apply_a11y_requests(&[request]);assert_eq!(h.s.runner.as_ref().unwrap().state().clicks,1);
}
#[test]fn leaf_author_name_differs_between_native_and_browser_lowering(){
    use layout_dom_api::LayoutDomMut;
    struct Leaf;
    impl sprigging::Leaf for Leaf{
        fn measure(&mut self,_:sprigging::SizeHint,_:sprigging::SizeHint)->sprigging::Size{sprigging::Size{width:100.,height:40.}}
        fn paint(&mut self,_:&mut sprigging::PaintCx<'_>){}
        fn paint_dirty(&self)->bool{false}
        fn accessibility(&mut self,n:&mut accesskit::Node){n.set_label("Leaf name")}
    }
    let mut dom=ScriptedDom::new();let root=dom.document();dom.set_inner_html(root,r#"<custom-leaf key="7" aria-label="Author name" style="display:block;width:100px;height:40px;"></custom-leaf>"#);
    let layout=OwnedLayout::new(&dom,&[""],400.,200.,&[],&Default::default());
    let mut leaves=LeafRegistry::new();leaves.insert(7,Box::new(Leaf));
    let (tree,_)=project_tree(&dom,&layout,&mut leaves,&mut ProducerRegistry::new(),None);
    assert!(tree.nodes.iter().any(|(_,n)|n.label()==Some("Leaf name")));
    let neutral=document_projection(&WindowDom::document(&dom),&layout,None);
    let mirror=web_mirror::plan(&neutral,1.,|_,name| (name==Some("Author name")).then(||web_mirror::LeafSemantics{role:None,name:Some("Leaf name".into()),..Default::default()}));
    fn find(ns:&[web_mirror::MirrorNode])->bool{ns.iter().any(|n|n.attrs.iter().any(|(k,v)|*k=="aria-label" && v=="Author name") || n.text.as_deref()==Some("Author name") || find(&n.children))}
    assert!(find(&mirror));
}
