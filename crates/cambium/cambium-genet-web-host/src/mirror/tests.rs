/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use cambium_genet_winit_host::Harness;
use document_session_api::{
    A11yCapability, DocumentA11yBounds, DocumentA11yNode, DocumentA11yNodeId,
    DocumentA11yProjection, DocumentA11yRole, DocumentA11yState, DocumentA11ySupport,
    DocumentA11yToggled,
};
use genet_scripted_dom::{NodeId, ScriptedDom};
use layout_dom_api::{LayoutDom, LayoutDomMut, LocalName, Namespace};

use super::*;

#[path = "../../examples/a11y_page/page.rs"]
mod page;

use page::{Child, Page};

type Logic = fn(&Page) -> Child;

/// Every node, parents first.
fn flat(nodes: &[MirrorNode]) -> Vec<&MirrorNode> {
    let mut out = Vec::new();
    let mut stack: Vec<&MirrorNode> = nodes.iter().rev().collect();
    while let Some(node) = stack.pop() {
        out.push(node);
        stack.extend(node.children.iter().rev());
    }
    out
}

fn attr<'a>(node: &'a MirrorNode, name: &str) -> Option<&'a str> {
    node.attrs
        .iter()
        .find(|(key, _)| *key == name)
        .map(|(_, value)| value.as_str())
}

/// A node's name as a reader hears it: `aria-label`, else its own text.
fn name(node: &MirrorNode) -> Option<&str> {
    attr(node, "aria-label").or(node.text.as_deref())
}

fn with_role<'a>(nodes: &'a [MirrorNode], role: &str) -> Vec<&'a MirrorNode> {
    flat(nodes)
        .into_iter()
        .filter(|node| attr(node, "role") == Some(role))
        .collect()
}

fn laid_out_page() -> Harness<Page, Logic, Child> {
    let mut h = Harness::new(page::SHEET, Page::default(), page::page as Logic);
    h.layout_at(800.0, 600.0);
    h
}

fn element_with_id(dom: &ScriptedDom, node: NodeId, id: &str) -> Option<NodeId> {
    if dom.attribute(node, &Namespace::from(""), &LocalName::from("id")) == Some(id) {
        return Some(node);
    }
    dom.dom_children(node)
        .find_map(|child| element_with_id(dom, child, id))
}

#[test]
fn each_control_on_the_page_lowers_to_its_aria_role_and_name() {
    let h = laid_out_page();
    let mirror = plan(&h.a11y_projection(), 1.0, |_, _| None);

    let named = |role: &str| -> Vec<Option<String>> {
        with_role(&mirror, role)
            .into_iter()
            .map(|node| name(node).map(str::to_owned))
            .collect()
    };
    assert_eq!(named("heading"), [Some("Accessibility page".into())]);
    assert_eq!(
        named("button"),
        [Some("Press".into()), Some("Open a file".into())]
    );
    assert_eq!(named("textbox"), [Some("Name".into())]);
    assert_eq!(named("checkbox"), [Some("Subscribe".into())]);
    assert_eq!(named("combobox"), [Some("Colour".into())]);
    assert_eq!(named("tablist"), [Some("Sections".into())]);
    assert_eq!(
        named("tab"),
        [Some("One".into()), Some("Two".into()), Some("Three".into())]
    );
    assert_eq!(
        named("listitem"),
        [
            Some("Alpha".into()),
            Some("Beta".into()),
            Some("Gamma".into())
        ]
    );
    assert_eq!(named("status"), [Some(page::status(&Page::default()))]);

    let checkbox = with_role(&mirror, "checkbox")[0];
    assert_eq!(attr(checkbox, "aria-checked"), Some("false"));
    let selected: Vec<Option<&str>> = with_role(&mirror, "tab")
        .into_iter()
        .map(|tab| attr(tab, "aria-selected"))
        .collect();
    assert_eq!(selected, [Some("true"), Some("false"), Some("false")]);
    assert!(
        with_role(&mirror, "button")[0].focusable,
        "a button takes a reader's focus"
    );
}

