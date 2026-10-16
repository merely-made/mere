// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Attribution recovery for pre-assertion-family Surface checkpoints.
//!
//! These checkpoints encoded relation membership in aggregate edge payloads,
//! so their later synthetic assertion IDs do not match raw replay IDs. This
//! tracker follows the old membership fact (directed pair + relation kind)
//! through replay and only repairs a structural legacy ID when that history
//! identifies its author.

use std::collections::{BTreeMap, BTreeSet};

use chartulary::Identified;
use uuid::Uuid;

use super::{AttributedDelta, CapturedDelta, EdgeAssertion, Graph, RelationKind};

/// Prefix reserved for IDs minted while converting an aggregate legacy
/// Surface member into a retained assertion. Canonical carried IDs are never
/// attribution-repaired by this adapter.
pub(crate) const LEGACY_FAMILY_ASSERTION_ID_PREFIX: &str = "urn:mere:legacy-assertion:v1:";

type Member = (Uuid, Uuid, u32);

#[derive(Clone, Debug, Default)]
struct MemberAttribution {
    held: bool,
    author_iri: Option<String>,
    ambiguous: bool,
}

/// Replay-local authorship evidence for aggregate Surface membership facts.
/// It is deliberately separate from statement-ID attribution because legacy
/// conversion minted IDs that differ from the original replay event IDs.
#[derive(Clone, Debug, Default)]
pub(crate) struct LegacyFamilyAttribution {
    members: BTreeMap<Member, MemberAttribution>,
}

impl LegacyFamilyAttribution {
    /// Seed known held members from the baseline. Unknown baseline actors stay
    /// unknown; a later checkpoint writer is not evidence of authorship.
    pub(crate) fn from_baseline(graph: &Graph) -> Self {
        let mut result = Self::default();
        for (member, (author, ambiguous)) in held_surface_members(graph) {
            let state = result.members.entry(member).or_default();
            state.held = true;
            merge_known_author(&mut state.author_iri, &mut state.ambiguous, author);
            state.ambiguous |= ambiguous;
        }
        result
    }

    /// Observe one replay step after applying this entry. The tracker itself
    /// carries the prior prefix state, so callers do not need to clone the
    /// graph per event. Full-pair snapshots preserve
    /// attribution already known for surviving members, but newly introduced
    /// members remain unknown. A real assertion event owns only the member it
    /// actually introduces.
    pub(crate) fn observe_delta(&mut self, after: &Graph, entry: &AttributedDelta) {
        let after_members = held_surface_members(after);
        let after_keys: BTreeSet<_> = after_members.keys().copied().collect();

        for (member, state) in &mut self.members {
            if state.held && !after_keys.contains(member) {
                state.held = false;
            }
        }

        for member in after_keys {
            let existing = self.members.entry(member).or_default();
            let was_held = existing.held;
            let is_direct_assertion = introduces_member(&entry.delta, member);

            if is_direct_assertion && !was_held {
                // A new generation starts with the actor on this replay entry.
                existing.author_iri = Some(entry.author.asserter_iri());
                existing.ambiguous = false;
            }

            existing.held = true;
            if !(is_full_surface_snapshot(&entry.delta) && !was_held)
                && let Some((author, ambiguous)) = after_members.get(&member).cloned()
            {
                merge_known_author(&mut existing.author_iri, &mut existing.ambiguous, author);
                existing.ambiguous |= ambiguous;
            }
        }
    }

