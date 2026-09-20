//! R3-A: a ranged read through real netfetcher on a persona-scoped context,
//! called the way a decoder calls it: blocking, from its own thread.
//! This file is the regression manifest. Nothing here is a proposed API.

#![cfg(test)]

use std::{
    io::{Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread,
    time::Duration,
};

use netfetcher::{
    CacheMode, CookieRecord, CookieStore, Destination, FetchContext, InMemoryCookieJar,
    InMemoryHttpCache, Request, Response, SameSiteContext,
};
use url::Url;

const OBJECT_BYTES: usize = 3_000_000;
const CHUNK: u64 = 64 * 1024;

fn object() -> Vec<u8> {
    (0..OBJECT_BYTES).map(|index| (index % 251) as u8).collect()
}

// --- the server ----------------------------------------------------------------

#[derive(Clone, Debug)]
struct Seen {
    path: String,
    headers: Vec<(String, String)>,
}

impl Seen {
    fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.as_str())
    }
}

struct Server {
    address: SocketAddr,
    log: Arc<Mutex<Vec<Seen>>>,
    etag: Arc<Mutex<String>>,
    release: mpsc::Sender<()>,
    stop: Arc<AtomicBool>,
    thread: Option<thread::JoinHandle<()>>,
}

impl Server {
    fn start() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let address = listener.local_addr().unwrap();
        let log = Arc::new(Mutex::new(Vec::new()));
        let etag = Arc::new(Mutex::new("\"v1\"".to_owned()));
        let (release, held) = mpsc::channel();
        let held = Arc::new(Mutex::new(held));
        let stop = Arc::new(AtomicBool::new(false));
        let data = Arc::new(object());
        let thread = {
            let (log, etag, stop) = (log.clone(), etag.clone(), stop.clone());
            thread::spawn(move || {
                while !stop.load(Ordering::Relaxed) {
                    match listener.accept() {
                        Ok((stream, _)) => {
                            stream.set_nonblocking(false).unwrap();
                            let (log, etag, data, held) =
                                (log.clone(), etag.clone(), data.clone(), held.clone());
                            thread::spawn(move || serve(stream, &data, &log, &etag, &held));
                        },
                        Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                            thread::sleep(Duration::from_millis(2));
                        },
                        Err(error) => panic!("probe server failed: {error}"),
                    }
                }
            })
        };
        Self {
            address,
            log,
            etag,
            release,
            stop,
            thread: Some(thread),
        }
    }

    fn url(&self, path: &str) -> Url {
        Url::parse(&format!("http://{}{path}", self.address)).unwrap()
    }

    fn seen(&self, path: &str) -> Vec<Seen> {
        self.log
            .lock()
            .unwrap()
            .iter()
            .filter(|seen| seen.path == path)
            .cloned()
            .collect()
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        let _ = TcpStream::connect(self.address);
        if let Some(thread) = self.thread.take() {
            thread.join().unwrap();
        }
    }
}