#[test]
fn app_textbox_marker_runs_through_the_browser_mirror_as_a_leaf() {
    let mut dom = ScriptedDom::new();
    let root = dom.document();
    dom.set_inner_html(
        root,
        "<div id=\"app-field\" role=\"textbox\" aria-label=\"Draft\" aria-multiline=\"true\" data-cambium-text-value=\"Café 👩🏽‍🚀\nSecond line\" style=\"display:block;width:240px;height:40px\">painted value<span> ghost</span><span>preedit</span><span>│</span></div><div id=\"ordinary-field\" role=\"textbox\" aria-label=\"Ordinary\" style=\"display:block;width:240px;height:32px\">ordinary<span> child</span></div>",
    );
    let app_field = element_with_id(&dom, dom.document(), "app-field")
        .expect("the fixture has its marked app textbox");
    let ordinary_field = element_with_id(&dom, dom.document(), "ordinary-field")
        .expect("the fixture has its ordinary textbox");
    let app_id = dom.opaque_id(app_field);
    let ordinary_id = dom.opaque_id(ordinary_field);
    let drawn_children: Vec<_> = dom.dom_children(app_field).map(|child| dom.opaque_id(child)).collect();
    let layout =
        cambium_rootstock::OwnedLayout::new(&dom, &[""], 320.0, 180.0, &[], &Default::default());
    let projection = cambium_rootstock::document_projection(
        &cambium_rootstock::WindowDom::document(&dom),
        &layout,
        Some(app_id),
    );
    let mirror = plan(&projection, 1.0, |_, _| None);
    let flat_mirror = flat(&mirror);
    let app = flat_mirror
        .iter()
        .find(|node| node.id == app_id)
        .expect("the app textbox is mirrored");
    assert_eq!(attr(app, "role"), Some("textbox"));
    assert_eq!(attr(app, "aria-label"), Some("Draft"));
    assert_eq!(app.text.as_deref(), Some("Café 👩🏽‍🚀\nSecond line"));
    assert!(app.children.is_empty(), "the app textbox is a mirror leaf");
    assert!(app.focusable, "the app textbox keeps its focus action");
    assert!(app.rect.is_some(), "the textbox keeps its painted bounds");
    assert_eq!(
        dom.dom_children(app_field).map(|child| dom.opaque_id(child)).collect::<Vec<_>>(),
        drawn_children,
        "the browser accessibility pipeline leaves drawing DOM children intact"
    );

    let ordinary = flat_mirror
        .iter()
        .find(|node| node.id == ordinary_id)
        .expect("the ordinary textbox remains mirrored");
    assert_eq!(attr(ordinary, "role"), Some("textbox"));
    assert!(
        !ordinary.children.is_empty(),
        "unmarked textboxes keep their generic descendants"
    );
}

#[test]
fn a_box_sits_where_the_layout_painted_its_node() {
    let h = laid_out_page();
    let press = h
        .with_dom(|dom| element_with_id(dom, dom.document(), "press"))
        .expect("the page has its button");
    let (x, y, width, height) = h.painted_rect(press).expect("the button paints");

    // Walk down to the button, adding each ancestor's origin.
    fn absolute(nodes: &[MirrorNode], id: u64, origin: [f32; 2]) -> Option<[f32; 4]> {
        nodes.iter().find_map(|node| {
            let rect = node.rect.unwrap_or([0.0, 0.0, 0.0, 0.0]);
            let here = [origin[0] + rect[0], origin[1] + rect[1]];
            if node.id == id {
                Some([here[0], here[1], rect[2], rect[3]])
            } else {
                absolute(&node.children, id, here)
            }
        })
    }
    let id = h.with_dom(|dom| dom.opaque_id(press));
    for scale in [1.0, 2.0] {
        let mirror = plan(&h.a11y_projection(), scale, |_, _| None);
        let rect = absolute(&mirror, id, [0.0, 0.0]).expect("the button is mirrored");
        let expected = [x * scale, y * scale, width * scale, height * scale];
        for (got, want) in rect.iter().zip(expected) {
            assert!((got - want).abs() < 0.01, "{rect:?} against {expected:?}");
        }
    }
}

fn blank(id: u64, role: DocumentA11yRole) -> DocumentA11yNode {
    DocumentA11yNode {
        id: DocumentA11yNodeId::new(id),
        parent: None,
        children: Vec::new(),
        role,
        description: None,
        name: None,
        value: None,
        numeric_value: None,
        numeric_minimum: None,
        numeric_maximum: None,
        bounds: None,
        state: DocumentA11yState::default(),
        actions: Vec::new(),
    }
}

/// A projection whose root holds `top`, over `nodes`.
fn projection(top: &[u64], nodes: Vec<DocumentA11yNode>) -> DocumentA11yProjection {
    let mut root = blank(1, DocumentA11yRole::Window);
    root.children = top.iter().copied().map(DocumentA11yNodeId::new).collect();
    let mut all = vec![root];
    all.extend(nodes);
    DocumentA11yProjection::new(
        0,
        DocumentA11ySupport::new(A11yCapability::Full, Vec::<String>::new()).unwrap(),
        DocumentA11yNodeId::new(1),
        all,
    )
}

