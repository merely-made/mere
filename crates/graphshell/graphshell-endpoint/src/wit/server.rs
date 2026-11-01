// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Serving a complete endpoint through the generated handler.

use std::collections::{BTreeMap, VecDeque};
use std::fmt::Display;
use std::time::Duration;

use chirograph::{
    ContentHash, IntentInvocation, ProjectionRequest, ProjectionSession, ResourceChunkRequest,
    ResourceRequest, ResumeRequest, Revision, SceneEpoch, SessionOpen, SessionOpened,
};
use script_wire::Serve;

use super::bindings::{Handler, Session, dispatch};
use super::{Failure, IntentCall, Notice, ProjectionAsk, payload};
use crate::{
    IntentSink, PresentationSource, ProjectionCatalog, ProjectionNoticeSource, ProjectionSource,
    ResumableProjectionSource,
};

/// Admission and authority for the sessions a server opens.
pub trait SessionGate {
    /// Admit an Open: the session's id, derived from the channel's
    /// admission, and the reply.
    fn open(&mut self, open: SessionOpen) -> Result<(ProjectionSession, SessionOpened), Failure>;
    /// Recheck the session's authority before a call discloses anything
    /// (D13): a lapse is `revoked` or `expired`.
    fn recheck(&mut self, session: &ProjectionSession) -> Result<(), Failure>;
    /// The session closed: dropped, or its channel ended.
    fn closed(&mut self, _session: &ProjectionSession) {}
}

struct Opened {
    id: ProjectionSession,
    notices: VecDeque<Notice>,
}

/// An endpoint and its gate behind the generated handler. Handles are
/// minted per server, so a handle from another channel names nothing here.
pub struct WitServer<E, G> {
    pub endpoint: E,
    pub gate: G,
    sessions: BTreeMap<u64, Opened>,
    next: u64,
}

/// How often [`Handler::session_wait_notice`] polls the endpoint, which
/// offers no blocking wait of its own.
const NOTICE_POLL: Duration = Duration::from_millis(5);

impl<E, G> WitServer<E, G> {
    pub fn new(endpoint: E, gate: G) -> Self {
        Self {
            endpoint,
            gate,
            sessions: BTreeMap::new(),
            next: 1,
        }
    }

    /// The sessions still open.
    pub fn open_sessions(&self) -> usize {
        self.sessions.len()
    }
}

/// Serve `server` on one channel until the consumer hangs up, then close
/// every session the channel opened (D19). A call in flight when the channel
/// breaks reaches the consumer as a disconnect.
pub fn serve<E, G>(server: &mut WitServer<E, G>, channel: impl Serve) -> std::io::Result<()>
where
    WitServer<E, G>: Handler,
    G: SessionGate,
{
    let ended = channel.serve(|frame| dispatch(server, frame));
    for (_, opened) in std::mem::take(&mut server.sessions) {
        server.gate.closed(&opened.id);
    }
    ended
}

fn refused(e: impl Display) -> Failure {
    Failure::Refused(e.to_string())
}

impl<E, G> WitServer<E, G>
where
    E: ProjectionNoticeSource,
    E::Error: Display,
    G: SessionGate,
{
    /// The session behind a handle, after its authority is rechecked.
    fn admit(&mut self, s: &Session) -> Result<ProjectionSession, Failure> {
        let handle = <[u8; 8]>::try_from(s.0.as_slice())
            .map(u64::from_le_bytes)
            .map_err(|_| Failure::Refused("malformed session handle".into()))?;
        let id = self
            .sessions
            .get(&handle)
            .map(|o| o.id.clone())
            .ok_or_else(|| Failure::Refused("no such session".into()))?;
        self.gate.recheck(&id)?;
        Ok(id)
    }

    /// Move the endpoint's notices into their sessions' queues.
    fn collect_notices(&mut self) -> Result<(), Failure> {
        while let Some(n) = self.endpoint.poll_notice().map_err(refused)? {
            if let Some(o) = self.sessions.values_mut().find(|o| o.id == n.session) {
                o.notices.push_back(Notice {
                    epoch: n.epoch.0,
                    revision: n.revision.0,
                });
            }
        }
        Ok(())
    }

    fn next_notice(&mut self, id: &ProjectionSession) -> Result<Option<Notice>, Failure> {
        self.collect_notices()?;
        Ok(self
            .sessions
            .values_mut()
            .find(|o| &o.id == id)
            .and_then(|o| o.notices.pop_front()))
    }
}

