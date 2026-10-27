// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0
//! Read current view configuration and topology without advancing its controller.
use super::relationship_topology::{PairInputs, inspect_pair_with_link_hides};
use super::*;
use scenograph::link::LinkKey;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

mod observations;

/// A runtime-only binding to this Canvas owner. Reuses its nonreused reader
/// keys, whose kernel membership witnesses retire removed/recreated appearances.
#[derive(Clone, Debug)]
pub struct DynamicsSubject {
    target: DynamicsTarget,
    members: Vec<(Uuid, u64)>,
}

impl DynamicsTarget {
    fn members(self) -> Vec<Uuid> {
        match self {
            Self::Access { member } => vec![member],
            Self::Pair { first, second } if first == second => vec![first],
            Self::Pair { first, second } => vec![first, second],
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DynamicsTopologyMode {
    RelationRows,
    OccurrenceController,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DynamicsTopologyRow {
    pub key: LinkKey,
    pub displayed_from: Uuid,
    pub displayed_to: Uuid,
    pub origin: DynamicsTopologyOrigin,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DynamicsTopologyOrigin {
    Assertion { statement_id: String },
    Traversal,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DynamicsTarget {
    Access { member: Uuid },
    Pair { first: Uuid, second: Uuid },
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct MemberDynamics {
    pub member: Uuid,
    pub title: String,
    pub role: Role,
    pub item_role: Option<Role>,
    pub group: Option<String>,
    pub explicit_position_pin: bool,
    pub arrangement_pin_installed: bool,
    pub has_arrangement_slot: bool,
    pub anchor_target_configured: bool,
    pub anchor_stiffness: f32,
    pub forme: Option<Uuid>,
    pub forme_hold: bool,
    pub pointer_drag: bool,
    pub axis_locks: String,
    pub face_treatment: String,
    pub face_screen_size: Option<f64>,
    pub rule_face: bool,
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct DynamicsInspection {
    pub target: DynamicsTarget,
    pub paused: bool,
    pub arrangement: Option<String>,
    pub law: String,
    pub law_label: String,
    pub overlays: Vec<String>,
    pub record: Result<Value, String>,
    pub live_stage: Result<Value, String>,
    pub members: Vec<MemberDynamics>,
    pub topology_source: String,
    pub topology_mode: DynamicsTopologyMode,
    pub pair_inputs: Option<PairInputs>,
    pub pair_controller_edges: Option<usize>,
    pub pair_controller_edge_runs: Option<usize>,
    pub pair_topology_rows: Option<Vec<DynamicsTopologyRow>>,
    pub controller_edges: usize,
    pub controller_edge_runs: usize,
    /// Current geometry readback for this displayed pair, separate from physics.
    pub path_response: Option<Value>,
    /// Latest bounded execution evidence for these exact endpoint bodies.
    pub coupling_observation: Value,
    pub limits: Vec<String>,
}
impl Canvas {
    pub fn bind_dynamics_target(&self, target: DynamicsTarget) -> Result<DynamicsSubject, String> {
        let members = target
            .members()
            .into_iter()
            .map(|id| {
                self.described_keys
                    .by_member
                    .get(&id)
                    .copied()
                    .filter(|key| self.member_of_described(*key) == Some(id))
                    .map(|key| (id, key))
                    .ok_or_else(|| format!("Access {id} is absent from this view"))
            })
            .collect::<Result<_, _>>()?;
        Ok(DynamicsSubject { target, members })
    }

    pub fn inspect_bound_dynamics(
        &self,
        subject: &DynamicsSubject,
    ) -> Result<DynamicsInspection, String> {
        for &(id, key) in &subject.members {
            if self.member_of_described(key) != Some(id) {
                return Err(format!(
                    "The inspected appearance {id} ended. Choose an appearance again."
                ));
            }
        }
        self.inspect_dynamics(subject.target)
    }

    pub fn inspect_dynamics(&self, target: DynamicsTarget) -> Result<DynamicsInspection, String> {
        let ids = target.members();
        let mut members = Vec::new();
        for member in ids {
            let (key, node) = self
                .graph
                .get_node_by_id(member)
                .ok_or_else(|| format!("Access {member} is absent from this view"))?;
            let role = self.arrangement_role_of(key);
            let slot = self.arrangement_slot(key).is_some();
            let explicit_pin = self.pinned_nodes.contains(&key);
            let pointer_drag = self.drag.is_some_and(|drag| drag.node == key);
            let forme_hold = self.forme_held.contains(&key) && role != Role::Pinned;
            members.push(MemberDynamics {
                member,
                title: node.title.clone(),
                role,
                item_role: self.member_role(member),
                group: self.role_group_of(key),
                explicit_position_pin: explicit_pin,
                arrangement_pin_installed: self.arrangement_pin_installed(key),
                has_arrangement_slot: slot,
                anchor_target_configured: slot
                    && role == Role::Anchored
                    && !self.physics_paused
                    && !explicit_pin
                    && !pointer_drag,
                anchor_stiffness: self.anchor_stiffness(),
                forme: self
                    .forme_region
                    .as_ref()
                    .filter(|forme| forme.cells.iter().any(|cell| cell.member == member))
                    .map(|forme| forme.id),
                forme_hold,
                pointer_drag,
                axis_locks: format!("{:?}", self.arrangement_axis_locks()),
                face_treatment: self.node_representation_class(key).trim().to_owned(),
                face_screen_size: self.node_presentation_screen_size(member),
                rule_face: self.node_rule_representation(member).is_some(),
            });
        }
        let relation_reading = self.controller_topology_reading();
        let topology_mode = if relation_reading.is_some() {
            DynamicsTopologyMode::RelationRows
        } else {
            DynamicsTopologyMode::OccurrenceController
        };
        let topology = relation_reading
            .as_ref()
            .map(|reading| reading.edge_topology())
            .unwrap_or_else(|| {
                self.occurrence_edges
                    .as_ref()
                    .expect("occurrence topology selected when no relation reading exists")
                    .iter()
                    .copied()
                    .collect()
            });
        let (pair_inputs, pair_controller_edges) = match target {
            DynamicsTarget::Access { .. } => (None, None),
            DynamicsTarget::Pair { first, second } => {
                let a = self.graph.get_node_key_by_id(first).unwrap();
                let b = self.graph.get_node_key_by_id(second).unwrap();
                let count = topology
                    .iter()
                    .filter(|(from, to)| (*from == a && *to == b) || (*from == b && *to == a))
                    .count();
                let inputs = if self.occurrence_edges.is_none() {
                    let mut inputs = inspect_pair_with_link_hides(
                        &self.graph,
                        first,
                        second,
                        &self.hidden_links.iter().cloned().collect(),
                        |from, to, selector| {
                            self.hidden_edges.contains(&EdgeCell { from, to, selector })
                        },
                    )?;
                    // This count comes from the exact source rows that feed the
                    // controller, including assertion hides and withdrawals.
                    inputs.visible_rows = count;
                    Some(inputs)
                } else {
                    None
                };
                (inputs, Some(count))
            },
        };
        let pair_controller_edge_runs = match target {
            DynamicsTarget::Access { .. } => None,
            DynamicsTarget::Pair { first, second } => {
                let a = self.graph.get_node_key_by_id(first).unwrap();
                let b = self.graph.get_node_key_by_id(second).unwrap();
                Some(
                    topology
                        .runs()
                        .iter()
                        .filter(|run| {
                            let (from, to) = run.endpoints();
                            (from == a && to == b) || (from == b && to == a)
                        })
                        .count(),
                )
            },
        };
        let pair_topology_rows = match (&relation_reading, target) {
            (Some(reading), DynamicsTarget::Pair { first, second }) => {
                let a = self.graph.get_node_key_by_id(first).unwrap();
                let b = self.graph.get_node_key_by_id(second).unwrap();
                Some(
                    reading
                        .rows
                        .iter()
                        .filter(|row| {
                            (row.displayed_from == a && row.displayed_to == b)
                                || (row.displayed_from == b && row.displayed_to == a)
                        })
                        .filter_map(|row| {
                            Some(DynamicsTopologyRow {
                                key: row.key.clone(),
                                displayed_from: self.graph.get_node(row.displayed_from)?.id,
                                displayed_to: self.graph.get_node(row.displayed_to)?.id,
                                origin: match &row.origin {
                                    super::seiche_bridge::VisibleRelationOrigin::Assertion {
                                        statement_id,
                                    } => DynamicsTopologyOrigin::Assertion {
                                        statement_id: statement_id.clone(),
                                    },
                                    super::seiche_bridge::VisibleRelationOrigin::Traversal => {
                                        DynamicsTopologyOrigin::Traversal
                                    },
                                },
                            })
                        })
                        .collect(),
                )
            },
            _ => None,
        };
        Ok(DynamicsInspection {
            target, paused: self.physics_paused(), arrangement: self.layout_strategy().map(str::to_owned),
            law: self.physics_law.id().into(), law_label: self.physics_law.label().into(), overlays: self.physics_overlays.iter().map(|overlay| overlay.id().into()).collect(),
            record: self.dynamics_spec().and_then(|spec| serde_json::to_value(spec).map_err(|error| error.to_string())),
            live_stage: self.live_stage_spec().and_then(|spec| serde_json::to_value(spec).map_err(|error| error.to_string())),
            members,
            topology_source: if topology_mode == DynamicsTopologyMode::OccurrenceController { "disclosed occurrence controller" } else { "visible relation source rows" }.into(),
            topology_mode,
            pair_inputs,
            pair_controller_edges,
            pair_controller_edge_runs,
            pair_topology_rows,
            controller_edges: topology.len(),
            controller_edge_runs: topology.run_count(),
            path_response: observations::paths(self, target),
            coupling_observation: observations::couplings(self, target),
            limits: vec![
                "This reads configuration and the current topology input seam; it does not step physics or measure a force vector.".into(),
                "A visible relation row supplies one topology slot, not one force per assertion. The live law/composition determines how those inputs are used.".into(),
                "Counted edge runs compress adjacent identical ordered endpoint rows for storage. Logical row count preserves multiplicity; run count is not a unique-link count or force measure.".into(),
                "For a pair, `pair_topology_rows` are the exact current controller rows, including each assertion ID; traversal rows are identified as rollups without assertion IDs. `pair_inputs` summarizes held-cell groups. Its held row totals include individually hidden source assertions, and its cell hidden flag describes cell or whole-link hiding.".into(),
                "Role policy, installed arrangement pins, explicit position pins, Forme holds and attention state have separate owners. Anchor presence is configuration, not a measured contribution.".into(),
                "Coupling rows describe actual endpoint calls before integration. They do not attribute a net force or final motion to an assertion or link. Other forces, contacts and foreground pins are outside this trace.".into(),
                "The record is current Canvas readback, not a storage acknowledgment or source-history snapshot.".into(),
                "The flat law label is a picker summary. Read the full live stage for mixtures/groupings and the authored record for schedules.".into(),
            ],
        })
    }
}
#[cfg(test)]
#[path = "dynamics_inspection_tests.rs"]
mod tests;

#[cfg(test)]
mod observation_tests;
