// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! One JSON-LD dataset per page, retaining the source block of every claim.

use std::collections::HashSet;

use oxrdf::{BlankNode, GraphName, Literal, NamedNode, NamedOrBlankNode, Quad, Term};

pub mod kept;
pub use kept::{KEPT_PAGE_CLAIMS_FACET, KeptPageClaims};

use crate::ContextCache;
use crate::dataset::View;
use crate::formats::canon::{CanonOptions, canonicalize_quads};
use crate::formats::parse::{JsonLdOptions, parse_jsonld};

/// Maximum bytes read from an individual embedded script.
pub const MAX_JSON_LD_BLOCK_BYTES: usize = 64 * 1024;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PageClaims {
    pub blocks: Vec<PageClaimBlock>,
    pub diagnostics: Vec<PageClaimDiagnostic>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PageClaimBlock {
    /// Original document order, as supplied by the extractor.
    pub order: u64,
    pub quads: Vec<Quad>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PageClaimDiagnostic {
    /// Locator of the block that did not produce claims.
    pub order: u64,
    pub message: String,
}

/// Interpret each block with the document base and the caller's offline contexts.
/// A failed block leaves the other blocks' claims available. An explicit blank
/// node label can identify the same node in multiple blocks of the document.
pub fn interpret<'a>(
    contexts: &ContextCache,
    base_iri: &str,
    blocks: impl IntoIterator<Item = (u64, &'a str)>,
) -> PageClaims {
    let mut result = PageClaims::default();
    // OxJSON-LD preserves explicit labels and creates fresh nodes for anonymous
    // objects and list cells. Compare its RDF terms before canonicalizing them.
    let mut seen = HashSet::new();
    let mut shared = HashSet::new();
    for (order, block) in blocks {
        match parse_block(contexts, base_iri, block) {
            Ok(quads) => {
                let local: HashSet<_> = quads.iter().flat_map(blank_nodes).collect();
                for node in local {
                    if !seen.insert(node.clone()) {
                        shared.insert(node.clone());
                    }
                }
                result.blocks.push(PageClaimBlock { order, quads });
            },
            Err(message) => result
                .diagnostics
                .push(PageClaimDiagnostic { order, message }),
        }
    }

    result
        .blocks
        .retain_mut(|block| match label_block(block, &shared) {
            Ok(()) => true,
            Err(message) => {
                result.diagnostics.push(PageClaimDiagnostic {
                    order: block.order,
                    message,
                });
                false
            },
        });
    result.diagnostics.sort_by_key(|issue| issue.order);
    result
}

fn blank_nodes(quad: &Quad) -> impl Iterator<Item = &BlankNode> {
    let subject = match &quad.subject {
        NamedOrBlankNode::BlankNode(node) => Some(node),
        _ => None,
    };
    let object = match &quad.object {
        Term::BlankNode(node) => Some(node),
        _ => None,
    };
    let graph = match &quad.graph_name {
        GraphName::BlankNode(node) => Some(node),
        _ => None,
    };
    [subject, object, graph].into_iter().flatten()
}

fn parse_block(contexts: &ContextCache, base_iri: &str, block: &str) -> Result<Vec<Quad>, String> {
    if block.len() > MAX_JSON_LD_BLOCK_BYTES {
        return Err(format!(
            "JSON-LD block exceeds the {MAX_JSON_LD_BLOCK_BYTES}-byte limit."
        ));
    }
    let dataset = parse_jsonld(
        block.as_bytes(),
        contexts.clone(),
        Some(base_iri),
        JsonLdOptions::default(),
    )
    .map_err(|error| error.to_string())?;
    Ok(dataset.quads(View::Live).collect())
}

fn label_block(block: &mut PageClaimBlock, shared: &HashSet<BlankNode>) -> Result<(), String> {
    let quads = &mut block.quads;
    let nodes: HashSet<_> = quads.iter().flat_map(blank_nodes).collect();
    if !nodes.is_empty() {
        // Keep the old deterministic labels for block-local nodes. Anchor each
        // shared node by its document label so symmetric local descriptions
        // cannot swap identities between retries. These extra quads are only
        // canonicalizer input and never enter the evidence or the dataset.
        let shared_here: Vec<_> = nodes.into_iter().filter(|n| shared.contains(*n)).collect();
        let canonical = if shared_here.is_empty() {
            canonicalize_quads(quads, &CanonOptions::default())
        } else {
            let predicates: HashSet<_> = quads.iter().map(|q| q.predicate.as_str()).collect();
            let marker = (0u32..)
                .map(|i| format!("urn:mere:page-label:{i}"))
                .find(|p| !predicates.contains(p.as_str()))
                .expect("bounded input");
            let marker = NamedNode::new_unchecked(marker);
            let mut input = quads.clone();
            input.extend(shared_here.into_iter().map(|n| {
                Quad::new(
                    n.clone(),
                    marker.clone(),
                    Literal::new_simple_literal(n.as_str()),
                    GraphName::DefaultGraph,
                )
            }));
            canonicalize_quads(&input, &CanonOptions::default())
        }
        .map_err(|error| error.to_string())?;
        let scoped = |node: &BlankNode| {
            if shared.contains(node) {
                BlankNode::new_unchecked(format!("page_shared_{}", node.as_str()))
            } else {
                let label = &canonical.issued[node.as_str()];
                BlankNode::new_unchecked(format!("page_block_{}_{label}", block.order))
            }
        };
        for quad in quads.iter_mut() {
            if let NamedOrBlankNode::BlankNode(node) = &quad.subject {
                quad.subject = scoped(node).into();
            }
            if let Term::BlankNode(node) = &quad.object {
                quad.object = scoped(node).into();
            }
            if let GraphName::BlankNode(node) = &quad.graph_name {
                quad.graph_name = scoped(node).into();
            }
        }
    }
    quads.sort_by_cached_key(ToString::to_string);
    Ok(())
}

#[cfg(test)]
mod tests {
    use oxrdf::NamedNode;

    use super::*;

    #[test]
    fn explicit_labels_span_scripts_while_failures_keep_their_locator() {
        let block = r##"{"@id":"#place","http://example.test/contains":{"@id":"_:node","http://example.test/name":"river"}}"##;
        let notes = "https://example.test/field-notes";
        let blocks = [
            (0, block),
            (1, "{"),
            (
                2,
                r#"{"@context":"https://offline.example/context","name":"x"}"#,
            ),
            (3, block),
        ];
        let contexts = ContextCache::new();
        let claims = interpret(&contexts, notes, blocks);
        assert_eq!(claims, interpret(&contexts, notes, blocks));
        assert_eq!(
            claims
                .diagnostics
                .iter()
                .map(|row| row.order)
                .collect::<Vec<_>>(),
            [1, 2]
        );
        assert_eq!(
            claims
                .blocks
                .iter()
                .map(|row| row.order)
                .collect::<Vec<_>>(),
            [0, 3]
        );
        let focus = NamedOrBlankNode::from(NamedNode::new_unchecked(format!("{notes}#place")));
        let targets: Vec<_> = claims
            .blocks
            .iter()
            .map(|block| {
                block
                    .quads
                    .iter()
                    .find(|quad| quad.subject == focus)
                    .unwrap()
                    .object
                    .clone()
            })
            .collect();
        assert!(targets.iter().all(Term::is_blank_node));
        assert_eq!(targets[0], targets[1]);
    }
    #[test]
    fn shared_graph_names_and_references_join_but_anonymous_nodes_stay_local() {
        // The same label occurs as graph name, subject, and object. Each script
        // also creates an anonymous node with an identical description.
        let a = r#"{"@id":"_:river","@graph":[{"@id":"urn:survey:map",
            "urn:survey:water":{"@id":"_:river"},"urn:survey:sample":{"urn:survey:score":7}}]}"#;
        let b = r#"{"@id":"_:river","urn:survey:sample":{"urn:survey:score":7}}"#;
        let read = || {
            interpret(
                &ContextCache::new(),
                "https://example.test/",
                [(3, a), (9, b)],
            )
        };
        let page = read();
        assert!(page.diagnostics.is_empty());
        assert_eq!(page, read());
        let [first, second] = page.blocks.as_slice() else {
            panic!("expected two source blocks")
        };
        let water = first
            .quads
            .iter()
            .find(|q| q.predicate.as_str() == "urn:survey:water")
            .unwrap();
        let Term::BlankNode(river) = &water.object else {
            panic!("expected a blank node")
        };
        assert_eq!(water.graph_name, GraphName::from(river.clone()));
        let sample = |block: &PageClaimBlock| {
            block
                .quads
                .iter()
                .find(|q| q.predicate.as_str() == "urn:survey:sample")
                .unwrap()
                .clone()
        };
        assert_eq!(
            sample(second).subject,
            NamedOrBlankNode::from(river.clone())
        );
        assert_ne!(sample(first).object, sample(second).object);

        let capture = |id| KeptPageClaims::new(NamedNode::new_unchecked(id), &page);
        let x = capture("urn:capture:x");
        let y = capture("urn:capture:y");
        assert_ne!(
            sample(&x.claims.blocks[1]).subject,
            sample(&y.claims.blocks[1]).subject
        );
    }
}
