use inker::*;
use inker::flip::api::{BackState, Carry, FlipBack, FlipDonor, FlipReceiver, LayerSet, PortableViewState};
use inker::flip::{orchestrator, scry::{NavSignal, ScryForward, ScrySurface}};

fn doc(blocks: Vec<Block>) -> EngineDocument {
    EngineDocument { address: "audit:fixture".into(), title: None, content_type: "text/x-knot".into(),
        lang: None, provenance: Default::default(), trust: Default::default(), diagnostics: vec![],
        navigation: Default::default(), blocks }
}
fn paragraph(spans: Vec<InlineSpan>) -> Block { Block::Paragraph { spans } }
fn text(s: &str) -> InlineSpan { InlineSpan::Text(s.into()) }
fn link(url: &str) -> Block { paragraph(vec![InlineSpan::Link { url: url.into(), title: None,
    predicate: None, spans: vec![text("visit")] }]) }
fn request(address: &str) -> EngineRouteRequest {
    EngineRouteRequest { workspace_id: WorkspaceRouteId::new("audit"), view: None, node: None,
        address: address.into(), content_type: None, pinned_engine: None }
}

#[test]
fn same_length_replacement_passes_navigation_currentness() {
    let mut a = doc(vec![Block::Heading { level: 1, spans: vec![text("old heading")] }, paragraph(vec![text("old body")])]);
    a.navigation = DocumentNavigation { block_count: 2,
        anchors: vec![DocumentAnchor { name: "old".into(), block: Some(0), source_line: 0, active: true }],
        folds: vec![DocumentFold { heading: 0, source_line: 0, source_text: "old heading".into(),
            initially_open: false, extent: 1..2 }] };
    let replacement = vec![Block::Rule, paragraph(vec![text("unrelated body")])];
    assert_ne!(a.blocks, replacement);
    assert!(a.navigation.is_current(&replacement));
    assert_eq!(a.navigation.resolve("old"), Some(0));
    assert!(!a.navigation.is_current(&[Block::Rule]));
    println!("navigation: unrelated same-length replacement is_current=true; length-change control=false");
}

#[test]
fn inline_and_frontmatter_exports_have_collisions() {
    let literal = doc(vec![paragraph(vec![text("*x*")])]);
    let emphasis = doc(vec![paragraph(vec![InlineSpan::Emphasis(vec![text("x")])])]);
    assert_ne!(literal.blocks, emphasis.blocks);
    assert_eq!(literal.to_markdown(), emphasis.to_markdown());
    assert_eq!(literal.to_knot(), emphasis.to_knot());
    assert_ne!(literal.to_html(), emphasis.to_html());
    let mut a = doc(vec![]); a.title = Some("Alpha\nsource: Beta".into());
    let mut b = doc(vec![]); b.title = Some("Alpha".into()); b.provenance.canonical_uri = Some("Beta".into());
    assert_ne!(a, b); assert_eq!(a.to_knot(), b.to_knot());
    println!("exports: distinct literal/emphasis blocks collide in Markdown/Knot; HTML distinguishes; distinct title/source metadata also collides");
}

fn feed(guid: &str) -> Block {
    Block::FeedEntry { title: "Entry".into(), date: None, summary: None, article_url: None,
        source_url: None, published: None, updated: None, guid: Some(guid.into()), enclosures: vec![], content_address: None }
}
#[test]
fn knot_export_drops_feed_identity() {
    let a = doc(vec![feed("first")]); let b = doc(vec![feed("second")]);
    assert_ne!(a.blocks, b.blocks); assert_eq!(a.to_knot(), b.to_knot());
    let c = doc(vec![Block::Badge { text: "changed".into() }]);
    assert_ne!(a.to_knot(), c.to_knot());
    println!("Knot: different FeedEntry guid values serialize identically; changed-block control differs");
}

#[test]
fn ipv6_is_misparsed_by_both_local_authority_splitters() {
    let ctx = GophermapContext { host: "export.test".into(), port: 70 };
    let got = doc(vec![link("gopher://[::1]:7070/1/notes")]).to_gophermap(&ctx);
    assert!(got.contains("\t/notes\t[\t70\r\n"), "{got:?}");
    let control = doc(vec![link("gopher://example.test:7070/1/notes")]).to_gophermap(&ctx);
    assert!(control.contains("\t/notes\texample.test\t7070\r\n"));
    assert_eq!(inker::routing::host_from_address("https://[::1]:7070/"), Some("["));
    let mut policy = EngineRoutePolicy::default();
    policy.per_host_overrides.insert("[::1]".into(), "audit.ipv6".into());
    assert_ne!(policy.route(&request("https://[::1]:7070/")).engine_id, "audit.ipv6");
    policy.per_host_overrides.insert("example.test".into(), "audit.hostname".into());
    assert_eq!(policy.route(&request("https://example.test:7070/")).engine_id, "audit.hostname");
    println!("IPv6: gopher exported host=[ port=70 instead of ::1/7070; host override also misses; DNS-host controls pass");
}

