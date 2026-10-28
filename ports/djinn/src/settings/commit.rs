// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Load-modify-commit for the owner settings file.
//!
//! Pair, unpair and the resident's dial hint all edit one file from separate
//! processes. A plain [`OwnerSettings::save`] replaces the file atomically but
//! protects no earlier read, so a writer that loaded before another committed
//! would silently undo that commit. [`LoadedSettings::commit`] writes only if the
//! file still holds the bytes that were loaded; [`OwnerSettings::update`] reloads
//! and reapplies the edit when it does not.

use std::path::Path;

use pandect::{Replacement, replace_bytes_if_unchanged};

use super::{OwnerSettings, OwnerSettingsError};

/// Commits one [`OwnerSettings::update`] tries before reporting the conflict.
const COMMIT_ATTEMPTS: usize = 3;

/// Settings with the bytes they were parsed from (`None` when the file did not
/// exist), so a commit can tell whether anyone has written since.
#[derive(Clone, Debug)]
pub struct LoadedSettings {
    pub settings: OwnerSettings,
    baseline: Option<Vec<u8>>,
}

impl LoadedSettings {
    /// Write the settings, serialized as [`OwnerSettings::save`] does, if the
    /// file still holds what was loaded. Otherwise refuse with
    /// [`OwnerSettingsError::Conflict`] and leave the file alone; the caller
    /// reloads and reapplies its change.
    pub fn commit(self, path: &Path) -> Result<(), OwnerSettingsError> {
        let document = self.settings.document(path)?;
        match replace_bytes_if_unchanged(path, self.baseline.as_deref(), &document) {
            Ok(Replacement::Installed) => Ok(()),
            Ok(Replacement::Conflict) => Err(OwnerSettingsError::Conflict {
                path: path.display().to_string(),
            }),
            Err(error) => Err(OwnerSettingsError::File {
                path: path.display().to_string(),
                message: error.to_string(),
            }),
        }
    }
}

impl OwnerSettings {
    /// [`load`](Self::load), keeping the file's bytes for a later
    /// [`commit`](LoadedSettings::commit).
    pub fn load_for_commit(path: &Path) -> Result<LoadedSettings, OwnerSettingsError> {
        let fail = |message: String| OwnerSettingsError::File {
            path: path.display().to_string(),
            message,
        };
        let baseline = match std::fs::read(path) {
            Ok(bytes) => Some(bytes),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
            Err(error) => return Err(fail(error.to_string())),
        };
        let settings = match &baseline {
            Some(bytes) => {
                serde_json::from_slice(bytes).map_err(|error| fail(error.to_string()))?
            },
            None => Self::default(),
        };
        Ok(LoadedSettings { settings, baseline })
    }