    /// Repair only held, currently unknown rows created by legacy family
    /// conversion, and only when the complete replay prefix proves an actor.
    /// Does not change IDs, timestamps, canonical carried records, or Resource
    /// assertions.
    pub(crate) fn repair_checkpoint(&self, graph: &mut Graph) {
        let unknown = super::edge_data::UNKNOWN_LEGACY_ASSERTER_IRI;
        let keys: Vec<_> = graph
            .surface_relation_buckets()
            .filter_map(|(key, from, to, payload)| {
                let from_id = *graph.inner.node(from)?.id();
                let to_id = *graph.inner.node(to)?.id();
                let members: Vec<_> = payload
                    .assertions()
                    .iter()
                    .filter(|statement| {
                        statement.is_held()
                            && statement
                                .statement_id
                                .starts_with(LEGACY_FAMILY_ASSERTION_ID_PREFIX)
                            && statement
                                .provenance_iri
                                .as_deref()
                                .is_none_or(|source| source == unknown)
                    })
                    .filter_map(|statement| {
                        let member = (from_id, to_id, statement.relation_kind().tag());
                        self.members
                            .get(&member)
                            .filter(|state| state.held && !state.ambiguous)
                            .and_then(|state| state.author_iri.as_ref())
                            .map(|author| (statement.statement_id.clone(), author.clone()))
                    })
                    .collect();
                (!members.is_empty()).then_some((key, members))
            })
            .collect();

        let mut changed = false;
        for (key, repairs) in keys {
            if let Some(payload) = graph.inner.edge_mut(key) {
                for statement in &mut payload.assertions.statements {
                    if let Some((_, author)) =
                        repairs.iter().find(|(id, _)| *id == statement.statement_id)
                        && (statement.provenance_iri.is_none()
                            || statement.provenance_iri.as_deref() == Some(unknown))
                    {
                        statement.provenance_iri = Some(author.clone());
                        changed = true;
                    }
                }
            }
        }
        if changed {
            graph.bump_revision();
        }
    }
}

fn merge_known_author(slot: &mut Option<String>, ambiguous: &mut bool, incoming: Option<String>) {
    if *ambiguous {
        return;
    }
    match (slot.as_deref(), incoming.as_deref()) {
        (None, Some(_)) => *slot = incoming,
        (Some(current), Some(candidate)) if current != candidate => {
            *slot = None;
            *ambiguous = true;
        },
        _ => {},
    }
}

fn held_surface_members(graph: &Graph) -> BTreeMap<Member, (Option<String>, bool)> {
    let mut result = BTreeMap::<Member, (Option<String>, bool)>::new();
    let unknown = super::edge_data::UNKNOWN_LEGACY_ASSERTER_IRI;
    for (_, from, to, payload) in graph.surface_relation_buckets() {
        let Some(from_id) = graph.inner.node(from).map(|node| *node.id()) else {
            continue;
        };
        let Some(to_id) = graph.inner.node(to).map(|node| *node.id()) else {
            continue;
        };
        for statement in payload.assertions().iter().filter(|row| row.is_held()) {
            let author = statement
                .provenance_iri
                .as_ref()
                .filter(|iri| iri.as_str() != unknown)
                .cloned();
            let (author_slot, ambiguous) = result
                .entry((from_id, to_id, statement.relation_kind().tag()))
                .or_insert((None, false));
            merge_known_author(author_slot, ambiguous, author);
        }
    }
    result
}

fn introduces_member(delta: &CapturedDelta, member: Member) -> bool {
    match delta {
        CapturedDelta::ReplayAssertRelationByIds {
            from_id,
            to_id,
            assertion,
        } => {
            parse_pair(from_id, to_id) == Some((member.0, member.1))
                && assertion_kind(assertion).is_some_and(|kind| kind.tag() == member.2)
        },
        CapturedDelta::ReplayAssertSemanticPredicateByIds {
            from_id,
            to_id,
            predicate,
        }
        | CapturedDelta::ReplaySetEdgeSemanticPredicateByIds {
            from_id,
            to_id,
            predicate: Some(predicate),
        } => {
            parse_pair(from_id, to_id) == Some((member.0, member.1))
                && RelationKind::from_predicate_iri(predicate)
                    .unwrap_or(RelationKind::OpenPredicate)
                    .tag()
                    == member.2
        },
        _ => false,
    }
}

