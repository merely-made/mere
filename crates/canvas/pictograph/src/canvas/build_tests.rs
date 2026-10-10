// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Tests for `build.rs` (sample graph, simulation construction, relation-edge
//! builders). Split into a sibling file per the 600-LOC ceiling.

use kernel::graph::fixtures::GraphFixtures;
use std::collections::{HashMap, HashSet};

use euclid::default::Point2D;
use kernel::graph::{
    ContainmentSubKind, EdgeAssertion, Graph, NodeKey, RelationSelector, SemanticSubKind,
};
use layout_dom_api::{LayoutDom, LayoutDomMut};

use crate::canvas::build::*;
use crate::canvas::palette;
use crate::canvas::seiche_bridge::{build_simulation, visible_relation_edges};

fn computed_value(
    dom: &genet_scripted_dom::ScriptedDom,
    node: genet_scripted_dom::NodeId,
    property: &str,
) -> Option<String> {
    let styles = genet_livery::resolve_styles(
        dom,
        &genet_livery::StyleSet::cambium(&NODE_SHEET),
        &genet_livery::Device::screen(800.0, 600.0),
        &genet_livery::InteractionStates::default(),
    );
    styles.computed_style(node, property)
}

#[test]
fn sample_graph_has_nodes_and_edges() {
    let g = sample_graph();
    assert_eq!(g.nodes().count(), 12, "the ring has twelve nodes");
    assert_eq!(
        g.relations().count(),
        0,
        "content links live in the Resource graph"
    );
    assert_eq!(
        g.resource_relations().count(),
        16,
        "twelve ring links and four spokes"
    );
    assert_eq!(
        g.projected_relations().count(),
        16,
        "each Resource link lifts once to this ring"
    );
    for i in 0..12 {
        let from = g.get_node_by_url(&format!("mere://node/{i}")).unwrap().0;
        let to = g
            .get_node_by_url(&format!("mere://node/{}", (i + 1) % 12))
            .unwrap()
            .0;
        let from_resource = g.shown_resource_id(from).unwrap();
        let to_resource = g.shown_resource_id(to).unwrap();
        assert_ne!(from_resource, to_resource);
        let (handle, payload) = g.projected_relations_between(from, to).next().unwrap();
        assert!(matches!(handle, kernel::graph::RelationKey::Resource(_)));
        assert!(payload.has_relation(RelationSelector::Semantic(SemanticSubKind::Hyperlink)));
        assert!(
            g.find_resource_edge_key(from_resource, to_resource)
                .is_some()
        );
    }
}

#[test]
fn pool_has_a_gnode_per_node() {
    let g = sample_graph();
    let (_dom, gnode_of, _stage) = build_pool_dom(&g);
    assert_eq!(
        gnode_of.len(),
        g.nodes().count(),
        "the pre-materialized pool has one gnode per graph node",
    );
}

#[test]
fn simulation_has_a_body_per_node_and_the_edge_topology() {
    let g = sample_graph();
    let sim = build_simulation(&g);
    assert_eq!(sim.body_count(), 12, "one physics body per node");
    assert!(
        sim.edge_count() >= 12,
        "the spring topology carries the edges"
    );
}

#[test]
fn visible_relation_edges_keeps_one_tuple_per_cell_and_drops_hidden_ones() {
    let mut graph = Graph::new();
    let a = graph.add_node("https://a".to_string(), Point2D::new(0.0, 0.0));
    let b = graph.add_node("https://b".to_string(), Point2D::new(1.0, 0.0));
    graph.assert_relation(
        a,
        b,
        EdgeAssertion::Semantic {
            sub_kind: SemanticSubKind::Hyperlink,
            label: None,
            decay_progress: None,
        },
    );
    graph.assert_relation(
        a,
        b,
        EdgeAssertion::Containment {
            sub_kind: ContainmentSubKind::Domain,
        },
    );

    let edges = visible_relation_edges(&graph, &HashSet::new());
    assert_eq!(
        edges.len(),
        2,
        "two live relation cells between the same pair pull as two springs, not one"
    );

    let mut hidden = HashSet::new();
    hidden.insert(crate::canvas::EdgeCell {
        from: a,
        to: b,
        selector: RelationSelector::Containment(ContainmentSubKind::Domain),
    });
    let edges = visible_relation_edges(&graph, &hidden);
    assert_eq!(
        edges,
        vec![(a, b)],
        "hiding one cell drops exactly its own spring, leaving the other live"
    );
}

