// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The remote session against a scripted board endpoint speaking NDJSON
//! lines. The board answers as the real serve loop does: the response, then
//! whatever bells are queued, before the next read.

use std::collections::VecDeque;

use chirograph::{
    ActionFormChoiceV1, ActionFormFieldV1, ActionFormV1, BoundsRelationship, CachePolicy,
    CacheRetention, CarrierFailure, CarrierNotice, CarrierOutput, CarrierRequest,
    CarrierRequestBody, CarrierResponse, CarrierResponseBody, ContentHash, EndpointDescriptor,
    IntentEffect, IntentReference, NativeGlyphV1, PresentationBinding, PresentationCapability,
    PresentationCodec, PresentationKey, PresentationManifest, PresentationOffer,
    PresentationSemantics, ProjectionOffer, ProjectionRequest, ProjectionSnapshot,
    ProtocolVersion, ResumeReply, SemanticRole,
};
use sceno::score::{Arrangement, Score};
use sceno::{Footprint, ProjectedItem, Representation, Scene, Size2, SourceRef, Transform2};
use scenotime::{Revision, SceneEpoch, SceneSnapshot};

use super::*;

const APPEND: &str = "board/append";
const FORBIDDEN: &str = "board/forbidden";
const CHOOSE: &str = "board/choose";

fn profile() -> CapabilityProfile {
    CapabilityProfile::new([PresentationCapability::NativeGlyph])
}

fn action(intent: &str, label: &str, bounded: bool) -> AdvertisedAction {
    AdvertisedAction {
        intent: IntentReference(intent.into()),
        label: label.into(),
        explanation: format!("{label} on the board."),
        payload_schema: format!("{intent}/v1"),
        input_form: bounded.then(|| {
            ActionFormV1::new(format!("{intent}/v1")).with_field(ActionFormFieldV1::choice(
                "colour",
                "Colour",
                [
                    ActionFormChoiceV1::new("red", "Red"),
                    ActionFormChoiceV1::new("blue", "Blue"),
                ],
            ))
        }),
        effect: IntentEffect::Curation,
    }
}

/// A board of cards that appends on `APPEND` or `CHOOSE`, refuses
/// `FORBIDDEN`, and answers a stale position as stale.
struct Board {
    session: ProjectionSession,
    revision: u64,
    cards: usize,
    notices: VecDeque<CarrierNotice>,
    /// The verbs answered, in order.
    log: Vec<&'static str>,
}

impl Board {
    fn new() -> Self {
        Self {
            session: ProjectionSession("board:one".into()),
            revision: 1,
            cards: 1,
            notices: VecDeque::new(),
            log: Vec::new(),
        }
    }

    fn request(&self) -> ProjectionRequest {
        ProjectionRequest {
            version: ProtocolVersion::V1,
            session: self.session.clone(),
            score: Score::new(Arrangement::Spiral(Default::default())),
        }
    }

    fn snapshot(&self) -> ProjectionSnapshot {
        let mut scene = Scene::new();
        let source = scene.intern_source(SourceRef::new("board", "cards"));
        let mut presentation = PresentationManifest::default();
        for index in 0..self.cards {
            scene.items.push(ProjectedItem {
                source,
                space: Scene::WORLD,
                transform: Transform2::translation(140.0 * index as f32, 0.0),
                footprint: Footprint::Rect {
                    size: Size2::new(120.0, 80.0),
                },
                representation: Representation::Glyph,
                layer: 0,
                visible: true,
                hit: None,
                channels: Vec::new(),
            });
            let glyph = serde_json::to_vec(&NativeGlyphV1 {
                label: format!("Card {index}"),
                icon: None,
                color: None,
            })
            .unwrap();
            let key = PresentationKey(format!("card:{index}"));
            presentation.bindings.push(PresentationBinding {
                instance: InstanceId(index as u32),
                key: key.clone(),
            });
            presentation.offers.insert(
                key,
                vec![PresentationOffer {
                    codec: PresentationCodec::NativeGlyphV1,
                    resource: ContentHash::of(&glyph),
                    byte_size: glyph.len() as u64,
                    requires: PresentationCapability::NativeGlyph,
                    semantics: PresentationSemantics {
                        label: format!("Card {index}"),
                        role: SemanticRole::Graphic,
                        bounds: BoundsRelationship::FillFootprint,
                        actions: vec![
                            action(APPEND, "Append a card", false),
                            action(FORBIDDEN, "Forbidden action", false),
                            action(CHOOSE, "Choose a colour", true),
                        ],
                    },
                }],
            );
        }
        ProjectionSnapshot {
            version: ProtocolVersion::V1,
            session: self.session.clone(),
            scene: SceneSnapshot::from_dense(SceneEpoch(1), Revision(self.revision), scene)
                .unwrap(),
            presentation,
            cache_policy: CachePolicy {
                retention: CacheRetention::MemoryOnly,
                expires_at_ms: None,
                purge_on_revocation: true,
            },
        }
    }

