// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The handle against a real local server, called the way a decoder calls it.
//! Moved in from the R3-A probe
//! (`design_docs/mere_docs/testing/receipts/2026-09-20_resource_resolution_probes`).

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

use netfetcher::InMemoryCookieJar;

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

#[test]
fn a_blocking_thread_reads_exact_ranges_with_identity_encoding() {
    let server = Server::start();
    let fetch: Arc<dyn Fetch> = Arc::new(handle(&Stores::in_memory()));
    let data = object();
    let url = server.url("/episode.mp3").to_string();

    // The decoder's shape: its own thread, one blocking call per range.
    let fetched = thread::spawn({
        let (fetch, url) = (fetch.clone(), url.clone());
        move || {
            let probe = fetch.read_range(&url, FIRST_BYTE, None).unwrap();
            assert_eq!(
                (probe.start, probe.total, probe.bytes.len()),
                (0, OBJECT_BYTES as u64, 1)
            );
            assert_eq!(probe.facts.content_type.as_deref(), Some("audio/mpeg"));
            assert_eq!(probe.facts.etag.as_deref(), Some("\"v1\""));
            let mut total = 1_u64;
            for start in [65_536_u64, 983_040, 2_490_368] {
                let reply = fetch
                    .read_range(&url, span(start), probe.facts.etag.as_deref())
                    .unwrap();
                assert_eq!(reply.start, start);
                assert_eq!(
                    &reply.bytes[..],
                    &data[start as usize..(start + CHUNK) as usize]
                );
                total += reply.bytes.len() as u64;
            }
            // An open-ended range reads to the end of the object.
            let tail = Range {
                start: OBJECT_BYTES as u64 - 10,
                end: None,
            };
            let tail = fetch.read_range(&url, tail, None).unwrap();
            assert_eq!(&tail.bytes[..], &data[OBJECT_BYTES - 10..]);
            total
        }
    })
    .join()
    .unwrap();

    let seen = server.seen("/episode.mp3");
    assert_eq!(seen.len(), 5);
    assert!(fetched * 10 < OBJECT_BYTES as u64, "fetched {fetched}");
    for request in &seen {
        assert_eq!(request.header("accept-encoding"), Some("identity"));
        assert!(request.header("range").is_some());
    }
}

#[test]
fn a_changed_representation_and_an_ignored_range_are_told_apart() {
    let server = Server::start();
    let fetch = handle(&Stores::in_memory());
    let url = server.url("/episode.mp3").to_string();

    let probe = fetch.read_range(&url, FIRST_BYTE, None).unwrap();
    *server.etag.lock().unwrap() = "\"v2\"".to_owned();
    let changed = fetch.read_range(&url, span(65_536), probe.facts.etag.as_deref());
    assert!(matches!(changed, Err(FetchError::Changed)), "{changed:?}");
    assert_eq!(
        server.seen("/episode.mp3")[1].header("if-range"),
        Some("\"v1\"")
    );

    // A server that answers a plain ranged request with the whole object.
    server.release.send(()).unwrap();
    let ignored = fetch.read_range(server.url("/slow").as_str(), span(0), None);
    assert!(
        matches!(ignored, Err(FetchError::RangeIgnored)),
        "{ignored:?}"
    );
}

#[test]
fn a_redirect_keeps_the_range_and_reports_the_final_url() {
    let server = Server::start();
    let fetch = handle(&Stores::in_memory());

    let reply = fetch
        .read_range(server.url("/moved").as_str(), span(65_536), None)
        .unwrap();
    assert_eq!(
        reply.facts.final_url,
        server.url("/episode.mp3").to_string()
    );
    let landed = server.seen("/episode.mp3");
    assert_eq!(landed.len(), 1);
    assert_eq!(landed[0].header("range"), Some("bytes=65536-131071"));
    assert_eq!(landed[0].header("accept-encoding"), Some("identity"));
}

#[test]
fn a_ranged_read_carries_its_own_scopes_cookie_and_no_other() {
    let server = Server::start();
    let (jar_a, jar_b) = (
        Arc::new(InMemoryCookieJar::new()),
        Arc::new(InMemoryCookieJar::new()),
    );
    let mut stores_a = Stores::in_memory();
    stores_a.cookies = jar_a.clone();
    let mut stores_b = Stores::in_memory();
    stores_b.cookies = jar_b.clone();
    let url = server.url("/episode.mp3").to_string();

    // Scope A logs in on a page through one handle, which is then discarded.
    let page = handle(&stores_a);
    let body = page
        .read_all(server.url("/page").as_str(), None, 1024)
        .unwrap();
    assert_eq!(body.bytes, b"ok");
    drop(page);
    assert_eq!(jar_a.len(), 1);

    // A later handle over the same stores, as a media reader would build.
    handle(&stores_a)
        .read_range(&url, FIRST_BYTE, None)
        .unwrap();
    handle(&stores_b)
        .read_range(&url, FIRST_BYTE, None)
        .unwrap();

    let seen = server.seen("/episode.mp3");
    assert_eq!(seen[0].header("cookie"), Some("session=persona-a"));
    assert_eq!(seen[1].header("cookie"), None);
    assert!(jar_b.is_empty());
}

