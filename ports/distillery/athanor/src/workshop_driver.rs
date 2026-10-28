// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Native worker pump shared by application owners. The owner serializes its
//! live source, authority and custody policy with dispatch and observation.

use crate::workshop::{Error, Workshop, WorkshopBackend, worker::Worker};
use alembic::workshop::{Control, Owner, Passage, RunStatus, WorkshopView};
use servitor::AuthorityProvider;
use std::sync::mpsc::TryRecvError;

pub struct Driver<B> {
    pub workshop: Workshop<B>,
    wake: armillary::Wake,
    worker: Option<Worker>,
    active: Option<u64>,
    inputs: Vec<Passage>,
}

// This runs before the workshop/backend fields are dropped, including while
// unwinding. It cannot persist a lifecycle transition; normal owners must use
// `shutdown` for that. Drop requests cancellation and observes worker join so
// a backend lease cannot be released while its worker is still executing.
impl<B> Drop for Driver<B> {
    fn drop(&mut self) {
        if let Some(worker) = self.worker.take() {
            worker.cancel();
            let _ = worker.join();
        }
    }
}

impl<B: WorkshopBackend> Driver<B> {
    pub fn new(backend: B, owner: Owner, installation: [u8; 16], wake: armillary::Wake) -> Self {
        Self {
            workshop: Workshop::new(backend, owner, installation),
            wake,
            worker: None,
            active: None,
            inputs: vec![],
        }
    }
    pub fn busy(&self) -> bool {
        self.active.is_some()
    }
    pub async fn open(&self, subject: [u8; 32]) -> Result<WorkshopView, Error> {
        match self.workshop.view(None).await {
            Ok(view) => {
                self.workshop.validate_installed_subject(subject).await?;
                Ok(view)
            },
            Err(Error::Missing) => {
                self.workshop.install(subject).await?;
                self.workshop.view(None).await
            },
            Err(error) => Err(error),
        }
    }
    pub async fn refresh(&self) -> Result<WorkshopView, Error> {
        self.workshop.view(None).await
    }

    /// The owner calls this only after it has exclusive ownership and knows the
    /// old worker is gone. It records interruption and pause; it never dispatches.
    pub async fn recover(
        &mut self,
        authority: &impl AuthorityProvider,
        clock: &impl Fn() -> u64,
    ) -> Result<WorkshopView, Error> {
        if self.busy() || self.worker.is_some() {
            return Err(Error::Invalid("worker still owned"));
        }
        let view = self.refresh().await?;
        if matches!(view.phase, RunStatus::Awaiting(_) | RunStatus::Ready(_)) {
            self.workshop
                .lifecycle(Control::Pause, authority, clock())
                .await?;
        }
        self.refresh().await
    }

    /// On an error after run admission, persist a paused/interrupted state before
    /// cancellation. A persistence failure is returned, not described as a pause.
    async fn quiesce(
        &mut self,
        authority: &impl AuthorityProvider,
        clock: &impl Fn() -> u64,
    ) -> Result<(), Error> {
        let persisted = match self.refresh().await {
            Ok(view) if matches!(view.phase, RunStatus::Ready(_) | RunStatus::Awaiting(_)) => {
                self.workshop
                    .lifecycle(Control::Pause, authority, clock())
                    .await
            },
            Ok(_) => Ok(()),
            Err(error) => Err(error),
        };
        let joined = self.stop();
        persisted?;
        joined
    }

