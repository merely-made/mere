// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! A mere's route as the mere view (reservoir plan V2b step 3, §7 items 35
//! and 37). The route serves two projections, the mere's sessions and the
//! attached session's graph. This fills a [`MereViewModel`] from them and
//! turns the view's requests into the route's intents.
//!
//! It holds no connection. The host reads the route through the client it
//! has, djinn's door in process or `AppBrokerClient`, and hands the snapshots
//! and the session cards' bytes here. A relation shows its kernel family as
//! its kind (§7 item 43).

use std::collections::{BTreeMap, HashMap};

use cartography::MERE_GRAPH_ADAPTER;
use chirograph::{
    ContentHash, EndpointDescriptor, IntentInvocation, IntentResult, PortableCardV1,
    PresentationOffer, ProjectionRequest, ProjectionSnapshot, ResourceRequest,
};
use kernel::graph::Part;
use mere_view::{
    ArchiveEntry, ArchiveModel, ArchiveRequest, GraphModel, MereViewModel, MereViewRequest,
    MergeConflictEntry, MergeModel, MergeOption, MergeRequest, NodeEntry, NodeState, Provenance,
    RelationEntry, SessionEntry, SessionStep,
};
use pandect::MergeChoice;
use pandect::SessionId;
use sceno::{InstanceId, SourceRef};
use serde_json::Value;
use uuid::Uuid;

use crate::wire::{
    ARCHIVE_SESSION_LABEL, ARCHIVE_SOURCE, ATTACH_SESSION_INTENT, ArchiveActionV1,
    BRANCH_SESSION_INTENT, CANCEL_MERGE_INTENT, COMPOSE_CODICILS_INTENT, FORK_SESSION_INTENT,
    MERE_ARCHIVE, MERE_GRAPH, MERE_SESSIONS, MERGE_SESSION_INTENT, MERGE_SOURCE,
    MINT_SESSION_INTENT, MergeChoiceV1, MergeResolutionV1, OPEN_CODICIL_INTENT, PendingMergeV1,
    RESOLVE_MERGE_INTENT, RESTORE_SESSION_INTENT, SAVE_CODICIL_INTENT, SESSION_ITEM_SOURCE,
    SESSIONS_SOURCE, SessionsActionV1, TRASH_SESSION_INTENT,
};

/// Each step a session's card can offer, and the intent that takes it.
const STEPS: [(&str, SessionStep); 6] = [
    (ATTACH_SESSION_INTENT, SessionStep::Switch),
    (FORK_SESSION_INTENT, SessionStep::Fork),
    (BRANCH_SESSION_INTENT, SessionStep::Branch),
    (MERGE_SESSION_INTENT, SessionStep::Merge),
    (TRASH_SESSION_INTENT, SessionStep::Trash),
    (RESTORE_SESSION_INTENT, SessionStep::Restore),
];

/// The route's two projections from its descriptor: sessions, then graph.
pub fn requests(descriptor: &EndpointDescriptor) -> Option<(ProjectionRequest, ProjectionRequest)> {
    let find = |name: &str| {
        descriptor
            .projections
            .iter()
            .find(|offer| offer.request.session.0 == name)
            .map(|offer| offer.request.clone())
    };
    Some((find(MERE_SESSIONS)?, find(MERE_GRAPH)?))
}

/// The optional immutable archive reading, through the same admitted route.
pub fn archive_request(descriptor: &EndpointDescriptor) -> Option<ProjectionRequest> {
    descriptor
        .projections
        .iter()
        .find(|offer| offer.request.session.0 == MERE_ARCHIVE)
        .map(|offer| offer.request.clone())
}

fn advertises(snapshot: &ProjectionSnapshot, instance: InstanceId, intent: &str) -> bool {
    offer(snapshot, instance).is_some_and(|offer| {
        offer
            .semantics
            .actions
            .iter()
            .any(|action| action.intent.0 == intent)
    })
}

