//! Audit-only, current-library observations. All cookies and bodies are synthetic.
use fetch::{Fetch, FetchError, NetFetch, Range, Stores};
use netfetcher::{CookieStore, RawResponse, SameSiteContext, Transport, TransportFuture, WireRequest};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}, mpsc};
use std::time::{Duration, Instant};
use muniment::Backend;

fn runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap()
}
fn persona(n: u128) -> pandect::PersonaId { pandect::PersonaId(uuid::Uuid::from_u128(n)) }
fn prefix(n: u128) -> String { format!("cookies/{}/", uuid::Uuid::from_u128(n)) }

#[test]
fn cookie_flip_export_erases_host_only_scope() {
    let parent = url::Url::parse("https://scope-cookie-audit.test/").unwrap();
    let child = url::Url::parse("https://child.scope-cookie-audit.test/").unwrap();
    let record = netfetcher::CookieRecord {
        name: "audit_scope".into(), value: "synthetic".into(), domain: "scope-cookie-audit.test".into(),
        host_only: true, path: "/".into(), secure: true, http_only: true, same_site: None, expires: None,
    };
    fetch::session_jar().load_records([record.clone()]);
    assert!(fetch::session_jar().cookies_for(&child, SameSiteContext::same_site()).is_empty());
    let host_only = fetch::session_cookies_for(parent.as_str()); assert_eq!(host_only.len(), 1);
    let mut wider = record; wider.host_only = false; fetch::session_jar().load_records([wider]);
    assert_eq!(fetch::session_jar().cookies_for(&child, SameSiteContext::same_site()).len(), 1);
    let domain_cookie = fetch::session_cookies_for(parent.as_str());
    assert_eq!(host_only, domain_cookie);
    fetch::session_jar().set_cookie(&parent, "audit_scope=; Path=/; Max-Age=0");
    assert!(fetch::session_jar().cookies_for(&parent, SameSiteContext::same_site()).is_empty());
    println!("Flip cookie loss: source host-only and domain cookies have different child-host matching but identical exported portable cookies; no WebView import executed");
}

#[test]
fn cookie_persona_load_merges_and_persistence_shadow_is_global() {
    let url = url::Url::parse("https://persona-cookie-audit.test/").unwrap();
    fetch::session_jar().set_cookie(&url, "audit_session=synthetic-a; Path=/");
    fetch::mark_cookies_dirty();
    let mut store = muniment::MemoryBackend::default();
    fetch::persist_cookies(&mut store, persona(0xa001));
    assert_eq!(runtime().block_on(store.list(&prefix(0xa001))).unwrap().len(), 1);

    // The public loader promises an empty new persona starts with an empty jar.
    fetch::load_cookies(&mut store, persona(0xb001));
    assert_eq!(fetch::session_jar().cookies_for(&url, SameSiteContext::same_site()), ["audit_session=synthetic-a"]);
    fetch::mark_cookies_dirty();
    fetch::persist_cookies(&mut store, persona(0xb001));
    assert!(runtime().block_on(store.list(&prefix(0xb001))).unwrap().is_empty());
    println!("Persona state: empty persona B still sees A's session; marking dirty and persisting B skips the unchanged cookie because shadow is shared");

    // A later change to that inherited cookie is written into B's namespace.
    fetch::session_jar().set_cookie(&url, "audit_session=synthetic-a-changed; Path=/");
    fetch::mark_cookies_dirty();
    fetch::persist_cookies(&mut store, persona(0xb001));
    let keys = runtime().block_on(store.list(&prefix(0xb001))).unwrap();
    assert_eq!(keys.len(), 1);
    assert!(runtime().block_on(store.get(&keys[0])).unwrap().unwrap().windows(b"synthetic-a-changed".len()).any(|w| w == b"synthetic-a-changed"));
    println!("Persona state: a later inherited-cookie change lands under B's durable prefix; only in-memory Backend used");
}

