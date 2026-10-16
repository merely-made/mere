// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Client side of the first-party door.
//!
//! What an application holds while talking to the resident host: the
//! handshake, a request counter, and typed calls for the three things a
//! reader of receipt cards does — open, snapshot, read a resource. The
//! server half is Graphshell's `native::app_broker`; the two share the types
//! in [`super::wire`] and the frames in [`super::frame`] so they cannot drift.
//!
//! The wire is strictly request/response, so the client is deliberately
//! sequential: one call at a time against `&mut self`, no demultiplexer. A
//! first-party app reading cards is human-paced; the day something needs
//! pipelining, that pressure should reshape the wire, not hide in a client.

use chirograph::{
    CapabilityProfile, Carrier, CarrierError, CarrierNotice, CarrierRequest, CarrierRequestBody,
    CarrierResponseBody, ContentHash, PortableCardV1, PresentationCodec, ProjectionRequest,
    ProjectionSnapshot, ProtocolVersion, ResourceAssembly, ResourceChunkRequest,
    ResourceChunkResponse, SessionOpen, SessionOpened,
};
use rand_core::{OsRng, RngCore};

use crate::door::app_admission::{AppHello, AppId, AppRouteId, configured_app_endpoint};
use crate::door::custody::{CustodyAnswer, CustodyCall, CustodyRefusal};
use crate::door::frame::{FrameError, read_native_message_async, write_native_message_async};
use crate::door::local_endpoint::{LocalStream, connect_local};
use crate::door::wire::{APP_CONNECT_SCHEMA, AppHostMessage, AppMessage};

