// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! One serialized occurrence-binding case shared by the native and Wasm
//! entry points. The case owns source identities, occurrence addresses,
//! disclosed relationship input, roles, the admitted slot and tick batches.

use super::{ProjectionDynamics, bind_slot as pictograph_slot};
use scenograph::{DynamicsSlot, ProjectionInputBinding, RevisionEvidence, SourceBinding};
use scenomise::projection::{
    DisclosedRelationship, ItemSizes, ProjectionCompiler, ProjectionDataset, ProjectionFieldType,
    ProjectionOccurrence, ProjectionValue, RelationshipDataset, RelationshipProvenance,
    RelationshipSnapshot, relationship_recipe,
};
use seiche::Role;
use serde::Deserialize;
use serde_json::json;
use state_witness::{Trace, Witness, label};
use std::collections::{BTreeMap, BTreeSet};

const MAX_OCCURRENCES: usize = 64;
const MAX_RELATIONSHIPS: usize = 128;
const MAX_TICKS: u32 = 600;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    source: SourceInput,
    revision: String,
    spec: String,
    occurrences: Vec<OccurrenceInput>,
    relationships: Vec<RelationshipInput>,
    roles: BTreeMap<String, String>,
    tick_batches: Vec<u32>,
    #[serde(default = "default_true")]
    include_relationships: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SourceInput {
    authority: String,
    domain: String,
    resource: String,
    adapter: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct OccurrenceInput {
    occurrence_id: String,
    source_id: String,
    label: String,
    order: f64,
    x: f32,
    y: f32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RelationshipInput {
    id: String,
    from_occurrence: String,
    to_occurrence: String,
    kind: String,
    label: String,
    explanation: String,
    evidence_source_ids: Vec<String>,
}

/// Canonical sample input. Hosts may pass another input with the same schema
/// to compare a changed source, role, slot, or tick sequence.
pub fn canonical_case() -> Result<String, String> {
    Ok(include_str!("../occurrence_replay.json").to_owned())
}

/// Run the input and return the labelled per-tick state-witness trace.
pub fn run(input: &str) -> Result<String, String> {
    let case: Case = serde_json::from_str(input).map_err(|error| error.to_string())?;
    let total_ticks: u32 = case.tick_batches.iter().try_fold(0u32, |total, bound| {
        if *bound == 0 {
            return Err("tick_batches must contain positive bounds".to_owned());
        }
        total
            .checked_add(*bound)
            .ok_or_else(|| "tick_batches total overflow".to_owned())
    })?;
    if case.tick_batches.is_empty() || total_ticks > MAX_TICKS {
        return Err(format!(
            "tick_batches must total between 1 and {MAX_TICKS} ticks"
        ));
    }
    if case.occurrences.is_empty() || case.occurrences.len() > MAX_OCCURRENCES {
        return Err(format!(
            "occurrence count must be between 1 and {MAX_OCCURRENCES}"
        ));
    }
    if case.relationships.len() > MAX_RELATIONSHIPS {
        return Err(format!(
            "relationship count must be at most {MAX_RELATIONSHIPS}"
        ));
    }
    if case.tick_batches.len() > MAX_TICKS as usize {
        return Err(format!("tick batch count must be at most {MAX_TICKS}"));
    }
    let source = SourceBinding {
        authority: case.source.authority,
        domain: case.source.domain,
        resource: case.source.resource,
    };
    let revision = scenograph::PublicSourceRevision::new(case.revision);
    let recipe = relationship_recipe(
        "occurrence-replay",
        "Occurrence replay",
        "fixture",
        "recipe:1",
        BTreeMap::from([(
            "copied".into(),
            ProjectionInputBinding {
                source: source.clone(),
                expects_generation: Some(revision.clone()),
                revision_evidence: RevisionEvidence::PublicGeneration,
            },
        )]),
    );
    let occurrences: Vec<_> = case
        .occurrences
        .iter()
        .map(|occurrence| ProjectionOccurrence {
            occurrence_id: occurrence.occurrence_id.clone(),
            source: sceno::SourceRef::new(&case.source.adapter, &occurrence.source_id),
            values: BTreeMap::from([
                (
                    "occurrence_id".into(),
                    ProjectionValue::Text(occurrence.occurrence_id.clone()),
                ),
                (
                    "label".into(),
                    ProjectionValue::Text(occurrence.label.clone()),
                ),
                ("order".into(), ProjectionValue::Number(occurrence.order)),
            ]),
        })
        .collect();
    let occurrence_ids: BTreeSet<_> = occurrences
        .iter()
        .map(|occurrence| occurrence.occurrence_id.as_str())
        .collect();
    if occurrence_ids.len() != occurrences.len() || occurrences.is_empty() {
        return Err("occurrence ids must be nonempty and unique".into());
    }
    let relationships = case
        .relationships
        .iter()
        .map(|relationship| DisclosedRelationship {
            id: relationship.id.clone(),
            from_occurrence: relationship.from_occurrence.clone(),
            to_occurrence: relationship.to_occurrence.clone(),
            kind: relationship.kind.clone(),
            label: relationship.label.clone(),
            explanation: relationship.explanation.clone(),
            provenance: RelationshipProvenance {
                source: source.clone(),
                source_revision: revision.clone(),
                method: "fixture".into(),
                method_version: 1,
                provider: "fixture".into(),
                evidence: relationship
                    .evidence_source_ids
                    .iter()
                    .map(|source_id| sceno::SourceRef::new(&case.source.adapter, source_id))
                    .collect(),
            },
        })
        .collect();
    let dataset = RelationshipDataset {
        dataset: ProjectionDataset {
            source,
            revision,
            fields: BTreeMap::from([
                ("occurrence_id".into(), ProjectionFieldType::Text),
                ("label".into(), ProjectionFieldType::Text),
                ("order".into(), ProjectionFieldType::Number),
            ]),
            occurrences,
        },
        facets: [
            "authored_order".into(),
            "occurrence_labels".into(),
            "explained_relationships".into(),
        ]
        .into(),
        relationships,
    };
    let definition = recipe
        .definition
        .bind("copied", None)
        .map_err(|error| error.to_string())?;
    let snapshot = RelationshipSnapshot {
        recipe,
        source_name: "copied".into(),
        selected_occurrence: None,
        selected_relationship: None,
    };
    let mut compiled = ProjectionCompiler::new(ItemSizes {
        card: sceno::Size2::new(16.0, 10.0),
    })
    .compile_relationship_snapshot(&snapshot, &dataset)
    .map_err(|error| format!("projection compile: {error:?}"))?;
    if compiled.projection.scene.items.len() != case.occurrences.len() {
        return Err("compiled occurrence count differs from the input".into());
    }
    for occurrence in &case.occurrences {
        if !occurrence.x.is_finite() || !occurrence.y.is_finite() || !occurrence.order.is_finite() {
            return Err(format!(
                "occurrence {} has a non-finite input value",
                occurrence.occurrence_id
            ));
        }
        let instance = compiled
            .projection
            .instance_by_occurrence
            .get(&occurrence.occurrence_id)
            .ok_or_else(|| format!("compiler omitted occurrence {}", occurrence.occurrence_id))?;
        compiled.projection.scene.items[instance.0 as usize]
            .transform
            .translate = sceno::Vec2::new(occurrence.x, occurrence.y);
    }
    for relation in &mut compiled.projection.scene.relations {
        relation.points = vec![
            compiled.projection.scene.items[relation.from.0 as usize]
                .transform
                .translate,
            compiled.projection.scene.items[relation.to.0 as usize]
                .transform
                .translate,
        ];
    }
    if !case.include_relationships {
        compiled.relationships.clear();
        compiled.projection.scene.relations.clear();
    }

    let slot = DynamicsSlot::from_json(1, &case.spec)?;
    let bound_spec =
        pictograph_slot(&slot, &definition.arrangement.kind).map_err(|error| error.to_string())?;
    if bound_spec
        .target
        .as_ref()
        .is_some_and(|target| !target.items.is_empty())
    {
        return Err(
            "fixture spec target.items roles require a graph-source role witness adapter".into(),
        );
    }
    let default_role = bound_spec
        .target
        .as_ref()
        .map_or(Role::Seeded, |target| target.default_role);
    let roles: BTreeMap<_, _> = case
        .roles
        .iter()
        .map(|(id, role)| {
            let role = match role.as_str() {
                "anchored" => Role::Anchored,
                "pinned" => Role::Pinned,
                "seeded" => Role::Seeded,
                _ => return Err(format!("unknown role {role:?} at occurrence {id}")),
            };
            Ok((id.clone(), role))
        })
        .collect::<Result<_, String>>()?;
    let mut run = ProjectionDynamics::new_relationship(&definition, &compiled, &slot, &roles)
        .map_err(|error| error.to_string())?;
    let mut trace = Trace::new();
    trace
        .push(
            0,
            capture(&run, &compiled, &case.occurrences, &roles, default_role)?,
        )
        .map_err(|error| error.to_string())?;
    for bound in case.tick_batches {
        for offset in 0..bound {
            let continuing = run.tick().map_err(|error| error.to_string())?;
            if !continuing && (offset + 1 < bound || run.ticks() < total_ticks as u64) {
                return Err(format!(
                    "settle ended before requested tick {}",
                    run.ticks() + 1
                ));
            }
            trace
                .push(
                    run.ticks(),
                    capture(&run, &compiled, &case.occurrences, &roles, default_role)?,
                )
                .map_err(|error| error.to_string())?;
        }
    }
    serde_json::to_string(&json!({
        "trace": trace,
        "relationship_ids": run.relationship_ids(),
        "ticks": run.ticks(),
    }))
    .map_err(|error| error.to_string())
}

fn capture(
    run: &ProjectionDynamics,
    compiled: &scenomise::projection::CompiledRelationshipProjection,
    occurrences: &[OccurrenceInput],
    roles: &BTreeMap<String, Role>,
    default_role: Role,
) -> Result<Witness, String> {
    if run.scene().sources != compiled.projection.scene.sources {
        return Err("dynamics preview changed the disclosed source table".into());
    }
    let mut witness = Witness::new();
    for occurrence in occurrences {
        let role = roles
            .get(&occurrence.occurrence_id)
            .copied()
            .unwrap_or(default_role);
        witness
            .insert(
                label("role", [occurrence.occurrence_id.as_str()]),
                role_name(role),
            )
            .map_err(|error| error.to_string())?;
        let instance = compiled
            .projection
            .instance_by_occurrence
            .get(&occurrence.occurrence_id)
            .ok_or_else(|| format!("missing compiled occurrence {}", occurrence.occurrence_id))?;
        let index = instance.0 as usize;
        let source_ix = run.scene().items[index].source;
        if run.scene().items[index].source != compiled.projection.scene.items[index].source {
            return Err(format!(
                "dynamics preview changed source addressing for {}",
                occurrence.occurrence_id
            ));
        }
        let source_ref = &run.scene().sources[source_ix.0 as usize];
        witness
            .insert(
                label("source", [occurrence.occurrence_id.as_str()]),
                &(source_ref.adapter.as_str(), source_ref.id.as_str()),
            )
            .map_err(|error| error.to_string())?;
        witness
            .insert(
                label("position-bits", [occurrence.occurrence_id.as_str()]),
                &[
                    run.scene().items[index].transform.translate.x.to_bits(),
                    run.scene().items[index].transform.translate.y.to_bits(),
                ],
            )
            .map_err(|error| error.to_string())?;
    }
    Ok(witness)
}

fn role_name(role: Role) -> &'static str {
    match role {
        Role::Anchored => "anchored",
        Role::Pinned => "pinned",
        Role::Seeded => "seeded",
    }
}