#[test]
fn cookie_expiry_without_a_write_does_not_trigger_durable_deletion() {
    let url = url::Url::parse("https://expiry-cookie-audit.test/").unwrap();
    fetch::session_jar().set_cookie(&url, "audit_expiring=synthetic; Path=/; Max-Age=1");
    fetch::mark_cookies_dirty();
    let mut store = muniment::MemoryBackend::default();
    fetch::persist_cookies(&mut store, persona(0xe001));
    let keys = runtime().block_on(store.list(&prefix(0xe001))).unwrap();
    assert_eq!(keys.len(), 1);
    std::thread::sleep(Duration::from_millis(1500));
    assert!(fetch::session_jar().records_for(&url, SameSiteContext::same_site()).is_empty());
    fetch::persist_cookies(&mut store, persona(0xe001));
    assert_eq!(runtime().block_on(store.list(&prefix(0xe001))).unwrap(), keys);
    fetch::mark_cookies_dirty();
    fetch::persist_cookies(&mut store, persona(0xe001));
    assert!(runtime().block_on(store.list(&prefix(0xe001))).unwrap().is_empty());
    println!("Expiry: expired cookie no longer usable but persist retains its durable blob until an unrelated dirty mark; explicit mark deletes it");
}

struct ReplyWire {
    status: u16,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
    asked: Mutex<Vec<WireRequest>>,
}
impl Transport for ReplyWire {
    fn send(&self, request: WireRequest) -> TransportFuture<'_> {
        self.asked.lock().unwrap().push(request);
        let (status, headers, body) = (self.status, self.headers.clone(), self.body.clone());
        Box::pin(async move { Some(RawResponse::once(status, headers, body.into())) })
    }
}
fn wire(status: u16, headers: &[(&str, &str)], body: &[u8]) -> Arc<ReplyWire> {
    Arc::new(ReplyWire { status, headers: headers.iter().map(|(k,v)| (k.to_string(),v.to_string())).collect(), body: body.to_vec(), asked: Mutex::new(vec![]) })
}
fn handle(wire: Arc<ReplyWire>) -> NetFetch {
    let mut stores = Stores::in_memory(); stores.transport = Some(wire);
    NetFetch::new(&stores).unwrap().with_patience(Duration::from_secs(2))
}

#[test]
fn range_header_capacity_overflow_panics_before_body_read() {
    let control = handle(wire(206, &[("content-range", "bytes 0-0/1")], b"x"));
    assert_eq!(control.read_range("https://example.test/object", Range { start: 0, end: None }, None).unwrap().bytes, b"x");
    let attacker = handle(wire(206, &[("content-range", "bytes 0-18446744073709551614/18446744073709551615")], b""));
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        attacker.read_range("https://example.test/object", Range { start: 0, end: None }, None)
    }));
    assert!(result.is_err());
    println!("Range: admitted u64 Content-Range produces caught capacity-overflow panic instead of FetchError; no huge allocation or body transfer attempted");
}

const GZIP: &[u8] = &[31,139,8,0,0,0,0,0,2,10,75,44,77,201,44,209,77,73,77,206,79,73,77,209,77,202,79,169,4,0,5,52,31,79,18,0,0,0];
#[test]
fn identity_download_accepts_decoding_and_reports_encoded_length() {
    let wire = wire(200, &[("content-encoding", "gzip"), ("content-length", "38")], GZIP);
    let fetch = handle(wire.clone());
    let mut sink = Vec::new();
    let facts = fetch.read_into("https://example.test/object", &mut sink, Some(100)).unwrap();
    assert_eq!(sink, b"audit-decoded-body");
    assert_eq!(sink.len(), 18);
    assert_eq!(facts.content_length, Some(38));
    let asked = wire.asked.lock().unwrap();
    assert!(asked[0].headers.iter().any(|(k,v)| k.eq_ignore_ascii_case("accept-encoding") && v == "identity"));
    drop(asked);
    let refused = runtime().block_on(fetch::crawler::fetch_crawler_capped_with_transport("https://example.test/object", 100, wire.as_ref()));
    assert!(matches!(refused, Err(fetch::crawler::CrawlerFetchError::UnsupportedEncoding(ref e)) if e == "gzip"));
    println!("Identity download: sink receives18 decoded bytes while Facts declares38; crawler control refuses the same gzip response; no network");
}

