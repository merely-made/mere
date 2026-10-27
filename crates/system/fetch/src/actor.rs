// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Host-owned admission and task lifetime, independent of transport policy.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, Mutex, Weak};
use std::time::{Duration, Instant};

use armillary::tasks::{ResidentTasks, TaskReport};
use armillary::{ActorError, ActorHandle, Emitter, Wake, spawn_named};
use tokio::runtime::{Builder, Runtime};
use tokio::sync::{OwnedSemaphorePermit, Semaphore, watch};

use super::*;

/// Scheduling policy shared by HTTP and small-web operations. Byte limits and
/// transport deadlines remain in [`ClientFetchPolicy`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FetchActorPolicy {
    /// Reserved running slots, including commands not yet polled by the worker.
    /// Zero disables acquisition; pending requests are refused too.
    pub max_active: usize,
    /// Additional waiting slots. Zero sheds work when all running slots are held.
    pub max_pending: usize,
    /// Separate allowance from enqueue until a running slot is obtained.
    /// The transport's existing whole-operation timer starts afterwards.
    pub queue_timeout: Duration,
}

impl Default for FetchActorPolicy {
    fn default() -> Self {
        Self {
            max_active: 16,
            max_pending: 64,
            queue_timeout: Duration::from_secs(30),
        }
    }
}

impl FetchActorPolicy {
    pub fn validate(&self) -> Result<(), String> {
        if self.max_active > Semaphore::MAX_PERMITS
            || self.max_pending > Semaphore::MAX_PERMITS
            || (self.max_pending != 0
                && (self.queue_timeout.is_zero()
                    || Instant::now().checked_add(self.queue_timeout).is_none()))
        {
            return Err("invalid fetch actor capacity or queue deadline".into());
        }
        Ok(())
    }
}

/// Joined task outcomes and admission counters. Policy refusals are distinct
/// from Armillary rejecting a task after its scope has closed.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FetchActorReport {
    pub tasks: TaskReport,
    /// Capacity reservations granted, including a later failed mailbox delivery.
    pub admitted: usize,
    /// Reserved jobs destroyed before a Tokio task was spawned, such as work
    /// still in the bounded mailbox at shutdown. Their payloads are dropped too.
    pub discarded_before_spawn: usize,
    pub overloaded: usize,
    pub queue_timeouts: usize,
}

/// Bounded fetch ingress. Commands move their bodies once; cancellation is a
/// direct exact-ID control and cannot be starved by the work mailbox.
///
/// Drop requests shutdown without blocking the UI. [`Self::close_and_join`]
/// establishes task destruction and worker completion; invoke it from a lifecycle
/// owner which may block, rather than an event-loop callback.
pub struct FetchActor {
    worker: Option<ActorHandle<Job>>,
    shared: Arc<Shared>,
    out: Emitter<FetchUpdate>,
}

impl FetchActor {
    /// False means work was not admitted. Overload and closed-handle refusals
    /// still produce the existing correlated failure update, so hosts can retire
    /// pending state without introducing a new update variant.
    pub fn command(&self, command: FetchCommand) -> bool {
        match self.shared.admit(command, &self.out) {
            Admission::Control => !self.shared.is_closed(),
            Admission::Rejected => false,
            Admission::Job(job) => self.worker.as_ref().is_some_and(|w| w.command(*job)),
        }
    }

    pub fn is_finished(&self) -> bool {
        self.worker.as_ref().is_none_or(ActorHandle::is_finished)
    }

    /// Fence ingress, cancel and join all tasks, then join the Armillary worker.
    /// Cancellation does not imply that an already started write was undelivered.
    pub fn close_and_join(mut self) -> Result<FetchActorReport, ActorError> {
        self.shared.close();
        self.worker
            .take()
            .expect("fetch actor owns worker")
            .join()?;
        self.shared
            .report
            .lock()
            .expect("fetch report mutex poisoned")
            .ok_or(ActorError::CompletionLost)
    }
}

impl Drop for FetchActor {
    fn drop(&mut self) {
        self.shared.close();
        // Dropping the worker closes ingress; its own run loop performs the join.
        // The caller uses close_and_join when completion must be observed.
    }
}

