// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Placement of new assertions and exact access to already held handles.

use petgraph::visit::{EdgeRef, IntoEdgeReferences};
use uuid::Uuid;

use super::{
    CapturedDelta, EdgePayload, Graph, GraphStratum, NodeKey, PredicateDeclarationError,
    RelationKey, ResourceEdgeKey, ResourceNode, SemanticStatement, SemanticStatementSpec,
    StatementAssert,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StatementWriteError {
    MissingEndpoint,
    SurfaceContextRequired,
    Declaration(PredicateDeclarationError),
    HandleCollision(String),
    AmbiguousAssertion {
        predicate: String,
        statement_ids: Vec<String>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::{
        Author, PredicateDeclarations, predicate_declarations::PREDICATE_DECLARATIONS_FACET,
    };
    use crate::persistence::{PersistedResourceFacet, PersistedResourceRecord};

    fn spec(predicate: &str, source: &str) -> SemanticStatementSpec {
        SemanticStatementSpec {
            predicate: predicate.into(),
            provenance_iri: Some(source.into()),
            ..Default::default()
        }
    }

    #[test]
    fn nature_changes_keep_held_handles_and_route_only_new_assertions() {
        let mut graph = Graph::new();
        let a = graph.add_node_with_id(
            Uuid::from_u128(100),
            "https://example.org/a".into(),
            Default::default(),
        );
        let b = graph.add_node_with_id(
            Uuid::from_u128(101),
            "https://example.org/b".into(),
            Default::default(),
        );
        let predicate = "https://example.org/predicate#rel";
        let (resource_key, first) = graph
            .try_assert_semantic_statement(a, b, spec(predicate, "urn:author:first"))
            .unwrap();
        assert!(matches!(resource_key, RelationKey::Resource(_)));
        graph
            .declare_predicate(predicate, GraphStratum::Surface)
            .unwrap();
        let mut update = spec(predicate, "urn:author:first");
        update.label = Some("updated".into());
        let (held_key, updated) = graph.try_assert_semantic_statement(a, b, update).unwrap();
        assert_eq!(held_key, resource_key);
        assert_eq!(updated.statement_id, first.statement_id);
        let (surface_key, second) = graph
            .try_assert_semantic_statement(a, b, spec(predicate, "urn:author:second"))
            .unwrap();
        assert!(matches!(surface_key, RelationKey::Surface(_)));
        graph
            .declare_predicate(predicate, GraphStratum::Resource)
            .unwrap();
        let (legacy_key, legacy) = graph
            .try_assert_semantic_statement_by_resource_ids(
                graph.shown_resource_id(a).unwrap(),
                graph.shown_resource_id(b).unwrap(),
                spec(predicate, "urn:author:second"),
            )
            .unwrap();
        assert_eq!(legacy_key, surface_key);
        assert_eq!(legacy.statement_id, second.statement_id);

        assert_eq!(
            graph
                .find_semantic_statement(&first.statement_id)
                .unwrap()
                .0,
            resource_key
        );
        graph.update_node_url(a, "https://example.org/later".into());
        graph.refresh_surface_resource(a);
        assert!(graph.retract_assertion(&first.statement_id));
        assert!(graph.find_semantic_statement(&first.statement_id).is_none());
        assert!(
            graph
                .find_semantic_statement(&second.statement_id)
                .is_some()
        );
        assert!(!graph.retract_assertion("absent"));
        assert!(graph.retract_assertion(&second.statement_id));
    }

    #[test]
    fn conflict_blocks_new_writes_atomically_with_existing_and_unaffected_controls() {
        let mut graph = Graph::new();
        let a = graph.add_node_with_id(
            Uuid::from_u128(102),
            "https://example.org/a".into(),
            Default::default(),
        );
        let b = graph.add_node_with_id(
            Uuid::from_u128(103),
            "https://example.org/b".into(),
            Default::default(),
        );
        let predicate = "urn:predicate:conflicted";
        let (_, held) = graph
            .try_assert_semantic_statement(a, b, spec(predicate, "urn:author:held"))
            .unwrap();
        graph
            .declare_predicate(predicate, GraphStratum::Resource)
            .unwrap();
        graph
            .write_as(Author::engine("other", "2"), |g| {
                g.declare_predicate(predicate, GraphStratum::Surface)
            })
            .unwrap();
        let mut declarations = graph.predicate_declarations(predicate).unwrap().unwrap();
        declarations.selected = None;
        let selected = declarations.variants[0].declaration_id.clone();
        let record = PersistedResourceRecord {
            canonical_iri: predicate.into(),
            facets: vec![PersistedResourceFacet {
                facet: PREDICATE_DECLARATIONS_FACET.into(),
                value_json: serde_json::to_string(&declarations).unwrap(),
            }],
        };
        assert!(graph.set_resource_record(ResourceNode::for_term(predicate).id(), Some(record)));
        let before = serde_json::to_value(graph.to_snapshot()).unwrap();
        assert!(matches!(
            graph.try_assert_semantic_statement(a, b, spec(predicate, "urn:author:new")),
            Err(StatementWriteError::Declaration(
                PredicateDeclarationError::Conflict { .. }
            ))
        ));
        assert_eq!(serde_json::to_value(graph.to_snapshot()).unwrap(), before);
        assert!(matches!(
            graph.try_assert_semantic_statement_by_resource_ids(
                graph.shown_resource_id(a).unwrap(),
                graph.shown_resource_id(b).unwrap(),
                spec(predicate, "urn:author:new")
            ),
            Err(StatementWriteError::Declaration(
                PredicateDeclarationError::Conflict { .. }
            ))
        ));
        assert_eq!(serde_json::to_value(graph.to_snapshot()).unwrap(), before);
        let (_, existing) = graph
            .try_assert_semantic_statement(a, b, spec(predicate, "urn:author:held"))
            .unwrap();
        assert_eq!(existing.statement_id, held.statement_id);
        assert!(!existing.changed);
        assert!(
            graph
                .try_assert_semantic_statement(
                    a,
                    b,
                    spec("urn:predicate:unaffected", "urn:author:new")
                )
                .is_ok()
        );
        graph
            .select_predicate_declaration(predicate, &selected)
            .unwrap();
        assert!(
            graph
                .try_assert_semantic_statement(a, b, spec(predicate, "urn:author:new"))
                .is_ok()
        );
        assert!(graph.retract_assertion(&held.statement_id));
        // The conflict's full origin records survive every assertion operation.
        let carried: PredicateDeclarations =
            graph.predicate_declarations(predicate).unwrap().unwrap();
        assert_eq!(carried.variants, declarations.variants);
    }

    #[test]
    fn caller_selected_handle_cannot_change_claim_with_distinct_handle_control() {
        let mut graph = Graph::new();
        let a = graph.add_node_with_id(
            Uuid::from_u128(104),
            "https://example.org/a".into(),
            Default::default(),
        );
        let b = graph.add_node_with_id(
            Uuid::from_u128(105),
            "https://example.org/b".into(),
            Default::default(),
        );
        let (_, held) = graph
            .try_assert_semantic_statement(a, b, spec("urn:predicate:original", "urn:author:held"))
            .unwrap();
        let mut other = SemanticStatement::new(
            "urn:predicate:other".into(),
            None,
            None,
            Default::default(),
            Some("urn:author:held".into()),
            None,
        );
        let distinct = other.statement_id.clone();
        other.statement_id = held.statement_id.clone();
        let before = serde_json::to_value(graph.to_snapshot()).unwrap();
        assert_eq!(
            graph.try_assert_persisted_semantic_statement(a, b, other.clone()),
            Err(StatementWriteError::HandleCollision(
                held.statement_id.clone()
            ))
        );
        assert_eq!(serde_json::to_value(graph.to_snapshot()).unwrap(), before);
        other.statement_id = distinct.clone();
        assert!(
            graph
                .try_assert_persisted_semantic_statement(a, b, other)
                .is_ok()
        );
        assert!(graph.find_semantic_statement(&distinct).is_some());
        assert!(graph.find_semantic_statement(&held.statement_id).is_some());
    }
}

#[cfg(test)]
mod migrated_handle_tests {
    use super::*;

    #[test]
    fn ambiguous_content_requires_exact_id_and_duplicate_copies_update_together() {
        let mut graph = Graph::new();
        let a = graph.add_node_with_id(
            Uuid::from_u128(106),
            "https://example.org/a".into(),
            Default::default(),
        );
        let b = graph.add_node_with_id(
            Uuid::from_u128(107),
            "https://example.org/b".into(),
            Default::default(),
        );
        let spec = SemanticStatementSpec {
            predicate: "urn:predicate:ambiguous".into(),
            provenance_iri: Some("urn:author:held".into()),
            ..Default::default()
        };
        let (_, first) = graph
            .try_assert_semantic_statement(a, b, spec.clone())
            .unwrap();
        let pair = (
            graph.shown_resource_id(a).unwrap(),
            graph.shown_resource_id(b).unwrap(),
        );
        let mut edges = graph.persisted_resource_edges_between(pair.0, pair.1);
        let mut other = edges[0].semantic.as_ref().unwrap().statements[0].clone();
        other.statement_id = "second-exact-handle".into();
        edges[0]
            .semantic
            .as_mut()
            .unwrap()
            .statements
            .push(other.clone());
        edges.push(edges[0].clone());
        assert!(graph.set_resource_edges_between(pair.0, pair.1, &edges));
        assert!(Graph::try_from_snapshot(&graph.to_snapshot()).is_ok());
        let before = serde_json::to_value(graph.to_snapshot()).unwrap();
        assert!(
            matches!(graph.try_assert_semantic_statement(a, b, spec.clone()), Err(StatementWriteError::AmbiguousAssertion { statement_ids, .. }) if statement_ids.len() == 2)
        );
        assert_eq!(serde_json::to_value(graph.to_snapshot()).unwrap(), before);
        assert!(
            matches!(graph.try_assert_semantic_statement_by_resource_ids(pair.0, pair.1, spec.clone()),
            Err(StatementWriteError::AmbiguousAssertion { statement_ids, .. }) if statement_ids.len() == 2)
        );
        assert_eq!(serde_json::to_value(graph.to_snapshot()).unwrap(), before);
        let mut precise = graph
            .find_semantic_statement(&other.statement_id)
            .unwrap()
            .1
            .clone();
        precise.label = Some("precise edit".into());
        precise.asserted_at_ms = Some(99);
        assert!(
            graph
                .try_assert_persisted_semantic_statement(a, b, precise)
                .is_ok()
        );
        assert_eq!(
            graph
                .find_semantic_statement(&first.statement_id)
                .unwrap()
                .1
                .label,
            None
        );
        let copies: Vec<_> = graph
            .resource_relations()
            .flat_map(|(_, _, _, payload)| payload.semantic_statements().iter())
            .filter(|statement| statement.statement_id == other.statement_id)
            .collect();
        assert_eq!(copies.len(), 2);
        assert!(copies.iter().all(
            |statement| statement.label.as_deref() == Some("precise edit")
                && statement.asserted_at_ms == Some(99)
        ));
        assert!(Graph::try_from_snapshot(&graph.to_snapshot()).is_ok());
        assert!(!graph.update_assertion_metadata(
            &other.statement_id,
            Some("precise edit".into()),
            Some(99)
        ));
        assert!(!graph.update_assertion_metadata("absent", None, None));
        assert!(graph.retract_assertion(&first.statement_id));
        assert!(graph.find_semantic_statement(&first.statement_id).is_none());
        assert!(graph.find_semantic_statement(&other.statement_id).is_some());
        assert!(Graph::try_from_snapshot(&graph.to_snapshot()).is_ok());
        assert!(
            graph.try_assert_semantic_statement(a, b, spec).is_ok(),
            "the remaining content key is unambiguous"
        );
        let copies: Vec<_> = graph
            .resource_relations()
            .flat_map(|(_, _, _, payload)| payload.semantic_statements().iter())
            .filter(|statement| statement.statement_id == other.statement_id)
            .collect();
        assert_eq!(copies.len(), 2);
        assert!(
            copies
                .iter()
                .all(|statement| statement.label.is_none() && statement.asserted_at_ms.is_none())
        );
        assert!(Graph::try_from_snapshot(&graph.to_snapshot()).is_ok());
    }
}

impl std::fmt::Display for StatementWriteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingEndpoint => f.write_str("assertion endpoint is absent"),
            Self::SurfaceContextRequired => f.write_str("the predicate requires Surface endpoints"),
            Self::Declaration(error) => error.fmt(f),
            Self::HandleCollision(id) => {
                write!(f, "assertion handle {id:?} belongs to another claim")
            },
            Self::AmbiguousAssertion {
                predicate,
                statement_ids,
            } => write!(
                f,
                "predicate {predicate:?} requires an exact assertion handle among {statement_ids:?}"
            ),
        }
    }
}

