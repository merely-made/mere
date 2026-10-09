// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use kernel::graph::{CoverageLayer, ResourceNode};
use muniment::WriteOp;
impl<B: Backend> AddressableGraphStore<B> {
    /// Publish immutable records and catalog, then their head pointer. Existing
    /// resident readers retain their prior generation until an explicit refresh.
    pub async fn write_recorded_graph(&self, graph: &Graph) -> Result<String, StoreError> {
        if graph
            .known_coverage()
            .limits
            .iter()
            .any(|l| l.layer == CoverageLayer::Residency)
        {
            return Err(fail(
                "cannot publish a residency-limited view as complete truth",
            ));
        }
        let snapshot = graph.to_snapshot_at(0);
        Graph::try_from_recorded_snapshot(&snapshot).map_err(|e| fail(e.to_string()))?;
        let mut catalog = Catalog {
            version: 1,
            globals: snapshot.clone(),
            entries: vec![],
            navigation: NavigationIndex {
                entries: vec![],
                visits: vec![],
                owners: vec![],
                identities: vec![],
                digest: hash(&encode(snapshot.navigation.snapshot())?),
            },
            orphan_facets: graph.facets().clone(),
        };
        catalog.globals.nodes.clear();
        catalog.globals.edges.clear();
        catalog.globals.resources.clear();
        catalog.globals.resource_edges.clear();
        catalog.globals.shown_resources.clear();
        catalog.globals.navigation = Default::default();
        let mut ops = vec![];
        for node in &snapshot.nodes {
            let id = Uuid::parse_str(&node.node_id).map_err(|e| fail(e.to_string()))?;
            catalog.orphan_facets.remove_node(&id);
            let facets = graph
                .facets()
                .facets_of(&id)
                .map(|f| {
                    f.iter()
                        .map(|(k, v)| (k.as_str().to_owned(), v.clone()))
                        .collect()
                })
                .unwrap_or_default();
            let edges = snapshot
                .edges
                .iter()
                .enumerate()
                .filter(|(_, e)| e.from_node_id == node.node_id || e.to_node_id == node.node_id)
                .map(|(i, e)| (i, e.clone()))
                .collect();
            let row = IndexRow {
                address: GraphAddress::Surface(id),
                hash: String::new(),
                shown: snapshot
                    .shown_resources
                    .iter()
                    .find(|b| b.surface_id == node.node_id)
                    .map(|b| Uuid::parse_str(&b.resource_id))
                    .transpose()
                    .map_err(|e| fail(e.to_string()))?,
                pinned: graph
                    .get_node_key_by_id(id)
                    .and_then(|k| graph.node_is_pinned(k))
                    .unwrap_or(false),
                declaration: false,
            };
            self.append_node(
                &mut catalog,
                &mut ops,
                row,
                StoredNode::Surface {
                    node: node.clone(),
                    facets,
                    edges,
                },
            )?;
        }
        for record in &snapshot.resources {
            let id = ResourceNode::for_term(&record.canonical_iri).id();
            let edges = snapshot
                .resource_edges
                .iter()
                .enumerate()
                .filter(|(_, e)| e.from_node_id == id.to_string() || e.to_node_id == id.to_string())
                .map(|(i, e)| (i, e.clone()))
                .collect();
            let declaration = record.facets.iter().any(|f| {
                f.facet == kernel::graph::predicate_declarations::PREDICATE_DECLARATIONS_FACET
            });
            self.append_node(
                &mut catalog,
                &mut ops,
                IndexRow {
                    address: GraphAddress::Resource(id),
                    hash: String::new(),
                    shown: None,
                    pinned: false,
                    declaration,
                },
                StoredNode::Resource {
                    record: record.clone(),
                    edges,
                },
            )?;
        }
        navigation::write(
            self,
            snapshot.navigation.snapshot(),
            &mut catalog.navigation,
            &mut ops,
        )?;
        let bytes = encode(&catalog)?;
        let generation = hash(&bytes);
        ops.push(WriteOp::Put {
            key: self.key(&format!("catalog/{generation}")),
            value: bytes,
        });
        ops.push(WriteOp::Put {
            key: self.key("head"),
            value: encode(&generation)?,
        });
        self.backend.apply(&ops).await?;
        Ok(generation)
    }
    fn append_node(
        &self,
        catalog: &mut Catalog,
        ops: &mut Vec<WriteOp>,
        mut row: IndexRow,
        node: StoredNode,
    ) -> Result<(), StoreError> {
        let bytes = encode(&node)?;
        row.hash = hash(&bytes);
        ops.push(WriteOp::Put {
            key: self.record_key(row.address, &row.hash),
            value: bytes,
        });
        catalog.entries.push(row);
        Ok(())
    }
}
