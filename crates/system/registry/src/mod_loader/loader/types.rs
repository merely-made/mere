// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use std::path::PathBuf;

use super::contributes::Contribution;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModType {
    Native,
    Wasm,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModStatus {
    Discovered,
    Loading,
    Active,
    Failed,
    Quarantined,
    Unloaded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ModCapability {
    Network,
    Filesystem,
    Identity,
    Clipboard,
    Exec,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModManifest {
    pub mod_id: String,
    pub display_name: String,
    pub mod_type: ModType,
    /// Dependency names other mods can require (`ProtocolRegistry`).
    pub provides: Vec<String>,
    /// Dependency names this mod needs, matched against `provides`.
    pub requires: Vec<String>,
    /// What this mod adds to the GUI, by kind and id. Defaults empty; set by
    /// the manifest reader or [`ModManifest::with_contributes`], not
    /// [`ModManifest::new`]. Activation is checked against it.
    pub contributes: Vec<Contribution>,
    pub capabilities: Vec<ModCapability>,
    /// Origin globs this mod's wasm binds as a DocumentScript (the "installed
    /// extension" auto-attach form). Empty = not a document-script mod. When
    /// non-empty, the mod's `module_path` is the component a host attaches over a
    /// page whose origin matches one of these globs (exact host or `*.suffix`).
    /// Defaults empty; set by the manifest reader, not [`ModManifest::new`].
    pub document_script_origins: Vec<String>,
}

impl ModManifest {
    pub fn new(
        mod_id: impl Into<String>,
        display_name: impl Into<String>,
        mod_type: ModType,
        provides: Vec<String>,
        requires: Vec<String>,
        capabilities: Vec<ModCapability>,
    ) -> Self {
        Self {
            mod_id: mod_id.into(),
            display_name: display_name.into(),
            mod_type,
            provides,
            requires,
            contributes: Vec::new(),
            capabilities,
            document_script_origins: Vec::new(),
        }
    }

    pub fn with_contributes(mut self, contributes: Vec<Contribution>) -> Self {
        self.contributes = contributes;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModDependencyError {
    DuplicateModId(String),
    MissingRequirement { mod_id: String, requirement: String },
    DependencyCycle(Vec<String>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModExtensionRecord {
    ProtocolScheme {
        scheme: String,
        previously_present: bool,
    },
    PresentationContentType {
        content_type: String,
        previous_presentation_id: Option<String>,
    },
    PresentationExtension {
        extension: String,
        previous_presentation_id: Option<String>,
    },
    PresentationReaches {
        presentation_id: String,
        previous: Option<inker::presentation::Reaches>,
    },
    Action {
        action_id: String,
    },
    IndexProvider {
        provider_id: String,
    },
    Lens {
        lens_id: String,
    },
    Theme {
        theme_id: String,
    },
    WasmRuntime {
        mod_id: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModUnloadError {
    UnknownMod(String),
    NotActive(String),
    DependencyActive {
        mod_id: String,
        dependent_id: String,
    },
    ExtensionRemovalFailed {
        mod_id: String,
        reason: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModActivationError {
    reason: String,
    applied_records: Vec<ModExtensionRecord>,
    undeclared: Option<ModExtensionRecord>,
}

impl ModActivationError {
    pub fn failed(reason: impl Into<String>) -> Self {
        Self::rollback(reason, Vec::new())
    }

    pub fn rollback(reason: impl Into<String>, applied_records: Vec<ModExtensionRecord>) -> Self {
        Self {
            reason: reason.into(),
            applied_records,
            undeclared: None,
        }
    }

    /// The mod registered `record`, which its manifest's `contributes` does
    /// not declare. `applied_records` are everything it registered, for
    /// rollback.
    pub fn undeclared(
        mod_id: &str,
        record: ModExtensionRecord,
        applied_records: Vec<ModExtensionRecord>,
    ) -> Self {
        Self {
            reason: format!("mod '{mod_id}' registered {record:?} but does not declare it"),
            applied_records,
            undeclared: Some(record),
        }
    }

    pub fn reason(&self) -> &str {
        &self.reason
    }

    /// The record that broke the manifest's declaration, if that was the cause.
    pub fn undeclared_record(&self) -> Option<&ModExtensionRecord> {
        self.undeclared.as_ref()
    }

    pub(super) fn into_parts(self) -> (String, Vec<ModExtensionRecord>) {
        (self.reason, self.applied_records)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WasmModSource {
    pub module_path: PathBuf,
    pub manifest_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModLoadPathError {
    UnsupportedModPath(PathBuf),
    MissingManifest(PathBuf),
    InvalidManifest { path: PathBuf, reason: String },
    InvalidCapability { capability: String },
    InvalidWasmBinary(PathBuf),
    Io { path: PathBuf, reason: String },
    DuplicateModId(String),
}
