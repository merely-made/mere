// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The graph ↔ seiche adapters — the canvas-side bridge that keeps seiche
//! kernel-free (S0 of the spatial completion): seiche takes plain
//! `(NodeKey, …)` lists and force values; everything that reads a kernel
//! [`Graph`] to produce them lives here. Split out of `build.rs` (which keeps
//! the DOM/paint construction helpers) when that file neared the workspace
//! size ceiling; this module also hosts the graph-resolution tests that were
//! deferred at the seiche extraction (the three `from_coupling` tests removed
//! with seiche's `kernel-bridge` feature).

use std::collections::HashSet;

use euclid::default::Point2D;
use kernel::graph::{Graph, NodeKey, RelationKey};
use numen::FieldRegistry;
use scenograph::link::{LinkKey, LinkStratum};
use seiche::{Boundary, CouplingForce, EdgeSpring, NodeExclusion, Simulation};

/// The relation-cell edges that feed the layout **springs**, respecting this instance's
/// hidden-cell visibility (swatch-primitive P5 — hiding relaxes the spring in that
/// instance only, never graph truth). Unlike `dedup_edges` (one topology edge per pair,
/// for graph-algorithm layout strategies that need plain connectivity, not weight), this
/// keeps one `(NodeKey, NodeKey)` tuple per **visible** relation cell: a pair with three
/// live cells pulls three times as hard as a pair with one, and hiding one cell drops the
/// pull by exactly that cell's share. seiche stays relation-taxonomy agnostic — this
/// multiplicity is how the canvas hands it weight without leaking `RelationSelector` into
/// seiche's edge type.
pub(crate) fn visible_relation_edges(
    graph: &Graph,
    hidden_edges: &HashSet<crate::canvas::EdgeCell>,
) -> Vec<(NodeKey, NodeKey)> {
    visible_relation_edges_with_link_hides(graph, hidden_edges, &HashSet::new())
}

/// Keep the historical one-spring-per-visible-source-row law. A canonical
/// link hide removes only rows in its owning stratum; another stratum over the
/// same displayed pair keeps its own rows and spring multiplicity.
pub(crate) fn visible_relation_edges_with_link_hides(
    graph: &Graph,
    hidden_edges: &HashSet<crate::canvas::EdgeCell>,
    hidden_links: &HashSet<LinkKey>,
) -> Vec<(NodeKey, NodeKey)> {
    visible_relation_edges_with_assertion_hides(graph, hidden_edges, hidden_links, &HashSet::new())
}

pub(crate) fn visible_relation_edges_with_assertion_hides(
    graph: &Graph,
    hidden_edges: &HashSet<crate::canvas::EdgeCell>,
    hidden_links: &HashSet<LinkKey>,
    hidden_assertions: &HashSet<String>,
) -> Vec<(NodeKey, NodeKey)> {
    let assertions = graph
        .projected_assertions()
        .filter(|(_, _, _, statement)| !hidden_assertions.contains(&statement.statement_id))
        .map(|(owner, from, to, statement)| {
            (
                owner,
                kernel::graph::RelationView {
                    from,
                    to,
                    kind: statement.relation_kind(),
                },
            )
        });
    let traversals = graph
        .projected_relations()
        .filter(|(_, relation)| relation.kind == kernel::graph::RelationKind::Traversal);
    assertions
        .chain(traversals)
        .filter_map(|(relation_key, relation)| {
            let selector = super::relationship_topology::selector_for_relation_kind(relation.kind);
            if hidden_edges.contains(&crate::canvas::EdgeCell {
                from: relation.from,
                to: relation.to,
                selector,
            }) {
                return None;
            }
            let key = match relation_key {
                RelationKey::Surface(_) => {
                    let from = graph.get_node(relation.from)?.id.to_string();
                    let to = graph.get_node(relation.to)?.id.to_string();
                    LinkKey::new(LinkStratum::Surface, from, to)
                },
                RelationKey::Resource(_) => {
                    let from_id = graph.shown_resource_id(relation.from)?;
                    let to_id = graph.shown_resource_id(relation.to)?;
                    let from = graph.resource(from_id)?.canonical_iri().to_owned();
                    let to = graph.resource(to_id)?.canonical_iri().to_owned();
                    LinkKey::new(LinkStratum::Resource, from, to)
                },
            };
            (!hidden_links.contains(&key)).then_some((relation.from, relation.to))
        })
        .collect()
}

