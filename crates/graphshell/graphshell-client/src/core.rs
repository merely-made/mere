// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The Graphshell session protocol with the I/O taken out.
//!
//! [`SessionCore`] holds everything [`crate::session::RetainedEndpointSession`]
//! held except the carrier, and answers one question: *given where this session
//! is, what should be asked of the endpoint next, and what does the answer
//! mean?* It never sends anything. A caller drives it by carrying
//! [`Progress::Ask`] to wherever the endpoint is and handing the reply back
//! through [`SessionCore::on_response`].
//!
//! ## Why the protocol had to stop owning its I/O
//!
//! [`chirograph::Carrier`] is blocking, and its own doc defers the question to
//! whoever writes a network carrier. The browser is that moment, and it cannot
//! answer it the way native did: blocking a browser task deadlocks against the
//! very data-channel callback that would deliver the bytes being waited on, and
//! moving the block into a worker only moves the deadlock — a worker parked in
//! `request` has stopped the event loop that delivers the message it is parked
//! on. Escaping that needs `Atomics.wait` on a `SharedArrayBuffer`, which needs
//! cross-origin isolation, which would then be imposed on anyone embedding
//! Graphshell.
//!
//! So the protocol stops owning its I/O instead. This is the shape
//! `notochord`'s handshake and the WebRTC carrier core already have, and the
//! reason str0m was chosen at C1: the sequencing is written once, and how bytes
//! move is somebody else's decision. `RetainedEndpointSession` is the blocking
//! adapter over this core and keeps its public API exactly; the browser writes
//! an event-driven one. Two adapters, one protocol.
//!
//! ## Multi-step operations
//!
//! Three operations are not one round trip, and they are why this is a state
//! machine rather than a pair of functions. Resolving a presentation may need a
//! resource first, and another after that. A resume may be answered with a
//! resynchronize that asks again from a new position, up to
//! [`RESUME_ATTEMPTS`] times. Polling drains however many notices are queued.
//! Each returns [`Progress::Ask`] again rather than looping over a carrier it
//! does not have.

use std::collections::{BTreeMap, BTreeSet};

use chirograph::{
    AdvertisedAction, CapabilityProfile, CarrierNotice, CarrierRequestBody, CarrierResponseBody,
    EndpointDescriptor, IntentInvocation, IntentResult, ProjectionRequest, ProjectionSession,
    ResourceRequest, ResourceResponse,
};
use sceno::InstanceId;
use serde::Serialize;

use crate::action_draft::{ActionDraft, ActionDraftTarget};
use crate::session::unexpected;
use crate::{ClientState, PresentationResolution, ResolvedPresentation, ResumeApplication};

/// How many times a resume may be answered with "resynchronize" before the
/// endpoint is treated as unable to produce an applicable one.
pub const RESUME_ATTEMPTS: u8 = 4;

/// What the core wants next.
#[must_use = "a core that asked for something is waiting to be driven"]
#[derive(Debug)]
pub enum Progress<T> {
    /// Put this to the endpoint and return the answer to
    /// [`SessionCore::on_response`].
    Ask(CarrierRequestBody),
    /// The operation is finished and this is its result.
    Done(T),
}

impl<T> Progress<T> {
    /// The request, when this step wants one sent.
    pub fn request(&self) -> Option<&CarrierRequestBody> {
        match self {
            Progress::Ask(body) => Some(body),
            Progress::Done(_) => None,
        }
    }

    /// The result, when the operation is finished.
    pub fn done(self) -> Option<T> {
        match self {
            Progress::Ask(_) => None,
            Progress::Done(value) => Some(value),
        }
    }
}

/// What an operation produced when it finished.
///
/// One enum rather than a generic, because a driver holds an in-flight
/// operation without knowing which one it is.
#[derive(Debug)]
pub enum Outcome {
    /// Discovery answered with the endpoint's descriptor.
    Descriptor(Box<EndpointDescriptor>),
    /// A projection is mounted under this session.
    Mounted(ProjectionSession),
    /// A snapshot refreshed a session already mounted.
    Resnapshotted,
    /// A presentation resolved, with every resource it needed in hand.
    Resolved(Box<ResolvedPresentation>),
    /// An opaque side resource explicitly named by a resolved portable card.
    /// Its bytes are checked but never entered into the presentation cache.
    MediaResource(Box<ResourceResponse>),
    /// The endpoint answered an intent — admitted or refused, both are this.
    Intent(Box<IntentResult>),
    /// Whether anything actually changed.
    Changed(bool),
    /// The session is closed.
    Closed,
}