type Log = Arc<Mutex<Vec<String>>>;
struct Server { address: std::net::SocketAddr, log: Log, stop: Arc<AtomicBool>, thread: Option<std::thread::JoinHandle<()>> }
impl Server {
    fn new(handler: impl Fn(&str) -> Vec<u8> + Send + 'static) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let address = listener.local_addr().unwrap();
        let log: Log = Arc::new(Mutex::new(vec![])); let stop = Arc::new(AtomicBool::new(false));
        let (records, halted) = (log.clone(), stop.clone());
        let thread = std::thread::spawn(move || {
            let deadline = Instant::now() + Duration::from_secs(10);
            while !halted.load(Ordering::Relaxed) && Instant::now() < deadline {
                match listener.accept() {
                    Ok((mut stream, _)) => {
                        stream.set_read_timeout(Some(Duration::from_secs(1))).unwrap();
                        let mut reader = BufReader::new(stream.try_clone().unwrap()); let mut request = String::new();
                        loop { let mut line = String::new(); if reader.read_line(&mut line).unwrap_or(0) == 0 { break; }
                            request.push_str(&line); if line == "\r\n" { break; }
                        }
                        if !request.is_empty() { records.lock().unwrap().push(request.clone()); let _ = stream.write_all(&handler(&request)); }
                    },
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => std::thread::sleep(Duration::from_millis(2)),
                    Err(e) => panic!("owned HTTP fixture: {e}"),
                }
            }
        });
        Self { address, log, stop, thread: Some(thread) }
    }
    fn url(&self, path: &str) -> String { format!("http://{}{path}", self.address) }
}
impl Drop for Server {
    fn drop(&mut self) { self.stop.store(true, Ordering::Relaxed); if let Some(t) = self.thread.take() { t.join().unwrap(); } }
}
fn ok(body: &[u8]) -> Vec<u8> {
    let mut bytes = format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", body.len()).into_bytes(); bytes.extend_from_slice(body); bytes
}

#[test]
fn https_only_blob_fetcher_accepts_plaintext_http_source() {
    use eidetic::BlobFetcher;
    let server = Server::new(|_| ok(b"plaintext-fixture"));
    let mut fetcher = fetch::https_blob::HttpsFetcher::new().with_max_response_bytes(32);
    let result = runtime().block_on(fetcher.fetch(&eidetic::BlobSource::Https { url: server.url("/blob") })).unwrap();
    assert_eq!(result.unwrap(), b"plaintext-fixture");
    assert_eq!(server.log.lock().unwrap().len(), 1);
    println!("HTTPS-only adapter: BlobSource::Https holding http:// loopback returns plaintext success; no TLS or remote service");
}

#[test]
fn fetchable_selector_refuses_uppercase_scheme_that_fetch_api_serves() {
    let server = Server::new(|_| ok(b"upper"));
    let lower = server.url("/body"); let upper = lower.replacen("http:", "HTTP:", 1);
    assert!(fetch::is_fetchable(&lower)); assert!(!fetch::is_fetchable(&upper));
    let got = runtime().block_on(fetch::fetch_page_capped(&upper, 16)).unwrap();
    assert_eq!(got.bytes, b"upper");
    println!("Scheme admission: is_fetchable rejects HTTP:// while actual current fetch_page_capped successfully fetches the same address");
}

#[test]
fn legacy_crawler_uses_browsing_cookies_and_follows_redirects() {
    let server = Server::new(|request| {
        if request.starts_with("GET /redirect ") { b"HTTP/1.1 302 Found\r\nLocation: /target\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".to_vec() }
        else { ok(b"ok") }
    });
    let url = server.url("/redirect");
    fetch::session_jar().set_cookie(&url::Url::parse(&url).unwrap(), "audit_crawler=synthetic; Path=/");
    let got = runtime().block_on(fetch::fetch_page_crawler(&url)).unwrap(); assert_eq!(got.bytes, b"ok");
    {
        let log = server.log.lock().unwrap(); assert_eq!(log.len(), 2);
        for request in log.iter() { assert!(request.to_ascii_lowercase().contains("cookie: audit_crawler=synthetic")); assert!(request.contains("merebot/0.1")); }
    }
    let control = runtime().block_on(fetch::crawler::fetch_crawler_capped(&url, 16)).unwrap();
    assert_eq!(control.status, 302); assert!(control.bytes.is_empty());
    let anonymous = runtime().block_on(fetch::fetch_page_anonymous_capped(&server.url("/anonymous"), 16)).unwrap();
    assert_eq!(anonymous.bytes, b"ok");
    let log = server.log.lock().unwrap(); assert_eq!(log.len(), 4);
    assert!(!log[2].to_ascii_lowercase().contains("cookie:")); assert!(!log[3].to_ascii_lowercase().contains("cookie:"));
    println!("Crawler split: legacy bot helper sends browser cookie on both followed hops; new crawler observes302 without cookie, and anonymous page control has no cookie");
}