/// The load-bearing check for the palette-as-custom-properties move: the sheet
/// names no color, so if genet's cascade ever stopped substituting `var()` the
/// gnodes would silently lose their fill. Assert the *resolved* computed color,
/// not the sheet text.
#[test]
fn gnode_fill_resolves_from_the_palette_custom_properties() {
    let g = sample_graph();
    let (dom, gnode_of, _stage) = build_pool_dom(&g);
    let gnode = *gnode_of.values().next().expect("the pool has gnodes");

    let idle_bg = palette::rgb(palette::IDLE.bg);
    assert_eq!(
        computed_value(&dom, gnode, "background-color").as_deref(),
        Some(idle_bg.as_str()),
        "the default .gnode must resolve --node-idle-bg through the cascade",
    );
    let idle_fg = palette::rgb(palette::IDLE.fg);
    assert_eq!(
        computed_value(&dom, gnode, "color").as_deref(),
        Some(idle_fg.as_str()),
        "the default .gnode must resolve --node-idle-fg through the cascade",
    );
}

/// Each activation class pulls its own `--node-*` pair, and selection wins. This
/// pins the class-to-var wiring the host relies on when it pushes node state.
#[test]
fn each_gnode_state_class_resolves_its_own_palette_entry() {
    let g = sample_graph();
    let (mut dom, gnode_of, _stage) = build_pool_dom(&g);
    let gnode = *gnode_of.values().next().expect("the pool has gnodes");

    for (class, expected) in [
        ("gnode gnode-open", palette::OPEN),
        ("gnode gnode-closed", palette::CLOSED),
        ("gnode gnode-idle", palette::IDLE),
        ("gnode gnode-selected", palette::SELECTED),
    ] {
        dom.set_attribute(gnode, qual("class"), class);
        let want_bg = palette::rgb(expected.bg);
        let want_fg = palette::rgb(expected.fg);
        assert_eq!(
            computed_value(&dom, gnode, "background-color").as_deref(),
            Some(want_bg.as_str()),
            "`{class}` must resolve its own --node-*-bg",
        );
        assert_eq!(
            computed_value(&dom, gnode, "color").as_deref(),
            Some(want_fg.as_str()),
            "`{class}` must resolve its own --node-*-fg",
        );
    }
}

/// The representation ladder must reach properties that alter the rendered
/// face, while leaving the gnode's measured box to the common geometry rule.
#[test]
fn representation_classes_change_caption_density_and_live_emphasis() {
    let g = sample_graph();
    let (mut dom, gnode_of, _stage) = build_pool_dom(&g);
    let gnode = *gnode_of.values().next().expect("the pool has gnodes");
    let caption = dom
        .dom_children(gnode)
        .next()
        .expect("the gnode has a caption");

    dom.set_attribute(
        gnode,
        qual("class"),
        "gnode-idle gnode-representation-glyph",
    );
    assert_eq!(
        computed_value(&dom, caption, "display").as_deref(),
        Some("none"),
        "the glyph rung suppresses the card caption",
    );
    assert_eq!(
        computed_value(&dom, gnode, "width").as_deref(),
        Some("36px"),
        "representation styling does not invent a second footprint",
    );

    dom.set_attribute(gnode, qual("class"), "gnode-idle gnode-representation-card");
    assert_eq!(
        computed_value(&dom, caption, "display").as_deref(),
        Some("block"),
        "the card rung retains the ordinary labelled face",
    );

    dom.set_attribute(
        gnode,
        qual("class"),
        "gnode-idle gnode-representation-live-pane",
    );
    assert_eq!(
        computed_value(&dom, caption, "font-weight").as_deref(),
        Some("700"),
        "the live-capable rung visibly emphasizes its caption",
    );
}

