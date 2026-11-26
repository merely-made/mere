// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Per-mere predicate nature, recorded as ordinary resource metadata: its
//! record rules. Moved from the kernel (engine split D243, D213).
#![allow(clippy::wildcard_imports)]

use super::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::fmt;

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

pub fn invalid(predicate: &str, detail: impl Into<String>) -> PredicateDeclarationError {
    PredicateDeclarationError::Invalid {
        predicate: predicate.into(),
        detail: detail.into(),
    }
}

impl PredicateDeclarations {
    /// Exact variant identities cannot select different carried origins or natures.
    pub fn validate(&self, predicate: &str) -> Result<(), PredicateDeclarationError> {
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
