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
mod tests {
    use std::sync::{Arc, Mutex};

    use super::*;
    use crate::graph::revert::{Part, revert_change};
    use crate::graph::{
        EdgePayload, SemanticStatement, SemanticSubKind, predicate_iri, replay_captured_deltas_onto,
    };
    use crate::types::GraphScope;
    use euclid::default::Point2D;

    const PREDICATE: &str = "https://Vocabulary.test/terms?edition=1#custom";

    fn value(id: &str, stratum: GraphStratum, author: Author) -> Value {
        serde_json::to_value(PredicateDeclarations {
            variants: vec![PredicateDeclaration {
                declaration_id: id.into(),
                stratum,
                author,
                declared_at_ms: Some(17),
            }],
            selected: Some(id.into()),
        })
        .unwrap()
    }

    fn record(predicate: &str, value: &Value) -> PersistedResourceRecord {
        PersistedResourceRecord {
            canonical_iri: predicate.into(),
            facets: vec![PersistedResourceFacet {
                facet: PREDICATE_DECLARATIONS_FACET.into(),
                value_json: value.to_string(),
            }],
        }
    }

    fn capture(graph: &mut Graph) -> Arc<Mutex<Vec<CapturedDelta>>> {
        let deltas = Arc::new(Mutex::new(vec![]));
        let recorded = Arc::clone(&deltas);
        graph.set_recorder(Some(Arc::new(move |delta| {
            recorded.lock().unwrap().push(delta.clone())
        })));
        deltas
    }

    #[test]
    fn predicate_declarations_persist_exact_origins_identity_and_replay() {
        let mut graph = Graph::new();
        let deltas = capture(&mut graph);
        let mut author = Author::engine("extractor", "v2");
        author.via = Some("origin-host".into());
        let id = graph.write_as(author.clone(), |graph| {
            graph
                .declare_predicate(PREDICATE, GraphStratum::Surface)
                .unwrap()
        });
        let declarations = graph.predicate_declarations(PREDICATE).unwrap().unwrap();
        assert_eq!(declarations.selected.as_deref(), Some(id.as_str()));
        assert_eq!(declarations.variants[0].author, author);
        assert!(declarations.variants[0].declared_at_ms.is_some());
        assert_eq!(
            graph
                .resource(ResourceNode::for_term(PREDICATE).id())
                .unwrap()
                .canonical_iri(),
            PREDICATE
        );
        assert_eq!(
            graph.node_count(),
            0,
            "a declaration needs no invented surface"
        );
        assert_eq!(
            graph.clone().predicate_declarations(PREDICATE).unwrap(),
            Some(declarations.clone())
        );
        let snapshot = graph.to_snapshot();
        let bytes = serde_json::to_vec(&snapshot).unwrap();
        let saved = serde_json::from_slice(&bytes).unwrap();
        let restored = Graph::try_from_snapshot(&saved).unwrap();
        assert_eq!(
            restored.predicate_declarations(PREDICATE).unwrap(),
            Some(declarations.clone())
        );
        let captured = deltas.lock().unwrap().clone();
        assert_eq!(captured.len(), 1);
        assert!(matches!(
            &captured[0],
            CapturedDelta::ReplaySetResourceRecordById { .. }
        ));
        let captured: Vec<CapturedDelta> =
            postcard::from_bytes(&postcard::to_allocvec(&captured).unwrap()).unwrap();
        let mut replayed = Graph::new();
        replay_captured_deltas_onto(&mut replayed, captured.iter().cloned());
        assert_eq!(
            replayed.predicate_declarations(PREDICATE).unwrap(),
            Some(declarations)
        );
        assert_eq!(
            replayed.effective_predicate_stratum(PREDICATE),
            Ok(GraphStratum::Surface)
        );
        assert_eq!(
            replayed.effective_predicate_stratum("https://unrelated.test/predicate"),
            Ok(GraphStratum::Resource)
        );
        let revision = graph.revision();
        let same_id = graph.write_as(author, |graph| {
            graph
                .declare_predicate(PREDICATE, GraphStratum::Surface)
                .unwrap()
        });
        assert_eq!(id, same_id);
        assert_eq!(graph.revision(), revision);
        assert_eq!(
            deltas.lock().unwrap().len(),
            1,
            "no-op declarations emit no capture"
        );
    }

