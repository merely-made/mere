// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

#[test]
fn closed_link_hides_preserve_controller_rows_in_the_other_stratum() {
    use crate::canvas::seiche_bridge::{
        visible_relation_edges_with_link_hides, visible_relation_topology_with_link_hides,
    };
    use scenograph::link::{LinkKey, LinkStratum};
    let (graph, [a, alias_a, b, alias_b, _]) = shown_content_graph();
    let surface_key = LinkKey::new(
        LinkStratum::Surface,
        graph.get_node(a).unwrap().id.to_string(),
        graph.get_node(b).unwrap().id.to_string(),
    );
    let all = visible_relation_edges_with_link_hides(&graph, &HashSet::new(), &HashSet::new());
    assert_eq!(all.len(), 13);
    let topology =
        visible_relation_topology_with_link_hides(&graph, &HashSet::new(), &HashSet::new());
    assert_eq!(topology.len(), 13);
    assert_eq!(
        topology.edge_topology().iter().copied().collect::<Vec<_>>(),
        topology
            .rows
            .iter()
            .map(|row| (row.displayed_from, row.displayed_to))
            .collect::<Vec<_>>(),
        "counted controller input expands in the source reader's exact row order"
    );
    let surface_rows: Vec<_> = topology
        .rows
        .iter()
        .filter(|row| row.key == surface_key)
        .collect();
    assert_eq!(surface_rows.len(), 1);
    assert_eq!(
        (surface_rows[0].displayed_from, surface_rows[0].displayed_to),
        (a, b),
        "source identity and displayed alias endpoints remain distinct"
    );

    let hidden_surface = HashSet::from([surface_key.clone()]);
    assert_eq!(
        visible_relation_edges_with_link_hides(&graph, &HashSet::new(), &hidden_surface).len(),
        12,
        "hiding the Surface link removes its single surface source row"
    );

    let resource_a = graph.resource(graph.shown_resource_id(a).unwrap()).unwrap();
    let resource_b = graph.resource(graph.shown_resource_id(b).unwrap()).unwrap();
    let resource_key = LinkKey::new(
        LinkStratum::Resource,
        resource_a.canonical_iri(),
        resource_b.canonical_iri(),
    );
    let resource_rows: Vec<_> = topology
        .rows
        .iter()
        .filter(|row| row.key == resource_key)
        .collect();
    assert_eq!(resource_rows.len(), 12);
    assert!(
        resource_rows
            .iter()
            .any(|row| { row.displayed_from == alias_a || row.displayed_to == alias_a })
    );
    assert!(
        resource_rows
            .iter()
            .any(|row| { row.displayed_from == alias_b || row.displayed_to == alias_b })
    );
    assert!(
        resource_rows
            .iter()
            .all(|row| row.key.stratum == LinkStratum::Resource)
    );
    let hidden_resource = HashSet::from([resource_key]);
    assert_eq!(
        visible_relation_edges_with_link_hides(&graph, &HashSet::new(), &hidden_resource).len(),
        1,
        "hiding the Resource link removes all aliases' resource rows but keeps Surface truth"
    );
    assert_ne!(a, alias_a);
    assert_ne!(b, alias_b);
}

#[test]
fn link_appearances_keep_resource_identity_across_aliases_and_separate_two_strata() {
    use crate::canvas::edge_cells::visible_link_appearances;
    use scenograph::link::{LinkKey, LinkStratum};
    let (graph, [a, alias_a, b, alias_b, unbound]) = shown_content_graph();
    let view = seiche::LayoutView::from_parts(
        [
            (a, Point2D::new(0.0, 0.0)),
            (alias_a, Point2D::new(0.0, 300.0)),
            (b, Point2D::new(1000.0, 0.0)),
            (alias_b, Point2D::new(1000.0, 300.0)),
            (unbound, Point2D::new(500.0, 900.0)),
        ],
        dedup_edges(&graph),
        1.0,
    );
    let appearances =
        visible_link_appearances(&graph, &view, &HashSet::new(), &HashSet::new(), |_| 36.0);
    assert_eq!(
        appearances.len(),
        5,
        "one Surface plus four Resource appearances"
    );
    let resource_a = graph.resource(graph.shown_resource_id(a).unwrap()).unwrap();
    let resource_b = graph.resource(graph.shown_resource_id(b).unwrap()).unwrap();
    let resource_key = LinkKey::new(
        LinkStratum::Resource,
        resource_a.canonical_iri(),
        resource_b.canonical_iri(),
    );
    assert_eq!(
        appearances
            .iter()
            .filter(|segment| segment.appearance.key == resource_key)
            .count(),
        4,
        "aliases are appearances of one canonical Resource link"
    );
    let mut selected = Canvas::with_graph(graph.clone());
    let chosen = if alias_a <= alias_b {
        (alias_a, alias_b)
    } else {
        (alias_b, alias_a)
    };
    selected.selected_links.insert(resource_key.clone());
    selected
        .selected_link_appearances
        .insert(resource_key.clone(), chosen);
    assert_eq!(
        selected.link_appearance_pair(&resource_key),
        Some((
            graph.get_node(chosen.0).unwrap().id,
            graph.get_node(chosen.1).unwrap().id,
        )),
        "opening a selected Resource key resolves its clicked surface appearance"
    );
    let pair = if a <= b { (a, b) } else { (b, a) };
    let same_display_pair: Vec<_> = appearances
        .iter()
        .filter(|segment| {
            let displayed = if segment.appearance.display_from <= segment.appearance.display_to {
                (
                    segment.appearance.display_from,
                    segment.appearance.display_to,
                )
            } else {
                (
                    segment.appearance.display_to,
                    segment.appearance.display_from,
                )
            };
            displayed == pair
        })
        .collect();
    assert_eq!(same_display_pair.len(), 2, "one stroke per stratum");
    let surface = same_display_pair
        .iter()
        .find(|segment| segment.appearance.key.stratum == LinkStratum::Surface)
        .unwrap();
    let resource = same_display_pair
        .iter()
        .find(|segment| segment.appearance.key.stratum == LinkStratum::Resource)
        .unwrap();
    assert!((surface.from.y - resource.from.y).abs() > 0.0);
}

#[test]
fn content_projection_drives_all_shown_pairs_through_topology_draw_and_hit() {
    use crate::canvas::edge_cells::visible_link_appearances;
    use kernel::graph::RelationKey;
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
    let segments =
        visible_link_appearances(&graph, &view, &HashSet::new(), &HashSet::new(), |_| 36.0);
    assert_eq!(segments.len(), 5, "one appearance per stratum/pair");
    let hidden = HashSet::from([crate::canvas::EdgeCell {
        from: a,
        to: b,
        selector: RelationSelector::Semantic(SemanticSubKind::Cites),
    }]);
    assert_eq!(visible_relation_edges(&graph, &hidden).len(), 11);
    let visible = visible_link_appearances(&graph, &view, &HashSet::new(), &hidden, |_| 36.0);
    assert_eq!(
        visible.len(),
        5,
        "the remaining member still keeps its link visible"
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
    let fold = incipit::curation::FoldRecord::selection(
        incipit::curation::FoldId::new("fold:content"),
        "content",
        [
            graph.get_node(a).unwrap().id.to_string(),
            graph.get_node(alias_a).unwrap().id.to_string(),
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
