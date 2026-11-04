// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Per-mere predicate nature, recorded as ordinary resource metadata.

use std::collections::BTreeMap;
use std::fmt;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{Author, CapturedDelta, Graph, GraphStratum, ResourceNode, built_in_predicate_stratum};
use crate::persistence::{PersistedResourceFacet, PersistedResourceRecord};
use crate::types::mint_local_statement_id;

/// Typed declaration data on the exact-IRI resource identifying a predicate.
pub const PREDICATE_DECLARATIONS_FACET: &str = "semantic.predicate-declarations/v1";

/// One carried declaration and its original author, version, application and time.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PredicateDeclaration {
    pub declaration_id: String,
    pub stratum: GraphStratum,
    pub author: Author,
    pub declared_at_ms: Option<u64>,
}

#[cfg(test)]
#[path = "predicate_declarations/tests.rs"]
mod tests;

/// All retained variants, with an explicit choice when their natures differ.
/// Every field is serialized, including an absent choice.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PredicateDeclarations {
    pub variants: Vec<PredicateDeclaration>,
    pub selected: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PredicateDeclarationError {
    Invalid {
        predicate: String,
        detail: String,
    },
    BuiltIn {
        predicate: String,
    },
    Conflict {
        predicate: String,
        declaration_ids: Vec<String>,
    },
}

impl fmt::Display for PredicateDeclarationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Invalid { predicate, detail } => write!(
                f,
                "predicate {predicate:?} has invalid declarations: {detail}"
            ),
            Self::BuiltIn { predicate } => {
                write!(f, "predicate {predicate:?} has fixed built-in placement")
            },
            Self::Conflict {
                predicate,
                declaration_ids,
            } => write!(
                f,
                "predicate {predicate:?} requires a declaration choice among {declaration_ids:?}"
            ),
        }
    }
}

impl std::error::Error for PredicateDeclarationError {}

fn invalid(predicate: &str, detail: impl Into<String>) -> PredicateDeclarationError {
    PredicateDeclarationError::Invalid {
        predicate: predicate.into(),
        detail: detail.into(),
    }
}

impl PredicateDeclarations {
    /// Exact variant identities cannot select different carried origins or natures.
    fn validate(&self, predicate: &str) -> Result<(), PredicateDeclarationError> {
        if built_in_predicate_stratum(predicate).is_some() {
            return Err(PredicateDeclarationError::BuiltIn {
                predicate: predicate.into(),
            });
        }
        let mut ids = BTreeMap::new();
        for variant in &self.variants {
            if let Some(previous) = ids.insert(variant.declaration_id.as_str(), variant) {
                return Err(invalid(
                    predicate,
                    if previous == variant {
                        format!("repeated declaration {:?}", variant.declaration_id)
                    } else {
                        format!("conflicting declaration {:?}", variant.declaration_id)
                    },
                ));
            }
        }
        if let Some(selected) = &self.selected
            && !ids.contains_key(selected.as_str())
        {
            return Err(invalid(
                predicate,
                format!("selected declaration {selected:?} is absent"),
            ));
        }
        Ok(())
    }

    /// A retained explicit selection wins; agreeing variants need no selection.
    pub fn effective_stratum(
        &self,
        predicate: &str,
    ) -> Result<GraphStratum, PredicateDeclarationError> {
        self.validate(predicate)?;
        if let Some(selected) = &self.selected {
            return Ok(self
                .variants
                .iter()
                .find(|variant| variant.declaration_id == *selected)
                .expect("validated selection")
                .stratum);
        }
        let Some(first) = self.variants.first() else {
            return Ok(GraphStratum::Resource);
        };
        if self
            .variants
            .iter()
            .all(|variant| variant.stratum == first.stratum)
        {
            return Ok(first.stratum);
        }
        let mut declaration_ids: Vec<_> = self
            .variants
            .iter()
            .map(|variant| variant.declaration_id.clone())
            .collect();
        declaration_ids.sort();
        Err(PredicateDeclarationError::Conflict {
            predicate: predicate.into(),
            declaration_ids,
        })
    }
}

/// Parse only the owned declaration namespace; unknown resource facets stay opaque.
pub fn parse_predicate_declarations(
    predicate: &str,
    value: &Value,
) -> Result<PredicateDeclarations, PredicateDeclarationError> {
    let declarations: PredicateDeclarations =
        Deserialize::deserialize(value).map_err(|error| invalid(predicate, error.to_string()))?;
    declarations.validate(predicate)?;
    Ok(declarations)
}

/// Read a declaration facet without canonicalizing the predicate's vocabulary IRI.
pub fn predicate_declarations_from_record(
    record: &PersistedResourceRecord,
) -> Result<Option<PredicateDeclarations>, PredicateDeclarationError> {
    let mut facet = record
        .facets
        .iter()
        .filter(|facet| facet.facet == PREDICATE_DECLARATIONS_FACET);
    let Some(declaration) = facet.next() else {
        return Ok(None);
    };
    if facet.next().is_some() {
        return Err(invalid(&record.canonical_iri, "repeated declaration facet"));
    }
    let value: Value = serde_json::from_str(&declaration.value_json)
        .map_err(|error| invalid(&record.canonical_iri, error.to_string()))?;
    parse_predicate_declarations(&record.canonical_iri, &value).map(Some)
}

