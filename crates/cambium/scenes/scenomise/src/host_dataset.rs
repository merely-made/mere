// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The versioned envelope in which a host page supplies a dataset.
//!
//! A page hands a viewer a [`ProjectionDataset`] it did not produce, so the
//! boundary is strict where stored formats are not (mer3ly site canvas plan,
//! Rulings 124 and 125): the envelope names its schema, and an unknown key at
//! any depth, a stale or disordered revision sequence, or a relationship whose
//! endpoints or provenance disagree with the dataset is refused. Saved
//! projections and [`ProjectionDataset`]'s own serde are unchanged; the key
//! check reads the JSON before it is typed, so the scenograph types gain no
//! `deny_unknown_fields`.
//!
//! Relationships are the relationship recipe's [`DisclosedRelationship`], and
//! they are checked by the same validator the relationship compiler runs.
//! They are routed onto whatever projection the dataset compiles to (site
//! canvas plan, Ruling 128): [`HostDatasetV1::compile`] compiles the host's
//! own definition as usual and then draws each relationship between its two
//! occurrences where that arrangement placed them. No semantic facet and no
//! layout is required of the dataset, and a dataset without relationships
//! compiles exactly as it would without the envelope. The relationship
//! recipe's compiler, which does require them, is untouched.
//!
//! [`HostDatasetV2`] (`scenomise.host-dataset/v2`, site canvas plan, Ruling
//! 144) carries a dataset and its relationships per revision, so one file
//! holds the history that [`crate::history`] classifies. It is checked with
//! the same strictness, revision by revision, and [`parse_host_history`]
//! reads a v1 envelope as the one-revision case. [`parse_host_dataset`] and
//! [`HostDatasetV1`] are unchanged.

pub mod folds;

use std::collections::HashSet;
use std::fmt;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::projection::{
    CompileIssue, CompiledProjection, CompiledRelationship, DisclosedRelationship,
    ProjectionCompiler, ProjectionDataset, relationship_disclosure_issues, stable_generation,
};
use scenograph::{ProjectionDefinition, PublicSourceRevision};

/// The schema tag of the first host dataset envelope.
pub const HOST_DATASET_SCHEMA_V1: &str = "scenomise.host-dataset/v1";

/// The schema tag of the envelope that carries a dataset per revision.
pub const HOST_DATASET_SCHEMA_V2: &str = "scenomise.host-dataset/v2";

/// Input larger than this is refused before it is parsed.
pub const MAX_HOST_DATASET_BYTES: usize = 1024 * 1024;

/// A v2 history larger than this is refused before it is parsed. A history
/// carries a dataset per revision, so it gets more room than one dataset: at
/// mer3ly's size (about 65 KB a revision pretty-printed) that is about 60
/// revisions (mer3ly site Ruling 148). A v1 envelope read as a history keeps
/// [`MAX_HOST_DATASET_BYTES`].
pub const MAX_HOST_HISTORY_BYTES: usize = 4 * 1024 * 1024;

/// A host-supplied dataset: the dataset, the relationships its source
/// discloses, and the public revisions it has published, oldest first.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HostDatasetV1 {
    /// Always [`HOST_DATASET_SCHEMA_V1`].
    pub schema: String,
    pub dataset: ProjectionDataset,
    pub relationships: Vec<DisclosedRelationship>,
    /// The source's published revisions in order. The last is the dataset's.
    pub revisions: Vec<HostDatasetRevision>,
}

/// One published revision in a host dataset's sequence.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HostDatasetRevision {
    /// Strictly increasing along the sequence; it orders revisions whose
    /// identities are opaque.
    pub sequence: u64,
    pub revision: PublicSourceRevision,
}

