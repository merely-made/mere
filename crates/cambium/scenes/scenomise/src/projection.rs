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
    Arrangement as ScenoArrangement, Footprint, Geographic, Grid, InstanceId, Placement,
    Representation, Score, ScoreItem, Size2, SourceRef, Vec2,
};
use serde::{Deserialize, Serialize};

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

/// Compile current disclosed evidence; never infer relationships from numeric
/// fields, compare domain values, execute scripts or acquire an authority.
pub fn compile_relationship_snapshot(
    snapshot: &RelationshipSnapshot,
    disclosed: &RelationshipDataset,
) -> Result<CompiledRelationshipProjection, Vec<CompileIssue>> {
    compile_relationship_snapshot_with_limits(
        snapshot,
        disclosed,
        &RelationshipCompileLimits::default(),
    )
}

pub fn compile_relationship_snapshot_with_limits(
    snapshot: &RelationshipSnapshot,
    disclosed: &RelationshipDataset,
    limits: &RelationshipCompileLimits,
) -> Result<CompiledRelationshipProjection, Vec<CompileIssue>> {
    relationship_bounds(snapshot, disclosed, limits)?;
    let mut issues = snapshot
        .validate()
        .err()
        .unwrap_or_default()
        .into_iter()
        .map(|issue| CompileIssue::new(issue.field, issue.message))
        .collect::<Vec<_>>();
    let dataset = &disclosed.dataset;
    for required in [
        AUTHORED_ORDER_FACET,
        OCCURRENCE_LABELS_FACET,
        EXPLAINED_RELATIONSHIPS_FACET,
        snapshot.recipe.requirements.facet.as_str(),
    ] {
        if !disclosed.facets.contains(required) {
            issues.push(CompileIssue::new(
                format!("requirements.facets.{required}"),
                format!("source does not disclose semantic facet {required}"),
            ));
        }
    }
    let definition = &snapshot.recipe.definition;
    if definition.arrangement.kind != GRID_ARRANGEMENT_ID {
        issues.push(CompileIssue::new("arrangement.kind", "authored occurrence order requires grid.default; it is not a physical scatter coordinate"));
    }
    if definition.reading.kind != "nodes"
        || definition.reading.key != "occurrence_id"
        || definition.reading.value.is_some()
        || definition.encoding.x != Channel::Field("order".into())
        || definition.encoding.y != Channel::Constant("0".into())
        || definition.encoding.label != Some(Channel::Field("label".into()))
    {
        issues.push(CompileIssue::new("definition.roles", "relationship recipes require authored order, occurrence identity, labels, and the constant layout row"));
    }
    if dataset.revision.as_str().trim().is_empty() {
        issues.push(CompileIssue::new(
            "dataset.revision",
            "a public source revision is required",
        ));
    }
    if dataset.occurrences.len() < snapshot.recipe.requirements.min_occurrences as usize {
        issues.push(CompileIssue::new(
            "requirements.min_occurrences",
            "source has fewer disclosed occurrences than the recipe requires",
        ));
    }
    let ids: HashSet<_> = dataset
        .occurrences
        .iter()
        .map(|occurrence| occurrence.occurrence_id.as_str())
        .collect();
    let mut orders = HashSet::new();
    for occurrence in &dataset.occurrences {
        let valid_order = number(occurrence, "order").filter(|order| {
            order.is_finite() && *order >= 0.0 && order.fract() == 0.0 && *order <= u32::MAX as f64
        });
        if valid_order.is_none() || !orders.insert(dense_rank_key(valid_order.unwrap_or_default()))
        {
            issues.push(CompileIssue::new(
                format!("occurrences.{}.order", occurrence.occurrence_id),
                "authored order must be a distinct nonnegative integer per occurrence",
            ));
        }
    }
    if dataset.fields.contains_key("__recipe_layout_row") {
        issues.push(CompileIssue::new(
            "dataset.fields",
            "reserved recipe layout row must not be disclosed as source truth",
        ));
    }
    let mut relationship_ids = HashSet::new();
    for relationship in &disclosed.relationships {
        let prefix = format!("relationships.{}", relationship.id);
        if relationship.id.trim().is_empty() || !relationship_ids.insert(&relationship.id) {
            issues.push(CompileIssue::new(
                format!("{prefix}.id"),
                "relationship identities must be nonempty and unique",
            ));
        }
        if !ids.contains(relationship.from_occurrence.as_str())
            || !ids.contains(relationship.to_occurrence.as_str())
            || relationship.from_occurrence == relationship.to_occurrence
        {
            issues.push(CompileIssue::new(
                format!("{prefix}.endpoints"),
                "a relationship must name two distinct disclosed occurrence identities",
            ));
        }
        for (field, value) in [
            ("kind", &relationship.kind),
            ("label", &relationship.label),
            ("explanation", &relationship.explanation),
            ("provenance.method", &relationship.provenance.method),
            ("provenance.provider", &relationship.provenance.provider),
        ] {
            if value.trim().is_empty() {
                issues.push(CompileIssue::new(
                    format!("{prefix}.{field}"),
                    "disclosed meaning and method provenance cannot be empty",
                ));
            }
        }
        if relationship.provenance.method_version == 0 {
            issues.push(CompileIssue::new(
                format!("{prefix}.provenance.method_version"),
                "a positive disclosed method version is required",
            ));
        }
        if relationship.provenance.source != dataset.source
            || relationship.provenance.source_revision != dataset.revision
        {
            issues.push(CompileIssue::new(
                format!("{prefix}.provenance.source_revision"),
                "relationship provenance must match the exact disclosed source and public revision",
            ));
        }
        if relationship.provenance.evidence.is_empty()
            || relationship
                .provenance
                .evidence
                .iter()
                .any(|source| source.adapter.trim().is_empty() || source.id.trim().is_empty())
        {
            issues.push(CompileIssue::new(
                format!("{prefix}.provenance.evidence"),
                "relationship evidence needs exact source references",
            ));
        }
    }
    let relationships: Vec<_> = disclosed
        .relationships
        .iter()
        .filter(|relationship| {
            snapshot
                .recipe
                .relationship_kind
                .as_ref()
                .is_none_or(|kind| kind == &relationship.kind)
        })
        .collect();
    if snapshot.recipe.relationship_kind.is_some() && relationships.is_empty() {
        issues.push(CompileIssue::new(
            "relationship_kind",
            "the requested semantic relationship kind is not disclosed by this source",
        ));
    }
    if relationships.len() < snapshot.recipe.requirements.min_relationships as usize {
        issues.push(CompileIssue::new(
            "requirements.min_relationships",
            "source has fewer applicable explained relationships than the recipe requires",
        ));
    }
    if snapshot
        .selected_relationship
        .as_ref()
        .is_some_and(|selected| {
            !relationships
                .iter()
                .any(|relationship| &relationship.id == selected)
        })
    {
        issues.push(CompileIssue::new(
            "selected_relationship",
            "the saved selected relationship is absent from this source or excluded by the recipe",
        ));
    }
    if !issues.is_empty() {
        return Err(issues);
    }
    let mut bound = snapshot
        .recipe
        .definition
        .bind(&snapshot.source_name, None)
        .map_err(|error| {
            vec![CompileIssue::new(
                "source_name",
                format!("binding refused: {error:?}"),
            )]
        })?;
    // This row is an explicit layout constant, not a claimed domain facet or
    // computed comparison result. Reuse the portable field compiler internally.
    bound.encoding.y = Channel::Field("__recipe_layout_row".into());
    let mut execution_dataset = dataset.clone();
    execution_dataset
        .fields
        .insert("__recipe_layout_row".into(), ProjectionFieldType::Number);
    for occurrence in &mut execution_dataset.occurrences {
        occurrence
            .values
            .insert("__recipe_layout_row".into(), ProjectionValue::Number(0.0));
    }
    let mut projection = compile_snapshot(
        &ProjectionSnapshot {
            definition: bound,
            selected_occurrence: snapshot.selected_occurrence.clone(),
        },
        &execution_dataset,
    )?;
    let mut compiled = Vec::with_capacity(relationships.len());
    let mut ordered = relationships;
    ordered.sort_by(|a, b| a.id.cmp(&b.id));
    for relationship in ordered {
        let from = projection.instance_by_occurrence[&relationship.from_occurrence];
        let to = projection.instance_by_occurrence[&relationship.to_occurrence];
        projection.scene.relations.push(sceno::RoutedRelation {
            from,
            to,
            space: sceno::Scene::WORLD,
            points: vec![
                projection.scene.items[from.0 as usize].transform.translate,
                projection.scene.items[to.0 as usize].transform.translate,
            ],
            kind: Some(relationship.kind.clone()),
            weight: None,
        });
        compiled.push(CompiledRelationship {
            disclosure: relationship.clone(),
            from,
            to,
        });
    }
    Ok(CompiledRelationshipProjection {
        projection,
        relationships: compiled,
        selected_relationship: snapshot.selected_relationship.clone(),
    })
}

