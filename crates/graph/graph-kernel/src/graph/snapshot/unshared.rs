// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Which entries of the replay state two graphs do not share (engine split
//! D164).
//!
//! Every graph entry sits behind an `Arc` and every write copies it first, so
//! two graphs cloned from one another share every entry neither has written
//! since. The replay state ([`to_snapshot_at`](Graph::to_snapshot_at) and the
//! facet store) of a shared entry is the same on both sides; [`Graph::
//! unshared_state`] walks the stores by pointer and lists, for each entry that
//! is not shared, its persisted form on either side, in the shape the snapshot
//! writes it. The caller compares those as bytes.
//!
//! The walk only decides what it can prove. Equal listed entries mean equal
//! states. A listed pair that differs means the states differ, because each
//! listed entry stands at the same place in both snapshots. Anything else (a
//! store laid out differently on the two sides, Surface edges that sort apart)
//! comes back as [`Unshared::Whole`], for the caller to compare whole.

use super::super::*;
use super::to::{persisted_coupling, persisted_field, persisted_node};
use crate::persistence::{
    PersistedCoupling, PersistedEdge, PersistedField, PersistedNode, PersistedResourceRecord,
};
use mere::model::Shared;
use petgraph::stable_graph::{EdgeIndex, NodeIndex, StableGraph};
use petgraph::visit::{EdgeIndexable, NodeIndexable};
use serde::Serialize;

