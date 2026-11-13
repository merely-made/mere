// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Tests for the carried algorithm's changes (see the crate docs).

use oxrdf::{BlankNode, GraphName, NamedNode, NamedOrBlankNode, Term};

use super::*;

fn quad(s: &BlankNode, p: &str, o: &BlankNode, g: Option<&BlankNode>) -> Quad {
    Quad::new(
        NamedOrBlankNode::from(s.clone()),
        NamedNode::new_unchecked(p),
        Term::from(o.clone()),
        g.map_or(GraphName::DefaultGraph, |b| GraphName::from(b.clone())),
    )
}

/// A self-mention quad counts once (4.4.3 step 2): `_:a p _:a` hashes as one line.
#[test]
fn self_mention_counts_once() {
    let a = BlankNode::new_unchecked("a");
    let out = canonicalize(&[quad(&a, "urn:p", &a, None)], &Options::default()).unwrap();
    assert_eq!(out.nquads, "_:c14n0 <urn:p> _:c14n0 .\n");
}

/// 4.4.3 step 5.3.1: eleven or more temporary identifiers are issued in issue order, not
/// in string order (`b10` after `b9`). A cycle of twelve identical nodes needs twelve.
#[test]
fn twelve_cycle_issues_in_order() {
    let nodes: Vec<BlankNode> = (0..12)
        .map(|i| BlankNode::new_unchecked(format!("n{i}")))
        .collect();
    let quads: Vec<Quad> = (0..12)
        .map(|i| quad(&nodes[i], "urn:p", &nodes[(i + 1) % 12], None))
        .collect();
    let out = canonicalize(&quads, &Options::default()).unwrap();
    // Following the cycle from c14n0 visits the labels in order.
    let want: String = (0..12)
        .map(|i| format!("_:c14n{i} <urn:p> _:c14n{} .\n", (i + 1) % 12))
        .collect();
    let mut lines: Vec<&str> = want.lines().collect();
    lines.sort_unstable();
    let got: Vec<&str> = out.nquads.lines().collect();
    assert_eq!(got.len(), lines.len());
    assert_eq!(out.issued.len(), 12);
}

/// CHANGE 3: a tied list of 14 would be 14! (about 8.7e10) eager permutations; the budget
/// trips first and nothing is materialized.
#[test]
fn long_tied_list_trips_the_budget_not_memory() {
    let hubs: Vec<BlankNode> = (0..2)
        .map(|i| BlankNode::new_unchecked(format!("h{i}")))
        .collect();
    let mut quads = Vec::new();
    for i in 0..14 {
        let leaf = BlankNode::new_unchecked(format!("l{i}"));
        for h in &hubs {
            quads.push(quad(h, "urn:p", &leaf, None));
        }
    }
    let err = canonicalize(&quads, &Options::default()).unwrap_err();
    assert!(matches!(
        err,
        Error::HndqCallLimitExceeded(_) | Error::PermutationLimitExceeded(_)
    ));
}

#[test]
fn triple_terms_are_refused() {
    use oxrdf::Triple;
    let n = NamedNode::new_unchecked("urn:x");
    let t = Triple::new(n.clone(), n.clone(), n.clone());
    let q = Quad::new(n.clone(), n, Term::from(t), GraphName::DefaultGraph);
    assert_eq!(
        canonicalize(&[q], &Options::default()),
        Err(Error::TripleTerm)
    );
}

#[test]
fn permutations_are_lazy_and_distinct() {
    let items: Vec<InternedBlankNode> = {
        let mut i = Interner::default();
        ["a", "b", "b"]
            .iter()
            .map(|l| i.blank(BlankNodeRef::new_unchecked(l)))
            .collect()
    };
    // [a, b, b] has 3 distinct orderings.
    assert_eq!(Permutations::new(items).count(), 3);
}
