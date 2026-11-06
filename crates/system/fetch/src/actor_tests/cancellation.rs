// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

#[test]
fn a_timed_out_waiter_releases_its_pending_reservation_only_after_task_destruction() {
    let server = StalledHttp::start();
    let (actor, updates) = actor(bounded(1, 1, Duration::from_millis(400)));
    assert!(actor.command(page(91, server.url("/held"))));
    let first_request = server.seen.recv_timeout(WAIT).unwrap_or_else(|error| {
        let diagnostic = match updates.try_recv() {
            Ok(FetchUpdate::Page(outcome)) => format!(
                "request {} for {} returned {:?}",
                outcome.request, outcome.url, outcome.result
            ),
            Ok(FetchUpdate::PageProgress(progress)) => format!(
                "request {} emitted progress for {}",
                progress.request, progress.url
            ),
            Ok(FetchUpdate::Subresource(outcome)) => {
                format!("subresource {} returned {:?}", outcome.url, outcome.result)
            },
            Ok(FetchUpdate::Favicon(outcome)) => {
                format!("favicon {} returned {:?}", outcome.request, outcome.result)
            },
            Ok(FetchUpdate::Submission(outcome)) => {
                format!(
                    "submission {} returned {:?}",
                    outcome.request, outcome.result
                )
            },
            Err(error) => format!("no update available ({error:?})"),
        };
        panic!("timed out waiting for initial HTTP request ({error}); {diagnostic}");
    });
    assert_eq!(first_request, "/held");
    assert!(actor.command(page(92, server.url("/expired"))));
    match next_update(&updates) {
        FetchUpdate::Page(outcome) => {
            assert_eq!(outcome.request, 92);
            assert!(matches!(outcome.result, Err(FetchFailure::Failed(_))));
        },
        _ => panic!("expected timed out waiter completion"),
    }

    let next_url = server.url("/reservation-reused");
    let retry_deadline = Instant::now() + WAIT;
    loop {
        if actor.command(page(93, next_url.clone())) {
            break;
        }
        match next_update(&updates) {
            FetchUpdate::Page(outcome) => {
                assert_eq!(outcome.request, 93);
                assert!(matches!(outcome.result, Err(FetchFailure::Failed(_))));
            },
            _ => panic!("expected refusal while the expired job still owns its slot"),
        }
        assert!(
            Instant::now() < retry_deadline,
            "pending slot was not released"
        );
        thread::yield_now();
    }
    server.release_one();
    assert_eq!(server.next_request(), "/reservation-reused");
    let mut completed = [false; 2];
    while completed != [true, true] {
        match next_update(&updates) {
            FetchUpdate::Page(outcome) if outcome.request == 91 => completed[0] = true,
            FetchUpdate::Page(outcome) if outcome.request == 93 => {
                assert!(outcome.result.is_ok());
                completed[1] = true;
            },
            FetchUpdate::PageProgress(progress) => assert!([91, 93].contains(&progress.request)),
            _ => panic!("unexpected result after pending reservation reuse"),
        }
    }
    assert_eq!(completed, [true, true]);
    let report = actor.close_and_join().unwrap();
    assert_eq!(report.tasks.joined(), 3);
    assert!(report.tasks.is_clean());
}

#[test]
fn cancelling_one_same_url_request_leaves_the_other_correlated_request_live() {
    let server = StalledHttp::start();
    let url = server.url("/same");
    let (actor, updates) = actor(bounded(2, 0, WAIT));

    assert!(actor.command(page(31, url.clone())));
    assert!(actor.command(page(32, url.clone())));
    assert_eq!(server.next_request(), "/same");
    assert_eq!(server.next_request(), "/same");
    assert!(actor.command(FetchCommand::CancelPage { request: 31 }));
    match next_update(&updates) {
        FetchUpdate::Page(outcome) => {
            assert_eq!(outcome.request, 31);
            assert!(matches!(outcome.result, Err(FetchFailure::Cancelled)));
        },
        _ => panic!("expected correlated cancellation result"),
    }
    server.release_one();
    server.release_one();
    let mut survived = false;
    while !survived {
        match next_update(&updates) {
            FetchUpdate::Page(outcome) if outcome.request == 32 => {
                assert!(outcome.result.is_ok());
                survived = true;
            },
            FetchUpdate::PageProgress(progress) => {
                assert!(progress.request == 31 || progress.request == 32);
            },
            _ => panic!("expected surviving request completion"),
        }
    }
    let report = actor.close_and_join().unwrap();
    assert_eq!(report.admitted, 2);
    assert_eq!(report.tasks.joined(), 2);
    assert!(report.tasks.is_clean());
}