/// One entry of the replay state, as the snapshot or facet store writes it.
#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum StateEntry<'a> {
    Node(PersistedNode),
    Field(PersistedField),
    Coupling(PersistedCoupling),
    Resource(Option<PersistedResourceRecord>),
    /// The Resource edges of one (from, to) pair, in snapshot order.
    ResourcePair(Vec<PersistedEdge>),
    ImportRecords(&'a [ImportRecord]),
    Navigation(&'a SharedNavigationMemory),
    /// One node's facets; absent writes `null`, which no present node does.
    Facets(Option<&'a portolan::NodeFacets>),
}

/// What [`Graph::unshared_state`] found.
#[derive(Debug)]
pub enum Unshared<'a> {
    /// The states are equal exactly when each pair writes the same bytes.
    Entries(Vec<(StateEntry<'a>, StateEntry<'a>)>),
    /// The states differ, for the reason named.
    Differ(&'static str),
    /// The walk cannot vouch either way; compare the whole states.
    Whole(&'static str),
}

/// Stops the walk with a verdict other than a list.
type Walk<T> = Result<T, Unshared<'static>>;

impl Graph {
    /// The replay-state entries `self` and `other` do not share, as pairs to
    /// compare (see the [module docs](self)).
    pub fn unshared_state<'a>(&'a self, other: &'a Graph) -> Unshared<'a> {
        let mut pairs = Vec::new();
        let walked = (|| {
            surface_nodes(self, other, &mut pairs)?;
            surface_edges(self, other)?;
            if !Shared::ptr_eq(&self.import_records, &other.import_records) {
                pairs.push((
                    StateEntry::ImportRecords(&self.import_records),
                    StateEntry::ImportRecords(&other.import_records),
                ));
            }
            keyed(&self.fields, &other.fields, &mut pairs, |f| {
                StateEntry::Field(persisted_field(f))
            })?;
            keyed(&self.couplings, &other.couplings, &mut pairs, |c| {
                StateEntry::Coupling(persisted_coupling(c))
            })?;
            if !Shared::ptr_eq(&self.nav, &other.nav) {
                pairs.push((
                    StateEntry::Navigation(&self.nav),
                    StateEntry::Navigation(&other.nav),
                ));
            }
            resources(self, other, &mut pairs)?;
            resource_edges(self, other, &mut pairs)?;
            if *self.shown_resources != *other.shown_resources {
                return Err(Unshared::Differ("shown resources"));
            }
            for id in self.facets.unshared_nodes(&other.facets) {
                pairs.push((
                    StateEntry::Facets(self.facets.facets_of(id)),
                    StateEntry::Facets(other.facets.facets_of(id)),
                ));
            }
            Ok(())
        })();
        match walked {
            Ok(()) => Unshared::Entries(pairs),
            Err(verdict) => verdict,
        }
    }
}

/// Surface nodes, in the snapshot's (index) order.
fn surface_nodes<'a>(
    a: &'a Graph,
    b: &'a Graph,
    pairs: &mut Vec<(StateEntry<'a>, StateEntry<'a>)>,
) -> Walk<()> {
    if a.inner.node_count() != b.inner.node_count() {
        return Err(Unshared::Differ("Surface node count"));
    }
    for ((_, x), (_, y)) in a.inner.nodes().zip(b.inner.nodes()) {
        if !Shared::ptr_eq(x, y) {
            pairs.push((
                StateEntry::Node(persisted_node(x)),
                StateEntry::Node(persisted_node(y)),
            ));
        }
    }
    Ok(())
}

/// Whether node slot `i` holds a node of one identity on both sides.
fn same_identity<N: portolan::Identified>(
    a: &StableGraph<Shared<N>, Shared<EdgePayload>>,
    b: &StableGraph<Shared<N>, Shared<EdgePayload>>,
) -> bool {
    (0..a.node_bound().max(b.node_bound())).all(|i| {
        match (a.node_weight(NodeIndex::new(i)), b.node_weight(NodeIndex::new(i))) {
            (None, None) => true,
            (Some(x), Some(y)) => Shared::ptr_eq(x, y) || x.id() == y.id(),
            _ => false,
        }
    })
}

/// Edge slots that do not hold one shared payload between the same two node
/// slots on both sides.
fn unshared_edge_slots(
    a: &StableGraph<impl Sized, Shared<EdgePayload>>,
    b: &StableGraph<impl Sized, Shared<EdgePayload>>,
) -> Vec<EdgeIndex> {
    (0..a.edge_bound().max(b.edge_bound()))
        .map(EdgeIndex::new)
        .filter(|&k| match (a.edge_weight(k), b.edge_weight(k)) {
            (None, None) => false,
            (Some(x), Some(y)) => {
                !Shared::ptr_eq(x, y) || a.edge_endpoints(k) != b.edge_endpoints(k)
            },
            _ => true,
        })
        .collect()
}

/// Surface edges, which the snapshot sorts by their plain JSON bytes: the
/// unshared ones must sort to the same keys on both sides.
fn surface_edges(a: &Graph, b: &Graph) -> Walk<()> {
    let (ga, gb) = (a.inner.inner(), b.inner.inner());
    let slots = unshared_edge_slots(ga, gb);
    if slots.is_empty() {
        return Ok(());
    }
    if !same_identity(ga, gb) {
        return Err(Unshared::Whole("Surface node slots changed identity"));
    }
    let keys = |graph: &Graph| {
        let g = graph.inner.inner();
        let mut keys: Vec<Vec<u8>> = slots
            .iter()
            .filter_map(|&k| {
                let (from, to) = g.edge_endpoints(k)?;
                let edge = graph.persisted_edge(from, to, g.edge_weight(k)?);
                Some(serde_json::to_vec(&edge).expect("persisted surface edge encodes"))
            })
            .collect();
        keys.sort();
        keys
    };
    if keys(a) == keys(b) {
        Ok(())
    } else {
        Err(Unshared::Whole("unshared Surface edges sort apart"))
    }
}

/// A map of shared entries, in key order.
fn keyed<'a, T>(
    a: &'a BTreeMap<Uuid, Shared<T>>,
    b: &'a BTreeMap<Uuid, Shared<T>>,
    pairs: &mut Vec<(StateEntry<'a>, StateEntry<'a>)>,
    entry: impl Fn(&T) -> StateEntry<'a>,
) -> Walk<()> {
    if a.len() != b.len() {
        return Err(Unshared::Differ("a keyed store's size"));
    }
    for ((ka, x), (kb, y)) in a.iter().zip(b) {
        if ka != kb {
            return Err(Unshared::Differ("a keyed store's keys"));
        }
        if !Shared::ptr_eq(x, y) {
            pairs.push((entry(x), entry(y)));
        }
    }
    Ok(())
}

/// Resources, in index order: a record is the node and its facets.
fn resources<'a>(
    a: &'a Graph,
    b: &'a Graph,
    pairs: &mut Vec<(StateEntry<'a>, StateEntry<'a>)>,
) -> Walk<()> {
    if a.resources.node_count() != b.resources.node_count() {
        return Err(Unshared::Differ("Resource count"));
    }
    let mut facets = a.resources.facets().unshared_nodes(b.resources.facets());
    facets.sort_unstable();
    for ((_, x), (_, y)) in a.resources.nodes().zip(b.resources.nodes()) {
        if !Shared::ptr_eq(x, y) || facets.binary_search(&&x.id()).is_ok() {
            pairs.push((
                StateEntry::Resource(a.resources.resource_record(x.id())),
                StateEntry::Resource(b.resources.resource_record(y.id())),
            ));
        }
    }
    Ok(())
}

/// Resource edges, which the snapshot writes pair by pair in (from, to)
/// order, each pair's edges in adjacency order. Every insertion takes a fresh
/// `Arc`, so a pair whose edges are all shared, in the same slots, holds them
/// in the same order on both sides; every other pair is listed whole.
fn resource_edges<'a>(
    a: &'a Graph,
    b: &'a Graph,
    pairs: &mut Vec<(StateEntry<'a>, StateEntry<'a>)>,
) -> Walk<()> {
    let (ga, gb) = (a.resources.inner(), b.resources.inner());
    let slots = unshared_edge_slots(ga, gb);
    if slots.is_empty() {
        return Ok(());
    }
    if !same_identity(ga, gb) {
        return Err(Unshared::Whole("Resource slots changed identity"));
    }
    let mut ends = std::collections::BTreeSet::new();
    for k in slots {
        for g in [ga, gb] {
            if let Some((from, to)) = g.edge_endpoints(k) {
                ends.insert((g[from].id(), g[to].id()));
            }
        }
    }
    for (from, to) in ends {
        pairs.push((
            StateEntry::ResourcePair(a.persisted_resource_edges_between(from, to)),
            StateEntry::ResourcePair(b.persisted_resource_edges_between(from, to)),
        ));
    }
    Ok(())
}