impl std::error::Error for StatementWriteError {}

fn matches_spec(statement: &SemanticStatement, spec: &SemanticStatementSpec) -> bool {
    statement.predicate == spec.predicate
        && statement.recognized_sub_kind == spec.recognized_sub_kind
        && statement.graph_scope == spec.graph_scope
        && statement.provenance_iri == spec.provenance_iri
}

impl Graph {
    pub(crate) fn held_statement_by_resource_ids(
        &self,
        from: Uuid,
        to: Uuid,
        spec: &SemanticStatementSpec,
    ) -> Result<Option<(RelationKey, String)>, StatementWriteError> {
        let mut matches = std::collections::BTreeMap::new();
        for (key, payload) in self
            .resource_relations()
            .filter(|(_, source, target, _)| *source == from && *target == to)
            .map(|(key, _, _, payload)| (RelationKey::Resource(key), payload))
            .chain(
                self.inner
                    .inner()
                    .edge_references()
                    .filter(|edge| {
                        self.shown_resource_id(edge.source()) == Some(from)
                            && self.shown_resource_id(edge.target()) == Some(to)
                    })
                    .map(|edge| (RelationKey::Surface(edge.id()), edge.weight())),
            )
        {
            for statement in payload
                .semantic_statements()
                .iter()
                .filter(|statement| matches_spec(statement, spec))
            {
                matches.insert(statement.statement_id.clone(), key);
            }
        }
        if matches.len() > 1 {
            return Err(StatementWriteError::AmbiguousAssertion {
                predicate: spec.predicate.clone(),
                statement_ids: matches.into_keys().collect(),
            });
        }
        Ok(matches.into_iter().next().map(|(id, key)| (key, id)))
    }

