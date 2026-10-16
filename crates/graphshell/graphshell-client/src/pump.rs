// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! An admitted native stream as lines in and lines out.
//!
//! The line-driven sessions in this crate and above it ([`crate::SessionDriver`],
//! `mere_overmap::route_session::MereRouteSession`) take the lines that arrive
//! and give the lines to send, and own no I/O. A browser feeds them from its
//! data channel. This is the native side of the same bargain (transport
//! agnosticism note, ruling T3): an admitted Tokio stream, read and written
//! as NDJSON lines in [`chirograph::line`]'s spelling, and a loop that runs a
//! [`LineDriven`] session over it until the session settles.
//!
//! Behind the `pump` feature, so the default client keeps no runtime.

use crate::driver::LineDriven;
use chirograph::{
    CapabilityProfile, CarrierOutput, CarrierRequest, CarrierRequestBody, CarrierResponseBody,
    ProtocolVersion, SessionOpen, SessionOpened,
};
use tokio::io::{
    AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader, Lines, ReadHalf, WriteHalf,
};

/// Why a pump stopped.
#[derive(Debug)]
pub enum PumpError {
    /// The stream failed.
    Io(std::io::Error),
    /// The stream ended while the session still waited for a line.
    Closed,
    /// The session, or the endpoint's answer to opening it, refused to go on.
    Session(String),
}

impl std::fmt::Display for PumpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(f, "session stream failed: {error}"),
            Self::Closed => write!(f, "the session stream ended"),
            Self::Session(reason) => write!(f, "{reason}"),
        }
    }
}

impl std::error::Error for PumpError {}

impl From<std::io::Error> for PumpError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

/// An admitted stream, split into a line reader and a line writer.
pub struct LinePump<S> {
    lines: Lines<BufReader<ReadHalf<S>>>,
    writer: WriteHalf<S>,
}

impl<S: AsyncRead + AsyncWrite> LinePump<S> {
    /// Pump lines over `stream`, which admission has already cleared.
    pub fn new(stream: S) -> Self {
        let (reader, writer) = tokio::io::split(stream);
        Self {
            lines: BufReader::new(reader).lines(),
            writer,
        }
    }

    /// Send one line and flush it.
    pub async fn send(&mut self, line: &str) -> Result<(), PumpError> {
        self.writer.write_all(line.as_bytes()).await?;
        self.writer.write_all(b"\n").await?;
        self.writer.flush().await?;
        Ok(())
    }

    /// The next line, without its newline; `None` when the stream ends.
    /// Cancel safe, as `Lines::next_line` is.
    pub async fn recv(&mut self) -> Result<Option<String>, PumpError> {
        Ok(self.lines.next_line().await?)
    }

    /// Open the session the stream carries and return what the endpoint
    /// described. The request id is 0, below any a [`crate::SessionDriver`]
    /// numbers its own requests from.
    pub async fn open(
        &mut self,
        capabilities: CapabilityProfile,
    ) -> Result<SessionOpened, PumpError> {
        let request = CarrierRequest {
            id: 0,
            body: CarrierRequestBody::Open(Box::new(SessionOpen {
                version: ProtocolVersion::V1,
                capabilities,
            })),
        };
        let line = chirograph::line::encode(&request)
            .map_err(|error| PumpError::Session(format!("could not encode the open: {error}")))?;
        self.send(&line).await?;
        loop {
            let Some(line) = self.recv().await? else {
                return Err(PumpError::Closed);
            };
            let output = chirograph::line::decode::<CarrierOutput>(&line).map_err(|error| {
                PumpError::Session(format!("the endpoint sent an unreadable line: {error}"))
            })?;
            match output {
                // A bell before the session is open has nothing to ring.
                None | Some(CarrierOutput::Notice(_)) => continue,
                Some(CarrierOutput::Response(response)) if response.id == 0 => {
                    return match response.body {
                        Ok(CarrierResponseBody::Opened(opened)) => Ok(*opened),
                        Ok(_) => Err(PumpError::Session(
                            "the endpoint answered the open with something else".into(),
                        )),
                        Err(failure) => Err(PumpError::Session(failure.message)),
                    };
                },
                Some(CarrierOutput::Response(response)) => {
                    return Err(PumpError::Session(format!(
                        "the endpoint answered request {} before the open",
                        response.id
                    )));
                },
            }
        }
    }

