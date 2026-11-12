// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Explicit legacy replay. Ordinary replay keeps its recorded stores.
//! The caller must establish a legacy placement profile before using this adapter.
//!
//! Legacy records repeat handles and pass through states in which both strata hold
//! one, which ordinary edge replay refuses. The migration replays its legacy source
//! and builds its output in an explicit staged scope, `Graph::stage_edges` to
//! `Graph::settle_staged_edges`: edge replays are not checked one by one, and the
//! finished output is checked once, refusing the migration if it would not reopen.
//! Whole identical rows never reach that check: they dedupe to the first (`copies`).

mod copies;

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

pub use super::legacy_content_migration::LegacyContentContext;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::capture::{
    OnRefusal, Replay as Outcome, ReplayAttribution, replay_attributed_deltas_onto,
    replay_graph_deltas_onto,
};
use super::predicate_declarations::PREDICATE_DECLARATIONS_FACET;
use super::snapshot::{payload_from_persisted, persisted_edge_for_ids};
use super::{AttributedDelta, CapturedDelta, EdgePayload, Graph, GraphStratum, ResourceNode};
use crate::persistence::{
    GraphSnapshot, PersistedContainmentEdgeData, PersistedEdge, PersistedImportedEdgeData,
    PersistedProvenanceEdgeData, PersistedResourceFacet, PersistedResourceRecord,
    PersistedSemanticStatement,
};

/// Baseline claims lack a retained minting event; the current endpoints are a fallback.
pub const LEGACY_RESOURCE_MIGRATION_FACET: &str = "semantic.legacy-resource-migration/v1";
pub const LEGACY_AGGREGATE_MIGRATION_FACET: &str = "semantic.legacy-aggregate-migration/v1";

/// A legacy aggregate has no carried handle or assertion timestamp.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LegacyAggregateMigrationNote {
    pub from_surface_id: String,
    pub to_surface_id: String,
    pub from_resource_id: String,
    pub to_resource_id: String,
    pub from_url: String,
    pub to_url: String,
    pub kind: super::RelationKind,
    pub author: Option<super::Author>,
    pub uncertainty: String,
}

/// An explicit endpoint choice for an ambiguous carried handle. No raw handle is aliased.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LegacyResourceOriginResolution {
    pub statement_id: String,
    pub from_resource_id: Uuid,
    pub to_resource_id: Uuid,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LegacyMigrationNote {
    pub from_surface_id: String,
    pub to_surface_id: String,
    pub from_resource_id: String,
    pub to_resource_id: String,
    pub from_url: String,
    pub to_url: String,
    pub statement: PersistedSemanticStatement,
    pub uncertainty: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LegacyMigrationError(pub String);

impl fmt::Display for LegacyMigrationError {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        out.write_str(&self.0)
    }
}
impl std::error::Error for LegacyMigrationError {}

/// The original journal is unchanged. Effective captures name the stores actually edited.
pub struct MigratedReplay {
    pub graph: Graph,
    pub baseline_effects: Vec<CapturedDelta>,
    pub entry_effects: Vec<Vec<CapturedDelta>>,
}

/// Evidence for an unresolved raw-event to carried-handle link, without selecting one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LegacyMintLinkDiagnostic {
    pub raw_entry_index: usize,
    pub carried_entry_index: usize,
    pub statement_id: String,
    pub surface_pair: (String, String),
    pub earlier_resource_pair: (String, String),
    pub current_resource_pair: (String, String),
    pub explicit_source_matches: bool,
}

