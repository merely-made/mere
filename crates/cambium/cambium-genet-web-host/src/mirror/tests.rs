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
use layout_dom_api::{LayoutDom, LocalName, Namespace};

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
