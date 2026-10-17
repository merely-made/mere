//! Audit-only observations against the current Cargo-built public APIs.
use page_load::*;
use content_contract::{ContentCommandMessage, ContentUpdateMessage, SceneTransferEncoder, SceneTransferDecoder};
use std::collections::HashMap;

#[test]
fn supported_registered_formats_are_classified_as_downloads() {
    let mut registry = inker::EngineRegistry::new();
    for engine in nematic::engines() { registry.register(engine); }
    let policy = inker::EngineRoutePolicy::default();
    for (address, mime, engine, body) in [
        ("nex://example.test/", "application/x-nex", "nematic.nex", "child\n"),
        ("guppy://example.test/", "application/x-guppy", "nematic.guppy", "# Guppy\n"),
        ("titan://example.test/", "application/x-titan", "nematic.titan", "# Titan\n"),
        ("https://example.test/feed", "application/feed+json", "nematic.feed", r#"{"version":"https://jsonfeed.org/version/1.1","title":"Feed","items":[{"id":"one","title":"Entry","content_text":"Body"}]}"#),
    ] {
        let decision = policy.route(&inker::EngineRouteRequest {
            workspace_id: inker::WorkspaceRouteId::new("audit"), view: None, node: None,
            address: address.into(), content_type: Some(mime.into()), pinned_engine: None,
        });
        assert_eq!(decision.engine_id, engine);
        let doc = registry.dispatch(&decision, &inker::EngineInput::new(address, body).with_content_type(mime)).unwrap();
        assert!(!doc.blocks.is_empty());
        let mut load = PageLoad::default(); load.begin(1);
        let answer = load.accept_outcome(FetchOutcome { request: 1, url: address.into(), result: Ok(Fetched::text(Some(mime.into()), body)) }, 10);
        assert!(matches!(answer, LoadAnswer::Download(_)));
        assert!(load.fetched(address).is_none());
    }
    assert!(!is_download_response(&Fetched::text(Some("text/gemini".into()), "# Control")));
    assert!(!is_download_response(&Fetched::text(Some("application/gopher-menu".into()), "iInfo\t\t\t")));
    println!("Format authority: actual default registry renders Nex/Guppy/Titan/JSON Feed, but PageLoad hands all four back as Download; Gemini/Gopher controls stay renderable");
}

const PAGE: &str = "gemini://example.test/page";
fn old_load() -> PageLoad {
    let mut load = PageLoad::default();
    load.note_fetched(PAGE.into(), LoadedDocument {
        bytes: b"old".to_vec(), body: "old".into(), content_type: Some("text/gemini".into()),
        effective_url: Some("gemini://example.test/old-target".into()), acquired_at_ms: 10,
    }, 3); load
}
fn terminal(request: u64) -> FetchOutcome {
    FetchOutcome { request, url: PAGE.into(), result: Ok(Fetched::text(None, "new bytes")) }
}
#[test]
fn new_same_address_request_inherits_old_response_metadata() {
    let mut load = old_load(); load.begin(2);
    assert_eq!(load.accept_outcome(terminal(2), 30), LoadAnswer::Document);
    let document = load.fetched(PAGE).unwrap(); assert_eq!(document.bytes, b"new bytes");
    assert_eq!(document.acquired_at_ms, 10);
    assert_eq!(document.effective_url.as_deref(), Some("gemini://example.test/old-target"));
    assert_eq!(document.content_type.as_deref(), Some("text/gemini"));
    let mut stream = old_load(); stream.begin(3);
    stream.accept_progress(PageProgress { request: 3, url: PAGE.into(), response_url: "gemini://example.test/new-target".into(), content_type: Some("text/plain".into()), bytes: b"new".to_vec() }, 40).unwrap();
    assert_eq!(stream.fetched(PAGE).unwrap().acquired_at_ms, 10);
    assert_eq!(stream.fetched(PAGE).unwrap().effective_url.as_deref(), Some("gemini://example.test/new-target"));
    let mut cleared = old_load(); cleared.forget_fetched(); cleared.begin(4);
    assert_eq!(cleared.accept_outcome(terminal(4), 50), LoadAnswer::Document);
    assert_eq!(cleared.fetched(PAGE).unwrap().acquired_at_ms, 50);
    assert_eq!(cleared.fetched(PAGE).unwrap().effective_url, None);
    println!("Request metadata: new bytes under same URL inherit prior time/redirect/MIME; even fresh progress inherits prior time. Explicit forget control starts fresh, as Pelt's Reload source currently does");
}

fn image_scene(url: &str, width: u32, rgba: Vec<u8>) -> netrender::Scene {
    use document_canvas::*;
    let packet = DocumentRenderPacket {
        viewport: Viewport::new(4., 4.), content_bounds: Rect::from_xywh(0., 0., 4., 4.),
        blocks: vec![RenderedBlock { source_block_index: 0, bounds: Rect::from_xywh(0., 0., 4., 4.), kind: RenderedBlockKind::Image { url: url.into(), alt: "fixture".into() } }], interactions: vec![],
    };
    let fonts = document_canvas::font_table::FontInterner::new().into_table();
    let images = HashMap::from([(url.into(), DecodedImage { width, height: 1, rgba8: rgba })]);
    let list = paint_list_from_packet_with_images(&packet, &fonts, &ColorVocabulary::default(), &images);
    paint_list_render::translate_paint_list(&list)
}
fn transfer(scene: netrender::Scene, encoder: &mut SceneTransferEncoder, decoder: &mut SceneTransferDecoder) -> netrender::Scene {
    let stats = content_contract::scene_stats(&scene);
    let buffer = ContentUpdateMessage::Scene { nav: 1, viewport_gen: 1, scene, stats, content_height: 4, band_y: 0, band_h: 4, links: vec![], masks: vec![] }.into_transfer_buffer(encoder).unwrap();
    let ContentUpdateMessage::Scene { scene, .. } = ContentUpdateMessage::from_transfer_buffer(buffer.as_bytes(), decoder).unwrap() else { panic!("Scene expected") }; scene
}
#[test]
fn reused_transfer_cache_replaces_new_image_with_old_image_at_recycled_key() {
    let red = vec![255, 0, 0, 255]; let blue = vec![0, 0, 255, 255, 0, 0, 255, 255];
    let first = image_scene("https://example.test/first.png", 1, red.clone());
    let second = image_scene("https://example.test/second.png", 2, blue.clone());
    assert!(!first.ops.is_empty()); assert!(!second.ops.is_empty());
    assert_eq!(first.image_sources.len(), 1); assert_eq!(second.image_sources.len(), 1);
    let key = *first.image_sources.keys().next().unwrap(); assert!(second.image_sources.contains_key(&key));
    assert_eq!(second.image_sources[&key].data.data(), &blue);
    let mut encoder = SceneTransferEncoder::default(); let mut decoder = SceneTransferDecoder::default();
    let a = transfer(first, &mut encoder, &mut decoder); assert_eq!(a.image_sources[&key].data.data(), &red);
    let b = transfer(second.clone(), &mut encoder, &mut decoder);
    assert_eq!(b.image_sources[&key].width, 1); assert_eq!(b.image_sources[&key].data.data(), &red);
    let fresh = transfer(second, &mut SceneTransferEncoder::default(), &mut SceneTransferDecoder::default());
    assert_eq!(fresh.image_sources[&key].width, 2); assert_eq!(fresh.image_sources[&key].data.data(), &blue);
    println!("Scene image identity: actual document packet/paint-list translator recycles image key; reused worker cache silently restores prior red1px image over new blue2px image. Fresh cache roundtrip preserves blue; no GPU rendering");
}

#[test]
fn transfer_buffer_direction_is_external_to_the_envelope() {
    let buffer = ContentCommandMessage::Scroll { band_y: 7, band_h: 8, viewport_gen: 0 }.into_transfer_buffer().unwrap();
    assert!(matches!(ContentCommandMessage::from_transfer_buffer(buffer.as_bytes()).unwrap(), ContentCommandMessage::Scroll { band_y: 7, band_h: 8, viewport_gen: 0 }));
    let other = ContentUpdateMessage::from_transfer_buffer(buffer.as_bytes(), &mut SceneTransferDecoder::default()).unwrap();
    assert!(matches!(other, ContentUpdateMessage::TextSelection { nav: 7, viewport_gen: 8, selection: None }));
    println!("Wire qualification: one legitimate Scroll command buffer also decodes as TextSelection update; direction/schema are channel/caller context, not self-identified. No production misdelivery or forged authority claim");
}

#[test]
fn framing_strict_header_and_declared_legacy_limits_remain_distinct() {
    const MAGIC: [u8; 8] = *b"AUDITREC"; const OTHER: [u8; 8] = *b"OTHERREC";
    let frame = framing::frame(MAGIC, 2, &7u8).unwrap();
    assert_eq!(framing::unframe::<u8>(MAGIC, 2, &frame), Ok(7));
    assert!(matches!(framing::unframe::<u8>(OTHER, 2, &frame), Err(framing::FrameError::WrongSchema { .. })));
    assert!(matches!(framing::unframe::<u8>(MAGIC, 3, &frame), Err(framing::FrameError::UnknownVersion { .. })));
    let mut trailing = frame; trailing.extend_from_slice(b"tail");
    assert_eq!(framing::unframe::<u8>(MAGIC, 2, &trailing), Ok(7));
    let foreign = framing::frame(OTHER, 0, &7u8).unwrap();
    assert!(matches!(framing::unframe::<u8>(MAGIC, 0, &foreign), Err(framing::FrameError::WrongSchema { .. })));
    assert_eq!(framing::unframe_or_legacy::<u8>(MAGIC, 0, &foreign), Ok(b'O'));
    println!("Framing controls: magic/version checked before payload; declared trailing-byte compatibility and no-magic legacy fallback are intentionally weaker. Legacy reading is not schema authentication or full byte exhaustion");
}