#[test]
fn a_cached_whole_object_never_answers_a_ranged_read() {
    let server = Server::start();
    let fetch = handle(&Stores::in_memory());
    let url = server.url("/episode.mp3").to_string();

    // Positive control: the whole object caches, and a second read is a hit.
    for _ in 0..2 {
        let whole = fetch.read_all(&url, None, 4_000_000).unwrap();
        assert_eq!(whole.bytes.len(), OBJECT_BYTES);
    }
    assert_eq!(
        server.seen("/episode.mp3").len(),
        1,
        "the second whole read must be a cache hit"
    );

    // The handle's ranged read goes to the network and gets its range.
    let reply = fetch.read_range(&url, span(65_536), None).unwrap();
    assert_eq!(reply.bytes.len(), CHUNK as usize);
    assert_eq!(server.seen("/episode.mp3").len(), 2);

    // Pinned negative, the reason `read_range` is no-store: netfetcher in its
    // default cache mode answers the same ranged request from the stored whole
    // object, 200 with every byte, without touching the network.
    let mut request = Request::get(server.url("/episode.mp3"));
    request
        .headers
        .push(("range".to_owned(), "bytes=65536-131071".to_owned()));
    let unguarded = fetch.send(request).unwrap();
    assert_eq!(unguarded.status, 200);
    assert_eq!(
        server.seen("/episode.mp3").len(),
        2,
        "the unguarded request must not have reached the network"
    );
}

#[test]
fn a_feed_is_read_whole_and_an_oversize_body_is_refused() {
    let server = Server::start();
    let fetch = handle(&Stores::in_memory());

    let feed = fetch
        .read_all(
            server.url("/feed.xml").as_str(),
            Some("application/rss+xml"),
            4 * 1024 * 1024,
        )
        .unwrap();
    assert_eq!(feed.bytes, b"<rss></rss>");
    assert_eq!(
        feed.facts.content_type.as_deref(),
        Some("application/rss+xml")
    );
    assert_eq!(
        server.seen("/feed.xml")[0].header("accept"),
        Some("application/rss+xml")
    );

    // Refused on the declared length, before any body is read.
    let oversize = fetch.read_all(server.url("/episode.mp3").as_str(), None, 1_000_000);
    assert!(
        matches!(oversize, Err(FetchError::TooLarge { limit: 1_000_000 })),
        "{oversize:?}"
    );
    let scheme = fetch.read_all("gemini://example.test/", None, 1024);
    assert!(
        matches!(scheme, Err(FetchError::BadAddress(_))),
        "{scheme:?}"
    );
}

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

#[test]
fn a_download_streams_into_its_sink_before_the_server_finishes() {
    let server = Server::start();
    let fetch = handle(&Stores::in_memory());
    let (started, first_write) = mpsc::channel();
    let mut sink = Watched {
        written: 0,
        first: None,
        started,
    };

    // The server holds the second half until the sink has seen bytes, so this
    // only completes if the body is streamed rather than collected.
    let release = server.release.clone();
    let waiter = thread::spawn(move || {
        first_write
            .recv_timeout(Duration::from_secs(8))
            .expect("the sink must see bytes before the server finishes");
        release.send(()).unwrap();
    });
    let facts = fetch
        .read_into(server.url("/slow").as_str(), &mut sink, None)
        .unwrap();
    waiter.join().unwrap();

    assert_eq!(sink.written, OBJECT_BYTES);
    assert!(sink.first.unwrap() <= OBJECT_BYTES / 2);
    assert_eq!(facts.content_length, Some(OBJECT_BYTES as u64));
    assert_eq!(
        server.seen("/slow")[0].header("accept-encoding"),
        Some("identity")
    );

    let mut small = Vec::new();
    let refused = fetch.read_into(
        server.url("/episode.mp3").as_str(),
        &mut small,
        Some(1_000_000),
    );
    assert!(
        matches!(refused, Err(FetchError::TooLarge { limit: 1_000_000 })),
        "{refused:?}"
    );
    assert!(
        small.is_empty(),
        "nothing is written when the declared length is over the limit"
    );
}
