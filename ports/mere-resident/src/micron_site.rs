// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Native NomadNet transport for the governed Micron endpoint. The host owns
//! interface admission, sessions, identity and shutdown; Retinue owns framing,
//! endpoint cryptography and the request/Resource wire exchange.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NomadNetListenerPolicyV1 {
    /// Loopback-only TCP Reticulum interface; zero selects an available port.
    pub port: u16,
    /// Accepted TCP interfaces, including those without an established link.
    pub max_connections: u16,
    /// Admitted resource-capable links and host-owned session tasks.
    pub max_sessions: u16,
    /// Whole session lifetime, including reused requests.
    pub session_timeout_ms: u64,
    /// Whole request receive/reply deadline; local synchronous owner I/O
    /// cannot be preempted by this asynchronous deadline.
    pub request_timeout_ms: u64,
    pub shutdown_grace_ms: u64,
    /// Inbound packed requests. Retinue ignores oversized request packets and
    /// rejects oversized request Resources before application dispatch.
    pub max_request_bytes: usize,
    /// Inbound Resource reassembly only. Outgoing saved pages retain the
    /// shared site's MAX_PAGE_BYTES bound and are not capped by this setting.
    pub max_resource_bytes: usize,
    pub resume_on_restart: bool,
}

impl Default for NomadNetListenerPolicyV1 {
    fn default() -> Self {
        Self {
            port: 0,
            max_connections: 8,
            max_sessions: 16,
            session_timeout_ms: 60_000,
            request_timeout_ms: 30_000,
            shutdown_grace_ms: 1_000,
            max_request_bytes: 64 * 1024,
            max_resource_bytes: 128 * 1024,
            resume_on_restart: false,
        }
    }
}

impl NomadNetListenerPolicyV1 {
    pub(super) fn validate(&self) -> Result<(), String> {
        for (name, value, maximum) in [
            ("max_connections", u64::from(self.max_connections), 32),
            ("max_sessions", u64::from(self.max_sessions), 64),
            ("session_timeout_ms", self.session_timeout_ms, 300_000),
            ("request_timeout_ms", self.request_timeout_ms, 60_000),
            ("shutdown_grace_ms", self.shutdown_grace_ms, 30_000),
            (
                "max_request_bytes",
                self.max_request_bytes as u64,
                1024 * 1024,
            ),
            (
                "max_resource_bytes",
                self.max_resource_bytes as u64,
                4 * 1024 * 1024,
            ),
        ] {
            if value == 0 || value > maximum {
                return Err(format!("NomadNet {name} must be 1–{maximum}"));
            }
        }
        if self.max_request_bytes > self.max_resource_bytes {
            return Err("NomadNet max_request_bytes must not exceed max_resource_bytes".into());
        }
        Ok(())
    }
}

#[cfg(feature = "moot-site-nomadnet")]
mod native {
    use super::*;
    use crate::resident_site::{load_json, save_json_atomic};
    use rand_core::RngCore;
    use retinue::{
        endpoint::{Endpoint, InboundLinkLimits, ResourceSession, ResourceTransferConfig},
        identity::PrivateIdentity,
        nomadnet::StaticNode,
        request::Request,
    };
    use std::{
        net::{Ipv4Addr, SocketAddr},
        path::Path,
        sync::{
            Arc,
            atomic::{AtomicBool, Ordering},
        },
        thread,
        time::Duration,
    };
    use tokio::sync::watch;

    const IDENTITY: &str = "moot-site-nomadnet-identity.json";

    #[derive(Serialize, Deserialize)]
    #[serde(deny_unknown_fields)]
    struct IdentityV1 {
        version: u8,
        secret: Vec<u8>,
    }

