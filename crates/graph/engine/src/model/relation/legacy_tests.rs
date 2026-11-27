// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::model::placement_table::PLACEMENT;
use crate::model::relation::RelationSelector;

#[test]
fn every_stored_name_reads_as_the_iri_it_meant() {
    for (family, sub_kind, iri, _) in PLACEMENT {
        assert_eq!(predicate_iri(family, sub_kind), iri, "{family}::{sub_kind}");
    }
    assert_eq!(
        family_iri("Containment"),
        core::iri(ids::FAMILY_CONTAINMENT)
    );
}

#[test]
fn a_name_the_core_pack_lacks_keeps_its_minting_scheme() {
    // Retired workspace names read as IRIs no table holds, so they drop.
    let retired = predicate_iri("Arrangement", "TabNeighbor");
    assert!(retired.ends_with("#arrangement.tab-neighbor"), "{retired}");
    assert!(core::terms().id(&retired).is_none());
}

#[test]
fn old_and_new_selectors_and_assertions_read_alike() {
    let old: RelationSelector = serde_json::from_str(r#"{"Containment":"UserFolder"}"#).unwrap();
    let new: RelationSelector =
        serde_json::from_str(&serde_json::to_string(&old).unwrap()).unwrap();
    assert_eq!(old, new);
    assert_eq!(
        old,
        RelationSelector::predicate(core::iri(ids::REL_CONTAINMENT_USER_FOLDER))
    );
    let family: RelationSelector = serde_json::from_str(r#"{"Family":"Provenance"}"#).unwrap();
    assert_eq!(family, RelationSelector::family(ids::FAMILY_PROVENANCE));
    let assertion: LegacyAssertion =
        serde_json::from_str(r#"{"Semantic":{"sub_kind":"Cites","label":"why"}}"#).unwrap();
    assert_eq!(assertion.predicate, core::iri(ids::REL_CITES));
    assert_eq!(assertion.label.as_deref(), Some("why"));
    // Negative control: two entries are refused, not merged.
    assert!(serde_json::from_str::<RelationSelector>(r#"{"Family":"a","Semantic":"b"}"#).is_err());
}
