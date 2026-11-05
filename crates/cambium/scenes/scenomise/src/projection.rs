// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Portable execution of disclosed projection readings and relationship recipes.
//!
//! This module deliberately accepts a resolved dataset rather than a product
//! handle. A product adapter reads its own authority and supplies the source
//! binding, revision, typed fields, and source references here. The compiler
//! then checks the saved definition against that disclosure, emits a genuine
//! [`sceno::Score`], and asks `scenomise` to make a scene. It does not infer
//! fields, source meaning, or missing coordinates.

use std::collections::{BTreeMap, HashMap, HashSet};

use sceno::{
    Arrangement as ScenoArrangement, AxisValue, Footprint, InstanceId, Placement, Representation,
    Score, ScoreItem, Size2, SourceRef, Vec2,
};
use serde::{Deserialize, Serialize};

use crate::catalog::{ChannelUse, Family, Measure, OptionIssue};
use crate::host_dataset::occurrence_identity_issues;
use crate::registry::SolverRegistry;

pub use scenograph::relationship::{
    AUTHORED_ORDER_FACET, EXPLAINED_RELATIONSHIPS_FACET, OCCURRENCE_LABELS_FACET, RecipeEdit,
    RelationshipRecipe, RelationshipRecipeDraft, RelationshipRequirements, RelationshipSnapshot,
    relationship_recipe,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RelationshipDataset {
    pub dataset: ProjectionDataset,
    pub facets: std::collections::BTreeSet<String>,
    pub relationships: Vec<DisclosedRelationship>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RelationshipProvenance {
    pub source: SourceBinding,
    pub source_revision: PublicSourceRevision,
    pub method: String,
    pub method_version: u32,
    pub provider: String,
    pub evidence: Vec<SourceRef>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DisclosedRelationship {
    pub id: String,
    pub from_occurrence: String,
    pub to_occurrence: String,
    pub kind: String,
    pub label: String,
    pub explanation: String,
    pub provenance: RelationshipProvenance,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompiledRelationship {
    pub disclosure: DisclosedRelationship,
    pub from: InstanceId,
    pub to: InstanceId,
}

#[derive(Debug)]
pub struct CompiledRelationshipProjection {
    pub projection: CompiledProjection,
    pub relationships: Vec<CompiledRelationship>,
    pub selected_relationship: Option<String>,
}

/// Bounds checked before cloning datasets, building indices or solving scenes.
#[derive(Clone, Debug)]
pub struct RelationshipCompileLimits {
    pub max_occurrences: usize,
    pub max_relationships: usize,
    pub max_fields: usize,
    pub max_inputs: usize,
    pub max_text_bytes: usize,
    pub max_total_text_bytes: usize,
}
impl Default for RelationshipCompileLimits {
    fn default() -> Self {
        Self {
            max_occurrences: 256,
            max_relationships: 512,
            max_fields: 32,
            max_inputs: 16,
            max_text_bytes: 16 * 1024,
            max_total_text_bytes: 512 * 1024,
        }
    }
}

#[cfg(test)]
#[path = "projection_relationship_tests.rs"]
mod relationship_tests;

#[cfg(test)]
#[path = "projection_catalog_tests.rs"]
mod catalog_tests;

use scenograph::{Channel, ProjectionDefinition, PublicSourceRevision, SourceBinding};

/// Arrangement ids saved recipes carry. The [`catalog`](crate::catalog) reads
/// them as aliases of `grid` and `geographic`.
pub const GRID_ARRANGEMENT_ID: &str = "grid.default";
pub const SCATTER_ARRANGEMENT_ID: &str = "scatter.default";
const COORDINATES_DIRECTION: &str = "coordinates";
const NODES_READING_ID: &str = "nodes";

pub use scenograph::dataset::{
    ProjectionDataset, ProjectionFieldType, ProjectionOccurrence, ProjectionValue,
};

/// Item sizes the host measured for the representations it realizes.
///
/// Presentation stays with the host: the compiler places what the host says it
/// will draw, and writes in no size of its own.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ItemSizes {
    /// The footprint of one card.
    pub card: Size2,
}

/// One host's compiler: its measured item sizes and the solvers it registered.
///
/// A recipe's arrangement id resolves through the built-in
/// [`catalog`](crate::catalog) first and this registry second.
#[derive(Debug)]
pub struct ProjectionCompiler {
    sizes: ItemSizes,
    registry: SolverRegistry,
}

impl ProjectionCompiler {
    /// A compiler with no custom solvers.
    pub fn new(sizes: ItemSizes) -> Self {
        Self::with_registry(sizes, SolverRegistry::new())
    }

    pub fn with_registry(sizes: ItemSizes, registry: SolverRegistry) -> Self {
        Self { sizes, registry }
    }

    pub fn sizes(&self) -> ItemSizes {
        self.sizes
    }

    pub fn registry(&self) -> &SolverRegistry {
        &self.registry
    }

    /// Compile one saved definition against the current disclosed dataset.
    pub fn compile(
        &self,
        definition: &ProjectionDefinition,
        dataset: &ProjectionDataset,
    ) -> Result<CompiledProjection, Vec<CompileIssue>> {
        compile_inner(self, definition, dataset, None, None)
    }

    /// Revalidate a reading while retaining solved geometry when all solver
    /// inputs are unchanged. The host's item sizes travel in the score's
    /// footprints, so a resized card re-solves; labels and unrelated disclosed
    /// values do not.
    pub fn refresh(
        &self,
        previous: &CompiledProjection,
        definition: &ProjectionDefinition,
        dataset: &ProjectionDataset,
    ) -> Result<CompiledProjection, Vec<CompileIssue>> {
        compile_inner(self, definition, dataset, None, Some(previous))
    }

    /// Reopen a saved definition and restore selection only when that exact
    /// occurrence still exists in the resolved dataset.
    pub fn compile_snapshot(
        &self,
        snapshot: &ProjectionSnapshot,
        dataset: &ProjectionDataset,
    ) -> Result<CompiledProjection, Vec<CompileIssue>> {
        compile_inner(
            self,
            &snapshot.definition,
            dataset,
            snapshot.selected_occurrence.as_deref(),
            None,
        )
    }

    /// Compile current disclosed evidence; never infer relationships from
    /// numeric fields, compare domain values, execute scripts or acquire an
    /// authority.
    pub fn compile_relationship_snapshot(
        &self,
        snapshot: &RelationshipSnapshot,
        disclosed: &RelationshipDataset,
    ) -> Result<CompiledRelationshipProjection, Vec<CompileIssue>> {
        self.compile_relationship_snapshot_with_limits(
            snapshot,
            disclosed,
            &RelationshipCompileLimits::default(),
        )
    }

    pub fn compile_relationship_snapshot_with_limits(
        &self,
        snapshot: &RelationshipSnapshot,
        disclosed: &RelationshipDataset,
        limits: &RelationshipCompileLimits,
    ) -> Result<CompiledRelationshipProjection, Vec<CompileIssue>> {
        relationship_snapshot(self, snapshot, disclosed, limits)
    }
}

/// Field-specific compiler feedback suitable for the authoring host.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompileIssue {
    pub field: String,
    pub message: String,
}

impl CompileIssue {
    fn new(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            message: message.into(),
        }
    }
}

/// The scene and exact reverse mappings a host realization needs.
#[derive(Debug)]
pub struct CompiledProjection {
    pub score: Score,
    pub scene: sceno::Scene,
    pub labels: HashMap<InstanceId, String>,
    pub occurrence_by_instance: HashMap<InstanceId, String>,
    pub instance_by_occurrence: BTreeMap<String, InstanceId>,
    pub selected: Option<InstanceId>,
    /// True when a validated refresh reused placement from the preceding result.
    pub placement_reused: bool,
}

/// The complete save/reopen payload for this proof. Pan and zoom do not belong
/// here until the host implements their durable view state.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProjectionSnapshot {
    pub definition: ProjectionDefinition,
    pub selected_occurrence: Option<String>,
}