#[test]
fn names_go_where_aria_reads_them() {
    let named = |id, role, name: &str| DocumentA11yNode {
        name: Some(name.into()),
        ..blank(id, role)
    };
    let mut field = named(4, DocumentA11yRole::TextField, "Title");
    field.value = Some("Draft".into());
    let nodes = vec![
        named(2, DocumentA11yRole::Button, "Save"),
        named(3, DocumentA11yRole::Group, "Tools"),
        field,
        named(5, DocumentA11yRole::Unknown, "Plain words"),
    ];
    let mirror = plan(&projection(&[2, 3, 4, 5], nodes), 1.0, |_, _| None);
    let [button, group, field, plain] = mirror.as_slice() else {
        panic!("{mirror:?}")
    };
    assert_eq!(
        (attr(button, "aria-label"), button.text.as_deref()),
        (None, Some("Save"))
    );
    assert_eq!(
        (attr(group, "aria-label"), group.text.as_deref()),
        (Some("Tools"), None)
    );
    assert_eq!(
        (attr(field, "aria-label"), field.text.as_deref()),
        (Some("Title"), Some("Draft"))
    );
    assert_eq!(
        (attr(plain, "role"), plain.text.as_deref()),
        (None, Some("Plain words"))
    );
}

#[test]
fn states_lower_to_their_aria_attributes() {
    let with = |id, role, state: DocumentA11yState| DocumentA11yNode {
        state,
        ..blank(id, role)
    };
    let mut slider = blank(8, DocumentA11yRole::Slider);
    (
        slider.numeric_value,
        slider.numeric_minimum,
        slider.numeric_maximum,
    ) = (Some(3.0), Some(0.0), Some(10.0));
    let nodes = vec![
        with(
            2,
            DocumentA11yRole::CheckBox,
            DocumentA11yState {
                checked: Some(true),
                ..Default::default()
            },
        ),
        with(
            3,
            DocumentA11yRole::Switch,
            DocumentA11yState {
                toggled: Some(DocumentA11yToggled::Mixed),
                ..Default::default()
            },
        ),
        with(
            4,
            DocumentA11yRole::Button,
            DocumentA11yState {
                toggled: Some(DocumentA11yToggled::On),
                disabled: true,
                ..Default::default()
            },
        ),
        with(
            5,
            DocumentA11yRole::Tab,
            DocumentA11yState {
                selected: Some(true),
                ..Default::default()
            },
        ),
        blank(6, DocumentA11yRole::Heading { level: 2 }),
        with(
            7,
            DocumentA11yRole::TextField,
            DocumentA11yState {
                multiline: true,
                read_only: true,
                required: true,
                ..Default::default()
            },
        ),
        slider,
    ];
    let mirror = plan(&projection(&[2, 3, 4, 5, 6, 7, 8], nodes), 1.0, |_, _| None);
    let states: Vec<Vec<(&str, &str)>> = mirror
        .iter()
        .map(|node| {
            node.attrs
                .iter()
                .filter(|(key, _)| *key != "role")
                .map(|(key, value)| (*key, value.as_str()))
                .collect()
        })
        .collect();
    assert_eq!(
        states,
        vec![
            vec![("aria-checked", "true")],
            vec![("aria-checked", "mixed")],
            vec![("aria-disabled", "true"), ("aria-pressed", "true")],
            vec![("aria-selected", "true")],
            vec![("aria-level", "2")],
            vec![
                ("aria-multiline", "true"),
                ("aria-readonly", "true"),
                ("aria-required", "true"),
            ],
            vec![
                ("aria-valuenow", "3"),
                ("aria-valuemin", "0"),
                ("aria-valuemax", "10"),
            ],
        ]
    );
}

