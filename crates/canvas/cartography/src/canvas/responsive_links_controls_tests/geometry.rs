// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use kernel::geometry::PortablePoint;
use kernel::graph::apply::add_node;
use kernel::graph::{Graph, GraphStratum, SemanticStatementSpec, SemanticSubKind, predicate_iri};
use kernel::types::AssertionBasis;
use uuid::Uuid;

pub(super) fn alias_graph() -> (Graph, [Uuid; 4], Vec<(String, Uuid, Uuid)>) {
    let mut graph = Graph::new();
    graph
        .declare_predicate("urn:test:surface-path", GraphStratum::Surface)
        .unwrap();
    let ids = [
        Uuid::from_u128(0xa1),
        Uuid::from_u128(0xa2),
        Uuid::from_u128(0xb1),
        Uuid::from_u128(0xb2),
    ];
    let urls = [
        "https://alias-a.test/page",
        "https://alias-a.test/page",
        "https://alias-b.test/page",
        "https://alias-b.test/page",
    ];
    let keys: Vec<_> = ids
        .iter()
        .zip(urls)
        .enumerate()
        .map(|(index, (id, url))| {
            let (x, y) = [
                (-180.0, -80.0),
                (-180.0, 80.0),
                (180.0, -80.0),
                (180.0, 80.0),
            ][index];
            add_node(&mut graph, Some(*id), url.into(), PortablePoint::new(x, y))
        })
        .collect();
    let pairs = [(0, 2), (0, 3), (1, 2), (1, 3)];
    let mut assertions = Vec::new();
    for (index, (from, to)) in pairs.into_iter().enumerate() {
        let (source_from, source_to) = if index == 3 { (to, from) } else { (from, to) };
        let statement = graph
            .try_assert_semantic_statement(
                keys[source_from],
                keys[source_to],
                SemanticStatementSpec {
                    predicate: predicate_iri(SemanticSubKind::Cites).into(),
                    recognized_sub_kind: Some(SemanticSubKind::Cites),
                    label: Some(format!("alias assertion {index}")),
                    basis: AssertionBasis {
                        evidence: vec![format!("urn:alias-evidence:{index}")],
                        ..Default::default()
                    },
                    ..Default::default()
                },
            )
            .unwrap()
            .1
            .statement_id;
        assertions.push((statement, ids[source_from], ids[source_to]));
        let surface = graph
            .try_assert_semantic_statement(
                keys[source_from],
                keys[source_to],
                SemanticStatementSpec {
                    predicate: "urn:test:surface-path".into(),
                    basis: AssertionBasis {
                        evidence: vec![format!("urn:surface-evidence:{index}")],
                        ..Default::default()
                    },
                    ..Default::default()
                },
            )
            .unwrap()
            .1
            .statement_id;
        assertions.push((surface, ids[source_from], ids[source_to]));
    }
    (graph, ids, assertions)
}

