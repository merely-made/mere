/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use super::*;

/// A node's visible label can say less than its accessible name, and a
/// host can put a label on the side it needs.
#[test]
fn a_visible_label_can_be_shorter_and_placed_by_the_host() {
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let runner =
        GenetAppRunner::<_, _, _, ()>::new(dom.clone(), short_labelled_view, State::default());
    let root = runner.root();
    let dom = dom.borrow();
    let second = find_attr(&dom, root, "aria-label", "Second node")
        .expect("the accessible name stays whole");
    assert_eq!(attr(&dom, second, "aria-label"), Some("Second node"));
    let left = find_attr(&dom, root, "data-label-placement", "left")
        .expect("node 1's label goes left as asked");
    assert_eq!(
        dom.text(dom.dom_children(left).next().unwrap()),
        Some("First node")
    );
    let mut labels = Vec::new();
    classes_of(&dom, root, "graph-canvas-swatch-label", &mut labels);
    assert_eq!(labels.len(), 2);
    let texts: Vec<String> = {
        fn walk(dom: &ScriptedDom, node: NodeId, out: &mut Vec<String>) {
            if attr(dom, node, "class").is_some_and(|c| c.starts_with("graph-canvas-swatch-label "))
                || attr(dom, node, "class") == Some("graph-canvas-swatch-label")
            {
                let text = dom
                    .dom_children(node)
                    .filter_map(|child| dom.text(child).map(str::to_string))
                    .collect::<String>();
                out.push(text);
            }
            for child in dom.dom_children(node).collect::<Vec<_>>() {
                walk(dom, child, out);
            }
        }
        let mut out = Vec::new();
        walk(&dom, root, &mut out);
        out
    };
    assert!(texts.contains(&"Second".to_string()), "{texts:?}");
    assert!(!texts.contains(&"Second node".to_string()), "{texts:?}");
}

/// A long label is cut in its middle, keeping its start and the end that
/// tells it apart, at word boundaries; a short one is left whole.
#[test]
fn a_long_label_is_cut_in_the_middle() {
    let fork = "Session (fork of Session (from blake3:f612a))";
    let cut = middle_cut(fork, GRAPH_LABEL_CHARS);
    assert!(cut.chars().count() <= GRAPH_LABEL_CHARS, "{cut}");
    assert!(cut.starts_with("Session"), "{cut}");
    assert_eq!(cut, "Session (fork of … blake3:f612a))");
    let picked = middle_cut(
        "Session (picked from Evening reading (from blake3:f612a))",
        GRAPH_LABEL_CHARS,
    );
    assert!(picked.starts_with("Session (picked"), "{picked}");
    assert!(picked.ends_with("blake3:f612a))"), "{picked}");
    assert!(picked.chars().count() <= GRAPH_LABEL_CHARS, "{picked}");
    assert!(cut.contains(" … "), "{cut}");
    let other = middle_cut(
        "Session (fork of Session (from blake3:0c0de))",
        GRAPH_LABEL_CHARS,
    );
    assert_ne!(cut, other, "names that differ at the end stay apart");
    assert_eq!(
        middle_cut("Evening reading", GRAPH_LABEL_CHARS),
        "Evening reading"
    );
    assert_eq!(
        middle_cut("abcdefghijklmnopqrstuvwxyz0123456789", 10)
            .chars()
            .count(),
        10,
        "a name without spaces is cut at the budget"
    );
    // The canvas shows the cut and keeps the whole accessible name.
    let swatch = model(None, None)
        .with_node_labels(true)
        .with_visible_label(2, fork);
    assert_eq!(swatch.label_text(1), cut);
    assert_eq!(swatch.graph.nodes[1].label, "Second node");
}

/// A node an overlay occupies shows the overlay, not its label too, with
/// and without culling.
#[test]
fn an_occupied_node_draws_no_label() {
    for cull in [false, true] {
        let swatch = model(None, None)
            .with_node_labels(true)
            .with_label_culling(cull)
            .with_node_footprint(1, 120.0, 60.0);
        assert_eq!(swatch.shown_labels(), [false, true], "cull {cull}");
    }
}

