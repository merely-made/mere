use inker::*;
use nematic::*;

fn render(engine: &dyn Engine, body: &str) -> EngineDocument {
    engine.render(&EngineInput::new("https://example.test/document", body)).unwrap()
}

#[test]
fn active_url_filter_accepts_browser_normalized_control_spelling() {
    let html = HtmlFragmentEngine::new();
    let blocked = render(&html, "<p><a href='javascript:fixture()'>label</a></p>");
    assert!(blocked.outgoing_links().is_empty());
    for entity in ["&#10;", "&#9;", "&#13;"] {
        let body = format!("<p><a href='java{entity}script:fixture()'>label</a></p>");
        let accepted = render(&html, &body);
        assert_eq!(accepted.outgoing_links().len(), 1);
        let target = accepted.outgoing_links()[0];
        assert_eq!(url::Url::parse(target).unwrap().scheme(), "javascript");
        assert!(accepted.diagnostics.is_empty());
    }
    let good = render(&html, "<p><a href='https://example.test/'>label</a></p>");
    assert_eq!(good.outgoing_links(), ["https://example.test/"]);
    println!("HTML URL admission: ordinary javascript blocked; embedded LF/TAB/CR spellings pass without diagnostic and url::Url normalizes scheme to javascript; no execution");
}

#[test]
fn html_whitespace_only_text_between_inline_nodes_is_lost() {
    let html = HtmlFragmentEngine::new();
    let joined = render(&html, "<p><em>red</em> <em>fox</em></p>");
    assert_eq!(joined.to_text().trim(), "redfox");
    for control in ["<p>red fox</p>", "<p>red <em>fox</em></p>"] {
        assert_eq!(render(&html, control).to_text().trim(), "red fox");
    }
    println!("HTML whitespace: separate em nodes with a source space lower to redfox; text-bearing whitespace controls retain red fox");
}

#[test]
fn default_djot_lane_silently_flattens_structural_blocks() {
    let engine = DjotKnotEngine::new();
    let quote = render(&engine, "> quoted\n");
    let list = render(&engine, "- first\n- second\n");
    let table = render(&engine, "| A | B |\n| --- | --- |\n| C | D |\n");
    assert!(!quote.blocks.iter().any(|b| matches!(b, Block::Quote { .. })));
    assert!(!list.blocks.iter().any(|b| matches!(b, Block::List { .. })));
    assert!(!table.blocks.iter().any(|b| matches!(b, Block::Table { .. })));
    assert!(quote.to_text().contains("quoted"));
    assert!(list.to_text().contains("first") && list.to_text().contains("second"));
    assert!(quote.diagnostics.is_empty() && list.diagnostics.is_empty() && table.diagnostics.is_empty());
    assert!(matches!(render(&KnotEngine::new(), "> quoted\n").blocks[0], Block::Quote { .. }));
    let policy = EngineRoutePolicy::default();
    let decision = policy.route(&EngineRouteRequest { workspace_id: WorkspaceRouteId::new("audit"),
        view: None, node: None, address: "file:///note.knot".into(), content_type: Some("text/x-knot".into()), pinned_engine: None });
    assert_eq!(decision.engine_id, ENGINE_KNOT_DJOT);
    println!("default Djot: quote/list containers absent and table blocks={:?}; no degradation diagnostics; compatibility quote control preserves Quote", table.blocks);
}

#[test]
fn djot_serializer_loses_recognized_link_and_predicate() {
    let a = render(&DjotKnotEngine::new(), "[Topic](mere://node/topic){rel=\"cites\"}\n");
    assert_eq!(a.outgoing_links(), ["mere://node/topic"]);
    assert_eq!(link_statements(&a).len(),1);
    let wire = nematic::knot::djot::blocks_to_djot(&a.blocks);
    let b = render(&DjotKnotEngine::new(), &wire);
    assert!(b.outgoing_links().is_empty()); assert!(link_statements(&b).is_empty());
    assert_eq!(a.to_text().contains("Topic"), b.to_text().contains("Topic"));
    println!("Djot serializer: recognized link/rel input exports as {:?}; reparse loses destination and statement", wire);
}

#[test]
fn block_sources_claimed_metadata_disappears_on_reparse() {
    let source = DocumentProvenance::for_engine("audit.source", "https://main.test/");
    let mut map = BlockProvenanceMap::new();
    map.insert(1, BlockProvenance::from_document(DocumentProvenance::for_engine("audit.other", "https://other.test/private-source")).with_anchor("L7-L9"));
    let blocks = vec![Block::Paragraph { spans: vec![InlineSpan::Text("first".into())] },
        Block::Paragraph { spans: vec![InlineSpan::Text("second".into())] }];
    let wire = nematic::knot::build_clip_knot_with_block_provenance(&blocks, &source, DocumentTrustState::Unknown, None, &map);
    assert!(wire.contains("block_sources:") && wire.contains("https://other.test/private-source"));
    for engine in [&KnotEngine::new() as &dyn Engine, &DjotKnotEngine::new() as &dyn Engine] {
        let doc = render(engine, &wire);
        assert_eq!(doc.provenance.canonical_uri.as_deref(), Some("https://main.test/"));
        assert!(!format!("{doc:?}").contains("https://other.test/private-source"));
        assert!(!doc.to_knot().contains("block_sources:"));
        assert!(!doc.blocks.iter().any(|b| matches!(b, Block::MetadataRow {label,..} if label=="block_sources")));
    }
    println!("block_sources: emitted wire contains override, both Knot parsers drop it entirely; document-source control remains");
}

