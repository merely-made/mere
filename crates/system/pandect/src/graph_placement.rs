// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Explicit input placement; a missing profile remains unqualified.

use crate::NodeFacetStore;
use kernel::graph::legacy_resource_migration::LegacyResourceOriginResolution;
use kernel::graph::{AttributedDelta, CapturedDelta, Graph, GraphJournal, Seq};
use kernel::persistence::GraphSnapshot;
use muniment::Journal;
use serde::{Deserialize, Serialize};

/// The writer's placement contract, independent of the claims currently held.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PlacementProfile {
    LegacySurfaceV1,
    RecordedStrataV1,
}

/// The retained baseline and the permanently qualified legacy journal prefix.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SessionPlacement {
    pub baseline: PlacementProfile,
    /// `None` is the empty prefix, never the session's future journal.
    pub legacy_journal_until: Option<Seq>,
    #[serde(default)]
    pub origins: Vec<LegacyResourceOriginResolution>,
}

impl SessionPlacement {
    pub fn recorded() -> Self {
        Self {
            baseline: PlacementProfile::RecordedStrataV1,
            legacy_journal_until: None,
            origins: Vec::new(),
        }
    }

    pub fn legacy(until: Seq, origins: Vec<LegacyResourceOriginResolution>) -> Self {
        Self {
            baseline: PlacementProfile::LegacySurfaceV1,
            legacy_journal_until: Some(until),
            origins,
        }
    }

    pub(crate) fn cutoff(&self) -> Seq {
        self.legacy_journal_until.unwrap_or(Seq(0))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct FrozenGraph {
    pub graph: GraphSnapshot,
    pub facets: NodeFacetStore,
}

impl PartialEq for FrozenGraph {
    fn eq(&self, other: &Self) -> bool {
        let semantic = |graph: &GraphSnapshot| {
            let mut value = serde_json::to_value(graph)?;
            // The envelope clock is not graph truth; its stored bytes remain
            // covered by the translation checksum.
            if let Some(fields) = value.as_object_mut() {
                fields.remove("timestamp_secs");
            }
            Ok::<_, serde_json::Error>(value)
        };
        self.facets == other.facets
            && matches!(
            (semantic(&self.graph), semantic(&other.graph)),
            (Ok(left), Ok(right)) if left == right)
    }
}

impl FrozenGraph {
    pub fn of(graph: &Graph) -> Self {
        let mut snapshot = graph.to_snapshot();
        // Surface edge slots can be reused after migration; full records are unordered.
        snapshot.edges.sort_by_cached_key(|edge| {
            serde_json::to_vec(edge).expect("persisted surface edge encodes")
        });
        Self {
            graph: snapshot,
            facets: graph.facets().clone(),
        }
    }

    pub fn materialize(&self) -> Result<Graph, String> {
        let mut graph =
            Graph::try_from_recorded_snapshot(&self.graph).map_err(|error| error.to_string())?;
        *graph.facets_mut() = self.facets.clone();
        Ok(graph)
    }
}

/// An exact translation, separate from its retained original authority.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct LegacyTranslationReceipt {
    version: u32,
    pub source_digest: String,
    pub translation_digest: String,
    pub placement: SessionPlacement,
    pub baseline: FrozenGraph,
    pub entry_effects: Vec<Vec<CapturedDelta>>,
    legacy_final: FrozenGraph,
}

impl LegacyTranslationReceipt {
    pub fn new(
        source_digest: String,
        placement: SessionPlacement,
        baseline: FrozenGraph,
        entry_effects: Vec<Vec<CapturedDelta>>,
        legacy_final: FrozenGraph,
    ) -> Result<Self, String> {
        let mut receipt = Self {
            version: 1,
            source_digest,
            translation_digest: String::new(),
            placement,
            baseline,
            entry_effects,
            legacy_final,
        };
        receipt.translation_digest = receipt.digest()?;
        Ok(receipt)
    }