/// Where a recipe's arrangement resolved.
enum Target {
    Builtin(Family),
    Custom(String),
}

impl Target {
    /// How the family reads x and y; a custom solver reads what they disclose.
    fn channels(&self) -> Option<ChannelUse> {
        match self {
            Self::Builtin(family) => Some(family.channels()),
            Self::Custom(_) => None,
        }
    }
}

fn resolve_target(
    compiler: &ProjectionCompiler,
    definition: &ProjectionDefinition,
) -> Option<Target> {
    let kind = definition.arrangement.kind.as_str();
    Family::resolve(kind).map(Target::Builtin).or_else(|| {
        compiler
            .registry
            .resolve(kind)
            .map(|_| Target::Custom(kind.to_owned()))
    })
}

const ANY_FIELD_TYPE: &[ProjectionFieldType] = &[
    ProjectionFieldType::Text,
    ProjectionFieldType::Number,
    ProjectionFieldType::Boolean,
];

pub(crate) fn stable_generation(dataset: &ProjectionDataset) -> u64 {
    let mut value = 0xcbf2_9ce4_8422_2325u64;
    for byte in dataset.revision.bytes().chain([0]) {
        value ^= u64::from(byte);
        value = value.wrapping_mul(0x0000_0100_0000_01b3);
    }
    let mut ids: Vec<_> = dataset
        .occurrences
        .iter()
        .map(|occurrence| occurrence.occurrence_id.as_bytes())
        .collect();
    ids.sort();
    for id in ids {
        for byte in id.iter().copied().chain([0]) {
            value ^= u64::from(byte);
            value = value.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
    value
}

#[path = "projection/relationships.rs"]
mod relationships;

use relationships::relationship_snapshot;

pub(crate) use relationships::relationship_disclosure_issues;

#[path = "projection/validation.rs"]
mod validation;

use validation::card_issue;

use validation::validation_issues;

use validation::field_name;

use validation::number;

#[path = "projection/compilation.rs"]
mod compilation;

use compilation::compile_inner;

use compilation::disclose;

use compilation::option_issues;

use compilation::dense_rank_key;
