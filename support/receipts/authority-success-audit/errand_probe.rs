//! Audit-only observations against the current Cargo-built APIs.
use errand::serve::{Directory, Entry, EntryKind, Item, Listing, Source, SourceRequest, project};
use inker::{Engine, EngineInput};
use std::sync::{Arc, Mutex};
use std::time::Duration;

fn runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap()
}
fn request(path: &str) -> SourceRequest {
    SourceRequest { scheme: errand::Scheme::Gemini, path: path.into(), query: None }
}
fn listing(display: &str, target: &str) -> Listing {
    Listing { entries: vec![Entry::new(display, target, EntryKind::Text)], ..Listing::default() }
}
fn fixture() -> Directory {
    Directory::new(std::env::var("ERRAND_AUDIT_FIXTURE").unwrap())
}

#[test]
fn directory_lexical_root_check_follows_junction_outside_published_root() {
    runtime().block_on(async {
        assert_eq!(fixture().get(&request("/../outside/sentinel.txt")).await, None);
        let got = fixture().get(&request("/link/sentinel.txt")).await.unwrap();
        assert!(matches!(got, Item::Document { body, .. } if body == b"audit-owned outside sentinel"));
    });
    println!("Directory: parent traversal refused; an owned junction serves an owned file outside published root");
}

#[test]
fn directory_hides_dotfile_from_menu_but_serves_direct_request() {
    runtime().block_on(async {
        let Item::Listing(root) = fixture().get(&request("/")).await.unwrap() else { panic!("listing expected") };
        assert!(!root.entries.iter().any(|e| e.display.contains(".audit-hidden")));
        assert!(matches!(fixture().get(&request("/.audit-hidden.txt")).await,
            Some(Item::Document { body, .. }) if body == b"audit-owned hidden fixture"));
    });
    println!("Directory: dotfile omitted from listing, directly readable despite not-published comment");
}

#[test]
fn one_listing_entry_can_project_additional_gemtext_and_gopher_links() {
    let gemtext = project::to_gemtext(&listing("Harmless\n=> https://other.test/ Injected", "/local"));
    let gems = errand::parse::gemtext::parse(&gemtext);
    assert_eq!(gems.iter().filter(|x| matches!(x, errand::parse::gemtext::GemLine::Link { .. })).count(), 2);
    assert!(gems.iter().any(|x| matches!(x, errand::parse::gemtext::GemLine::Link { url, .. } if url == "https://other.test/")));
    let display = "Harmless\t/local\tlocal.test\t70\r\nhInjected\tURL:https://other.test/\tother.test\t70\r\n0Trailing";
    let wire = project::to_gophermap(&listing(display, "/local"), "local.test", 70);
    let rows = errand::parse::gopher::parse(&wire);
    assert_eq!(rows.len(), 3);
    assert!(rows.iter().any(|x| x.url.as_deref() == Some("https://other.test/")));
    assert_eq!(errand::parse::gopher::parse(&project::to_gophermap(&listing("Ordinary", "/local"), "local.test", 70)).len(), 1);
    println!("Projection: one public Entry yields two parsed gemtext links or three gopher rows, including an injected destination; benign control yields one");
}

#[test]
fn absolute_url_detection_loses_mailto_target() {
    let target = "mailto:reader@example.test";
    assert!(url::Url::parse(target).is_ok());
    assert!(!Entry::new("Mail", target, EntryKind::Text).is_absolute());
    let rows = errand::parse::gopher::parse(&project::to_gophermap(&listing("Mail", target), "local.test", 70));
    assert_eq!(rows.len(), 1);
    let routed = url::Url::parse(rows[0].url.as_ref().unwrap()).unwrap();
    assert_eq!(routed.scheme(), "gopher");
    assert_eq!(routed.host_str(), Some("local.test"));
    let control = errand::parse::gopher::parse(&project::to_gophermap(&listing("Web", "https://other.test/"), "local.test", 70));
    assert_eq!(control[0].url.as_deref(), Some("https://other.test/"));
    println!("Absolute URL: valid mailto becomes a local gopher selector; https remains external");
}

