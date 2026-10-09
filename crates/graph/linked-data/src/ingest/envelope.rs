// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Parser evidence for the definition part of the RDF import profile.

use std::collections::{BTreeMap, BTreeSet};

use kernel::types::{GraphScope, NodeProperty};
use oxrdf::{GraphName, Quad, Term};

use super::{GraphContribution, PROV_WAS_ATTRIBUTED_TO, RDF_TYPE, XSD_STRING, subject_iri};

pub(super) const SKOS_CONCEPT: &str = "http://www.w3.org/2004/02/skos/core#Concept";
const SKOS_PREF_LABEL: &str = "http://www.w3.org/2004/02/skos/core#prefLabel";

/// An RDF contribution and the parser evidence needed to import its definitions.
/// The enclosed DTO is read-only so its entries cannot diverge from the evidence.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImportEnvelope {
    pub(super) contribution: GraphContribution,
    pub(super) evidence: ImportEvidence,
}

impl ImportEnvelope {
    /// Read the unchanged contribution DTO.
    pub fn contribution(&self) -> &GraphContribution {
        &self.contribution
    }

    /// Discard the profile evidence for a caller that needs the legacy DTO API.
    pub fn into_contribution(self) -> GraphContribution {
        self.contribution
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ConceptDefinition {
    pub label: String,
    pub owner: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct ImportEvidence {
    pub definitions: BTreeMap<String, ConceptDefinition>,
    plain_properties: BTreeSet<(String, String)>,
    plain_edges: BTreeSet<usize>,
    carried_properties: BTreeSet<(String, String)>,
}

impl ImportEvidence {
    pub fn from_quads(quads: &[Quad], namespace: &str) -> Self {
        #[derive(Default)]
        struct Candidate {
            concept: bool,
            labels: BTreeSet<String>,
            owners: BTreeSet<String>,
            unsupported: bool,
        }
        let mut candidates: BTreeMap<String, Candidate> = BTreeMap::new();
        for quad in quads {
            if quad.graph_name != GraphName::DefaultGraph {
                continue;
            }
            let candidate = candidates
                .entry(subject_iri(&quad.subject, namespace))
                .or_default();
            match (quad.predicate.as_str(), &quad.object) {
                (RDF_TYPE, Term::NamedNode(node)) if node.as_str() == SKOS_CONCEPT => {
                    candidate.concept = true;
                },
                (SKOS_PREF_LABEL, Term::Literal(label))
                    if label.language().is_none() && label.datatype().as_str() == XSD_STRING =>
                {
                    candidate.labels.insert(label.value().into());
                },
                (PROV_WAS_ATTRIBUTED_TO, Term::NamedNode(owner)) => {
                    candidate.owners.insert(owner.as_str().into());
                },
                (SKOS_PREF_LABEL | PROV_WAS_ATTRIBUTED_TO, _) => {
                    candidate.unsupported = true;
                },
                _ => {},
            }
        }
        let definitions = candidates
            .into_iter()
            .filter_map(|(subject, candidate)| {
                if candidate.concept
                    && !candidate.unsupported
                    && candidate.labels.len() == 1
                    && candidate.owners.len() == 1
                {
                    Some((
                        subject,
                        ConceptDefinition {
                            label: candidate.labels.into_iter().next().unwrap(),
                            owner: candidate.owners.into_iter().next().unwrap(),
                        },
                    ))
                } else {
                    None
                }
            })
            .collect();
        Self {
            definitions,
            ..Self::default()
        }
    }

    pub fn plain_property(&mut self, subject: &str, property: &NodeProperty) {
        if self.definitions.get(subject).is_some_and(|definition| {
            property.predicate == SKOS_PREF_LABEL
                && property.value == definition.label
                && property.lang.is_none()
                && property
                    .datatype
                    .as_deref()
                    .is_none_or(|datatype| datatype == XSD_STRING)
                && property.graph_scope == GraphScope::Default
        }) {
            self.plain_properties
                .insert((subject.into(), property.statement_id.clone()));
        }
    }

    pub fn reified_property(&mut self, subject: &str, plain_id: &str) {
        self.plain_properties
            .remove(&(subject.into(), plain_id.into()));
    }

    pub fn carried_property(&mut self, subject: &str, property: &NodeProperty) {
        self.carried_properties
            .insert((subject.into(), property.statement_id.clone()));
    }

    pub fn is_carried_property(&self, subject: &str, property: &NodeProperty) -> bool {
        self.carried_properties
            .contains(&(subject.into(), property.statement_id.clone()))
    }

    pub fn plain_edge(&mut self, index: usize, edge: &super::EdgeContribution) {
        if self
            .definitions
            .get(&edge.subject)
            .is_some_and(|definition| {
                edge.predicate == PROV_WAS_ATTRIBUTED_TO
                    && edge.object == definition.owner
                    && edge.graph_scope == GraphScope::Default
            })
        {
            self.plain_edges.insert(index);
        }
    }

    pub fn reified_edge(&mut self, index: usize) {
        self.plain_edges.remove(&index);
    }

    pub fn is_plain_property(&self, subject: &str, property: &NodeProperty) -> bool {
        self.plain_properties
            .contains(&(subject.into(), property.statement_id.clone()))
    }

    pub fn is_plain_edge(&self, index: usize) -> bool {
        self.plain_edges.contains(&index)
    }
}
