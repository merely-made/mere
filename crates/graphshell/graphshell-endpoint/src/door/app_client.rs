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

use rand_core::{OsRng, RngCore};

use crate::door::app_admission::{AppHello, AppId, AppRouteId, configured_app_endpoint};
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
    #[error("a card resource was not valid PortableCardV1 JSON")]
    MalformedCard,
    #[error("the resident resource has another scope or content address")]
    InvalidResourceAddress,
    #[error("the resident resource chunk is invalid: {0}")]
    InvalidResourceChunk(String),
    #[error("transport failed: {0}")]
    Io(#[from] std::io::Error),
}

/// An open session against the resident host, as `app`.
pub struct AppConnection {
    stream: Box<dyn LocalStream>,
    session: String,
    next_id: u64,
}

impl AppConnection {
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
        let challenge =
            match read_host::<serde::de::IgnoredAny, serde::de::IgnoredAny, serde::de::IgnoredAny>(
                &mut stream,
            )
            .await?
            {
                AppHostMessage::Challenge { challenge } => challenge,
                other => return Err(unexpected("a challenge", &other)),
            };
        let mut client_nonce = [0u8; 32];
        OsRng.fill_bytes(&mut client_nonce);
        write_native_message_async(
            &mut stream,
            &AppMessage::<(), ()>::Connect {
                schema: APP_CONNECT_SCHEMA.to_string(),
                host_nonce: challenge.host_nonce,
                client_nonce: base64::Engine::encode(
                    &base64::engine::general_purpose::URL_SAFE_NO_PAD,
                    client_nonce,
                ),
            },
        )
        .await?;
        let session =
            match read_host::<serde::de::IgnoredAny, serde::de::IgnoredAny, serde::de::IgnoredAny>(
                &mut stream,
            )
            .await?
            {
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

    pub fn next_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
    pub async fn request<
        R: serde::Serialize,
        S: serde::de::DeserializeOwned,
        N: serde::de::DeserializeOwned,
    >(
        &mut self,
        request: R,
    ) -> Result<S, AppClientError> {
        write_native_message_async(&mut self.stream, &AppMessage::<R, ()>::Request { request })
            .await?;
        match read_host::<S, N, serde::de::IgnoredAny>(&mut self.stream).await? {
            AppHostMessage::Response { response } => Ok(response),
            other => Err(unexpected("a carrier response", &other)),
        }
    }
    pub async fn call<C: serde::Serialize, A: serde::de::DeserializeOwned>(
        &mut self,
        call: C,
    ) -> Result<A, AppClientError> {
        let id = self.next_id();
        write_native_message_async(&mut self.stream, &AppMessage::<(), C>::Call { id, call })
            .await?;
        match read_host::<serde::de::IgnoredAny, serde::de::IgnoredAny, A>(&mut self.stream).await?
        {
            AppHostMessage::Answer {
                id: answered,
                answer,
            } if id == answered => Ok(answer),
            AppHostMessage::Answer { id: got, .. } => {
                Err(AppClientError::MismatchedResponse { expected: id, got })
            },
            AppHostMessage::Failure { message } => Err(AppClientError::Refused(message)),
            other => Err(unexpected("a custody answer", &other)),
        }
    }
    pub async fn take_notice<N: serde::de::DeserializeOwned>(
        &mut self,
    ) -> Result<Option<N>, AppClientError> {
        self.notice(false).await
    }
    pub async fn wait_for_notice<N: serde::de::DeserializeOwned>(
        &mut self,
    ) -> Result<N, AppClientError> {
        self.notice(true).await?.ok_or(AppClientError::Closed)
    }
    async fn notice<N: serde::de::DeserializeOwned>(
        &mut self,
        wait: bool,
    ) -> Result<Option<N>, AppClientError> {
        let message = if wait {
            AppMessage::<(), ()>::WaitNotice
        } else {
            AppMessage::<(), ()>::TakeNotice
        };
        write_native_message_async(&mut self.stream, &message).await?;
        match read_host::<serde::de::IgnoredAny, N, serde::de::IgnoredAny>(&mut self.stream).await?
        {
            AppHostMessage::Notice { notice } => Ok(notice),
            other => Err(unexpected("a carrier notice", &other)),
        }
    }
}

async fn read_host<
    R: serde::de::DeserializeOwned,
    N: serde::de::DeserializeOwned,
    A: serde::de::DeserializeOwned,
>(
    stream: &mut Box<dyn LocalStream>,
) -> Result<AppHostMessage<R, N, A>, AppClientError> {
    read_native_message_async(stream)
        .await?
        .ok_or(AppClientError::Closed)
}
fn unexpected<R, N, A>(expected: &'static str, got: &AppHostMessage<R, N, A>) -> AppClientError {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn unexpected_released_answer_does_not_leak_payload_in_error() {
        let (client, mut server) = tokio::io::duplex(4096);
        let mut connection = AppConnection {
            stream: Box::new(client),
            session: "fixture".into(),
            next_id: 1,
        };
        #[derive(serde::Serialize)]
        struct Released<'a> {
            answer: &'static str,
            master: &'a str,
            keys: Vec<()>,
        }
        let server_task = tokio::spawn(async move {
            let _: Option<AppMessage<String, ()>> =
                read_native_message_async(&mut server).await.unwrap();
            let answer = AppHostMessage::<(), (), _>::Answer {
                id: 1,
                answer: Ok::<_, ()>(Released {
                    answer: "released",
                    master: "private-release-material",
                    keys: Vec::new(),
                }),
            };
            write_native_message_async(&mut server, &answer)
                .await
                .unwrap();
        });
        let error = connection
            .request::<_, String, ()>("request")
            .await
            .unwrap_err();
        assert!(matches!(error, AppClientError::UnexpectedAnswer { .. }));
        assert!(!format!("{error:?} {error}").contains("private-release-material"));
        server_task.await.unwrap();
    }
}
