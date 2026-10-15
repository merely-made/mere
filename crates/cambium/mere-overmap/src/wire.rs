// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The wire vocabulary of a mere route (reservoir plan §7 items 31 to 37):
//! the `djinn.mere/v1/*` projection names, the session item's and the
//! sessions projection's intents and schemas, and their payloads. djinn
//! serves it; every application that reads a mere route speaks it.
//!
//! The session item is the attached session itself in the route's graph
//! projection, carrying the intents that act on the session rather than on
//! one node: apply edits, undo, redo and change a view. Each is accepted by id
//! at any revision of the graph (§7 item 32), though not across an epoch,
//! which names another session. The route's other projection, the mere's
//! sessions, carries the lifecycle intents, which name their session by id
//! (§7 item 33).
//!
//! The server-side builders (the session item's card and its advertised
//! actions over a live `GraphSession`) stay with the host that owns the
//! session; [`advertised`] is the one shape they share.

use chirograph::{AdvertisedAction, IntentEffect, IntentReference};
use kernel::graph::{CapturedDelta, Graph, NodeFacetStore};
use kernel::persistence::GraphSnapshot;
use pandect::{SessionId, ViewIntent};
use scenotime::{Revision, SceneEpoch};
use serde::{Deserialize, Serialize};

/// Complete owner-disclosed graph material for an explicit attached session.
/// The session card names this resource in `media`. It is an ephemeral read
/// cache, not a client-owned persistence format. Facets are separate because
/// the kernel snapshot deliberately externalizes them.
pub const SESSION_GRAPH_SCHEMA: &str = "mere.session.graph/v1";

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SessionGraphV1 {
    pub schema: String,
    /// The admitted application's own parallel view state only.
    pub application: String,
    pub views: std::collections::BTreeMap<String, ViewIntent>,
    pub session: SessionId,
    /// A thaw remains immutable; the first explicit edit forks it.
    pub read_only: bool,
    pub epoch: SceneEpoch,
    pub revision: Revision,
    pub graph: GraphSnapshot,
    pub facets: NodeFacetStore,
}

impl SessionGraphV1 {
    /// Hydrate only the disclosed graph, with its facet sidecar intact.
    pub fn into_graph(self) -> Result<Graph, String> {
        if self.schema != SESSION_GRAPH_SCHEMA {
            return Err(format!("unknown session graph schema {:?}", self.schema));
        }
        let mut graph = Graph::try_from_snapshot(&self.graph).map_err(|error| error.to_string())?;
        *graph.facets_mut() = self.facets;
        Ok(graph)
    }
}

/// A mere route's projection of its sessions: the mere's card, then one card
/// per session, oldest first.
pub const MERE_SESSIONS: &str = "djinn.mere/v1/sessions";
/// A mere route's projection of the attached session's graph.
pub const MERE_GRAPH: &str = "djinn.mere/v1/graph";
/// The scene source kind of the sessions projection's cards.
pub const SESSIONS_SOURCE: &str = "mere.sessions";
/// Mint a new, empty session.
pub const MINT_SESSION_INTENT: &str = "mere.sessions.mint";
/// Attach this connection to another session.
pub const ATTACH_SESSION_INTENT: &str = "mere.sessions.attach";
/// Fork a session at its live cursor.
pub const FORK_SESSION_INTENT: &str = "mere.sessions.fork";
/// Put a session in the trash.
pub const TRASH_SESSION_INTENT: &str = "mere.sessions.trash";
/// Take a session back out of the trash.
pub const RESTORE_SESSION_INTENT: &str = "mere.sessions.restore";
/// Schema of [`SessionsActionV1`].
pub const SESSIONS_SCHEMA: &str = "mere.sessions/v1";

/// The reservoir archive projection and source vocabulary.
pub const MERE_ARCHIVE: &str = "djinn.mere/v1/archive";
pub const ARCHIVE_SOURCE: &str = "mere.archive";
pub const SAVE_CODICIL_INTENT: &str = "mere.archive.save";
pub const OPEN_CODICIL_INTENT: &str = "mere.archive.open";
pub const COMPOSE_CODICILS_INTENT: &str = "mere.archive.compose";
pub const ARCHIVE_SCHEMA: &str = "mere.archive/v1";
/// The label of a codicil card's value naming a session the codicil was
/// saved from, by its uuid. A codicil saved from one session carries one.
pub const ARCHIVE_SESSION_LABEL: &str = "Session";

