// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Native loopback receipts for fetch actor admission and task lifetime.

use super::*;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::Barrier;
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::thread;
use std::thread::JoinHandle;

const WAIT: Duration = Duration::from_secs(3);

fn wake() -> Wake {
    Arc::new(|| {})
}

fn bounded(active: usize, pending: usize, queue_timeout: Duration) -> FetchActorPolicy {
    FetchActorPolicy {
        max_active: active,
        max_pending: pending,
        queue_timeout,
    }
}

/// A tiny HTTP endpoint which records each request before parking its response.
/// A finite park keeps failed assertions from leaking a permanently blocked
/// fixture thread.
struct StalledHttp {
    address: SocketAddr,
    seen: Receiver<String>,
    release: Option<Sender<()>>,
    stop: Arc<AtomicBool>,
    accept_thread: Option<thread::JoinHandle<()>>,
    children: Arc<Mutex<Vec<JoinHandle<()>>>>,
}

#[cfg(feature = "smolweb")]
struct StalledSpartan {
    address: SocketAddr,
    seen: Receiver<usize>,
    release: Option<Sender<()>>,
    stop: Arc<AtomicBool>,
    accept_thread: Option<thread::JoinHandle<()>>,
    children: Arc<Mutex<Vec<JoinHandle<()>>>>,
}

#[cfg(feature = "smolweb")]
impl StalledSpartan {
    fn start() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let address = listener.local_addr().unwrap();
        let (seen_tx, seen) = mpsc::channel();
        let (release, release_rx) = mpsc::channel();
        let release_rx = Arc::new(Mutex::new(release_rx));
        let stop = Arc::new(AtomicBool::new(false));
        let children = Arc::new(Mutex::new(Vec::new()));
        let accept_thread = {
            let (stop, release_rx, children) = (stop.clone(), release_rx.clone(), children.clone());
            thread::spawn(move || {
                while !stop.load(Ordering::Acquire) {
                    match listener.accept() {
                        Ok((stream, _)) => {
                            let seen = seen_tx.clone();
                            let release_rx = release_rx.clone();
                            children.lock().unwrap().push(thread::spawn(move || {
                                serve_spartan(stream, seen, release_rx)
                            }));
                        },
                        Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                            thread::sleep(Duration::from_millis(1));
                        },
                        Err(_) if stop.load(Ordering::Acquire) => break,
                        Err(error) => panic!("Spartan fixture accept failed: {error}"),
                    }
                }
            })
        };
        Self {
            address,
            seen,
            release: Some(release),
            stop,
            accept_thread: Some(accept_thread),
            children,
        }
    }

    fn url(&self) -> String {
        format!("spartan://{}/write", self.address)
    }
}

#[cfg(feature = "smolweb")]
impl Drop for StalledSpartan {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        self.release.take();
        if let Some(thread) = self.accept_thread.take() {
            thread.join().unwrap();
        }
        for child in std::mem::take(&mut *self.children.lock().unwrap()) {
            child.join().unwrap();
        }
    }
}

#[cfg(feature = "smolweb")]
fn serve_spartan(mut stream: TcpStream, seen: Sender<usize>, release: Arc<Mutex<Receiver<()>>>) {
    // Winsock can inherit the nonblocking listener mode on accept. Establish
    // blocking reads explicitly before relying on the fixture read deadline.
    stream.set_nonblocking(false).unwrap();
    let _ = stream.set_read_timeout(Some(WAIT));
    let mut line = Vec::new();
    let mut byte = [0];
    while stream.read_exact(&mut byte).is_ok() {
        if byte[0] == b'\n' {
            break;
        }
        line.push(byte[0]);
        if line.len() > 8192 {
            return;
        }
    }
    if line.is_empty() {
        return;
    }
    let body_len = String::from_utf8_lossy(&line)
        .split_whitespace()
        .last()
        .and_then(|count| count.parse::<usize>().ok())
        .unwrap_or(0);
    let mut body = vec![0; body_len];
    if stream.read_exact(&mut body).is_err() {
        return;
    }
    let _ = seen.send(body_len);
    if release
        .lock()
        .unwrap()
        .recv_timeout(Duration::from_secs(8))
        .is_ok()
    {
        let _ = stream.write_all(b"2 text/plain\r\nok");
    }
}

