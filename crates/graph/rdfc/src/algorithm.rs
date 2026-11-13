// Copyright 2026 Mark Alan Boykin (changes)
// Copyright Tpt <thomas@pellissier-tanon.fr> (the carried algorithm)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The RDFC-1.0 algorithm, carried from oxrdf 0.3.5 `src/dataset.rs` (see the crate docs for the
//! line ranges and the numbered changes). Spec: <https://www.w3.org/TR/rdf-canon/>.

use std::collections::{BTreeMap, BTreeSet, HashMap};

use oxrdf::{BlankNodeRef, GraphNameRef, NamedOrBlankNodeRef, Quad, TermRef};
use sha2::{Digest, Sha256, Sha384};

use crate::intern::{
    EncodedQuad, InternedBlankNode, InternedGraphName, InternedNamedOrBlankNode, InternedTerm,
    Interner,
};
use crate::{Canonicalized, Error, HashAlgorithm, Options};

/// CHANGE 2: the work budget (Security Considerations, "Dataset Poisoning").
struct Budget {
    calls: usize,
    permutations: usize,
    max_calls: Option<usize>,
    max_permutations: Option<usize>,
}

impl Budget {
    fn call(&mut self) -> Result<(), Error> {
        self.calls += 1;
        match self.max_calls {
            Some(limit) if self.calls > limit => Err(Error::HndqCallLimitExceeded(limit)),
            _ => Ok(()),
        }
    }

    fn permutation(&mut self) -> Result<(), Error> {
        self.permutations += 1;
        match self.max_permutations {
            Some(limit) if self.permutations > limit => Err(Error::PermutationLimitExceeded(limit)),
            _ => Ok(()),
        }
    }
}

/// A RDFC identifier issuer (4.3).
#[derive(Clone)]
struct IdentifierIssuer {
    identifier_prefix: &'static str,
    identifier_counter: u32,
    issued_identifier_map: HashMap<InternedBlankNode, String>,
    /// The order of issue (the spec's "in the same order", 4.4.3 step 5.3.1).
    issued_identifier_order: Vec<InternedBlankNode>,
}

impl IdentifierIssuer {
    fn new(identifier_prefix: &'static str) -> Self {
        Self {
            identifier_prefix,
            identifier_counter: 0,
            issued_identifier_map: HashMap::new(),
            issued_identifier_order: Vec::new(),
        }
    }

    /// 4.5 Issue Identifier.
    fn issue(&mut self, blank_node: InternedBlankNode) -> String {
        // 1)
        if let Some(id) = self.issued_identifier_map.get(&blank_node) {
            return id.clone();
        }
        // 2)
        let issued = format!("{}{}", self.identifier_prefix, self.identifier_counter);
        // 3)
        self.issued_identifier_map
            .insert(blank_node, issued.clone());
        self.issued_identifier_order.push(blank_node);
        // 4)
        self.identifier_counter += 1;
        // 5)
        issued
    }
}

/// CHANGE 3: permutations are produced one at a time (lexicographic order over the values), so a
/// long list is never materialized. A list with repeated blank nodes (CHANGE 1) yields each
/// distinct ordering once; a repeat would give the same path and issuer.
struct Permutations {
    items: Vec<InternedBlankNode>,
    started: bool,
}

impl Permutations {
    fn new(mut items: Vec<InternedBlankNode>) -> Self {
        items.sort_unstable();
        Self {
            items,
            started: false,
        }
    }
}

impl Iterator for Permutations {
    type Item = Vec<InternedBlankNode>;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.started {
            self.started = true;
            return Some(self.items.clone());
        }
        let a = &mut self.items;
        let i = (0..a.len().checked_sub(1)?)
            .rev()
            .find(|&i| a[i] < a[i + 1])?;
        let j = (i + 1..a.len()).rev().find(|&j| a[j] > a[i])?;
        a.swap(i, j);
        a[i + 1..].reverse();
        Some(a.clone())
    }
}

struct Canonicalizer {
    interner: Interner,
    quads: Vec<EncodedQuad>,
    hash: HashAlgorithm,
    /// 4.4.1 blank node to quads map (indices into `quads`), in blank node id order.
    blank_node_to_quads: BTreeMap<InternedBlankNode, Vec<usize>>,
    /// 4.4.1 hash to blank nodes map.
    hash_to_blank_nodes: BTreeMap<String, Vec<InternedBlankNode>>,
    canonical_issuer: IdentifierIssuer,
}