/// Populate archive controls only from actions the owner advertised. Reading
/// these cards grants no additional authority and owns no persistence.
pub fn fill_archive(
    model: &mut MereViewModel,
    sessions: &ProjectionSnapshot,
    archive: &ProjectionSnapshot,
    cards: &HashMap<ContentHash, Vec<u8>>,
) {
    let mut result = ArchiveModel::default();
    result.save_sessions = items(sessions, SESSIONS_SOURCE)
        .filter_map(|(instance, source)| {
            advertises(sessions, instance, SAVE_CODICIL_INTENT)
                .then(|| Uuid::parse_str(&source.id).ok().map(SessionId))
                .flatten()
        })
        .collect();
    for (instance, source) in items(archive, ARCHIVE_SOURCE) {
        if source.id == "archive" {
            result.can_compose = advertises(archive, instance, COMPOSE_CODICILS_INTENT);
            continue;
        }
        let card = offer(archive, instance)
            .and_then(|offer| cards.get(&offer.resource))
            .and_then(|bytes| serde_json::from_slice::<PortableCardV1>(bytes).ok());
        result.entries.push(ArchiveEntry {
            key: source.id.clone(),
            name: card
                .as_ref()
                .map(|card| card.title.clone())
                .unwrap_or_else(|| source.id.clone()),
            detail: card
                .as_ref()
                .map(|card| card.badges.join(", "))
                .unwrap_or_default(),
            can_open: advertises(archive, instance, OPEN_CODICIL_INTENT),
            session: card.as_ref().and_then(archive_session),
        });
    }
    model.archive = Some(result);
}

/// The one session a codicil card says it was saved from. A card naming
/// none, or more than one (a composition), names no single session.
pub fn archive_session(card: &PortableCardV1) -> Option<SessionId> {
    let mut sessions = card
        .values
        .iter()
        .filter(|value| value.label == ARCHIVE_SESSION_LABEL)
        .map(|value| Uuid::parse_str(&value.value).ok());
    let first = sessions.next()??;
    sessions
        .all(|other| other == Some(first))
        .then_some(SessionId(first))
}

/// Translate an archive request against the exact advertised projection.
/// Stable ids, rather than list indices, survive refreshed and reordered rows.
pub fn archive_intent(
    request: &MereViewRequest,
    sessions: &ProjectionSnapshot,
    archive: &ProjectionSnapshot,
) -> Option<IntentInvocation> {
    let MereViewRequest::Archive(request) = request else {
        return None;
    };
    let (snapshot, target, intent, payload) = match request {
        ArchiveRequest::Save(id) => {
            let (target, _) = items(sessions, SESSIONS_SOURCE)
                .find(|(_, source)| Uuid::parse_str(&source.id).ok() == Some(*id.as_uuid()))?;
            (
                sessions,
                target,
                SAVE_CODICIL_INTENT,
                ArchiveActionV1::save(*id),
            )
        },
        ArchiveRequest::Open(id) => {
            let (target, _) =
                items(archive, ARCHIVE_SOURCE).find(|(_, source)| &source.id == id)?;
            (
                archive,
                target,
                OPEN_CODICIL_INTENT,
                ArchiveActionV1::on(vec![id.clone()]),
            )
        },
        ArchiveRequest::Compose(ids) => {
            if ids.len() < 2 {
                return None;
            }
            let mut unique = std::collections::HashSet::new();
            if !ids.iter().all(|id| {
                unique.insert(id)
                    && items(archive, ARCHIVE_SOURCE)
                        .any(|(_, source)| &source.id == id && source.id != "archive")
            }) {
                return None;
            }
            let (target, _) =
                items(archive, ARCHIVE_SOURCE).find(|(_, source)| source.id == "archive")?;
            (
                archive,
                target,
                COMPOSE_CODICILS_INTENT,
                ArchiveActionV1::on(ids.clone()),
            )
        },
    };
    if !advertises(snapshot, target, intent) {
        return None;
    }
    Some(IntentInvocation {
        session: snapshot.session.clone(),
        target,
        observed_epoch: snapshot.scene.epoch,
        observed_revision: snapshot.scene.revision,
        intent: intent.into(),
        payload: serde_json::to_vec(&payload).expect("ArchiveActionV1 always serializes"),
    })
}

