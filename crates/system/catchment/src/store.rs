// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The reservoir's index: which meres exist, which domain each holds, and the
//! owner's access decisions per application. The device resident owns it;
//! applications reach meres through admitted routes and never open these
//! files (reservoir plan, V1).

use std::collections::BTreeMap;

use identity::PersonaId;
use muniment::{Backend, JsonSlots};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::catalog::{
    DomainId, MERE_RECORD_SCHEMA, MereId, MereRecord, RESERVOIR_INDEX_SCHEMA, RESERVOIR_INDEX_SLOT,
    ReservoirError,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ReservoirIndex<A> {
    pub(crate) schema: String,
    pub(crate) persona: PersonaId,
    pub(crate) meres: BTreeMap<DomainId, MereRecord>,
    #[serde(default = "BTreeMap::new", skip_serializing_if = "BTreeMap::is_empty")]
    pub(crate) access: BTreeMap<MereId, BTreeMap<String, MereApplicationAccess<A>>>,
}

/// A recorded owner decision for one application and one mere. Explicit
/// access defaults on; ambient crossing defaults off. A denial blocks both
/// routes without erasing the independent ambient decision.
///
/// `A` is who recorded it. The reservoir stores it and never reads it, so the
/// owner of the attribution vocabulary names it (Pandect: the journal's
/// `Author`), and the stored bytes are that type's own.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MereApplicationAccess<A> {
    pub denied: bool,
    pub ambient: bool,
    pub recorded_at_ms: u64,
    pub author: A,
}

/// A persona's reservoir index, persisted through a muniment backend.
pub struct ReservoirStore<B, A> {
    slots: JsonSlots<B>,
    index: ReservoirIndex<A>,
}

impl<B: Backend, A: Clone + Serialize + DeserializeOwned> ReservoirStore<B, A> {
    /// Open `persona`'s reservoir. An empty backend yields an empty reservoir;
    /// a stored index is verified before it is trusted.
    pub async fn open(backend: B, persona: PersonaId) -> Result<Self, ReservoirError> {
        let slots = JsonSlots::new(backend);
        let index = match slots
            .load::<ReservoirIndex<A>>(RESERVOIR_INDEX_SLOT)
            .await?
        {
            None => ReservoirIndex {
                schema: RESERVOIR_INDEX_SCHEMA.to_string(),
                persona,
                meres: BTreeMap::new(),
                access: BTreeMap::new(),
            },
            Some(index) => {
                verify_index(&index, persona)?;
                index
            },
        };
        Ok(Self { slots, index })
    }

    pub fn persona(&self) -> PersonaId {
        self.index.persona
    }

    /// Every mere, ordered by domain.
    pub fn meres(&self) -> impl Iterator<Item = &MereRecord> {
        self.index.meres.values()
    }

    pub fn get(&self, domain: &DomainId) -> Option<&MereRecord> {
        self.index.meres.get(domain)
    }

    pub fn get_by_id(&self, id: MereId) -> Option<&MereRecord> {
        self.index.meres.values().find(|record| record.id == id)
    }

    /// Recorded application decisions, in mere/application order.
    pub fn access(&self) -> impl Iterator<Item = (MereId, &str, &MereApplicationAccess<A>)> {
        self.index.access.iter().flat_map(|(mere, apps)| {
            apps.iter()
                .map(move |(app, access)| (*mere, app.as_str(), access))
        })
    }

    pub fn application_access(&self, mere: MereId, app: &str) -> Option<&MereApplicationAccess<A>> {
        self.index.access.get(&mere)?.get(app)
    }

    /// Commit a decision in the reservoir's existing muniment index. Neither
    /// an unknown mere nor an ambiguous application label may acquire policy.
    /// Failed writes leave the in-memory decision unchanged.
    pub async fn set_application_access(
        &mut self,
        mere: MereId,
        app: String,
        access: MereApplicationAccess<A>,
    ) -> Result<(), ReservoirError> {
        if self.get_by_id(mere).is_none() || !valid_application(&app) {
            return Err(ReservoirError::Corrupt(
                "access decision names an unknown mere or invalid application".into(),
            ));
        }
        let mut candidate = self.index.clone();
        candidate
            .access
            .entry(mere)
            .or_default()
            .insert(app, access);
        self.slots.save(RESERVOIR_INDEX_SLOT, &candidate).await?;
        self.index = candidate;
        Ok(())
    }

    /// The domain's mere, created if the reservoir has none yet. Returns the
    /// record and whether this call created it. A domain has exactly one mere.
    pub async fn ensure(
        &mut self,
        domain: DomainId,
        created_at_ms: u64,
    ) -> Result<(MereRecord, bool), ReservoirError> {
        if let Some(existing) = self.index.meres.get(&domain) {
            return Ok((existing.clone(), false));
        }
        let record = MereRecord {
            schema: MERE_RECORD_SCHEMA.to_string(),
            id: MereId::derive(self.index.persona, &domain),
            persona: self.index.persona,
            domain: domain.clone(),
            created_at_ms,
        };
        self.index.meres.insert(domain.clone(), record.clone());
        if let Err(error) = self.slots.save(RESERVOIR_INDEX_SLOT, &self.index).await {
            // Keep memory equal to what is stored.
            self.index.meres.remove(&domain);
            return Err(error.into());
        }
        Ok((record, true))
    }
}

fn verify_index<A>(index: &ReservoirIndex<A>, persona: PersonaId) -> Result<(), ReservoirError> {
    if index.schema != RESERVOIR_INDEX_SCHEMA {
        return Err(ReservoirError::Corrupt(format!(
            "unknown schema {:?}",
            index.schema
        )));
    }
    if index.persona != persona {
        return Err(ReservoirError::PersonaMismatch {
            expected: persona,
            found: index.persona,
        });
    }
    for (domain, record) in &index.meres {
        if record.schema != MERE_RECORD_SCHEMA
            || &record.domain != domain
            || record.persona != persona
            || record.id != MereId::derive(persona, domain)
        {
            return Err(ReservoirError::Corrupt(format!(
                "the record for domain {domain} does not match its identity"
            )));
        }
    }
    for (mere, apps) in &index.access {
        if !index.meres.values().any(|record| record.id == *mere)
            || apps.keys().any(|app| !valid_application(app))
        {
            return Err(ReservoirError::Corrupt(
                "access decisions name an unknown mere or invalid application".into(),
            ));
        }
    }
    Ok(())
}

fn valid_application(app: &str) -> bool {
    !app.is_empty()
        && app.len() <= 256
        && app == app.trim()
        && app == app.to_ascii_lowercase()
        && !app.chars().any(char::is_control)
}

#[cfg(test)]
#[path = "store_tests.rs"]
mod tests;