/// The operation in flight, and where it has got to.
enum Pending {
    /// A discovery on a live session: the descriptor is refreshed, nothing
    /// mounted is touched.
    Rediscover,
    Mount {
        request: ProjectionRequest,
    },
    Resnapshot {
        session: ProjectionSession,
        request: ProjectionRequest,
    },
    Resolve {
        session: ProjectionSession,
        instance: InstanceId,
        request: ResourceRequest,
    },
    MediaResource {
        request: ResourceRequest,
    },
    Intent,
    /// A resume driven by one notice, counting its resynchronize attempts.
    Resume {
        notice: CarrierNotice,
        attempts: u8,
    },
    /// The discovery round trip that lets a carrier collect queued notices.
    PollDiscover,
    Close,
}

/// One endpoint's protocol state, with no way to reach it.
pub struct SessionCore {
    client: ClientState,
    profile: CapabilityProfile,
    descriptor: EndpointDescriptor,
    mounted: BTreeSet<ProjectionSession>,
    requests: BTreeMap<ProjectionSession, ProjectionRequest>,
    pending: Option<Pending>,
}

impl SessionCore {
    /// The request that starts discovery, before any core exists.
    ///
    /// Mounting an endpoint begins with a descriptor, and the descriptor is
    /// what a core needs in order to exist — so this is an associated function
    /// rather than a method. An adapter sends it, then builds the core from the
    /// answer with [`SessionCore::from_descriptor`].
    pub fn discover_request() -> CarrierRequestBody {
        CarrierRequestBody::Discover
    }

    /// Build a core from the descriptor discovery returned.
    pub fn from_descriptor(descriptor: EndpointDescriptor, profile: CapabilityProfile) -> Self {
        Self {
            client: ClientState::default(),
            profile,
            descriptor,
            mounted: BTreeSet::new(),
            requests: BTreeMap::new(),
            pending: None,
        }
    }

    pub fn descriptor(&self) -> &EndpointDescriptor {
        &self.descriptor
    }

    pub fn client(&self) -> &ClientState {
        &self.client
    }

    pub fn client_mut(&mut self) -> &mut ClientState {
        &mut self.client
    }

    pub fn profile(&self) -> &CapabilityProfile {
        &self.profile
    }

    /// Whether an operation is waiting on an answer.
    pub fn is_awaiting(&self) -> bool {
        self.pending.is_some()
    }

    /// The sessions currently mounted.
    pub fn mounted_sessions(&self) -> impl Iterator<Item = &ProjectionSession> {
        self.mounted.iter()
    }

    /// Forget one mounted projection and every resource cached beneath it.
    pub fn forget(&mut self, session: &ProjectionSession) {
        self.mounted.remove(session);
        self.requests.remove(session);
        self.client.forget_session(session);
    }

    /// Every mounted projection stops being live.
    ///
    /// The scene is kept rather than dropped: a host still wants to show what
    /// was there, it just must not offer to save into it.
    pub fn disconnect(&mut self) {
        self.pending = None;
        for session in self.mounted.iter() {
            self.client.mark_disconnected(session);
        }
    }

    /// Drop every mounted scene and its cached bytes.
    pub fn purge(&mut self) {
        let sessions: Vec<ProjectionSession> = self.mounted.iter().cloned().collect();
        for session in sessions {
            self.client.forget_session(&session);
        }
        self.mounted.clear();
        self.requests.clear();
        self.pending = None;
    }

    // ── Starting an operation ───────────────────────────────────────────────

    /// Mount one discovered projection.
    pub fn mount(&mut self, offer_index: usize) -> Result<Progress<Outcome>, String> {
        let request = self
            .descriptor
            .projections
            .get(offer_index)
            .map(|offer| offer.request.clone())
            .ok_or_else(|| format!("endpoint has no projection {offer_index}"))?;
        Ok(self.begin(
            Pending::Mount {
                request: request.clone(),
            },
            CarrierRequestBody::Snapshot(request),
        ))
    }

    /// Ask for a fresh full snapshot on the request that mounted this session.
    pub fn resnapshot(&mut self, session: &ProjectionSession) -> Result<Progress<Outcome>, String> {
        let request = self
            .requests
            .get(session)
            .cloned()
            .ok_or_else(|| format!("Graphshell did not mount {}", session.0))?;
        Ok(self.begin(
            Pending::Resnapshot {
                session: session.clone(),
                request: request.clone(),
            },
            CarrierRequestBody::Snapshot(request),
        ))
    }

