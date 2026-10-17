//! Public capture/codec boundaries; generated data only, no real network.
use chirograph::*;
use graphshell_client::ClientState;
use sceno::{Scene, SourceRef, InstanceId, ProjectedItem, Transform2, Footprint, Representation};
use tokio::io::{AsyncWriteExt, BufReader};

fn bytes() -> Vec<u8> { serde_json::to_vec(&NativeGlyphV1 { label: "audit".into(), icon: None, color: None }).unwrap() }
fn snapshot() -> ProjectionSnapshot {
    let mut scene = Scene::new();
    let source = scene.intern_source(SourceRef::new("audit", "one"));
    scene.items.push(ProjectedItem { source, space: Scene::WORLD, transform: Transform2::IDENTITY, footprint: Footprint::Point, representation: Representation::Glyph, layer: 0, visible: true, hit: None, channels: vec![] });
    let key = PresentationKey("audit:one".into());
    ProjectionSnapshot { version: ProtocolVersion::V1, session: ProjectionSession("audit:one".into()),
        scene: SceneSnapshot::from_dense(SceneEpoch(1), Revision(1), scene).unwrap(),
        presentation: PresentationManifest { bindings: vec![PresentationBinding { instance: InstanceId(0), key: key.clone() }],
            offers: [(key, vec![PresentationOffer { codec: PresentationCodec::NativeGlyphV1, resource: ContentHash::of(&bytes()), byte_size: bytes().len() as u64, requires: PresentationCapability::NativeGlyph,
                semantics: PresentationSemantics { label: "audit".into(), role: SemanticRole::Graphic, bounds: BoundsRelationship::FillFootprint, actions: vec![] } }])].into_iter().collect() },
        cache_policy: CachePolicy::default() }
}
#[test] fn valid_capture_and_live_client_control_agree() {
    let snapshot = snapshot();
    ClientState::default().apply_snapshot(snapshot.clone()).unwrap();
    let v1 = ProjectionCaptureV1::from_projection(&snapshot).unwrap();
    let roundtrip = ProjectionCaptureV1::decode(&v1.encode().unwrap()).unwrap();
    roundtrip.verify_resources(|hash| hash == ContentHash::of(&bytes())).unwrap();
    println!("CONTROL valid-capture live-and-capture-accept=true");
}
#[test] fn malformed_presentations_survive_capture_validation_and_decode() {
    for defect in ["absent-item", "duplicate-binding", "wrong-capability"] {
        let mut snapshot = snapshot();
        match defect {
            "absent-item" => snapshot.presentation.bindings[0].instance = InstanceId(7),
            "duplicate-binding" => snapshot.presentation.bindings.push(snapshot.presentation.bindings[0].clone()),
            _ => snapshot.presentation.offers.values_mut().next().unwrap()[0].requires = PresentationCapability::Image,
        }
        snapshot.scene.validate().unwrap();
        assert!(ClientState::default().apply_snapshot(snapshot.clone()).is_err());
        let v1 = ProjectionCaptureV1::from_projection(&snapshot).unwrap();
        let v1 = ProjectionCaptureV1::decode(&v1.encode().unwrap()).unwrap();
        v1.verify_resources(|hash| hash == ContentHash::of(&bytes())).unwrap();
        let v2 = ProjectionCaptureV2::from(v1);
        let v2 = ProjectionCaptureV2::decode(&v2.encode().unwrap()).unwrap();
        v2.verify_resources(|hash| hash == ContentHash::of(&bytes())).unwrap();
        println!("OBSERVED {defect} live-refuses=true v1-v2-validate-decode-resource-check-accept=true");
    }
}
fn assembled(bytes: &[u8], partial: bool) -> String {
    let mut assembler = line::LineAssembler::default();
    assembler.push(bytes);
    if partial { assembler.take_partial().unwrap() } else { assembler.next_line().unwrap() }
}
fn native_read(bytes: &[u8]) -> std::io::Result<Option<Result<serde_json::Value, serde_json::Error>>> {
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        let (client, mut writer) = tokio::io::duplex(1024);
        writer.write_all(bytes).await.unwrap();
        writer.shutdown().await.unwrap();
        graphshell_host::line::LineReader::new(BufReader::new(client)).next::<serde_json::Value>().await
    })
}
#[test] fn valid_utf8_split_across_frames_control_agrees() {
    let text = "{\"label\":\"café\"}\n";
    let bytes = text.as_bytes();
    let cut = bytes.iter().position(|b| *b == 0xc3).unwrap()+1;
    let mut assembler = line::LineAssembler::default();
    assembler.push(&bytes[..cut]);
    assert!(assembler.next_line().is_none());
    assembler.push(&bytes[cut..]);
    let browser = line::decode::<serde_json::Value>(&assembler.next_line().unwrap()).unwrap().unwrap();
    let native = native_read(bytes).unwrap().unwrap().unwrap();
    assert_eq!(browser, native);
    println!("CONTROL split-multibyte-utf8 readers-agree=true");
}
#[test] fn invalid_utf8_is_rewritten_into_valid_json_by_frame_assembler() {
    let bad = b"{\"id\":1,\"body\":{\"Ok\":{\"Descriptor\":{\"label\":\"\xff\",\"projections\":[]}}}}\n";
    let browser = line::decode::<CarrierOutput>(&assembled(bad, false)).unwrap().unwrap();
    assert!(matches!(&browser, CarrierOutput::Response(CarrierResponse { body: Ok(CarrierResponseBody::Descriptor(d)), .. }) if d.label == "\u{fffd}"));
    let native = native_read(bad).unwrap_err();
    assert_eq!(native.kind(), std::io::ErrorKind::InvalidData);
    let partial = line::decode::<CarrierOutput>(&assembled(&bad[..bad.len()-1], true)).unwrap().unwrap();
    assert_eq!(partial, browser);
    println!("OBSERVED invalid-utf8 frame-assembler=accepted-with-replacement native-line-reader=InvalidData unterminated-tail-also-rewritten=true");
}
#[test] fn chunk_assembler_only_verifies_chunk_hash_and_final_resource() {
    let original = ResourceResponse::new(ProjectionSession("one".into()), b"abcd".to_vec());
    let other = ResourceResponse::new(ProjectionSession("two".into()), b"abXY".to_vec());
    let mut assembly = ResourceAssembly::new();
    assert!(assembly.accept(&ResourceChunkResponse::slice(&original, 0, 2)).unwrap().is_none());
    let switched = ResourceChunkResponse::slice(&other, 2, 2);
    assert_eq!(assembly.accept(&switched).unwrap(), Some(b"abXY".to_vec()));
    let mut empty = ResourceChunkResponse::slice(&original, 0, 2);
    empty.bytes.clear(); empty.chunk = ContentHash::of(b"");
    let mut assembly = ResourceAssembly::new();
    for _ in 0..3 { assert!(assembly.accept(&empty).unwrap().is_none()); }
    assert_eq!(assembly.received(), 0);
    println!("QUALIFIED shared-assembler scope-change-and-empty-progress-accepted=true native-app-wrapper-adds-these-checks-in-source=true");
}
