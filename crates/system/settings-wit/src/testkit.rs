// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The fixture owner, gate and call script every settings binding is
//! checked with: natively over a channel, and from a component guest.

use mere_surface_api::settings::{
    CommittedSettingsWrite, RevisionedSettingSpec, RevisionedSettingsProvider,
    RevisionedSettingsSnapshot, RevisionedSettingsWriteRequest, RevisionedWriteOutcome,
    SettingControl, SettingMovement, SettingMutability, SettingOption, SettingScope,
    SettingSecurity, SettingSpec, SettingValue, SettingsDomainRevision, SettingsOwnerIncarnation,
    SettingsRevisionDomain, SettingsRevisionGuard, SettingsStorageFailure, SettingsUnavailable,
    SettingsValidationFailure,
};
use workbench::SettingsRef;

use crate::bindings::{Failure, Guard, SettingsSnapshot, Value, WriteOutcome, WriteRequest};
use crate::{Access, SettingsCalls, SettingsGate};

/// The page the owner serves and callers may write.
pub const APPEARANCE: &str = "product/appearance";
/// A page the owner serves and the gate keeps read-only.
pub const ABOUT: &str = "product/about";
/// A page the gate refuses outright.
pub const PRIVATE: &str = "product/private";
/// The fixture owner's incarnation.
pub const INCARNATION: &str = "fixture-run-1";
/// A setting whose persistence always fails.
pub const FAILING: &str = "backup.target";

/// One owner with three revision domains on [`APPEARANCE`] and a read-only
/// [`ABOUT`] page, covering every control and value kind.
#[derive(Clone, Debug)]
pub struct FixtureOwner {
    pub chrome: bool,
    pub zoom: f64,
    pub theme: String,
    pub history: i64,
    pub downloads: String,
    pub appearance_rev: u64,
    pub history_rev: u64,
    pub downloads_rev: u64,
}

impl Default for FixtureOwner {
    fn default() -> Self {
        Self {
            chrome: false,
            zoom: 1.0,
            theme: "light".into(),
            history: 500,
            downloads: "/downloads".into(),
            appearance_rev: 4,
            history_rev: 1,
            downloads_rev: 2,
        }
    }
}

fn row(
    id: &str,
    label: &str,
    scope: SettingScope,
    security: SettingSecurity,
    control: SettingControl,
    value: SettingValue,
) -> SettingSpec {
    SettingSpec {
        id: id.into(),
        label: label.into(),
        scope,
        movement: SettingMovement::LocalOnly,
        mutability: SettingMutability::Live,
        security,
        control,
        value,
    }
}

impl FixtureOwner {
    fn domain_of(setting: &str) -> Option<&'static str> {
        match setting {
            "chrome.enabled" | "ui.zoom" | "theme.name" => Some("appearance"),
            "history.limit" => Some("history"),
            "downloads.directory" | FAILING => Some("downloads"),
            _ => None,
        }
    }

    fn revision(&self, domain: &str) -> u64 {
        match domain {
            "appearance" => self.appearance_rev,
            "history" => self.history_rev,
            _ => self.downloads_rev,
        }
    }

    pub fn guard(&self, domain: &str) -> SettingsRevisionGuard {
        SettingsRevisionGuard {
            owner: SettingsOwnerIncarnation(INCARNATION.into()),
            domain: SettingsRevisionDomain(domain.into()),
            revision: self.revision(domain),
        }
    }

    fn specs(&self, reference: &str) -> Vec<SettingSpec> {
        use SettingScope::*;
        use SettingSecurity::*;
        if reference == ABOUT {
            return vec![row(
                "about.version",
                "Version",
                Process,
                Ordinary,
                SettingControl::Text,
                SettingValue::Text("1.0 — ✓".into()),
            )];
        }
        vec![
            row(
                "chrome.enabled",
                "Chrome enabled",
                Application,
                Ordinary,
                SettingControl::Toggle,
                SettingValue::Boolean(self.chrome),
            ),
            row(
                "ui.zoom",
                "Zoom",
                Device,
                Ordinary,
                SettingControl::Number {
                    min: Some(0.25),
                    max: Some(4.0),
                    step: None,
                },
                SettingValue::Number(self.zoom),
            ),
            row(
                "theme.name",
                "Theme",
                Persona,
                Ordinary,
                SettingControl::Choice {
                    options: vec![
                        SettingOption {
                            value: "light".into(),
                            label: "Light".into(),
                        },
                        SettingOption {
                            value: "dark".into(),
                            label: "Dark".into(),
                        },
                    ],
                },
                SettingValue::Text(self.theme.clone()),
            ),
            row(
                "history.limit",
                "History limit",
                Persona,
                Private,
                SettingControl::Number {
                    min: None,
                    max: None,
                    step: Some(1.0),
                },
                SettingValue::Integer(self.history),
            ),
            row(
                "downloads.directory",
                "Download directory",
                Application,
                Ordinary,
                SettingControl::Text,
                SettingValue::Text(self.downloads.clone()),
            ),
            row(
                FAILING,
                "Backup target",
                Device,
                SecretReference,
                SettingControl::Text,
                SettingValue::Text("vault:backup".into()),
            ),
        ]
    }
}

