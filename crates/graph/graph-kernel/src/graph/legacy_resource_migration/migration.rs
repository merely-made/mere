// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

/// Validate against all retained, qualified legacy entries before reading shorter prefixes.
pub fn validate_legacy_origin_resolutions(
    baseline: &Graph,
    entries: &[AttributedDelta],
    resolutions: &[LegacyResourceOriginResolution],
) -> Result<(), LegacyMigrationError> {
    validated_resolutions(&probe_legacy_mint_links(baseline, entries)?, resolutions).map(|_| ())
}

pub(super) fn validated_resolutions(
    diagnostics: &[LegacyMintLinkDiagnostic],
    resolutions: &[LegacyResourceOriginResolution],
) -> Result<BTreeMap<String, Pair>, LegacyMigrationError> {
    let mut candidates = BTreeMap::<String, BTreeSet<Pair>>::new();
    for diagnostic in diagnostics {
        let pairs = candidates
            .entry(diagnostic.statement_id.clone())
            .or_default();
        for pair in [
            &diagnostic.earlier_resource_pair,
            &diagnostic.current_resource_pair,
        ] {
            pairs.insert((parse_id(&pair.0)?, parse_id(&pair.1)?));
        }
    }
    let mut selected = BTreeMap::new();
    for resolution in resolutions {
        let pair = (resolution.from_resource_id, resolution.to_resource_id);
        if !candidates
            .get(&resolution.statement_id)
            .is_some_and(|pairs| pairs.contains(&pair))
        {
            return Err(LegacyMigrationError(format!(
                "checkpoint C28: invalid resource origin for {}",
                resolution.statement_id
            )));
        }
        if let Some(previous) = selected.insert(resolution.statement_id.clone(), pair)
            && previous != pair
        {
            return Err(LegacyMigrationError(format!(
                "checkpoint C28: conflicting resource origins for {}",
                resolution.statement_id
            )));
        }
    }
    if candidates.keys().any(|id| !selected.contains_key(id)) {
        return Err(LegacyMigrationError(format!(
            "checkpoint C28: {} unresolved legacy mint-link handles",
            candidates
                .keys()
                .filter(|id| !selected.contains_key(*id))
                .count()
        )));
    }
    Ok(selected)
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
    migrate_legacy_prefix_with_resolutions(baseline, entries, &[])
}

/// Replay with explicit, evidenced choices for every ambiguous carried handle in this prefix.
pub fn migrate_legacy_prefix_with_resolutions(
    baseline: &Graph,
    entries: &[AttributedDelta],
    resolutions: &[LegacyResourceOriginResolution],
) -> Result<MigratedReplay, LegacyMigrationError> {
    migrate_legacy_prefix_with_content_context(baseline, entries, resolutions, None)
}

