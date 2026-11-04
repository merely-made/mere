// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

impl Replay {
    pub(super) fn check_typed_record(
        &self,
        delta: &CapturedDelta,
    ) -> Result<(), LegacyMigrationError> {
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

    pub(super) fn ensure_resource(
        &mut self,
        surface: Uuid,
    ) -> Result<(Uuid, String), LegacyMigrationError> {
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

    pub(super) fn first_origin(
        &mut self,
        pair: Pair,
        payload: &EdgePayload,
        id: &str,
        baseline: bool,
    ) -> Result<Origin, LegacyMigrationError> {
        let statement = payload
            .assertions()
            .iter()
            .find(|statement| statement.statement_id == id)
            .expect("claim present");
        let fixed = statement
            .recognized_sub_kind
            .map(|kind| {
                super::super::built_in_relation_stratum(super::super::RelationKind::Semantic(kind))
            })
            .or_else(|| super::super::built_in_predicate_stratum(&statement.predicate));
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
                claim: statement.clone(),
            });
        }
        let (current_from, from_url) = self.ensure_resource(pair.0)?;
        let (current_to, to_url) = self.ensure_resource(pair.1)?;
        let (from, to) = self
            .resolutions
            .get(id)
            .copied()
            .unwrap_or((current_from, current_to));
        if baseline {
            let edge = persisted_edge_for_ids(pair.0, pair.1, payload);
            let statement = edge
                .retained_assertions()
                .iter()
                .find(|statement| statement.statement_id == id)
                .expect("statement serialized")
                .clone();
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
            claim: statement.clone(),
        })
    }

    pub(super) fn add_note(
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

    pub(super) fn first_aggregate_origin(
        &mut self,
        pair: Pair,
        kind: super::super::RelationKind,
        author: Option<&super::super::Author>,
    ) -> Result<Pair, LegacyMigrationError> {
        let (from, from_url) = self.ensure_resource(pair.0)?;
        let (to, to_url) = self.ensure_resource(pair.1)?;
        let note = LegacyAggregateMigrationNote {
            from_surface_id: pair.0.to_string(),
            to_surface_id: pair.1.to_string(),
            from_resource_id: from.to_string(),
            to_resource_id: to.to_string(),
            from_url,
            to_url,
            kind,
            author: author.cloned(),
            uncertainty: if author.is_some() {
                "legacy_pair_kind_identity_without_assertion_handle_or_time"
            } else {
                "baseline_current_resources_without_mint_event_handle_or_time"
            }
            .into(),
        };
        let mut record = self.output.resource_record(from).expect("resource ensured");
        let mut notes: Vec<LegacyAggregateMigrationNote> = record
            .facets
            .iter()
            .find(|facet| facet.facet == LEGACY_AGGREGATE_MIGRATION_FACET)
            .map(|facet| serde_json::from_str(&facet.value_json))
            .transpose()
            .map_err(|error| {
                LegacyMigrationError(format!("invalid aggregate migration note: {error}"))
            })?
            .unwrap_or_default();
        if notes.contains(&note) {
            return Ok((from, to));
        }
        notes.push(note);
        record
            .facets
            .retain(|facet| facet.facet != LEGACY_AGGREGATE_MIGRATION_FACET);
        record.facets.push(PersistedResourceFacet {
            facet: LEGACY_AGGREGATE_MIGRATION_FACET.into(),
            value_json: serde_json::to_string(&notes).expect("aggregate note serializes"),
        });
        if !self.output.set_resource_record(from, Some(record)) {
            return Err(LegacyMigrationError(
                "aggregate migration note insertion was refused".into(),
            ));
        }
        Ok((from, to))
    }

    pub(super) fn sync(
        &mut self,
        before: &Pairs,
        baseline: bool,
        entry: Option<&AttributedDelta>,
    ) -> Result<(), LegacyMigrationError> {
        let after = surface_pairs(&self.source)?;
        let previous = self.active.clone();
        let aggregate_previous = self.aggregate_active.clone();
        let present = statement_ids(&after.values().flatten().cloned().collect::<Vec<_>>());
        self.active.retain(|id, _| present.contains(id));
        let mut seen = BTreeMap::new();
        let mut next = BTreeMap::<String, Membership>::new();
        let mut aggregate_next = BTreeMap::<(Pair, u32), Membership>::new();
        for (&pair, edges) in &after {
            let old_ids = statement_ids(before.get(&pair).map(Vec::as_slice).unwrap_or(&[]));
            let old_kinds: BTreeSet<_> = before
                .get(&pair)
                .into_iter()
                .flatten()
                .flat_map(|edge| aggregate_kinds(&payload_from_persisted(edge)))
                .map(super::super::RelationKind::tag)
                .collect();
            for edge in edges {
                let mut payload = payload_from_persisted(edge);
                for kind in aggregate_kinds(&payload).into_iter().filter(|kind| {
                    super::super::built_in_relation_stratum(*kind) == GraphStratum::Resource
                }) {
                    let key = (pair, kind.tag());
                    if !self.aggregate_origins.contains_key(&key) {
                        let proven_author = entry
                            .filter(|entry| delta_introduces_member(&entry.delta, pair, kind))
                            .map(|entry| &entry.author);
                        let resource = self.first_aggregate_origin(pair, kind, proven_author)?;
                        self.aggregate_origins.insert(key, resource);
                    }
                    if self.aggregate_active.contains_key(&key) || !old_kinds.contains(&kind.tag())
                    {
                        let resource = self.aggregate_origins[&key];
                        let mut claim = EdgePayload::new();
                        // The prefix marks the deterministic record created
                        // when an old family sidecar is decoded. Lift that
                        // exact record into its Resource owner; minting a new
                        // membership assertion here loses identity and makes
                        // two parallel Surface owners indistinguishable.
                        let statement = payload
                            .assertions()
                            .iter()
                            .find(|statement| {
                                statement.is_held()
                                    && statement.relation_kind() == kind
                                    && statement.statement_id.starts_with(
                                        super::super::legacy_family_attribution::LEGACY_FAMILY_ASSERTION_ID_PREFIX,
                                    )
                            })
                            .expect("legacy aggregate kind has its deterministic source record")
                            .clone();
                        claim.assertions.statements.push(statement);
                        claim.assertions.rebuild_compat();
                        aggregate_next
                            .entry(key)
                            .or_insert_with(|| Membership {
                                surface: pair,
                                resource,
                                edges: Vec::new(),
                            })
                            .edges
                            .push(persisted_edge_for_ids(resource.0, resource.1, &claim));
                    }
                }
                {
                    let semantic = &mut payload.assertions;
                    for statement in &mut semantic.statements {
                        statement.normalize_legacy_asserter();
                    }
                }
                let ids: Vec<_> = payload
                    .assertions()
                    .iter()
                    .map(|statement| statement.statement_id.clone())
                    .collect();
                for id in ids {
                    let statement = payload
                        .assertions()
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
                    if !origin.claim.same_claim_as(statement) {
                        return Err(LegacyMigrationError(format!(
                            "legacy handle {id} changed its original claim identity"
                        )));
                    }
                    if let Some(resource) = origin.resource
                        // Exact records minted from canonical input keep their
                        // carried IDs in the Resource stratum. The only records
                        // handled by the pair+kind compatibility migration are
                        // IDs synthesized while decoding pre-canonical family
                        // sidecars; those have no source-owned assertion handle.
                        && !(id.starts_with(super::super::legacy_family_attribution::LEGACY_FAMILY_ASSERTION_ID_PREFIX)
                            && aggregate_kinds(&payload)
                                .contains(&statement.relation_kind()))
                        && (self.active.contains_key(&id) || !old_ids.contains(&id))
                    {
                        let mut claim = EdgePayload::new();
                        claim.assertions = payload.assertions.clone();
                        let semantic = &mut claim.assertions;
                        semantic
                            .statements
                            .retain(|statement| statement.statement_id == id);
                        semantic.rebuild_compat();
                        let edge = persisted_edge_for_ids(resource.0, resource.1, &claim);
                        let held = next.entry(id).or_insert_with(|| Membership {
                            surface: pair,
                            resource,
                            edges: Vec::new(),
                        });
                        if held.edges.contains(&edge) {
                            self.note_deduped_copy(pair, resource, &edge)?;
                        } else {
                            held.edges.push(edge);
                        }
                    }
                }
            }
        }
        self.active = next;
        self.aggregate_active = aggregate_next;
        // Staged: the replay arms' edge check waits for the settle. Remove surface
        // authority first anyway, before the exact resource handle admission check.
        for pair in before
            .keys()
            .chain(after.keys())
            .copied()
            .collect::<BTreeSet<_>>()
        {
            let edges = after.get(&pair).map(Vec::as_slice).unwrap_or(&[]);
            let semantic_filtered = filter_edges(pair, edges, |id| {
                self.origins
                    .get(id)
                    .is_some_and(|origin| origin.resource.is_some())
            });
            let (filtered, copies) =
                copies::split_copies(filter_resource_aggregates(pair, &semantic_filtered));
            let (Some(from), Some(to)) = (
                self.output.get_node_key_by_id(pair.0),
                self.output.get_node_key_by_id(pair.1),
            ) else {
                continue;
            };
            for copy in &copies {
                self.note_deduped_surface_copy(pair, copy)?;
            }
            if self.output.persisted_edges_between(from, to) != filtered {
                // Surface restore inserts in petgraph order; reverse to retain the read order.
                self.output.set_edges_between(
                    from,
                    to,
                    &filtered.iter().rev().cloned().collect::<Vec<_>>(),
                );
            }
        }
        let mut affected: BTreeSet<_> = previous
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
        affected.extend(
            aggregate_previous
                .keys()
                .chain(self.aggregate_active.keys())
                .filter(|key| aggregate_previous.get(*key) != self.aggregate_active.get(*key))
                .flat_map(|key| {
                    aggregate_previous
                        .get(key)
                        .into_iter()
                        .chain(self.aggregate_active.get(key))
                        .map(|member| member.resource)
                }),
        );
        for pair in affected {
            let existing = self.output.persisted_resource_edges_between(pair.0, pair.1);
            let mut desired = existing.clone();
            // Remove only copies this migration previously placed. A typed
            // checkpoint may have replaced the pair since then; preserve its
            // independently owned records while reconciling the legacy source.
            for edge in previous
                .values()
                .filter(|member| member.resource == pair)
                .flat_map(|member| &member.edges)
            {
                if let Some(index) = desired.iter().position(|held| held == edge) {
                    desired.remove(index);
                }
            }
            for edge in aggregate_previous
                .values()
                .filter(|member| member.resource == pair)
                .flat_map(|member| &member.edges)
            {
                if let Some(index) = desired.iter().position(|held| held == edge) {
                    desired.remove(index);
                }
            }
            desired.extend(
                self.active
                    .values()
                    .filter(|member| member.resource == pair)
                    .flat_map(|member| member.edges.iter().cloned()),
            );
            desired.extend(
                self.aggregate_active
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
        self.dedupe_resource_copies()?;
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