    /// Assert over held Resources without creating or selecting a Surface.
    /// Held handles keep their recorded placement, even after a nature change.
    pub fn try_assert_semantic_statement_by_resource_ids(
        &mut self,
        from: Uuid,
        to: Uuid,
        mut spec: SemanticStatementSpec,
    ) -> Result<(RelationKey, StatementAssert), StatementWriteError> {
        if self.resource(from).is_none() || self.resource(to).is_none() {
            return Err(StatementWriteError::MissingEndpoint);
        }
        if spec.provenance_iri.is_none() {
            spec.provenance_iri = Some(self.write_author().asserter_iri());
        }
        if let Some((key, id)) = self.held_statement_by_resource_ids(from, to, &spec)? {
            let changed = self.update_assertion_metadata(&id, spec.label, spec.asserted_at_ms);
            return Ok((
                key,
                StatementAssert {
                    statement_id: id,
                    changed,
                },
            ));
        }
        if self
            .effective_predicate_stratum(&spec.predicate)
            .map_err(StatementWriteError::Declaration)?
            != GraphStratum::Resource
        {
            return Err(StatementWriteError::SurfaceContextRequired);
        }
        let key = match self.find_resource_edge_key(from, to) {
            Some(key) => key,
            None => ResourceEdgeKey::from_raw(self.resources.connect(
                self.resources.key_of(&from).expect("held source"),
                self.resources.key_of(&to).expect("held target"),
                EdgePayload::new(),
            )),
        };
        let key = RelationKey::Resource(key);
        let asserted = self
            .get_relation_mut(key)
            .expect("resource bucket")
            .assert_semantic_statement(spec);
        if asserted.changed {
            self.bump_revision();
            self.capture_statement_bucket(key);
        }
        Ok((key, asserted))
    }