    pub(crate) fn identity(
        root: &Path,
        expected: Option<[u8; 16]>,
    ) -> Result<PrivateIdentity, String> {
        let path = root.join(IDENTITY);
        let mut stored = if path.exists() {
            load_json::<IdentityV1>(&path, 1024)?
        } else {
            if expected.is_some() {
                return Err("NomadNet identity is missing; restore its original key".into());
            }
            let mut secret = vec![0; 64];
            rand_core::OsRng.fill_bytes(&mut secret);
            let stored = IdentityV1 { version: 1, secret };
            save_json_atomic(&path, &stored)?;
            stored
        };
        if stored.version != 1 || stored.secret.len() != 64 {
            return Err("Invalid NomadNet identity".into());
        }
        let mut secret: [u8; 64] = stored
            .secret
            .as_slice()
            .try_into()
            .expect("checked identity length");
        let identity = PrivateIdentity::from_secret_bytes(&secret);
        secret.fill(0);
        stored.secret.fill(0);
        if expected.is_some_and(|expected| destination(&identity) != expected) {
            return Err("NomadNet identity does not match its descriptor".into());
        }
        Ok(identity)
    }

    pub(crate) fn destination(identity: &PrivateIdentity) -> [u8; 16] {
        *StaticNode::new().destination(identity.public()).as_bytes()
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[tokio::test]
        async fn native_interface_capacity_recovers_and_stalled_sessions_meet_deadlines() {
            for session_deadline in [false, true] {
                let private = PrivateIdentity::from_secret_bytes(&[0x42; 64]);
                let peer = *private.public();
                let destination = StaticNode::new().destination(&peer);
                let policy = NomadNetListenerPolicyV1 {
                    max_connections: 1,
                    request_timeout_ms: if session_deadline { 2_000 } else { 500 },
                    session_timeout_ms: if session_deadline { 500 } else { 2_000 },
                    ..Default::default()
                };
                let mut running = start(|_| Vec::new(), |_| (), &policy, private).unwrap();
                running.activate();
                let first = std::net::TcpStream::connect(running.address).unwrap();
                let mut refused = std::net::TcpStream::connect(running.address).unwrap();
                refused
                    .set_read_timeout(Some(Duration::from_secs(2)))
                    .unwrap();
                assert_eq!(std::io::Read::read(&mut refused, &mut [0u8; 1]).unwrap(), 0);
                drop(refused);
                drop(first);

                let deadline = std::time::Instant::now() + Duration::from_secs(5);
                let (client, mut session) = loop {
                    let client = Endpoint::new(PrivateIdentity::from_secret_bytes(&[0x43; 64]));
                    let attempt = tokio::time::timeout(Duration::from_millis(500), async {
                        client.attach_tcp_client(running.address).await?;
                        client.open_resource(destination, peer).await
                    })
                    .await;
                    if let Ok(Ok(session)) = attempt {
                        break (client, session);
                    }
                    client.close();
                    assert!(
                        std::time::Instant::now() < deadline,
                        "carrier admission must recover after release"
                    );
                };
                let closed =
                    tokio::time::timeout(Duration::from_secs(3), session.receive_raw_request())
                        .await
                        .expect("stalled admitted link must close at its deadline");
                assert!(closed.is_err());
                drop(session);
                client.close();
                let report = running.close_and_join().unwrap();
                assert!(report.is_clean());
                assert!(report.completed >= 1);
            }
        }

        #[test]
        fn persistent_destination_reopens_and_refuses_key_loss_or_replacement() {
            let root = tempfile::tempdir().unwrap();
            let original = identity(root.path(), None).unwrap();
            let expected = destination(&original);
            let reopened = identity(root.path(), Some(expected)).unwrap();
            assert_eq!(
                original.public().to_public_bytes(),
                reopened.public().to_public_bytes()
            );
            assert!(!root.path().join("published-site-identity.json").exists());
            let key = root.path().join(IDENTITY);
            let original_bytes = std::fs::read(&key).unwrap();
            std::fs::write(&key, b"{\"version\":1,\"secret\":[]}").unwrap();
            assert!(identity(root.path(), Some(expected)).is_err());
            let other = tempfile::tempdir().unwrap();
            identity(other.path(), None).unwrap();
            std::fs::copy(other.path().join(IDENTITY), &key).unwrap();
            assert!(identity(root.path(), Some(expected)).is_err());
            std::fs::write(&key, original_bytes).unwrap();
            assert!(identity(root.path(), Some(expected)).is_ok());
            std::fs::remove_file(&key).unwrap();
            assert!(identity(root.path(), Some(expected)).is_err());
            assert!(!key.exists());
        }
    }

