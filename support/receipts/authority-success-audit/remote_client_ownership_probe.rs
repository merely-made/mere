#![allow(dead_code,private_interfaces)]
extern crate self as burn_ir;
pub trait BackendIr {type Handle;}
pub type HandleKind<B> = <B as BackendIr>::Handle;
struct Cpu;impl BackendIr for Cpu{type Handle=i32;}
mod shared {
 pub type SessionId=u64;
 pub type RequestId=u64;
 pub enum TaskResponseContent {SyncBackend(Result<(),()>)}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LocalTransferId(u64);

impl LocalTransferId {
    #[cfg(feature = "client")]
    pub fn next(&mut self) {
        self.0 += 1;
    }
}

impl From<u64> for LocalTransferId {
    fn from(value: u64) -> Self {
        Self(value)
    }
}


}
fn finishes_within(timeout: std::time::Duration, body: impl FnOnce() + Send + 'static) -> bool {
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(body));
            let _ = tx.send(());
        });
        rx.recv_timeout(timeout).is_ok()
    }
mod pending {
//! Request/response correlation.

use crate::shared::{RequestId, TaskResponseContent};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use tokio::sync::oneshot;

/// The callbacks awaiting replies, plus whether the connection is still live.
///
/// Both live under one mutex so that registering a callback and tearing every callback down on
/// disconnect are mutually exclusive: a registration can't slip into the map *after* the
/// response-demux task has already drained it on disconnect and left it to wait for a reply that
/// will never come.
struct State {
    callbacks: HashMap<RequestId, oneshot::Sender<TaskResponseContent>>,
    /// Cleared by [`Responder::disconnect`] once the response stream is gone; gates new
    /// registrations so a post-disconnect request fails fast instead of parking forever.
    connected: bool,
}

type SharedState = Arc<Mutex<State>>;

/// Correlates response-producing requests with the caller awaiting each one.
///
/// Each response-producing task ([`ReadTensor`](crate::shared::Task::ReadTensor),
/// `SyncBackend`, `DTypeUsage`, `ProfileStart`, `ProfileEnd`) carries a [`RequestId`]; the
/// server echoes it on the
/// response. The runner thread [`register`](Self::register)s a [`oneshot`] callback before
/// sending the task, and the response-demux task delivers the reply through a [`Responder`].
///
/// The state is guarded by a plain [`std::sync::Mutex`]: the lock is only ever held for a single
/// insert/remove/drain and never across an `.await`, so neither the runner thread nor the demux
/// task needs the tokio runtime to touch it.
pub(crate) struct PendingResponses {
    state: SharedState,
    next_id: RequestId,
}

impl PendingResponses {
    pub(crate) fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(State {
                callbacks: HashMap::new(),
                connected: true,
            })),
            next_id: 0,
        }
    }

    /// Allocate the next monotonic [`RequestId`].
    pub(crate) fn next_id(&mut self) -> RequestId {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// Register a callback for `id`, returning the receiver the caller awaits for the reply.
    ///
    /// If the connection has already dropped (the demux task called [`Responder::disconnect`]),
    /// the sender is dropped immediately, so the returned receiver resolves to a `RecvError` right
    /// away rather than blocking on a response the dead server will never send.
    pub(crate) fn register(&self, id: RequestId) -> oneshot::Receiver<TaskResponseContent> {
        let (tx, rx) = oneshot::channel();
        let mut state = self.state.lock().unwrap();
        if state.connected {
            state.callbacks.insert(id, tx);
        }
        // Disconnected: drop `tx` here, leaving `rx` already-closed.
        rx
    }

    /// A cheap, cloneable handle the response-demux task uses to deliver replies.
    pub(crate) fn responder(&self) -> Responder {
        Responder {
            state: self.state.clone(),
        }
    }
}

/// Delivers responses to the callbacks registered in [`PendingResponses`]. Held by the
/// response-demux task, decoupled from the [`PendingResponses`] the runner thread owns.
pub(crate) struct Responder {
    state: SharedState,
}

impl Responder {
    /// Deliver `content` to the caller waiting on `id`. Returns `false` if no callback is
    /// registered (unknown id, or the caller dropped its receiver), in which case the
    /// response is discarded.
    pub(crate) fn complete(&self, id: RequestId, content: TaskResponseContent) -> bool {
        match self.state.lock().unwrap().callbacks.remove(&id) {
            Some(tx) => {
                // Receiver dropped is fine (caller no longer cares).
                let _ = tx.send(content);
                true
            },
            None => false,
        }
    }