fn parse_pair(from: &str, to: &str) -> Option<(Uuid, Uuid)> {
    Some((Uuid::parse_str(from).ok()?, Uuid::parse_str(to).ok()?))
}

fn is_full_surface_snapshot(delta: &CapturedDelta) -> bool {
    matches!(delta, CapturedDelta::ReplaySetEdgesByIds { .. })
}

fn assertion_kind(assertion: &EdgeAssertion) -> Option<RelationKind> {
    Some(match assertion {
        EdgeAssertion::Semantic { sub_kind, .. } => RelationKind::Semantic(*sub_kind),
        EdgeAssertion::Containment { sub_kind } => RelationKind::Containment(*sub_kind),
        EdgeAssertion::Arrangement { sub_kind } => RelationKind::Arrangement(*sub_kind),
        EdgeAssertion::Imported { sub_kind } => RelationKind::Imported(*sub_kind),
        EdgeAssertion::Provenance { sub_kind } => RelationKind::Provenance(*sub_kind),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::apply::apply_graph_delta;
    use crate::graph::{Author, ContainmentSubKind, RelationSelector, SemanticStatement};
    use crate::types::{AssertionBasis, GraphScope};
    use euclid::default::Point2D;

    fn graph() -> Graph {
        let mut graph = Graph::new();
        graph.add_node_with_id(
            Uuid::from_u128(1),
            "https://a.test/".into(),
            Point2D::zero(),
        );
        graph.add_node_with_id(
            Uuid::from_u128(2),
            "https://b.test/".into(),
            Point2D::zero(),
        );
        graph
    }

    fn assertion() -> CapturedDelta {
        CapturedDelta::ReplayAssertRelationByIds {
            from_id: Uuid::from_u128(1).to_string(),
            to_id: Uuid::from_u128(2).to_string(),
            assertion: EdgeAssertion::Containment {
                sub_kind: ContainmentSubKind::CollectionMember,
            },
        }
    }

    fn step(graph: &mut Graph, tracker: &mut LegacyFamilyAttribution, entry: AttributedDelta) {
        let delta = entry.delta.replay_delta_as(&entry.author).unwrap();
        let _ = apply_graph_delta(graph, delta);
        tracker.observe_delta(graph, &entry);
    }

    fn force_structural_unknown(graph: &mut Graph) -> (String, u32) {
        let (key, _, _, _) = graph.surface_relation_buckets().next().unwrap();
        let payload = graph.inner.edge_mut(key).unwrap();
        let statement = payload
            .assertions
            .statements
            .iter_mut()
            .find(|statement| statement.is_held())
            .unwrap();
        statement.statement_id = format!("{LEGACY_FAMILY_ASSERTION_ID_PREFIX}test-id");
        statement.provenance_iri =
            Some(super::super::edge_data::UNKNOWN_LEGACY_ASSERTER_IRI.into());
        // Keep an explicitly unknown first time as a guard against causality
        // repair accidentally fabricating clocks.
        statement.first_asserted_ms = None;
        statement.asserted_at_ms = None;
        (
            statement.statement_id.clone(),
            statement.relation_kind().tag(),
        )
    }

    fn source(graph: &Graph) -> Option<String> {
        graph
            .surface_relation_buckets()
            .flat_map(|(_, _, _, payload)| payload.assertions())
            .find(|statement| statement.is_held())
            .and_then(|statement| statement.provenance_iri.clone())
    }

    #[test]
    fn prefix_replay_preserves_first_author_then_reassertion_owns_new_generation() {
        let mut graph = graph();
        let mut attribution = LegacyFamilyAttribution::from_baseline(&graph);
        let first = AttributedDelta {
            author: Author::person("https://author-a.test/"),
            delta: assertion(),
        };
        step(&mut graph, &mut attribution, first.clone());

        // A repeated membership ensure by another actor retains the original
        // generation's author.
        let duplicate = AttributedDelta {
            author: Author::person("https://author-b.test/"),
            delta: assertion(),
        };
        step(&mut graph, &mut attribution, duplicate);
        force_structural_unknown(&mut graph);
        attribution.repair_checkpoint(&mut graph);
        assert_eq!(source(&graph).as_deref(), Some("https://author-a.test/"));
        let row = graph
            .surface_relation_buckets()
            .flat_map(|(_, _, _, payload)| payload.assertions())
            .next()
            .unwrap();
        assert_eq!(row.first_asserted_ms, None);
        assert_eq!(row.asserted_at_ms, None);

        let withdraw = AttributedDelta {
            author: Author::person("https://author-b.test/"),
            delta: CapturedDelta::ReplayRetractRelationsByIds {
                from_id: Uuid::from_u128(1).to_string(),
                to_id: Uuid::from_u128(2).to_string(),
                selector: RelationSelector::Containment(ContainmentSubKind::CollectionMember),
            },
        };
        step(&mut graph, &mut attribution, withdraw);
        let reassert = AttributedDelta {
            author: Author::person("https://author-b.test/"),
            delta: assertion(),
        };
        step(&mut graph, &mut attribution, reassert);
        force_structural_unknown(&mut graph);
        attribution.repair_checkpoint(&mut graph);
        assert_eq!(source(&graph).as_deref(), Some("https://author-b.test/"));
    }

    #[test]
    fn snapshot_introduces_unknown_members_without_borrowing_checkpoint_writer() {
        let mut graph = graph();
        let mut attribution = LegacyFamilyAttribution::from_baseline(&graph);
        let author = Author::person("https://known.test/");
        step(
            &mut graph,
            &mut attribution,
            AttributedDelta {
                author: author.clone(),
                delta: assertion(),
            },
        );

        let first_payload = graph.surface_relation_buckets().next().unwrap().3.clone();
        let mut added_payload = crate::graph::EdgePayload::new();
        added_payload.push_persisted_semantic_statement(SemanticStatement {
            statement_id: format!("{LEGACY_FAMILY_ASSERTION_ID_PREFIX}new-kind"),
            predicate: crate::graph::RelationKind::Containment(ContainmentSubKind::UserFolder)
                .predicate_iri()
                .unwrap()
                .into(),
            recognized_sub_kind: None,
            label: Some("folder".into()),
            basis: AssertionBasis::default(),
            graph_scope: GraphScope::User,
            provenance_iri: Some(crate::graph::edge_data::UNKNOWN_LEGACY_ASSERTER_IRI.into()),
            first_asserted_ms: None,
            asserted_at_ms: None,
            withdrawn: None,
        });
        let entries = vec![
            crate::graph::snapshot::persisted_edge_for_ids(
                Uuid::from_u128(1),
                Uuid::from_u128(2),
                &first_payload,
            ),
            crate::graph::snapshot::persisted_edge_for_ids(
                Uuid::from_u128(1),
                Uuid::from_u128(2),
                &added_payload,
            ),
        ];
        let snapshot = AttributedDelta {
            author: Author::person("https://checkpoint-writer.test/"),
            delta: CapturedDelta::ReplaySetEdgesByIds {
                from_id: Uuid::from_u128(1).to_string(),
                to_id: Uuid::from_u128(2).to_string(),
                edges: entries,
            },
        };
        step(&mut graph, &mut attribution, snapshot);
        attribution.repair_checkpoint(&mut graph);
        let rows: Vec<_> = graph
            .surface_relation_buckets()
            .flat_map(|(_, _, _, payload)| payload.assertions())
            .collect();
        assert_eq!(rows.len(), 2);
        let added = rows
            .iter()
            .find(|row| row.statement_id.ends_with("new-kind"))
            .unwrap();
        assert_eq!(
            added.provenance_iri.as_deref(),
            Some(crate::graph::edge_data::UNKNOWN_LEGACY_ASSERTER_IRI)
        );
    }

    #[test]
    fn repair_never_rewrites_a_carried_canonical_statement_id() {
        let mut graph = graph();
        let mut attribution = LegacyFamilyAttribution::from_baseline(&graph);
        step(
            &mut graph,
            &mut attribution,
            AttributedDelta {
                author: Author::person("https://author.test/"),
                delta: assertion(),
            },
        );
        let (key, _, _, _) = graph.surface_relation_buckets().next().unwrap();
        let statement = graph
            .inner
            .edge_mut(key)
            .unwrap()
            .assertions
            .statements
            .iter_mut()
            .find(|statement| statement.is_held())
            .unwrap();
        statement.statement_id = "urn:carried:canonical-id".into();
        statement.provenance_iri =
            Some(super::super::edge_data::UNKNOWN_LEGACY_ASSERTER_IRI.into());

        attribution.repair_checkpoint(&mut graph);
        assert_eq!(
            source(&graph).as_deref(),
            Some(super::super::edge_data::UNKNOWN_LEGACY_ASSERTER_IRI)
        );
    }

    #[test]
    fn checkpoint_writer_does_not_replace_prefix_proven_legacy_member_author() {
        let mut graph = graph();
        let mut attribution = super::super::capture::ReplayAttribution::from_baseline(&graph);
        let mint = AttributedDelta {
            author: Author::person("https://original-author.test/"),
            delta: assertion(),
        };
        super::super::capture::replay_attributed_deltas_onto(&mut graph, &mut attribution, [&mint]);

        let mut carried = graph.to_snapshot().edges;
        let statement = &mut carried[0].retained_assertions_mut()[0];
        statement.statement_id = format!("{LEGACY_FAMILY_ASSERTION_ID_PREFIX}snapshot");
        statement.provenance_iri = None;
        let checkpoint = AttributedDelta {
            author: Author::person("https://checkpoint-writer.test/"),
            delta: CapturedDelta::ReplaySetEdgesByIds {
                from_id: Uuid::from_u128(1).to_string(),
                to_id: Uuid::from_u128(2).to_string(),
                edges: carried,
            },
        };
        super::super::capture::replay_attributed_deltas_onto(
            &mut graph,
            &mut attribution,
            [&checkpoint],
        );
        attribution.repair_checkpoint(&mut graph);
        assert_eq!(
            source(&graph).as_deref(),
            Some("https://original-author.test/")
        );
    }

    #[test]
    fn empty_prefix_and_unknown_baseline_do_not_invent_authors() {
        let mut graph = graph();
        let mut attribution = LegacyFamilyAttribution::from_baseline(&graph);
        attribution.repair_checkpoint(&mut graph);
        assert!(source(&graph).is_none());

        let mut payload = crate::graph::EdgePayload::new();
        payload.push_persisted_semantic_statement(SemanticStatement {
            statement_id: format!("{LEGACY_FAMILY_ASSERTION_ID_PREFIX}baseline"),
            predicate: crate::graph::RelationKind::Containment(
                ContainmentSubKind::CollectionMember,
            )
            .predicate_iri()
            .unwrap()
            .into(),
            recognized_sub_kind: None,
            label: None,
            basis: AssertionBasis::default(),
            graph_scope: GraphScope::User,
            provenance_iri: Some(crate::graph::edge_data::UNKNOWN_LEGACY_ASSERTER_IRI.into()),
            first_asserted_ms: None,
            asserted_at_ms: None,
            withdrawn: None,
        });
        let from = graph.get_node_key_by_id(Uuid::from_u128(1)).unwrap();
        let to = graph.get_node_key_by_id(Uuid::from_u128(2)).unwrap();
        graph.inner.connect(from, to, payload);
        let attribution = LegacyFamilyAttribution::from_baseline(&graph);
        attribution.repair_checkpoint(&mut graph);
        assert_eq!(
            source(&graph).as_deref(),
            Some(crate::graph::edge_data::UNKNOWN_LEGACY_ASSERTER_IRI)
        );
    }
}
