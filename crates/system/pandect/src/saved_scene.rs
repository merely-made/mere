// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The saved scene both hosts persist (dynamics grammar plan, F142, F151):
//! the stored shapes and facet ids. A scene is a node plus a facet (Scenograph
//! editor plan, SE14); Graphshell saves it on a `mere://scene/<name>` node and
//! Turnstone on its graph's container node, so either host can open the
//! other's. Capture, restore and the version 2/3 conversion live with the
//! canvas that knows the vocabularies (`pictograph::canvas::saved_scene`);
//! this module holds only what is stored. The dynamics spec rides opaque
//! (F114), so a newer spec passes through a host that cannot read it (F158).
//!
//! Strict own versioning (F98): a version-4 scene is read as version 4 only.
//! The older facets are read, never written.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::cartography_geometry::CartographyGeometry;
use crate::facet_store::{AcceptAll, FacetId, NodeFacetStore};

/// Where a scene is saved: [`SavedSceneV3`], the dynamics spec its only
/// physics and arrangement record (dynamics grammar plan, F142, F151).
pub const SAVED_SCENE_FACET: &str = "graphshell.saved-scene/v4";
/// Where a scene was saved with flat physics fields and roles (G7, F44).
/// Read only, as [`SavedSceneV2`], and converted (F142).
pub const SAVED_SCENE_FACET_V3: &str = "graphshell.saved-scene/v3";
/// Where a scene was saved before the arrangement roles. Read only: its
/// anchor pull is read as the roles it acted as, and converted as a
/// version-3 scene is.
pub const SAVED_SCENE_FACET_V2: &str = "graphshell.saved-scene/v2";
/// The arrangement a scene that names none opens on.
pub const DEFAULT_ARRANGEMENT: &str = "phyllotaxis.default";

/// A saved scene as written under [`SAVED_SCENE_FACET_V3`] and
/// [`SAVED_SCENE_FACET_V2`]: read only since G4b1, and converted to a
/// [`SavedSceneV3`] (F142). Version 2 (dynamics grammar plan, G7, F44) added
/// the arrangement's roles, and `arrangement_pull` became the anchored role's
/// stiffness; a version-1 scene, saved under [`SAVED_SCENE_FACET_V2`], reads
/// into this type with no roles and is read as it behaved.
///
/// The defaults are the catalog's ids as they stood (`pictograph`'s
/// `saved_scene` tests hold them to the catalog).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SavedSceneV2 {
    pub name: String,
    pub selected: Vec<Uuid>,
    pub layout_strategy: Option<String>,
    pub physics_paused: bool,
    pub physics_damping: f32,
    /// The physics law id; Springs when absent, so a scene saved before the
    /// catalog reads as it always did.
    #[serde(default = "default_physics_law")]
    pub physics_law: String,
    /// The overlay ids composed onto the law, in run order.
    #[serde(default)]
    pub physics_overlays: Vec<String>,
    /// Where the Kinds law reads a node's kind from (`site`, `cluster`, `degree`).
    #[serde(default = "default_physics_kind_source")]
    pub physics_kind_source: String,
    /// Where Group pull reads its groups from (any kind source); site when
    /// absent, so a scene saved before the channel reads as it did. (Dynamics
    /// grammar plan, G2, F37.)
    #[serde(default = "default_physics_group_source")]
    pub physics_group_source: String,
    /// Where Orbit's masses and the hub overlays' weights come from (`degree`, `pagerank`).
    #[serde(default = "default_physics_mass_source")]
    pub physics_mass_source: String,
    /// Where the Depth overlay reads depth from (`roots`, `layers`, `focus`).
    #[serde(default = "default_physics_depth_source")]
    pub physics_depth_source: String,
    /// The anchored role's return stiffness. A scene saved before G7 stored
    /// its anchor pull here, and with no `arrangement_roles` that pull is read
    /// as the roles it acted as.
    #[serde(default = "default_anchor_stiffness")]
    pub arrangement_pull: f32,
    /// The arrangement's roles: recipe default, site groups, items by member
    /// (dynamics grammar plan, G7). Absent in a scene saved before G7.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub arrangement_roles: Option<SavedRolesV1>,
    /// The scene's dynamics spec (dynamics grammar plan, G4a): absent from
    /// every scene saved before it and written by nothing before G4b1 (F100).
    /// It rides opaque and is read when the scene opens, where a refused spec
    /// fails that open alone (F97, F114); converted, it wins over the flat
    /// fields, which fill only what it lacks (F153).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dynamics: Option<SavedDynamics>,
    pub camera_offset: (f32, f32),
    pub camera_zoom: f32,
    pub default_handler: String,
    pub cartography: CartographyGeometry,
}

/// An arrangement's roles as a scene saves them, by role id
/// (`seeded`, `anchored`, `pinned`).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SavedRolesV1 {
    pub default: String,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub groups: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub items: BTreeMap<Uuid, String>,
}