    pub(crate) struct RunningNomadNet {
        pub address: SocketAddr,
        ready: Arc<AtomicBool>,
        shutdown: Option<watch::Sender<bool>>,
        worker: Option<thread::JoinHandle<armillary::tasks::TaskReport>>,
    }

    impl RunningNomadNet {
        pub fn activate(&self) {
            self.ready.store(true, Ordering::Release);
        }

        pub fn close_and_join(&mut self) -> Result<armillary::tasks::TaskReport, String> {
            self.ready.store(false, Ordering::Release);
            if let Some(shutdown) = self.shutdown.take() {
                let _ = shutdown.send(true);
            }
            let report = match self.worker.take() {
                Some(worker) => worker.join().map_err(|_| "NomadNet worker panicked")?,
                None => armillary::tasks::TaskReport::default(),
            };
            if !report.is_clean() {
                return Err(format!(
                    "NomadNet tasks joined with {} panics and {} refused admissions",
                    report.panicked, report.rejected
                ));
            }
            Ok(report)
        }
    }

    impl Drop for RunningNomadNet {
        fn drop(&mut self) {
            if let Err(error) = self.close_and_join() {
                tracing::warn!(%error, "NomadNet task shutdown failed");
            }
        }
    }

    async fn serve<F, R>(
        mut session: ResourceSession,
        reply: Arc<F>,
        refuse: Arc<R>,
        ready: Arc<AtomicBool>,
        policy: NomadNetListenerPolicyV1,
    ) where
        F: Fn(&Request) -> Vec<u8> + Send + Sync + 'static,
        R: Fn(&str) + Send + Sync + 'static,
    {
        session.set_config(ResourceTransferConfig {
            timeout: Duration::from_millis(policy.request_timeout_ms),
            ..Default::default()
        });
        session.set_max_request_size(Some(policy.max_request_bytes));
        session.set_max_resource_size(policy.max_resource_bytes);
        loop {
            let exchange = async {
                let request = session.receive_raw_request().await?;
                // Retinue's byte profile accepts nil/empty binary, rejects maps.
                // Its existing trailing-byte tolerance remains supplier-owned.
                let body = if ready.load(Ordering::Acquire) {
                    match Request::unpack(&request.packed) {
                        Ok(request) => reply(&request),
                        Err(_) => {
                            refuse("Static Micron pages reject map or malformed requests");
                            Vec::new()
                        },
                    }
                } else {
                    refuse("Publication not committed");
                    Vec::new()
                };
                // Once a successful response starts, a wire failure closes this
                // session; never append an empty fallback after partial output.
                session.respond_auto(request.request_id, body).await?;
                Ok::<_, std::io::Error>(())
            };
            if !matches!(
                tokio::time::timeout(Duration::from_millis(policy.request_timeout_ms), exchange)
                    .await,
                Ok(Ok(()))
            ) {
                return;
            }
        }
    }