#[cfg(feature = "smolweb")]
#[test]
fn exact_cancel_drops_one_native_stream_before_shutdown_and_same_url_peer_finishes() {
    let peers = GopherPeers::start();
    let url = peers.url();
    let (actor, updates) = actor(bounded(2, 0, WAIT));
    assert!(actor.command(page(33, url.clone())));
    assert!(actor.command(page(34, url)));
    let first_peer = peers
        .accepted
        .recv_timeout(WAIT)
        .expect("first Gopher stream");
    let second_peer = peers
        .accepted
        .recv_timeout(WAIT)
        .expect("second Gopher stream");
    assert_ne!(first_peer.0, second_peer.0);

    assert!(actor.command(FetchCommand::CancelPage { request: 33 }));
    let mut cancelled = false;
    while !cancelled {
        match next_update(&updates) {
            FetchUpdate::Page(outcome) if outcome.request == 33 => {
                assert!(matches!(outcome.result, Err(FetchFailure::Cancelled)));
                cancelled = true;
            },
            FetchUpdate::PageProgress(_) => {},
            _ => panic!("unexpected update while cancelling exact Gopher request"),
        }
    }
    let closed_peer = peers
        .eof
        .recv_timeout(WAIT)
        .expect("cancelled TCP stream EOF");
    let survivor = if first_peer.0 == closed_peer {
        second_peer
    } else {
        first_peer
    };
    survivor.1.send(()).unwrap();
    let mut completed = false;
    while !completed {
        match next_update(&updates) {
            FetchUpdate::Page(outcome) if outcome.request == 34 => {
                assert!(outcome.result.is_ok());
                completed = true;
            },
            FetchUpdate::PageProgress(progress) => assert_eq!(progress.request, 34),
            _ => panic!("unexpected update from surviving Gopher request"),
        }
    }
    let report = actor.close_and_join().unwrap();
    assert_eq!(report.tasks.joined(), 2);
    assert!(report.tasks.is_clean());
}

#[cfg(feature = "smolweb")]
#[test]
fn native_transport_timeout_starts_after_admission_queue_wait() {
    let peers = GopherPeers::start();
    let url = peers.url();
    let client_timeout = Duration::from_millis(1600);
    let client_policy = ClientFetchPolicy {
        timeout: client_timeout,
        ..ClientFetchPolicy::default()
    };
    let (actor, updates) = spawn_fetcher_with_admission(
        wake(),
        Stores::in_memory(),
        client_policy,
        bounded(1, 1, Duration::from_secs(3)),
    )
    .unwrap();
    assert!(actor.command(page(101, url.clone())));
    let first = peers
        .accepted
        .recv_timeout(WAIT)
        .expect("first admitted Gopher stream");

    let queued_at = Instant::now();
    assert!(actor.command(page(102, url)));
    assert!(matches!(
        peers.accepted.recv_timeout(Duration::from_millis(900)),
        Err(mpsc::RecvTimeoutError::Timeout)
    ));
    first.1.send(()).unwrap();
    let mut first_completed = false;
    while !first_completed {
        match next_update(&updates) {
            FetchUpdate::Page(outcome) if outcome.request == 101 => {
                assert!(outcome.result.is_ok());
                first_completed = true;
            },
            FetchUpdate::PageProgress(progress) => assert_eq!(progress.request, 101),
            _ => panic!("unexpected update from first Gopher request"),
        }
    }
    let second = peers
        .accepted
        .recv_timeout(WAIT)
        .expect("queued request starts when the first task releases its slot");
    thread::sleep(Duration::from_millis(900));
    second.1.send(()).unwrap();
    let mut second_completed = false;
    while !second_completed {
        match next_update(&updates) {
            FetchUpdate::Page(outcome) if outcome.request == 102 => {
                assert!(outcome.result.is_ok());
                second_completed = true;
            },
            FetchUpdate::PageProgress(progress) => assert_eq!(progress.request, 102),
            _ => panic!("unexpected update from queued Gopher request"),
        }
    }
    assert!(queued_at.elapsed() > client_timeout);
    let report = actor.close_and_join().unwrap();
    assert_eq!(report.admitted, 2);
    assert_eq!(report.tasks.joined(), 2);
    assert!(report.tasks.is_clean());
}

#[test]
fn page_progress_keeps_its_request_id_and_has_one_terminal_completion() {
    let server = StalledHttp::start();
    let url = server.url("/stream");
    let (actor, updates) = actor(bounded(1, 0, WAIT));
    assert!(actor.command(page(35, url.clone())));
    assert_eq!(server.next_request(), "/stream");
    server.release_one();
    let mut progress_seen = false;
    let mut terminal_seen = false;
    while !terminal_seen {
        match next_update(&updates) {
            FetchUpdate::PageProgress(progress) => {
                assert_eq!(progress.request, 35);
                assert_eq!(progress.url, url);
                assert!(!progress.bytes.is_empty());
                progress_seen = true;
            },
            FetchUpdate::Page(outcome) => {
                assert_eq!(outcome.request, 35);
                assert!(outcome.result.is_ok());
                terminal_seen = true;
            },
            _ => panic!("unexpected update during streamed page fetch"),
        }
    }
    assert!(progress_seen);
    assert!(matches!(updates.try_recv(), Err(TryRecvError::Empty)));
    let report = actor.close_and_join().unwrap();
    assert_eq!(report.tasks.joined(), 1);
    assert!(report.tasks.is_clean());
}
