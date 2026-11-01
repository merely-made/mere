//! Route B (D12): wit-bindgen-wrpc 0.12.0's bindings, our transport over
//! Windows named pipes (one connection per invocation, as wRPC's frame
//! protocol asks), and adapters to the shared contract. wRPC is async and
//! needs `Send`, so the blocking consumer drives a current-thread runtime and
//! the provider sits behind a mutex on its own thread.

#[allow(clippy::all, dead_code, unused)]
pub mod provider_bindings {
    include!(concat!(env!("OUT_DIR"), "/provider.rs"));
}

#[allow(clippy::all, dead_code, unused)]
pub mod consumer_bindings {
    include!(concat!(env!("OUT_DIR"), "/consumer.rs"));
}

use std::sync::{Arc, Mutex};

use c0_contract::{Api, Failure, Notice, Provider, Reply};
use tokio::io::BufReader;
use wit_bindgen_wrpc::anyhow;
use wit_bindgen_wrpc::bytes::Bytes;
use wit_bindgen_wrpc::futures::StreamExt;
use wit_bindgen_wrpc::wrpc_transport::frame::{Incoming, Outgoing};
use wit_bindgen_wrpc::wrpc_transport::{self, Invoke, ResourceBorrow, ResourceOwn};

use consumer_bindings::mere::script::projection_session as client;
use provider_bindings::exports::mere::script::projection_session as server;

macro_rules! failure_into {
    ($from:path => $to:path, $f:expr) => {{
        use $from as F;
        use $to as T;
        match $f {
            F::Revoked => T::Revoked,
            F::Expired => T::Expired,
            F::TooLarge(n) => T::TooLarge(n),
            F::Refused(s) => T::Refused(s),
            F::Malformed(s) => T::Malformed(s),
        }
    }};
}

fn to_wire(f: Failure) -> server::Failure {
    failure_into!(Failure => server::Failure, f)
}

fn from_wire(f: client::Failure) -> Failure {
    failure_into!(client::Failure => Failure, f)
}

fn handle(b: &[u8]) -> u64 {
    b.try_into().map(u64::from_le_bytes).unwrap_or(u64::MAX)
}

/// A provider served through wRPC's generated handlers. wRPC has no
/// resource-drop hook, so the provider never hears Close.
pub struct Serve<P>(pub Arc<Mutex<P>>);

impl<P> Clone for Serve<P> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

type Wire<T> = anyhow::Result<Result<T, server::Failure>>;

impl<P: Provider + Send + 'static> server::Handler<()> for Serve<P> {
    async fn discover(&self, _: ()) -> Wire<Bytes> {
        Ok(self
            .0
            .lock()
            .unwrap()
            .discover()
            .map(Bytes::from)
            .map_err(to_wire))
    }
    async fn open(&self, _: (), request: Bytes) -> Wire<(ResourceOwn<server::Session>, Bytes)> {
        Ok(self
            .0
            .lock()
            .unwrap()
            .open(&request)
            .map(|(h, opened)| {
                (
                    ResourceOwn::from(h.to_le_bytes().to_vec()),
                    Bytes::from(opened),
                )
            })
            .map_err(to_wire))
    }
}

impl<P: Provider + Send + 'static> server::HandlerSession<()> for Serve<P> {
    async fn snapshot(&self, _: (), s: ResourceBorrow<server::Session>, ask: Bytes) -> Wire<Bytes> {
        Ok(self
            .0
            .lock()
            .unwrap()
            .snapshot(handle(s.as_ref()), &ask)
            .map(Bytes::from)
            .map_err(to_wire))
    }
    async fn fetch(
        &self,
        _: (),
        s: ResourceBorrow<server::Session>,
        address: Bytes,
    ) -> Wire<Bytes> {
        Ok(self
            .0
            .lock()
            .unwrap()
            .fetch(handle(s.as_ref()), &address)
            .map(Bytes::from)
            .map_err(to_wire))
    }
    async fn chunk(
        &self,
        _: (),
        s: ResourceBorrow<server::Session>,
        address: Bytes,
        offset: u64,
        length: u32,
    ) -> Wire<Bytes> {
        Ok(self
            .0
            .lock()
            .unwrap()
            .chunk(handle(s.as_ref()), &address, offset, length)
            .map(Bytes::from)
            .map_err(to_wire))
    }
    async fn resume(
        &self,
        _: (),
        s: ResourceBorrow<server::Session>,
        epoch: u64,
        revision: u64,
    ) -> Wire<Bytes> {
        Ok(self
            .0
            .lock()
            .unwrap()
            .resume(handle(s.as_ref()), epoch, revision)
            .map(Bytes::from)
            .map_err(to_wire))
    }
    async fn intent(&self, _: (), s: ResourceBorrow<server::Session>, call: Bytes) -> Wire<Bytes> {
        Ok(self
            .0
            .lock()
            .unwrap()
            .intent(handle(s.as_ref()), &call)
            .map(Bytes::from)
            .map_err(to_wire))
    }
    async fn suspend(&self, _: (), s: ResourceBorrow<server::Session>) -> Wire<()> {
        Ok(self
            .0
            .lock()
            .unwrap()
            .suspend(handle(s.as_ref()))
            .map_err(to_wire))
    }
    async fn take_notice(
        &self,
        _: (),
        s: ResourceBorrow<server::Session>,
    ) -> anyhow::Result<Option<server::Notice>> {
        Ok(self
            .0
            .lock()
            .unwrap()
            .take_notice(handle(s.as_ref()))
            .map(|n| server::Notice {
                epoch: n.epoch,
                revision: n.revision,
            }))
    }
    async fn wait_notice(&self, _: (), s: ResourceBorrow<server::Session>) -> Wire<server::Notice> {
        Ok(self
            .0
            .lock()
            .unwrap()
            .wait_notice(handle(s.as_ref()))
            .map(|n| server::Notice {
                epoch: n.epoch,
                revision: n.revision,
            })
            .map_err(to_wire))
    }
}