impl StalledHttp {
    fn start() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let address = listener.local_addr().unwrap();
        let (seen_tx, seen) = mpsc::channel();
        let (release, release_rx) = mpsc::channel();
        let release_rx = Arc::new(Mutex::new(release_rx));
        let stop = Arc::new(AtomicBool::new(false));
        let children = Arc::new(Mutex::new(Vec::new()));
        let accept_thread = {
            let (stop, release_rx, children) = (stop.clone(), release_rx.clone(), children.clone());
            thread::spawn(move || {
                while !stop.load(Ordering::Acquire) {
                    match listener.accept() {
                        Ok((stream, _)) => {
                            let seen = seen_tx.clone();
                            let release_rx = release_rx.clone();
                            children
                                .lock()
                                .unwrap()
                                .push(thread::spawn(move || serve_http(stream, seen, release_rx)));
                        },
                        Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                            thread::sleep(Duration::from_millis(1));
                        },
                        Err(_) if stop.load(Ordering::Acquire) => break,
                        Err(error) => panic!("loopback accept failed: {error}"),
                    }
                }
            })
        };
        Self {
            address,
            seen,
            release: Some(release),
            stop,
            accept_thread: Some(accept_thread),
            children,
        }
    }

    fn url(&self, path: &str) -> String {
        format!("http://{}{path}", self.address)
    }

    fn next_request(&self) -> String {
        self.seen.recv_timeout(WAIT).expect("loopback request")
    }

    fn release_one(&self) {
        self.release.as_ref().unwrap().send(()).unwrap();
    }
}

#[cfg(feature = "smolweb")]
struct GopherPeers {
    address: SocketAddr,
    accepted: Receiver<(usize, Sender<()>)>,
    eof: Receiver<usize>,
    stop: Arc<AtomicBool>,
    accept_thread: Option<JoinHandle<()>>,
    children: Arc<Mutex<Vec<JoinHandle<()>>>>,
}

#[cfg(feature = "smolweb")]
impl GopherPeers {
    fn start() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let address = listener.local_addr().unwrap();
        let (accepted_tx, accepted) = mpsc::channel();
        let (eof_tx, eof) = mpsc::channel();
        let next_id = Arc::new(AtomicUsize::new(1));
        let stop = Arc::new(AtomicBool::new(false));
        let children = Arc::new(Mutex::new(Vec::new()));
        let accept_thread = {
            let (stop, children, next_id) = (stop.clone(), children.clone(), next_id.clone());
            thread::spawn(move || {
                while !stop.load(Ordering::Acquire) {
                    match listener.accept() {
                        Ok((stream, _)) => {
                            let id = next_id.fetch_add(1, Ordering::Relaxed);
                            let (release, release_rx) = mpsc::channel();
                            let accepted = accepted_tx.clone();
                            let eof = eof_tx.clone();
                            let handler_release = release.clone();
                            children.lock().unwrap().push(thread::spawn(move || {
                                serve_gopher(stream, id, release_rx, handler_release, accepted, eof)
                            }));
                            drop(release);
                        },
                        Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                            thread::sleep(Duration::from_millis(1));
                        },
                        Err(_) if stop.load(Ordering::Acquire) => break,
                        Err(error) => panic!("Gopher fixture accept failed: {error}"),
                    }
                }
            })
        };
        Self {
            address,
            accepted,
            eof,
            stop,
            accept_thread: Some(accept_thread),
            children,
        }
    }

    fn url(&self) -> String {
        format!("gopher://{}/0same", self.address)
    }
}

#[cfg(feature = "smolweb")]
impl Drop for GopherPeers {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(thread) = self.accept_thread.take() {
            thread.join().unwrap();
        }
        for child in std::mem::take(&mut *self.children.lock().unwrap()) {
            child.join().unwrap();
        }
    }
}