/// A scene's dynamics spec as it rides: an opaque value, so whatever carries
/// the scene (the facet, a codicil, a personal graph record) reads it whole
/// and the spec is read only when the scene opens (F114). The canvas reads it
/// through seiche's path-tracking reader, so a refusal names where it sits
/// (F113).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SavedDynamics(Value);

impl SavedDynamics {
    pub fn from_value(value: Value) -> Self {
        Self(value)
    }

    /// The spec as written, unread.
    pub fn value(&self) -> &Value {
        &self.0
    }
}

/// A saved scene, version 4 of the facet ([`SAVED_SCENE_FACET`]; dynamics
/// grammar plan, F142, F151): the dynamics spec is its only physics and
/// arrangement record, the law, overlays, sources, seed, damping, the
/// arrangement and its roles all inside it. What stays beside it is view
/// state: the selection, the camera, whether physics plays (F152), the
/// handler and the representations.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SavedSceneV3 {
    pub name: String,
    pub selected: Vec<Uuid>,
    /// Whether physics is paused when the scene opens: view state, as the
    /// camera is (F152).
    pub physics_paused: bool,
    /// The record (F142), opaque until the scene opens (F114).
    pub dynamics: SavedDynamics,
    pub camera_offset: (f32, f32),
    pub camera_zoom: f32,
    pub default_handler: String,
    pub cartography: CartographyGeometry,
}

fn default_anchor_stiffness() -> f32 {
    12.0
}

fn default_physics_law() -> String {
    "spring.rapier".into()
}

fn default_physics_kind_source() -> String {
    "site".into()
}

fn default_physics_group_source() -> String {
    "site".into()
}

fn default_physics_mass_source() -> String {
    "degree".into()
}

fn default_physics_depth_source() -> String {
    "roots".into()
}

/// The scene saved on `node` in a facet store, as its stored value.
pub fn stored_scene(store: &NodeFacetStore, node: Uuid) -> Option<StoredScene<'_>> {
    let facets = store.facets_of(&node)?;
    StoredScene::find(|id| facets.get(&FacetId::new(id)))
}

/// What a node's scene facets hold: a version-4 value, or an older one that
/// reads as a [`SavedSceneV2`], for the caller to convert.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum StoredScene<'a> {
    V4(&'a Value),
    Older(&'a Value),
}

impl<'a> StoredScene<'a> {
    /// The scene in the facets `get` looks up (a graph's node facets, a
    /// facet store's): version 4 first, else version 3, else version 2.
    pub fn find(get: impl Fn(&str) -> Option<&'a Value>) -> Option<Self> {
        if let Some(value) = get(SAVED_SCENE_FACET) {
            return Some(Self::V4(value));
        }
        get(SAVED_SCENE_FACET_V3)
            .or_else(|| get(SAVED_SCENE_FACET_V2))
            .map(Self::Older)
    }
}

/// Save `scene` on `node` under [`SAVED_SCENE_FACET`] (F142: every save
/// writes version 4), clearing the older facets the node may carry so one
/// scene is never held twice.
pub fn write_saved_scene(store: &mut NodeFacetStore, node: Uuid, scene: &SavedSceneV3) {
    for older in [SAVED_SCENE_FACET_V3, SAVED_SCENE_FACET_V2] {
        store.remove(&node, &FacetId::new(older));
    }
    let value = serde_json::to_value(scene).expect("a saved scene always serializes");
    let _ = store.set(node, FacetId::new(SAVED_SCENE_FACET), value, &AcceptAll);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scene() -> SavedSceneV3 {
        SavedSceneV3 {
            name: "kept".into(),
            selected: vec![Uuid::from_u128(1)],
            physics_paused: true,
            dynamics: SavedDynamics::from_value(serde_json::json!({"k": 1})),
            camera_offset: (3.0, 4.0),
            camera_zoom: 2.0,
            default_handler: "web".into(),
            cartography: CartographyGeometry::from_positions([(Uuid::from_u128(1), (1.0, 2.0))]),
        }
    }

    #[test]
    fn a_scene_round_trips_through_the_facet_and_replaces_older_ones() {
        let mut store = NodeFacetStore::default();
        let node = Uuid::from_u128(9);
        let _ = store.set(
            node,
            FacetId::new(SAVED_SCENE_FACET_V3),
            serde_json::json!({}),
            &AcceptAll,
        );
        assert!(matches!(
            stored_scene(&store, node),
            Some(StoredScene::Older(_))
        ));
        write_saved_scene(&mut store, node, &scene());
        let Some(StoredScene::V4(value)) = stored_scene(&store, node) else {
            panic!("a version-4 scene is stored");
        };
        assert_eq!(
            serde_json::from_value::<SavedSceneV3>(value.clone()).unwrap(),
            scene()
        );
        assert!(
            store
                .facets_of(&node)
                .unwrap()
                .get(&FacetId::new(SAVED_SCENE_FACET_V3))
                .is_none()
        );
    }

    #[test]
    fn dynamics_ride_as_the_value_they_were_written() {
        let value = serde_json::json!({"future": {"nested": [1, 2]}});
        let json = serde_json::to_value(SavedDynamics::from_value(value.clone())).unwrap();
        assert_eq!(json, value);
    }
}
