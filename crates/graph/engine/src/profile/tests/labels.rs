// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Declared-lens D29: the labels users see are pack data, inverse labels
//! included. The core rows reproduce the text of Graphshell's tables at
//! f61ae5c9 (`incipit/src/navigation.rs` relationship_label for the semantic
//! predicates, `incipit/src/roster.rs` edge_family_label for the families).

use super::support::{
    Source, TRAVERSAL_IRI, WORKSPACE, WORKSPACE_LENS, compiled, edit, load_with, local,
    workspace_lens, workspace_profile,
};
use crate::profile::core::{self, ids};
use crate::profile::intern::TermId;

/// The old UI's text: forward, then inverse.
const OLD_UI: [(TermId, &str, Option<&str>); 21] = [
    (ids::FAMILY_SEMANTIC, "Semantic", None),
    (ids::FAMILY_CONTAINMENT, "Containment", None),
    (ids::FAMILY_IMPORTED, "Imported", None),
    (ids::FAMILY_PROVENANCE, "Provenance", None),
    (ids::REL_HYPERLINK, "Links to", Some("Linked from")),
    (ids::REL_CITES, "Cites", Some("Cited by")),
    (ids::REL_QUOTES, "Quotes", Some("Quoted by")),
    (ids::REL_SUMMARIZES, "Summarizes", Some("Summarized by")),
    (ids::REL_ELABORATES, "Elaborates on", Some("Elaborated by")),
    (ids::REL_EXAMPLE_OF, "Example of", Some("Has example")),
    (ids::REL_SUPPORTS, "Supports", Some("Supported by")),
    (ids::REL_CONTRADICTS, "Contradicts", Some("Contradicted by")),
    (ids::REL_QUESTIONS, "Questions", Some("Questioned by")),
    (
        ids::REL_SAME_ENTITY_AS,
        "Same entity as",
        Some("Same entity as"),
    ),
    (ids::REL_DUPLICATE_OF, "Duplicate of", Some("Has duplicate")),
    (
        ids::REL_CANONICAL_MIRROR_OF,
        "Canonical mirror of",
        Some("Has canonical mirror"),
    ),
    (ids::REL_DEPENDS_ON, "Depends on", Some("Required by")),
    (ids::REL_BLOCKS, "Blocks", Some("Blocked by")),
    (ids::REL_NEXT_STEP, "Next step", Some("Previous step")),
    (ids::REL_USER_GROUPED, "Grouped with", Some("Grouped with")),
    (
        ids::REL_AGENT_DERIVED,
        "Derived relation to",
        Some("Derived relation from"),
    ),
];

#[test]
fn the_core_labels_are_the_old_ui_text() {
    let terms = core::terms();
    for (id, label, inverse) in OLD_UI {
        let entry = terms.get(id).unwrap();
        assert_eq!(entry.display_label("en"), label, "{}", entry.iri);
        assert_eq!(entry.display_inverse_label("en"), inverse, "{}", entry.iri);
    }
    // Every core semantic predicate has its old text.
    let semantic = ids::TERMS
        .into_iter()
        .filter(|(_, id)| terms.family_of(*id) == Some(ids::FAMILY_SEMANTIC))
        .count();
    assert_eq!(semantic, 17);
    assert_eq!(OLD_UI.len(), 4 + semantic);
}

#[test]
fn the_workspace_stand_in_carries_traversal_s_old_text() {
    let p = workspace_profile();
    let traversal = p.terms.entry(TRAVERSAL_IRI).unwrap();
    assert_eq!(traversal.display_label("en"), "Went to");
    assert_eq!(
        traversal.display_inverse_label("en-GB"),
        Some("Reached from")
    );
    // No French label: English is the fallback, as for the forward label.
    assert_eq!(traversal.display_inverse_label("fr"), Some("Reached from"));
    let family = p
        .terms
        .entry("https://mere.computer/ns/family#traversal")
        .unwrap();
    assert_eq!(family.display_label("en"), "Traversal");
    assert_eq!(
        family.display_inverse_label("en"),
        None,
        "no inverse declared"
    );
}

/// Break and restore: the text users see moves with the pack row.
#[test]
fn changing_a_label_changes_the_display_text() {
    let text = edit(
        WORKSPACE,
        "labels = { en = \"Went to\" }\ninverse-labels = { en = \"Reached from\" }",
        "labels = { en = \"Visited\" }\ninverse-labels = { en = \"Reached from\", fr = \"Atteint depuis\" }",
    );
    let source = Source::of(vec![
        local(compiled(&text)),
        local(compiled(WORKSPACE_LENS)),
    ]);
    let p = load_with(&workspace_lens(), &source).unwrap();
    let traversal = p.terms.entry(TRAVERSAL_IRI).unwrap();
    assert_eq!(traversal.display_label("en"), "Visited");
    assert_eq!(
        traversal.display_inverse_label("fr-CA"),
        Some("Atteint depuis")
    );
    assert_eq!(traversal.display_inverse_label("en"), Some("Reached from"));
}