    /// Mark the connection dropped and fail every pending caller.
    ///
    /// Called by the response-demux task when the response stream closes or errors (server down,
    /// connection reset). Clearing the map drops every registered sender, so each waiting receiver
    /// resolves to a `RecvError` that the callers (`sync`, `read_tensor`, `dtype_usage`) translate
    /// into an error instead of blocking forever. Clearing `connected` makes any later request
    /// fail fast in [`PendingResponses::register`] rather than registering a doomed callback.
    pub(crate) fn disconnect(&self) {
        let mut state = self.state.lock().unwrap();
        state.connected = false;
        state.callbacks.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn content() -> TaskResponseContent {
        TaskResponseContent::SyncBackend(Ok(()))
    }

    #[test]
    fn next_id_is_monotonic() {
        let mut pending = PendingResponses::new();
        assert_eq!(pending.next_id(), 0);
        assert_eq!(pending.next_id(), 1);
        assert_eq!(pending.next_id(), 2);
    }

    #[test]
    fn register_then_complete_delivers_to_receiver() {
        let mut pending = PendingResponses::new();
        let id = pending.next_id();
        let mut rx = pending.register(id);

        assert!(pending.responder().complete(id, content()));
        // `try_recv` resolves synchronously once the sender has fired — no runtime needed.
        assert!(matches!(
            rx.try_recv(),
            Ok(TaskResponseContent::SyncBackend(Ok(())))
        ));
    }

    #[test]
    fn complete_unknown_id_returns_false() {
        let pending = PendingResponses::new();
        assert!(!pending.responder().complete(42, content()));
    }

    #[test]
    fn complete_consumes_the_callback() {
        let mut pending = PendingResponses::new();
        let id = pending.next_id();
        let _rx = pending.register(id);

        assert!(pending.responder().complete(id, content()));
        // The callback was removed on first delivery; a duplicate response finds nothing.
        assert!(!pending.responder().complete(id, content()));
    }

    #[test]
    fn disconnect_fails_every_pending_caller() {
        let mut pending = PendingResponses::new();
        let id0 = pending.next_id();
        let id1 = pending.next_id();
        let mut rx0 = pending.register(id0);
        let mut rx1 = pending.register(id1);

        // The response stream died with both requests still in flight.
        pending.responder().disconnect();

        // Both receivers resolve immediately with an error instead of hanging.
        assert!(matches!(
            rx0.try_recv(),
            Err(oneshot::error::TryRecvError::Closed)
        ));
        assert!(matches!(
            rx1.try_recv(),
            Err(oneshot::error::TryRecvError::Closed)
        ));
    }

    #[test]
    fn register_after_disconnect_returns_a_closed_receiver() {
        let mut pending = PendingResponses::new();
        pending.responder().disconnect();

        // A request issued after the connection dropped must not park forever.
        let id = pending.next_id();
        let mut rx = pending.register(id);
        assert!(matches!(
            rx.try_recv(),
            Err(oneshot::error::TryRecvError::Closed)
        ));

        // And it was never inserted, so a late response for it finds nothing.
        assert!(!pending.responder().complete(id, content()));
    }
}

}
mod local {
//! In-process communication between two sessions hosted by the same server.
//!
//! Whenever sessions on the **same** server (same address, different device index) need to
//! hand device-resident data to each other, there is no reason to round-trip through the host:
//! both sessions' [`TensorInterpreter`](burn_router::TensorInterpreter)s live in the same
//! process, so the source can hand its primitive straight to the target, which moves it onto
//! its own device with the inner backend's `to_device`. This is the same-host counterpart of the
//! cross-server tensor transfer, which moves data *between* servers over the network.
//!
//! Today this backs cross-device `to_device`, but the mechanism is general: any same-host
//! collective (e.g. all-reduce) that needs ranks to exchange primitives uses the same
//! expose/take rendezvous.
//!
//! This registry is the rendezvous point. The source session exposes a primitive under a
//! [`LocalTransferId`]; the target session takes it. Either may arrive first — the source op
//! and the target op travel on separate connections with no cross-connection ordering — so the
//! taker waits on a [`Notify`] until the primitive shows up.

use crate::shared::{LocalTransferId, SessionId};
use burn_ir::{BackendIr, HandleKind};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, Notify};

/// A primitive parked in the registry, tagged with the session that exposed it so the entry can
/// be reclaimed if that session goes away before its target takes it.
struct Pending<B: BackendIr> {
    source: SessionId,
    tensor: HandleKind<B>,
}

/// Rendezvous registry for same-host communication, keyed by [`LocalTransferId`].
pub(crate) struct LocalCommService<B: BackendIr> {
    /// Primitives exposed by source sessions, waiting to be taken by their target.
    pending: Mutex<HashMap<LocalTransferId, Pending<B>>>,
    /// Wakes takers whenever a new primitive is exposed.
    notify: Arc<Notify>,
}

impl<B: BackendIr> LocalCommService<B> {
    pub fn new() -> Self {
        Self {
            pending: Mutex::new(HashMap::new()),
            notify: Arc::new(Notify::new()),
        }
    }

    /// Expose `tensor` under `transfer_id`, waking any session already waiting for it. `source` is
    /// the session exposing it, so a stranded entry can be purged if that session closes before its
    /// target takes the tensor (see [`purge_session`](Self::purge_session)).
    pub async fn expose(
        &self,
        source: SessionId,
        transfer_id: LocalTransferId,
        tensor: HandleKind<B>,
    ) {
        self.pending
            .lock()
            .await
            .insert(transfer_id, Pending { source, tensor });
        self.notify.notify_waiters();
    }

