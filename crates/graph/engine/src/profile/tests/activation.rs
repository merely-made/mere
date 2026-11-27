// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! D8's activation check, in D13's two sets: every IRI the engine references
//! outside the workspace terms is in the core pack with a generated constant;
//! every workspace IRI is in the stand-in and not in the core pack.

use std::collections::BTreeSet;

use super::support::*;
use crate::model::TAGGED_WITH_IRI;
use crate::model::placement_table::PLACEMENT;
use crate::model::relation::legacy;
use crate::profile::core::{CORE_PACK_ID, ids};
use crate::profile::resolve::PackRef;

#[test]
fn engine_ids_are_in_the_core_pack_and_workspace_ids_in_the_stand_in() {
    let (core_set, workspace_set): (Vec<_>, Vec<_>) = PLACEMENT
        .iter()
        .partition(|(family, ..)| *family != "Arrangement");
    let mut core_iris: Vec<&str> = core_set.iter().map(|(_, _, iri, _)| *iri).collect();
    core_iris.push(TAGGED_WITH_IRI);
    let mut workspace_iris: Vec<&str> = workspace_set.iter().map(|(_, _, iri, _)| *iri).collect();
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
    let families: BTreeSet<String> = core_set
        .iter()
        .map(|(family, ..)| legacy::family_iri(family))
        .collect();
    assert_eq!(families.len(), 4);
    assert_eq!(core.terms.len(), core_iris.len() + families.len());
    for (family, _, iri, _) in &core_set {
        let entry = core.terms.entry(iri).unwrap();
        assert_eq!(entry.family, Some(legacy::family_iri(family)), "{iri}");
        assert_eq!(
            entry.family_id,
            core.terms.id(&legacy::family_iri(family)),
            "{iri}'s family id"
        );
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
    assert_eq!(core.terms.id_of_uuid(&uuid), Some(ids::REL_CITES));
    // The instrument's own control: RFC 9562 Appendix A.4, in the DNS namespace.
    assert_eq!(
        uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_DNS, b"www.example.com").to_string(),
        "2ed6657d-e927-568b-95e1-2665a8aea6a2"
    );
    // The process's core profile is the same table.
    assert_eq!(crate::profile::core::terms(), &core.terms);
}
