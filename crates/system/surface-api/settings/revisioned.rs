/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Opt-in compare-and-commit settings contract.
//!
//! A provider implementing this contract must validate the supplied guard,
//! persist the new value, and publish the next guard as one owner-controlled
//! atomic operation. An adapter that calls `SettingsProvider::describe`, then
//! later calls `apply`, cannot provide this guarantee: another writer may act
//! between those calls, and arbitrary existing storage cannot be made
//! transactional by wrapping it. The older [`super::SettingsProvider`]
//! remains available for source compatibility, but its writes are unguarded.

use workbench::SettingsRef;

use super::{SettingSpec, SettingValue};

/// Identifies one lifetime of an owner of settings.
///
/// Owners issue a fresh incarnation when their state is recreated, even if a
/// revision counter starts again at the same number. Values are opaque and
/// must be stable for the lifetime they identify.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct SettingsOwnerIncarnation(pub String);

/// Names the scope whose settings advance together.
///
/// The provider chooses this boundary. A write advances the revision for the
/// named domain; unrelated domains may retain independent revision sequences.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct SettingsRevisionDomain(pub String);

/// Owner-issued version of a settings revision domain.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct SettingsRevisionGuard {
    pub owner: SettingsOwnerIncarnation,
    pub domain: SettingsRevisionDomain,
    pub revision: u64,
}

/// One entry in a coherent snapshot's revision vector.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct SettingsDomainRevision {
    pub domain: SettingsRevisionDomain,
    pub revision: u64,
}

/// One setting and the guard that was current when it was described.
#[derive(Clone, Debug, PartialEq)]
pub struct RevisionedSettingSpec {
    pub spec: SettingSpec,
    pub guard: SettingsRevisionGuard,
}

/// A coherent read of one settings reference, including all its revision
/// domains.
///
/// `revisions` is the owner's revision vector observed with the specs. Each
/// spec carries a guard for the domain that owns its value. Multiple domains
/// may advance independently; the vector is not itself a global write
/// revision.
#[derive(Clone, Debug, PartialEq)]
pub struct RevisionedSettingsSnapshot {
    pub reference: SettingsRef,
    pub owner: SettingsOwnerIncarnation,
    pub revisions: Vec<SettingsDomainRevision>,
    pub specs: Vec<RevisionedSettingSpec>,
}

/// A correlated request to change one setting using a previously read guard.
#[derive(Clone, Debug, PartialEq)]
pub struct RevisionedSettingsWriteRequest {
    /// Stable caller-generated identity used to correlate delayed responses.
    pub request_id: String,
    pub reference: SettingsRef,
    pub setting_id: String,
    pub expected: SettingsRevisionGuard,
    pub value: SettingValue,
}

/// Owner-confirmed committed write. The returned guard is the published
/// revision after the value was durably accepted.
/// `value` is the owner's canonical stored value, which may differ from the
/// requested value after a documented conversion (for example, `f64` to `f32`).
/// Clients adopt this value rather than assuming the request was stored verbatim.
#[derive(Clone, Debug, PartialEq)]
pub struct CommittedSettingsWrite {
    pub request_id: String,
    pub reference: SettingsRef,
    pub setting_id: String,
    pub value: SettingValue,
    pub guard: SettingsRevisionGuard,
}

/// Why the requested settings source is currently unavailable.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SettingsUnavailable {
    pub message: String,
}

/// A value rejected by the setting owner's validation rules.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SettingsValidationFailure {
    pub setting_id: String,
    pub message: String,
}

/// A storage failure while committing the requested value.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SettingsStorageFailure {
    pub message: String,
}

/// Result of processing a revision-guarded write request.
///
/// Only `Committed` confirms a value and revision advance. A conflict returns
/// the owner's current guard; all other outcomes make no claim that a value or
/// revision changed.
#[derive(Clone, Debug, PartialEq)]
pub enum RevisionedWriteOutcome {
    Committed(CommittedSettingsWrite),
    Conflict {
        request_id: String,
        current: SettingsRevisionGuard,
    },
    Unavailable {
        request_id: String,
        reason: SettingsUnavailable,
    },
    Validation {
        request_id: String,
        failure: SettingsValidationFailure,
    },
    Storage {
        request_id: String,
        failure: SettingsStorageFailure,
    },
}

