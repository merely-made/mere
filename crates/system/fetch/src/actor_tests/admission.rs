// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;

#[test]
fn admission_sheds_overflow_without_contacting_the_second_origin() {
    let server = StalledHttp::start();
    let url_a = server.url("/active");
    let url_b = server.url("/overflow");
    let (actor, updates) = actor(bounded(1, 0, WAIT));

    assert!(actor.command(page(11, url_a.clone())));
    assert_eq!(server.next_request(), "/active");
    assert!(!actor.command(page(12, url_b.clone())));
    match next_update(&updates) {
        FetchUpdate::Page(outcome) => {
            assert_eq!(outcome.request, 12);
            assert_eq!(outcome.url, url_b);
            assert!(matches!(outcome.result, Err(FetchFailure::Failed(_))));
        },
        _ => panic!("expected correlated overflow page result"),
    }
    assert!(matches!(server.seen.try_recv(), Err(TryRecvError::Empty)));

    let report = actor.close_and_join().unwrap();
    assert_eq!(report.overloaded, 1);
    assert_eq!(report.tasks.joined(), 1);
    assert!(report.tasks.is_clean());
    assert!(matches!(next_update(&updates), FetchUpdate::Page(_)));
}

#[test]
fn rejected_duplicate_id_is_latest_intent_and_suppresses_older_completion() {
    let server = StalledHttp::start();
    let old_url = server.url("/old-intent");
    let latest_url = server.url("/latest-intent");
    let (actor, updates) = actor(bounded(1, 0, WAIT));
    assert!(actor.command(page(15, old_url.clone())));
    assert_eq!(server.next_request(), "/old-intent");

    assert!(!actor.command(page(15, latest_url.clone())));
    match next_update(&updates) {
        FetchUpdate::Page(outcome) => {
            assert_eq!(outcome.request, 15);
            assert_eq!(outcome.url, latest_url);
            assert!(matches!(outcome.result, Err(FetchFailure::Failed(_))));
        },
        _ => panic!("expected rejected latest-intent completion"),
    }
    server.release_one();
    let report = actor.close_and_join().unwrap();
    assert_eq!(report.overloaded, 1);
    assert_eq!(report.tasks.joined(), 1);
    assert!(report.tasks.is_clean());
    assert!(matches!(
        updates.try_recv(),
        Err(TryRecvError::Disconnected)
    ));
}

#[test]
fn concurrent_same_id_admission_leaves_only_the_latest_completion_registered() {
    let (actor, updates) = actor(bounded(2, 2, WAIT));
    let barrier = Arc::new(Barrier::new(3));
    let mut callers = Vec::new();
    for url in ["http://127.0.0.1/left", "http://127.0.0.1/right"] {
        let (shared, out, barrier) = (actor.shared.clone(), actor.out.clone(), barrier.clone());
        let url = url.to_owned();
        callers.push(thread::spawn(move || {
            barrier.wait();
            shared.admit(page(16, url), &out)
        }));
    }
    barrier.wait();
    let admissions: Vec<_> = callers
        .into_iter()
        .map(|caller| caller.join().unwrap())
        .collect();
    let current = actor
        .shared
        .pages
        .lock()
        .unwrap()
        .get(&16)
        .and_then(Weak::upgrade)
        .expect("one current exact-ID completion");
    assert!(!current.terminal.load(Ordering::Acquire));
    match &current.key {
        Key::Page { url, .. } => assert!(url.ends_with("/left") || url.ends_with("/right")),
        _ => panic!("page request ID mapped to another operation kind"),
    }
    drop(admissions);
    let _ = actor.close_and_join().unwrap();
    let _ = updates.try_recv();
}

#[test]
fn queued_request_expires_before_transport_contact() {
    let server = StalledHttp::start();
    let active_url = server.url("/held");
    let queued_url = server.url("/must-not-open");
    let (actor, updates) = actor(bounded(1, 1, Duration::from_millis(150)));

    assert!(actor.command(page(21, active_url)));
    assert_eq!(server.next_request(), "/held");
    assert!(actor.command(page(22, queued_url.clone())));
    match next_update(&updates) {
        FetchUpdate::Page(outcome) => {
            assert_eq!(outcome.request, 22);
            assert_eq!(outcome.url, queued_url);
            assert!(matches!(outcome.result, Err(FetchFailure::Failed(_))));
        },
        _ => panic!("expected correlated queued timeout result"),
    }
    assert!(matches!(server.seen.try_recv(), Err(TryRecvError::Empty)));

    let report = actor.close_and_join().unwrap();
    assert_eq!(report.queue_timeouts, 1);
    assert_eq!(report.tasks.joined(), 2);
    assert!(report.tasks.is_clean());
    assert!(matches!(next_update(&updates), FetchUpdate::Page(_)));
}

#[test]
fn queued_cancel_is_terminal_and_never_opens_its_origin() {
    let server = StalledHttp::start();
    let first_url = server.url("/active");
    let cancelled_url = server.url("/queued-cancelled");
    let (actor, updates) = actor(bounded(1, 1, WAIT));

    assert!(actor.command(page(51, first_url)));
    assert_eq!(server.next_request(), "/active");
    assert!(actor.command(page(52, cancelled_url.clone())));
    assert!(actor.command(FetchCommand::CancelPage { request: 52 }));
    match next_update(&updates) {
        FetchUpdate::Page(outcome) => {
            assert_eq!(outcome.request, 52);
            assert!(matches!(outcome.result, Err(FetchFailure::Cancelled)));
        },
        _ => panic!("expected correlated queued cancellation result"),
    }
    assert!(matches!(server.seen.try_recv(), Err(TryRecvError::Empty)));
    let report = actor.close_and_join().unwrap();
    assert_eq!(report.admitted, 2);
    assert_eq!(report.tasks.joined() + report.discarded_before_spawn, 2);
    assert!(report.tasks.is_clean());
    assert!(matches!(next_update(&updates), FetchUpdate::Page(_)));
}

#[test]
fn zero_active_disables_work_and_zero_queue_sheds_instead_of_waiting() {
    let server = StalledHttp::start();
    let url = server.url("/disabled");
    let (actor, updates) = actor(bounded(0, 3, WAIT));

    assert!(!actor.command(page(61, url.clone())));
    match next_update(&updates) {
        FetchUpdate::Page(outcome) => {
            assert_eq!(outcome.request, 61);
            assert_eq!(outcome.url, url);
            assert!(matches!(outcome.result, Err(FetchFailure::Failed(_))));
        },
        _ => panic!("expected correlated disabled-capacity result"),
    }
    assert!(matches!(server.seen.try_recv(), Err(TryRecvError::Empty)));
    let report = actor.close_and_join().unwrap();
    assert_eq!(report.overloaded, 1);
    assert_eq!(report.tasks.joined(), 0);
    assert!(report.tasks.is_clean());
}

#[test]
fn invalid_capacity_and_zero_queue_deadline_are_rejected_before_startup() {
    let stores = Stores::in_memory();
    for admission in [
        bounded(usize::MAX, 1, WAIT),
        bounded(1, usize::MAX, WAIT),
        bounded(1, 1, Duration::ZERO),
    ] {
        assert!(
            spawn_fetcher_with_admission(
                wake(),
                stores.clone(),
                ClientFetchPolicy::default(),
                admission,
            )
            .is_err()
        );
    }
    assert!(
        spawn_fetcher_with_admission(
            wake(),
            stores,
            ClientFetchPolicy {
                max_page_bytes: 0,
                ..ClientFetchPolicy::default()
            },
            bounded(1, 0, Duration::ZERO),
        )
        .is_err()
    );
}
