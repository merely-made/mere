// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

/// A host's card must be a finite positive size on both sides (S32): a
/// degenerate measure is reported, not laid out.
pub(super) fn card_issue(sizes: ItemSizes) -> Option<CompileIssue> {
    let Size2 { w, h } = sizes.card;
    let usable = |side: f32| side.is_finite() && side > 0.0;
    (!(usable(w) && usable(h))).then(|| {
        CompileIssue::new(
            "items.card",
            format!("the host's card must be a finite positive size; it was {w} by {h}"),
        )
    })
}

pub(super) fn validation_issues(
    compiler: &ProjectionCompiler,
    definition: &ProjectionDefinition,
    dataset: &ProjectionDataset,
) -> Vec<CompileIssue> {
    let mut issues: Vec<_> = definition
        .clone()
        .to_json_bytes()
        .ok()
        .and_then(|_| {
            // Definitions are normally promoted through ProjectionDraft. Keep
            // the compiler independently defensive for persisted input.
            let draft = scenograph::ProjectionDraft {
                dynamics: definition.dynamics.clone(),
                version: definition.version,
                id: definition.id.clone(),
                label: definition.label.clone(),
                source: definition.source.clone(),
                reading: definition.reading.clone(),
                encoding: definition.encoding.clone(),
                arrangement: definition.arrangement.clone(),
                interaction: definition.interaction.clone(),
                appearance: definition.appearance.clone(),
                provenance: definition.provenance.clone(),
            };
            draft.validate().err()
        })
        .unwrap_or_default()
        .into_iter()
        .map(|issue| CompileIssue::new(issue.field, issue.message))
        .collect();
    issues.extend(card_issue(compiler.sizes));

    source_matches(&mut issues, definition, dataset);
    if definition.reading.value.is_some() {
        issues.push(CompileIssue::new(
            "reading.value",
            "nodes does not accept a value aggregation",
        ));
    }
    if definition.interaction.selection != scenograph::SelectionMode::Single
        || definition.interaction.pan
        || definition.interaction.zoom
    {
        issues.push(CompileIssue::new(
            "interaction",
            "this realization supports single selection with a fitted viewport",
        ));
    }
    if definition.appearance.realization != "canvas" {
        issues.push(CompileIssue::new(
            "appearance.realization",
            "supported realization is canvas",
        ));
    }
    if definition.appearance.theme != "slate" {
        issues.push(CompileIssue::new(
            "appearance.theme",
            "supported theme is slate",
        ));
    }
    if definition.reading.kind != NODES_READING_ID {
        issues.push(CompileIssue::new(
            "reading.kind",
            "only the nodes reading is executable in this compiler",
        ));
    }
    required_field(
        &mut issues,
        dataset,
        "reading.key",
        &definition.reading.key,
        &[ProjectionFieldType::Text],
    );
    let target = resolve_target(compiler, definition);
    let (x_types, reads_y): (&[ProjectionFieldType], bool) =
        match target.as_ref().and_then(Target::channels) {
            Some(channels) => (channels.x_types(), channels.reads_y()),
            // A custom solver reads whatever x discloses; an unresolved
            // arrangement is reported below.
            None => (ANY_FIELD_TYPE, false),
        };
    required_channel_field(
        &mut issues,
        dataset,
        "encoding.x",
        &definition.encoding.x,
        x_types,
    );
    if reads_y {
        required_channel_field(
            &mut issues,
            dataset,
            "encoding.y",
            &definition.encoding.y,
            &[ProjectionFieldType::Number],
        );
    }
    match definition.encoding.label.as_ref() {
        Some(label) => required_channel_field(
            &mut issues,
            dataset,
            "encoding.label",
            label,
            &[ProjectionFieldType::Text],
        ),
        None => issues.push(CompileIssue::new(
            "encoding.label",
            "an executable projection needs a text label field",
        )),
    }
    if definition.encoding.color.is_some() {
        issues.push(CompileIssue::new(
            "encoding.color",
            "color encoding is not implemented by this compiler",
        ));
    }
    validate_arrangement(&mut issues, compiler, definition, dataset, target.as_ref());
    issues.extend(occurrence_identity_issues(dataset));
    validate_occurrences(&mut issues, definition, dataset, target.as_ref());
    issues
}

