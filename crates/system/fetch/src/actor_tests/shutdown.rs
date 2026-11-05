// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;

#[test]
fn closing_with_active_page_subresource_and_favicon_joins_and_finishes_each_once() {
    let server = StalledHttp::start();
    let page_url = server.url("/page");
    let subresource_url = server.url("/image");
    let favicon_url = server.url("/favicon");
    let (actor, updates) = actor(bounded(3, 0, WAIT));

    assert!(actor.command(page(41, page_url.clone())));
    assert!(actor.command(FetchCommand::Subresource(subresource_url.clone())));
    assert!(actor.command(FetchCommand::Favicon {
        request: 42,
        url: favicon_url.clone(),
    }));
    let mut opened = vec![
        server.next_request(),
        server.next_request(),
        server.next_request(),
    ];
    opened.sort();
    assert_eq!(opened, ["/favicon", "/image", "/page"]);

    let report = actor.close_and_join().unwrap();
    assert_eq!(report.admitted, 3);
    assert_eq!(report.tasks.joined(), 3);
    assert!(report.tasks.is_clean());
    let mut page_done = 0;
    let mut subresource_done = 0;
    let mut favicon_done = 0;
    for _ in 0..3 {
        match next_update(&updates) {
            FetchUpdate::Page(outcome) => {
                assert_eq!(outcome.request, 41);
                assert!(matches!(outcome.result, Err(FetchFailure::Cancelled)));
                page_done += 1;
            },
            FetchUpdate::Subresource(outcome) => {
                assert_eq!(outcome.url, subresource_url);
                assert!(outcome.result.is_err());
                subresource_done += 1;
            },
            FetchUpdate::Favicon(outcome) => {
                assert_eq!(outcome.request, 42);
                assert!(outcome.result.is_err());
                favicon_done += 1;
            },
            _ => panic!("unexpected update during joined shutdown"),
        }
    }
    assert_eq!((page_done, subresource_done, favicon_done), (1, 1, 1));
    assert!(matches!(
        updates.try_recv(),
        Err(TryRecvError::Empty | TryRecvError::Disconnected)
    ));
}

#[test]
fn original_actor_handle_join_waits_for_task_cleanup() {
    let server = StalledHttp::start();
    let url = server.url("/legacy");
    let (handle, updates) = spawn_fetcher_with(wake(), Stores::in_memory());
    assert!(handle.command(page(71, url)));
    assert_eq!(server.next_request(), "/legacy");
    handle.join().unwrap();
    assert!(matches!(next_update(&updates), FetchUpdate::Page(_)));
}

#[test]
fn dropping_fetch_actor_does_not_wait_for_a_blocked_cancellation_wake() {
    let server = StalledHttp::start();
    let url = server.url("/drop");
    let (wake_entered_tx, wake_entered_rx) = mpsc::channel();
    let (wake_release_tx, wake_release_rx) = mpsc::channel();
    let wake_release_rx = Mutex::new(wake_release_rx);
    let wake: Wake = Arc::new(move || {
        let _ = wake_entered_tx.send(());
        let _ = wake_release_rx.lock().unwrap().recv_timeout(WAIT);
    });
    let (actor, updates) = spawn_fetcher_with_admission(
        wake,
        Stores::in_memory(),
        ClientFetchPolicy::default(),
        bounded(1, 0, WAIT),
    )
    .unwrap();
    assert!(actor.command(page(76, url)));
    assert_eq!(server.next_request(), "/drop");

    let (drop_returned_tx, drop_returned_rx) = mpsc::channel();
    let dropper = thread::spawn(move || {
        drop(actor);
        let _ = drop_returned_tx.send(());
    });
    let wake_started = wake_entered_rx.recv_timeout(WAIT).is_ok();
    let drop_returned_while_wake_held = drop_returned_rx
        .recv_timeout(Duration::from_millis(250))
        .is_ok();
    let _ = wake_release_tx.send(());
    dropper.join().unwrap();

    assert!(
        wake_started,
        "shutdown never reached the blocking wake callback"
    );
    assert!(
        drop_returned_while_wake_held,
        "FetchActor::drop waited for a blocked wake callback"
    );
    match updates.recv_timeout(WAIT).expect("one cancellation update") {
        FetchUpdate::Page(outcome) => {
            assert_eq!(outcome.request, 76);
            assert!(matches!(outcome.result, Err(FetchFailure::Cancelled)));
        },
        _ => panic!("expected cancellation completion after wake release"),
    }
    assert!(matches!(
        updates.recv_timeout(WAIT),
        Err(mpsc::RecvTimeoutError::Disconnected)
    ));
}