/// Why a host dataset was refused.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HostDatasetError {
    /// The input exceeds [`MAX_HOST_DATASET_BYTES`].
    TooLarge { bytes: usize, limit: usize },
    /// The input is not JSON, or a value has the wrong shape.
    Malformed(String),
    /// The envelope names no schema, or one this reader does not know.
    UnknownSchema(Option<String>),
    /// A key the schema does not define, at `path`.
    UnknownKey { path: String, key: String },
    /// The revision sequence is empty.
    NoRevisions,
    /// A revision in the sequence is blank.
    BlankRevision { index: usize },
    /// A revision's sequence number does not exceed its predecessor's.
    RevisionOutOfOrder { index: usize },
    /// A revision appears twice in the sequence.
    DuplicateRevision { revision: String },
    /// The dataset is not at the sequence's latest revision.
    StaleRevision { dataset: String, latest: String },
    /// Relationships that disagree with the dataset: endpoints, identity,
    /// meaning, or provenance at another source or revision.
    Relationships(Vec<CompileIssue>),
    /// A history envelope names no schema, or one the history reader does not
    /// know.
    UnknownHistorySchema(Option<String>),
    /// A history revision's dataset is at another revision than the one the
    /// entry claims: a stale dataset filed under a newer revision.
    RevisionMismatch {
        index: usize,
        revision: String,
        dataset: String,
    },
    /// A history revision's dataset binds another source than the first's.
    SourceChanged { index: usize },
    /// The declared compared fields are empty, repeat a field, or name a
    /// field no revision declares.
    ComparedFields(String),
    /// The declared compared relationship fields are empty, repeat a part,
    /// or name a part outside [`crate::history::RELATIONSHIP_FIELDS`].
    ComparedRelationshipFields(String),
}

impl fmt::Display for HostDatasetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooLarge { bytes, limit } => {
                write!(f, "the dataset is {bytes} bytes; the limit is {limit}")
            },
            Self::Malformed(error) => write!(f, "the dataset is malformed: {error}"),
            Self::UnknownSchema(Some(schema)) => {
                write!(
                    f,
                    "unknown dataset schema {schema}; expected {HOST_DATASET_SCHEMA_V1}"
                )
            },
            Self::UnknownSchema(None) => {
                write!(
                    f,
                    "the dataset names no schema; expected {HOST_DATASET_SCHEMA_V1}"
                )
            },
            Self::UnknownKey { path, key } if path.is_empty() => {
                write!(f, "unknown key {key} in the dataset envelope")
            },
            Self::UnknownKey { path, key } => write!(f, "unknown key {key} in {path}"),
            Self::NoRevisions => write!(f, "the dataset discloses no revisions"),
            Self::BlankRevision { index } => write!(f, "revision {index} is blank"),
            Self::RevisionOutOfOrder { index } => {
                write!(f, "revision {index} is out of order")
            },
            Self::DuplicateRevision { revision } => {
                write!(f, "revision {revision} appears twice")
            },
            Self::StaleRevision { dataset, latest } => write!(
                f,
                "the dataset is at revision {dataset}, but the latest revision is {latest}"
            ),
            Self::Relationships(issues) => {
                write!(f, "refused relationships:")?;
                for issue in issues {
                    write!(f, " {}: {};", issue.field, issue.message)?;
                }
                Ok(())
            },
            Self::UnknownHistorySchema(Some(schema)) => write!(
                f,
                "unknown dataset schema {schema}; expected {HOST_DATASET_SCHEMA_V2} or \
                 {HOST_DATASET_SCHEMA_V1}"
            ),
            Self::UnknownHistorySchema(None) => write!(
                f,
                "the dataset names no schema; expected {HOST_DATASET_SCHEMA_V2} or \
                 {HOST_DATASET_SCHEMA_V1}"
            ),
            Self::RevisionMismatch {
                index,
                revision,
                dataset,
            } => write!(
                f,
                "revision {index} claims {revision}, but its dataset is at revision {dataset}"
            ),
            Self::SourceChanged { index } => {
                write!(f, "revision {index} discloses another source")
            },
            Self::ComparedFields(problem) => write!(f, "the compared fields {problem}"),
            Self::ComparedRelationshipFields(problem) => {
                write!(f, "the compared relationship fields {problem}")
            },
        }
    }
}

impl std::error::Error for HostDatasetError {}

/// Parse and check one host-supplied dataset.
pub fn parse_host_dataset(json: &str) -> Result<HostDatasetV1, HostDatasetError> {
    if json.len() > MAX_HOST_DATASET_BYTES {
        return Err(HostDatasetError::TooLarge {
            bytes: json.len(),
            limit: MAX_HOST_DATASET_BYTES,
        });
    }
    let value: Value = serde_json::from_str(json)
        .map_err(|error| HostDatasetError::Malformed(error.to_string()))?;
    // The tag first, so a later schema's new keys read as a later schema.
    match value.get("schema") {
        Some(Value::String(schema)) if schema == HOST_DATASET_SCHEMA_V1 => {},
        Some(Value::String(schema)) => {
            return Err(HostDatasetError::UnknownSchema(Some(schema.clone())));
        },
        _ => return Err(HostDatasetError::UnknownSchema(None)),
    }
    known_keys(&value)?;
    let envelope: HostDatasetV1 = serde_json::from_value(value)
        .map_err(|error| HostDatasetError::Malformed(error.to_string()))?;
    check_revisions(&envelope)?;
    let issues = relationship_disclosure_issues(&envelope.dataset, &envelope.relationships);
    if !issues.is_empty() {
        return Err(HostDatasetError::Relationships(issues));
    }
    Ok(envelope)
}

