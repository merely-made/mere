// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use kernel::graph::{CoverageLayer, CoverageLimit, CoverageNote};
use kernel::persistence::{PersistedEdge, PersistedShownResource};
use std::collections::btree_map::Entry;
/// Resident edges by `(from, to)` pair: the pair's edges as its records list
/// them, and the endpoints whose records listed them.
type Pairs = BTreeMap<(Uuid, Uuid), (Vec<PersistedEdge>, BTreeSet<Uuid>)>;
pub(super) async fn materialize<B: Backend>(
    store: &AddressableGraphStore<B>,
    catalog: &Catalog,
    records: &BTreeMap<GraphAddress, StoredNode>,
    full: bool,
) -> Result<
    (
        kernel::persistence::GraphSnapshot,
        NodeFacetStore,
        CoverageNote,
        ResidentFootprint,
    ),
    StoreError,
> {
    let addresses: BTreeSet<_> = records.keys().copied().collect();
    let mut snapshot = catalog.globals.clone();
    let mut facets = catalog.orphan_facets.clone();
    let mut surface_edges = Pairs::new();
    let mut resource_edges = Pairs::new();
    let mut footprint = ResidentFootprint::default();
    let mut limit = CoverageLimit::new(
        CoverageLayer::Residency,
        "stored graph items are not resident",
    );
    for row in &catalog.entries {
        let Some(record) = records.get(&row.address) else {
            match row.address {
                GraphAddress::Surface(id) => limit.surfaces.push(id),
                GraphAddress::Resource(id) => limit.resources.push(id),
            }
            continue;
        };
        footprint.record_bytes += encode(record)?.len();
        match (row.address, record) {
            (
                GraphAddress::Surface(id),
                StoredNode::Surface {
                    node,
                    facets: values,
                    edges,
                },
            ) => {
                if node.node_id != id.to_string() {
                    return Err(fail("Surface address mismatch"));
                }
                footprint.surfaces += 1;
                snapshot.nodes.push(node.clone());
                for (name, value) in values {
                    facets
                        .set(
                            id,
                            portolan::FacetId::new(name),
                            value.clone(),
                            &portolan::AcceptAll,
                        )
                        .map_err(|e| fail(e.to_string()))?;
                }
                if let Some(resource) = row.shown {
                    if !addresses.contains(&GraphAddress::Resource(resource)) {
                        return Err(fail("resident Surface lacks shown Resource"));
                    }
                    snapshot.shown_resources.push(PersistedShownResource {
                        surface_id: id.to_string(),
                        resource_id: resource.to_string(),
                    });
                }
                admit(id, edges, &addresses, true, &mut surface_edges)?;
            },
            (GraphAddress::Resource(id), StoredNode::Resource { record, edges }) => {
                if kernel::graph::ResourceNode::for_term(&record.canonical_iri).id() != id {
                    return Err(fail("Resource address mismatch"));
                }
                footprint.resources += 1;
                snapshot.resources.push(record.clone());
                admit(id, edges, &addresses, false, &mut resource_edges)?;
            },
            _ => return Err(fail("address stratum mismatch")),
        }
    }
    listed_by_both(&surface_edges)?;
    listed_by_both(&resource_edges)?;
    // `to_snapshot_at`'s order, rebuilt from content (engine split D175):
    // Surface edges by their JSON bytes, Resource edges by `(from, to)` pair,
    // each pair's edges as listed.
    let mut keyed = (surface_edges.into_values().flat_map(|(edges, _)| edges))
        .map(|edge| Ok((encode(&edge)?, edge)))
        .collect::<Result<Vec<_>, StoreError>>()?;
    keyed.sort_by(|a, b| a.0.cmp(&b.0));
    snapshot.edges = keyed.into_iter().map(|(_, edge)| edge).collect();
    snapshot.resource_edges = (resource_edges.into_values())
        .flat_map(|(edges, _)| edges)
        .collect();
    let (navigation, bytes) = navigation::load(
        store,
        &catalog.navigation,
        &addresses,
        full || addresses.len() == catalog.entries.len(),
    )
    .await?;
    snapshot.navigation = navigation;
    footprint.navigation_bytes = bytes;
    let mut coverage = CoverageNote::default();
    if !limit.surfaces.is_empty() || !limit.resources.is_empty() {
        limit.count = Some(limit.surfaces.len() + limit.resources.len());
        coverage.push(limit);
    }
    Ok((snapshot, facets, coverage, footprint))
}
/// `owner`'s record's resident edges, merged by pair with the other
/// endpoint's copy, which must list the same edges in the same order.
fn admit(
    owner: Uuid,
    edges: &[PersistedEdge],
    addresses: &BTreeSet<GraphAddress>,
    surface: bool,
    out: &mut Pairs,
) -> Result<(), StoreError> {
    let address = |id| {
        if surface {
            GraphAddress::Surface(id)
        } else {
            GraphAddress::Resource(id)
        }
    };
    let mut mine: BTreeMap<(Uuid, Uuid), Vec<PersistedEdge>> = BTreeMap::new();
    for edge in edges {
        let a = Uuid::parse_str(&edge.from_node_id).map_err(|e| fail(e.to_string()))?;
        let b = Uuid::parse_str(&edge.to_node_id).map_err(|e| fail(e.to_string()))?;
        if addresses.contains(&address(a)) && addresses.contains(&address(b)) {
            mine.entry((a, b)).or_default().push(edge.clone());
        }
    }
    for (pair, list) in mine {
        match out.entry(pair) {
            Entry::Occupied(copy) if copy.get().0 != list => {
                return Err(fail("inconsistent incident relation copies"));
            },
            Entry::Occupied(mut copy) => {
                copy.get_mut().1.insert(owner);
            },
            Entry::Vacant(slot) => {
                slot.insert((list, BTreeSet::from([owner])));
            },
        }
    }
    Ok(())
}
/// Every resident pair was listed by both its endpoints' records (engine
/// split D176): an edge one record lists and the other omits is a store that
/// disagrees with itself, refused as disagreeing copies are.
fn listed_by_both(pairs: &Pairs) -> Result<(), StoreError> {
    for ((a, b), (_, by)) in pairs {
        if *by != BTreeSet::from([*a, *b]) {
            return Err(fail(
                "inconsistent incident relation copies: listed by one endpoint",
            ));
        }
    }
    Ok(())
}