/// The cards to read before [`fill`]: the mere's and each session's.
pub fn card_requests(sessions: &ProjectionSnapshot) -> Vec<ResourceRequest> {
    sessions
        .presentation
        .offers
        .values()
        .filter_map(|offers| offers.first())
        .map(|offer| ResourceRequest {
            session: sessions.session.clone(),
            resource: offer.resource,
        })
        .collect()
}

/// Readable cards keyed by domain node identity. The synthetic session item
/// is deliberately excluded from both the graph and selected-record reader.
pub fn node_cards(
    graph: &ProjectionSnapshot,
    cards: &HashMap<ContentHash, Vec<u8>>,
) -> HashMap<String, PortableCardV1> {
    items(graph, MERE_GRAPH_ADAPTER)
        .filter_map(|(instance, source)| {
            let card = offer(graph, instance)
                .and_then(|offer| cards.get(&offer.resource))
                .and_then(|bytes| serde_json::from_slice(bytes).ok())?;
            Some((source.id.clone(), card))
        })
        .collect()
}

/// The complete explicit session reading, named by its owner-advertised card.
/// This does not synthesize an address or make ambient readings more capable.
pub fn session_graph_request(
    graph: &ProjectionSnapshot,
    cards: &HashMap<ContentHash, Vec<u8>>,
) -> Option<ResourceRequest> {
    let (instance, _) = items(graph, SESSION_ITEM_SOURCE).next()?;
    let card: PortableCardV1 =
        serde_json::from_slice(cards.get(&offer(graph, instance)?.resource)?).ok()?;
    Some(ResourceRequest {
        session: graph.session.clone(),
        resource: *card.media.first()?,
    })
}

/// Fill `model` from the route's projections and the cards
/// [`card_requests`] named, by content hash. The host's own parts stay: its
/// status, layout, actions and notice.
pub fn fill(
    model: &mut MereViewModel,
    sessions: &ProjectionSnapshot,
    graph: &ProjectionSnapshot,
    cards: &HashMap<ContentHash, Vec<u8>>,
) {
    // The session this connection is attached to is the one its graph shows.
    let attached = items(graph, SESSION_ITEM_SOURCE)
        .next()
        .and_then(|(_, source)| Uuid::parse_str(&source.id).ok())
        .map(SessionId);
    model.sessions.clear();
    model.can_mint = false;
    for (instance, source) in items(sessions, SESSIONS_SOURCE) {
        let Some(offer) = offer(sessions, instance) else {
            continue;
        };
        let title = cards
            .get(&offer.resource)
            .and_then(|bytes| serde_json::from_slice::<PortableCardV1>(bytes).ok())
            .map(|card| card.title)
            .unwrap_or_else(|| offer.semantics.label.clone());
        let offers = |intent: &str| {
            offer
                .semantics
                .actions
                .iter()
                .any(|action| action.intent.0 == intent)
        };
        // A session's card is keyed by the session's id; the mere's is not.
        let Ok(id) = Uuid::parse_str(&source.id).map(SessionId) else {
            model.title = title;
            model.can_mint = offers(MINT_SESSION_INTENT);
            continue;
        };
        model.sessions.push(SessionEntry {
            id,
            name: title,
            detail: None,
            attached: attached == Some(id),
            trashed: offers(RESTORE_SESSION_INTENT),
            steps: STEPS
                .into_iter()
                .filter(|(intent, _)| offers(intent))
                .map(|(_, step)| step)
                .collect(),
        });
    }
    model.graph = graph_model(graph);
}