    /// Load, apply `edit`, and commit it if it changed anything. On a conflict
    /// the file is reloaded and `edit` runs again on the fresh settings, up to
    /// three attempts, so `edit` must be safe to repeat. An edit that errors or
    /// changes nothing writes nothing.
    pub fn update<T, E>(
        path: &Path,
        mut edit: impl FnMut(&mut OwnerSettings) -> Result<T, E>,
    ) -> Result<T, E>
    where
        E: From<OwnerSettingsError>,
    {
        let mut attempt = 1;
        loop {
            let mut loaded = Self::load_for_commit(path)?;
            let before = loaded.settings.clone();
            let outcome = edit(&mut loaded.settings)?;
            if loaded.settings == before {
                return Ok(outcome);
            }
            match loaded.commit(path) {
                Ok(()) => return Ok(outcome),
                Err(OwnerSettingsError::Conflict { .. }) if attempt < COMMIT_ATTEMPTS => {
                    attempt += 1;
                },
                Err(error) => return Err(error.into()),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    const FIRST: &str = "http://127.0.0.1:3341/";
    const SECOND: &str = "http://127.0.0.1:3342/";

    fn settings_file(directory: &tempfile::TempDir) -> PathBuf {
        directory.path().join("settings").join("default.json")
    }

    fn relays(path: &Path) -> Vec<String> {
        OwnerSettings::load(path).unwrap().coop_relay_urls
    }

    /// A rival writer that commits one relay between a load and its commit.
    fn rival_adds(path: &Path, relay: &str) {
        let mut rival = OwnerSettings::load_for_commit(path).unwrap();
        rival.settings.coop_relay_urls.push(relay.into());
        rival.commit(path).unwrap();
    }

    #[test]
    fn two_writers_from_one_load_refuse_the_second_until_it_reloads() {
        let directory = tempfile::tempdir().unwrap();
        let path = settings_file(&directory);
        OwnerSettings::default().save(&path).unwrap();

        let mut first = OwnerSettings::load_for_commit(&path).unwrap();
        let mut second = OwnerSettings::load_for_commit(&path).unwrap();
        first.settings.coop_relay_urls.push(FIRST.into());
        second.settings.coop_relay_urls.push(SECOND.into());

        first.commit(&path).unwrap();
        let error = second.commit(&path).unwrap_err();
        assert!(
            matches!(error, OwnerSettingsError::Conflict { .. }),
            "a commit from a stale load must be refused, not written over the first: {error}"
        );
        assert_eq!(relays(&path), [FIRST], "the refusal left the file alone");

        let mut retry = OwnerSettings::load_for_commit(&path).unwrap();
        retry.settings.coop_relay_urls.push(SECOND.into());
        retry.commit(&path).unwrap();
        assert_eq!(relays(&path), [FIRST, SECOND]);
    }

    #[test]
    fn a_missing_file_commits_only_while_it_is_still_missing() {
        let directory = tempfile::tempdir().unwrap();
        let path = settings_file(&directory);

        let mut first = OwnerSettings::load_for_commit(&path).unwrap();
        let mut second = OwnerSettings::load_for_commit(&path).unwrap();
        first.settings.coop_relay_urls.push(FIRST.into());
        second.settings.coop_relay_urls.push(SECOND.into());

        first.commit(&path).unwrap();
        assert_eq!(relays(&path), [FIRST]);
        let error = second.commit(&path).unwrap_err();
        assert!(
            matches!(error, OwnerSettingsError::Conflict { .. }),
            "{error}"
        );
        assert_eq!(relays(&path), [FIRST]);
    }

    #[test]
    fn update_reloads_and_reapplies_its_edit_after_a_conflict() {
        let directory = tempfile::tempdir().unwrap();
        let path = settings_file(&directory);
        OwnerSettings::default().save(&path).unwrap();
        let mut attempts = 0;

        OwnerSettings::update(&path, |settings| -> Result<(), OwnerSettingsError> {
            attempts += 1;
            if attempts == 1 {
                rival_adds(&path, FIRST);
            }
            settings.coop_relay_urls.push(SECOND.into());
            Ok(())
        })
        .unwrap();

        assert_eq!(attempts, 2);
        assert_eq!(relays(&path), [FIRST, SECOND]);
    }

    #[test]
    fn update_reports_the_conflict_once_three_attempts_have_lost() {
        let directory = tempfile::tempdir().unwrap();
        let path = settings_file(&directory);
        OwnerSettings::default().save(&path).unwrap();
        let mut attempts = 0;

        let error = OwnerSettings::update(&path, |settings| -> Result<(), OwnerSettingsError> {
            attempts += 1;
            rival_adds(&path, &format!("http://127.0.0.1:{}/", 3350 + attempts));
            settings.coop_relay_urls.push(SECOND.into());
            Ok(())
        })
        .unwrap_err();

        assert!(
            matches!(error, OwnerSettingsError::Conflict { .. }),
            "{error}"
        );
        assert_eq!(attempts, 3);
        assert!(!relays(&path).contains(&SECOND.to_string()));
    }

    #[test]
    fn update_writes_nothing_for_an_edit_that_changes_nothing_or_fails() {
        let directory = tempfile::tempdir().unwrap();
        let path = settings_file(&directory);

        OwnerSettings::update(&path, |_| -> Result<(), OwnerSettingsError> { Ok(()) }).unwrap();
        assert!(!path.exists(), "an unchanged edit must not create the file");

        let error = OwnerSettings::update(&path, |settings| {
            settings.coop_relay_urls.push(FIRST.into());
            Err::<(), _>(OwnerSettingsError::NotHex { value: "x".into() })
        })
        .unwrap_err();
        assert!(matches!(error, OwnerSettingsError::NotHex { .. }));
        assert!(!path.exists(), "a failed edit must not be committed");
    }
}