fn serve(
    mut stream: TcpStream,
    data: &[u8],
    log: &Mutex<Vec<Seen>>,
    etag: &Mutex<String>,
    held: &Mutex<mpsc::Receiver<()>>,
) {
    let mut raw = vec![0_u8; 8192];
    let count = stream.read(&mut raw).unwrap_or(0);
    let text = String::from_utf8_lossy(&raw[..count]).into_owned();
    let mut lines = text.lines();
    let path = lines
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .unwrap_or("/")
        .to_owned();
    let headers: Vec<(String, String)> = lines
        .take_while(|line| !line.is_empty())
        .filter_map(|line| line.split_once(':'))
        .map(|(key, value)| (key.trim().to_owned(), value.trim().to_owned()))
        .collect();
    let seen = Seen {
        path: path.clone(),
        headers,
    };
    log.lock().unwrap().push(seen.clone());
    let etag = etag.lock().unwrap().clone();

    match path.as_str() {
        "/page" => {
            let _ = stream.write_all(
                b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nContent-Type: text/plain\r\nSet-Cookie: session=persona-a; Path=/\r\nConnection: close\r\n\r\nok",
            );
        },
        "/moved" => {
            let _ = stream.write_all(
                b"HTTP/1.1 302 Found\r\nLocation: /episode.mp3\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
            );
        },
        "/slow" => {
            let half = data.len() / 2;
            let _ = write!(
                stream,
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: audio/mpeg\r\nConnection: close\r\n\r\n",
                data.len()
            );
            let _ = stream.write_all(&data[..half]);
            let _ = stream.flush();
            let _ = held.lock().unwrap().recv_timeout(Duration::from_secs(20));
            let _ = stream.write_all(&data[half..]);
        },
        _ => {
            let range = seen
                .header("range")
                .and_then(|value| value.strip_prefix("bytes="))
                .and_then(|value| value.split_once('-'))
                .map(|(start, end)| (start.parse::<usize>().unwrap(), end.parse::<usize>().unwrap()));
            let validator_holds = seen.header("if-range").is_none_or(|value| value == etag);
            match range {
                Some((start, end)) if validator_holds => {
                    let end = end.min(data.len() - 1);
                    let body = &data[start..=end];
                    let _ = write!(
                        stream,
                        "HTTP/1.1 206 Partial Content\r\nContent-Length: {}\r\nContent-Range: bytes {start}-{end}/{}\r\nContent-Type: audio/mpeg\r\nETag: {etag}\r\nConnection: close\r\n\r\n",
                        body.len(),
                        data.len()
                    );
                    let _ = stream.write_all(body);
                },
                _ => {
                    let _ = write!(
                        stream,
                        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: audio/mpeg\r\nCache-Control: max-age=3600\r\nETag: {etag}\r\nConnection: close\r\n\r\n",
                        data.len()
                    );
                    let _ = stream.write_all(data);
                },
            }
        },
    }
}

// --- the host side ---------------------------------------------------------------

/// One persona's jar, shared by every context built for that persona: the
/// shape `mere-fetch`'s `SharedJar` already uses.
struct PersonaJar(Arc<InMemoryCookieJar>);

impl CookieStore for PersonaJar {
    fn cookies_for(&self, url: &Url, ctx: SameSiteContext) -> Vec<String> {
        self.0.cookies_for(url, ctx)
    }
    fn records_for(&self, url: &Url, ctx: SameSiteContext) -> Vec<CookieRecord> {
        self.0.records_for(url, ctx)
    }
    fn set_cookie(&self, url: &Url, header: &str) {
        self.0.set_cookie(url, header);
    }
}

fn persona_context(jar: &Arc<InMemoryCookieJar>, cache: &Arc<InMemoryHttpCache>) -> FetchContext {
    let mut context = FetchContext::permissive();
    context.cookies = Box::new(PersonaJar(jar.clone()));
    context.cache = cache.clone();
    context
}

fn runtime() -> Arc<tokio::runtime::Runtime> {
    Arc::new(
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .unwrap(),
    )
}

fn ranged(url: Url, start: u64, end: u64, cache: CacheMode) -> Request {
    let mut request = Request::get(url).with_destination(Destination::Audio);
    request
        .headers
        .push(("range".to_owned(), format!("bytes={start}-{end}")));
    request.cache = cache;
    request
}

fn header<'a>(response: &'a Response, name: &str) -> Option<&'a str> {
    response
        .headers
        .iter()
        .find(|(key, _)| key.eq_ignore_ascii_case(name))
        .map(|(_, value)| value.as_str())
}

// --- the manifest ----------------------------------------------------------------