    /// Run `session` over the stream until `settled` says it is: send what it
    /// has, and while it is not settled, hand it the next line. Returns once
    /// it is settled with nothing left to send.
    pub async fn drive<L, F>(&mut self, session: &mut L, settled: F) -> Result<(), PumpError>
    where
        L: LineDriven,
        F: Fn(&L) -> bool,
    {
        loop {
            for line in session.take_outgoing() {
                self.send(&line).await?;
            }
            if settled(session) {
                return Ok(());
            }
            let Some(line) = self.recv().await? else {
                return Err(PumpError::Closed);
            };
            session.on_line(&line).map_err(PumpError::Session)?;
        }
    }

    /// Close the writing half, so the endpoint sees the end of the stream.
    pub async fn shutdown(mut self) -> Result<(), PumpError> {
        self.writer.shutdown().await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chirograph::{CarrierResponse, EndpointDescriptor, SessionStatus};

    /// A session that sends `count` requests one at a time and settles once
    /// each has an answer.
    struct Counting {
        sent: u64,
        answered: u64,
        count: u64,
        outbox: Vec<String>,
    }

    impl LineDriven for Counting {
        fn take_outgoing(&mut self) -> Vec<String> {
            if self.outbox.is_empty() && self.sent == self.answered && self.sent < self.count {
                self.sent += 1;
                let request = CarrierRequest {
                    id: self.sent,
                    body: CarrierRequestBody::Close,
                };
                self.outbox
                    .push(chirograph::line::encode(&request).unwrap());
            }
            std::mem::take(&mut self.outbox)
        }

        fn on_line(&mut self, line: &str) -> Result<(), String> {
            match chirograph::line::decode::<CarrierOutput>(line).map_err(|e| e.to_string())? {
                Some(CarrierOutput::Response(response)) if response.id == self.sent => {
                    self.answered += 1;
                    Ok(())
                },
                None | Some(CarrierOutput::Notice(_)) => Ok(()),
                Some(other) => Err(format!("unexpected {other:?}")),
            }
        }
    }

    async fn echo_endpoint(stream: tokio::io::DuplexStream) {
        let (reader, mut writer) = tokio::io::split(stream);
        let mut lines = BufReader::new(reader).lines();
        while let Some(line) = lines.next_line().await.unwrap() {
            let Some(request) = chirograph::line::decode::<CarrierRequest>(&line).unwrap() else {
                continue;
            };
            let body = match request.body {
                CarrierRequestBody::Open(open) => {
                    CarrierResponseBody::Opened(Box::new(SessionOpened {
                        version: open.version,
                        descriptor: EndpointDescriptor {
                            label: "echo".into(),
                            projections: Vec::new(),
                        },
                        status: SessionStatus::Live,
                        expires_at_ms: None,
                    }))
                },
                _ => CarrierResponseBody::Closed,
            };
            // A blank line between frames is skipped by the reader.
            writer.write_all(b"\n").await.unwrap();
            let output = CarrierOutput::Response(CarrierResponse {
                id: request.id,
                body: Ok(body),
            });
            writer
                .write_all(&chirograph::line::encode_terminated(&output).unwrap())
                .await
                .unwrap();
            writer.flush().await.unwrap();
        }
    }

    #[tokio::test]
    async fn a_session_opens_and_runs_until_settled_over_one_stream() {
        let (client, server) = tokio::io::duplex(4096);
        let endpoint = tokio::spawn(echo_endpoint(server));
        let mut pump = LinePump::new(client);
        let opened = pump.open(CapabilityProfile::default()).await.unwrap();
        assert_eq!(opened.descriptor.label, "echo");

        let mut session = Counting {
            sent: 0,
            answered: 0,
            count: 3,
            outbox: Vec::new(),
        };
        pump.drive(&mut session, |session| session.answered == session.count)
            .await
            .unwrap();
        assert_eq!((session.sent, session.answered), (3, 3));
        pump.shutdown().await.unwrap();
        endpoint.await.unwrap();
    }

    #[tokio::test]
    async fn a_stream_that_ends_early_is_reported() {
        let (client, server) = tokio::io::duplex(4096);
        drop(server);
        let mut pump = LinePump::new(client);
        assert!(matches!(
            pump.open(CapabilityProfile::default()).await,
            Err(PumpError::Io(_) | PumpError::Closed)
        ));
    }
}