pub(crate) fn canonicalize(quads: &[Quad], options: &Options) -> Result<Canonicalized, Error> {
    let mut interner = Interner::default();
    let mut set = BTreeSet::new();
    for q in quads {
        set.insert((
            interner.subject(q.subject.as_ref()),
            interner.named(q.predicate.as_ref()),
            interner.term(q.object.as_ref())?,
            interner.graph(q.graph_name.as_ref()),
        ));
    }
    let mut state = Canonicalizer {
        interner,
        quads: set.into_iter().collect(),
        hash: options.hash,
        blank_node_to_quads: BTreeMap::new(),
        hash_to_blank_nodes: BTreeMap::new(),
        canonical_issuer: IdentifierIssuer::new("c14n"),
    };
    let mut budget = Budget {
        calls: 0,
        permutations: 0,
        max_calls: options.max_hndq_calls,
        max_permutations: options.max_permutations,
    };
    state.run(&mut budget)?;
    Ok(state.output(&budget))
}

impl Canonicalizer {
    /// 4.4.3 Canonicalization Algorithm, steps 2 to 5.
    fn run(&mut self, budget: &mut Budget) -> Result<(), Error> {
        // 2) For each blank node that is a component of Q, add a reference to Q from the entry for
        // the blank node: a set of quads per blank node ("the set of quads of which it is a
        // component"), so a quad naming one blank node twice is added once.
        for (index, quad) in self.quads.iter().enumerate() {
            let components = [
                match quad.0 {
                    InternedNamedOrBlankNode::BlankNode(b) => Some(b),
                    InternedNamedOrBlankNode::NamedNode(_) => None,
                },
                match quad.2 {
                    InternedTerm::BlankNode(b) => Some(b),
                    _ => None,
                },
                match quad.3 {
                    InternedGraphName::BlankNode(b) => Some(b),
                    _ => None,
                },
            ];
            for b in components.into_iter().flatten() {
                let entry = self.blank_node_to_quads.entry(b).or_default();
                if entry.last() != Some(&index) {
                    entry.push(index);
                }
            }
        }
        // 3)
        let blank_nodes: Vec<InternedBlankNode> =
            self.blank_node_to_quads.keys().copied().collect();
        for n in blank_nodes {
            // 3.1)
            let hash = self.hash_first_degree_quads(n);
            // 3.2)
            self.hash_to_blank_nodes.entry(hash).or_default().push(n);
        }
        // 4)
        let singles: Vec<(String, InternedBlankNode)> = self
            .hash_to_blank_nodes
            .iter()
            .filter(|(_, list)| list.len() == 1)
            .map(|(h, list)| (h.clone(), list[0]))
            .collect();
        for (hash, n) in singles {
            // 4.2)
            self.canonical_issuer.issue(n);
            // 4.3)
            self.hash_to_blank_nodes.remove(&hash);
        }
        // 5)
        let remaining = std::mem::take(&mut self.hash_to_blank_nodes);
        for (_, identifier_list) in remaining {
            // 5.1)
            let mut hash_path_list = Vec::new();
            // 5.2)
            for n in identifier_list {
                // 5.2.1)
                if self.canonical_issuer.issued_identifier_map.contains_key(&n) {
                    continue;
                }
                // 5.2.2)
                let mut temporary_issuer = IdentifierIssuer::new("b");
                // 5.2.3)
                temporary_issuer.issue(n);
                // 5.2.4)
                hash_path_list.push(self.hash_n_degree_quads(n, &temporary_issuer, budget)?);
            }
            // 5.3)
            hash_path_list.sort_by(|(_, l), (_, r)| l.cmp(r));
            for (result_issuer, _) in hash_path_list {
                // 5.3.1) in the order the temporary identifiers were issued
                for existing in result_issuer.issued_identifier_order {
                    self.canonical_issuer.issue(existing);
                }
            }
        }
        Ok(())
    }

