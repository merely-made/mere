// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The rules a frozen selection or link record must meet. Moved from the
//! kernel so the resource store checks records itself (engine split D243).
#![allow(clippy::wildcard_imports)]

use super::*;
use uuid::Uuid;

pub struct OpenedFrozenSelection<'a> {
    pub selection: &'a FrozenSelection,
    pub coverage: CoverageNote,
}
pub fn validate_retained_keys(
    input: &serde_json::Value,
    known: &serde_json::Value,
) -> Result<(), String> {
    match (input, known) {
        (serde_json::Value::Object(input), serde_json::Value::Object(known)) => {
            for (key, value) in input {
                let expected = known
                    .get(key)
                    .ok_or_else(|| format!("unknown frozen payload field {key}"))?;
                validate_retained_keys(value, expected)?;
            }
        },
        (serde_json::Value::Array(input), serde_json::Value::Array(known)) => {
            for (value, expected) in input.iter().zip(known) {
                validate_retained_keys(value, expected)?;
            }
        },
        (serde_json::Value::Object(_), _) | (serde_json::Value::Array(_), _) => {
            return Err("unrepresentable frozen payload shape".into());
        },
        _ => {},
    }
    Ok(())
}

/// The fields an edge written before L7 also carries: read, never written
/// (declared-lens D17). A stored value is accepted as it stands.
const LEGACY_EDGE_FIELDS: [&str; 7] = [
    "families",
    "semantic",
    "traversal",
    "containment",
    "arrangement",
    "imported",
    "provenance",
];

/// Widen the re-serialized `statements` edges to every key a stored payload
/// may carry, so a payload written by an older engine still validates.
pub fn known_edge_keys(raw: &serde_json::Value, known: &mut serde_json::Value) {
    let raw_edges = raw["statements"].as_array();
    for (index, edge) in known["statements"]
        .as_array_mut()
        .expect("serialized edge array")
        .iter_mut()
        .enumerate()
    {
        let stored = raw_edges.and_then(|edges| edges.get(index));
        if let Some(object) = edge.as_object_mut() {
            for field in LEGACY_EDGE_FIELDS {
                let value = stored.and_then(|s| s.get(field)).cloned();
                object
                    .entry(field)
                    .or_insert(value.unwrap_or(serde_json::Value::Null));
            }
        }
        if let Some(statements) = edge
            .get_mut("assertions")
            .and_then(serde_json::Value::as_array_mut)
        {
            normalize_statement_defaults(statements);
        }
    }
}

pub fn normalize_statement_defaults(statements: &mut [serde_json::Value]) {
    for statement in statements {
        let object = statement
            .as_object_mut()
            .expect("serialized statement object");
        for field in [
            "recognized_sub_kind",
            "label",
            "graph_scope",
            "provenance_iri",
            "asserted_at_ms",
            "first_asserted_ms",
            "withdrawn",
            "basis",
        ] {
            object.entry(field).or_insert(serde_json::Value::Null);
        }
    }
}

/// Decode and check nested references without requiring member residency.
pub fn selection_from_record(
    record: &super::PersistedResourceRecord,
) -> Result<Option<FrozenSelection>, String> {
    let Some(facet) = record
        .facets
        .iter()
        .find(|facet| facet.facet == FROZEN_SELECTION)
    else {
        return Ok(None);
    };
    let raw: serde_json::Value = serde_json::from_str(&facet.value_json)
        .map_err(|e| format!("invalid frozen selection: {e}"))?;
    let selection: FrozenSelection = serde_json::from_value(raw.clone())
        .map_err(|e| format!("invalid frozen selection: {e}"))?;
    let mut known = serde_json::to_value(&selection).map_err(|e| e.to_string())?;
    // Human-readable statement serialization omits default metadata. Their
    // known fields are still accepted explicitly, including null/default values.
    known_edge_keys(&raw, &mut known);
    validate_retained_keys(&raw, &known)?;
    if selection.version() != 1 {
        return Err("unsupported frozen selection version".into());
    }
    let members: std::collections::BTreeSet<_> = selection.members().iter().copied().collect();
    if members.len() != selection.members().len() || members.contains(&Uuid::nil()) {
        return Err("invalid frozen member references".into());
    }
    super::assertion_handles::validate_assertion_handles(
        selection
            .statements()
            .iter()
            .map(|edge| (GraphStratum::Resource, edge)),
    )
    .map_err(|error| error.to_string())?;
    for edge in selection.statements() {
        let from = edge
            .from_node_id
            .parse::<Uuid>()
            .map_err(|_| "invalid frozen source id")?;
        let to = edge
            .to_node_id
            .parse::<Uuid>()
            .map_err(|_| "invalid frozen target id")?;
        if from.to_string() != edge.from_node_id
            || to.to_string() != edge.to_node_id
            || !members.contains(&from)
            || !members.contains(&to)
        {
            return Err("frozen statement endpoints must be member references".into());
        }
        if edge.assertions.is_some() && edge.has_legacy_assertion_data() {
            return Err("frozen relation mixes canonical and legacy assertion data".into());
        }
        if edge.assertions.is_none() {
            if let Some(semantic) = &edge.semantic
                && semantic.statements.is_empty()
                && (!semantic.sub_kinds.is_empty() || semantic.predicate.is_some())
            {
                return Err("frozen semantic statements require exact handles".into());
            }
        }
        // Reject malformed family/payload combinations instead of normalizing away data.
        let payload = super::edge_decode::payload_from_persisted(edge);
        if super::edge_encode::persisted_edge_for_ids(from, to, &payload) != *edge {
            return Err("inconsistent frozen relation payload".into());
        }
    }
    Ok(Some(selection))
}