/// Spawn with bounded ingress and an observable joined shutdown. Configuration
/// is validated before starting a worker or accepting any command.
pub fn spawn_fetcher_with_admission(
    wake: Wake,
    stores: Stores,
    policy: ClientFetchPolicy,
    admission: FetchActorPolicy,
) -> Result<(FetchActor, Receiver<FetchUpdate>), String> {
    policy.validate()?;
    admission.validate()?;
    let shared = Arc::new(Shared::new(admission));
    let state = shared.clone();
    let (ready, started) = mpsc::channel();
    let (worker, updates) = spawn_named("fetch", wake, move |commands: Receiver<Job>, out| {
        let runtime = match build_runtime() {
            Ok(runtime) => runtime,
            Err(error) => {
                let _ = ready.send(Err(error));
                return;
            },
        };
        let _ = ready.send(Ok(out.clone()));
        run_jobs(&runtime, &state, || commands.recv().ok(), stores, policy);
    });
    match started.recv() {
        Ok(Ok(out)) => Ok((
            FetchActor {
                worker: Some(worker),
                shared,
                out,
            },
            updates,
        )),
        result => {
            let _ = worker.join();
            Err(match result {
                Ok(Err(error)) => error,
                _ => "fetch worker stopped before readiness".into(),
            })
        },
    }
}

/// Compatibility entry point. The Armillary command mailbox remains unbounded;
/// use [`spawn_fetcher_with_admission`] to reserve capacity before enqueue.
pub fn spawn_fetcher(wake: Wake) -> (ActorHandle<FetchCommand>, Receiver<FetchUpdate>) {
    spawn_fetcher_with(wake, session_stores().clone())
}

/// Compatibility entry point over host-owned session stores.
pub fn spawn_fetcher_with(
    wake: Wake,
    stores: Stores,
) -> (ActorHandle<FetchCommand>, Receiver<FetchUpdate>) {
    spawn_fetcher_with_policy(wake, stores, ClientFetchPolicy::default())
}

/// Preserve the original handle type and command/update vocabulary. Task
/// execution is bounded after dequeue, and explicit ActorHandle::join waits
/// for the same task-scope cleanup as the bounded companion.
pub fn spawn_fetcher_with_policy(
    wake: Wake,
    stores: Stores,
    policy: ClientFetchPolicy,
) -> (ActorHandle<FetchCommand>, Receiver<FetchUpdate>) {
    spawn_named("fetch", wake, move |commands, out| {
        let runtime = build_runtime().expect("build the fetch runtime");
        let shared = Arc::new(Shared::new(FetchActorPolicy::default()));
        run_jobs(
            &runtime,
            &shared,
            || loop {
                match shared.admit(commands.recv().ok()?, &out) {
                    Admission::Job(job) => return Some(*job),
                    Admission::Control | Admission::Rejected => {},
                }
            },
            stores,
            policy,
        );
        if shared
            .report
            .lock()
            .unwrap()
            .is_some_and(|r| r.tasks.panicked != 0)
        {
            panic!("fetch task panicked");
        }
    })
}

fn build_runtime() -> Result<Runtime, String> {
    Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|_| "could not build fetch runtime".into())
}

fn run_jobs(
    runtime: &Runtime,
    shared: &Arc<Shared>,
    mut next: impl FnMut() -> Option<Job>,
    stores: Stores,
    policy: ClientFetchPolicy,
) {
    let tasks = ResidentTasks::new();
    while let Some(mut job) = next() {
        if shared.is_closed() {
            drop(job);
        } else {
            let _entered = runtime.enter();
            job.spawned = true;
            tasks.spawn(job.run(stores.clone(), policy));
        }
    }
    shared.close();
    let tasks = runtime.block_on(tasks.close_and_join());
    *shared.report.lock().expect("fetch report mutex poisoned") = Some(FetchActorReport {
        tasks,
        admitted: shared.admitted.load(Ordering::Relaxed),
        discarded_before_spawn: shared.discarded_before_spawn.load(Ordering::Relaxed),
        overloaded: shared.overloaded.load(Ordering::Relaxed),
        queue_timeouts: shared.queue_timeouts.load(Ordering::Relaxed),
    });
}