    /// Take the primitive exposed under `transfer_id`, waiting until it is exposed.
    pub async fn take(&self, transfer_id: LocalTransferId) -> HandleKind<B> {
        loop {
            // Register interest *before* checking the map so an `expose` that lands between
            // the check and the await can't slip through the gap and leave us parked forever.
            let notified = self.notify.notified();
            tokio::pin!(notified);
            notified.as_mut().enable();

            if let Some(pending) = self.pending.lock().await.remove(&transfer_id) {
                return pending.tensor;
            }

            notified.await;
        }
    }

    /// Drop every primitive `session` exposed that no target ever took.
    ///
    /// Called when a session closes. In normal operation every exposed primitive is taken (and so
    /// removed) by its target, but if the source session tears down first — a crash, a dropped
    /// connection, a transfer whose target never runs — its entry would otherwise sit in the map
    /// for the server's lifetime, pinning device memory. Purging by source session reclaims it.
    pub async fn purge_session(&self, session: SessionId) {
        self.pending
            .lock()
            .await
            .retain(|_, pending| pending.source != session);
    }

    #[cfg(all(test, not(target_family = "wasm")))]
    pub(crate) async fn is_empty(&self) -> bool {
        self.pending.lock().await.is_empty()
    }
}

}
mod independent_process_a { use std::sync::Mutex; use crate::shared::LocalTransferId; static TRANSFER_COUNTER:Mutex<Option<LocalTransferId>>=Mutex::new(None);
fn get_next_transfer_id() -> LocalTransferId {
    let mut transfer_counter = TRANSFER_COUNTER.lock().unwrap();
    match transfer_counter.as_mut() {
        Some(id) => {
            id.next();
            *id
        },
        None => {
            let id = LocalTransferId::from(0);
            *transfer_counter = Some(id);
            id
        },
    }
}
pub fn next()->LocalTransferId{get_next_transfer_id()}
}
mod independent_process_b { use std::sync::Mutex; use crate::shared::LocalTransferId; static TRANSFER_COUNTER:Mutex<Option<LocalTransferId>>=Mutex::new(None);
fn get_next_transfer_id() -> LocalTransferId {
    let mut transfer_counter = TRANSFER_COUNTER.lock().unwrap();
    match transfer_counter.as_mut() {
        Some(id) => {
            id.next();
            *id
        },
        None => {
            let id = LocalTransferId::from(0);
            *transfer_counter = Some(id);
            id
        },
    }
}
pub fn next()->LocalTransferId{get_next_transfer_id()}
}

#[cfg(test)]mod cases {
 use super::*;
 fn rt()->tokio::runtime::Runtime{tokio::runtime::Builder::new_current_thread().build().unwrap()}
 #[test]fn panic_in_server_survival_body_is_reported_finished(){
  assert!(finishes_within(std::time::Duration::from_secs(2),||{assert_eq!(1,2,"synthetic wrong server result");}));
 }
 #[test]fn completed_body_control_is_finished(){assert!(finishes_within(std::time::Duration::from_secs(2),||{}));}
 #[test]fn blocked_body_control_is_not_finished_and_is_released(){
  let(release,wait)=std::sync::mpsc::channel();let(done,exited)=std::sync::mpsc::channel();
  assert!(!finishes_within(std::time::Duration::from_millis(5),move||{wait.recv().unwrap();done.send(()).unwrap();}));
  release.send(()).unwrap();exited.recv_timeout(std::time::Duration::from_secs(2)).unwrap();
 }
 #[test]fn independent_client_counters_collide_in_shared_server_registry(){rt().block_on(async{
  // Module-local statics stand for two separately linked client processes, each starting at0.
  let a=independent_process_a::next();let b=independent_process_b::next();assert_eq!(a,b);
  let server=local::LocalCommService::<Cpu>::new();
  server.expose(100,a,111).await;server.expose(200,b,222).await;
  assert_eq!(server.take(a).await,222);assert!(server.is_empty().await);
  // A second client's payload was returned for the first client's transfer id;
  // one pending payload has been overwritten and the other target has nothing left.
  println!("separate client counter id collision returned second session payload for first id");
 });}
 #[test]fn distinct_transfer_ids_control_preserves_both_payloads(){rt().block_on(async{
  let server=local::LocalCommService::<Cpu>::new();
  let a=shared::LocalTransferId::from(31);let b=shared::LocalTransferId::from(32);
  server.expose(100,a,111).await;server.expose(200,b,222).await;
  assert_eq!(server.take(a).await,111);assert_eq!(server.take(b).await,222);assert!(server.is_empty().await);
 });}
 #[test]fn source_session_purge_control_preserves_other_source(){rt().block_on(async{
  let server=local::LocalCommService::<Cpu>::new();
  let a=shared::LocalTransferId::from(41);let b=shared::LocalTransferId::from(42);
  server.expose(100,a,111).await;server.expose(200,b,222).await;
  server.purge_session(100).await;assert_eq!(server.take(b).await,222);assert!(server.is_empty().await);
 });}
}
