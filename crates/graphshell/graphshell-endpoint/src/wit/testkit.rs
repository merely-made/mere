// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! A fixture endpoint, a gate whose authority lapses after a set number of
//! calls, and one call script run through any `Carrier`. Every binding of
//! the projection session must produce the same transcript.

use std::collections::VecDeque;

use chirograph::{
    CachePolicy, CapabilityProfile, Carrier, CarrierNotice, CarrierRequestBody, ContentHash,
    EndpointDescriptor, IntentInvocation, IntentResult, PresentationCapability,
    PresentationManifest, ProjectionAck, ProjectionOffer, ProjectionRequest, ProjectionSession,
    ProjectionSnapshot, ProtocolVersion, ResourceChunkRequest, ResourceRequest, ResourceResponse,
    ResumeReply, ResumeRequest, Revision, SceneEpoch, SceneSnapshot, SessionOpen, SessionOpened,
    SessionStatus,
};
use sceno::{Arrangement, InstanceId, Scene, Score, Spiral};

use super::{Failure, payload};
use crate::{
    IntentSink, PresentationSource, ProjectionCatalog, ProjectionNoticeSource, ProjectionSource,
    ResumableProjectionSource,
};

/// A small resource every session can read.
pub const CARD: &[u8] = b"card bytes for the projection session";
/// A resource whose reply exceeds the payload cap.
pub const HUGE_LEN: usize = 2 * payload::MAX_PAYLOAD;
/// Session calls the gate allows before revoking; the script's last calls
/// land after it.
pub const CALLS_BEFORE_REVOCATION: usize = 11;

fn huge() -> Vec<u8> {
    vec![0xa5; HUGE_LEN]
}

fn score() -> Score {
    Score::new(Arrangement::Spiral(Spiral::default()))
}

fn descriptor(session: &ProjectionSession) -> EndpointDescriptor {
    EndpointDescriptor {
        label: "fixture".into(),
        projections: vec![ProjectionOffer {
            label: "spiral".into(),
            request: ProjectionRequest {
                version: ProtocolVersion::V1,
                session: session.clone(),
                score: score(),
            },
        }],
    }
}

/// An endpoint with one card resource, one oversized one, and an intent
/// that advances the scene and raises a notice.
#[derive(Default)]
pub struct FixtureEndpoint {
    notices: VecDeque<CarrierNotice>,
}

impl ProjectionCatalog for FixtureEndpoint {
    fn describe(&self) -> EndpointDescriptor {
        descriptor(&ProjectionSession("fixture:unopened".into()))
    }
}

impl ProjectionSource for FixtureEndpoint {
    type Error = String;
    fn snapshot(&mut self, request: ProjectionRequest) -> Result<ProjectionSnapshot, String> {
        Ok(ProjectionSnapshot {
            version: request.version,
            session: request.session,
            scene: SceneSnapshot::from_dense(SceneEpoch(1), Revision(1), Scene::new())
                .map_err(|e| format!("{e:?}"))?,
            presentation: PresentationManifest::default(),
            cache_policy: CachePolicy::default(),
        })
    }
}

impl PresentationSource for FixtureEndpoint {
    type Error = String;
    fn resource(&mut self, request: ResourceRequest) -> Result<ResourceResponse, String> {
        let bytes = if request.resource == ContentHash::of(CARD) {
            CARD.to_vec()
        } else if request.resource == ContentHash::of(&huge()) {
            huge()
        } else {
            return Err("unknown resource".into());
        };
        Ok(ResourceResponse::new(request.session, bytes))
    }
}

impl ResumableProjectionSource for FixtureEndpoint {
    type Error = String;
    fn resume(&mut self, request: ResumeRequest) -> Result<ResumeReply, String> {
        Ok(ResumeReply::Current(ProjectionAck {
            session: request.session,
            epoch: request.epoch,
            revision: request.revision,
        }))
    }
}