fn relationship_bounds(
    snapshot: &RelationshipSnapshot,
    disclosure: &RelationshipDataset,
    limits: &RelationshipCompileLimits,
) -> Result<(), Vec<CompileIssue>> {
    let fail = |field: &str| {
        vec![CompileIssue::new(
            field,
            "relationship compiler disclosure limit exceeded",
        )]
    };
    if disclosure.dataset.occurrences.len() > limits.max_occurrences {
        return Err(fail("limits.occurrences"));
    }
    if disclosure.relationships.len() > limits.max_relationships {
        return Err(fail("limits.relationships"));
    }
    if disclosure.dataset.fields.len() > limits.max_fields
        || disclosure.facets.len() > limits.max_fields
    {
        return Err(fail("limits.fields"));
    }
    if snapshot.recipe.definition.sources.len() > limits.max_inputs {
        return Err(fail("limits.inputs"));
    }
    let mut total = 0usize;
    let mut check = |value: &str| -> Result<(), Vec<CompileIssue>> {
        total = total
            .checked_add(value.len())
            .ok_or_else(|| fail("limits.text"))?;
        if value.len() > limits.max_text_bytes || total > limits.max_total_text_bytes {
            return Err(fail("limits.text"));
        }
        Ok(())
    };
    fn source(value: &SourceBinding) -> [&String; 3] {
        [&value.authority, &value.domain, &value.resource]
    }
    for value in source(&disclosure.dataset.source) {
        check(value)?;
    }
    check(disclosure.dataset.revision.as_str())?;
    for field in disclosure.dataset.fields.keys() {
        check(field)?;
    }
    for facet in &disclosure.facets {
        check(facet)?;
    }
    for occurrence in &disclosure.dataset.occurrences {
        if occurrence.values.len() > limits.max_fields {
            return Err(fail("limits.fields"));
        }
        check(&occurrence.occurrence_id)?;
        check(&occurrence.source.adapter)?;
        check(&occurrence.source.id)?;
        for (name, value) in &occurrence.values {
            check(name)?;
            match value {
                ProjectionValue::Text(text) => check(text)?,
                ProjectionValue::Number(number) if !number.is_finite() => {
                    return Err(vec![CompileIssue::new(
                        "dataset.values",
                        "all disclosed numbers must be finite",
                    )]);
                },
                _ => (),
            }
        }
    }
    for relationship in &disclosure.relationships {
        if relationship.provenance.evidence.len() > limits.max_occurrences {
            return Err(fail("limits.evidence"));
        }
        for text in [
            &relationship.id,
            &relationship.from_occurrence,
            &relationship.to_occurrence,
            &relationship.kind,
            &relationship.label,
            &relationship.explanation,
            &relationship.provenance.method,
            &relationship.provenance.provider,
        ] {
            check(text)?;
        }
        for text in source(&relationship.provenance.source) {
            check(text)?;
        }
        check(relationship.provenance.source_revision.as_str())?;
        for evidence in &relationship.provenance.evidence {
            check(&evidence.adapter)?;
            check(&evidence.id)?;
        }
    }
    let definition = &snapshot.recipe.definition;
    for text in [
        &definition.id,
        &definition.label,
        &definition.reading.kind,
        &definition.reading.key,
        &definition.arrangement.kind,
        &definition.arrangement.direction,
        &definition.appearance.realization,
        &definition.appearance.title,
        &definition.appearance.theme,
        &definition.provenance.author,
        &definition.provenance.note,
        &snapshot.source_name,
        &snapshot.recipe.requirements.facet,
    ] {
        check(text)?;
    }
    for channel in [&definition.encoding.x, &definition.encoding.y]
        .into_iter()
        .chain(definition.encoding.color.iter())
        .chain(definition.encoding.label.iter())
    {
        match channel {
            Channel::Field(value) | Channel::Constant(value) => check(value)?,
        }
    }
    for text in [
        definition.reading.value.as_deref(),
        definition
            .provenance
            .source_revision
            .as_ref()
            .map(PublicSourceRevision::as_str),
        snapshot.recipe.relationship_kind.as_deref(),
        snapshot.selected_occurrence.as_deref(),
        snapshot.selected_relationship.as_deref(),
    ]
    .into_iter()
    .flatten()
    {
        check(text)?;
    }
    if definition.arrangement.options.len() > limits.max_fields {
        return Err(fail("limits.fields"));
    }
    for (key, value) in &definition.arrangement.options {
        check(key)?;
        check(value)?;
    }
    for (name, binding) in &definition.sources {
        check(name)?;
        for value in source(&binding.source) {
            check(value)?;
        }
        if let Some(revision) = &binding.expects_generation {
            check(revision.as_str())?;
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "projection_relationship_tests.rs"]
mod relationship_tests;

use scenograph::{Channel, ProjectionDefinition, PublicSourceRevision, SourceBinding};

/// The two stable arrangement ids Graphshell currently compiles.
pub const GRID_ARRANGEMENT_ID: &str = "grid.default";
pub const SCATTER_ARRANGEMENT_ID: &str = "scatter.default";
const COORDINATES_DIRECTION: &str = "coordinates";
const NODES_READING_ID: &str = "nodes";

/// The type a product disclosed for a field in one resolved dataset.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectionFieldType {
    Text,
    Number,
    Boolean,
}

/// One source value, kept small enough to make an adapter's disclosure plain.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind", content = "value")]
pub enum ProjectionValue {
    Text(String),
    Number(f64),
    Boolean(bool),
}

impl ProjectionValue {
    fn field_type(&self) -> ProjectionFieldType {
        match self {
            Self::Text(_) => ProjectionFieldType::Text,
            Self::Number(_) => ProjectionFieldType::Number,
            Self::Boolean(_) => ProjectionFieldType::Boolean,
        }
    }

    fn text(&self) -> Option<&str> {
        match self {
            Self::Text(value) => Some(value),
            _ => None,
        }
    }

    fn number(&self) -> Option<f64> {
        match self {
            Self::Number(value) => Some(*value),
            _ => None,
        }
    }
}

/// One occurrence in a product's resolved reading.
///
/// `occurrence_id` is Graphshell's selection/persistence identity. `source`
/// remains source truth identity, so two occurrences may intentionally name
/// the same source.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProjectionOccurrence {
    pub occurrence_id: String,
    pub source: SourceRef,
    pub values: BTreeMap<String, ProjectionValue>,
}

