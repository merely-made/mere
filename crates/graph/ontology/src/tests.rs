// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use std::fs;

const LENS: &str = r#"id = "pack:test"
version = "0.1.0"
kind = ["lens"]
licence = "MPL-2.0"
[prefixes]
pack = "https://mere.computer/packs/"
[lens]
select = [{ pack = "pack:citation", version = "0.1.0" }]
"#;
const CITES: &str = "http://purl.org/spar/cito/cites";
const ANNOTATION: &str = "http://www.w3.org/ns/oa#Annotation";

#[test]
fn selection_controls_subjects_and_pins() {
    let bundled = [CITATION, WEB_ANNOTATION];
    let original = load(LENS, &bundled, None).unwrap();
    assert!(original.terms.entry(CITES).is_some());
    assert!(original.terms.entry(ANNOTATION).is_none());
    let dir = tempfile::tempdir().unwrap();
    fs::write(
        dir.path().join("lens.toml"),
        LENS.replace("pack:citation", "pack:web-annotation"),
    )
    .unwrap();
    let other = load(LENS, &bundled, Some(dir.path())).unwrap();
    assert!(other.terms.entry(CITES).is_none());
    assert!(other.terms.entry(ANNOTATION).is_some());
    assert_ne!(original.selection, other.selection);
}

#[test]
fn local_subject_replaces_a_bundled_subject_and_duplicates_are_refused() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("lens.toml"), LENS).unwrap();
    let replacement = CITATION.replace("cito:cites\"", "cito:credits\"");
    fs::write(dir.path().join("replacement.toml"), &replacement).unwrap();
    let selected = load(LENS, &[CITATION], Some(dir.path())).unwrap();
    assert!(selected.terms.entry(CITES).is_none());
    assert!(
        selected
            .terms
            .entry("http://purl.org/spar/cito/credits")
            .is_some()
    );
    fs::write(dir.path().join("ambiguous.toml"), replacement).unwrap();
    assert!(load(LENS, &[CITATION], Some(dir.path())).is_err());
}

#[test]
fn unavailable_selection_is_an_error() {
    assert!(load(LENS, &[], None).is_err());
    let dir = tempfile::tempdir().unwrap();
    assert!(load(LENS, &[CITATION], Some(dir.path())).is_err());
}

#[test]
fn additional_local_subject_extends_the_default_selection() {
    let dir = tempfile::tempdir().unwrap();
    let lens = LENS.replace("pack:citation\", version = \"0.1.0\" }",
        "pack:citation\", version = \"0.1.0\" }, { pack = \"pack:web-annotation\", version = \"0.1.0\" }");
    fs::write(dir.path().join("lens.toml"), lens).unwrap();
    fs::write(dir.path().join("notes.toml"), WEB_ANNOTATION).unwrap();
    let profile = load(LENS, &[CITATION], Some(dir.path())).unwrap();
    assert!(profile.terms.entry(CITES).is_some());
    assert!(profile.terms.entry(ANNOTATION).is_some());
}

#[test]
fn local_subject_cannot_shadow_the_engine_core() {
    let forged = CITATION.replace("pack:citation", "pack:core");
    assert!(load(LENS, &[&forged], None).is_err());
}

#[cfg(unix)]
#[test]
fn a_local_directory_can_select_a_linked_subject_file() {
    let packs = tempfile::tempdir().unwrap();
    let dir = tempfile::tempdir().unwrap();
    fs::write(packs.path().join("citation.toml"), CITATION).unwrap();
    fs::write(dir.path().join("lens.toml"), LENS).unwrap();
    std::os::unix::fs::symlink(
        packs.path().join("citation.toml"),
        dir.path().join("shared.toml"),
    )
    .unwrap();
    assert!(
        load(LENS, &[], Some(dir.path()))
            .unwrap()
            .terms
            .entry(CITES)
            .is_some()
    );
}

#[test]
fn one_statement_is_reachable_from_both_ends_under_its_original_predicate() {
    let selected = load(LENS, &[CITATION], None).unwrap();
    let links = RecordIndex::new(
        &selected,
        [
            ("urn:verse", CITES, "urn:take"),
            ("urn:verse", CITES, "urn:take"),
            ("urn:verse", "urn:unselected", "urn:missing"),
            ("urn:take", CITES, "urn:take"),
        ],
    );
    assert_eq!(
        links.records().collect::<Vec<_>>(),
        ["urn:take", "urn:verse"]
    );
    assert_eq!(links.links("urn:take").count(), 2);
    let forward = links.links("urn:verse").next().unwrap();
    let incoming = links
        .links("urn:take")
        .find(|s| s.subject == "urn:verse")
        .unwrap();
    assert!(std::ptr::eq(forward, incoming));
    assert_eq!(incoming.predicate, CITES);
    assert_eq!(incoming.object, "urn:take");
    let term = selected.terms.entry(CITES).unwrap();
    assert_eq!(term.display_label("EN-us"), "cites");
    let changed = CITATION.replace("en = \"cites\"", "en = \"cites\", fr = \"cite\"");
    let local = load(LENS, &[&changed], None).unwrap();
    assert_eq!(
        local.terms.entry(CITES).unwrap().display_label("fr-CA"),
        "cite"
    );
}