impl RevisionedSettingsProvider for FixtureOwner {
    fn snapshot(
        &self,
        reference: &SettingsRef,
    ) -> Result<RevisionedSettingsSnapshot, SettingsUnavailable> {
        let domains: &[&str] = match reference.0.as_str() {
            APPEARANCE => &["appearance", "history", "downloads"],
            ABOUT => &[],
            _ => {
                return Err(SettingsUnavailable {
                    message: "no such settings page".into(),
                });
            },
        };
        let specs = self
            .specs(&reference.0)
            .into_iter()
            .map(|spec| {
                let guard = self.guard(Self::domain_of(&spec.id).unwrap_or("about"));
                RevisionedSettingSpec { spec, guard }
            })
            .collect();
        Ok(RevisionedSettingsSnapshot {
            reference: reference.clone(),
            owner: SettingsOwnerIncarnation(INCARNATION.into()),
            revisions: domains
                .iter()
                .map(|d| SettingsDomainRevision {
                    domain: SettingsRevisionDomain((*d).into()),
                    revision: self.revision(d),
                })
                .collect(),
            specs,
        })
    }

    fn apply_revisioned(
        &mut self,
        request: RevisionedSettingsWriteRequest,
    ) -> RevisionedWriteOutcome {
        let request_id = request.request_id.clone();
        let invalid = |message: &str| RevisionedWriteOutcome::Validation {
            request_id: request_id.clone(),
            failure: SettingsValidationFailure {
                setting_id: request.setting_id.clone(),
                message: message.into(),
            },
        };
        if request.reference.0 != APPEARANCE {
            return RevisionedWriteOutcome::Unavailable {
                request_id,
                reason: SettingsUnavailable {
                    message: "page is not writable".into(),
                },
            };
        }
        let Some(domain) = Self::domain_of(&request.setting_id) else {
            return invalid("unknown setting");
        };
        let current = self.guard(domain);
        if request.expected != current {
            return RevisionedWriteOutcome::Conflict {
                request_id,
                current,
            };
        }
        let stored = match (request.setting_id.as_str(), request.value) {
            ("chrome.enabled", SettingValue::Boolean(b)) => {
                self.chrome = b;
                SettingValue::Boolean(b)
            },
            ("ui.zoom", SettingValue::Number(z)) if (0.25..=4.0).contains(&z) => {
                // The owner stores zoom as f32: the canonical value comes back.
                self.zoom = f64::from(z as f32);
                SettingValue::Number(self.zoom)
            },
            ("ui.zoom", SettingValue::Number(_)) => return invalid("zoom out of range"),
            ("theme.name", SettingValue::Text(t)) if t == "light" || t == "dark" => {
                self.theme = t.clone();
                SettingValue::Text(t)
            },
            ("history.limit", SettingValue::Integer(i)) => {
                self.history = i;
                SettingValue::Integer(i)
            },
            ("downloads.directory", SettingValue::Text(t)) => {
                self.downloads = t.clone();
                SettingValue::Text(t)
            },
            (FAILING, SettingValue::Text(_)) => {
                return RevisionedWriteOutcome::Storage {
                    request_id,
                    failure: SettingsStorageFailure {
                        message: "backup store is offline".into(),
                    },
                };
            },
            _ => return invalid("wrong value kind"),
        };
        match domain {
            "appearance" => self.appearance_rev += 1,
            "history" => self.history_rev += 1,
            _ => self.downloads_rev += 1,
        }
        RevisionedWriteOutcome::Committed(CommittedSettingsWrite {
            request_id,
            reference: request.reference,
            setting_id: request.setting_id,
            value: stored,
            guard: self.guard(domain),
        })
    }
}

/// Refuses everything on [`PRIVATE`] and writes to [`ABOUT`], and records
/// every question it was asked.
#[derive(Clone, Debug, Default)]
pub struct FixtureGate {
    pub asked: Vec<(String, Access)>,
}

impl SettingsGate for FixtureGate {
    fn admit(&mut self, reference: &SettingsRef, access: Access) -> Result<(), String> {
        self.asked.push((reference.0.clone(), access));
        match (reference.0.as_str(), access) {
            (PRIVATE, _) => Err("no grant for product/private".into()),
            (ABOUT, Access::Write) => Err("product/about is read-only to this caller".into()),
            _ => Ok(()),
        }
    }
}

