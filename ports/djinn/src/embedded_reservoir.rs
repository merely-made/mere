// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Client-first reservoir attachment (reservoir plan V5).
//!
//! A product always uses the admitted application door. Only an absent door
//! permits it to embed the same [`ResidentReservoir`] and [`MereRoutes`] that
//! Djinn composes. A live door's refusal never opens a second store. The
//! identity factory is lazy, so the ordinary client path does not even open
//! local identity custody. Hosts supply their selected persona and domain
//! registration; this module owns no product semantics or persistence format.
//! Embedded operation uses a multi-thread Tokio runtime, as Djinn's existing
//! synchronous endpoint adapters require `block_in_place` for store calls.

use std::future::Future;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use castellan::custody::IdentityStorage;
use chirograph::{CarrierRequestBody, CarrierResponseBody, IntentInvocation, IntentResult};
use graphshell::native::app_admission::{AllowedAppRoutes, AppId, AppRouteGrants, AppRouteId};
use graphshell::native::app_broker::{AppEndpointCatalog, serve_app_broker};
use graphshell::native::app_client::{AppBrokerClient, AppClientError};
use graphshell::native::endpoint_catalog::ResidentEndpointCatalog;
use graphshell::native::personae_host::PersonaeHost;
use graphshell::native::tasks::ResidentTasks;
use pandect::DomainId;
use personae::PersonaId;
use sceno::InstanceId;
use tokio::sync::oneshot;

use crate::resident_mere::{MereRoutes, mere_route_id};
use crate::resident_reservoir::{
    EnsureMereV1, RESERVOIR_ENSURE_MERE_INTENT, RESIDENT_RESERVOIR_ROUTE, ResidentReservoir,
};

const ATTACH_TIMEOUT: Duration = Duration::from_secs(5);

/// Explicit identity, endpoint and storage selection for one application.
///
/// `applications` names the owner's admitted first-party clients; it must
/// include `application`. It is not a list supplied by an untrusted client.
#[derive(Clone)]
pub struct ReservoirAttachmentOptions {
    pub endpoint: String,
    pub application: AppId,
    pub applications: Vec<AppId>,
    pub shared_root: PathBuf,
    pub persona: PersonaId,
    pub domain: DomainId,
    /// Lease length selected by the owner for an embedded application door.
    pub session_duration: Duration,
}

/// Why attachment or embedded ownership could not complete.
#[derive(Debug, thiserror::Error)]
pub enum ReservoirAttachmentError {
    #[error("resident attachment failed: {0}")]
    Client(#[from] AppClientError),
    #[error("resident attachment timed out")]
    Timeout,
    #[error("resident reservoir belongs to persona {found}, expected {expected}")]
    Persona { expected: String, found: String },
    #[error("resident reservoir did not disclose its persona")]
    MissingPersona,
    #[error("embedded reservoir refused: {0}")]
    Embedded(String),
    #[error("resident refused to ensure the mere: {0}")]
    Ensure(String),
}

/// An admitted mere client, retaining an embedded owner only when needed.
///
/// Close this after all other clients of this embedded owner have detached.
/// Dropping it requests the same shutdown; [`Self::close`] additionally waits
/// for task cancellation and store release.
pub struct ReservoirAttachment {
    client: Option<AppBrokerClient>,
    embedded: Option<EmbeddedOwner>,
}

impl ReservoirAttachment {
    /// Attach to Djinn, or compose its reservoir lane if the endpoint is absent.
    pub async fn open<S, F, Fut>(
        options: ReservoirAttachmentOptions,
        identity: F,
    ) -> Result<Self, ReservoirAttachmentError>
    where
        S: IdentityStorage + 'static,
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<Arc<PersonaeHost<S>>, String>>,
    {
        Self::open_with(options, identity, |routes| async { Ok(routes) }).await
    }