#[test]
fn a_leaf_describes_itself_and_an_author_name_wins() {
    let mut labelled = blank(3, DocumentA11yRole::Unknown);
    labelled.name = Some("Reading map".into());
    let nodes = vec![blank(2, DocumentA11yRole::Unknown), labelled];
    let mut asked = Vec::new();
    let mirror = plan(&projection(&[2, 3], nodes), 1.0, |id, name| {
        asked.push((id, name.map(str::to_owned)));
        Some(LeafSemantics {
            role: Some("graphics-object"),
            name: Some("graph: 3 nodes, 2 links".into()),
            children: Vec::new(),
            names_itself: false,
            slot: None,
        })
    });
    assert_eq!(asked, [(2, None), (3, Some("Reading map".into()))]);
    let described: Vec<(Option<&str>, Option<&str>)> = mirror
        .iter()
        .map(|node| (attr(node, "role"), name(node)))
        .collect();
    assert_eq!(
        described,
        [
            (Some("graphics-object"), Some("graph: 3 nodes, 2 links")),
            (Some("graphics-object"), Some("Reading map")),
        ]
    );

    let mut node = accesskit::Node::new(accesskit::Role::GraphicsObject);
    node.set_label("graph: 3 nodes, 2 links");
    assert_eq!(
        leaf_semantics(&node),
        LeafSemantics {
            role: Some("graphics-object"),
            name: Some("graph: 3 nodes, 2 links".into()),
            children: Vec::new(),
            names_itself: false,
            slot: None,
        }
    );
}

#[test]
fn boxes_are_relative_to_their_parent_and_hidden_nodes_are_left_out() {
    let bounds = |x, y, width, height| {
        Some(DocumentA11yBounds {
            x,
            y,
            width,
            height,
        })
    };
    let mut list = blank(2, DocumentA11yRole::List);
    list.bounds = bounds(10.0, 10.0, 200.0, 100.0);
    list.children = vec![DocumentA11yNodeId::new(3), DocumentA11yNodeId::new(4)];
    let mut item = blank(3, DocumentA11yRole::ListItem);
    item.bounds = bounds(15.0, 20.0, 50.0, 10.0);
    let mut hidden = blank(4, DocumentA11yRole::ListItem);
    hidden.state.hidden = true;
    let mirror = plan(&projection(&[2], vec![list, item, hidden]), 2.0, |_, _| {
        None
    });
    assert_eq!(mirror[0].rect, Some([20.0, 20.0, 400.0, 200.0]));
    assert_eq!(mirror[0].children.len(), 1, "the hidden item is left out");
    assert_eq!(mirror[0].children[0].rect, Some([10.0, 20.0, 100.0, 20.0]));
    assert_eq!(ids(&mirror), [2, 3]);
}

#[test]
fn a_description_lowers_to_aria_description_and_an_empty_one_is_left_out() {
    let mut described = blank(2, DocumentA11yRole::Button);
    described.name = Some("Append a card".into());
    described.description = Some("Adds one card and advances the revision.".into());
    let mut empty = blank(3, DocumentA11yRole::Button);
    empty.name = Some("Forbidden action".into());
    empty.description = Some("  ".into());
    let mirror = plan(&projection(&[2, 3], vec![described, empty]), 1.0, |_, _| None);
    assert_eq!(
        attr(&mirror[0], "aria-description"),
        Some("Adds one card and advances the revision.")
    );
    assert_eq!(name(&mirror[0]), Some("Append a card"));
    assert_eq!(attr(&mirror[1], "aria-description"), None);
}

#[test]
fn what_a_leaf_draws_lowers_to_named_boxes_under_it() {
    let mut leaf = blank(2, DocumentA11yRole::Unknown);
    leaf.name = Some("Graph".into());
    leaf.bounds = Some(DocumentA11yBounds {
        x: 10.0,
        y: 100.0,
        width: 900.0,
        height: 600.0,
    });
    let mirror = plan(&projection(&[2], vec![leaf]), 2.0, |id, _| {
        (id == 2).then(|| {
            LeafSemantics::from_producer(1, cambium_rootstock::ProducerSemantics {
                role: Some(cambium_rootstock::ProducerRole::List),
                name: Some("Remote board".into()),
                children: vec![
                    cambium_rootstock::ProducerNode {
                        key: 10,
                        role: cambium_rootstock::ProducerRole::ListItem,
                        name: "Card 0".into(),
                        rect: [24.0, 24.0, 120.0, 80.0],
                        actions: Vec::new(),
                    },
                    cambium_rootstock::ProducerNode {
                        key: 11,
                        role: cambium_rootstock::ProducerRole::Image,
                        name: "Card 1".into(),
                        rect: [164.0, 24.0, 120.0, 80.0],
                        actions: Vec::new(),
                    },
                ],
            })
        })
    });
    let board = &mirror[0];
    assert_eq!(attr(board, "role"), Some("list"));
    assert_eq!(name(board), Some("Remote board"), "the producer names its slot");
    assert_eq!(board.children.len(), 2);
    let (first, second) = (&board.children[0], &board.children[1]);
    assert_eq!(attr(first, "role"), Some("listitem"));
    assert_eq!(name(first), Some("Card 0"));
    assert_eq!(
        first.rect,
        Some([48.0, 48.0, 240.0, 160.0]),
        "relative to the leaf, in CSS pixels"
    );
    assert_eq!(attr(second, "role"), Some("img"));
    assert_eq!(name(second), Some("Card 1"));
    assert_ne!(first.id, second.id);
    assert!(first.id >= 1 << 63, "outside the DOM's id range");

    // With nothing drawn, the producer still names its slot.
    let mut empty = blank(2, DocumentA11yRole::Unknown);
    empty.name = Some("Graph".into());
    let mirror = plan(&projection(&[2], vec![empty]), 1.0, |_, _| {
        Some(LeafSemantics::from_producer(1, cambium_rootstock::ProducerSemantics {
            role: Some(cambium_rootstock::ProducerRole::List),
            name: Some("Remote board · 0 cards".into()),
            children: Vec::new(),
        }))
    });
    assert_eq!(name(&mirror[0]), Some("Remote board · 0 cards"));
}