#[cfg(windows)]
mod pipe {
    use super::*;
    use tokio::net::windows::named_pipe::{
        ClientOptions, NamedPipeClient, NamedPipeServer, ServerOptions,
    };

    type Server = wrpc_transport::frame::Server<
        (),
        BufReader<tokio::io::ReadHalf<NamedPipeServer>>,
        tokio::io::WriteHalf<NamedPipeServer>,
    >;

    /// The consumer's transport: a fresh pipe connection per invocation.
    #[derive(Clone)]
    pub struct Client(pub String);

    impl Invoke for Client {
        type Context = ();

        async fn invoke<P>(
            &self,
            (): (),
            instance: &str,
            func: &str,
            params: Bytes,
            paths: impl AsRef<[P]> + Send,
        ) -> anyhow::Result<(Outgoing, Incoming)>
        where
            P: AsRef<[Option<usize>]> + Send + Sync,
        {
            let conn: NamedPipeClient = loop {
                match ClientOptions::new().open(&self.0) {
                    Ok(c) => break c,
                    // ERROR_PIPE_BUSY: every instance is taken; the server makes another.
                    Err(e) if e.raw_os_error() == Some(231) => tokio::task::yield_now().await,
                    Err(e) => return Err(e.into()),
                }
            };
            let (rx, tx) = tokio::io::split(conn);
            wrpc_transport::frame::invoke(tx, BufReader::new(rx), instance, func, params, paths)
                .await
        }
    }

    /// Serve `handler` on a named pipe from its own thread.
    pub fn spawn<H>(name: String, handler: H) -> std::io::Result<()>
    where
        H: server::Handler<()> + server::HandlerSession<()> + Send + Sync + Clone + 'static,
    {
        let (ready_tx, ready_rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            rt.block_on(async move {
                let srv: Arc<Server> = Arc::new(Server::default());
                let streams = provider_bindings::serve(srv.as_ref(), handler)
                    .await
                    .unwrap();
                for (_, _, mut stream) in streams {
                    tokio::spawn(async move {
                        while let Some(Ok(call)) = stream.next().await {
                            tokio::spawn(call);
                        }
                    });
                }
                let mut next = ServerOptions::new()
                    .first_pipe_instance(true)
                    .create(&name)
                    .unwrap();
                ready_tx.send(()).unwrap();
                loop {
                    next.connect().await.unwrap();
                    let conn =
                        std::mem::replace(&mut next, ServerOptions::new().create(&name).unwrap());
                    let srv = srv.clone();
                    tokio::spawn(async move {
                        let (rx, tx) = tokio::io::split(conn);
                        let _ = srv.accept((), tx, BufReader::new(rx)).await;
                    });
                }
            });
        });
        ready_rx
            .recv()
            .map_err(|_| std::io::Error::other("server did not start"))
    }
}

#[cfg(windows)]
pub use pipe::{Client, spawn};

fn carrier(e: anyhow::Error) -> Failure {
    Failure::Refused(format!("carrier: {e:#}"))
}

/// The blocking consumer: each call drives a current-thread runtime.
pub struct Consumer<C> {
    pub wrpc: C,
    rt: tokio::runtime::Runtime,
}

impl<C> Consumer<C> {
    pub fn new(wrpc: C) -> Self {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        Self { wrpc, rt }
    }
}