    /// Move the board, as the host does natively or for an accepted intent,
    /// and ring the bell.
    fn append(&mut self) {
        self.cards += 1;
        self.revision += 1;
        self.notices.push_back(CarrierNotice {
            session: self.session.clone(),
            epoch: SceneEpoch(1),
            revision: Revision(self.revision),
        });
    }

    fn answer(&mut self, line: &str) -> Vec<String> {
        let request: CarrierRequest = serde_json::from_str(line).expect("a request line");
        let body = match request.body {
            CarrierRequestBody::Discover => {
                self.log.push("discover");
                Ok(CarrierResponseBody::Descriptor(EndpointDescriptor {
                    label: "Scripted board".into(),
                    projections: vec![ProjectionOffer {
                        label: "board".into(),
                        request: self.request(),
                    }],
                }))
            },
            CarrierRequestBody::Snapshot(_) => {
                self.log.push("snapshot");
                Ok(CarrierResponseBody::Snapshot(Box::new(self.snapshot())))
            },
            CarrierRequestBody::Intent(invocation) => {
                self.log.push("intent");
                let result = if invocation.observed_revision.0 != self.revision {
                    IntentResult::Stale {
                        current_epoch: SceneEpoch(1),
                        current_revision: Revision(self.revision),
                    }
                } else if invocation.intent == FORBIDDEN {
                    IntentResult::Rejected {
                        reason: "forbidden by the board".into(),
                    }
                } else {
                    self.append();
                    IntentResult::Accepted
                };
                Ok(CarrierResponseBody::Intent(result))
            },
            CarrierRequestBody::Resume(_) => {
                self.log.push("resume");
                Ok(CarrierResponseBody::Resume(ResumeReply::Snapshot(Box::new(
                    self.snapshot(),
                ))))
            },
            other => Err(CarrierFailure {
                message: format!("unsupported {other:?}"),
            }),
        };
        let mut lines = vec![
            serde_json::to_string(&CarrierOutput::Response(CarrierResponse {
                id: request.id,
                body,
            }))
            .unwrap(),
        ];
        lines.extend(
            self.notices
                .drain(..)
                .map(|notice| serde_json::to_string(&CarrierOutput::Notice(notice)).unwrap()),
        );
        lines
    }

    fn count(&self, verb: &str) -> usize {
        self.log.iter().filter(|logged| **logged == verb).count()
    }
}

/// Carry every queued line to the board and every answer back until quiet.
fn pump(remote: &mut RemoteSession, board: &mut Board) {
    for _ in 0..64 {
        let outgoing = remote.take_outgoing();
        if outgoing.is_empty() {
            return;
        }
        for line in outgoing {
            for answer in board.answer(&line) {
                remote.on_line(&answer);
            }
        }
    }
    panic!("the session never went quiet");
}

fn mounted() -> (RemoteSession, Board) {
    let mut remote = RemoteSession::new(profile());
    let mut board = Board::new();
    remote.joined();
    pump(&mut remote, &mut board);
    (remote, board)
}

fn cards(remote: &RemoteSession) -> usize {
    remote.mounted().map_or(0, |mounted| mounted.scene.tables.items.len())
}

fn index_of(remote: &RemoteSession, intent: &str) -> usize {
    remote
        .actions()
        .iter()
        .position(|(_, action)| action.intent.0 == intent)
        .expect("the intent is advertised")
}