#[test]
fn ticking_moves_nodes_from_the_seed() {
    let g = sample_graph();
    let mut sim = build_simulation(&g);
    let before: Vec<(NodeKey, Point2D<f32>)> = sim.positions().collect();
    for _ in 0..60 {
        sim.tick(crate::canvas::TICK_DT);
    }
    let after: HashMap<NodeKey, Point2D<f32>> = sim.positions().collect();
    let moved = before.iter().any(|(k, p0)| {
        after
            .get(k)
            .is_some_and(|p1| (p1.x - p0.x).hypot(p1.y - p0.y) > 1.0)
    });
    assert!(moved, "the force-directed settle moves nodes off the seed");
}

#[test]
fn independent_assertions_group_into_one_drawn_link() {
    use kernel::graph::SemanticStatementSpec;
    let mut graph = Graph::new();
    let a = graph.add_node("https://a.test/".into(), Default::default());
    let b = graph.add_node("https://b.test/".into(), Default::default());
    let assertion = |asserter: &str| SemanticStatementSpec {
        predicate: kernel::graph::predicate_iri(SemanticSubKind::Cites).into(),
        recognized_sub_kind: Some(SemanticSubKind::Cites),
        provenance_iri: Some(asserter.into()),
        ..Default::default()
    };
    let (key, alice) = graph
        .assert_semantic_statement(a, b, assertion("https://alice.test/"))
        .unwrap();
    let (_, bob) = graph
        .assert_semantic_statement(a, b, assertion("https://bob.test/"))
        .unwrap();
    assert_ne!(alice.statement_id, bob.statement_id);
    assert_eq!(
        graph.get_relation(key).unwrap().semantic_statements().len(),
        2
    );
    assert_eq!(dedup_edges(&graph), vec![(a, b)]);
    assert!(graph.retract_semantic_statement(a, b, &alice.statement_id));
    assert_eq!(
        dedup_edges(&graph),
        vec![(a, b)],
        "Bob still draws the link"
    );
    assert!(graph.retract_semantic_statement(a, b, &bob.statement_id));
    assert!(dedup_edges(&graph).is_empty());
}

