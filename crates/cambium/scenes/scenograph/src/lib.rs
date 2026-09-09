// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Scenograph: host-neutral projection authoring definitions.
//!
//! This crate owns durable authoring data, local validation, source binding,
//! and deterministic JSON. Hosts own their editor UI, source acquisition,
//! catalog resolution, persistence policy, compilation, and realization.
//! It deliberately has no runtime, solver, widget, or product dependency.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// The stable schema version for authored projection definitions.
pub const PROJECTION_DEFINITION_VERSION: u16 = 1;

/// A durable, public revision emitted by a source authority.
///
/// JSON keeps the v1 string shape. This is deliberately distinct from an
/// in-process concurrency witness: a revision may enter provenance and a
/// saved definition; a witness may not.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PublicSourceRevision(String);

impl PublicSourceRevision {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }

    pub fn bytes(&self) -> std::str::Bytes<'_> {
        self.0.bytes()
    }
}

impl From<String> for PublicSourceRevision {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for PublicSourceRevision {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

impl std::fmt::Display for PublicSourceRevision {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(formatter)
    }
}

/// What makes a source binding reproducible at the time it was authored.
///
/// The default keeps v1 public-generation JSON byte-for-byte unchanged. A
/// runtime-verified binding records only that a host checked an opaque
/// in-process witness; it never represents that witness as a revision.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RevisionEvidence {
    #[default]
    PublicGeneration,
    RuntimeVerified,
}

impl RevisionEvidence {
    fn is_public_generation(&self) -> bool {
        matches!(self, Self::PublicGeneration)
    }
}

/// A source and domain binding selected by the author.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct SourceBinding {
    /// The authority or owner identifier, not an authority handle.
    pub authority: String,
    /// The domain or namespace in which the source is interpreted.
    pub domain: String,
    /// A source-local resource, dataset, or graph identifier.
    pub resource: String,
}

/// The facts and grain an arrangement reads from its source.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Reading {
    /// A registered reading id resolved by the host catalog.
    pub kind: String,
    /// The identity field used to address records.
    pub key: String,
    /// The value field for a value-oriented reading.
    pub value: Option<String>,
}

impl Default for Reading {
    fn default() -> Self {
        Self {
            kind: "nodes".into(),
            key: String::new(),
            value: None,
        }
    }
}

/// A field or literal assigned to a visual channel.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind", content = "value")]
pub enum Channel {
    Field(String),
    Constant(String),
}

/// Encodings map reading fields to arrangement channels.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Encoding {
    pub x: Channel,
    pub y: Channel,
    pub color: Option<Channel>,
    pub label: Option<Channel>,
}

impl Default for Encoding {
    fn default() -> Self {
        Self {
            x: Channel::Field(String::new()),
            y: Channel::Field(String::new()),
            color: None,
            label: None,
        }
    }
}

/// The spatial or tabular arrangement requested by the author.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Arrangement {
    /// A registered arrangement id resolved by the host catalog.
    pub kind: String,
    /// An open direction or coordinate parameter understood by that id.
    pub direction: String,
    /// Positive spacing in the arrangement's own units.
    pub spacing: u32,
    /// Arrangement-owned configuration resolved by the named arrangement.
    ///
    /// The ordered map preserves deterministic encoding without teaching the
    /// shared grammar a host's guide vocabulary.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub options: BTreeMap<String, String>,
}

impl Default for Arrangement {
    fn default() -> Self {
        Self {
            kind: "grid".into(),
            direction: "horizontal".into(),
            spacing: 16,
            options: BTreeMap::new(),
        }
    }
}

/// Interaction affordances offered by a realization.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Interaction {
    pub selection: SelectionMode,
    pub pan: bool,
    pub zoom: bool,
}

