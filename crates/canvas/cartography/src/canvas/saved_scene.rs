// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The saved scene as the canvas captures and restores it (dynamics grammar
//! plan, F142, F151). The stored shapes and facet ids are pandect's
//! (`pandect::saved_scene`); this module reads and writes them against the
//! canvas's vocabularies, so Graphshell and Turnstone share one capture, one
//! restore, one validation and one version 2/3 conversion.

use std::collections::{BTreeMap, HashMap, HashSet};

use kernel::geometry::PortablePoint;
use kernel::graph::Graph;
use pandect::{
    CartographyGeometry, DEFAULT_ARRANGEMENT, SAVED_SCENE_FACET, SAVED_SCENE_FACET_V2,
    SAVED_SCENE_FACET_V3, SavedDynamics, SavedRolesV1, SavedSceneV2, SavedSceneV3, StoredScene,
};
use uuid::Uuid;

use super::dynamics_spec::{self, DynamicsSpec, GroupRoles, Realization, Target};
use super::physics_controls::FREE_ARRANGEMENT;
use super::{
    CameraView, Canvas, PhysicsChoice, PhysicsDepthSource, PhysicsKindSource, PhysicsLaw,
    PhysicsMassSource, PhysicsOverlay, Role, RoleTable,
};

/// A spec as a scene carries it.
pub fn saved_dynamics(spec: &DynamicsSpec) -> SavedDynamics {
    SavedDynamics::from_value(
        serde_json::to_value(spec).expect("a dynamics spec always serializes"),
    )
}

/// The spec a scene carries, read through seiche's path-tracking reader: a
/// refusal names where it sits (F113).
pub fn read_dynamics(dynamics: &SavedDynamics) -> Result<DynamicsSpec, String> {
    dynamics_spec::read(dynamics.value()).map_err(|error| format!("dynamics spec: {error}"))
}

/// The scene's spec, read, derived and bound over the canvas's catalog
/// (graph-free); a refusal names the id and where it sits (F97, F113, F149).
pub fn scene_spec(scene: &SavedSceneV3) -> Result<DynamicsSpec, String> {
    let spec = read_dynamics(&scene.dynamics)?;
    dynamics_spec::bind(&spec).map_err(|error| format!("dynamics spec: {error}"))?;
    Ok(spec)
}

/// The arrangement the scene's target names, or [`DEFAULT_ARRANGEMENT`].
pub fn scene_arrangement(scene: &SavedSceneV3) -> Result<String, String> {
    Ok(scene_spec(scene)?
        .target
        .map(|target| target.arrangement)
        .filter(|id| !id.is_empty())
        .unwrap_or_else(|| DEFAULT_ARRANGEMENT.to_string()))
}

/// What a scene read from its facet can fail on.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SceneReadError {
    /// The stored value is not a scene of its version.
    Malformed(String),
    /// A scene whose spec is refused, or whose conversion meets an unknown id,
    /// does not open (F97, F143, F154).
    Refused(String),
}

/// The scene a facet holds: version 4 as it is, an older one converted
/// (F142), and its spec checked either way.
pub fn read_scene(stored: StoredScene<'_>) -> Result<SavedSceneV3, SceneReadError> {
    let scene = match stored {
        StoredScene::V4(value) => serde_json::from_value::<SavedSceneV3>(value.clone())
            .map_err(|error| SceneReadError::Malformed(error.to_string()))?,
        StoredScene::Older(value) => scene_from_v2(
            serde_json::from_value::<SavedSceneV2>(value.clone())
                .map_err(|error| SceneReadError::Malformed(error.to_string()))?,
        )
        .map_err(SceneReadError::Refused)?,
    };
    scene_spec(&scene).map_err(SceneReadError::Refused)?;
    Ok(scene)
}

/// [`SavedRolesV1`] parsed: items by member, since node keys are a canvas's
/// own.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SavedRoles {
    pub default: Role,
    pub groups: BTreeMap<String, Role>,
    pub items: BTreeMap<Uuid, Role>,
}

impl SavedRoles {
    pub fn uniform(role: Role) -> Self {
        Self {
            default: role,
            groups: BTreeMap::new(),
            items: BTreeMap::new(),
        }
    }

