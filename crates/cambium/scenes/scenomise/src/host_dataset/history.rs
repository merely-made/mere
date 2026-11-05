// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

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

pub(super) fn check_history(history: &HostDatasetV2) -> Result<(), HostDatasetError> {
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
        let identity_issues = occurrence_identity_issues(&entry.dataset)
            .into_iter()
            .map(|issue| CompileIssue {
                field: format!("revisions[{index}].{}", issue.field),
                message: issue.message,
            })
            .collect::<Vec<_>>();
        if !identity_issues.is_empty() {
            return Err(HostDatasetError::OccurrenceIdentities(identity_issues));
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

/// Check that occurrence ids are nonblank and unique within this dataset.
/// Returns one issue for each blank or repeated occurrence id; an empty
/// vector means identities are valid.
pub fn occurrence_identity_issues(dataset: &ProjectionDataset) -> Vec<CompileIssue> {
    let mut issues = Vec::new();
    let mut seen = HashSet::new();
    for occurrence in &dataset.occurrences {
        let prefix = format!("occurrences.{}", occurrence.occurrence_id);
        if occurrence.occurrence_id.trim().is_empty()
            || !seen.insert(occurrence.occurrence_id.as_str())
        {
            issues.push(CompileIssue {
                field: format!("{prefix}.occurrence_id"),
                message: "occurrence ids must be non-empty and unique".into(),
            });
        }
    }
    issues
}

pub(super) fn check_revisions(envelope: &HostDatasetV1) -> Result<(), HostDatasetError> {
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

/// [`known_keys`] for a history envelope.
pub(super) fn history_known_keys(root: &Value) -> Result<(), HostDatasetError> {
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
