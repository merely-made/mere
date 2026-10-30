// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The first-party application door.
//!
//! Serves `djinn::door::app_admission`'s endpoint: an owner-only socket that a
//! first-party application — turnstone, today — opens to browse this device's
//! graph. Everything past the hello is the same machinery the browser door
//! uses, from `local_session`, so the two doors cannot come to two different
//! answers about what an admitted local client holds.
//!
//! **What is different here, and why.** Two things only:
//!
//! - *The wire.* An application speaks [`AppMessage`] (the shared envelope in
//!   `djinn::door::wire`), not the browser's
//!   native-messaging frames. The carrier payload inside is identical; the
//!   envelope is separate so this door can version independently of a shipped
//!   browser extension.
//! - *No identity actions.* The browser door carries SSH import, because a
//!   browser has no other way to reach the resident identity UI. A first-party
//!   application runs on this device and can open that UI itself, so the door
//!   does not forward identity actions at all rather than forwarding them and
//!   relying on a session check to refuse.

use crate::door::projection_offers::{
    OfferRegistry, ProviderEndpoint, ProviderPermission, ProviderPolicyError,
};
use graphshell_endpoint::door::projection_offer::{
    PROJECTION_PROVIDER_ROUTE, ProviderAnswer, ProviderCall, WorkOperation, WorkOutcome, WorkResult,
};
use std::sync::Arc;
use std::sync::RwLock as StdRwLock;

use armillary::tasks::spawn_tracked_with_handle;
use chirograph::{CarrierRequestBody, CarrierResponseBody, ResumeRequest};
use djinn::door::app_admission::{
    APP_IDENTITY_ROUTE, AllowedAppRoutes, AppAdmissionError, AppHello, AppId, AppRouteGrants,
    AppRouteId,
};
use djinn::door::custody::CUSTODY_ROUTE;
use djinn::door::declaration::AppAuthority;
use djinn::door::local_endpoint::{LocalListener, LocalStream, connect_local};
use djinn::door::route::ResidentEndpointRoute;
#[cfg(test)]
use djinn::door::wire::APP_CONNECT_SCHEMA;
use djinn::door::wire::{AppHostMessage as WireHostMessage, AppMessage as WireMessage};
use tokio::io::{AsyncRead, AsyncWrite};
type AppMessage = WireMessage<chirograph::CarrierRequest, djinn::door::custody::CustodyCall>;
type AppHostMessage = WireHostMessage<
    chirograph::CarrierResponse,
    chirograph::CarrierNotice,
    Result<djinn::door::custody::CustodyAnswer, djinn::door::custody::CustodyRefusal>,
>;

use crate::door::browser_carrier::{read_native_message_async, write_native_message_async};
use crate::door::browser_host::BrowserHostError;
use crate::door::device_broker::{DeviceSurface, DeviceSurfaceHandle};
use crate::door::identity_endpoint::IdentityEndpoint;
use crate::door::local_session::{LocalSession, admit_local_client, identity_endpoint_for};
use crate::door::resident_identity::ResidentIdentity;
use djinn::door::link::LocalLink;
use graphshell_host::endpoint_catalog::{
    ResidentEndpointCatalog, ResidentEndpointCatalogError, ResidentEndpointSession,
};
use graphshell_host::lifecycle::SessionAuthority;
use graphshell_host::session_loop::{SessionSummary, serve_admitted_session};
use graphshell_host::session_notices::{
    serve_admitted_session_notifying, serve_admitted_session_notifying_checked,
};

