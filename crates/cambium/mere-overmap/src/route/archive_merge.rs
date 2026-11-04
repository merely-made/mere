// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

/// The optional immutable archive reading, through the same admitted route.
pub fn archive_request(descriptor: &EndpointDescriptor) -> Option<ProjectionRequest> {
    descriptor
        .projections
        .iter()
        .find(|offer| offer.request.session.0 == MERE_ARCHIVE)
        .map(|offer| offer.request.clone())
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