pub(super) fn shown_content_graph() -> (Graph, [NodeKey; 5]) {
    use kernel::graph::ResourceNode;
    use kernel::persistence::{
        PersistedEdge, PersistedEdgeFamily, PersistedResourceRecord, PersistedSemanticEdgeData,
        PersistedSemanticStatement, PersistedSemanticSubKind, PersistedShownResource,
    };
    let mut graph = Graph::new();
    let keys = ["a", "alias-a", "b", "alias-b", "unbound"].map(|suffix| {
        graph.add_node(
            format!("https://{suffix}.surface.test/"),
            Default::default(),
        )
    });
    let [a, _, b, _, _] = keys;
    graph.assert_relation(
        a,
        b,
        EdgeAssertion::Semantic {
            sub_kind: SemanticSubKind::UserGrouped,
            label: None,
            decay_progress: None,
        },
    );
    let ids = keys.map(|key| graph.get_node(key).unwrap().id);
    let mut snapshot = graph.to_snapshot();
    let resources = [
        "https://content.test/a",
        "https://content.test/b",
        "https://content.test/unshown",
    ];
    let [ra, rb, unshown] = resources.map(|iri| ResourceNode::for_term(iri).id());
    snapshot.resources = resources
        .map(|iri| PersistedResourceRecord {
            canonical_iri: iri.into(),
            facets: vec![],
        })
        .into_iter()
        .collect();
    let claim = |id: &str, kind| PersistedSemanticStatement {
        statement_id: id.into(),
        predicate: kernel::graph::predicate_iri(kind).into(),
        recognized_sub_kind: Some(match kind {
            SemanticSubKind::Cites => PersistedSemanticSubKind::Cites,
            SemanticSubKind::Quotes => PersistedSemanticSubKind::Quotes,
            _ => unreachable!(),
        }),
        label: Some(id.into()),
        graph_scope: Default::default(),
        provenance_iri: Some(format!("https://author.test/{id}")),
        asserted_at_ms: Some(17),
    };
    let pair = |from: uuid::Uuid, to: uuid::Uuid, statements| PersistedEdge {
        from_node_id: from.to_string(),
        to_node_id: to.to_string(),
        families: vec![PersistedEdgeFamily::Semantic],
        semantic: Some(PersistedSemanticEdgeData {
            statements,
            ..Default::default()
        }),
        traversal: None,
        containment: None,
        arrangement: None,
        imported: None,
        provenance: None,
    };
    snapshot.resource_edges = vec![
        pair(
            ra,
            rb,
            vec![
                claim("alice", SemanticSubKind::Cites),
                claim("bob", SemanticSubKind::Cites),
                claim("quote", SemanticSubKind::Quotes),
            ],
        ),
        pair(
            ra,
            unshown,
            vec![claim("not-shown", SemanticSubKind::Cites)],
        ),
    ];
    snapshot.shown_resources = [(ids[0], ra), (ids[1], ra), (ids[2], rb), (ids[3], rb)]
        .map(|(surface, resource)| PersistedShownResource {
            surface_id: surface.to_string(),
            resource_id: resource.to_string(),
        })
        .into_iter()
        .collect();
    let graph = Graph::try_from_snapshot(&snapshot).unwrap();
    let keys = ids.map(|id| graph.get_node_key_by_id(id).unwrap());
    (graph, keys)
}

#[test]
fn content_projection_drives_all_shown_pairs_through_topology_draw_and_hit() {
    use crate::canvas::edge_cells::{edge_cell_hit_test, visible_edge_cell_segments};
    use kernel::graph::{RelationKey, RelationKind};
    let (graph, [a, alias_a, b, alias_b, unbound]) = shown_content_graph();
    assert_eq!(
        graph.relations().count(),
        1,
        "surface assertion remains in its own store"
    );
    assert_eq!(
        graph.resource_relations().count(),
        2,
        "unshown truth remains stored"
    );
    let rows: Vec<_> = graph.projected_relations().collect();
    assert_eq!(rows.len(), 13); // 3 claims * 4 shown pairs + one surface claim
    assert_eq!(
        rows.iter()
            .filter(|(key, _)| matches!(key, RelationKey::Resource(_)))
            .count(),
        12
    );
    assert!(
        rows.iter()
            .all(|(key, _)| graph.get_relation(*key).is_some())
    );
    assert!(
        rows.iter()
            .all(|(_, row)| row.from != unbound && row.to != unbound)
    );
    let pairs = dedup_edges(&graph);
    assert_eq!(pairs.len(), 4);
    for source in [a, alias_a] {
        for target in [b, alias_b] {
            assert!(pairs.contains(&(source, target)));
        }
    }
    assert_eq!(visible_relation_edges(&graph, &HashSet::new()).len(), 13);
    let weighted = dedup_edges_weighted(&graph);
    assert_eq!(weighted.len(), 4);
    assert_eq!(
        weighted.iter().map(|(_, _, weight)| weight).sum::<u32>(),
        13
    );
    let projection = crate::canvas::underlay::projection_from_graph(&graph);
    assert_eq!(projection.edges.len(), 4);
    assert_eq!(
        projection.edges.iter().map(|edge| edge.weight).sum::<f32>(),
        13.0
    );
    assert!(
        projection.edges.iter().all(|edge| edge.edge.is_none()),
        "mixed rows never become surface handles"
    );
    let view = seiche::LayoutView::from_parts(
        [
            (a, Point2D::new(0.0, 0.0)),
            (alias_a, Point2D::new(0.0, 300.0)),
            (b, Point2D::new(1000.0, 0.0)),
            (alias_b, Point2D::new(1000.0, 300.0)),
            (unbound, Point2D::new(500.0, 900.0)),
        ],
        pairs,
        1.0,
    );
    let segments = visible_edge_cell_segments(&graph, &view, &HashSet::new());
    assert_eq!(
        segments.len(),
        13,
        "every assertion contributes its fanned lane"
    );
    for segment in segments
        .iter()
        .filter(|segment| segment.kind == RelationKind::Semantic(SemanticSubKind::Quotes))
    {
        let point = Point2D::new(
            segment.from.x * 0.75 + segment.to.x * 0.25,
            segment.from.y * 0.75 + segment.to.y * 0.25,
        );
        assert_eq!(
            edge_cell_hit_test(&graph, &view, &HashSet::new(), point, 1.0),
            Some(segment.cell)
        );
    }
    let hidden = HashSet::from([crate::canvas::EdgeCell {
        from: a,
        to: b,
        selector: RelationSelector::Semantic(SemanticSubKind::Cites),
    }]);
    assert_eq!(visible_relation_edges(&graph, &hidden).len(), 11);
    let visible = visible_edge_cell_segments(&graph, &view, &hidden);
    assert_eq!(visible.len(), 11);
    assert!(
        visible
            .iter()
            .all(|segment| !hidden.contains(&segment.cell))
    );
    assert!(
        visible.iter().any(|segment| segment.cell.from == a
            && segment.cell.to == b
            && segment.kind == RelationKind::Semantic(SemanticSubKind::Quotes)),
        "same-pair quote is the hiding control"
    );
    assert_eq!(graph.resource_relations().count(), 2);
}