/// A product-resolved dataset supplied to the compiler without a product
/// dependency or a portable product-data contract.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProjectionDataset {
    pub source: SourceBinding,
    pub revision: PublicSourceRevision,
    pub fields: BTreeMap<String, ProjectionFieldType>,
    pub occurrences: Vec<ProjectionOccurrence>,
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

/// The scene and exact reverse mappings a Graphshell realization needs.
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

/// Compile one saved definition against the current disclosed dataset.
pub fn compile(
    definition: &ProjectionDefinition,
    dataset: &ProjectionDataset,
) -> Result<CompiledProjection, Vec<CompileIssue>> {
    compile_inner(definition, dataset, None, None)
}

/// Revalidate a reading while retaining solved geometry when all solver inputs
/// are unchanged. Labels and unrelated disclosed values do not require solving
/// this fixed-footprint card representation. Future measured representations
/// must put their measurements in the score before this comparison.
pub fn refresh(
    previous: &CompiledProjection,
    definition: &ProjectionDefinition,
    dataset: &ProjectionDataset,
) -> Result<CompiledProjection, Vec<CompileIssue>> {
    compile_inner(definition, dataset, None, Some(previous))
}

/// Reopen a saved definition and restore selection only when that exact
/// occurrence still exists in the resolved dataset.
pub fn compile_snapshot(
    snapshot: &ProjectionSnapshot,
    dataset: &ProjectionDataset,
) -> Result<CompiledProjection, Vec<CompileIssue>> {
    compile_inner(
        &snapshot.definition,
        dataset,
        snapshot.selected_occurrence.as_deref(),
        None,
    )
}

