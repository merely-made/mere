// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! An occurrence-scoped dynamics preview (Scenograph SE89). The private
//! controller is disposable. Its body identities and bare input graph never
//! replace the dataset's source references or become source facts.

use euclid::default::Point2D;
use kernel::graph::apply::add_node;
use kernel::graph::{Graph, NodeKey};
use scenograph::{DynamicsSlot, ProjectionDefinition};
use scenomise::projection::CompiledProjection;
use seiche::{DynamicsRunner, NodeCollider, SettleEnd, SettleError, SettleReport};
use std::collections::BTreeMap;
use uuid::Uuid;

use super::Canvas;
use super::dynamics_recipe::{Progress, bind_slot};
use super::dynamics_spec::{BindError, DynamicsSpec};

pub struct ProjectionDynamics {
    canvas: Canvas,
    spec: DynamicsSpec,
    slot: DynamicsSlot,
    progress: Progress,
    scene: sceno::Scene,
    keys: Vec<NodeKey>,
    started: bool,
    finished: Option<SettleEnd>,
    ticks: u64,
}

fn refuse(place: &str, reason: &str) -> BindError {
    BindError::Unbound {
        place: place.into(),
        reason: reason.into(),
    }
}

impl ProjectionDynamics {
    /// Bind the recipe over its exact compiled occurrences. This first host
    /// supports one flat, relation-free scene and the catalog's bare inputs.
    /// Named channels and group roles need a disclosed-input adapter.
    pub fn new(
        definition: &ProjectionDefinition,
        compiled: &CompiledProjection,
        slot: &DynamicsSlot,
    ) -> Result<Self, BindError> {
        let mut spec = bind_slot(slot, &definition.arrangement.kind)?;
        if !spec.channels.is_empty() || spec.target.as_ref().is_some_and(|t| t.groups.is_some()) {
            return Err(refuse(
                "dynamics.spec.channels",
                "this occurrence adapter has no disclosed physics channels or groups",
            ));
        }
        if !compiled.scene.relations.is_empty()
            || !compiled.scene.backdrops.is_empty()
            || !compiled.scene.regions.is_empty()
            || !compiled.scene.folds.is_empty()
            || compiled
                .scene
                .items
                .iter()
                .any(|item| item.space != sceno::Scene::WORLD)
        {
            return Err(refuse(
                "settle.inputs.scene",
                "this occurrence adapter requires a flat scene without relations, regions, folds or backdrops",
            ));
        }
        let mut graph = Graph::new();
        let mut keys = Vec::new();
        let mut positions = Vec::new();
        let mut ids = Vec::new();
        let namespace = Uuid::from_u128(0x7363_656e_6f67_7261_7068_6479_6e61_6d69);
        for (index, item) in compiled.scene.items.iter().enumerate() {
            let instance = sceno::InstanceId(index as u32);
            let occurrence = compiled
                .occurrence_by_instance
                .get(&instance)
                .ok_or_else(|| {
                    refuse(
                        "settle.inputs.occurrences",
                        "an item has no occurrence address",
                    )
                })?;
            let name = serde_json::to_vec(&(&definition.id, occurrence)).unwrap();
            let id = Uuid::new_v5(&namespace, &name);
            let at = Point2D::new(item.transform.translate.x, item.transform.translate.y);
            if !at.x.is_finite() || !at.y.is_finite() {
                return Err(refuse(
                    "settle.inputs.positions",
                    "non-finite arrangement position",
                ));
            }
            // Empty addresses disclose no site partition or browsing history.
            let key = add_node(&mut graph, Some(id), String::new(), at);
            keys.push(key);
            ids.push(id);
            positions.push((key, at));
        }
        if let Some(target) = &mut spec.target {
            let mut roles = BTreeMap::new();
            for (source_id, role) in &target.items {
                let mut found = false;
                for (index, item) in compiled.scene.items.iter().enumerate() {
                    let source = &compiled.scene.sources[item.source.0 as usize];
                    if source.adapter == cartography::MERE_GRAPH_ADAPTER && &source.id == source_id
                    {
                        roles.insert(ids[index].to_string(), *role);
                        found = true;
                    }
                }
                if !found {
                    return Err(refuse(
                        "dynamics.spec.target.items",
                        "a graph item role has no disclosed graph-source occurrence",
                    ));
                }
            }
            target.items = roles;
        }
        let mut canvas = Canvas::with_graph(graph);
        canvas.set_physics_paused(true);
        canvas.set_layout_strategy(Some(definition.arrangement.kind.clone()));
        canvas.apply_strategy_positions(&positions);
        canvas.apply_strategy_to_view();
        let mut out = Self {
            canvas,
            spec,
            slot: slot.clone(),
            progress: Progress::default(),
            scene: compiled.scene.clone(),
            keys,
            started: false,
            finished: None,
            ticks: 0,
        };
        out.install_colliders()?;
        let axes = encoded_axes(definition)?;
        if axes != seiche::Axes::NONE
            && axes != seiche::Axes::BOTH
            && super::dynamics_spec::derive(&out.spec)
                .map_err(BindError::Refused)?
                .leaves
                .iter()
                .any(|leaf| {
                    matches!(
                        leaf.currency,
                        seiche::Currency::Kinematic | seiche::Currency::Resident
                    )
                })
        {
            return Err(refuse(
                "settle.inputs.axes",
                "position-writing laws need an encoded-axis constraint adapter",
            ));
        }
        out.set_encoded_axes(axes)?;
        Ok(out)
    }