struct Shared {
    active: Arc<Semaphore>,
    pending: Arc<Semaphore>,
    policy: FetchActorPolicy,
    closed: AtomicBool,
    stop: watch::Sender<bool>,
    pages: Mutex<HashMap<FetchRequestId, Weak<Completion>>>,
    admitted: AtomicUsize,
    discarded_before_spawn: AtomicUsize,
    overloaded: AtomicUsize,
    queue_timeouts: AtomicUsize,
    report: Mutex<Option<FetchActorReport>>,
}

enum Admission {
    Control,
    Rejected,
    Job(Box<Job>),
}

impl Shared {
    fn new(policy: FetchActorPolicy) -> Self {
        Self {
            active: Arc::new(Semaphore::new(policy.max_active)),
            pending: Arc::new(Semaphore::new(policy.max_pending)),
            policy,
            closed: AtomicBool::new(false),
            stop: watch::channel(false).0,
            pages: Mutex::new(HashMap::new()),
            admitted: AtomicUsize::new(0),
            discarded_before_spawn: AtomicUsize::new(0),
            overloaded: AtomicUsize::new(0),
            queue_timeouts: AtomicUsize::new(0),
            report: Mutex::new(None),
        }
    }

    fn is_closed(&self) -> bool {
        self.closed.load(Ordering::Acquire)
    }

    fn close(&self) {
        self.closed.store(true, Ordering::Release);
        self.stop.send_replace(true);
    }

    fn admit(self: &Arc<Self>, command: FetchCommand, out: &Emitter<FetchUpdate>) -> Admission {
        if let FetchCommand::CancelPage { request } = command {
            let page = self
                .pages
                .lock()
                .unwrap()
                .get(&request)
                .and_then(Weak::upgrade);
            if let Some(page) = page {
                page.cancel();
            }
            return Admission::Control;
        }
        let completion = Arc::new(Completion::new(&command, out.clone()));
        // Publish the replacement and fence its predecessor atomically. Latest
        // exact-ID intent wins even if it is subsequently refused for overload.
        // Browser requests normally have distinct IDs; URLs are never deduped.
        let old = if let Key::Page { request, .. } = &completion.key {
            let mut pages = self.pages.lock().unwrap();
            let old = pages
                .insert(*request, Arc::downgrade(&completion))
                .and_then(|p| p.upgrade());
            if let Some(old) = &old {
                old.terminal.store(true, Ordering::Release);
            }
            old
        } else {
            None
        };
        if let Some(old) = old {
            old.stop.send_replace(true);
        }
        if completion.terminal.load(Ordering::Acquire) {
            self.retire(&completion);
            return Admission::Rejected;
        }
        if self.is_closed() {
            self.retire(&completion);
            completion.fail("fetch actor is closed");
            return Admission::Rejected;
        }
        let slot = if self.policy.max_active == 0 {
            None
        } else if let Ok(permit) = self.active.clone().try_acquire_owned() {
            Some(Slot::Active { _permit: permit })
        } else {
            self.pending
                .clone()
                .try_acquire_owned()
                .ok()
                .and_then(|permit| {
                    Instant::now()
                        .checked_add(self.policy.queue_timeout)
                        .map(|deadline| Slot::Pending {
                            _permit: permit,
                            deadline,
                        })
                })
        };
        let Some(slot) = slot else {
            self.overloaded.fetch_add(1, Ordering::Relaxed);
            self.retire(&completion);
            completion.fail("fetch actor admission capacity exhausted");
            return Admission::Rejected;
        };
        self.admitted.fetch_add(1, Ordering::Relaxed);
        Admission::Job(Box::new(Job {
            command: Some(command),
            spawned: false,
            slot: Some(slot),
            completion,
            shared: self.clone(),
        }))
    }
    fn retire(&self, completion: &Arc<Completion>) {
        if let Key::Page { request, .. } = &completion.key {
            let mut pages = self.pages.lock().unwrap();
            if pages
                .get(request)
                .is_some_and(|p| p.ptr_eq(&Arc::downgrade(completion)))
            {
                pages.remove(request);
            }
        }
    }
}

