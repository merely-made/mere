// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The `rdf-canon` suite: RDFC-1.0 eval (canonical N-Quads), map (issued identifiers) and negative
//! (poison graph) tests, at two layers.
//!
//! - `library`: `oxrdf`'s `rdfc-10` called directly (no limit, no refusal): the control.
//! - `engine`: `mere::canonicalize_quads` (the carried `mere-rdfc`) under its default work budget.
//!
//! The negative test is the work limit's instrument. Both layers run it under a time box on a
//! thread, so a missing refusal is a recorded failure and not a hang. The harness prints the
//! largest call and permutation counts any positive test used (`RDFC` line) and the time the
//! engine took to refuse the poison graph.

use std::collections::BTreeMap;
use std::path::Path;
use std::time::Duration;

use oxrdf::dataset::{CanonicalizationAlgorithm, CanonicalizationHashAlgorithm};
use oxrdf::{Dataset, Quad};
use oxttl::NQuadsParser;

use crate::manifest::{Entry, Index, local, rel};
use crate::report::{Outcome, Report};

const RDFC: &str = "https://w3c.github.io/rdf-canon/tests/vocab#";
/// How long the library control waits for a poison graph before calling it unbounded.
const POISON_BOX: Duration = Duration::from_secs(10);

fn read_quads(path: &Path) -> Result<Vec<Quad>, String> {
    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    NQuadsParser::new()
        .for_slice(&bytes)
        .map(|q| q.map_err(|e| e.to_string()))
        .collect()
}

/// `oxrdf` direct: canonical N-Quads and the issued map, no limit.
fn library(quads: &[Quad], sha384: bool) -> (String, BTreeMap<String, String>) {
    let dataset: Dataset = quads.iter().cloned().collect();
    let hash_algorithm = if sha384 {
        CanonicalizationHashAlgorithm::Sha384
    } else {
        CanonicalizationHashAlgorithm::Sha256
    };
    let algorithm = CanonicalizationAlgorithm::Rdfc10 { hash_algorithm };
    let issued = dataset
        .canonicalize_blank_nodes(algorithm)
        .iter()
        .map(|(a, b)| (a.as_str().to_owned(), b.as_str().to_owned()))
        .collect();
    let mut canonical = dataset.clone();
    canonical.canonicalize(algorithm);
    let mut lines: Vec<String> = canonical.iter().map(|q| format!("{q} .\n")).collect();
    lines.sort_unstable();
    (lines.concat(), issued)
}

fn engine(quads: &[Quad], sha384: bool) -> Result<mere::Canonical, mere::CanonError> {
    let options = mere::CanonOptions {
        hash: if sha384 {
            mere::CanonHash::Sha384
        } else {
            mere::CanonHash::Sha256
        },
        ..Default::default()
    };
    mere::canonicalize_quads(quads, &options)
}

fn score<T>(got: Result<T, String>, ok: impl FnOnce(&T) -> bool, why: &str) -> Outcome {
    match got {
        Ok(v) if ok(&v) => Outcome::Pass,
        Ok(_) => Outcome::Fail(why.into()),
        Err(err) => Outcome::Fail(format!("refused: {err}")),
    }
}