#[derive(Debug, thiserror::Error)]
pub enum AppBrokerError {
    #[error(transparent)]
    Door(#[from] djinn::door::server::AppServerError),
    #[error(transparent)]
    Admission(#[from] AppAdmissionError),
    #[error(transparent)]
    Host(#[from] BrowserHostError),
    #[error(transparent)]
    Catalog(#[from] ResidentEndpointCatalogError),
    #[error(transparent)]
    Carrier(#[from] crate::door::browser_carrier::BrowserCarrierError),
    #[error("projection provider failed: {0}")]
    Provider(String),
    #[error("application broker transport failed: {0}")]
    Io(#[from] std::io::Error),
}

/// The resident endpoint registrations available to first-party routes.
///
/// The mutex protects only factory selection. A session takes ownership of
/// the endpoint it opens, so product traffic never serializes on the catalog.
#[derive(Clone)]
pub struct AppEndpointCatalog {
    inner: Arc<tokio::sync::Mutex<ResidentEndpointCatalog>>,
    pub(crate) offers: Arc<OfferRegistry>,
}
impl Default for AppEndpointCatalog {
    fn default() -> Self {
        Self::new(ResidentEndpointCatalog::new())
    }
}

/// Dynamic registrars must not own the catalog that owns their factories.
#[derive(Clone)]
pub struct WeakAppEndpointCatalog {
    inner: std::sync::Weak<tokio::sync::Mutex<ResidentEndpointCatalog>>,
    offers: std::sync::Weak<OfferRegistry>,
}
impl WeakAppEndpointCatalog {
    pub fn upgrade(&self) -> Option<AppEndpointCatalog> {
        self.inner.upgrade().and_then(|inner| {
            self.offers
                .upgrade()
                .map(|offers| AppEndpointCatalog { inner, offers })
        })
    }
}

impl AppEndpointCatalog {
    pub fn new(catalog: ResidentEndpointCatalog) -> Self {
        Self {
            inner: Arc::new(tokio::sync::Mutex::new(catalog)),
            offers: Arc::new(OfferRegistry::new()),
        }
    }

    pub fn downgrade(&self) -> WeakAppEndpointCatalog {
        WeakAppEndpointCatalog {
            inner: Arc::downgrade(&self.inner),
            offers: Arc::downgrade(&self.offers),
        }
    }

    /// Replace owner-granted provider permissions, withdrawing every live offer.
    /// Viewer route denials retain their existing next-connection semantics;
    /// replacing this policy also cancels all active offered sessions.
    pub async fn set_provider_permissions(
        &self,
        permissions: Vec<ProviderPermission>,
    ) -> Result<(), ProviderPolicyError> {
        let mut catalog = self.inner.lock().await;
        for lease in self.offers.set_policy(permissions)? {
            catalog.remove_owned(lease.route.as_str(), lease.generation);
        }
        Ok(())
    }

    async fn open(
        &self,
        id: &str,
        context: &graphshell_host::lifecycle::AdmittedEndpointContext,
    ) -> Result<ResidentEndpointSession, ResidentEndpointCatalogError> {
        self.inner.lock().await.open(id, context)
    }

    /// Change the registrations while the door serves: a route registered
    /// here opens for the next session that selects it.
    pub async fn update<R>(&self, change: impl FnOnce(&mut ResidentEndpointCatalog) -> R) -> R {
        change(&mut *self.inner.lock().await)
    }
}

/// Open a connection to the resident host as `app`.
///
/// The application side of the door, for a first-party client that lives in
/// this workspace.
pub async fn connect_as_app(
    endpoint: &str,
    app: AppId,
) -> Result<Box<dyn LocalStream>, AppBrokerError> {
    let mut stream = connect_local(endpoint).await?;
    write_native_message_async(&mut stream, &AppHello::new(app)).await?;
    Ok(stream)
}

/// Open a route-aware connection to the resident host as `app`.
pub async fn connect_as_app_route(
    endpoint: &str,
    app: AppId,
    route: AppRouteId,
) -> Result<Box<dyn LocalStream>, AppBrokerError> {
    let mut stream = connect_local(endpoint).await?;
    write_native_message_async(&mut stream, &AppHello::for_route(app, route)).await?;
    Ok(stream)
}

/// Serve first-party applications from the resident authority.
pub async fn serve_app_broker<A>(
    endpoint: &str,
    personae: Arc<A>,
    grants: AppRouteGrants,
    session_duration_ms: u64,
    surface: Option<DeviceSurfaceHandle>,
    catalog: AppEndpointCatalog,
) -> Result<(), AppBrokerError>
where
    A: ResidentIdentity,
{
    let listener = LocalListener::bind(endpoint, "first-party").await?;
    serve_app_broker_on(
        listener,
        personae,
        grants,
        session_duration_ms,
        surface,
        catalog,
    )
    .await
}

/// Serve first-party applications on an already bound owner-only listener.
pub async fn serve_app_broker_on<A>(
    listener: LocalListener,
    personae: Arc<A>,
    grants: AppRouteGrants,
    session_duration_ms: u64,
    surface: Option<DeviceSurfaceHandle>,
    catalog: AppEndpointCatalog,
) -> Result<(), AppBrokerError>
where
    A: ResidentIdentity,
{
    djinn::door::server::serve_on(
        listener,
        grants,
        move |stream: Box<dyn LocalStream>, allowed| {
            let personae = Arc::clone(&personae);
            let surface = surface.clone();
            let catalog = catalog.clone();
            async move {
                let (mut reader, mut writer) = tokio::io::split(stream);
                if let Err(error) = serve_app_connection(
                    &mut reader,
                    &mut writer,
                    personae,
                    &allowed,
                    session_duration_ms,
                    surface,
                    catalog,
                )
                .await
                {
                    tracing::warn!(%error, "first-party session failed");
                }
            }
        },
    )
    .await?;
    Ok(())
}

#[cfg(test)]
pub(crate) use serve_app_connection as serve_app_connection_for_tests;

pub(crate) async fn serve_app_connection<A, R, W>(
    reader: &mut R,
    writer: &mut W,
    personae: Arc<A>,
    allowed: &AllowedAppRoutes,
    session_duration_ms: u64,
    surface: Option<DeviceSurfaceHandle>,
    catalog: AppEndpointCatalog,
) -> Result<Option<SessionSummary>, AppBrokerError>
where
    A: ResidentIdentity,
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    let admitted = djinn::door::server::admit(reader, allowed, |request, allowed| {
        if request.route.as_str() == PROJECTION_PROVIDER_ROUTE {
            if !catalog.offers.provider_allowed(&request.app) {
                return Err(AppAdmissionError::NotAllowed(request.app.clone()).into());
            }
            let mut allowed = allowed.clone();
            allowed.grant(
                request.app.clone(),
                ResidentEndpointRoute::new(
                    PROJECTION_PROVIDER_ROUTE,
                    std::time::Duration::from_millis(50),
                )
                .expect("valid provider route"),
            );
            allowed.admit(request)
        } else {
            let mut allowed = allowed.clone();
            if catalog.offers.viewer_route(&request.app, &request.route) {
                allowed.grant(
                    request.app.clone(),
                    ResidentEndpointRoute::new(
                        request.route.as_str(),
                        std::time::Duration::from_millis(50),
                    )
                    .expect("validated policy route"),
                );
            }
            allowed.admit(request)
        }
    })
    .await?;
    let djinn::door::server::AdmittedApp {
        app,
        route,
        authority,
    } = admitted;
    // Read once, at session start, exactly as the browser door does: a
    // transfer accepted while an application is already connected becomes
    // visible on its next session rather than mid-stream.
    let surface = match surface {
        Some(surface) => surface.read().await.clone(),
        None => DeviceSurface::default(),
    };
    let Some(link) = djinn::door::server::connect(reader, writer, &app).await? else {
        return Ok(None);
    };
    serve_admitted_app(
        reader,
        writer,
        personae,
        app,
        authority,
        route,
        link,
        session_duration_ms,
        surface,
        catalog,
    )
    .await
}

async fn serve_admitted_app<A, R, W>(
    reader: &mut R,
    writer: &mut W,
    personae: Arc<A>,
    app: AppId,
    authority: Option<AppAuthority>,
    route: ResidentEndpointRoute,
    link: LocalLink,
    session_duration_ms: u64,
    surface: DeviceSurface,
    catalog: AppEndpointCatalog,
) -> Result<Option<SessionSummary>, AppBrokerError>
where
    A: ResidentIdentity,
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    if route.id() == PROJECTION_PROVIDER_ROUTE {
        return serve_provider(reader, writer, app, authority, catalog).await;
    }

    // Custody is calls, not a projection, and it must answer while the vault
    // is Locked (D12: the app shows the identity pending). So it needs no
    // door keys and no admitted carrier session: reaching the owner-only
    // endpoint is the trust root, as for every route.
    if route.id() == CUSTODY_ROUTE {
        return serve_custody(reader, writer, personae, app, authority, &link).await;
    }

    let identity = Arc::clone(&personae);
    let LocalSession {
        client: mut carrier,
        mut admitted,
        revocations,
    } = admit_local_client(identity.as_ref(), &link, session_duration_ms).await?;

    let authority = SessionAuthority::retain_admitted(&admitted);
    let endpoint_context = authority.endpoint_context().with_application(app.as_str());
    let session = endpoint_context.session().0.clone();
    let mut outer_cancel = None;
    let server = if route.id() == APP_IDENTITY_ROUTE {
        let mut endpoint = identity_endpoint_for(Arc::clone(&personae), &authority, surface);
        spawn_tracked_with_handle(async move {
            let revocations = StdRwLock::new(revocations);
            let mut resume = |_: &mut IdentityEndpoint<A>, _: ResumeRequest| {
                Err("identity resume is not implemented".to_string())
            };
            serve_admitted_session(
                &mut admitted,
                &authority,
                &revocations,
                &mut endpoint,
                &mut resume,
                crate::door::browser_host::now_ms,
            )
            .await
        })
    } else {
        // Grant selection happened before the Graphshell challenge, but the
        // product factory opens only after Graphshell admission has produced
        // this narrow context.
        let mut endpoint = catalog.open(route.id(), &endpoint_context).await?;
        let notice_poll_interval = route.notice_poll_interval();
        if endpoint.requires_blocking() {
            let mut cancellation = endpoint
                .cancellation()
                .ok_or_else(|| AppBrokerError::Provider("provider lacks cancellation".into()))?;
            outer_cancel = Some(cancellation.clone());
            let cancellation_check = cancellation.clone();
            let (ready_tx, ready_rx) = tokio::sync::oneshot::channel();
            let handle = tokio::runtime::Handle::current();
            let worker = tokio::task::spawn_blocking(move || {
                if let Err(error) = endpoint.prepare() {
                    let _ = ready_tx.send(Err(error));
                    return Err(graphshell_host::session_loop::SessionLoopError::Notices(
                        "provider could not prepare".into(),
                    ));
                }
                if !endpoint.is_live() {
                    let _ = ready_tx.send(Err("projection offer withdrawn".into()));
                    return Err(graphshell_host::session_loop::SessionLoopError::Notices(
                        "projection offer withdrawn".into(),
                    ));
                }
                let _ = ready_tx.send(Ok(()));
                handle.block_on(async move {
                    let revocations = StdRwLock::new(revocations);
                    let mut resume = |endpoint: &mut ResidentEndpointSession, request: ResumeRequest| endpoint.resume(request);
                    tokio::select! {
                        biased;
                        _ = async { if !*cancellation.borrow() { let _ = cancellation.changed().await; } } => Err(graphshell_host::session_loop::SessionLoopError::Notices("projection offer withdrawn".into())),
                        result = serve_admitted_session_notifying_checked(&mut admitted, &authority, &revocations, &mut endpoint, &mut resume, crate::door::browser_host::now_ms, notice_poll_interval, ResidentEndpointSession::is_live) => result,
                    }
                })
            });
            ready_rx
                .await
                .map_err(|_| AppBrokerError::Provider("provider worker stopped".into()))?
                .map_err(AppBrokerError::Provider)?;
            if *cancellation_check.borrow() {
                return Err(AppBrokerError::Provider(
                    "projection offer withdrawn".into(),
                ));
            }
            spawn_tracked_with_handle(async move {
                worker.await.map_err(|_| {
                    graphshell_host::session_loop::SessionLoopError::Notices(
                        "provider worker stopped".into(),
                    )
                })?
            })
        } else {
            spawn_tracked_with_handle(async move {
                let revocations = StdRwLock::new(revocations);
                let mut resume = |endpoint: &mut ResidentEndpointSession,
                                  request: ResumeRequest| {
                    endpoint.resume(request)
                };
                serve_admitted_session_notifying(
                    &mut admitted,
                    &authority,
                    &revocations,
                    &mut endpoint,
                    &mut resume,
                    crate::door::browser_host::now_ms,
                    notice_poll_interval,
                )
                .await
            })
        }
    };
    {
        let forwarding = async {
            write_native_message_async(
                writer,
                &AppHostMessage::Connected {
                    app: app.clone(),
                    session,
                },
            )
            .await?;

            while let Some(message) = read_native_message_async::<_, AppMessage>(reader).await? {
                match message {
                    AppMessage::Request { request } => {
                        let terminal = matches!(
                            request.body,
                            CarrierRequestBody::Close | CarrierRequestBody::Suspend
                        );
                        let response = carrier.request(&request).await?;
                        let ended = matches!(
                            response.body,
                            Ok(CarrierResponseBody::Closed | CarrierResponseBody::Suspended)
                        );
                        write_native_message_async(writer, &AppHostMessage::Response { response })
                            .await?;
                        if terminal || ended {
                            break;
                        }
                    },
                    AppMessage::TakeNotice => {
                        // Notices from a quiet endpoint are still in its duplex
                        // stream until read. Bounded, cancel-safe intake makes idle
                        // polls receive them without issuing a user command.
                        let notice = carrier
                            .poll_notice(std::time::Duration::from_millis(1))
                            .await?;
                        write_native_message_async(writer, &AppHostMessage::Notice { notice })
                            .await?;
                    },
                    AppMessage::WaitNotice => {
                        let notice = carrier.wait_for_notice().await?;
                        write_native_message_async(
                            writer,
                            &AppHostMessage::Notice {
                                notice: Some(notice),
                            },
                        )
                        .await?;
                    },
                    AppMessage::Connect { .. } => {
                        write_native_message_async(
                            writer,
                            &AppHostMessage::Failure {
                                message: "this application session is already connected"
                                    .to_string(),
                            },
                        )
                        .await?;
                    },
                    AppMessage::Call { .. } => {
                        write_native_message_async(
                            writer,
                            &AppHostMessage::Failure {
                                message: "custody calls belong to the custody route".to_string(),
                            },
                        )
                        .await?;
                    },
                }
            }
            Ok::<(), AppBrokerError>(())
        };
        tokio::pin!(forwarding);
        tokio::select! {
            biased;
            _ = async {
                if let Some(receiver) = outer_cancel.as_mut() {
                    if !*receiver.borrow() { let _ = receiver.changed().await; }
                } else {
                    std::future::pending::<()>().await;
                }
            } => (),
            result = &mut forwarding => { result?; },
        }
    }

    drop(carrier);
    let summary = server
        .await
        .map_err(BrowserHostError::from)?
        .map_err(BrowserHostError::from)?;
    Ok(Some(summary))
}

/// Serve the custody route: one call at a time, each answered by the
/// resident authority on behalf of `app`.
async fn serve_custody<A, R, W>(
    reader: &mut R,
    writer: &mut W,
    personae: Arc<A>,
    app: AppId,
    authority: Option<AppAuthority>,
    link: &LocalLink,
) -> Result<Option<SessionSummary>, AppBrokerError>
where
    A: ResidentIdentity,
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    let session = format!("custody:{}", blake3::hash(&link.shared_link).to_hex());
    write_native_message_async(
        writer,
        &AppHostMessage::Connected {
            app: app.clone(),
            session,
        },
    )
    .await?;
    while let Some(message) = read_native_message_async::<_, AppMessage>(reader).await? {
        let reply = match message {
            AppMessage::Call { id, call } => AppHostMessage::Answer {
                id,
                answer: Arc::clone(&personae)
                    .custody(app.clone(), authority.clone(), call)
                    .await,
            },
            _ => AppHostMessage::Failure {
                message: "the custody route answers custody calls only".to_string(),
            },
        };
        write_native_message_async(writer, &reply).await?;
    }
    Ok(None)
}

struct ProviderLeaseGuard {
    catalog: AppEndpointCatalog,
    lease: Arc<crate::door::projection_offers::Lease>,
}
impl Drop for ProviderLeaseGuard {
    fn drop(&mut self) {
        self.catalog.offers.retire(&self.lease);
        let inner = self.catalog.inner.clone();
        let route = self.lease.route.clone();
        let generation = self.lease.generation;
        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            handle.spawn(async move {
                inner.lock().await.remove_owned(route.as_str(), generation);
            });
        }
    }
}

fn valid_provider_result(
    work: &graphshell_endpoint::door::projection_offer::Work,
    result: &WorkResult,
) -> bool {
    work.id == result.id
        && work.session == result.session
        && matches!(
            (&work.operation, &result.result),
            (
                WorkOperation::Open { .. },
                WorkOutcome::Opened { .. } | WorkOutcome::Failed { .. }
            ) | (
                WorkOperation::Request { .. },
                WorkOutcome::Response { .. } | WorkOutcome::Failed { .. }
            ) | (WorkOperation::PollNotice, WorkOutcome::Notice { .. })
                | (WorkOperation::Close, WorkOutcome::Closed)
        )
}

type ProviderMessage = WireMessage<(), ProviderCall>;
type ProviderHostMessage = WireHostMessage<(), (), ProviderAnswer>;

async fn write_provider_answer<W: AsyncWrite + Unpin>(
    writer: &mut W,
    id: u64,
    answer: ProviderAnswer,
    timeout: std::time::Duration,
) -> Result<(), AppBrokerError> {
    tokio::time::timeout(
        timeout,
        write_native_message_async(writer, &ProviderHostMessage::Answer { id, answer }),
    )
    .await
    .map_err(|_| AppBrokerError::Provider("provider write timed out".into()))??;
    Ok(())
}

async fn serve_provider<R: AsyncRead + Unpin, W: AsyncWrite + Unpin>(
    reader: &mut R,
    writer: &mut W,
    app: AppId,
    authority: Option<AppAuthority>,
    catalog: AppEndpointCatalog,
) -> Result<Option<SessionSummary>, AppBrokerError> {
    write_native_message_async(
        writer,
        &ProviderHostMessage::Connected {
            app: app.clone(),
            session: "projection-provider".into(),
        },
    )
    .await?;
    let first = tokio::time::timeout(
        std::time::Duration::from_secs(300),
        read_native_message_async::<_, ProviderMessage>(reader),
    )
    .await
    .map_err(|_| AppBrokerError::Provider("provider offer timed out".into()))??;
    let Some(ProviderMessage::Call {
        id,
        call: ProviderCall::Offer { route, label },
    }) = first
    else {
        return Ok(None);
    };
    if label.is_empty() || label.len() > 1024 {
        return Ok(None);
    }
    // A declared provider offers only what it declared; the owner's provider
    // policy still decides whether it may serve it.
    if let Some(Err(refusal)) = authority.as_ref().map(|authority| authority.offer(&route)) {
        write_native_message_async(
            writer,
            &ProviderHostMessage::Failure {
                message: refusal.to_string(),
            },
        )
        .await?;
        return Ok(None);
    }
    let mut guard = catalog.inner.lock().await;
    let (lease, mut rx) = match catalog.offers.new_lease(&app, &route) {
        Ok(pair) => pair,
        Err(error) => {
            drop(guard);
            write_native_message_async(writer, &ProviderHostMessage::Failure { message: error })
                .await?;
            return Ok(None);
        },
    };
    let factory_lease = lease.clone();
    if guard
        .register_owned(route.as_str(), label, lease.generation, move |ctx| {
            ProviderEndpoint::new(factory_lease.clone(), ctx).map(|v| {
                Box::new(v) as Box<dyn graphshell_host::endpoint_catalog::ResidentEndpoint>
            })
        })
        .is_err()
    {
        catalog.offers.retire(&lease);
        drop(guard);
        write_native_message_async(
            writer,
            &ProviderHostMessage::Failure {
                message: "projection route is already registered".into(),
            },
        )
        .await?;
        return Ok(None);
    }
    let lease_guard = ProviderLeaseGuard {
        catalog: catalog.clone(),
        lease: lease.clone(),
    };
    drop(guard);
    let result = async {
        write_provider_answer(writer, id, ProviderAnswer::Registered {
            max_sessions: lease.permission.max_sessions,
        }, lease.permission.timeout).await?;
        let mut outstanding: Option<crate::door::projection_offers::WorkItem> = None;
        let mut next_message = tokio::select! {
            biased;
            _ = lease.wait_withdrawn() => None,
            incoming = tokio::time::timeout(lease.permission.timeout, read_native_message_async::<_, ProviderMessage>(reader)) => {
                match incoming { Ok(result) => result?, Err(_) => None }
            },
        };
        let mut next_work_id = 1u64;
        loop {
            let Some(ProviderMessage::Call { id, call }) = next_message else {
                break;
            };
            match call {
                ProviderCall::Withdraw => {
                    write_provider_answer(writer, id, ProviderAnswer::Withdrawn, lease.permission.timeout).await?;
                    break;
                }
                ProviderCall::Next { completed } => {
                    match (outstanding.take(), completed) {
                        (None, None) => (),
                        (Some(item), Some(result)) if valid_provider_result(&item.work, &result) => {
                            let _ = item.answer.send(result.result);
                        },
                        _ => break,
                    }
                    let mut read = Box::pin(read_native_message_async::<_, ProviderMessage>(reader));
                    let mut item = tokio::select! {
                        biased;
                        _ = lease.wait_withdrawn() => break,
                        incoming = &mut read => {
                            let _ = incoming?;
                            break;
                        },
                        item = rx.recv() => match item {
                            Some(item) => item,
                            None => break,
                        },
                    };
                    item.work.id = next_work_id;
                    next_work_id = next_work_id.checked_add(1)
                        .ok_or_else(|| AppBrokerError::Provider("provider work IDs exhausted".into()))?;
                    let work = item.work.clone();
                    outstanding = Some(item);
                    let answer = ProviderAnswer::Work { work };
                    write_provider_answer(writer, id, answer, lease.permission.timeout).await?;
                    next_message = tokio::select! {
                        biased;
                        _ = lease.wait_withdrawn() => break,
                        incoming = tokio::time::timeout(lease.permission.timeout, read) => match incoming {
                            Ok(result) => result?,
                            Err(_) => break,
                        },
                    };
                }
                _ => break,
            }
        }
        Ok::<(), AppBrokerError>(())
    }.await;
    drop(lease_guard);
    catalog
        .inner
        .lock()
        .await
        .remove_owned(route.as_str(), lease.generation);
    result?;
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_results_must_match_work_id_session_and_operation() {
        use graphshell_endpoint::door::projection_offer::Work;
        let work = Work {
            id: 7,
            session: "viewer".into(),
            operation: WorkOperation::PollNotice,
        };
        let mut reply = WorkResult {
            id: 7,
            session: "viewer".into(),
            result: WorkOutcome::Notice { notice: None },
        };
        assert!(valid_provider_result(&work, &reply));
        reply.id = 8;
        assert!(!valid_provider_result(&work, &reply));
        reply.id = 7;
        reply.session = "other".into();
        assert!(!valid_provider_result(&work, &reply));
        reply.session = "viewer".into();
        reply.result = WorkOutcome::Closed;
        assert!(!valid_provider_result(&work, &reply));
    }

    #[tokio::test]
    async fn dropping_provider_guard_withdraws_and_removes_its_route() {
        let catalog = AppEndpointCatalog::default();
        let route = AppRouteId::new("independent/docs").unwrap();
        catalog
            .set_provider_permissions(vec![ProviderPermission {
                provider: AppId::new("independent"),
                route: route.clone(),
                viewers: vec![],
                max_sessions: 1,
                timeout: std::time::Duration::from_secs(1),
            }])
            .await
            .unwrap();
        let (lease, _rx) = catalog
            .offers
            .new_lease(&AppId::new("independent"), &route)
            .unwrap();
        catalog
            .update(|inner| {
                inner
                    .register_owned(route.as_str(), "fixture", lease.generation, |_| {
                        Err("fixture".into())
                    })
                    .unwrap()
            })
            .await;
        let guard = ProviderLeaseGuard {
            catalog: catalog.clone(),
            lease: lease.clone(),
        };
        drop(guard);
        assert!(!lease.is_live());
        tokio::time::timeout(std::time::Duration::from_secs(1), async {
            loop {
                if catalog.update(|inner| inner.offers().is_empty()).await {
                    break;
                }
                tokio::task::yield_now().await;
            }
        })
        .await
        .expect("route removal");
    }

    /// The connect schema is this door's, not the browser's. A client that
    /// reached the wrong endpoint is refused at the envelope rather than
    /// admitted into a session it cannot use.
    #[test]
    fn the_app_connect_schema_is_its_own() {
        assert_ne!(APP_CONNECT_SCHEMA, "mere.graphshell/browser-connect/v1");
    }

    /// The wire carries no identity action. This is a shape assertion, not a
    /// policy check: there is no variant to forward, so the door cannot grow
    /// one by accident.
    #[test]
    fn the_application_wire_cannot_carry_an_identity_action() {
        let json = serde_json::to_string(&AppMessage::Connect {
            schema: APP_CONNECT_SCHEMA.to_string(),
            host_nonce: "a".to_string(),
            client_nonce: "b".to_string(),
        })
        .unwrap();
        assert!(json.contains("connect"));
        let identity_action = serde_json::json!({
            "type": "native_identity",
            "request": {"id": 1, "session": "s", "action": {"type": "import_ssh_private"}},
        });
        assert!(
            serde_json::from_value::<AppMessage>(identity_action).is_err(),
            "the application wire must have no identity-action variant",
        );
    }

    /// A hello for an application this device does not admit never reaches
    /// the challenge.
    #[test]
    fn an_unadmitted_application_is_refused_at_the_hello() {
        let hello = AppHello::new(AppId::new("not-ours"));
        let request = hello.accept().unwrap();
        assert!(AllowedAppRoutes::default().admit(&request).is_err());
    }
}