/// The attached session's nodes and relations. Each relation stays apart,
/// keyed by its endpoints, its kind and its place among relations that share
/// both.
fn graph_model(graph: &ProjectionSnapshot) -> GraphModel {
    let mut keys = HashMap::new();
    let mut nodes = Vec::new();
    for (instance, source) in items(graph, MERE_GRAPH_ADAPTER) {
        let label = offer(graph, instance)
            .map(|offer| offer.semantics.label.clone())
            .filter(|label| !label.is_empty())
            .unwrap_or_else(|| source.id.clone());
        keys.insert(instance, source.id.clone());
        nodes.push(NodeEntry {
            key: source.id.clone(),
            label,
            state: NodeState::Available,
        });
    }
    let mut places: HashMap<(String, String, String), usize> = HashMap::new();
    let mut relations = Vec::new();
    for relation in graph.scene.tables.relations.iter().flatten() {
        let (Some(from), Some(to)) = (keys.get(&relation.from), keys.get(&relation.to)) else {
            continue;
        };
        let kind = relation
            .kind
            .clone()
            .unwrap_or_else(|| "related".to_string());
        let place = places
            .entry((from.clone(), to.clone(), kind.clone()))
            .or_default();
        relations.push(RelationEntry {
            key: format!("{from}>{to}:{kind}:{place}"),
            from: from.clone(),
            to: to.clone(),
            provenance: Provenance::Other(kind),
        });
        *place += 1;
    }
    GraphModel { nodes, relations }
}

/// The route's intent for a request, or `None` for one that is the host's
/// own: opening a node, a host action or a layout. A step names its session
/// by id, so it holds however the list has moved (§7 item 33).
pub fn intent(
    request: &MereViewRequest,
    sessions: &ProjectionSnapshot,
) -> Option<IntentInvocation> {
    let (intent, payload, target) = match request {
        MereViewRequest::Mint => (
            MINT_SESSION_INTENT,
            SessionsActionV1::mint(None),
            mere_card(sessions),
        ),
        MereViewRequest::Session(id, step) => {
            let (intent, _) = STEPS.into_iter().find(|(_, of)| of == step)?;
            let card = items(sessions, SESSIONS_SOURCE)
                .find(|(_, source)| Uuid::parse_str(&source.id).ok() == Some(*id.as_uuid()))
                .map_or_else(|| mere_card(sessions), |(instance, _)| instance);
            (intent, SessionsActionV1::on(*id), card)
        },
        // Archives, merges, nodes, host actions and layouts take other
        // paths; a request this route does not know names no intent.
        _ => return None,
    };
    Some(IntentInvocation {
        session: sessions.session.clone(),
        target,
        observed_epoch: sessions.scene.epoch,
        observed_revision: sessions.scene.revision,
        intent: intent.into(),
        payload: serde_json::to_vec(&payload).expect("SessionsActionV1 always serializes"),
    })
}

/// The pending merge's card in the sessions projection, and the material
/// request its first media names. The card is read with the others
/// ([`card_requests`]); this names what it discloses.
pub fn merge_cards(sessions: &ProjectionSnapshot) -> Vec<InstanceId> {
    items(sessions, MERGE_SOURCE)
        .map(|(instance, _)| instance)
        .collect()
}

/// Admit zero or one pending merge card; multiple cards make the owner's
/// current merge ambiguous.
pub(crate) fn unique_merge_card(
    sessions: &ProjectionSnapshot,
) -> Result<Option<InstanceId>, String> {
    let mut cards = merge_cards(sessions).into_iter();
    let first = cards.next();
    if cards.next().is_some() {
        return Err("Multiple pending merge cards.".into());
    }
    Ok(first)
}

/// The pending merge material is the first resource advertised by its card.
pub(crate) fn pending_merge_media(
    card: &chirograph::PortableCardV1,
) -> Result<ContentHash, String> {
    card.media
        .first()
        .copied()
        .ok_or_else(|| "The material card must advertise a media resource.".into())
}

/// Decode the owner's pending merge representation and enforce its version.
pub(crate) fn decode_pending_merge(bytes: &[u8]) -> Result<PendingMergeV1, String> {
    let pending: PendingMergeV1 = serde_json::from_slice(bytes)
        .map_err(|error| format!("Invalid pending merge material: {error}"))?;
    if pending.schema != crate::wire::PENDING_MERGE_SCHEMA {
        return Err("The pending merge material has another schema.".into());
    }
    Ok(pending)
}

/// A conflict's stable key: its part, as the wire spells it.
pub fn conflict_key(part: &Part) -> String {
    serde_json::to_string(part).expect("a part encodes")
}

