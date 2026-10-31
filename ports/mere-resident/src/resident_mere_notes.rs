// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The notes projection (`djinn.mere/v1/notes`): a memory-shaped surface over
//! the attached session, so an agent remembers by appending a typed note
//! rather than by composing graph deltas.
//!
//! A note is a Knot-shaped node of the session graph: `notes://<uuid>`, its
//! subject as the title, its djot body inline, and its kind, evidence and
//! append time in the `mere.note` facet. Appending is one journaled change
//! through the shared host, so it is attributed, undone and merged like any
//! other edit. Searching sets this connection's query; the projection then
//! lists what matches, newest first.

use super::*;
use mere_overmap::wire::{
    APPEND_NOTE_INTENT, APPEND_NOTE_SCHEMA, AppendNoteV1, MERE_NOTES, NOTE_FACET, NOTE_SCHEME,
    NOTES_SOURCE, SEARCH_NOTES_INTENT, SEARCH_NOTES_SCHEMA, SearchNotesV1,
};

/// The most notes one snapshot lists.
const NOTES_LISTED: usize = 50;

/// What the `mere.note` facet holds.
#[derive(serde::Serialize, serde::Deserialize)]
struct NoteFacet {
    kind: String,
    #[serde(default)]
    evidence: Vec<String>,
    #[serde(default)]
    at_ms: u64,
}

/// One note as read back from the graph.
struct Note {
    address: String,
    subject: String,
    body: String,
    facet: NoteFacet,
}

impl Note {
    fn matches(&self, terms: &[String]) -> bool {
        let haystack = [
            self.subject.as_str(),
            self.body.as_str(),
            self.facet.kind.as_str(),
            &self.facet.evidence.join("\n"),
        ]
        .join("\n")
        .to_lowercase();
        terms.iter().all(|term| haystack.contains(term.as_str()))
    }

    fn card(&self) -> Result<SessionsCard, String> {
        let value = |label: &str, value: &str| CardValueV1 {
            label: label.into(),
            value: value.into(),
        };
        let mut values = vec![
            value("Kind", &self.facet.kind),
            value("Note", &self.address),
            value("Body", &self.body),
        ];
        values.extend(self.facet.evidence.iter().map(|r| value("Evidence", r)));
        let card = PortableCardV1 {
            title: self.subject.clone(),
            values,
            badges: vec![self.facet.kind.clone()],
            media: Vec::new(),
        };
        Ok(SessionsCard {
            source: self.address.clone(),
            label: self.subject.clone(),
            bytes: serde_json::to_vec(&card).map_err(|e| e.to_string())?,
            actions: Vec::new(),
            kind: None,
            media: Vec::new(),
        })
    }
}

/// Every note in `host`'s graph, newest first.
fn notes_in(host: &MereHost<DirectoryBackend>) -> Vec<Note> {
    let mut notes: Vec<Note> = host
        .graph()
        .nodes()
        .filter(|(_, node)| node.url().starts_with(NOTE_SCHEME))
        .filter_map(|(_, node)| {
            let facet = host.facet_value(node.url(), NOTE_FACET)?;
            Some(Note {
                address: node.url().to_string(),
                subject: node.title.clone(),
                body: node.body.clone().unwrap_or_default(),
                facet: <NoteFacet as serde::Deserialize>::deserialize(facet).ok()?,
            })
        })
        .collect();
    notes.sort_by(|a, b| (b.facet.at_ms, &b.address).cmp(&(a.facet.at_ms, &a.address)));
    notes
}