impl Default for Interaction {
    fn default() -> Self {
        Self {
            selection: SelectionMode::Single,
            pan: true,
            zoom: true,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SelectionMode {
    None,
    Single,
    Multiple,
}

/// Appearance and realization choices. Rendering remains host-owned.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct Appearance {
    /// A registered realization id. Rendering and resolution stay host-owned.
    pub realization: String,
    pub title: String,
    pub theme: String,
}

/// Human and machine provenance attached to a saved definition.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct Provenance {
    pub author: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_revision: Option<PublicSourceRevision>,
    #[serde(
        default,
        skip_serializing_if = "RevisionEvidence::is_public_generation"
    )]
    pub revision_evidence: RevisionEvidence,
    pub note: String,
}

/// The editable form, which intentionally retains incomplete values so hosts
/// can present field-level validation without rejecting keystrokes.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProjectionDraft {
    pub version: u16,
    pub id: String,
    pub label: String,
    pub source: SourceBinding,
    pub reading: Reading,
    pub encoding: Encoding,
    pub arrangement: Arrangement,
    pub interaction: Interaction,
    pub appearance: Appearance,
    pub provenance: Provenance,
}

/// The complete, validated, durable projection definition.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProjectionDefinition {
    pub version: u16,
    pub id: String,
    pub label: String,
    pub source: SourceBinding,
    pub reading: Reading,
    pub encoding: Encoding,
    pub arrangement: Arrangement,
    pub interaction: Interaction,
    pub appearance: Appearance,
    pub provenance: Provenance,
}

/// One source binding a reusable authored definition may read.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProjectionInputBinding {
    pub source: SourceBinding,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expects_generation: Option<PublicSourceRevision>,
    #[serde(
        default,
        skip_serializing_if = "RevisionEvidence::is_public_generation"
    )]
    pub revision_evidence: RevisionEvidence,
}

/// A source selected by a host for one in-process operation.
///
/// `W` is intentionally opaque here. This type implements none of `Clone`,
/// `Debug`, comparison, hashing, or serde, and only exposes the witness to a
/// host-supplied closure. That keeps a runtime concurrency token out of every
/// durable Scenograph type while leaving its issuance and verification with the
/// source host. It carries no source revision and must not be used to invent
/// one for a source that lacks a durable public revision.
///
/// ```compile_fail
/// use scenograph::{RuntimeSourceBinding, SourceBinding};
///
/// let runtime = RuntimeSourceBinding::new(SourceBinding::default(), ());
/// serde_json::to_vec(&runtime).unwrap();
/// ```
pub struct RuntimeSourceBinding<W> {
    source: SourceBinding,
    witness: W,
}

impl<W> RuntimeSourceBinding<W> {
    pub fn new(source: SourceBinding, witness: W) -> Self {
        Self { source, witness }
    }

    pub fn source(&self) -> &SourceBinding {
        &self.source
    }

    /// Invoke a host-owned verifier without making the witness part of a
    /// durable definition or public source binding.
    pub fn with_witness<R>(&self, verifier: impl FnOnce(&W) -> R) -> R {
        verifier(&self.witness)
    }
}

/// A validated definition paired with the runtime witness that selected it.
///
/// Only [`Self::definition`] and [`Self::into_recorded_definition`] expose the
/// durable portion. The latter deliberately discards executable freshness: its
/// `runtime_verified` marker records that a host supplied a witness at binding
/// time, never a source revision or a witness that can be replayed later.
pub struct RuntimeProjectionBinding<W> {
    definition: ProjectionDefinition,
    source: RuntimeSourceBinding<W>,
}

impl<W> RuntimeProjectionBinding<W> {
    pub fn definition(&self) -> &ProjectionDefinition {
        &self.definition
    }

    pub fn into_recorded_definition(self) -> ProjectionDefinition {
        self.definition
    }

    pub fn source(&self) -> &SourceBinding {
        self.source.source()
    }

    /// Invoke the host's runtime witness verifier for this binding.
    pub fn with_witness<R>(&self, verifier: impl FnOnce(&W) -> R) -> R {
        self.source.with_witness(verifier)
    }
}

