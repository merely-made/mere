// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Declared-lens D29, D33: the labels users see are pack data, inverse and
//! short labels included. The core rows reproduce the text of Graphshell's
//! tables at f61ae5c9: `incipit/src/navigation.rs` relationship_label for
//! labels and inverses, `incipit/src/roster.rs` relation_kind_label and
//! RELATE_PICKER_KINDS for short labels, edge_family_label for families.

use super::support::{
    Source, TRAVERSAL_IRI, WORKSPACE, WORKSPACE_LENS, compiled, edit, load_with, local,
    workspace_lens, workspace_profile,
};
use crate::profile::core::{self, ids};
use crate::profile::intern::TermId;

/// The old UI's text: label, inverse, short label (D29, D33), for every core
/// row but `taggedWith`, which no table named; next-step's labels are verbs
/// (D42) and every predicate's short label is its own name (D43).
const OLD_UI: [(TermId, &str, Option<&str>, &str); 45] = [
    (ids::FAMILY_SEMANTIC, "Semantic", None, "Semantic"),
    (ids::FAMILY_CONTAINMENT, "Containment", None, "Containment"),
    (ids::FAMILY_IMPORTED, "Imported", None, "Imported"),
    (ids::FAMILY_PROVENANCE, "Provenance", None, "Provenance"),
    (
        ids::REL_HYPERLINK,
        "Links to",
        Some("Linked from"),
        "Hyperlink",
    ),
    (
        ids::REL_USER_GROUPED,
        "Grouped with",
        Some("Grouped with"),
        "Grouped",
    ),
    (
        ids::REL_AGENT_DERIVED,
        "Derived relation to",
        Some("Derived relation from"),
        "Agent",
    ),
    (ids::REL_CITES, "Cites", Some("Cited by"), "Cites"),
    (ids::REL_QUOTES, "Quotes", Some("Quoted by"), "Quotes"),
    (
        ids::REL_SUMMARIZES,
        "Summarizes",
        Some("Summarized by"),
        "Summarizes",
    ),
    (
        ids::REL_ELABORATES,
        "Elaborates on",
        Some("Elaborated by"),
        "Elaborates",
    ),
    (
        ids::REL_EXAMPLE_OF,
        "Example of",
        Some("Has example"),
        "Example",
    ),
    (
        ids::REL_SUPPORTS,
        "Supports",
        Some("Supported by"),
        "Supports",
    ),
    (
        ids::REL_CONTRADICTS,
        "Contradicts",
        Some("Contradicted by"),
        "Contradicts",
    ),
    (
        ids::REL_QUESTIONS,
        "Questions",
        Some("Questioned by"),
        "Questions",
    ),
    (
        ids::REL_SAME_ENTITY_AS,
        "Same entity as",
        Some("Same entity as"),
        "Same As",
    ),
    (
        ids::REL_DUPLICATE_OF,
        "Duplicate of",
        Some("Has duplicate"),
        "Duplicate",
    ),
    (
        ids::REL_CANONICAL_MIRROR_OF,
        "Canonical mirror of",
        Some("Has canonical mirror"),
        "Mirror",
    ),
    (
        ids::REL_DEPENDS_ON,
        "Depends on",
        Some("Required by"),
        "Depends",
    ),
    (ids::REL_BLOCKS, "Blocks", Some("Blocked by"), "Blocks"),
    (ids::REL_NEXT_STEP, "Leads to", Some("Follows from"), "Next"),
    (
        ids::REL_CONTAINMENT_URL_PATH,
        "Contains",
        Some("Contained by"),
        "URL path",
    ),
    (
        ids::REL_CONTAINMENT_DOMAIN,
        "Contains",
        Some("Contained by"),
        "Domain",
    ),
    (
        ids::REL_CONTAINMENT_FILESYSTEM,
        "Contains",
        Some("Contained by"),
        "File system",
    ),
    (
        ids::REL_CONTAINMENT_USER_FOLDER,
        "Contains",
        Some("Contained by"),
        "User folder",
    ),
    (
        ids::REL_CONTAINMENT_CLIP_SOURCE,
        "Contains",
        Some("Contained by"),
        "Clip source",
    ),
    (
        ids::REL_CONTAINMENT_NOTEBOOK_SECTION,
        "Contains",
        Some("Contained by"),
        "Notebook section",
    ),
    (
        ids::REL_CONTAINMENT_COLLECTION_MEMBER,
        "Contains",
        Some("Contained by"),
        "Collection member",
    ),
    (
        ids::REL_IMPORTED_BOOKMARK_FOLDER,
        "Import relation to",
        Some("Import relation from"),
        "Bookmark folder",
    ),
    (
        ids::REL_IMPORTED_HISTORY_IMPORT,
        "Import relation to",
        Some("Import relation from"),
        "History import",
    ),
    (
        ids::REL_IMPORTED_SESSION_IMPORT,
        "Import relation to",
        Some("Import relation from"),
        "Session import",
    ),
    (
        ids::REL_IMPORTED_RSS_MEMBERSHIP,
        "Import relation to",
        Some("Import relation from"),
        "RSS membership",
    ),
    (
        ids::REL_IMPORTED_FILESYSTEM_IMPORT,
        "Import relation to",
        Some("Import relation from"),
        "File system import",
    ),
    (
        ids::REL_IMPORTED_ARCHIVE_MEMBERSHIP,
        "Import relation to",
        Some("Import relation from"),
        "Archive membership",
    ),
    (
        ids::REL_IMPORTED_SHARED_COLLECTION,
        "Import relation to",
        Some("Import relation from"),
        "Shared collection",
    ),
    (
        ids::REL_PROVENANCE_CLIPPED_FROM,
        "Derived from",
        Some("Source for"),
        "Clipped from",
    ),
    (
        ids::REL_PROVENANCE_EXCERPTED_FROM,
        "Derived from",
        Some("Source for"),
        "Excerpted from",
    ),
    (
        ids::REL_PROVENANCE_SUMMARIZED_FROM,
        "Derived from",
        Some("Source for"),
        "Summarized from",
    ),
    (
        ids::REL_PROVENANCE_TRANSLATED_FROM,
        "Derived from",
        Some("Source for"),
        "Translated from",
    ),
    (
        ids::REL_PROVENANCE_REWRITTEN_FROM,
        "Derived from",
        Some("Source for"),
        "Rewritten from",
    ),
    (
        ids::REL_PROVENANCE_GENERATED_FROM,
        "Derived from",
        Some("Source for"),
        "Generated from",
    ),
    (
        ids::REL_PROVENANCE_EXTRACTED_FROM,
        "Derived from",
        Some("Source for"),
        "Extracted from",
    ),
    (
        ids::REL_PROVENANCE_IMPORTED_FROM_SOURCE,
        "Derived from",
        Some("Source for"),
        "Imported from source",
    ),
    (
        ids::REL_PROVENANCE_COPIED_FROM,
        "Derived from",
        Some("Source for"),
        "Copied from",
    ),
    (
        ids::REL_PROVENANCE_MERGED_FROM,
        "Derived from",
        Some("Source for"),
        "Merged from",
    ),
];