    /// Recheck public source admission after each persistence await and before
    /// sending captured bytes. The original run deadline is never extended.
    pub async fn begin(
        &mut self,
        inputs: Vec<Passage>,
        authority: &impl AuthorityProvider,
        clock: &impl Fn() -> u64,
        source_gate: &impl Fn(&Passage) -> bool,
    ) -> Result<WorkshopView, Error> {
        if self.busy() {
            return Err(Error::ActiveRun(self.active.unwrap()));
        }
        if !inputs.iter().all(source_gate) {
            return Err(Error::Unauthorized);
        }
        let view = self.refresh().await?;
        let id = view
            .run
            .unwrap_or(0)
            .checked_add(1)
            .ok_or(Error::Invalid("run ID overflow"))?;
        if let Err(error) = self
            .workshop
            .begin(id, inputs.clone(), authority, clock())
            .await
        {
            // A storage error may have followed a durable admission. Reopen the
            // exact ID before deciding whether there is anything to pause.
            if matches!(error, Error::Store(_)) {
                match self.refresh().await {
                    Ok(view) if view.run == Some(id) => self.quiesce(authority, clock).await?,
                    Ok(_) => {},
                    Err(recovery) => return Err(recovery),
                }
            }
            return Err(error);
        }
        self.inputs = inputs;
        self.active = Some(id);
        self.worker = Some(Worker::spawn(self.wake.clone()));
        if let Err(error) = self.dispatch_next(authority, clock, source_gate).await {
            self.quiesce(authority, clock).await?;
            return Err(error);
        }
        match self.refresh().await {
            Ok(view) => Ok(view),
            Err(error) => {
                self.quiesce(authority, clock).await?;
                Err(error)
            },
        }
    }

    async fn dispatch_next(
        &mut self,
        authority: &impl AuthorityProvider,
        clock: &impl Fn() -> u64,
        source_gate: &impl Fn(&Passage) -> bool,
    ) -> Result<(), Error> {
        if !self.inputs.iter().all(source_gate) {
            return Err(Error::Unauthorized);
        }
        let dispatch = self
            .workshop
            .dispatch(self.active.ok_or(Error::Missing)?, authority, clock())
            .await?;
        if !self.inputs.iter().all(source_gate) {
            return Err(Error::Unauthorized);
        }
        if !self
            .workshop
            .dispatch_authorized(&dispatch, authority)
            .await?
        {
            return Err(Error::Unauthorized);
        }
        if !self.inputs.iter().all(source_gate) {
            return Err(Error::Unauthorized);
        }
        let now = clock();
        if now < dispatch.issued_at_ms() {
            return Err(Error::Stale);
        }
        if now >= dispatch.deadline_at_ms() {
            return Err(Error::Budget);
        }
        if !self.worker.as_ref().is_some_and(|w| w.dispatch(dispatch)) {
            return Err(Error::Invalid("worker stopped"));
        }
        Ok(())
    }

    pub async fn poll(
        &mut self,
        authority: &impl AuthorityProvider,
        clock: &impl Fn() -> u64,
        source_gate: &impl Fn(&Passage) -> bool,
    ) -> Result<Option<WorkshopView>, Error> {
        let Some(worker) = &self.worker else {
            return Ok(None);
        };
        let received = match worker.observations.try_recv() {
            Ok(result) => result,
            Err(TryRecvError::Empty) => return Ok(None),
            Err(TryRecvError::Disconnected) => Err(Error::Invalid("worker disconnected")),
        };
        let result = async {
            let observation = received?;
            if !self.inputs.iter().all(source_gate) {
                return Err(Error::Unauthorized);
            }
            self.workshop
                .observe_guarded(observation, authority, clock, source_gate)
                .await?;
            let view = self.refresh().await?;
            if matches!(view.phase, RunStatus::Ready(_)) {
                self.dispatch_next(authority, clock, source_gate).await?;
            } else {
                self.stop()?;
            }
            self.refresh().await
        }
        .await;
        match result {
            Ok(view) => Ok(Some(view)),
            Err(error) => {
                self.quiesce(authority, clock).await?;
                Err(error)
            },
        }
    }

