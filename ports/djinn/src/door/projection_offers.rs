// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Owner policy and short-lived application projection registrations.
use chirograph::{
    CarrierNotice, CarrierRequest, CarrierRequestBody, CarrierResponseBody, EndpointDescriptor,
    IntentInvocation, IntentResult, ProjectionRequest, ProjectionSnapshot, ResourceChunkRequest,
    ResourceChunkResponse, ResourceRequest, ResourceResponse, ResumeReply, ResumeRequest,
};
use graphshell_endpoint::door::app_admission::{APP_IDENTITY_ROUTE, AppId, AppRouteId};
use graphshell_endpoint::door::custody::CUSTODY_ROUTE;
use graphshell_endpoint::door::projection_offer::{
    PROJECTION_PROVIDER_ROUTE, ViewerContext, Work, WorkOperation, WorkOutcome,
};
use graphshell_host::endpoint_catalog::ResidentEndpoint;
use graphshell_host::lifecycle::AdmittedEndpointContext;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::{OwnedSemaphorePermit, Semaphore, mpsc};

#[derive(Clone, Debug)]
pub struct ProviderPermission {
    pub provider: AppId,
    pub route: AppRouteId,
    pub viewers: Vec<AppId>,
    pub max_sessions: usize,
    pub timeout: Duration,
}

#[derive(Debug, thiserror::Error)]
#[error("invalid or conflicting projection provider policy")]
pub struct ProviderPolicyError;

#[derive(Default)]
struct ProviderPolicy {
    routes: BTreeMap<AppRouteId, ProviderPermission>,
}
impl ProviderPolicy {
    fn new(permissions: Vec<ProviderPermission>) -> Result<Self, ProviderPolicyError> {
        let mut routes = BTreeMap::new();
        for mut p in permissions {
            let route = p.route.as_str();
            if p.provider.as_str().is_empty()
                || p.provider.as_str().len() > 1024
                || route.len() > 1024
                || AppRouteId::new(route).is_err()
                || [
                    APP_IDENTITY_ROUTE,
                    CUSTODY_ROUTE,
                    PROJECTION_PROVIDER_ROUTE,
                    "knot",
                    "reservoir",
                    "published-site-v1",
                    "device-directory-v1",
                    "resident-status-v1",
                    "resident-control-v1",
                    "inference.ask",
                    "distillery.chronicle",
                ]
                .contains(&route)
                || ["mere/", "ambient/mere/", "coop/"]
                    .iter()
                    .any(|prefix| route.starts_with(prefix))
                || !(1..=64).contains(&p.max_sessions)
                || p.timeout.is_zero()
                || p.timeout > Duration::from_secs(300)
                || p.viewers
                    .iter()
                    .any(|v| v.as_str().is_empty() || v.as_str().len() > 1024)
            {
                return Err(ProviderPolicyError);
            }
            let mut seen = BTreeSet::new();
            p.viewers.retain(|v| seen.insert(v.clone()));
            if routes.insert(p.route.clone(), p).is_some() {
                return Err(ProviderPolicyError);
            }
        }
        Ok(Self { routes })
    }
    fn find(&self, provider: &AppId, route: &AppRouteId) -> Option<&ProviderPermission> {
        self.routes.get(route).filter(|p| &p.provider == provider)
    }
}

#[derive(Debug)]
pub(crate) struct WorkItem {
    pub work: Work,
    pub answer: std::sync::mpsc::SyncSender<WorkOutcome>,
}
pub(crate) struct Lease {
    pub generation: u64,
    pub route: AppRouteId,
    pub permission: ProviderPermission,
    live: AtomicBool,
    next: AtomicU64,
    tx: mpsc::Sender<WorkItem>,
    sessions: Arc<Semaphore>,
    global: Arc<Semaphore>,
    pub cancel: tokio::sync::watch::Sender<bool>,
}
impl Lease {
    pub fn withdraw(&self) {
        self.live.store(false, Ordering::Release);
        self.cancel.send_replace(true);
    }
    pub fn is_live(&self) -> bool {
        self.live.load(Ordering::Acquire)
    }
    pub async fn wait_withdrawn(&self) {
        let mut receiver = self.cancel.subscribe();
        if *receiver.borrow() {
            return;
        }
        let _ = receiver.changed().await;
    }
    fn work(&self, session: &str, operation: WorkOperation) -> Result<WorkOutcome, String> {
        if !self.is_live() {
            return Err("projection offer withdrawn".into());
        }
        let id = self.next.fetch_add(1, Ordering::Relaxed);
        let (tx, rx) = std::sync::mpsc::sync_channel(1);
        self.tx
            .try_send(WorkItem {
                work: Work {
                    id,
                    session: session.to_owned(),
                    operation,
                },
                answer: tx,
            })
            .map_err(|_| "projection provider is busy".to_string())?;
        let deadline = Instant::now() + self.permission.timeout;
        loop {
            if !self.is_live() {
                return Err("projection offer withdrawn".into());
            }
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                self.withdraw();
                return Err("projection provider timed out".into());
            }
            match rx.recv_timeout(remaining.min(Duration::from_millis(25))) {
                Ok(outcome) if self.is_live() => return Ok(outcome),
                Ok(_) => return Err("projection offer withdrawn".into()),
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => (),
                Err(_) => {
                    self.withdraw();
                    return Err("projection provider disconnected".into());
                },
            }
        }
    }
}

