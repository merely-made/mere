// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

pub(super) fn compile_inner(
    compiler: &ProjectionCompiler,
    definition: &ProjectionDefinition,
    dataset: &ProjectionDataset,
    selected_occurrence: Option<&str>,
    previous: Option<&CompiledProjection>,
) -> Result<CompiledProjection, Vec<CompileIssue>> {
    let mut issues = validation_issues(compiler, definition, dataset);
    if let Some(selected) = selected_occurrence
        && !dataset
            .occurrences
            .iter()
            .any(|occurrence| occurrence.occurrence_id == selected)
    {
        issues.push(CompileIssue::new(
            "selected_occurrence",
            "the saved selected occurrence is absent from this dataset revision",
        ));
    }
    if !issues.is_empty() {
        return Err(issues);
    }

    let label_field = definition
        .encoding
        .label
        .as_ref()
        .and_then(|channel| field_name(channel, "encoding.label"))
        .expect("validated label field");
    let x_field = field_name(&definition.encoding.x, "encoding.x").expect("validated x field");
    let y_field = field_name(&definition.encoding.y, "encoding.y");
    let target = resolve_target(compiler, definition).expect("validated arrangement");
    let channels = target.channels();

    let mut occurrences: Vec<_> = dataset.occurrences.iter().collect();
    occurrences.sort_by(|left, right| left.occurrence_id.cmp(&right.occurrence_id));
    if channels == Some(ChannelUse::Order) {
        // Order-only families place by ordinal, so x decides the order; the
        // stable sort keeps occurrence ids as the tiebreak.
        occurrences.sort_by(|left, right| order_cmp(left, right, x_field));
    }
    let measure = Measure {
        largest: compiler.sizes.card,
        count: occurrences.len(),
        spacing: definition.arrangement.spacing as f32,
        coordinates: (channels == Some(ChannelUse::Coordinate))
            .then(|| coordinate_extent(&occurrences, x_field, y_field.expect("validated y field")))
            .flatten(),
    };
    let arrangement = match &target {
        Target::Builtin(family) => family
            .arrangement(&definition.arrangement.options, &measure)
            .map_err(option_issues)?,
        Target::Custom(id) => ScenoArrangement::Custom {
            id: id.clone(),
            config: serde_json::Value::Object(
                definition
                    .arrangement
                    .options
                    .iter()
                    .map(|(key, value)| (key.clone(), serde_json::Value::String(value.clone())))
                    .collect(),
            ),
        },
    };
    let grid_ranks = (channels == Some(ChannelUse::Cells)).then(|| {
        (
            dense_ranks(&occurrences, x_field),
            dense_ranks(&occurrences, y_field.expect("validated y field")),
        )
    });
    let mut score = Score::new(arrangement);
    score.generation = stable_generation(dataset);
    for (ordinal, occurrence) in occurrences.iter().enumerate() {
        let mut item = ScoreItem {
            source: occurrence.source.clone(),
            ordinal: ordinal as u32,
            footprint: Footprint::Rect {
                size: compiler.sizes.card,
            },
            representation: Representation::Card,
            placement: Placement::Ordinal,
            layer: 0,
            visible: true,
            axis: None,
            embedding: None,
            weight: None,
        };
        disclose(
            &mut item,
            channels,
            occurrence,
            x_field,
            y_field,
            grid_ranks.as_ref(),
        );
        score.items.push(item);
    }
    // Compare exact solver inputs, not revision hashes: source revisions can
    // change without affecting placement, and unchanged revisions cannot excuse
    // changed data. Ordered occurrence identity also protects repeated sources.
    let reusable = previous.filter(|old| {
        old.score.version == score.version
            && old.score.arrangement == score.arrangement
            && old.score.items == score.items
            && old.score.holds == score.holds
            && occurrences.iter().enumerate().all(|(index, occurrence)| {
                old.occurrence_by_instance.get(&InstanceId(index as u32))
                    == Some(&occurrence.occurrence_id)
            })
    });
    let placement_reused = reusable.is_some();
    let scene = if let Some(old) = reusable {
        let mut scene = old.scene.clone();
        scene.generation = score.generation;
        scene
    } else {
        crate::solve_via(&score, &compiler.registry)
            .map_err(|error| vec![CompileIssue::new("arrangement.kind", error.to_string())])?
    };
    let mut labels = HashMap::new();
    let mut occurrence_by_instance = HashMap::new();
    let mut instance_by_occurrence = BTreeMap::new();
    for (index, occurrence) in occurrences.into_iter().enumerate() {
        let instance = InstanceId(index as u32);
        labels.insert(
            instance,
            occurrence
                .values
                .get(label_field)
                .and_then(ProjectionValue::text)
                .expect("validated label value")
                .to_owned(),
        );
        occurrence_by_instance.insert(instance, occurrence.occurrence_id.clone());
        instance_by_occurrence.insert(occurrence.occurrence_id.clone(), instance);
    }
    let selected = selected_occurrence.and_then(|id| instance_by_occurrence.get(id).copied());
    Ok(CompiledProjection {
        score,
        scene,
        labels,
        occurrence_by_instance,
        instance_by_occurrence,
        selected,
        placement_reused,
    })
}

