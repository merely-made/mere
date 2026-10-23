// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Fresh direct Canvas replay of a bounded two-stage schedule. Witnesses use
//! only public Canvas positions, roles, and schedule-stage readback.

use kernel::geometry::PortablePoint;
use kernel::graph::apply::{GraphDelta, GraphDeltaResult, apply_graph_delta};
use kernel::graph::{EdgeAssertion, Graph, NodeKey, SemanticSubKind};
use pictograph::canvas::Canvas;
use pictograph::canvas::dynamics_spec::DynamicsSpec;
use serde::Deserialize;
use serde_json::json;
use state_witness::{Trace, Witness, label};
use std::collections::{BTreeSet, HashMap};
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

pub fn canonical_case() -> Result<String, String> {
    Ok(include_str!("../schedule_replay.json").to_owned())
}

pub fn run(input: &str) -> Result<String, String> {
    let case: Case = serde_json::from_str(input).map_err(|e| e.to_string())?;
    validate_case(&case)?;
    if case.ticks == 0 || case.ticks > MAX_TICKS {
        return Err(format!("ticks must be between 1 and {MAX_TICKS}"));
    }
    let authored: DynamicsSpec =
        serde_json::from_str(&case.spec).map_err(|e| format!("spec: {e}"))?;
    let (trace, opened, stage_indices) = replay(&case, &authored)?;
    if opened != authored {
        return Err("Canvas dynamics record changed the authored recipe".into());
    }
    let saved = serde_json::to_string(&opened).map_err(|e| e.to_string())?;
    let reopened: DynamicsSpec = serde_json::from_str(&saved).map_err(|e| e.to_string())?;
    if reopened != opened {
        return Err("serialized Canvas recipe changed during reopen".into());
    }
    let (reopened_trace, reopened_record, reopened_stage_indices) = replay(&case, &reopened)?;
    if reopened_record != opened {
        return Err("fresh Canvas reopened a different authored recipe".into());
    }
    Ok(serde_json::to_string(&json!({"trace":trace,"reopened_trace":reopened_trace,"stage_indices":stage_indices,"reopened_stage_indices":reopened_stage_indices,"saved_spec":opened,"ticks":case.ticks})).map_err(|e| e.to_string())?)
}

fn validate_case(case: &Case) -> Result<(), String> {
    if case.nodes.is_empty() || case.nodes.len() > MAX_NODES {
        return Err(format!("node count must be between 1 and {MAX_NODES}"));
    }
    if case.edges.len() > MAX_EDGES {
        return Err(format!("edge count must be at most {MAX_EDGES}"));
    }
    let mut ids = BTreeSet::new();
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

fn replay(
    case: &Case,
    spec: &DynamicsSpec,
) -> Result<(Trace, DynamicsSpec, Vec<Option<usize>>), String> {
    let (graph, keys) = build_graph(case)?;
    let mut canvas = Canvas::with_graph(graph);
    canvas.set_layout_strategy(Some("schedule-replay-seed".into()));
    canvas.apply_strategy_positions(
        &case
            .nodes
            .iter()
            .zip(&keys)
            .map(|(n, k)| (*k, PortablePoint::new(n.x, n.y)))
            .collect::<Vec<_>>(),
    );
    canvas.set_dynamics_spec(spec).map_err(|e| e.to_string())?;
    let opened = canvas
        .dynamics_spec()
        .map_err(|e| format!("Canvas dynamics record: {e}"))?;
    canvas.set_layout_strategy(None);
    let ids: HashMap<_, _> = keys
        .iter()
        .copied()
        .zip(case.nodes.iter().map(|n| n.id.as_str()))
        .collect();
    let mut trace = Trace::new();
    let mut stages = vec![canvas.physics_schedule_stage()];
    trace
        .push(0, capture(&canvas, &keys, &ids)?)
        .map_err(|e| e.to_string())?;
    for tick in 1..=case.ticks {
        let _ = canvas.frame(WIDTH, HEIGHT);
        stages.push(canvas.physics_schedule_stage());
        trace
            .push(tick as u64, capture(&canvas, &keys, &ids)?)
            .map_err(|e| e.to_string())?;
    }
    Ok((trace, opened, stages))
}

fn build_graph(case: &Case) -> Result<(Graph, Vec<NodeKey>), String> {
    let mut graph = Graph::new();
    let mut keys = Vec::new();
    let mut by_id = HashMap::new();
    for n in &case.nodes {
        let id = Uuid::parse_str(&n.id).map_err(|e| e.to_string())?;
        let result = apply_graph_delta(
            &mut graph,
            GraphDelta::AddNode {
                id: Some(id),
                url: n.url.clone(),
                position: PortablePoint::new(n.x, n.y),
            },
        );
        let GraphDeltaResult::NodeAdded(key) = result else {
            return Err(format!("fixed-id node creation returned {result:?}"));
        };
        by_id.insert(n.id.as_str(), key);
        keys.push(key);
    }
    for edge in &case.edges {
        let from = *by_id
            .get(edge.from.as_str())
            .ok_or_else(|| format!("missing edge source {}", edge.from))?;
        let to = *by_id
            .get(edge.to.as_str())
            .ok_or_else(|| format!("missing edge target {}", edge.to))?;
        let result = apply_graph_delta(
            &mut graph,
            GraphDelta::AssertRelation {
                from,
                to,
                assertion: EdgeAssertion::Semantic {
                    sub_kind: SemanticSubKind::UserGrouped,
                    label: Some("schedule replay fixture".into()),
                },
                asserter_iri: "urn:mere:seiche:schedule-replay".into(),
            },
        );
        if !matches!(result, GraphDeltaResult::EdgeAdded(Some(_))) {
            return Err(format!("fixture edge creation returned {result:?}"));
        }
    }
    Ok((graph, keys))
}

fn capture(
    canvas: &Canvas,
    keys: &[NodeKey],
    ids: &HashMap<NodeKey, &str>,
) -> Result<Witness, String> {
    let mut witness = Witness::new();
    for key in keys {
        let id = *ids
            .get(key)
            .ok_or_else(|| "fixture node has no stable UUID label".to_owned())?;
        let p = canvas
            .node_position(*key)
            .ok_or_else(|| format!("Canvas has no position for {id}"))?;
        if !p.x.is_finite() || !p.y.is_finite() {
            return Err(format!("Canvas position for {id} is non-finite"));
        }
        witness
            .insert(
                label("position-bits", [id]),
                &[p.x.to_bits(), p.y.to_bits()],
            )
            .map_err(|e| e.to_string())?;
        witness
            .insert(
                label("role", [id]),
                role_name(canvas.arrangement_role_of(*key)),
            )
            .map_err(|e| e.to_string())?;
    }
    witness
        .insert(
            label("schedule-stage", std::iter::empty::<&str>()),
            &canvas.physics_schedule_stage().map(|n| n as u64),
        )
        .map_err(|e| e.to_string())?;
    Ok(witness)
}
fn role_name(role: seiche::Role) -> &'static str {
    match role {
        seiche::Role::Anchored => "anchored",
        seiche::Role::Pinned => "pinned",
        seiche::Role::Seeded => "seeded",
    }
}