#[test]
fn both_strata_and_resource_aliases_keep_exact_display_pairs_and_bucket_direction() {
    let (graph, ids, assertions) = alias_graph();
    let mut canvas = Canvas::with_graph(graph);
    for (id, (x, y)) in ids.iter().zip([
        (-180.0, -80.0),
        (-180.0, 80.0),
        (180.0, -80.0),
        (180.0, 80.0),
    ]) {
        let key = canvas.graph.get_node_key_by_id(*id).unwrap();
        canvas
            .view
            .set_position(key, euclid::default::Point2D::new(x, y));
    }
    let paths = canvas.visible_link_paths();
    assert_eq!(
        paths
            .iter()
            .filter(|path| path.stratum == LinkStratum::Surface)
            .count(),
        4
    );
    let resource_paths: Vec<_> = paths
        .iter()
        .filter(|path| path.stratum == LinkStratum::Resource)
        .collect();
    assert_eq!(resource_paths.len(), 4);
    assert_eq!(
        resource_paths
            .iter()
            .map(|path| &path.key)
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        1,
        "the four displayed pairs should alias one canonical resource link"
    );

    for path in &paths {
        let from = ids
            .iter()
            .position(|id| *id == path.displayed_from)
            .unwrap();
        let to = ids.iter().position(|id| *id == path.displayed_to).unwrap();
        assert!(matches!((from, to), (0..=1, 2..=3) | (2..=3, 0..=1)));
        assert!(!path.buckets.is_empty());
        assert_eq!(
            path.buckets
                .iter()
                .map(|bucket| bucket.assertion_ids.len())
                .sum::<usize>(),
            if path.stratum == LinkStratum::Resource {
                4
            } else {
                1
            }
        );
        for bucket in &path.buckets {
            for assertion in &bucket.assertion_ids {
                let (_, source_from, source_to) = assertions
                    .iter()
                    .find(|(id, _, _)| id == assertion)
                    .expect("bucket assertion has a fixture source direction");
                if path.stratum == LinkStratum::Surface {
                    assert_eq!(bucket.displayed_from, source_from.to_string());
                    assert_eq!(bucket.displayed_to, source_to.to_string());
                } else {
                    // Resource claims project onto every displayed alias pair;
                    // their source direction remains within the same Resource sides.
                    let source_is_left = ids[..2].contains(source_from);
                    assert_eq!(
                        ids[..2]
                            .iter()
                            .any(|id| id.to_string() == bucket.displayed_from),
                        source_is_left
                    );
                    assert_eq!(
                        ids[..2]
                            .iter()
                            .any(|id| id.to_string() == bucket.displayed_to),
                        !source_is_left
                    );
                }
            }
        }
        assert_eq!(path.points.len(), 2, "resting ordinary paths are straight");
        let middle = (
            path.points[0].0 * 0.75 + path.points[1].0 * 0.25,
            path.points[0].1 * 0.75 + path.points[1].1 * 0.25,
        );
        let hit = crate::canvas::edge_cells::link_hit_test_segments(
            canvas.rendered_link_segments(),
            euclid::default::Point2D::new(middle.0, middle.1),
            0.5,
            |_| true,
            |_| true,
        )
        .expect("painted path midpoint must remain pickable");
        assert_eq!(hit.key, path.key);
        assert_eq!(
            hit.display_from,
            canvas
                .graph
                .get_node_key_by_id(path.displayed_from)
                .unwrap()
        );
        assert_eq!(
            hit.display_to,
            canvas.graph.get_node_key_by_id(path.displayed_to).unwrap()
        );
    }
}