    /// The canvas's table, items rebound from members to `graph`'s keys; a
    /// member the graph lacks is dropped.
    pub fn table(&self, graph: &Graph) -> RoleTable {
        RoleTable {
            default: self.default,
            groups: self.groups.clone(),
            items: self
                .items
                .iter()
                .filter_map(|(member, role)| Some((graph.get_node_key_by_id(*member)?, *role)))
                .collect(),
        }
    }

    /// A canvas's table, items by member.
    pub fn from_table(table: &RoleTable, graph: &Graph) -> Self {
        Self {
            default: table.default,
            groups: table.groups.clone(),
            items: table
                .items
                .iter()
                .filter_map(|(key, role)| Some((graph.get_node(*key)?.id, *role)))
                .collect(),
        }
    }

    /// The saved form, by role id.
    pub fn saved(&self) -> SavedRolesV1 {
        SavedRolesV1 {
            default: self.default.id().to_string(),
            groups: self
                .groups
                .iter()
                .map(|(group, role)| (group.clone(), role.id().to_string()))
                .collect(),
            items: self
                .items
                .iter()
                .map(|(member, role)| (*member, role.id().to_string()))
                .collect(),
        }
    }
}

/// The roles a version-2 scene opens with, and the anchored stiffness. A
/// scene saved before G7 has only its pull: the canvas anchored every item at
/// that pull while playing, so a positive pull reads as anchored at it and
/// zero as seeded, and the scene behaves as it did. An unknown role id fails
/// rather than falling back.
pub fn v2_roles(scene: &SavedSceneV2) -> Result<(SavedRoles, f32), String> {
    let parse = |id: &str| Role::parse(id).ok_or_else(|| format!("unknown arrangement role {id}"));
    let Some(saved) = &scene.arrangement_roles else {
        return Ok(if scene.arrangement_pull > 0.0 {
            (SavedRoles::uniform(Role::Anchored), scene.arrangement_pull)
        } else {
            (
                SavedRoles::uniform(Role::Seeded),
                super::DEFAULT_ANCHOR_STIFFNESS,
            )
        });
    };
    let mut roles = SavedRoles::uniform(parse(&saved.default)?);
    for (group, id) in &saved.groups {
        roles.groups.insert(group.clone(), parse(id)?);
    }
    for (member, id) in &saved.items {
        roles.items.insert(*member, parse(id)?);
    }
    Ok((roles, scene.arrangement_pull))
}

/// A version-2 or version-3 scene as a version-4 scene (F142). A spec it
/// carries wins, and the flat fields fill only what it lacks: the target when
/// it has none, the damping when its realization leaves it unset (F153).
/// Without a spec the flat fields become one, an unknown law, overlay or
/// source id failing with the id (F143, F154), and an unknown role id as
/// before.
pub fn scene_from_v2(scene: SavedSceneV2) -> Result<SavedSceneV3, String> {
    let mut spec = match &scene.dynamics {
        Some(dynamics) => read_dynamics(dynamics)?,
        None => v2_choice(&scene)?.into_spec(),
    };
    if spec.target.is_none() {
        spec.target = Some(v2_target(&scene)?);
    }
    let Realization::Integrate { damping } = &mut spec.realization;
    if damping.is_none() {
        *damping = Some(f64::from(scene.physics_damping));
    }
    Ok(SavedSceneV3 {
        name: scene.name,
        selected: scene.selected,
        physics_paused: scene.physics_paused,
        dynamics: saved_dynamics(&spec),
        camera_offset: scene.camera_offset,
        camera_zoom: scene.camera_zoom,
        default_handler: scene.default_handler,
        cartography: scene.cartography,
    })
}

/// The flat physics fields as a choice; an unknown id fails with it.
fn v2_choice(scene: &SavedSceneV2) -> Result<PhysicsChoice, String> {
    let kind = |id: &str, what: &str| {
        PhysicsKindSource::parse(id).ok_or_else(|| format!("unknown {what} source {id}"))
    };
    let mut overlays: Vec<PhysicsOverlay> = Vec::new();
    for id in &scene.physics_overlays {
        let overlay =
            PhysicsOverlay::parse(id).ok_or_else(|| format!("unknown physics overlay {id}"))?;
        if !overlays.contains(&overlay) {
            overlays.push(overlay);
        }
    }
    Ok(PhysicsChoice {
        law: PhysicsLaw::parse(&scene.physics_law)
            .ok_or_else(|| format!("unknown physics law {}", scene.physics_law))?,
        overlays,
        kind: kind(&scene.physics_kind_source, "kind")?,
        groups: kind(&scene.physics_group_source, "group")?,
        mass: PhysicsMassSource::parse(&scene.physics_mass_source)
            .ok_or_else(|| format!("unknown mass source {}", scene.physics_mass_source))?,
        depth: PhysicsDepthSource::parse(&scene.physics_depth_source)
            .ok_or_else(|| format!("unknown depth source {}", scene.physics_depth_source))?,
    })
}