    #[test]
    fn predicate_declarations_conflicts_require_exact_choice_with_unambiguous_controls() {
        let left = value("alice", GraphStratum::Resource, Author::person("alice"));
        let right = value("bob", GraphStratum::Surface, Author::person("bob"));
        let merged = merge_predicate_declarations(PREDICATE, &left, &right).unwrap();
        assert_eq!(
            merge_predicate_declarations(PREDICATE, &right, &left).unwrap(),
            merged
        );
        let declarations = parse_predicate_declarations(PREDICATE, &merged).unwrap();
        assert_eq!(declarations.variants.len(), 2);
        assert_eq!(declarations.selected, None);
        assert!(matches!(
            declarations.effective_stratum(PREDICATE),
            Err(PredicateDeclarationError::Conflict { .. })
        ));
        assert_eq!(
            merge_predicate_declarations(PREDICATE, &left, &left).unwrap(),
            left
        );
        let agreement = value("carol", GraphStratum::Resource, Author::person("carol"));
        let agreed = merge_predicate_declarations(PREDICATE, &left, &agreement).unwrap();
        assert_eq!(
            parse_predicate_declarations(PREDICATE, &agreed)
                .unwrap()
                .effective_stratum(PREDICATE),
            Ok(GraphStratum::Resource)
        );
        let mut graph = Graph::new();
        let resource_id = ResourceNode::for_term(PREDICATE).id();
        assert!(graph.set_resource_record(resource_id, Some(record(PREDICATE, &merged))));
        assert!(matches!(
            graph.effective_predicate_stratum(PREDICATE),
            Err(PredicateDeclarationError::Conflict { .. })
        ));
        assert_eq!(
            graph.effective_predicate_stratum("https://unrelated.test/predicate"),
            Ok(GraphStratum::Resource)
        );
        let deltas = capture(&mut graph);
        let before = graph.predicate_declarations(PREDICATE).unwrap().unwrap();
        assert!(
            graph
                .select_predicate_declaration(PREDICATE, "absent")
                .is_err()
        );
        assert_eq!(
            graph.predicate_declarations(PREDICATE).unwrap().unwrap(),
            before
        );
        assert!(deltas.lock().unwrap().is_empty());
        assert!(
            graph
                .select_predicate_declaration(PREDICATE, "bob")
                .unwrap()
        );
        assert_eq!(
            graph.effective_predicate_stratum(PREDICATE),
            Ok(GraphStratum::Surface)
        );
        assert_eq!(
            graph
                .predicate_declarations(PREDICATE)
                .unwrap()
                .unwrap()
                .variants,
            before.variants
        );
        assert!(
            !graph
                .select_predicate_declaration(PREDICATE, "bob")
                .unwrap()
        );
        assert_eq!(deltas.lock().unwrap().len(), 1);
        let mut replayed = Graph::new();
        assert!(replayed.set_resource_record(resource_id, Some(record(PREDICATE, &merged))));
        replay_captured_deltas_onto(&mut replayed, deltas.lock().unwrap().iter().cloned());
        assert_eq!(
            replayed.effective_predicate_stratum(PREDICATE),
            Ok(GraphStratum::Surface)
        );
    }

    #[test]
    fn predicate_declarations_reused_origin_and_builtin_edits_are_atomic_errors() {
        let left = value("same-id", GraphStratum::Resource, Author::person("alice"));
        let different_origin = value("same-id", GraphStratum::Resource, Author::person("bob"));
        assert!(merge_predicate_declarations(PREDICATE, &left, &different_origin).is_err());
        let distinct = value(
            "different-id",
            GraphStratum::Resource,
            Author::person("bob"),
        );
        assert!(merge_predicate_declarations(PREDICATE, &left, &distinct).is_ok());
        assert_eq!(
            left,
            value("same-id", GraphStratum::Resource, Author::person("alice"))
        );
        let mut graph = Graph::new();
        let deltas = capture(&mut graph);
        for kind in [SemanticSubKind::UserGrouped, SemanticSubKind::Cites] {
            let iri = predicate_iri(kind);
            let before = graph.effective_predicate_stratum(iri).unwrap();
            assert!(matches!(
                graph.declare_predicate(iri, GraphStratum::Surface),
                Err(PredicateDeclarationError::BuiltIn { .. })
            ));
            assert!(graph.select_predicate_declaration(iri, "any").is_err());
            assert_eq!(graph.effective_predicate_stratum(iri), Ok(before));
        }
        assert_eq!(graph.resource_nodes().count(), 0);
        assert_eq!(graph.revision(), 0);
        assert!(deltas.lock().unwrap().is_empty());
        assert!(
            graph
                .declare_predicate(PREDICATE, GraphStratum::Surface)
                .is_ok()
        );
        assert_eq!(graph.resource_nodes().count(), 1);
        assert_eq!(deltas.lock().unwrap().len(), 1);
    }