#[test]
fn exact_assertion_hide_removes_only_that_reason_from_the_painted_source_bucket() {
    let mut graph = Graph::new();
    graph
        .declare_predicate("urn:test:surface-path", GraphStratum::Surface)
        .unwrap();
    let a = Uuid::from_u128(0xc1);
    let b = Uuid::from_u128(0xc2);
    let from = add_node(
        &mut graph,
        Some(a),
        "https://hide-a.test/".into(),
        PortablePoint::zero(),
    );
    let to = add_node(
        &mut graph,
        Some(b),
        "https://hide-b.test/".into(),
        PortablePoint::zero(),
    );
    let mut ids = Vec::new();
    for label in ["first independent reason", "second independent reason"] {
        ids.push(
            graph
                .try_assert_semantic_statement(
                    from,
                    to,
                    SemanticStatementSpec {
                        predicate: predicate_iri(SemanticSubKind::Cites).into(),
                        recognized_sub_kind: Some(SemanticSubKind::Cites),
                        label: Some(label.into()),
                        basis: AssertionBasis {
                            evidence: vec![format!("urn:independent-evidence:{label}")],
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                )
                .unwrap()
                .1
                .statement_id,
        );
    }
    graph
        .try_assert_semantic_statement(
            from,
            to,
            SemanticStatementSpec {
                predicate: "urn:test:surface-path".into(),
                ..Default::default()
            },
        )
        .unwrap();
    let mut canvas = Canvas::with_graph(graph);
    let before_revision = canvas.graph.revision();
    let before = canvas.visible_link_paths();
    let target = before
        .iter()
        .find(|path| path.stratum == LinkStratum::Resource)
        .expect("resource appearance");
    assert_eq!(target.buckets[0].assertion_ids.len(), 2);
    assert!(canvas.set_hidden_assertion_ids([ids[0].clone()]));

    let after = canvas.visible_link_paths();
    let remaining = after
        .iter()
        .find(|path| path.key == target.key && path.displayed_from == target.displayed_from)
        .expect("independent assertion keeps the canonical resource path visible");
    assert_eq!(remaining.buckets.len(), 1);
    assert_eq!(remaining.buckets[0].assertion_ids, vec![ids[1].clone()]);
    assert!(
        after
            .iter()
            .any(|path| path.stratum == LinkStratum::Surface)
    );
    assert_eq!(canvas.graph.revision(), before_revision);
}

#[test]
fn more_than_numen_appearance_limit_refuses_without_truncating_rest_paths() {
    let mut graph = Graph::new();
    for index in 0..130u128 {
        add_node(
            &mut graph,
            Some(Uuid::from_u128(10_000 + index)),
            format!("https://bounded-{index}.test/"),
            PortablePoint::zero(),
        );
    }
    for index in 0..65u128 {
        let from = graph
            .get_node_key_by_id(Uuid::from_u128(10_000 + index * 2))
            .unwrap();
        let to = graph
            .get_node_key_by_id(Uuid::from_u128(10_001 + index * 2))
            .unwrap();
        graph
            .try_assert_semantic_statement(
                from,
                to,
                SemanticStatementSpec {
                    predicate: predicate_iri(SemanticSubKind::Cites).into(),
                    recognized_sub_kind: Some(SemanticSubKind::Cites),
                    label: Some(format!("bounded link {index}")),
                    ..Default::default()
                },
            )
            .unwrap();
    }
    let mut canvas = Canvas::with_graph(graph);
    let before_revision = canvas.graph.revision();
    let resting = geometry(&canvas);
    assert!(resting.len() > numen::MAX_LINKS);
    let mut response = LinkPathResponse::default();
    response.field_response = true;
    response.view_field = Some(VectorField::ConstVec { x: 0.0, y: 500.0 }.into());
    response.config.control_points = 9;
    response.config.iterations = 8;
    canvas.set_link_path_response(response);

    let report = canvas.link_path_response_report();
    assert_eq!(report.state, LinkPathState::Refused, "{report:?}");
    assert!(matches!(report.failure, Some(LinkPathFailure::Adapter(_))));
    assert_eq!(
        geometry(&canvas),
        resting,
        "overflow must keep all resting geometry"
    );
    assert_eq!(report.paths.len(), resting.len());
    assert!(report.paths.iter().all(|path| !path.responsive));
    assert_eq!(canvas.graph.revision(), before_revision);
}

#[test]
fn self_link_keeps_its_resting_arch_while_other_links_respond() {
    let mut canvas = Canvas::with_sample_graph();
    let node_key = canvas.graph.nodes().next().unwrap().0;
    let node_id = canvas.graph.get_node(node_key).unwrap().id;
    canvas
        .graph
        .try_assert_semantic_statement(
            node_key,
            node_key,
            SemanticStatementSpec {
                predicate: predicate_iri(SemanticSubKind::Cites).into(),
                recognized_sub_kind: Some(SemanticSubKind::Cites),
                label: Some("ordinary self link".into()),
                ..Default::default()
            },
        )
        .unwrap();
    let before = canvas.visible_link_paths();
    let self_before: Vec<_> = before
        .iter()
        .filter(|path| path.displayed_from == node_id && path.displayed_to == node_id)
        .map(|path| (path.key.clone(), path.points.clone()))
        .collect();
    assert!(!self_before.is_empty());
    assert!(self_before.iter().all(|(_, points)| points.len() > 2));

    let mut response = LinkPathResponse::default();
    response.field_response = true;
    response.view_field = Some(VectorField::ConstVec { x: 0.0, y: 500.0 }.into());
    response.config.control_points = 9;
    response.config.iterations = 8;
    response.config.field_gain = 1.0;
    response.config.max_step = 20.0;
    canvas.set_link_path_response(response);

    let report = canvas.link_path_response_report();
    assert_eq!(report.state, LinkPathState::Applied, "{report:?}");
    assert!(report.ineligible_self_links >= self_before.len());
    assert!(
        report
            .paths
            .iter()
            .any(|path| { path.responsive && path.displayed_from != path.displayed_to })
    );
    for (key, points) in self_before {
        let path = report
            .paths
            .iter()
            .find(|path| {
                path.key == key && path.displayed_from == node_id && path.displayed_to == node_id
            })
            .unwrap();
        assert!(!path.responsive);
        assert_eq!(path.points, points);
    }
}