/// Union exact origins and variants without choosing either input's selection.
/// Differing selections clear the choice; agreeing natures remain unambiguous.
pub fn merge_predicate_declarations(
    predicate: &str,
    left: &Value,
    right: &Value,
) -> Result<Value, PredicateDeclarationError> {
    let left = parse_predicate_declarations(predicate, left)?;
    let right = parse_predicate_declarations(predicate, right)?;
    let selected = (left.selected == right.selected)
        .then(|| left.selected.clone())
        .flatten();
    let mut variants = BTreeMap::new();
    for variant in left.variants.into_iter().chain(right.variants) {
        if let Some(previous) = variants.get(&variant.declaration_id)
            && previous != &variant
        {
            return Err(invalid(
                predicate,
                format!("conflicting declaration {:?}", variant.declaration_id),
            ));
        }
        variants.insert(variant.declaration_id.clone(), variant);
    }
    let merged = PredicateDeclarations {
        variants: variants.into_values().collect(),
        selected,
    };
    serde_json::to_value(merged).map_err(|error| invalid(predicate, error.to_string()))
}

impl Graph {
    /// The declarations held by this mere for one predicate.
    pub fn predicate_declarations(
        &self,
        predicate: &str,
    ) -> Result<Option<PredicateDeclarations>, PredicateDeclarationError> {
        let Some(record) = self.resource_record(ResourceNode::for_term(predicate).id()) else {
            return Ok(None);
        };
        predicate_declarations_from_record(&record)
    }

    /// Placement for a future assertion; exact historical handles retain their stores.
    pub fn effective_predicate_stratum(
        &self,
        predicate: &str,
    ) -> Result<GraphStratum, PredicateDeclarationError> {
        if let Some(stratum) = built_in_predicate_stratum(predicate) {
            return Ok(stratum);
        }
        match self.predicate_declarations(predicate)? {
            Some(declarations) => declarations.effective_stratum(predicate),
            None => Ok(GraphStratum::Resource),
        }
    }

    /// Retain an authored nature change and explicitly select it for future writes.
    pub fn declare_predicate(
        &mut self,
        predicate: &str,
        stratum: GraphStratum,
    ) -> Result<String, PredicateDeclarationError> {
        if built_in_predicate_stratum(predicate).is_some() {
            return Err(PredicateDeclarationError::BuiltIn {
                predicate: predicate.into(),
            });
        }
        let mut declarations =
            self.predicate_declarations(predicate)?
                .unwrap_or(PredicateDeclarations {
                    variants: vec![],
                    selected: None,
                });
        if let Some(selected) = &declarations.selected
            && let Some(existing) = declarations
                .variants
                .iter()
                .find(|variant| variant.declaration_id == *selected)
            && existing.stratum == stratum
            && existing.author == *self.write_author()
        {
            return Ok(existing.declaration_id.clone());
        }
        let declaration_id = mint_local_statement_id();
        declarations.variants.push(PredicateDeclaration {
            declaration_id: declaration_id.clone(),
            stratum,
            author: self.write_author().clone(),
            declared_at_ms: Some(Self::epoch_ms()),
        });
        declarations.selected = Some(declaration_id.clone());
        self.write_predicate_declarations(predicate, declarations)?;
        Ok(declaration_id)
    }

    /// Resolve a retained conflict by exact declaration id, without moving statements.
    pub fn select_predicate_declaration(
        &mut self,
        predicate: &str,
        declaration_id: &str,
    ) -> Result<bool, PredicateDeclarationError> {
        if built_in_predicate_stratum(predicate).is_some() {
            return Err(PredicateDeclarationError::BuiltIn {
                predicate: predicate.into(),
            });
        }
        let mut declarations = self
            .predicate_declarations(predicate)?
            .ok_or_else(|| invalid(predicate, "no retained declarations"))?;
        if !declarations
            .variants
            .iter()
            .any(|variant| variant.declaration_id == declaration_id)
        {
            return Err(invalid(
                predicate,
                format!("declaration {declaration_id:?} is absent"),
            ));
        }
        if declarations.selected.as_deref() == Some(declaration_id) {
            return Ok(false);
        }
        declarations.selected = Some(declaration_id.into());
        self.write_predicate_declarations(predicate, declarations)
    }

    fn write_predicate_declarations(
        &mut self,
        predicate: &str,
        mut declarations: PredicateDeclarations,
    ) -> Result<bool, PredicateDeclarationError> {
        declarations.validate(predicate)?;
        declarations
            .variants
            .sort_by(|a, b| a.declaration_id.cmp(&b.declaration_id));
        let resource_id = ResourceNode::for_term(predicate).id();
        let mut record = self
            .resource_record(resource_id)
            .unwrap_or(PersistedResourceRecord {
                canonical_iri: predicate.into(),
                facets: vec![],
            });
        record
            .facets
            .retain(|facet| facet.facet != PREDICATE_DECLARATIONS_FACET);
        record.facets.push(PersistedResourceFacet {
            facet: PREDICATE_DECLARATIONS_FACET.into(),
            value_json: serde_json::to_string(&declarations)
                .map_err(|error| invalid(predicate, error.to_string()))?,
        });
        let changed = self.set_resource_record(resource_id, Some(record));
        if changed {
            self.record_delta(&CapturedDelta::ReplaySetResourceRecordById {
                resource_id: resource_id.to_string(),
                record: self.resource_record(resource_id),
            });
        }
        Ok(changed)
    }
}