/// Visible labels carry the node's state, not just its text.
///
/// Without this a consumer that needs to emphasize one label (which node is
/// "here") has to hand-roll the entire label layer to get it, which defeats
/// the point of the component rendering labels at all.
#[test]
fn visible_labels_carry_node_state() {
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let state = State {
        hovered: Some(2),
        ..State::default()
    };
    let runner = GenetAppRunner::<_, _, _, ()>::new(dom.clone(), labelled_view, state);
    let root = runner.root();
    let mut found = Vec::new();
    classes_of(&dom.borrow(), root, "graph-canvas-swatch-label", &mut found);

    assert_eq!(found.len(), 2, "one label per node: {found:?}");
    // `model` selects node 1; the test state hovers node 2.
    assert!(
        found
            .iter()
            .any(|c| c == "graph-canvas-swatch-label selected"),
        "the selected node's label must say so: {found:?}"
    );
    assert!(
        found
            .iter()
            .any(|c| c == "graph-canvas-swatch-label hovered"),
        "the hovered node's label must say so: {found:?}"
    );

    let swatch = model(None, None).with_node_labels(true);
    assert_eq!(swatch.label_placement(0), GraphLabelPlacement::Above);
    assert_eq!(swatch.label_placement(1), GraphLabelPlacement::Above);

    // On a narrow canvas a node two thirds across has no room for a whole
    // label on its right, and more on its left.
    let lone = |x: f32| {
        GraphCanvasSwatch::new(
            7,
            GraphCanvasSubgraph {
                nodes: vec![GraphCanvasNode {
                    id: 1,
                    kind: "document",
                    position: (x, 0.5),
                    label: "Archive 2025, unavailable".into(),
                    key: None,
                }],
                edges: Vec::new(),
            },
        )
        .with_size(280, 400)
    };
    assert_eq!(lone(0.68).label_placement(0), GraphLabelPlacement::Left);
    assert_eq!(lone(0.30).label_placement(0), GraphLabelPlacement::Right);
    // A left label is held by its right edge beside the node, so it hugs
    // the node whatever width its text takes.
    let size = Size {
        width: 280.0,
        height: 400.0,
    };
    let bounds = GraphLabelPlacement::Left.bounds(190.0, 200.0, 12.0, size, 100.0);
    assert_eq!(bounds.left, 78.0);
    assert_eq!(bounds.width, 100.0);

    // Three nodes stacked closer than a label's height: culling keeps the
    // first, and a focused node keeps its label whatever it covers.
    let crowded = GraphCanvasSwatch::new(
        8,
        GraphCanvasSubgraph {
            nodes: (1..=3)
                .map(|id| GraphCanvasNode {
                    id,
                    kind: "document",
                    position: (0.3, 0.5 + id as f32 * 0.01),
                    label: format!("A long label for node {id}"),
                    key: None,
                })
                .collect(),
            edges: Vec::new(),
        },
    )
    .with_size(280, 400)
    .with_node_labels(true)
    .with_label_culling(false);
    assert_eq!(crowded.shown_labels(), [true, true, true], "culling is off");
    let culled = crowded.with_label_culling(true);
    assert!(culled.shown_labels()[0]);
    assert!(culled.shown_labels().iter().any(|shown| !shown));
    let mut focused = culled;
    focused.focus = Some(3);
    assert!(focused.shown_labels()[2]);
    assert!(
        find_attr(&dom.borrow(), root, "data-label-placement", "above").is_some(),
        "a horizontal graph publishes its label-clear placement"
    );
}

#[test]
fn radial_labels_share_bounded_collision_free_geometry() {
    let names = [
        "Minor Pentatonic",
        "Minor",
        "Minor 7",
        "Quartal Tetrad",
        "Quartal Triad",
        "Suspended 4th",
        "Minor",
        "Minor 7",
        "Quartal Tetrad",
        "Quartal Triad",
        "Suspended 4th",
        "Major Pentatonic",
    ];
    let nodes = names
        .iter()
        .enumerate()
        .map(|(id, name)| {
            let angle = (id.saturating_sub(1)) as f32 / 11.0 * std::f32::consts::TAU
                - std::f32::consts::FRAC_PI_2;
            GraphCanvasNode {
                id,
                kind: "material",
                label: (*name).into(),
                key: None,
                position: if id == 0 {
                    (0.5, 0.5)
                } else {
                    (0.5 + angle.cos() * 0.42, 0.5 + angle.sin() * 0.42)
                },
            }
        })
        .collect();
    let mut swatch = GraphCanvasSwatch::new(
        9,
        GraphCanvasSubgraph {
            nodes,
            edges: Vec::<GraphCanvasEdge<usize>>::new(),
        },
    )
    .with_size(300, 210)
    .with_node_labels(true);
    swatch.selected = Some(0);
    for zoom in [1.0, 1.10, 1.25, 0.85] {
        swatch.viewport.zoom = zoom;
        let labels = swatch.resolved_labels();
        assert!(labels[0].is_some(), "selected label at {zoom}");
        let boxes: Vec<_> = labels.iter().flatten().map(|label| label.bounds).collect();
        assert!(
            boxes.len() >= 8,
            "readable radial disclosure at {zoom}: {}",
            boxes.len()
        );
        for (i, a) in boxes.iter().enumerate() {
            assert!(
                a.left >= 0.0
                    && a.top >= 0.0
                    && a.left + a.width <= 300.01
                    && a.top + a.height <= 210.01
            );
            for b in &boxes[i + 1..] {
                assert!(!a.overlaps(*b), "{zoom}: {a:?} {b:?}");
            }
        }
        assert_eq!(
            format!("{:?}", labels),
            format!("{:?}", swatch.resolved_labels())
        );
    }
}

#[test]
fn label_priority_does_not_allow_mutual_emphasis_collisions() {
    let mut swatch = model(Some(1), Some(2)).with_node_labels(true);
    swatch.graph.nodes[1].position = swatch.graph.nodes[0].position;
    swatch.selected = Some(1);
    swatch.focus = Some(2);
    swatch.hovered = Some(2);
    let labels = swatch.resolved_labels();
    assert!(labels[0].is_some());
    if let (Some(a), Some(b)) = (labels[0], labels[1]) {
        assert!(!a.bounds.overlaps(b.bounds));
    }
    swatch.viewport.pan = (10.0, 10.0);
    assert!(swatch.resolved_labels().iter().all(Option::is_none));
}
