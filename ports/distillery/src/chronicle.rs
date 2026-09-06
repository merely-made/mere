// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! A read-only Chronicle projection of a Distillery job board.
//!
//! The caller supplies an admitted session and explicit input generation,
//! scene epoch, and revision. Chronicle reads that observation, serves cards,
//! and retains one honest contiguous transition. It never owns the board,
//! starts a resident, or changes a job.

use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex, MutexGuard},
};

use chirograph::{
    BoundsRelationship, CachePolicy, CardValueV1, CarrierNotice, ContentHash, EndpointDescriptor,
    IntentInvocation, IntentResult, PresentationBinding, PresentationCapability,
    PresentationChange, PresentationCodec, PresentationKey, PresentationManifest,
    PresentationOffer, PresentationSemantics, ProjectionAck, ProjectionDiff, ProjectionOffer,
    ProjectionRequest, ProjectionSession, ProjectionSnapshot, ProtocolVersion, ResourceRequest,
    ResourceResponse, ResumeReply, ResumeRequest, SemanticRole, SessionStatus,
};
use graphshell_endpoint::{
    IntentSink, PresentationSource, ProjectionCatalog, ProjectionNoticeSource, ProjectionSource,
    ResumableProjectionSource,
};
use mesh::{Job, JobBoard, JobState};
use sceno::{
    Arrangement, AxisValue, Footprint, InstanceId, Placement, Representation, Scene, Score,
    ScoreItem, Size2, SourceRef, Timeline,
};
use scenotime::{Revision, SceneDiff, SceneEpoch, SceneOp, SceneSnapshot};

use crate::ResidentReceipt;

const DEFAULT_SESSION: &str = "distillery.chronicle/v1";
const SOURCE_NAMESPACE: &str = "distillery.chronicle";

/// A caller-owned materialization version.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChronicleRevision {
    /// The source generation used to construct the score and scene.
    pub generation: u64,
    /// Stable scene-index lifetime.
    pub epoch: SceneEpoch,
    /// Monotonic scene-delivery revision inside [`Self::epoch`].
    pub revision: Revision,
}

impl ChronicleRevision {
    /// Build an explicit source generation, epoch, and revision.
    pub const fn new(generation: u64, epoch: u64, revision: u64) -> Self {
        Self {
            generation,
            epoch: SceneEpoch(epoch),
            revision: Revision(revision),
        }
    }
}

/// Why Chronicle could not answer a request.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChronicleEndpointError {
    /// The caller named another projection session.
    WrongSession,
    /// The caller selected a score Chronicle did not offer.
    UnsupportedScore,
    /// The caller requested a resource absent from this observation.
    NoSuchResource,
    /// The caller tried to replace the observation non-monotonically.
    InvalidRevision,
}

impl std::fmt::Display for ChronicleEndpointError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::WrongSession => write!(formatter, "request names another session"),
            Self::UnsupportedScore => write!(formatter, "request names another projection score"),
            Self::NoSuchResource => write!(formatter, "no such Chronicle presentation resource"),
            Self::InvalidRevision => {
                write!(formatter, "observation did not advance Chronicle revision")
            },
        }
    }
}

impl std::error::Error for ChronicleEndpointError {}

/// The shared Distillery observation behind its admitted Chronicle endpoints.
///
/// A resident owns this observer for the lifetime of its board and receipt
/// stream. Carrier reconnects create fresh [`ChronicleEndpoint`] values from
/// it, retaining the current snapshot and one honest contiguous diff without
/// sharing a caller's admitted session.
#[derive(Clone)]
pub struct ChronicleObserver(Arc<Mutex<ChronicleObservation>>);

impl ChronicleObserver {
    /// Read the current board and receipt stream into a new shared observation.
    pub fn new(board: &JobBoard, receipts: &[ResidentReceipt], version: ChronicleRevision) -> Self {
        Self(Arc::new(Mutex::new(
            ChronicleObservation::from_materialization(
                materialize(board, receipts, version),
                version,
            ),
        )))
    }