/// Build the force-directed simulation from `graph`: a body per node, one spring
/// edge per visible relation cell as the topology (see [`visible_relation_edges`]),
/// the standard force trio (exclusion + edge-springs + a centering boundary), seeded
/// into a tight central spiral so the first settle is visible.
pub(crate) fn build_simulation(graph: &Graph) -> Simulation {
    let mut sim = Simulation::new();
    sync_sim_with_graph(&mut sim, graph);
    // No `Canvas` (and so no hidden-cell set) exists yet at construction time; the first
    // real sync happens once session/view-intent restore runs and calls `reconcile_derived`.
    sim.sync_edges(visible_relation_edges(graph, &HashSet::new()));

    sim.add_force(NodeExclusion::default());
    sim.add_force(EdgeSpring::default());
    sim.add_force(Boundary::default());
    // Field couplings: each resolves to a CouplingForce seiche integrates, so a placed
    // field's well actually pulls its nodes. The live place / move / new-node rebuild
    // re-resolves these via `Physics::set_coupling_forces`. (Field regions.)
    let coupling_forces: Vec<CouplingForce> = graph
        .couplings()
        .filter_map(|c| coupling_force_from_graph(c, graph))
        .collect();
    sim.set_coupling_forces(coupling_forces);

    sim.seed_positions(seed_cluster(graph));
    sim
}

/// Reconcile a simulation's bodies to a kernel [`Graph`]'s nodes — the canvas-side
/// bridge now that seiche is kernel-free. seiche's [`Simulation::sync_nodes`] takes
/// a `(NodeKey, position)` list; positions are no longer graph truth (S2), so new
/// bodies spawn at the origin and the caller places them via [`Simulation::
/// seed_positions`] (a spiral seed, or the host's cartography facets). Existing
/// bodies keep their simulated position. (Was `seiche::Simulation::sync_with_graph`
/// before the seiche extraction.)
pub(crate) fn sync_sim_with_graph(sim: &mut Simulation, graph: &Graph) {
    sim.sync_nodes(graph.nodes().map(|(key, _node)| (key, Point2D::zero())));
}

/// Resolve a coupling against the graph into a [`CouplingForce`] — the canvas-side
/// bridge (was `seiche::CouplingForce::from_coupling`). Looks up the field
/// definition and the selector's matching nodes, seeding the registry from the
/// whole field layer so inter-field `Sample` references resolve. `None` if the
/// field id is unknown. The captured target set is a snapshot; rebuild on graph
/// mutation.
pub(crate) fn coupling_force_from_graph(
    coupling: &kernel::graph::Coupling,
    graph: &Graph,
) -> Option<CouplingForce> {
    let field = graph.field(coupling.field)?;
    if !field.is_active() {
        return None;
    }
    let targets: Vec<NodeKey> = graph.nodes_matching(&coupling.selector).collect();
    let mut registry = FieldRegistry::new();
    for f in graph.fields() {
        registry.insert_with_id(f.id, f.definition.clone());
    }
    Some(
        CouplingForce::new(
            coupling.response.clone(),
            coupling.strength,
            targets,
            field.definition.clone(),
        )
        .with_registry(registry)
        .with_extent(field.extent.clone())
        .with_selector(coupling.selector.clone())
        .with_provenance(
            seiche::CouplingSource {
                field: coupling.field,
                coupling: coupling.id,
                revision: Some(graph.revision()),
            },
            |_, key| {
                graph
                    .get_node(key)
                    .map(|node| seiche::CouplingTargetIdentity {
                        member: node.id.as_u128(),
                        occurrence: None,
                    })
            },
        ),
    )
}

