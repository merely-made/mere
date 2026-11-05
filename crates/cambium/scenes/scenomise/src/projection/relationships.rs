// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

pub(super) fn relationship_snapshot(
    compiler: &ProjectionCompiler,
    snapshot: &RelationshipSnapshot,
    disclosed: &RelationshipDataset,
    limits: &RelationshipCompileLimits,
) -> Result<CompiledRelationshipProjection, Vec<CompileIssue>> {
    relationship_bounds(snapshot, disclosed, limits)?;
    let card = card_issue(compiler.sizes);
    let mut issues = snapshot
        .validate()
        .err()
        .unwrap_or_default()
        .into_iter()
        .map(|issue| CompileIssue::new(issue.field, issue.message))
        .collect::<Vec<_>>();
    let dataset = &disclosed.dataset;
    for required in [
        AUTHORED_ORDER_FACET,
        OCCURRENCE_LABELS_FACET,
        EXPLAINED_RELATIONSHIPS_FACET,
        snapshot.recipe.requirements.facet.as_str(),
    ] {
        if !disclosed.facets.contains(required) {
            issues.push(CompileIssue::new(
                format!("requirements.facets.{required}"),
                format!("source does not disclose semantic facet {required}"),
            ));
        }
    }
    let definition = &snapshot.recipe.definition;
    if !matches!(
        Family::resolve(&definition.arrangement.kind),
        Some(Family::Grid | Family::Spiral)
    ) {
        issues.push(CompileIssue::new("arrangement.kind", "authored occurrence order requires a grid or spiral arrangement; it is not a physical scatter coordinate"));
    }
    if definition.reading.kind != "nodes"
        || definition.reading.key != "occurrence_id"
        || definition.reading.value.is_some()
        || definition.encoding.x != Channel::Field("order".into())
        || definition.encoding.y != Channel::Constant("0".into())
        || definition.encoding.label != Some(Channel::Field("label".into()))
    {
        issues.push(CompileIssue::new("definition.roles", "relationship recipes require authored order, occurrence identity, labels, and the constant layout row"));
    }
    if dataset.revision.as_str().trim().is_empty() {
        issues.push(CompileIssue::new(
            "dataset.revision",
            "a public source revision is required",
        ));
    }
    if dataset.occurrences.len() < snapshot.recipe.requirements.min_occurrences as usize {
        issues.push(CompileIssue::new(
            "requirements.min_occurrences",
            "source has fewer disclosed occurrences than the recipe requires",
        ));
    }
    let mut orders = HashSet::new();
    for occurrence in &dataset.occurrences {
        let valid_order = number(occurrence, "order").filter(|order| {
            order.is_finite() && *order >= 0.0 && order.fract() == 0.0 && *order <= u32::MAX as f64
        });
        if valid_order.is_none() || !orders.insert(dense_rank_key(valid_order.unwrap_or_default()))
        {
            issues.push(CompileIssue::new(
                format!("occurrences.{}.order", occurrence.occurrence_id),
                "authored order must be a distinct nonnegative integer per occurrence",
            ));
        }
    }
    if dataset.fields.contains_key("__recipe_layout_row")
        || dataset.fields.contains_key("__recipe_layout_column")
    {
        issues.push(CompileIssue::new(
            "dataset.fields",
            "reserved recipe layout row must not be disclosed as source truth",
        ));
    }
    issues.extend(relationship_disclosure_issues(
        dataset,
        &disclosed.relationships,
    ));
    let relationships: Vec<_> = disclosed
        .relationships
        .iter()
        .filter(|relationship| {
            snapshot
                .recipe
                .relationship_kind
                .as_ref()
                .is_none_or(|kind| kind == &relationship.kind)
        })
        .collect();
    if snapshot.recipe.relationship_kind.is_some() && relationships.is_empty() {
        issues.push(CompileIssue::new(
            "relationship_kind",
            "the requested semantic relationship kind is not disclosed by this source",
        ));
    }
    if relationships.len() < snapshot.recipe.requirements.min_relationships as usize {
        issues.push(CompileIssue::new(
            "requirements.min_relationships",
            "source has fewer applicable explained relationships than the recipe requires",
        ));
    }
    if snapshot
        .selected_relationship
        .as_ref()
        .is_some_and(|selected| {
            !relationships
                .iter()
                .any(|relationship| &relationship.id == selected)
        })
    {
        issues.push(CompileIssue::new(
            "selected_relationship",
            "the saved selected relationship is absent from this source or excluded by the recipe",
        ));
    }
    issues.extend(card);
    if !issues.is_empty() {
        return Err(issues);
    }
    let mut bound = snapshot
        .recipe
        .definition
        .bind(&snapshot.source_name, None)
        .map_err(|error| {
            vec![CompileIssue::new(
                "source_name",
                format!("binding refused: {error:?}"),
            )]
        })?;
    // This row is an explicit layout constant, not a claimed domain facet or
    // computed comparison result. Reuse the portable field compiler internally.
    bound.encoding.y = Channel::Field("__recipe_layout_row".into());
    let mut execution_dataset = dataset.clone();
    execution_dataset
        .fields
        .insert("__recipe_layout_row".into(), ProjectionFieldType::Number);
    for occurrence in &mut execution_dataset.occurrences {
        occurrence
            .values
            .insert("__recipe_layout_row".into(), ProjectionValue::Number(0.0));
    }
    // Opt-in ranked-grid layout. The legacy grid keeps its constant row.
    // These cells are compiler-owned layout values, never source assertions.
    if definition.arrangement.kind == "grid.ordered" {
        let columns = definition
            .arrangement
            .options
            .get("columns")
            .and_then(|v| v.parse::<u32>().ok())
            .filter(|v| *v > 0)
            .unwrap_or_else(|| (dataset.occurrences.len() as f64).sqrt().ceil().max(1.0) as u32);
        let mut order: Vec<_> = dataset
            .occurrences
            .iter()
            .map(|o| {
                (
                    number(o, "order").expect("validated order") as u32,
                    o.occurrence_id.clone(),
                )
            })
            .collect();
        order.sort();
        let ranks: BTreeMap<_, _> = order
            .into_iter()
            .enumerate()
            .map(|(rank, (_, id))| (id, rank as u32))
            .collect();
        bound.encoding.x = Channel::Field("__recipe_layout_column".into());
        execution_dataset
            .fields
            .insert("__recipe_layout_column".into(), ProjectionFieldType::Number);
        for occurrence in &mut execution_dataset.occurrences {
            let rank = ranks[&occurrence.occurrence_id];
            occurrence.values.insert(
                "__recipe_layout_column".into(),
                ProjectionValue::Number((rank % columns) as f64),
            );
            occurrence.values.insert(
                "__recipe_layout_row".into(),
                ProjectionValue::Number((rank / columns) as f64),
            );
        }
    }
    let mut projection = compiler.compile_snapshot(
        &ProjectionSnapshot {
            definition: bound,
            selected_occurrence: snapshot.selected_occurrence.clone(),
        },
        &execution_dataset,
    )?;
    let mut compiled = Vec::with_capacity(relationships.len());
    let mut ordered = relationships;
    ordered.sort_by(|a, b| a.id.cmp(&b.id));
    for relationship in ordered {
        let from = projection.instance_by_occurrence[&relationship.from_occurrence];
        let to = projection.instance_by_occurrence[&relationship.to_occurrence];
        projection.scene.relations.push(sceno::RoutedRelation {
            from,
            to,
            space: sceno::Scene::WORLD,
            points: vec![
                projection.scene.items[from.0 as usize].transform.translate,
                projection.scene.items[to.0 as usize].transform.translate,
            ],
            kind: Some(relationship.kind.clone()),
            weight: None,
            link_key: None,
            marks: sceno::LinkMarks::default(),
            strength: None,
        });
        compiled.push(CompiledRelationship {
            disclosure: relationship.clone(),
            from,
            to,
        });
    }
    Ok(CompiledRelationshipProjection {
        projection,
        relationships: compiled,
        selected_relationship: snapshot.selected_relationship.clone(),
    })
}