    pub(crate) fn start<F, R>(
        reply: F,
        refuse: R,
        policy: &NomadNetListenerPolicyV1,
        identity: PrivateIdentity,
    ) -> Result<RunningNomadNet, String>
    where
        F: Fn(&Request) -> Vec<u8> + Send + Sync + 'static,
        R: Fn(&str) + Send + Sync + 'static,
    {
        policy.validate()?;
        let listener = std::net::TcpListener::bind((Ipv4Addr::LOCALHOST, policy.port))
            .map_err(|e| e.to_string())?;
        listener.set_nonblocking(true).map_err(|e| e.to_string())?;
        let address = listener.local_addr().map_err(|e| e.to_string())?;
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| e.to_string())?;
        let (shutdown, mut stopped) = watch::channel(false);
        let ready = Arc::new(AtomicBool::new(false));
        let (ready_tx, ready_rx) = std::sync::mpsc::sync_channel(1);
        let policy = policy.clone();
        let worker = thread::spawn({
            let ready = ready.clone();
            move || {
                let report = runtime.block_on(async move {
                    let listener = match tokio::net::TcpListener::from_std(listener) {
                        Ok(listener) => listener,
                        Err(error) => { let _ = ready_tx.send(Err(error.to_string())); return Default::default(); }
                    };
                    let endpoint = Arc::new(Endpoint::new(identity));
                    endpoint.set_inbound_link_limits(InboundLinkLimits { total: policy.max_sessions.into(), per_destination: policy.max_sessions.into() });
                    let node = StaticNode::new();
                    let destination = node.destination(endpoint.identity());
                    node.register(&endpoint);
                    let slots = Arc::new(tokio::sync::Semaphore::new(policy.max_sessions.into()));
                    let tasks = armillary::tasks::ResidentTasks::new();
                    let reply = Arc::new(reply);
                    let refuse = Arc::new(refuse);
                    let _ = ready_tx.send(Ok(()));
                    loop {
                        tokio::select! {
                            biased;
                            _ = stopped.changed() => break,
                            accepted = endpoint.accept_resource() => {
                                let Ok(accepted) = accepted else { break; };
                                if accepted.destination != destination { continue; }
                                let Ok(permit) = slots.clone().try_acquire_owned() else { continue; };
                                let reply = reply.clone();
                                let refuse = refuse.clone();
                                let ready = ready.clone();
                                let policy = policy.clone();
                                tasks.spawn(async move {
                                    let _permit = permit;
                                    let _ = tokio::time::timeout(Duration::from_millis(policy.session_timeout_ms), serve(accepted.session, reply, refuse, ready, policy)).await;
                                });
                            }
                            accepted = listener.accept() => match accepted {
                                Ok((stream, _)) if endpoint.interface_count() < usize::from(policy.max_connections) => { endpoint.attach_stream(stream); },
                                Ok(_) => {},
                                Err(error) => tracing::warn!(%error, "NomadNet interface accept failed"),
                            },

                        }
                    }
                    drop(listener);
                    let report = tasks.close_and_join().await;
                    endpoint.shutdown(Duration::from_millis(policy.shutdown_grace_ms)).await;
                    drop(endpoint);
                    report
                });
                // Retinue tracks internal transport tasks. Dropping this dedicated
                // runtime waits for their cancellation before OS worker completion.
                drop(runtime);
                report
            }
        });
        match ready_rx.recv().map_err(|e| e.to_string()).and_then(|r| r) {
            Ok(()) => Ok(RunningNomadNet {
                address,
                ready,
                shutdown: Some(shutdown),
                worker: Some(worker),
            }),
            Err(error) => {
                let _ = shutdown.send(true);
                let _ = worker.join();
                Err(error)
            },
        }
    }
}

#[cfg(feature = "moot-site-nomadnet")]
pub(super) use native::{RunningNomadNet, destination, identity, start};

#[cfg(test)]
mod policy_tests {
    use super::*;

    #[test]
    fn native_policy_has_positive_bounded_limits_and_explicit_restart() {
        let policy = NomadNetListenerPolicyV1::default();
        assert!(policy.validate().is_ok());
        assert!(!policy.resume_on_restart);
        for invalid in [
            NomadNetListenerPolicyV1 {
                max_connections: 0,
                ..policy.clone()
            },
            NomadNetListenerPolicyV1 {
                max_sessions: 65,
                ..policy.clone()
            },
            NomadNetListenerPolicyV1 {
                session_timeout_ms: 300_001,
                ..policy.clone()
            },
            NomadNetListenerPolicyV1 {
                request_timeout_ms: 0,
                ..policy.clone()
            },
            NomadNetListenerPolicyV1 {
                shutdown_grace_ms: 30_001,
                ..policy.clone()
            },
            NomadNetListenerPolicyV1 {
                max_request_bytes: 0,
                ..policy.clone()
            },
            NomadNetListenerPolicyV1 {
                max_resource_bytes: 4 * 1024 * 1024 + 1,
                ..policy.clone()
            },
            NomadNetListenerPolicyV1 {
                max_resource_bytes: 1,
                ..policy
            },
        ] {
            assert!(invalid.validate().is_err());
        }
    }
}