/// A reusable authored projection recipe with its named, checkable sources.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AuthoredProjectionDefinition {
    pub version: u16,
    pub id: String,
    pub label: String,
    /// Named datasets this definition may read. The ordered map makes the body
    /// deterministic and identifies the source a host bound.
    pub sources: BTreeMap<String, ProjectionInputBinding>,
    pub reading: Reading,
    pub encoding: Encoding,
    pub arrangement: Arrangement,
    pub interaction: Interaction,
    pub appearance: Appearance,
    /// This revision belongs to the authored recipe. Bound definitions receive
    /// their selected input generation instead.
    pub provenance: Provenance,
}

/// An authored delta over one definition, rather than another definition.
///
/// v1 owns arrangement options only. Other section deltas need their own
/// forcing consumer before entering the portable contract.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProjectionVariant {
    pub definition_id: String,
    pub id: String,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub arrangement_options: BTreeMap<String, String>,
}

/// Why a reusable authored definition could not be bound to a host payload.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AuthoredDefinitionError {
    Invalid(Vec<ValidationIssue>),
    UnknownSource(String),
    EmptyExpectedGeneration(String),
    VariantForAnotherDefinition(String),
    EmptyVariantId,
    EmptyArrangementOption(String),
    RuntimeSourceMismatch(String),
    RuntimeBindingRequiresWitness(String),
    RuntimeEvidenceRequired(String),
    RuntimeBindingHasPublicGeneration(String),
}

impl ProjectionDraft {
    /// Start an empty draft with safe enum and layout defaults.
    pub fn new() -> Self {
        Self {
            version: PROJECTION_DEFINITION_VERSION,
            id: String::new(),
            label: String::new(),
            source: SourceBinding::default(),
            reading: Reading::default(),
            encoding: Encoding::default(),
            arrangement: Arrangement::default(),
            interaction: Interaction::default(),
            appearance: Appearance::default(),
            provenance: Provenance::default(),
        }
    }

    /// Validate and promote this draft to the durable definition.
    pub fn to_definition(&self) -> Result<ProjectionDefinition, Vec<ValidationIssue>> {
        self.validate()?;
        Ok(ProjectionDefinition {
            version: self.version,
            id: self.id.clone(),
            label: self.label.clone(),
            source: self.source.clone(),
            reading: self.reading.clone(),
            encoding: self.encoding.clone(),
            arrangement: self.arrangement.clone(),
            interaction: self.interaction.clone(),
            appearance: self.appearance.clone(),
            provenance: self.provenance.clone(),
        })
    }