/// The checks every disclosed relationship meets against its dataset: a
/// unique identity, two distinct disclosed endpoints, stated meaning, and
/// method provenance at the dataset's exact source and public revision. The
/// relationship compiler and the host dataset boundary
/// ([`crate::host_dataset`]) share this one validator.
pub(crate) fn relationship_disclosure_issues(
    dataset: &ProjectionDataset,
    relationships: &[DisclosedRelationship],
) -> Vec<CompileIssue> {
    let mut issues = Vec::new();
    let ids: HashSet<_> = dataset
        .occurrences
        .iter()
        .map(|occurrence| occurrence.occurrence_id.as_str())
        .collect();
    let mut relationship_ids = HashSet::new();
    for relationship in relationships {
        let prefix = format!("relationships.{}", relationship.id);
        if relationship.id.trim().is_empty() || !relationship_ids.insert(&relationship.id) {
            issues.push(CompileIssue::new(
                format!("{prefix}.id"),
                "relationship identities must be nonempty and unique",
            ));
        }
        if !ids.contains(relationship.from_occurrence.as_str())
            || !ids.contains(relationship.to_occurrence.as_str())
            || relationship.from_occurrence == relationship.to_occurrence
        {
            issues.push(CompileIssue::new(
                format!("{prefix}.endpoints"),
                "a relationship must name two distinct disclosed occurrence identities",
            ));
        }
        for (field, value) in [
            ("kind", &relationship.kind),
            ("label", &relationship.label),
            ("explanation", &relationship.explanation),
            ("provenance.method", &relationship.provenance.method),
            ("provenance.provider", &relationship.provenance.provider),
        ] {
            if value.trim().is_empty() {
                issues.push(CompileIssue::new(
                    format!("{prefix}.{field}"),
                    "disclosed meaning and method provenance cannot be empty",
                ));
            }
        }
        if relationship.provenance.method_version == 0 {
            issues.push(CompileIssue::new(
                format!("{prefix}.provenance.method_version"),
                "a positive disclosed method version is required",
            ));
        }
        if relationship.provenance.source != dataset.source
            || relationship.provenance.source_revision != dataset.revision
        {
            issues.push(CompileIssue::new(
                format!("{prefix}.provenance.source_revision"),
                "relationship provenance must match the exact disclosed source and public revision",
            ));
        }
        if relationship.provenance.evidence.is_empty()
            || relationship
                .provenance
                .evidence
                .iter()
                .any(|source| source.adapter.trim().is_empty() || source.id.trim().is_empty())
        {
            issues.push(CompileIssue::new(
                format!("{prefix}.provenance.evidence"),
                "relationship evidence needs exact source references",
            ));
        }
    }
    issues
}