/// Opt-in owner contract for coherent settings snapshots and guarded writes.
///
/// Implementations own their revision scope and must atomically compare the
/// full guard (owner incarnation, domain, and revision), validate the value,
/// persist it, and publish the resulting value and next guard. A stale or
/// foreign-incarnation request must not overwrite the current value. A failed
/// validation or persistence operation must not publish a revision advance.
///
/// This trait intentionally says nothing about scheduling, runtime component
/// identity, or how a host transports delayed requests. The caller retains the
/// request ID until it receives the corresponding outcome.
pub trait RevisionedSettingsProvider {
    /// Read one coherent snapshot for `reference`, covering every revision domain
    /// represented by its specs.
    fn snapshot(
        &self,
        reference: &SettingsRef,
    ) -> Result<RevisionedSettingsSnapshot, SettingsUnavailable>;

    /// Process one compare-and-commit request at the owner.
    fn apply_revisioned(
        &mut self,
        request: RevisionedSettingsWriteRequest,
    ) -> RevisionedWriteOutcome;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::{
        SettingControl, SettingMovement, SettingMutability, SettingScope, SettingSecurity,
        SettingsError, SettingsProvider,
    };

    const REF: &str = "product/appearance";

    struct Owner {
        incarnation: SettingsOwnerIncarnation,
        revision: u64,
        downloads_revision: u64,
        enabled: bool,
        download_directory: String,
        fail_persist: bool,
    }

    impl Owner {
        fn guard(&self, domain: &str) -> SettingsRevisionGuard {
            SettingsRevisionGuard {
                owner: self.incarnation.clone(),
                domain: SettingsRevisionDomain(domain.into()),
                revision: match domain {
                    "appearance" => self.revision,
                    "downloads" => self.downloads_revision,
                    _ => 0,
                },
            }
        }

        fn specs(&self) -> Vec<RevisionedSettingSpec> {
            vec![
                RevisionedSettingSpec {
                    spec: SettingSpec {
                        id: "chrome.enabled".into(),
                        label: "Chrome enabled".into(),
                        scope: SettingScope::Application,
                        movement: SettingMovement::LocalOnly,
                        mutability: SettingMutability::Live,
                        security: SettingSecurity::Ordinary,
                        control: SettingControl::Toggle,
                        value: SettingValue::Boolean(self.enabled),
                    },
                    guard: self.guard("appearance"),
                },
                RevisionedSettingSpec {
                    spec: SettingSpec {
                        id: "downloads.directory".into(),
                        label: "Download directory".into(),
                        scope: SettingScope::Application,
                        movement: SettingMovement::LocalOnly,
                        mutability: SettingMutability::Live,
                        security: SettingSecurity::Ordinary,
                        control: SettingControl::Text,
                        value: SettingValue::Text(self.download_directory.clone()),
                    },
                    guard: self.guard("downloads"),
                },
            ]
        }
    }

    impl RevisionedSettingsProvider for Owner {
        fn snapshot(
            &self,
            reference: &SettingsRef,
        ) -> Result<RevisionedSettingsSnapshot, SettingsUnavailable> {
            if reference.0 != REF {
                return Err(SettingsUnavailable {
                    message: "unsupported settings reference".into(),
                });
            }
            let guard = self.guard("appearance");
            Ok(RevisionedSettingsSnapshot {
                reference: reference.clone(),
                owner: self.incarnation.clone(),
                revisions: vec![
                    SettingsDomainRevision {
                        domain: SettingsRevisionDomain("appearance".into()),
                        revision: guard.revision,
                    },
                    SettingsDomainRevision {
                        domain: SettingsRevisionDomain("downloads".into()),
                        revision: self.downloads_revision,
                    },
                ],
                specs: self.specs(),
            })
        }

