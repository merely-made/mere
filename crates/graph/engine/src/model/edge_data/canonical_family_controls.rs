// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::profile::core::{self, ids};

#[test]
fn the_predicate_alone_classifies_a_record() {
    let vocabulary = Vocabulary::core();
    let mut record = SemanticStatement::new(
        core::iri(ids::REL_CONTAINMENT_COLLECTION_MEMBER).into(),
        None,
        GraphScope::Default,
        None,
        None,
    );
    assert_eq!(
        vocabulary.family(&record.predicate),
        Some(ids::FAMILY_CONTAINMENT)
    );
    assert!(!record.is_semantic());
    let mut data = SemanticData {
        statements: vec![record.clone()],
        ..Default::default()
    };
    data.rebuild_compat();
    assert_eq!(data.predicate, None, "a containment record is not semantic");
    record.predicate = core::iri(ids::REL_QUOTES).into();
    data.statements = vec![record.clone()];
    data.rebuild_compat();
    assert_eq!(data.predicate.as_deref(), Some(core::iri(ids::REL_QUOTES)));
    // An undeclared IRI is an open predicate, which counts as semantic.
    record.predicate = "urn:legacy:custom-predicate".into();
    assert_eq!(vocabulary.family(&record.predicate), None);
    assert_eq!(
        vocabulary.relation_family(&record.predicate),
        ids::FAMILY_SEMANTIC
    );
    assert!(record.is_semantic());
}

#[test]
fn a_record_stored_with_its_old_sub_kind_reads_by_its_predicate() {
    let stored = r#"{"statement_id":"s","predicate":"https://mere.computer/ns/rel#cites",
        "recognized_sub_kind":"Quotes","label":null}"#;
    let record: SemanticStatement = serde_json::from_str(stored).unwrap();
    assert_eq!(record.predicate, core::iri(ids::REL_CITES));
    let written = serde_json::to_string(&record).unwrap();
    assert!(!written.contains("recognized_sub_kind"), "{written}");
}