    fn install_colliders(&mut self) -> Result<(), BindError> {
        let mut colliders = Vec::new();
        for (key, item) in self.keys.iter().zip(&self.scene.items) {
            if item.transform.rotate != 0.0 || item.transform.scale != 1.0 {
                return Err(refuse(
                    "settle.inputs.footprint",
                    "rotated or scaled occurrence bodies require a shape adapter",
                ));
            }
            let rect = item.footprint.bounds().ok_or_else(|| {
                refuse(
                    "settle.inputs.footprint",
                    "an occurrence body requires a measured footprint",
                )
            })?;
            if !rect.size.w.is_finite()
                || !rect.size.h.is_finite()
                || !rect.origin.x.is_finite()
                || !rect.origin.y.is_finite()
                || rect.size.w <= 0.0
                || rect.size.h <= 0.0
            {
                return Err(refuse(
                    "settle.inputs.footprint",
                    "invalid measured footprint",
                ));
            }
            let x = rect.origin.x;
            let y = rect.origin.y;
            let collider = match &item.footprint {
                sceno::Footprint::Circle { radius } => NodeCollider::Ball { radius: *radius },
                sceno::Footprint::Rect { .. } => NodeCollider::Hull {
                    points: vec![
                        (x, y),
                        (x + rect.size.w, y),
                        (x + rect.size.w, y + rect.size.h),
                        (x, y + rect.size.h),
                    ],
                    fallback: rect.size.w.max(rect.size.h) * 0.5,
                },
                _ => {
                    return Err(refuse(
                        "settle.inputs.footprint",
                        "this occurrence adapter supports rectangular or circular bodies",
                    ));
                },
            };
            colliders.push((*key, collider));
        }
        self.canvas.physics.set_node_colliders(colliders);
        Ok(())
    }

    pub fn scene(&self) -> &sceno::Scene {
        &self.scene
    }
    pub fn is_running(&self) -> bool {
        self.finished.is_none()
    }

    pub fn ticks(&self) -> u64 {
        self.ticks
    }

    /// Refresh presentation without resetting the bodies or their trajectory.
    /// The caller must first establish that placement was reused.
    pub fn refresh_display(&mut self, slot: &DynamicsSlot, scene: &sceno::Scene) -> bool {
        if slot != &self.slot
            || scene.sources != self.scene.sources
            || scene.items.len() != self.scene.items.len()
            || scene.items.iter().zip(&self.scene.items).any(|(a, b)| {
                a.source != b.source
                    || a.space != b.space
                    || a.footprint != b.footprint
                    || a.transform.scale != b.transform.scale
                    || a.transform.rotate != b.transform.rotate
            })
        {
            return false;
        }
        self.scene = scene.clone();
        self.refresh_scene();
        true
    }