#[test]
fn smolweb_body_cap_is_not_an_acquisition_stop() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap(); let address = listener.local_addr().unwrap();
    let (release, held) = mpsc::channel();
    let sent = Arc::new(AtomicBool::new(false)); let flushed = sent.clone();
    let serving = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap(); stream.set_read_timeout(Some(Duration::from_secs(1))).unwrap();
        let mut request = [0; 256]; let _ = stream.read(&mut request).unwrap();
        stream.write_all(b"eight!!!").unwrap(); stream.flush().unwrap();
        flushed.store(true, Ordering::Release);
        held.recv_timeout(Duration::from_secs(3)).unwrap();
    });
    runtime().block_on(async {
        let endpoint = format!("gopher://{address}/0fixture");
        let mut future = Box::pin(fetch::fetch_page_capped(&endpoint, 4));
        let first = tokio::time::timeout(Duration::from_millis(100), &mut future).await;
        let delayed = first.is_err();
        let sent_before_timeout = sent.load(Ordering::Acquire);
        release.send(()).unwrap();
        let result = match first { Ok(result) => result, Err(_) => tokio::time::timeout(Duration::from_secs(2), future).await.unwrap() };
        assert!(sent_before_timeout);
        assert!(delayed, "eight bytes did not terminate a four-byte-capped acquisition before EOF");
        assert!(result.unwrap_err().contains("4-byte cap"));
    });
    serving.join().unwrap();
    println!("Smolweb cap: eight received bytes under cap4 remain pending until EOF, then terminal refusal; no unbounded payload sent");
}

fn spartan_chain(redirects: usize) -> (Result<fetch::Fetched, String>, usize) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap(); let address = listener.local_addr().unwrap();
    let count = Arc::new(std::sync::atomic::AtomicUsize::new(0)); let seen = count.clone();
    let serving = std::thread::spawn(move || {
        for i in 0..(redirects + 1).min(5) {
            let (mut stream, _) = listener.accept().unwrap(); stream.set_read_timeout(Some(Duration::from_secs(1))).unwrap();
            let mut request = String::new(); BufReader::new(stream.try_clone().unwrap()).read_line(&mut request).unwrap();
            seen.fetch_add(1, Ordering::Relaxed);
            let response = if i < redirects { format!("3 spartan://{address}/{}\r\n", i + 1) } else { "2 text/plain\r\nok".into() };
            stream.write_all(response.as_bytes()).unwrap();
        }
    });
    let result = runtime().block_on(fetch::fetch_page_capped(&format!("spartan://{address}/0"), 16));
    serving.join().unwrap(); (result, count.load(Ordering::Relaxed))
}
#[test]
fn five_redirect_budget_actually_allows_only_four_redirects_to_a_response() {
    let (control, calls) = spartan_chain(4); assert_eq!(control.unwrap().bytes, b"ok"); assert_eq!(calls, 5);
    let (five, calls) = spartan_chain(5); assert_eq!(five.unwrap_err(), "too many redirects"); assert_eq!(calls, 5);
    println!("Smolweb redirect budget: four redirects plus final response succeed; fifth redirect is refused before its target request; constant is described as most redirects5");
}

#[test]
fn titan_page_with_matching_family_identity_is_refused_before_transport() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap(); listener.set_nonblocking(true).unwrap();
    let address = listener.local_addr().unwrap(); let endpoint = format!("titan://{address}/fixture");
    let identity = fetch::GeminiClientIdentity::new(&endpoint, vec![1], vec![2]).unwrap();
    let mut stores = Stores::in_memory(); stores.transport = Some(Arc::new(netfetcher::NoTransport));
    let (actor, updates) = fetch::spawn_fetcher_with(Arc::new(|| {}), stores);
    assert!(actor.command(fetch::FetchCommand::Page { request: 42, url: endpoint, identity: Some(identity) }));
    let update = updates.recv_timeout(Duration::from_secs(3)).unwrap();
    let fetch::FetchUpdate::Page(outcome) = update else { panic!("terminal page expected") };
    let failure = outcome.result.unwrap_err().to_string();
    assert!(failure.contains("unsupported scheme") && failure.contains("titan"));
    assert!(matches!(listener.accept(), Err(e) if e.kind() == std::io::ErrorKind::WouldBlock));
    drop(actor);
    println!("Titan identity: host accepts a matching Titan identity but generic page fetch calls Gemini-only identity API and refuses unsupported titan before any connection; dummy key material never used for TLS");
}
