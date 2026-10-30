//! Host controls for the Errand Gemini listener's fixed policy and lifetime.

use super::*;
use gemini_protocol::server::{Reply, Request};
use rustls::pki_types::{CertificateDer, ServerName};
use std::{
    io::{Read, Write},
    net::TcpStream,
    sync::{Arc, mpsc},
    time::Instant,
};

fn identity(root: &Path) -> GeminiIdentityV1 {
    persistent_gemini_identity(root, None).unwrap()
}

fn policy(max_connections: u16, request_timeout_ms: u64) -> GeminiListenerPolicyV1 {
    GeminiListenerPolicyV1 {
        port: 0,
        max_connections,
        request_timeout_ms,
        resume_on_restart: false,
    }
}

fn tls_client(
    address: SocketAddr,
    identity: &GeminiIdentityV1,
    timeout: Duration,
) -> rustls::StreamOwned<rustls::ClientConnection, TcpStream> {
    let socket = TcpStream::connect_timeout(&address, Duration::from_secs(2)).unwrap();
    let mut tls = tls_client_on_socket(socket, identity, timeout);
    // Flush performs and completes the TLS handshake without sending a Gemini
    // request, so callers can hold an admitted TLS connection deterministically.
    tls.flush().unwrap();
    assert!(!tls.conn.is_handshaking());
    tls
}

fn tls_client_on_socket(
    socket: TcpStream,
    identity: &GeminiIdentityV1,
    timeout: Duration,
) -> rustls::StreamOwned<rustls::ClientConnection, TcpStream> {
    let mut roots = rustls::RootCertStore::empty();
    roots
        .add(CertificateDer::from(identity.certificate_der.clone()))
        .unwrap();
    let config = rustls::ClientConfig::builder_with_provider(Arc::new(
        rustls::crypto::ring::default_provider(),
    ))
    .with_safe_default_protocol_versions()
    .unwrap()
    .with_root_certificates(roots)
    .with_no_client_auth();
    let connection =
        rustls::ClientConnection::new(Arc::new(config), ServerName::try_from("localhost").unwrap())
            .unwrap();
    socket.set_read_timeout(Some(timeout)).unwrap();
    socket.set_write_timeout(Some(timeout)).unwrap();
    rustls::StreamOwned::new(connection, socket)
}

fn read_tls_to_end(tls: &mut rustls::StreamOwned<rustls::ClientConnection, TcpStream>) -> Vec<u8> {
    let mut bytes = Vec::new();
    match tls.read_to_end(&mut bytes) {
        Ok(_) => bytes,
        Err(error)
            if matches!(
                error.kind(),
                std::io::ErrorKind::UnexpectedEof
                    | std::io::ErrorKind::ConnectionReset
                    | std::io::ErrorKind::ConnectionAborted
            ) =>
        {
            bytes
        },
        Err(error) => panic!("TLS read failed: {error}"),
    }
}

#[test]
fn listener_policy_keeps_the_existing_default_and_bounds() {
    let default = GeminiListenerPolicyV1::default();
    assert_eq!(default.max_connections, 8);
    assert_eq!(default.request_timeout_ms, 2_000);
    assert!(!default.resume_on_restart);
    assert!(default.validate().is_ok());

    for (connections, timeout) in [(1, 1), (32, 30_000)] {
        assert!(policy(connections, timeout).validate().is_ok());
    }
    for (connections, timeout) in [(0, 2_000), (33, 2_000), (8, 0), (8, 30_001)] {
        assert!(policy(connections, timeout).validate().is_err());
    }
}

#[test]
fn overload_is_dropped_after_accept_while_the_tls_task_holds_capacity() {
    let temp = tempfile::tempdir().unwrap();
    let identity = identity(temp.path());
    let mut server = start_gemini_handler(
        |_| |_request: Request| std::future::ready(Reply::gemtext(b"ok".to_vec())),
        &policy(1, 5_000),
        &identity,
    )
    .unwrap();
    server.activate();

    let mut admitted = tls_client(server.address, &identity, Duration::from_secs(2));
    write!(admitted, "gemini://localhost:{}/", server.address.port()).unwrap();
    admitted.flush().unwrap();

    // The first completed TLS handshake plus incomplete URL proves its task
    // occupies the only permit. A second accepted TCP peer is closed at once.
    let mut overflow = TcpStream::connect_timeout(&server.address, Duration::from_secs(2)).unwrap();
    overflow
        .set_read_timeout(Some(Duration::from_secs(2)))
        .unwrap();
    let mut byte = [0_u8; 1];
    match overflow.read(&mut byte) {
        Ok(0) => {},
        Err(error)
            if matches!(
                error.kind(),
                std::io::ErrorKind::ConnectionReset | std::io::ErrorKind::ConnectionAborted
            ) => {},
        result => panic!("overloaded peer was not closed: {result:?}"),
    }

    let report = server.close_and_join().unwrap();
    assert_eq!(report.cancelled, 1);
    assert_eq!(report.panicked, 0);
    assert_eq!(report.rejected, 0);
}

