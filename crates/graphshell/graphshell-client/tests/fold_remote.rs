// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The remote reader honours the fold fact (site canvas plan, S5): a mounted
//! scene's folded members leave the reader's items, the stand-in carries its
//! "+N", and unfolding arrives as an endpoint diff, the way every remote edit
//! does, and round-trips back to the unfolded tree.

use chirograph::{
    BoundsRelationship, CachePolicy, CacheRetention, CapabilityProfile, ContentHash,
    PresentationBinding, PresentationCapability, PresentationCodec, PresentationKey,
    PresentationManifest, PresentationOffer, PresentationSemantics, ProjectionDiff,
    ProjectionSession, ProjectionSnapshot, ProtocolVersion, SemanticRole,
};
use graphshell_client::{ClientDiffError, ClientState, DiffApplication, read_folds};
use sceno::{
    Fold, FoldDirection, FoldRule, Footprint, InstanceId, ProjectedItem, Representation, Scene,
    SourceRef, StandIn, Transform2, Vec2,
};
use scenotime::{FoldId, Revision, SceneDiff, SceneEpoch, SceneOp, SceneSnapshot};

const NAMES: [&str; 4] = ["Mere", "genet", "netrender", "retinue"];

fn dependencies_of_mere() -> Fold {
    Fold {
        members: (0..4).map(InstanceId).collect(),
        stand_in: StandIn::Member(InstanceId(0)),
        rule: Some(FoldRule::Descendants {
            root: InstanceId(0),
            family: "depends_on".into(),
            direction: FoldDirection::Outgoing,
        }),
        boundary: None,
    }
}

fn mounted(session: &ProjectionSession, fold: Option<Fold>) -> ProjectionSnapshot {
    let mut scene = Scene::new();
    let mut presentation = PresentationManifest::default();
    for (index, name) in NAMES.into_iter().enumerate() {
        let source = scene.intern_source(SourceRef::new("fixture", name.to_lowercase()));
        scene.items.push(ProjectedItem {
            source,
            space: Scene::WORLD,
            transform: Transform2::translation(index as f32 * 30.0, 10.0),
            footprint: Footprint::Circle { radius: 8.0 },
            representation: Representation::Glyph,
            layer: 0,
            visible: true,
            hit: None,
            channels: Vec::new(),
        });
        let key = PresentationKey(format!("item:{index}"));
        presentation.bindings.push(PresentationBinding {
            instance: InstanceId(index as u32),
            key: key.clone(),
        });
        presentation.offers.insert(
            key,
            vec![PresentationOffer {
                codec: PresentationCodec::NativeGlyphV1,
                resource: ContentHash::of(name.as_bytes()),
                byte_size: name.len() as u64,
                requires: PresentationCapability::NativeGlyph,
                semantics: PresentationSemantics {
                    label: name.into(),
                    role: SemanticRole::Graphic,
                    bounds: BoundsRelationship::FillFootprint,
                    actions: Vec::new(),
                },
            }],
        );
    }
    scene.folds.extend(fold);
    ProjectionSnapshot {
        version: ProtocolVersion::V1,
        session: session.clone(),
        scene: SceneSnapshot::from_dense(SceneEpoch(1), Revision(1), scene).unwrap(),
        presentation,
        cache_policy: CachePolicy {
            retention: CacheRetention::MemoryOnly,
            expires_at_ms: None,
            purge_on_revocation: true,
        },
    }
}

/// Everything crosses the wire as JSON, as it would from an endpoint.
fn over_the_wire<T: serde::Serialize + serde::de::DeserializeOwned>(value: &T) -> T {
    serde_json::from_str(&serde_json::to_string(value).unwrap()).unwrap()
}

fn profile() -> CapabilityProfile {
    CapabilityProfile::new([PresentationCapability::NativeGlyph])
}

fn diff(session: &ProjectionSession, base: u64, operations: Vec<SceneOp>) -> ProjectionDiff {
    ProjectionDiff {
        version: ProtocolVersion::V1,
        session: session.clone(),
        scene: SceneDiff {
            epoch: SceneEpoch(1),
            base: Revision(base),
            revision: Revision(base + 1),
            operations,
        },
        presentation: Vec::new(),
        status: None,
    }
}

fn reached(client: &ClientState, session: &ProjectionSession) -> Vec<String> {
    client
        .accessibility_tree(session, &profile())
        .unwrap()
        .children
        .into_iter()
        .map(|item| item.label)
        .collect()
}