/// Supply the original mere namespace for unattributed legacy tag concepts.
pub fn migrate_legacy_prefix_with_content_context(
    baseline: &Graph,
    entries: &[AttributedDelta],
    resolutions: &[LegacyResourceOriginResolution],
    context: Option<&super::super::legacy_content_migration::LegacyContentContext>,
) -> Result<MigratedReplay, LegacyMigrationError> {
    if let Some(context) = context {
        let iri = url::Url::parse(&context.original_mere_iri).map_err(|_| {
            LegacyMigrationError("original-mere content namespace must be an absolute IRI".into())
        })?;
        if iri.scheme().is_empty()
            || context
                .original_mere_iri
                .chars()
                .any(|c| c.is_whitespace() || c.is_control())
        {
            return Err(LegacyMigrationError(
                "invalid original-mere content namespace".into(),
            ));
        }
    }
    let diagnostics = probe_legacy_mint_links(baseline, entries)?;
    let resolutions = validated_resolutions(&diagnostics, resolutions)?;
    let mut replay = Replay {
        source: baseline.clone(),
        output: baseline.clone(),
        attribution: ReplayAttribution::from_baseline(baseline),
        origins: BTreeMap::new(),
        active: BTreeMap::new(),
        aggregate_origins: BTreeMap::new(),
        aggregate_active: BTreeMap::new(),
        resolutions,
        content: Default::default(),
        content_context: context.cloned(),
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
    replay.source.stage_edges();
    replay.output.stage_edges();
    let before = replay.output.to_snapshot();
    let before_facets = replay.output.facets().clone();
    replay.sync(&Pairs::new(), true, None)?;
    replay.content.sync(
        &replay.source,
        &mut replay.output,
        replay.content_context.as_ref(),
        None,
    )?;
    let mut baseline_effects = facet_effects_between(&before_facets, replay.output.facets());
    baseline_effects.extend(effects_between(
        &before,
        &replay.output.to_snapshot(),
        replay.output.facets(),
    )?);
    let mut entry_effects = Vec::with_capacity(entries.len());
    for (index, entry) in entries.iter().enumerate() {
        let before = replay.output.to_snapshot();
        let before_facets = replay.output.facets().clone();
        let old_pairs = surface_pairs(&replay.source)?;
        let typed_pair = typed_resource_pair(&entry.delta)?;
        replay.content.typed_record(&entry.delta);
        if let Some(pair) = typed_pair {
            // This explicit replacement supersedes only this resource pair's old membership.
            replay.active.retain(|_, member| member.resource != pair);
            replay
                .aggregate_active
                .retain(|_, member| member.resource != pair);
            let Outcome::Apply(delta) = replay.attribution.replay(&entry.delta, &entry.author)
            else {
                return Err(LegacyMigrationError(
                    "invalid typed resource capture".into(),
                ));
            };
            let super::super::apply::GraphDelta::ReplaySetResourceEdgesByIds { edges, .. } = &delta
            else {
                unreachable!()
            };
            let expected = edges.clone();
            let _ = replay_graph_deltas_onto(&mut replay.output, [delta], OnRefusal::Drop);
            let actual = replay
                .output
                .persisted_resource_edges_between(pair.0, pair.1);
            if actual != expected {
                return Err(LegacyMigrationError(
                    "typed resource pair replacement was refused".into(),
                ));
            }
        } else {
            let _ = replay_attributed_deltas_onto(
                &mut replay.source,
                &mut replay.attribution,
                [entry],
                OnRefusal::Drop,
            );
            if !legacy_edge_edit(&entry.delta) {
                match entry.delta.replay_as(&entry.author) {
                    Outcome::Apply(delta) => {
                        let _ =
                            replay_graph_deltas_onto(&mut replay.output, [delta], OnRefusal::Drop);
                    },
                    Outcome::Skip => {},
                    Outcome::Malformed(bad) => {
                        return Err(LegacyMigrationError(format!(
                            "legacy journal entry {index}: {bad}"
                        )));
                    },
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
        let after_original = replay.output.to_snapshot();
        let after_original_facets = replay.output.facets().clone();
        replay.sync(&old_pairs, false, Some(entry))?;
        replay.content.sync(
            &replay.source,
            &mut replay.output,
            replay.content_context.as_ref(),
            Some(&entry.author),
        )?;
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
        effective.extend(facet_effects_between(
            &before_facets,
            replay.output.facets(),
        ));
        effective.extend(effects_between(
            &before,
            &replay.output.to_snapshot(),
            replay.output.facets(),
        )?);
        // Retained raw writes can transiently touch Surface content that the
        // migration clears back to its previous value.
        let mut corrections = facet_effects_between(&after_original_facets, replay.output.facets());
        corrections.extend(effects_between(
            &after_original,
            &replay.output.to_snapshot(),
            replay.output.facets(),
        )?);
        effective.extend(corrections);
        entry_effects.push(effective);
    }
    replay
        .output
        .settle_staged_edges()
        .map_err(|error| LegacyMigrationError(error.to_string()))?;
    // The same checked boundary used by ordinary loads verifies final cross-store handles.
    Graph::try_from_snapshot(&replay.output.to_snapshot())
        .map_err(|error| LegacyMigrationError(error.to_string()))?;
    Ok(MigratedReplay {
        graph: replay.output,
        baseline_effects,
        entry_effects,
    })
}