pub(crate) struct OfferRegistry {
    policy: Mutex<ProviderPolicy>,
    leases: Mutex<BTreeMap<AppRouteId, Arc<Lease>>>,
    generation: AtomicU64,
    global: Arc<Semaphore>,
}
impl OfferRegistry {
    pub fn new() -> Self {
        Self {
            policy: Mutex::new(ProviderPolicy::default()),
            leases: Mutex::new(BTreeMap::new()),
            generation: AtomicU64::new(1),
            global: Arc::new(Semaphore::new(64)),
        }
    }
    pub fn set_policy(
        &self,
        permissions: Vec<ProviderPermission>,
    ) -> Result<Vec<Arc<Lease>>, ProviderPolicyError> {
        let policy = ProviderPolicy::new(permissions)?;
        *self.policy.lock().unwrap() = policy;
        let leases = std::mem::take(&mut *self.leases.lock().unwrap());
        for lease in leases.values() {
            lease.withdraw();
        }
        Ok(leases.into_values().collect())
    }
    pub fn provider_allowed(&self, provider: &AppId) -> bool {
        self.policy
            .lock()
            .unwrap()
            .routes
            .values()
            .any(|p| &p.provider == provider)
    }
    pub fn viewer_route(&self, viewer: &AppId, route: &AppRouteId) -> bool {
        self.policy
            .lock()
            .unwrap()
            .routes
            .get(route)
            .is_some_and(|p| p.viewers.contains(viewer))
    }
    pub fn new_lease(
        &self,
        provider: &AppId,
        route: &AppRouteId,
    ) -> Result<(Arc<Lease>, mpsc::Receiver<WorkItem>), String> {
        let p = self
            .policy
            .lock()
            .unwrap()
            .find(provider, route)
            .cloned()
            .ok_or("provider not permitted")?;
        let mut leases = self.leases.lock().unwrap();
        if leases.contains_key(route) {
            return Err("projection route is already offered".into());
        }
        let (tx, rx) = mpsc::channel(p.max_sessions.saturating_mul(2).max(1));
        let (cancel, _) = tokio::sync::watch::channel(false);
        let lease = Arc::new(Lease {
            generation: self.generation.fetch_add(1, Ordering::Relaxed),
            route: route.clone(),
            sessions: Arc::new(Semaphore::new(p.max_sessions)),
            global: self.global.clone(),
            permission: p,
            live: AtomicBool::new(true),
            next: AtomicU64::new(1),
            tx,
            cancel,
        });
        leases.insert(route.clone(), lease.clone());
        Ok((lease, rx))
    }
    pub fn retire(&self, lease: &Arc<Lease>) {
        lease.withdraw();
        let mut leases = self.leases.lock().unwrap();
        if leases
            .get(&lease.route)
            .is_some_and(|v| v.generation == lease.generation)
        {
            leases.remove(&lease.route);
        }
    }
}