    pub async fn control(
        &mut self,
        control: Control,
        authority: &impl AuthorityProvider,
        clock: &impl Fn() -> u64,
        source_gate: &impl Fn(&Passage) -> bool,
    ) -> Result<WorkshopView, Error> {
        if !matches!(
            control,
            Control::Pause | Control::Resume | Control::Revoke | Control::Dissolve
        ) {
            return Err(Error::Invalid("driver lifecycle control"));
        }
        if let Err(error) = self.workshop.lifecycle(control, authority, clock()).await {
            // The transition may already have committed despite a lost ack.
            self.quiesce(authority, clock).await?;
            return Err(error);
        }
        self.stop()?;
        let view = self.refresh().await?;
        if control == Control::Resume && matches!(view.phase, RunStatus::Ready(_)) {
            self.active = view.run;
            let result = async {
                self.inputs = self
                    .workshop
                    .run_inputs(view.run.ok_or(Error::Missing)?)
                    .await?;
                self.worker = Some(Worker::spawn(self.wake.clone()));
                self.dispatch_next(authority, clock, source_gate).await
            }
            .await;
            if let Err(error) = result {
                self.quiesce(authority, clock).await?;
                return Err(error);
            }
        }
        self.refresh().await
    }

    /// Persist a pause for an active or orphaned run, then request cancellation
    /// and observe worker completion. The host calls this while it still owns
    /// the policy and persistence gates.
    pub async fn shutdown(
        &mut self,
        authority: &impl AuthorityProvider,
        clock: &impl Fn() -> u64,
    ) -> Result<(), Error> {
        self.quiesce(authority, clock).await
    }

    /// Request cancellation and observe join. This low-level operation does not
    /// itself record a pause; use `shutdown` when abandoning an active run.
    pub fn stop(&mut self) -> Result<(), Error> {
        self.active = None;
        self.inputs.clear();
        if let Some(worker) = self.worker.take() {
            worker.cancel();
            worker.join().map_err(|_| Error::Invalid("worker join"))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workshop::capture;
    use alembic::workshop::Source;
    use muniment::MemoryBackend;
    use servitor::{Cap, Mode, Subject};
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    struct Granted;
    impl AuthorityProvider for Granted {
        fn covers(&self, _: Subject, _: &Cap, _: Mode) -> bool {
            true
        }
    }
    fn owner() -> Owner {
        Owner {
            host: "driver-test".into(),
            workspace: "public".into(),
            session: "one".into(),
        }
    }
    fn inputs(owner: &Owner) -> Vec<Passage> {
        ["first", "second"]
            .into_iter()
            .enumerate()
            .map(|(n, text)| {
                capture(
                    Source {
                        owner: owner.clone(),
                        resource: format!("resource-{n}"),
                        version: "1".into(),
                        content_digest: [0; 32],
                        start: 0,
                        end: text.len() as u64,
                        title: format!("Source {n}"),
                    },
                    text,
                    true,
                )
                .unwrap()
            })
            .collect()
    }
    fn new_driver(store: MemoryBackend, owner: Owner) -> Driver<MemoryBackend> {
        Driver::new(store, owner, [1; 16], Arc::new(|| {}))
    }

    #[test]
    fn driver_unwind_joins_worker_before_backend_is_released() {
        use muniment::{Backend, StoreError, WriteOp};
        use std::sync::{Mutex, mpsc};
        use std::time::Duration;

        struct LeaseBackend {
            store: MemoryBackend,
            _lease: Arc<()>,
        }
        #[async_trait::async_trait]
        impl Backend for LeaseBackend {
            async fn get(&self, key: &str) -> Result<Option<Vec<u8>>, StoreError> {
                self.store.get(key).await
            }
            async fn put(&self, key: &str, bytes: &[u8]) -> Result<(), StoreError> {
                self.store.put(key, bytes).await
            }
            async fn delete(&self, key: &str) -> Result<(), StoreError> {
                self.store.delete(key).await
            }
            async fn list(&self, prefix: &str) -> Result<Vec<String>, StoreError> {
                self.store.list(prefix).await
            }
            async fn scan(&self, start: &str, end: &str) -> Result<Vec<String>, StoreError> {
                self.store.scan(start, end).await
            }
            async fn apply(&self, ops: &[WriteOp]) -> Result<(), StoreError> {
                self.store.apply(ops).await
            }
            async fn transact(&self, op: muniment::TransactFn) -> Result<(), StoreError> {
                self.store.transact(op).await
            }
        }

        let (entered_tx, entered_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let release_rx = Mutex::new(release_rx);
        let wake = Arc::new(move || {
            let _ = entered_tx.send(());
            let _ = release_rx
                .lock()
                .unwrap()
                .recv_timeout(Duration::from_secs(5));
        });
        let lease = Arc::new(());
        let weak = Arc::downgrade(&lease);
        let backend = LeaseBackend {
            store: MemoryBackend::new(),
            _lease: lease,
        };
        let owner = owner();
        let mut driver = Driver::new(backend, owner.clone(), [1; 16], wake);
        pollster::block_on(async {
            driver.open([7; 32]).await.unwrap();
            driver
                .begin(inputs(&owner), &Granted, &|| 1, &|_| true)
                .await
                .unwrap();
        });
        entered_rx.recv_timeout(Duration::from_secs(3)).unwrap();
        let (started_tx, started_rx) = mpsc::channel();
        let (finished_tx, finished_rx) = mpsc::channel();
        let dropper = std::thread::spawn(move || {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let _driver = driver;
                started_tx.send(()).unwrap();
                panic!("exercise unwinding while the worker is still running");
            }));
            let _ = finished_tx.send(result.is_err());
        });
        started_rx.recv_timeout(Duration::from_secs(3)).unwrap();
        let finished_early = finished_rx.recv_timeout(Duration::from_millis(100)).ok();
        let lease_held_during_worker = weak.upgrade().is_some();
        let _ = release_tx.send(());
        let finished =
            finished_early.or_else(|| finished_rx.recv_timeout(Duration::from_secs(3)).ok());
        dropper.join().unwrap();
        assert!(
            finished_early.is_none(),
            "driver unwound before the worker joined"
        );
        assert!(
            lease_held_during_worker,
            "backend lease was released before the worker joined"
        );
        assert_eq!(finished, Some(true));
        assert!(
            weak.upgrade().is_none(),
            "backend lease should release after join"
        );
    }