    /// Bind a newly admitted endpoint to this retained observation.
    ///
    /// The session remains endpoint-local. It is never retained in the shared
    /// observation or copied from an earlier carrier connection.
    pub fn endpoint(&self, session: ProjectionSession) -> ChronicleEndpoint {
        ChronicleEndpoint {
            observer: self.clone(),
            session,
            noticed_generation: 0,
        }
    }

    /// Replace the observed board and receipts with a later same-epoch state.
    ///
    /// When the scene's stable tables remain unchanged, this retains exactly
    /// one replayable diff. A topology change intentionally discards that
    /// history so reconnecting carriers receive a snapshot instead.
    pub fn observe(
        &self,
        board: &JobBoard,
        receipts: &[ResidentReceipt],
        version: ChronicleRevision,
    ) -> Result<(), ChronicleEndpointError> {
        let mut current = self.state();
        if version.epoch != current.version.epoch || version.revision <= current.version.revision {
            return Err(ChronicleEndpointError::InvalidRevision);
        }
        let next = materialize(board, receipts, version);
        let history = scene_diff_between(&current, &next, version).map(|scene| RetainedDiff {
            scene,
            presentation: presentation_changes(&current.presentation, &next.presentation),
        });
        let notice_generation = current
            .notice_generation
            .checked_add(1)
            .expect("Chronicle observation notices cannot wrap");
        *current = ChronicleObservation::from_materialization(next, version);
        current.history = history;
        current.notice_generation = notice_generation;
        Ok(())
    }

    /// The latest product-owned materialization version.
    pub fn revision(&self) -> ChronicleRevision {
        self.state().version
    }

    fn state(&self) -> MutexGuard<'_, ChronicleObservation> {
        self.0
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

/// One session-bound view of a shared Chronicle observation.
pub struct ChronicleEndpoint {
    observer: ChronicleObserver,
    session: ProjectionSession,
    noticed_generation: u64,
}

impl ChronicleEndpoint {
    /// Read a board and receipt stream into the caller's admitted session.
    pub fn new(
        session: ProjectionSession,
        board: &JobBoard,
        receipts: &[ResidentReceipt],
        version: ChronicleRevision,
    ) -> Self {
        ChronicleObserver::new(board, receipts, version).endpoint(session)
    }

    /// Read an unauthenticated local observation under Distillery's default id.
    ///
    /// A carrier host should bind its negotiated session with [`Self::new`].
    pub fn with_default_session(
        board: &JobBoard,
        receipts: &[ResidentReceipt],
        version: ChronicleRevision,
    ) -> Self {
        Self::new(
            ProjectionSession(DEFAULT_SESSION.to_string()),
            board,
            receipts,
            version,
        )
    }

    /// The offered request for this session.
    pub fn request(&self) -> ProjectionRequest {
        ProjectionRequest {
            version: ProtocolVersion::V1,
            session: self.session.clone(),
            score: self.observer.state().score.clone(),
        }
    }

    /// The caller-owned state version currently being served.
    pub fn revision(&self) -> ChronicleRevision {
        self.observer.revision()
    }

    /// The product-free timeline score supplied to the scene solver.
    pub fn score(&self) -> Score {
        self.observer.state().score.clone()
    }

    /// Replace this source's observation with a strictly later same-epoch one.
    ///
    /// When job identities retain their stable scene slots, the endpoint keeps
    /// one contiguous diff. A changed topology returns a snapshot on resume:
    /// a source may not relabel old item slots merely to manufacture a replay.
    pub fn observe(
        &mut self,
        board: &JobBoard,
        receipts: &[ResidentReceipt],
        version: ChronicleRevision,
    ) -> Result<(), ChronicleEndpointError> {
        self.observer.observe(board, receipts, version)
    }