    #[test]
    fn predicate_declarations_checked_load_and_raw_record_reject_invalid_typed_data() {
        let mut graph = Graph::new();
        let resource_id = ResourceNode::for_term(PREDICATE).id();
        let valid = value("valid", GraphStratum::Surface, Author::person("alice"));
        assert!(graph.set_resource_record(resource_id, Some(record(PREDICATE, &valid))));
        let snapshot = graph.to_snapshot();
        let mut bad = valid.clone();
        bad["selected"] = Value::String("absent".into());
        assert!(!graph.set_resource_record(resource_id, Some(record(PREDICATE, &bad))));
        assert_eq!(graph.to_snapshot().resources, snapshot.resources);
        let mut invalid = snapshot.clone();
        invalid.resources[0] = record(PREDICATE, &bad);
        let bytes = serde_json::to_vec(&invalid).unwrap();
        let error = Graph::try_from_snapshot(&invalid)
            .err()
            .expect("invalid typed declaration")
            .to_string();
        assert!(
            error.contains(PREDICATE_DECLARATIONS_FACET) && error.contains("absent"),
            "{error}"
        );
        assert_eq!(
            serde_json::to_vec(&invalid).unwrap(),
            bytes,
            "failed load retains source bytes"
        );
        assert!(Graph::try_from_snapshot(&snapshot).is_ok());
        let mut opaque = record(PREDICATE, &bad);
        opaque.facets[0].facet = "foreign.uninterpreted".into();
        invalid.resources[0] = opaque.clone();
        let restored = Graph::try_from_snapshot(&invalid).unwrap();
        assert_eq!(restored.resource_record(resource_id).unwrap(), opaque);
        assert_eq!(
            restored.effective_predicate_stratum(PREDICATE),
            Ok(GraphStratum::Resource)
        );
        let legacy = Graph::new().to_snapshot();
        assert_eq!(
            Graph::try_from_snapshot(&legacy)
                .unwrap()
                .effective_predicate_stratum(PREDICATE),
            Ok(GraphStratum::Resource)
        );
    }

    #[test]
    fn predicate_declarations_conditional_undo_keeps_later_facets_and_declarations() {
        let mut graph = Graph::new();
        let resource_id = ResourceNode::for_term(PREDICATE).id();
        assert!(graph.set_resource_record(
            resource_id,
            Some(PersistedResourceRecord {
                canonical_iri: PREDICATE.into(),
                facets: vec![PersistedResourceFacet {
                    facet: "foreign.before".into(),
                    value_json: "1".into()
                }]
            })
        ));
        let before = graph.clone();
        let deltas = capture(&mut graph);
        graph
            .declare_predicate(PREDICATE, GraphStratum::Surface)
            .unwrap();
        let change = deltas.lock().unwrap().clone();
        let after = graph.clone();
        let mut updated = graph.resource_record(resource_id).unwrap();
        updated.facets.push(PersistedResourceFacet {
            facet: "foreign.later".into(),
            value_json: "{\"nested\":[2]}".into(),
        });
        assert!(graph.set_resource_record(resource_id, Some(updated)));
        let undo = revert_change(&change, &before, &after, &graph);
        assert!(undo.kept.is_empty());
        replay_captured_deltas_onto(&mut graph, undo.edits.iter().cloned());
        assert_eq!(graph.predicate_declarations(PREDICATE).unwrap(), None);
        assert_eq!(graph.resource_record(resource_id).unwrap().facets.len(), 2);
        assert_eq!(
            graph.effective_predicate_stratum(PREDICATE),
            Ok(GraphStratum::Resource)
        );
        let mut later = after.clone();
        later.write_as(Author::person("later-author"), |graph| {
            graph
                .declare_predicate(PREDICATE, GraphStratum::Resource)
                .unwrap()
        });
        let current = later.predicate_declarations(PREDICATE).unwrap();
        let kept = revert_change(&change, &before, &after, &later);
        assert!(kept.kept.contains(&Part::ResourceFacet(
            resource_id,
            PREDICATE_DECLARATIONS_FACET.into()
        )));
        replay_captured_deltas_onto(&mut later, kept.edits.iter().cloned());
        assert_eq!(later.predicate_declarations(PREDICATE).unwrap(), current);
        assert_eq!(
            later.effective_predicate_stratum(PREDICATE),
            Ok(GraphStratum::Resource)
        );
    }