#[test]
fn a_blocking_thread_reads_exact_ranges_with_identity_encoding() {
    let server = Server::start();
    let runtime = runtime();
    let context = Arc::new(persona_context(&Default::default(), &Default::default()));
    let data = object();
    let url = server.url("/episode.mp3");

    // The decoder's shape: its own thread, one blocking call per range.
    let fetched = thread::spawn({
        let (runtime, context, url) = (runtime.clone(), context.clone(), url.clone());
        move || {
            let mut total = 0_u64;
            for start in [0_u64, 65_536, 983_040, 2_490_368] {
                let end = if start == 0 { 0 } else { start + CHUNK - 1 };
                let response = runtime.block_on(netfetcher::fetch(
                    ranged(url.clone(), start, end, CacheMode::NoStore),
                    &context,
                ));
                assert_eq!(response.status, 206, "range {start}-{end}");
                assert_eq!(
                    header(&response, "content-range").map(str::to_owned),
                    Some(format!("bytes {start}-{end}/{OBJECT_BYTES}"))
                );
                let bytes = runtime.block_on(response.bytes()).unwrap();
                assert_eq!(&bytes[..], &data[start as usize..=end as usize]);
                total += bytes.len() as u64;
            }
            total
        }
    })
    .join()
    .unwrap();

    let seen = server.seen("/episode.mp3");
    assert_eq!(seen.len(), 4);
    assert!(fetched * 10 < OBJECT_BYTES as u64, "fetched {fetched}");
    for request in &seen {
        assert_eq!(request.header("accept-encoding"), Some("identity"));
        assert!(request.header("range").is_some());
    }
    println!("R3-A ranges: 4 requests, {fetched} of {OBJECT_BYTES} bytes, http audio with no initiator not upgraded");
}

#[test]
fn a_changed_representation_is_visible_through_if_range() {
    let server = Server::start();
    let runtime = runtime();
    let context = persona_context(&Default::default(), &Default::default());
    let url = server.url("/episode.mp3");

    let probe = runtime.block_on(netfetcher::fetch(
        ranged(url.clone(), 0, 0, CacheMode::NoStore),
        &context,
    ));
    let validator = header(&probe, "etag").unwrap().to_owned();
    *server.etag.lock().unwrap() = "\"v2\"".to_owned();

    let mut request = ranged(url, 65_536, 131_071, CacheMode::NoStore);
    request.headers.push(("if-range".to_owned(), validator));
    let response = runtime.block_on(netfetcher::fetch(request, &context));
    // The caller sees the server's refusal to honour the range: a whole 200.
    assert_eq!(response.status, 200);
    assert_eq!(header(&response, "etag"), Some("\"v2\""));
    assert_eq!(server.seen("/episode.mp3")[1].header("if-range"), Some("\"v1\""));
}

#[test]
fn a_redirect_keeps_the_range_and_reports_the_final_url() {
    let server = Server::start();
    let runtime = runtime();
    let context = persona_context(&Default::default(), &Default::default());

    let response = runtime.block_on(netfetcher::fetch(
        ranged(server.url("/moved"), 65_536, 131_071, CacheMode::NoStore),
        &context,
    ));
    assert_eq!(response.status, 206);
    assert_eq!(
        response.url_list.last().map(Url::path),
        Some("/episode.mp3")
    );
    let landed = server.seen("/episode.mp3");
    assert_eq!(landed.len(), 1);
    assert_eq!(landed[0].header("range"), Some("bytes=65536-131071"));
    assert_eq!(landed[0].header("accept-encoding"), Some("identity"));
}

#[test]
fn a_ranged_read_carries_its_own_personas_cookie_and_no_other() {
    let server = Server::start();
    let runtime = runtime();
    let (jar_a, jar_b) = (Arc::new(InMemoryCookieJar::new()), Arc::new(InMemoryCookieJar::new()));
    let context_a = persona_context(&jar_a, &Default::default());
    let context_b = persona_context(&jar_b, &Default::default());
    let url = server.url("/episode.mp3");

    // Persona A logs in on a page; the context that fetched it is discarded.
    let page = runtime.block_on(netfetcher::fetch(Request::get(server.url("/page")), &context_a));
    assert_eq!(page.status, 200);
    drop(context_a);
    assert_eq!(jar_a.len(), 1);

    // A later context over the same jar, as a media reader would build.
    let reader_a = persona_context(&jar_a, &Default::default());
    runtime.block_on(netfetcher::fetch(ranged(url.clone(), 0, 0, CacheMode::NoStore), &reader_a));
    runtime.block_on(netfetcher::fetch(ranged(url, 0, 0, CacheMode::NoStore), &context_b));

    let seen = server.seen("/episode.mp3");
    assert_eq!(seen[0].header("cookie"), Some("session=persona-a"));
    assert_eq!(seen[1].header("cookie"), None);
    assert!(jar_b.is_empty());
}

