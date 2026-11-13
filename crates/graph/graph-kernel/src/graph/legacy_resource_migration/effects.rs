// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

pub(super) fn effects_between(
    before: &GraphSnapshot,
    after: &GraphSnapshot,
    after_facets: &super::super::node_facets::NodeFacetStore,
) -> Result<Vec<CapturedDelta>, LegacyMigrationError> {
    let mut effects = Vec::new();
    let before_nodes: BTreeMap<_, _> = before
        .nodes
        .iter()
        .map(|node| (node.node_id.as_str(), node))
        .collect();
    // Content migration removes Surface tags; replay must carry that removal.
    for node in &after.nodes {
        if let Some(previous) = before_nodes.get(node.node_id.as_str()) {
            let mut removed = false;
            for tag in &previous.tags {
                if !node.tags.contains(tag) {
                    removed = true;
                    effects.push(CapturedDelta::ReplayRemoveNodeTagById {
                        node_id: node.node_id.clone(),
                        tag: tag.clone(),
                    });
                }
            }
            if removed {
                // Raw removal prunes presentation. Migration retains it for the
                // same labels now shown through the Resource.
                let facet = super::super::node_facets::PRESENTATION_TAGS;
                let id = parse_id(&node.node_id)?;
                effects.push(
                    match after_facets.get(&id, &portolan::FacetId::new(facet)) {
                        Some(value) => CapturedDelta::ReplaySetNodeFacetById {
                            node_id: node.node_id.clone(),
                            facet: facet.into(),
                            value_json: serde_json::to_string(value).expect("facet serializes"),
                        },
                        None => CapturedDelta::ReplayRemoveNodeFacetById {
                            node_id: node.node_id.clone(),
                            facet: facet.into(),
                        },
                    },
                );
            }
        }
    }
    let before_records: BTreeMap<_, _> = before
        .resources
        .iter()
        .map(|record| (ResourceNode::for_term(&record.canonical_iri).id(), record))
        .collect();
    let after_records: BTreeMap<_, _> = after
        .resources
        .iter()
        .map(|record| (ResourceNode::for_term(&record.canonical_iri).id(), record))
        .collect();
    // Preserve materialized resource insertion order in the exact reconstruction.
    let mut seen_records = BTreeSet::new();
    for id in after
        .resources
        .iter()
        .chain(&before.resources)
        .map(|record| ResourceNode::for_term(&record.canonical_iri).id())
        .filter(|id| seen_records.insert(*id))
    {
        if before_records.get(&id) != after_records.get(&id) {
            effects.push(CapturedDelta::ReplaySetResourceRecordById {
                resource_id: id.to_string(),
                record: after_records.get(&id).map(|record| (*record).clone()),
            });
        }
    }
    let before_shown: BTreeMap<_, _> = before
        .shown_resources
        .iter()
        .map(|shown| (&shown.surface_id, &shown.resource_id))
        .collect();
    let after_shown: BTreeMap<_, _> = after
        .shown_resources
        .iter()
        .map(|shown| (&shown.surface_id, &shown.resource_id))
        .collect();
    for surface in before_shown
        .keys()
        .chain(after_shown.keys())
        .copied()
        .collect::<BTreeSet<_>>()
    {
        if before_shown.get(surface) != after_shown.get(surface) {
            effects.push(CapturedDelta::ReplaySetShownResourceById {
                surface_id: surface.clone(),
                resource_id: after_shown.get(surface).map(|id| (*id).clone()),
            });
        }
    }
    for (resource, before, after) in [
        (false, &before.edges, &after.edges),
        (true, &before.resource_edges, &after.resource_edges),
    ] {
        let before = pairs(before)?;
        let after = pairs(after)?;
        for pair in before
            .keys()
            .chain(after.keys())
            .copied()
            .collect::<BTreeSet<_>>()
        {
            if before.get(&pair) != after.get(&pair) {
                let edges = after.get(&pair).cloned().unwrap_or_default();
                effects.push(if resource {
                    CapturedDelta::ReplaySetResourceEdgesByIds {
                        from_resource_id: pair.0.to_string(),
                        to_resource_id: pair.1.to_string(),
                        edges,
                    }
                } else {
                    CapturedDelta::ReplaySetEdgesByIds {
                        from_id: pair.0.to_string(),
                        to_id: pair.1.to_string(),
                        edges: edges.into_iter().rev().collect(),
                    }
                });
            }
        }
    }
    Ok(effects)
}

pub(super) fn facet_effects_between(
    before: &super::super::node_facets::NodeFacetStore,
    after: &super::super::node_facets::NodeFacetStore,
) -> Vec<CapturedDelta> {
    let values = |store: &super::super::node_facets::NodeFacetStore| -> BTreeMap<_, _> {
        store
            .iter()
            .flat_map(|(node, facets)| {
                facets
                    .iter()
                    .map(move |(facet, value)| ((*node, facet.as_str().to_owned()), value.clone()))
            })
            .collect()
    };
    let before = values(before);
    let after = values(after);
    before
        .keys()
        .chain(after.keys())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .filter(|key| before.get(*key) != after.get(*key))
        .map(|(node, facet)| match after.get(&(*node, facet.clone())) {
            Some(value) => CapturedDelta::ReplaySetNodeFacetById {
                node_id: node.to_string(),
                facet: facet.clone(),
                value_json: serde_json::to_string(value).expect("facet value serializes"),
            },
            None => CapturedDelta::ReplayRemoveNodeFacetById {
                node_id: node.to_string(),
                facet: facet.clone(),
            },
        })
        .collect()
}