/// Archive operations name ids, never positions in the current projection.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArchiveActionV1 {
    pub schema: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session: Option<SessionId>,
    #[serde(default)]
    pub codicils: Vec<String>,
}

impl ArchiveActionV1 {
    pub fn save(session: SessionId) -> Self {
        Self {
            schema: ARCHIVE_SCHEMA.into(),
            session: Some(session),
            codicils: Vec::new(),
        }
    }
    pub fn on(codicils: Vec<String>) -> Self {
        Self {
            schema: ARCHIVE_SCHEMA.into(),
            session: None,
            codicils,
        }
    }
}

/// The shared advertised archive actions used by reservoir endpoints.
pub fn archive_action(intent: &str, label: &str) -> AdvertisedAction {
    advertised(
        intent,
        label,
        "Keep an immutable archive, browse a thaw, or compose source codicils.",
        ARCHIVE_SCHEMA,
        IntentEffect::DomainTruth,
    )
}

/// The scene source kind of the session item.
pub const SESSION_ITEM_SOURCE: &str = "mere.session";

/// Apply edits in stable-id form as one change.
pub const APPLY_EDITS_INTENT: &str = "mere.session.apply";
/// Schema of [`ApplyEditsV1`].
pub const APPLY_EDITS_SCHEMA: &str = "mere.session.apply/v1";
/// Undo the application's own latest change still in effect.
pub const UNDO_INTENT: &str = "mere.session.undo";
/// Redo the application's most recent undo.
pub const REDO_INTENT: &str = "mere.session.redo";
/// Schema of [`StepV1`], the undo and redo payload.
pub const STEP_SCHEMA: &str = "mere.session.step/v1";
/// Change one of the application's views.
pub const SET_VIEW_INTENT: &str = "mere.session.view";
/// Schema of [`SetViewV1`].
pub const SET_VIEW_SCHEMA: &str = "mere.session.view/v1";

/// Payload of [`APPLY_EDITS_INTENT`].
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApplyEditsV1 {
    pub schema: String,
    pub edits: Vec<CapturedDelta>,
    /// Candidate-derived edits must match the graph revision they read.
    /// Absence preserves the existing stable-id action semantics.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_revision: Option<Revision>,
}

impl ApplyEditsV1 {
    pub fn new(edits: Vec<CapturedDelta>) -> Self {
        Self {
            schema: APPLY_EDITS_SCHEMA.to_string(),
            edits,
            expected_revision: None,
        }
    }

    pub fn checked(revision: Revision, edits: Vec<CapturedDelta>) -> Self {
        Self {
            expected_revision: Some(revision),
            ..Self::new(edits)
        }
    }
}

/// Payload of [`UNDO_INTENT`] and [`REDO_INTENT`].
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StepV1 {
    pub schema: String,
}

impl Default for StepV1 {
    fn default() -> Self {
        Self {
            schema: STEP_SCHEMA.to_string(),
        }
    }
}

/// Payload of [`SET_VIEW_INTENT`]: one of the application's views, by name.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SetViewV1 {
    pub schema: String,
    pub view: String,
    pub state: ViewIntent,
}

impl SetViewV1 {
    pub fn new(view: impl Into<String>, state: ViewIntent) -> Self {
        Self {
            schema: SET_VIEW_SCHEMA.to_string(),
            view: view.into(),
            state,
        }
    }
}

/// Payload of the sessions projection's intents. A step on a session names
/// it by id, so it lands however the list has moved since the application
/// looked (§7 item 33); minting names none, and only minting reads the name.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SessionsActionV1 {
    pub schema: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session: Option<SessionId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
}

impl SessionsActionV1 {
    /// A step on session `id`: attach, fork, trash or restore.
    pub fn on(id: SessionId) -> Self {
        Self {
            schema: SESSIONS_SCHEMA.to_string(),
            session: Some(id),
            display_name: None,
        }
    }

    /// Mint a session, named or not.
    pub fn mint(display_name: Option<String>) -> Self {
        Self {
            schema: SESSIONS_SCHEMA.to_string(),
            session: None,
            display_name,
        }
    }
}

/// An advertised action on a mere route's vocabulary.
pub fn advertised(
    intent: &str,
    label: &str,
    explanation: &str,
    schema: &str,
    effect: IntentEffect,
) -> AdvertisedAction {
    AdvertisedAction {
        intent: IntentReference(intent.to_string()),
        label: label.to_string(),
        explanation: explanation.to_string(),
        payload_schema: schema.to_string(),
        input_form: None,
        effect,
    }
}
