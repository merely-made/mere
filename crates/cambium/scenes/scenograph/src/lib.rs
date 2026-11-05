// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Scenograph: host-neutral projection authoring definitions.
//!
//! This crate owns durable authoring data, local validation, source binding,
//! deterministic JSON, and the resolved [`dataset`] a host compiles against.
//! Hosts own their editor UI, source acquisition, persistence policy, item
//! measurement, and realization; arrangement resolution and compilation are
//! shared in `scenomise`. It deliberately has no runtime, solver, widget, or
//! product dependency.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

pub mod attention;
pub mod dataset;
pub mod dynamics;
pub use dynamics::{DYNAMICS_SLOT_VERSION, DynamicsSlot, DynamicsVariant};
pub mod link;
pub mod options;
pub mod presentation;
pub mod relationship;
pub mod seed;
pub mod suggestion;
pub mod swatch;

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
    /// Optional motion; the binding host validates its opaque specification.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dynamics: Option<DynamicsSlot>,
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
    /// Optional motion; the binding host validates its opaque specification.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dynamics: Option<DynamicsSlot>,
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
    /// Optional motion; the binding host validates its opaque specification.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dynamics: Option<DynamicsSlot>,
    pub interaction: Interaction,
    pub appearance: Appearance,
    /// This revision belongs to the authored recipe. Bound definitions receive
    /// their selected input generation instead.
    pub provenance: Provenance,
}

/// An authored delta over one definition, rather than another definition.
///
/// Carries arrangement options and the dynamics replacement required by the
/// swatch grid (F194). Other section deltas need their own forcing consumer.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProjectionVariant {
    pub definition_id: String,
    pub id: String,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub arrangement_options: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dynamics: Option<DynamicsVariant>,
}

/// Why a reusable authored definition could not be bound to a host payload.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AuthoredDefinitionError {
    Invalid(Vec<ValidationIssue>),
    UnknownSource(String),
    EmptyExpectedGeneration(String),
    VariantForAnotherDefinition(String),
    EmptyVariantId,
    Dynamics(String),
    EmptyArrangementOption(String),
    RuntimeSourceMismatch(String),
    RuntimeBindingRequiresWitness(String),
    RuntimeEvidenceRequired(String),
    RuntimeBindingHasPublicGeneration(String),
}

impl Default for ProjectionDraft {
    fn default() -> Self {
        Self::new()
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
            Self::Dynamics(error) => write!(formatter, "{error}"),
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
#[path = "tests.rs"]
mod tests;

#[path = "draft.rs"]
mod draft;

#[path = "authored.rs"]
mod authored;