/// The flat arrangement fields as the spec's target: the layout, the anchored
/// stiffness and the roles, read as [`v2_roles`] reads them; group roles keyed
/// by site, the only group source before G4b1.
fn v2_target(scene: &SavedSceneV2) -> Result<Target, String> {
    let (roles, stiffness) = v2_roles(scene)?;
    Ok(Target {
        arrangement: scene
            .layout_strategy
            .clone()
            .unwrap_or_else(|| DEFAULT_ARRANGEMENT.to_string()),
        anchored_pull: f64::from(stiffness),
        default_role: roles.default,
        groups: (!roles.groups.is_empty()).then(|| GroupRoles {
            channel: "groups.site".to_string(),
            roles: roles.groups,
        }),
        items: roles
            .items
            .into_iter()
            .map(|(member, role)| (member.to_string(), role))
            .collect(),
    })
}

// Scenes under another set of ids: a codicil's export and import, a fork's
// carry (F159). A member the copy leaves out is dropped.

/// A spec's target item roles under a copy's new ids, as the scene's own
/// item roles are: an item the copy leaves out is dropped (dynamics grammar
/// plan, G4a; F105's item roles are keyed by node UUID).
/// A spec this reader refuses rides on unchanged, to be refused where the
/// scene opens (F114).
fn remap_dynamics(
    dynamics: &SavedDynamics,
    remap: &impl Fn(Uuid) -> Option<Uuid>,
) -> SavedDynamics {
    match read_dynamics(dynamics) {
        Ok(spec) => saved_dynamics(&remap_spec(&spec, remap)),
        Err(_) => dynamics.clone(),
    }
}

pub fn remap_spec(spec: &DynamicsSpec, remap: &impl Fn(Uuid) -> Option<Uuid>) -> DynamicsSpec {
    let mut spec = spec.clone();
    if let Some(target) = spec.target.as_mut() {
        target.items = std::mem::take(&mut target.items)
            .into_iter()
            .filter_map(|(id, role)| Some((remap(Uuid::parse_str(&id).ok()?)?.to_string(), role)))
            .collect();
    }
    spec
}

/// A scene's representations under a copy's new ids; a member the copy
/// leaves out is dropped.
fn remap_cartography(
    cartography: &CartographyGeometry,
    remap: &impl Fn(Uuid) -> Option<Uuid>,
) -> CartographyGeometry {
    CartographyGeometry::from_positions(
        cartography
            .iter()
            .filter_map(|(id, position)| remap(id).map(|new_id| (new_id, position))),
    )
    .with_sizes(
        cartography
            .size_iter()
            .filter_map(|(id, size)| remap(id).map(|new_id| (new_id, size))),
    )
    .with_size_by_degree(cartography.size_by_degree())
    .with_size_by_importance(cartography.size_by_importance())
    .with_importance_metric(cartography.importance_metric())
    .with_sprites(
        cartography
            .sprite_iter()
            .filter_map(|(id, uri)| remap(id).map(|new_id| (new_id, uri.to_string()))),
    )
    .with_sprite_hulls(
        cartography
            .sprite_hull_iter()
            .filter_map(|(id, hull)| remap(id).map(|new_id| (new_id, hull))),
    )
    .with_materials(
        cartography
            .material_iter()
            .filter_map(|(id, material)| remap(id).map(|new_id| (new_id, material))),
    )
    .with_faces(
        cartography
            .face_iter()
            .filter_map(|(id, face)| remap(id).map(|new_id| (new_id, face.to_string()))),
    )
}

/// A version-4 scene under a copy's new ids: the selection, the
/// representations and the spec's target items (F159).
pub fn remap_scene(scene: &SavedSceneV3, ids: &HashMap<Uuid, Uuid>) -> SavedSceneV3 {
    let remap = |id: Uuid| ids.get(&id).copied();
    SavedSceneV3 {
        name: scene.name.clone(),
        selected: scene.selected.iter().filter_map(|id| remap(*id)).collect(),
        physics_paused: scene.physics_paused,
        dynamics: remap_dynamics(&scene.dynamics, &remap),
        camera_offset: scene.camera_offset,
        camera_zoom: scene.camera_zoom,
        default_handler: scene.default_handler.clone(),
        cartography: remap_cartography(&scene.cartography, &remap),
    }
}

