// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
use mere_ontology::{AdmittedSource, RecordIndex, SavedView, ViewQuery, WEB_ANNOTATION};
use std::{
    fs,
    path::Path,
    process::{Command, Output},
};

const LENS: &str = r#"id = "urn:test:lens"
version = "0.1.0"
kind = ["lens"]
licence = "MPL-2.0"
[lens]
select = [{pack = "https://mere.computer/packs/web-annotation", version = "0.1.0"}]
"#;
const CLASS: &str = "http://www.w3.org/ns/oa#Annotation";
const REL: &str = "http://www.w3.org/ns/oa#hasTarget";

#[test]
fn scope_and_lens_are_checked_before_a_resolver_can_observe_source_ids() {
    let profile = mere_ontology::load(LENS, &[WEB_ANNOTATION], None).unwrap();
    let query = ViewQuery::Outgoing {
        record: "urn:note".into(),
    };
    let view = SavedView::new(
        "Notes".into(),
        "A".into(),
        vec!["S".into()],
        query,
        &profile,
    )
    .unwrap();
    let other = mere_ontology::load(
        &LENS.replace("urn:test:lens", "urn:other"),
        &[WEB_ANNOTATION],
        None,
    )
    .unwrap();
    assert!(
        view.evaluate("B", &profile, |_| panic!("scope leaked"))
            .is_err()
    );
    assert!(
        view.evaluate("A", &other, |_| panic!("source leaked"))
            .is_err()
    );
    let stale = RecordIndex::new(&other, [("urn:note", REL, "urn:stale")]);
    assert!(
        view.evaluate("A", &profile, |_| Some(AdmittedSource {
            revision: "v1",
            index: &stale
        }))
        .is_err()
    );
    let mut forged = serde_json::from_str::<serde_json::Value>(&view.to_json().unwrap()).unwrap();
    forged["sources"] = serde_json::json!(["S", "S"]);
    assert!(SavedView::from_json(&forged.to_string()).is_err());
    let index = RecordIndex::new(
        &profile,
        [
            ("urn:note", REL, "urn:page"),
            ("urn:other", REL, "urn:note"),
        ],
    );
    let resolved = view
        .evaluate("A", &profile, |_| {
            Some(AdmittedSource {
                revision: "rev-2",
                index: &index,
            })
        })
        .unwrap();
    assert_eq!(resolved[0].statements.len(), 1);
    assert_eq!(resolved[0].statements[0].object, "urn:page");
    let class_view = SavedView::new(
        "Typed notes".into(),
        "A".into(),
        vec!["S".into()],
        ViewQuery::Type {
            class: CLASS.into(),
        },
        &profile,
    )
    .unwrap();
    let typed = RecordIndex::new(
        &profile,
        [(
            "urn:note",
            "http://www.w3.org/1999/02/22-rdf-syntax-ns#type",
            CLASS,
        )],
    );
    assert_eq!(
        class_view
            .evaluate("A", &profile, |_| Some(AdmittedSource {
                revision: "r",
                index: &typed
            }))
            .unwrap()[0]
            .statements[0]
            .object,
        CLASS
    );
}

fn command(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_mere-view"))
        .current_dir(dir)
        .args(args)
        .output()
        .unwrap()
}

#[test]
fn membership_is_visible_from_both_ends_of_the_statement() {
    let profile = mere_ontology::load(LENS, &[WEB_ANNOTATION], None).unwrap();
    let index = RecordIndex::new(
        &profile,
        [(
            "urn:note",
            "http://www.w3.org/1999/02/22-rdf-syntax-ns#type",
            CLASS,
        )],
    );
    for query in [
        ViewQuery::Outgoing {
            record: "urn:note".into(),
        },
        ViewQuery::Incoming {
            record: CLASS.into(),
        },
    ] {
        let view = SavedView::new(
            "Membership".into(),
            "A".into(),
            vec!["S".into()],
            query,
            &profile,
        )
        .unwrap();
        let rows = view
            .evaluate("A", &profile, |_| {
                Some(AdmittedSource {
                    revision: "r",
                    index: &index,
                })
            })
            .unwrap();
        assert_eq!(rows[0].statements, index.statements().collect::<Vec<_>>());
        assert_eq!(rows[0].statements.len(), 1);
        assert_eq!(rows[0].statements[0].subject, "urn:note");
        assert_eq!(rows[0].statements[0].object, CLASS);
    }
}