enum Key {
    Page {
        request: FetchRequestId,
        url: String,
    },
    Subresource(String),
    Favicon(FetchRequestId),
    Submission {
        request: u64,
        url: String,
    },
}

struct Completion {
    key: Key,
    out: Emitter<FetchUpdate>,
    terminal: AtomicBool,
    started: AtomicBool,
    stop: watch::Sender<bool>,
}

impl Completion {
    fn new(command: &FetchCommand, out: Emitter<FetchUpdate>) -> Self {
        let key = match command {
            FetchCommand::Page { request, url, .. } => Key::Page {
                request: *request,
                url: url.clone(),
            },
            FetchCommand::Subresource(url) => Key::Subresource(url.clone()),
            FetchCommand::Favicon { request, .. } => Key::Favicon(*request),
            FetchCommand::Submit {
                request,
                submission,
            } => Key::Submission {
                request: *request,
                url: submission.url().into(),
            },
            FetchCommand::CancelPage { .. } => unreachable!("control commands do not own tasks"),
        };
        Self {
            key,
            out,
            terminal: AtomicBool::new(false),
            started: AtomicBool::new(false),
            stop: watch::channel(false).0,
        }
    }

    fn finish(&self, update: FetchUpdate) {
        if !self.terminal.swap(true, Ordering::AcqRel) {
            self.out.emit(update);
        }
    }

    fn fail(&self, reason: &str) {
        let update = match &self.key {
            Key::Page { request, url } => FetchUpdate::Page(FetchOutcome {
                request: *request,
                url: url.clone(),
                result: Err(FetchFailure::Failed(reason.into())),
            }),
            Key::Subresource(url) => FetchUpdate::Subresource(SubresourceOutcome {
                url: url.clone(),
                result: Err(reason.into()),
            }),
            Key::Favicon(request) => FetchUpdate::Favicon(FaviconOutcome {
                request: *request,
                result: Err(reason.into()),
            }),
            Key::Submission { request, url } => FetchUpdate::Submission(SubmissionOutcome {
                request: *request,
                url: url.clone(),
                result: Err(FetchFailure::Failed(reason.into())),
            }),
        };
        self.finish(update);
    }

    fn cancel(&self) {
        self.stop.send_replace(true);
        if let Key::Page { request, url } = &self.key {
            self.finish(FetchUpdate::Page(FetchOutcome {
                request: *request,
                url: url.clone(),
                result: Err(FetchFailure::Cancelled),
            }));
        }
    }

    fn stopped(&self) {
        if self.terminal.load(Ordering::Acquire) {
            return;
        }
        if matches!(self.key, Key::Submission { .. }) {
            self.fail(if self.started.load(Ordering::Acquire) {
                "submission stopped; delivery outcome unknown; not retried"
            } else {
                "submission stopped before transport; not sent"
            });
        } else if std::thread::panicking() {
            self.fail("fetch task panicked");
        } else if matches!(self.key, Key::Page { .. }) {
            self.cancel();
        } else {
            self.fail("fetch stopped before completion");
        }
    }
}

enum Slot {
    Active {
        _permit: OwnedSemaphorePermit,
    },
    Pending {
        _permit: OwnedSemaphorePermit,
        deadline: Instant,
    },
}

struct Job {
    command: Option<FetchCommand>,
    spawned: bool,
    completion: Arc<Completion>,
    shared: Arc<Shared>,
    // Dropped last, after the transport future and completion cleanup.
    slot: Option<Slot>,
}

async fn stopped(stop: &mut watch::Receiver<bool>) {
    let _ = stop.wait_for(|value| *value).await;
}

