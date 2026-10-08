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

/// Input larger than this is refused before it is parsed.
pub const MAX_HOST_DATASET_BYTES: usize = 1024 * 1024;

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

/// Refuse a key the envelope does not define at any depth. A value of the
/// wrong shape is left to the typed parse, which reports it as malformed.
fn known_keys(root: &Value) -> Result<(), HostDatasetError> {
    object(root, "", ENVELOPE_KEYS)?;
    if let Some(dataset) = root.get("dataset") {
        object(dataset, "dataset", DATASET_KEYS)?;
        if let Some(source) = dataset.get("source") {
            object(source, "dataset.source", BINDING_KEYS)?;
        }
        for (index, occurrence) in array(dataset.get("occurrences")) {
            let path = format!("dataset.occurrences[{index}]");
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
    }
    for (index, relationship) in array(root.get("relationships")) {
        let path = format!("relationships[{index}]");
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
    for (index, revision) in array(root.get("revisions")) {
        object(revision, &format!("revisions[{index}]"), REVISION_KEYS)?;
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
