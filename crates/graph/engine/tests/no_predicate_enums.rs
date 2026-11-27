// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Declared-lens D21: no code in the Mere workspace names a predicate enum;
//! relations are IRIs, matched through the core pack's generated ids. The
//! check reads every Rust source under `crates/`, and proves itself on a
//! planted match first.

use std::path::{Path, PathBuf};

/// The enums L7 removed, and their persisted mirrors.
const REMOVED: [&str; 20] = [
    "EdgeFamily",
    "SemanticSubKind",
    "ContainmentSubKind",
    "ImportedSubKind",
    "ProvenanceSubKind",
    "ArrangementSubKind",
    "RelationKind",
    "EdgeAssertion",
    "NavigationTrigger",
    "RelationDurability",
    "PersistedEdgeFamily",
    "PersistedSemanticSubKind",
    "PersistedContainmentSubKind",
    "PersistedArrangementSubKind",
    "PersistedImportedSubKind",
    "PersistedProvenanceSubKind",
    "PersistedEdgeAssertion",
    "PersistedRelationSelector",
    "PersistedNavigationTrigger",
    "PersistedTraversalRecord",
];

fn is_ident(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

/// The lines of `text` naming a removed enum as a whole identifier.
fn offenders(text: &str) -> Vec<(usize, String)> {
    let mut found = Vec::new();
    for (number, line) in text.lines().enumerate() {
        for name in REMOVED {
            let hit = line.match_indices(name).any(|(at, _)| {
                let before = line[..at].chars().next_back();
                let after = line[at + name.len()..].chars().next();
                !before.is_some_and(is_ident) && !after.is_some_and(is_ident)
            });
            if hit {
                found.push((number + 1, line.trim().to_owned()));
                break;
            }
        }
    }
    found
}

fn sources(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            if path.file_name().is_some_and(|n| n == "target") {
                continue;
            }
            sources(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

#[test]
fn the_check_finds_a_planted_enum_match() {
    let planted = "match kind {\n    RelationKind::Semantic(SemanticSubKind::Cites) => 1,\n}";
    assert_eq!(offenders(planted).len(), 1);
    // Negative controls: a longer identifier and a generated constant.
    assert!(offenders("let MyEdgeFamilyish = ids::FAMILY_SEMANTIC;").is_empty());
}

#[test]
fn no_mere_workspace_source_names_a_predicate_enum() {
    let crates = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut files = Vec::new();
    sources(&crates, &mut files);
    assert!(
        files.len() > 100,
        "the walk reached the workspace: {}",
        files.len()
    );
    let this = Path::new(file!()).file_name().unwrap();
    let mut found = Vec::new();
    for file in files.iter().filter(|f| f.file_name() != Some(this)) {
        let text = std::fs::read_to_string(file).unwrap();
        for (line, source) in offenders(&text) {
            found.push(format!("{}:{line}: {source}", file.display()));
        }
    }
    assert!(
        found.is_empty(),
        "predicate enums named:\n{}",
        found.join("\n")
    );
}