pub(crate) struct ProviderEndpoint {
    lease: Arc<Lease>,
    context: ViewerContext,
    descriptor: Option<EndpointDescriptor>,
    notice: Option<std::sync::mpsc::Receiver<WorkOutcome>>,
    next_notice: Instant,
    _local: OwnedSemaphorePermit,
    _global: OwnedSemaphorePermit,
}
impl ProviderEndpoint {
    pub fn new(lease: Arc<Lease>, context: &AdmittedEndpointContext) -> Result<Self, String> {
        let viewer = context
            .application()
            .map(AppId::new)
            .ok_or("projection viewer has no application")?;
        if !lease.is_live() || !lease.permission.viewers.contains(&viewer) {
            return Err("viewer not permitted".into());
        }
        let local = lease
            .sessions
            .clone()
            .try_acquire_owned()
            .map_err(|_| "projection session limit reached")?;
        let global = lease
            .global
            .clone()
            .try_acquire_owned()
            .map_err(|_| "global projection session limit reached")?;
        Ok(Self {
            context: ViewerContext {
                session: context.session().0.clone(),
                subject: context.subject(),
                application: Some(viewer),
            },
            descriptor: None,
            notice: None,
            next_notice: Instant::now(),
            lease,
            _local: local,
            _global: global,
        })
    }
    fn request(&self, body: CarrierRequestBody) -> Result<CarrierResponseBody, String> {
        let id = 1;
        match self.lease.work(
            &self.context.session,
            WorkOperation::Request {
                request: CarrierRequest { id, body },
            },
        )? {
            WorkOutcome::Response { response } if response.id == id => {
                response.body.map_err(|e| e.message)
            },
            _ => {
                self.lease.withdraw();
                Err("invalid projection provider response".into())
            },
        }
    }
}
impl ResidentEndpoint for ProviderEndpoint {
    fn prepare(&mut self) -> Result<(), String> {
        match self.lease.work(
            &self.context.session,
            WorkOperation::Open {
                context: self.context.clone(),
            },
        )? {
            WorkOutcome::Opened { descriptor } if self.lease.is_live() => {
                self.descriptor = Some(descriptor);
                Ok(())
            },
            _ => Err("projection provider could not open session".into()),
        }
    }
    fn requires_blocking(&self) -> bool {
        true
    }
    fn is_live(&self) -> bool {
        self.lease.is_live()
    }
    fn cancellation(&self) -> Option<tokio::sync::watch::Receiver<bool>> {
        Some(self.lease.cancel.subscribe())
    }
    fn describe(&self) -> EndpointDescriptor {
        self.descriptor
            .clone()
            .expect("provider endpoint prepared before use")
    }
    fn snapshot(&mut self, r: ProjectionRequest) -> Result<ProjectionSnapshot, String> {
        let session = r.session.clone();
        match self.request(CarrierRequestBody::Snapshot(r))? {
            CarrierResponseBody::Snapshot(v) if v.session == session => Ok(*v),
            _ => {
                self.lease.withdraw();
                Err("invalid snapshot response".into())
            },
        }
    }
    fn resource(&mut self, r: ResourceRequest) -> Result<ResourceResponse, String> {
        let session = r.session.clone();
        let resource = r.resource;
        match self.request(CarrierRequestBody::Resource(r))? {
            CarrierResponseBody::Resource(v) if v.session == session && v.resource == resource => {
                Ok(v)
            },
            _ => {
                self.lease.withdraw();
                Err("invalid resource response".into())
            },
        }
    }
    fn resource_chunk(&mut self, r: ResourceChunkRequest) -> Result<ResourceChunkResponse, String> {
        let session = r.session.clone();
        let resource = r.resource;
        let offset = r.offset;
        match self.request(CarrierRequestBody::ResourceChunk(r))? {
            CarrierResponseBody::ResourceChunk(v)
                if v.session == session && v.resource == resource && v.offset == offset =>
            {
                Ok(*v)
            },
            _ => {
                self.lease.withdraw();
                Err("invalid resource response".into())
            },
        }
    }
    fn invoke(&mut self, r: IntentInvocation) -> Result<IntentResult, String> {
        match self.request(CarrierRequestBody::Intent(r))? {
            CarrierResponseBody::Intent(v) => Ok(v),
            _ => {
                self.lease.withdraw();
                Err("invalid intent response".into())
            },
        }
    }
    fn resume(&mut self, r: ResumeRequest) -> Result<ResumeReply, String> {
        match self.request(CarrierRequestBody::Resume(r))? {
            CarrierResponseBody::Resume(v) => Ok(v),
            _ => {
                self.lease.withdraw();
                Err("invalid resume response".into())
            },
        }
    }
    fn poll_notice(&mut self) -> Result<Option<CarrierNotice>, String> {
        if !self.lease.is_live() {
            return Err("projection offer withdrawn".into());
        }
        if let Some(rx) = &self.notice {
            match rx.try_recv() {
                Ok(WorkOutcome::Notice { notice }) => {
                    self.notice = None;
                    return Ok(notice);
                },
                Ok(_) | Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.lease.withdraw();
                    return Err("invalid projection notice".into());
                },
                Err(std::sync::mpsc::TryRecvError::Empty) => return Ok(None),
            }
        }
        if Instant::now() < self.next_notice {
            return Ok(None);
        }
        self.next_notice = Instant::now() + Duration::from_secs(1);
        let (answer, receiver) = std::sync::mpsc::sync_channel(1);
        let work = Work {
            id: self.lease.next.fetch_add(1, Ordering::Relaxed),
            session: self.context.session.clone(),
            operation: WorkOperation::PollNotice,
        };
        if self.lease.tx.try_send(WorkItem { work, answer }).is_ok() {
            self.notice = Some(receiver);
        }
        Ok(None)
    }
}
impl Drop for ProviderEndpoint {
    fn drop(&mut self) {
        if self.descriptor.is_none() || !self.lease.is_live() {
            return;
        }
        if self
            .lease
            .tx
            .try_send(WorkItem {
                work: Work {
                    id: self.lease.next.fetch_add(1, Ordering::Relaxed),
                    session: self.context.session.clone(),
                    operation: WorkOperation::Close,
                },
                answer: std::sync::mpsc::sync_channel(1).0,
            })
            .is_err()
        {
            self.lease.withdraw();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn permission() -> ProviderPermission {
        ProviderPermission {
            provider: AppId::new("example"),
            route: AppRouteId::new("example/notes").unwrap(),
            viewers: vec![AppId::new("viewer")],
            max_sessions: 2,
            timeout: Duration::from_secs(2),
        }
    }

    #[test]
    fn replacement_cancels_lease_and_stale_cleanup_preserves_new_lease() {
        let registry = OfferRegistry::new();
        registry.set_policy(vec![permission()]).unwrap();
        let (first, _rx) = registry
            .new_lease(&AppId::new("example"), &permission().route)
            .unwrap();
        assert!(
            registry
                .new_lease(&AppId::new("example"), &permission().route)
                .is_err()
        );
        registry.set_policy(vec![permission()]).unwrap();
        assert!(!first.is_live());
        let (second, _rx) = registry
            .new_lease(&AppId::new("example"), &permission().route)
            .unwrap();
        registry.retire(&first);
        assert!(second.is_live());
        assert!(
            registry
                .new_lease(&AppId::new("example"), &permission().route)
                .is_err()
        );
    }

    #[test]
    fn permissions_do_not_automatically_admit_another_viewer() {
        let registry = OfferRegistry::new();
        registry.set_policy(vec![permission()]).unwrap();
        assert!(registry.viewer_route(&AppId::new("viewer"), &permission().route));
        assert!(!registry.viewer_route(&AppId::new("other"), &permission().route));
    }

    #[test]
    fn mismatched_carrier_response_withdraws_the_lease() {
        let registry = OfferRegistry::new();
        registry.set_policy(vec![permission()]).unwrap();
        let (lease, mut rx) = registry
            .new_lease(&AppId::new("example"), &permission().route)
            .unwrap();
        let context =
            AdmittedEndpointContext::new(chirograph::ProjectionSession("viewer".into()), [0; 32])
                .with_application("viewer");
        let mut endpoint = ProviderEndpoint::new(lease.clone(), &context).unwrap();
        let worker = std::thread::spawn(move || {
            endpoint.resource(ResourceRequest {
                session: chirograph::ProjectionSession("product".into()),
                resource: chirograph::ContentHash::of(b"fixture"),
            })
        });
        let item = loop {
            if let Ok(item) = rx.try_recv() {
                break item;
            }
            std::thread::yield_now();
        };
        let WorkOperation::Request { request } = item.work.operation else {
            panic!("expected request");
        };
        item.answer
            .send(WorkOutcome::Response {
                response: chirograph::CarrierResponse {
                    id: request.id,
                    body: Ok(CarrierResponseBody::Closed),
                },
            })
            .unwrap();
        assert!(worker.join().unwrap().is_err());
        assert!(!lease.is_live());
    }

    #[test]
    fn unserved_work_times_out_and_withdraws_the_lease() {
        let registry = OfferRegistry::new();
        let mut allowed = permission();
        allowed.timeout = Duration::from_millis(20);
        registry.set_policy(vec![allowed]).unwrap();
        let (lease, _rx) = registry
            .new_lease(&AppId::new("example"), &permission().route)
            .unwrap();
        assert!(lease.work("viewer", WorkOperation::PollNotice).is_err());
        assert!(!lease.is_live());
    }

    #[tokio::test]
    async fn a_late_cancellation_watcher_completes_immediately() {
        let registry = OfferRegistry::new();
        registry.set_policy(vec![permission()]).unwrap();
        let (lease, _rx) = registry
            .new_lease(&AppId::new("example"), &permission().route)
            .unwrap();
        lease.withdraw();
        tokio::time::timeout(Duration::from_millis(100), lease.wait_withdrawn())
            .await
            .expect("already withdrawn");
    }

    #[test]
    fn withdrawing_a_lease_wakes_an_inflight_worker() {
        let registry = OfferRegistry::new();
        registry.set_policy(vec![permission()]).unwrap();
        let (lease, mut rx) = registry
            .new_lease(&AppId::new("example"), &permission().route)
            .unwrap();
        let worker_lease = lease.clone();
        let worker =
            std::thread::spawn(move || worker_lease.work("session", WorkOperation::PollNotice));
        let item = loop {
            if let Ok(item) = rx.try_recv() {
                break item;
            }
            std::thread::yield_now();
        };
        lease.withdraw();
        assert!(worker.join().unwrap().is_err());
        assert!(
            item.answer
                .send(WorkOutcome::Notice { notice: None })
                .is_err()
        );
    }

    #[test]
    fn a_full_close_queue_withdraws_the_lease() {
        let registry = OfferRegistry::new();
        registry.set_policy(vec![permission()]).unwrap();
        let (lease, _rx) = registry
            .new_lease(&AppId::new("example"), &permission().route)
            .unwrap();
        let context = AdmittedEndpointContext::new(
            chirograph::ProjectionSession("viewer-session".into()),
            [0; 32],
        )
        .with_application("viewer");
        let mut endpoint = ProviderEndpoint::new(lease.clone(), &context).unwrap();
        endpoint.descriptor = Some(EndpointDescriptor {
            label: "fixture".into(),
            projections: vec![],
        });
        for _ in 0..(permission().max_sessions * 2) {
            let (answer, _) = std::sync::mpsc::sync_channel(1);
            lease
                .tx
                .try_send(WorkItem {
                    work: Work {
                        id: 0,
                        session: "viewer-session".into(),
                        operation: WorkOperation::PollNotice,
                    },
                    answer,
                })
                .unwrap();
        }
        drop(endpoint);
        assert!(!lease.is_live());
    }

    #[test]
    fn provider_permission_is_explicit_and_bounded() {
        let policy = ProviderPolicy::default();
        assert!(
            policy
                .find(&AppId::new("example"), &permission().route)
                .is_none()
        );
        let mut invalid = permission();
        invalid.max_sessions = 0;
        assert!(ProviderPolicy::new(vec![invalid]).is_err());
        let mut invalid = permission();
        invalid.timeout = Duration::ZERO;
        assert!(ProviderPolicy::new(vec![invalid]).is_err());
        let mut invalid = permission();
        invalid.provider = AppId::new("");
        assert!(ProviderPolicy::new(vec![invalid]).is_err());
        let policy = ProviderPolicy::new(vec![permission()]).unwrap();
        assert!(
            policy
                .find(&AppId::new("example"), &permission().route)
                .is_some()
        );
        assert!(
            policy
                .find(&AppId::new("other"), &permission().route)
                .is_none()
        );
    }

    #[test]
    fn permission_rejects_unbounded_session_and_wait_limits() {
        let mut invalid = permission();
        invalid.max_sessions = 65;
        assert!(ProviderPolicy::new(vec![invalid]).is_err());
        let mut invalid = permission();
        invalid.timeout = Duration::from_secs(301);
        assert!(ProviderPolicy::new(vec![invalid]).is_err());
    }

    #[test]
    fn permissions_reject_reserved_routes_and_duplicate_routes() {
        let mut reserved = permission();
        reserved.route = AppRouteId::new("identity").unwrap();
        assert!(ProviderPolicy::new(vec![reserved]).is_err());
        for route in [
            "knot",
            "inference.ask",
            "mere/example",
            "ambient/mere/example",
            "coop/example",
        ] {
            let mut reserved = permission();
            reserved.route = AppRouteId::new(route).unwrap();
            assert!(ProviderPolicy::new(vec![reserved]).is_err());
        }
        assert!(ProviderPolicy::new(vec![permission(), permission()]).is_err());
    }
}