#[test]
fn offline_context_identity_tracks_content_and_urls_but_ignores_registration_order() {
    use mere::ContextCache;
    let cache = ContextCache::new().with("urn:a", b"1").with("urn:b", b"2");
    assert_eq!(
        cache.revision(),
        ContextCache::new()
            .with("urn:b", b"2")
            .with("urn:a", b"1")
            .revision()
    );
    assert_ne!(
        cache.revision(),
        cache.clone().with("urn:b", b"3").revision()
    );
    assert_ne!(
        cache.revision(),
        cache.clone().with("urn:c", b"2").revision()
    );
}

#[test]
fn independent_sources_keep_their_evidence_when_they_make_the_same_claim() {
    let profile = mere_ontology::load(LENS, &[WEB_ANNOTATION], None).unwrap();
    let view = SavedView::new(
        "Notes on a page".into(),
        "library".into(),
        vec!["field notes".into(), "draft".into()],
        ViewQuery::Incoming {
            record: "urn:page".into(),
        },
        &profile,
    )
    .unwrap();
    let index = RecordIndex::new(&profile, [("urn:note", REL, "urn:page")]);
    let open = |include_notes| {
        view.evaluate("library", &profile, |id| match id {
            "field notes" if include_notes => Some(AdmittedSource {
                revision: "notes-v1",
                index: &index,
            }),
            "draft" => Some(AdmittedSource {
                revision: "draft-v2",
                index: &index,
            }),
            _ => None,
        })
        .unwrap()
    };
    let both = open(true);
    assert_eq!(both[0].source, "field notes");
    assert_eq!(both[0].statements, both[1].statements);
    assert_eq!(both[0].statements.len(), 1);
    assert_ne!(both[0].revision, both[1].revision);
    let withdrawn = open(false);
    assert!(withdrawn[0].revision.is_none());
    assert!(withdrawn[0].statements.is_empty());
    assert_eq!(withdrawn[1], both[1]);
}

#[test]
fn reopening_recomputes_only_currently_disclosed_sources_and_revisions() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path();
    fs::create_dir(path.join("lens")).unwrap();
    fs::write(path.join("lens/lens.toml"), LENS).unwrap();
    let define = command(
        path,
        &[
            "define",
            "lens",
            "owner",
            "References",
            "incoming",
            "urn:page",
            "A",
            "B",
        ],
    );
    assert!(
        define.status.success(),
        "{}",
        String::from_utf8_lossy(&define.stderr)
    );
    fs::write(path.join("refs.json"), &define.stdout).unwrap();
    let run = |args: &[&str]| {
        let mut full = vec!["run", "lens", "owner", "refs.json"];
        full.extend_from_slice(args);
        let output = command(path, &full);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        serde_json::from_slice::<serde_json::Value>(&output.stdout).unwrap()
    };
    fs::write(path.join("a.ttl"), format!("<note> <{REL}> <urn:page> .")).unwrap();
    let a = ["A", "https://example.org/", "a.ttl"];
    let first = run(&a);
    assert_eq!(
        first[0]["statements"][0]["subject"],
        "https://example.org/note"
    );
    assert!(first[1]["revision"].is_null());
    let moved = run(&["A", "https://example.net/", "a.ttl"]);
    assert_ne!(moved[0]["revision"], first[0]["revision"]);
    assert_eq!(
        moved[0]["statements"][0]["subject"],
        "https://example.net/note"
    );
    fs::write(
        path.join("a.ttl"),
        format!("<note> <{REL}> <urn:elsewhere> ."),
    )
    .unwrap();
    fs::write(
        path.join("b.nq"),
        format!(
            "<urn:other> <{REL}> <urn:page> .\n<urn:private> <{REL}> <urn:page> <urn:foreign> ."
        ),
    )
    .unwrap();
    let both = run(&[&a[..], &["B", "urn:source:b", "b.nq"]].concat());
    assert_ne!(first[0]["revision"], both[0]["revision"]);
    assert_eq!(both[0]["statements"], serde_json::json!([]));
    assert_eq!(both[1]["statements"].as_array().unwrap().len(), 1);
    assert_eq!(both[1]["statements"][0]["subject"], "urn:other");
    let revoked = run(&a);
    assert!(revoked[1]["revision"].is_null());
    assert_eq!(revoked[1]["statements"], serde_json::json!([]));
    assert_eq!(fs::read(path.join("refs.json")).unwrap(), define.stdout);
    let wrong_scope = command(
        path,
        &[
            "run",
            "lens",
            "different",
            "refs.json",
            "unknown",
            "urn:base",
            "never-open.ttl",
        ],
    );
    assert!(!wrong_scope.status.success());
    assert!(String::from_utf8_lossy(&wrong_scope.stderr).contains("another owner scope"));
}