/// A version-3 or version-2 scene facet a copied node carries, under the
/// copy's new ids, kept in its own version (F159).
fn remap_scene_v2(scene: &SavedSceneV2, ids: &HashMap<Uuid, Uuid>) -> SavedSceneV2 {
    let remap = |id: Uuid| ids.get(&id).copied();
    SavedSceneV2 {
        selected: scene.selected.iter().filter_map(|id| remap(*id)).collect(),
        arrangement_roles: scene.arrangement_roles.as_ref().map(|roles| SavedRolesV1 {
            items: roles
                .items
                .iter()
                .filter_map(|(id, role)| Some((remap(*id)?, role.clone())))
                .collect(),
            ..roles.clone()
        }),
        dynamics: scene
            .dynamics
            .as_ref()
            .map(|dynamics| remap_dynamics(dynamics, &remap)),
        cartography: remap_cartography(&scene.cartography, &remap),
        ..scene.clone()
    }
}

/// A copied node's facet: a saved scene under the copy's new ids, anything
/// else as it was. A scene this reader cannot read is copied unchanged.
pub fn remap_scene_facet(
    facet: &str,
    value: &serde_json::Value,
    ids: &HashMap<Uuid, Uuid>,
) -> serde_json::Value {
    let remapped = if facet == SAVED_SCENE_FACET {
        serde_json::from_value::<SavedSceneV3>(value.clone())
            .ok()
            .and_then(|scene| serde_json::to_value(remap_scene(&scene, ids)).ok())
    } else if facet == SAVED_SCENE_FACET_V3 || facet == SAVED_SCENE_FACET_V2 {
        serde_json::from_value::<SavedSceneV2>(value.clone())
            .ok()
            .and_then(|scene| serde_json::to_value(remap_scene_v2(&scene, ids)).ok())
    } else {
        None
    };
    remapped.unwrap_or_else(|| value.clone())
}

/// The scene an export carries: its selection, representations and target
/// items narrowed to the exported members (F159). A spec this reader
/// refuses rides on unchanged, to be refused where the scene opens (F114).
pub fn filter_scene(mut scene: SavedSceneV3, members: &HashSet<Uuid>) -> SavedSceneV3 {
    scene.selected.retain(|id| members.contains(id));
    if let Ok(mut spec) = read_dynamics(&scene.dynamics) {
        if let Some(target) = spec.target.as_mut() {
            target
                .items
                .retain(|id, _| Uuid::parse_str(id).is_ok_and(|member| members.contains(&member)));
        }
        scene.dynamics = saved_dynamics(&spec);
    }
    scene.cartography.retain_members(|id| members.contains(id));
    scene
}

/// Graph-free rejection before a scene changes anything: its spec reads and
/// binds, and its camera and positions are finite.
pub fn validate_scene(scene: &SavedSceneV3) -> Result<(), String> {
    prepare_scene(scene).map(|_| ())
}

struct PreparedScene {
    spec: DynamicsSpec,
    arrangement: String,
}

fn prepare_scene(scene: &SavedSceneV3) -> Result<PreparedScene, String> {
    let spec = scene_spec(scene)?;
    let arrangement = scene_arrangement(scene)?;
    if !scene.camera_offset.0.is_finite()
        || !scene.camera_offset.1.is_finite()
        || !scene.camera_zoom.is_finite()
        || scene.camera_zoom <= 0.0
    {
        return Err("saved scene has an invalid camera".into());
    }
    if scene
        .cartography
        .iter()
        .any(|(_, (x, y))| !x.is_finite() || !y.is_finite())
    {
        return Err("saved scene has an invalid position".into());
    }
    Ok(PreparedScene { spec, arrangement })
}

