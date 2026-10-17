// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Shared read adapter from the kernel's current relation stores to the
//! portable Scenograph link reading.

use std::collections::{BTreeMap, BTreeSet};

use kernel::graph::{CoverageLayer, EdgeFamily, EdgePayload, Graph, GraphStratum, RelationKey};
use kernel::types::GraphScope;
use scenograph::link::{
    LinkAssertion, LinkAssertionBasis, LinkAssertionMethod, LinkBucket, LinkCoverage,
    LinkCoverageLayer, LinkCoverageLimit, LinkEndpoint, LinkGraphScope, LinkKey, LinkLegacyFamily,
    LinkLegacyMember, LinkPairReading, LinkReading, LinkSource, LinkStratum, LinkWithdrawal,
};
use uuid::Uuid;

fn scope(scope: &GraphScope) -> LinkGraphScope {
    match scope {
        GraphScope::Default => LinkGraphScope::Default,
        GraphScope::Source => LinkGraphScope::Source,
        GraphScope::User => LinkGraphScope::User,
        GraphScope::Agent => LinkGraphScope::Agent,
        GraphScope::Moot => LinkGraphScope::Moot,
        GraphScope::Custom(value) => LinkGraphScope::Custom(value.clone()),
    }
}

fn serialized_name<T: serde::Serialize>(value: &T) -> String {
    serde_json::to_value(value)
        .ok()
        .and_then(|value| value.as_str().map(str::to_owned))
        .unwrap_or_else(|| "unknown".to_owned())
}

fn family(
    family: EdgeFamily,
    data: impl FnOnce() -> serde_json::Value,
    out: &mut Vec<LinkLegacyFamily>,
    include_summaries: bool,
) {
    if !include_summaries {
        return;
    }
    out.push(LinkLegacyFamily {
        family: serialized_name(&family),
        data: data(),
    });
}

// Every relation family now has exact records. Traversal remains an event
// rollup and is the sole compatibility member/summary in this adapter.
fn legacy_data(
    payload: &EdgePayload,
    include_summaries: bool,
) -> (Vec<LinkLegacyMember>, Vec<LinkLegacyFamily>) {
    let mut members = Vec::new();
    let mut out = Vec::new();
    if let Some(data) = &payload.traversal {
        family(
            EdgeFamily::Traversal,
            || {
                serde_json::json!({
                    "traversals": data.traversals.iter().map(|event| serde_json::json!({
                        "timestamp_ms": event.timestamp_ms, "trigger": event.trigger
                    })).collect::<Vec<_>>(),
                    "metrics": {
                        "total_navigations": data.metrics.total_navigations,
                        "forward_navigations": data.metrics.forward_navigations,
                        "backward_navigations": data.metrics.backward_navigations,
                        "last_navigated_at": data.metrics.last_navigated_at
                    }
                })
            },
            &mut out,
            include_summaries,
        );
        members.push(LinkLegacyMember {
            family: serialized_name(&EdgeFamily::Traversal),
            sub_kind: None,
        });
    }
    (members, out)
}

fn coverage_layer(layer: CoverageLayer) -> LinkCoverageLayer {
    match layer {
        CoverageLayer::Possession => LinkCoverageLayer::Possession,
        CoverageLayer::Residency => LinkCoverageLayer::Residency,
        CoverageLayer::Disclosure => LinkCoverageLayer::Disclosure,
        CoverageLayer::Synchronization => LinkCoverageLayer::Synchronization,
        CoverageLayer::Projection => LinkCoverageLayer::Projection,
    }
}

fn endpoint(
    graph: &Graph,
    stratum: GraphStratum,
    surface: Uuid,
    resource: Option<Uuid>,
) -> Option<LinkEndpoint> {
    match stratum {
        GraphStratum::Surface => {
            let (_, node) = graph.get_node_by_id(surface)?;
            Some(LinkEndpoint {
                id: surface.to_string(),
                address: Some(node.url().to_owned()),
            })
        },
        GraphStratum::Resource => {
            let resource = graph.resource(resource?)?;
            Some(LinkEndpoint {
                id: resource.id().to_string(),
                address: Some(resource.canonical_iri().to_owned()),
            })
        },
    }
}

