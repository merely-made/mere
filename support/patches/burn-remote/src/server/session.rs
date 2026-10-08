use burn_backend::tensor::Device;
use burn_ir::{BackendIr, DeviceIdIr, DistributedOperationIr, OperationIr};
use burn_router::{CustomOpRegistry, TensorInterpreter};
use burn_std::device::Device as _;
use std::{
    collections::HashMap,
    sync::{Arc, Once},
};
use tokio::sync::{Mutex, mpsc, oneshot, watch};

use crate::metrics::{MetricSide, logger_task};
use crate::server::local_comm::LocalCommService;
use crate::server::service::{SessionBinding, SessionService};
use crate::server::spawn::spawn_detached;
use crate::server::transfer::TensorTransfer;
use crate::server::worker::SessionHandler;
use crate::shared::{SessionId, Task};
use crate::telemetry::{TelemetryEvent, TelemetryProbe};

/// Capacity for the per-session response queue.
///
/// Sized larger than the typical in-flight read/sync count so that request processing
/// doesn't block on backpressure during a burst, but small enough that a stuck response
/// writer surfaces as a backpressure stall rather than memory growth.
const RESPONSE_CHANNEL_CAPACITY: usize = 64;

/// The backend id of each hosted device, by its position on this server, which is how a client
/// names the devices of a collective.
#[derive(Clone, Debug)]
pub(crate) struct HostedDeviceIds(Arc<[DeviceIdIr]>);

impl HostedDeviceIds {
    pub(crate) fn of<B: BackendIr>(devices: &[Device<B>]) -> Self {
        Self(devices.iter().map(|device| device.to_id().into()).collect())
    }

    /// Point an all-reduce's devices, named by position, at their backend ids.
    pub(crate) fn resolve(&self, op: &mut OperationIr) -> Result<(), String> {
        if let OperationIr::Distributed(DistributedOperationIr::AllReduce(desc)) = op {
            for id in desc.device_ids.iter_mut() {
                *id = *self.0.get(usize::from(id.index_id)).ok_or_else(|| {
                    format!(
                        "an all_reduce names device {} of this server, which hosts {}",
                        id.index_id,
                        self.0.len()
                    )
                })?;
            }
        }
        Ok(())
    }
}

/// Coordinates per-session state.
///
/// Each [`Session`] owns a dedicated [`SessionHandler`] that holds the session's
/// [`TensorInterpreter`] with its own [`HandleContainer`](burn_ir::HandleContainer) — different
/// sessions never share tensor handles, so concurrent sessions can't race on each other's backend
/// state. Cross-session tensor transfers go through `external_comm` (cross-server) or `local_comm`
/// (same-host), each of which has its own rendezvous.
///
/// Tasks run on the handler's worker threads, not the submit handler: the latter only decodes the
/// incoming batch and forwards each [`Task`] to the session over a bounded channel. Inside the
/// session a dispatcher routes each task to a per-stream worker thread, so per-stream ordering is
/// preserved while independent streams — and other sessions — keep making progress even when one
/// stream is parked on a blocking op (a same-host transfer rendezvous or an all-reduce barrier).
pub struct SessionManager<B, T>
where
    B: BackendIr,
    T: TensorTransfer<B>,
{
    /// All devices this server hosts, indexed by the device index the client selects at
    /// session init. `devices[0]` is the default device (`DeviceIndex::Default`).
    devices: Vec<Device<B>>,
    device_ids: HostedDeviceIds,
    pub(crate) transfer: Arc<T>,
    /// Rendezvous registry for same-host tensor transfers between this server's sessions.
    pub(crate) local_comm: Arc<LocalCommService<B>>,
    /// Custom-op handlers shared (read-only) with every session's interpreter.
    custom_ops: CustomOpRegistry<B>,
    sessions: Mutex<HashMap<SessionId, Session>>,
    probe: TelemetryProbe,
    /// Spawns the telemetry logger once, on the first session.
    logger: Once,
}

struct Session {
    /// Inbound channel to the session's dispatcher thread; cloned once per submit connection.
    _task_sender: Option<mpsc::Sender<Task>>,
    device_index: u32,
    authorization: Arc<[u8]>,
    close: watch::Sender<bool>,
    done: watch::Sender<SessionCompletion>,
    worker_done: Option<oneshot::Receiver<Result<(), String>>>,
    finishing: bool,
}