/// The tight central spiral (golden-angle) seed for every node, so a ticked
/// settle visibly expands it into a readable layout. Returns the `(node,
/// position)` pairs; the caller applies them to the simulation (in-thread) or
/// sends them to the physics actor (offloaded), and mirrors them into the view.
pub(crate) fn seed_cluster(graph: &Graph) -> Vec<(NodeKey, Point2D<f32>)> {
    graph
        .nodes()
        .enumerate()
        .map(|(i, (key, _node))| {
            let r = 6.0 + i as f32 * 3.0;
            let theta = i as f32 * 2.399_963; // golden angle in radians
            (key, Point2D::new(r * theta.cos(), r * theta.sin()))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use kernel::geometry::PortablePoint;
    use kernel::graph::apply::{GraphDelta, apply_graph_delta};
    use kernel::graph::fixtures::GraphFixtures;
    use kernel::graph::{
        COUPLING_VOCAB, Coupling, CouplingId, CouplingResponse, Field, FieldDefinition, FieldId,
        NodeSelector, ScalarField,
    };
    use uuid::Uuid;

    fn graph_with_nodes(n: u128) -> (Graph, Vec<NodeKey>) {
        let mut graph = Graph::new();
        let keys = (0..n)
            .map(|i| {
                graph.add_node_with_id(
                    Uuid::from_u128(i + 1),
                    format!("mere://{i}"),
                    PortablePoint::zero(),
                )
            })
            .collect();
        (graph, keys)
    }

    fn gaussian_field(graph: &mut Graph, id: u128) -> FieldId {
        let fid = FieldId::from_uuid(Uuid::from_u128(id));
        graph.add_field(Field::new(
            fid,
            FieldDefinition::Scalar(ScalarField::gaussian_at(0.0, 0.0, 50.0)),
        ));
        fid
    }

    fn force_coupling(fid: FieldId, selector: NodeSelector) -> Coupling {
        Coupling::new(
            CouplingId::from_uuid(Uuid::from_u128(0xC0)),
            fid,
            selector,
            CouplingResponse::open(format!("{COUPLING_VOCAB}force/attract")),
            1.0,
        )
    }

    // The deferred graph-resolution coverage (the three `from_coupling` tests
    // that left with seiche's kernel-bridge feature), rehomed on the canvas
    // side of the bridge.

    #[test]
    fn coupling_resolves_field_and_selector_targets() {
        let (mut graph, keys) = graph_with_nodes(3);
        let fid = gaussian_field(&mut graph, 0xF1);
        let coupling = force_coupling(fid, NodeSelector::All);
        let force = coupling_force_from_graph(&coupling, &graph).expect("field id resolves");
        assert_eq!(force.target_count(), keys.len(), "All selects every node");
    }

    #[test]
    fn coupling_with_unknown_field_resolves_to_none() {
        let (graph, _) = graph_with_nodes(2);
        let coupling = force_coupling(
            FieldId::from_uuid(Uuid::from_u128(0xDEAD)),
            NodeSelector::All,
        );
        assert!(
            coupling_force_from_graph(&coupling, &graph).is_none(),
            "an unknown field id yields no force"
        );
    }

    #[test]
    fn coupling_selector_narrows_the_target_snapshot() {
        let (mut graph, keys) = graph_with_nodes(3);
        let fid = gaussian_field(&mut graph, 0xF1);
        // Tag exactly one node through the public delta path; the Tagged
        // selector's snapshot then holds just that node.
        let _ = apply_graph_delta(
            &mut graph,
            GraphDelta::InsertNodeTag {
                key: keys[1],
                tag: "well".to_string(),
            },
        );
        let coupling = force_coupling(fid, NodeSelector::Tagged("well".to_string()));
        let force = coupling_force_from_graph(&coupling, &graph).expect("field id resolves");
        assert_eq!(
            force.target_count(),
            1,
            "the selector's snapshot holds exactly the tagged node"
        );
    }

    #[test]
    fn build_simulation_wires_couplings_in() {
        let (mut graph, _) = graph_with_nodes(2);
        let fid = gaussian_field(&mut graph, 0xF1);
        graph.add_coupling(force_coupling(fid, NodeSelector::All));
        let sim = build_simulation(&graph);
        assert_eq!(
            sim.coupling_force_count(),
            1,
            "the placed coupling resolves into the built simulation"
        );
    }
}

#[cfg(test)]
mod assertion_hide_controls {
    use super::*;
    use kernel::graph::{ContainmentSubKind, RelationKind, SemanticStatementSpec};
    use kernel::types::AssertionBasis;

    #[test]
    fn exact_hide_removes_one_reason_spring_without_hiding_its_same_kind_sibling() {
        let mut graph = Graph::new();
        use kernel::graph::fixtures::GraphFixtures;
        let from = graph.add_node("urn:assertion-hide:a".into(), Default::default());
        let to = graph.add_node("urn:assertion-hide:b".into(), Default::default());
        let mut ids = Vec::new();
        for evidence in ["urn:reason:one", "urn:reason:two"] {
            let (_, result) = graph
                .try_assert_relation_statement(
                    from,
                    to,
                    SemanticStatementSpec {
                        predicate: RelationKind::Containment(ContainmentSubKind::CollectionMember)
                            .predicate_iri()
                            .unwrap()
                            .into(),
                        basis: AssertionBasis {
                            evidence: vec![evidence.into()],
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                )
                .unwrap();
            ids.push(result.statement_id);
        }
        let before = graph.to_snapshot_at(0);
        assert_eq!(
            visible_relation_edges(&graph, &HashSet::new()),
            vec![(from, to), (from, to)]
        );
        let hidden = HashSet::from([ids[0].clone()]);
        assert_eq!(
            visible_relation_edges_with_assertion_hides(
                &graph,
                &HashSet::new(),
                &HashSet::new(),
                &hidden
            ),
            vec![(from, to)]
        );
        assert_eq!(
            visible_relation_edges_with_assertion_hides(
                &graph,
                &HashSet::new(),
                &HashSet::new(),
                &HashSet::from(["urn:missing".into()])
            ),
            vec![(from, to), (from, to)]
        );
        assert_eq!(
            serde_json::to_value(graph.to_snapshot_at(0)).unwrap(),
            serde_json::to_value(before).unwrap()
        );
    }
}