/// Inspect mixed histories before choosing how to link raw events to later handles.
/// Equal content is evidence, not proof that distinct handles identify one assertion.
pub fn probe_legacy_mint_links(
    baseline: &Graph,
    entries: &[AttributedDelta],
) -> Result<Vec<LegacyMintLinkDiagnostic>, LegacyMigrationError> {
    let mut source = baseline.clone();
    source.stage_edges();
    let mut attribution = ReplayAttribution::from_baseline(baseline);
    let mut known = statement_ids(&baseline.to_snapshot().edges);
    let mut raw = Vec::<(usize, Pair, super::SemanticStatement, Pair)>::new();
    let mut diagnostics = Vec::new();
    for (index, entry) in entries.iter().enumerate() {
        let before = surface_pairs(&source)?;
        let _ =
            replay_attributed_deltas_onto(&mut source, &mut attribution, [entry], OnRefusal::Drop);
        let after = surface_pairs(&source)?;
        for (&pair, edges) in &after {
            let old = statement_ids(before.get(&pair).map(Vec::as_slice).unwrap_or(&[]));
            let endpoints = current_resource_pair(&source, pair)?;
            for edge in edges {
                let payload = payload_from_persisted(edge);
                for statement in payload.semantic_statements() {
                    if old.contains(&statement.statement_id) {
                        continue;
                    }
                    let identity = statement.clone();
                    if matches!(entry.delta, CapturedDelta::ReplaySetEdgesByIds { .. })
                        && !known.contains(&statement.statement_id)
                        && statement.recognized_sub_kind.is_none()
                        && super::built_in_predicate_stratum(&statement.predicate).is_none()
                    {
                        source.effective_predicate_stratum(&statement.predicate).map_err(|error|
                            LegacyMigrationError(format!("checkpoint C22: new carried handle {} has unresolved placement: {error}", statement.statement_id)))?;
                    }
                    let resource_placed =
                        historical_statement_stratum(&source, statement) == GraphStratum::Resource;
                    if raw_semantic_edit(&entry.delta) && resource_placed {
                        raw.push((index, pair, identity, endpoints));
                    } else if let CapturedDelta::ReplaySetEdgesByIds { edges, .. } = &entry.delta
                        && !known.contains(&statement.statement_id)
                        && resource_placed
                    {
                        let explicit = edges
                            .iter()
                            .flat_map(|edge| edge.retained_assertions())
                            .find(|carried| carried.statement_id == statement.statement_id)
                            .and_then(|carried| carried.provenance_iri.as_ref())
                            .filter(|source| {
                                source.as_str()
                                    != mere::model::edge_data::UNKNOWN_LEGACY_ASSERTER_IRI
                            });
                        for (raw_index, raw_pair, earlier, earlier_endpoints) in &raw {
                            if *raw_pair == pair
                                && *earlier_endpoints != endpoints
                                && earlier.predicate == identity.predicate
                                && earlier.recognized_sub_kind == identity.recognized_sub_kind
                                && earlier.graph_scope == identity.graph_scope
                                && (explicit.is_none()
                                    || earlier.provenance_iri == identity.provenance_iri)
                            {
                                diagnostics.push(LegacyMintLinkDiagnostic {
                                    raw_entry_index: *raw_index,
                                    carried_entry_index: index,
                                    statement_id: statement.statement_id.clone(),
                                    surface_pair: (pair.0.to_string(), pair.1.to_string()),
                                    earlier_resource_pair: (
                                        earlier_endpoints.0.to_string(),
                                        earlier_endpoints.1.to_string(),
                                    ),
                                    current_resource_pair: (
                                        endpoints.0.to_string(),
                                        endpoints.1.to_string(),
                                    ),
                                    explicit_source_matches: explicit.is_some()
                                        && earlier.provenance_iri == identity.provenance_iri,
                                });
                            }
                        }
                    }
                    known.insert(statement.statement_id.clone());
                }
            }
        }
    }
    Ok(diagnostics)
}

type Pair = (Uuid, Uuid);
type Pairs = BTreeMap<Pair, Vec<PersistedEdge>>;

#[derive(Clone)]
struct Origin {
    surface: Pair,
    resource: Option<Pair>,
    /// The statement as first seen: later records of the handle must be the same claim.
    claim: super::SemanticStatement,
}

#[derive(Clone, PartialEq)]
struct Membership {
    surface: Pair,
    resource: Pair,
    edges: Vec<PersistedEdge>,
}

struct Replay {
    source: Graph,
    output: Graph,
    attribution: ReplayAttribution,
    origins: BTreeMap<String, Origin>,
    active: BTreeMap<String, Membership>,
    aggregate_origins: BTreeMap<(Pair, u32), Pair>,
    aggregate_active: BTreeMap<(Pair, u32), Membership>,
    resolutions: BTreeMap<String, Pair>,
    authored_declarations: BTreeSet<String>,
    content: super::legacy_content_migration::ContentReplay,
    content_context: Option<super::legacy_content_migration::LegacyContentContext>,
}

fn parse_id(id: &str) -> Result<Uuid, LegacyMigrationError> {
    Uuid::parse_str(id).map_err(|error| LegacyMigrationError(error.to_string()))
}