#[derive(Debug, thiserror::Error)]
pub enum AppClientError {
    #[error(transparent)]
    Frame(#[from] FrameError),
    #[error("the resident host closed the stream mid-conversation")]
    Closed,
    #[error("the resident host refused: {0}")]
    Refused(String),
    #[error("expected {expected}, the host answered with {got}")]
    UnexpectedAnswer {
        expected: &'static str,
        got: &'static str,
    },
    #[error("resident response {got} does not answer request {expected}")]
    MismatchedResponse { expected: u64, got: u64 },
    #[error("a card resource was not valid PortableCardV1 JSON: {0}")]
    MalformedCard(#[from] serde_json::Error),
    #[error("the resident resource has another scope or content address")]
    InvalidResourceAddress,
    #[error("the resident resource chunk is invalid: {0}")]
    InvalidResourceChunk(String),
    #[error("transport failed: {0}")]
    Io(#[from] std::io::Error),
}

/// An open session against the resident host, as `app`.
pub struct AppBrokerClient {
    stream: Box<dyn LocalStream>,
    session: String,
    next_id: u64,
}

/// One card the resident host is showing, decoded and labeled.
#[derive(Clone, Debug, PartialEq)]
pub struct DeviceCard {
    pub label: String,
    pub card: PortableCardV1,
}

impl AppBrokerClient {
    /// Connect to the configured endpoint and complete the whole handshake:
    /// hello, challenge, connect, connected.
    pub async fn open(app: AppId) -> Result<Self, AppClientError> {
        Self::open_at(&configured_app_endpoint(), app).await
    }

    /// Connect to the configured endpoint and request one granted resident
    /// route through the version-two hello.
    pub async fn open_route(app: AppId, route: AppRouteId) -> Result<Self, AppClientError> {
        Self::open_route_at(&configured_app_endpoint(), app, route).await
    }

    /// The same, at an explicit endpoint. Tests and receipts use this.
    pub async fn open_at(endpoint: &str, app: AppId) -> Result<Self, AppClientError> {
        Self::open_with_hello(endpoint, AppHello::new(app.clone()), app).await
    }

    /// Route-aware connection at an explicit endpoint.
    pub async fn open_route_at(
        endpoint: &str,
        app: AppId,
        route: AppRouteId,
    ) -> Result<Self, AppClientError> {
        Self::open_with_hello(endpoint, AppHello::for_route(app.clone(), route), app).await
    }

    async fn open_with_hello(
        endpoint: &str,
        hello: AppHello,
        app: AppId,
    ) -> Result<Self, AppClientError> {
        let mut stream = connect_local(endpoint).await?;
        write_native_message_async(&mut stream, &hello).await?;
        let challenge = match read_host(&mut stream).await? {
            AppHostMessage::Challenge { challenge } => challenge,
            other => return Err(unexpected("a challenge", &other)),
        };
        let mut client_nonce = [0u8; 32];
        OsRng.fill_bytes(&mut client_nonce);
        write_native_message_async(
            &mut stream,
            &AppMessage::Connect {
                schema: APP_CONNECT_SCHEMA.to_string(),
                host_nonce: challenge.host_nonce,
                client_nonce: base64::Engine::encode(
                    &base64::engine::general_purpose::URL_SAFE_NO_PAD,
                    client_nonce,
                ),
            },
        )
        .await?;
        let session = match read_host(&mut stream).await? {
            AppHostMessage::Connected {
                app: connected_app,
                session,
            } => {
                if connected_app != app {
                    return Err(AppClientError::Refused(format!(
                        "resident host connected {connected_app}, expected {app}"
                    )));
                }
                session
            },
            other => return Err(unexpected("connected", &other)),
        };
        Ok(Self {
            stream,
            session,
            next_id: 1,
        })
    }

    /// The session id the host admitted.
    pub fn session(&self) -> &str {
        &self.session
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
                card: serde_json::from_slice(&bytes)?,
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
        let id = self.next_id;
        self.next_id += 1;
        write_native_message_async(
            &mut self.stream,
            &AppMessage::Request {
                request: CarrierRequest { id, body },
            },
        )
        .await?;
        match read_host(&mut self.stream).await? {
            AppHostMessage::Response { response } if response.id == id => response
                .body
                .map_err(|error| AppClientError::Refused(error.message)),
            AppHostMessage::Response { response } => Err(AppClientError::MismatchedResponse {
                expected: id,
                got: response.id,
            }),
            other => Err(unexpected("a carrier response", &other)),
        }
    }

    /// One call on the custody route. Only a client opened on
    /// [`crate::door::custody::CUSTODY_ROUTE`] is answered; see
    /// [`crate::door::custody_client`] for the typed calls.
    pub async fn custody_call(
        &mut self,
        call: CustodyCall,
    ) -> Result<Result<CustodyAnswer, CustodyRefusal>, AppClientError> {
        let id = self.next_id;
        self.next_id += 1;
        write_native_message_async(&mut self.stream, &AppMessage::Call { id, call }).await?;
        match read_host(&mut self.stream).await? {
            AppHostMessage::Answer {
                id: answered,
                answer,
            } if answered == id => Ok(answer),
            AppHostMessage::Answer { id: answered, .. } => {
                Err(AppClientError::MismatchedResponse {
                    expected: id,
                    got: answered,
                })
            },
            AppHostMessage::Failure { message } => Err(AppClientError::Refused(message)),
            other => Err(unexpected("a custody answer", &other)),
        }
    }

    /// Take one already-received resident notice.
    pub async fn take_notice(&mut self) -> Result<Option<CarrierNotice>, AppClientError> {
        write_native_message_async(&mut self.stream, &AppMessage::TakeNotice).await?;
        match read_host(&mut self.stream).await? {
            AppHostMessage::Notice { notice } => Ok(notice),
            other => Err(unexpected("a carrier notice", &other)),
        }
    }

    /// Wait until the resident endpoint reports a new revision.
    pub async fn wait_for_notice(&mut self) -> Result<CarrierNotice, AppClientError> {
        write_native_message_async(&mut self.stream, &AppMessage::WaitNotice).await?;
        match read_host(&mut self.stream).await? {
            AppHostMessage::Notice {
                notice: Some(notice),
            } => Ok(notice),
            AppHostMessage::Notice { notice: None } => Err(AppClientError::Closed),
            other => Err(unexpected("a carrier notice", &other)),
        }
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
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(AppClientError::Io)?;
        let client = runtime.block_on(AppBrokerClient::open_route_at(endpoint, app, route))?;
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

async fn read_host(stream: &mut Box<dyn LocalStream>) -> Result<AppHostMessage, AppClientError> {
    read_native_message_async::<_, AppHostMessage>(stream)
        .await?
        .ok_or(AppClientError::Closed)
}

fn unexpected(expected: &'static str, got: &AppHostMessage) -> AppClientError {
    let got = match got {
        AppHostMessage::Challenge { .. } => "a challenge",
        AppHostMessage::Connected { .. } => "connected",
        AppHostMessage::Response { .. } => "a carrier response",
        AppHostMessage::Notice { .. } => "a carrier notice",
        AppHostMessage::Answer { .. } => "a custody answer",
        AppHostMessage::Failure { .. } => "a failure",
    };
    AppClientError::UnexpectedAnswer { expected, got }
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
        return Err(AppClientError::InvalidResourceAddress);
    }
    if response.total_len > MAX_APP_RESOURCE_BYTES
        || total.is_some_and(|size| size != response.total_len)
    {
        return Err(AppClientError::InvalidResourceChunk(
            "inconsistent or excessive total length".into(),
        ));
    }
    let bytes = response
        .decode()
        .ok_or_else(|| AppClientError::InvalidResourceChunk("chunk address or encoding".into()))?;
    let end = response
        .offset
        .checked_add(bytes.len() as u64)
        .ok_or_else(|| AppClientError::InvalidResourceChunk("chunk length overflow".into()))?;
    if bytes.len() > chirograph::MAX_RESOURCE_CHUNK_BYTES as usize
        || end > response.total_len
        || (bytes.is_empty() && response.offset < response.total_len)
    {
        return Err(AppClientError::InvalidResourceChunk(
            "chunk exceeds its bounds or makes no progress".into(),
        ));
    }
    *total = Some(response.total_len);
    assembly
        .accept(response)
        .map_err(|e| AppClientError::InvalidResourceChunk(e.to_string()))
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
            let framed = AppHostMessage::Response { response };
            let mut wire = Vec::new();
            crate::door::frame::write_native_message(&mut wire, &framed).unwrap();
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
