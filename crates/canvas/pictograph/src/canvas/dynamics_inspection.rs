// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0
//! Read current view configuration and topology without advancing its controller.
use super::relationship_topology::{PairInputs, inspect_pair_with_link_hides};
use super::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

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
    pub pair_inputs: Option<PairInputs>,
    pub pair_controller_edges: Option<usize>,
    pub controller_edges: usize,
    pub limits: Vec<String>,
}
impl Canvas {
    pub fn inspect_dynamics(&self, target: DynamicsTarget) -> Result<DynamicsInspection, String> {
        let ids: Vec<_> = match target {
            DynamicsTarget::Access { member } => vec![member],
            DynamicsTarget::Pair { first, second } if first == second => vec![first],
            DynamicsTarget::Pair { first, second } => vec![first, second],
        };
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
        let edges = self.controller_edges();
        let (pair_inputs, pair_controller_edges) = match target {
            DynamicsTarget::Access { .. } => (None, None),
            DynamicsTarget::Pair { first, second } => {
                let a = self.graph.get_node_key_by_id(first).unwrap();
                let b = self.graph.get_node_key_by_id(second).unwrap();
                let count = edges
                    .iter()
                    .filter(|(from, to)| (*from == a && *to == b) || (*from == b && *to == a))
                    .count();
                let inputs = if self.occurrence_edges.is_none() {
                    Some(inspect_pair_with_link_hides(
                        &self.graph,
                        first,
                        second,
                        &self.hidden_links.iter().cloned().collect(),
                        |from, to, selector| {
                            self.hidden_edges.contains(&EdgeCell { from, to, selector })
                        },
                    )?)
                } else {
                    None
                };
                (inputs, Some(count))
            },
        };
        Ok(DynamicsInspection {
            target, paused: self.physics_paused(), arrangement: self.layout_strategy().map(str::to_owned),
            law: self.physics_law.id().into(), law_label: self.physics_law.label().into(), overlays: self.physics_overlays.iter().map(|overlay| overlay.id().into()).collect(),
            record: self.dynamics_spec().and_then(|spec| serde_json::to_value(spec).map_err(|error| error.to_string())),
            live_stage: self.live_stage_spec().and_then(|spec| serde_json::to_value(spec).map_err(|error| error.to_string())),
            members, topology_source: if self.occurrence_edges.is_some() { "disclosed occurrence controller" } else { "held relation cells" }.into(),
            pair_inputs, pair_controller_edges, controller_edges: edges.len(),
            limits: vec![
                "This reads configuration and the current topology input seam; it does not step physics or measure a force vector.".into(),
                "A visible relation row supplies one topology slot, not one force per assertion. The live law/composition determines how those inputs are used.".into(),
                "Role policy, installed arrangement pins, explicit position pins, Forme holds and attention state have separate owners. Anchor presence is configuration, not a measured contribution.".into(),
                "Per-assertion force attribution, installed coupling target snapshots, derived display-link execution evidence and foreground pins are not available here.".into(),
                "The record is current Canvas readback, not a storage acknowledgment or source-history snapshot.".into(),
                "The flat law label is a picker summary. Read the full live stage for mixtures/groupings and the authored record for schedules.".into(),
            ],
        })
    }
}
#[cfg(test)]
#[path = "dynamics_inspection_tests.rs"]
mod tests;