    /// Resolve one presentation, fetching resources only as it needs them.
    ///
    /// Returns [`Progress::Done`] without asking anything when every resource is
    /// already cached, which is the ordinary case after the first resolve.
    pub fn resolve(
        &mut self,
        session: &ProjectionSession,
        instance: InstanceId,
    ) -> Result<Progress<Outcome>, String> {
        match self
            .client
            .resolve(session, instance, &self.profile)
            .map_err(|error| format!("could not resolve {}: {error:?}", session.0))?
        {
            PresentationResolution::Ready(presentation) => {
                Ok(Progress::Done(Outcome::Resolved(Box::new(presentation))))
            },
            PresentationResolution::NeedsResource(request) => Ok(self.begin(
                Pending::Resolve {
                    session: session.clone(),
                    instance,
                    request: request.clone(),
                },
                CarrierRequestBody::Resource(request),
            )),
        }
    }

    /// Read a card's disclosed media by role, after the card itself has been
    /// resolved and content-address verified. It cannot request an arbitrary
    /// hash or promote opaque owner data into a presentation codec.
    pub fn media_resource(
        &mut self,
        session: &ProjectionSession,
        instance: InstanceId,
        index: usize,
    ) -> Result<Progress<Outcome>, String> {
        let resolved = self
            .client
            .resolve(session, instance, &self.profile)
            .map_err(|e| format!("media card is unavailable: {e:?}"))?;
        let crate::PresentationResolution::Ready(resolved) = resolved else {
            return Err("Resolve the advertised card before reading its media.".into());
        };
        let crate::ResolvedContent::PortableCard(card) = resolved.content else {
            return Err("Only a portable card may disclose this media role.".into());
        };
        let resource = *card
            .media
            .get(index)
            .ok_or("The card does not disclose that media role.")?;
        let request = ResourceRequest {
            session: session.clone(),
            resource,
        };
        Ok(self.begin(
            Pending::MediaResource {
                request: request.clone(),
            },
            CarrierRequestBody::Resource(request),
        ))
    }

    /// Invoke an action exactly as advertised.
    pub fn invoke<T: Serialize>(
        &mut self,
        session: &ProjectionSession,
        target: InstanceId,
        action: &AdvertisedAction,
        payload: &T,
    ) -> Result<Progress<Outcome>, String> {
        let ack = self
            .client
            .acknowledgement(session)
            .ok_or_else(|| format!("Graphshell did not acknowledge {}", session.0))?;
        let payload = serde_json::to_vec(payload)
            .map_err(|error| format!("could not encode intent payload: {error}"))?;
        self.submit_command(
            &ActionDraftTarget {
                session: session.clone(),
                target,
                observed_epoch: ack.epoch,
                observed_revision: ack.revision,
            },
            action,
            &action.command_invocation(payload),
        )
    }

    /// Submit a draft composed from endpoint-advertised values.
    ///
    /// Missing or invalid local selections fail here, before anything is asked;
    /// endpoint authorization, replay, and stale checks remain authoritative.
    pub fn submit_action_draft(
        &mut self,
        target: &ActionDraftTarget,
        draft: &mut ActionDraft,
    ) -> Result<Progress<Outcome>, String> {
        let composed = draft
            .invocation(target)
            .map_err(|error| format!("could not compose advertised action: {error}"))?;
        self.submit_command(
            target,
            draft.action(),
            &draft.action().command_invocation(composed.payload),
        )
    }

    /// Submit an existing owner command, including domain-specific payloads
    /// composed by that domain's adapter. No routing or grant is inferred from
    /// the command ID, schema or effect. The original observation is preserved.
    pub fn submit_command(
        &mut self,
        target: &ActionDraftTarget,
        offered: &AdvertisedAction,
        command: &hest::Invocation,
    ) -> Result<Progress<Outcome>, String> {
        if self.is_awaiting() {
            return Err("Graphshell is awaiting the endpoint's answer".into());
        }
        let advertised = self
            .current_actions(&target.session, target.target)?
            .into_iter()
            .find(|candidate| candidate == offered)
            .ok_or("action is no longer advertised for the selected presentation")?;
        let invocation = IntentInvocation::from_command(
            command,
            &advertised,
            target.session.clone(),
            target.target,
            target.observed_epoch,
            target.observed_revision,
        )
        .map_err(|error| format!("could not bind advertised command: {error:?}"))?;
        Ok(self.begin(Pending::Intent, CarrierRequestBody::Intent(invocation)))
    }

