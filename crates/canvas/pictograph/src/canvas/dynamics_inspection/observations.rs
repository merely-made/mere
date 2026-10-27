// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Inspector projections of existing reports. No retained geometry, trace,
//! field registry or relationship authority is introduced here.
use super::*;
use serde_json::json;

pub(super) fn paths(canvas: &Canvas, target: DynamicsTarget) -> Option<Value> {
    let DynamicsTarget::Pair { first, second } = target else {
        return None;
    };
    let report = canvas.link_path_response_report();
    let settings = &report.settings;
    let paths: Vec<_> = report.paths.iter().filter(|path| {
        (path.displayed_from == first && path.displayed_to == second)
            || (path.displayed_from == second && path.displayed_to == first)
    }).map(|path| json!({
        "key": path.key, "displayed_from": path.displayed_from,
        "displayed_to": path.displayed_to, "stratum": path.stratum,
        "responsive": path.responsive,
        "compatible": settings.compatible_links.contains(&path.key),
        "points": path.points, "max_bend": path.max_bend(), "source": path.source,
        "buckets": path.buckets.iter().map(|bucket| json!({
            "displayed_from": bucket.displayed_from, "displayed_to": bucket.displayed_to,
            "assertion_ids": bucket.assertion_ids, "legacy_members": bucket.legacy_members,
            "reading_index": bucket.bucket.reading_index,
            "link_index": bucket.bucket.link_index, "bucket_index": bucket.bucket.bucket_index,
        })).collect::<Vec<_>>(),
    })).collect();
    Some(json!({
        "state": format!("{:?}", report.state).to_lowercase(),
        "failure": report.failure.map(|failure| format!("{failure:?}")),
        "field_response": settings.field_response,
        "compatible_attraction": settings.compatible_attraction,
        "selected_fields": settings.selected_fields,
        "selected_field_definitions": settings.selected_fields.iter().map(|id| canvas.graph.field(*id)).collect::<Vec<_>>(),
        "temporary_field": settings.view_field.as_deref(),
        "field_strength": settings.field_strength,
        "compatible_links": settings.compatible_links,
        "evaluator": format!("{:?}", settings.config),
        "paths": paths,
        // These totals concern the whole evaluation cohort, not just this pair.
        "cohort_presentation_fallbacks": report.presentation_fallbacks,
        "cohort_excluded_endpoints": report.excluded_endpoints,
        "cohort_ineligible_self_links": report.ineligible_self_links,
    }))
}

pub(super) fn couplings(canvas: &Canvas, target: DynamicsTarget) -> Value {
    let clock = canvas.field_clock();
    let members = target.members();
    let trace = canvas.coupling_trace().map(|trace| {
        let mut bindings = std::collections::BTreeMap::new();
        let mut rows = Vec::new();
        for row in &trace.records {
            let Some(Some(identity)) = row.binding.target_identities.get(row.target_index) else {
                continue;
            };
            let member = Uuid::from_u128(identity.member);
            // Membership reconciliation clears observations before a replacement
            // body can reuse a key. Also require both captured owner identity and
            // current handle; occurrence-owned bodies are a different domain.
            if identity.occurrence.is_some()
                || !members.contains(&member)
                || canvas.graph.get_node_key_by_id(member) != Some(row.node)
            {
                continue;
            }
            let binding = &row.binding;
            // One exact immutable binding per observed coupling, shared by its
            // rows. Do not copy the full field registry into every target row.
            bindings.entry(row.coupling_index).or_insert_with(|| {
                json!({
                    "source": binding.source.map(|source| json!({
                        "field": source.field, "coupling": source.coupling,
                        "revision_hint": source.revision,
                    })),
                    "response": binding.response, "strength": binding.strength,
                    "field": binding.field, "extent": binding.extent, "selector": binding.selector,
                    "registry": binding.registry, "animated": binding.animated,
                    "target_count": binding.targets.len(),
                })
            });
            rows.push(json!({
                "member": member, "coupling_index": row.coupling_index,
                "target_index": row.target_index, "sample_position": row.sample_position,
                "sample_time": row.evaluation.map(|value| value.sample_time),
                "evaluation": row.evaluation.map(|value| format!("{:?}", value.effect)),
                "applied": format!("{:?}", row.applied),
            }));
        }
        json!({
            "step": trace.step, "binding_generation": trace.binding_generation,
            "clock_elapsed": trace.clock.elapsed, "clock_epoch": trace.clock.epoch,
            "sample_time": trace.clock.sample_time(), "dt": trace.dt,
            "rows": rows, "bindings": bindings,
            "cohort_omitted": trace.omitted, "limit": trace.limit.get(),
        })
    });
    json!({
        "clock_elapsed": clock.elapsed, "clock_epoch": clock.epoch, "clock_paused": clock.paused,
        "trace": trace,
    })
}