#[cfg(feature = "smolweb")]
fn serve_gopher(
    mut stream: TcpStream,
    id: usize,
    release: Receiver<()>,
    release_tx: Sender<()>,
    accepted: Sender<(usize, Sender<()>)>,
    eof: Sender<usize>,
) {
    // Winsock can inherit the nonblocking listener mode on accept. Establish
    // blocking reads explicitly before relying on the fixture read deadline.
    stream.set_nonblocking(false).unwrap();
    let _ = stream.set_read_timeout(Some(WAIT));
    let mut request = Vec::new();
    let mut byte = [0];
    while stream.read_exact(&mut byte).is_ok() {
        request.push(byte[0]);
        if byte[0] == b'\n' {
            break;
        }
        if request.len() > 8192 {
            return;
        }
    }
    if request.is_empty() {
        return;
    }
    if accepted.send((id, release_tx)).is_err() {
        return;
    }
    let _ = stream.set_nonblocking(true);
    let mut probe = [0];
    let deadline = Instant::now() + WAIT;
    while Instant::now() < deadline {
        match stream.peek(&mut probe) {
            Ok(0) => {
                let _ = eof.send(id);
                return;
            },
            Ok(_) => {},
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {},
            Err(_) => {
                let _ = eof.send(id);
                return;
            },
        }
        match release.recv_timeout(Duration::from_millis(2)) {
            Ok(()) => {
                let _ = stream.set_nonblocking(false);
                let _ = stream.write_all(b"hello\r\n.\r\n");
                return;
            },
            Err(mpsc::RecvTimeoutError::Disconnected) => return,
            Err(mpsc::RecvTimeoutError::Timeout) => {},
        }
    }
}

impl Drop for StalledHttp {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        self.release.take();
        if let Some(thread) = self.accept_thread.take() {
            thread.join().unwrap();
        }
        for child in std::mem::take(&mut *self.children.lock().unwrap()) {
            child.join().unwrap();
        }
    }
}

fn serve_http(mut stream: TcpStream, seen: Sender<String>, release: Arc<Mutex<Receiver<()>>>) {
    // Winsock can inherit the nonblocking listener mode on accept. Establish
    // blocking reads explicitly before relying on the fixture read deadline.
    stream.set_nonblocking(false).unwrap();
    let _ = stream.set_read_timeout(Some(WAIT));
    let mut request = [0; 4096];
    let count = stream.read(&mut request).unwrap_or(0);
    if count == 0 {
        return;
    }
    let path = String::from_utf8_lossy(&request[..count])
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .unwrap_or("/")
        .to_owned();
    let _ = seen.send(path);
    if String::from_utf8_lossy(&request[..count]).contains("GET /stream ") {
        let _ = stream.write_all(
            b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nContent-Type: text/plain\r\nConnection: close\r\n\r\no",
        );
        let _ = stream.flush();
        if release
            .lock()
            .unwrap()
            .recv_timeout(Duration::from_secs(8))
            .is_ok()
        {
            let _ = stream.write_all(b"k");
        }
        return;
    }
    if String::from_utf8_lossy(&request[..count]).contains("GET /reservation-reused ") {
        let _ = stream.write_all(
            b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nContent-Type: text/plain\r\nConnection: close\r\n\r\nok",
        );
        return;
    }
    if release
        .lock()
        .unwrap()
        .recv_timeout(Duration::from_secs(8))
        .is_ok()
    {
        let _ = stream.write_all(
            b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nContent-Type: text/plain\r\nConnection: close\r\n\r\nok",
        );
    }
}

fn actor(policy: FetchActorPolicy) -> (FetchActor, Receiver<FetchUpdate>) {
    spawn_fetcher_with_admission(
        wake(),
        Stores::in_memory(),
        ClientFetchPolicy::default(),
        policy,
    )
    .expect("valid fetch actor policy")
}

fn page(request: FetchRequestId, url: String) -> FetchCommand {
    FetchCommand::Page {
        request,
        url,
        identity: None,
    }
}

fn next_update(updates: &Receiver<FetchUpdate>) -> FetchUpdate {
    updates.recv_timeout(WAIT).expect("fetch update")
}

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
    let mut client_policy = ClientFetchPolicy::default();
    client_policy.timeout = client_timeout;
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
    assert_eq!(report.tasks.joined(), 2);
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
    let wake: Wake = Arc::new(move || {
        let _ = wake_entered_tx.send(());
        let _ = wake_release_rx.recv_timeout(WAIT);
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