    /// 4.5 serialization of the relabeled dataset, and the issued map.
    fn output(&self, budget: &Budget) -> Canonicalized {
        let label = |b: InternedBlankNode| -> &str {
            self.canonical_issuer
                .issued_identifier_map
                .get(&b)
                .map_or("", String::as_str)
        };
        let mut lines: Vec<String> = self
            .quads
            .iter()
            .map(|(s, p, o, g)| {
                let subject = match s {
                    InternedNamedOrBlankNode::NamedNode(n) => {
                        NamedOrBlankNodeRef::NamedNode(self.interner.named_ref(*n))
                    },
                    InternedNamedOrBlankNode::BlankNode(b) => {
                        NamedOrBlankNodeRef::BlankNode(BlankNodeRef::new_unchecked(label(*b)))
                    },
                };
                let predicate = self.interner.named_ref(*p);
                let object = match o {
                    InternedTerm::NamedNode(n) => TermRef::NamedNode(self.interner.named_ref(*n)),
                    InternedTerm::BlankNode(b) => {
                        TermRef::BlankNode(BlankNodeRef::new_unchecked(label(*b)))
                    },
                    InternedTerm::Literal(l) => TermRef::Literal(self.interner.literal_ref(*l)),
                };
                match g {
                    InternedGraphName::DefaultGraph => {
                        format!("{subject} {predicate} {object} .\n")
                    },
                    InternedGraphName::NamedNode(n) => {
                        let g = GraphNameRef::NamedNode(self.interner.named_ref(*n));
                        format!("{subject} {predicate} {object} {g} .\n")
                    },
                    InternedGraphName::BlankNode(b) => {
                        let g = GraphNameRef::BlankNode(BlankNodeRef::new_unchecked(label(*b)));
                        format!("{subject} {predicate} {object} {g} .\n")
                    },
                }
            })
            .collect();
        lines.sort_unstable();
        let issued = self
            .canonical_issuer
            .issued_identifier_map
            .iter()
            .map(|(b, c)| (self.interner.blank_ref(*b).as_str().to_owned(), c.clone()))
            .collect();
        Canonicalized {
            nquads: lines.concat(),
            issued,
            hndq_calls: budget.calls,
            permutations: budget.permutations,
        }
    }

    /// 4.6 Hash First Degree Quads.
    fn hash_first_degree_quads(&self, reference: InternedBlankNode) -> String {
        // 1)
        let mut nquads = Vec::new();
        // 2)
        let quads = &self.blank_node_to_quads[&reference];
        // 3)
        for &index in quads {
            let (s, p, o, g) = &self.quads[index];
            // 3.1) a blank node is `a` when it is the reference, else `z`
            let special = |b: InternedBlankNode| {
                BlankNodeRef::new_unchecked(if b == reference { "a" } else { "z" })
            };
            let subject = match s {
                InternedNamedOrBlankNode::NamedNode(n) => {
                    NamedOrBlankNodeRef::NamedNode(self.interner.named_ref(*n))
                },
                InternedNamedOrBlankNode::BlankNode(b) => {
                    NamedOrBlankNodeRef::BlankNode(special(*b))
                },
            };
            let predicate = self.interner.named_ref(*p);
            let object = match o {
                InternedTerm::NamedNode(n) => TermRef::NamedNode(self.interner.named_ref(*n)),
                InternedTerm::BlankNode(b) => TermRef::BlankNode(special(*b)),
                InternedTerm::Literal(l) => TermRef::Literal(self.interner.literal_ref(*l)),
            };
            nquads.push(match g {
                InternedGraphName::DefaultGraph => format!("{subject} {predicate} {object} .\n"),
                InternedGraphName::NamedNode(n) => {
                    let g = GraphNameRef::NamedNode(self.interner.named_ref(*n));
                    format!("{subject} {predicate} {object} {g} .\n")
                },
                InternedGraphName::BlankNode(b) => {
                    let g = GraphNameRef::BlankNode(special(*b));
                    format!("{subject} {predicate} {object} {g} .\n")
                },
            });
        }
        // 3) sorted in code point order
        nquads.sort();
        // 4)
        self.hash_function(&nquads.concat())
    }

    /// 4.7 Hash Related Blank Node.
    fn hash_related_blank_node(
        &self,
        related: InternedBlankNode,
        predicate: &str,
        issuer: &IdentifierIssuer,
        position: &str,
    ) -> String {
        // 1)
        let mut input = position.to_owned();
        // 2)
        if position != "g" {
            input.push('<');
            input.push_str(predicate);
            input.push('>');
        }
        // 3)
        if let Some(id) = self
            .canonical_issuer
            .issued_identifier_map
            .get(&related)
            .or_else(|| issuer.issued_identifier_map.get(&related))
        {
            input.push_str("_:");
            input.push_str(id);
        } else {
            // 4)
            input.push_str(&self.hash_first_degree_quads(related));
        }
        // 5)
        self.hash_function(&input)
    }