/// A host dataset compiled through a host's definition, its disclosed
/// relationships routed onto the arrangement that definition chose.
#[derive(Debug)]
pub struct HostProjection {
    /// The projection; its `scene.relations` holds one routed relation per
    /// disclosed relationship, in relationship id order.
    pub projection: CompiledProjection,
    /// Each relationship with the instances it joins, in the same order.
    pub relationships: Vec<CompiledRelationship>,
}

impl HostDatasetV1 {
    /// Compile `definition` against the dataset and route its relationships.
    pub fn compile(
        &self,
        compiler: &ProjectionCompiler,
        definition: &ProjectionDefinition,
    ) -> Result<HostProjection, Vec<CompileIssue>> {
        self.routed(compiler.compile(definition, &self.dataset)?)
    }

    /// As [`ProjectionCompiler::refresh`], reusing `previous`'s placement when
    /// its solver inputs are unchanged, then routing the relationships afresh.
    pub fn refresh(
        &self,
        compiler: &ProjectionCompiler,
        previous: &CompiledProjection,
        definition: &ProjectionDefinition,
    ) -> Result<HostProjection, Vec<CompileIssue>> {
        self.routed(compiler.refresh(previous, definition, &self.dataset)?)
    }

    fn routed(
        &self,
        mut projection: CompiledProjection,
    ) -> Result<HostProjection, Vec<CompileIssue>> {
        let relationships =
            route_relationships(&mut projection, &self.dataset, &self.relationships)?;
        Ok(HostProjection {
            projection,
            relationships,
        })
    }
}

/// Route disclosed relationships onto a projection compiled from `dataset`,
/// replacing its `scene.relations`.
///
/// Each relation runs between the placed positions of its two occurrences, so
/// it follows whichever arrangement the projection's definition chose. The
/// relationships are checked by the validator the relationship compiler runs,
/// and a projection compiled from any other dataset or revision is refused.
pub fn route_relationships(
    projection: &mut CompiledProjection,
    dataset: &ProjectionDataset,
    relationships: &[DisclosedRelationship],
) -> Result<Vec<CompiledRelationship>, Vec<CompileIssue>> {
    let mut issues = relationship_disclosure_issues(dataset, relationships);
    if projection.score.generation != stable_generation(dataset) {
        issues.push(CompileIssue {
            field: "projection".into(),
            message: "the projection was compiled from another dataset or revision".into(),
        });
    }
    if !issues.is_empty() {
        return Err(issues);
    }
    let mut ordered: Vec<_> = relationships.iter().collect();
    ordered.sort_by(|a, b| a.id.cmp(&b.id));
    let mut routed = Vec::with_capacity(ordered.len());
    let mut compiled = Vec::with_capacity(ordered.len());
    for relationship in ordered {
        let endpoint = |occurrence: &str| {
            projection
                .instance_by_occurrence
                .get(occurrence)
                .copied()
                .ok_or_else(|| {
                    vec![CompileIssue {
                        field: format!("relationships.{}.endpoints", relationship.id),
                        message: "the projection does not place this occurrence".into(),
                    }]
                })
        };
        let (from, to) = (
            endpoint(&relationship.from_occurrence)?,
            endpoint(&relationship.to_occurrence)?,
        );
        let at = |instance: sceno::InstanceId| {
            projection.scene.items[instance.0 as usize]
                .transform
                .translate
        };
        routed.push(sceno::RoutedRelation {
            from,
            to,
            space: sceno::Scene::WORLD,
            points: vec![at(from), at(to)],
            kind: Some(relationship.kind.clone()),
            weight: None,
        });
        compiled.push(CompiledRelationship {
            disclosure: relationship.clone(),
            from,
            to,
        });
    }
    projection.scene.relations = routed;
    Ok(compiled)
}

