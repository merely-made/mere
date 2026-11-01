// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! [`bridge!`](crate::bridge): conversions between [`crate::bindings`] and
//! another generator's types for the same interface (wit-bindgen in a
//! guest, Wasmtime's `bindgen!` in a host). Every generator names the types,
//! fields and cases alike, so one macro serves both.

/// Emits `mod bridge` in the calling crate, with `wire_*` (theirs to
/// [`crate::bindings`]) and `theirs_*` (the reverse) for the snapshot, its
/// failure, a write request and a write outcome. `$m` is the crate-rooted
/// path of the module holding their types.
#[macro_export]
macro_rules! bridge {
    ($m:path) => {
        #[allow(dead_code, unused_imports)]
        mod bridge {
            use $crate::bindings as w;
            use $m as t;

            $crate::__enum_mod!(
                scope,
                Scope,
                Process,
                Device,
                Persona,
                Application,
                SessionDataspace,
                ArtifactEntity,
                GovernedPlace
            );
            $crate::__enum_mod!(
                movement,
                Movement,
                LocalOnly,
                PersonaSynced,
                TravelsWithArtifact,
                GovernedReplication
            );
            $crate::__enum_mod!(mutability, Mutability, StartupOnly, Live, RestartRequired);
            $crate::__enum_mod!(security, Security, Ordinary, Private, SecretReference);

            pub mod wire {
                use super::{t, w};
                pub fn value(v: t::Value) -> w::Value {
                    match v {
                        t::Value::Boolean(b) => w::Value::Boolean(b),
                        t::Value::Integer(i) => w::Value::Integer(i),
                        t::Value::Number(n) => w::Value::Number(n),
                        t::Value::Text(s) => w::Value::Text(s),
                    }
                }
                pub fn control(c: t::Control) -> w::Control {
                    match c {
                        t::Control::Toggle => w::Control::Toggle,
                        t::Control::Choice(o) => w::Control::Choice(
                            o.into_iter()
                                .map(|o| w::ChoiceOption {
                                    value: o.value,
                                    label: o.label,
                                })
                                .collect(),
                        ),
                        t::Control::Number(r) => w::Control::Number(w::NumberRange {
                            min: r.min,
                            max: r.max,
                            step: r.step,
                        }),
                        t::Control::Text => w::Control::Text,
                    }
                }
                pub fn guard(g: t::Guard) -> w::Guard {
                    w::Guard {
                        owner: g.owner,
                        domain: g.domain,
                        revision: g.revision,
                    }
                }
                pub fn snapshot(s: t::SettingsSnapshot) -> w::SettingsSnapshot {
                    w::SettingsSnapshot {
                        reference: s.reference,
                        owner: s.owner,
                        revisions: s
                            .revisions
                            .into_iter()
                            .map(|r| w::DomainRevision {
                                domain: r.domain,
                                revision: r.revision,
                            })
                            .collect(),
                        specs: s
                            .specs
                            .into_iter()
                            .map(|g| w::GuardedSpec {
                                spec: w::Spec {
                                    id: g.spec.id,
                                    label: g.spec.label,
                                    scope: super::scope::wire(g.spec.scope),
                                    movement: super::movement::wire(g.spec.movement),
                                    mutability: super::mutability::wire(g.spec.mutability),
                                    security: super::security::wire(g.spec.security),
                                    control: control(g.spec.control),
                                    value: value(g.spec.value),
                                },
                                guard: guard(g.guard),
                            })
                            .collect(),
                    }
                }
                pub fn failure(f: t::Failure) -> w::Failure {
                    match f {
                        t::Failure::Denied(m) => w::Failure::Denied(m),
                        t::Failure::Unavailable(m) => w::Failure::Unavailable(m),
                    }
                }
                pub fn request(r: t::WriteRequest) -> w::WriteRequest {
                    w::WriteRequest {
                        request_id: r.request_id,
                        reference: r.reference,
                        setting_id: r.setting_id,
                        expected: guard(r.expected),
                        value: value(r.value),
                    }
                }
                fn refusal(r: t::Refusal) -> w::Refusal {
                    w::Refusal {
                        request_id: r.request_id,
                        message: r.message,
                    }
                }
                pub fn outcome(o: t::WriteOutcome) -> w::WriteOutcome {
                    match o {
                        t::WriteOutcome::Committed(c) => {
                            w::WriteOutcome::Committed(w::CommittedWrite {
                                request_id: c.request_id,
                                reference: c.reference,
                                setting_id: c.setting_id,
                                value: value(c.value),
                                guard: guard(c.guard),
                            })
                        },
                        t::WriteOutcome::Conflict(c) => w::WriteOutcome::Conflict(w::Conflict {
                            request_id: c.request_id,
                            current: guard(c.current),
                        }),
                        t::WriteOutcome::Unavailable(r) => w::WriteOutcome::Unavailable(refusal(r)),
                        t::WriteOutcome::Validation(v) => {
                            w::WriteOutcome::Validation(w::ValidationFailure {
                                request_id: v.request_id,
                                setting_id: v.setting_id,
                                message: v.message,
                            })
                        },
                        t::WriteOutcome::Storage(r) => w::WriteOutcome::Storage(refusal(r)),
                        t::WriteOutcome::Denied(r) => w::WriteOutcome::Denied(refusal(r)),
                    }
                }
            }

            pub mod theirs {
                use super::{t, w};
                pub fn value(v: w::Value) -> t::Value {
                    match v {
                        w::Value::Boolean(b) => t::Value::Boolean(b),
                        w::Value::Integer(i) => t::Value::Integer(i),
                        w::Value::Number(n) => t::Value::Number(n),
                        w::Value::Text(s) => t::Value::Text(s),
                    }
                }
                pub fn control(c: w::Control) -> t::Control {
                    match c {
                        w::Control::Toggle => t::Control::Toggle,
                        w::Control::Choice(o) => t::Control::Choice(
                            o.into_iter()
                                .map(|o| t::ChoiceOption {
                                    value: o.value,
                                    label: o.label,
                                })
                                .collect(),
                        ),
                        w::Control::Number(r) => t::Control::Number(t::NumberRange {
                            min: r.min,
                            max: r.max,
                            step: r.step,
                        }),
                        w::Control::Text => t::Control::Text,
                    }
                }
                pub fn guard(g: w::Guard) -> t::Guard {
                    t::Guard {
                        owner: g.owner,
                        domain: g.domain,
                        revision: g.revision,
                    }
                }
                pub fn snapshot(s: w::SettingsSnapshot) -> t::SettingsSnapshot {
                    t::SettingsSnapshot {
                        reference: s.reference,
                        owner: s.owner,
                        revisions: s
                            .revisions
                            .into_iter()
                            .map(|r| t::DomainRevision {
                                domain: r.domain,
                                revision: r.revision,
                            })
                            .collect(),
                        specs: s
                            .specs
                            .into_iter()
                            .map(|g| t::GuardedSpec {
                                spec: t::Spec {
                                    id: g.spec.id,
                                    label: g.spec.label,
                                    scope: super::scope::theirs(g.spec.scope),
                                    movement: super::movement::theirs(g.spec.movement),
                                    mutability: super::mutability::theirs(g.spec.mutability),
                                    security: super::security::theirs(g.spec.security),
                                    control: control(g.spec.control),
                                    value: value(g.spec.value),
                                },
                                guard: guard(g.guard),
                            })
                            .collect(),
                    }
                }
                pub fn failure(f: w::Failure) -> t::Failure {
                    match f {
                        w::Failure::Denied(m) => t::Failure::Denied(m),
                        w::Failure::Unavailable(m) => t::Failure::Unavailable(m),
                    }
                }
                pub fn request(r: w::WriteRequest) -> t::WriteRequest {
                    t::WriteRequest {
                        request_id: r.request_id,
                        reference: r.reference,
                        setting_id: r.setting_id,
                        expected: guard(r.expected),
                        value: value(r.value),
                    }
                }
                fn refusal(r: w::Refusal) -> t::Refusal {
                    t::Refusal {
                        request_id: r.request_id,
                        message: r.message,
                    }
                }
                pub fn outcome(o: w::WriteOutcome) -> t::WriteOutcome {
                    match o {
                        w::WriteOutcome::Committed(c) => {
                            t::WriteOutcome::Committed(t::CommittedWrite {
                                request_id: c.request_id,
                                reference: c.reference,
                                setting_id: c.setting_id,
                                value: value(c.value),
                                guard: guard(c.guard),
                            })
                        },
                        w::WriteOutcome::Conflict(c) => t::WriteOutcome::Conflict(t::Conflict {
                            request_id: c.request_id,
                            current: guard(c.current),
                        }),
                        w::WriteOutcome::Unavailable(r) => t::WriteOutcome::Unavailable(refusal(r)),
                        w::WriteOutcome::Validation(v) => {
                            t::WriteOutcome::Validation(t::ValidationFailure {
                                request_id: v.request_id,
                                setting_id: v.setting_id,
                                message: v.message,
                            })
                        },
                        w::WriteOutcome::Storage(r) => t::WriteOutcome::Storage(refusal(r)),
                        w::WriteOutcome::Denied(r) => t::WriteOutcome::Denied(refusal(r)),
                    }
                }
            }
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __enum_mod {
    ($m:ident, $ty:ident, $($case:ident),+) => {
        pub mod $m {
            use super::{t, w};
            pub fn wire(v: t::$ty) -> w::$ty {
                match v { $(t::$ty::$case => w::$ty::$case),+ }
            }
            pub fn theirs(v: w::$ty) -> t::$ty {
                match v { $(w::$ty::$case => t::$ty::$case),+ }
            }
        }
    };
}