#[derive(Clone, Debug)]
enum SessionCompletion {
    Running,
    Clean,
    Failed(Arc<str>),
}

/// One session currently served by an Iroh remote protocol.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServedSession {
    /// Client-chosen process-local session id.
    pub id: SessionId,
    /// Hosted compute device selected at admission.
    pub device_index: u32,
    /// Opaque application credential accepted for this session.
    pub credential: Arc<[u8]>,
}

impl<B, T> SessionManager<B, T>
where
    B: BackendIr,
    T: TensorTransfer<B>,
{
    pub fn new(devices: Vec<Device<B>>, transfer: Arc<T>) -> Self {
        assert!(
            !devices.is_empty(),
            "A remote server must host at least one device"
        );
        Self {
            device_ids: HostedDeviceIds::of::<B>(&devices),
            devices,
            transfer,
            local_comm: Arc::new(LocalCommService::new()),
            custom_ops: CustomOpRegistry::default(),
            sessions: Mutex::new(HashMap::new()),
            probe: TelemetryProbe::disabled(),
            logger: Once::new(),
        }
    }

    /// Register custom-op handlers, shared read-only with every session's interpreter.
    pub fn with_custom_ops(mut self, custom_ops: CustomOpRegistry<B>) -> Self {
        self.custom_ops = custom_ops;
        self
    }

    /// Emit telemetry into `probe`.
    pub fn with_telemetry(mut self, probe: TelemetryProbe) -> Self {
        self.probe = probe;
        self
    }

    fn ensure_logger(&self) {
        self.logger.call_once(|| {
            if let Some(task) = logger_task(&self.probe, MetricSide::Server) {
                spawn_detached(task);
            }
        });
    }

    /// Resolve the device at `device_index`.
    ///
    /// The index is validated against the server's device count on the client init handshake, so
    /// an out-of-range index here is a protocol/configuration error (e.g. a client enumerating
    /// more devices than this server hosts). Fail loudly rather than silently collapsing onto
    /// device 0 — for a collective that would reduce a device against itself and silently corrupt
    /// the result instead of producing a clear failure.
    pub(crate) fn device(&self, device_index: u32) -> Device<B> {
        self.devices
            .get(device_index as usize)
            .cloned()
            .unwrap_or_else(|| {
                panic!(
                    "Requested device index {device_index} but server hosts only {} device(s)",
                    self.devices.len()
                )
            })
    }

    /// Snapshot active sessions in deterministic id order.
    pub async fn sessions(&self) -> Vec<ServedSession> {
        let sessions = self.sessions.lock().await;
        let mut served: Vec<_> = sessions
            .iter()
            .map(|(id, session)| ServedSession {
                id: *id,
                device_index: session.device_index,
                credential: session.authorization.clone(),
            })
            .collect();
        served.sort_by_key(|session| session.id);
        served
    }

    /// Ask one active pump to close, and wait until its worker has released backend state.
    pub async fn close_session(&self, session_id: SessionId) -> Result<bool, String> {
        let (close, mut done) = {
            let sessions = self.sessions.lock().await;
            let Some(session) = sessions.get(&session_id) else {
                return Ok(false);
            };
            (session.close.clone(), session.done.subscribe())
        };

        close.send_replace(true);
        loop {
            match done.borrow().clone() {
                SessionCompletion::Running => {},
                SessionCompletion::Clean => return Ok(true),
                SessionCompletion::Failed(error) => return Err(error.to_string()),
            }
            done.changed()
                .await
                .map_err(|_| format!("Session {session_id} teardown signal was dropped"))?;
        }
    }
}

