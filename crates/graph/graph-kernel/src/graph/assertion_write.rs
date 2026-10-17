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
use crate::time::unix_epoch_millis;
use crate::types::AssertionWithdrawal;

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
        assert!(
            graph
                .find_semantic_statement(&first.statement_id)
                .unwrap()
                .1
                .withdrawn
                .is_some()
        );
        assert!(
            graph
                .find_held_semantic_statement(&first.statement_id)
                .is_none()
        );
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
        let mut other = edges[0].retained_assertions()[0].clone();
        other.statement_id = "second-exact-handle".into();
        edges[0].assertions.as_mut().unwrap().push(other.clone());
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
        precise.basis.note = Some("precise edit".into());
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
        assert!(
            graph
                .find_semantic_statement(&first.statement_id)
                .unwrap()
                .1
                .withdrawn
                .is_some()
        );
        let withdrawn_copies: Vec<_> = graph
            .resource_relations()
            .flat_map(|(_, _, _, payload)| payload.semantic_statements().iter())
            .filter(|statement| statement.statement_id == first.statement_id)
            .collect();
        assert_eq!(withdrawn_copies.len(), 2);
        assert!(
            withdrawn_copies
                .iter()
                .all(|statement| statement.withdrawn.is_some())
        );
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
                .all(|statement| statement.label.is_none() && statement.asserted_at_ms == Some(99))
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
    statement.withdrawn.is_none()
        && statement.predicate == spec.predicate
        && statement.recognized_sub_kind == spec.recognized_sub_kind
        && statement.graph_scope == spec.graph_scope
        && statement.provenance_iri == spec.provenance_iri
        && statement.basis.same_reason_as(&spec.basis)
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
                .assertions()
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
        spec: SemanticStatementSpec,
    ) -> Result<(RelationKey, StatementAssert), StatementWriteError> {
        let mut spec = spec.normalized();
        if self.resource(from).is_none() || self.resource(to).is_none() {
            return Err(StatementWriteError::MissingEndpoint);
        }
        if spec.provenance_iri.is_none() {
            spec.provenance_iri = Some(self.write_author().asserter_iri());
        }
        if let Some((key, id)) = self.held_statement_by_resource_ids(from, to, &spec)? {
            let changed = self.update_assertion_metadata_if_requested(
                &id,
                spec.label,
                spec.asserted_at_ms,
                Some(spec.basis),
            );
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
        spec.asserted_at_ms.get_or_insert_with(unix_epoch_millis);
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

    /// Locate any retained handle, including withdrawn history, independent of
    /// current nature. Use `find_held_semantic_statement` for default reads.
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
                    .assertions()
                    .iter()
                    .find(|statement| statement.statement_id == statement_id)
                    .map(|statement| (key, statement))
            })
    }

    pub fn find_held_semantic_statement(
        &self,
        statement_id: &str,
    ) -> Option<(RelationKey, &SemanticStatement)> {
        self.find_semantic_statement(statement_id)
            .filter(|(_, statement)| statement.is_held())
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

    fn semantic_owner_matches_pair(&self, owner: RelationKey, from: NodeKey, to: NodeKey) -> bool {
        match owner {
            RelationKey::Surface(key) => self.inner.inner().edge_endpoints(key) == Some((from, to)),
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
        }
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
                .assertions()
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
        spec: SemanticStatementSpec,
    ) -> Result<(RelationKey, StatementAssert), StatementWriteError> {
        let spec = spec.normalized();
        self.try_assert_semantic_statement_with_clock(from, to, spec, true)
    }

    /// Import without a carried ID: retain the source time if present, but do
    /// not create chronology when the source supplied none. Content ensure
    /// keeps repeated imports idempotent until the source provides identity.
    pub fn try_import_semantic_statement(
        &mut self,
        from: NodeKey,
        to: NodeKey,
        spec: SemanticStatementSpec,
    ) -> Result<(RelationKey, StatementAssert), StatementWriteError> {
        let spec = spec.normalized();
        self.try_assert_semantic_statement_with_clock(from, to, spec, false)
    }

    fn try_assert_semantic_statement_with_clock(
        &mut self,
        from: NodeKey,
        to: NodeKey,
        mut spec: SemanticStatementSpec,
        stamp_new: bool,
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
                .assertions()
                .iter()
                .find(|statement| matches_spec(statement, &spec))
                .expect("held statement")
                .statement_id
                .clone();
            let changed = self.update_assertion_metadata_if_requested(
                &statement_id,
                spec.label,
                spec.asserted_at_ms,
                Some(spec.basis),
            );
            return Ok((
                key,
                StatementAssert {
                    statement_id,
                    changed,
                },
            ));
        }
        if stamp_new {
            spec.asserted_at_ms.get_or_insert_with(unix_epoch_millis);
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
        statement.normalize_basis_compat();
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
            basis: statement.basis.clone(),
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
            let same_claim = existing.predicate == spec.predicate
                && existing.recognized_sub_kind == spec.recognized_sub_kind
                && existing.graph_scope == spec.graph_scope
                && existing.provenance_iri == spec.provenance_iri
                && existing.basis.same_reason_as(&statement.basis);
            if !pair_matches || !same_claim {
                return Err(StatementWriteError::HandleCollision(statement.statement_id));
            }
            if existing.withdrawn.is_some() {
                // A stale exact-ID import is observed but cannot revive a
                // tombstone. Causal history may still carry later events.
                return Ok(owner);
            }
            let statement_id = statement.statement_id.clone();
            let existing_first = existing.first_asserted_ms;
            let existing_last = existing.asserted_at_ms;
            self.update_assertion_record(
                &statement_id,
                statement.label,
                statement.asserted_at_ms.or(existing_last),
                existing_first,
                statement.withdrawn.clone(),
                statement.basis,
            );
            return Ok(owner);
        }
        let key = self.new_statement_bucket(from, to, &statement.predicate)?;
        if self
            .get_relation_mut(key)
            .expect("assertion bucket")
            .push_persisted_semantic_statement(statement)
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
        let Some((_, statement)) = self.find_semantic_statement(statement_id) else {
            return false;
        };
        if statement.withdrawn.is_some() {
            return false;
        }
        let mut basis = statement.basis.clone();
        basis.note = label.clone();
        if statement.basis == basis && statement.asserted_at_ms == asserted_at_ms {
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
                    .assertions()
                    .iter()
                    .any(|statement| statement.statement_id == statement_id)
            })
            .map(|(key, _)| key)
            .collect();
        for owner in &owners {
            let semantic = &mut self.get_relation_mut(*owner).unwrap().assertions;
            for statement in semantic
                .statements
                .iter_mut()
                .filter(|statement| statement.statement_id == statement_id)
            {
                statement.label = label.clone();
                statement.basis = basis.clone();
                statement.asserted_at_ms = asserted_at_ms;
            }
            semantic.rebuild_compat();
        }
        self.bump_revision();
        for owner in owners {
            self.capture_statement_bucket(owner);
        }
        true
    }

    pub(crate) fn update_assertion_metadata_if_requested(
        &mut self,
        statement_id: &str,
        label: Option<String>,
        asserted_at_ms: Option<u64>,
        basis: Option<crate::types::AssertionBasis>,
    ) -> bool {
        let Some((_, current)) = self.find_semantic_statement(statement_id) else {
            return false;
        };
        let desired_basis = basis.unwrap_or_else(|| crate::types::AssertionBasis {
            note: label.clone(),
            ..Default::default()
        });
        if current.basis == desired_basis && asserted_at_ms.is_none() {
            return false;
        }
        let Some((_, current)) = self.find_semantic_statement(statement_id) else {
            return false;
        };
        self.update_assertion_record(
            statement_id,
            desired_basis.note.clone(),
            asserted_at_ms.or(current.asserted_at_ms),
            current.first_asserted_ms,
            current.withdrawn.clone(),
            desired_basis,
        )
    }

    fn update_assertion_record(
        &mut self,
        statement_id: &str,
        label: Option<String>,
        asserted_at_ms: Option<u64>,
        first_asserted_ms: Option<u64>,
        withdrawn: Option<AssertionWithdrawal>,
        basis: crate::types::AssertionBasis,
    ) -> bool {
        let owners = self.semantic_statement_owner_keys(statement_id);
        if owners.is_empty() {
            return false;
        }
        let mut changed_owners = Vec::new();
        for owner in owners {
            let Some(data) = self
                .get_relation_mut(owner)
                .map(|payload| &mut payload.assertions)
            else {
                continue;
            };
            let mut changed = false;
            for statement in data
                .statements
                .iter_mut()
                .filter(|statement| statement.statement_id == statement_id)
            {
                if statement.label == label
                    && statement.asserted_at_ms == asserted_at_ms
                    && statement.first_asserted_ms == first_asserted_ms
                    && statement.withdrawn == withdrawn
                    && statement.basis == basis
                {
                    continue;
                }
                statement.label = label.clone();
                statement.asserted_at_ms = asserted_at_ms;
                statement.first_asserted_ms = first_asserted_ms;
                statement.withdrawn = withdrawn.clone();
                statement.basis = basis.clone();
                changed = true;
            }
            if changed {
                data.rebuild_compat();
                changed_owners.push(owner);
            }
        }
        if changed_owners.is_empty() {
            return false;
        }
        self.bump_revision();
        for owner in changed_owners {
            self.capture_statement_bucket(owner);
        }
        true
    }

    fn semantic_statement_owner_keys(&self, statement_id: &str) -> Vec<RelationKey> {
        self.inner
            .inner()
            .edge_references()
            .map(|edge| (RelationKey::Surface(edge.id()), edge.weight()))
            .chain(
                self.resource_relations()
                    .map(|(key, _, _, payload)| (RelationKey::Resource(key), payload)),
            )
            .filter(|(_, payload)| {
                payload
                    .assertions()
                    .iter()
                    .any(|statement| statement.statement_id == statement_id)
            })
            .map(|(key, _)| key)
            .collect()
    }

    /// Ensure an assertion by its L13 reason key. Method version and note may
    /// update on the same ID; a changed method IRI or evidence set creates a
    /// new independently withdrawable assertion.
    pub fn try_assert_semantic_reason(
        &mut self,
        from: NodeKey,
        to: NodeKey,
        mut spec: SemanticStatementSpec,
    ) -> Result<(RelationKey, StatementAssert), StatementWriteError> {
        spec.asserted_at_ms.get_or_insert_with(unix_epoch_millis);
        self.try_assert_semantic_statement(from, to, spec)
    }

    /// Confirm one exact held ID. Missing caller time is stamped once here;
    /// an unknown or withdrawn ID is never selected by matching its content.
    pub fn reconfirm_semantic_assertion(
        &mut self,
        statement_id: &str,
        label: Option<String>,
        asserted_at_ms: Option<u64>,
    ) -> bool {
        let Some((key, current)) = self.find_semantic_statement(statement_id) else {
            return false;
        };
        if current.withdrawn.is_some() {
            return false;
        }
        self.update_assertion_record(
            statement_id,
            label.clone(),
            Some(asserted_at_ms.unwrap_or_else(unix_epoch_millis)),
            current.first_asserted_ms,
            None,
            crate::types::AssertionBasis {
                note: label,
                ..current.basis.clone()
            },
        )
    }

    /// Apply an exact-ID withdrawal. Unknown IDs are explicitly unapplied;
    /// callers with causal event streams must defer/reject until admitted.
    pub fn withdraw_semantic_assertion(
        &mut self,
        statement_id: &str,
        at_ms: Option<u64>,
        by: Option<String>,
    ) -> bool {
        let actor = by.or_else(|| Some(self.write_author().asserter_iri()));
        self.withdraw_semantic_assertion_record(
            statement_id,
            AssertionWithdrawal {
                at_ms: Some(at_ms.unwrap_or_else(unix_epoch_millis)),
                by: actor,
            },
        )
    }

    /// Import/replay path: preserve missing time and never synthesize the
    /// clock. It returns false for unknown IDs so an event is not acknowledged.
    pub fn apply_imported_semantic_withdrawal(
        &mut self,
        statement_id: &str,
        at_ms: Option<u64>,
        by: Option<String>,
    ) -> bool {
        self.withdraw_semantic_assertion_record(statement_id, AssertionWithdrawal { at_ms, by })
    }

    /// Apply an imported exact-ID withdrawal only when its authored pair owns
    /// the assertion. Source events with no carried time remain undated.
    pub fn apply_imported_semantic_withdrawal_for_pair(
        &mut self,
        from: NodeKey,
        to: NodeKey,
        statement_id: &str,
        at_ms: Option<u64>,
        by: Option<String>,
    ) -> bool {
        let Some((owner, _)) = self.find_semantic_statement(statement_id) else {
            return false;
        };
        self.semantic_owner_matches_pair(owner, from, to)
            && self.apply_imported_semantic_withdrawal(statement_id, at_ms, by)
    }

    /// Authored pair-checked exact-ID withdrawal. It records the current time
    /// and configured writer only after verifying the owning bucket.
    pub fn withdraw_semantic_assertion_for_pair(
        &mut self,
        from: NodeKey,
        to: NodeKey,
        statement_id: &str,
    ) -> bool {
        let Some((owner, _)) = self.find_semantic_statement(statement_id) else {
            return false;
        };
        self.semantic_owner_matches_pair(owner, from, to)
            && self.withdraw_semantic_assertion(statement_id, None, None)
    }

    fn withdraw_semantic_assertion_record(
        &mut self,
        statement_id: &str,
        withdrawal: AssertionWithdrawal,
    ) -> bool {
        let owners = self.semantic_statement_owner_keys(statement_id);
        let mut changed_owners = Vec::new();
        for owner in owners {
            let Some(data) = self
                .get_relation_mut(owner)
                .map(|payload| &mut payload.assertions)
            else {
                continue;
            };
            let mut changed = false;
            for statement in data.statements.iter_mut().filter(|statement| {
                statement.statement_id == statement_id && statement.withdrawn.is_none()
            }) {
                statement.withdrawn = Some(withdrawal.clone());
                changed = true;
            }
            if changed {
                data.rebuild_compat();
                changed_owners.push(owner);
            }
        }
        if changed_owners.is_empty() {
            return false;
        }
        self.bump_revision();
        for owner in changed_owners {
            self.capture_statement_bucket(owner);
        }
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

    /// All-family aliases retain exact owner, ID, basis and lifecycle admission.
    pub fn find_assertion(&self, id: &str) -> Option<(RelationKey, &SemanticStatement)> {
        self.find_semantic_statement(id)
    }
    pub fn find_held_assertion(&self, id: &str) -> Option<(RelationKey, &SemanticStatement)> {
        self.find_held_semantic_statement(id)
    }
    pub fn try_assert_relation_statement(
        &mut self,
        from: NodeKey,
        to: NodeKey,
        spec: SemanticStatementSpec,
    ) -> Result<(RelationKey, StatementAssert), StatementWriteError> {
        self.try_assert_semantic_statement(from, to, spec)
    }
    pub fn try_assert_persisted_relation_statement(
        &mut self,
        from: NodeKey,
        to: NodeKey,
        statement: SemanticStatement,
    ) -> Result<RelationKey, StatementWriteError> {
        self.try_assert_persisted_semantic_statement(from, to, statement)
    }

    /// Compatibility spelling for authored exact-ID withdrawal.
    pub fn retract_assertion(&mut self, statement_id: &str) -> bool {
        self.withdraw_semantic_assertion(statement_id, None, None)
    }
}
