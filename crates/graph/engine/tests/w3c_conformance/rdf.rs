// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! RDF 1.1 and 1.2 syntax and evaluation tests (N-Triples, N-Quads, Turtle, TriG, RDF/XML).

use std::path::Path;

use oxrdf::dataset::CanonicalizationAlgorithm;
use oxrdf::{Dataset, GraphName};
use oxttl::{NQuadsParser, NTriplesParser, TriGParser, TurtleParser};

use crate::manifest::{Entry, local, rel, test_base};
use crate::report::{Outcome, Report};

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Fmt {
    NTriples,
    NQuads,
    Turtle,
    TriG,
    Xml,
}

pub(crate) fn fmt_of(kind: &str) -> Option<Fmt> {
    let rest = kind.strip_prefix("Test")?;
    [
        ("NTriples", Fmt::NTriples),
        ("NQuads", Fmt::NQuads),
        ("Turtle", Fmt::Turtle),
        ("Trig", Fmt::TriG),
        ("XML", Fmt::Xml),
    ]
    .into_iter()
    .find(|(p, _)| rest.starts_with(p))
    .map(|(_, f)| f)
}

pub(crate) fn parse_rdf(fmt: Fmt, bytes: &[u8], base: &str) -> Result<Dataset, String> {
    let mut ds = Dataset::new();
    match fmt {
        Fmt::NTriples => {
            for t in NTriplesParser::new().for_slice(bytes) {
                ds.insert(
                    &t.map_err(|e| e.to_string())?
                        .in_graph(GraphName::DefaultGraph),
                );
            }
        },
        Fmt::NQuads => {
            for q in NQuadsParser::new().for_slice(bytes) {
                ds.insert(&q.map_err(|e| e.to_string())?);
            }
        },
        Fmt::Turtle => {
            let p = TurtleParser::new()
                .with_base_iri(base)
                .map_err(|e| e.to_string())?;
            for t in p.for_slice(bytes) {
                ds.insert(
                    &t.map_err(|e| e.to_string())?
                        .in_graph(GraphName::DefaultGraph),
                );
            }
        },
        Fmt::TriG => {
            let p = TriGParser::new()
                .with_base_iri(base)
                .map_err(|e| e.to_string())?;
            for q in p.for_slice(bytes) {
                ds.insert(&q.map_err(|e| e.to_string())?);
            }
        },
        Fmt::Xml => {
            let p = oxrdfxml::RdfXmlParser::new()
                .with_base_iri(base)
                .map_err(|e| e.to_string())?;
            for t in p.for_slice(bytes) {
                ds.insert(
                    &t.map_err(|e| e.to_string())?
                        .in_graph(GraphName::DefaultGraph),
                );
            }
        },
    }
    Ok(ds)
}

pub(crate) fn isomorphic(mut a: Dataset, mut b: Dataset) -> bool {
    a.canonicalize(CanonicalizationAlgorithm::Unstable);
    b.canonicalize(CanonicalizationAlgorithm::Unstable);
    a == b
}

/// The adapter's own entry point for `fmt`, accept or reject only. `None`: it has none.
pub(crate) fn adapter_accepts(fmt: Fmt, bytes: &[u8]) -> Option<Result<(), String>> {
    let text = match std::str::from_utf8(bytes) {
        Ok(t) => t,
        Err(e) => return Some(Err(e.to_string())),
    };
    match fmt {
        Fmt::NQuads => Some(
            mere::from_nquads(text, "urn:w3c")
                .map(|_| ())
                .map_err(|e| e.to_string()),
        ),
        Fmt::TriG => Some(
            mere::from_trig(text, "urn:w3c")
                .map(|_| ())
                .map_err(|e| e.to_string()),
        ),
        _ => None,
    }
}

