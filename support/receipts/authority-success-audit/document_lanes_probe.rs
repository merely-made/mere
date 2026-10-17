use mere_document_lanes::*;
use inker::*;
use document_canvas::{DocumentStyleSheet, LinkAdornment, InteractionKind};
use genet_host_api::{ResourceFetcher, ResourceResponse};
fn doc(blocks: Vec<Block>) -> EngineDocument {
    EngineDocument { address: "gemini://example.test/a".into(), title: None, content_type: "text/gemini".into(),
        lang: None, provenance: Default::default(), trust: Default::default(), diagnostics: vec![], navigation: Default::default(), blocks }
}
fn text(s: &str) -> InlineSpan { InlineSpan::Text(s.into()) }
fn link(label: &str, url: &str) -> Block { Block::Paragraph { spans: vec![InlineSpan::Link {
    url: url.into(), title: None, predicate: None, spans: vec![text(label)] }] } }
fn image() -> Block { Block::Image { url: "https://example.test/a.png".into(), alt: "picture".into() } }
struct NoFetch;
impl ResourceFetcher for NoFetch { fn fetch(&self, _: &str) -> Option<Vec<u8>> { panic!("body fixture must not fetch") } }

#[test]
fn selected_text_engine_still_spawns_gemtext_parser() {
    let engine = SmolwebSessionEngine::new(inker::routing::ENGINE_NEMATIC_TEXT, NoFetch, SmolwebTheme::Plain);
    assert_eq!(engine.engine_id(), inker::routing::ENGINE_NEMATIC_TEXT);
    let session = engine.spawn(&SessionSpawnRequest::new("https://example.test/plain").with_body("# Literal heading syntax\n")).unwrap();
    let mut_session = session.as_any_ref().downcast_ref::<SmolwebDocumentSession>().unwrap();
    let report = mut_session.inspect().unwrap();
    assert_eq!(report.title.as_deref(), Some("Literal heading syntax"));
    let mut session = session;
    let concrete = session.as_any().downcast_mut::<SmolwebDocumentSession>().unwrap();
    assert_eq!(concrete.document_mut().document().provenance.source_kind.as_deref(), Some(inker::routing::ENGINE_NEMATIC_GEMTEXT));
    assert!(matches!(concrete.document_mut().document().blocks[0], Block::Heading { .. }));
    println!("selected engine nematic.text -> actual nematic.gemtext and Heading; no fetch");
}

#[test]
fn nested_images_are_absent_from_resource_requests() {
    let top = SmolwebDocument::from_document_with_theme(doc(vec![image()]), SmolwebTheme::Plain);
    let quoted = SmolwebDocument::from_document_with_theme(doc(vec![Block::Quote { blocks: vec![image()] }]), SmolwebTheme::Plain);
    let listed = SmolwebDocument::from_document_with_theme(doc(vec![Block::List { ordered: false, items: vec![vec![image()]] }]), SmolwebTheme::Plain);
    assert_eq!(top.subresources(), ["https://example.test/a.png"]);
    assert!(quoted.subresources().is_empty()); assert!(listed.subresources().is_empty());
    println!("subresources: top-level image requested; identical image inside Quote/List omitted");
}

#[test]
fn retained_focus_retargets_after_empty_link_adornment_is_hidden() {
    fn run(first: &str) -> (InteractionKind, InteractionKind) {
        let mut view = SmolwebDocument::from_document(doc(vec![link(first, "gemini://example.test/empty"),
            link("Second", "gemini://example.test/second"), link("Third", "gemini://example.test/third")]), DocumentStyleSheet::default(), [1.0;4]);
        view.frame(500, 500); view.focus_move(SessionFocusDirection::Forward, 500, 500); view.focus_move(SessionFocusDirection::Forward, 500, 500);
        let before = view.focused_interaction(500, 500).unwrap();
        let mut style = view.style_sheet().clone(); style.link_adornment = LinkAdornment::None;
        assert!(view.set_style_sheet(style)); view.frame(500, 500);
        (before, view.focused_interaction(500, 500).unwrap())
    }
    let expected = InteractionKind::Link { url: "gemini://example.test/second".into() };
    let (before, after) = run(""); assert_eq!(before, expected);
    assert_eq!(after, InteractionKind::Link { url: "gemini://example.test/third".into() });
    let (before, after) = run("First"); assert_eq!(before, expected); assert_eq!(after, expected);
    println!("Q144 consumer: retained keyboard focus moves Second -> Third after hiding earlier empty-link adornment; nonempty-label control retains Second");
}