pub(super) fn source_matches(
    issues: &mut Vec<CompileIssue>,
    definition: &ProjectionDefinition,
    dataset: &ProjectionDataset,
) {
    for (field, expected, actual) in [
        (
            "source.authority",
            &definition.source.authority,
            &dataset.source.authority,
        ),
        (
            "source.domain",
            &definition.source.domain,
            &dataset.source.domain,
        ),
        (
            "source.resource",
            &definition.source.resource,
            &dataset.source.resource,
        ),
    ] {
        if expected != actual {
            issues.push(CompileIssue::new(
                field,
                "definition does not match the resolved dataset",
            ));
        }
    }
    if definition.provenance.source_revision.as_ref() != Some(&dataset.revision) {
        issues.push(CompileIssue::new(
            "provenance.source_revision",
            "definition does not match the resolved dataset",
        ));
    }
}

pub(super) fn required_field(
    issues: &mut Vec<CompileIssue>,
    dataset: &ProjectionDataset,
    target: &str,
    field: &str,
    expected: &[ProjectionFieldType],
) {
    match dataset.fields.get(field) {
        Some(actual) if expected.contains(actual) => {},
        Some(_) => issues.push(CompileIssue::new(target, "field has an incompatible type")),
        None => issues.push(CompileIssue::new(
            target,
            "field is not disclosed by this dataset",
        )),
    }
}

pub(super) fn required_channel_field(
    issues: &mut Vec<CompileIssue>,
    dataset: &ProjectionDataset,
    target: &str,
    channel: &Channel,
    expected: &[ProjectionFieldType],
) {
    match channel {
        Channel::Field(field) => required_field(issues, dataset, target, field, expected),
        Channel::Constant(_) => issues.push(CompileIssue::new(
            target,
            "constants are not executable for this channel",
        )),
    }
}

pub(super) fn validate_arrangement(
    issues: &mut Vec<CompileIssue>,
    compiler: &ProjectionCompiler,
    definition: &ProjectionDefinition,
    dataset: &ProjectionDataset,
    target: Option<&Target>,
) {
    match target {
        None => issues.push(CompileIssue::new(
            "arrangement.kind",
            format!(
                "{} names no built-in arrangement and no registered solver",
                definition.arrangement.kind
            ),
        )),
        Some(Target::Builtin(family)) => {
            // Options are judged against a provisional measure; their meaning
            // does not depend on where the items fall.
            let measure = Measure {
                largest: compiler.sizes.card,
                count: dataset.occurrences.len(),
                spacing: definition.arrangement.spacing as f32,
                coordinates: None,
            };
            if let Err(refused) = family.arrangement(&definition.arrangement.options, &measure) {
                issues.extend(option_issues(refused));
            }
        },
        // A registered solver's options are judged against what it declares,
        // before it solves (SE8).
        Some(Target::Custom(id)) => {
            let declared = compiler
                .registry
                .resolve(id)
                .map(|solver| solver.capability().options)
                .unwrap_or_default();
            for (key, value) in &definition.arrangement.options {
                let field = format!("arrangement.options.{key}");
                match declared.iter().find(|spec| &spec.key == key) {
                    None => issues.push(CompileIssue::new(
                        field,
                        format!("{id} does not read this option"),
                    )),
                    Some(spec) => {
                        if let Some(refusal) = spec.kind.refusal(value) {
                            issues.push(CompileIssue::new(field, refusal));
                        }
                    },
                }
            }
        },
    }
    if definition.arrangement.direction != COORDINATES_DIRECTION {
        issues.push(CompileIssue::new(
            "arrangement.direction",
            "this compiler requires the coordinates direction",
        ));
    }
}