pub(super) fn relationship_bounds(
    snapshot: &RelationshipSnapshot,
    disclosure: &RelationshipDataset,
    limits: &RelationshipCompileLimits,
) -> Result<(), Vec<CompileIssue>> {
    let fail = |field: &str| {
        vec![CompileIssue::new(
            field,
            "relationship compiler disclosure limit exceeded",
        )]
    };
    if disclosure.dataset.occurrences.len() > limits.max_occurrences {
        return Err(fail("limits.occurrences"));
    }
    if disclosure.relationships.len() > limits.max_relationships {
        return Err(fail("limits.relationships"));
    }
    if disclosure.dataset.fields.len() > limits.max_fields
        || disclosure.facets.len() > limits.max_fields
    {
        return Err(fail("limits.fields"));
    }
    if snapshot.recipe.definition.sources.len() > limits.max_inputs {
        return Err(fail("limits.inputs"));
    }
    let mut total = 0usize;
    let mut check = |value: &str| -> Result<(), Vec<CompileIssue>> {
        total = total
            .checked_add(value.len())
            .ok_or_else(|| fail("limits.text"))?;
        if value.len() > limits.max_text_bytes || total > limits.max_total_text_bytes {
            return Err(fail("limits.text"));
        }
        Ok(())
    };
    fn source(value: &SourceBinding) -> [&String; 3] {
        [&value.authority, &value.domain, &value.resource]
    }
    for value in source(&disclosure.dataset.source) {
        check(value)?;
    }
    check(disclosure.dataset.revision.as_str())?;
    for field in disclosure.dataset.fields.keys() {
        check(field)?;
    }
    for facet in &disclosure.facets {
        check(facet)?;
    }
    for occurrence in &disclosure.dataset.occurrences {
        if occurrence.values.len() > limits.max_fields {
            return Err(fail("limits.fields"));
        }
        check(&occurrence.occurrence_id)?;
        check(&occurrence.source.adapter)?;
        check(&occurrence.source.id)?;
        for (name, value) in &occurrence.values {
            check(name)?;
            match value {
                ProjectionValue::Text(text) => check(text)?,
                ProjectionValue::Number(number) if !number.is_finite() => {
                    return Err(vec![CompileIssue::new(
                        "dataset.values",
                        "all disclosed numbers must be finite",
                    )]);
                },
                _ => (),
            }
        }
    }
    for relationship in &disclosure.relationships {
        if relationship.provenance.evidence.len() > limits.max_occurrences {
            return Err(fail("limits.evidence"));
        }
        for text in [
            &relationship.id,
            &relationship.from_occurrence,
            &relationship.to_occurrence,
            &relationship.kind,
            &relationship.label,
            &relationship.explanation,
            &relationship.provenance.method,
            &relationship.provenance.provider,
        ] {
            check(text)?;
        }
        for text in source(&relationship.provenance.source) {
            check(text)?;
        }
        check(relationship.provenance.source_revision.as_str())?;
        for evidence in &relationship.provenance.evidence {
            check(&evidence.adapter)?;
            check(&evidence.id)?;
        }
    }
    let definition = &snapshot.recipe.definition;
    for text in [
        &definition.id,
        &definition.label,
        &definition.reading.kind,
        &definition.reading.key,
        &definition.arrangement.kind,
        &definition.arrangement.direction,
        &definition.appearance.realization,
        &definition.appearance.title,
        &definition.appearance.theme,
        &definition.provenance.author,
        &definition.provenance.note,
        &snapshot.source_name,
        &snapshot.recipe.requirements.facet,
    ] {
        check(text)?;
    }
    for channel in [&definition.encoding.x, &definition.encoding.y]
        .into_iter()
        .chain(definition.encoding.color.iter())
        .chain(definition.encoding.label.iter())
    {
        match channel {
            Channel::Field(value) | Channel::Constant(value) => check(value)?,
        }
    }
    for text in [
        definition.reading.value.as_deref(),
        definition
            .provenance
            .source_revision
            .as_ref()
            .map(PublicSourceRevision::as_str),
        snapshot.recipe.relationship_kind.as_deref(),
        snapshot.selected_occurrence.as_deref(),
        snapshot.selected_relationship.as_deref(),
    ]
    .into_iter()
    .flatten()
    {
        check(text)?;
    }
    if definition.arrangement.options.len() > limits.max_fields {
        return Err(fail("limits.fields"));
    }
    for (key, value) in &definition.arrangement.options {
        check(key)?;
        check(value)?;
    }
    for (name, binding) in &definition.sources {
        check(name)?;
        for value in source(&binding.source) {
            check(value)?;
        }
        if let Some(revision) = &binding.expects_generation {
            check(revision.as_str())?;
        }
    }
    Ok(())
}