impl<B, T> SessionService for SessionManager<B, T>
where
    B: BackendIr,
    T: TensorTransfer<B>,
{
    async fn reserve_session(
        &self,
        session_id: SessionId,
        device_index: u32,
        authorization: Arc<[u8]>,
    ) -> Result<(), String> {
        self.ensure_logger();
        let mut sessions = self.sessions.lock().await;
        if sessions.contains_key(&session_id) {
            return Err(format!("Session {session_id} is already active"));
        }

        let (close, _) = watch::channel(false);
        let (done, _) = watch::channel(SessionCompletion::Running);
        sessions.insert(
            session_id,
            Session {
                _task_sender: None,
                device_index,
                authorization,
                close,
                done,
                worker_done: None,
                finishing: false,
            },
        );
        Ok(())
    }

    async fn bind_session(&self, session_id: SessionId) -> Result<SessionBinding, String> {
        let mut sessions = self.sessions.lock().await;
        let session = sessions
            .get_mut(&session_id)
            .ok_or_else(|| format!("Session {session_id} was not reserved"))?;
        if *session.close.borrow() {
            return Err(format!("Session {session_id} was closed during admission"));
        }
        if session._task_sender.is_some() {
            return Err(format!("Session {session_id} is already active"));
        }

        let device_index = session.device_index;
        let (response_sender, responses) = mpsc::channel(RESPONSE_CHANNEL_CAPACITY);
        let runner =
            TensorInterpreter::with_custom_ops(self.device(device_index), self.custom_ops.clone());
        let (task_sender, worker_done) = SessionHandler::spawn(
            session_id,
            runner,
            self.device_ids.clone(),
            response_sender,
            self.transfer.clone(),
            self.local_comm.clone(),
            self.probe.clone(),
        );
        session._task_sender = Some(task_sender.clone());
        session.worker_done = Some(worker_done);
        let close_receiver = session.close.subscribe();
        self.probe.emit(|| TelemetryEvent::SessionOpened {
            session: session_id,
            device: device_index,
        });
        Ok(SessionBinding {
            task_sender,
            responses,
            close: close_receiver,
        })
    }

    /// The device settings for `device_index`, used by the handshake before any session-specific
    /// runner is needed.
    fn device_settings(&self, device_index: u32) -> burn_std::DeviceSettings {
        use burn_backend::backend::DeviceOps;
        self.device(device_index).defaults()
    }

    /// The total number of devices this server hosts. Sent to the client on the init handshake so
    /// it can enumerate every device behind the address (see [`RemoteDevice::enumerate`]).
    fn device_count(&self) -> u32 {
        self.devices.len() as u32
    }

    async fn finish_session(&self, session_id: SessionId) -> Result<(), String> {
        let (worker_done, mut concurrent_done, was_bound) = {
            let mut sessions = self.sessions.lock().await;
            let Some(session) = sessions.get_mut(&session_id) else {
                return Ok(());
            };
            if session.finishing {
                (None, Some(session.done.subscribe()), false)
            } else {
                session.finishing = true;
                session.close.send_replace(true);
                let was_bound = session._task_sender.is_some();
                // The manager retains one sender so a worker cannot disappear while the pump is
                // active. Drop it before waiting, then acknowledge closure only after the worker
                // has synced, dropped its interpreter, run backend memory cleanup, and waited for
                // that release to complete on the device. A teardown failure is reported, never
                // acknowledged as a clean close.
                drop(session._task_sender.take());
                (session.worker_done.take(), None, was_bound)
            }
        };

        if let Some(done) = concurrent_done.as_mut() {
            loop {
                match done.borrow().clone() {
                    SessionCompletion::Running => {},
                    SessionCompletion::Clean => return Ok(()),
                    SessionCompletion::Failed(error) => return Err(error.to_string()),
                }
                done.changed()
                    .await
                    .map_err(|_| format!("Session {session_id} teardown signal was dropped"))?;
            }
        }

        let cleanup = match worker_done {
            Some(worker_done) => worker_done
                .await
                .map_err(|_| {
                    format!("Session {session_id} worker stopped before backend cleanup completed")
                })
                .and_then(|teardown| teardown),
            None => Ok(()),
        };

        let mut sessions = self.sessions.lock().await;
        let Some(session) = sessions.get_mut(&session_id) else {
            return Err(format!(
                "Session {session_id} disappeared before teardown completed"
            ));
        };
        match cleanup {
            Ok(()) => {
                let done = session.done.clone();
                sessions.remove(&session_id);
                done.send_replace(SessionCompletion::Clean);
                if was_bound {
                    self.probe.emit(|| TelemetryEvent::SessionClosed {
                        session: session_id,
                    });
                }
                Ok(())
            },
            Err(error) => {
                session
                    .done
                    .send_replace(SessionCompletion::Failed(Arc::from(error.as_str())));
                Err(error)
            },
        }
    }
}