/// A drawn node with actions is a group named by the node, its actions
/// focusable buttons over its box, named by label, described by their
/// description, each naming the producer's slot, the node's key and the
/// action, which is what a reader's click on one routes as (dynamics
/// grammar plan, F65 and F66). Ids hold whatever order the nodes come in.
#[test]
fn a_drawn_nodes_actions_lower_to_buttons_that_name_their_action() {
    let mut leaf = blank(2, DocumentA11yRole::Unknown);
    leaf.bounds = Some(DocumentA11yBounds {
        x: 0.0,
        y: 0.0,
        width: 900.0,
        height: 600.0,
    });
    let item = |key: u64, name: &str| cambium_rootstock::ProducerNode {
        key,
        role: cambium_rootstock::ProducerRole::Group,
        name: name.into(),
        rect: [10.0, 20.0, 36.0, 36.0],
        actions: vec![
            cambium_rootstock::ProducerAction {
                id: "drag".into(),
                label: "Drag".into(),
                description: "Drag this item".into(),
            },
            cambium_rootstock::ProducerAction {
                id: "pin".into(),
                label: "Pin".into(),
                description: "Pin this item".into(),
            },
        ],
    };
    let lower = |children: Vec<cambium_rootstock::ProducerNode>| {
        plan(&projection(&[2], vec![leaf.clone()]), 1.0, move |_, _| {
            Some(LeafSemantics::from_producer(
                7,
                cambium_rootstock::ProducerSemantics {
                    role: Some(cambium_rootstock::ProducerRole::Group),
                    name: Some("2 of 2 shown".into()),
                    children: children.clone(),
                },
            ))
        })
    };
    let mirror = lower(vec![item(3, "Port notes"), item(4, "Relay")]);
    let canvas = &mirror[0];
    assert_eq!(attr(canvas, "role"), Some("group"));
    assert_eq!(name(canvas), Some("2 of 2 shown"));
    let notes = &canvas.children[0];
    assert_eq!(attr(notes, "role"), Some("group"));
    assert_eq!(attr(notes, "aria-label"), Some("Port notes"));
    assert!(!notes.focusable);
    assert_eq!(notes.children.len(), 2);
    let pin = &notes.children[1];
    assert_eq!(attr(pin, "role"), Some("button"));
    assert_eq!(pin.text.as_deref(), Some("Pin"));
    assert_eq!(attr(pin, "aria-description"), Some("Pin this item"));
    assert!(pin.focusable);
    assert_eq!(pin.rect, Some([0.0, 0.0, 36.0, 36.0]), "over the item's box");
    assert_eq!(
        pin.action,
        Some(cambium_rootstock::ProducedAction {
            slot: 7,
            key: 3,
            id: "pin".into()
        })
    );
    let produced = produced_actions(&mirror);
    assert_eq!(produced.len(), 4, "every button, and only buttons");
    assert!(produced.iter().any(|(id, action)| *id == pin.id && action.key == 3));

    // Reordered, the same item and button keep their ids.
    let reordered = lower(vec![item(4, "Relay"), item(3, "Port notes")]);
    let moved = &reordered[0].children[1];
    assert_eq!(moved.id, notes.id);
    assert_eq!(moved.children[1].id, pin.id);
}