/// A host-supplied history: the source's published revisions, oldest first,
/// each with its own dataset and relationships. The last is current.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HostDatasetV2 {
    /// Always [`HOST_DATASET_SCHEMA_V2`].
    pub schema: String,
    pub revisions: Vec<HostDatasetRevisionV2>,
    /// The fields whose values decide whether an occurrence was updated
    /// between revisions (site canvas plan, Ruling 145). Absent means every
    /// disclosed field; a host narrows it to the fields its readers treat as
    /// change. Incident relationships count either way.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compared_fields: Option<Vec<String>>,
    /// The parts of a relationship whose change counts, from
    /// [`crate::history::RELATIONSHIP_FIELDS`] (Ruling 146). Absent means
    /// everything a relationship discloses except its source and revision.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compared_relationship_fields: Option<Vec<String>>,
}

/// One revision in a host history, with what the source disclosed at it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HostDatasetRevisionV2 {
    /// Strictly increasing along the history.
    pub sequence: u64,
    /// The revision; the dataset must be at it.
    pub revision: PublicSourceRevision,
    pub dataset: ProjectionDataset,
    pub relationships: Vec<DisclosedRelationship>,
}

impl HostDatasetV2 {
    /// The current revision: the last.
    pub fn current(&self) -> &HostDatasetRevisionV2 {
        self.revisions
            .last()
            .expect("a parsed history has at least one revision")
    }

    /// The current revision as a v1 envelope, so a reader of one dataset
    /// reads a history's present unchanged.
    pub fn current_v1(&self) -> HostDatasetV1 {
        let current = self.current();
        HostDatasetV1 {
            schema: HOST_DATASET_SCHEMA_V1.to_owned(),
            dataset: current.dataset.clone(),
            relationships: current.relationships.clone(),
            revisions: self
                .revisions
                .iter()
                .map(|entry| HostDatasetRevision {
                    sequence: entry.sequence,
                    revision: entry.revision.clone(),
                })
                .collect(),
        }
    }
}

impl From<HostDatasetV1> for HostDatasetV2 {
    /// A v1 envelope is the one-revision history: its dataset at its latest
    /// revision. Earlier v1 revisions name no dataset, so they do not become
    /// revisions here.
    fn from(envelope: HostDatasetV1) -> Self {
        let sequence = envelope
            .revisions
            .last()
            .map_or(1, |latest| latest.sequence);
        Self {
            schema: HOST_DATASET_SCHEMA_V2.to_owned(),
            revisions: vec![HostDatasetRevisionV2 {
                sequence,
                revision: envelope.dataset.revision.clone(),
                dataset: envelope.dataset,
                relationships: envelope.relationships,
            }],
            compared_fields: None,
            compared_relationship_fields: None,
        }
    }
}

/// Parse and check a host-supplied history. A `scenomise.host-dataset/v2`
/// envelope is checked revision by revision as strictly as v1 checks its
/// one dataset; a v1 envelope is parsed as v1 and lifted to one revision.
pub fn parse_host_history(json: &str) -> Result<HostDatasetV2, HostDatasetError> {
    if json.len() > MAX_HOST_HISTORY_BYTES {
        return Err(HostDatasetError::TooLarge {
            bytes: json.len(),
            limit: MAX_HOST_HISTORY_BYTES,
        });
    }
    let value: Value = serde_json::from_str(json)
        .map_err(|error| HostDatasetError::Malformed(error.to_string()))?;
    match value.get("schema") {
        Some(Value::String(schema)) if schema == HOST_DATASET_SCHEMA_V2 => {},
        Some(Value::String(schema)) if schema == HOST_DATASET_SCHEMA_V1 => {
            return parse_host_dataset(json).map(HostDatasetV2::from);
        },
        Some(Value::String(schema)) => {
            return Err(HostDatasetError::UnknownHistorySchema(Some(schema.clone())));
        },
        _ => return Err(HostDatasetError::UnknownHistorySchema(None)),
    }
    history_known_keys(&value)?;
    let history: HostDatasetV2 = serde_json::from_value(value)
        .map_err(|error| HostDatasetError::Malformed(error.to_string()))?;
    check_history(&history)?;
    Ok(history)
}

