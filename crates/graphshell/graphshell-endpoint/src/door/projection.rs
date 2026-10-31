// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! Projection client and carrier over the application connection.
use chirograph::{
    CapabilityProfile, Carrier, CarrierError, CarrierNotice, CarrierRequest, CarrierRequestBody,
    CarrierResponseBody, ContentHash, PortableCardV1, PresentationCodec, ProjectionRequest,
    ProjectionSnapshot, ProtocolVersion, ResourceAssembly, ResourceChunkRequest,
    ResourceChunkResponse, SessionOpen, SessionOpened,
};
use djinn::door::app_admission::{AppId, AppRouteId, configured_app_endpoint};
use djinn::door::app_client::{AppClientError, AppConnection};
use djinn::door::declaration::AppDeclaration;

pub struct AppBrokerClient {
    inner: AppConnection,
}
#[derive(Clone, Debug, PartialEq)]
pub struct DeviceCard {
    pub label: String,
    pub card: PortableCardV1,
}
impl AppBrokerClient {
    pub async fn open(app: AppId) -> Result<Self, AppClientError> {
        Ok(Self {
            inner: AppConnection::open(app).await?,
        })
    }
    pub async fn open_at(endpoint: &str, app: AppId) -> Result<Self, AppClientError> {
        Ok(Self {
            inner: AppConnection::open_at(endpoint, app).await?,
        })
    }
    pub async fn open_route(app: AppId, route: AppRouteId) -> Result<Self, AppClientError> {
        Ok(Self {
            inner: AppConnection::open_route(app, route).await?,
        })
    }
    pub async fn open_route_at(
        endpoint: &str,
        app: AppId,
        route: AppRouteId,
    ) -> Result<Self, AppClientError> {
        Ok(Self {
            inner: AppConnection::open_route_at(endpoint, app, route).await?,
        })
    }
    /// Open `route` carrying the application's declaration.
    pub async fn open_declared_route(
        app: AppId,
        route: AppRouteId,
        declaration: AppDeclaration,
    ) -> Result<Self, AppClientError> {
        Self::open_declared_route_at(&configured_app_endpoint(), app, route, declaration).await
    }
    pub async fn open_declared_route_at(
        endpoint: &str,
        app: AppId,
        route: AppRouteId,
        declaration: AppDeclaration,
    ) -> Result<Self, AppClientError> {
        Ok(Self {
            inner: AppConnection::open_declared_route_at(endpoint, app, route, declaration).await?,
        })
    }
    pub fn session(&self) -> &str {
        self.inner.session()
    }
    /// Open the endpoint session and learn what it projects.
    pub async fn open_session(&mut self) -> Result<SessionOpened, AppClientError> {
        let body = self
            .request_body(CarrierRequestBody::Open(Box::new(SessionOpen {
                version: ProtocolVersion { major: 1, minor: 0 },
                capabilities: CapabilityProfile::default(),
            })))
            .await?;
        match body {
            CarrierResponseBody::Opened(opened) => Ok(*opened),
            other => Err(unexpected_body("opened", &other)),
        }
    }

    /// One projection snapshot.
    pub async fn snapshot(
        &mut self,
        request: ProjectionRequest,
    ) -> Result<ProjectionSnapshot, AppClientError> {
        let body = self
            .request_body(CarrierRequestBody::Snapshot(request))
            .await?;
        match body {
            CarrierResponseBody::Snapshot(snapshot) => Ok(*snapshot),
            other => Err(unexpected_body("a snapshot", &other)),
        }
    }

    /// The bytes behind one content hash — a card, or a capture it names.
    pub async fn resource(
        &mut self,
        session: chirograph::ProjectionSession,
        resource: ContentHash,
    ) -> Result<Vec<u8>, AppClientError> {
        let mut assembly = ResourceAssembly::new();
        let mut total = None;
        loop {
            let body = self
                .request_body(CarrierRequestBody::ResourceChunk(ResourceChunkRequest {
                    session: session.clone(),
                    resource,
                    offset: assembly.received(),
                    length: chirograph::MAX_RESOURCE_CHUNK_BYTES,
                }))
                .await?;
            let CarrierResponseBody::ResourceChunk(chunk) = body else {
                return Err(unexpected_body("a resource chunk", &body));
            };
            if let Some(bytes) =
                accept_resource_chunk(&mut assembly, &chunk, &session, resource, &mut total)?
            {
                return Ok(bytes);
            }
        }
    }

    /// Open, snapshot, and decode every portable card on offer.
    ///
    /// The one call a card reader needs. Captures are *not* fetched here —
    /// each card's `media` names them by hash, and the caller reads the ones
    /// it will actually show through [`Self::resource`], which is the whole
    /// point of the store read-through.
    pub async fn read_cards(&mut self) -> Result<Vec<DeviceCard>, AppClientError> {
        let opened = self.open_session().await?;
        let Some(projection) = opened.descriptor.projections.first() else {
            return Ok(Vec::new());
        };
        let snapshot = self.snapshot(projection.request.clone()).await?;
        let mut cards = Vec::new();
        for offer in snapshot.presentation.offers.values().flatten() {
            if offer.codec != PresentationCodec::PortableCardV1 {
                continue;
            }
            let bytes = self
                .resource(snapshot.session.clone(), offer.resource)
                .await?;
            cards.push(DeviceCard {
                label: offer.semantics.label.clone(),
                card: serde_json::from_slice(&bytes).map_err(|_| {
                    AppClientError::InvalidResponse(
                        "a card resource was not valid PortableCardV1 JSON".into(),
                    )
                })?,
            });
        }
        Ok(cards)
    }

