// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! A chirograph `Carrier` over the WIT calls.

use chirograph::{
    Carrier, CarrierError, CarrierNotice, CarrierRequestBody, CarrierResponseBody,
    EndpointDescriptor, IntentResult, ProjectionSession, ProjectionSnapshot, ResourceChunkResponse,
    ResourceResponse, ResumeReply, Revision, SceneEpoch, SessionOpened, SessionStatus,
};

use super::{Failure, IntentCall, Notice, ProjectionAsk, payload};

/// Why a call returned no answer: the provider declined it, or the channel
/// is gone.
#[derive(Clone, Debug, PartialEq)]
pub enum Fault {
    Refused(Failure),
    Disconnected(String),
}

impl From<Failure> for Fault {
    fn from(f: Failure) -> Self {
        Fault::Refused(f)
    }
}

impl From<Fault> for CarrierError {
    fn from(f: Fault) -> Self {
        match f {
            // A lapse reaches the client as its status (D20).
            Fault::Refused(Failure::Revoked) => CarrierError::Lapsed(SessionStatus::Revoked),
            Fault::Refused(Failure::Expired) => CarrierError::Lapsed(SessionStatus::Expired),
            Fault::Refused(f) => CarrierError::Refused(format!("{f:?}")),
            Fault::Disconnected(m) => CarrierError::Disconnected(m),
        }
    }
}

/// The WIT interface's calls, through some binding: the generated client
/// natively, or a component's imports. `Session` is the binding's resource;
/// [`SessionCalls::close`] drops it.
pub trait SessionCalls {
    type Session;
    fn discover(&mut self) -> Result<Vec<u8>, Fault>;
    fn open(&mut self, request: Vec<u8>) -> Result<(Self::Session, Vec<u8>), Fault>;
    fn snapshot(&mut self, s: &Self::Session, ask: Vec<u8>) -> Result<Vec<u8>, Fault>;
    fn fetch(&mut self, s: &Self::Session, address: Vec<u8>) -> Result<Vec<u8>, Fault>;
    fn chunk(
        &mut self,
        s: &Self::Session,
        address: Vec<u8>,
        offset: u64,
        length: u32,
    ) -> Result<Vec<u8>, Fault>;
    fn resume(&mut self, s: &Self::Session, epoch: u64, revision: u64) -> Result<Vec<u8>, Fault>;
    fn intent(&mut self, s: &Self::Session, call: Vec<u8>) -> Result<Vec<u8>, Fault>;
    fn suspend(&mut self, s: &Self::Session) -> Result<(), Fault>;
    fn take_notice(&mut self, s: &Self::Session) -> Result<Option<Notice>, Fault>;
    fn wait_notice(&mut self, s: &Self::Session) -> Result<Notice, Fault>;
    fn close(&mut self, s: Self::Session) -> Result<(), Fault>;
}

/// A `Carrier` over [`SessionCalls`]. It holds the opened session, fills
/// chirograph's session field from it, and refuses a request or reply that
/// names another session (D15). A `revoked` or `expired` failure is
/// [`CarrierError::Lapsed`] (D20); one met where it cannot be returned, in
/// `take_notice`, is kept and returned by the next call.
pub struct WitCarrier<C: SessionCalls> {
    calls: C,
    session: Option<C::Session>,
    id: Option<ProjectionSession>,
    lapsed: Option<SessionStatus>,
}

impl<C: SessionCalls> WitCarrier<C> {
    pub fn new(calls: C) -> Self {
        Self {
            calls,
            session: None,
            id: None,
            lapsed: None,
        }
    }

    /// Call through the open session.
    fn call<R>(
        &mut self,
        f: impl FnOnce(&mut C, &C::Session) -> Result<R, Fault>,
    ) -> Result<R, CarrierError> {
        let session = self
            .session
            .as_ref()
            .ok_or_else(|| CarrierError::Refused("no session is open".into()))?;
        Ok(f(&mut self.calls, session)?)
    }

    /// Refuse a session other than the one this carrier holds.
    fn same(&mut self, session: &ProjectionSession) -> Result<(), CarrierError> {
        match &self.id {
            Some(id) if id != session => Err(CarrierError::Refused(format!(
                "session {} is not this carrier's {}",
                session.0, id.0
            ))),
            Some(_) => Ok(()),
            None => {
                self.id = Some(session.clone());
                Ok(())
            },
        }
    }

    fn notice(&self, n: Notice) -> Result<CarrierNotice, CarrierError> {
        let session = self
            .id
            .clone()
            .ok_or_else(|| CarrierError::Refused("notice before any session id".into()))?;
        Ok(CarrierNotice {
            session,
            epoch: SceneEpoch(n.epoch),
            revision: Revision(n.revision),
        })
    }
}