    pub(crate) fn literal_statement_exists(&self, statement_id: &str) -> bool {
        self.resource_nodes().any(|resource| {
            self.resource_properties(resource.id())
                .iter()
                .any(|property| property.statement_id == statement_id)
        }) || self.nodes().any(|(key, _)| {
            self.legacy_node_properties(key)
                .into_iter()
                .flatten()
                .any(|property| property.statement_id == statement_id)
        })
    }

    /// Locate a held handle in its recorded store, independent of current nature.
    pub fn find_semantic_statement(
        &self,
        statement_id: &str,
    ) -> Option<(RelationKey, &SemanticStatement)> {
        self.inner
            .inner()
            .edge_references()
            .map(|edge| (RelationKey::Surface(edge.id()), edge.weight()))
            .chain(
                self.resource_relations()
                    .map(|(key, _, _, payload)| (RelationKey::Resource(key), payload)),
            )
            .find_map(|(key, payload)| {
                payload
                    .semantic_statements()
                    .iter()
                    .find(|statement| statement.statement_id == statement_id)
                    .map(|statement| (key, statement))
            })
    }

    fn prospective_resource_pair(&self, from: NodeKey, to: NodeKey) -> Option<(Uuid, Uuid)> {
        let identity = |key| {
            self.shown_resource_id(key).or_else(|| {
                self.get_node(key)
                    .map(|node| ResourceNode::new(node.url()).id())
            })
        };
        Some((identity(from)?, identity(to)?))
    }