fn compile_inner(
    definition: &ProjectionDefinition,
    dataset: &ProjectionDataset,
    selected_occurrence: Option<&str>,
    previous: Option<&CompiledProjection>,
) -> Result<CompiledProjection, Vec<CompileIssue>> {
    let mut issues = validation_issues(definition, dataset);
    if let Some(selected) = selected_occurrence
        && !dataset
            .occurrences
            .iter()
            .any(|occurrence| occurrence.occurrence_id == selected)
    {
        issues.push(CompileIssue::new(
            "selected_occurrence",
            "the saved selected occurrence is absent from this dataset revision",
        ));
    }
    if !issues.is_empty() {
        return Err(issues);
    }

    let label_field = definition
        .encoding
        .label
        .as_ref()
        .and_then(|channel| field_name(channel, "encoding.label"))
        .expect("validated label field");
    let x_field = field_name(&definition.encoding.x, "encoding.x").expect("validated x field");
    let y_field = field_name(&definition.encoding.y, "encoding.y").expect("validated y field");
    let arrangement = arrangement_for(definition).expect("validated arrangement");

    let mut occurrences: Vec<_> = dataset.occurrences.iter().collect();
    occurrences.sort_by(|left, right| left.occurrence_id.cmp(&right.occurrence_id));
    let grid_ranks = (definition.arrangement.kind == GRID_ARRANGEMENT_ID).then(|| {
        (
            dense_ranks(&occurrences, x_field),
            dense_ranks(&occurrences, y_field),
        )
    });
    let mut score = Score::new(arrangement);
    score.generation = stable_generation(dataset);
    for (ordinal, occurrence) in occurrences.iter().enumerate() {
        let x = number(occurrence, x_field).expect("validated x value");
        let y = number(occurrence, y_field).expect("validated y value");
        score.items.push(ScoreItem {
            source: occurrence.source.clone(),
            ordinal: ordinal as u32,
            footprint: Footprint::Rect {
                size: Size2::new(164.0, 68.0),
            },
            representation: Representation::Card,
            placement: if let Some((x_ranks, y_ranks)) = &grid_ranks {
                // Dense numeric ranks preserve the two declared encodings
                // without allocating ninety-six empty rows for a 96 bpm card.
                Placement::Cell {
                    column: x_ranks[&dense_rank_key(x)],
                    row: y_ranks[&dense_rank_key(y)],
                }
            } else {
                placement_for(definition, x, y).expect("validated placement")
            },
            layer: 0,
            visible: true,
            axis: None,
            embedding: None,
            weight: None,
        });
    }
    // Compare exact solver inputs, not revision hashes: source revisions can
    // change without affecting placement, and unchanged revisions cannot excuse
    // changed data. Ordered occurrence identity also protects repeated sources.
    let reusable = previous.filter(|old| {
        old.score.version == score.version
            && old.score.arrangement == score.arrangement
            && old.score.items == score.items
            && old.score.holds == score.holds
            && occurrences.iter().enumerate().all(|(index, occurrence)| {
                old.occurrence_by_instance.get(&InstanceId(index as u32))
                    == Some(&occurrence.occurrence_id)
            })
    });
    let placement_reused = reusable.is_some();
    let scene = if let Some(old) = reusable {
        let mut scene = old.scene.clone();
        scene.generation = score.generation;
        scene
    } else {
        crate::solve(&score)
    };
    let mut labels = HashMap::new();
    let mut occurrence_by_instance = HashMap::new();
    let mut instance_by_occurrence = BTreeMap::new();
    for (index, occurrence) in occurrences.into_iter().enumerate() {
        let instance = InstanceId(index as u32);
        labels.insert(
            instance,
            occurrence
                .values
                .get(label_field)
                .and_then(ProjectionValue::text)
                .expect("validated label value")
                .to_owned(),
        );
        occurrence_by_instance.insert(instance, occurrence.occurrence_id.clone());
        instance_by_occurrence.insert(occurrence.occurrence_id.clone(), instance);
    }
    let selected = selected_occurrence.and_then(|id| instance_by_occurrence.get(id).copied());
    Ok(CompiledProjection {
        score,
        scene,
        labels,
        occurrence_by_instance,
        instance_by_occurrence,
        selected,
        placement_reused,
    })
}