    /// Check all required fields without contacting a host or authority.
    pub fn validate(&self) -> Result<(), Vec<ValidationIssue>> {
        let mut issues = Vec::new();
        if self.version != PROJECTION_DEFINITION_VERSION {
            issues.push(ValidationIssue::error(
                "version",
                "unsupported projection definition version",
            ));
        }
        required(&mut issues, "id", &self.id, "an id is required");
        required(&mut issues, "label", &self.label, "a label is required");
        required(
            &mut issues,
            "source.authority",
            &self.source.authority,
            "an authority binding is required",
        );
        required(
            &mut issues,
            "source.domain",
            &self.source.domain,
            "a domain binding is required",
        );
        required(
            &mut issues,
            "source.resource",
            &self.source.resource,
            "a source resource is required",
        );
        required(
            &mut issues,
            "reading.kind",
            &self.reading.kind,
            "a registered reading id is required",
        );
        required(
            &mut issues,
            "reading.key",
            &self.reading.key,
            "a reading key is required",
        );
        if self.reading.kind == "values" {
            match self.reading.value.as_deref() {
                Some(value) if !value.trim().is_empty() => {},
                _ => issues.push(ValidationIssue::error(
                    "reading.value",
                    "a value field is required for a values reading",
                )),
            }
        }
        channel_required(&mut issues, "encoding.x", &self.encoding.x);
        channel_required(&mut issues, "encoding.y", &self.encoding.y);
        required(
            &mut issues,
            "arrangement.kind",
            &self.arrangement.kind,
            "a registered arrangement id is required",
        );
        required(
            &mut issues,
            "arrangement.direction",
            &self.arrangement.direction,
            "an arrangement direction or coordinate mode is required",
        );
        if self.arrangement.spacing == 0 {
            issues.push(ValidationIssue::error(
                "arrangement.spacing",
                "spacing must be greater than zero",
            ));
        }
        required(
            &mut issues,
            "appearance.realization",
            &self.appearance.realization,
            "a registered realization id is required",
        );
        required(
            &mut issues,
            "appearance.title",
            &self.appearance.title,
            "a preview title is required",
        );
        required(
            &mut issues,
            "provenance.author",
            &self.provenance.author,
            "an author is required",
        );
        match self.provenance.revision_evidence {
            RevisionEvidence::PublicGeneration => match &self.provenance.source_revision {
                Some(revision) => required(
                    &mut issues,
                    "provenance.source_revision",
                    revision.as_str(),
                    "the source revision is required for reproducibility",
                ),
                None => issues.push(ValidationIssue::error(
                    "provenance.source_revision",
                    "the source revision is required for reproducibility",
                )),
            },
            RevisionEvidence::RuntimeVerified => {
                if self.provenance.source_revision.is_some() {
                    issues.push(ValidationIssue::error(
                        "provenance.source_revision",
                        "runtime-verified sources cannot claim a public source revision",
                    ));
                }
            },
        }
        if issues.is_empty() {
            Ok(())
        } else {
            Err(issues)
        }
    }
}

impl Default for ProjectionDraft {
    fn default() -> Self {
        Self::new()
    }
}

impl AuthoredProjectionDefinition {
    /// Validate the recipe and every source it declares without contacting an
    /// authority. A bad dormant source is still a bad durable definition.
    pub fn validate(&self) -> Result<(), AuthoredDefinitionError> {
        let mut issues = Vec::new();
        required(
            &mut issues,
            "provenance.author",
            &self.provenance.author,
            "an author is required",
        );
        required(
            &mut issues,
            "provenance.source_revision",
            self.provenance
                .source_revision
                .as_ref()
                .map(PublicSourceRevision::as_str)
                .unwrap_or_default(),
            "an authored definition revision is required",
        );
        for (name, value) in &self.arrangement.options {
            if name.trim().is_empty() || value.trim().is_empty() {
                return Err(AuthoredDefinitionError::EmptyArrangementOption(
                    name.clone(),
                ));
            }
        }
        if !issues.is_empty() {
            return Err(AuthoredDefinitionError::Invalid(issues));
        }
        if self.sources.is_empty() {
            return Err(AuthoredDefinitionError::Invalid(vec![
                ValidationIssue::error("sources", "at least one source binding is required"),
            ]));
        }
        for (name, binding) in &self.sources {
            if name.trim().is_empty() {
                return Err(AuthoredDefinitionError::Invalid(vec![
                    ValidationIssue::error("sources", "a source binding name is required"),
                ]));
            }
            match binding.revision_evidence {
                RevisionEvidence::PublicGeneration => match &binding.expects_generation {
                    Some(generation) if !generation.as_str().trim().is_empty() => {},
                    _ => {
                        return Err(AuthoredDefinitionError::EmptyExpectedGeneration(
                            name.clone(),
                        ));
                    },
                },
                RevisionEvidence::RuntimeVerified if binding.expects_generation.is_some() => {
                    return Err(AuthoredDefinitionError::RuntimeBindingHasPublicGeneration(
                        name.clone(),
                    ));
                },
                RevisionEvidence::RuntimeVerified => {},
            }
            self.draft_for(binding, self.arrangement.clone())
                .validate()
                .map_err(AuthoredDefinitionError::Invalid)?;
        }
        Ok(())
    }

