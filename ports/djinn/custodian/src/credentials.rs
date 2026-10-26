// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The credential catalog's operations, moved out of Djinn's keeper with the
//! stores they act on. Results are the same public records Djinn's custody
//! route already answered (quarantine cards, older imports, CXF previews and
//! receipts), carried as JSON. Every act re-checks the exact item or import
//! revision under profile authority; the owner's decision is the
//! custodian's consent gate in front of this module, not a caller boolean.
//! CXF parsing runs here until import parsing is isolated (H1's open item).

use custody_wire::contract::CredentialOperation;
use personae::{PersonaId, ProfileId};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::host::HostCustody;

const MAX_CXF_BYTES: usize = 900 * 1024;

#[derive(Deserialize)]
struct AssignRequest {
    store: ProfileId,
    persona: uuid::Uuid,
    revision: String,
    profile: ProfileId,
}

#[derive(Deserialize)]
struct ReviewRequest {
    profile: ProfileId,
    store: ProfileId,
    persona: uuid::Uuid,
    item: chatelaine::ItemId,
    revision: String,
    action: ReviewAction,
}

#[derive(Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum ReviewAction {
    Accept,
    Delete,
}

#[derive(Serialize)]
struct Card {
    store: ProfileId,
    persona: uuid::Uuid,
    revision: String,
    accept_blocked: Option<String>,
    item: chatelaine::Item,
}

#[derive(Serialize)]
struct Unassigned {
    store: ProfileId,
    persona: uuid::Uuid,
    titles: Vec<String>,
    item_count: usize,
    revision: String,
}

pub(crate) fn run(custody: &HostCustody, operation: &CredentialOperation) -> Result<Value, String> {
    let host = custody.host().map_err(|r| r.0.to_string())?;
    let stores = custody.credential_stores();
    if stores.is_empty() {
        return Err("no credential catalog is configured".into());
    }
    let authority = |profile: &ProfileId| host.with_profile_authority(profile, || ()).map_err(|e| e.to_string());
    match operation {
        CredentialOperation::Quarantine { profile } => {
            let profile = ProfileId(profile.clone());
            authority(&profile)?;
            serde_json::to_value(quarantine(stores, &profile)?).map_err(|e| e.to_string())
        },
        CredentialOperation::UnassignedImports {} => {
            authority(&host.current_profile_id())?;
            serde_json::to_value(unassigned(stores)?).map_err(|e| e.to_string())
        },
        CredentialOperation::AssignImport { request } => {
            let request: AssignRequest = serde_json::from_value(request.clone()).map_err(|_| "invalid assignment request")?;
            authority(&request.profile)?;
            unassigned(stores)?
                .into_iter()
                .find(|i| i.store == request.store && i.persona == request.persona && i.revision == request.revision)
                .ok_or("Older import changed or was already assigned.")?;
            stores
                .get(&request.store)
                .ok_or("unknown credential store")?
                .assign_import(PersonaId::from_uuid(request.persona), &request.profile, &request.revision)
                .map_err(|e| e.to_string())?;
            Ok(json!({}))
        },
        CredentialOperation::Review { request } => {
            let request: ReviewRequest = serde_json::from_value(request.clone()).map_err(|_| "invalid review request")?;
            authority(&request.profile)?;
            let card = quarantine(stores, &request.profile)?
                .into_iter()
                .find(|c| c.store == request.store && c.persona == request.persona && c.item.id == request.item)
                .ok_or("Item is not quarantined under this persona.")?;
            if card.revision != request.revision {
                return Err("Item changed; review it again.".into());
            }
            if request.action == ReviewAction::Accept
                && let Some(reason) = card.accept_blocked
            {
                return Err(reason);
            }
            let resident = stores.get(&request.store).ok_or("unknown credential store")?;
            let persona = PersonaId::from_uuid(request.persona);
            if !resident.profile_personas(&request.profile).map_err(|e| e.to_string())?.contains(&persona) {
                return Err("Item namespace is not owned by this persona.".into());
            }
            resident
                .items(persona)
                .review(card.item.id, &request.revision, request.action == ReviewAction::Accept)
                .map_err(|e| e.to_string())?;
            Ok(json!({}))
        },
        CredentialOperation::CxfPreview { source } => {
            let import = read_cxf(&source.0)?;
            serde_json::to_value(import.preview()).map_err(|e| e.to_string())
        },
        CredentialOperation::CxfImport { source, destinations, preview_sha256 } => {
            let import = read_cxf(&source.0)?;
            if preview_sha256 != &import.preview().source_sha256 {
                return Err("CXF source changed; preview it again".into());
            }
            let destinations = serde_json::from_value(destinations.clone()).map_err(|_| "invalid destination choices")?;
            let resident = stores.get(&host.current_profile_id()).ok_or("credential authority is unavailable")?;
            match import.commit(destinations, resident, host.as_ref()) {
                Ok(receipt) => serde_json::to_value(receipt).map_err(|e| e.to_string()),
                Err(failure) => Err(format!(
                    "{}\n{}",
                    failure.reason,
                    serde_json::to_string_pretty(&failure.receipt).map_err(|e| e.to_string())?
                )),
            }
        },
    }
}

fn read_cxf(bytes: &[u8]) -> Result<castellan::cxf::Import, String> {
    if bytes.len() > MAX_CXF_BYTES {
        return Err("CXF source is larger than the custody frame allows".into());
    }
    castellan::cxf::Import::read(bytes.to_vec()).map_err(|e| e.to_string())
}

fn quarantine(stores: &std::collections::BTreeMap<ProfileId, castellan::resident::CastellanResident>, profile: &ProfileId) -> Result<Vec<Card>, String> {
    let mut cards = Vec::new();
    for (store, resident) in stores {
        for persona in resident.profile_personas(profile).map_err(|e| e.to_string())? {
            for item in resident.items(persona).list().map_err(|e| e.to_string())? {
                if item.state == chatelaine::ItemState::Quarantined {
                    cards.push(Card {
                        store: store.clone(),
                        persona: *persona.as_uuid(),
                        revision: castellan::review::revision(&item),
                        accept_blocked: castellan::review::accept_blocked(&item),
                        item,
                    });
                }
            }
        }
    }
    Ok(cards)
}

fn unassigned(stores: &std::collections::BTreeMap<ProfileId, castellan::resident::CastellanResident>) -> Result<Vec<Unassigned>, String> {
    let mut imports = Vec::new();
    for (store, resident) in stores {
        for (persona, items) in resident.unassigned_items().map_err(|e| e.to_string())? {
            imports.push(Unassigned {
                store: store.clone(),
                persona: *persona.as_uuid(),
                titles: items.iter().map(|i| i.title.clone()).collect(),
                item_count: items.len(),
                revision: castellan::review::namespace_revision(&items),
            });
        }
    }
    Ok(imports)
}