    /// Close the session. Consumes the client; the stream is done either way.
    pub async fn close(mut self) -> Result<(), AppClientError> {
        let _ = self.request_body(CarrierRequestBody::Close).await?;
        Ok(())
    }

    pub async fn request_body(
        &mut self,
        body: CarrierRequestBody,
    ) -> Result<CarrierResponseBody, AppClientError> {
        let id = self.inner.next_id();
        match self
            .inner
            .request::<_, chirograph::CarrierResponse, CarrierNotice>(CarrierRequest { id, body })
            .await?
        {
            response if response.id == id => response
                .body
                .map_err(|error| AppClientError::Refused(error.message)),
            response => Err(AppClientError::MismatchedResponse {
                expected: id,
                got: response.id,
            }),
        }
    }

    /// Take one already-received resident notice.
    pub async fn take_notice(&mut self) -> Result<Option<CarrierNotice>, AppClientError> {
        self.inner.take_notice().await
    }

    /// Wait until the resident endpoint reports a new revision.
    pub async fn wait_for_notice(&mut self) -> Result<CarrierNotice, AppClientError> {
        self.inner.wait_for_notice().await
    }
}

/// Blocking carrier over the owner-only first-party application door.
///
/// A desktop application uses this exactly like an in-process or stdio
/// carrier, while the resident process remains the only owner of persona
/// files and network hosts.
pub struct AppRouteCarrier {
    runtime: tokio::runtime::Runtime,
    client: Option<AppBrokerClient>,
}

impl AppRouteCarrier {
    pub fn open(app: AppId, route: AppRouteId) -> Result<Self, AppClientError> {
        Self::open_at(&configured_app_endpoint(), app, route)
    }

    pub fn open_at(endpoint: &str, app: AppId, route: AppRouteId) -> Result<Self, AppClientError> {
        Self::open_at_with(endpoint, app, route, None)
    }

    /// Open `route` carrying the application's declaration.
    pub fn open_declared(
        app: AppId,
        route: AppRouteId,
        declaration: AppDeclaration,
    ) -> Result<Self, AppClientError> {
        Self::open_at_with(&configured_app_endpoint(), app, route, Some(declaration))
    }

    fn open_at_with(
        endpoint: &str,
        app: AppId,
        route: AppRouteId,
        declaration: Option<AppDeclaration>,
    ) -> Result<Self, AppClientError> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(AppClientError::Io)?;
        let client = runtime.block_on(async {
            match declaration {
                Some(declaration) => {
                    AppBrokerClient::open_declared_route_at(endpoint, app, route, declaration).await
                },
                None => AppBrokerClient::open_route_at(endpoint, app, route).await,
            }
        })?;
        Ok(Self {
            runtime,
            client: Some(client),
        })
    }
}

impl Carrier for AppRouteCarrier {
    fn request(&mut self, body: CarrierRequestBody) -> Result<CarrierResponseBody, CarrierError> {
        let Self { runtime, client } = self;
        let client = client.as_mut().ok_or_else(|| {
            CarrierError::Disconnected("resident application route is closed".into())
        })?;
        runtime
            .block_on(client.request_body(body))
            .map_err(carrier_error)
    }

    fn take_notice(&mut self) -> Option<CarrierNotice> {
        let result = {
            let client = self.client.as_mut()?;
            self.runtime.block_on(client.take_notice())
        };
        match result {
            Ok(notice) => notice,
            Err(_) => {
                self.client = None;
                None
            },
        }
    }

    fn wait_for_notice(&mut self) -> Result<CarrierNotice, CarrierError> {
        let Self { runtime, client } = self;
        let client = client.as_mut().ok_or_else(|| {
            CarrierError::Disconnected("resident application route is closed".into())
        })?;
        runtime
            .block_on(client.wait_for_notice())
            .map_err(carrier_error)
    }

    fn shutdown(&mut self) -> Result<(), CarrierError> {
        self.client.take();
        Ok(())
    }
}

fn carrier_error(error: AppClientError) -> CarrierError {
    match error {
        AppClientError::Refused(message) => CarrierError::Refused(message),
        other => CarrierError::Disconnected(other.to_string()),
    }
}