#[test]
fn content_projection_reaches_score_edges_and_fold_boundary_accounting() {
    let (graph, [a, alias_a, b, alias_b, unbound]) = shown_content_graph();
    let signals = cartography::IntelligenceSignals::default();
    let request = cartography::ProjectionRequest {
        graph: &graph,
        signals: &signals,
        intent: Default::default(),
    };
    let edges = cartography::adapters::score::build_positioned_edges(&request, &HashMap::new());
    assert_eq!(edges.len(), 13);
    assert!(
        edges
            .iter()
            .all(|edge| edge.from != unbound && edge.to != unbound && edge.edge.is_none())
    );
    let mut registry = crate::signals::ChannelRegistry::new();
    let channels = registry.disclose(
        &graph,
        &[cartography::ORDER_TIMELINE, cartography::WEIGHT_RECENCY],
        None,
    );
    let spiral = cartography::project_spiral_score(
        &graph,
        &channels,
        cartography::ORDER_TIMELINE,
        None,
        None,
    );
    assert_eq!(spiral.projection.edges.len(), 13);
    assert_eq!(
        spiral.projection.nodes.len(),
        5,
        "unbound surface still has a body"
    );
    let fold = forme::FoldRecord::from_selection(
        "content",
        [
            graph.get_node(a).unwrap().id,
            graph.get_node(alias_a).unwrap().id,
        ],
    )
    .unwrap();
    let projection = crate::canvas::fold_projection::project_fold(&graph, &fold).unwrap();
    assert_eq!(projection.internal_relation_count, 0);
    assert_eq!(projection.boundary_bundles.len(), 2);
    for (target, count) in [(b, 7), (alias_b, 6)] {
        let bundle = projection
            .boundary_bundles
            .iter()
            .find(|bundle| bundle.outside == target)
            .unwrap();
        assert_eq!(bundle.count, count);
        assert_eq!(
            bundle.direction,
            crate::canvas::fold_projection::FoldBoundaryDirection::Outgoing
        );
    }
    assert!(
        !projection
            .boundary_bundles
            .iter()
            .any(|bundle| bundle.outside == unbound)
    );
    assert_eq!(
        graph.resource_relations().count(),
        2,
        "layout and folding preserve stored truth"
    );
}