/// A part in words, naming a node by its label in `graph` when shown.
fn part_words(part: &Part, graph: &GraphModel) -> String {
    let name = |id: &uuid::Uuid| {
        let key = id.to_string();
        graph
            .nodes
            .iter()
            .find(|node| node.key == key)
            .map(|node| node.label.clone())
            .unwrap_or_else(|| key[..8].to_string())
    };
    match part {
        Part::Node(id) => name(id),
        Part::Title(id) => format!("Title of {}", name(id)),
        Part::Url(id) => format!("Address of {}", name(id)),
        Part::Tag(id, tag) => format!("Tag {tag} on {}", name(id)),
        Part::Body(id) => format!("Body of {}", name(id)),
        Part::Content(id) => format!("Content of {}", name(id)),
        Part::MediaType(id) => format!("Media type of {}", name(id)),
        Part::Nested(id) => format!("World of {}", name(id)),
        Part::Facet(id, facet) => format!("{facet} of {}", name(id)),
        Part::History(id) => format!("History of {}", name(id)),
        Part::Edges(from, to) => format!("Relations from {} to {}", name(from), name(to)),
        Part::Field(id) => format!("Field {}", &id.to_string()[..8]),
        Part::Coupling(id) => format!("Coupling {}", &id.to_string()[..8]),
        Part::ImportRecords => "Import records".to_string(),
        Part::Resource(id) => format!("Resource {}", &id.to_string()[..8]),
        Part::ResourceFacet(id, facet) => {
            format!("{facet} of resource {}", &id.to_string()[..8])
        },
        Part::ResourceEdges(from, to) => format!(
            "Resource relations from {} to {}",
            &from.to_string()[..8],
            &to.to_string()[..8]
        ),
        Part::ShownResource(id) => format!("Resource shown by {}", name(id)),
    }
}

/// A part's value in words: text as it is, a node by its title, anything
/// else as compact JSON; empty for absent.
fn value_words(value: Option<&Value>) -> String {
    match value {
        None | Some(Value::Null) => String::new(),
        Some(Value::String(text)) => text.clone(),
        Some(Value::Object(fields)) if fields.get("title").is_some_and(Value::is_string) => {
            fields["title"].as_str().unwrap_or_default().to_string()
        },
        Some(other) => other.to_string(),
    }
}

/// A choice in words, for a dropped choice's line.
fn choice_words(choice: &MergeChoice) -> String {
    match choice {
        MergeChoice::Source => "Source".to_string(),
        MergeChoice::Target => "Target".to_string(),
        MergeChoice::Hand(value) => format!("by hand, {:?}", value_words(Some(value))),
        MergeChoice::AddAsNew => "Add as new".to_string(),
    }
}

/// A wire choice as the view holds it: a value given by hand as its text.
pub fn merge_option(choice: &MergeChoice) -> MergeOption {
    match choice {
        MergeChoice::Source => MergeOption::Source,
        MergeChoice::Target => MergeOption::Target,
        MergeChoice::Hand(value) => MergeOption::Hand(value_words(Some(value))),
        MergeChoice::AddAsNew => MergeOption::AddAsNew,
    }
}

/// Take what the resident says of a re-planned merge into the choices a
/// panel holds: each carried choice, and no choice for one it dropped.
pub fn adopt_merge(chosen: &mut BTreeMap<String, MergeOption>, pending: &PendingMergeV1) {
    for dropped in &pending.dropped {
        chosen.remove(&conflict_key(&dropped.part));
    }
    for carried in &pending.carried {
        chosen.insert(conflict_key(&carried.part), merge_option(&carried.choice));
    }
}

/// The request for a pending merge's material: its card's first media, from
/// the cards [`card_requests`] read.
pub fn merge_material_request(
    sessions: &ProjectionSnapshot,
    cards: &HashMap<ContentHash, Vec<u8>>,
) -> Option<ResourceRequest> {
    let instance = *merge_cards(sessions).first()?;
    let card: PortableCardV1 =
        serde_json::from_slice(cards.get(&offer(sessions, instance)?.resource)?).ok()?;
    Some(ResourceRequest {
        session: sessions.session.clone(),
        resource: *card.media.first()?,
    })
}