    /// As [`Self::open`], with the owner's domain registration hook.
    ///
    /// The hook runs only in embedded mode, before any route is served. The
    /// same registrations must also be installed in the daemon composition.
    /// It receives the reusable routes, never a client-owned graph or store.
    pub async fn open_with<S, F, Fut, C, CFut>(
        options: ReservoirAttachmentOptions,
        identity: F,
        configure: C,
    ) -> Result<Self, ReservoirAttachmentError>
    where
        S: IdentityStorage + 'static,
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<Arc<PersonaeHost<S>>, String>>,
        C: FnOnce(MereRoutes) -> CFut,
        CFut: Future<Output = Result<MereRoutes, String>>,
    {
        let reservoir_route = AppRouteId::new(RESIDENT_RESERVOIR_ROUTE).expect("valid route");
        let initial = connect(&options, reservoir_route.clone()).await;
        let (mut reservoir, embedded) = match initial {
            Ok(client) => (client, None),
            Err(ReservoirAttachmentError::Client(AppClientError::Io(error)))
                if matches!(
                    error.kind(),
                    ErrorKind::NotFound | ErrorKind::ConnectionRefused
                ) =>
            {
                if !matches!(
                    tokio::runtime::Handle::current().runtime_flavor(),
                    tokio::runtime::RuntimeFlavor::MultiThread
                ) {
                    return Err(ReservoirAttachmentError::Embedded(
                        "embedded resident requires a multi-thread Tokio runtime".into(),
                    ));
                }
                let host = identity()
                    .await
                    .map_err(ReservoirAttachmentError::Embedded)?;
                if host.is_locked() {
                    return Err(ReservoirAttachmentError::Embedded(
                        "selected identity is locked".into(),
                    ));
                }
                let owner = EmbeddedOwner::start(&options, host, configure).await?;
                let client = match connect(&options, reservoir_route).await {
                    Ok(client) => client,
                    Err(error) => {
                        owner.close().await?;
                        return Err(error);
                    },
                };
                (client, Some(owner))
            },
            Err(error) => return Err(error),
        };
        let attach = tokio::time::timeout(ATTACH_TIMEOUT, async {
            check_persona(&mut reservoir, options.persona).await?;
            ensure(&mut reservoir, &options.domain).await?;
            let route = AppRouteId::new(mere_route_id(options.domain.as_str()))
                .map_err(|error| ReservoirAttachmentError::Embedded(error.to_string()))?;
            connect(&options, route).await
        })
        .await
        .unwrap_or(Err(ReservoirAttachmentError::Timeout));
        let _ = tokio::time::timeout(ATTACH_TIMEOUT, reservoir.close()).await;
        match attach {
            Ok(client) => Ok(Self {
                client: Some(client),
                embedded,
            }),
            Err(error) => {
                if let Some(owner) = embedded {
                    owner.close().await?;
                }
                Err(error)
            },
        }
    }

    /// Whether this process owns the embedded resident lane.
    pub fn is_embedded(&self) -> bool {
        self.embedded.is_some()
    }

    pub fn client_mut(&mut self) -> &mut AppBrokerClient {
        self.client.as_mut().expect("open attachment has a client")
    }