#[test]
fn task_wake_panic_is_reported_after_the_terminal_update_without_hanging_join() {
    let server = StalledHttp::start();
    let url = server.url("/reservation-reused");
    let should_panic = Arc::new(AtomicBool::new(true));
    let panic_once = should_panic.clone();
    let wake: Wake = Arc::new(move || {
        if panic_once.swap(false, Ordering::AcqRel) {
            panic!("intentional one-shot test wake panic");
        }
    });
    let (actor, updates) = spawn_fetcher_with_admission(
        wake,
        Stores::in_memory(),
        ClientFetchPolicy::default(),
        bounded(1, 0, WAIT),
    )
    .unwrap();
    assert!(actor.command(FetchCommand::Subresource(url)));
    assert_eq!(server.next_request(), "/reservation-reused");
    match next_update(&updates) {
        FetchUpdate::Subresource(outcome) => assert!(outcome.result.is_ok()),
        _ => panic!("expected the terminal update sent before the wake panic"),
    }
    let report = actor.close_and_join().unwrap();
    assert_eq!(report.tasks.panicked, 1);
}

#[cfg(feature = "smolweb")]
#[test]
fn closing_a_started_write_reports_unknown_delivery_without_retry() {
    let server = StalledSpartan::start();
    let url = server.url();
    let (actor, updates) = actor(bounded(1, 0, WAIT));
    assert!(actor.command(FetchCommand::Submit {
        request: 81,
        submission: SmolwebSubmission::Spartan {
            url: url.clone(),
            body: b"payload".to_vec(),
        },
    }));
    assert_eq!(server.seen.recv_timeout(WAIT).unwrap(), 7);
    let report = actor.close_and_join().unwrap();
    assert_eq!(report.admitted, 1);
    assert_eq!(report.tasks.joined(), 1);
    assert!(report.tasks.is_clean());
    match next_update(&updates) {
        FetchUpdate::Submission(outcome) => {
            assert_eq!(outcome.request, 81);
            let error = outcome.result.unwrap_err().to_string();
            assert!(error.contains("unknown"));
            assert!(error.contains("not retried"));
        },
        _ => panic!("expected correlated stopped-write result"),
    }
    assert!(matches!(server.seen.try_recv(), Err(TryRecvError::Empty)));
}

#[cfg(feature = "smolweb")]
#[test]
fn closing_a_pending_write_reports_not_started_and_never_contacts_its_origin() {
    let server = StalledHttp::start();
    let (actor, updates) = actor(bounded(1, 1, WAIT));
    assert!(actor.command(page(82, server.url("/hold"))));
    assert_eq!(server.next_request(), "/hold");
    let url = format!("spartan://{}/pending", server.address);
    assert!(actor.command(FetchCommand::Submit {
        request: 83,
        submission: SmolwebSubmission::Spartan {
            url: url.clone(),
            body: b"payload".to_vec(),
        },
    }));
    let report = actor.close_and_join().unwrap();
    assert_eq!(report.admitted, 2);
    assert_eq!(
        report.tasks.joined() + report.discarded_before_spawn,
        report.admitted
    );
    assert!(report.tasks.is_clean());
    let mut saw_page = false;
    let mut saw_submission = false;
    for _ in 0..2 {
        match next_update(&updates) {
            FetchUpdate::Page(outcome) => {
                assert_eq!(outcome.request, 82);
                assert!(matches!(outcome.result, Err(FetchFailure::Cancelled)));
                saw_page = true;
            },
            FetchUpdate::Submission(outcome) => {
                assert_eq!(outcome.request, 83);
                assert_eq!(outcome.url, url);
                assert!(outcome.result.is_err());
                saw_submission = true;
            },
            _ => panic!("unexpected result while closing pending write"),
        }
    }
    assert!(saw_page && saw_submission);
    assert!(matches!(server.seen.try_recv(), Err(TryRecvError::Empty)));
}