fn unexpected_body(expected: &'static str, got: &CarrierResponseBody) -> AppClientError {
    let got = match got {
        CarrierResponseBody::Descriptor(_) => "a descriptor",
        CarrierResponseBody::Snapshot(_) => "a snapshot",
        CarrierResponseBody::Resource(_) => "a resource",
        CarrierResponseBody::ResourceChunk(_) => "a resource chunk",
        CarrierResponseBody::Resume(_) => "a resume reply",
        CarrierResponseBody::Intent(_) => "an intent result",
        CarrierResponseBody::Opened(_) => "opened",
        _ => "another response",
    };
    AppClientError::UnexpectedAnswer { expected, got }
}

/// A native app shares the carrier's chunk protocol. Check the requested
/// scope and hash before assembly, consistent lengths and bounded progress on
/// every response, then the shared assembler verifies each chunk and the whole.
const MAX_APP_RESOURCE_BYTES: u64 = 64 * 1024 * 1024;
fn accept_resource_chunk(
    assembly: &mut ResourceAssembly,
    response: &ResourceChunkResponse,
    session: &chirograph::ProjectionSession,
    resource: ContentHash,
    total: &mut Option<u64>,
) -> Result<Option<Vec<u8>>, AppClientError> {
    if &response.session != session || response.resource != resource {
        return Err(AppClientError::InvalidResponse(
            "resource scope or content address mismatch".into(),
        ));
    }
    if response.total_len > MAX_APP_RESOURCE_BYTES
        || total.is_some_and(|size| size != response.total_len)
    {
        return Err(AppClientError::InvalidResponse(
            "inconsistent or excessive total length".into(),
        ));
    }
    let bytes = response
        .decode()
        .ok_or_else(|| AppClientError::InvalidResponse("chunk address or encoding".into()))?;
    let end = response
        .offset
        .checked_add(bytes.len() as u64)
        .ok_or_else(|| AppClientError::InvalidResponse("chunk length overflow".into()))?;
    if bytes.len() > chirograph::MAX_RESOURCE_CHUNK_BYTES as usize
        || end > response.total_len
        || (bytes.is_empty() && response.offset < response.total_len)
    {
        return Err(AppClientError::InvalidResponse(
            "chunk exceeds its bounds or makes no progress".into(),
        ));
    }
    *total = Some(response.total_len);
    assembly
        .accept(response)
        .map_err(|e| AppClientError::InvalidResponse(e.to_string()))
}
#[cfg(test)]
mod resource_boundary_tests {
    use super::*;
    #[test]
    fn native_resource_chunks_verify_requested_scope_and_complete_content_above_one_frame() {
        let session = chirograph::ProjectionSession("mere.graph".into());
        let bytes = vec![0xa5; 1_200_007];
        let whole = chirograph::ResourceResponse::new(session.clone(), bytes.clone());
        let mut assembly = ResourceAssembly::new();
        let mut total = None;
        let mut frames = 0;
        loop {
            let chunk = ResourceChunkResponse::slice(
                &whole,
                assembly.received(),
                chirograph::MAX_RESOURCE_CHUNK_BYTES,
            );
            let response = chirograph::CarrierResponse {
                id: 1,
                body: Ok(CarrierResponseBody::ResourceChunk(Box::new(chunk.clone()))),
            };
            let framed = djinn::door::wire::AppHostMessage::<_, (), ()>::Response { response };
            let mut wire = Vec::new();
            djinn::door::frame::write_native_message(&mut wire, &framed).unwrap();
            assert!(wire.len() < 1024 * 1024);
            frames += 1;
            if let Some(actual) =
                accept_resource_chunk(&mut assembly, &chunk, &session, whole.resource, &mut total)
                    .unwrap()
            {
                assert_eq!(actual, bytes);
                break;
            }
        }
        assert_eq!(frames, 3);
    }
    #[test]
    fn native_resource_chunks_refuse_scope_address_order_length_and_no_progress_faults() {
        let session = chirograph::ProjectionSession("mere.graph".into());
        let whole = chirograph::ResourceResponse::new(session.clone(), vec![7; 16]);
        for fault in 0..8 {
            let mut chunk = ResourceChunkResponse::slice(&whole, 0, 8);
            let mut total = None;
            match fault {
                0 => chunk.session = chirograph::ProjectionSession("other".into()),
                1 => chunk.resource = ContentHash::of(b"other"),
                2 => chunk.bytes.push('!'),
                3 => chunk.offset = 1,
                4 => chunk.total_len = 4,
                5 => chunk.total_len = MAX_APP_RESOURCE_BYTES + 1,
                6 => total = Some(15),
                _ => {
                    chunk.bytes = String::new();
                    chunk.chunk = ContentHash::of(b"");
                },
            }
            assert!(
                accept_resource_chunk(
                    &mut ResourceAssembly::new(),
                    &chunk,
                    &session,
                    whole.resource,
                    &mut total
                )
                .is_err(),
                "accepted fault {fault}"
            );
        }
        let empty = chirograph::ResourceResponse::new(session.clone(), Vec::new());
        assert_eq!(
            accept_resource_chunk(
                &mut ResourceAssembly::new(),
                &ResourceChunkResponse::slice(&empty, 0, 8),
                &session,
                empty.resource,
                &mut None
            )
            .unwrap(),
            Some(Vec::new())
        );
    }
}
