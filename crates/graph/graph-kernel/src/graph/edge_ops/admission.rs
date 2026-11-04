// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

impl Graph {
    pub(crate) fn assert_relation(
        &mut self,
        from: NodeKey,
        to: NodeKey,
        assertion: EdgeAssertion,
    ) -> Option<RelationKey> {
        self.assert_relation_as(
            from,
            to,
            assertion,
            self.write_author().asserter_iri(),
            None,
        )
    }

    pub(crate) fn assert_relation_as(
        &mut self,
        from: NodeKey,
        to: NodeKey,
        assertion: EdgeAssertion,
        asserter_iri: String,
        asserted_at_ms: Option<u64>,
    ) -> Option<RelationKey> {
        if let EdgeAssertion::Semantic {
            sub_kind, label, ..
        } = assertion
        {
            return self
                .assert_semantic_statement(
                    from,
                    to,
                    SemanticStatementSpec {
                        predicate: super::super::predicate_iri(sub_kind).into(),
                        recognized_sub_kind: Some(sub_kind),
                        label,
                        graph_scope: GraphScope::Default,
                        provenance_iri: Some(asserter_iri),
                        asserted_at_ms,
                        basis: Default::default(),
                    },
                )
                .and_then(|(edge, outcome)| outcome.changed.then_some(edge));
        }
        let kind = match &assertion {
            EdgeAssertion::Containment { sub_kind } => RelationKind::Containment(*sub_kind),
            EdgeAssertion::Arrangement { sub_kind } => RelationKind::Arrangement(*sub_kind),
            EdgeAssertion::Imported { sub_kind } => RelationKind::Imported(*sub_kind),
            EdgeAssertion::Provenance { sub_kind } => RelationKind::Provenance(*sub_kind),
            EdgeAssertion::Semantic { .. } => unreachable!(),
        };
        let derived = is_derived_containment(kind);
        let stratum = super::super::built_in_relation_stratum(kind);
        if derived
            && self.derived_output_id_is_retained_for_pair(
                match stratum {
                    super::super::GraphStratum::Surface => "surface",
                    super::super::GraphStratum::Resource => "resource",
                },
                from,
                to,
                kind,
            )
        {
            return None;
        }
        let key = self.relation_bucket(from, to, stratum).ok()?;
        if derived && self.derived_output_id_is_retained(key, kind) {
            return None;
        }
        if !self.get_relation_mut(key)?.assert_relation_with_context(
            assertion,
            Some(asserter_iri),
            if derived {
                None
            } else {
                Some(asserted_at_ms.unwrap_or_else(unix_epoch_millis))
            },
        ) {
            return None;
        }
        if derived {
            self.normalize_derived_membership(key, kind);
        }
        self.bump_revision();
        self.capture_statement_bucket(key);
        Some(key)
    }

    pub(crate) fn assert_surface_relation_as(
        &mut self,
        from: NodeKey,
        to: NodeKey,
        assertion: EdgeAssertion,
        asserter_iri: String,
        asserted_at_ms: Option<u64>,
    ) -> Option<RelationKey> {
        if let EdgeAssertion::Semantic { sub_kind, label } = assertion {
            return self
                .assert_surface_semantic_statement(
                    from,
                    to,
                    SemanticStatementSpec {
                        predicate: super::super::predicate_iri(sub_kind).into(),
                        recognized_sub_kind: Some(sub_kind),
                        label,
                        graph_scope: GraphScope::Default,
                        provenance_iri: Some(asserter_iri),
                        asserted_at_ms,
                        basis: Default::default(),
                    },
                )
                .and_then(|(edge, outcome)| outcome.changed.then_some(edge));
        }
        let derived_kind = match &assertion {
            EdgeAssertion::Containment { sub_kind }
                if matches!(
                    sub_kind,
                    super::super::ContainmentSubKind::UrlPath
                        | super::super::ContainmentSubKind::Domain
                ) =>
            {
                Some(RelationKind::Containment(*sub_kind))
            },
            _ => None,
        };
        if !self.inner.contains_node(from) || !self.inner.contains_node(to) {
            return None;
        }
        if let Some(edge_key) = self.find_edge_key(from, to) {
            if let Some(kind) = derived_kind {
                if self.derived_output_id_is_retained(RelationKey::Surface(edge_key), kind) {
                    return None;
                }
            }
            // Existing edge: assert onto its payload. A real change (a new relation on the pair —
            // a multiplicity bump a weighted signal reads) advances the revision.
            let changed = {
                let payload = self.inner.edge_mut(edge_key)?;
                payload.assert_relation_with_context(
                    assertion,
                    Some(asserter_iri.clone()),
                    asserted_at_ms,
                )
            };
            if changed {
                if let Some(kind) = derived_kind {
                    self.normalize_derived_membership(RelationKey::Surface(edge_key), kind);
                }
                self.bump_revision();
                return Some(RelationKey::Surface(edge_key));
            }
            return None;
        }
        if let Some(kind) = derived_kind {
            if self.derived_output_id_is_retained_for_pair("surface", from, to, kind) {
                return None;
            }
        }
        let mut payload = EdgePayload::new();
        if !payload.assert_relation_with_context(
            assertion,
            Some(asserter_iri.clone()),
            asserted_at_ms,
        ) {
            return None;
        }
        let edge_key = self.inner.connect(from, to, payload);
        if let Some(kind) = derived_kind {
            self.normalize_derived_membership(RelationKey::Surface(edge_key), kind);
        }
        self.bump_revision();
        Some(RelationKey::Surface(edge_key))
    }

