// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Standard-vocabulary alignment for Mere's recognized relations (petgraph-RDF
//! plan, Phase 2's "standard-vocab mapping as a 3-category table").
//!
//! Mere projects its `Semantic` relations under canonical `mere:rel#` IRIs.
//! That keeps the kernel authoritative and the round trip byte-stable, but an
//! outside consumer has no way to know that `mere:rel#cites` *is* `cito:cites`.
//! This module publishes that knowledge as **alignment quads about the
//! properties themselves** — not by rewriting instance data, which is standard
//! linked-data practice: a vocabulary declares how its terms relate to others,
//! and a reasoner bridges the instances.
//!
//! Each core semantic predicate falls in one of three categories (the
//! compliance-audit cut in `2026-05-22_statements_over_schema_stance.md`):
//!
//! - **Exact** — the standard term means the same thing. Emitted as
//!   `mere:rel#x owl:equivalentProperty std:y` (a reasoner infers `std:y`
//!   wherever `mere:rel#x` appears, and vice versa). `Cites -> cito:cites`,
//!   `SameEntityAs -> owl:sameAs`.
//! - **Approximate** — the Mere term is *more specific* than a standard term.
//!   Emitted as `mere:rel#x rdfs:subPropertyOf std:y` (one-way: `mere:rel#x`
//!   entails `std:y`, not the reverse). `Summarizes -> cito:cites`,
//!   `DependsOn -> dcterms:requires`.
//! - **Mere-only** — no standard term is even a superproperty; the `mere:rel#`
//!   IRI stands alone. `Blocks`, `NextStep`, `UserGrouped`.
//!
//! The rows are schema triples in the engine-embedded alignment pack
//! (`alignment/pack.toml`, declared-lens D35). The alignment set is a
//! deterministic constant (a function of the vocabulary, not of any graph's
//! data), emitted into the [`GRAPH_VOCABULARY`] named graph so
//! a consumer can query or ignore it independently of the instance data. Because
//! it is re-derivable, the file-I/O ingest path treats this graph as schema and
//! drops it ([`is_vocabulary_quad`]), keeping a round trip through a shared file
//! lossless.

use std::sync::LazyLock;

use eidetic::pack::{OWL_EQUIVALENT_PROPERTY, PackHeader, RDFS_SUB_PROPERTY_OF, Triple};
use oxrdf::{GraphName, NamedNode, Quad};

use super::core::{self, ids};
use super::intern::TermId;

/// The named graph carrying Mere's vocabulary alignment. Kept out of the default
/// graph and the scope graphs so alignment (schema) never mixes with instance
/// facts in a consumer's default-graph query.
pub const GRAPH_VOCABULARY: &str = "https://mere.computer/ns/graph#vocabulary";

/// The alignment pack's id (declared-lens D35).
pub const ALIGNMENT_PACK_ID: &str = "https://mere.computer/packs/alignment";

/// The alignment pack's canonical CBOR, compiled by build.rs from
/// `alignment/pack.toml` and embedded beside the core pack (D31 (3)).
pub static ALIGNMENT_PACK: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/alignment_pack.cbor"));

static TRIPLES: LazyLock<Vec<Triple>> =
    LazyLock::new(|| triples_of(ALIGNMENT_PACK).expect("the embedded alignment pack reads"));

/// The schema triples a pack's canonical bytes carry.
fn triples_of(canonical: &[u8]) -> Result<Vec<Triple>, String> {
    let header = PackHeader::from_canonical(canonical).map_err(|e| e.to_string())?;
    let body = header.vocabulary().map_err(|e| e.to_string())?;
    Ok(body.and_then(|v| v.triples).unwrap_or_default())
}