    fn held_statement_on_pair(
        &self,
        from: NodeKey,
        to: NodeKey,
        spec: &SemanticStatementSpec,
    ) -> Result<Option<RelationKey>, StatementWriteError> {
        let resource_pair = self.prospective_resource_pair(from, to);
        let mut matches = std::collections::BTreeMap::new();
        for (key, payload) in self
            .inner
            .inner()
            .edges_connecting(from, to)
            .map(|edge| (RelationKey::Surface(edge.id()), edge.weight()))
            .chain(
                self.resource_relations()
                    .filter(move |(_, source, target, _)| resource_pair == Some((*source, *target)))
                    .map(|(key, _, _, payload)| (RelationKey::Resource(key), payload)),
            )
        {
            for statement in payload
                .semantic_statements()
                .iter()
                .filter(|statement| matches_spec(statement, spec))
            {
                matches.insert(statement.statement_id.clone(), key);
            }
        }
        if matches.len() > 1 {
            return Err(StatementWriteError::AmbiguousAssertion {
                predicate: spec.predicate.clone(),
                statement_ids: matches.into_keys().collect(),
            });
        }
        Ok(matches.into_values().next())
    }

    fn new_statement_bucket(
        &mut self,
        from: NodeKey,
        to: NodeKey,
        predicate: &str,
    ) -> Result<RelationKey, StatementWriteError> {
        if self.get_node(from).is_none() || self.get_node(to).is_none() {
            return Err(StatementWriteError::MissingEndpoint);
        }
        // Resolve conflicts before creating records or changing any association.
        let stratum = self
            .effective_predicate_stratum(predicate)
            .map_err(StatementWriteError::Declaration)?;
        self.relation_bucket(from, to, stratum)
    }

    pub(crate) fn relation_bucket(
        &mut self,
        from: NodeKey,
        to: NodeKey,
        stratum: GraphStratum,
    ) -> Result<RelationKey, StatementWriteError> {
        if self.get_node(from).is_none() || self.get_node(to).is_none() {
            return Err(StatementWriteError::MissingEndpoint);
        }
        match stratum {
            GraphStratum::Surface => Ok(RelationKey::Surface(
                self.find_edge_key(from, to)
                    .unwrap_or_else(|| self.inner.connect(from, to, EdgePayload::new())),
            )),
            GraphStratum::Resource => {
                let source_id = self
                    .ensure_surface_resource(from)
                    .ok_or(StatementWriteError::MissingEndpoint)?;
                let target_id = self
                    .ensure_surface_resource(to)
                    .ok_or(StatementWriteError::MissingEndpoint)?;
                let key = match self.find_resource_edge_key(source_id, target_id) {
                    Some(key) => key,
                    None => {
                        let source = self.resources.key_of(&source_id).expect("bound resource");
                        let target = self.resources.key_of(&target_id).expect("bound resource");
                        ResourceEdgeKey::from_raw(self.resources.connect(
                            source,
                            target,
                            EdgePayload::new(),
                        ))
                    },
                };
                Ok(RelationKey::Resource(key))
            },
        }
    }

