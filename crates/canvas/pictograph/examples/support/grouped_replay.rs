// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Direct Canvas replay of an authored grouped dynamics recipe. This adapter
//! observes only public Canvas group membership, roles, and positions.

use kernel::geometry::PortablePoint;
use kernel::graph::apply::{GraphDelta, GraphDeltaResult, apply_graph_delta};
use kernel::graph::{EdgeAssertion, Graph, NodeKey, SemanticSubKind};
use pictograph::canvas::Canvas;
use pictograph::canvas::dynamics_spec::{DynamicsSpec, Node as DynamicsNode};
use pictograph::canvas::physics_catalog::PhysicsKindSource;
use seiche::Role;
use serde::Deserialize;
use serde_json::json;
use state_witness::{Trace, Witness, label};
use std::collections::{BTreeMap, HashMap};
use uuid::Uuid;

const MAX_NODES: usize = 16;
const MAX_EDGES: usize = 32;
const MAX_TICKS: u32 = 600;
const WIDTH: u32 = 1024;
const HEIGHT: u32 = 768;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    nodes: Vec<NodeInput>,
    edges: Vec<EdgeInput>,
    /// The exact authored wire recipe; it is also the saved recipe carrier.
    spec: String,
    ticks: u32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NodeInput {
    id: String,
    url: String,
    x: f32,
    y: f32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct EdgeInput {
    from: String,
    to: String,
}

/// The canonical input is kept separate from this runtime so native and Wasm
/// execute byte-identical fixture data.
pub fn canonical_case() -> Result<String, String> {
    Ok(include_str!("../grouped_replay.json").to_owned())
}

pub fn run(input: &str) -> Result<String, String> {
    let case: Case = serde_json::from_str(input).map_err(|error| error.to_string())?;
    validate_case(&case)?;
    let authored: DynamicsSpec =
        serde_json::from_str(&case.spec).map_err(|error| format!("spec: {error}"))?;
    if case.ticks == 0 || case.ticks > MAX_TICKS {
        return Err(format!("ticks must be between 1 and {MAX_TICKS}"));
    }
    let partition = partition_source(&authored)?;

    let (trace, opened, group_sizes) = replay(&case, &authored, partition)?;
    if opened != authored {
        return Err("Canvas dynamics record changed the authored recipe".into());
    }
    let saved_recipe = serde_json::to_string(&opened).map_err(|error| error.to_string())?;
    let reopened: DynamicsSpec =
        serde_json::from_str(&saved_recipe).map_err(|error| error.to_string())?;
    if reopened != opened {
        return Err("serialized Canvas recipe changed during reopen".into());
    }

    // Recipe reopen: a second graph and Canvas are built from the same fixture
    // and the recipe the first Canvas reported. No physics/checkpoint state is
    // carried between these independent runs.
    let (reopened_trace, reopened_record, reopened_group_sizes) =
        replay(&case, &reopened, partition)?;
    if reopened_record != opened {
        return Err("fresh Canvas reopened a different authored recipe".into());
    }
    if group_sizes != vec![2, 2] || reopened_group_sizes != group_sizes {
        return Err(format!(
            "expected two Canvas groups of two members; got {group_sizes:?} and {reopened_group_sizes:?}"
        ));
    }
    serde_json::to_string(&json!({
        "trace": trace,
        "reopened_trace": reopened_trace,
        "saved_spec": opened,
        "ticks": case.ticks,
        "partition": partition_id(partition),
        "initial_group_sizes": group_sizes,
    }))
    .map_err(|error| error.to_string())
}

fn validate_case(case: &Case) -> Result<(), String> {
    if case.nodes.is_empty() || case.nodes.len() > MAX_NODES {
        return Err(format!("node count must be between 1 and {MAX_NODES}"));
    }
    if case.edges.len() > MAX_EDGES {
        return Err(format!("edge count must be at most {MAX_EDGES}"));
    }
    let mut ids = std::collections::BTreeSet::new();
    for node in &case.nodes {
        let id =
            Uuid::parse_str(&node.id).map_err(|error| format!("node id {:?}: {error}", node.id))?;
        if !ids.insert(id) {
            return Err(format!("duplicate node id {}", node.id));
        }
        if node.url.trim().is_empty() || !node.x.is_finite() || !node.y.is_finite() {
            return Err(format!("node {} has invalid url or position", node.id));
        }
    }
    for edge in &case.edges {
        if edge.from == edge.to
            || !ids.contains(&Uuid::parse_str(&edge.from).map_err(|e| e.to_string())?)
            || !ids.contains(&Uuid::parse_str(&edge.to).map_err(|e| e.to_string())?)
        {
            return Err(format!(
                "edge {} -> {} has invalid endpoints",
                edge.from, edge.to
            ));
        }
    }
    Ok(())
}

fn partition_source(spec: &DynamicsSpec) -> Result<PhysicsKindSource, String> {
    let DynamicsNode::Grouped { partition, .. } = &spec.root else {
        return Err("spec.root must be one flat grouped recipe".into());
    };
    match partition.as_str() {
        "groups.component" => Ok(PhysicsKindSource::Component),
        "groups.coloring" => Ok(PhysicsKindSource::Coloring),
        // Route this negative through the Canvas binder so the failure records
        // its public root.partition path before capture ever asks for a channel.
        "groups.bridges" => Ok(PhysicsKindSource::Component),
        _ => Err(format!("unsupported fixture partition {partition:?}")),
    }
}

fn replay(
    case: &Case,
    spec: &DynamicsSpec,
    partition: PhysicsKindSource,
) -> Result<(Trace, DynamicsSpec, Vec<usize>), String> {
    let (graph, keys) = build_graph(case)?;
    let mut canvas = Canvas::with_graph(graph);
    canvas.set_layout_strategy(Some("grouped-replay-seed".into()));
    canvas.apply_strategy_positions(
        &case
            .nodes
            .iter()
            .zip(&keys)
            .map(|(node, key)| (*key, PortablePoint::new(node.x, node.y)))
            .collect::<Vec<_>>(),
    );
    canvas
        .set_dynamics_spec(spec)
        .map_err(|error| error.to_string())?;
    let opened = canvas
        .dynamics_spec()
        .map_err(|error| format!("Canvas dynamics record: {error}"))?;
    if &opened != spec {
        return Err("Canvas dynamics record differs from the authored recipe".into());
    }
    canvas.set_layout_strategy(None);

    let id_by_key: HashMap<_, _> = keys
        .iter()
        .copied()
        .zip(case.nodes.iter().map(|node| node.id.as_str()))
        .collect();
    let (initial, group_sizes) = capture(&mut canvas, partition, &keys, &id_by_key)?;
    let mut trace = Trace::new();
    trace.push(0, initial).map_err(|error| error.to_string())?;
    for tick in 1..=case.ticks {
        let _ = canvas.frame(WIDTH, HEIGHT);
        trace
            .push(
                tick as u64,
                capture(&mut canvas, partition, &keys, &id_by_key)?.0,
            )
            .map_err(|error| error.to_string())?;
    }
    Ok((trace, opened, group_sizes))
}

fn build_graph(case: &Case) -> Result<(Graph, Vec<NodeKey>), String> {
    let mut graph = Graph::new();
    let mut keys = Vec::with_capacity(case.nodes.len());
    let mut key_by_id = HashMap::new();
    for node in &case.nodes {
        let id = Uuid::parse_str(&node.id).map_err(|error| error.to_string())?;
        let result = apply_graph_delta(
            &mut graph,
            GraphDelta::AddNode {
                id: Some(id),
                url: node.url.clone(),
                position: PortablePoint::new(node.x, node.y),
            },
        );
        let GraphDeltaResult::NodeAdded(key) = result else {
            return Err(format!("fixed-id node creation returned {result:?}"));
        };
        if key_by_id.insert(node.id.as_str(), key).is_some() {
            return Err(format!("duplicate node id {}", node.id));
        }
        keys.push(key);
    }
    for edge in &case.edges {
        let from = *key_by_id
            .get(edge.from.as_str())
            .ok_or_else(|| format!("missing edge source {}", edge.from))?;
        let to = *key_by_id
            .get(edge.to.as_str())
            .ok_or_else(|| format!("missing edge target {}", edge.to))?;
        let result = apply_graph_delta(
            &mut graph,
            GraphDelta::AssertRelation {
                from,
                to,
                assertion: EdgeAssertion::Semantic {
                    sub_kind: SemanticSubKind::UserGrouped,
                    label: Some("grouped replay fixture".into()),
                },
                asserter_iri: "urn:mere:seiche:grouped-replay".into(),
            },
        );
        if !matches!(result, GraphDeltaResult::EdgeAdded(Some(_))) {
            return Err(format!("fixture edge creation returned {result:?}"));
        }
    }
    Ok((graph, keys))
}

fn capture(
    canvas: &mut Canvas,
    partition: PhysicsKindSource,
    keys: &[NodeKey],
    id_by_key: &HashMap<NodeKey, &str>,
) -> Result<(Witness, Vec<usize>), String> {
    let groups = canvas.channel_groups(partition);
    let group_by_key: HashMap<_, _> = groups.into_iter().collect();
    if group_by_key.len() != keys.len() {
        return Err("Canvas groups channel omitted a fixture node".into());
    }
    let mut smallest: BTreeMap<u32, String> = BTreeMap::new();
    for key in keys {
        let id = *id_by_key
            .get(key)
            .ok_or_else(|| "fixture node has no stable UUID label".to_owned())?;
        let group = *group_by_key
            .get(key)
            .ok_or_else(|| format!("Canvas groups channel omitted {id}"))?;
        smallest
            .entry(group)
            .and_modify(|held| {
                if id < held.as_str() {
                    held.clear();
                    held.push_str(id);
                }
            })
            .or_insert_with(|| id.to_owned());
    }
    let mut size_by_group = BTreeMap::<u32, usize>::new();
    for group in group_by_key.values() {
        *size_by_group.entry(*group).or_default() += 1;
    }
    let group_sizes = size_by_group.values().copied().collect::<Vec<_>>();
    let mut witness = Witness::new();
    for key in keys {
        let id = *id_by_key
            .get(key)
            .ok_or_else(|| "fixture node has no stable UUID label".to_owned())?;
        let group = group_by_key[key];
        let group_label = smallest
            .get(&group)
            .ok_or_else(|| format!("Canvas group {group} has no member"))?;
        let point = canvas
            .node_position(*key)
            .ok_or_else(|| format!("Canvas has no position for {id}"))?;
        if !point.x.is_finite() || !point.y.is_finite() {
            return Err(format!("Canvas position for {id} is non-finite"));
        }
        witness
            .insert(label("group", [id]), group_label)
            .map_err(|error| error.to_string())?;
        witness
            .insert(
                label("position-bits", [id]),
                &[point.x.to_bits(), point.y.to_bits()],
            )
            .map_err(|error| error.to_string())?;
        witness
            .insert(
                label("role", [id]),
                role_name(canvas.arrangement_role_of(*key)),
            )
            .map_err(|error| error.to_string())?;
    }
    Ok((witness, group_sizes))
}

fn partition_id(source: PhysicsKindSource) -> &'static str {
    match source {
        PhysicsKindSource::Component => "groups.component",
        PhysicsKindSource::Coloring => "groups.coloring",
        _ => "groups.unsupported",
    }
}

fn role_name(role: Role) -> &'static str {
    match role {
        Role::Anchored => "anchored",
        Role::Pinned => "pinned",
        Role::Seeded => "seeded",
    }
}
