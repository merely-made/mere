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

/// The media on an explicit session card is its scoped graph envelope. Unlike
/// other material cards, an explicit session must name exactly one envelope.
#[cfg(feature = "view")]
pub(crate) fn session_media(
    card: &chirograph::PortableCardV1,
) -> Result<chirograph::ContentHash, String> {
    if card.media.len() != 1 {
        return Err("The explicit session must disclose its scoped graph material with exactly one media resource.".into());
    }
    Ok(card.media[0])
}

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
/// Branch a session at its live cursor: a copy meant to merge back.
pub const BRANCH_SESSION_INTENT: &str = "mere.sessions.branch";
/// Merge a branch or picked-off session back into the session it came from.
/// A clean merge applies at once; one with conflicts becomes a pending merge.
pub const MERGE_SESSION_INTENT: &str = "mere.sessions.merge";
/// Put a session in the trash.
pub const TRASH_SESSION_INTENT: &str = "mere.sessions.trash";
/// Take a session back out of the trash.
pub const RESTORE_SESSION_INTENT: &str = "mere.sessions.restore";
/// Schema of [`SessionsActionV1`].
pub const SESSIONS_SCHEMA: &str = "mere.sessions/v1";

/// The scene source kind of a pending merge's card in the sessions
/// projection, keyed by the source session's id. Its card's first media
/// resource is the [`PendingMergeV1`].
pub const MERGE_SOURCE: &str = "mere.merge";
/// Apply a pending merge with a choice for each conflict.
pub const RESOLVE_MERGE_INTENT: &str = "mere.merge.resolve";
/// Drop a pending merge, writing nothing.
pub const CANCEL_MERGE_INTENT: &str = "mere.merge.cancel";
/// Schema of [`MergeResolutionV1`].
pub const MERGE_SCHEMA: &str = "mere.merge/v1";
/// Schema of [`PendingMergeV1`].
pub const PENDING_MERGE_SCHEMA: &str = "mere.merge.pending/v1";

/// A pending merge as its card's media discloses it: which session merges
/// into which, and each conflict with its three values (session merge-in
/// note, M5).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PendingMergeV1 {
    pub schema: String,
    pub source: SessionId,
    pub source_name: String,
    pub target: SessionId,
    pub target_name: String,
    pub conflicts: Vec<pandect::MergeConflict>,
    /// Choices the resident carried over from an earlier plan of this merge
    /// that could not apply (a session moved): the view starts from these.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub carried: Vec<MergeChoiceV1>,
    /// Choices from that earlier plan the new one could not take, and why.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dropped: Vec<pandect::DroppedChoice>,
}

/// One conflict's choice, naming its part.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MergeChoiceV1 {
    pub part: kernel::graph::Part,
    pub choice: pandect::MergeChoice,
}

/// Payload of [`RESOLVE_MERGE_INTENT`] and [`CANCEL_MERGE_INTENT`]: the
/// pending merge, by its source session, and the choices (none to cancel).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MergeResolutionV1 {
    pub schema: String,
    pub source: SessionId,
    #[serde(default)]
    pub choices: Vec<MergeChoiceV1>,
}

impl MergeResolutionV1 {
    pub fn new(source: SessionId, choices: Vec<MergeChoiceV1>) -> Self {
        Self {
            schema: MERGE_SCHEMA.to_string(),
            source,
            choices,
        }
    }
}

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
    /// A step on session `id`: attach, fork, branch, merge, trash or restore.
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

// ── Remote admission ────────────────────────────────────────────────────────
//
// A mere route reached over a dialled carrier names its route and the
// application acting in the session's requested action path, under
// Graphshell's projection service (remote mere route admission note, §2). The
// host serves the route the path names; the application is accepted only when
// the session's grant names it too (§4). The spelling lives here, beside the
// route names, so a dialler and the host parse one grammar.

/// Graphshell's action domain, as `graphshell_host::admission` names it.
pub const REMOTE_DOMAIN: &str = "mere.graphshell";
/// Where a mere's explicit route is requested.
pub const REMOTE_MERE_SERVICE: &str = "/services/projection/mere";
/// Where a mere's ambient route is requested, apart from the explicit one.
pub const REMOTE_AMBIENT_SERVICE: &str = "/services/projection/ambient";
/// The action that opens the explicit route (`graphshell_host`'s `connect`).
pub const REMOTE_MERE_ACTION: &str = "connect";
/// The action that opens the ambient route (`graphshell_host`'s `ambient`).
pub const REMOTE_AMBIENT_ACTION: &str = "ambient";

/// Which of a mere's two routes a remote session opens.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RemoteRoute {
    /// `mere/<domain>`: the mere's sessions and graph, with its intents.
    Explicit,
    /// `ambient/mere/<domain>`: the read-only ambient crossing.
    Ambient,
}

/// What a remote session asks for: a mere's route, as one application.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RemoteMereRequest {
    pub route: RemoteRoute,
    pub domain: String,
    pub application: String,
}

impl RemoteMereRequest {
    /// The explicit route of `domain`, as `application`.
    pub fn explicit(domain: impl Into<String>, application: impl Into<String>) -> Self {
        Self {
            route: RemoteRoute::Explicit,
            domain: domain.into(),
            application: application.into(),
        }
    }