pub(crate) fn run_rdfc(root: &Path, entries: &[Entry], report: &mut Report) -> Result<(), String> {
    let (mut max_calls, mut max_perms, mut refusal) = (0usize, 0usize, None::<std::time::Duration>);
    let manifest_path = root.join("tests/manifest.ttl");
    let index = Index::load(&manifest_path)?;
    let manifest = rel(root, manifest_path.parent().unwrap_or(root));
    for e in entries {
        let id = local(&e.id);
        let kind = e.kinds.first().map(String::as_str).unwrap_or("");
        let category = match kind {
            "RDFC10EvalTest" => "eval",
            "RDFC10MapTest" => "map",
            "RDFC10NegativeEvalTest" => "negative",
            _ => "other",
        };
        let both = |report: &mut Report, out: Outcome| {
            for layer in ["library", "engine"] {
                report.record(layer, &manifest, category, &id, out.clone());
            }
        };
        let Some(action) = e.action.as_ref().filter(|_| category != "other") else {
            both(report, Outcome::Skip("unrecognised test type"));
            continue;
        };
        let sha384 = index
            .objects(&e.id, &format!("{RDFC}hashAlgorithm"))
            .iter()
            .any(|t| matches!(t, oxrdf::Term::Literal(l) if l.value() == "SHA384"));
        let quads = match read_quads(action) {
            Ok(q) => q,
            Err(err) => {
                both(report, Outcome::Fail(format!("input: {err}")));
                continue;
            },
        };
        let want = e
            .result
            .as_ref()
            // The suite's git index is LF; a Windows checkout may rewrite it to CRLF. A raw CR
            // cannot occur inside a canonical line (the serializer escapes it), so this is safe.
            .map(|r| {
                std::fs::read_to_string(r)
                    .map(|t| t.replace("\r\n", "\n"))
                    .map_err(|e| e.to_string())
            });
        let (lib, eng) = match (category, want) {
            ("negative", _) => {
                // Pass means refused. Each layer runs under a time box on a thread: the poison
                // graph does not return, and the limit has to refuse before any work starts.
                let boxed = |run: fn(&[Quad], bool) -> Result<(), String>| {
                    let owned = quads.clone();
                    let (tx, rx) = std::sync::mpsc::channel();
                    std::thread::spawn(move || {
                        let _ = tx.send(run(&owned, sha384));
                    });
                    match rx.recv_timeout(POISON_BOX) {
                        Ok(Err(_)) => Outcome::Pass,
                        Ok(Ok(())) => Outcome::Fail("completed: not refused".into()),
                        Err(_) => Outcome::Fail(format!(
                            "not refused: still running after {}s",
                            POISON_BOX.as_secs()
                        )),
                    }
                };
                let lib = boxed(|q, h| {
                    library(q, h);
                    Ok(())
                });
                let started = std::time::Instant::now();
                let eng = boxed(|q, h| engine(q, h).map(|_| ()).map_err(|e| e.to_string()));
                if matches!(eng, Outcome::Pass) {
                    refusal = Some(started.elapsed());
                }
                (lib, eng)
            },
            ("eval", Some(Ok(want))) => (
                score(
                    Ok(library(&quads, sha384).0),
                    |g| *g == want,
                    "canonical N-Quads differ",
                ),
                score(
                    engine(&quads, sha384).map_err(|e| e.to_string()),
                    |c| {
                        max_calls = max_calls.max(c.hndq_calls);
                        max_perms = max_perms.max(c.permutations);
                        c.nquads == want
                    },
                    "canonical N-Quads differ",
                ),
            ),
            ("map", Some(Ok(want))) => {
                let want: BTreeMap<String, String> = match serde_json::from_str(&want) {
                    Ok(m) => m,
                    Err(err) => {
                        both(report, Outcome::Fail(format!("map: {err}")));
                        continue;
                    },
                };
                (
                    score(
                        Ok(library(&quads, sha384).1),
                        |g| *g == want,
                        "issued identifiers differ",
                    ),
                    score(
                        engine(&quads, sha384).map_err(|e| e.to_string()),
                        |c| c.issued == want,
                        "issued identifiers differ",
                    ),
                )
            },
            _ => {
                both(
                    report,
                    Outcome::Fail("expected result missing or unreadable".into()),
                );
                continue;
            },
        };
        report.record("library", &manifest, category, &id, lib);
        report.record("engine", &manifest, category, &id, eng);
    }
    println!(
        "RDFC	max_hndq_calls	{max_calls}	max_permutations	{max_perms}	budget	{}	{}	poison_refusal_ms	{}",
        mere::DEFAULT_MAX_HNDQ_CALLS,
        mere::DEFAULT_MAX_PERMUTATIONS,
        refusal.map_or_else(|| "none".to_owned(), |d| d.as_millis().to_string()),
    );
    Ok(())
}