pub(super) fn validate_occurrences(
    issues: &mut Vec<CompileIssue>,
    definition: &ProjectionDefinition,
    dataset: &ProjectionDataset,
    target: Option<&Target>,
) {
    let channels = target.and_then(Target::channels);
    let label = definition
        .encoding
        .label
        .as_ref()
        .and_then(|channel| field_name(channel, ""));
    let x = field_name(&definition.encoding.x, "");
    let y = channels
        .is_some_and(ChannelUse::reads_y)
        .then(|| field_name(&definition.encoding.y, ""))
        .flatten();
    for occurrence in &dataset.occurrences {
        let prefix = format!("occurrences.{}", occurrence.occurrence_id);
        if occurrence.source.adapter.trim().is_empty() || occurrence.source.id.trim().is_empty() {
            issues.push(CompileIssue::new(
                format!("{prefix}.source"),
                "each occurrence needs an exact source reference",
            ));
        }
        for (name, value) in &occurrence.values {
            match dataset.fields.get(name) {
                Some(field_type) if *field_type == value.field_type() => {},
                Some(_) => issues.push(CompileIssue::new(
                    format!("{prefix}.values.{name}"),
                    "value does not match the disclosed field type",
                )),
                None => issues.push(CompileIssue::new(
                    format!("{prefix}.values.{name}"),
                    "value names a field absent from the disclosed schema",
                )),
            }
        }
        if occurrence
            .values
            .get(&definition.reading.key)
            .and_then(ProjectionValue::text)
            != Some(occurrence.occurrence_id.as_str())
        {
            issues.push(CompileIssue::new(
                format!("{prefix}.values.{}", definition.reading.key),
                "the reading key must repeat this occurrence id exactly",
            ));
        }
        let finite = |value: Option<&ProjectionValue>| {
            value
                .and_then(ProjectionValue::number)
                .is_some_and(f64::is_finite)
        };
        let checks = [
            (
                "encoding.label",
                label,
                "a text",
                label.map(|field| {
                    occurrence
                        .values
                        .get(field)
                        .and_then(ProjectionValue::text)
                        .is_some()
                }),
            ),
            (
                "encoding.x",
                x,
                x_kind(channels),
                x.map(|field| x_valid(channels, occurrence.values.get(field))),
            ),
            (
                "encoding.y",
                y,
                "a finite number",
                y.map(|field| finite(occurrence.values.get(field))),
            ),
        ];
        for (target, field, kind, valid) in checks {
            if let (Some(field), Some(false)) = (field, valid) {
                issues.push(CompileIssue::new(
                    format!("{prefix}.values.{field}"),
                    format!("{target} needs {kind} value"),
                ));
            }
        }
        if matches!(
            channels,
            Some(ChannelUse::Coordinate | ChannelUse::Embedding)
        ) {
            for field in [x, y].into_iter().flatten() {
                if let Some(value) = occurrence
                    .values
                    .get(field)
                    .and_then(ProjectionValue::number)
                    && (!value.is_finite()
                        || value.abs() * definition.arrangement.spacing as f64 > 1.0e9)
                {
                    issues.push(CompileIssue::new(
                        format!("{prefix}.values.{field}"),
                        "coordinates must fit finite scene units",
                    ));
                }
            }
        }
    }
}

/// What x must hold for a channel use, in an issue's words.
pub(super) fn x_kind(channels: Option<ChannelUse>) -> &'static str {
    match channels {
        Some(ChannelUse::CategoricalAxis) => "a text",
        Some(ChannelUse::Order) => "a text or finite number",
        Some(ChannelUse::IntegerAxis) => "a whole number",
        Some(_) => "a finite number",
        None => "a",
    }
}

pub(super) fn x_valid(channels: Option<ChannelUse>, value: Option<&ProjectionValue>) -> bool {
    let number = value.and_then(ProjectionValue::number);
    match channels {
        Some(ChannelUse::CategoricalAxis) => value.and_then(ProjectionValue::text).is_some(),
        Some(ChannelUse::Order) => {
            value.and_then(ProjectionValue::text).is_some() || number.is_some_and(f64::is_finite)
        },
        Some(ChannelUse::IntegerAxis) => {
            number.is_some_and(|value| value.is_finite() && value.fract() == 0.0)
        },
        Some(_) => number.is_some_and(f64::is_finite),
        // A custom solver reads whatever x carries.
        None => value.is_some(),
    }
}

pub(super) fn field_name<'a>(channel: &'a Channel, _target: &str) -> Option<&'a str> {
    match channel {
        Channel::Field(field) if !field.trim().is_empty() => Some(field),
        Channel::Field(_) | Channel::Constant(_) => None,
    }
}

pub(super) fn number(occurrence: &ProjectionOccurrence, field: &str) -> Option<f64> {
    occurrence
        .values
        .get(field)
        .and_then(ProjectionValue::number)
}