type Own = ResourceOwn<client::Session>;

fn borrow(s: &Own) -> ResourceBorrow<client::Session> {
    ResourceBorrow::from(s.clone())
}

impl<C: Invoke<Context = ()>> Api for Consumer<C> {
    type Session = Own;
    fn discover(&mut self) -> Reply {
        let r = self
            .rt
            .block_on(client::discover(&self.wrpc, ()))
            .map_err(carrier)?;
        r.map(|b| b.to_vec()).map_err(from_wire)
    }
    fn open(&mut self, request: Vec<u8>) -> Result<(Own, Vec<u8>), Failure> {
        let request = Bytes::from(request);
        let r = self
            .rt
            .block_on(client::open(&self.wrpc, (), &request))
            .map_err(carrier)?;
        r.map(|(s, b)| (s, b.to_vec())).map_err(from_wire)
    }
    fn snapshot(&mut self, s: &Own, ask: Vec<u8>) -> Reply {
        let (s, ask) = (borrow(s), Bytes::from(ask));
        let r = self
            .rt
            .block_on(client::Session::snapshot(&self.wrpc, (), &s, &ask))
            .map_err(carrier)?;
        r.map(|b| b.to_vec()).map_err(from_wire)
    }
    fn fetch(&mut self, s: &Own, address: Vec<u8>) -> Reply {
        let (s, address) = (borrow(s), Bytes::from(address));
        let r = self
            .rt
            .block_on(client::Session::fetch(&self.wrpc, (), &s, &address))
            .map_err(carrier)?;
        r.map(|b| b.to_vec()).map_err(from_wire)
    }
    fn chunk(&mut self, s: &Own, address: Vec<u8>, offset: u64, length: u32) -> Reply {
        let (s, address) = (borrow(s), Bytes::from(address));
        let r = self
            .rt
            .block_on(client::Session::chunk(
                &self.wrpc,
                (),
                &s,
                &address,
                offset,
                length,
            ))
            .map_err(carrier)?;
        r.map(|b| b.to_vec()).map_err(from_wire)
    }
    fn resume(&mut self, s: &Own, epoch: u64, revision: u64) -> Reply {
        let s = borrow(s);
        let r = self
            .rt
            .block_on(client::Session::resume(&self.wrpc, (), &s, epoch, revision))
            .map_err(carrier)?;
        r.map(|b| b.to_vec()).map_err(from_wire)
    }
    fn intent(&mut self, s: &Own, call: Vec<u8>) -> Reply {
        let (s, call) = (borrow(s), Bytes::from(call));
        let r = self
            .rt
            .block_on(client::Session::intent(&self.wrpc, (), &s, &call))
            .map_err(carrier)?;
        r.map(|b| b.to_vec()).map_err(from_wire)
    }
    fn suspend(&mut self, s: &Own) -> Result<(), Failure> {
        let s = borrow(s);
        let r = self
            .rt
            .block_on(client::Session::suspend(&self.wrpc, (), &s))
            .map_err(carrier)?;
        r.map_err(from_wire)
    }
    fn take_notice(&mut self, s: &Own) -> Option<Notice> {
        let s = borrow(s);
        let n = self
            .rt
            .block_on(client::Session::take_notice(&self.wrpc, (), &s))
            .ok()??;
        Some(Notice {
            epoch: n.epoch,
            revision: n.revision,
        })
    }
    fn wait_notice(&mut self, s: &Own) -> Result<Notice, Failure> {
        let s = borrow(s);
        let r = self
            .rt
            .block_on(client::Session::wait_notice(&self.wrpc, (), &s))
            .map_err(carrier)?;
        r.map(|n| Notice {
            epoch: n.epoch,
            revision: n.revision,
        })
        .map_err(from_wire)
    }
    /// wRPC has no resource drop: nothing reaches the provider.
    fn close(&mut self, _: Own) {}
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;
    use c0_contract::provider::TestProvider;
    use c0_contract::script::{CALLS_BEFORE_REVOCATION, Direct, run};

    #[test]
    fn route_b_matches_the_reference_transcript() {
        let reference = run(&mut Direct(TestProvider::new(CALLS_BEFORE_REVOCATION)));
        let name = format!(r"\\.\pipe\c0-route-b-{}", std::process::id());
        let provider = Arc::new(Mutex::new(TestProvider::new(CALLS_BEFORE_REVOCATION)));
        spawn(name.clone(), Serve(provider.clone())).unwrap();
        let transcript = run(&mut Consumer::new(Client(name)));
        assert_eq!(transcript, reference);
        // The fit finding: Close never reached the provider.
        assert!(provider.lock().unwrap().closed.is_empty());
    }
}