/// The edits that make `note` a node: one change, nothing else touched.
fn note_edits(note: AppendNoteV1, at_ms: u64) -> Vec<pandect::CapturedDelta> {
    let id = uuid::Uuid::new_v4().to_string();
    let facet = NoteFacet {
        kind: note.kind,
        evidence: note.evidence,
        at_ms,
    };
    let mut edits = vec![
        pandect::CapturedDelta::ReplayAddNodeWithIdIfMissing {
            id: id.clone(),
            url: format!("{NOTE_SCHEME}{id}"),
            position: [0.0, 0.0],
        },
        pandect::CapturedDelta::ReplaySetNodeTitleById {
            node_id: id.clone(),
            title: note.subject,
        },
        pandect::CapturedDelta::ReplaySetNodeFacetById {
            node_id: id.clone(),
            facet: NOTE_FACET.into(),
            value_json: serde_json::to_string(&facet).expect("a note facet serializes"),
        },
    ];
    if !note.body.is_empty() {
        edits.push(pandect::CapturedDelta::ReplaySetNodeBodyById {
            node_id: id,
            body: Some(note.body),
        });
    }
    edits
}

fn notes_action(intent: &str, label: &str, explanation: &str, schema: &str) -> AdvertisedAction {
    let effect = if intent == APPEND_NOTE_INTENT {
        IntentEffect::DomainTruth
    } else {
        IntentEffect::Curation
    };
    session_item::advertised(intent, label, explanation, schema, effect)
}

impl MereEndpoint {
    pub(super) fn notes_session() -> ProjectionSession {
        ProjectionSession(MERE_NOTES.into())
    }

    pub(super) fn notes_offer() -> ProjectionOffer {
        ProjectionOffer {
            label: "Notes".into(),
            request: ProjectionRequest {
                version: ProtocolVersion::V1,
                session: Self::notes_session(),
                score: Score::new(Arrangement::Spiral(Default::default())),
            },
        }
    }

    /// Whether this mere's writes need a journaled domain candidate, which a
    /// note is not.
    fn domain_bound(&self) -> bool {
        matches!(self.shared.domain.as_str(), "divination" | "personal-facts")
    }

    /// The graph's epoch, and the notes revision: the graph's revision moved
    /// once more for each search this connection set.
    pub(super) fn notes_position(&self, graph: (SceneEpoch, Revision)) -> (SceneEpoch, Revision) {
        (
            graph.0,
            Revision(graph.1.0.wrapping_add(self.notes_searches)),
        )
    }

    /// The notes card first, carrying the actions, then each note listed.
    fn notes_cards(&self) -> Result<(Vec<SessionsCard>, (SceneEpoch, Revision)), String> {
        let host = Arc::clone(&self.host);
        let (notes, read_only, graph) = run(async move {
            let host = host.lock().await;
            (
                notes_in(&host),
                host.graph_session().manifest().codicil_read_only,
                (host.projection_epoch(), host.projection_revision()),
            )
        });
        let terms: Vec<String> = self
            .notes_query
            .split_whitespace()
            .map(str::to_lowercase)
            .collect();
        let found: Vec<&Note> = notes.iter().filter(|note| note.matches(&terms)).collect();
        let mut actions = Vec::new();
        if !read_only && !self.domain_bound() {
            actions.push(notes_action(
                APPEND_NOTE_INTENT,
                "Append a note",
                "Remember something in this session as a note, attributed to your app. Payload: \
                 {\"schema\":\"mere.notes.append/v1\",\"kind\":\"decision\",\"subject\":\"one \
                 line\",\"body\":\"djot text\",\"evidence\":[\"https://… or notes://…\"]}; body \
                 and evidence are optional.",
                APPEND_NOTE_SCHEMA,
            ));
        }
        actions.push(notes_action(
            SEARCH_NOTES_INTENT,
            "Search notes",
            "List only the notes whose subject, body, kind or evidence hold every word. Payload: \
             {\"schema\":\"mere.notes.search/v1\",\"text\":\"words\"}; empty text lists all. Then \
             read the Notes projection.",
            SEARCH_NOTES_SCHEMA,
        ));
        let mut values = vec![CardValueV1 {
            label: "Found".into(),
            value: format!("{} of {}", found.len().min(NOTES_LISTED), notes.len()),
        }];
        if !terms.is_empty() {
            values.push(CardValueV1 {
                label: "Search".into(),
                value: self.notes_query.clone(),
            });
        }
        let header = PortableCardV1 {
            title: "Notes".into(),
            values,
            badges: Vec::new(),
            media: Vec::new(),
        };
        let mut cards = vec![SessionsCard {
            source: "notes".into(),
            label: "Notes".into(),
            bytes: serde_json::to_vec(&header).map_err(|e| e.to_string())?,
            actions,
            kind: None,
            media: Vec::new(),
        }];
        for note in found.into_iter().take(NOTES_LISTED) {
            cards.push(note.card()?);
        }
        Ok((cards, self.notes_position(graph)))
    }

