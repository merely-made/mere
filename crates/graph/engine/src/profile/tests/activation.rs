// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! D8's activation check, in D13's two sets: every IRI the engine references
//! outside the workspace terms is in the core pack with a generated constant;
//! every workspace IRI is in the stand-in and not in the core pack.

use std::collections::BTreeSet;

use strum::IntoEnumIterator;

use super::support::*;
use crate::model::{
    ArrangementSubKind, ContainmentSubKind, EdgeFamily, ImportedSubKind, ProvenanceSubKind,
    RelationKind, SemanticSubKind, TAGGED_WITH_IRI,
};
use crate::profile::core::{CORE_PACK_ID, ids};
use crate::profile::resolve::PackRef;

fn sub_kinds() -> Vec<RelationKind> {
    SemanticSubKind::iter()
        .map(RelationKind::Semantic)
        .chain(ContainmentSubKind::iter().map(RelationKind::Containment))
        .chain(ImportedSubKind::iter().map(RelationKind::Imported))
        .chain(ProvenanceSubKind::iter().map(RelationKind::Provenance))
        .chain(ArrangementSubKind::iter().map(RelationKind::Arrangement))
        .collect()
}

fn family_iri(family: EdgeFamily) -> String {
    format!(
        "https://mere.computer/ns/family#{}",
        format!("{family:?}").to_lowercase()
    )
}

#[test]
fn engine_ids_are_in_the_core_pack_and_workspace_ids_in_the_stand_in() {
    let (core_set, workspace_set): (Vec<_>, Vec<_>) = sub_kinds()
        .into_iter()
        .partition(|k| !matches!(k, RelationKind::Arrangement(_)));
    let mut core_iris: Vec<&str> = core_set.iter().filter_map(|k| k.predicate_iri()).collect();
    core_iris.push(TAGGED_WITH_IRI);
    let mut workspace_iris: Vec<&str> = workspace_set
        .iter()
        .filter_map(|k| k.predicate_iri())
        .collect();
    workspace_iris.push(TRAVERSAL_IRI);
    assert_eq!((core_iris.len(), workspace_iris.len()), (42, 4));

    let core = load_with(&PackRef::Core, &Source::default()).unwrap();
    let constants: BTreeSet<&str> = ids::TERMS.iter().map(|(iri, _)| *iri).collect();
    for iri in &core_iris {
        let entry = core
            .terms
            .entry(iri)
            .unwrap_or_else(|| panic!("{iri} not in the core pack"));
        assert_eq!(entry.pack, CORE_PACK_ID);
        assert!(constants.contains(iri), "{iri} has no generated constant");
    }
    let ws = workspace_profile();
    for iri in &workspace_iris {
        assert!(
            core.terms.entry(iri).is_none(),
            "{iri} is a workspace term in the core pack"
        );
        assert_eq!(
            ws.terms.entry(iri).map(|e| e.pack.as_str()),
            Some(WORKSPACE_ID),
            "{iri}"
        );
    }
    // The core pack holds nothing else but the four family terms.
    let families: BTreeSet<String> = core_set.iter().map(|k| family_iri(k.family())).collect();
    assert_eq!(families.len(), 4);
    assert_eq!(core.terms.len(), core_iris.len() + families.len());
    for kind in &core_set {
        let iri = kind.predicate_iri().unwrap();
        let family = core.terms.entry(iri).unwrap().family.clone();
        assert_eq!(family, Some(family_iri(kind.family())), "{iri}");
    }
    for family in &families {
        assert_eq!(
            core.terms.entry(family).map(|e| e.kind),
            Some(eidetic::pack::TermKind::Family)
        );
    }
}

#[test]
fn generated_constants_are_the_interned_ids() {
    let core = load_with(&PackRef::Core, &Source::default()).unwrap();
    let ws = workspace_profile();
    assert_eq!(ids::TERMS.len(), core.terms.len());
    for (iri, id) in ids::TERMS {
        assert_eq!(core.terms.id(iri), Some(id), "{iri} alone");
        assert_eq!(ws.terms.id(iri), Some(id), "{iri} under the workspace lens");
    }
    assert_eq!(
        core.terms.get(ids::REL_CITES).map(|e| e.iri.as_str()),
        Some("https://mere.computer/ns/rel#cites")
    );
    let uuid = core.terms.get(ids::REL_CITES).unwrap().uuid;
    // Known answer from Python's uuid.uuid5(NAMESPACE_URL, ...) (RFC 9562 §5.5).
    assert_eq!(uuid.to_string(), "f184840f-9510-5436-9d46-d021cf2525af");
    assert_eq!(uuid.get_version_num(), 5);
}
