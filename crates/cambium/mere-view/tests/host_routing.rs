/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! The view through the host's own pointer and keyboard routing, without a
//! window: a press lands where layout painted a node, and the host hears it.

use cambium::GRAPH_CANVAS_SWATCH_CSS;
use cambium_genet_winit_host::Harness;
use layout_dom_api::{LayoutDom, LocalName, Namespace};
use mere_view::{
    GraphModel, MERE_VIEW_CSS, MereView, MereViewElement, MereViewModel, MereViewRequest,
    MereViewState, NodeEntry, NodeState, Provenance, RelationEntry,
};
use taproot::Selector;

struct Host {
    model: MereViewModel,
    view: MereViewState,
    requests: Vec<MereViewRequest>,
}

const SIZE: (u32, u32) = (900, 600);

fn root(host: &Host) -> MereViewElement<Host, ()> {
    mere_view::mere_view(
        MereView {
            model: &host.model,
            state: &host.view,
            leaf_key: 9,
            width: SIZE.0,
            height: SIZE.1,
        },
        |host: &mut Host, event| host.view.apply(event),
        |host: &mut Host, request| host.requests.push(request),
    )
}

fn host() -> Host {
    let node = |key: &str, label: &str| NodeEntry {
        key: key.into(),
        label: label.into(),
        state: NodeState::Available,
    };
    // A 2x2 grid with one diagonal relation. A rotated relation cell spans its
    // whole segment, so it reaches both endpoint nodes' centres.
    let model = MereViewModel {
        title: "notes".into(),
        graph: GraphModel {
            nodes: vec![
                node("a", "Alpha"),
                node("b", "Beta"),
                node("c", "Gamma"),
                node("d", "Delta"),
            ],
            relations: vec![RelationEntry {
                key: "ad".into(),
                from: "a".into(),
                to: "d".into(),
                provenance: Provenance::Extracted,
            }],
        },
        layout: "grid.default".into(),
        ..MereViewModel::default()
    };
    let mut view = MereViewState::default();
    view.lay_out(&model, SIZE.0, SIZE.1);
    Host {
        model,
        view,
        requests: Vec::new(),
    }
}

fn describe(
    h: &Harness<Host, fn(&Host) -> MereViewElement<Host, ()>, MereViewElement<Host, ()>>,
) -> String {
    let Some(node) = h.hit() else {
        return "nothing".into();
    };
    h.with_dom(|dom| {
        let name = dom
            .element_name(node)
            .map(|n| n.local.to_string())
            .unwrap_or_default();
        let class = dom
            .attribute(node, &Namespace::from(""), &LocalName::from("class"))
            .unwrap_or_default()
            .to_string();
        format!("<{name} class={class:?}>")
    })
}

#[test]
fn a_press_on_a_node_reaches_the_host_even_where_a_relation_ends() {
    let sheet = format!("{GRAPH_CANVAS_SWATCH_CSS}\n{MERE_VIEW_CSS}");
    let mut h = Harness::new(
        sheet,
        host(),
        root as fn(&Host) -> MereViewElement<Host, ()>,
    );
    h.layout_at(SIZE.0 as f32, SIZE.1 as f32);
    for key in ["a", "b", "d"] {
        let target = Selector::class("graph-canvas-swatch-node").with_attr("data-key", key);
        let (x, y) = h.resolve(&target).expect("the node's target paints");
        h.move_to(x, y);
        let hit = describe(&h);
        assert!(h.click_on(&target), "the selector resolves");
        assert_eq!(
            h.state().requests.last(),
            Some(&MereViewRequest::Activate(key.into())),
            "a press on {key} at ({x}, {y}) hit {hit}"
        );
    }
}