    fn validate_positions(&self) -> Result<(), BindError> {
        if self
            .canvas
            .view
            .positions()
            .any(|(_, p)| !p.x.is_finite() || !p.y.is_finite())
        {
            Err(refuse(
                "settle.positions",
                "the law produced a non-finite position",
            ))
        } else {
            Ok(())
        }
    }

    /// Declare direct translation encodings before the run starts. A named
    /// recipe's rank/order is not enough to infer a physical axis constraint.
    pub fn set_encoded_axes(&mut self, axes: seiche::Axes) -> Result<(), BindError> {
        if self.started {
            return Err(refuse(
                "settle.inputs.axes",
                "declare encoded axes before starting a preview",
            ));
        }
        self.canvas.roles.encoded = axes;
        self.canvas.sync_arrangement_roles();
        self.canvas.rebuild_law_forces();
        Ok(())
    }

    /// Continue the focused cell by one fixed tick. No other cell needs a
    /// controller. A capped snapshot can continue from precisely that state.
    pub fn tick(&mut self) -> Result<bool, BindError> {
        if !self.started {
            self.progress.start(&mut self.canvas, &self.spec)?;
            self.started = true;
        }
        if self.finished.is_some() {
            return Ok(false);
        }
        self.finished = self.progress.step(&mut self.canvas);
        self.ticks += 1;
        self.validate_positions()?;
        self.refresh_scene();
        Ok(self.finished.is_none())
    }

    /// One bounded frame for the non-focused grid cells. The same controller
    /// supplies the live trajectory; reaching the bound does not halt it.
    pub fn snapshot(&mut self, bound: u32) -> Result<SettleReport, SettleError<BindError>> {
        let spec = self.spec.clone();
        let report = seiche::settle(&spec, self, bound)?;
        self.refresh_scene();
        Ok(report)
    }

    fn refresh_scene(&mut self) {
        let mut bounds: Option<sceno::Rect> = None;
        for (key, item) in self.keys.iter().zip(&mut self.scene.items) {
            if let Some(at) = self.canvas.view.position_of(*key) {
                item.transform.translate = sceno::Vec2::new(at.x, at.y);
            }
            if let Some(mut rect) = item.footprint.bounds() {
                rect.origin.x += item.transform.translate.x;
                rect.origin.y += item.transform.translate.y;
                bounds = Some(bounds.map_or(rect, |b| b.union(rect)));
            }
        }
        self.scene.bounds = bounds.unwrap_or_default();
    }
}

/// F200: classify what placement means, rather than locking every Field.
/// Grid cells, ring indices, layers, buckets and order are layout slots.
fn encoded_axes(definition: &ProjectionDefinition) -> Result<seiche::Axes, BindError> {
    use scenograph::Channel;
    use scenomise::catalog::Family;
    let field = |channel: &Channel| matches!(channel, Channel::Field(_));
    let axes = match Family::resolve(&definition.arrangement.kind) {
        Some(Family::Geographic | Family::Hulls | Family::Embedded) => seiche::Axes {
            x: field(&definition.encoding.x),
            y: field(&definition.encoding.y),
        },
        Some(Family::Timeline) => seiche::Axes {
            x: field(&definition.encoding.x),
            y: false,
        },
        Some(_) => seiche::Axes::NONE,
        None => {
            return Err(refuse(
                "settle.inputs.axes",
                "this arrangement needs a disclosed position-constraint adapter",
            ));
        },
    };
    Ok(axes)
}

impl DynamicsRunner for ProjectionDynamics {
    type Error = BindError;
    fn prepare(&mut self, spec: &DynamicsSpec) -> Result<(), BindError> {
        if spec != &self.spec {
            return Err(refuse("dynamics.spec", "a preview is bound to one recipe"));
        }
        if !self.started {
            self.progress.start(&mut self.canvas, spec)?;
            self.started = true;
        }
        Ok(())
    }
    fn step_fixed(&mut self) -> Result<Option<SettleEnd>, BindError> {
        if self.finished.is_none() {
            self.finished = self.progress.step(&mut self.canvas);
            self.ticks += 1;
        }
        self.validate_positions()?;
        Ok(self.finished)
    }
    fn positions(&self) -> Vec<(NodeKey, Point2D<f32>)> {
        self.canvas.view.positions().collect()
    }
}
