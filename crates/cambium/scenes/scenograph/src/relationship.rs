// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Durable relationship recipe and host-neutral edit state. Domain evidence,
//! compilation, persistence authority and widgets remain outside this module.

use crate::*;

pub const EXPLAINED_RELATIONSHIPS_FACET: &str = "explained_relationships";
pub const AUTHORED_ORDER_FACET: &str = "authored_order";
pub const OCCURRENCE_LABELS_FACET: &str = "occurrence_labels";

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RelationshipRequirements {
    pub facet: String,
    pub min_occurrences: u32,
    pub min_relationships: u32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RelationshipRecipe {
    pub definition: AuthoredProjectionDefinition,
    pub requirements: RelationshipRequirements,
    /// None includes every disclosed kind, never derives a new relationship.
    pub relationship_kind: Option<String>,
}

impl RelationshipRecipe {
    pub fn validate(&self) -> Result<(), Vec<ValidationIssue>> {
        let mut issues = match self.definition.validate() {
            Ok(()) => Vec::new(),
            Err(AuthoredDefinitionError::Invalid(issues)) => issues,
            Err(error) => vec![ValidationIssue::error("definition", &format!("{error:?}"))],
        };
        if self.requirements.facet.trim().is_empty() {
            issues.push(ValidationIssue::error(
                "requirements.facet",
                "a named semantic facet is required",
            ));
        }
        if self.requirements.min_occurrences < 2 {
            issues.push(ValidationIssue::error(
                "requirements.min_occurrences",
                "a relationship recipe requires at least two occurrences",
            ));
        }
        if self.requirements.min_relationships == 0 {
            issues.push(ValidationIssue::error(
                "requirements.min_relationships",
                "a relationship recipe requires a disclosed relationship",
            ));
        }
        if self
            .relationship_kind
            .as_ref()
            .is_some_and(|kind| kind.trim().is_empty())
        {
            issues.push(ValidationIssue::error(
                "relationship_kind",
                "a selected relationship kind cannot be empty",
            ));
        }
        if issues.is_empty() {
            Ok(())
        } else {
            Err(issues)
        }
    }

    pub fn to_json_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }
}

/// One common recipe, not a domain-specific comparison algorithm. Source
/// adapters declare authored occurrence order, labels and explained relations.
pub fn relationship_recipe(
    id: impl Into<String>,
    label: impl Into<String>,
    author: impl Into<String>,
    recipe_revision: impl Into<String>,
    sources: BTreeMap<String, ProjectionInputBinding>,
) -> RelationshipRecipe {
    let label = label.into();
    RelationshipRecipe {
        definition: AuthoredProjectionDefinition {
            version: PROJECTION_DEFINITION_VERSION, id: id.into(), label: label.clone(), sources,
            reading: Reading { kind: "nodes".into(), key: "occurrence_id".into(), value: None },
            encoding: Encoding { x: Channel::Field("order".into()), y: Channel::Constant("0".into()), color: None, label: Some(Channel::Field("label".into())) },
            arrangement: Arrangement { kind: "grid.default".into(), direction: "coordinates".into(), spacing: 16, options: BTreeMap::new() },
            interaction: Interaction { selection: SelectionMode::Single, pan: false, zoom: false },
            appearance: Appearance { realization: "canvas".into(), title: label, theme: "slate".into() },
            provenance: Provenance { author: author.into(), source_revision: Some(PublicSourceRevision::new(recipe_revision)), revision_evidence: RevisionEvidence::PublicGeneration, note: "Disclosed occurrence labels and authored order; relationships retain owner explanations and method provenance.".into() },
        },
        requirements: RelationshipRequirements { facet: EXPLAINED_RELATIONSHIPS_FACET.into(), min_occurrences: 2, min_relationships: 1 },
        relationship_kind: None,
    }
}

/// The same incomplete edit state can back native, web, or nonvisual controls.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RelationshipRecipeDraft {
    recipe: RelationshipRecipe,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RecipeEdit {
    SetLabel(String),
    SetArrangement(Arrangement),
    SetRelationshipKind(Option<String>),
    BindInput {
        name: String,
        binding: ProjectionInputBinding,
    },
    SetRequirements(RelationshipRequirements),
    SetProvenance(Provenance),
}

impl RelationshipRecipeDraft {
    pub fn new(recipe: RelationshipRecipe) -> Self {
        Self { recipe }
    }
    pub fn recipe(&self) -> &RelationshipRecipe {
        &self.recipe
    }
    pub fn apply(&mut self, edit: RecipeEdit) {
        match edit {
            RecipeEdit::SetLabel(label) => {
                self.recipe.definition.label = label.clone();
                self.recipe.definition.appearance.title = label;
            },
            RecipeEdit::SetArrangement(value) => self.recipe.definition.arrangement = value,
            RecipeEdit::SetRelationshipKind(value) => self.recipe.relationship_kind = value,
            RecipeEdit::BindInput { name, binding } => {
                self.recipe.definition.sources.insert(name, binding);
            },
            RecipeEdit::SetRequirements(value) => self.recipe.requirements = value,
            RecipeEdit::SetProvenance(value) => self.recipe.definition.provenance = value,
        }
    }
    pub fn to_recipe(&self) -> Result<RelationshipRecipe, Vec<ValidationIssue>> {
        self.recipe.validate()?;
        Ok(self.recipe.clone())
    }
}

/// Saved authored state. A host retains the corresponding bounded disclosure
/// beside this value, under its own authority, never in a free-form note.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RelationshipSnapshot {
    pub recipe: RelationshipRecipe,
    pub source_name: String,
    pub selected_occurrence: Option<String>,
    pub selected_relationship: Option<String>,
}
impl RelationshipSnapshot {
    pub fn validate(&self) -> Result<(), Vec<ValidationIssue>> {
        self.recipe.validate()?;
        if self.source_name.trim().is_empty()
            || !self
                .recipe
                .definition
                .sources
                .contains_key(&self.source_name)
        {
            return Err(vec![ValidationIssue::error(
                "source_name",
                "select a declared input binding",
            )]);
        }
        for (field, value) in [
            ("selected_occurrence", &self.selected_occurrence),
            ("selected_relationship", &self.selected_relationship),
        ] {
            if value.as_ref().is_some_and(|value| value.trim().is_empty()) {
                return Err(vec![ValidationIssue::error(
                    field,
                    "a selection identity cannot be empty",
                )]);
            }
        }
        Ok(())
    }
}