    /// 4.8 Hash N-Degree Quads.
    fn hash_n_degree_quads(
        &self,
        identifier: InternedBlankNode,
        issuer: &IdentifierIssuer,
        budget: &mut Budget,
    ) -> Result<(IdentifierIssuer, String), Error> {
        // CHANGE 2: one call to this algorithm.
        budget.call()?;
        let mut issuer = issuer.clone();
        // 1) CHANGE 1: a list per related hash that keeps every mention (step 3.1.2 adds "a
        // mapping of hash to the blank node identifier", and 4.8's overview records the identifier
        // "once for every mention"); upstream used a set.
        let mut h_n = BTreeMap::<String, Vec<InternedBlankNode>>::new();
        // 2)
        let quads = &self.blank_node_to_quads[&identifier];
        // 3)
        for &index in quads {
            let (s, p, o, g) = &self.quads[index];
            let predicate = self.interner.named_ref(*p).as_str();
            // 3.1) each blank node component that is not `identifier`
            let components = [
                (
                    match s {
                        InternedNamedOrBlankNode::BlankNode(b) => Some(*b),
                        InternedNamedOrBlankNode::NamedNode(_) => None,
                    },
                    "s",
                ),
                (
                    match o {
                        InternedTerm::BlankNode(b) => Some(*b),
                        _ => None,
                    },
                    "o",
                ),
                (
                    match g {
                        InternedGraphName::BlankNode(b) => Some(*b),
                        _ => None,
                    },
                    "g",
                ),
            ];
            for (component, position) in components {
                let Some(component) = component.filter(|c| *c != identifier) else {
                    continue;
                };
                // 3.1.1)
                let hash = self.hash_related_blank_node(component, predicate, &issuer, position);
                // 3.1.2)
                h_n.entry(hash).or_default().push(component);
            }
        }
        // 4)
        let mut data_to_hash = String::new();
        // 5)
        for (related_hash, blank_node_list) in h_n {
            // 5.1)
            data_to_hash.push_str(&related_hash);
            // 5.2)
            let mut chosen_path = String::new();
            // 5.3)
            let mut chosen_issuer = IdentifierIssuer::new("");
            // 5.4)
            'perm: for p in Permutations::new(blank_node_list) {
                // CHANGE 2: one permutation considered.
                budget.permutation()?;
                // 5.4.1)
                let mut issuer_copy = issuer.clone();
                // 5.4.2)
                let mut path = String::new();
                // 5.4.3)
                let mut recursion_list = Vec::new();
                // 5.4.4)
                for related in p {
                    // 5.4.4.1)
                    if let Some(id) = self.canonical_issuer.issued_identifier_map.get(&related) {
                        path.push_str("_:");
                        path.push_str(id);
                    } else {
                        // 5.4.4.2)
                        // 5.4.4.2.1)
                        if !issuer_copy.issued_identifier_map.contains_key(&related) {
                            recursion_list.push(related);
                        }
                        // 5.4.4.2.2)
                        let id = issuer_copy.issue(related);
                        path.push_str("_:");
                        path.push_str(&id);
                    }
                    // 5.4.4.3)
                    if !chosen_path.is_empty()
                        && path.len() >= chosen_path.len()
                        && path > chosen_path
                    {
                        continue 'perm;
                    }
                }
                // 5.4.5)
                for related in recursion_list {
                    // 5.4.5.1)
                    let (result_issuer, result_hash) =
                        self.hash_n_degree_quads(related, &issuer_copy, budget)?;
                    // 5.4.5.2)
                    let id = issuer_copy.issue(related);
                    path.push_str("_:");
                    path.push_str(&id);
                    // 5.4.5.3)
                    path.push('<');
                    path.push_str(&result_hash);
                    path.push('>');
                    // 5.4.5.4)
                    issuer_copy = result_issuer;
                    // 5.4.5.5)
                    if !chosen_path.is_empty()
                        && path.len() >= chosen_path.len()
                        && path > chosen_path
                    {
                        continue 'perm;
                    }
                }
                // 5.4.6)
                if chosen_path.is_empty() || path < chosen_path {
                    chosen_path = path;
                    chosen_issuer = issuer_copy;
                }
            }
            // 5.5)
            data_to_hash.push_str(&chosen_path);
            // 5.6)
            issuer = chosen_issuer;
        }
        // 6)
        Ok((issuer, self.hash_function(&data_to_hash)))
    }

    /// The hash function of 4.4 (CHANGE 7: `sha2` 0.11, hex written here).
    fn hash_function(&self, input: &str) -> String {
        let bytes: Vec<u8> = match self.hash {
            HashAlgorithm::Sha256 => Sha256::digest(input.as_bytes()).to_vec(),
            HashAlgorithm::Sha384 => Sha384::digest(input.as_bytes()).to_vec(),
        };
        bytes.iter().map(|b| format!("{b:02x}")).collect()
    }
}

#[cfg(test)]
mod tests;
