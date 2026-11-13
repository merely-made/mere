// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

//! A table interner for the terms of one canonicalization (CHANGE 4: replaces oxrdf's
//! `src/interning.rs`). Ids are dense and assigned in first-seen order, so iteration over the
//! encoded quads is deterministic. Triple terms have no id: the entry refuses them first.

use std::collections::HashMap;

use oxrdf::{
    BlankNode, BlankNodeRef, GraphNameRef, Literal, LiteralRef, NamedNode, NamedNodeRef,
    NamedOrBlankNodeRef, TermRef,
};

use crate::Error;

macro_rules! id {
    ($name:ident) => {
        #[derive(Eq, PartialEq, Ord, PartialOrd, Debug, Clone, Copy, Hash)]
        pub(crate) struct $name(u32);
    };
}

id!(InternedNamedNode);
id!(InternedBlankNode);
id!(InternedLiteral);

#[derive(Eq, PartialEq, Ord, PartialOrd, Debug, Clone, Copy, Hash)]
pub(crate) enum InternedNamedOrBlankNode {
    NamedNode(InternedNamedNode),
    BlankNode(InternedBlankNode),
}

#[derive(Eq, PartialEq, Ord, PartialOrd, Debug, Clone, Copy, Hash)]
pub(crate) enum InternedTerm {
    NamedNode(InternedNamedNode),
    BlankNode(InternedBlankNode),
    Literal(InternedLiteral),
}

#[derive(Eq, PartialEq, Ord, PartialOrd, Debug, Clone, Copy, Hash)]
pub(crate) enum InternedGraphName {
    NamedNode(InternedNamedNode),
    BlankNode(InternedBlankNode),
    DefaultGraph,
}

/// One encoded quad, in subject, predicate, object, graph order.
pub(crate) type EncodedQuad = (
    InternedNamedOrBlankNode,
    InternedNamedNode,
    InternedTerm,
    InternedGraphName,
);

#[derive(Default)]
pub(crate) struct Interner {
    named: Vec<NamedNode>,
    named_ids: HashMap<NamedNode, u32>,
    blank: Vec<BlankNode>,
    blank_ids: HashMap<BlankNode, u32>,
    literals: Vec<Literal>,
    literal_ids: HashMap<Literal, u32>,
}

fn intern<T: Clone + Eq + std::hash::Hash>(
    items: &mut Vec<T>,
    ids: &mut HashMap<T, u32>,
    value: &T,
) -> u32 {
    if let Some(id) = ids.get(value) {
        return *id;
    }
    let id = u32::try_from(items.len()).unwrap_or(u32::MAX);
    items.push(value.clone());
    ids.insert(value.clone(), id);
    id
}

impl Interner {
    pub(crate) fn named(&mut self, n: NamedNodeRef<'_>) -> InternedNamedNode {
        InternedNamedNode(intern(
            &mut self.named,
            &mut self.named_ids,
            &n.into_owned(),
        ))
    }

    pub(crate) fn blank(&mut self, b: BlankNodeRef<'_>) -> InternedBlankNode {
        InternedBlankNode(intern(
            &mut self.blank,
            &mut self.blank_ids,
            &b.into_owned(),
        ))
    }

    fn literal(&mut self, l: LiteralRef<'_>) -> InternedLiteral {
        InternedLiteral(intern(
            &mut self.literals,
            &mut self.literal_ids,
            &l.into_owned(),
        ))
    }

    pub(crate) fn subject(&mut self, s: NamedOrBlankNodeRef<'_>) -> InternedNamedOrBlankNode {
        match s {
            NamedOrBlankNodeRef::NamedNode(n) => InternedNamedOrBlankNode::NamedNode(self.named(n)),
            NamedOrBlankNodeRef::BlankNode(b) => InternedNamedOrBlankNode::BlankNode(self.blank(b)),
        }
    }

    pub(crate) fn term(&mut self, t: TermRef<'_>) -> Result<InternedTerm, Error> {
        Ok(match t {
            TermRef::NamedNode(n) => InternedTerm::NamedNode(self.named(n)),
            TermRef::BlankNode(b) => InternedTerm::BlankNode(self.blank(b)),
            TermRef::Literal(l) => InternedTerm::Literal(self.literal(l)),
            TermRef::Triple(_) => return Err(Error::TripleTerm),
        })
    }

    pub(crate) fn graph(&mut self, g: GraphNameRef<'_>) -> InternedGraphName {
        match g {
            GraphNameRef::NamedNode(n) => InternedGraphName::NamedNode(self.named(n)),
            GraphNameRef::BlankNode(b) => InternedGraphName::BlankNode(self.blank(b)),
            GraphNameRef::DefaultGraph => InternedGraphName::DefaultGraph,
        }
    }

    pub(crate) fn named_ref(&self, n: InternedNamedNode) -> NamedNodeRef<'_> {
        self.named[n.0 as usize].as_ref()
    }

    pub(crate) fn blank_ref(&self, b: InternedBlankNode) -> BlankNodeRef<'_> {
        self.blank[b.0 as usize].as_ref()
    }

    pub(crate) fn literal_ref(&self, l: InternedLiteral) -> LiteralRef<'_> {
        self.literals[l.0 as usize].as_ref()
    }
}