    #[test]
    fn driver_revalidation_refuses_retargeted_dispatch_inputs() {
        pollster::block_on(async {
            let owner = owner();
            let driver = new_driver(MemoryBackend::new(), owner.clone());
            driver.open([7; 32]).await.unwrap();
            driver
                .workshop
                .begin(1, inputs(&owner), &Granted, 0)
                .await
                .unwrap();
            let old = driver.workshop.dispatch(1, &Granted, 1).await.unwrap();
            driver
                .workshop
                .observe(crate::workshop::evaluate(old.clone()).unwrap(), &Granted, 2)
                .await
                .unwrap();
            for step in 0..2 {
                let next = driver
                    .workshop
                    .dispatch(1, &Granted, 3 + step * 2)
                    .await
                    .unwrap();
                driver
                    .workshop
                    .observe(
                        crate::workshop::evaluate(next).unwrap(),
                        &Granted,
                        4 + step * 2,
                    )
                    .await
                    .unwrap();
            }
            let mut replacement = inputs(&owner);
            replacement[0].source.resource = "different-resource".into();
            driver
                .workshop
                .begin(2, replacement, &Granted, 7)
                .await
                .unwrap();
            let next = driver.workshop.dispatch(2, &Granted, 8).await.unwrap();
            assert!(
                driver
                    .workshop
                    .dispatch_authorized(&next, &Granted)
                    .await
                    .unwrap()
            );
            let mut forged = old;
            forged.correlation = next.correlation;
            assert!(
                !driver
                    .workshop
                    .dispatch_authorized(&forged, &Granted)
                    .await
                    .unwrap()
            );
        });
    }

    #[test]
    fn driver_open_refuses_another_installed_subject() {
        pollster::block_on(async {
            let driver = new_driver(MemoryBackend::new(), owner());
            driver.open([7; 32]).await.unwrap();
            assert!(matches!(
                driver.open([8; 32]).await,
                Err(Error::Unauthorized)
            ));
            assert!(driver.open([7; 32]).await.is_ok());
        });
    }