fn historical_statement_stratum(
    graph: &Graph,
    statement: &super::SemanticStatement,
) -> GraphStratum {
    statement
        .recognized_sub_kind
        .map(|kind| super::built_in_relation_stratum(super::RelationKind::Semantic(kind)))
        .or_else(|| super::built_in_predicate_stratum(&statement.predicate))
        .unwrap_or_else(|| {
            graph
                .effective_predicate_stratum(&statement.predicate)
                .unwrap_or(GraphStratum::Resource)
        })
}

fn aggregate_kinds(payload: &EdgePayload) -> Vec<super::RelationKind> {
    let mut kinds: Vec<_> = payload
        .held_assertions()
        .filter(|statement| {
            statement
                .statement_id
                .starts_with(super::legacy_family_attribution::LEGACY_FAMILY_ASSERTION_ID_PREFIX)
        })
        .map(|statement| statement.relation_kind())
        .filter(|kind| {
            matches!(
                kind,
                super::RelationKind::Containment(_)
                    | super::RelationKind::Imported(_)
                    | super::RelationKind::Provenance(_)
            )
        })
        .collect();
    kinds.sort_by_key(|kind| kind.tag());
    kinds.dedup();
    kinds
}

fn filter_resource_aggregates(pair: Pair, edges: &[PersistedEdge]) -> Vec<PersistedEdge> {
    edges
        .iter()
        .filter_map(|edge| {
            let mut payload = payload_from_persisted(edge);
            let kinds: Vec<_> = aggregate_kinds(&payload)
                .into_iter()
                .filter(|kind| super::built_in_relation_stratum(*kind) == GraphStratum::Resource)
                .collect();
            if !kinds.is_empty() {
                payload.assertions.statements.retain(|statement| {
                    !statement.statement_id.starts_with(
                        super::legacy_family_attribution::LEGACY_FAMILY_ASSERTION_ID_PREFIX,
                    ) || !kinds.contains(&statement.relation_kind())
                });
                payload.assertions.rebuild_compat();
            }
            (!payload.is_empty()).then(|| persisted_edge_for_ids(pair.0, pair.1, &payload))
        })
        .collect()
}

fn record_value(
    record: &PersistedResourceRecord,
) -> Result<(String, BTreeMap<String, serde_json::Value>), LegacyMigrationError> {
    let mut facets = BTreeMap::new();
    for facet in &record.facets {
        let value = serde_json::from_str(&facet.value_json)
            .map_err(|error| LegacyMigrationError(error.to_string()))?;
        if facets.insert(facet.facet.clone(), value).is_some() {
            return Err(LegacyMigrationError(
                "duplicate typed resource facet".into(),
            ));
        }
    }
    Ok((record.canonical_iri.clone(), facets))
}
fn current_resource_pair(graph: &Graph, pair: Pair) -> Result<Pair, LegacyMigrationError> {
    let resource = |surface| {
        graph
            .get_node_by_id(surface)
            .map(|(_, node)| ResourceNode::new(node.primary_address().as_url_str()).id())
            .ok_or_else(|| LegacyMigrationError("legacy endpoint is absent".into()))
    };
    Ok((resource(pair.0)?, resource(pair.1)?))
}
fn raw_semantic_edit(delta: &CapturedDelta) -> bool {
    matches!(
        delta,
        CapturedDelta::ReplayAssertRelationByIds {
            assertion: super::EdgeAssertion::Semantic { .. },
            ..
        } | CapturedDelta::ReplayAssertSemanticPredicateByIds { .. }
            | CapturedDelta::ReplaySetEdgeSemanticPredicateByIds { .. }
    )
}

/// A legacy aggregate can retain an actor only when this exact replay event
/// first asserted the directed member. Full-pair snapshots and unrelated
/// checkpoint writes do not attribute newly seen sidecar membership.
fn delta_introduces_member(delta: &CapturedDelta, pair: Pair, kind: super::RelationKind) -> bool {
    match delta {
        CapturedDelta::ReplayAssertRelationByIds {
            from_id,
            to_id,
            assertion,
        } => {
            let asserted_kind = match assertion {
                super::EdgeAssertion::Semantic { sub_kind, .. } => {
                    super::RelationKind::Semantic(*sub_kind)
                },
                super::EdgeAssertion::Containment { sub_kind } => {
                    super::RelationKind::Containment(*sub_kind)
                },
                super::EdgeAssertion::Arrangement { sub_kind } => {
                    super::RelationKind::Arrangement(*sub_kind)
                },
                super::EdgeAssertion::Imported { sub_kind } => {
                    super::RelationKind::Imported(*sub_kind)
                },
                super::EdgeAssertion::Provenance { sub_kind } => {
                    super::RelationKind::Provenance(*sub_kind)
                },
            };
            (parse_id(from_id).ok(), parse_id(to_id).ok()) == (Some(pair.0), Some(pair.1))
                && asserted_kind == kind
        },
        _ => false,
    }
}