impl IntentSink for FixtureEndpoint {
    type Error = String;
    fn invoke(&mut self, intent: IntentInvocation) -> Result<IntentResult, String> {
        if intent.observed_revision != Revision(1) {
            return Ok(IntentResult::Stale {
                current_epoch: SceneEpoch(1),
                current_revision: Revision(2),
            });
        }
        self.notices.push_back(CarrierNotice {
            session: intent.session,
            epoch: SceneEpoch(1),
            revision: Revision(2),
        });
        Ok(IntentResult::Accepted)
    }
}

impl ProjectionNoticeSource for FixtureEndpoint {
    type Error = String;
    fn poll_notice(&mut self) -> Result<Option<CarrierNotice>, String> {
        Ok(self.notices.pop_front())
    }
}

/// Admits any Open that names a presentation capability, under an id that
/// stands in for the admission transcript's digest, and revokes the
/// authority after a set number of calls.
pub struct CountdownGate {
    calls_left: usize,
    next: u64,
    pub closed: Vec<ProjectionSession>,
}

impl CountdownGate {
    pub fn new(calls_before_revocation: usize) -> Self {
        Self {
            calls_left: calls_before_revocation,
            next: 1,
            closed: Vec::new(),
        }
    }
}

impl super::SessionGate for CountdownGate {
    fn open(&mut self, open: SessionOpen) -> Result<(ProjectionSession, SessionOpened), Failure> {
        if open.capabilities == CapabilityProfile::new([]) {
            return Err(Failure::Refused("no presentation capability".into()));
        }
        let id = ProjectionSession(format!("fixture:admitted:{}", self.next));
        self.next += 1;
        let opened = SessionOpened {
            version: ProtocolVersion::V1,
            descriptor: descriptor(&id),
            status: SessionStatus::Live,
            expires_at_ms: None,
        };
        Ok((id, opened))
    }

    fn recheck(&mut self, _: &ProjectionSession) -> Result<(), Failure> {
        if self.calls_left == 0 {
            return Err(Failure::Revoked);
        }
        self.calls_left -= 1;
        Ok(())
    }

    fn closed(&mut self, session: &ProjectionSession) {
        self.closed.push(session.clone());
    }
}

pub type Transcript = Vec<(String, String)>;

fn intent(session: &ProjectionSession, revision: u64, len: usize) -> CarrierRequestBody {
    CarrierRequestBody::Intent(IntentInvocation {
        session: session.clone(),
        target: InstanceId(0),
        observed_epoch: SceneEpoch(1),
        observed_revision: Revision(revision),
        intent: "select".into(),
        payload: vec![7; len],
    })
}

