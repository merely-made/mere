// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Between `mere-surface-api`'s settings types and the generated ones.

use mere_surface_api::settings::{
    CommittedSettingsWrite, RevisionedSettingSpec, RevisionedSettingsSnapshot,
    RevisionedSettingsWriteRequest, RevisionedWriteOutcome, SettingControl, SettingMovement,
    SettingMutability, SettingOption, SettingScope, SettingSecurity, SettingSpec, SettingValue,
    SettingsDomainRevision, SettingsOwnerIncarnation, SettingsRevisionDomain,
    SettingsRevisionGuard, SettingsStorageFailure, SettingsUnavailable, SettingsValidationFailure,
};
use workbench::SettingsRef;

use crate::bindings as w;

fn scope(s: SettingScope) -> w::Scope {
    match s {
        SettingScope::Process => w::Scope::Process,
        SettingScope::Device => w::Scope::Device,
        SettingScope::Persona => w::Scope::Persona,
        SettingScope::Application => w::Scope::Application,
        SettingScope::SessionDataspace => w::Scope::SessionDataspace,
        SettingScope::ArtifactEntity => w::Scope::ArtifactEntity,
        SettingScope::GovernedPlace => w::Scope::GovernedPlace,
    }
}

fn native_scope(s: w::Scope) -> SettingScope {
    match s {
        w::Scope::Process => SettingScope::Process,
        w::Scope::Device => SettingScope::Device,
        w::Scope::Persona => SettingScope::Persona,
        w::Scope::Application => SettingScope::Application,
        w::Scope::SessionDataspace => SettingScope::SessionDataspace,
        w::Scope::ArtifactEntity => SettingScope::ArtifactEntity,
        w::Scope::GovernedPlace => SettingScope::GovernedPlace,
    }
}

fn movement(m: SettingMovement) -> w::Movement {
    match m {
        SettingMovement::LocalOnly => w::Movement::LocalOnly,
        SettingMovement::PersonaSynced => w::Movement::PersonaSynced,
        SettingMovement::TravelsWithArtifact => w::Movement::TravelsWithArtifact,
        SettingMovement::GovernedReplication => w::Movement::GovernedReplication,
    }
}

fn native_movement(m: w::Movement) -> SettingMovement {
    match m {
        w::Movement::LocalOnly => SettingMovement::LocalOnly,
        w::Movement::PersonaSynced => SettingMovement::PersonaSynced,
        w::Movement::TravelsWithArtifact => SettingMovement::TravelsWithArtifact,
        w::Movement::GovernedReplication => SettingMovement::GovernedReplication,
    }
}

fn mutability(m: SettingMutability) -> w::Mutability {
    match m {
        SettingMutability::StartupOnly => w::Mutability::StartupOnly,
        SettingMutability::Live => w::Mutability::Live,
        SettingMutability::RestartRequired => w::Mutability::RestartRequired,
    }
}

fn native_mutability(m: w::Mutability) -> SettingMutability {
    match m {
        w::Mutability::StartupOnly => SettingMutability::StartupOnly,
        w::Mutability::Live => SettingMutability::Live,
        w::Mutability::RestartRequired => SettingMutability::RestartRequired,
    }
}

fn security(s: SettingSecurity) -> w::Security {
    match s {
        SettingSecurity::Ordinary => w::Security::Ordinary,
        SettingSecurity::Private => w::Security::Private,
        SettingSecurity::SecretReference => w::Security::SecretReference,
    }
}

fn native_security(s: w::Security) -> SettingSecurity {
    match s {
        w::Security::Ordinary => SettingSecurity::Ordinary,
        w::Security::Private => SettingSecurity::Private,
        w::Security::SecretReference => SettingSecurity::SecretReference,
    }
}

fn value(v: &SettingValue) -> w::Value {
    match v {
        SettingValue::Boolean(b) => w::Value::Boolean(*b),
        SettingValue::Integer(i) => w::Value::Integer(*i),
        SettingValue::Number(n) => w::Value::Number(*n),
        SettingValue::Text(t) => w::Value::Text(t.clone()),
    }
}

fn native_value(v: w::Value) -> SettingValue {
    match v {
        w::Value::Boolean(b) => SettingValue::Boolean(b),
        w::Value::Integer(i) => SettingValue::Integer(i),
        w::Value::Number(n) => SettingValue::Number(n),
        w::Value::Text(t) => SettingValue::Text(t),
    }
}

fn control(c: &SettingControl) -> w::Control {
    match c {
        SettingControl::Toggle => w::Control::Toggle,
        SettingControl::Choice { options } => w::Control::Choice(
            options
                .iter()
                .map(|o| w::ChoiceOption {
                    value: o.value.clone(),
                    label: o.label.clone(),
                })
                .collect(),
        ),
        SettingControl::Number { min, max, step } => w::Control::Number(w::NumberRange {
            min: *min,
            max: *max,
            step: *step,
        }),
        SettingControl::Text => w::Control::Text,
    }
}

fn native_control(c: w::Control) -> SettingControl {
    match c {
        w::Control::Toggle => SettingControl::Toggle,
        w::Control::Choice(options) => SettingControl::Choice {
            options: options
                .into_iter()
                .map(|o| SettingOption {
                    value: o.value,
                    label: o.label,
                })
                .collect(),
        },
        w::Control::Number(r) => SettingControl::Number {
            min: r.min,
            max: r.max,
            step: r.step,
        },
        w::Control::Text => SettingControl::Text,
    }
}

fn spec(s: &SettingSpec) -> w::Spec {
    w::Spec {
        id: s.id.clone(),
        label: s.label.clone(),
        scope: scope(s.scope),
        movement: movement(s.movement),
        mutability: mutability(s.mutability),
        security: security(s.security),
        control: control(&s.control),
        value: value(&s.value),
    }
}