    #[test]
    fn predicate_declarations_nature_changes_leave_held_claims_in_both_strata_exact() {
        let mut graph = Graph::new();
        let a = graph.add_node("https://surface.test/a".into(), Point2D::zero());
        let b = graph.add_node("https://surface.test/b".into(), Point2D::zero());
        let held = |id: &str| {
            let mut payload = EdgePayload::new();
            payload.push_persisted_semantic_statement(SemanticStatement {
                statement_id: id.into(),
                predicate: PREDICATE.into(),
                recognized_sub_kind: None,
                label: Some("held claim".into()),
                graph_scope: GraphScope::User,
                provenance_iri: Some("https://original-author.test/".into()),
                asserted_at_ms: Some(19),
            });
            payload
        };
        graph.inner.connect(a, b, held("surface-claim"));
        let ra = ResourceNode::for_term("https://resource.test/a");
        let rb = ResourceNode::for_term("https://resource.test/b");
        for resource in [&ra, &rb] {
            assert!(graph.set_resource_record(
                resource.id(),
                Some(PersistedResourceRecord {
                    canonical_iri: resource.canonical_iri().into(),
                    facets: vec![]
                })
            ));
        }
        let edge = super::super::snapshot::persisted_edge_for_ids(
            ra.id(),
            rb.id(),
            &held("resource-claim"),
        );
        assert!(graph.set_resource_edges_between(ra.id(), rb.id(), &[edge]));
        let before = graph.to_snapshot();
        let deltas = capture(&mut graph);
        let first = graph
            .declare_predicate(PREDICATE, GraphStratum::Surface)
            .unwrap();
        let second = graph
            .declare_predicate(PREDICATE, GraphStratum::Resource)
            .unwrap();
        assert_ne!(first, second);
        assert_eq!(
            graph.effective_predicate_stratum(PREDICATE),
            Ok(GraphStratum::Resource)
        );
        assert_eq!(
            graph
                .predicate_declarations(PREDICATE)
                .unwrap()
                .unwrap()
                .variants
                .len(),
            2
        );
        assert_eq!(graph.to_snapshot().edges, before.edges);
        assert_eq!(graph.to_snapshot().resource_edges, before.resource_edges);
        assert_eq!(deltas.lock().unwrap().len(), 2);
        assert!(
            deltas
                .lock()
                .unwrap()
                .iter()
                .all(|delta| matches!(delta, CapturedDelta::ReplaySetResourceRecordById { .. }))
        );
        let mut replayed = Graph::try_from_snapshot(&before).unwrap();
        let restored_before = replayed.to_snapshot();
        assert!(
            restored_before
                .edges
                .iter()
                .any(
                    |edge| edge.semantic.as_ref().is_some_and(|semantic| semantic
                        .statements
                        .iter()
                        .any(|statement| statement.statement_id == "surface-claim"))
                )
        );
        assert!(
            restored_before
                .edges
                .iter()
                .any(|edge| edge.containment.is_some()),
            "restored aggregate control is present"
        );
        replay_captured_deltas_onto(&mut replayed, deltas.lock().unwrap().iter().cloned());
        assert_eq!(replayed.to_snapshot().edges, restored_before.edges);
        assert_eq!(
            replayed.to_snapshot().resource_edges,
            restored_before.resource_edges
        );
        assert_eq!(
            replayed.predicate_declarations(PREDICATE).unwrap(),
            graph.predicate_declarations(PREDICATE).unwrap()
        );
    }
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