        fn apply_revisioned(
            &mut self,
            request: RevisionedSettingsWriteRequest,
        ) -> RevisionedWriteOutcome {
            let request_id = request.request_id.clone();
            if request.reference.0 != REF {
                return RevisionedWriteOutcome::Unavailable {
                    request_id,
                    reason: SettingsUnavailable {
                        message: "unsupported settings reference".into(),
                    },
                };
            }
            let target_domain = match request.setting_id.as_str() {
                "chrome.enabled" => "appearance",
                "downloads.directory" => "downloads",
                _ => {
                    return RevisionedWriteOutcome::Validation {
                        request_id,
                        failure: SettingsValidationFailure {
                            setting_id: request.setting_id,
                            message: "unknown setting".into(),
                        },
                    };
                },
            };
            let current = self.guard(target_domain);
            if request.expected != current {
                return RevisionedWriteOutcome::Conflict {
                    request_id,
                    current,
                };
            }
            let value = match (target_domain, request.value) {
                ("appearance", SettingValue::Boolean(enabled)) => SettingValue::Boolean(enabled),
                ("downloads", SettingValue::Text(directory)) => SettingValue::Text(directory),
                (_, _) => {
                    return RevisionedWriteOutcome::Validation {
                        request_id,
                        failure: SettingsValidationFailure {
                            setting_id: request.setting_id,
                            message: "expected a boolean".into(),
                        },
                    };
                },
            };
            if self.fail_persist {
                return RevisionedWriteOutcome::Storage {
                    request_id,
                    failure: SettingsStorageFailure {
                        message: "simulated persistence failure".into(),
                    },
                };
            }
            match &value {
                SettingValue::Boolean(enabled) => {
                    self.enabled = *enabled;
                    self.revision += 1;
                },
                SettingValue::Text(directory) => {
                    self.download_directory = directory.clone();
                    self.downloads_revision += 1;
                },
                _ => unreachable!("value was validated above"),
            }
            RevisionedWriteOutcome::Committed(CommittedSettingsWrite {
                request_id,
                reference: request.reference,
                setting_id: request.setting_id,
                value,
                guard: self.guard(target_domain),
            })
        }
    }

    // Models a request delivered now but processed after the owner has moved
    // on; it deliberately stores the read guard rather than rereading it.
    fn request(
        request_id: &str,
        guard: SettingsRevisionGuard,
        value: bool,
    ) -> RevisionedSettingsWriteRequest {
        request_for(
            request_id,
            "chrome.enabled",
            guard,
            SettingValue::Boolean(value),
        )
    }

    fn request_for(
        request_id: &str,
        setting_id: &str,
        guard: SettingsRevisionGuard,
        value: SettingValue,
    ) -> RevisionedSettingsWriteRequest {
        RevisionedSettingsWriteRequest {
            request_id: request_id.into(),
            reference: SettingsRef(REF.into()),
            setting_id: setting_id.into(),
            expected: guard,
            value,
        }
    }

    fn owner() -> Owner {
        Owner {
            incarnation: SettingsOwnerIncarnation("owner-run-1".into()),
            revision: 4,
            downloads_revision: 2,
            enabled: false,
            download_directory: "/downloads".into(),
            fail_persist: false,
        }
    }

    #[test]
    fn current_guard_commits_and_correlates_the_owner_result() {
        let mut owner = owner();
        let snapshot = owner.snapshot(&SettingsRef(REF.into())).unwrap();
        assert_eq!(snapshot.owner, snapshot.specs[0].guard.owner);
        assert_eq!(snapshot.owner, snapshot.specs[1].guard.owner);
        assert_eq!(snapshot.specs[0].guard.domain.0, "appearance");
        assert_eq!(snapshot.specs[1].guard.domain.0, "downloads");
        assert_eq!(snapshot.revisions.len(), 2);
        assert_eq!(
            snapshot.revisions[0].revision,
            snapshot.specs[0].guard.revision
        );
        assert_eq!(
            snapshot.revisions[1].revision,
            snapshot.specs[1].guard.revision
        );

        let outcome =
            owner.apply_revisioned(request("req-1", snapshot.specs[0].guard.clone(), true));
        let RevisionedWriteOutcome::Committed(committed) = outcome else {
            panic!("current guard should commit");
        };
        assert_eq!(committed.request_id, "req-1");
        assert_eq!(committed.value, SettingValue::Boolean(true));
        assert_eq!(committed.guard.revision, 5);
    }