/// Builds the same dense ranks as the former per-occurrence closure, once for
/// each grid channel. Finite values have one `==` equivalence class per bit
/// pattern except signed zero, which deliberately shares a rank.
fn dense_ranks(occurrences: &[&ProjectionOccurrence], field: &str) -> BTreeMap<u64, i32> {
    let mut values: Vec<_> = occurrences
        .iter()
        .map(|occurrence| number(occurrence, field).expect("validated numeric value"))
        .collect();
    values.sort_by(f64::total_cmp);
    values.dedup();
    values
        .into_iter()
        .enumerate()
        .map(|(rank, value)| (dense_rank_key(value), rank as i32))
        .collect()
}

fn dense_rank_key(value: f64) -> u64 {
    if value == 0.0 { 0 } else { value.to_bits() }
}

fn validation_issues(
    definition: &ProjectionDefinition,
    dataset: &ProjectionDataset,
) -> Vec<CompileIssue> {
    let mut issues: Vec<_> = definition
        .clone()
        .to_json_bytes()
        .ok()
        .and_then(|_| {
            // Definitions are normally promoted through ProjectionDraft. Keep
            // the compiler independently defensive for persisted input.
            let draft = scenograph::ProjectionDraft {
                version: definition.version,
                id: definition.id.clone(),
                label: definition.label.clone(),
                source: definition.source.clone(),
                reading: definition.reading.clone(),
                encoding: definition.encoding.clone(),
                arrangement: definition.arrangement.clone(),
                interaction: definition.interaction.clone(),
                appearance: definition.appearance.clone(),
                provenance: definition.provenance.clone(),
            };
            draft.validate().err()
        })
        .unwrap_or_default()
        .into_iter()
        .map(|issue| CompileIssue::new(issue.field, issue.message))
        .collect();

    source_matches(&mut issues, definition, dataset);
    if definition.reading.value.is_some() {
        issues.push(CompileIssue::new(
            "reading.value",
            "nodes does not accept a value aggregation",
        ));
    }
    if definition.interaction.selection != scenograph::SelectionMode::Single
        || definition.interaction.pan
        || definition.interaction.zoom
    {
        issues.push(CompileIssue::new(
            "interaction",
            "this realization supports single selection with a fitted viewport",
        ));
    }
    if definition.appearance.realization != "canvas" {
        issues.push(CompileIssue::new(
            "appearance.realization",
            "supported realization is canvas",
        ));
    }
    if definition.appearance.theme != "slate" {
        issues.push(CompileIssue::new(
            "appearance.theme",
            "supported theme is slate",
        ));
    }
    if definition.reading.kind != NODES_READING_ID {
        issues.push(CompileIssue::new(
            "reading.kind",
            "only the nodes reading is executable in this compiler",
        ));
    }
    required_field(
        &mut issues,
        dataset,
        "reading.key",
        &definition.reading.key,
        ProjectionFieldType::Text,
    );
    required_channel_field(
        &mut issues,
        dataset,
        "encoding.x",
        &definition.encoding.x,
        ProjectionFieldType::Number,
    );
    required_channel_field(
        &mut issues,
        dataset,
        "encoding.y",
        &definition.encoding.y,
        ProjectionFieldType::Number,
    );
    let Some(label) = definition.encoding.label.as_ref() else {
        issues.push(CompileIssue::new(
            "encoding.label",
            "an executable projection needs a text label field",
        ));
        validate_arrangement(&mut issues, definition);
        validate_occurrences(&mut issues, definition, dataset);
        return issues;
    };
    required_channel_field(
        &mut issues,
        dataset,
        "encoding.label",
        label,
        ProjectionFieldType::Text,
    );
    if definition.encoding.color.is_some() {
        issues.push(CompileIssue::new(
            "encoding.color",
            "color encoding is not implemented by this compiler",
        ));
    }
    validate_arrangement(&mut issues, definition);
    validate_occurrences(&mut issues, definition, dataset);
    issues
}

