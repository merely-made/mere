// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use kernel::graph::{CoverageLayer, CoverageLimit, CoverageNote};
use kernel::persistence::PersistedShownResource;
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
    let mut surface_edges = BTreeMap::new();
    let mut resource_edges = BTreeMap::new();
    let mut footprint = ResidentFootprint {
        catalog_bytes: encode(catalog)?.len(),
        ..Default::default()
    };
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
                            chartulary::FacetId::new(name),
                            value.clone(),
                            &chartulary::AcceptAll,
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
                admit(edges, &addresses, true, &mut surface_edges)?;
            },
            (GraphAddress::Resource(id), StoredNode::Resource { record, edges }) => {
                if kernel::graph::ResourceNode::for_term(&record.canonical_iri).id() != id {
                    return Err(fail("Resource address mismatch"));
                }
                footprint.resources += 1;
                snapshot.resources.push(record.clone());
                admit(edges, &addresses, false, &mut resource_edges)?;
            },
            _ => return Err(fail("address stratum mismatch")),
        }
    }
    snapshot.edges = surface_edges.into_values().collect();
    snapshot.resource_edges = resource_edges.into_values().collect();
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
fn admit(
    edges: &[(usize, kernel::persistence::PersistedEdge)],
    addresses: &BTreeSet<GraphAddress>,
    surface: bool,
    out: &mut BTreeMap<usize, kernel::persistence::PersistedEdge>,
) -> Result<(), StoreError> {
    for (index, edge) in edges {
        let a = Uuid::parse_str(&edge.from_node_id).map_err(|e| fail(e.to_string()))?;
        let b = Uuid::parse_str(&edge.to_node_id).map_err(|e| fail(e.to_string()))?;
        let address = |id| {
            if surface {
                GraphAddress::Surface(id)
            } else {
                GraphAddress::Resource(id)
            }
        };
        if addresses.contains(&address(a)) && addresses.contains(&address(b)) {
            if out.get(index).is_some_and(|previous| previous != edge) {
                return Err(fail("inconsistent incident relation copies"));
            }
            out.insert(*index, edge.clone());
        }
    }
    Ok(())
}