fn decode<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> Result<T, CarrierError> {
    payload::decode(bytes).map_err(|f| Fault::Refused(f).into())
}

fn encode<T: serde::Serialize>(value: &T) -> Result<Vec<u8>, CarrierError> {
    payload::encode(value).map_err(|f| Fault::Refused(f).into())
}

impl<C: SessionCalls> Carrier for WitCarrier<C> {
    fn request(&mut self, body: CarrierRequestBody) -> Result<CarrierResponseBody, CarrierError> {
        if let Some(status) = self.lapsed.take() {
            return Err(CarrierError::Lapsed(status));
        }
        use CarrierRequestBody as Q;
        use CarrierResponseBody as A;
        Ok(match body {
            Q::Discover => A::Descriptor(decode::<EndpointDescriptor>(&self.calls.discover()?)?),
            Q::Open(open) => {
                if self.session.is_some() {
                    return Err(CarrierError::Refused("a session is already open".into()));
                }
                let (session, opened) = self.calls.open(encode(&*open)?)?;
                let opened: SessionOpened = decode(&opened)?;
                self.session = Some(session);
                // One admission mints one id; every offer carries it.
                self.id = opened
                    .descriptor
                    .projections
                    .first()
                    .map(|o| o.request.session.clone());
                A::Opened(Box::new(opened))
            },
            Q::Snapshot(request) => {
                self.same(&request.session)?;
                let ask = encode(&ProjectionAsk {
                    version: request.version,
                    score: request.score,
                })?;
                let snapshot: ProjectionSnapshot = decode(&self.call(|c, s| c.snapshot(s, ask))?)?;
                self.same(&snapshot.session)?;
                A::Snapshot(Box::new(snapshot))
            },
            Q::Resource(request) => {
                self.same(&request.session)?;
                let address = encode(&request.resource)?;
                let bytes = self.call(|c, s| c.fetch(s, address))?;
                A::Resource(decode::<ResourceResponse>(&bytes)?)
            },
            Q::ResourceChunk(request) => {
                self.same(&request.session)?;
                let address = encode(&request.resource)?;
                let (offset, length) = (request.offset, request.length);
                let bytes = self.call(|c, s| c.chunk(s, address, offset, length))?;
                A::ResourceChunk(Box::new(decode::<ResourceChunkResponse>(&bytes)?))
            },
            Q::Resume(request) => {
                self.same(&request.session)?;
                let (epoch, revision) = (request.epoch.0, request.revision.0);
                let bytes = self.call(|c, s| c.resume(s, epoch, revision))?;
                A::Resume(decode::<ResumeReply>(&bytes)?)
            },
            Q::Intent(intent) => {
                self.same(&intent.session)?;
                let call = encode(&IntentCall {
                    target: intent.target,
                    observed_epoch: intent.observed_epoch,
                    observed_revision: intent.observed_revision,
                    intent: intent.intent,
                    payload: intent.payload,
                })?;
                A::Intent(decode::<IntentResult>(
                    &self.call(|c, s| c.intent(s, call))?,
                )?)
            },
            Q::Suspend => {
                self.call(|c, s| c.suspend(s))?;
                A::Suspended
            },
            // The WIT interface has no page-text call; the door carries it.
            Q::RetainedPages(_) => {
                return Err(CarrierError::Refused(
                    "this carrier has no retained page text read".into(),
                ));
            },
            Q::Close => {
                let session = self
                    .session
                    .take()
                    .ok_or_else(|| CarrierError::Refused("no session is open".into()))?;
                self.calls.close(session)?;
                A::Closed
            },
        })
    }

    fn take_notice(&mut self) -> Option<CarrierNotice> {
        let session = self.session.as_ref()?;
        match self.calls.take_notice(session) {
            Ok(n) => self.notice(n?).ok(),
            Err(fault) => {
                self.lapsed = CarrierError::from(fault).lapse();
                None
            },
        }
    }

    fn wait_for_notice(&mut self) -> Result<CarrierNotice, CarrierError> {
        if let Some(status) = self.lapsed.take() {
            return Err(CarrierError::Lapsed(status));
        }
        let n = self.call(|c, s| c.wait_notice(s))?;
        self.notice(n)
    }

    fn shutdown(&mut self) -> Result<(), CarrierError> {
        match self.session.take() {
            Some(session) => self.calls.close(session).map_err(Into::into),
            None => Ok(()),
        }
    }
}