fn source_matches(
    issues: &mut Vec<CompileIssue>,
    definition: &ProjectionDefinition,
    dataset: &ProjectionDataset,
) {
    for (field, expected, actual) in [
        (
            "source.authority",
            &definition.source.authority,
            &dataset.source.authority,
        ),
        (
            "source.domain",
            &definition.source.domain,
            &dataset.source.domain,
        ),
        (
            "source.resource",
            &definition.source.resource,
            &dataset.source.resource,
        ),
    ] {
        if expected != actual {
            issues.push(CompileIssue::new(
                field,
                "definition does not match the resolved dataset",
            ));
        }
    }
    if definition.provenance.source_revision.as_ref() != Some(&dataset.revision) {
        issues.push(CompileIssue::new(
            "provenance.source_revision",
            "definition does not match the resolved dataset",
        ));
    }
}

fn required_field(
    issues: &mut Vec<CompileIssue>,
    dataset: &ProjectionDataset,
    target: &str,
    field: &str,
    expected: ProjectionFieldType,
) {
    match dataset.fields.get(field) {
        Some(actual) if *actual == expected => {},
        Some(_) => issues.push(CompileIssue::new(target, "field has an incompatible type")),
        None => issues.push(CompileIssue::new(
            target,
            "field is not disclosed by this dataset",
        )),
    }
}