    /// Current endpoint actions for one exact, active item. Cached scenes and
    /// unsupported presentation fallbacks remain readable, but offer no live
    /// actions here. Viewer-local actions have their separate dispatch path.
    pub fn current_actions(
        &self,
        session: &ProjectionSession,
        target: InstanceId,
    ) -> Result<Vec<AdvertisedAction>, String> {
        if !self.mounted.contains(session) {
            return Err(format!("Graphshell did not mount {}", session.0));
        }
        let mounted = self.client.mounted(session).ok_or("owner is unavailable")?;
        if mounted.status != chirograph::SessionStatus::Live {
            return Err("owner projection is not live".into());
        }
        if mounted.scene.active_item(target).is_none() {
            return Err("owner action target is unavailable".into());
        }
        let offer = mounted
            .presentation
            .offers_for(target)
            .and_then(|offers| {
                offers
                    .iter()
                    .find(|offer| self.profile.supports(offer.requires))
            })
            .ok_or("owner target has no supported presentation")?;
        Ok(offer.semantics.actions.clone())
    }

    /// Open one endpoint-advertised bounded action form at the client's current
    /// acknowledgement.
    ///
    /// Pure: it reads the advertised accessibility tree and asks the endpoint
    /// nothing, which is why it returns its answer directly rather than a
    /// [`Progress`].
    pub fn open_action_draft(
        &self,
        session: &ProjectionSession,
        target: InstanceId,
        intent: &str,
    ) -> Result<(ActionDraft, ActionDraftTarget), String> {
        let tree = self
            .client
            .accessibility_tree(session, &self.profile)
            .map_err(|error| format!("could not inspect {}: {error:?}", session.0))?;
        let action = tree
            .children
            .iter()
            .find(|item| item.instance == target)
            .and_then(|item| item.actions.iter().find(|action| action.intent.0 == intent))
            .cloned()
            .ok_or_else(|| {
                format!(
                    "intent {intent} was not advertised for {} in {}",
                    target.0, session.0
                )
            })?;
        if action.input_form.is_none() {
            return Err(format!(
                "intent {intent} does not advertise a bounded action form"
            ));
        }
        let acknowledgement = self
            .client
            .acknowledgement(session)
            .ok_or_else(|| format!("Graphshell did not acknowledge {}", session.0))?;
        Ok((
            ActionDraft::new(action),
            ActionDraftTarget {
                session: session.clone(),
                target,
                observed_epoch: acknowledgement.epoch,
                observed_revision: acknowledgement.revision,
            },
        ))
    }

    /// Begin recovering from one revision bell.
    ///
    /// Returns `Done(Changed(false))` without asking anything when the notice
    /// names a position this end has already acknowledged — the ordinary case
    /// for a bell that merely confirms what a resume already applied.
    pub fn resume_from_notice(
        &mut self,
        notice: CarrierNotice,
    ) -> Result<Progress<Outcome>, String> {
        self.resume_step(notice, 0)
    }

    /// Ask the endpoint to confirm or advance a retained session after a
    /// reconnect, even when the link delivered no revision notice.
    pub fn resume_session(
        &mut self,
        session: &ProjectionSession,
    ) -> Result<Progress<Outcome>, String> {
        let request = self
            .client
            .resume_request(session)
            .ok_or_else(|| format!("Graphshell did not acknowledge {}", session.0))?;
        let notice = CarrierNotice {
            session: request.session.clone(),
            epoch: request.epoch,
            revision: request.revision,
        };
        self.client.mark_stale(session);
        Ok(self.begin(
            Pending::Resume {
                notice,
                attempts: 0,
            },
            CarrierRequestBody::Resume(request),
        ))
    }

    /// Discover again on a session that already exists.
    ///
    /// A reconnect is a new link to the same endpoint, and the first thing
    /// said on any link is discovery. Before this existed, discovery was only
    /// modelled for the link that *built* the core, so a second one arrived
    /// as an answer nobody was waiting for and the reconnect died there —
    /// found by the first headed reconnect, not by any native test, because
    /// none had ever discovered twice. The descriptor is refreshed; every
    /// mounted session and its acknowledgement is kept, which is what lets the
    /// bells that follow resume by diff rather than by snapshot.
    pub fn rediscover(&mut self) -> Progress<Outcome> {
        self.begin(Pending::Rediscover, CarrierRequestBody::Discover)
    }