#[test]
fn a_cached_whole_object_and_a_ranged_request() {
    let server = Server::start();
    let runtime = runtime();
    let cache = Arc::new(InMemoryHttpCache::new());
    let context = persona_context(&Default::default(), &cache);
    let url = server.url("/episode.mp3");

    // Positive control: the whole object caches, and a second whole GET is a hit.
    for _ in 0..2 {
        let whole = runtime.block_on(netfetcher::fetch(Request::get(url.clone()), &context));
        assert_eq!(whole.status, 200);
        assert_eq!(runtime.block_on(whole.bytes()).unwrap().len(), OBJECT_BYTES);
    }
    assert_eq!(server.seen("/episode.mp3").len(), 1, "second whole GET must be a cache hit");

    // The guarded request: no-store reaches the network and gets its range.
    let guarded = runtime.block_on(netfetcher::fetch(
        ranged(url.clone(), 65_536, 131_071, CacheMode::NoStore),
        &context,
    ));
    assert_eq!(guarded.status, 206);
    assert_eq!(runtime.block_on(guarded.bytes()).unwrap().len(), CHUNK as usize);
    assert_eq!(server.seen("/episode.mp3").len(), 2);

    // The unguarded request: default cache mode. Observed, then pinned below.
    let unguarded = runtime.block_on(netfetcher::fetch(
        ranged(url, 65_536, 131_071, CacheMode::Default),
        &context,
    ));
    let status = unguarded.status;
    let length = runtime.block_on(unguarded.bytes()).unwrap().len();
    let reached_network = server.seen("/episode.mp3").len() == 3;
    println!(
        "R3-A cache: default-mode ranged request after a cached whole object -> status {status}, {length} bytes, reached network: {reached_network}"
    );
    assert_eq!(
        (status, length, reached_network),
        EXPECTED_UNGUARDED,
        "the unguarded outcome moved; re-read the cache guard requirement"
    );
}

/// What netfetcher 5ae30cad does with a default-mode ranged request once the
/// whole object is cached: a cache hit that ignores `Range` and answers 200 with
/// the entire body, without touching the network. Pinned from observation
/// (2026-09-20), so a change is noticed. Until netfetcher answers such a request
/// with the stored range or bypasses the cache, a ranged caller sends no-store.
const EXPECTED_UNGUARDED: (u16, usize, bool) = (200, 3_000_000, false);

#[test]
fn a_whole_body_streams_before_the_server_finishes() {
    let server = Server::start();
    let runtime = runtime();
    let context = persona_context(&Default::default(), &Default::default());
    let mut request = Request::get(server.url("/slow")).with_destination(Destination::Audio);
    request.cache = CacheMode::NoStore;

    let mut response = runtime.block_on(netfetcher::fetch(request, &context));
    assert_eq!(response.status, 200);
    // The server is holding the second half until released.
    let first = runtime
        .block_on(async {
            tokio::time::timeout(Duration::from_secs(5), response.body.next_chunk()).await
        })
        .expect("a chunk must arrive while the server still holds half the body")
        .unwrap()
        .unwrap();
    assert!(!first.is_empty() && first.len() <= OBJECT_BYTES / 2);

    server.release.send(()).unwrap();
    let mut total = first.len();
    let mut peak = first.len();
    while let Some(chunk) = runtime.block_on(response.body.next_chunk()) {
        let chunk = chunk.unwrap();
        peak = peak.max(chunk.len());
        total += chunk.len();
    }
    assert_eq!(total, OBJECT_BYTES);
    println!("R3-A stream: first chunk {} bytes before release, largest chunk {peak} bytes, nothing held whole", first.len());
}