    pub(super) fn notes_snapshot(&mut self) -> Result<ProjectionSnapshot, String> {
        let (cards, (epoch, revision)) = self.notes_cards()?;
        self.seen_notes = (epoch, revision);
        card_projection(&cards, Self::notes_session(), NOTES_SOURCE, epoch, revision)
    }

    pub(super) fn notes_resource(
        &self,
        request: ResourceRequest,
    ) -> Result<ResourceResponse, String> {
        let bytes = self
            .notes_cards()?
            .0
            .into_iter()
            .map(|card| card.bytes)
            .find(|bytes| ContentHash::of(bytes) == request.resource)
            .ok_or("notes resource is no longer current; request a new snapshot")?;
        Ok(ResourceResponse {
            session: request.session,
            resource: request.resource,
            bytes,
        })
    }

    pub(super) fn invoke_notes(
        &mut self,
        intent: IntentInvocation,
    ) -> Result<IntentResult, String> {
        let host = Arc::clone(&self.host);
        let (graph, read_only) = run(async move {
            let host = host.lock().await;
            (
                (host.projection_epoch(), host.projection_revision()),
                host.graph_session().manifest().codicil_read_only,
            )
        });
        // By intent at any revision of the session, but not across an epoch.
        if intent.observed_epoch != graph.0 {
            let (current_epoch, current_revision) = self.notes_position(graph);
            return Ok(IntentResult::Stale {
                current_epoch,
                current_revision,
            });
        }
        let reject = |reason: String| Ok(IntentResult::Rejected { reason });
        if intent.target != InstanceId(0) {
            return reject("notes actions are on the Notes card".into());
        }
        match intent.intent.as_str() {
            SEARCH_NOTES_INTENT => {
                let search = match serde_json::from_slice::<SearchNotesV1>(&intent.payload) {
                    Ok(search) if search.schema == SEARCH_NOTES_SCHEMA => search,
                    Ok(search) => {
                        return reject(format!("unknown search schema {:?}", search.schema));
                    },
                    Err(error) => return reject(format!("invalid search payload: {error}")),
                };
                self.notes_query = search.text.trim().to_string();
                self.notes_searches += 1;
                self.seen_notes = self.notes_position(graph);
                Ok(IntentResult::Accepted)
            },
            APPEND_NOTE_INTENT => {
                if self.domain_bound() {
                    return reject(
                        "domain writes require a complete journaled session candidate".into(),
                    );
                }
                if read_only {
                    return reject(
                        "read-only archive; fork the session before appending a note".into(),
                    );
                }
                let note = match serde_json::from_slice::<AppendNoteV1>(&intent.payload) {
                    Ok(note) => note,
                    Err(error) => return reject(format!("invalid note payload: {error}")),
                };
                if let Err(reason) = note.check() {
                    return reject(reason);
                }
                let at_ms = u64::try_from(
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis(),
                )
                .unwrap_or(u64::MAX);
                let (host, app) = (Arc::clone(&self.host), self.app.clone());
                let (written, graph) = run(async move {
                    let mut host = host.lock().await;
                    let written = host
                        .transact_edits(&app, note_edits(note, at_ms))
                        .await
                        .map_err(|error| error.to_string());
                    (
                        written,
                        (host.projection_epoch(), host.projection_revision()),
                    )
                });
                // An application is not rung for its own change.
                self.seen_graph = graph;
                self.seen_notes = self.notes_position(graph);
                match written {
                    Ok(()) => Ok(IntentResult::Accepted),
                    Err(reason) => reject(reason),
                }
            },
            other => reject(format!("the Notes card does not offer {other:?}")),
        }
    }
}
