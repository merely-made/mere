// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Declared-lens D41: the engine expands a CURIE over the core pack's
//! prefixes, which build.rs keeps from the authored form.

use crate::profile::core::{self, expand, ids, prefixes::PREFIXES};

#[test]
fn a_core_curie_expands_to_its_term() {
    assert_eq!(
        expand("rel:supports").as_deref(),
        Some(core::iri(ids::REL_SUPPORTS))
    );
    assert_eq!(
        expand("rel:next-step").as_deref(),
        Some(core::iri(ids::REL_NEXT_STEP))
    );
    assert_eq!(
        expand("family:semantic").as_deref(),
        Some(core::iri(ids::FAMILY_SEMANTIC))
    );
    // Every core term is reachable from some prefix.
    for (iri, _) in ids::TERMS {
        assert!(
            PREFIXES.iter().any(|(_, ns)| iri.starts_with(ns)),
            "{iri} has no prefix"
        );
    }
}

#[test]
fn text_that_is_not_a_core_curie_does_not_expand() {
    assert_eq!(expand("supports"), None, "a bare local name");
    assert_eq!(expand("cito:cites"), None, "a prefix the core pack lacks");
    assert_eq!(
        expand("https://mere.computer/ns/rel#supports"),
        None,
        "a full IRI"
    );
    assert_eq!(expand("urn:x:y"), None);
}