impl<E, G> Handler for WitServer<E, G>
where
    E: ProjectionCatalog
        + ProjectionSource
        + PresentationSource
        + IntentSink
        + ResumableProjectionSource
        + ProjectionNoticeSource,
    <E as ProjectionSource>::Error: Display,
    <E as PresentationSource>::Error: Display,
    <E as IntentSink>::Error: Display,
    <E as ResumableProjectionSource>::Error: Display,
    <E as ProjectionNoticeSource>::Error: Display,
    G: SessionGate,
{
    fn discover(&mut self) -> Result<Vec<u8>, Failure> {
        payload::encode(&self.endpoint.describe())
    }

    fn open(&mut self, request: Vec<u8>) -> Result<(Session, Vec<u8>), Failure> {
        let open: SessionOpen = payload::decode(&request)?;
        let (id, opened) = self.gate.open(open)?;
        let reply = payload::encode(&opened)?;
        let handle = self.next;
        self.next += 1;
        self.sessions.insert(
            handle,
            Opened {
                id,
                notices: VecDeque::new(),
            },
        );
        Ok((Session(handle.to_le_bytes().to_vec()), reply))
    }

    fn session_snapshot(&mut self, this: &Session, ask: Vec<u8>) -> Result<Vec<u8>, Failure> {
        let session = self.admit(this)?;
        let ask: ProjectionAsk = payload::decode(&ask)?;
        let snapshot = self
            .endpoint
            .snapshot(ProjectionRequest {
                version: ask.version,
                session,
                score: ask.score,
            })
            .map_err(refused)?;
        payload::encode(&snapshot)
    }

    fn session_fetch(&mut self, this: &Session, address: Vec<u8>) -> Result<Vec<u8>, Failure> {
        let session = self.admit(this)?;
        let resource: ContentHash = payload::decode(&address)?;
        let response = self
            .endpoint
            .resource(ResourceRequest { session, resource })
            .map_err(refused)?;
        payload::encode(&response)
    }

    fn session_chunk(
        &mut self,
        this: &Session,
        address: Vec<u8>,
        offset: u64,
        length: u32,
    ) -> Result<Vec<u8>, Failure> {
        let session = self.admit(this)?;
        let resource: ContentHash = payload::decode(&address)?;
        let chunk = self
            .endpoint
            .resource_chunk(ResourceChunkRequest {
                session,
                resource,
                offset,
                length,
            })
            .map_err(refused)?;
        payload::encode(&chunk)
    }

    fn session_resume(
        &mut self,
        this: &Session,
        epoch: u64,
        revision: u64,
    ) -> Result<Vec<u8>, Failure> {
        let session = self.admit(this)?;
        let reply = self
            .endpoint
            .resume(ResumeRequest {
                session,
                epoch: SceneEpoch(epoch),
                revision: Revision(revision),
            })
            .map_err(refused)?;
        payload::encode(&reply)
    }

    fn session_intent(&mut self, this: &Session, call: Vec<u8>) -> Result<Vec<u8>, Failure> {
        let session = self.admit(this)?;
        let call: IntentCall = payload::decode(&call)?;
        let result = self
            .endpoint
            .invoke(IntentInvocation {
                session,
                target: call.target,
                observed_epoch: call.observed_epoch,
                observed_revision: call.observed_revision,
                intent: call.intent,
                payload: call.payload,
            })
            .map_err(refused)?;
        payload::encode(&result)
    }

    fn session_suspend(&mut self, this: &Session) -> Result<(), Failure> {
        self.admit(this).map(|_| ())
    }

    fn session_take_notice(&mut self, this: &Session) -> Option<Notice> {
        let id = self.admit(this).ok()?;
        self.next_notice(&id).ok()?
    }

    fn session_wait_notice(&mut self, this: &Session) -> Result<Notice, Failure> {
        loop {
            // Rechecked each round, so a revocation ends the wait.
            let id = self.admit(this)?;
            if let Some(n) = self.next_notice(&id)? {
                return Ok(n);
            }
            std::thread::sleep(NOTICE_POLL);
        }
    }

    fn drop_session(&mut self, this: Session) {
        let handle = <[u8; 8]>::try_from(this.0.as_slice()).map(u64::from_le_bytes);
        if let Some(opened) = handle.ok().and_then(|h| self.sessions.remove(&h)) {
            self.gate.closed(&opened.id);
        }
    }
}
