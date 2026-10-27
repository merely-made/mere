// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Query authoring lifecycle, independent of a UI, executor, graph or store.
//! The owner validates each request and acknowledges persistence explicitly.
//! Rebinding retires UI interest; it does not cancel an accepted owner write.
use super::QueryViewSpec;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Scope {
    pub session: Uuid,
    pub persona: String,
}
impl From<(Uuid, String)> for Scope {
    fn from((session, persona): (Uuid, String)) -> Self {
        Self { session, persona }
    }
}
impl Scope {
    pub fn pair(&self) -> (Uuid, String) {
        (self.session, self.persona.clone())
    }
}

/// Correlates a completion to one instance, binding and operation. A reopened
/// draft in the same session also gets a new binding generation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RequestId {
    pub instance: Uuid,
    pub generation: u64,
    pub sequence: u64,
    pub attempt: u64,
}
impl RequestId {
    pub fn same_operation(self, other: Self) -> bool {
        self.instance == other.instance
            && self.generation == other.generation
            && self.sequence == other.sequence
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SaveRequest {
    pub id: RequestId,
    pub scope: Scope,
    pub owner: Option<Uuid>,
    pub expected: Option<QueryViewSpec>,
    pub spec: QueryViewSpec,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Effect {
    Save(SaveRequest),
    /// Retry the same owner operation, including its previously created IDs.
    Retry {
        id: RequestId,
        scope: Scope,
    },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SaveResult {
    Committed {
        owner: Uuid,
        access: Option<Uuid>,
    },
    /// The owner retains the pending write. The editor must not submit a new
    /// create operation or claim that the failed acknowledgement rolled back.
    Unconfirmed {
        reason: String,
    },
    /// The owner refused the request before accepting a pending write.
    Refused {
        reason: String,
    },
    Stale,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Completion {
    pub id: RequestId,
    pub scope: Scope,
    pub result: SaveResult,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SaveState {
    Idle,
    Saving(RequestId),
    Unconfirmed(RequestId),
}

pub struct Editor {
    instance: Uuid,
    generation: u64,
    sequence: u64,
    scope: Option<Scope>,
    owner: Option<Uuid>,
    expected: Option<QueryViewSpec>,
    draft: Option<QueryViewSpec>,
    submitted: Option<QueryViewSpec>,
    state: SaveState,
    status: String,
}
impl Editor {
    pub fn new(instance: Uuid) -> Self {
        Self {
            instance,
            generation: 0,
            sequence: 0,
            scope: None,
            owner: None,
            expected: None,
            draft: None,
            submitted: None,
            state: SaveState::Idle,
            status: String::new(),
        }
    }
    pub fn scope(&self) -> Option<&Scope> {
        self.scope.as_ref()
    }
    pub fn owner(&self) -> Option<Uuid> {
        self.owner
    }
    pub fn draft(&self) -> Option<&QueryViewSpec> {
        self.draft.as_ref()
    }
    pub fn state(&self) -> &SaveState {
        &self.state
    }
    pub fn status(&self) -> &str {
        &self.status
    }
    pub fn notice(&mut self, message: impl Into<String>) {
        self.status = message.into();
    }
    pub fn editable(&self) -> bool {
        self.draft.is_some() && self.state == SaveState::Idle
    }
    pub fn draft_mut(&mut self) -> Option<&mut QueryViewSpec> {
        self.editable().then_some(())?;
        self.draft.as_mut()
    }
    pub fn open(&mut self, scope: Scope, owner: Option<Uuid>, spec: QueryViewSpec) {
        self.retire();
        self.scope = Some(scope);
        self.owner = owner;
        self.expected = owner.map(|_| spec.clone());
        self.draft = Some(spec);
        self.status = "Query draft opened".into();
    }
    /// The caller invokes this on removal or owner change. Late completions
    /// cannot resurrect the retired draft, even if the same owner reopens.
    pub fn retire(&mut self) {
        self.generation = self
            .generation
            .checked_add(1)
            .expect("query binding generation exhausted");
        self.scope = None;
        self.owner = None;
        self.expected = None;
        self.draft = None;
        self.submitted = None;
        self.state = SaveState::Idle;
        self.status = "Query draft retired after owner change".into();
    }
    pub fn save(&mut self, spec: QueryViewSpec) -> Result<Effect, String> {
        if !self.editable() {
            return Err("Open an editable query draft before saving".into());
        }
        spec.validate()?;
        self.sequence = self
            .sequence
            .checked_add(1)
            .expect("query operation sequence exhausted");
        let id = RequestId {
            instance: self.instance,
            generation: self.generation,
            sequence: self.sequence,
            attempt: 0,
        };
        let request = SaveRequest {
            id,
            scope: self.scope.clone().unwrap(),
            owner: self.owner,
            expected: self.expected.clone(),
            spec: spec.clone(),
        };
        self.draft = Some(spec.clone());
        self.submitted = Some(spec);
        self.state = SaveState::Saving(id);
        self.status = "Saving query view…".into();
        Ok(Effect::Save(request))
    }
    pub fn retry(&mut self) -> Option<Effect> {
        let SaveState::Unconfirmed(mut id) = self.state else {
            return None;
        };
        id.attempt = id
            .attempt
            .checked_add(1)
            .expect("query retry attempt exhausted");
        self.state = SaveState::Saving(id);
        self.status = "Retrying query save…".into();
        Some(Effect::Retry {
            id,
            scope: self.scope.clone()?,
        })
    }
    /// Returns the explicitly acknowledged appearance the host may open. A
    /// successful save need not create or open an appearance in another host.
    pub fn complete(&mut self, completion: Completion) -> Option<Uuid> {
        let id = match self.state {
            SaveState::Saving(id) | SaveState::Unconfirmed(id) => id,
            SaveState::Idle => return None,
        };
        if completion.id != id || self.scope.as_ref() != Some(&completion.scope) {
            return None;
        }
        match completion.result {
            SaveResult::Committed { owner, access } => {
                self.owner = Some(owner);
                self.expected = self.submitted.take();
                self.state = SaveState::Idle;
                self.status = "Query view saved".into();
                access
            },
            SaveResult::Unconfirmed { reason } => {
                self.state = SaveState::Unconfirmed(id);
                self.status = format!("Query save unconfirmed: {reason}. Retry the pending save.");
                None
            },
            SaveResult::Refused { reason } => {
                self.state = SaveState::Idle;
                self.submitted = None;
                self.status = reason;
                None
            },
            SaveResult::Stale => {
                self.retire();
                None
            },
        }
    }
}

#[cfg(test)]
mod tests;