    #[test]
    fn delayed_stale_guard_conflicts_without_overwriting_newer_value() {
        let mut owner = owner();
        let delayed_guard = owner.snapshot(&SettingsRef(REF.into())).unwrap().specs[0]
            .guard
            .clone();
        let current_guard = owner.guard("appearance");
        let committed = owner.apply_revisioned(request("newer", current_guard, true));
        assert!(matches!(committed, RevisionedWriteOutcome::Committed(_)));

        let outcome = owner.apply_revisioned(request("delayed", delayed_guard, false));
        assert!(
            matches!(outcome, RevisionedWriteOutcome::Conflict { request_id, .. } if request_id == "delayed")
        );
        assert!(owner.enabled);
        assert_eq!(owner.revision, 5);
    }

    #[test]
    fn unrelated_domain_commit_does_not_conflict_with_appearance_guard() {
        let mut owner = owner();
        let snapshot = owner.snapshot(&SettingsRef(REF.into())).unwrap();
        let appearance_guard = snapshot.specs[0].guard.clone();
        let downloads_guard = snapshot.specs[1].guard.clone();

        let download_outcome = owner.apply_revisioned(request_for(
            "download-change",
            "downloads.directory",
            downloads_guard,
            SettingValue::Text("/new-downloads".into()),
        ));
        assert!(matches!(
            download_outcome,
            RevisionedWriteOutcome::Committed(_)
        ));

        let appearance_outcome =
            owner.apply_revisioned(request("appearance-change", appearance_guard, true));
        assert!(matches!(
            appearance_outcome,
            RevisionedWriteOutcome::Committed(_)
        ));
        assert!(owner.enabled);
        assert_eq!(owner.revision, 5);
        assert_eq!(owner.downloads_revision, 3);
    }

    #[test]
    fn old_owner_incarnation_is_rejected_even_at_the_same_numeric_revision() {
        let mut owner = owner();
        let old = owner.guard("appearance");
        owner.incarnation = SettingsOwnerIncarnation("owner-run-2".into());
        assert_eq!(old.revision, owner.revision);
        let outcome = owner.apply_revisioned(request("old-owner", old, true));
        assert!(matches!(outcome, RevisionedWriteOutcome::Conflict { .. }));
        assert!(!owner.enabled);
        assert_eq!(owner.revision, 4);
    }

    #[test]
    fn failed_persistence_publishes_neither_value_nor_revision() {
        let mut owner = owner();
        owner.fail_persist = true;
        let guard = owner.guard("appearance");
        let outcome = owner.apply_revisioned(request("storage-failure", guard.clone(), true));
        assert!(matches!(outcome, RevisionedWriteOutcome::Storage { .. }));
        assert!(!owner.enabled);
        assert_eq!(owner.guard("appearance"), guard);
    }

    struct LegacyOwner {
        enabled: bool,
    }

    impl SettingsProvider for LegacyOwner {
        fn describe(&self, reference: &SettingsRef) -> Result<Vec<SettingSpec>, SettingsError> {
            if reference.0 != REF {
                return Err(SettingsError::UnsupportedReference(reference.clone()));
            }
            let mut spec = Owner {
                incarnation: SettingsOwnerIncarnation("unused".into()),
                revision: 0,
                downloads_revision: 0,
                enabled: self.enabled,
                download_directory: String::new(),
                fail_persist: false,
            }
            .specs()[0]
                .spec
                .clone();
            spec.value = SettingValue::Boolean(self.enabled);
            Ok(vec![spec])
        }

        fn apply(
            &mut self,
            reference: &SettingsRef,
            setting_id: &str,
            value: SettingValue,
        ) -> Result<(), SettingsError> {
            if reference.0 != REF {
                return Err(SettingsError::UnsupportedReference(reference.clone()));
            }
            if setting_id != "chrome.enabled" {
                return Err(SettingsError::UnknownSetting(setting_id.into()));
            }
            match value {
                SettingValue::Boolean(enabled) => {
                    self.enabled = enabled;
                    Ok(())
                },
                other => Err(SettingsError::InvalidValue {
                    setting_id: setting_id.into(),
                    message: format!("expected Boolean, got {other:?}"),
                }),
            }
        }
    }

    #[test]
    fn legacy_provider_contract_remains_usable_without_guard_claims() {
        let mut provider = LegacyOwner { enabled: false };
        let reference = SettingsRef(REF.into());
        provider
            .apply(&reference, "chrome.enabled", SettingValue::Boolean(true))
            .unwrap();
        assert_eq!(
            provider.describe(&reference).unwrap()[0].value,
            SettingValue::Boolean(true)
        );
    }
}