/// The pending merge as the view shows it, with the choices made so far by
/// conflict key. `None` clears it.
pub fn fill_merge(
    model: &mut MereViewModel,
    pending: Option<&PendingMergeV1>,
    chosen: &BTreeMap<String, MergeOption>,
) {
    model.merge = pending.map(|pending| MergeModel {
        source: pending.source,
        source_name: pending.source_name.clone(),
        target_name: pending.target_name.clone(),
        dropped: pending
            .dropped
            .iter()
            .map(|dropped| {
                format!(
                    "{}: {} ({})",
                    part_words(&dropped.part, &model.graph),
                    choice_words(&dropped.choice),
                    dropped.reason
                )
            })
            .collect(),
        conflicts: pending
            .conflicts
            .iter()
            .map(|conflict| {
                let key = conflict_key(&conflict.part);
                MergeConflictEntry {
                    label: part_words(&conflict.part, &model.graph),
                    base: value_words(conflict.base.as_ref()),
                    source: value_words(conflict.source.as_ref()),
                    target: value_words(conflict.target.as_ref()),
                    add_as_new: conflict.add_as_new,
                    chosen: chosen.get(&key).cloned(),
                    key,
                }
            })
            .collect(),
    });
}

/// The resolve or cancel intent for a merge request on `pending`'s card. A
/// hand value reads as text where the conflict's values are text, and as
/// JSON otherwise (text when it does not parse); empty is absent. `None` for
/// a choice (the host keeps those) or an unresolved Apply.
pub fn merge_intent(
    request: &MereViewRequest,
    sessions: &ProjectionSnapshot,
    pending: &PendingMergeV1,
    chosen: &BTreeMap<String, MergeOption>,
) -> Option<IntentInvocation> {
    let MereViewRequest::Merge(request) = request else {
        return None;
    };
    let (intent, choices) = match request {
        MergeRequest::Choose(..) => return None,
        MergeRequest::Cancel => (CANCEL_MERGE_INTENT, Vec::new()),
        MergeRequest::Apply => {
            let mut choices = Vec::with_capacity(pending.conflicts.len());
            for conflict in &pending.conflicts {
                let choice = match chosen.get(&conflict_key(&conflict.part))? {
                    MergeOption::Source => MergeChoice::Source,
                    MergeOption::Target => MergeChoice::Target,
                    MergeOption::AddAsNew => MergeChoice::AddAsNew,
                    MergeOption::Hand(text) if text.is_empty() => MergeChoice::Hand(Value::Null),
                    MergeOption::Hand(text) => {
                        let textual = [&conflict.source, &conflict.target, &conflict.base]
                            .into_iter()
                            .flatten()
                            .any(Value::is_string);
                        MergeChoice::Hand(if textual {
                            Value::String(text.clone())
                        } else {
                            serde_json::from_str(text).unwrap_or(Value::String(text.clone()))
                        })
                    },
                };
                choices.push(MergeChoiceV1 {
                    part: conflict.part.clone(),
                    choice,
                });
            }
            (RESOLVE_MERGE_INTENT, choices)
        },
    };
    let (target, _) = items(sessions, MERGE_SOURCE)
        .find(|(_, source)| Uuid::parse_str(&source.id).ok() == Some(*pending.source.as_uuid()))?;
    Some(IntentInvocation {
        session: sessions.session.clone(),
        target,
        observed_epoch: sessions.scene.epoch,
        observed_revision: sessions.scene.revision,
        intent: intent.into(),
        payload: serde_json::to_vec(&MergeResolutionV1::new(pending.source, choices))
            .expect("MergeResolutionV1 always serializes"),
    })
}

/// What the view shows when the route declines a step: its reason.
pub fn declined(result: &IntentResult) -> Option<String> {
    match result {
        IntentResult::Accepted => None,
        IntentResult::Rejected { reason } => Some(reason.clone()),
        IntentResult::Stale { .. } => Some("The sessions changed; read them again.".to_string()),
    }
}