pub struct OpenedFrozenLink<'a> {
    pub link: &'a FrozenLink,
    pub coverage: CoverageNote,
}

pub fn validate_frozen_link_record(
    record: &PersistedResourceRecord,
) -> Result<Option<FrozenLink>, String> {
    let facets = record
        .facets
        .iter()
        .filter(|facet| facet.facet == FROZEN_LINK)
        .collect::<Vec<_>>();
    if facets.len() > 1 {
        return Err("duplicate frozen link facets".into());
    }
    let Some(facet) = facets.first() else {
        return Ok(None);
    };
    let raw: serde_json::Value = serde_json::from_str(&facet.value_json)
        .map_err(|error| format!("invalid frozen link: {error}"))?;
    let link: FrozenLink = serde_json::from_value(raw.clone())
        .map_err(|error| format!("invalid frozen link: {error}"))?;
    let mut known = serde_json::to_value(&link).map_err(|error| error.to_string())?;
    known_edge_keys(&raw, &mut known);
    validate_retained_keys(&raw, &known)?;
    if link.version() != 1 {
        return Err("unsupported frozen link version".into());
    }
    link.source().validate()?;
    if link.first().id > link.second().id {
        return Err("frozen link endpoints must be in canonical order".into());
    }
    for endpoint in [&link.first(), &link.second()] {
        if endpoint.id.is_nil()
            || endpoint.resource.is_some_and(|id| id.is_nil())
            || endpoint
                .address
                .as_ref()
                .is_some_and(|address| address.trim().is_empty() || address != address.trim())
        {
            return Err("invalid frozen link endpoint".into());
        }
    }
    if link.stratum() == GraphStratum::Resource
        && (link.first().resource != Some(link.first().id)
            || link.second().resource != Some(link.second().id))
    {
        return Err("resource link endpoints must retain resource identity".into());
    }
    if link.stratum() == GraphStratum::Resource
        && [&link.first(), &link.second()].into_iter().any(|endpoint| {
            endpoint.address.as_ref().is_none_or(|address| {
                portolan::resource_id_from_canonical_iri(address) != endpoint.id
            })
        })
    {
        return Err("resource endpoint address does not match its identity".into());
    }
    if link.statements().is_empty() {
        return Err("frozen link must retain at least one pair bucket".into());
    }
    super::assertion_handles::validate_assertion_handles(
        link.statements().iter().map(|edge| (link.stratum(), edge)),
    )
    .map_err(|error| error.to_string())?;
    for edge in link.statements() {
        let from = edge
            .from_node_id
            .parse::<Uuid>()
            .map_err(|_| "invalid frozen link source id")?;
        let to = edge
            .to_node_id
            .parse::<Uuid>()
            .map_err(|_| "invalid frozen link target id")?;
        let valid_pair = (from == link.first().id && to == link.second().id)
            || (from == link.second().id && to == link.first().id);
        if from.to_string() != edge.from_node_id || to.to_string() != edge.to_node_id || !valid_pair
        {
            return Err("frozen link bucket is outside its canonical endpoint pair".into());
        }
        for statement in edge.retained_assertions() {
            if statement.statement_id.trim().is_empty() {
                return Err("frozen link assertion id must be nonempty".into());
            }
        }
        let payload = super::edge_decode::payload_from_persisted(edge);
        if super::edge_encode::persisted_edge_for_ids(from, to, &payload) != *edge {
            return Err("inconsistent frozen link bucket payload".into());
        }
    }
    Ok(Some(link))
}
