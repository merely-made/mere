// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::model::author::Author;
use crate::model::predicate_declarations::PredicateDeclaration;
use crate::profile::core::{self, ids};

fn declared(id: &str, stratum: GraphStratum) -> PredicateDeclaration {
    PredicateDeclaration {
        declaration_id: id.into(),
        stratum,
        author: Author::person("local-test"),
        declared_at_ms: Some(1),
    }
}

fn one(stratum: GraphStratum) -> PredicateDeclarations {
    PredicateDeclarations {
        variants: vec![declared("d1", stratum)],
        selected: None,
    }
}

#[test]
fn declarations_layer_after_the_core_terms() {
    let surface = one(GraphStratum::Surface);
    let table = layered([("https://local.test/kept", &surface)]).unwrap();
    assert_eq!(
        table.stratum("https://local.test/kept"),
        GraphStratum::Surface
    );
    let entry = table.entry("https://local.test/kept").unwrap();
    assert_eq!(entry.pack, DECLARATIONS_PACK_ID);
    // The core ids stay where the generated constants say.
    assert_eq!(table.len(), core::terms().len() + 1);
    assert_eq!(table.id(core::iri(ids::REL_CITES)), Some(ids::REL_CITES));
    // Negative control: an undeclared IRI takes the core default.
    assert_eq!(
        table.stratum("https://local.test/other"),
        GraphStratum::Resource
    );
    // No declarations is the core table itself.
    assert_eq!(&layered([]).unwrap(), core::terms());
}

#[test]
fn conflicting_or_built_in_declarations_are_refused() {
    let split = PredicateDeclarations {
        variants: vec![
            declared("d1", GraphStratum::Surface),
            declared("d2", GraphStratum::Resource),
        ],
        selected: None,
    };
    assert!(matches!(
        layered([("https://local.test/split", &split)]),
        Err(LocalRefusal::Declaration(
            PredicateDeclarationError::Conflict { .. }
        ))
    ));
    let chosen = PredicateDeclarations {
        selected: Some("d2".into()),
        ..split
    };
    let table = layered([("https://local.test/split", &chosen)]).unwrap();
    assert_eq!(
        table.stratum("https://local.test/split"),
        GraphStratum::Resource
    );
    let surface = one(GraphStratum::Surface);
    assert!(matches!(
        layered([(core::iri(ids::REL_CITES), &surface)]),
        Err(LocalRefusal::Declaration(
            PredicateDeclarationError::BuiltIn { .. }
        ))
    ));
}
