// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Explicit legacy Semantic replay. Ordinary replay keeps its recorded stores.
//! The caller must establish a legacy placement profile before using this adapter.
//! Aggregate families remain on surfaces until their identity protocol is ruled.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::capture::{ReplayAttribution, replay_attributed_deltas_onto, replay_graph_deltas_onto};
use super::predicate_declarations::PREDICATE_DECLARATIONS_FACET;
use super::snapshot::{payload_from_persisted, persisted_edge_for_ids};
use super::{AttributedDelta, CapturedDelta, EdgePayload, Graph, GraphStratum, ResourceNode};
use crate::persistence::{
    GraphSnapshot, PersistedEdge, PersistedResourceFacet, PersistedResourceRecord,
    PersistedSemanticStatement,
};

/// Baseline claims lack a retained minting event; the current endpoints are a fallback.
pub const LEGACY_RESOURCE_MIGRATION_FACET: &str = "semantic.legacy-resource-migration/v1";

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
    let mut attribution = ReplayAttribution::from_baseline(baseline);
    let mut known = statement_ids(&baseline.to_snapshot().edges);
    let mut raw = Vec::<(usize, Pair, ClaimIdentity, Pair)>::new();
    let mut diagnostics = Vec::new();
    for (index, entry) in entries.iter().enumerate() {
        let before = surface_pairs(&source)?;
        replay_attributed_deltas_onto(&mut source, &mut attribution, [entry]);
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
                    let identity = ClaimIdentity::from(statement);
                    if raw_semantic_edit(&entry.delta) {
                        raw.push((index, pair, identity, endpoints));
                    } else if let CapturedDelta::ReplaySetEdgesByIds { edges, .. } = &entry.delta
                        && !known.contains(&statement.statement_id)
                    {
                        let explicit = edges
                            .iter()
                            .filter_map(|edge| edge.semantic.as_ref())
                            .flat_map(|semantic| &semantic.statements)
                            .find(|carried| carried.statement_id == statement.statement_id)
                            .and_then(|carried| carried.provenance_iri.as_ref())
                            .filter(|source| {
                                source.as_str() != super::edge_data::UNKNOWN_LEGACY_ASSERTER_IRI
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
    identity: ClaimIdentity,
}

#[derive(Clone, PartialEq, Eq)]
struct ClaimIdentity {
    predicate: String,
    recognized_sub_kind: Option<super::SemanticSubKind>,
    graph_scope: crate::types::GraphScope,
    provenance_iri: Option<String>,
}

impl From<&super::SemanticStatement> for ClaimIdentity {
    fn from(statement: &super::SemanticStatement) -> Self {
        Self {
            predicate: statement.predicate.clone(),
            recognized_sub_kind: statement.recognized_sub_kind,
            graph_scope: statement.graph_scope.clone(),
            provenance_iri: statement.provenance_iri.clone(),
        }
    }
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
    authored_declarations: BTreeSet<String>,
}

/// Replay a caller-qualified legacy baseline and its retained prefix atomically.
/// Existing typed resource records and claims keep their explicit store.
/// Exact carried handles retain identity. Raw records keep the existing replay
/// exception: they did not record stable handles. Unresolved mixed-history
/// evidence stops at checkpoint C28 before any migrated graph is produced.
pub fn migrate_legacy_prefix(
    baseline: &Graph,
    entries: &[AttributedDelta],
) -> Result<MigratedReplay, LegacyMigrationError> {
    let diagnostics = probe_legacy_mint_links(baseline, entries)?;
    if !diagnostics.is_empty() {
        return Err(LegacyMigrationError(format!(
            "checkpoint C28: {} unresolved legacy mint-link candidates",
            diagnostics.len()
        )));
    }
    let mut replay = Replay {
        source: baseline.clone(),
        output: baseline.clone(),
        attribution: ReplayAttribution::from_baseline(baseline),
        origins: BTreeMap::new(),
        active: BTreeMap::new(),
        authored_declarations: baseline
            .to_snapshot()
            .resources
            .into_iter()
            .filter(|record| {
                record
                    .facets
                    .iter()
                    .any(|facet| facet.facet == PREDICATE_DECLARATIONS_FACET)
            })
            .map(|record| record.canonical_iri)
            .collect(),
    };
    let before = replay.output.to_snapshot();
    replay.sync(&Pairs::new(), true)?;
    let baseline_effects = effects_between(&before, &replay.output.to_snapshot())?;
    let mut entry_effects = Vec::with_capacity(entries.len());
    for entry in entries {
        let before = replay.output.to_snapshot();
        let old_pairs = surface_pairs(&replay.source)?;
        let typed_pair = typed_resource_pair(&entry.delta)?;
        if let Some(pair) = typed_pair {
            // This explicit replacement supersedes only this resource pair's old membership.
            replay.active.retain(|_, member| member.resource != pair);
            let delta = replay
                .attribution
                .replay_delta(&entry.delta, &entry.author)
                .ok_or_else(|| LegacyMigrationError("invalid typed resource capture".into()))?;
            let super::apply::GraphDelta::ReplaySetResourceEdgesByIds { edges, .. } = &delta else {
                unreachable!()
            };
            let expected = edges.clone();
            replay_graph_deltas_onto(&mut replay.output, [delta]);
            let actual = replay
                .output
                .persisted_resource_edges_between(pair.0, pair.1);
            if actual != expected {
                return Err(LegacyMigrationError(
                    "typed resource pair replacement was refused".into(),
                ));
            }
        } else {
            replay_attributed_deltas_onto(&mut replay.source, &mut replay.attribution, [entry]);
            if !legacy_edge_edit(&entry.delta) {
                if let Some(delta) = entry.delta.replay_delta_as(&entry.author) {
                    replay_graph_deltas_onto(&mut replay.output, [delta]);
                }
                replay.check_typed_record(&entry.delta)?;
            }
        }
        if let CapturedDelta::ReplaySetResourceRecordById {
            record: Some(record),
            ..
        } = &entry.delta
            && record
                .facets
                .iter()
                .any(|facet| facet.facet == PREDICATE_DECLARATIONS_FACET)
        {
            replay
                .authored_declarations
                .insert(record.canonical_iri.clone());
        }
        replay.sync(&old_pairs, false)?;
        if let Some(surface) = lifecycle_surface(&entry.delta)?
            && replay.source.get_node_key_by_id(surface).is_some()
        {
            let (resource, _) = replay.ensure_resource(surface)?;
            replay.output.set_shown_resource(surface, Some(resource));
        }
        let mut effective = Vec::new();
        if !legacy_edge_edit(&entry.delta) {
            effective.push(entry.delta.clone());
        }
        effective.extend(effects_between(&before, &replay.output.to_snapshot())?);
        entry_effects.push(effective);
    }
    // The same checked boundary used by ordinary loads verifies final cross-store handles.
    Graph::try_from_snapshot(&replay.output.to_snapshot())
        .map_err(|error| LegacyMigrationError(error.to_string()))?;
    Ok(MigratedReplay {
        graph: replay.output,
        baseline_effects,
        entry_effects,
    })
}

impl Replay {
    fn check_typed_record(&self, delta: &CapturedDelta) -> Result<(), LegacyMigrationError> {
        match delta {
            CapturedDelta::ReplaySetResourceRecordById {
                resource_id,
                record,
            } => {
                let id = parse_id(resource_id)?;
                let actual = self.output.resource_record(id);
                let actual = actual.as_ref().map(record_value).transpose()?;
                let expected = record.as_ref().map(record_value).transpose()?;
                if actual != expected {
                    return Err(LegacyMigrationError(
                        "typed resource record replacement was refused".into(),
                    ));
                }
            },
            CapturedDelta::ReplaySetShownResourceById {
                surface_id,
                resource_id,
            } => {
                let surface = self
                    .output
                    .get_node_key_by_id(parse_id(surface_id)?)
                    .ok_or_else(|| {
                        LegacyMigrationError("typed shown binding has no surface".into())
                    })?;
                let expected = resource_id.as_deref().map(parse_id).transpose()?;
                if self.output.shown_resource_id(surface) != expected {
                    return Err(LegacyMigrationError(
                        "typed shown binding was refused".into(),
                    ));
                }
            },
            _ => {},
        }
        Ok(())
    }

    fn ensure_resource(&mut self, surface: Uuid) -> Result<(Uuid, String), LegacyMigrationError> {
        let key = self
            .source
            .get_node_key_by_id(surface)
            .ok_or_else(|| LegacyMigrationError("legacy claim endpoint is absent".into()))?;
        let node = self.source.get_node(key).expect("surface key exists");
        let url = node.primary_address().as_url_str().to_owned();
        let resource = ResourceNode::new(&url);
        if self.output.resource(resource.id()).is_none() {
            let record = PersistedResourceRecord {
                canonical_iri: resource.canonical_iri().into(),
                facets: vec![],
            };
            if !self.output.set_resource_record(resource.id(), Some(record)) {
                return Err(LegacyMigrationError(
                    "resource identity insertion was refused".into(),
                ));
            }
        }
        Ok((resource.id(), url))
    }

    fn first_origin(
        &mut self,
        pair: Pair,
        payload: &EdgePayload,
        id: &str,
        baseline: bool,
    ) -> Result<Origin, LegacyMigrationError> {
        let statement = payload
            .semantic_statements()
            .iter()
            .find(|statement| statement.statement_id == id)
            .expect("claim present");
        let fixed = statement
            .recognized_sub_kind
            .map(|kind| super::built_in_relation_stratum(super::RelationKind::Semantic(kind)))
            .or_else(|| super::built_in_predicate_stratum(&statement.predicate));
        let nature = if let Some(nature) = fixed {
            nature
        } else if !baseline && self.authored_declarations.contains(&statement.predicate) {
            // A selected authored declaration in this prefix is evidence at mint time.
            // A conflict does not reinterpret historical raw assertions.
            self.source
                .effective_predicate_stratum(&statement.predicate)
                .unwrap_or(GraphStratum::Resource)
        } else {
            GraphStratum::Resource
        };
        if nature == GraphStratum::Surface {
            return Ok(Origin {
                surface: pair,
                resource: None,
                identity: statement.into(),
            });
        }
        let (from, from_url) = self.ensure_resource(pair.0)?;
        let (to, to_url) = self.ensure_resource(pair.1)?;
        if baseline {
            let edge = persisted_edge_for_ids(pair.0, pair.1, payload);
            let statement = edge
                .semantic
                .expect("semantic payload")
                .statements
                .into_iter()
                .find(|statement| statement.statement_id == id)
                .expect("statement serialized");
            self.add_note(
                from,
                LegacyMigrationNote {
                    from_surface_id: pair.0.to_string(),
                    to_surface_id: pair.1.to_string(),
                    from_resource_id: from.to_string(),
                    to_resource_id: to.to_string(),
                    from_url,
                    to_url,
                    statement,
                    uncertainty: "baseline_current_resources_without_mint_event".into(),
                },
            )?;
        }
        Ok(Origin {
            surface: pair,
            resource: Some((from, to)),
            identity: statement.into(),
        })
    }

    fn add_note(
        &mut self,
        resource: Uuid,
        note: LegacyMigrationNote,
    ) -> Result<(), LegacyMigrationError> {
        let mut record = self
            .output
            .resource_record(resource)
            .expect("resource ensured");
        let mut notes: Vec<LegacyMigrationNote> = record
            .facets
            .iter()
            .find(|facet| facet.facet == LEGACY_RESOURCE_MIGRATION_FACET)
            .map(|facet| serde_json::from_str(&facet.value_json))
            .transpose()
            .map_err(|error| {
                LegacyMigrationError(format!("invalid prior migration note: {error}"))
            })?
            .unwrap_or_default();
        if !notes.contains(&note) {
            notes.push(note);
        }
        record
            .facets
            .retain(|facet| facet.facet != LEGACY_RESOURCE_MIGRATION_FACET);
        record.facets.push(PersistedResourceFacet {
            facet: LEGACY_RESOURCE_MIGRATION_FACET.into(),
            value_json: serde_json::to_string(&notes).expect("migration note serializes"),
        });
        self.output.set_resource_record(resource, Some(record));
        Ok(())
    }

    fn sync(&mut self, before: &Pairs, baseline: bool) -> Result<(), LegacyMigrationError> {
        let after = surface_pairs(&self.source)?;
        let previous = self.active.clone();
        let present = statement_ids(&after.values().flatten().cloned().collect::<Vec<_>>());
        self.active.retain(|id, _| present.contains(id));
        let mut seen = BTreeMap::new();
        let mut next = BTreeMap::<String, Membership>::new();
        for (&pair, edges) in &after {
            let old_ids = statement_ids(before.get(&pair).map(Vec::as_slice).unwrap_or(&[]));
            for edge in edges {
                let mut payload = payload_from_persisted(edge);
                if let Some(semantic) = &mut payload.semantic {
                    for statement in &mut semantic.statements {
                        statement.normalize_legacy_asserter();
                    }
                }
                let ids: Vec<_> = payload
                    .semantic_statements()
                    .iter()
                    .map(|statement| statement.statement_id.clone())
                    .collect();
                for id in ids {
                    let statement = payload
                        .semantic_statements()
                        .iter()
                        .find(|statement| statement.statement_id == id)
                        .expect("claim present");
                    if let Some((old_pair, old_statement)) =
                        seen.insert(id.clone(), (pair, statement.clone()))
                        && (old_pair != pair || old_statement != *statement)
                    {
                        return Err(LegacyMigrationError(format!(
                            "conflicting legacy statement handle {id}"
                        )));
                    }
                    if !self.origins.contains_key(&id) {
                        let origin = self.first_origin(pair, &payload, &id, baseline)?;
                        self.origins.insert(id.clone(), origin);
                    }
                    let origin = &self.origins[&id];
                    if origin.surface != pair {
                        return Err(LegacyMigrationError(format!(
                            "legacy handle {id} changed original endpoints"
                        )));
                    }
                    if origin.identity != ClaimIdentity::from(statement) {
                        return Err(LegacyMigrationError(format!(
                            "legacy handle {id} changed its original claim identity"
                        )));
                    }
                    if let Some(resource) = origin.resource
                        && (self.active.contains_key(&id) || !old_ids.contains(&id))
                    {
                        let mut claim = EdgePayload::new();
                        claim.semantic = payload.semantic.clone();
                        let semantic = claim.semantic.as_mut().expect("semantic claim");
                        semantic
                            .statements
                            .retain(|statement| statement.statement_id == id);
                        semantic.rebuild_compat();
                        next.entry(id)
                            .or_insert_with(|| Membership {
                                surface: pair,
                                resource,
                                edges: Vec::new(),
                            })
                            .edges
                            .push(persisted_edge_for_ids(resource.0, resource.1, &claim));
                    }
                }
            }
        }
        self.active = next;
        // Remove surface authority first, before the exact resource handle admission check.
        for pair in before
            .keys()
            .chain(after.keys())
            .copied()
            .collect::<BTreeSet<_>>()
        {
            let edges = after.get(&pair).map(Vec::as_slice).unwrap_or(&[]);
            let filtered = filter_edges(pair, edges, |id| {
                self.origins
                    .get(id)
                    .is_some_and(|origin| origin.resource.is_some())
            });
            let (Some(from), Some(to)) = (
                self.output.get_node_key_by_id(pair.0),
                self.output.get_node_key_by_id(pair.1),
            ) else {
                continue;
            };
            if self.output.persisted_edges_between(from, to) != filtered {
                // Surface restore inserts in petgraph order; reverse to retain the read order.
                self.output.set_edges_between(
                    from,
                    to,
                    &filtered.iter().rev().cloned().collect::<Vec<_>>(),
                );
            }
        }
        let affected: BTreeSet<_> = previous
            .keys()
            .chain(self.active.keys())
            .filter(|id| previous.get(*id) != self.active.get(*id))
            .flat_map(|id| {
                previous
                    .get(id)
                    .into_iter()
                    .chain(self.active.get(id))
                    .map(|member| member.resource)
            })
            .collect();
        for pair in affected {
            let old_ids: BTreeSet<_> = previous
                .iter()
                .filter(|(_, member)| member.resource == pair)
                .map(|(id, _)| id.clone())
                .collect();
            let existing = self.output.persisted_resource_edges_between(pair.0, pair.1);
            let mut desired = filter_edges(pair, &existing, |id| old_ids.contains(id));
            desired.extend(
                self.active
                    .values()
                    .filter(|member| member.resource == pair)
                    .flat_map(|member| member.edges.iter().cloned()),
            );
            if desired != existing
                && !self
                    .output
                    .set_resource_edges_between(pair.0, pair.1, &desired)
            {
                return Err(LegacyMigrationError(format!(
                    "resource handle collision on {} -> {}",
                    pair.0, pair.1
                )));
            }
        }
        // Raw legacy lifecycle has no shown-resource column. Bind its current pages explicitly.
        let surfaces: Vec<_> = self.source.nodes().map(|(_, node)| node.id).collect();
        for surface in surfaces {
            let source_key = self
                .source
                .get_node_key_by_id(surface)
                .expect("surface exists");
            if self.source.shown_resource_id(source_key).is_some() {
                continue;
            }
            let (resource, _) = self.ensure_resource(surface)?;
            self.output.set_shown_resource(surface, Some(resource));
        }
        Ok(())
    }
}

fn parse_id(id: &str) -> Result<Uuid, LegacyMigrationError> {
    Uuid::parse_str(id).map_err(|error| LegacyMigrationError(error.to_string()))
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
        .filter_map(|edge| edge.semantic.as_ref())
        .flat_map(|semantic| &semantic.statements)
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
                .semantic_statements()
                .iter()
                .filter(|statement| remove(&statement.statement_id))
                .map(|statement| statement.statement_id.clone())
                .collect();
            for id in ids {
                payload.retract_semantic_statement(&id);
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

fn effects_between(
    before: &GraphSnapshot,
    after: &GraphSnapshot,
) -> Result<Vec<CapturedDelta>, LegacyMigrationError> {
    let mut effects = Vec::new();
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
    for id in before_records
        .keys()
        .chain(after_records.keys())
        .copied()
        .collect::<BTreeSet<_>>()
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

#[cfg(test)]
mod tests {
    use super::super::{
        Author, EdgeAssertion, GraphJournal, ImportedSubKind, ProvenanceSubKind, SemanticStatement,
        SemanticSubKind, Seq, predicate_iri, replay_captured_deltas_onto,
    };
    use super::*;
    use crate::types::GraphScope;
    use euclid::default::Point2D;

    fn baseline(urls: &[&str]) -> Graph {
        let mut graph = Graph::new();
        for (index, url) in urls.iter().enumerate() {
            graph.add_node_with_id(
                Uuid::from_u128(index as u128 + 1),
                (*url).into(),
                Point2D::zero(),
            );
        }
        graph
    }

    fn payload(id: &str, predicate: &str, source: Option<&str>) -> EdgePayload {
        let mut payload = EdgePayload::new();
        payload.push_persisted_semantic_statement(SemanticStatement {
            statement_id: id.into(),
            predicate: predicate.into(),
            recognized_sub_kind: super::super::sub_kind_from_iri(predicate),
            label: Some(format!("label-{id}")),
            graph_scope: GraphScope::User,
            provenance_iri: source.map(str::to_owned),
            asserted_at_ms: Some(100),
        });
        payload
    }

    fn exact(from: u128, to: u128, payload: &EdgePayload) -> CapturedDelta {
        let pair = (Uuid::from_u128(from), Uuid::from_u128(to));
        CapturedDelta::ReplaySetEdgesByIds {
            from_id: pair.0.to_string(),
            to_id: pair.1.to_string(),
            edges: vec![persisted_edge_for_ids(pair.0, pair.1, payload)],
        }
    }

    fn clear(from: u128, to: u128) -> CapturedDelta {
        CapturedDelta::ReplaySetEdgesByIds {
            from_id: Uuid::from_u128(from).to_string(),
            to_id: Uuid::from_u128(to).to_string(),
            edges: vec![],
        }
    }

    fn resource_pair(a: &str, b: &str) -> Pair {
        (ResourceNode::new(a).id(), ResourceNode::new(b).id())
    }

    #[test]
    fn legacy_resource_migration_preserves_baseline_handles_and_uncertainty_with_surface_controls()
    {
        let mut baseline = baseline(&["https://a.test/current", "https://b.test/current"]);
        let mut held = payload("unknown", predicate_iri(SemanticSubKind::Cites), None);
        held.push_persisted_semantic_statement(SemanticStatement {
            statement_id: "group".into(),
            predicate: predicate_iri(SemanticSubKind::UserGrouped).into(),
            recognized_sub_kind: Some(SemanticSubKind::UserGrouped),
            label: Some("surface control".into()),
            graph_scope: GraphScope::User,
            provenance_iri: Some("https://group-author.test/".into()),
            asserted_at_ms: Some(9),
        });
        held.assert_relation(EdgeAssertion::Imported {
            sub_kind: ImportedSubKind::BookmarkFolder,
        });
        held.assert_relation(EdgeAssertion::Provenance {
            sub_kind: ProvenanceSubKind::ClippedFrom,
        });
        let a = baseline.get_node_key_by_id(Uuid::from_u128(1)).unwrap();
        let b = baseline.get_node_key_by_id(Uuid::from_u128(2)).unwrap();
        baseline.inner.connect(a, b, held.clone());
        let before = baseline.to_snapshot();
        let replay = migrate_legacy_prefix(&baseline, &[]).unwrap();
        assert_eq!(
            baseline.to_snapshot().edges,
            before.edges,
            "input remains exact"
        );
        let pair = resource_pair("https://a.test/current", "https://b.test/current");
        let claims = replay
            .graph
            .persisted_resource_edges_between(pair.0, pair.1);
        assert_eq!(statement_ids(&claims), BTreeSet::from(["unknown".into()]));
        let claim = &claims[0].semantic.as_ref().unwrap().statements[0];
        assert_eq!(claim.asserted_at_ms, Some(100));
        assert_eq!(
            claim.provenance_iri.as_deref(),
            Some(super::super::edge_data::UNKNOWN_LEGACY_ASSERTER_IRI)
        );
        let held_surface = replay.graph.persisted_edges_between(a, b);
        assert_eq!(
            statement_ids(&held_surface),
            BTreeSet::from(["group".into()])
        );
        assert_eq!(held_surface[0].imported, before.edges[0].imported);
        assert_eq!(
            held_surface[0].provenance, before.edges[0].provenance,
            "resource-nature aggregate retained pending identity ruling"
        );
        assert_eq!(
            held_surface[0].semantic.as_ref().unwrap().statements[0]
                .provenance_iri
                .as_deref(),
            Some("https://group-author.test/")
        );
        let record = replay.graph.resource_record(pair.0).unwrap();
        let notes: Vec<LegacyMigrationNote> = serde_json::from_str(
            &record
                .facets
                .iter()
                .find(|facet| facet.facet == LEGACY_RESOURCE_MIGRATION_FACET)
                .unwrap()
                .value_json,
        )
        .unwrap();
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].statement, *claim);
        assert_eq!(notes[0].from_surface_id, Uuid::from_u128(1).to_string());
        assert_eq!(notes[0].to_surface_id, Uuid::from_u128(2).to_string());
        assert_eq!(
            notes[0].uncertainty,
            "baseline_current_resources_without_mint_event"
        );
        assert_eq!(replay.graph.shown_resource_id(a), Some(pair.0));
        let mut effective = baseline.clone();
        replay_captured_deltas_onto(&mut effective, replay.baseline_effects);
        assert_eq!(
            effective.to_snapshot().resource_edges,
            replay.graph.to_snapshot().resource_edges
        );
        assert_eq!(
            effective.to_snapshot().edges,
            replay.graph.to_snapshot().edges
        );
    }

    #[test]
    fn legacy_resource_migration_lifetime_and_pair_membership_survive_navigation_and_restore() {
        let baseline = baseline(&[
            "https://a.test/old",
            "https://a.test/old",
            "https://b.test/",
        ]);
        let alice = Author::person("alice");
        let bob = Author::person("bob");
        let mut journal = GraphJournal::new();
        let predicate = predicate_iri(SemanticSubKind::Cites);
        journal.record_as(alice.clone(), exact(1, 3, &payload("one", predicate, None)));
        journal.record_as(bob.clone(), exact(2, 3, &payload("two", predicate, None)));
        journal.record(CapturedDelta::ReplaySetNodeUrlById {
            node_id: Uuid::from_u128(1).to_string(),
            new_url: "https://a.test/new".into(),
        });
        journal.record(clear(1, 3));
        let pair = resource_pair("https://a.test/old", "https://b.test/");
        let withdrawn = journal
            .migrated_prefix_at_from(&baseline, journal.live_cursor())
            .unwrap()
            .unwrap();
        assert_eq!(
            statement_ids(
                &withdrawn
                    .graph
                    .persisted_resource_edges_between(pair.0, pair.1)
            ),
            BTreeSet::from(["two".into()]),
            "other original surface pair survives withdrawal"
        );
        let checkpoint_cursor = journal.live_cursor();
        let mut checkpoint = journal
            .snapshot_at_from(&baseline, checkpoint_cursor)
            .unwrap();
        journal.record_as(
            Author::person("restorer"),
            exact(1, 3, &payload("one", predicate, None)),
        );
        let replay = journal
            .migrated_prefix_at_from(&baseline, journal.live_cursor())
            .unwrap()
            .unwrap();
        let claims = replay
            .graph
            .persisted_resource_edges_between(pair.0, pair.1);
        assert_eq!(
            statement_ids(&claims),
            BTreeSet::from(["one".into(), "two".into()])
        );
        for (id, author) in [("one", alice), ("two", bob)] {
            let statement = claims
                .iter()
                .flat_map(|edge| &edge.semantic.as_ref().unwrap().statements)
                .find(|statement| statement.statement_id == id)
                .unwrap();
            assert_eq!(statement.asserted_at_ms, Some(100));
            assert_eq!(statement.provenance_iri, Some(author.asserter_iri()));
        }
        let new = ResourceNode::new("https://a.test/new").id();
        assert!(
            replay
                .graph
                .persisted_resource_edges_between(new, pair.1)
                .is_empty(),
            "restored handle does not follow navigation"
        );
        assert_eq!(
            replay
                .graph
                .shown_resource_id(replay.graph.get_node_key_by_id(Uuid::from_u128(1)).unwrap()),
            Some(new)
        );
        assert!(
            replay
                .graph
                .to_snapshot()
                .edges
                .iter()
                .all(|edge| edge.semantic.is_none())
        );
        journal
            .migrated_replay_from_with_baseline(checkpoint_cursor, &mut checkpoint, &baseline)
            .unwrap();
        assert_eq!(
            checkpoint.to_snapshot().resource_edges,
            replay.graph.to_snapshot().resource_edges
        );
        let mut effective = baseline.clone();
        replay_captured_deltas_onto(
            &mut effective,
            replay
                .baseline_effects
                .into_iter()
                .chain(replay.entry_effects.into_iter().flatten()),
        );
        assert_eq!(
            effective.to_snapshot().resource_edges,
            replay.graph.to_snapshot().resource_edges
        );
        assert!(
            journal
                .migrated_snapshot_at_from(&baseline, Seq(journal.live_cursor().0 + 1))
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn legacy_resource_migration_typed_pair_owns_only_its_pair_and_is_not_repopulated() {
        let baseline = baseline(&[
            "https://a.test/",
            "https://b.test/",
            "https://c.test/",
            "https://d.test/",
        ]);
        let mut journal = GraphJournal::new();
        let predicate = predicate_iri(SemanticSubKind::Cites);
        journal.record(exact(
            1,
            2,
            &payload("old-a", predicate, Some("https://author.test/")),
        ));
        journal.record(exact(
            3,
            4,
            &payload("old-c", predicate, Some("https://author.test/")),
        ));
        let first = resource_pair("https://a.test/", "https://b.test/");
        let second = resource_pair("https://c.test/", "https://d.test/");
        let typed = persisted_edge_for_ids(
            first.0,
            first.1,
            &payload("typed", predicate, Some("https://typed-author.test/")),
        );
        journal.record(CapturedDelta::ReplaySetResourceEdgesByIds {
            from_resource_id: first.0.to_string(),
            to_resource_id: first.1.to_string(),
            edges: vec![typed.clone()],
        });
        let mut update = payload("old-c", predicate, Some("https://author.test/"));
        update.semantic.as_mut().unwrap().statements[0].label = Some("later unrelated edit".into());
        journal.record(exact(3, 4, &update));
        let replay = journal
            .migrated_prefix_at_from(&baseline, journal.live_cursor())
            .unwrap()
            .unwrap();
        assert_eq!(
            replay
                .graph
                .persisted_resource_edges_between(first.0, first.1),
            vec![typed]
        );
        assert_eq!(
            statement_ids(
                &replay
                    .graph
                    .persisted_resource_edges_between(second.0, second.1)
            ),
            BTreeSet::from(["old-c".into()])
        );
        assert!(
            replay
                .graph
                .to_snapshot()
                .edges
                .iter()
                .all(|edge| edge.semantic.is_none())
        );
        journal.record(CapturedDelta::ReplaySetResourceEdgesByIds {
            from_resource_id: first.0.to_string(),
            to_resource_id: first.1.to_string(),
            edges: vec![],
        });
        journal.record(exact(3, 4, &update));
        let cleared = journal
            .migrated_snapshot_at_from(&baseline, journal.live_cursor())
            .unwrap()
            .unwrap();
        assert!(
            cleared
                .persisted_resource_edges_between(first.0, first.1)
                .is_empty(),
            "unrelated old membership cannot undo explicit typed clear"
        );
        assert_eq!(
            statement_ids(&cleared.persisted_resource_edges_between(second.0, second.1)),
            BTreeSet::from(["old-c".into()])
        );
    }

    #[test]
    fn legacy_resource_migration_authored_overrides_apply_at_mint_and_conflicts_keep_typed_history()
    {
        let baseline = baseline(&["https://a.test/", "https://b.test/"]);
        let predicate = "https://vocab.test/p#custom";
        let mut declarations = Graph::new();
        declarations
            .declare_predicate(predicate, GraphStratum::Surface)
            .unwrap();
        let id = ResourceNode::for_term(predicate).id();
        let surface = declarations.resource_record(id).unwrap();
        declarations
            .declare_predicate(predicate, GraphStratum::Resource)
            .unwrap();
        let resource = declarations.resource_record(id).unwrap();
        let mut journal = GraphJournal::new();
        journal.record(CapturedDelta::ReplaySetResourceRecordById {
            resource_id: id.to_string(),
            record: Some(surface),
        });
        journal.record(exact(
            1,
            2,
            &payload("surface-mint", predicate, Some("https://author.test/")),
        ));
        journal.record(CapturedDelta::ReplaySetResourceRecordById {
            resource_id: id.to_string(),
            record: Some(resource),
        });
        let mut both = payload("surface-mint", predicate, Some("https://author.test/"));
        both.push_persisted_semantic_statement(SemanticStatement {
            statement_id: "resource-mint".into(),
            predicate: predicate.into(),
            recognized_sub_kind: None,
            label: None,
            graph_scope: GraphScope::User,
            provenance_iri: Some("https://other.test/".into()),
            asserted_at_ms: Some(200),
        });
        journal.record(exact(1, 2, &both));
        let migrated = journal
            .migrated_snapshot_at_from(&baseline, journal.live_cursor())
            .unwrap()
            .unwrap();
        assert_eq!(
            statement_ids(&migrated.to_snapshot().edges),
            BTreeSet::from(["surface-mint".into()])
        );
        assert_eq!(
            statement_ids(&migrated.to_snapshot().resource_edges),
            BTreeSet::from(["resource-mint".into()])
        );
        let mut conflicted = migrated.clone();
        let mut record = conflicted.resource_record(id).unwrap();
        let facet = record
            .facets
            .iter_mut()
            .find(|facet| facet.facet == PREDICATE_DECLARATIONS_FACET)
            .unwrap();
        let mut declarations: super::super::PredicateDeclarations =
            serde_json::from_str(&facet.value_json).unwrap();
        declarations.selected = None;
        facet.value_json = serde_json::to_string(&declarations).unwrap();
        assert!(conflicted.set_resource_record(id, Some(record)));
        assert!(conflicted.effective_predicate_stratum(predicate).is_err());
        // Only caller-qualified raw legacy bytes are adapted; typed claims keep their store.
        let mut typed_only = baseline.clone();
        for record in &conflicted.to_snapshot().resources {
            let resource = ResourceNode::for_term(&record.canonical_iri).id();
            typed_only.set_resource_record(resource, Some(record.clone()));
        }
        let pair = resource_pair("https://a.test/", "https://b.test/");
        let typed = conflicted.persisted_resource_edges_between(pair.0, pair.1);
        assert!(typed_only.set_resource_edges_between(pair.0, pair.1, &typed));
        let replay = migrate_legacy_prefix(&typed_only, &[]).unwrap();
        assert_eq!(
            replay
                .graph
                .persisted_resource_edges_between(pair.0, pair.1),
            typed
        );
        assert!(replay.graph.effective_predicate_stratum(predicate).is_err());
    }

    #[test]
    fn legacy_resource_migration_handle_collision_is_atomic_with_distinct_handle_control() {
        let mut baseline = baseline(&["https://a.test/", "https://b.test/"]);
        let a = baseline.get_node_key_by_id(Uuid::from_u128(1)).unwrap();
        let b = baseline.get_node_key_by_id(Uuid::from_u128(2)).unwrap();
        let predicate = predicate_iri(SemanticSubKind::Cites);
        baseline.inner.connect(
            a,
            b,
            payload("collision", predicate, Some("https://legacy.test/")),
        );
        let resources = [
            ResourceNode::for_term("https://other.test/a"),
            ResourceNode::for_term("https://other.test/b"),
        ];
        for resource in &resources {
            assert!(baseline.set_resource_record(
                resource.id(),
                Some(PersistedResourceRecord {
                    canonical_iri: resource.canonical_iri().into(),
                    facets: vec![]
                })
            ));
        }
        let pair = (resources[0].id(), resources[1].id());
        // Deliberately malformed in-memory input bypasses the checked setter.
        let from = baseline.resources.key_of(&pair.0).unwrap();
        let to = baseline.resources.key_of(&pair.1).unwrap();
        baseline.resources.connect(
            from,
            to,
            payload("collision", predicate, Some("https://typed.test/")),
        );
        let before = baseline.to_snapshot();
        assert!(migrate_legacy_prefix(&baseline, &[]).is_err());
        assert_eq!(baseline.to_snapshot().edges, before.edges);
        assert_eq!(baseline.to_snapshot().resource_edges, before.resource_edges);
        let keys: Vec<_> = baseline.resources.inner().edge_indices().collect();
        for key in keys {
            baseline.resources.disconnect(key);
        }
        assert!(baseline.set_resource_edges_between(
            pair.0,
            pair.1,
            &[persisted_edge_for_ids(
                pair.0,
                pair.1,
                &payload("distinct", predicate, Some("https://typed.test/"))
            )]
        ));
        let valid = migrate_legacy_prefix(&baseline, &[]).unwrap();
        assert_eq!(
            statement_ids(&valid.graph.to_snapshot().resource_edges),
            BTreeSet::from(["collision".into(), "distinct".into()])
        );
    }

    #[test]
    fn legacy_resource_migration_claim_identity_survives_absence_and_allows_metadata_updates() {
        let baseline = baseline(&["https://a.test/", "https://b.test/"]);
        let predicate = predicate_iri(SemanticSubKind::Cites);
        let held = payload("held", predicate, Some("https://author.test/"));
        let mut journal = GraphJournal::new();
        journal.record(exact(1, 2, &held));
        journal.record(clear(1, 2));
        let mut metadata = held.clone();
        let statement = &mut metadata.semantic.as_mut().unwrap().statements[0];
        statement.label = Some("new label".into());
        statement.asserted_at_ms = Some(300);
        journal.record(exact(1, 2, &metadata));
        let valid = journal
            .migrated_snapshot_at_from(&baseline, journal.live_cursor())
            .unwrap()
            .unwrap();
        let valid_snapshot = valid.to_snapshot();
        let statement = &valid_snapshot.resource_edges[0]
            .semantic
            .as_ref()
            .unwrap()
            .statements[0];
        assert_eq!(statement.statement_id, "held");
        assert_eq!(statement.label.as_deref(), Some("new label"));
        assert_eq!(statement.asserted_at_ms, Some(300));
        for change in 0..4 {
            let mut conflict = held.clone();
            let statement = &mut conflict.semantic.as_mut().unwrap().statements[0];
            match change {
                0 => statement.predicate = "https://vocab.test/changed".into(),
                1 => statement.recognized_sub_kind = Some(SemanticSubKind::Quotes),
                2 => statement.graph_scope = GraphScope::Default,
                _ => statement.provenance_iri = Some("https://changed-author.test/".into()),
            }
            let mut divergent = journal.clone();
            divergent.record(clear(1, 2));
            divergent.record(exact(1, 2, &conflict));
            let source_before = serde_json::to_vec(divergent.entries()).unwrap();
            let mut checkpoint = valid.clone();
            let before = checkpoint.to_snapshot();
            assert!(
                divergent
                    .migrated_replay_from_with_baseline(Seq(0), &mut checkpoint, &baseline)
                    .is_err()
            );
            assert_eq!(
                checkpoint.to_snapshot().resource_edges,
                before.resource_edges
            );
            assert_eq!(
                serde_json::to_vec(divergent.entries()).unwrap(),
                source_before
            );
            conflict.semantic.as_mut().unwrap().statements[0].statement_id =
                format!("distinct-{change}");
            let mut distinct = journal.clone();
            distinct.record(exact(1, 2, &conflict));
            assert!(
                distinct
                    .migrated_snapshot_at_from(&baseline, distinct.live_cursor())
                    .unwrap()
                    .is_some(),
                "distinct-handle control"
            );
        }
    }

    #[test]
    fn legacy_resource_migration_keeps_identical_parallel_handle_copies() {
        let mut baseline = baseline(&["https://a.test/", "https://b.test/"]);
        let a = baseline.get_node_key_by_id(Uuid::from_u128(1)).unwrap();
        let b = baseline.get_node_key_by_id(Uuid::from_u128(2)).unwrap();
        let claim = payload(
            "copied",
            predicate_iri(SemanticSubKind::Cites),
            Some("https://author.test/"),
        );
        baseline.inner.connect(a, b, claim.clone());
        baseline.inner.connect(a, b, claim);
        let migrated = migrate_legacy_prefix(&baseline, &[]).unwrap();
        let pair = resource_pair("https://a.test/", "https://b.test/");
        let copies = migrated
            .graph
            .persisted_resource_edges_between(pair.0, pair.1);
        assert_eq!(copies.len(), 2);
        assert_eq!(copies[0], copies[1]);
        assert_eq!(statement_ids(&copies), BTreeSet::from(["copied".into()]));
        assert!(migrated.graph.persisted_edges_between(a, b).is_empty());
        let mut invalid = baseline.clone();
        let key = invalid.find_edge_key(a, b).unwrap();
        invalid
            .get_edge_mut(key)
            .unwrap()
            .semantic
            .as_mut()
            .unwrap()
            .statements[0]
            .label = Some("conflicting copy".into());
        assert!(migrate_legacy_prefix(&invalid, &[]).is_err());
        assert_eq!(baseline.persisted_edges_between(a, b).len(), 2);
    }

    #[test]
    fn legacy_resource_migration_new_handles_honor_selected_baseline_declarations() {
        let mut baseline = baseline(&["https://a.test/", "https://b.test/"]);
        let surface_predicate = "https://vocab.test/surface";
        baseline
            .declare_predicate(surface_predicate, GraphStratum::Surface)
            .unwrap();
        let mut journal = GraphJournal::new();
        journal.record(exact(
            1,
            2,
            &payload(
                "new-surface",
                surface_predicate,
                Some("https://author.test/"),
            ),
        ));
        let mut combined = payload(
            "new-surface",
            surface_predicate,
            Some("https://author.test/"),
        );
        combined.push_persisted_semantic_statement(SemanticStatement {
            statement_id: "resource-control".into(),
            predicate: "https://vocab.test/default".into(),
            recognized_sub_kind: None,
            label: None,
            graph_scope: GraphScope::User,
            provenance_iri: Some("https://author.test/".into()),
            asserted_at_ms: Some(200),
        });
        journal.record(exact(1, 2, &combined));
        let graph = journal
            .migrated_snapshot_at_from(&baseline, journal.live_cursor())
            .unwrap()
            .unwrap();
        assert_eq!(
            statement_ids(&graph.to_snapshot().edges),
            BTreeSet::from(["new-surface".into()])
        );
        assert_eq!(
            statement_ids(&graph.to_snapshot().resource_edges),
            BTreeSet::from(["resource-control".into()])
        );
        assert_eq!(
            graph.effective_predicate_stratum(surface_predicate),
            Ok(GraphStratum::Surface)
        );
    }

    #[test]
    fn legacy_resource_migration_effective_captures_undo_claim_after_later_navigation() {
        let baseline = baseline(&[
            "https://a.test/old",
            "https://a.test/old",
            "https://b.test/",
        ]);
        let predicate = predicate_iri(SemanticSubKind::Cites);
        let mut journal = GraphJournal::new();
        journal.record(exact(
            2,
            3,
            &payload("other", predicate, Some("https://other.test/")),
        ));
        let before = journal
            .migrated_snapshot_at_from(&baseline, journal.live_cursor())
            .unwrap()
            .unwrap();
        journal.record(exact(
            1,
            3,
            &payload("mine", predicate, Some("https://author.test/")),
        ));
        let after = journal
            .migrated_prefix_at_from(&baseline, journal.live_cursor())
            .unwrap()
            .unwrap();
        let change = after.entry_effects.last().unwrap().clone();
        assert!(
            change
                .iter()
                .any(|delta| matches!(delta, CapturedDelta::ReplaySetResourceEdgesByIds { .. }))
        );
        journal.record(CapturedDelta::ReplaySetNodeUrlById {
            node_id: Uuid::from_u128(1).to_string(),
            new_url: "https://a.test/new".into(),
        });
        let mut live = journal
            .migrated_snapshot_at_from(&baseline, journal.live_cursor())
            .unwrap()
            .unwrap();
        let undo = super::super::revert_change(&change, &before, &after.graph, &live);
        assert!(undo.kept.is_empty());
        replay_captured_deltas_onto(&mut live, undo.edits);
        let pair = resource_pair("https://a.test/old", "https://b.test/");
        assert_eq!(
            statement_ids(&live.persisted_resource_edges_between(pair.0, pair.1)),
            BTreeSet::from(["other".into()])
        );
        let key = live.get_node_key_by_id(Uuid::from_u128(1)).unwrap();
        assert_eq!(
            live.get_node(key).unwrap().primary_address().as_url_str(),
            "https://a.test/new"
        );
        assert_eq!(
            live.shown_resource_id(key),
            Some(ResourceNode::new("https://a.test/new").id())
        );
    }

    #[test]
    fn legacy_resource_migration_raw_assertion_retains_author_and_historical_endpoints() {
        let baseline = baseline(&["https://a.test/old", "https://b.test/"]);
        let mut journal = GraphJournal::new();
        let author = Author::person("raw-author");
        journal.record_as(
            author.clone(),
            CapturedDelta::ReplayAssertRelationByIds {
                from_id: Uuid::from_u128(1).to_string(),
                to_id: Uuid::from_u128(2).to_string(),
                assertion: EdgeAssertion::Semantic {
                    sub_kind: SemanticSubKind::Cites,
                    label: None,
                    decay_progress: None,
                },
            },
        );
        journal.record(CapturedDelta::ReplaySetNodeUrlById {
            node_id: Uuid::from_u128(1).to_string(),
            new_url: "https://a.test/new".into(),
        });
        for _ in 0..2 {
            let graph = journal
                .migrated_snapshot_at_from(&baseline, journal.live_cursor())
                .unwrap()
                .unwrap();
            let pair = resource_pair("https://a.test/old", "https://b.test/");
            let claims = graph.persisted_resource_edges_between(pair.0, pair.1);
            assert_eq!(claims.len(), 1);
            assert_eq!(
                claims[0].semantic.as_ref().unwrap().statements[0].provenance_iri,
                Some(author.asserter_iri())
            );
            assert!(
                graph
                    .persisted_resource_edges_between(
                        ResourceNode::new("https://a.test/new").id(),
                        pair.1
                    )
                    .is_empty()
            );
            assert!(
                graph
                    .to_snapshot()
                    .edges
                    .iter()
                    .all(|edge| edge.semantic.is_none())
            );
        }
        // Raw records did not carry stable handles; repeated replay deliberately
        // makes no identity-equality promise absent from the retained history.
        assert!(
            probe_legacy_mint_links(&baseline, journal.entries())
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn legacy_resource_migration_probe_reports_mixed_raw_and_carried_evidence_without_aliasing() {
        let baseline = baseline(&["https://a.test/old", "https://b.test/"]);
        let mut journal = GraphJournal::new();
        let author = Author::person("mint-author");
        journal.record_as(
            author.clone(),
            CapturedDelta::ReplayAssertRelationByIds {
                from_id: Uuid::from_u128(1).to_string(),
                to_id: Uuid::from_u128(2).to_string(),
                assertion: EdgeAssertion::Semantic {
                    sub_kind: SemanticSubKind::Cites,
                    label: None,
                    decay_progress: None,
                },
            },
        );
        journal.record(CapturedDelta::ReplaySetNodeUrlById {
            node_id: Uuid::from_u128(1).to_string(),
            new_url: "https://a.test/new".into(),
        });
        let mut carried = payload(
            "carried",
            predicate_iri(SemanticSubKind::Cites),
            Some(&author.asserter_iri()),
        );
        carried.semantic.as_mut().unwrap().statements[0].graph_scope = GraphScope::Default;
        journal.record_as(Author::person("later-author"), exact(1, 2, &carried));
        let source_before = serde_json::to_vec(journal.entries()).unwrap();
        let diagnostics = probe_legacy_mint_links(&baseline, journal.entries()).unwrap();
        assert_eq!(diagnostics.len(), 1);
        let diagnostic = &diagnostics[0];
        assert_eq!(
            (diagnostic.raw_entry_index, diagnostic.carried_entry_index),
            (0, 2)
        );
        assert_eq!(diagnostic.statement_id, "carried");
        assert!(diagnostic.explicit_source_matches);
        assert_eq!(
            diagnostic.earlier_resource_pair.0,
            ResourceNode::new("https://a.test/old").id().to_string()
        );
        assert_eq!(
            diagnostic.current_resource_pair.0,
            ResourceNode::new("https://a.test/new").id().to_string()
        );
        assert_eq!(
            serde_json::to_vec(journal.entries()).unwrap(),
            source_before
        );
        let error = migrate_legacy_prefix(&baseline, journal.entries())
            .err()
            .expect("direct adapter must hold unresolved linkage");
        assert!(error.to_string().contains("checkpoint C28"));
        assert!(
            journal
                .migrated_prefix_at_from(&baseline, journal.live_cursor())
                .is_err()
        );
        assert!(
            journal
                .migrated_snapshot_at_from(&baseline, journal.live_cursor())
                .is_err()
        );
        let mut checkpoint = baseline.clone();
        let checkpoint_author = Author::person("checkpoint-author");
        checkpoint.write_author = checkpoint_author.clone();
        checkpoint.current_session = 55;
        let seen = std::sync::Arc::new(std::sync::Mutex::new(Vec::<CapturedDelta>::new()));
        let sink = seen.clone();
        checkpoint.set_recorder(Some(std::sync::Arc::new(move |delta| {
            sink.lock().unwrap().push(delta.clone())
        })));
        let before_checkpoint = serde_json::to_value(checkpoint.to_snapshot()).unwrap();
        let before_revision = checkpoint.revision();
        let before_url_revision = checkpoint.url_grouping_revision();
        assert!(
            journal
                .migrated_replay_from_with_baseline(Seq(0), &mut checkpoint, &baseline)
                .is_err()
        );
        let mut after_checkpoint = serde_json::to_value(checkpoint.to_snapshot()).unwrap();
        after_checkpoint["timestamp_secs"] = before_checkpoint["timestamp_secs"].clone();
        assert_eq!(
            after_checkpoint, before_checkpoint,
            "failed checkpoint replay leaves the whole graph unchanged"
        );
        assert_eq!(checkpoint.revision(), before_revision);
        assert_eq!(checkpoint.url_grouping_revision(), before_url_revision);
        assert_eq!(checkpoint.write_author, checkpoint_author);
        assert_eq!(checkpoint.current_session, 55);
        assert!(checkpoint.is_recording());
        assert!(seen.lock().unwrap().is_empty());
        assert_eq!(
            serde_json::to_vec(journal.entries()).unwrap(),
            source_before
        );
        let witness = CapturedDelta::ReplaySetNodeTitleById {
            node_id: Uuid::from_u128(1).to_string(),
            title: "recorder remains live".into(),
        };
        super::super::apply::apply_graph_delta(&mut checkpoint, witness.replay_delta().unwrap());
        assert_eq!(
            *seen.lock().unwrap(),
            vec![witness],
            "a real edit proves the retained recorder works"
        );
        let mut marker = carried.clone();
        marker.semantic.as_mut().unwrap().statements[0].provenance_iri =
            Some(super::super::edge_data::UNKNOWN_LEGACY_ASSERTER_IRI.into());
        let mut uncertain = journal.entries()[..2].to_vec();
        uncertain.push(AttributedDelta {
            author: Author::person("later-author"),
            delta: exact(1, 2, &marker),
        });
        let unknown = probe_legacy_mint_links(&baseline, &uncertain).unwrap();
        assert_eq!(
            unknown.len(),
            1,
            "unknown legacy attribution cannot suppress earlier mint evidence"
        );
        assert!(!unknown[0].explicit_source_matches);
        assert!(migrate_legacy_prefix(&baseline, &uncertain).is_err());
        marker.semantic.as_mut().unwrap().statements[0].provenance_iri =
            Some(Author::person("different-author").asserter_iri());
        uncertain[2].delta = exact(1, 2, &marker);
        assert!(
            probe_legacy_mint_links(&baseline, &uncertain)
                .unwrap()
                .is_empty(),
            "a concrete different Author is a distinct claim control"
        );
        let author_control = migrate_legacy_prefix(&baseline, &uncertain).unwrap();
        assert_eq!(
            statement_ids(&author_control.graph.to_snapshot().resource_edges),
            BTreeSet::from(["carried".into()])
        );
        // A distinct claim key is an unambiguous carried handle control.
        carried.semantic.as_mut().unwrap().statements[0].predicate =
            "https://vocab.test/distinct".into();
        carried.semantic.as_mut().unwrap().statements[0].recognized_sub_kind = None;
        let mut independent = journal.entries()[..2].to_vec();
        independent.push(AttributedDelta {
            author,
            delta: exact(1, 2, &carried),
        });
        assert!(
            probe_legacy_mint_links(&baseline, &independent)
                .unwrap()
                .is_empty()
        );
        let predicate_control = migrate_legacy_prefix(&baseline, &independent).unwrap();
        assert_eq!(
            statement_ids(&predicate_control.graph.to_snapshot().resource_edges),
            BTreeSet::from(["carried".into()])
        );
    }
}