fn required_channel_field(
    issues: &mut Vec<CompileIssue>,
    dataset: &ProjectionDataset,
    target: &str,
    channel: &Channel,
    expected: ProjectionFieldType,
) {
    match channel {
        Channel::Field(field) => required_field(issues, dataset, target, field, expected),
        Channel::Constant(_) => issues.push(CompileIssue::new(
            target,
            "constants are not executable for this channel",
        )),
    }
}

fn validate_arrangement(issues: &mut Vec<CompileIssue>, definition: &ProjectionDefinition) {
    match definition.arrangement.kind.as_str() {
        GRID_ARRANGEMENT_ID | SCATTER_ARRANGEMENT_ID => {},
        _ => issues.push(CompileIssue::new(
            "arrangement.kind",
            "supported arrangements are grid.default and scatter.default",
        )),
    }
    if definition.arrangement.direction != COORDINATES_DIRECTION {
        issues.push(CompileIssue::new(
            "arrangement.direction",
            "this compiler requires the coordinates direction",
        ));
    }
    if !definition.arrangement.options.is_empty() {
        issues.push(CompileIssue::new(
            "arrangement.options",
            "this compiler does not support arrangement options",
        ));
    }
}

fn validate_occurrences(
    issues: &mut Vec<CompileIssue>,
    definition: &ProjectionDefinition,
    dataset: &ProjectionDataset,
) {
    let mut seen = HashSet::new();
    let label = definition
        .encoding
        .label
        .as_ref()
        .and_then(|channel| field_name(channel, ""));
    let x = field_name(&definition.encoding.x, "");
    let y = field_name(&definition.encoding.y, "");
    for occurrence in &dataset.occurrences {
        let prefix = format!("occurrences.{}", occurrence.occurrence_id);
        if occurrence.occurrence_id.trim().is_empty() || !seen.insert(&occurrence.occurrence_id) {
            issues.push(CompileIssue::new(
                format!("{prefix}.occurrence_id"),
                "occurrence ids must be non-empty and unique",
            ));
        }
        if occurrence.source.adapter.trim().is_empty() || occurrence.source.id.trim().is_empty() {
            issues.push(CompileIssue::new(
                format!("{prefix}.source"),
                "each occurrence needs an exact source reference",
            ));
        }
        for (name, value) in &occurrence.values {
            match dataset.fields.get(name) {
                Some(field_type) if *field_type == value.field_type() => {},
                Some(_) => issues.push(CompileIssue::new(
                    format!("{prefix}.values.{name}"),
                    "value does not match the disclosed field type",
                )),
                None => issues.push(CompileIssue::new(
                    format!("{prefix}.values.{name}"),
                    "value names a field absent from the disclosed schema",
                )),
            }
        }
        if occurrence
            .values
            .get(&definition.reading.key)
            .and_then(ProjectionValue::text)
            != Some(occurrence.occurrence_id.as_str())
        {
            issues.push(CompileIssue::new(
                format!("{prefix}.values.{}", definition.reading.key),
                "the reading key must repeat this occurrence id exactly",
            ));
        }
        for (target, field, type_name) in [
            ("encoding.label", label, "text"),
            ("encoding.x", x, "number"),
            ("encoding.y", y, "number"),
        ] {
            let Some(field) = field else {
                continue;
            };
            let value = occurrence.values.get(field);
            let valid = match type_name {
                "text" => value.and_then(ProjectionValue::text).is_some(),
                "number" => value
                    .and_then(ProjectionValue::number)
                    .is_some_and(f64::is_finite),
                _ => false,
            };
            if !valid {
                issues.push(CompileIssue::new(
                    format!("{prefix}.values.{field}"),
                    format!("{target} needs a finite {type_name} value"),
                ));
            }
        }
        if definition.arrangement.kind == SCATTER_ARRANGEMENT_ID {
            for field in [x, y].into_iter().flatten() {
                if let Some(value) = occurrence
                    .values
                    .get(field)
                    .and_then(ProjectionValue::number)
                    && (!value.is_finite()
                        || value.abs() * definition.arrangement.spacing as f64 > 1.0e9)
                {
                    issues.push(CompileIssue::new(
                        format!("{prefix}.values.{field}"),
                        "scatter coordinates must fit finite scene units",
                    ));
                }
            }
        }
    }
}