    /// New assertions follow the stored declaration. Reassertions keep their handle's store.
    /// A conflict or missing endpoint returns an error without partial writes.
    pub fn try_assert_semantic_statement(
        &mut self,
        from: NodeKey,
        to: NodeKey,
        mut spec: SemanticStatementSpec,
    ) -> Result<(RelationKey, StatementAssert), StatementWriteError> {
        if self.get_node(from).is_none() || self.get_node(to).is_none() {
            return Err(StatementWriteError::MissingEndpoint);
        }
        if spec.provenance_iri.is_none() {
            spec.provenance_iri = Some(self.write_author().asserter_iri());
        }
        if let Some(key) = self.held_statement_on_pair(from, to, &spec)? {
            let statement_id = self
                .get_relation(key)
                .expect("held bucket")
                .semantic_statements()
                .iter()
                .find(|statement| matches_spec(statement, &spec))
                .expect("held statement")
                .statement_id
                .clone();
            let changed =
                self.update_assertion_metadata(&statement_id, spec.label, spec.asserted_at_ms);
            return Ok((
                key,
                StatementAssert {
                    statement_id,
                    changed,
                },
            ));
        }
        let key = self.new_statement_bucket(from, to, &spec.predicate)?;
        let outcome = self
            .get_relation_mut(key)
            .expect("assertion bucket")
            .assert_semantic_statement(spec);
        if outcome.changed {
            self.bump_revision();
            self.capture_statement_bucket(key);
        }
        Ok((key, outcome))
    }

    /// Carried handles may update their claim, but cannot select a different claim or store.
    pub fn try_assert_persisted_semantic_statement(
        &mut self,
        from: NodeKey,
        to: NodeKey,
        mut statement: SemanticStatement,
    ) -> Result<RelationKey, StatementWriteError> {
        if self.get_node(from).is_none() || self.get_node(to).is_none() {
            return Err(StatementWriteError::MissingEndpoint);
        }
        if self.literal_statement_exists(&statement.statement_id) {
            return Err(StatementWriteError::HandleCollision(statement.statement_id));
        }
        if statement.provenance_iri.is_none() {
            statement.provenance_iri = Some(self.write_author().asserter_iri());
        }
        let spec = SemanticStatementSpec {
            predicate: statement.predicate.clone(),
            recognized_sub_kind: statement.recognized_sub_kind,
            graph_scope: statement.graph_scope.clone(),
            provenance_iri: statement.provenance_iri.clone(),
            label: statement.label.clone(),
            asserted_at_ms: statement.asserted_at_ms,
        };
        if let Some((owner, existing)) = self.find_semantic_statement(&statement.statement_id) {
            let pair_matches = match owner {
                RelationKey::Surface(key) => {
                    self.inner.inner().edge_endpoints(key) == Some((from, to))
                },
                RelationKey::Resource(key) => {
                    let actual = self
                        .resources
                        .inner()
                        .edge_endpoints(key.raw())
                        .map(|(a, b)| {
                            (
                                self.resources.node(a).unwrap().id(),
                                self.resources.node(b).unwrap().id(),
                            )
                        });
                    actual == self.prospective_resource_pair(from, to)
                },
            };
            if !pair_matches || !matches_spec(existing, &spec) {
                return Err(StatementWriteError::HandleCollision(statement.statement_id));
            }
            let statement_id = statement.statement_id.clone();
            self.update_assertion_metadata(
                &statement_id,
                statement.label,
                statement.asserted_at_ms,
            );
            return Ok(owner);
        }
        let held = self.held_statement_on_pair(from, to, &spec)?;
        let key = match held {
            Some(key) => key,
            None => self.new_statement_bucket(from, to, &statement.predicate)?,
        };
        if self
            .get_relation_mut(key)
            .expect("assertion bucket")
            .upsert_persisted_semantic_statement(statement)
        {
            self.bump_revision();
            self.capture_statement_bucket(key);
        }
        Ok(key)
    }