fn projected_endpoint_id(endpoint: &LinkEndpoint, stratum: LinkStratum) -> String {
    match stratum {
        LinkStratum::Surface => endpoint.id.clone(),
        LinkStratum::Resource => endpoint
            .address
            .clone()
            .unwrap_or_else(|| endpoint.id.clone()),
    }
}

/// Read every directed bucket contributing to the pair, preserving parallel
/// buckets and the identity of the owning stratum. Source authority is always
/// supplied by the caller; no public source or revision is inferred here.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LinkReadOptions {
    /// Default public reads expose held assertions. History is opt-in and means
    /// retained lifecycle records in this supplied graph snapshot.
    pub scope: LinkReadScope,
    /// Include the transitional raw family summaries required by Graphshell's
    /// current inspector output. PairInputs should use the summary-free default.
    pub include_legacy_summaries: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LinkReadScope {
    #[default]
    Held,
    History,
}

pub fn read_link(
    graph: &Graph,
    first: Uuid,
    second: Uuid,
    source: LinkSource,
) -> Result<LinkPairReading, String> {
    read_link_with_options(graph, first, second, source, LinkReadOptions::default())
}

/// Explicitly include retained held and withdrawn assertion lifecycle records.
/// This reads the supplied graph snapshot; it does not replay a journal.
pub fn read_link_history(
    graph: &Graph,
    first: Uuid,
    second: Uuid,
    source: LinkSource,
) -> Result<LinkPairReading, String> {
    read_link_with_options(
        graph,
        first,
        second,
        source,
        LinkReadOptions {
            scope: LinkReadScope::History,
            ..LinkReadOptions::default()
        },
    )
}

