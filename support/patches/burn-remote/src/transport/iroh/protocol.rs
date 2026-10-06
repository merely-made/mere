//! Iroh protocol handler for Burn Remote compute and tensor-transfer streams.

use std::{fmt, sync::Arc};

use burn_ir::BackendIr;
use iroh::{
    EndpointId,
    endpoint::{Connection, RecvStream, SendStream},
    protocol::{AcceptError, DynProtocolHandler, ProtocolHandler},
};
use tokio_util::sync::CancellationToken;

use crate::{
    Credential, PeerId,
    server::{
        AuthorizationRequest, ClientId, PeerAuthorizer, SessionSetup, pump::drive_session,
        session::SessionManager, spawn::spawn_detached,
    },
};

use super::{
    IrohTransfer,
    node::{RemoteNode, StreamKind},
};

/// Serves Burn Remote's compute and tensor-transfer streams on an Iroh endpoint.
pub struct IrohRemoteProtocol<B: BackendIr> {
    node: RemoteNode,
    sessions: Arc<SessionManager<B, IrohTransfer<B>>>,
    transfer: Arc<IrohTransfer<B>>,
    authorizer: Arc<dyn PeerAuthorizer>,
    shutdown: CancellationToken,
    lifetime: Arc<ProtocolLifetime>,
}

// Only protocol owners retain this guard; worker token clones cannot keep it alive.
struct ProtocolLifetime(CancellationToken);
impl Drop for ProtocolLifetime {
    fn drop(&mut self) {
        self.0.cancel();
    }
}
impl<B: BackendIr> Clone for IrohRemoteProtocol<B> {
    fn clone(&self) -> Self {
        Self {
            node: self.node.clone(),
            sessions: self.sessions.clone(),
            transfer: self.transfer.clone(),
            authorizer: self.authorizer.clone(),
            shutdown: self.shutdown.clone(),
            lifetime: self.lifetime.clone(),
        }
    }
}

impl<B: BackendIr> fmt::Debug for IrohRemoteProtocol<B> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IrohRemoteProtocol")
            .field("endpoint_id", &self.node.id())
            .finish_non_exhaustive()
    }
}

impl<B: BackendIr> IrohRemoteProtocol<B> {
    pub(crate) fn new(node: RemoteNode, setup: SessionSetup<B>) -> Self {
        let transfer = Arc::new(IrohTransfer::new(node.clone()));
        Self {
            sessions: Arc::new(setup.manager(transfer.clone())),
            node,
            transfer,
            authorizer: setup.authorizer,
            lifetime: Arc::new(ProtocolLifetime(setup.shutdown.clone())),
            shutdown: setup.shutdown,
        }
    }

    /// Mount on an application's endpoint with inspectable, individually reclaimable sessions.
    pub fn from_endpoint(
        endpoint: iroh::Endpoint,
        devices: Vec<burn_backend::tensor::Device<B>>,
        authorizer: Arc<dyn PeerAuthorizer>,
        telemetry: crate::telemetry::TelemetryProbe,
        custom_ops: burn_router::CustomOpRegistry<B>,
    ) -> Result<Self, crate::server::ServeError> {
        let settings = crate::server::ServerSettings::default()
            .with_authorizer(move |request: AuthorizationRequest<'_>| authorizer.authorize(request))
            .with_telemetry(telemetry)
            .with_custom_ops(custom_ops);
        let setup = settings.sessions_of::<B>(devices, CancellationToken::new())?;
        Ok(Self::new(RemoteNode::from_endpoint(endpoint), setup))
    }

    /// Snapshot reserved and active sessions.
    pub async fn sessions(&self) -> Vec<crate::server::ServedSession> {
        self.sessions.sessions().await
    }

    /// Close one session, acknowledging only completed backend teardown.
    pub async fn close_session(&self, id: crate::shared::SessionId) -> Result<bool, String> {
        self.sessions.close_session(id).await
    }

    async fn handle_session(
        sessions: Arc<SessionManager<B, IrohTransfer<B>>>,
        authorizer: Arc<dyn PeerAuthorizer>,
        shutdown: CancellationToken,
        server_id: EndpointId,
        client_id: EndpointId,
        send: SendStream,
        recv: RecvStream,
    ) -> Result<(), String> {
        drive_session(
            recv,
            send,
            sessions,
            Some(PeerId::Iroh(server_id)),
            &shutdown,
            |init| {
                authorizer.authorize(AuthorizationRequest {
                    client: ClientId::Iroh(client_id),
                    device_index: init.device_index,
                    credential: &Credential::from(init.authorization.as_slice()),
                })
            },
        )
        .await
    }
}

/// Burn Remote's handler for an application's own Iroh router, from `into_protocol`. Register it
/// under [`BURN_REMOTE_ALPN`](crate::BURN_REMOTE_ALPN); shutting the router down ends its
/// sessions.
pub struct RemoteProtocol(Box<dyn DynProtocolHandler>);

impl RemoteProtocol {
    pub(crate) fn new(handler: impl ProtocolHandler) -> Self {
        Self(handler.into())
    }
}

impl fmt::Debug for RemoteProtocol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RemoteProtocol").finish_non_exhaustive()
    }
}

impl From<RemoteProtocol> for Box<dyn DynProtocolHandler> {
    fn from(protocol: RemoteProtocol) -> Self {
        protocol.0
    }
}

impl<B: BackendIr> ProtocolHandler for IrohRemoteProtocol<B> {
    async fn accept(&self, connection: Connection) -> Result<(), AcceptError> {
        let client_id = connection.remote_id();
        loop {
            let Some((kind, send, recv)) = RemoteNode::accept_stream(&connection)
                .await
                .map_err(user_error)?
            else {
                return Ok(());
            };

            match kind {
                StreamKind::Session => {
                    let sessions = self.sessions.clone();
                    let authorizer = self.authorizer.clone();
                    let shutdown = self.shutdown.clone();
                    let server_id = self.node.id();
                    spawn_detached(async move {
                        if let Err(err) = Self::handle_session(
                            sessions, authorizer, shutdown, server_id, client_id, send, recv,
                        )
                        .await
                        {
                            log::warn!("Rejected or failed Iroh remote session: {err}");
                        }
                    });
                },
                StreamKind::TensorTransfer => {
                    let transfer = self.transfer.clone();
                    spawn_detached(async move {
                        if let Err(err) = transfer.handle_stream(client_id, send, recv).await {
                            log::warn!("Iroh tensor-transfer stream failed: {err}");
                        }
                    });
                },
            }
        }
    }

    async fn shutdown(&self) {
        self.shutdown.cancel();
    }
}

fn user_error(reason: String) -> AcceptError {
    AcceptError::from_err(std::io::Error::other(reason))
}