    #[test]
    fn driver_lost_admission_ack_recovers_before_returning() {
        use muniment::{Backend, StoreError, WriteOp};
        use std::sync::atomic::AtomicBool;
        #[derive(Clone)]
        struct LostAck {
            store: MemoryBackend,
            fail: Arc<AtomicBool>,
        }
        #[async_trait::async_trait]
        impl Backend for LostAck {
            async fn get(&self, key: &str) -> Result<Option<Vec<u8>>, StoreError> {
                self.store.get(key).await
            }
            async fn put(&self, key: &str, bytes: &[u8]) -> Result<(), StoreError> {
                self.store.put(key, bytes).await
            }
            async fn delete(&self, key: &str) -> Result<(), StoreError> {
                self.store.delete(key).await
            }
            async fn list(&self, prefix: &str) -> Result<Vec<String>, StoreError> {
                self.store.list(prefix).await
            }
            async fn scan(&self, start: &str, end: &str) -> Result<Vec<String>, StoreError> {
                self.store.scan(start, end).await
            }
            async fn apply(&self, ops: &[WriteOp]) -> Result<(), StoreError> {
                self.store.apply(ops).await
            }
            async fn transact(&self, op: muniment::TransactFn) -> Result<(), StoreError> {
                self.store.transact(op).await?;
                if self.fail.swap(false, Ordering::SeqCst) {
                    return Err(StoreError::NotTransactional);
                }
                Ok(())
            }
        }
        pollster::block_on(async {
            let owner = owner();
            let backend = LostAck {
                store: MemoryBackend::new(),
                fail: Arc::new(AtomicBool::new(false)),
            };
            let mut driver = Driver::new(backend.clone(), owner.clone(), [1; 16], Arc::new(|| {}));
            driver.open([7; 32]).await.unwrap();
            backend.fail.store(true, Ordering::SeqCst);
            assert!(matches!(
                driver
                    .begin(inputs(&owner), &Granted, &|| 1, &|_| true)
                    .await,
                Err(Error::Store(_))
            ));
            assert!(!driver.busy());
            assert!(matches!(
                driver.refresh().await.unwrap().phase,
                RunStatus::Paused(_)
            ));
        });
    }

    #[test]
    fn driver_source_revocation_after_intent_pauses_and_joins() {
        pollster::block_on(async {
            let owner = owner();
            let mut driver = new_driver(MemoryBackend::new(), owner.clone());
            driver.open([7; 32]).await.unwrap();
            let calls = AtomicUsize::new(0);
            let gate = |_: &Passage| calls.fetch_add(1, Ordering::SeqCst) < 6;
            assert_eq!(
                driver
                    .begin(inputs(&owner), &Granted, &|| 1, &gate)
                    .await
                    .unwrap_err(),
                Error::Unauthorized
            );
            assert!(!driver.busy());
            let view = driver.refresh().await.unwrap();
            assert!(matches!(view.phase, RunStatus::Paused(_)));
            assert!(view.proposal.is_none());
        });
    }

    #[test]
    fn driver_rechecks_source_after_observation_load() {
        pollster::block_on(async {
            let owner = owner();
            let (wake_send, wake_receive) = std::sync::mpsc::channel();
            let wake = Arc::new(move || {
                let _ = wake_send.send(());
            });
            let mut driver = Driver::new(MemoryBackend::new(), owner.clone(), [1; 16], wake);
            driver.open([7; 32]).await.unwrap();
            driver
                .begin(inputs(&owner), &Granted, &|| 1, &|_| true)
                .await
                .unwrap();
            wake_receive
                .recv_timeout(std::time::Duration::from_secs(2))
                .unwrap();
            let calls = AtomicUsize::new(0);
            let gate = |_: &Passage| calls.fetch_add(1, Ordering::SeqCst) < 2;
            assert!(matches!(
                driver.poll(&Granted, &|| 2, &gate).await,
                Err(Error::Unauthorized)
            ));
            assert!(matches!(
                driver.refresh().await.unwrap().phase,
                RunStatus::Paused(_)
            ));
            assert!(!driver.busy());
        });
    }