#[test]
fn joining_discovers_then_mounts() {
    let mut remote = RemoteSession::new(profile());
    let mut board = Board::new();
    remote.joined();
    assert!(remote.in_flight(), "discovery is on the wire");
    assert_eq!(remote.pending(), Some(RemoteOp::Discover));
    pump(&mut remote, &mut board);
    assert_eq!(remote.status(), "open");
    assert_eq!(remote.revision(), Some(1));
    assert_eq!(cards(&remote), 1);
    assert!(!remote.in_flight());
    assert_eq!(remote.label(), "Remote projection · 1 objects · revision 1");
    assert_eq!(remote.selection(), ("Scripted board".into(), "board:one".into()));
    assert_eq!(remote.card_labels(), vec!["Card 0".to_string()]);
    let events = remote.take_events();
    assert_eq!(
        events,
        vec![
            "remote-joined",
            "remote-done Some(Discover)",
            "remote-done Some(Mount)"
        ]
    );
}

#[test]
fn actions_are_one_per_intent_not_one_per_card() {
    let (mut remote, mut board) = mounted();
    remote.invoke_action(index_of(&remote, APPEND));
    pump(&mut remote, &mut board);
    assert_eq!(cards(&remote), 2);
    let actions = remote.actions();
    assert_eq!(actions.len(), 3, "two cards, three intents: {actions:?}");
    assert!(actions.iter().all(|(target, _)| *target == InstanceId(0)));
}

#[test]
fn an_accepted_intent_polls_and_the_bell_resumes_by_diff() {
    let (mut remote, mut board) = mounted();
    remote.take_events();
    remote.invoke_action(index_of(&remote, APPEND));
    assert_eq!(remote.form.status, "Invoking · Append a card");
    assert!(remote.in_flight());
    pump(&mut remote, &mut board);
    assert_eq!(remote.form.status, "Accepted · 1 invocation(s)");
    assert_eq!(remote.form.draft, None, "an answered intent closes its draft");
    assert_eq!(remote.revision(), Some(2));
    assert_eq!(cards(&remote), 2);
    assert_eq!(remote.last_resume(), "diff · 1 → 2");
    assert!(!remote.in_flight());
    let events = remote.take_events();
    assert!(events.contains(&"remote-bell revision 2".to_string()), "{events:?}");
    assert_eq!(board.count("resume"), 1);
    assert_eq!(board.count("discover"), 2, "discovery, then the poll after acceptance");
}

#[test]
fn a_rejected_intent_resnapshots_and_rings_no_bell() {
    let (mut remote, mut board) = mounted();
    remote.take_events();
    remote.invoke_action(index_of(&remote, FORBIDDEN));
    pump(&mut remote, &mut board);
    assert_eq!(
        remote.form.status,
        "Rejected · forbidden by the board · revision after 1"
    );
    assert_eq!(remote.revision(), Some(1));
    assert_eq!(cards(&remote), 1);
    assert_eq!(board.count("snapshot"), 2, "the mount, then the measurement");
    assert!(
        !remote
            .take_events()
            .iter()
            .any(|event| event.starts_with("remote-bell"))
    );
    assert!(!remote.in_flight());
}

#[test]
fn a_bounded_action_waits_for_its_values() {
    let (mut remote, mut board) = mounted();
    remote.invoke_action(index_of(&remote, CHOOSE));
    assert_eq!(remote.form.status, "Choose values · Choose a colour");
    assert!(remote.take_outgoing().is_empty(), "nothing is sent for a form");
    assert!(!remote.in_flight());

    remote.form.choose("colour", "green");
    assert!(remote.form.status.starts_with("Choose values ·"), "{}", remote.form.status);
    remote.form.choose("colour", "red");
    assert_eq!(remote.form.status, "Selected colour");
    remote.submit_draft();
    pump(&mut remote, &mut board);
    assert_eq!(remote.form.status, "Accepted · 1 invocation(s)");
    assert_eq!(cards(&remote), 2);
}

#[test]
fn an_incomplete_draft_is_refused_locally_and_stays_open() {
    let (mut remote, _board) = mounted();
    remote.invoke_action(index_of(&remote, CHOOSE));
    remote.submit_draft();
    assert!(remote.take_outgoing().is_empty(), "nothing is sent");
    assert!(
        remote.form.status.starts_with("Choose required values ·"),
        "{}",
        remote.form.status
    );
    assert_eq!(remote.status(), "open", "the link is not at fault");
    assert!(remote.form.draft.is_some(), "the draft stays open");
    assert_eq!(remote.form.count, 0, "nothing was invoked");
    assert_eq!(remote.pending(), None);
}