    fn snapshot_for_current_score(&self) -> ProjectionSnapshot {
        let state = self.observer.state();
        ProjectionSnapshot {
            version: ProtocolVersion::V1,
            session: self.session.clone(),
            scene: SceneSnapshot::from_dense(
                state.version.epoch,
                state.version.revision,
                state.scene.clone(),
            )
            .expect("Chronicle realizes a valid dense scene"),
            presentation: state.presentation.clone(),
            cache_policy: CachePolicy::default(),
        }
    }
}

impl ProjectionCatalog for ChronicleEndpoint {
    fn describe(&self) -> EndpointDescriptor {
        EndpointDescriptor {
            label: "Distillery Chronicle".to_string(),
            projections: vec![ProjectionOffer {
                label: "Job Chronicle".to_string(),
                request: self.request(),
            }],
        }
    }
}

impl ProjectionSource for ChronicleEndpoint {
    type Error = ChronicleEndpointError;

    fn snapshot(&mut self, request: ProjectionRequest) -> Result<ProjectionSnapshot, Self::Error> {
        if request.session != self.session {
            return Err(ChronicleEndpointError::WrongSession);
        }
        if request.score != self.observer.state().score {
            return Err(ChronicleEndpointError::UnsupportedScore);
        }
        Ok(self.snapshot_for_current_score())
    }
}

impl PresentationSource for ChronicleEndpoint {
    type Error = ChronicleEndpointError;

    fn resource(&mut self, request: ResourceRequest) -> Result<ResourceResponse, Self::Error> {
        if request.session != self.session {
            return Err(ChronicleEndpointError::WrongSession);
        }
        let bytes = self
            .observer
            .state()
            .resources
            .get(&request.resource)
            .ok_or(ChronicleEndpointError::NoSuchResource)?
            .clone();
        Ok(ResourceResponse {
            session: self.session.clone(),
            resource: request.resource,
            bytes,
        })
    }
}

impl ResumableProjectionSource for ChronicleEndpoint {
    type Error = ChronicleEndpointError;

    fn resume(&mut self, request: ResumeRequest) -> Result<ResumeReply, Self::Error> {
        if request.session != self.session {
            return Err(ChronicleEndpointError::WrongSession);
        }
        let state = self.observer.state();
        if request.epoch == state.version.epoch && request.revision == state.version.revision {
            return Ok(ResumeReply::Current(ProjectionAck {
                session: self.session.clone(),
                epoch: state.version.epoch,
                revision: state.version.revision,
            }));
        }
        if let Some(diff) = &state.history {
            if request.epoch == diff.scene.epoch && request.revision == diff.scene.base {
                return Ok(ResumeReply::Diffs(vec![ProjectionDiff {
                    version: ProtocolVersion::V1,
                    session: self.session.clone(),
                    scene: diff.scene.clone(),
                    presentation: diff.presentation.clone(),
                    status: Some(SessionStatus::Live),
                }]));
            }
        }
        drop(state);
        Ok(ResumeReply::Snapshot(Box::new(
            self.snapshot_for_current_score(),
        )))
    }
}

impl ProjectionNoticeSource for ChronicleEndpoint {
    type Error = ChronicleEndpointError;

    fn poll_notice(&mut self) -> Result<Option<CarrierNotice>, Self::Error> {
        let state = self.observer.state();
        if state.notice_generation == self.noticed_generation {
            return Ok(None);
        }
        self.noticed_generation = state.notice_generation;
        Ok(Some(CarrierNotice {
            session: self.session.clone(),
            epoch: state.version.epoch,
            revision: state.version.revision,
        }))
    }
}

impl IntentSink for ChronicleEndpoint {
    type Error = ChronicleEndpointError;

