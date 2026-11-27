// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Placement by the core pack's stratum axis (declared-lens L7). Effective
//! custom placement also consults the mere's durable declarations, layered
//! after the core pack ([`super::predicate_declarations`]).

use crate::profile::core;
use eidetic::pack::TermKind;

/// The owning graph stratum of a relation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GraphStratum {
    Resource,
    Surface,
}

/// The stratum the core pack places `iri` in, or `None` when the core pack
/// declares no such predicate.
pub fn built_in_predicate_stratum(iri: &str) -> Option<GraphStratum> {
    core::terms()
        .entry(iri)
        .filter(|entry| entry.kind == TermKind::Predicate)
        .map(|entry| entry.row.stratum)
}

/// The core pack's stratum for `iri`, or the core default (Resource).
pub fn default_predicate_stratum(iri: &str) -> GraphStratum {
    built_in_predicate_stratum(iri).unwrap_or(core::terms().defaults().stratum)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::TAGGED_WITH_IRI;
    use crate::model::placement_table::{BEYOND_THE_TABLE, PLACEMENT};
    use crate::profile::core::ids;

    /// The pack row's parity half (D27 (1)): the core pack with the workspace
    /// stand-in places every row as the frozen table does.
    #[test]
    fn all_recognized_kinds_match_the_settled_placement_table() {
        use crate::profile::tests::support::{pack_stratum, workspace_profile};
        let profile = workspace_profile();
        let mut counts = (0, 0);
        for (family, sub_kind, iri, expected) in PLACEMENT {
            match expected {
                GraphStratum::Resource => counts.0 += 1,
                GraphStratum::Surface => counts.1 += 1,
            }
            assert_eq!(
                pack_stratum(&profile, iri),
                expected,
                "{family}::{sub_kind} from the pack"
            );
            if family != "Arrangement" {
                assert_eq!(built_in_predicate_stratum(iri), Some(expected), "{iri}");
            }
        }
        assert_eq!(counts, (32, 12));
        for (iri, expected) in BEYOND_THE_TABLE {
            assert_eq!(pack_stratum(&profile, iri), expected, "{iri} from the pack");
        }
    }

    #[test]
    fn canonical_predicates_tags_and_open_defaults_have_distinct_controls() {
        assert_eq!(
            built_in_predicate_stratum(core::iri(ids::REL_USER_GROUPED)),
            Some(GraphStratum::Surface)
        );
        assert_eq!(
            built_in_predicate_stratum(core::iri(ids::REL_IMPORTED_SHARED_COLLECTION)),
            Some(GraphStratum::Resource)
        );
        assert_eq!(
            built_in_predicate_stratum(TAGGED_WITH_IRI),
            Some(GraphStratum::Resource)
        );
        // A family term is not a predicate.
        assert_eq!(
            built_in_predicate_stratum(core::iri(ids::FAMILY_SEMANTIC)),
            None
        );
        for unfamiliar in [
            "https://foreign.test/predicate",
            "",
            "arbitrary predicate",
            "https://mere.computer/ns/rel#User-grouped",
            "https://mere.computer/ns/rel#containment.UserFolder",
            "https://mere.computer/ns/rel#arrangement.frame-member",
        ] {
            assert_eq!(built_in_predicate_stratum(unfamiliar), None, "{unfamiliar}");
            assert_eq!(
                default_predicate_stratum(unfamiliar),
                GraphStratum::Resource
            );
        }
    }

    #[test]
    fn the_frozen_table_names_each_predicate_once() {
        let mut seen = std::collections::BTreeSet::new();
        for (_, _, iri, _) in PLACEMENT {
            assert!(seen.insert(iri), "duplicate predicate IRI {iri}");
        }
    }
}