fn field_name<'a>(channel: &'a Channel, _target: &str) -> Option<&'a str> {
    match channel {
        Channel::Field(field) if !field.trim().is_empty() => Some(field),
        Channel::Field(_) | Channel::Constant(_) => None,
    }
}

fn number(occurrence: &ProjectionOccurrence, field: &str) -> Option<f64> {
    occurrence
        .values
        .get(field)
        .and_then(ProjectionValue::number)
}

fn arrangement_for(definition: &ProjectionDefinition) -> Option<ScenoArrangement> {
    let spacing = definition.arrangement.spacing as f32;
    match definition.arrangement.kind.as_str() {
        GRID_ARRANGEMENT_ID => Some(ScenoArrangement::Grid(Grid {
            origin: Vec2::ZERO,
            cell: Vec2::new(184.0, 84.0),
            columns: 8,
            gap: spacing,
        })),
        SCATTER_ARRANGEMENT_ID => Some(ScenoArrangement::Geographic(Geographic {
            origin: Vec2::ZERO,
            units_per_coordinate: spacing,
            invert_y: false,
        })),
        _ => None,
    }
}

fn placement_for(definition: &ProjectionDefinition, x: f64, y: f64) -> Option<Placement> {
    match definition.arrangement.kind.as_str() {
        GRID_ARRANGEMENT_ID => Some(Placement::Cell {
            column: x as i32,
            row: y as i32,
        }),
        SCATTER_ARRANGEMENT_ID => Some(Placement::Coordinate(Vec2::new(x as f32, y as f32))),
        _ => None,
    }
}

fn stable_generation(dataset: &ProjectionDataset) -> u64 {
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