    fn invoke(&mut self, intent: IntentInvocation) -> Result<IntentResult, Self::Error> {
        if intent.session != self.session {
            return Err(ChronicleEndpointError::WrongSession);
        }
        let version = self.observer.revision();
        if intent.observed_epoch != version.epoch || intent.observed_revision != version.revision {
            return Ok(IntentResult::Stale {
                current_epoch: version.epoch,
                current_revision: version.revision,
            });
        }
        Ok(IntentResult::Rejected {
            reason: "Distillery Chronicle v1 does not offer intents".to_string(),
        })
    }
}

struct ChronicleObservation {
    version: ChronicleRevision,
    score: Score,
    scene: Scene,
    presentation: PresentationManifest,
    resources: BTreeMap<ContentHash, Vec<u8>>,
    history: Option<RetainedDiff>,
    notice_generation: u64,
}

impl ChronicleObservation {
    fn from_materialization(materialization: Materialization, version: ChronicleRevision) -> Self {
        Self {
            version,
            score: materialization.score,
            scene: materialization.scene,
            presentation: materialization.presentation,
            resources: materialization.resources,
            history: None,
            notice_generation: 0,
        }
    }
}

struct RetainedDiff {
    scene: SceneDiff,
    presentation: Vec<PresentationChange>,
}

struct Materialization {
    score: Score,
    scene: Scene,
    presentation: PresentationManifest,
    resources: BTreeMap<ContentHash, Vec<u8>>,
}

fn materialize(
    board: &JobBoard,
    receipts: &[ResidentReceipt],
    version: ChronicleRevision,
) -> Materialization {
    let tick = u64::try_from(receipts.len()).unwrap_or(u64::MAX);
    let jobs: Vec<&Job> = board.jobs().collect();
    let score = score_for(&jobs, tick, version.generation);
    let scene = scenomise::solve(&score);
    let (presentation, resources) = presentation_for(&jobs, tick);
    Materialization {
        score,
        scene,
        presentation,
        resources,
    }
}

fn score_for(jobs: &[&Job], tick: u64, generation: u64) -> Score {
    let mut score = Score::new(Arrangement::Timeline(Timeline::default()));
    score.generation = generation;
    score.items = jobs
        .iter()
        .enumerate()
        .map(|(ordinal, job)| ScoreItem {
            source: source_for(job),
            ordinal: u32::try_from(ordinal).expect("a scene cannot index beyond u32"),
            footprint: card_footprint(),
            representation: Representation::Card,
            placement: Placement::Ordinal,
            layer: 0,
            visible: true,
            axis: Some(AxisValue::Numeric(tick as f64)),
            embedding: None,
            weight: None,
        })
        .collect();
    score
}

fn presentation_for(
    jobs: &[&Job],
    tick: u64,
) -> (PresentationManifest, BTreeMap<ContentHash, Vec<u8>>) {
    let mut presentation = PresentationManifest::default();
    let mut resources = BTreeMap::new();
    for (ordinal, job) in jobs.iter().enumerate() {
        let card = card_for(job, tick);
        let bytes = serde_json::to_vec(&card).expect("PortableCardV1 always serializes");
        let resource = ContentHash::of(&bytes);
        let key = PresentationKey(format!("distillery:chronicle:job:{}", job_id(job)));
        presentation.bindings.push(PresentationBinding {
            instance: InstanceId(u32::try_from(ordinal).expect("a scene cannot index beyond u32")),
            key: key.clone(),
        });
        presentation.offers.insert(
            key,
            vec![PresentationOffer {
                codec: PresentationCodec::PortableCardV1,
                resource,
                byte_size: bytes.len() as u64,
                requires: PresentationCapability::PortableCard,
                semantics: PresentationSemantics {
                    label: card.title,
                    role: SemanticRole::Article,
                    bounds: BoundsRelationship::FitWithinFootprint,
                    actions: Vec::new(),
                },
            }],
        );
        resources.insert(resource, bytes);
    }
    (presentation, resources)
}

fn source_for(job: &Job) -> SourceRef {
    SourceRef::new(SOURCE_NAMESPACE, format!("job:{}", job_id(job)))
}

fn card_footprint() -> Footprint {
    Footprint::Rect {
        size: Size2::new(280.0, 156.0),
    }
}

fn card_for(job: &Job, tick: u64) -> chirograph::PortableCardV1 {
    let current_epoch = job.lease.current();
    let mut values = vec![
        card_value("Observation tick", tick),
        card_value(
            "Lease epoch",
            current_epoch.map_or_else(|| "none".to_string(), |epoch| epoch.epoch.to_string()),
        ),
        card_value("Poster", key_id(&job.posted_by)),
        card_value("Next claimants", job.next_claimants.len()),
    ];
    if let Some(epoch) = current_epoch {
        values.push(card_value(
            "Lease window",
            format!("{}..{}", epoch.granted_at_ms, epoch.boundary_ms()),
        ));
    }
    if let Some(winner) = job.state.winner() {
        values.push(card_value("Winner", key_id(&winner)));
    }
    chirograph::PortableCardV1 {
        title: format!("Job {}", job_id(job)),
        values,
        badges: vec![state_label(&job.state).to_string()],
        media: Vec::new(),
    }
}

fn card_value(label: impl Into<String>, value: impl ToString) -> CardValueV1 {
    CardValueV1 {
        label: label.into(),
        value: value.to_string(),
    }
}

fn state_label(state: &JobState) -> &'static str {
    match state {
        JobState::Posted => "posted",
        JobState::Claimed { .. } => "claimed",
        JobState::Done { .. } => "done",
        JobState::Committed { .. } => "committed",
    }
}