#[cfg(test)]
mod teardown_tests {
    //! Mere: close reports a failed teardown instead of acknowledging it as clean.

    use super::*;
    use crate::server::worker::teardown_fault;
    use crate::shared::TransferCapability;
    use crate::{PeerAddr, PeerId};
    use burn_backend::TensorData;
    use burn_flex::Flex;
    use std::future::Future;

    struct NoTransfer;

    impl<B: BackendIr> TensorTransfer<B> for NoTransfer {
        fn expose_data(
            &self,
            _data: TensorData,
            _max_downloads: u32,
            _capability: TransferCapability,
            _target: PeerId,
        ) -> impl Future<Output = ()> + Send {
            async {}
        }

        fn download_tensor(
            &self,
            _remote: PeerAddr,
            _capability: TransferCapability,
        ) -> impl Future<Output = Option<TensorData>> + Send {
            async { None }
        }

        fn fail(
            &self,
            _capability: TransferCapability,
            _target: PeerId,
            _reason: String,
        ) -> impl Future<Output = ()> + Send {
            async {}
        }
    }

    async fn bound(manager: &SessionManager<Flex, NoTransfer>) -> SessionId {
        let session_id = SessionId::new();
        manager
            .reserve_session(session_id, 0, Arc::from(&b"teardown-test"[..]))
            .await
            .expect("reserve");
        // Dropping the binding drops its task sender and response queue, as a closed pump does.
        drop(manager.bind_session(session_id).await.expect("bind"));
        session_id
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn a_failed_teardown_sync_is_reported_and_never_acknowledged_clean() {
        let manager =
            SessionManager::<Flex, NoTransfer>::new(vec![Default::default()], Arc::new(NoTransfer));

        // Success path in the same run: an unarmed session closes clean and leaves the registry.
        let clean = bound(&manager).await;
        assert_eq!(manager.finish_session(clean).await, Ok(()));
        assert!(manager.sessions().await.iter().all(|s| s.id != clean));
        assert_eq!(manager.close_session(clean).await, Ok(false));

        // Injected failure of the post-release completion wait.
        let faulted = bound(&manager).await;
        teardown_fault::arm(faulted);
        let closed = manager.finish_session(faulted).await;
        assert!(
            closed
                .as_ref()
                .is_err_and(|error| error.contains("injected teardown sync failure")),
            "a failed teardown must be reported: {closed:?}"
        );
        assert!(
            manager.sessions().await.iter().any(|s| s.id == faulted),
            "a failed teardown must not be acknowledged by removing the session"
        );
        let again = manager.close_session(faulted).await;
        assert!(
            again
                .as_ref()
                .is_err_and(|error| error.contains("injected teardown sync failure")),
            "a later close must report the same failure: {again:?}"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use burn_backend::{DType, Shape, distributed::ReduceOperation};
    use burn_ir::{AllReduceOpIr, TensorId, TensorIr};

    fn device(type_id: u16, index_id: u16) -> DeviceIdIr {
        DeviceIdIr { type_id, index_id }
    }

    fn all_reduce(device_ids: Vec<DeviceIdIr>) -> OperationIr {
        let tensor = TensorIr::uninit(TensorId::new(0), Shape::new([2]), DType::F32);
        OperationIr::Distributed(DistributedOperationIr::AllReduce(AllReduceOpIr {
            out: tensor.clone(),
            tensor,
            op: ReduceOperation::Sum,
            device_ids,
        }))
    }

    #[test]
    fn a_collective_names_the_hosted_devices_by_position() {
        for devices in [[device(3, 2), device(3, 3)], [device(3, 3), device(3, 2)]] {
            let hosted = HostedDeviceIds(Arc::from(devices));
            let mut op = all_reduce(vec![device(0, 0), device(0, 1)]);

            hosted.resolve(&mut op).unwrap();

            assert_eq!(op, all_reduce(devices.to_vec()));
        }
    }

    #[test]
    fn a_collective_naming_an_unhosted_position_is_refused() {
        let hosted = HostedDeviceIds(Arc::from([device(3, 2)]));

        assert!(hosted.resolve(&mut all_reduce(vec![device(0, 1)])).is_err());
    }
}