#[test]
fn extension_and_mime_routes_parse_the_same_knot_differently() {
    let body = ": Trust\n\n  tofu\n";
    let mut registry = EngineRegistry::new(); for engine in engines() { registry.register(engine); }
    let mut req = EngineRouteRequest { workspace_id: WorkspaceRouteId::new("audit"), view: None, node: None,
        address: "file:///note.knot".into(), content_type: None, pinned_engine: None };
    let policy = EngineRoutePolicy::default();
    let extension = policy.route(&req);
    assert_eq!(extension.engine_id, ENGINE_FILE);
    let a = registry.dispatch(&extension, &EngineInput::new(&req.address, body)).unwrap();
    assert_eq!(a.provenance.source_label.as_deref(), Some(ENGINE_KNOT));
    assert!(!a.blocks.iter().any(|b| matches!(b, Block::MetadataRow { .. })));
    req.content_type = Some("text/x-knot".into());
    let mime = policy.route(&req); assert_eq!(mime.engine_id, ENGINE_KNOT_DJOT);
    let b = registry.dispatch(&mime, &EngineInput::new(&req.address, body).with_content_type("text/x-knot")).unwrap();
    assert!(b.blocks.iter().any(|b| matches!(b, Block::MetadataRow {label,value} if label=="Trust" && value=="tofu")));
    println!("same file/body: extension dispatch -> CommonMark compatibility, explicit native MIME -> Djot and real MetadataRow");
}

#[test]
fn tight_nested_markdown_list_places_child_before_parent_text() {
    let tight = render(&MarkdownEngine::new(), "- outer\n  - inner\n");
    let Block::List { items, .. } = &tight.blocks[0] else { panic!("list") };
    assert!(matches!(items[0][0], Block::List { .. }));
    assert!(matches!(&items[0][1], Block::Paragraph {spans} if inline_text(spans)=="outer"));
    let loose = render(&MarkdownEngine::new(), "- outer\n\n  - inner\n");
    let Block::List { items, .. } = &loose.blocks[0] else { panic!("list") };
    assert!(matches!(&items[0][0], Block::Paragraph {spans} if inline_text(spans)=="outer"));
    assert!(matches!(items[0][1], Block::List { .. }));
    println!("Markdown nested tight item children are [inner List, outer Paragraph]; loose-list control retains [outer Paragraph, inner List]");
}

#[test]
fn crlf_blank_lines_do_not_split_text_paragraphs() {
    let lf = render(&TextEngine::new(), "alpha\n\nbeta\n");
    let crlf = render(&TextEngine::new(), "alpha\r\n\r\nbeta\r\n");
    assert_eq!(lf.blocks.len(), 2); assert_eq!(crlf.blocks.len(), 1);
    let Block::Paragraph {spans} = &crlf.blocks[0] else { panic!("paragraph") };
    assert_eq!(inline_text(spans), "alpha  beta");
    println!("Text blank lines: LF -> two paragraphs; CRLF -> one paragraph with empty line retained as soft breaks");
}

#[test]
fn actual_knot_reparse_confirms_literal_and_semantic_export_loss() {
    let engine = KnotEngine::new();
    let a = render(&engine, "\\*x\\*\n");
    assert!(matches!(&a.blocks[0], Block::Paragraph {spans} if inline_text(spans)=="*x*"));
    let b = render(&engine, &a.to_knot());
    assert_ne!(a.blocks,b.blocks);
    assert!(matches!(&b.blocks[0], Block::Paragraph {spans} if spans.iter().any(|s|matches!(s, InlineSpan::Emphasis(_)))));
    let feed = render(&FeedEngine::new(), r#"{"title":"Feed","items":[{"id":"stable-id","title":"Entry"}]}"#);
    assert!(feed.blocks.iter().any(|b|matches!(b, Block::FeedEntry {guid:Some(id),..} if id=="stable-id")));
    let reopened = render(&engine, &feed.to_knot());
    assert!(reopened.blocks.iter().any(|b|matches!(b, Block::FeedEntry {guid:None,..})));
    println!("Q146/Q147 parser follow-up: escaped literal *x* becomes emphasis after Knot export; parsed FeedEntry stable-id becomes guid=None");
}
