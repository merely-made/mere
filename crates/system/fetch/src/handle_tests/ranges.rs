// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

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

#[cfg(feature = "actor")]
#[test]
fn a_page_login_through_the_actor_is_carried_by_a_ranged_read() {
    let server = Server::start();
    let jar = Arc::new(InMemoryCookieJar::new());
    let mut stores = Stores::in_memory();
    stores.cookies = jar.clone();

    // The page actor and the handle are built from one store set.
    let (woken, wakes) = mpsc::channel();
    let wake: armillary::Wake = Arc::new(move || {
        let _ = woken.send(());
    });
    let (actor, updates) = crate::spawn_fetcher_with(wake, stores.clone());
    assert!(actor.command(crate::FetchCommand::Page {
        request: 1,
        url: server.url("/page").to_string(),
        identity: None,
    }));
    wakes
        .recv_timeout(Duration::from_secs(10))
        .expect("the actor must wake its host");
    match updates.recv_timeout(Duration::from_secs(10)).unwrap() {
        crate::FetchUpdate::PageProgress(progress) => {
            assert_eq!(progress.request, 1);
            assert_eq!(progress.url, server.url("/page").to_string());
            assert_eq!(progress.response_url, server.url("/page").to_string());
            assert_eq!(progress.bytes, b"ok");
        },
        _ => panic!("expected exact completed HTTP source progress before the page outcome"),
    }
    match updates.recv_timeout(Duration::from_secs(10)).unwrap() {
        crate::FetchUpdate::Page(outcome) => {
            assert_eq!(outcome.request, 1);
            assert_eq!(outcome.result.unwrap().body, "ok");
        },
        _ => panic!("expected the terminal page outcome after its source progress"),
    }
    assert_eq!(jar.len(), 1, "the login landed in the host's jar");

    handle(&stores)
        .read_range(server.url("/episode.mp3").as_str(), FIRST_BYTE, None)
        .unwrap();
    assert_eq!(
        server.seen("/episode.mp3")[0].header("cookie"),
        Some("session=persona-a")
    );
    // Control: the process session, which this actor was not given, saw nothing.
    assert!(crate::session_jar().is_empty());
}
