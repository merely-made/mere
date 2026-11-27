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
//! The alignment set is a deterministic constant (a function of the vocabulary,
//! not of any graph's data), emitted into the [`GRAPH_VOCABULARY`] named graph so
//! a consumer can query or ignore it independently of the instance data. Because
//! it is re-derivable, the file-I/O ingest path treats this graph as schema and
//! drops it ([`is_vocabulary_quad`]), keeping a round trip through a shared file
//! lossless.

use super::core::{self, ids};
use super::intern::TermId;
use oxrdf::{GraphName, NamedNode, Quad};

/// The named graph carrying Mere's vocabulary alignment. Kept out of the default
/// graph and the scope graphs so alignment (schema) never mixes with instance
/// facts in a consumer's default-graph query.
pub const GRAPH_VOCABULARY: &str = "https://mere.computer/ns/graph#vocabulary";

/// `owl:equivalentProperty` — links an *exact* Mere/standard term pair.
const OWL_EQUIVALENT_PROPERTY: &str = "http://www.w3.org/2002/07/owl#equivalentProperty";
/// `rdfs:subPropertyOf` — links an *approximate* (Mere-more-specific) term to its
/// standard superproperty.
const RDFS_SUB_PROPERTY_OF: &str = "http://www.w3.org/2000/01/rdf-schema#subPropertyOf";

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

/// The standard-vocabulary alignment for a recognized semantic sub-kind.
///
/// The exact/CiTO/OWL/PROV anchors are the compliance audit's
/// (`statements_over_schema_stance`); the remainder are categorized
/// conservatively — a relation is only mapped when a standard term genuinely is
/// the same as, or a superproperty of, the Mere term, and is left `MereOnly`
/// otherwise rather than forced onto an ill-fitting IRI. Keyed by the core
/// pack's generated ids (D21); a test holds every core semantic predicate to
/// a category, so a newly added one must be categorized here. `None` for a
/// predicate outside the core semantic family.
pub fn alignment(predicate: TermId) -> Option<Alignment> {
    use Alignment::{Approximate, Exact, MereOnly};

    // CiTO — the Citation Typing Ontology (`http://purl.org/spar/cito/`).
    const CITO_CITES: &str = "http://purl.org/spar/cito/cites";
    const CITO_INCLUDES_QUOTATION_FROM: &str = "http://purl.org/spar/cito/includesQuotationFrom";
    // The evidence sense, ruled 2026-09-15 from Knot's consumer audit: CiTO
    // separates agreeing with a statement (opinion) from supporting it with
    // evidence, and disagreeing from disputing it. A writer's relation is the
    // evidence sense. `cito:refutes` (disputing with evidence) is reached by a
    // consumer's narrower predicate, not by this table.
    const CITO_SUPPORTS: &str = "http://purl.org/spar/cito/supports";
    const CITO_DISPUTES: &str = "http://purl.org/spar/cito/disputes";
    const CITO_EXTENDS: &str = "http://purl.org/spar/cito/extends";
    const CITO_LINKS_TO: &str = "http://purl.org/spar/cito/linksTo";
    // OWL / Dublin Core Terms.
    const OWL_SAME_AS: &str = "http://www.w3.org/2002/07/owl#sameAs";
    const DCTERMS_REQUIRES: &str = "http://purl.org/dc/terms/requires";

    Some(match predicate {
        // Exact CiTO / OWL correspondences (stance-doc anchored; the four
        // 2026-09-15 rows are recorded in the petgraph-RDF plan's progress).
        ids::REL_CITES => Exact(CITO_CITES),
        ids::REL_QUOTES => Exact(CITO_INCLUDES_QUOTATION_FROM),
        ids::REL_SUPPORTS => Exact(CITO_SUPPORTS),
        ids::REL_CONTRADICTS => Exact(CITO_DISPUTES),
        ids::REL_ELABORATES => Exact(CITO_EXTENDS),
        // `linksTo` is defined as providing a URL link to the cited entity.
        ids::REL_HYPERLINK => Exact(CITO_LINKS_TO),
        ids::REL_SAME_ENTITY_AS => Exact(OWL_SAME_AS),

        // Approximate: the Mere relation is a narrower case of a standard term.
        // Summarizing / questioning a work both entail referencing it, so each
        // is a subproperty of `cito:cites`.
        ids::REL_SUMMARIZES | ids::REL_QUESTIONS => Approximate(CITO_CITES),
        // A dependency is a narrower "requires".
        ids::REL_DEPENDS_ON => Approximate(DCTERMS_REQUIRES),

        // Mere-only: no standard term is even a superproperty. Grouping and
        // agent-derivation are experience/provenance flavored; example-of and
        // the near-identity pair (duplicate / canonical-mirror) have no safe
        // standard superproperty (subPropertyOf owl:sameAs would over-claim
        // logical identity); Blocks / NextStep are workflow relations.
        ids::REL_USER_GROUPED
        | ids::REL_AGENT_DERIVED
        | ids::REL_EXAMPLE_OF
        | ids::REL_DUPLICATE_OF
        | ids::REL_CANONICAL_MIRROR_OF
        | ids::REL_BLOCKS
        | ids::REL_NEXT_STEP => MereOnly,
        _ => return None,
    })
}

/// The vocabulary-alignment quads: for every recognized semantic relation with a
/// standard correspondence, one quad linking its canonical `mere:rel#` IRI to a
/// standard term (`owl:equivalentProperty` for [`Alignment::Exact`],
/// `rdfs:subPropertyOf` for [`Alignment::Approximate`]), in the
/// [`GRAPH_VOCABULARY`] named graph. [`Alignment::MereOnly`] relations contribute
/// nothing. A deterministic constant, independent of any graph's data.
pub fn vocabulary_alignment_quads() -> Vec<Quad> {
    let Ok(graph) = NamedNode::new(GRAPH_VOCABULARY) else {
        return Vec::new();
    };
    let graph = GraphName::from(graph);
    let mut quads = Vec::new();
    for (_, id) in ids::TERMS {
        let (alignment_predicate, standard) = match alignment(id) {
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

    #[test]
    fn alignment_is_deterministic() {
        assert_eq!(vocabulary_alignment_quads(), vocabulary_alignment_quads());
    }
}