    /// The ambient route of `domain`, as `application`.
    pub fn ambient(domain: impl Into<String>, application: impl Into<String>) -> Self {
        Self {
            route: RemoteRoute::Ambient,
            domain: domain.into(),
            application: application.into(),
        }
    }

    /// The requested path, which is also the narrowest grant's path prefix.
    pub fn path(&self) -> String {
        let service = match self.route {
            RemoteRoute::Explicit => REMOTE_MERE_SERVICE,
            RemoteRoute::Ambient => REMOTE_AMBIENT_SERVICE,
        };
        format!("{service}/{}/app/{}", self.domain, self.application)
    }

    /// The requested action.
    pub fn action(&self) -> &'static str {
        match self.route {
            RemoteRoute::Explicit => REMOTE_MERE_ACTION,
            RemoteRoute::Ambient => REMOTE_AMBIENT_ACTION,
        }
    }

    /// The catalog route the host serves for it.
    pub fn route_id(&self) -> String {
        match self.route {
            RemoteRoute::Explicit => format!("mere/{}", self.domain),
            RemoteRoute::Ambient => format!("ambient/mere/{}", self.domain),
        }
    }

    /// Read a requested action back. `None` for anything that is not one of
    /// the two routes in exactly this spelling, with the action that route
    /// takes: an ambient path under `connect`, or an explicit one under
    /// `ambient`, is not a request either route serves.
    pub fn parse(domain: &str, path: &str, action: &str) -> Option<Self> {
        if domain != REMOTE_DOMAIN {
            return None;
        }
        let (route, rest) = if let Some(rest) = path.strip_prefix(REMOTE_MERE_SERVICE) {
            (RemoteRoute::Explicit, rest)
        } else if let Some(rest) = path.strip_prefix(REMOTE_AMBIENT_SERVICE) {
            (RemoteRoute::Ambient, rest)
        } else {
            return None;
        };
        let mut parts = rest.strip_prefix('/')?.split('/');
        let (mere, app, application, end) =
            (parts.next()?, parts.next()?, parts.next()?, parts.next());
        if end.is_some() || app != "app" || !segment(mere) || !segment(application) {
            return None;
        }
        let request = Self {
            route,
            domain: mere.to_string(),
            application: application.to_string(),
        };
        (request.action() == action).then_some(request)
    }

    /// Whether a grant whose scope selects `path_prefix` names this request's
    /// application: the prefix sits at or under this request's path, so the
    /// grant cannot be used as another application, or for another mere.
    pub fn named_by(&self, path_prefix: &str) -> bool {
        let path = self.path();
        path_prefix == path
            || path_prefix
                .strip_prefix(&path)
                .is_some_and(|rest| rest.starts_with('/'))
    }
}

fn segment(value: &str) -> bool {
    !value.is_empty() && !value.contains('/')
}

#[cfg(test)]
mod remote_tests {
    use super::*;

    #[test]
    fn a_request_reads_back_as_itself_and_names_its_route() {
        for request in [
            RemoteMereRequest::explicit("divination", "turnstone"),
            RemoteMereRequest::ambient("divination", "turnstone"),
        ] {
            let read = RemoteMereRequest::parse(REMOTE_DOMAIN, &request.path(), request.action());
            assert_eq!(read.as_ref(), Some(&request));
        }
        assert_eq!(
            RemoteMereRequest::explicit("divination", "turnstone").route_id(),
            "mere/divination"
        );
        assert_eq!(
            RemoteMereRequest::ambient("divination", "turnstone").route_id(),
            "ambient/mere/divination"
        );
    }

    #[test]
    fn a_route_does_not_take_the_other_routes_action_or_a_loose_spelling() {
        let explicit = RemoteMereRequest::explicit("divination", "turnstone");
        assert!(RemoteMereRequest::parse(REMOTE_DOMAIN, &explicit.path(), "ambient").is_none());
        let ambient = RemoteMereRequest::ambient("divination", "turnstone");
        assert!(RemoteMereRequest::parse(REMOTE_DOMAIN, &ambient.path(), "connect").is_none());
        for path in [
            "/services/projection",
            "/services/projection/mere/divination",
            "/services/projection/mere/divination/app",
            "/services/projection/mere/divination/app/",
            "/services/projection/mere//app/turnstone",
            "/services/projection/mere/divination/app/turnstone/extra",
            "/services/projection/mere/divination/apps/turnstone",
            "/services/projection/merely/divination/app/turnstone",
        ] {
            assert!(
                RemoteMereRequest::parse(REMOTE_DOMAIN, path, "connect").is_none(),
                "{path} is not a remote mere request"
            );
        }
        assert!(RemoteMereRequest::parse("mere.other", &explicit.path(), "connect").is_none());
    }

    #[test]
    fn only_a_grant_at_or_under_the_application_names_it() {
        let request = RemoteMereRequest::explicit("divination", "turnstone");
        assert!(request.named_by("/services/projection/mere/divination/app/turnstone"));
        assert!(request.named_by("/services/projection/mere/divination/app/turnstone/x"));
        assert!(!request.named_by("/services/projection/mere/divination"));
        assert!(!request.named_by("/services/projection/mere"));
        assert!(!request.named_by("/services/projection/mere/divination/app/turnstone-x"));
        assert!(!request.named_by("/services/projection/mere/divination/app/knot"));
    }
}
