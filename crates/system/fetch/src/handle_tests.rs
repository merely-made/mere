// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The handle against a real local server, called the way a decoder calls it.
//! Moved in from the R3-A probe
//! (`Code/testing/mere/design_docs/mere_docs/testing/receipts/2026-09-20_resource_resolution_probes`).

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

use netfetcher::{InMemoryCookieJar, RawResponse, Transport, TransportFuture, WireRequest};

use super::*;

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
        "/feed.xml" => {
            let _ = stream.write_all(
                b"HTTP/1.1 200 OK\r\nContent-Length: 11\r\nContent-Type: application/rss+xml; charset=utf-8\r\nConnection: close\r\n\r\n<rss></rss>",
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
                .map(|(start, end)| {
                    // An open end (`bytes=N-`) reads to the end of the object.
                    (
                        start.parse::<usize>().unwrap(),
                        end.parse::<usize>().unwrap_or(usize::MAX),
                    )
                });
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

// --- the manifest ----------------------------------------------------------------

fn handle(stores: &Stores) -> NetFetch {
    NetFetch::new(stores)
        .unwrap()
        .with_patience(Duration::from_secs(10))
}

fn span(start: u64) -> Range {
    Range {
        start,
        end: Some(start + CHUNK - 1),
    }
}

const FIRST_BYTE: Range = Range {
    start: 0,
    end: Some(0),
};

/// A sink that reports when it is first written to.
struct Watched {
    written: usize,
    first: Option<usize>,
    started: mpsc::Sender<()>,
}

impl Write for Watched {
    fn write(&mut self, chunk: &[u8]) -> std::io::Result<usize> {
        if self.first.is_none() {
            self.first = Some(chunk.len());
            let _ = self.started.send(());
        }
        self.written += chunk.len();
        Ok(chunk.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// A wire that answers every request itself and remembers what it was asked.
struct Recorded {
    asked: Mutex<Vec<WireRequest>>,
}

impl Transport for Recorded {
    fn send(&self, request: WireRequest) -> TransportFuture<'_> {
        let range = request
            .headers
            .iter()
            .find(|(name, _)| name == "range")
            .map(|(_, value)| value.clone());
        self.asked.lock().unwrap().push(request);
        Box::pin(async move {
            let (start, end) = range
                .as_deref()
                .and_then(|value| value.strip_prefix("bytes="))
                .and_then(|value| value.split_once('-'))
                .map(|(start, end)| (start.parse::<u64>().unwrap(), end.parse::<u64>().unwrap()))
                .unwrap_or((0, 0));
            let body: Vec<u8> = (start..=end).map(|index| (index % 251) as u8).collect();
            Some(RawResponse::once(
                206,
                vec![
                    (
                        "content-range".to_owned(),
                        format!("bytes {start}-{end}/{OBJECT_BYTES}"),
                    ),
                    ("content-length".to_owned(), body.len().to_string()),
                    ("content-type".to_owned(), "audio/mpeg".to_owned()),
                ],
                body.into(),
            ))
        })
    }
}

#[test]
fn a_supplied_transport_is_the_wire_the_handle_sends_through() {
    let wire = Arc::new(Recorded {
        asked: Mutex::new(Vec::new()),
    });
    let mut stores = Stores::in_memory();
    stores.transport = Some(wire.clone());
    let fetch = handle(&stores);

    // No server is listening here: only the supplied wire can answer.
    let reply = fetch
        .read_range("http://127.0.0.1:9/episode.mp3", span(65_536), None)
        .unwrap();
    assert_eq!(reply.start, 65_536);
    assert_eq!(reply.total, OBJECT_BYTES as u64);
    assert_eq!(&reply.bytes[..], &object()[65_536..65_536 + CHUNK as usize]);

    let asked = wire.asked.lock().unwrap();
    assert_eq!(asked.len(), 1);
    assert_eq!(asked[0].url.path(), "/episode.mp3");
    assert!(
        asked[0]
            .headers
            .iter()
            .any(|(name, value)| name == "range" && value == "bytes=65536-131071")
    );

    // Control: a wire with no network behind it, and the same address fails.
    let mut cut = Stores::in_memory();
    cut.transport = Some(Arc::new(netfetcher::NoTransport));
    let direct = handle(&cut).read_range("http://127.0.0.1:9/episode.mp3", span(0), None);
    assert!(
        matches!(direct, Err(FetchError::Unreachable(_))),
        "{direct:?}"
    );
}

/// A fallback that records what reached it and answers with a fixed body.
#[derive(Clone)]
struct Recording(Arc<Mutex<Vec<String>>>);

impl ResourceFetcher for Recording {
    fn fetch_request(
        &self,
        _request: &genet_host_api::ResourceRequest,
    ) -> Option<genet_host_api::ResourceFetchResponse> {
        None
    }
    fn fetch(&self, url: &str) -> Option<Vec<u8>> {
        self.0.lock().unwrap().push(url.to_owned());
        Some(b"from the fallback".to_vec())
    }
}

#[test]
fn the_handle_serves_a_documents_resources_and_hands_other_schemes_on() {
    let server = Server::start();
    let stores = Stores::in_memory();
    let shared: Arc<dyn Fetch> = Arc::new(handle(&stores));

    // Without a fallback: http(s) served, the rest refused.
    let alone = Resources::new(shared.clone());
    let feed = alone
        .fetch_response(server.url("/feed.xml").as_str())
        .unwrap();
    assert_eq!(feed.bytes, b"<rss></rss>");
    assert_eq!(feed.final_url, server.url("/feed.xml").to_string());
    assert_eq!(feed.content_type.as_deref(), Some("application/rss+xml"));
    let rich = alone
        .fetch_request(&ResourceRequest::plain_get(
            server.url("/feed.xml").to_string(),
        ))
        .unwrap();
    assert_eq!(
        rich.content_type.as_deref(),
        Some("application/rss+xml; charset=utf-8")
    );
    assert!(rich.headers.iter().any(|(name, value)| {
        name.eq_ignore_ascii_case("content-type") && value == "application/rss+xml; charset=utf-8"
    }));
    assert!(alone.fetch("gemini://example.test/").is_none());
    // A redirect reports where the bytes really came from.
    let moved = alone.fetch_response(server.url("/moved").as_str()).unwrap();
    assert_eq!(moved.final_url, server.url("/episode.mp3").to_string());
    // The cap is the document engine's, applied before any body is read.
    assert!(
        Resources::new(shared.clone())
            .with_limit(1_000_000)
            .fetch(server.url("/episode.mp3").as_str())
            .is_none()
    );

    // With a fallback: the same http(s) path, and other schemes handed on.
    let seen = Recording(Arc::new(Mutex::new(Vec::new())));
    let chained = Resources::new(shared).with_fallback(seen.clone());
    assert_eq!(
        chained.fetch(server.url("/feed.xml").as_str()).unwrap(),
        b"<rss></rss>"
    );
    assert_eq!(
        chained.fetch("gemini://example.test/").unwrap(),
        b"from the fallback"
    );
    assert_eq!(
        *seen.0.lock().unwrap(),
        vec!["gemini://example.test/".to_owned()]
    );

    // And it shares the scope: a cookie set by the actor's page reaches an image.
    let page = handle(&stores);
    page.read_all(server.url("/page").as_str(), None, 1024)
        .unwrap();
    Resources::new(Arc::new(handle(&stores))).fetch(server.url("/episode.mp3").as_str());
    let last = server.seen("/episode.mp3").last().cloned().unwrap();
    assert_eq!(last.header("cookie"), Some("session=persona-a"));
}

#[test]
fn scoped_resource_fetch_keeps_request_and_http_error_metadata() {
    let mut server = mockito::Server::new();
    let mock = server
        .mock("POST", "/submit")
        .match_header("content-type", "application/octet-stream")
        .match_body(vec![0, 128, 255])
        .with_status(409)
        .with_header("content-type", "text/plain")
        .with_header("x-result", "conflict")
        .with_body("conflict")
        .create();
    let stores = Stores::in_memory();
    let resources = Resources::new(Arc::new(handle(&stores)));
    let mut request = ResourceRequest::plain_get(format!("{}/submit", server.url()));
    request.method = "POST".into();
    request
        .headers
        .push(("content-type".into(), "application/octet-stream".into()));
    request.body = Some(vec![0, 128, 255]);
    let response = resources
        .fetch_request(&request)
        .expect("HTTP error is a response");
    assert_eq!(response.status, 409);
    assert_eq!(response.bytes, b"conflict");
    assert_eq!(response.final_url, request.url);
    assert_eq!(response.content_type.as_deref(), Some("text/plain"));
    assert!(
        response
            .headers
            .iter()
            .any(|(name, value)| name == "x-result" && value == "conflict")
    );
    mock.assert();
}

#[test]
fn scoped_resource_fetch_refuses_unknown_request_policy() {
    let stores = Stores::in_memory();
    let resources = Resources::new(Arc::new(handle(&stores)));
    let mut request = ResourceRequest::plain_get("http://127.0.0.1:9/unused");
    request.credentials = "ambient".into();
    assert!(resources.fetch_request(&request).is_none());
}

struct DripWire;

impl Transport for DripWire {
    fn send(&self, _request: WireRequest) -> TransportFuture<'_> {
        Box::pin(async {
            let body = futures_util::stream::unfold(0, |index| async move {
                if index >= 3 {
                    return None;
                }
                tokio::time::sleep(Duration::from_millis(20)).await;
                Some((
                    Ok::<bytes::Bytes, std::io::Error>(bytes::Bytes::from_static(b"x")),
                    index + 1,
                ))
            });
            Some(RawResponse {
                status: 200,
                headers: Vec::new(),
                body: Box::pin(body),
            })
        })
    }
}

#[test]
fn rich_request_deadline_covers_all_body_chunks() {
    let mut stores = Stores::in_memory();
    stores.transport = Some(Arc::new(DripWire));
    let fetch = handle(&stores).with_patience(Duration::from_millis(45));
    let request = ResourceRequest::plain_get("http://example.test/drip");
    let started = std::time::Instant::now();
    assert!(matches!(
        fetch.read_request(&request, 1024),
        Err(FetchError::Body(error)) if error.kind() == std::io::ErrorKind::TimedOut
    ));
    assert!(started.elapsed() < Duration::from_secs(1));
}

#[path = "handle_tests/ranges.rs"]
mod ranges;