    #[test]
    fn driver_samples_observation_time_after_load() {
        pollster::block_on(async {
            let owner = owner();
            let driver = new_driver(MemoryBackend::new(), owner.clone());
            driver.open([7; 32]).await.unwrap();
            driver
                .workshop
                .begin(1, inputs(&owner), &Granted, 0)
                .await
                .unwrap();
            let dispatch = driver.workshop.dispatch(1, &Granted, 1).await.unwrap();
            let observation = crate::workshop::evaluate(dispatch).unwrap();
            assert!(matches!(
                driver
                    .workshop
                    .observe_guarded(observation, &Granted, &|| 30_000, &|_| true)
                    .await,
                Err(Error::Budget)
            ));
            assert!(driver.refresh().await.unwrap().proposal.is_none());
        });
    }

    #[test]
    fn driver_refuses_revoked_source_at_observation_and_pauses() {
        pollster::block_on(async {
            let owner = owner();
            let (wake_send, wake_receive) = std::sync::mpsc::channel();
            let wake = Arc::new(move || {
                let _ = wake_send.send(());
            });
            let mut driver = Driver::new(MemoryBackend::new(), owner.clone(), [1; 16], wake);
            driver.open([7; 32]).await.unwrap();
            driver
                .begin(inputs(&owner), &Granted, &|| 1, &|_| true)
                .await
                .unwrap();
            wake_receive
                .recv_timeout(std::time::Duration::from_secs(2))
                .unwrap();
            assert!(matches!(
                driver.poll(&Granted, &|| 2, &|_| false).await,
                Err(Error::Unauthorized)
            ));
            assert!(!driver.busy());
            let view = driver.refresh().await.unwrap();
            assert!(matches!(view.phase, RunStatus::Paused(_)));
            assert!(view.proposal.is_none());
        });
    }

    #[test]
    fn driver_deadline_after_authority_await_never_dispatches() {
        pollster::block_on(async {
            let owner = owner();
            let mut driver = new_driver(MemoryBackend::new(), owner.clone());
            driver.open([7; 32]).await.unwrap();
            let calls = AtomicUsize::new(0);
            let clock = || {
                if calls.fetch_add(1, Ordering::SeqCst) < 2 {
                    0
                } else {
                    30_000
                }
            };
            assert_eq!(
                driver
                    .begin(inputs(&owner), &Granted, &clock, &|_| true)
                    .await
                    .unwrap_err(),
                Error::Budget
            );
            assert!(!driver.busy());
            let view = driver.refresh().await.unwrap();
            assert!(matches!(view.phase, RunStatus::Paused(_)));
            assert!(view.proposal.is_none());
        });
    }

    #[test]
    fn driver_recovery_pauses_orphan_without_redispatch() {
        pollster::block_on(async {
            let store = MemoryBackend::new();
            let owner = owner();
            let driver = new_driver(store.clone(), owner.clone());
            driver.open([7; 32]).await.unwrap();
            driver
                .workshop
                .begin(1, inputs(&owner), &Granted, 0)
                .await
                .unwrap();
            let delayed = driver.workshop.dispatch(1, &Granted, 1).await.unwrap();
            let mut reopened = new_driver(store, owner);
            assert!(matches!(
                reopened.open([7; 32]).await.unwrap().phase,
                RunStatus::Awaiting(_)
            ));
            let recovered = reopened.recover(&Granted, &|| 2).await.unwrap();
            assert!(matches!(recovered.phase, RunStatus::Paused(_)));
            assert!(!reopened.busy());
            assert_eq!(
                reopened
                    .workshop
                    .observe(crate::workshop::evaluate(delayed).unwrap(), &Granted, 3)
                    .await,
                Err(Error::Unauthorized)
            );
            assert!(matches!(
                reopened.recover(&Granted, &|| 4).await.unwrap().phase,
                RunStatus::Paused(_)
            ));
        });
    }
}