    /// Start the discovery round trip a polling host uses to let its carrier
    /// collect whatever notices the endpoint has already written.
    pub fn poll(&mut self) -> Progress<Outcome> {
        self.begin(Pending::PollDiscover, CarrierRequestBody::Discover)
    }

    /// Close the session.
    pub fn close(&mut self) -> Progress<Outcome> {
        self.begin(Pending::Close, CarrierRequestBody::Close)
    }

    // ── Feeding answers back ────────────────────────────────────────────────

    /// Fold one endpoint answer into the session.
    ///
    /// Returns [`Progress::Ask`] again when the operation needs another round
    /// trip — a resolve wanting a second resource, or a resume the endpoint
    /// answered with a resynchronize.
    pub fn on_response(&mut self, body: CarrierResponseBody) -> Result<Progress<Outcome>, String> {
        let pending = self
            .pending
            .take()
            .ok_or_else(|| "Graphshell received an answer it did not ask for".to_string())?;
        match pending {
            Pending::Rediscover => match body {
                CarrierResponseBody::Descriptor(descriptor) => {
                    self.descriptor = descriptor.clone();
                    Ok(Progress::Done(Outcome::Descriptor(Box::new(descriptor))))
                },
                other => Err(unexpected("descriptor", &other)),
            },
            Pending::Mount { request } => match body {
                CarrierResponseBody::Snapshot(snapshot) => {
                    let session = self.apply_snapshot(*snapshot, request)?;
                    Ok(Progress::Done(Outcome::Mounted(session)))
                },
                other => Err(unexpected("snapshot", &other)),
            },
            Pending::Resnapshot { session, request } => match body {
                CarrierResponseBody::Snapshot(snapshot) => {
                    if snapshot.session != session {
                        return Err(format!(
                            "endpoint resnapshot changed session {} to {}",
                            session.0, snapshot.session.0
                        ));
                    }
                    self.apply_snapshot(*snapshot, request)?;
                    Ok(Progress::Done(Outcome::Resnapshotted))
                },
                other => Err(unexpected("snapshot", &other)),
            },
            Pending::Resolve {
                session,
                instance,
                request,
            } => match body {
                CarrierResponseBody::Resource(response) => {
                    if !response.matches_request(&request) {
                        return Err(
                            "The resource response has another scope or content address.".into(),
                        );
                    }
                    self.client
                        .apply_resource(response)
                        .map_err(|error| format!("resource was rejected: {error:?}"))?;
                    // May want another resource; `resolve` asks again if so.
                    self.resolve(&session, instance)
                },
                other => Err(unexpected("resource", &other)),
            },
            Pending::MediaResource { request } => match body {
                CarrierResponseBody::Resource(response) => {
                    if !response.matches_request(&request) {
                        return Err(
                            "The media response has another scope or content address.".into()
                        );
                    }
                    Ok(Progress::Done(Outcome::MediaResource(Box::new(response))))
                },
                other => Err(unexpected("media resource", &other)),
            },
            Pending::Intent => match body {
                CarrierResponseBody::Intent(result) => {
                    Ok(Progress::Done(Outcome::Intent(Box::new(result))))
                },
                other => Err(unexpected("intent result", &other)),
            },
            Pending::Resume { notice, attempts } => match body {
                CarrierResponseBody::Resume(reply) => {
                    let applied =
                        self.client
                            .apply_resume(&notice.session, reply)
                            .map_err(|error| {
                                format!(
                                    "Graphshell rejected resume for {}: {error:?}",
                                    notice.session.0
                                )
                            })?;
                    match applied {
                        ResumeApplication::Current(_) | ResumeApplication::Applied(_) => {
                            Ok(Progress::Done(Outcome::Changed(true)))
                        },
                        ResumeApplication::Resynchronize(next) => {
                            let attempts = attempts + 1;
                            if attempts >= RESUME_ATTEMPTS {
                                return Err(format!(
                                    "endpoint did not produce an applicable resume after {RESUME_ATTEMPTS} attempts"
                                ));
                            }
                            Ok(self.begin(
                                Pending::Resume { notice, attempts },
                                CarrierRequestBody::Resume(next),
                            ))
                        },
                    }
                },
                other => Err(unexpected("resume reply", &other)),
            },
            Pending::PollDiscover => match body {
                CarrierResponseBody::Descriptor(_) => Ok(Progress::Done(Outcome::Changed(false))),
                other => Err(unexpected("descriptor", &other)),
            },
            Pending::Close => match body {
                CarrierResponseBody::Closed => {
                    self.purge();
                    Ok(Progress::Done(Outcome::Closed))
                },
                other => Err(unexpected("session close", &other)),
            },
        }
    }