fn surface_pairs(graph: &Graph) -> Result<Pairs, LegacyMigrationError> {
    pairs(&graph.to_snapshot().edges)
}
fn pairs(edges: &[PersistedEdge]) -> Result<Pairs, LegacyMigrationError> {
    let mut pairs = Pairs::new();
    for edge in edges {
        pairs
            .entry((parse_id(&edge.from_node_id)?, parse_id(&edge.to_node_id)?))
            .or_default()
            .push(edge.clone());
    }
    Ok(pairs)
}
fn statement_ids(edges: &[PersistedEdge]) -> BTreeSet<String> {
    edges
        .iter()
        .flat_map(|edge| edge.retained_assertions())
        .map(|statement| statement.statement_id.clone())
        .collect()
}
fn filter_edges(
    pair: Pair,
    edges: &[PersistedEdge],
    remove: impl Fn(&str) -> bool,
) -> Vec<PersistedEdge> {
    edges
        .iter()
        .filter_map(|edge| {
            let mut payload = payload_from_persisted(edge);
            let ids: Vec<_> = payload
                .assertions()
                .iter()
                .filter(|statement| remove(&statement.statement_id))
                .map(|statement| statement.statement_id.clone())
                .collect();
            for id in ids {
                payload.compact_semantic_statement(&id);
            }
            (!payload.is_empty()).then(|| persisted_edge_for_ids(pair.0, pair.1, &payload))
        })
        .collect()
}
fn legacy_edge_edit(delta: &CapturedDelta) -> bool {
    matches!(
        delta,
        CapturedDelta::ReplayAssertRelationByIds { .. }
            | CapturedDelta::ReplayRetractRelationsByIds { .. }
            | CapturedDelta::ReplayAppendTraversalByIds { .. }
            | CapturedDelta::ReplaySetEdgeSemanticPredicateByIds { .. }
            | CapturedDelta::ReplayAssertSemanticPredicateByIds { .. }
            | CapturedDelta::ReplaySetEdgesByIds { .. }
    )
}
fn typed_resource_pair(delta: &CapturedDelta) -> Result<Option<Pair>, LegacyMigrationError> {
    match delta {
        CapturedDelta::ReplaySetResourceEdgesByIds {
            from_resource_id,
            to_resource_id,
            ..
        } => Ok(Some((
            parse_id(from_resource_id)?,
            parse_id(to_resource_id)?,
        ))),
        _ => Ok(None),
    }
}

fn lifecycle_surface(delta: &CapturedDelta) -> Result<Option<Uuid>, LegacyMigrationError> {
    match delta {
        CapturedDelta::ReplayAddNodeWithIdIfMissing { id: node_id, .. }
        | CapturedDelta::ReplaySetNodeUrlById { node_id, .. }
        | CapturedDelta::ReplayNavigateNodeById { node_id, .. }
        | CapturedDelta::ReplayNodeHistoryBackById { node_id, .. }
        | CapturedDelta::ReplayNodeHistoryForwardById { node_id, .. } => {
            Ok(Some(parse_id(node_id)?))
        },
        _ => Ok(None),
    }
}

#[cfg(test)]
#[path = "legacy_resource_migration/tests.rs"]
mod tests;

#[path = "legacy_resource_migration/replay.rs"]
mod replay;

#[path = "legacy_resource_migration/migration.rs"]
mod migration;

pub use migration::{
    migrate_legacy_prefix, migrate_legacy_prefix_with_content_context,
    migrate_legacy_prefix_with_resolutions, validate_legacy_origin_resolutions,
};

#[path = "legacy_resource_migration/effects.rs"]
mod effects;

use effects::{effects_between, facet_effects_between};