#[test]
fn filtered_route_returns_an_engine_rejected_by_filter() {
    let decision = EngineRoutePolicy::default().route_filtered(&request("https://example.test/"), |_| false);
    assert_eq!(decision.engine_id, inker::routing::ENGINE_EXTERNAL_PROTOCOL);
    assert!(!EngineRegistry::new().contains(&decision.engine_id));
    let control = EngineRoutePolicy::default().route_filtered(&request("https://example.test/"), |id| id == inker::routing::ENGINE_GENET_WEB);
    assert_eq!(control.engine_id, inker::routing::ENGINE_GENET_WEB);
    println!("route_filtered: all engines unavailable still yields external.protocol; available Genet control selects Genet");
}

#[derive(Default)]
struct Secondary { state: PortableViewState, forward_receipts: usize }
impl FlipBack for Secondary { fn extract(&self) -> BackState { BackState::default() } }
impl FlipDonor for Secondary {
    fn donates(&self) -> LayerSet { LayerSet::all() }
    fn capture(&self) -> PortableViewState { self.state.clone() }
}
impl FlipReceiver for Secondary {
    fn receives(&self) -> LayerSet { LayerSet::all() }
    fn present(&mut self, carry: Carry) { if let Carry::Forward(s) = carry { self.state = s; self.forward_receipts += 1; } }
}
#[test]
fn public_flip_traits_allow_secondary_chaining() {
    let a = Secondary { state: PortableViewState { url: Some("https://example.test/".into()), ..Default::default() }, ..Default::default() };
    let mut b = Secondary::default(); let mut c = Secondary::default();
    orchestrator::flip_forward(&a, &mut b); orchestrator::flip_forward(&b, &mut c);
    assert_eq!(b.forward_receipts, 1); assert_eq!(c.forward_receipts, 1); assert_eq!(c.state.url, a.state.url);
    println!("flip: a type implementing FlipBack + FlipDonor + FlipReceiver compiles and forwards twice; built-in ScryForward is not being claimed to implement FlipDonor");
}

#[derive(Default)]
struct FailingSurface { cookies: usize, scripts: usize, navigations: usize }
impl ScrySurface for FailingSurface {
    fn set_cookie(&mut self, _: &inker::flip::api::Cookie) -> Result<(), String> { self.cookies += 1; Err("audit cookie refusal".into()) }
    fn navigate(&mut self, _: &str) -> Result<(), String> { self.navigations += 1; Ok(()) }
    fn run_script(&mut self, _: &str) -> Result<String, String> { self.scripts += 1; Err("audit script refusal".into()) }
}
#[test]
fn scry_exposes_no_structured_cookie_or_restore_failure() {
    let mut surface = FailingSurface::default();
    let mut flip = ScryForward::new(PortableViewState { url: Some("https://example.test/".into()), scroll: (0.0, 9.0),
        cookies: vec![inker::flip::api::Cookie::default()], ..Default::default() });
    assert!(flip.begin(&mut surface).is_ok());
    flip.on_nav(NavSignal::Completed { success: true }, &mut surface);
    assert!(flip.is_done()); assert_eq!((surface.cookies, surface.scripts, surface.navigations), (1, 1, 1));
    println!("Scry: cookie refusal -> begin Ok; restore refusal -> terminal is_done; debug warnings are not a host result API");
}

#[test]
fn xml_feed_sniff_does_not_reach_default_feed_route() {
    let without = b"<feed xmlns=\"http://www.w3.org/2005/Atom\"/>";
    let with = b"<?xml version=\"1.0\"?>\n<feed xmlns=\"http://www.w3.org/2005/Atom\"/>";
    let mut a = request("https://example.test/feed"); a.content_type = sniff_content_type(without).map(str::to_owned);
    let mut b = a.clone(); b.content_type = sniff_content_type(with).map(str::to_owned);
    assert_eq!(a.content_type.as_deref(), Some("application/atom+xml"));
    assert_eq!(b.content_type.as_deref(), Some("application/xml"));
    let policy = EngineRoutePolicy::default();
    assert_eq!(policy.route(&a).engine_id, inker::routing::ENGINE_NEMATIC_FEED);
    assert_eq!(policy.route(&b).engine_id, inker::routing::ENGINE_GENET_WEB);
    println!("sniff+route: Atom without XML declaration -> feed engine; same root with declaration -> Genet over HTTPS; no host fetch executed");
}

#[test]
fn svg_sniff_reads_beyond_its_claimed_window() {
    let mut a = vec![b' '; 65536]; let mut b = a.clone();
    a.extend_from_slice(b"<svg/>"); b.extend_from_slice(b"<html/>");
    assert_eq!(&a[..1024], &b[..1024]);
    assert_eq!(sniff_content_type(&a), Some("image/svg+xml"));
    assert_eq!(sniff_content_type(&b), Some("text/plain"));
    println!("sniff bound: identical 1024-byte heads differ on SVG marker at 65536; HTML path remains windowed");
}
