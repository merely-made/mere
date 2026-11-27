// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::{PersonalGraphError, PersonalGraphEvent, RelationClaim};
use mere::model::{GraphScope, SemanticStatement};
use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};
use stickleback::Reject;
use uuid::Uuid;

pub(super) fn prepare_event(event: &mut PersonalGraphEvent) {
    if let PersonalGraphEvent::AssertRelation {
        relation,
        assertion_version,
        statement_id,
        asserted_at_ms,
        ..
    } = event
    {
        assertion_version.get_or_insert(1);
        if statement_id.is_none() {
            *statement_id = Some(new_statement_id(relation));
        }
        asserted_at_ms.get_or_insert_with(now_ms);
    }
    if let PersonalGraphEvent::RetractAssertion {
        withdrawn_at_ms, ..
    } = event
    {
        withdrawn_at_ms.get_or_insert_with(|| {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64
        });
    }
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn new_statement_id(relation: &RelationClaim) -> String {
    SemanticStatement::new(
        relation.predicate.clone(),
        None,
        GraphScope::Default,
        None,
        None,
    )
    .statement_id
}

pub(super) fn validate_event(event: &PersonalGraphEvent) -> Result<(), Reject> {
    let id = match event {
        PersonalGraphEvent::AssertRelation {
            relation,
            assertion_version,
            statement_id,
            asserted_at_ms,
            ..
        } => {
            if matches!(*assertion_version, Some(version) if version != 1) {
                return Err(Reject::new(
                    "unsupported-assertion-version",
                    "only exact assertion event version 1 is supported",
                ));
            }
            let paired = statement_id.is_some() && asserted_at_ms.is_some();
            let unpaired = statement_id.is_none() && asserted_at_ms.is_none();
            let valid = match *assertion_version {
                Some(1) => paired,
                Some(_) => false,
                None if relation.is_semantic() => paired || unpaired,
                None => unpaired,
            };
            if !valid {
                return Err(Reject::new(
                    "invalid-assertion-metadata",
                    "version 1 requires an assertion id and authored time; legacy events may omit both",
                ));
            }
            statement_id.as_deref()
        },
        PersonalGraphEvent::RetractAssertion { statement_id, .. } => Some(statement_id.as_str()),
        _ => None,
    };
    if id.is_some_and(|id| id.is_empty() || id.chars().any(|c| c.is_whitespace() || c.is_control()))
    {
        return Err(Reject::new(
            "invalid-assertion-id",
            "assertion id must be nonempty and contain no whitespace or control characters",
        ));
    }
    Ok(())
}

#[derive(Clone)]
struct AssertionOwner {
    from: Uuid,
    to: Uuid,
    predicate: String,
    asserter: String,
    operation: [u8; 32],
    asserted_at_ms: Option<u64>,
    note: Option<String>,
    method_iri: Option<String>,
    evidence_key: Vec<String>,
}

#[derive(Default)]
pub(super) struct IdentityIndex(BTreeMap<String, AssertionOwner>);

impl IdentityIndex {
    pub(super) fn observe(
        &mut self,
        event: &PersonalGraphEvent,
        asserter: &str,
        fallback: &str,
        operation: [u8; 32],
        causally_known: &std::collections::HashSet<[u8; 32]>,
    ) -> Result<(), PersonalGraphError> {
        match event {
            PersonalGraphEvent::AssertRelation {
                from,
                to,
                relation,
                assertion_version,
                statement_id,
                asserted_at_ms,
                basis,
                ..
            } => {
                if assertion_version.is_none() && statement_id.is_none() && !relation.is_semantic()
                {
                    return Ok(());
                }
                let id = statement_id.as_deref().unwrap_or(fallback);
                let basis = basis.clone().unwrap_or_default();
                let mut evidence_key = basis.evidence.clone();
                evidence_key.sort();
                evidence_key.dedup();
                let identity = AssertionOwner {
                    from: *from,
                    to: *to,
                    predicate: relation.predicate.clone(),
                    asserter: asserter.to_owned(),
                    operation,
                    asserted_at_ms: *asserted_at_ms,
                    note: basis.note.clone().or_else(|| relation.label.clone()),
                    method_iri: basis.method.map(|method| method.iri),
                    evidence_key,
                };
                if let Some(existing) = self.0.get(id) {
                    if existing.from != identity.from
                        || existing.to != identity.to
                        || existing.predicate != identity.predicate
                        || existing.asserter != identity.asserter
                        || existing.asserted_at_ms != identity.asserted_at_ms
                        || existing.note != identity.note
                        || existing.method_iri != identity.method_iri
                        || existing.evidence_key != identity.evidence_key
                    {
                        return Err(PersonalGraphError::Excluded(format!(
                            "assertion id {id} was reused for another claim"
                        )));
                    }
                } else {
                    self.0.insert(id.to_owned(), identity);
                }
            },
            PersonalGraphEvent::RetractAssertion {
                from,
                to,
                statement_id,
                ..
            } => match self.0.get(statement_id) {
                Some(owner)
                    if owner.from == *from
                        && owner.to == *to
                        && owner.asserter == asserter
                        && (owner.operation == operation
                            || causally_known.contains(&owner.operation)) => {},
                Some(_) => {
                    return Err(PersonalGraphError::Excluded(format!(
                        "retraction {statement_id} does not match its assertion pair and author"
                    )));
                },
                None => {
                    return Err(PersonalGraphError::Excluded(format!(
                        "retraction {statement_id} precedes or lacks its assertion"
                    )));
                },
            },
            _ => {},
        }
        Ok(())
    }
}

#[cfg(test)]
mod lifecycle_tests {
    use super::*;
    use mere::model::relation::legacy;
    use mere::model::{AssertionBasis, AssertionMethod};
    use mere::profile::core::{self, ids};
    use std::collections::HashSet;

    fn event(relation: RelationClaim, id: &str, version: Option<u8>) -> PersonalGraphEvent {
        PersonalGraphEvent::AssertRelation {
            from: Uuid::from_u128(1),
            to: Uuid::from_u128(2),
            relation,
            assertion_version: version,
            statement_id: Some(id.into()),
            asserted_at_ms: Some(10),
            basis: None,
        }
    }

    #[test]
    fn all_relation_families_prepare_as_version_one_exact_records() {
        let assertions = [
            RelationClaim::new(core::iri(ids::REL_CITES), None),
            RelationClaim::new(core::iri(ids::REL_CONTAINMENT_COLLECTION_MEMBER), None),
            RelationClaim::new(legacy::predicate_iri("Arrangement", "TileGroup"), None),
            RelationClaim::new(core::iri(ids::REL_IMPORTED_BOOKMARK_FOLDER), None),
            RelationClaim::new(core::iri(ids::REL_PROVENANCE_CLIPPED_FROM), None),
        ];
        for assertion in assertions {
            let mut event = PersonalGraphEvent::AssertRelation {
                from: Uuid::from_u128(1),
                to: Uuid::from_u128(2),
                relation: assertion,
                assertion_version: None,
                statement_id: None,
                asserted_at_ms: None,
                basis: None,
            };
            prepare_event(&mut event);
            validate_event(&event).unwrap();
            assert!(matches!(
                event,
                PersonalGraphEvent::AssertRelation {
                    assertion_version: Some(1),
                    statement_id: Some(_),
                    asserted_at_ms: Some(_),
                    ..
                }
            ));
            let first = event.clone();
            prepare_event(&mut event);
            assert_eq!(event, first, "preparing a current event is idempotent");
        }
    }

    #[test]
    fn assertion_event_versions_preserve_legacy_and_reject_unknown_or_partial_v1() {
        let assertion = RelationClaim::new(core::iri(ids::REL_CONTAINMENT_COLLECTION_MEMBER), None);
        assert!(validate_event(&event(assertion.clone(), "exact", Some(1))).is_ok());
        assert!(validate_event(&event(assertion.clone(), "legacy-id", None)).is_err());
        let legacy = PersonalGraphEvent::AssertRelation {
            from: Uuid::from_u128(1),
            to: Uuid::from_u128(2),
            relation: assertion.clone(),
            assertion_version: None,
            statement_id: None,
            asserted_at_ms: None,
            basis: None,
        };
        assert!(validate_event(&legacy).is_ok());
        assert!(validate_event(&event(assertion.clone(), "future", Some(9))).is_err());
        let partial = PersonalGraphEvent::AssertRelation {
            from: Uuid::from_u128(1),
            to: Uuid::from_u128(2),
            relation: assertion,
            assertion_version: Some(1),
            statement_id: Some("id-only".into()),
            asserted_at_ms: None,
            basis: None,
        };
        assert!(validate_event(&partial).is_err());
    }

    #[test]
    fn assertion_retraction_requires_matching_pair_author_and_causal_knowledge() {
        let id = "urn:assertion:one";
        let author = "urn:agent:alice";
        let operation = [1; 32];
        let assert = PersonalGraphEvent::AssertRelation {
            from: Uuid::from_u128(1),
            to: Uuid::from_u128(2),
            relation: RelationClaim::new(core::iri(ids::REL_CITES), None),
            assertion_version: None,
            statement_id: Some(id.into()),
            asserted_at_ms: Some(10),
            basis: None,
        };
        let mut index = IdentityIndex::default();
        let known = HashSet::from([operation]);
        index
            .observe(&assert, author, "fallback", operation, &known)
            .unwrap();

        let retract = |from, to| PersonalGraphEvent::RetractAssertion {
            from,
            to,
            statement_id: id.into(),
            withdrawn_at_ms: Some(11),
        };
        assert!(
            index
                .observe(
                    &retract(Uuid::from_u128(1), Uuid::from_u128(2)),
                    "urn:agent:bob",
                    "",
                    [2; 32],
                    &known
                )
                .is_err()
        );
        assert!(
            index
                .observe(
                    &retract(Uuid::from_u128(3), Uuid::from_u128(2)),
                    author,
                    "",
                    [2; 32],
                    &known
                )
                .is_err()
        );
        assert!(
            index
                .observe(
                    &retract(Uuid::from_u128(1), Uuid::from_u128(2)),
                    author,
                    "",
                    [2; 32],
                    &HashSet::new()
                )
                .is_err()
        );
        assert!(
            index
                .observe(
                    &retract(Uuid::from_u128(1), Uuid::from_u128(2)),
                    author,
                    "",
                    [2; 32],
                    &HashSet::from([[1; 32]])
                )
                .is_ok()
        );
        assert!(
            index
                .observe(
                    &retract(Uuid::from_u128(1), Uuid::from_u128(2)),
                    author,
                    "",
                    [2; 32],
                    &known
                )
                .is_ok()
        );
    }

    #[test]
    fn same_operation_retraction_requires_prior_assertion_event_order() {
        let id = "urn:assertion:ordered";
        let author = "urn:agent:alice";
        let operation = [7; 32];
        let assertion = PersonalGraphEvent::AssertRelation {
            from: Uuid::from_u128(1),
            to: Uuid::from_u128(2),
            relation: RelationClaim::new(core::iri(ids::REL_CITES), None),
            assertion_version: None,
            statement_id: Some(id.into()),
            asserted_at_ms: Some(10),
            basis: None,
        };
        let withdrawal = PersonalGraphEvent::RetractAssertion {
            from: Uuid::from_u128(1),
            to: Uuid::from_u128(2),
            statement_id: id.into(),
            withdrawn_at_ms: Some(11),
        };
        let mut ordered = IdentityIndex::default();
        ordered
            .observe(&assertion, author, "fallback", operation, &HashSet::new())
            .unwrap();
        ordered
            .observe(&withdrawal, author, "fallback", operation, &HashSet::new())
            .unwrap();

        let mut reversed = IdentityIndex::default();
        assert!(
            reversed
                .observe(&withdrawal, author, "fallback", operation, &HashSet::new())
                .is_err()
        );
        reversed
            .observe(&assertion, author, "fallback", operation, &HashSet::new())
            .unwrap();
    }

    #[test]
    fn reason_identity_ignores_evidence_order_duplicates_and_method_version() {
        let event = |evidence: Vec<String>, version: &str| PersonalGraphEvent::AssertRelation {
            from: Uuid::from_u128(1),
            to: Uuid::from_u128(2),
            relation: RelationClaim::new(core::iri(ids::REL_CITES), None),
            assertion_version: None,
            statement_id: Some("urn:assertion:same-reason".into()),
            asserted_at_ms: Some(10),
            basis: Some(AssertionBasis {
                method: Some(AssertionMethod {
                    iri: "urn:method:extractor".into(),
                    version: Some(version.into()),
                }),
                evidence,
                ..Default::default()
            }),
        };
        let mut index = IdentityIndex::default();
        let known = HashSet::new();
        index
            .observe(
                &event(vec!["urn:evidence:a".into(), "urn:evidence:b".into()], "v1"),
                "urn:agent:alice",
                "fallback",
                [1; 32],
                &known,
            )
            .unwrap();
        index
            .observe(
                &event(
                    vec![
                        "urn:evidence:b".into(),
                        "urn:evidence:a".into(),
                        "urn:evidence:a".into(),
                    ],
                    "v2",
                ),
                "urn:agent:alice",
                "fallback",
                [2; 32],
                &known,
            )
            .unwrap();
    }

    #[test]
    fn exact_identity_checks_kind_pair_author_and_reason_for_every_family() {
        let author = "urn:agent:alice";
        let known = HashSet::new();
        let assertion = RelationClaim::new(core::iri(ids::REL_CONTAINMENT_COLLECTION_MEMBER), None);
        let mut index = IdentityIndex::default();
        index
            .observe(
                &event(assertion.clone(), "same-id", Some(1)),
                author,
                "fallback",
                [1; 32],
                &known,
            )
            .unwrap();
        let independent = event(assertion.clone(), "other-id", Some(1));
        index
            .observe(&independent, author, "fallback", [2; 32], &known)
            .unwrap();
        assert!(
            index
                .observe(
                    &event(
                        RelationClaim::new(legacy::predicate_iri("Arrangement", "TileGroup"), None),
                        "same-id",
                        Some(1),
                    ),
                    author,
                    "fallback",
                    [3; 32],
                    &known,
                )
                .is_err()
        );

        let retargeted = PersonalGraphEvent::AssertRelation {
            from: Uuid::from_u128(9),
            to: Uuid::from_u128(2),
            relation: assertion.clone(),
            assertion_version: Some(1),
            statement_id: Some("same-id".into()),
            asserted_at_ms: Some(10),
            basis: None,
        };
        assert!(
            index
                .observe(&retargeted, author, "fallback", [4; 32], &known)
                .is_err()
        );
        let mut changed_reason = event(assertion, "same-id", Some(1));
        if let PersonalGraphEvent::AssertRelation { basis, .. } = &mut changed_reason {
            *basis = Some(AssertionBasis {
                note: Some("different reason".into()),
                ..Default::default()
            });
        }
        assert!(
            index
                .observe(&changed_reason, author, "fallback", [5; 32], &known)
                .is_err()
        );
        assert!(
            index
                .observe(
                    &event(
                        RelationClaim::new(core::iri(ids::REL_CONTAINMENT_COLLECTION_MEMBER), None),
                        "same-id",
                        Some(1),
                    ),
                    "urn:agent:bob",
                    "fallback",
                    [6; 32],
                    &known,
                )
                .is_err()
        );
    }

    #[test]
    fn authored_retraction_time_is_stamped_but_old_event_time_remains_absent() {
        let mut authored = PersonalGraphEvent::RetractAssertion {
            from: Uuid::from_u128(1),
            to: Uuid::from_u128(2),
            statement_id: "urn:assertion:old".into(),
            withdrawn_at_ms: None,
        };
        prepare_event(&mut authored);
        assert!(matches!(
            authored,
            PersonalGraphEvent::RetractAssertion {
                withdrawn_at_ms: Some(_),
                ..
            }
        ));
        let replayed: PersonalGraphEvent = serde_json::from_str(
            r#"{"type":"retract_assertion","from":"00000000-0000-0000-0000-000000000001","to":"00000000-0000-0000-0000-000000000002","statement_id":"urn:assertion:old"}"#,
        ).unwrap();
        assert!(matches!(
            replayed,
            PersonalGraphEvent::RetractAssertion {
                withdrawn_at_ms: None,
                ..
            }
        ));
    }
}