    fn digest(&self) -> Result<String, String> {
        let bytes = serde_json::to_vec(&(
            self.version,
            &self.source_digest,
            &self.placement,
            &self.baseline,
            &self.entry_effects,
            &self.legacy_final,
        ))
        .map_err(|error| error.to_string())?;
        Ok(blake3::hash(&bytes).to_hex().to_string())
    }

    pub fn validate(
        &self,
        placement: &SessionPlacement,
        source_digest: &str,
        entries: &[AttributedDelta],
    ) -> Result<(), String> {
        if self.version != 1
            || self.placement != *placement
            || placement.baseline != PlacementProfile::LegacySurfaceV1
            || self.entry_effects.len() != placement.cutoff().index()
            || placement.cutoff().index() > entries.len()
        {
            return Err("invalid legacy translation profile or cursor".into());
        }
        if self.source_digest != source_digest {
            return Err("retained legacy source changed".into());
        }
        if self.translation_digest != self.digest()? {
            return Err("legacy translation checksum mismatch".into());
        }
        for effect in self.entry_effects.iter().flatten() {
            validate_effect_ids(effect)?;
        }
        let graph = self
            .graph_at(entries, placement.cutoff())?
            .ok_or("invalid legacy translation cursor")?;
        if FrozenGraph::of(&graph) != self.legacy_final {
            return Err("legacy translation effects do not reproduce their frozen result".into());
        }
        self.legacy_final.materialize()?;
        Ok(())
    }

    pub fn journal(&self, entries: &[AttributedDelta], cursor: Seq) -> Option<GraphJournal> {
        if cursor.index() > entries.len() {
            return None;
        }
        let mut translated = Journal::new();
        for (index, entry) in entries[..cursor.index()].iter().enumerate() {
            if let Some(effects) = self.entry_effects.get(index) {
                for effect in effects {
                    translated.append(AttributedDelta {
                        author: entry.author.clone(),
                        delta: effect.clone(),
                    });
                }
            } else {
                translated.append(entry.clone());
            }
        }
        Some(GraphJournal::from_log(translated))
    }

    pub fn graph_at(
        &self,
        entries: &[AttributedDelta],
        cursor: Seq,
    ) -> Result<Option<Graph>, String> {
        let Some(journal) = self.journal(entries, cursor) else {
            return Ok(None);
        };
        Ok(journal.snapshot_at_from(&self.baseline.materialize()?, journal.live_cursor()))
    }