    /// Edit one exact held assertion without selecting a content-key survivor.
    pub fn update_assertion_metadata(
        &mut self,
        statement_id: &str,
        label: Option<String>,
        asserted_at_ms: Option<u64>,
    ) -> bool {
        let Some((key, statement)) = self.find_semantic_statement(statement_id) else {
            return false;
        };
        if statement.label == label && statement.asserted_at_ms == asserted_at_ms {
            return false;
        }
        let owners: Vec<_> = self
            .inner
            .inner()
            .edge_references()
            .map(|edge| (RelationKey::Surface(edge.id()), edge.weight()))
            .chain(
                self.resource_relations()
                    .map(|(key, _, _, payload)| (RelationKey::Resource(key), payload)),
            )
            .filter(|(_, payload)| {
                payload
                    .semantic_statements()
                    .iter()
                    .any(|statement| statement.statement_id == statement_id)
            })
            .map(|(key, _)| key)
            .collect();
        for owner in owners {
            let semantic = self
                .get_relation_mut(owner)
                .unwrap()
                .semantic
                .as_mut()
                .unwrap();
            for statement in semantic
                .statements
                .iter_mut()
                .filter(|statement| statement.statement_id == statement_id)
            {
                statement.label = label.clone();
                statement.asserted_at_ms = asserted_at_ms;
            }
            semantic.rebuild_compat();
        }
        self.bump_revision();
        self.capture_statement_bucket(key);
        true
    }

    pub(crate) fn capture_statement_bucket(&self, key: RelationKey) {
        match key {
            RelationKey::Surface(key) => {
                if let Some((from, to)) = self.inner.inner().edge_endpoints(key) {
                    self.capture_semantic_pair(from, to);
                }
            },
            RelationKey::Resource(key) => {
                if let Some((from, to)) = self.resources.inner().edge_endpoints(key.raw()) {
                    self.capture_resource_pair(
                        self.resources.node(from).expect("resource endpoint").id(),
                        self.resources.node(to).expect("resource endpoint").id(),
                    );
                }
            },
        }
    }

    pub(crate) fn capture_resource_pair(&self, from: Uuid, to: Uuid) {
        self.record_delta(&CapturedDelta::ReplaySetResourceEdgesByIds {
            from_resource_id: from.to_string(),
            to_resource_id: to.to_string(),
            edges: self.persisted_resource_edges_between(from, to),
        });
    }

    /// Withdraw a handle from its owning store even after navigation or nature changes.
    pub fn retract_assertion(&mut self, statement_id: &str) -> bool {
        let mut removed = false;
        while let Some((key, _)) = self.find_semantic_statement(statement_id) {
            let key = key;
            let endpoints = match key {
                RelationKey::Surface(key) => self
                    .inner
                    .inner()
                    .edge_endpoints(key)
                    .map(|(a, b)| (self.get_node(a).unwrap().id, self.get_node(b).unwrap().id)),
                RelationKey::Resource(key) => {
                    self.resources
                        .inner()
                        .edge_endpoints(key.raw())
                        .map(|(a, b)| {
                            (
                                self.resources.node(a).unwrap().id(),
                                self.resources.node(b).unwrap().id(),
                            )
                        })
                },
            }
            .expect("assertion endpoints");
            let payload = self.get_relation_mut(key).expect("held assertion");
            payload.retract_semantic_statement(statement_id);
            let empty = payload.is_empty();
            if empty {
                match key {
                    RelationKey::Surface(key) => {
                        self.inner.disconnect(key);
                    },
                    RelationKey::Resource(key) => {
                        self.resources.disconnect(key.raw());
                    },
                }
            }
            self.bump_revision();
            match key {
                RelationKey::Surface(_) => self.capture_semantic_pair(
                    self.get_node_key_by_id(endpoints.0).unwrap(),
                    self.get_node_key_by_id(endpoints.1).unwrap(),
                ),
                RelationKey::Resource(_) => self.capture_resource_pair(endpoints.0, endpoints.1),
            }
            removed = true;
        }
        removed
    }
}