    // ── Internals ───────────────────────────────────────────────────────────

    /// Arm one pending operation and hand back what to ask.
    fn begin(&mut self, pending: Pending, body: CarrierRequestBody) -> Progress<Outcome> {
        self.pending = Some(pending);
        Progress::Ask(body)
    }

    /// The resume state machine's entry, shared by the first attempt and every
    /// resynchronize after it.
    fn resume_step(
        &mut self,
        notice: CarrierNotice,
        attempts: u8,
    ) -> Result<Progress<Outcome>, String> {
        let acknowledged = self
            .client
            .acknowledgement(&notice.session)
            .ok_or_else(|| format!("revision notice names unknown session {}", notice.session.0))?;
        if notice.epoch == acknowledged.epoch && notice.revision <= acknowledged.revision {
            return Ok(Progress::Done(Outcome::Changed(false)));
        }
        self.client.mark_stale(&notice.session);
        let Some(request) = self.client.resume_request(&notice.session) else {
            return Ok(Progress::Done(Outcome::Changed(false)));
        };
        Ok(self.begin(
            Pending::Resume { notice, attempts },
            CarrierRequestBody::Resume(request),
        ))
    }

    fn apply_snapshot(
        &mut self,
        snapshot: chirograph::ProjectionSnapshot,
        request: ProjectionRequest,
    ) -> Result<ProjectionSession, String> {
        let session = snapshot.session.clone();
        self.client
            .apply_snapshot(snapshot)
            .map_err(|error| format!("Graphshell rejected {session:?}: {error:?}"))?;
        self.mounted.insert(session.clone());
        self.requests.insert(session.clone(), request);
        Ok(session)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chirograph::{EndpointDescriptor, ProjectionOffer, ProjectionRequest, ProtocolVersion};
    use sceno::score::{Arrangement, Score};
    use scenotime::{Revision, SceneEpoch};

    fn request_for(session: &ProjectionSession) -> ProjectionRequest {
        ProjectionRequest {
            version: ProtocolVersion::V1,
            session: session.clone(),
            score: Score::new(Arrangement::Spiral(Default::default())),
        }
    }

    /// A descriptor advertising exactly one projection offer.
    fn descriptor_with_one_offer() -> EndpointDescriptor {
        let session = ProjectionSession("fixture:one".into());
        EndpointDescriptor {
            label: "fixture endpoint".into(),
            projections: vec![ProjectionOffer {
                label: "one".into(),
                request: request_for(&session),
            }],
        }
    }

    fn core() -> SessionCore {
        SessionCore::from_descriptor(descriptor_with_one_offer(), CapabilityProfile::default())
    }

    /// The point of the split: a whole operation can be started and inspected
    /// with no carrier anywhere in the test.
    #[test]
    fn mounting_asks_for_a_snapshot_without_any_carrier() {
        let mut core = core();
        assert!(!core.is_awaiting());
        let progress = core.mount(0).expect("offer 0 exists");
        assert!(
            matches!(progress.request(), Some(CarrierRequestBody::Snapshot(_))),
            "mount asks for a snapshot first"
        );
        assert!(core.is_awaiting(), "the core is now waiting on an answer");
    }

    #[test]
    fn mounting_an_offer_that_does_not_exist_asks_nothing() {
        let mut core = core();
        let error = core.mount(7).expect_err("there is no offer 7");
        assert!(error.contains("no projection 7"), "{error}");
        assert!(
            !core.is_awaiting(),
            "a rejected start must not leave an operation armed"
        );
    }

    /// An answer nobody asked for is a protocol error rather than a panic or a
    /// silent state change, because an event-driven adapter can be handed one
    /// by a peer at any moment.
    #[test]
    fn an_unsolicited_answer_is_refused() {
        let mut core = core();
        let error = core
            .on_response(CarrierResponseBody::Closed)
            .expect_err("nothing was asked");
        assert!(error.contains("did not ask for"), "{error}");
    }

    /// The wrong answer to the right question names both, which is what makes a
    /// misbehaving endpoint diagnosable from a receipt line.
    #[test]
    fn the_wrong_answer_shape_is_named() {
        let mut core = core();
        let _ = core.mount(0).expect("offer 0 exists");
        let error = core
            .on_response(CarrierResponseBody::Closed)
            .expect_err("a close is not a snapshot");
        assert!(error.contains("expected snapshot"), "{error}");
        assert!(error.contains("a session close"), "{error}");
    }

    #[test]
    fn a_notice_for_an_unknown_session_is_refused() {
        let mut core = core();
        let notice = CarrierNotice {
            session: ProjectionSession("never-mounted".into()),
            epoch: SceneEpoch(1),
            revision: Revision(2),
        };
        let error = core
            .resume_from_notice(notice)
            .expect_err("the session was never mounted");
        assert!(error.contains("unknown session"), "{error}");
        assert!(!core.is_awaiting());
    }

    /// Closing is a round trip, and the core stays armed until it is answered —
    /// which is what stops an adapter dropping the carrier before the verb
    /// travels.
    #[test]
    fn closing_asks_before_it_purges() {
        let mut core = core();
        let progress = core.close();
        assert!(matches!(
            progress.request(),
            Some(CarrierRequestBody::Close)
        ));
        assert!(core.is_awaiting());
        let done = core
            .on_response(CarrierResponseBody::Closed)
            .expect("close was answered");
        assert!(matches!(done, Progress::Done(Outcome::Closed)));
        assert!(!core.is_awaiting());
    }
    fn media_core() -> (SessionCore, ProjectionSession, Vec<u8>, Vec<u8>) {
        use chirograph::{
            BoundsRelationship, CachePolicy, CacheRetention, PortableCardV1, PresentationBinding,
            PresentationCapability, PresentationCodec, PresentationKey, PresentationManifest,
            PresentationOffer, PresentationSemantics, ProjectionSnapshot, SemanticRole,
        };
        use sceno::{
            Footprint, ProjectedItem, Representation, Scene, Size2, SourceRef, Transform2,
        };
        let mut core = SessionCore::from_descriptor(
            descriptor_with_one_offer(),
            CapabilityProfile::new([PresentationCapability::PortableCard]),
        );
        let session = core.descriptor.projections[0].request.session.clone();
        let media = b"opaque owner-disclosed material".to_vec();
        let card = serde_json::to_vec(&PortableCardV1 {
            title: "Session".into(),
            values: vec![],
            badges: vec![],
            media: vec![chirograph::ContentHash::of(&media)],
        })
        .unwrap();
        let mut scene = Scene::new();
        let source = scene.intern_source(SourceRef::new("session", "one"));
        scene.items.push(ProjectedItem {
            source,
            space: Scene::WORLD,
            transform: Transform2::translation(0., 0.),
            footprint: Footprint::Rect {
                size: Size2::new(120., 80.),
            },
            representation: Representation::Glyph,
            layer: 0,
            visible: true,
            hit: None,
            channels: vec![],
        });
        let key = PresentationKey("one".into());
        let mut presentation = PresentationManifest::default();
        presentation.bindings.push(PresentationBinding {
            instance: InstanceId(0),
            key: key.clone(),
        });
        presentation.offers.insert(
            key,
            vec![
                PresentationOffer {
                    codec: PresentationCodec::PortableCardV1,
                    resource: chirograph::ContentHash::of(&card),
                    byte_size: card.len() as u64,
                    requires: PresentationCapability::PortableCard,
                    semantics: PresentationSemantics {
                        label: "Session".into(),
                        role: SemanticRole::Article,
                        bounds: BoundsRelationship::FillFootprint,
                        actions: vec![],
                    },
                },
                PresentationOffer {
                    codec: PresentationCodec::NativeGlyphV1,
                    resource: chirograph::ContentHash::of(b"other advertised resource"),
                    byte_size: b"other advertised resource".len() as u64,
                    requires: PresentationCapability::NativeGlyph,
                    semantics: PresentationSemantics {
                        label: "Other representation".into(),
                        role: SemanticRole::Graphic,
                        bounds: BoundsRelationship::FillFootprint,
                        actions: vec![],
                    },
                },
            ],
        );
        let _ = core.mount(0).unwrap();
        let _ = core
            .on_response(CarrierResponseBody::Snapshot(Box::new(
                ProjectionSnapshot {
                    version: ProtocolVersion::V1,
                    session: session.clone(),
                    scene: scenotime::SceneSnapshot::from_dense(SceneEpoch(1), Revision(1), scene)
                        .unwrap(),
                    presentation,
                    cache_policy: CachePolicy {
                        retention: CacheRetention::MemoryOnly,
                        expires_at_ms: None,
                        purge_on_revocation: true,
                    },
                },
            )))
            .unwrap();
        (core, session, card, media)
    }
    #[test]
    fn media_requires_a_verified_card_and_keeps_owner_bytes_out_of_presentation_cache() {
        let (mut core, session, card, media) = media_core();
        assert!(
            core.media_resource(&session, InstanceId(0), 0)
                .unwrap_err()
                .contains("Resolve")
        );
        let _ = core.resolve(&session, InstanceId(0)).unwrap();
        let _ = core
            .on_response(CarrierResponseBody::Resource(ResourceResponse::new(
                session.clone(),
                card,
            )))
            .unwrap();
        assert!(
            core.media_resource(&session, InstanceId(0), 1)
                .unwrap_err()
                .contains("does not disclose")
        );
        assert!(!core.is_awaiting());
        let requested = core.media_resource(&session, InstanceId(0), 0).unwrap();
        assert!(
            matches!(requested.request(),Some(CarrierRequestBody::Resource(request)) if request.resource==chirograph::ContentHash::of(&media))
        );
        let outcome = core
            .on_response(CarrierResponseBody::Resource(ResourceResponse::new(
                session.clone(),
                media.clone(),
            )))
            .unwrap();
        assert!(
            matches!(outcome,Progress::Done(Outcome::MediaResource(response)) if response.bytes==media)
        );
        assert!(
            !core
                .client
                .resources
                .contains_key(&(session, chirograph::ContentHash::of(&media)))
        );
    }

    #[test]
    fn resolve_rejects_an_advertised_resource_other_than_the_one_requested() {
        let (mut core, session, _card, _media) = media_core();
        let _ = core.resolve(&session, InstanceId(0)).unwrap();
        let response =
            ResourceResponse::new(session.clone(), b"other advertised resource".to_vec());
        let error = core
            .on_response(CarrierResponseBody::Resource(response))
            .expect_err("a valid but unrequested resource must be refused");
        assert!(
            error.contains("another scope or content address"),
            "{error}"
        );
        assert!(
            !core.is_awaiting(),
            "the refusal starts no follow-up request"
        );
        assert!(!core.client.resources.contains_key(&(
            session,
            chirograph::ContentHash::of(b"other advertised resource")
        )));
    }

    #[test]
    fn resolve_rejects_the_requested_resource_from_another_session() {
        let (mut core, session, card, _) = media_core();
        let _ = core.resolve(&session, InstanceId(0)).unwrap();
        let response = ResourceResponse::new(ProjectionSession("another-session".into()), card);

        let error = core
            .on_response(CarrierResponseBody::Resource(response))
            .expect_err("the exact requested resource from another session must be refused");
        assert!(
            error.contains("another scope or content address"),
            "{error}"
        );
        assert!(
            !core.is_awaiting(),
            "the refusal starts no follow-up request"
        );
        assert!(
            core.client.resources.is_empty(),
            "wrong-session bytes are never cached"
        );
    }

    #[test]
    fn resolve_rejects_bytes_that_do_not_match_the_requested_content_address() {
        let (mut core, session, card, _) = media_core();
        let _ = core.resolve(&session, InstanceId(0)).unwrap();
        let mut response = ResourceResponse::new(session.clone(), card);
        response.bytes.push(0);

        let error = core
            .on_response(CarrierResponseBody::Resource(response))
            .expect_err("bytes that do not match the requested hash must be refused");
        assert!(
            error.contains("another scope or content address"),
            "{error}"
        );
        assert!(
            !core.is_awaiting(),
            "the refusal starts no follow-up request"
        );
        assert!(
            core.client.resources.is_empty(),
            "corrupt bytes are never cached"
        );
    }

    #[test]
    fn media_refuses_wrong_scope_wrong_hash_and_corrupted_bytes() {
        for failure in 0..3 {
            let (mut core, session, card, media) = media_core();
            let _ = core.resolve(&session, InstanceId(0)).unwrap();
            let _ = core
                .on_response(CarrierResponseBody::Resource(ResourceResponse::new(
                    session.clone(),
                    card,
                )))
                .unwrap();
            let _ = core.media_resource(&session, InstanceId(0), 0).unwrap();
            let mut response = ResourceResponse::new(session, media);
            match failure {
                0 => response.session = ProjectionSession("another".into()),
                1 => response.resource = chirograph::ContentHash::of(b"another"),
                _ => response.bytes.push(0),
            }
            assert!(
                core.on_response(CarrierResponseBody::Resource(response))
                    .unwrap_err()
                    .contains("another scope or content address")
            );
            assert!(!core.is_awaiting());
        }
    }
}
