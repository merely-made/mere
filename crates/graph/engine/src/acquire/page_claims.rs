// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! One independently interpreted JSON-LD block per document locator.

use oxrdf::{BlankNode, GraphName, NamedOrBlankNode, Quad, Term};

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
/// A failed block leaves the other blocks' claims available. Blank nodes have
/// stable identities scoped to the block within this page.
pub fn interpret<'a>(
    contexts: &ContextCache,
    base_iri: &str,
    blocks: impl IntoIterator<Item = (u64, &'a str)>,
) -> PageClaims {
    let mut result = PageClaims::default();
    for (order, block) in blocks {
        match interpret_block(contexts, base_iri, order, block) {
            Ok(quads) => result.blocks.push(PageClaimBlock { order, quads }),
            Err(message) => result
                .diagnostics
                .push(PageClaimDiagnostic { order, message }),
        }
    }
    result
}

fn interpret_block(
    contexts: &ContextCache,
    base_iri: &str,
    order: u64,
    block: &str,
) -> Result<Vec<Quad>, String> {
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
    let mut quads: Vec<_> = dataset.quads(View::Live).collect();
    if quads.iter().any(|quad| {
        quad.subject.is_blank_node()
            || quad.object.is_blank_node()
            || quad.graph_name.is_blank_node()
    }) {
        let canonical = canonicalize_quads(&quads, &CanonOptions::default())
            .map_err(|error| error.to_string())?;
        let scoped = |node: &BlankNode| {
            let label = &canonical.issued[node.as_str()];
            BlankNode::new_unchecked(format!("page_block_{order}_{label}"))
        };
        for quad in &mut quads {
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
    Ok(quads)
}

#[cfg(test)]
mod tests {
    use oxrdf::NamedNode;

    use super::*;

    #[test]
    fn failures_and_blank_node_labels_stay_with_their_blocks() {
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
        assert_ne!(targets[0], targets[1]);
    }
}
