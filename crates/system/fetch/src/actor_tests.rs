// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
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

#[path = "actor_tests/admission.rs"]
mod admission;

#[path = "actor_tests/cancellation.rs"]
mod cancellation;

#[path = "actor_tests/shutdown.rs"]
mod shutdown;
