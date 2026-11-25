// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Portable questions. Hosts resolve source identities anew under the current owner's policy.

use crate::{Profile, RecordIndex, RecordLink, TermKind};
use eidetic::pack::Pin;
use mere::profile::lens::{AdmittedAs, SelectionRecord};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

const SCHEMA: &str = "mere.saved-view/1";

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ViewQuery {
    Type { class: String },
    Incoming { record: String },
    Outgoing { record: String },
}

// The engine's selection record encodes multihashes as canonical CBOR byte
// strings. Use their standard text form in the portable JSON document.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SavedPin {
    id: String,
    version: String,
    digest: String,
}

impl From<&Pin> for SavedPin {
    fn from(pin: &Pin) -> Self {
        Self {
            id: pin.id.clone(),
            version: pin.version.clone(),
            digest: pin.digest.0.to_string(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SavedSelection {
    lens: SavedPin,
    pins: Vec<(SavedPin, AdmittedAs)>,
}

impl From<&SelectionRecord> for SavedSelection {
    fn from(record: &SelectionRecord) -> Self {
        Self {
            lens: (&record.lens).into(),
            pins: record
                .pins
                .iter()
                .map(|p| ((&p.pin).into(), p.admitted.clone()))
                .collect(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SavedView {
    schema: String,
    pub name: String,
    pub scope: String,
    pub sources: Vec<String>,
    pub query: ViewQuery,
    selection: SavedSelection,
}

/// The host supplies only sources currently disclosed to this reader. It builds
/// the index from the identified revision under the supplied evaluation profile.
pub struct AdmittedSource<'a> {
    pub revision: &'a str,
    pub index: &'a RecordIndex,
}

/// A missing revision is an unavailable source. An available source may have zero matches.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SourceResult {
    pub source: String,
    pub revision: Option<String>,
    pub statements: Vec<RecordLink>,
}

impl SavedView {
    /// Upper bound for a portable definition, excluding source contents and results.
    pub const MAX_BYTES: usize = 1024 * 1024;

    pub fn new(
        name: String,
        scope: String,
        sources: Vec<String>,
        query: ViewQuery,
        profile: &Profile,
    ) -> Result<Self, String> {
        let view = Self {
            schema: SCHEMA.into(),
            name,
            scope,
            sources,
            query,
            selection: (&profile.selection).into(),
        };
        view.check_profile(profile)?;
        view.to_json()?;
        Ok(view)
    }

    fn validate(&self) -> Result<(), String> {
        if self.schema != SCHEMA {
            return Err("Unsupported saved view format".into());
        }
        if self.name.trim().is_empty() || self.scope.trim().is_empty() || self.sources.is_empty() {
            return Err("A saved view needs a name, an owner scope and at least one source".into());
        }
        let mut unique = BTreeSet::new();
        for id in &self.sources {
            if id.trim().is_empty() || !unique.insert(id) {
                return Err("Each source must have a distinct, nonempty identity".into());
            }
        }
        let iri = match &self.query {
            ViewQuery::Type { class } => class,
            ViewQuery::Incoming { record } | ViewQuery::Outgoing { record } => record,
        };
        oxiri::Iri::parse(iri.as_str()).map_err(|e| format!("Invalid query IRI: {e}"))?;
        Ok(())
    }

    fn check_profile(&self, profile: &Profile) -> Result<(), String> {
        if self.selection != SavedSelection::from(&profile.selection) {
            return Err(
                "This view requires its original lens and selected vocabulary revisions".into(),
            );
        }
        if let ViewQuery::Type { class } = &self.query
            && !profile.terms.entry(class).is_some_and(|t| {
                t.kind == TermKind::Class && t.pack != mere::profile::core::CORE_PACK_ID
            })
        {
            return Err("The selected vocabulary does not declare this record class".into());
        }
        Ok(())
    }

    pub fn check_scope(&self, scope: &str, profile: &Profile) -> Result<(), String> {
        self.validate()?;
        if self.scope != scope {
            return Err("This view belongs to another owner scope".into());
        }
        self.check_profile(profile)
    }

    pub fn from_json(text: &str) -> Result<Self, String> {
        if text.len() > Self::MAX_BYTES {
            return Err("Saved view exceeds 1 MiB".into());
        }
        let view: Self = serde_json::from_str(text).map_err(|e| e.to_string())?;
        view.validate()?;
        Ok(view)
    }

    pub fn to_json(&self) -> Result<String, String> {
        self.validate()?;
        let text = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        if text.len() > Self::MAX_BYTES {
            return Err("Saved view exceeds 1 MiB".into());
        }
        Ok(text)
    }

    /// The recipe cannot discover sources or grant access. Check the active scope
    /// and profile before asking the host to resolve any of its opaque identifiers.
    pub fn evaluate<'a>(
        &self,
        scope: &str,
        profile: &Profile,
        mut resolve: impl FnMut(&str) -> Option<AdmittedSource<'a>>,
    ) -> Result<Vec<SourceResult>, String> {
        self.check_scope(scope, profile)?;
        self.sources
            .iter()
            .map(|id| self.project(id, resolve(id), &profile.selection))
            .collect()
    }

    /// Evaluate a fallible source reader one document at a time. The owner can drop
    /// each parsed graph before loading the next; only the matching rows survive.
    pub fn evaluate_loaded(
        &self,
        scope: &str,
        profile: &Profile,
        mut load: impl FnMut(&str) -> Result<Option<(String, RecordIndex)>, String>,
    ) -> Result<Vec<SourceResult>, String> {
        self.check_scope(scope, profile)?;
        self.sources
            .iter()
            .map(|id| {
                let held = load(id)?;
                let source = held
                    .as_ref()
                    .map(|(revision, index)| AdmittedSource { revision, index });
                self.project(id, source, &profile.selection)
            })
            .collect()
    }

    fn project(
        &self,
        id: &str,
        source: Option<AdmittedSource<'_>>,
        selection: &SelectionRecord,
    ) -> Result<SourceResult, String> {
        let mut result = SourceResult {
            source: id.to_owned(),
            revision: None,
            statements: Vec::new(),
        };
        if let Some(source) = source {
            if !source.index.built_with(selection) {
                return Err("Rebuild the source index with the saved vocabulary selection".into());
            }
            if source.revision.trim().is_empty() {
                return Err("An admitted source requires a revision".into());
            }
            result.revision = Some(source.revision.to_owned());
            result.statements = match &self.query {
                ViewQuery::Type { class } => source
                    .index
                    .members(class)
                    .iter()
                    .map(|subject| RecordLink::membership(subject, class))
                    .collect(),
                ViewQuery::Incoming { record } => source
                    .index
                    .links(record)
                    .filter(|l| &l.object == record)
                    .cloned()
                    .chain(
                        source
                            .index
                            .members(record)
                            .iter()
                            .map(|s| RecordLink::membership(s, record)),
                    )
                    .collect(),
                ViewQuery::Outgoing { record } => source
                    .index
                    .links(record)
                    .filter(|l| &l.subject == record)
                    .cloned()
                    .chain(
                        source
                            .index
                            .types(record)
                            .iter()
                            .map(|c| RecordLink::membership(record, c)),
                    )
                    .collect(),
            };
        }
        Ok(result)
    }
}