#[test]
fn accepted_request_deadline_covers_tls_wait_and_handler_execution() {
    let temp = tempfile::tempdir().unwrap();
    let identity = identity(temp.path());
    let (entered_tx, entered_rx) = mpsc::channel();
    let handler = move |_request: Request| {
        let entered = entered_tx.clone();
        async move {
            let _ = entered.send(());
            tokio::time::sleep(Duration::from_secs(5)).await;
            Reply::gemtext(b"late response".to_vec())
        }
    };
    let total_timeout = Duration::from_secs(3);
    let mut server = start_gemini_handler(
        |_| handler,
        &policy(1, total_timeout.as_millis() as u64),
        &identity,
    )
    .unwrap();
    server.activate();

    let socket = TcpStream::connect_timeout(&server.address, Duration::from_secs(2)).unwrap();
    // A second peer is shed only after the first owns the sole permit. This
    // observes admission before charging time to the held TLS handshake.
    let mut overflow = TcpStream::connect_timeout(&server.address, Duration::from_secs(2)).unwrap();
    overflow
        .set_read_timeout(Some(Duration::from_secs(1)))
        .unwrap();
    match overflow.read(&mut [0]) {
        Ok(0) => {},
        Err(error)
            if matches!(
                error.kind(),
                std::io::ErrorKind::ConnectionReset | std::io::ErrorKind::ConnectionAborted
            ) => {},
        result => panic!("first task was not observed at capacity: {result:?}"),
    }
    let began = Instant::now();
    let mut tls = tls_client_on_socket(socket, &identity, Duration::from_secs(4));
    // Leave TLS acceptance pending for half the absolute allowance. A fresh
    // timeout after the handshake or handler would run too long.
    std::thread::sleep(Duration::from_millis(1_500));
    tls.flush().unwrap();
    assert!(!tls.conn.is_handshaking());
    write!(tls, "gemini://localhost:{}/\r\n", server.address.port()).unwrap();
    tls.flush().unwrap();
    entered_rx
        .recv_timeout(Duration::from_secs(1))
        .expect("the accepted request reached its handler");

    let response = read_tls_to_end(&mut tls);
    let elapsed = began.elapsed();
    assert!(response.is_empty(), "timed-out request wrote a response");
    assert!(elapsed >= Duration::from_millis(2_200), "{elapsed:?}");
    assert!(
        elapsed < Duration::from_millis(3_800),
        "deadline restarted by phase: {elapsed:?}"
    );

    let report = server.close_and_join().unwrap();
    assert_eq!(report.panicked, 0);
    assert_eq!(report.rejected, 0);
}

#[test]
fn close_and_join_cancels_active_handler_and_returns_armillary_report() {
    struct DropNotice(mpsc::Sender<()>);
    impl Drop for DropNotice {
        fn drop(&mut self) {
            let _ = self.0.send(());
        }
    }

    let temp = tempfile::tempdir().unwrap();
    let identity = identity(temp.path());
    let (entered_tx, entered_rx) = mpsc::channel();
    let (dropped_tx, dropped_rx) = mpsc::channel();
    let handler = move |_request: Request| {
        let entered = entered_tx.clone();
        let notice = DropNotice(dropped_tx.clone());
        async move {
            let _notice = notice;
            let _ = entered.send(());
            std::future::pending::<Reply>().await
        }
    };
    let mut server = start_gemini_handler(|_| handler, &policy(1, 10_000), &identity).unwrap();
    server.activate();
    let mut tls = tls_client(server.address, &identity, Duration::from_secs(2));
    write!(tls, "gemini://localhost:{}/\r\n", server.address.port()).unwrap();
    tls.flush().unwrap();
    entered_rx
        .recv_timeout(Duration::from_secs(2))
        .expect("the handler started before shutdown");

    let report = server.close_and_join().unwrap();
    dropped_rx
        .try_recv()
        .expect("handler dropped before close_and_join returned");
    assert_eq!(report.cancelled, 1);
    assert_eq!(report.panicked, 0);
    assert_eq!(report.rejected, 0);
    assert_eq!(report.joined(), 1);
}

#[test]
fn close_and_join_reports_an_admitted_handler_panic() {
    let temp = tempfile::tempdir().unwrap();
    let identity = identity(temp.path());
    let (entered_tx, entered_rx) = mpsc::channel();
    let handler = move |_request: Request| {
        let _ = entered_tx.send(());
        panic!("deliberate adopted-handler panic");
        #[allow(unreachable_code)]
        std::future::ready(Reply::gemtext(Vec::new()))
    };
    let mut server = start_gemini_handler(|_| handler, &policy(1, 5_000), &identity).unwrap();
    server.activate();
    let mut tls = tls_client(server.address, &identity, Duration::from_secs(2));
    write!(tls, "gemini://localhost:{}/\r\n", server.address.port()).unwrap();
    tls.flush().unwrap();
    entered_rx
        .recv_timeout(Duration::from_secs(2))
        .expect("the panicking handler was admitted");

    let error = server.close_and_join().unwrap_err();
    assert!(error.contains("1 panics"), "{error}");
}