fn check_history(history: &HostDatasetV2) -> Result<(), HostDatasetError> {
    let Some(first) = history.revisions.first() else {
        return Err(HostDatasetError::NoRevisions);
    };
    let mut seen = HashSet::new();
    let mut previous: Option<u64> = None;
    for (index, entry) in history.revisions.iter().enumerate() {
        if entry.revision.as_str().trim().is_empty() {
            return Err(HostDatasetError::BlankRevision { index });
        }
        if previous.is_some_and(|previous| entry.sequence <= previous) {
            return Err(HostDatasetError::RevisionOutOfOrder { index });
        }
        if !seen.insert(entry.revision.as_str()) {
            return Err(HostDatasetError::DuplicateRevision {
                revision: entry.revision.as_str().to_owned(),
            });
        }
        if entry.dataset.revision != entry.revision {
            return Err(HostDatasetError::RevisionMismatch {
                index,
                revision: entry.revision.as_str().to_owned(),
                dataset: entry.dataset.revision.as_str().to_owned(),
            });
        }
        if entry.dataset.source != first.dataset.source {
            return Err(HostDatasetError::SourceChanged { index });
        }
        let issues = relationship_disclosure_issues(&entry.dataset, &entry.relationships);
        if !issues.is_empty() {
            return Err(HostDatasetError::Relationships(
                issues
                    .into_iter()
                    .map(|issue| CompileIssue {
                        field: format!("revisions[{index}].{}", issue.field),
                        message: issue.message,
                    })
                    .collect(),
            ));
        }
        previous = Some(entry.sequence);
    }
    if let Some(fields) = &history.compared_fields {
        if fields.is_empty() {
            return Err(HostDatasetError::ComparedFields("are empty".into()));
        }
        let mut named = HashSet::new();
        for field in fields {
            if !named.insert(field.as_str()) {
                return Err(HostDatasetError::ComparedFields(format!(
                    "name {field} twice"
                )));
            }
            if !history
                .revisions
                .iter()
                .any(|entry| entry.dataset.fields.contains_key(field))
            {
                return Err(HostDatasetError::ComparedFields(format!(
                    "name {field}, which no revision declares"
                )));
            }
        }
    }
    if let Some(parts) = &history.compared_relationship_fields {
        if parts.is_empty() {
            return Err(HostDatasetError::ComparedRelationshipFields(
                "are empty".into(),
            ));
        }
        let mut named = HashSet::new();
        for part in parts {
            if !named.insert(part.as_str()) {
                return Err(HostDatasetError::ComparedRelationshipFields(format!(
                    "name {part} twice"
                )));
            }
            if !crate::history::RELATIONSHIP_FIELDS.contains(&part.as_str()) {
                return Err(HostDatasetError::ComparedRelationshipFields(format!(
                    "name {part}, which is not a relationship field"
                )));
            }
        }
    }
    Ok(())
}

fn check_revisions(envelope: &HostDatasetV1) -> Result<(), HostDatasetError> {
    let Some(latest) = envelope.revisions.last() else {
        return Err(HostDatasetError::NoRevisions);
    };
    let mut seen = HashSet::new();
    let mut previous: Option<u64> = None;
    for (index, entry) in envelope.revisions.iter().enumerate() {
        if entry.revision.as_str().trim().is_empty() {
            return Err(HostDatasetError::BlankRevision { index });
        }
        if previous.is_some_and(|previous| entry.sequence <= previous) {
            return Err(HostDatasetError::RevisionOutOfOrder { index });
        }
        if !seen.insert(entry.revision.as_str()) {
            return Err(HostDatasetError::DuplicateRevision {
                revision: entry.revision.as_str().to_owned(),
            });
        }
        previous = Some(entry.sequence);
    }
    if latest.revision != envelope.dataset.revision {
        return Err(HostDatasetError::StaleRevision {
            dataset: envelope.dataset.revision.as_str().to_owned(),
            latest: latest.revision.as_str().to_owned(),
        });
    }
    Ok(())
}