    /// Derived containment is a rule output, not authored history. Its ID is
    /// stable for this directed source bucket and its time remains undated.
    pub(super) fn normalize_derived_membership(&mut self, key: RelationKey, kind: RelationKind) {
        let endpoints = match key {
            RelationKey::Surface(key) => self.inner.inner().edge_endpoints(key).map(|(a, b)| {
                (
                    "surface",
                    self.inner.node(a).unwrap().id,
                    self.inner.node(b).unwrap().id,
                )
            }),
            RelationKey::Resource(key) => {
                self.resources
                    .inner()
                    .edge_endpoints(key.raw())
                    .map(|(a, b)| {
                        (
                            "resource",
                            self.resources.node(a).unwrap().id(),
                            self.resources.node(b).unwrap().id(),
                        )
                    })
            },
        };
        let Some((stratum, from, to)) = endpoints else {
            return;
        };
        let id = derived_containment_statement_id(stratum, from, to, kind);
        if self.find_semantic_statement(&id).is_some() {
            return;
        }
        for statement in &mut self.get_relation_mut(key).unwrap().assertions.statements {
            if !statement.is_held() || statement.relation_kind() != kind {
                continue;
            }
            statement.statement_id = id.clone();
            statement.provenance_iri = Some("urn:mere:rule:derived-containment:v1".into());
            statement.first_asserted_ms = None;
            statement.asserted_at_ms = None;
            statement.basis = crate::types::AssertionBasis {
                method: Some(crate::types::AssertionMethod {
                    iri: "urn:mere:method:derived-containment".into(),
                    version: Some("1".into()),
                }),
                ..Default::default()
            };
        }
    }

    pub(super) fn derived_output_id_is_retained(
        &self,
        key: RelationKey,
        kind: RelationKind,
    ) -> bool {
        let endpoints = match key {
            RelationKey::Surface(key) => {
                self.inner.inner().edge_endpoints(key).and_then(|(a, b)| {
                    Some(("surface", self.inner.node(a)?.id, self.inner.node(b)?.id))
                })
            },
            RelationKey::Resource(key) => self
                .resources
                .inner()
                .edge_endpoints(key.raw())
                .and_then(|(a, b)| {
                    Some((
                        "resource",
                        self.resources.node(a)?.id(),
                        self.resources.node(b)?.id(),
                    ))
                }),
        };
        endpoints.is_some_and(|(stratum, from, to)| {
            self.derived_output_id_is_retained_for_identity(stratum, from, to, kind)
        })
    }

    pub(super) fn derived_output_id_is_retained_for_pair(
        &self,
        stratum: &str,
        from_key: NodeKey,
        to_key: NodeKey,
        kind: RelationKind,
    ) -> bool {
        let (Some(from), Some(to)) = (self.get_node(from_key), self.get_node(to_key)) else {
            return false;
        };
        let (from, to) = if stratum == "resource" {
            let from_id = self
                .shown_resource_id(from_key)
                .unwrap_or_else(|| super::super::resource::ResourceNode::new(from.url()).id());
            let to_id = self
                .shown_resource_id(to_key)
                .unwrap_or_else(|| super::super::resource::ResourceNode::new(to.url()).id());
            (from_id, to_id)
        } else {
            (from.id, to.id)
        };
        self.derived_output_id_is_retained_for_identity(stratum, from, to, kind)
    }

    pub(super) fn derived_output_id_is_retained_for_identity(
        &self,
        stratum: &str,
        from: Uuid,
        to: Uuid,
        kind: RelationKind,
    ) -> bool {
        let id = derived_containment_statement_id(stratum, from, to, kind);
        self.find_semantic_statement(&id).is_some()
    }

    pub(crate) fn assert_semantic_relation_in_scope(
        &mut self,
        from: NodeKey,
        to: NodeKey,
        sub_kind: SemanticSubKind,
        label: Option<String>,
        graph_scope: GraphScope,
    ) -> Option<RelationKey> {
        self.assert_semantic_statement(
            from,
            to,
            SemanticStatementSpec {
                predicate: super::super::predicate_iri(sub_kind).into(),
                recognized_sub_kind: Some(sub_kind),
                label,
                graph_scope,
                provenance_iri: Some(self.write_author().asserter_iri()),
                asserted_at_ms: None,
                basis: Default::default(),
            },
        )
        .and_then(|(edge, outcome)| outcome.changed.then_some(edge))
    }

    /// Assert an **open predicate** semantic relation from `from` to `to`,
    /// identified by an IRI rather than a closed [`SemanticSubKind`]. Creates the
    /// pair-local edge bucket if absent and appends one open semantic statement to
    /// it; an edge carrying only such a statement still reports the `Semantic`
    /// family. Returns the edge, or `None` if either endpoint is missing or the
    /// same open predicate statement already exists. Write path for raw web
    /// predicates (linked-data ingest / knot `rel`s outside Mere's vocabulary).
    pub(crate) fn assert_semantic_predicate(
        &mut self,
        from: NodeKey,
        to: NodeKey,
        predicate: String,
    ) -> Option<RelationKey> {
        self.assert_semantic_predicate_in_scope(from, to, predicate, GraphScope::Default)
    }

    pub(crate) fn assert_semantic_predicate_in_scope(
        &mut self,
        from: NodeKey,
        to: NodeKey,
        predicate: String,
        graph_scope: GraphScope,
    ) -> Option<RelationKey> {
        self.assert_semantic_statement(
            from,
            to,
            SemanticStatementSpec {
                recognized_sub_kind: None,
                predicate,
                label: None,
                graph_scope,
                provenance_iri: Some(self.write_author().asserter_iri()),
                asserted_at_ms: None,
                basis: Default::default(),
            },
        )
        .and_then(|(edge, outcome)| outcome.changed.then_some(edge))
    }
}