fn job_id(job: &Job) -> String {
    key_id(&job.id.0)
}

fn key_id(bytes: &[u8; 32]) -> String {
    let mut output = String::with_capacity(64);
    for byte in bytes {
        use std::fmt::Write as _;
        write!(&mut output, "{byte:02x}").expect("writing to String cannot fail");
    }
    output
}

fn scene_diff_between(
    current: &ChronicleObservation,
    next: &Materialization,
    version: ChronicleRevision,
) -> Option<SceneDiff> {
    if current.scene.sources != next.scene.sources
        || current.scene.spaces != next.scene.spaces
        || current.scene.backdrops != next.scene.backdrops
        || current.scene.relations != next.scene.relations
        || current.scene.regions != next.scene.regions
        || current.scene.items.len() != next.scene.items.len()
        || current.scene.unmet_holds != next.scene.unmet_holds
        || current.scene.honored_holds != next.scene.honored_holds
        || current.presentation.bindings != next.presentation.bindings
    {
        return None;
    }
    let mut operations = Vec::new();
    for (index, (before, after)) in current
        .scene
        .items
        .iter()
        .zip(&next.scene.items)
        .enumerate()
    {
        if before != after {
            operations.push(SceneOp::UpdateItem {
                index: InstanceId(u32::try_from(index).expect("a scene cannot index beyond u32")),
                value: after.clone(),
            });
        }
    }
    if current.scene.bounds != next.scene.bounds {
        operations.push(SceneOp::SetBounds {
            bounds: next.scene.bounds,
        });
    }
    if current.scene.generation != next.scene.generation {
        operations.push(SceneOp::SetGeneration {
            generation: next.scene.generation,
        });
    }
    Some(SceneDiff {
        epoch: version.epoch,
        base: current.version.revision,
        revision: version.revision,
        operations,
    })
}

fn presentation_changes(
    current: &PresentationManifest,
    next: &PresentationManifest,
) -> Vec<PresentationChange> {
    let mut changes = Vec::new();
    for (key, offers) in &next.offers {
        if current.offers.get(key) != Some(offers) {
            changes.push(PresentationChange::ReplaceOffers {
                key: key.clone(),
                offers: offers.clone(),
            });
        }
    }
    for (key, offers) in &current.offers {
        if !next.offers.contains_key(key) {
            changes.push(PresentationChange::RemoveOffers { key: key.clone() });
        }
        for resource in offers.iter().map(|offer| offer.resource) {
            if !next
                .offers
                .values()
                .flatten()
                .any(|offer| offer.resource == resource)
            {
                changes.push(PresentationChange::InvalidateResource { resource });
            }
        }
    }
    changes
}