// The keys each object may carry, as serialized. A test checks every list
// against a serialized sample, so a field added to a scenograph or sceno type
// fails here until the list follows it.
const ENVELOPE_KEYS: &[&str] = &["schema", "dataset", "relationships", "revisions"];
const DATASET_KEYS: &[&str] = &["source", "revision", "fields", "occurrences"];
const BINDING_KEYS: &[&str] = &["authority", "domain", "resource"];
const OCCURRENCE_KEYS: &[&str] = &["occurrence_id", "source", "values"];
const SOURCE_REF_KEYS: &[&str] = &["adapter", "id"];
const VALUE_KEYS: &[&str] = &["kind", "value"];
const RELATIONSHIP_KEYS: &[&str] = &[
    "id",
    "from_occurrence",
    "to_occurrence",
    "kind",
    "label",
    "explanation",
    "provenance",
];
const PROVENANCE_KEYS: &[&str] = &[
    "source",
    "source_revision",
    "method",
    "method_version",
    "provider",
    "evidence",
];
const REVISION_KEYS: &[&str] = &["sequence", "revision"];
const HISTORY_KEYS: &[&str] = &[
    "schema",
    "revisions",
    "compared_fields",
    "compared_relationship_fields",
];
const HISTORY_REVISION_KEYS: &[&str] = &["sequence", "revision", "dataset", "relationships"];

/// Refuse a key the envelope does not define at any depth. A value of the
/// wrong shape is left to the typed parse, which reports it as malformed.
fn known_keys(root: &Value) -> Result<(), HostDatasetError> {
    object(root, "", ENVELOPE_KEYS)?;
    if let Some(dataset) = root.get("dataset") {
        dataset_keys(dataset, "dataset")?;
    }
    relationship_keys(root.get("relationships"), "relationships")?;
    for (index, revision) in array(root.get("revisions")) {
        object(revision, &format!("revisions[{index}]"), REVISION_KEYS)?;
    }
    Ok(())
}

fn dataset_keys(dataset: &Value, path: &str) -> Result<(), HostDatasetError> {
    object(dataset, path, DATASET_KEYS)?;
    if let Some(source) = dataset.get("source") {
        object(source, &format!("{path}.source"), BINDING_KEYS)?;
    }
    for (index, occurrence) in array(dataset.get("occurrences")) {
        let path = format!("{path}.occurrences[{index}]");
        object(occurrence, &path, OCCURRENCE_KEYS)?;
        if let Some(source) = occurrence.get("source") {
            object(source, &format!("{path}.source"), SOURCE_REF_KEYS)?;
        }
        if let Some(Value::Object(values)) = occurrence.get("values") {
            for (name, value) in values {
                object(value, &format!("{path}.values.{name}"), VALUE_KEYS)?;
            }
        }
    }
    Ok(())
}

fn relationship_keys(relationships: Option<&Value>, path: &str) -> Result<(), HostDatasetError> {
    for (index, relationship) in array(relationships) {
        let path = format!("{path}[{index}]");
        object(relationship, &path, RELATIONSHIP_KEYS)?;
        if let Some(provenance) = relationship.get("provenance") {
            let path = format!("{path}.provenance");
            object(provenance, &path, PROVENANCE_KEYS)?;
            if let Some(source) = provenance.get("source") {
                object(source, &format!("{path}.source"), BINDING_KEYS)?;
            }
            for (index, evidence) in array(provenance.get("evidence")) {
                object(
                    evidence,
                    &format!("{path}.evidence[{index}]"),
                    SOURCE_REF_KEYS,
                )?;
            }
        }
    }
    Ok(())
}

/// [`known_keys`] for a history envelope.
fn history_known_keys(root: &Value) -> Result<(), HostDatasetError> {
    object(root, "", HISTORY_KEYS)?;
    for (index, revision) in array(root.get("revisions")) {
        let path = format!("revisions[{index}]");
        object(revision, &path, HISTORY_REVISION_KEYS)?;
        if let Some(dataset) = revision.get("dataset") {
            dataset_keys(dataset, &format!("{path}.dataset"))?;
        }
        relationship_keys(
            revision.get("relationships"),
            &format!("{path}.relationships"),
        )?;
    }
    Ok(())
}

fn object(value: &Value, path: &str, keys: &[&str]) -> Result<(), HostDatasetError> {
    if let Value::Object(map) = value
        && let Some(key) = map.keys().find(|key| !keys.contains(&key.as_str()))
    {
        return Err(HostDatasetError::UnknownKey {
            path: path.to_owned(),
            key: key.clone(),
        });
    }
    Ok(())
}

fn array(value: Option<&Value>) -> impl Iterator<Item = (usize, &Value)> {
    value
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .enumerate()
}

#[cfg(test)]
#[path = "host_dataset_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "host_dataset_v2_tests.rs"]
mod v2_tests;