/// Give an item what its family reads from x and y.
pub(super) fn disclose(
    item: &mut ScoreItem,
    channels: Option<ChannelUse>,
    occurrence: &ProjectionOccurrence,
    x_field: &str,
    y_field: Option<&str>,
    grid_ranks: Option<&(BTreeMap<u64, i32>, BTreeMap<u64, i32>)>,
) {
    let x_number = number(occurrence, x_field);
    let x_text = occurrence
        .values
        .get(x_field)
        .and_then(ProjectionValue::text);
    let y_number = y_field.and_then(|field| number(occurrence, field));
    let xy = || {
        let (x, y) = (
            x_number.expect("validated x value"),
            y_number.expect("validated y value"),
        );
        (x, y)
    };
    match channels {
        Some(ChannelUse::Cells) => {
            let (x, y) = xy();
            let (x_ranks, y_ranks) = grid_ranks.expect("grid ranks for a cell family");
            // Dense numeric ranks preserve the two declared encodings without
            // allocating ninety-six empty rows for a 96 bpm card.
            item.placement = Placement::Cell {
                column: x_ranks[&dense_rank_key(x)],
                row: y_ranks[&dense_rank_key(y)],
            };
        },
        Some(ChannelUse::Coordinate) => {
            let (x, y) = xy();
            item.placement = Placement::Coordinate(Vec2::new(x as f32, y as f32));
        },
        Some(ChannelUse::Embedding) => {
            let (x, y) = xy();
            item.embedding = Some(Vec2::new(x as f32, y as f32));
        },
        Some(ChannelUse::NumericAxis | ChannelUse::IntegerAxis) => {
            item.axis = Some(AxisValue::Numeric(x_number.expect("validated x value")));
        },
        Some(ChannelUse::CategoricalAxis) => {
            item.axis = Some(AxisValue::Categorical(
                x_text.expect("validated x value").to_owned(),
            ));
        },
        // The ordinal carries the order.
        Some(ChannelUse::Order) => {},
        // A custom solver reads whatever x and y can disclose; its declared
        // requirements are checked when it solves.
        None => {
            item.axis = x_number
                .map(AxisValue::Numeric)
                .or_else(|| x_text.map(|text| AxisValue::Categorical(text.to_owned())));
            item.embedding = x_number
                .zip(y_number)
                .map(|(x, y)| Vec2::new(x as f32, y as f32));
            item.weight = y_number.map(|y| y as f32);
        },
    }
}

/// Order two occurrences by their x value; validation keeps one type per field.
pub(super) fn order_cmp(
    left: &ProjectionOccurrence,
    right: &ProjectionOccurrence,
    field: &str,
) -> std::cmp::Ordering {
    match (left.values.get(field), right.values.get(field)) {
        (Some(ProjectionValue::Number(a)), Some(ProjectionValue::Number(b))) => a.total_cmp(b),
        (Some(ProjectionValue::Text(a)), Some(ProjectionValue::Text(b))) => a.cmp(b),
        _ => std::cmp::Ordering::Equal,
    }
}

/// The smallest and largest disclosed coordinate, in the source's units.
pub(super) fn coordinate_extent(
    occurrences: &[&ProjectionOccurrence],
    x_field: &str,
    y_field: &str,
) -> Option<(Vec2, Vec2)> {
    occurrences
        .iter()
        .filter_map(|occurrence| {
            Some(Vec2::new(
                number(occurrence, x_field)? as f32,
                number(occurrence, y_field)? as f32,
            ))
        })
        .fold(None, |extent, at| {
            Some(match extent {
                None => (at, at),
                Some((min, max)) => (
                    Vec2::new(min.x.min(at.x), min.y.min(at.y)),
                    Vec2::new(max.x.max(at.x), max.y.max(at.y)),
                ),
            })
        })
}

pub(super) fn option_issues(issues: Vec<OptionIssue>) -> Vec<CompileIssue> {
    issues
        .into_iter()
        .map(|issue| CompileIssue::new(issue.field, issue.message))
        .collect()
}

/// Builds the same dense ranks as the former per-occurrence closure, once for
/// each grid channel. Finite values have one `==` equivalence class per bit
/// pattern except signed zero, which deliberately shares a rank.
pub(super) fn dense_ranks(
    occurrences: &[&ProjectionOccurrence],
    field: &str,
) -> BTreeMap<u64, i32> {
    let mut values: Vec<_> = occurrences
        .iter()
        .map(|occurrence| number(occurrence, field).expect("validated numeric value"))
        .collect();
    values.sort_by(f64::total_cmp);
    values.dedup();
    values
        .into_iter()
        .enumerate()
        .map(|(rank, value)| (dense_rank_key(value), rank as i32))
        .collect()
}

pub(super) fn dense_rank_key(value: f64) -> u64 {
    if value == 0.0 { 0 } else { value.to_bits() }
}
