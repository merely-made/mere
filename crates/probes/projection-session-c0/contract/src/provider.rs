//! A test provider over chirograph values. Every session call rechecks the
//! session's authority first (D13), here a countdown standing in for
//! Notochord's retained-authority lapse.

use std::collections::{BTreeMap, VecDeque};

use chirograph::{
    CachePolicy, CapabilityProfile, ContentHash, EndpointDescriptor, IntentResult,
    PresentationManifest, ProjectionAck, ProjectionOffer, ProjectionRequest, ProjectionSession,
    ProjectionSnapshot, ProtocolVersion, ResourceChunkResponse, ResourceResponse, ResumeReply,
    Revision, SceneEpoch, SceneSnapshot, SessionOpen, SessionOpened, SessionStatus,
};
use sceno::{Arrangement, Scene, Score, Spiral};

use crate::{Failure, IntentCall, Notice, ProjectionAsk, Provider, Reply, payload};

/// A small resource every session can read.
pub const CARD: &[u8] = b"card bytes for the C0 probe";
/// A resource whose reply exceeds the payload cap.
pub const HUGE_LEN: usize = 2 * payload::MAX_PAYLOAD;

pub fn card_address() -> ContentHash {
    ContentHash::of(CARD)
}

pub fn huge_address() -> ContentHash {
    ContentHash::of(&huge())
}

fn huge() -> Vec<u8> {
    vec![0xa5; HUGE_LEN]
}

struct Session {
    id: ProjectionSession,
    notices: VecDeque<Notice>,
}

pub struct TestProvider {
    next: u64,
    sessions: BTreeMap<u64, Session>,
    /// Session calls left before the authority is revoked.
    calls_left: usize,
    pub closed: Vec<u64>,
}

impl TestProvider {
    pub fn new(calls_before_revocation: usize) -> Self {
        Self {
            next: 1,
            sessions: BTreeMap::new(),
            calls_left: calls_before_revocation,
            closed: Vec::new(),
        }
    }

    fn descriptor(id: &ProjectionSession) -> EndpointDescriptor {
        EndpointDescriptor {
            label: "c0-test".into(),
            projections: vec![ProjectionOffer {
                label: "spiral".into(),
                request: ProjectionRequest {
                    version: ProtocolVersion::V1,
                    session: id.clone(),
                    score: Score::new(Arrangement::Spiral(Spiral::default())),
                },
            }],
        }
    }

    /// The recheck every session call makes before disclosing anything.
    fn admit(&mut self, session: u64) -> Result<&mut Session, Failure> {
        if self.calls_left == 0 {
            return Err(Failure::Revoked);
        }
        self.calls_left -= 1;
        self.sessions
            .get_mut(&session)
            .ok_or_else(|| Failure::Refused("no such session".into()))
    }
}

impl Provider for TestProvider {
    fn discover(&mut self) -> Reply {
        payload::encode(&Self::descriptor(&ProjectionSession("c0:unopened".into())))
    }

    fn open(&mut self, request: &[u8]) -> Result<(u64, Vec<u8>), Failure> {
        let open: SessionOpen = payload::decode(request)?;
        if open.capabilities == CapabilityProfile::new([]) {
            return Err(Failure::Refused("no presentation capability".into()));
        }
        let handle = self.next;
        self.next += 1;
        // The id stands in for the admission transcript's digest.
        let id = ProjectionSession(format!("c0:admitted:{handle}"));
        let opened = SessionOpened {
            version: ProtocolVersion::V1,
            descriptor: Self::descriptor(&id),
            status: SessionStatus::Live,
            expires_at_ms: None,
        };
        self.sessions.insert(
            handle,
            Session {
                id,
                notices: VecDeque::new(),
            },
        );
        Ok((handle, payload::encode(&opened)?))
    }

    fn snapshot(&mut self, session: u64, ask: &[u8]) -> Reply {
        let id = self.admit(session)?.id.clone();
        let ask: ProjectionAsk = payload::decode(ask)?;
        let snapshot = ProjectionSnapshot {
            version: ask.version,
            session: id,
            scene: SceneSnapshot::from_dense(SceneEpoch(1), Revision(1), Scene::new())
                .map_err(|e| Failure::Refused(format!("{e:?}")))?,
            presentation: PresentationManifest::default(),
            cache_policy: CachePolicy::default(),
        };
        payload::encode(&snapshot)
    }

    fn fetch(&mut self, session: u64, address: &[u8]) -> Reply {
        let id = self.admit(session)?.id.clone();
        let address: ContentHash = payload::decode(address)?;
        let bytes = if address == card_address() {
            CARD.to_vec()
        } else if address == huge_address() {
            huge()
        } else {
            return Err(Failure::Refused("unknown resource".into()));
        };
        payload::encode(&ResourceResponse::new(id, bytes))
    }

    fn chunk(&mut self, session: u64, address: &[u8], offset: u64, length: u32) -> Reply {
        let id = self.admit(session)?.id.clone();
        let address: ContentHash = payload::decode(address)?;
        if address != card_address() {
            return Err(Failure::Refused("unknown resource".into()));
        }
        let whole = ResourceResponse::new(id, CARD.to_vec());
        payload::encode(&ResourceChunkResponse::slice(&whole, offset, length))
    }

    fn resume(&mut self, session: u64, epoch: u64, revision: u64) -> Reply {
        let id = self.admit(session)?.id.clone();
        payload::encode(&ResumeReply::Current(ProjectionAck {
            session: id,
            epoch: SceneEpoch(epoch),
            revision: Revision(revision),
        }))
    }

    fn intent(&mut self, session: u64, call: &[u8]) -> Reply {
        let state = self.admit(session)?;
        let call: IntentCall = payload::decode(call)?;
        let result = if call.observed_revision == Revision(1) {
            state.notices.push_back(Notice {
                epoch: 1,
                revision: 2,
            });
            IntentResult::Accepted
        } else {
            IntentResult::Stale {
                current_epoch: SceneEpoch(1),
                current_revision: Revision(2),
            }
        };
        payload::encode(&result)
    }

    fn suspend(&mut self, session: u64) -> Result<(), Failure> {
        self.admit(session).map(|_| ())
    }

    fn take_notice(&mut self, session: u64) -> Option<Notice> {
        self.admit(session).ok()?.notices.pop_front()
    }

    fn wait_notice(&mut self, session: u64) -> Result<Notice, Failure> {
        // A test provider never blocks: no queued notice is a refusal.
        self.admit(session)?
            .notices
            .pop_front()
            .ok_or_else(|| Failure::Refused("no notice".into()))
    }

    fn close(&mut self, session: u64) {
        self.sessions.remove(&session);
        self.closed.push(session);
    }
}