pub fn read_link_with_options(
    graph: &Graph,
    first: Uuid,
    second: Uuid,
    source: LinkSource,
    options: LinkReadOptions,
) -> Result<LinkPairReading, String> {
    let first_key = graph
        .get_node_key_by_id(first)
        .ok_or_else(|| format!("Access {first} is absent from this view"))?;
    let second_key = graph
        .get_node_key_by_id(second)
        .ok_or_else(|| format!("Access {second} is absent from this view"))?;
    let mut links = BTreeMap::<LinkKey, Vec<LinkBucket>>::new();
    let mut resource_ids = BTreeSet::new();
    let mut surface_ids = BTreeSet::from([first, second]);
    for surface in [first, second] {
        if let Some(id) =
            graph.shown_resource_id(graph.get_node_key_by_id(surface).expect("resolved above"))
        {
            resource_ids.insert(id);
            surface_ids.extend(graph.surface_ids_showing_resource(id));
        }
    }
    let pairs = if first == second {
        vec![(first, second, first_key, second_key)]
    } else {
        vec![
            (first, second, first_key, second_key),
            (second, first, second_key, first_key),
        ]
    };
    for (displayed_from, displayed_to, from_key, to_key) in pairs {
        let pair_relations: Vec<(RelationKey, &EdgePayload)> = match options.scope {
            LinkReadScope::Held => graph
                .projected_relations_between(from_key, to_key)
                .collect(),
            LinkReadScope::History => graph.retained_relations_between(from_key, to_key).collect(),
        };
        for (relation_key, payload) in pair_relations {
            let stratum = match relation_key {
                RelationKey::Surface(_) => GraphStratum::Surface,
                RelationKey::Resource(_) => GraphStratum::Resource,
            };
            let source_from_id = if stratum == GraphStratum::Resource {
                graph.shown_resource_id(from_key)
            } else {
                None
            };
            let source_to_id = if stratum == GraphStratum::Resource {
                graph.shown_resource_id(to_key)
            } else {
                None
            };
            if let Some(id) = source_from_id {
                resource_ids.insert(id);
            }
            if let Some(id) = source_to_id {
                resource_ids.insert(id);
            }
            let Some(source_from) = endpoint(graph, stratum, displayed_from, source_from_id) else {
                continue;
            };
            let Some(source_to) = endpoint(graph, stratum, displayed_to, source_to_id) else {
                continue;
            };
            let portable_stratum = match stratum {
                GraphStratum::Surface => LinkStratum::Surface,
                GraphStratum::Resource => LinkStratum::Resource,
            };
            let key = LinkKey::new(
                portable_stratum,
                projected_endpoint_id(&source_from, portable_stratum),
                projected_endpoint_id(&source_to, portable_stratum),
            );
            let mut assertions = Vec::new();
            let mut add_assertion = |statement: &kernel::graph::SemanticStatement| {
                assertions.push(LinkAssertion {
                    id: statement.statement_id.clone(),
                    predicate: statement.predicate.clone(),
                    recognized_kind: match statement.relation_kind() {
                        kernel::graph::RelationKind::Semantic(kind) => Some(serialized_name(&kind)),
                        _ => None,
                    },
                    label: statement.label.clone(),
                    scope: scope(&statement.graph_scope),
                    asserter: statement.provenance_iri.clone(),
                    first_asserted_ms: statement.first_asserted_ms,
                    asserted_at_ms: statement.asserted_at_ms,
                    withdrawn: statement
                        .withdrawn
                        .as_ref()
                        .map(|withdrawal| LinkWithdrawal {
                            at_ms: withdrawal.at_ms,
                            by: withdrawal.by.clone(),
                        }),
                    basis: LinkAssertionBasis {
                        note: statement.basis.note.clone(),
                        method: statement
                            .basis
                            .method
                            .as_ref()
                            .map(|method| LinkAssertionMethod {
                                iri: method.iri.clone(),
                                version: method.version.clone(),
                            }),
                        evidence: statement.basis.evidence.clone(),
                    },
                })
            };
            match options.scope {
                LinkReadScope::Held => payload.held_assertions().for_each(&mut add_assertion),
                LinkReadScope::History => payload.assertions().iter().for_each(&mut add_assertion),
            }
            let (legacy_members, legacy_summaries) =
                legacy_data(payload, options.include_legacy_summaries);
            if assertions.is_empty() && legacy_members.is_empty() && legacy_summaries.is_empty() {
                continue;
            }
            links.entry(key).or_default().push(LinkBucket {
                displayed_from: displayed_from.to_string(),
                displayed_to: displayed_to.to_string(),
                source_from,
                source_to,
                assertions,
                legacy_members,
                legacy_summaries,
            });
        }
    }
    let coverage = graph
        .coverage_note()
        .limits
        .into_iter()
        .filter(|limit| {
            (limit.resources.is_empty() && limit.surfaces.is_empty())
                || limit.resources.iter().any(|id| resource_ids.contains(id))
                || limit.surfaces.iter().any(|id| surface_ids.contains(id))
        })
        .map(|limit| LinkCoverageLimit {
            layer: coverage_layer(limit.layer),
            reason: limit.reason,
            resources: limit
                .resources
                .into_iter()
                .map(|id| id.to_string())
                .collect(),
            surfaces: limit
                .surfaces
                .into_iter()
                .map(|id| id.to_string())
                .collect(),
            count: limit.count,
        })
        .collect::<Vec<_>>();
    let readings = links
        .into_iter()
        .map(|(key, mut buckets)| {
            buckets.sort_by(|a, b| {
                (
                    &a.displayed_from,
                    &a.displayed_to,
                    &a.source_from,
                    &a.source_to,
                )
                    .cmp(&(
                        &b.displayed_from,
                        &b.displayed_to,
                        &b.source_from,
                        &b.source_to,
                    ))
            });
            LinkReading { key, buckets }
        })
        .collect();
    Ok(LinkPairReading {
        source,
        links: readings,
        coverage: LinkCoverage { limits: coverage },
    })
}

#[cfg(test)]
#[path = "link_reading_tests.rs"]
mod tests;