    /// Select one named source and optionally apply an authored arrangement
    /// variant. The result retains the recipe id and records the selected input
    /// public generation for the executable host path.
    pub fn bind(
        &self,
        source_name: &str,
        variant: Option<&ProjectionVariant>,
    ) -> Result<ProjectionDefinition, AuthoredDefinitionError> {
        self.validate()?;
        let binding = self
            .sources
            .get(source_name)
            .ok_or_else(|| AuthoredDefinitionError::UnknownSource(source_name.to_owned()))?;
        if binding.revision_evidence == RevisionEvidence::RuntimeVerified {
            return Err(AuthoredDefinitionError::RuntimeBindingRequiresWitness(
                source_name.to_owned(),
            ));
        }
        let arrangement = self.arrangement_with_variant(variant)?;
        self.draft_for(binding, arrangement)
            .to_definition()
            .map_err(AuthoredDefinitionError::Invalid)
    }

    /// Bind a host-selected source whose only freshness evidence is an opaque
    /// runtime witness. This does not verify source authority or interpret the
    /// witness; both remain host-owned.
    pub fn bind_runtime<W>(
        &self,
        source_name: &str,
        runtime_source: RuntimeSourceBinding<W>,
        variant: Option<&ProjectionVariant>,
    ) -> Result<RuntimeProjectionBinding<W>, AuthoredDefinitionError> {
        self.validate()?;
        let declared = self
            .sources
            .get(source_name)
            .ok_or_else(|| AuthoredDefinitionError::UnknownSource(source_name.to_owned()))?;
        if declared.source != runtime_source.source {
            return Err(AuthoredDefinitionError::RuntimeSourceMismatch(
                source_name.to_owned(),
            ));
        }
        if declared.revision_evidence != RevisionEvidence::RuntimeVerified {
            return Err(AuthoredDefinitionError::RuntimeEvidenceRequired(
                source_name.to_owned(),
            ));
        }
        let arrangement = self.arrangement_with_variant(variant)?;
        let definition = self
            .draft_for(declared, arrangement)
            .to_definition()
            .map_err(AuthoredDefinitionError::Invalid)?;
        Ok(RuntimeProjectionBinding {
            definition,
            source: runtime_source,
        })
    }

    /// Serialize in declaration and sorted-map order for a host-readable
    /// receipt or a durable authoring store.
    pub fn to_json_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }

    fn arrangement_with_variant(
        &self,
        variant: Option<&ProjectionVariant>,
    ) -> Result<Arrangement, AuthoredDefinitionError> {
        let Some(variant) = variant else {
            return Ok(self.arrangement.clone());
        };
        if variant.definition_id != self.id {
            return Err(AuthoredDefinitionError::VariantForAnotherDefinition(
                variant.definition_id.clone(),
            ));
        }
        if variant.id.trim().is_empty() {
            return Err(AuthoredDefinitionError::EmptyVariantId);
        }
        let mut arrangement = self.arrangement.clone();
        for (name, value) in &variant.arrangement_options {
            if name.trim().is_empty() || value.trim().is_empty() {
                return Err(AuthoredDefinitionError::EmptyArrangementOption(
                    name.clone(),
                ));
            }
            arrangement.options.insert(name.clone(), value.clone());
        }
        Ok(arrangement)
    }

    fn draft_for(
        &self,
        binding: &ProjectionInputBinding,
        arrangement: Arrangement,
    ) -> ProjectionDraft {
        ProjectionDraft {
            version: self.version,
            id: self.id.clone(),
            label: self.label.clone(),
            source: binding.source.clone(),
            reading: self.reading.clone(),
            encoding: self.encoding.clone(),
            arrangement,
            interaction: self.interaction.clone(),
            appearance: self.appearance.clone(),
            provenance: Provenance {
                author: self.provenance.author.clone(),
                source_revision: binding.expects_generation.clone(),
                revision_evidence: binding.revision_evidence,
                note: self.provenance.note.clone(),
            },
        }
    }
}