#[cfg(test)]
mod tests {
    use super::*;
    use graphshell_endpoint::{
        IntentSink, PresentationSource, ProjectionCatalog, ProjectionNoticeSource,
        ProjectionSource, ResumableProjectionSource,
    };
    use mesh::JobBoardSnapshot;
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct ChronicleFixture {
        jobs: Vec<Job>,
    }

    fn board() -> JobBoard {
        let fixture: ChronicleFixture = serde_json::from_str(include_str!(
            "../tests/fixtures/chronicle/distillery_board.json"
        ))
        .expect("W0 fixture parses");
        JobBoard::fold_from_snapshot(
            [0; 32],
            &JobBoardSnapshot { jobs: fixture.jobs },
            std::iter::empty(),
        )
    }

    fn endpoint() -> ChronicleEndpoint {
        ChronicleEndpoint::new(
            ProjectionSession("admitted:distillery-1".into()),
            &board(),
            &[
                ResidentReceipt::MaintenanceIdle,
                ResidentReceipt::StopRequested,
            ],
            ChronicleRevision::new(41, 7, 11),
        )
    }

    #[test]
    fn chronology_uses_the_timeline_solver_and_serves_verified_portable_cards() {
        let mut endpoint = endpoint();
        let descriptor = endpoint.describe();
        assert_eq!(descriptor.projections.len(), 1);
        assert!(matches!(
            descriptor.projections[0].request.score.arrangement,
            Arrangement::Timeline(_)
        ));
        assert_eq!(descriptor.projections[0].request.score.generation, 41);
        assert!(
            descriptor.projections[0]
                .request
                .score
                .items
                .iter()
                .all(|item| item.axis == Some(AxisValue::Numeric(2.0)))
        );

        let snapshot = endpoint.snapshot(endpoint.request()).expect("snapshot");
        assert_eq!(snapshot.scene.epoch, SceneEpoch(7));
        assert_eq!(snapshot.scene.revision, Revision(11));
        assert_eq!(snapshot.scene.tables.generation, 41);
        assert_eq!(snapshot.presentation.bindings.len(), 3);
        let offer = snapshot
            .presentation
            .offers
            .values()
            .next()
            .unwrap()
            .first()
            .unwrap();
        assert_eq!(offer.codec, PresentationCodec::PortableCardV1);
        assert!(offer.semantics.actions.is_empty());
        let response = endpoint
            .resource(ResourceRequest {
                session: endpoint.request().session,
                resource: offer.resource,
            })
            .expect("card resource");
        assert!(response.has_valid_address());
        assert_eq!(ContentHash::of(&response.bytes), offer.resource);
        assert_eq!(response.bytes.len() as u64, offer.byte_size);
        let card: chirograph::PortableCardV1 = serde_json::from_slice(&response.bytes).unwrap();
        assert!(
            card.values
                .iter()
                .any(|value| value.label == "Observation tick")
        );
        assert!(card.values.iter().any(|value| value.label == "Lease epoch"));
    }

    #[test]
    fn identical_inputs_are_byte_identical_and_wrong_selection_is_refused() {
        let mut first = endpoint();
        let mut second = endpoint();
        assert_eq!(
            serde_json::to_vec(&first.snapshot(first.request()).unwrap()).unwrap(),
            serde_json::to_vec(&second.snapshot(second.request()).unwrap()).unwrap()
        );
        let mut wrong_score = first.request();
        wrong_score.score.generation += 1;
        assert_eq!(
            first.snapshot(wrong_score),
            Err(ChronicleEndpointError::UnsupportedScore)
        );
        let mut wrong_session = first.request();
        wrong_session.session = ProjectionSession("admitted:other".into());
        assert_eq!(
            first.snapshot(wrong_session),
            Err(ChronicleEndpointError::WrongSession)
        );
    }