#[test]
fn a_mounted_fold_hides_its_members_behind_a_counted_stand_in() {
    let session = ProjectionSession("loopback:fold".into());
    let mut client = ClientState::default();
    client
        .apply_snapshot(over_the_wire(&mounted(
            &session,
            Some(dependencies_of_mere()),
        )))
        .unwrap();

    let tree = client.accessibility_tree(&session, &profile()).unwrap();
    assert_eq!(
        tree.children
            .iter()
            .map(|item| item.label.as_str())
            .collect::<Vec<_>>(),
        ["Mere"]
    );
    assert_eq!(tree.children[0].stands_in_for, Some(FoldId(0)));
    assert_eq!(tree.folds.len(), 1);
    let fold = &tree.folds[0];
    assert_eq!(fold.label, "Mere");
    assert_eq!(fold.badge, "+3");
    assert_eq!(
        fold.hidden,
        [
            (InstanceId(1), "genet".to_owned()),
            (InstanceId(2), "netrender".to_owned()),
            (InstanceId(3), "retinue".to_owned()),
        ]
    );
    assert_eq!(
        fold.rule.as_deref(),
        Some("Mere and everything it reaches by depends on")
    );
}

#[test]
fn unfolding_arrives_as_a_diff_and_round_trips() {
    let session = ProjectionSession("loopback:unfold".into());
    let mut client = ClientState::default();
    client
        .apply_snapshot(over_the_wire(&mounted(
            &session,
            Some(dependencies_of_mere()),
        )))
        .unwrap();
    let folded = client.mounted(&session).unwrap().clone();

    let unfold = diff(
        &session,
        1,
        vec![SceneOp::TombstoneFold { index: FoldId(0) }],
    );
    assert!(matches!(
        client.apply_diff(&over_the_wire(&unfold)),
        Ok(DiffApplication::Applied(_))
    ));
    assert_eq!(reached(&client, &session), NAMES);
    assert!(
        client
            .accessibility_tree(&session, &profile())
            .unwrap()
            .folds
            .is_empty()
    );

    let refold = diff(
        &session,
        2,
        vec![SceneOp::AddFold {
            index: FoldId(1),
            value: dependencies_of_mere(),
        }],
    );
    client.apply_diff(&over_the_wire(&refold)).unwrap();
    assert_eq!(reached(&client, &session), ["Mere"]);
    let refolded = client.mounted(&session).unwrap();
    assert_eq!(
        refolded.scene.fold_effect().hidden_instances(),
        folded.scene.fold_effect().hidden_instances(),
        "refolding hides exactly what the first fold hid"
    );
    assert_eq!(
        refolded.scene.tables.items, folded.scene.tables.items,
        "no item was rewritten by the round trip"
    );
}

#[test]
fn an_invalid_fold_from_the_endpoint_changes_nothing() {
    let session = ProjectionSession("loopback:refused".into());
    let mut client = ClientState::default();
    client.apply_snapshot(mounted(&session, None)).unwrap();
    let before = client.mounted(&session).unwrap().clone();
    let stray = diff(
        &session,
        1,
        vec![SceneOp::AddFold {
            index: FoldId(0),
            value: Fold {
                stand_in: StandIn::Member(InstanceId(9)),
                ..dependencies_of_mere()
            },
        }],
    );
    assert!(matches!(
        client.apply_diff(&stray),
        Err(ClientDiffError::InvalidScene(_))
    ));
    assert_eq!(client.mounted(&session).unwrap(), &before);
    assert_eq!(reached(&client, &session), NAMES);
}

#[test]
fn a_summary_stand_in_sits_at_its_members_centroid() {
    let session = ProjectionSession("loopback:summary".into());
    let fold = Fold {
        members: vec![InstanceId(1), InstanceId(3)],
        stand_in: StandIn::Summary {
            label: Some("two dependencies".into()),
        },
        rule: Some(FoldRule::Selection),
        boundary: None,
    };
    let mut client = ClientState::default();
    client
        .apply_snapshot(mounted(&session, Some(fold)))
        .unwrap();
    assert_eq!(reached(&client, &session), ["Mere", "netrender"]);
    let tree = client.accessibility_tree(&session, &profile()).unwrap();
    assert_eq!(tree.folds[0].stand_in, None);
    assert_eq!(tree.folds[0].label, "two dependencies");
    assert_eq!(tree.folds[0].badge, "+2");
    assert_eq!(tree.folds[0].rule.as_deref(), Some("chosen by selection"));
    let readings = read_folds(&client.mounted(&session).unwrap().scene);
    assert_eq!(readings[0].summary_at, Some(Vec2::new(60.0, 10.0)));
}