impl std::fmt::Display for AuthoredDefinitionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Invalid(_) => write!(formatter, "the authored definition is invalid"),
            Self::UnknownSource(source) => write!(formatter, "unknown source binding {source:?}"),
            Self::EmptyExpectedGeneration(source) => {
                write!(
                    formatter,
                    "source binding {source:?} has no expected generation"
                )
            },
            Self::VariantForAnotherDefinition(definition) => {
                write!(
                    formatter,
                    "variant belongs to another definition {definition:?}"
                )
            },
            Self::EmptyVariantId => write!(formatter, "variant id is required"),
            Self::EmptyArrangementOption(option) => {
                write!(formatter, "arrangement option {option:?} is invalid")
            },
            Self::RuntimeSourceMismatch(source) => {
                write!(
                    formatter,
                    "runtime source does not match binding {source:?}"
                )
            },
            Self::RuntimeBindingRequiresWitness(source) => write!(
                formatter,
                "runtime-verified binding {source:?} requires a runtime witness"
            ),
            Self::RuntimeEvidenceRequired(source) => write!(
                formatter,
                "binding {source:?} requires a public generation, not a runtime witness"
            ),
            Self::RuntimeBindingHasPublicGeneration(source) => write!(
                formatter,
                "runtime-verified binding {source:?} cannot contain a public generation"
            ),
        }
    }
}

impl std::error::Error for AuthoredDefinitionError {}

impl ProjectionDefinition {
    /// Serialize in declaration order using the existing serde JSON stack.
    pub fn to_json_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }
}

fn required(issues: &mut Vec<ValidationIssue>, field: &str, value: &str, message: &str) {
    if value.trim().is_empty() {
        issues.push(ValidationIssue::error(field, message));
    }
}

fn channel_required(issues: &mut Vec<ValidationIssue>, field: &str, channel: &Channel) {
    let value = match channel {
        Channel::Field(value) | Channel::Constant(value) => value,
    };
    required(issues, field, value, "an encoding channel is required");
}

/// A field-specific validation problem suitable for a host editor or summary.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ValidationIssue {
    pub field: String,
    pub message: String,
    pub severity: ValidationSeverity,
}