    #[test]
    fn one_later_observation_notifies_replays_a_diff_and_refuses_intents() {
        let mut endpoint = endpoint();
        let session = endpoint.request().session;
        endpoint
            .observe(
                &board(),
                &[
                    ResidentReceipt::MaintenanceIdle,
                    ResidentReceipt::StopRequested,
                    ResidentReceipt::MaintenanceIdle,
                ],
                ChronicleRevision::new(42, 7, 12),
            )
            .unwrap();
        assert_eq!(endpoint.score().generation, 42);
        assert_eq!(
            endpoint.poll_notice().unwrap(),
            Some(CarrierNotice {
                session: session.clone(),
                epoch: SceneEpoch(7),
                revision: Revision(12),
            })
        );
        assert!(endpoint.poll_notice().unwrap().is_none());
        assert!(matches!(
            endpoint
                .resume(ResumeRequest {
                    session: session.clone(),
                    epoch: SceneEpoch(7),
                    revision: Revision(11),
                })
                .unwrap(),
            ResumeReply::Diffs(diffs) if diffs.len() == 1 && diffs[0].scene.base == Revision(11)
        ));
        assert_eq!(
            endpoint
                .invoke(IntentInvocation {
                    session,
                    target: InstanceId(0),
                    observed_epoch: SceneEpoch(7),
                    observed_revision: Revision(12),
                    intent: "distillery.reclaim".into(),
                    payload: Vec::new(),
                })
                .unwrap(),
            IntentResult::Rejected {
                reason: "Distillery Chronicle v1 does not offer intents".into()
            }
        );
    }

    #[test]
    fn shared_observer_keeps_snapshot_and_diff_for_a_reconnected_endpoint() {
        let observer = ChronicleObserver::new(
            &board(),
            &[
                ResidentReceipt::MaintenanceIdle,
                ResidentReceipt::StopRequested,
            ],
            ChronicleRevision::new(41, 7, 11),
        );
        let session = ProjectionSession("admitted:reconnect".into());
        let mut mounted = observer.endpoint(session.clone());
        let mounted_snapshot = mounted.snapshot(mounted.request()).expect("snapshot");
        assert_eq!(mounted_snapshot.scene.revision, Revision(11));
        assert!(
            mounted.poll_notice().expect("initial poll").is_none(),
            "an unadvanced observer must not ring a newly admitted endpoint"
        );

        observer
            .observe(
                &board(),
                &[
                    ResidentReceipt::MaintenanceIdle,
                    ResidentReceipt::StopRequested,
                    ResidentReceipt::MaintenanceIdle,
                ],
                ChronicleRevision::new(42, 7, 12),
            )
            .expect("later observation");

        // This is a fresh endpoint value, as a carrier factory makes after a
        // reconnect. Its admitted session is local, while its state is the
        // resident-owned observer that survived the first value.
        let mut reconnected = observer.endpoint(session.clone());
        assert_eq!(
            reconnected.poll_notice().expect("reconnect bell"),
            Some(CarrierNotice {
                session: session.clone(),
                epoch: SceneEpoch(7),
                revision: Revision(12),
            })
        );
        assert!(
            reconnected
                .poll_notice()
                .expect("one reconnect bell")
                .is_none(),
            "one retained observation rings once per endpoint"
        );
        let resumed = reconnected
            .resume(ResumeRequest {
                session: session.clone(),
                epoch: SceneEpoch(7),
                revision: Revision(11),
            })
            .expect("shared contiguous diff");
        assert!(matches!(
            resumed,
            ResumeReply::Diffs(diffs)
                if diffs.len() == 1
                    && diffs[0].session == session
                    && diffs[0].scene.base == Revision(11)
                    && diffs[0].scene.revision == Revision(12)
        ));
        let current = reconnected
            .snapshot(reconnected.request())
            .expect("retained current snapshot");
        assert_eq!(current.scene.revision, Revision(12));
        assert_eq!(current.scene.tables.generation, 42);

        let mut other = observer.endpoint(ProjectionSession("admitted:other".into()));
        assert_eq!(
            other.resume(ResumeRequest {
                session,
                epoch: SceneEpoch(7),
                revision: Revision(11),
            }),
            Err(ChronicleEndpointError::WrongSession)
        );
    }
}
