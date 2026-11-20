// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use oxrdf::{BlankNode, GraphName, NamedNode, NamedOrBlankNode, Term, Triple};
use oxttl::{NQuadsParser, NQuadsSerializer};
use serde::{Deserialize, Serialize};

use super::{PageClaimBlock, PageClaimDiagnostic, PageClaims};
use crate::dataset::IndexedDataset;
use crate::model::ResourceNode;

/// The exact block claims of one immutable captured edition, in a Resource facet.
pub const KEPT_PAGE_CLAIMS_FACET: &str = "mere.kept-page-claims/v1";

/// A journaled source's claims. `source` is the kept capture's graph IRI;
/// provenance in `claims` retains the original graph names and block locators.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeptPageClaims {
    pub source: NamedNode,
    pub claims: PageClaims,
}

impl KeptPageClaims {
    /// Scope blank nodes to their immutable kept capture before storing.
    #[must_use]
    pub fn new(source: NamedNode, claims: &PageClaims) -> Self {
        let mut claims = claims.clone();
        let namespace = ResourceNode::for_term(source.as_str()).id().simple();
        let scope = |node: &BlankNode| {
            BlankNode::new_unchecked(format!("kept_{namespace}_{}", node.as_str()))
        };
        for block in &mut claims.blocks {
            for quad in &mut block.quads {
                scope_subject(&mut quad.subject, &scope);
                scope_term(&mut quad.object, &scope);
                if let GraphName::BlankNode(node) = &quad.graph_name {
                    quad.graph_name = scope(node).into();
                }
            }
            block.quads.sort_by_cached_key(ToString::to_string);
        }
        Self { source, claims }
    }

    /// The isolated, indexed data for validation. Every block contributes to
    /// this kept capture's named graph; original graph names remain in evidence.
    #[must_use]
    pub fn dataset(&self) -> IndexedDataset {
        IndexedDataset::from_quads(self.claims.blocks.iter().flat_map(|block| {
            block.quads.iter().cloned().map(|mut quad| {
                quad.graph_name = self.source.clone().into();
                (quad, true)
            })
        }))
    }

    /// Blocks asserting the selected property of `focus`. If the property is
    /// absent (such as a failed minimum count), return blocks that describe
    /// the focus so the host can open evidence of the node that was validated.
    #[must_use]
    pub fn evidence_for(
        &self,
        focus: &Term,
        predicate: Option<&NamedNode>,
    ) -> Vec<&PageClaimBlock> {
        let subject = NamedOrBlankNode::try_from(focus.clone()).ok();
        let mut focus_blocks = Vec::new();
        let mut exact_blocks = Vec::new();
        let mut incoming_blocks = Vec::new();
        for block in &self.claims.blocks {
            let mut statements = block
                .quads
                .iter()
                .filter(|quad| Some(&quad.subject) == subject.as_ref());
            if let Some(first) = statements.next() {
                focus_blocks.push(block);
                if predicate.is_some_and(|predicate| {
                    first.predicate == *predicate
                        || statements.any(|quad| quad.predicate == *predicate)
                }) {
                    exact_blocks.push(block);
                }
            } else if block.quads.iter().any(|quad| &quad.object == focus) {
                incoming_blocks.push(block);
            }
        }
        if !exact_blocks.is_empty() {
            exact_blocks
        } else if !focus_blocks.is_empty() {
            focus_blocks
        } else {
            incoming_blocks
        }
    }

    /// Encode the exact evidence, without storing a duplicate of the source bytes.
    #[must_use]
    pub fn facet_value(&self) -> serde_json::Value {
        let blocks = self.claims.blocks.iter().map(StoredBlock::from).collect();
        serde_json::to_value(StoredClaims {
            blocks,
            diagnostics: self.claims.diagnostics.clone(),
        })
        .expect("page claims facet serializes")
    }

    /// Decode the journaled evidence. A malformed stored row is a read error.
    ///
    /// # Errors
    ///
    /// The facet or any stored RDF block could not be decoded.
    pub fn from_facet(source: NamedNode, value: &serde_json::Value) -> Result<Self, String> {
        let stored = StoredClaims::deserialize(value).map_err(|error| error.to_string())?;
        let blocks = stored
            .blocks
            .into_iter()
            .map(PageClaimBlock::try_from)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            source,
            claims: PageClaims {
                blocks,
                diagnostics: stored.diagnostics,
            },
        })
    }
}

fn scope_subject(subject: &mut NamedOrBlankNode, scope: &impl Fn(&BlankNode) -> BlankNode) {
    if let NamedOrBlankNode::BlankNode(node) = subject {
        *subject = scope(node).into();
    }
}

fn scope_term(term: &mut Term, scope: &impl Fn(&BlankNode) -> BlankNode) {
    match term {
        Term::BlankNode(node) => *term = scope(node).into(),
        Term::Triple(triple) => {
            let Triple {
                subject, object, ..
            } = triple.as_mut();
            scope_subject(subject, scope);
            scope_term(object, scope);
        },
        Term::NamedNode(_) | Term::Literal(_) => {},
    }
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct StoredClaims {
    blocks: Vec<StoredBlock>,
    diagnostics: Vec<PageClaimDiagnostic>,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct StoredBlock {
    order: u64,
    nquads: String,
}

impl From<&PageClaimBlock> for StoredBlock {
    fn from(block: &PageClaimBlock) -> Self {
        let mut writer = NQuadsSerializer::new().for_writer(Vec::new());
        for quad in &block.quads {
            writer
                .serialize_quad(quad)
                .expect("in-memory N-Quads write");
        }
        Self {
            order: block.order,
            nquads: String::from_utf8(writer.finish()).expect("N-Quads is UTF-8"),
        }
    }
}

impl TryFrom<StoredBlock> for PageClaimBlock {
    type Error = String;

    fn try_from(block: StoredBlock) -> Result<Self, String> {
        let quads = NQuadsParser::new()
            .for_slice(block.nquads.as_bytes())
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| error.to_string())?;
        Ok(Self {
            order: block.order,
            quads,
        })
    }
}