pub(crate) fn run_rdf(root: &Path, entries: &[Entry], report: &mut Report) {
    for e in entries {
        let manifest = rel(
            root,
            &e.manifest.parent().unwrap_or(&e.manifest).to_path_buf(),
        );
        let kind = e.kinds.first().map(String::as_str).unwrap_or("");
        let id = local(&e.id);
        let (category, positive, eval) = if let Some(k) = kind.strip_suffix("PositiveSyntax") {
            let _ = k;
            ("syntax", true, false)
        } else if kind.ends_with("NegativeSyntax") {
            ("syntax", false, false)
        } else if kind.ends_with("NegativeEval") {
            ("eval", false, true)
        } else if kind.ends_with("Eval") {
            ("eval", true, true)
        } else if kind.ends_with("C14N") {
            ("c14n", true, false)
        } else if kind.contains("Entailment") {
            ("entailment", true, false)
        } else {
            ("other", true, false)
        };
        let fmt = fmt_of(kind);
        let skip_both = if e.rejected {
            Some("approval Rejected in the manifest")
        } else if category == "c14n" {
            Some("canonical N-Triples/N-Quads output is not exposed")
        } else if category == "entailment" {
            Some("no entailment stage yet")
        } else if category == "other" || fmt.is_none() || e.action.is_none() {
            Some("unrecognised test type")
        } else {
            None
        };
        if let Some(reason) = skip_both {
            for layer in ["parser", "adapter"] {
                report.record(layer, &manifest, category, &id, Outcome::Skip(reason));
            }
            continue;
        }
        let (fmt, action) = (fmt.unwrap(), e.action.as_ref().unwrap());
        let bytes = match std::fs::read(action) {
            Ok(b) => b,
            Err(err) => {
                for layer in ["parser", "adapter"] {
                    report.record(
                        layer,
                        &manifest,
                        category,
                        &id,
                        Outcome::Fail(err.to_string()),
                    );
                }
                continue;
            },
        };
        let base = test_base(root, action);

        // Parser layer.
        let parsed = parse_rdf(fmt, &bytes, &base);
        let out = if !eval {
            match (&parsed, positive) {
                (Ok(_), true) | (Err(_), false) => Outcome::Pass,
                (Err(err), true) => Outcome::Fail(format!("rejected: {err}")),
                (Ok(_), false) => Outcome::Fail("accepted".into()),
            }
        } else if !positive {
            match &parsed {
                Err(_) => Outcome::Pass,
                Ok(_) => Outcome::Fail("accepted".into()),
            }
        } else {
            match (&parsed, e.result.as_ref()) {
                (Err(err), _) => Outcome::Fail(format!("rejected: {err}")),
                (Ok(_), None) => Outcome::Skip("no expected result"),
                (Ok(got), Some(res)) => {
                    let want_fmt = if res.extension().is_some_and(|x| x == "nq") {
                        Fmt::NQuads
                    } else {
                        Fmt::NTriples
                    };
                    match std::fs::read(res)
                        .map_err(|e| e.to_string())
                        .and_then(|b| parse_rdf(want_fmt, &b, &test_base(root, res)))
                    {
                        Ok(want) if isomorphic(got.clone(), want.clone()) => Outcome::Pass,
                        Ok(_) => Outcome::Fail("not isomorphic to the expected dataset".into()),
                        Err(err) => Outcome::Skip(Box::leak(
                            format!("expected result unparseable: {err}").into_boxed_str(),
                        )),
                    }
                },
            }
        };
        report.record("parser", &manifest, category, &id, out);

        // Adapter layer: syntax tests through the crate's public entry points.
        let out = if eval {
            Outcome::Skip("adapter yields a graph contribution, not an RDF dataset")
        } else {
            match adapter_accepts(fmt, &bytes) {
                None => Outcome::Skip("no adapter entry point for this format"),
                Some(r) => match (r, positive) {
                    (Ok(()), true) | (Err(_), false) => Outcome::Pass,
                    (Err(err), true) => Outcome::Fail(format!("rejected: {err}")),
                    (Ok(()), false) => Outcome::Fail("accepted".into()),
                },
            }
        };
        report.record("adapter", &manifest, category, &id, out);
    }
}