/// How many calls [`run`] makes; the gate is asked once for each.
pub const CALLS: usize = 16;

fn guard_of(snapshot: &SettingsSnapshot, setting: &str) -> Guard {
    snapshot
        .specs
        .iter()
        .find(|g| g.spec.id == setting)
        .map(|g| g.guard.clone())
        .expect("setting in snapshot")
}

fn write(id: &str, reference: &str, setting: &str, expected: Guard, value: Value) -> WriteRequest {
    WriteRequest {
        request_id: id.into(),
        reference: reference.into(),
        setting_id: setting.into(),
        expected,
        value,
    }
}

/// The shared call script: every outcome the contract has, the gate's
/// refusals, and values at their edges. Each entry is a call's label and
/// its result as written by `Debug`.
pub fn run(calls: &mut impl SettingsCalls) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let log = |out: &mut Vec<(String, String)>, label: &str, result: String| {
        out.push((label.to_string(), result))
    };

    let first = calls.snapshot(APPEARANCE.into());
    log(&mut out, "snapshot", format!("{first:?}"));
    let Ok(Ok(first)) = first else {
        return out;
    };
    log(
        &mut out,
        "snapshot missing",
        format!("{:?}", calls.snapshot("product/missing".into())),
    );
    log(
        &mut out,
        "snapshot private",
        format!("{:?}", calls.snapshot(PRIVATE.into())),
    );
    let about = calls.snapshot(ABOUT.into());
    log(&mut out, "snapshot about", format!("{about:?}"));

    let chrome = guard_of(&first, "chrome.enabled");
    let mut step = |label: &str, request: WriteRequest| {
        let result = calls.write(request);
        log(&mut out, label, format!("{result:?}"));
    };
    step(
        "commit",
        write(
            "w1",
            APPEARANCE,
            "chrome.enabled",
            chrome.clone(),
            Value::Boolean(true),
        ),
    );
    step(
        "stale guard",
        write(
            "w2",
            APPEARANCE,
            "chrome.enabled",
            chrome.clone(),
            Value::Boolean(false),
        ),
    );
    step(
        "other domain",
        write(
            "w3",
            APPEARANCE,
            "downloads.directory",
            guard_of(&first, "downloads.directory"),
            Value::Text("/tmp/ä — ✓".into()),
        ),
    );
    step(
        "integer edge",
        write(
            "w4",
            APPEARANCE,
            "history.limit",
            guard_of(&first, "history.limit"),
            Value::Integer(i64::MIN),
        ),
    );
    let foreign = Guard {
        owner: "fixture-run-0".into(),
        ..guard_of(&first, "history.limit")
    };
    step(
        "foreign owner",
        write(
            "w5",
            APPEARANCE,
            "history.limit",
            foreign,
            Value::Integer(i64::MAX),
        ),
    );
    let appearance = Guard {
        revision: chrome.revision + 1,
        ..chrome.clone()
    };
    step(
        "wrong kind",
        write(
            "w6",
            APPEARANCE,
            "ui.zoom",
            appearance.clone(),
            Value::Text("2".into()),
        ),
    );
    step(
        "nan",
        write(
            "w7",
            APPEARANCE,
            "ui.zoom",
            appearance.clone(),
            Value::Number(f64::NAN),
        ),
    );
    step(
        "canonical value",
        write("w8", APPEARANCE, "ui.zoom", appearance, Value::Number(1.1)),
    );
    step(
        "storage",
        write(
            "w9",
            APPEARANCE,
            FAILING,
            // "other domain" advanced this domain once.
            Guard {
                revision: guard_of(&first, FAILING).revision + 1,
                ..guard_of(&first, FAILING)
            },
            Value::Text("elsewhere".into()),
        ),
    );
    step(
        "unknown setting",
        write("w10", APPEARANCE, "nope", chrome, Value::Boolean(true)),
    );
    let about_guard = match &about {
        Ok(Ok(s)) => s.specs[0].guard.clone(),
        _ => guard_of(&first, "chrome.enabled"),
    };
    step(
        "denied write",
        write(
            "w11",
            ABOUT,
            "about.version",
            about_guard,
            Value::Text("2.0".into()),
        ),
    );

    let last = calls.snapshot(APPEARANCE.into());
    log(&mut out, "snapshot after", format!("{last:?}"));
    out
}

/// Separates an entry's label from its result in a participant's log line.
const SEPARATOR: char = '\u{1f}';

#[path = "testkit/outcomes.rs"]
mod outcomes;

pub use outcomes::log_line;

pub use outcomes::from_log;

pub use outcomes::assert_outcomes;

pub use outcomes::edge_snapshot;

pub use outcomes::edge_outcomes;

pub use outcomes::edge_failures;