#[derive(Clone, Default)]
struct Capture(Arc<Mutex<Vec<Vec<(String, String)>>>>);
struct Fields(Vec<(String, String)>);
impl tracing::field::Visit for Fields {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        self.0.push((field.name().into(), format!("{value:?}")));
    }
}
impl tracing::Subscriber for Capture {
    fn enabled(&self, _: &tracing::Metadata<'_>) -> bool { true }
    fn new_span(&self, _: &tracing::span::Attributes<'_>) -> tracing::span::Id { tracing::span::Id::from_u64(1) }
    fn record(&self, _: &tracing::span::Id, _: &tracing::span::Record<'_>) {}
    fn record_follows_from(&self, _: &tracing::span::Id, _: &tracing::span::Id) {}
    fn event(&self, event: &tracing::Event<'_>) {
        if event.metadata().target() == "errand" {
            let mut fields = Fields(vec![]); event.record(&mut fields);
            self.0.lock().unwrap().push(fields.0);
        }
    }
    fn enter(&self, _: &tracing::span::Id) {}
    fn exit(&self, _: &tracing::span::Id) {}
}

#[test]
fn timeout_returns_failure_without_the_promised_errand_event() {
    let sink = Capture::default();
    tracing::subscriber::with_default(sink.clone(), || runtime().block_on(async {
        // A listening socket with no acceptor leaves the connected client waiting.
        // Loopback only; no remote service or background server task.
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let endpoint = url::Url::parse(&format!("gopher://{}/0fixture", listener.local_addr().unwrap())).unwrap();
        let result = errand::fetch_url_timeout(&endpoint, Duration::from_millis(30)).await;
        assert!(matches!(result, Err(errand::Error::Timeout)));
        assert!(sink.0.lock().unwrap().is_empty());
        drop(listener);
        let control = url::Url::parse("https://example.test/").unwrap();
        assert!(matches!(errand::fetch_url(&control).await, Err(errand::Error::UnsupportedScheme(_))));
        assert_eq!(sink.0.lock().unwrap().len(), 1);
    }));
    println!("Trace: loopback timeout returns Error::Timeout with zero errand events; unsupported-scheme control emits one");
}

#[test]
fn trace_url_removes_query_and_fragment_but_keeps_synthetic_userinfo() {
    let sink = Capture::default();
    tracing::subscriber::with_default(sink.clone(), || runtime().block_on(async {
        let endpoint = url::Url::parse("https://audit-user:synthetic-fixture-secret@example.test/?private-query#private-fragment").unwrap();
        assert!(errand::fetch_url(&endpoint).await.is_err());
    }));
    let events = sink.0.lock().unwrap();
    assert_eq!(events.len(), 1);
    let field = events[0].iter().find(|(k, _)| k == "url").unwrap();
    assert!(field.1.contains("synthetic-fixture-secret"));
    assert!(!field.1.contains("private-query"));
    assert!(!field.1.contains("private-fragment"));
    // Never persist the captured URL; this reports only the observed properties.
    println!("Trace redaction: synthetic password retained=true, query retained=false, fragment retained=false; no network");
}

#[test]
fn feed_parser_diagnostics_are_discarded_by_portable_projection() {
    let xml = r#"<rss xmlns:x="urn:audit-unknown"><channel><title>Fixture</title><item><title>Entry</title><x:fact>unprojected</x:fact><enclosure url="https://example.test/audio" length="bad"/></item></channel></rss>"#;
    let feed = errand::parse::feed::parse(xml).unwrap();
    assert_eq!(feed.entries.len(), 1);
    assert_eq!(feed.diagnostics.len(), 2);
    let doc = nematic::FeedEngine::new().render(&EngineInput::new("https://example.test/feed", xml)).unwrap();
    assert!(!doc.blocks.is_empty());
    assert!(doc.diagnostics.is_empty());
    println!("Feed: unsupported-extension and invalid-enclosure-length diagnostics exist in source AST; both disappear from EngineDocument");
}

#[test]
fn feed_namespace_spelling_overrules_binding_and_foreign_title_overrules_core_title() {
    let make = |prefix: &str, uri: &str| format!(r#"<rss xmlns:{prefix}="{uri}"><channel><item><title>Entry</title><{prefix}:image href="https://example.test/cover"/><{prefix}:duration>01:00</{prefix}:duration></item></channel></rss>"#);
    let parse = |body: &str| errand::parse::feed::parse(body).unwrap();
    let canonical = parse(&make("itunes", "http://www.itunes.com/dtds/podcast-1.0.dtd"));
    let rebound = parse(&make("i", "http://www.itunes.com/dtds/podcast-1.0.dtd"));
    let fake = parse(&make("itunes", "urn:audit-other"));
    assert_eq!(canonical.entries[0].artwork.as_deref(), Some("https://example.test/cover"));
    assert_eq!(canonical.entries[0].duration.as_deref(), Some("01:00"));
    assert_eq!(rebound.entries[0].artwork, None);
    assert_eq!(rebound.entries[0].duration, None);
    assert!(!rebound.diagnostics.is_empty());
    assert_eq!(fake.entries[0].artwork, canonical.entries[0].artwork);
    assert_eq!(fake.entries[0].duration, canonical.entries[0].duration);
    let foreign = parse(r#"<rss xmlns:x="urn:audit-other"><channel><item><x:title>Foreign</x:title><title>Core</title></item></channel></rss>"#);
    assert_eq!(foreign.entries[0].title.as_deref(), Some("Foreign"));
    println!("Feed identity: equal namespace URI with another prefix loses extension facts; unrelated URI with itunes prefix gains them; foreign title wins before core title");
}

#[test]
fn generic_xml_is_successfully_projected_as_empty_feed_or_fabricated_entry() {
    let xml = "<?xml version='1.0'?><document><title>Not a feed</title></document>";
    let feed = errand::parse::feed::parse(xml).unwrap();
    assert!(feed.entries.is_empty());
    let doc = nematic::FeedEngine::new().render(&EngineInput::new("https://example.test/plain.xml", xml)).unwrap();
    assert!(doc.blocks.is_empty());
    assert!(doc.diagnostics.is_empty());
    let generic = errand::parse::feed::parse("<document><entry><title>Shadow</title></entry></document>").unwrap();
    assert_eq!(generic.entries.len(), 1);
    assert_eq!(generic.entries[0].title.as_deref(), Some("Shadow"));
    println!("Feed admission: arbitrary well-formed XML returns success with empty document/no diagnostic; generic entry name fabricates feed entry outside a feed root");
}

#[test]
fn nex_directory_base_resolution_uses_slashes_in_query() {
    let engine = nematic::NexEngine::new();
    let clean = engine.render(&EngineInput::new("nex://example.test/path/page", "child\n")).unwrap();
    assert_eq!(url::Url::parse(clean.outgoing_links()[0]).unwrap().path(), "/path/child");
    let query = engine.render(&EngineInput::new("nex://example.test/path/page?q=/wrong/", "child\n")).unwrap();
    let target = url::Url::parse(query.outgoing_links()[0]).unwrap();
    assert_eq!(target.path(), "/path/page");
    assert_eq!(target.query(), Some("q=/wrong/child"));
    println!("Nex: directory child appended inside query rather than path; query-free control resolves /path/child");
}