/// The items whose source is `adapter`'s, in order.
fn items<'a>(
    snapshot: &'a ProjectionSnapshot,
    adapter: &'a str,
) -> impl Iterator<Item = (InstanceId, &'a SourceRef)> + 'a {
    snapshot
        .scene
        .active_items_in_order()
        .into_iter()
        .filter_map(move |(instance, item)| {
            let source = snapshot.scene.tables.sources[item.source.0 as usize].as_ref()?;
            (source.adapter == adapter).then_some((instance, source))
        })
}

fn offer(snapshot: &ProjectionSnapshot, instance: InstanceId) -> Option<&PresentationOffer> {
    let binding = snapshot
        .presentation
        .bindings
        .iter()
        .find(|binding| binding.instance == instance)?;
    snapshot.presentation.offers.get(&binding.key)?.first()
}

/// The mere's own card, which offers minting.
fn mere_card(sessions: &ProjectionSnapshot) -> InstanceId {
    items(sessions, SESSIONS_SOURCE)
        .find(|(_, source)| Uuid::parse_str(&source.id).is_err())
        .map_or(InstanceId(0), |(instance, _)| instance)
}

/// A generic readable record over owner-disclosed facets. It preserves unknown
/// fields as text without interpreting or rewriting a domain's sealed values.
pub fn add_node_facets(
    cards: &mut HashMap<String, PortableCardV1>,
    facets: &kernel::graph::NodeFacetStore,
) -> Result<(), String> {
    for (key, card) in cards {
        let id = Uuid::parse_str(key).map_err(|e| e.to_string())?;
        if let Some(facets) = facets.facets_of(&id) {
            let mut values = Vec::new();
            for (_, value) in facets.iter() {
                record_values("", value, &mut values);
            }
            values.append(&mut card.values);
            card.values = values;
        }
    }
    Ok(())
}
fn record_values(
    prefix: &str,
    value: &serde_json::Value,
    values: &mut Vec<chirograph::CardValueV1>,
) {
    match value {
        serde_json::Value::Object(fields) => {
            for (label, value) in fields {
                let label = label.replace('_', " ");
                let label = if prefix.is_empty() {
                    label
                } else {
                    format!("{prefix} · {label}")
                };
                record_values(&label, value, values);
            }
        },
        serde_json::Value::Array(items) => {
            for (index, value) in items.iter().enumerate() {
                record_values(&format!("{prefix} {}", index + 1), value, values);
            }
        },
        serde_json::Value::Null => {},
        value => values.push(chirograph::CardValueV1 {
            label: prefix.into(),
            value: value
                .as_str()
                .map(str::to_owned)
                .unwrap_or_else(|| value.to_string()),
        }),
    }
}

#[cfg(test)]
mod tests {
    use chirograph::CardValueV1;

    use super::*;

    fn card(values: &[(&str, &str)]) -> PortableCardV1 {
        PortableCardV1 {
            title: "Codicil blake3:f612a".into(),
            values: values
                .iter()
                .map(|(label, value)| CardValueV1 {
                    label: (*label).into(),
                    value: (*value).into(),
                })
                .collect(),
            badges: Vec::new(),
            media: Vec::new(),
        }
    }

    #[test]
    fn a_codicil_card_names_the_one_session_it_was_saved_from() {
        let one = "00000000-0000-0000-0000-000000000001";
        let two = "00000000-0000-0000-0000-000000000002";
        assert_eq!(
            archive_session(&card(&[("Codicil", "blake3:f612a"), ("Session", one)])),
            Some(SessionId(Uuid::parse_str(one).unwrap()))
        );
        assert_eq!(
            archive_session(&card(&[("Session", one), ("Session", one)])),
            Some(SessionId(Uuid::parse_str(one).unwrap())),
            "the same session twice is still one"
        );
        assert_eq!(
            archive_session(&card(&[("Session", one), ("Session", two)])),
            None,
            "a composition names no single session"
        );
        assert_eq!(archive_session(&card(&[("Codicil", "blake3:f612a")])), None);
        assert_eq!(archive_session(&card(&[("Session", "not a uuid")])), None);
    }
}