    pub fn effect_cursor(&self, cursor: Seq) -> Seq {
        let prefix = cursor.index().min(self.entry_effects.len());
        Seq(self.entry_effects[..prefix]
            .iter()
            .map(Vec::len)
            .sum::<usize>() as u64
            + cursor.0.saturating_sub(self.entry_effects.len() as u64))
    }
}

fn validate_effect_ids(effect: &CapturedDelta) -> Result<(), String> {
    let encoded = serde_json::to_value(effect).map_err(|error| error.to_string())?;
    let (_, fields) = encoded
        .as_object()
        .and_then(|value| value.iter().next())
        .ok_or("invalid legacy translation capture")?;
    for (key, value) in fields
        .as_object()
        .ok_or("invalid legacy translation fields")?
    {
        if matches!(
            key.as_str(),
            "id" | "node_id"
                | "from_id"
                | "to_id"
                | "child_id"
                | "parent_id"
                | "resource_id"
                | "from_resource_id"
                | "to_resource_id"
                | "surface_id"
        ) && !value.is_null()
        {
            let id = value
                .as_str()
                .ok_or("invalid legacy translation stable id")?;
            uuid::Uuid::parse_str(id).map_err(|_| "invalid legacy translation stable id")?;
        }
    }
    Ok(())
}

pub(crate) fn source_digest(
    placement: &SessionPlacement,
    baseline: Option<&[u8]>,
    entries: &[Vec<u8>],
) -> Result<String, String> {
    let mut hash = blake3::Hasher::new();
    hash.update(b"mere.legacy-translation-source/v1");
    let mut field = |bytes: Option<&[u8]>| {
        hash.update(&[u8::from(bytes.is_some())]);
        if let Some(bytes) = bytes {
            hash.update(&(bytes.len() as u64).to_le_bytes());
            hash.update(bytes);
        }
    };
    let profile = serde_json::to_vec(placement).map_err(|error| error.to_string())?;
    field(Some(&profile));
    field(baseline);
    for entry in entries {
        field(Some(entry));
    }
    Ok(hash.finalize().to_hex().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn recorded_receipt_preserves_same_host_resource_containment_without_surface_duplicates() {
        use kernel::persistence::PersistedContainmentSubKind;
        let original = kernel::graph::replay_captured_deltas([
            CapturedDelta::ReplayAddNodeWithIdIfMissing {
                id: Uuid::from_u128(1).to_string(),
                url: "https://same-host.test/".into(),
                position: [0.0, 0.0],
            },
            CapturedDelta::ReplayAddNodeWithIdIfMissing {
                id: Uuid::from_u128(2).to_string(),
                url: "https://same-host.test/child".into(),
                position: [1.0, 0.0],
            },
        ]);
        let source = Graph::try_from_snapshot(&original.to_snapshot()).unwrap();
        let has = |edges: &[kernel::persistence::PersistedEdge], kind| {
            edges.iter().any(|edge| {
                edge.containment
                    .as_ref()
                    .is_some_and(|bucket| bucket.sub_kinds.contains(&kind))
            })
        };
        assert!(has(
            &source.to_snapshot().edges,
            PersistedContainmentSubKind::Domain
        ));
        assert!(has(
            &source.to_snapshot().edges,
            PersistedContainmentSubKind::UrlPath
        ));
        let migrated =
            kernel::graph::legacy_resource_migration::migrate_legacy_prefix(&source, &[]).unwrap();
        let final_state = FrozenGraph::of(&migrated.graph);
        assert!(has(
            &final_state.graph.resource_edges,
            PersistedContainmentSubKind::Domain
        ));
        assert!(has(
            &final_state.graph.resource_edges,
            PersistedContainmentSubKind::UrlPath
        ));
        assert!(!has(
            &final_state.graph.edges,
            PersistedContainmentSubKind::Domain
        ));
        assert!(!has(
            &final_state.graph.edges,
            PersistedContainmentSubKind::UrlPath
        ));
        let mut baseline = source;
        kernel::graph::replay_captured_deltas_onto(&mut baseline, migrated.baseline_effects);
        let placement = SessionPlacement::legacy(Seq(0), vec![]);
        let receipt = LegacyTranslationReceipt::new(
            "same-host-source".into(),
            placement.clone(),
            FrozenGraph::of(&baseline),
            vec![],
            final_state.clone(),
        )
        .unwrap();
        receipt
            .validate(&placement, "same-host-source", &[])
            .unwrap();
        for _ in 0..2 {
            assert_eq!(
                FrozenGraph::of(&receipt.baseline.materialize().unwrap()),
                final_state
            );
            assert_eq!(
                FrozenGraph::of(&receipt.graph_at(&[], Seq(0)).unwrap().unwrap()),
                final_state
            );
        }
        let legacy = Graph::try_from_snapshot(&final_state.graph)
            .unwrap()
            .to_snapshot();
        assert!(
            has(&legacy.edges, PersistedContainmentSubKind::Domain),
            "legacy loader retains its existing derivation behavior"
        );
        assert!(has(&legacy.edges, PersistedContainmentSubKind::UrlPath));
        assert_eq!(legacy.resource_edges, final_state.graph.resource_edges);
    }

    #[test]
    fn frozen_surface_edge_records_ignore_slots_but_preserve_exact_contents_and_multiplicity() {
        let grouped = |from: u128, to: u128| CapturedDelta::ReplayAssertRelationByIds {
            from_id: Uuid::from_u128(from).to_string(),
            to_id: Uuid::from_u128(to).to_string(),
            assertion: kernel::graph::EdgeAssertion::Semantic {
                sub_kind: kernel::graph::SemanticSubKind::UserGrouped,
                label: Some("held".into()),
                decay_progress: None,
            },
        };
        let mut graph = kernel::graph::replay_captured_deltas((1..=3).map(|id| {
            CapturedDelta::ReplayAddNodeWithIdIfMissing {
                id: Uuid::from_u128(id).to_string(),
                url: format!("https://surface-{id}.test/"),
                position: [id as f32, 0.0],
            }
        }));
        kernel::graph::replay_captured_deltas_onto(&mut graph, [grouped(1, 2), grouped(2, 3)]);
        let original = FrozenGraph::of(&graph);
        assert_eq!(original.graph.edges.len(), 2);
        let mut alternate = original.clone();
        alternate.graph.edges.reverse();
        assert_eq!(FrozenGraph::of(&alternate.materialize().unwrap()), original);

        for field in ["id", "source", "time", "endpoints", "dropped", "duplicate"] {
            let mut changed = original.clone();
            match field {
                "id" => changed.graph.edges[0].semantic.as_mut().unwrap().statements[0]
                    .statement_id
                    .push_str("-changed"),
                "source" => {
                    changed.graph.edges[0].semantic.as_mut().unwrap().statements[0].provenance_iri =
                        Some("urn:mere:other-source".into())
                },
                "time" => {
                    changed.graph.edges[0].semantic.as_mut().unwrap().statements[0].asserted_at_ms =
                        Some(42)
                },
                "endpoints" => changed.graph.edges[0].to_node_id = Uuid::from_u128(3).to_string(),
                "dropped" => {
                    changed.graph.edges.pop();
                },
                "duplicate" => changed.graph.edges.push(changed.graph.edges[0].clone()),
                _ => unreachable!(),
            }
            assert_ne!(
                FrozenGraph::of(&changed.materialize().unwrap()),
                original,
                "changed {field} retained as exact difference"
            );
        }

        let mut two_assertions = original.clone();
        let statements = &mut two_assertions.graph.edges[0]
            .semantic
            .as_mut()
            .unwrap()
            .statements;
        let mut second = statements[0].clone();
        second.statement_id.push_str("-second");
        second.provenance_iri = Some("urn:mere:second-source".into());
        statements.push(second);
        let ordered = FrozenGraph::of(&two_assertions.materialize().unwrap());
        two_assertions.graph.edges[0]
            .semantic
            .as_mut()
            .unwrap()
            .statements
            .reverse();
        assert_ne!(
            FrozenGraph::of(&two_assertions.materialize().unwrap()),
            ordered,
            "nested statement order remains exact"
        );

        let profile = SessionPlacement::legacy(Seq(0), vec![]);
        let receipt = LegacyTranslationReceipt::new(
            "source".into(),
            profile.clone(),
            original.clone(),
            vec![],
            original.clone(),
        )
        .unwrap();
        receipt.validate(&profile, "source", &[]).unwrap();
        let reordered_bytes = LegacyTranslationReceipt::new(
            "source".into(),
            profile.clone(),
            original,
            vec![],
            alternate,
        )
        .unwrap();
        assert_ne!(
            receipt.translation_digest, reordered_bytes.translation_digest,
            "checksum binds exact stored order even though frozen creation canonicalizes it"
        );
        assert!(reordered_bytes.validate(&profile, "source", &[]).is_err());
    }

    #[test]
    fn translation_receipt_validates_exact_frozen_data_before_replay() {
        let baseline = FrozenGraph::of(&Graph::new());
        let profile = SessionPlacement::legacy(Seq(0), vec![]);
        let raw = b"{\"graph\":{},\"facets\":{}}";
        let digest = source_digest(&profile, Some(raw), &[]).unwrap();
        let receipt = LegacyTranslationReceipt::new(
            digest.clone(),
            profile.clone(),
            baseline.clone(),
            vec![],
            baseline,
        )
        .unwrap();
        receipt.validate(&profile, &digest, &[]).unwrap();
        let mut later_clock = receipt.legacy_final.clone();
        later_clock.graph.timestamp_secs = later_clock.graph.timestamp_secs.saturating_add(100);
        assert_eq!(
            later_clock, receipt.legacy_final,
            "envelope clock is excluded only from semantic equality"
        );
        let same_truth = LegacyTranslationReceipt::new(
            digest.clone(),
            profile.clone(),
            receipt.baseline.clone(),
            vec![],
            later_clock,
        )
        .unwrap();
        assert_ne!(
            same_truth.translation_digest, receipt.translation_digest,
            "checksum still binds the exact stored envelope"
        );
        same_truth.validate(&profile, &digest, &[]).unwrap();
        let mut changed_truth = receipt.legacy_final.clone();
        changed_truth
            .graph
            .resources
            .push(kernel::persistence::PersistedResourceRecord {
                canonical_iri: "urn:mere:receipt:changed-truth".into(),
                facets: Vec::new(),
            });
        assert_ne!(changed_truth, receipt.legacy_final);
        let wrong_result = LegacyTranslationReceipt::new(
            digest.clone(),
            profile.clone(),
            receipt.baseline.clone(),
            vec![],
            changed_truth,
        )
        .unwrap();
        assert!(wrong_result.validate(&profile, &digest, &[]).is_err());
        let mut changed_facets = receipt.legacy_final.clone();
        changed_facets
            .facets
            .set(
                Uuid::from_u128(1),
                chartulary::FacetId::new("receipt.control"),
                serde_json::json!({"changed":true}),
                &chartulary::AcceptAll,
            )
            .unwrap();
        assert_ne!(changed_facets, receipt.legacy_final);
        let wrong_facets = LegacyTranslationReceipt::new(
            digest.clone(),
            profile.clone(),
            receipt.baseline.clone(),
            vec![],
            changed_facets,
        )
        .unwrap();
        assert!(wrong_facets.validate(&profile, &digest, &[]).is_err());
        let mut changed = receipt.clone();
        changed.translation_digest.push('0');
        assert!(changed.validate(&profile, &digest, &[]).is_err());
        assert!(receipt.validate(&profile, "changed-source", &[]).is_err());
        assert!(
            receipt
                .validate(&SessionPlacement::legacy(Seq(1), vec![]), &digest, &[])
                .is_err()
        );
        let mut changed_raw = raw.to_vec();
        changed_raw.push(b'\n');
        assert_ne!(
            source_digest(&profile, Some(&changed_raw), &[]).unwrap(),
            digest,
            "hash binds raw bytes, not their parsed values"
        );
        assert_ne!(
            source_digest(&profile, None, &[]).unwrap(),
            digest,
            "absent source stays distinct"
        );
        assert_eq!(
            receipt.graph_at(&[], Seq(0)).unwrap().unwrap().node_count(),
            0
        );
        assert!(receipt.graph_at(&[], Seq(1)).unwrap().is_none());
        receipt.validate(&profile, &digest, &[]).unwrap();
    }

    #[test]
    fn cutover_and_origin_choices_are_exact_and_missing_cursor_is_empty() {
        let choice = LegacyResourceOriginResolution {
            statement_id: "opaque\nhandle".into(),
            from_resource_id: Uuid::from_u128(1),
            to_resource_id: Uuid::from_u128(2),
        };
        let placement = SessionPlacement::legacy(Seq(7), vec![choice]);
        let bytes = serde_json::to_vec(&placement).unwrap();
        let decoded: SessionPlacement = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(decoded, placement);
        assert_eq!(decoded.cutoff(), Seq(7));
        let empty: SessionPlacement = serde_json::from_value(serde_json::json!({
            "baseline": "legacy-surface-v1", "legacy_journal_until": null,
        }))
        .unwrap();
        assert_eq!(empty.cutoff(), Seq(0));
        assert!(empty.origins.is_empty());
        assert_eq!(SessionPlacement::recorded().cutoff(), Seq(0));
        assert!(
            serde_json::from_value::<SessionPlacement>(serde_json::json!({
                "baseline": "future-v9", "legacy_journal_until": null,
            }))
            .is_err()
        );
        assert!(
            serde_json::from_value::<SessionPlacement>(serde_json::json!({
                "baseline": "recorded-strata-v1", "guess_from_resources": true,
            }))
            .is_err()
        );
    }
}