/// How a recognized Mere relation aligns to a standard-vocabulary term.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Alignment {
    /// Same meaning as the standard term (`owl:equivalentProperty`).
    Exact(&'static str),
    /// More specific than the standard term (`rdfs:subPropertyOf`).
    Approximate(&'static str),
    /// No standard superproperty; the `mere:rel#` IRI stands alone.
    MereOnly,
}

/// The standard-vocabulary alignment for a core semantic predicate, as the
/// alignment pack's triples state it (D35): `owl:equivalentProperty` is
/// [`Alignment::Exact`], `rdfs:subPropertyOf` [`Alignment::Approximate`], and
/// a semantic predicate with no triple [`Alignment::MereOnly`]. `None` for a
/// predicate outside the core semantic family.
pub fn alignment(predicate: TermId) -> Option<Alignment> {
    alignment_in(&TRIPLES, predicate)
}

fn alignment_in(triples: &'static [Triple], predicate: TermId) -> Option<Alignment> {
    let terms = core::terms();
    if terms.family_of(predicate) != Some(ids::FAMILY_SEMANTIC) {
        return None;
    }
    let iri = terms.iri(predicate)?;
    Some(
        triples
            .iter()
            .find(|t| t.subject == iri)
            .map_or(Alignment::MereOnly, |t| match t.predicate.as_str() {
                OWL_EQUIVALENT_PROPERTY => Alignment::Exact(&t.object),
                _ => Alignment::Approximate(&t.object),
            }),
    )
}

/// The vocabulary-alignment quads: for every recognized semantic relation with a
/// standard correspondence, one quad linking its canonical `mere:rel#` IRI to a
/// standard term (`owl:equivalentProperty` for [`Alignment::Exact`],
/// `rdfs:subPropertyOf` for [`Alignment::Approximate`]), in the
/// [`GRAPH_VOCABULARY`] named graph, in core id order. [`Alignment::MereOnly`]
/// relations contribute nothing. A deterministic constant, independent of any
/// graph's data.
pub fn vocabulary_alignment_quads() -> Vec<Quad> {
    quads_in(&TRIPLES)
}

fn quads_in(triples: &'static [Triple]) -> Vec<Quad> {
    let Ok(graph) = NamedNode::new(GRAPH_VOCABULARY) else {
        return Vec::new();
    };
    let graph = GraphName::from(graph);
    let mut quads = Vec::new();
    for (_, id) in ids::TERMS {
        let (alignment_predicate, standard) = match alignment_in(triples, id) {
            Some(Alignment::Exact(iri)) => (OWL_EQUIVALENT_PROPERTY, iri),
            Some(Alignment::Approximate(iri)) => (RDFS_SUB_PROPERTY_OF, iri),
            Some(Alignment::MereOnly) | None => continue,
        };
        if let (Ok(subject), Ok(predicate), Ok(object)) = (
            NamedNode::new(core::iri(id)),
            NamedNode::new(alignment_predicate),
            NamedNode::new(standard),
        ) {
            quads.push(Quad::new(subject, predicate, object, graph.clone()));
        }
    }
    quads
}

/// Whether a quad belongs to the vocabulary-alignment graph. The file-I/O ingest
/// path drops these (they are re-derivable schema, not instance content) so a
/// round trip through a shared N-Quads / TriG file stays lossless.
pub fn is_vocabulary_quad(quad: &Quad) -> bool {
    matches!(&quad.graph_name, GraphName::NamedNode(g) if g.as_str() == GRAPH_VOCABULARY)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn semantic_predicates() -> impl Iterator<Item = TermId> {
        ids::TERMS
            .into_iter()
            .map(|(_, id)| id)
            .filter(|id| core::terms().family_of(*id) == Some(ids::FAMILY_SEMANTIC))
    }

    #[test]
    fn every_core_semantic_predicate_is_categorized_and_maps_to_a_valid_iri() {
        assert_eq!(semantic_predicates().count(), 17);
        for id in semantic_predicates() {
            match alignment(id) {
                Some(Alignment::Exact(iri) | Alignment::Approximate(iri)) => {
                    assert!(NamedNode::new(iri).is_ok(), "{id:?} maps to a valid IRI");
                },
                Some(Alignment::MereOnly) => {},
                None => panic!("{} is not categorized", core::iri(id)),
            }
        }
        assert_eq!(alignment(ids::REL_CONTAINMENT_USER_FOLDER), None);
    }

    #[test]
    fn alignment_quads_use_the_right_predicate_per_category() {
        let quads = vocabulary_alignment_quads();
        let quad_of = |id: TermId| {
            let subject = format!("<{}>", core::iri(id));
            quads.iter().find(move |q| q.subject.to_string() == subject)
        };
        let cites_quad = quad_of(ids::REL_CITES).expect("cites has an alignment quad");
        assert_eq!(cites_quad.predicate.as_str(), OWL_EQUIVALENT_PROPERTY);
        assert_eq!(
            cites_quad.object.to_string(),
            "<http://purl.org/spar/cito/cites>"
        );
        assert!(
            is_vocabulary_quad(cites_quad),
            "alignment lives in the vocab graph"
        );
        // The 2026-09-15 evidence-sense rows are exact.
        for (id, iri) in [
            (ids::REL_SUPPORTS, "<http://purl.org/spar/cito/supports>"),
            (ids::REL_CONTRADICTS, "<http://purl.org/spar/cito/disputes>"),
            (ids::REL_ELABORATES, "<http://purl.org/spar/cito/extends>"),
            (ids::REL_HYPERLINK, "<http://purl.org/spar/cito/linksTo>"),
        ] {
            let quad = quad_of(id).unwrap_or_else(|| panic!("{id:?} has an alignment quad"));
            assert_eq!(quad.predicate.as_str(), OWL_EQUIVALENT_PROPERTY);
            assert_eq!(quad.object.to_string(), iri);
        }
        let summarizes = quad_of(ids::REL_SUMMARIZES).expect("summarizes is aligned");
        assert_eq!(summarizes.predicate.as_str(), RDFS_SUB_PROPERTY_OF);
        // A Mere-only relation contributes no alignment quad.
        assert!(quad_of(ids::REL_BLOCKS).is_none(), "Blocks is Mere-only");
        assert_eq!(quads.len(), 10);
    }

    /// The quads every export carries today, pinned so that moving the rows
    /// into an embedded alignment pack (declared-lens D31 (3)) exports the
    /// same ones.
    #[test]
    fn the_exported_alignment_quads_are_pinned() {
        const REL: &str = "https://mere.computer/ns/rel#";
        const CITO: &str = "http://purl.org/spar/cito/";
        let expected = [
            ("cites", OWL_EQUIVALENT_PROPERTY, format!("{CITO}cites")),
            (
                "quotes",
                OWL_EQUIVALENT_PROPERTY,
                format!("{CITO}includesQuotationFrom"),
            ),
            (
                "supports",
                OWL_EQUIVALENT_PROPERTY,
                format!("{CITO}supports"),
            ),
            (
                "contradicts",
                OWL_EQUIVALENT_PROPERTY,
                format!("{CITO}disputes"),
            ),
            (
                "elaborates",
                OWL_EQUIVALENT_PROPERTY,
                format!("{CITO}extends"),
            ),
            (
                "hyperlink",
                OWL_EQUIVALENT_PROPERTY,
                format!("{CITO}linksTo"),
            ),
            (
                "same-entity-as",
                OWL_EQUIVALENT_PROPERTY,
                "http://www.w3.org/2002/07/owl#sameAs".to_string(),
            ),
            ("summarizes", RDFS_SUB_PROPERTY_OF, format!("{CITO}cites")),
            ("questions", RDFS_SUB_PROPERTY_OF, format!("{CITO}cites")),
            (
                "depends-on",
                RDFS_SUB_PROPERTY_OF,
                "http://purl.org/dc/terms/requires".to_string(),
            ),
        ];
        let mut want: Vec<String> = expected
            .iter()
            .map(|(local, p, o)| format!("<{REL}{local}> <{p}> <{o}> <{GRAPH_VOCABULARY}> ."))
            .collect();
        want.sort();
        let mut got: Vec<String> = vocabulary_alignment_quads()
            .iter()
            .map(|q| format!("{q} ."))
            .collect();
        got.sort();
        assert_eq!(got, want);
    }

    /// Break and restore (D35): a triple dropped from the pack is a quad
    /// dropped from every export.
    #[test]
    fn dropping_a_triple_changes_the_exported_quads() {
        let text = include_str!("alignment/pack.toml").replace("\r\n", "\n");
        let hyperlink = "[[vocabulary.triples]]\nsubject = \"rel:hyperlink\"\npredicate = \"owl:equivalentProperty\"\nobject = \"cito:linksTo\"\n";
        assert_eq!(text.matches(hyperlink).count(), 1);
        let compile = |text: &str| {
            let (_, bytes) = eidetic::pack::compile_authored(text).unwrap();
            let triples: &'static [Triple] =
                Box::leak(triples_of(&bytes).unwrap().into_boxed_slice());
            quads_in(triples)
        };
        assert_eq!(
            compile(&text),
            vocabulary_alignment_quads(),
            "control: the pack as embedded"
        );
        let dropped = compile(&text.replace(hyperlink, ""));
        assert_eq!(dropped.len(), 9);
        assert_ne!(dropped, vocabulary_alignment_quads());
        assert!(
            !dropped
                .iter()
                .any(|q| q.subject.to_string() == format!("<{}>", core::iri(ids::REL_HYPERLINK)))
        );
    }

    #[test]
    fn every_triple_is_about_a_core_semantic_predicate() {
        let terms = core::terms();
        for t in TRIPLES.iter() {
            let id = terms
                .id(&t.subject)
                .unwrap_or_else(|| panic!("{} is not core", t.subject));
            assert_eq!(
                terms.family_of(id),
                Some(ids::FAMILY_SEMANTIC),
                "{}",
                t.subject
            );
        }
        assert_eq!(TRIPLES.len(), 10);
    }

    #[test]
    fn alignment_is_deterministic() {
        assert_eq!(vocabulary_alignment_quads(), vocabulary_alignment_quads());
    }
}
