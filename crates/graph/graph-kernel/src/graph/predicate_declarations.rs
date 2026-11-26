// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0
//! Per-mere predicate nature, recorded as ordinary resource metadata.

use super::{CapturedDelta, Graph, GraphStratum, ResourceNode, built_in_predicate_stratum};
use crate::persistence::{PersistedResourceFacet, PersistedResourceRecord};
use crate::types::mint_local_statement_id;
pub use mere::model::predicate_declarations::{
    PREDICATE_DECLARATIONS_FACET, PredicateDeclaration, PredicateDeclarationError,
    PredicateDeclarations, merge_predicate_declarations, parse_predicate_declarations,
    predicate_declarations_from_record,
};

pub(crate) use mere::model::predicate_declarations::invalid;

#[cfg(test)]
#[path = "predicate_declarations/tests.rs"]
mod tests;

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