impl ValidationIssue {
    fn error(field: &str, message: &str) -> Self {
        Self {
            field: field.to_string(),
            message: message.to_string(),
            severity: ValidationSeverity::Error,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationSeverity {
    Error,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_draft() -> ProjectionDraft {
        ProjectionDraft {
            version: PROJECTION_DEFINITION_VERSION,
            id: "notes-by-topic".into(),
            label: "Notes by topic".into(),
            source: SourceBinding {
                authority: "local-device".into(),
                domain: "notes".into(),
                resource: "graph:notes".into(),
            },
            reading: Reading {
                kind: "nodes".into(),
                key: "topic".into(),
                value: None,
            },
            encoding: Encoding {
                x: Channel::Field("topic_x".into()),
                y: Channel::Field("topic_y".into()),
                color: Some(Channel::Field("kind".into())),
                label: Some(Channel::Field("title".into())),
            },
            arrangement: Arrangement::default(),
            interaction: Interaction::default(),
            appearance: Appearance {
                realization: "canvas".into(),
                title: "Notes by topic".into(),
                theme: "light".into(),
            },
            provenance: Provenance {
                author: "mark".into(),
                source_revision: Some("rev-7".into()),
                revision_evidence: RevisionEvidence::PublicGeneration,
                note: "fixture".into(),
            },
        }
    }

    fn sources() -> BTreeMap<String, ProjectionInputBinding> {
        BTreeMap::from([
            (
                "personal".to_owned(),
                ProjectionInputBinding {
                    source: valid_draft().source,
                    expects_generation: Some("generation:personal".into()),
                    revision_evidence: RevisionEvidence::PublicGeneration,
                },
            ),
            (
                "shared".to_owned(),
                ProjectionInputBinding {
                    source: SourceBinding {
                        authority: "community".into(),
                        domain: "notes".into(),
                        resource: "graph:shared-notes".into(),
                    },
                    expects_generation: Some("generation:shared".into()),
                    revision_evidence: RevisionEvidence::PublicGeneration,
                },
            ),
        ])
    }

    fn authored() -> AuthoredProjectionDefinition {
        let draft = valid_draft();
        AuthoredProjectionDefinition {
            version: draft.version,
            id: draft.id,
            label: draft.label,
            sources: sources(),
            reading: draft.reading,
            encoding: draft.encoding,
            arrangement: draft.arrangement,
            interaction: draft.interaction,
            appearance: draft.appearance,
            provenance: draft.provenance,
        }
    }

    #[test]
    fn validation_reports_useful_fields() {
        let mut draft = valid_draft();
        draft.source.domain.clear();
        draft.arrangement.spacing = 0;
        draft
            .provenance
            .source_revision
            .as_mut()
            .expect("fixture has source revision")
            .clear();
        let issues = draft.validate().expect_err("invalid draft");
        assert!(issues.iter().any(|issue| issue.field == "source.domain"));
        assert!(
            issues
                .iter()
                .any(|issue| issue.field == "arrangement.spacing")
        );
        assert!(
            issues
                .iter()
                .any(|issue| issue.field == "provenance.source_revision")
        );
    }

    #[test]
    fn definition_json_is_repeatable_and_round_trips() {
        let definition = valid_draft().to_definition().expect("valid definition");
        let first = definition.to_json_bytes().expect("definition serializes");
        let second = definition.to_json_bytes().expect("definition serializes");
        assert_eq!(first, second);
        assert_eq!(
            first,
            br#"{"version":1,"id":"notes-by-topic","label":"Notes by topic","source":{"authority":"local-device","domain":"notes","resource":"graph:notes"},"reading":{"kind":"nodes","key":"topic","value":null},"encoding":{"x":{"kind":"field","value":"topic_x"},"y":{"kind":"field","value":"topic_y"},"color":{"kind":"field","value":"kind"},"label":{"kind":"field","value":"title"}},"arrangement":{"kind":"grid","direction":"horizontal","spacing":16},"interaction":{"selection":"single","pan":true,"zoom":true},"appearance":{"realization":"canvas","title":"Notes by topic","theme":"light"},"provenance":{"author":"mark","source_revision":"rev-7","note":"fixture"}}"#
        );
        assert_eq!(
            serde_json::from_slice::<ProjectionDefinition>(&first).expect("definition wire"),
            definition
        );
    }

    #[test]
    fn authored_definition_binds_each_named_source_without_changing_recipe() {
        let authored = authored();
        authored.validate().expect("complete authored definition");
        let personal = authored.bind("personal", None).expect("personal bind");
        let shared = authored.bind("shared", None).expect("shared bind");

        assert_eq!(personal.id, authored.id);
        assert_eq!(shared.id, authored.id);
        assert_ne!(personal.source, shared.source);
        assert_eq!(personal.reading, shared.reading);
        assert_eq!(
            personal.provenance.source_revision,
            Some(PublicSourceRevision::from("generation:personal"))
        );
        assert_eq!(
            shared.provenance.source_revision,
            Some(PublicSourceRevision::from("generation:shared"))
        );

        let bytes = authored.to_json_bytes().expect("authoring bytes");
        assert_eq!(
            serde_json::to_vec(authored.sources.get("personal").expect("personal source"))
                .expect("public source binding bytes"),
            br#"{"source":{"authority":"local-device","domain":"notes","resource":"graph:notes"},"expects_generation":"generation:personal"}"#
        );
        assert!(
            !String::from_utf8(bytes.clone())
                .expect("authoring json")
                .contains("revision_evidence")
        );
        assert_eq!(
            serde_json::from_slice::<AuthoredProjectionDefinition>(&bytes).expect("authoring wire"),
            authored
        );
    }

    #[test]
    fn runtime_witness_stays_out_of_the_durable_definition() {
        struct OpaqueWitness(&'static str);

        let mut authored = authored();
        let source = {
            let declared = authored
                .sources
                .get_mut("personal")
                .expect("personal source");
            declared.expects_generation = None;
            declared.revision_evidence = RevisionEvidence::RuntimeVerified;
            declared.source.clone()
        };
        let bound = authored
            .bind_runtime(
                "personal",
                RuntimeSourceBinding::new(source, OpaqueWitness("private-base-token")),
                None,
            )
            .expect("matching runtime source binds");

        assert_eq!(
            bound.with_witness(|witness| witness.0),
            "private-base-token"
        );
        assert_eq!(bound.definition().provenance.source_revision, None);
        assert_eq!(
            bound.definition().provenance.revision_evidence,
            RevisionEvidence::RuntimeVerified
        );
        let recorded_json = String::from_utf8(
            bound
                .into_recorded_definition()
                .to_json_bytes()
                .expect("definition bytes"),
        )
        .expect("definition json");
        assert!(recorded_json.contains("\"revision_evidence\":\"runtime_verified\""));
        assert!(!recorded_json.contains("source_revision"));
        assert!(!recorded_json.contains("private-base-token"));
    }

    #[test]
    fn runtime_binding_requires_runtime_evidence() {
        let authored = authored();
        let declared = authored.sources.get("personal").expect("personal source");
        let result = authored.bind_runtime(
            "personal",
            RuntimeSourceBinding::new(declared.source.clone(), ()),
            None,
        );
        match result {
            Err(error) => assert_eq!(
                error,
                AuthoredDefinitionError::RuntimeEvidenceRequired("personal".to_owned())
            ),
            Ok(_) => panic!("public-generation binding cannot accept a runtime witness"),
        }
    }

    #[test]
    fn ordinary_binding_cannot_bypass_a_runtime_witness() {
        let mut authored = authored();
        {
            let declared = authored
                .sources
                .get_mut("personal")
                .expect("personal source");
            declared.expects_generation = None;
            declared.revision_evidence = RevisionEvidence::RuntimeVerified;
        }

        assert_eq!(
            authored.bind("personal", None),
            Err(AuthoredDefinitionError::RuntimeBindingRequiresWitness(
                "personal".to_owned()
            ))
        );
    }

    #[test]
    fn variant_changes_arrangement_options_without_creating_another_recipe() {
        let authored = authored();
        let variant = ProjectionVariant {
            definition_id: authored.id.clone(),
            id: "compact".to_owned(),
            arrangement_options: BTreeMap::from([("guides".to_owned(), "false".to_owned())]),
        };
        let base = authored.bind("personal", None).expect("base bind");
        let compact = authored
            .bind("personal", Some(&variant))
            .expect("variant bind");

        assert_eq!(base.id, compact.id);
        assert_eq!(base.source, compact.source);
        assert_eq!(base.reading, compact.reading);
        assert_eq!(base.encoding, compact.encoding);
        assert_eq!(base.interaction, compact.interaction);
        assert_eq!(base.appearance, compact.appearance);
        assert_eq!(base.provenance, compact.provenance);
        assert_eq!(
            compact.arrangement.options.get("guides"),
            Some(&"false".to_owned())
        );
    }

    #[test]
    fn variant_for_another_definition_is_refused() {
        let authored = authored();
        let variant = ProjectionVariant {
            definition_id: "other.recipe".to_owned(),
            id: "compact".to_owned(),
            arrangement_options: BTreeMap::new(),
        };
        assert_eq!(
            authored.bind("personal", Some(&variant)),
            Err(AuthoredDefinitionError::VariantForAnotherDefinition(
                "other.recipe".to_owned()
            ))
        );
    }
}