#[test]
fn a_stale_position_is_answered_as_stale_and_the_bell_still_resumes() {
    let (mut remote, mut board) = mounted();
    remote.invoke_action(index_of(&remote, CHOOSE));
    remote.form.choose("colour", "blue");
    // The board moves while the form is open; its bell has not arrived.
    board.append();
    remote.submit_draft();
    pump(&mut remote, &mut board);
    assert_eq!(remote.form.status, "Stale · host at 2 · reopen the action");
    assert_eq!(remote.revision(), Some(2), "the bell after the answer resumed");
    assert!(!remote.in_flight());
}

#[test]
fn reconnect_keeps_the_mount_and_resumes_the_missed_change_by_diff() {
    let (mut remote, mut board) = mounted();
    remote.invoke_action(index_of(&remote, APPEND));
    pump(&mut remote, &mut board);
    assert_eq!(remote.revision(), Some(2));
    remote.take_events();

    remote.disconnect();
    assert_eq!(remote.status(), "disconnecting");
    assert!(remote.in_flight(), "the close has not landed");
    assert!(!remote.driver().is_awaiting());
    remote.channel_closed();
    assert_eq!(remote.status(), "disconnected");
    assert!(!remote.in_flight());

    // The host moves on without us.
    board.append();
    remote.reconnecting();
    assert_eq!(remote.status(), "reconnecting");
    assert!(remote.in_flight());
    remote.set_status("rejoining: challenge, admission");
    remote.rejoined();
    assert_eq!(remote.rejoins(), 1);
    pump(&mut remote, &mut board);
    assert_eq!(remote.status(), "open");
    assert_eq!(remote.revision(), Some(3));
    assert_eq!(cards(&remote), 3);
    assert_eq!(remote.last_resume(), "diff · 2 → 3");
    assert_eq!(board.count("snapshot"), 1, "the mount was kept, never re-taken");
    assert!(!remote.in_flight());
    let events = remote.take_events();
    assert!(events.contains(&"remote-rejoined".to_string()), "{events:?}");
    assert!(events.contains(&"remote-bell revision 3".to_string()), "{events:?}");
}

#[test]
fn a_channel_the_host_closed_is_named_as_such() {
    let (mut remote, _board) = mounted();
    remote.channel_closed();
    assert_eq!(remote.status(), "closed: the host closed the channel");
}

#[test]
fn a_nudge_reports_the_hosts_revision_or_its_failure() {
    let (mut remote, _board) = mounted();
    remote.take_events();
    remote.nudging();
    assert!(remote.in_flight());
    remote.nudged(Ok("3\n".into()));
    assert!(!remote.in_flight());
    assert_eq!(remote.form.status, "Nudged · host at revision 3");
    assert_eq!(
        remote.take_events(),
        vec!["remote-nudge", "remote-nudged revision 3"]
    );
    remote.nudging();
    remote.nudged(Err("503".into()));
    assert_eq!(remote.status(), "error: nudge: 503");
}

#[test]
fn an_unreadable_line_fails_the_operation_in_flight() {
    let mut remote = RemoteSession::new(profile());
    remote.joined();
    remote.take_outgoing();
    remote.on_line("{not json");
    assert!(remote.status().starts_with("error:"), "{}", remote.status());
    assert!(remote.form.status.starts_with("Failed · remote:"));
    assert_eq!(remote.pending(), None);
    assert!(
        remote
            .take_events()
            .iter()
            .any(|event| event.starts_with("remote-error"))
    );
}

#[test]
fn nothing_is_invoked_before_a_mount() {
    let mut remote = RemoteSession::new(profile());
    remote.invoke_action(0);
    assert_eq!(remote.form.status, "Failed · no remote action #0");
    remote.submit_draft();
    assert_eq!(
        remote.form.status,
        "Failed · no remote action draft target is open"
    );
    assert!(remote.take_outgoing().is_empty());
}

#[test]
fn the_first_bounded_form_opens_from_any_client() {
    let (remote, _board) = mounted();
    let mut form = ActionForm::new("Ready");
    form.open_first_bounded(
        remote.client().unwrap(),
        remote.session().unwrap(),
        &profile(),
    );
    assert_eq!(form.status, "Choose values · Choose a colour");
    assert_eq!(form.target.as_ref().unwrap().observed_revision, Revision(1));
}