/// Drive a carrier through every verb, both caps, both notice calls and a
/// revocation.
pub fn run(carrier: &mut impl Carrier) -> Transcript {
    let mut t = Transcript::new();
    let ask = |t: &mut Transcript, carrier: &mut dyn Carrier, name: &str, body| {
        let reply = carrier.request(body);
        t.push((name.into(), format!("{reply:?}")));
        reply.ok()
    };
    ask(&mut t, carrier, "discover", CarrierRequestBody::Discover);
    let open = |caps: Vec<PresentationCapability>| {
        CarrierRequestBody::Open(Box::new(SessionOpen {
            version: ProtocolVersion::V1,
            capabilities: CapabilityProfile::new(caps),
        }))
    };
    ask(&mut t, carrier, "open.empty", open(vec![]));
    let opened = ask(
        &mut t,
        carrier,
        "open",
        open(vec![PresentationCapability::PortableCard]),
    );
    let Some(chirograph::CarrierResponseBody::Opened(opened)) = opened else {
        return t;
    };
    let id = opened.descriptor.projections[0].request.session.clone();
    let snapshot = |session: &ProjectionSession| {
        CarrierRequestBody::Snapshot(ProjectionRequest {
            version: ProtocolVersion::V1,
            session: session.clone(),
            score: score(),
        })
    };
    let fetch = |bytes: &[u8]| {
        CarrierRequestBody::Resource(ResourceRequest {
            session: id.clone(),
            resource: ContentHash::of(bytes),
        })
    };
    let chunk = |offset, length| {
        CarrierRequestBody::ResourceChunk(ResourceChunkRequest {
            session: id.clone(),
            resource: ContentHash::of(CARD),
            offset,
            length,
        })
    };
    ask(&mut t, carrier, "snapshot", snapshot(&id));
    ask(&mut t, carrier, "fetch", fetch(CARD));
    ask(&mut t, carrier, "fetch.huge", fetch(&huge()));
    ask(&mut t, carrier, "chunk.0", chunk(0, 4));
    ask(&mut t, carrier, "chunk.4", chunk(4, 0));
    ask(&mut t, carrier, "intent", intent(&id, 1, 8));
    t.push((
        "wait-notice".into(),
        format!("{:?}", carrier.wait_for_notice()),
    ));
    t.push(("take-notice".into(), format!("{:?}", carrier.take_notice())));
    ask(&mut t, carrier, "intent.stale", intent(&id, 7, 8));
    let resume = CarrierRequestBody::Resume(ResumeRequest {
        session: id.clone(),
        epoch: SceneEpoch(1),
        revision: Revision(2),
    });
    ask(&mut t, carrier, "resume", resume);
    // Over the cap on the consumer side: refused before it is sent.
    ask(
        &mut t,
        carrier,
        "intent.huge",
        intent(&id, 1, payload::MAX_PAYLOAD),
    );
    ask(&mut t, carrier, "suspend", CarrierRequestBody::Suspend);
    // Another session's request is refused before it is sent.
    let other = ProjectionSession("fixture:someone-else".into());
    ask(&mut t, carrier, "snapshot.other", snapshot(&other));
    // The authority has lapsed: every method rechecks and discloses nothing.
    ask(&mut t, carrier, "snapshot.revoked", snapshot(&id));
    t.push((
        "take-notice.revoked".into(),
        format!("{:?}", carrier.take_notice()),
    ));
    t.push((
        "wait-notice.revoked".into(),
        format!("{:?}", carrier.wait_for_notice()),
    ));
    ask(&mut t, carrier, "close", CarrierRequestBody::Close);
    t
}

/// The outcomes every binding's transcript must show.
pub fn assert_outcomes(t: &Transcript) {
    let get = |name: &str| {
        t.iter()
            .find(|(n, _)| n == name)
            .map(|(_, v)| v.as_str())
            .unwrap_or_else(|| panic!("no `{name}` in the transcript"))
    };
    assert!(
        get("discover").starts_with("Ok(Descriptor("),
        "{}",
        get("discover")
    );
    assert!(get("open.empty").contains("no presentation capability"));
    assert!(get("open").starts_with("Ok(Opened("));
    assert!(get("snapshot").starts_with("Ok(Snapshot("));
    assert!(get("fetch").starts_with("Ok(Resource("));
    assert!(get("fetch.huge").contains("TooLarge"));
    assert!(get("chunk.0").starts_with("Ok(ResourceChunk("));
    assert_eq!(get("intent"), "Ok(Intent(Accepted))");
    assert!(get("wait-notice").contains("revision: Revision(2)"));
    assert_eq!(get("take-notice"), "None");
    assert!(get("intent.stale").contains("Stale"));
    assert!(get("resume").starts_with("Ok(Resume(Current("));
    assert!(get("intent.huge").contains("TooLarge"));
    assert_eq!(get("suspend"), "Ok(Suspended)");
    assert!(get("snapshot.other").contains("is not this carrier's"));
    assert_eq!(get("snapshot.revoked"), "Err(Refused(\"Revoked\"))");
    assert_eq!(get("take-notice.revoked"), "None");
    assert_eq!(get("wait-notice.revoked"), "Err(Refused(\"Revoked\"))");
    assert_eq!(get("close"), "Ok(Closed)");
}