#[test]
fn redirect_final_metadata_is_discarded_before_inline_image_resolution() {
    struct Redirected;
    impl ResourceFetcher for Redirected {
        fn fetch(&self, url: &str) -> Option<Vec<u8>> { self.fetch_response(url).map(|r| r.bytes) }
        fn fetch_response(&self, _: &str) -> Option<ResourceResponse> {
            Some(ResourceResponse::new("gemini://final.test/actual/index.gmi", b"=> image.png Picture\n".to_vec()).with_content_type("text/gemini"))
        }
    }
    let view = SmolwebDocument::load_with_inline_media(&Redirected, "gemini://alias.test/old/index.gmi", SmolwebTheme::Plain, SmolwebInlineMediaPolicy::images()).unwrap();
    assert_eq!(view.document().address, "gemini://alias.test/old/index.gmi");
    assert_eq!(view.subresources(), ["gemini://alias.test/old/image.png"]);
    let correct = SmolwebDocument::parse_with_inline_media("gemini://final.test/actual/index.gmi", "=> image.png Picture\n", SmolwebTheme::Plain, SmolwebInlineMediaPolicy::images());
    assert_eq!(correct.subresources(), ["gemini://final.test/actual/image.png"]);
    println!("load: rich mock response final URL discarded; relative image uses requested alias directory instead of final directory");
}

#[test]
fn ragged_table_semantics_take_the_preceding_rows_padding_box() {
    let cell = |s: &str| vec![text(s)];
    let mut session = SmolwebDocumentSession::new(SmolwebDocument::from_document_with_theme(doc(vec![Block::Table {
        alignments: vec![], header: vec![], rows: vec![vec![cell("A0")], vec![cell("B0"), cell("B1")]] }]), SmolwebTheme::Plain), (600, 400));
    session.frame(600,400); let projection = session.accessibility_projection().unwrap();
    let bounds = |name: &str| projection.nodes().iter().find(|n| n.name.as_deref()==Some(name)).unwrap().bounds.unwrap();
    assert_eq!(bounds("A0").y, bounds("B0").y);
    assert!(bounds("B0").x > bounds("A0").x);
    assert!(bounds("B1").y > bounds("B0").y);
    println!("ragged table: semantic B0 gets row0 column1 padding geometry; B1 gets row1 column0");
}

#[test]
fn clickable_image_has_no_accessibility_activation() {
    let mut session = SmolwebDocumentSession::new(SmolwebDocument::from_document_with_theme(doc(vec![image()]), SmolwebTheme::Plain), (500, 400));
    session.frame(500,400); let links = session.links(); assert_eq!(links.len(),1);
    let [x,y,w,h] = links[0].rect;
    assert_eq!(session.click_at(x+w/2.0,y+h/2.0), SessionClick::Navigate("https://example.test/a.png".into()));
    let projection = session.accessibility_projection().unwrap();
    let image = projection.nodes().iter().find(|n| n.role==DocumentA11yRole::Image).unwrap();
    assert_eq!(image.actions, [DocumentA11yAction::ScrollIntoView]);
    assert!(projection.nodes().iter().all(|n| !n.actions.contains(&DocumentA11yAction::Click)));
    assert!(session.accessibility_click_target(image.id).is_none());
    println!("image: pointer navigates; semantic image has only ScrollIntoView and no click target");
}

#[test]
fn synchronous_http_bridge_panics_inside_a_tokio_task_before_fetching() {
    let fetcher = RemoteFetcher::new(ResourceFetchPolicy::default());
    let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| rt.block_on(async {
        fetcher.fetch("http://127.0.0.1:1/")
    })));
    assert!(outcome.is_err());
    assert!(fetcher.fetch("audit-unsupported:fixture").is_none());
    println!("remote bridge: nested Runtime::block_on panics in async task before polling HTTP future; unsupported-scheme control returns None");
}
