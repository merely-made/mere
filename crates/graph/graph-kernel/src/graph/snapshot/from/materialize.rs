// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

impl Graph {
    /// Load a snapshot, stopping on invalid explicit resource data or
    /// navigation indices. Fallible load boundaries should use
    /// [`Self::try_from_snapshot`].
    pub fn from_snapshot(snapshot: &GraphSnapshot) -> Self {
        Self::try_from_snapshot(snapshot)
            .unwrap_or_else(|error| panic!("invalid graph snapshot: {error}"))
    }

    pub(in super::super) fn from_snapshot_unchecked(snapshot: &GraphSnapshot) -> Self {
        Self::materialize_snapshot(snapshot, true)
    }

    pub(in super::super) fn from_recorded_snapshot_unchecked(snapshot: &GraphSnapshot) -> Self {
        Self::materialize_snapshot(snapshot, false)
    }

    pub(super) fn materialize_snapshot(
        snapshot: &GraphSnapshot,
        legacy_derived_containment: bool,
    ) -> Self {
        let mut graph = Graph::new();
        // Restore the shared navigation history (one visit space, owner per node)
        // before the node loop, so each node's restored current page reads from it.
        graph.nav = snapshot.navigation.clone();

        // Checked load has already refused an unparseable or repeated Surface id.
        for pnode in &snapshot.nodes {
            let Ok(node_id) = Uuid::parse_str(&pnode.node_id) else {
                continue;
            };
            // Prefer the typed address field (Stage C.2+); fall back to the legacy
            // `url` string field for snapshots written before Stage C.2.
            let node_url = {
                let from_address = pnode.address.as_url_str();
                if from_address.is_empty() {
                    pnode.url.clone()
                } else {
                    from_address.to_string()
                }
            };
            // Positions are no longer graph truth; they live in the cartography
            // sidecar (the Cartography projection's geometry). Seed at the origin and
            // let the sidecar override on load. (Position gut.)
            let (key, inserted) =
                graph.insert_node_if_missing(node_id, node_url, Point2D::new(0.0, 0.0));
            if !inserted {
                continue;
            }
            if let Some(node) = graph.inner.node_mut(key) {
                if node.title != pnode.title {
                    graph.content_revision = graph.content_revision.wrapping_add(1);
                }
                node.title = pnode.title.clone();
                node.tags = pnode.tags.iter().cloned().collect();
                // References only. Legacy inline bytes are externalized by the
                // host before conversion (`pandect::image_store::
                // migrate_legacy_images`), because hashing and blob storage
                // are the store's job, not the kernel's. A snapshot that still
                // carries them here has skipped that pass — see
                // `GraphSnapshot::legacy_image_count`.
                node.images = pnode.images.clone();
                node.media_type = pnode.mime_hint.clone();
                // address was already set by add_node_with_id from pnode.url; no re-derivation needed.
                node.body = pnode.body.clone();
                node.nested = pnode.nested.clone().map(muniment::LogId::new);
                node.content = pnode
                    .content_hash
                    .as_deref()
                    .and_then(muniment::Hash::from_hex);
                if node.content.is_some() {
                    node.body = None;
                }
            }
            // One-time legacy-column import. Canonical saves put these records
            // in `facets.json`; the old graph fields are written empty.
            graph.set_node_facet(
                key,
                super::super::super::node_facets::PRESENTATION_TAGS,
                &pnode.tag_presentation,
            );
            graph.set_node_facet(
                key,
                super::super::super::node_facets::PROVENANCE_IMPORT,
                &pnode.import_provenance,
            );
            graph.set_node_facet(
                key,
                super::super::super::node_facets::SEMANTIC_CLASSIFICATIONS,
                &pnode.classifications,
            );
            graph.set_node_facet(
                key,
                super::super::super::node_facets::SEMANTIC_PROPERTIES,
                &pnode.properties,
            );
            graph.set_node_facet(
                key,
                super::super::super::node_facets::PROVENANCE_DERIVATIONS,
                &pnode.derivations,
            );
            graph.set_node_facet(
                key,
                super::super::super::node_facets::ARRANGEMENT_PIN,
                &pnode.is_pinned,
            );
            graph.set_node_facet(
                key,
                super::super::super::node_facets::ARRANGEMENT_FRAME_LAYOUT,
                &pnode.frame_layout_hints,
            );
            graph.set_node_facet(
                key,
                super::super::super::node_facets::ARRANGEMENT_SPLIT_OFFER_SUPPRESSED,
                &pnode.frame_split_offer_suppressed,
            );
            let last_visited_ms = pnode
                .session_state
                .as_ref()
                .and_then(|session| session.last_visited_ms);
            if let Some(last_visited_ms) = last_visited_ms {
                graph.set_node_facet(
                    key,
                    super::super::super::node_facets::VISIT_HISTORY,
                    &super::super::super::node_facets::VisitHistoryFacet {
                        last_visited_ms: Some(last_visited_ms),
                        last_session_visited: pnode.last_session_visited,
                    },
                );
            } else if pnode.last_session_visited != 0 {
                // Old snapshots could carry a session ordinal without an
                // explicit visit timestamp. Preserve the creation-time visit
                // installed by `add_node_with_id` while importing that ordinal.
                graph.set_node_last_session_visited(key, pnode.last_session_visited);
            }
            // The saved node address is the owner of the current address. The
            // navigation cursor is only a history position and never rewrites it
            // (SetNodeUrl changes the address without a visit).
        }

        let mut surface_pair_ordinals = BTreeMap::new();
        for pedge in &snapshot.edges {
            let from_id = Uuid::parse_str(&pedge.from_node_id).ok();
            let to_id = Uuid::parse_str(&pedge.to_node_id).ok();
            let from_key = from_id.and_then(|id| graph.get_node_key_by_id(id));
            let to_key = to_id.and_then(|id| graph.get_node_key_by_id(id));
            if let (Some(from), Some(to), Some(from_id), Some(to_id)) =
                (from_key, to_key, from_id, to_id)
            {
                // Legacy assertion identities are scoped to their ordered
                // endpoint pair and bucket position. Replay uses this same
                // pair-local ordinal in `set_edges_between`; an unrelated
                // earlier pair must not change the synthesized identity.
                let ordinal = surface_pair_ordinals.entry((from_id, to_id)).or_insert(0);
                let pair_ordinal = *ordinal;
                *ordinal += 1;
                let mut payload =
                    payload_from_persisted_at(pedge, GraphStratum::Surface, pair_ordinal);
                if legacy_derived_containment
                    && snapshot.assertion_schema == 0
                    && pedge.assertions.is_none()
                {
                    mark_legacy_containment_sidecars_as_derived(&mut payload, pedge);
                }
                for statement in &mut payload.assertions.statements {
                    statement.normalize_legacy_asserter();
                }
                if !payload.is_empty() || pedge.semantic.is_some() || pedge.traversal.is_some() {
                    graph.connect_surface(from, to, payload);
                    graph.bump_revision();
                }
            }
        }

        for record in &snapshot.resources {
            let id = portolan::resource_id_from_canonical_iri(&record.canonical_iri);
            graph.set_resource_record(id, Some(record.clone()));
        }
        let mut resource_pairs: BTreeMap<(Uuid, Uuid), Vec<PersistedEdge>> = BTreeMap::new();
        for edge in &snapshot.resource_edges {
            let (Ok(from), Ok(to)) = (
                Uuid::parse_str(&edge.from_node_id),
                Uuid::parse_str(&edge.to_node_id),
            ) else {
                continue;
            };
            let mut edge = edge.clone();
            if let Some(semantic) = &mut edge.semantic {
                for statement in &mut semantic.statements {
                    statement
                        .provenance_iri
                        .get_or_insert_with(|| edge_data::UNKNOWN_LEGACY_ASSERTER_IRI.to_owned());
                }
            }
            resource_pairs.entry((from, to)).or_default().push(edge);
        }
        for ((from, to), edges) in resource_pairs {
            graph.set_resource_edges_between(from, to, &edges);
        }
        for shown in &snapshot.shown_resources {
            if let (Ok(surface), Ok(resource)) = (
                Uuid::parse_str(&shown.surface_id),
                Uuid::parse_str(&shown.resource_id),
            ) {
                graph.set_shown_resource(surface, Some(resource));
            }
        }

        // Field layer (field-system Phase 2). Malformed ids / definitions skip the
        // entry rather than failing the whole load, matching the node/edge loops.
        for pfield in &snapshot.fields {
            let Ok(id) = Uuid::parse_str(&pfield.id) else {
                continue;
            };
            let Ok(definition) = serde_json::from_str::<FieldDefinition>(&pfield.definition_json)
            else {
                continue;
            };
            let extent = match &pfield.extent {
                PersistedFieldExtent::Global => FieldExtent::Global,
                PersistedFieldExtent::Region {
                    min_x,
                    min_y,
                    max_x,
                    max_y,
                } => FieldExtent::Region {
                    min_x: *min_x,
                    min_y: *min_y,
                    max_x: *max_x,
                    max_y: *max_y,
                },
                PersistedFieldExtent::AttachedToNode(s) => match Uuid::parse_str(s) {
                    Ok(node_id) => FieldExtent::AttachedToNode(node_id),
                    Err(_) => FieldExtent::Global,
                },
                PersistedFieldExtent::Polygon { points } => FieldExtent::Polygon {
                    points: points.clone(),
                },
            };
            let mut field = Field::new(FieldId::from_uuid(id), definition).with_extent(extent);
            if let Some(name) = &pfield.name {
                field = field.with_name(name.clone());
            }
            field.lifecycle = match pfield.lifecycle {
                PersistedFieldLifecycle::Active => FieldLifecycle::Active,
                PersistedFieldLifecycle::Retired => FieldLifecycle::Retired,
            };
            graph.add_field(field);
        }

        for pcoupling in &snapshot.couplings {
            let Ok(cid) = Uuid::parse_str(&pcoupling.id) else {
                continue;
            };
            let Ok(fid) = Uuid::parse_str(&pcoupling.field_id) else {
                continue;
            };
            let selector = match &pcoupling.selector {
                PersistedNodeSelector::All => NodeSelector::All,
                PersistedNodeSelector::Tagged(t) => NodeSelector::Tagged(t.clone()),
                PersistedNodeSelector::Kind(k) => NodeSelector::Kind(k.clone()),
                PersistedNodeSelector::NotTagged(t) => NodeSelector::NotTagged(t.clone()),
            };
            let response = match &pcoupling.response {
                PersistedCouplingResponse::AttractToMin => CouplingResponse::AttractToMin,
                PersistedCouplingResponse::RepelFromMax => CouplingResponse::RepelFromMax,
                PersistedCouplingResponse::AlignVelocity => CouplingResponse::AlignVelocity,
                PersistedCouplingResponse::FlowAdvect => CouplingResponse::FlowAdvect,
                PersistedCouplingResponse::DampenInside { factor } => {
                    CouplingResponse::DampenInside { factor: *factor }
                },
                PersistedCouplingResponse::ContainmentWall => CouplingResponse::ContainmentWall,
                PersistedCouplingResponse::Open { predicate } => CouplingResponse::Open {
                    predicate: predicate.clone(),
                },
            };
            graph.add_coupling(Coupling::new(
                CouplingId::from_uuid(cid),
                FieldId::from_uuid(fid),
                selector,
                response,
                pcoupling.strength,
            ));
        }

        graph.import_records = if snapshot.import_records.is_empty() {
            // Predates the table: the legacy per-node column is the only source.
            import_records_from_legacy_provenance(
                snapshot.nodes.iter().filter_map(|pnode| {
                    let node_id = Uuid::parse_str(&pnode.node_id).ok()?;
                    Some((node_id, pnode.import_provenance.as_slice()))
                }),
                snapshot.timestamp_secs,
            )
        } else {
            snapshot.import_records.clone()
        };
        normalize_import_records(&mut graph.import_records);
        graph.sync_node_import_provenance_from_records();

        if legacy_derived_containment {
            graph.rebuild_derived_containment_relations();
        }

        graph
    }
}