#[test]
fn the_core_labels_are_the_old_ui_text() {
    let terms = core::terms();
    for (id, label, inverse, short) in OLD_UI {
        let entry = terms.get(id).unwrap();
        assert_eq!(entry.display_label("en"), label, "{}", entry.iri);
        assert_eq!(entry.display_inverse_label("en"), inverse, "{}", entry.iri);
        assert_eq!(entry.display_short_label("en"), short, "{}", entry.iri);
    }
    // Every core term but taggedWith has its old text.
    assert_eq!(OLD_UI.len() + 1, ids::TERMS.len());
    let tagged = terms.get(ids::REL_TAGGEDWITH).unwrap();
    assert!(tagged.short_labels.is_empty());
    assert_eq!(tagged.display_short_label("en"), tagged.display_label("en"));
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

/// D43: no two predicates share a short label, so navigation's qualifier
/// names each.
#[test]
fn every_core_predicate_has_its_own_short_label() {
    let terms = core::terms();
    let mut seen = std::collections::BTreeMap::new();
    for (id, entry) in terms.iter() {
        if entry.kind != eidetic::pack::TermKind::Predicate || id == ids::REL_TAGGEDWITH {
            continue;
        }
        let short = entry.display_short_label("en");
        if let Some(other) = seen.insert(short.to_owned(), entry.iri.clone()) {
            panic!("{} and {other} share {short:?}", entry.iri);
        }
    }
    assert_eq!(seen.len(), 41);
    // The qualifier is the short label lowercased: today's text.
    let fs = terms.get(ids::REL_IMPORTED_FILESYSTEM_IMPORT).unwrap();
    assert_eq!(
        fs.display_short_label("en").to_lowercase(),
        "file system import"
    );
}

/// D39: an IRI no pack declares, and a predicate that declares no labels,
/// read the core pack's label defaults; a row with labels keeps its own.
#[test]
fn undeclared_and_label_less_predicates_take_the_core_defaults() {
    let terms = core::terms();
    let open = "urn:mere:test:open";
    assert!(terms.entry(open).is_none());
    assert_eq!(terms.display_label(open, "en"), "Relation to");
    assert_eq!(
        terms.display_inverse_label(open, "fr"),
        Some("Relation from")
    );
    assert_eq!(terms.display_short_label(open, "en-GB"), "Predicate");
    let cites = core::iri(ids::REL_CITES);
    assert_eq!(terms.display_label(cites, "en"), "Cites");
    // A family declares no inverse and keeps none (D44 (6)).
    let family = core::iri(ids::FAMILY_SEMANTIC);
    assert_eq!(terms.display_inverse_label(family, "en"), None);
    let tagged = core::iri(ids::REL_TAGGEDWITH);
    assert_eq!(terms.display_inverse_label(tagged, "en"), None);
    assert_eq!(terms.display_short_label(tagged, "en"), "tagged with");

    // A stand-in pack row with no labels takes them too.
    let text = edit(
        WORKSPACE,
        "labels = { en = \"Went to\" }\ninverse-labels = { en = \"Reached from\" }\nshort-labels = { en = \"Traversal\" }\n",
        "labels = {}\n",
    );
    let source = Source::of(vec![
        local(compiled(&text)),
        local(compiled(WORKSPACE_LENS)),
    ]);
    let p = load_with(&workspace_lens(), &source).unwrap();
    let traversal = p.terms.entry(TRAVERSAL_IRI).unwrap();
    assert_eq!(traversal.display_label("en"), "Relation to");
    assert_eq!(traversal.display_inverse_label("en"), Some("Relation from"));
    assert_eq!(traversal.display_short_label("en"), "Predicate");
}