fn native_spec(s: w::Spec) -> SettingSpec {
    SettingSpec {
        id: s.id,
        label: s.label,
        scope: native_scope(s.scope),
        movement: native_movement(s.movement),
        mutability: native_mutability(s.mutability),
        security: native_security(s.security),
        control: native_control(s.control),
        value: native_value(s.value),
    }
}

fn guard(g: &SettingsRevisionGuard) -> w::Guard {
    w::Guard {
        owner: g.owner.0.clone(),
        domain: g.domain.0.clone(),
        revision: g.revision,
    }
}

fn native_guard(g: w::Guard) -> SettingsRevisionGuard {
    SettingsRevisionGuard {
        owner: SettingsOwnerIncarnation(g.owner),
        domain: SettingsRevisionDomain(g.domain),
        revision: g.revision,
    }
}

pub fn wire_snapshot(s: &RevisionedSettingsSnapshot) -> w::SettingsSnapshot {
    w::SettingsSnapshot {
        reference: s.reference.0.clone(),
        owner: s.owner.0.clone(),
        revisions: s
            .revisions
            .iter()
            .map(|r| w::DomainRevision {
                domain: r.domain.0.clone(),
                revision: r.revision,
            })
            .collect(),
        specs: s
            .specs
            .iter()
            .map(|g| w::GuardedSpec {
                spec: spec(&g.spec),
                guard: guard(&g.guard),
            })
            .collect(),
    }
}

pub fn native_snapshot(s: w::SettingsSnapshot) -> RevisionedSettingsSnapshot {
    RevisionedSettingsSnapshot {
        reference: SettingsRef(s.reference),
        owner: SettingsOwnerIncarnation(s.owner),
        revisions: s
            .revisions
            .into_iter()
            .map(|r| SettingsDomainRevision {
                domain: SettingsRevisionDomain(r.domain),
                revision: r.revision,
            })
            .collect(),
        specs: s
            .specs
            .into_iter()
            .map(|g| RevisionedSettingSpec {
                spec: native_spec(g.spec),
                guard: native_guard(g.guard),
            })
            .collect(),
    }
}

pub fn wire_request(r: &RevisionedSettingsWriteRequest) -> w::WriteRequest {
    w::WriteRequest {
        request_id: r.request_id.clone(),
        reference: r.reference.0.clone(),
        setting_id: r.setting_id.clone(),
        expected: guard(&r.expected),
        value: value(&r.value),
    }
}

pub fn native_request(r: w::WriteRequest) -> RevisionedSettingsWriteRequest {
    RevisionedSettingsWriteRequest {
        request_id: r.request_id,
        reference: SettingsRef(r.reference),
        setting_id: r.setting_id,
        expected: native_guard(r.expected),
        value: native_value(r.value),
    }
}

pub fn wire_outcome(o: &RevisionedWriteOutcome) -> w::WriteOutcome {
    match o {
        RevisionedWriteOutcome::Committed(c) => w::WriteOutcome::Committed(w::CommittedWrite {
            request_id: c.request_id.clone(),
            reference: c.reference.0.clone(),
            setting_id: c.setting_id.clone(),
            value: value(&c.value),
            guard: guard(&c.guard),
        }),
        RevisionedWriteOutcome::Conflict {
            request_id,
            current,
        } => w::WriteOutcome::Conflict(w::Conflict {
            request_id: request_id.clone(),
            current: guard(current),
        }),
        RevisionedWriteOutcome::Unavailable { request_id, reason } => {
            w::WriteOutcome::Unavailable(w::Refusal {
                request_id: request_id.clone(),
                message: reason.message.clone(),
            })
        },
        RevisionedWriteOutcome::Validation {
            request_id,
            failure,
        } => w::WriteOutcome::Validation(w::ValidationFailure {
            request_id: request_id.clone(),
            setting_id: failure.setting_id.clone(),
            message: failure.message.clone(),
        }),
        RevisionedWriteOutcome::Storage {
            request_id,
            failure,
        } => w::WriteOutcome::Storage(w::Refusal {
            request_id: request_id.clone(),
            message: failure.message.clone(),
        }),
    }
}

/// The owner's outcome for a wire one. The owner contract has no refusal
/// for authority, so `denied` becomes `Unavailable` with a `denied:` message,
/// which claims no change.
pub fn native_outcome(o: w::WriteOutcome) -> RevisionedWriteOutcome {
    match o {
        w::WriteOutcome::Committed(c) => {
            RevisionedWriteOutcome::Committed(CommittedSettingsWrite {
                request_id: c.request_id,
                reference: SettingsRef(c.reference),
                setting_id: c.setting_id,
                value: native_value(c.value),
                guard: native_guard(c.guard),
            })
        },
        w::WriteOutcome::Conflict(c) => RevisionedWriteOutcome::Conflict {
            request_id: c.request_id,
            current: native_guard(c.current),
        },
        w::WriteOutcome::Unavailable(r) => RevisionedWriteOutcome::Unavailable {
            request_id: r.request_id,
            reason: SettingsUnavailable { message: r.message },
        },
        w::WriteOutcome::Validation(v) => RevisionedWriteOutcome::Validation {
            request_id: v.request_id,
            failure: SettingsValidationFailure {
                setting_id: v.setting_id,
                message: v.message,
            },
        },
        w::WriteOutcome::Storage(r) => RevisionedWriteOutcome::Storage {
            request_id: r.request_id,
            failure: SettingsStorageFailure { message: r.message },
        },
        w::WriteOutcome::Denied(r) => RevisionedWriteOutcome::Unavailable {
            request_id: r.request_id,
            reason: SettingsUnavailable {
                message: format!("denied: {}", r.message),
            },
        },
    }
}
