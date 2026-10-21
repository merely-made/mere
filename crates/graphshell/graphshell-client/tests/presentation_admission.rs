//! Q067 public-API probe, adapted from the sealed Chirograph audit probe.
//! Keeps its three malformed sidecars and valid control; expects repaired admission.
use chirograph::*;
use graphshell_client::ClientState;
use sceno::{Footprint, InstanceId, ProjectedItem, Representation, Scene, SourceRef, Transform2};

fn bytes() -> Vec<u8> {
    serde_json::to_vec(&NativeGlyphV1 {
        label: "audit".into(),
        icon: None,
        color: None,
    })
    .unwrap()
}
fn snapshot() -> ProjectionSnapshot {
    let mut scene = Scene::new();
    let source = scene.intern_source(SourceRef::new("audit", "one"));
    scene.items.push(ProjectedItem {
        source,
        space: Scene::WORLD,
        transform: Transform2::IDENTITY,
        footprint: Footprint::Point,
        representation: Representation::Glyph,
        layer: 0,
        visible: true,
        hit: None,
        channels: vec![],
    });
    let key = PresentationKey("audit:one".into());
    ProjectionSnapshot {
        version: ProtocolVersion::V1,
        session: ProjectionSession("audit:one".into()),
        scene: SceneSnapshot::from_dense(SceneEpoch(1), Revision(1), scene).unwrap(),
        presentation: PresentationManifest {
            bindings: vec![PresentationBinding {
                instance: InstanceId(0),
                key: key.clone(),
            }],
            offers: [(
                key,
                vec![PresentationOffer {
                    codec: PresentationCodec::NativeGlyphV1,
                    resource: ContentHash::of(&bytes()),
                    byte_size: bytes().len() as u64,
                    requires: PresentationCapability::NativeGlyph,
                    semantics: PresentationSemantics {
                        label: "audit".into(),
                        role: SemanticRole::Graphic,
                        bounds: BoundsRelationship::FillFootprint,
                        actions: vec![],
                    },
                }],
            )]
            .into_iter()
            .collect(),
        },
        cache_policy: CachePolicy::default(),
    }
}
#[test]
fn valid_capture_and_live_client_control_agree() {
    let snapshot = snapshot();
    ClientState::default()
        .apply_snapshot(snapshot.clone())
        .unwrap();
    let v1 = ProjectionCaptureV1::from_projection(&snapshot).unwrap();
    let roundtrip = ProjectionCaptureV1::decode(&v1.encode().unwrap()).unwrap();
    roundtrip
        .verify_resources(|hash| hash == ContentHash::of(&bytes()))
        .unwrap();
    let v2 = ProjectionCaptureV2::from(roundtrip);
    let v2 = ProjectionCaptureV2::decode(&v2.encode().unwrap()).unwrap();
    v2.verify_resources(|hash| hash == ContentHash::of(&bytes()))
        .unwrap();
}

#[test]
fn q067_capture_versions_and_live_client_reject_the_same_malformed_presentations() {
    for defect in [
        "absent-item",
        "tombstoned-item",
        "duplicate-binding",
        "wrong-capability",
    ] {
        let mut snapshot = snapshot();
        let mut client = ClientState::default();
        client.apply_snapshot(snapshot.clone()).unwrap();
        let before = client.clone();
        match defect {
            "absent-item" => snapshot.presentation.bindings[0].instance = InstanceId(7),
            "tombstoned-item" => {
                snapshot.scene.tables.items[0] = None;
                snapshot.scene.tables.item_order[0] = None;
            },
            "duplicate-binding" => snapshot
                .presentation
                .bindings
                .push(snapshot.presentation.bindings[0].clone()),
            _ => {
                snapshot.presentation.offers.values_mut().next().unwrap()[0].requires =
                    PresentationCapability::Image
            },
        }
        snapshot.scene.validate().unwrap();
        assert!(client.apply_snapshot(snapshot.clone()).is_err(), "{defect}");
        assert_eq!(client, before, "{defect} mutated live state");
        assert!(
            matches!(
                ProjectionCaptureV1::from_projection(&snapshot),
                Err(ProjectionCaptureError::InvalidPresentation(_))
            ),
            "{defect}"
        );
        // Wire data can bypass constructors; encoding and all decode versions
        // must still reject it before any resource-availability callback runs.
        let v1 = ProjectionCaptureV1 {
            version: PROJECTION_CAPTURE_VERSION,
            scene: snapshot.scene,
            presentation: snapshot.presentation,
        };
        assert!(
            matches!(
                v1.encode(),
                Err(ProjectionCaptureError::InvalidPresentation(_))
            ),
            "{defect}"
        );
        let raw_v1 = serde_json::to_vec(&v1).unwrap();
        assert!(
            matches!(
                ProjectionCaptureV1::decode(&raw_v1),
                Err(ProjectionCaptureError::InvalidPresentation(_))
            ),
            "{defect}"
        );
        assert!(
            matches!(
                ProjectionCaptureV2::decode(&raw_v1),
                Err(ProjectionCaptureError::InvalidPresentation(_))
            ),
            "{defect}"
        );
        assert!(
            matches!(
                v1.verify_resources(|_| panic!("malformed capture reached resource lookup")),
                Err(ProjectionCaptureError::InvalidPresentation(_))
            ),
            "{defect}"
        );
        let v2 = ProjectionCaptureV2::from(v1);
        assert!(
            matches!(
                v2.encode(),
                Err(ProjectionCaptureError::InvalidPresentation(_))
            ),
            "{defect}"
        );
        assert!(
            matches!(
                ProjectionCaptureV2::decode(&serde_json::to_vec(&v2).unwrap()),
                Err(ProjectionCaptureError::InvalidPresentation(_))
            ),
            "{defect}"
        );
        assert!(
            matches!(
                v2.verify_resources(|_| panic!("malformed capture reached resource lookup")),
                Err(ProjectionCaptureError::InvalidPresentation(_))
            ),
            "{defect}"
        );
    }
}

#[test]
fn captures_do_not_invent_session_cache_policy() {
    let mut snapshot = snapshot();
    let offer = snapshot
        .presentation
        .offers
        .values_mut()
        .next()
        .unwrap()
        .first_mut()
        .unwrap();
    offer.codec = PresentationCodec::EditableTextV1;
    offer.requires = PresentationCapability::EditableText;
    // Immutable capture bytes contain no live session cache policy.
    snapshot.cache_policy.retention = CacheRetention::Exportable;
    snapshot.cache_policy.purge_on_revocation = false;
    assert!(
        ClientState::default()
            .apply_snapshot(snapshot.clone())
            .is_err()
    );
    let v1 = ProjectionCaptureV1::from_projection(&snapshot).unwrap();
    ProjectionCaptureV1::decode(&v1.encode().unwrap()).unwrap();
    let v2 = ProjectionCaptureV2::from(v1);
    ProjectionCaptureV2::decode(&v2.encode().unwrap()).unwrap();
    snapshot.cache_policy = CachePolicy::default();
    ClientState::default().apply_snapshot(snapshot).unwrap();
}
