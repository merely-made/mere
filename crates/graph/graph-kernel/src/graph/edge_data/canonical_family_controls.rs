// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::graph::{ContainmentSubKind, EdgeFamily, RelationKind};

#[test]
fn canonical_predicate_owns_classification_and_legacy_hint_is_only_a_fallback() {
    let mut record = SemanticStatement::new(
        RelationKind::Containment(ContainmentSubKind::CollectionMember)
            .predicate_iri()
            .unwrap()
            .into(),
        Some(SemanticSubKind::Cites),
        None,
        GraphScope::Default,
        None,
        None,
    );
    assert_eq!(
        record.relation_kind(),
        RelationKind::Containment(ContainmentSubKind::CollectionMember)
    );
    assert!(!record.is_semantic());
    let mut data = SemanticData {
        statements: vec![record.clone()],
        ..Default::default()
    };
    data.rebuild_compat();
    assert!(data.sub_kinds.is_empty());
    record.predicate = predicate_iri(SemanticSubKind::Quotes).into();
    assert_eq!(
        record.relation_kind(),
        RelationKind::Semantic(SemanticSubKind::Quotes)
    );
    data.statements = vec![record.clone()];
    data.rebuild_compat();
    assert_eq!(data.sub_kinds, BTreeSet::from([SemanticSubKind::Quotes]));
    record.predicate = "urn:legacy:custom-predicate".into();
    assert_eq!(
        record.relation_kind(),
        RelationKind::Semantic(SemanticSubKind::Cites)
    );
    record.recognized_sub_kind = None;
    assert_eq!(record.relation_kind(), RelationKind::OpenPredicate);
    assert_eq!(record.relation_kind().family(), EdgeFamily::Semantic);
}