impl Job {
    async fn run(mut self, stores: Stores, policy: ClientFetchPolicy) {
        let mut cancel = self.completion.stop.subscribe();
        let mut closing = self.shared.stop.subscribe();
        if self.shared.is_closed() || self.completion.terminal.load(Ordering::Acquire) {
            return;
        }
        let deadline = match self.slot.as_ref().expect("job owns admission slot") {
            Slot::Active { .. } => None,
            Slot::Pending { deadline, .. } => Some(*deadline),
        };
        if let Some(deadline) = deadline {
            if Instant::now() >= deadline {
                self.queue_expired();
                return;
            }
            let result = tokio::select! {
                biased;
                _ = stopped(&mut cancel) => return,
                _ = stopped(&mut closing) => return,
                result = tokio::time::timeout_at(tokio::time::Instant::from_std(deadline), self.shared.active.clone().acquire_owned()) => result,
            };
            match result {
                Ok(Ok(active)) if Instant::now() < deadline => {
                    self.slot = Some(Slot::Active { _permit: active })
                },
                _ => {
                    self.queue_expired();
                    return;
                },
            }
        }
        if self.shared.is_closed() || self.completion.terminal.load(Ordering::Acquire) {
            return;
        }
        self.completion.started.store(true, Ordering::Release);
        let command = self.command.take().expect("job owns command");
        tokio::select! {
            // Preserve an operation's already-ready result before observing stop.
            biased;
            update = execute(command, stores, policy, self.completion.clone()) => self.completion.finish(update),
            _ = stopped(&mut cancel) => {},
            _ = stopped(&mut closing) => {},
        }
    }

    fn queue_expired(&self) {
        self.shared.queue_timeouts.fetch_add(1, Ordering::Relaxed);
        self.completion
            .fail("fetch actor admission queue timed out");
    }
}

impl Drop for Job {
    fn drop(&mut self) {
        if !self.spawned {
            self.shared
                .discarded_before_spawn
                .fetch_add(1, Ordering::Relaxed);
        }
        // Wake callbacks are host code. A callback panic during cancellation or
        // unwinding must not turn a destructor into a double-panic process abort.
        let _ =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| self.completion.stopped()));
        self.shared.retire(&self.completion);
    }
}

async fn execute(
    command: FetchCommand,
    stores: Stores,
    policy: ClientFetchPolicy,
    completion: Arc<Completion>,
) -> FetchUpdate {
    match command {
        FetchCommand::Page {
            request,
            url,
            identity,
        } => {
            let request_url = url.clone();
            let progress = completion.clone();
            let result = fetch_page_interactive_capped(
                &url,
                policy.max_page_bytes,
                identity.as_ref(),
                &stores,
                policy,
                move |response_url, content_type, bytes| {
                    if !progress.terminal.load(Ordering::Acquire) {
                        progress.out.emit(FetchUpdate::PageProgress(PageProgress {
                            request,
                            url: request_url.clone(),
                            response_url: response_url.to_string(),
                            content_type: content_type.map(str::to_string),
                            bytes: bytes.to_vec(),
                        }));
                    }
                },
            )
            .await;
            FetchUpdate::Page(FetchOutcome {
                request,
                url,
                result,
            })
        },
        FetchCommand::Subresource(url) => {
            let result = fetch_bytes(&url, &stores, policy).await;
            FetchUpdate::Subresource(SubresourceOutcome { url, result })
        },
        FetchCommand::Favicon { request, url } => FetchUpdate::Favicon(FaviconOutcome {
            request,
            result: fetch_bytes(&url, &stores, policy).await,
        }),
        FetchCommand::Submit {
            request,
            submission,
        } => {
            let url = submission.url().into();
            #[cfg(feature = "smolweb")]
            let result = submit_smolweb(submission, policy).await;
            #[cfg(not(feature = "smolweb"))]
            let result = Err(FetchFailure::Failed(
                "small-web submissions are not built into this host".into(),
            ));
            FetchUpdate::Submission(SubmissionOutcome {
                request,
                url,
                result,
            })
        },
        FetchCommand::CancelPage { .. } => unreachable!("controls bypass work admission"),
    }
}

#[cfg(test)]
#[path = "actor_tests.rs"]
mod tests;