    /// Detach, stop embedded connections and await release of every store.
    pub async fn close(mut self) -> Result<(), ReservoirAttachmentError> {
        let client_result = tokio::time::timeout(
            ATTACH_TIMEOUT,
            self.client.take().expect("open client").close(),
        )
        .await;
        if let Some(owner) = self.embedded.take() {
            owner.close().await?;
        }
        client_result
            .map_err(|_| ReservoirAttachmentError::Timeout)?
            .map_err(Into::into)
    }
}

async fn connect(
    options: &ReservoirAttachmentOptions,
    route: AppRouteId,
) -> Result<AppBrokerClient, ReservoirAttachmentError> {
    tokio::time::timeout(
        ATTACH_TIMEOUT,
        AppBrokerClient::open_route_at(&options.endpoint, options.application.clone(), route),
    )
    .await
    .map_err(|_| ReservoirAttachmentError::Timeout)?
    .map_err(Into::into)
}

async fn check_persona(
    client: &mut AppBrokerClient,
    expected: PersonaId,
) -> Result<(), ReservoirAttachmentError> {
    let found = client
        .read_cards()
        .await?
        .into_iter()
        .flat_map(|card| card.card.values)
        .find(|value| value.label == "Persona")
        .map(|value| value.value)
        .ok_or(ReservoirAttachmentError::MissingPersona)?;
    let expected = expected.as_uuid().to_string();
    if found != expected {
        return Err(ReservoirAttachmentError::Persona { expected, found });
    }
    Ok(())
}

async fn ensure(
    client: &mut AppBrokerClient,
    domain: &DomainId,
) -> Result<(), ReservoirAttachmentError> {
    let opened = client.open_session().await?;
    let request = opened
        .descriptor
        .projections
        .first()
        .ok_or_else(|| ReservoirAttachmentError::Ensure("no reservoir projection".into()))?
        .request
        .clone();
    let snapshot = client.snapshot(request).await?;
    let response = client
        .request_body(CarrierRequestBody::Intent(IntentInvocation {
            session: snapshot.session,
            target: InstanceId(0),
            observed_epoch: snapshot.scene.epoch,
            observed_revision: snapshot.scene.revision,
            intent: RESERVOIR_ENSURE_MERE_INTENT.into(),
            payload: serde_json::to_vec(&EnsureMereV1::new(domain.as_str()))
                .expect("ensure payload serializes"),
        }))
        .await?;
    match response {
        CarrierResponseBody::Intent(IntentResult::Accepted) => Ok(()),
        other => Err(ReservoirAttachmentError::Ensure(format!("{other:?}"))),
    }
}

struct EmbeddedOwner {
    stop: Option<oneshot::Sender<()>>,
    task: Option<tokio::task::JoinHandle<Result<(), String>>>,
}

impl EmbeddedOwner {
    async fn start<S, C, CFut>(
        options: &ReservoirAttachmentOptions,
        host: Arc<PersonaeHost<S>>,
        configure: C,
    ) -> Result<Self, ReservoirAttachmentError>
    where
        S: IdentityStorage + 'static,
        C: FnOnce(MereRoutes) -> CFut,
        CFut: Future<Output = Result<MereRoutes, String>>,
    {
        if !options.applications.contains(&options.application) {
            return Err(ReservoirAttachmentError::Embedded(
                "application is not in the owner's admitted application set".into(),
            ));
        }
        let session_duration_ms = u64::try_from(options.session_duration.as_millis())
            .ok()
            .filter(|duration| *duration > 0)
            .ok_or_else(|| {
                ReservoirAttachmentError::Embedded("invalid session lease duration".into())
            })?;
        let reservoir = ResidentReservoir::open(&options.shared_root, Some(options.persona))
            .await
            .map_err(ReservoirAttachmentError::Embedded)?;
        let catalog = AppEndpointCatalog::new(ResidentEndpointCatalog::new());
        let grants = AppRouteGrants::new(AllowedAppRoutes::new(
            options
                .applications
                .iter()
                .cloned()
                .map(|app| (app, ResidentReservoir::route())),
        ));
        let routes = MereRoutes::new(
            options.shared_root.clone(),
            options.persona,
            catalog.clone(),
            grants.clone(),
            options.applications.clone(),
        );
        let routes = configure(routes)
            .await
            .map_err(ReservoirAttachmentError::Embedded)?;
        routes
            .load_access(&reservoir)
            .await
            .map_err(ReservoirAttachmentError::Embedded)?;
        for mere in reservoir.meres().await {
            routes
                .serve(&mere)
                .await
                .map_err(ReservoirAttachmentError::Embedded)?;
        }
        catalog
            .update(|catalog| reservoir.register(catalog, Some(routes)))
            .await
            .map_err(|error| ReservoirAttachmentError::Embedded(error.to_string()))?;
        let (stop, mut stopped) = oneshot::channel();
        let (ready, mut started) = oneshot::channel();
        let endpoint = options.endpoint.clone();
        let task = tokio::spawn(async move {
            let tasks = ResidentTasks::new();
            let mut lock = host.lock_state();
            // The readiness probe happens in the caller after the task starts.
            let _ = ready.send(());
            let result = tasks.scope(async {
                if host.is_locked() {
                    return Err("selected identity locked during embedded startup".into());
                }
                tokio::select! {
                    result = serve_app_broker(&endpoint, host, grants, session_duration_ms, None, catalog.clone()) => {
                        result.map_err(|error| error.to_string())
                    },
                    _ = &mut stopped => Ok(()),
                    _ = lock.changed() => Err("selected identity changed lock state; embedded reservoir stopped".into()),
                }
            }).await;
            tasks.cancel_and_join().await;
            // Catalog factories retain the route map which retains the catalog.
            // Break that ownership cycle before releasing the reservoir lock.
            catalog
                .update(|catalog| *catalog = ResidentEndpointCatalog::new())
                .await;
            drop(reservoir);
            result
        });
        let owner = Self {
            stop: Some(stop),
            task: Some(task),
        };
        let _ = (&mut started).await;
        // Listener binding is asynchronous. Only the owner startup polls;
        // ordinary client attachment never converts another refusal to absence.
        for _ in 0..100 {
            if owner.task.as_ref().expect("owner task").is_finished() {
                owner.close().await?;
                return Err(ReservoirAttachmentError::Embedded(
                    "resident stopped before listening".into(),
                ));
            }
            if graphshell::native::local_endpoint::connect_local(&options.endpoint)
                .await
                .is_ok()
            {
                return Ok(owner);
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        owner.close().await?;
        Err(ReservoirAttachmentError::Embedded(
            "resident did not start listening".into(),
        ))
    }

    async fn close(mut self) -> Result<(), ReservoirAttachmentError> {
        if let Some(stop) = self.stop.take() {
            let _ = stop.send(());
        }
        self.task
            .take()
            .expect("owner task")
            .await
            .map_err(|error| ReservoirAttachmentError::Embedded(error.to_string()))?
            .map_err(ReservoirAttachmentError::Embedded)
    }
}

impl Drop for EmbeddedOwner {
    fn drop(&mut self) {
        if let Some(stop) = self.stop.take() {
            let _ = stop.send(());
        }
    }
}