impl Canvas {
    /// The canvas as a scene: its dynamics spec (its target naming
    /// `arrangement`), the selection less derived members, whether physics is
    /// paused, the camera and the cartography geometry.
    pub fn capture_scene(
        &self,
        name: &str,
        arrangement: &str,
        handler: &str,
    ) -> Result<SavedSceneV3, String> {
        let mut spec = self.dynamics_spec()?;
        if let Some(target) = spec.target.as_mut() {
            target.arrangement = arrangement.to_owned();
        }
        let camera = self.camera();
        Ok(SavedSceneV3 {
            name: name.to_owned(),
            selected: self
                .selected_members()
                .into_iter()
                .filter(|member| !self.is_derived_member(*member))
                .collect(),
            physics_paused: self.physics_paused(),
            dynamics: saved_dynamics(&spec),
            camera_offset: camera.offset,
            camera_zoom: camera.zoom,
            default_handler: handler.to_owned(),
            cartography: self.cartography_geometry(),
        })
    }

    /// Validate, then replace the canvas's graph with `owner` and dress it
    /// with `scene`. Members the scene names that the graph lacks are skipped
    /// by the member-keyed restoration APIs.
    pub fn restore_scene(
        &mut self,
        owner: &Graph,
        scene: &SavedSceneV3,
        viewport: (u32, u32),
    ) -> Result<(), String> {
        let prepared = prepare_scene(scene)?;
        self.set_graph(owner.clone());
        self.dress_scene(scene, prepared, Some(viewport))
    }

    /// [`Self::restore_scene`] on the graph the canvas already holds (a
    /// session switch having set it). With a `viewport` the arrangement is
    /// recorded as computed for it, so the saved positions stand; without
    /// one the host records that itself, on its first frame.
    pub fn apply_scene(
        &mut self,
        scene: &SavedSceneV3,
        viewport: Option<(u32, u32)>,
    ) -> Result<(), String> {
        let prepared = prepare_scene(scene)?;
        self.dress_scene(scene, prepared, viewport)
    }

    fn dress_scene(
        &mut self,
        scene: &SavedSceneV3,
        PreparedScene { spec, arrangement }: PreparedScene,
        viewport: Option<(u32, u32)>,
    ) -> Result<(), String> {
        self.set_layout_strategy((arrangement != FREE_ARRANGEMENT).then(|| arrangement.clone()));
        let positions: Vec<_> = scene
            .cartography
            .iter()
            .filter_map(|(id, (x, y))| {
                self.graph()
                    .get_node_key_by_id(id)
                    .map(|key| (key, PortablePoint::new(x, y)))
            })
            .collect();
        self.apply_strategy_positions(&positions);
        if let Some((width, height)) = viewport {
            self.note_strategy_computed(&arrangement, width, height, self.focused_key());
        }
        self.apply_cartography_importance_metric(scene.cartography.importance_metric());
        self.apply_cartography_sizing(
            scene.cartography.size_iter(),
            scene.cartography.size_by_degree(),
            scene.cartography.size_by_importance(),
        );
        self.apply_cartography_sprites(scene.cartography.sprite_iter());
        self.apply_cartography_sprite_hulls(scene.cartography.sprite_hull_iter());
        self.apply_cartography_materials(scene.cartography.material_iter());
        self.apply_cartography_faces(scene.cartography.face_iter());
        self.apply_cartography_scene_state(&scene.cartography);
        // Binding was checked above, before the graph changed.
        self.set_dynamics_spec(&spec)
            .map_err(|error| format!("dynamics spec: {error}"))?;
        self.seed_cartography(scene.cartography.iter());
        self.set_physics_paused(scene.physics_paused);
        self.set_selected_members(&scene.selected);
        self.set_camera(CameraView {
            offset: scene.camera_offset,
            zoom: scene.camera_zoom,
        });
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn v2_defaults_are_the_catalogs() {
        let scene: SavedSceneV2 = serde_json::from_value(serde_json::json!({
            "name": "old", "selected": [], "layout_strategy": null,
            "physics_paused": false, "physics_damping": 2.0,
            "camera_offset": [0.0, 0.0], "camera_zoom": 1.0,
            "default_handler": "", "cartography": {"positions": []}
        }))
        .unwrap();
        assert_eq!(scene.physics_law, PhysicsLaw::Springs.id());
        assert_eq!(scene.physics_kind_source, PhysicsKindSource::Site.id());
        assert_eq!(scene.physics_group_source, PhysicsKindSource::Site.id());
        assert_eq!(scene.physics_mass_source, PhysicsMassSource::Degree.id());
        assert_eq!(scene.physics_depth_source, PhysicsDepthSource::Roots.id());
        assert_eq!(
            scene.arrangement_pull,
            super::super::DEFAULT_ANCHOR_STIFFNESS
        );
    }
}
