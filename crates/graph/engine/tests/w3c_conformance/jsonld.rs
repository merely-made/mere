// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! JSON-LD 1.1 toRdf tests; the other API algorithms are recorded as not run.

use std::path::{Path, PathBuf};

use oxrdf::{Dataset, Quad};

use crate::manifest::rel;
use crate::rdf::{Fmt, isomorphic, layers, parse_rdf};
use crate::report::{Outcome, Report};

pub(crate) const JLD_BASE: &str = "https://w3c.github.io/json-ld-api/";

pub(crate) fn jsonld_loader(
    tests_dir: PathBuf,
) -> impl Fn(
    &str,
    &oxjsonld::JsonLdLoadDocumentOptions,
) -> Result<oxjsonld::JsonLdRemoteDocument, Box<dyn std::error::Error + Send + Sync>>
+ Send
+ Sync
+ std::panic::UnwindSafe
+ std::panic::RefUnwindSafe
+ 'static {
    move |url, _opts| {
        let rest = url
            .strip_prefix(&format!("{JLD_BASE}tests/"))
            .ok_or_else(|| format!("remote context outside the suite: {url}"))?;
        let document = std::fs::read(tests_dir.join(rest)).map_err(|e| format!("{url}: {e}"))?;
        Ok(oxjsonld::JsonLdRemoteDocument {
            document,
            document_url: url.to_string(),
        })
    }
}

pub(crate) fn run_jsonld(suite: &Path, report: &mut Report) -> Result<(), String> {
    let tests = suite.join("tests");
    let manifest_json = |name: &str| -> Result<serde_json::Value, String> {
        let bytes = std::fs::read(tests.join(name)).map_err(|e| e.to_string())?;
        serde_json::from_slice(&bytes).map_err(|e| e.to_string())
    };

    // Every JSON-LD document in the suite, served to the adapter layer as a ContextCache.
    let mut cache = mere::ContextCache::new();
    let mut stack = vec![tests.clone()];
    while let Some(dir) = stack.pop() {
        let Ok(rd) = std::fs::read_dir(&dir) else {
            continue;
        };
        for f in rd.flatten() {
            let p = f.path();
            if p.is_dir() {
                stack.push(p);
            } else if p.extension().is_some_and(|x| x == "jsonld")
                && let Ok(b) = std::fs::read(&p)
            {
                let rel_path = rel(&tests, &p);
                cache = cache.with(format!("{JLD_BASE}tests/{rel_path}"), b);
            }
        }
    }

    let doc = manifest_json("toRdf-manifest.jsonld")?;
    for t in doc["sequence"].as_array().ok_or("no sequence")? {
        let id = t["@id"].as_str().unwrap_or("?").to_string();
        let kinds: Vec<&str> = t["@type"]
            .as_array()
            .map(|a| a.iter().filter_map(|k| k.as_str()).collect())
            .unwrap_or_default();
        let negative = kinds.contains(&"jld:NegativeEvaluationTest");
        let syntax_only = kinds.contains(&"jld:PositiveSyntaxTest");
        let opt = &t["option"];
        let skip = if opt["specVersion"] == "json-ld-1.0" {
            Some("json-ld-1.0 only: outside the 1.1 profile")
        } else if opt.get("expandContext").is_some() {
            Some("expandContext option: no parser API")
        } else if opt.get("rdfDirection").is_some() {
            Some("rdfDirection option: parser is RDF 1.2 directional only")
        } else {
            None
        };
        let manifest = "tests/toRdf-manifest.jsonld";
        if let Some(reason) = skip {
            for layer in layers() {
                report.record(layer, manifest, "toRdf", &id, Outcome::Skip(reason));
            }
            continue;
        }
        let input = tests.join(t["input"].as_str().unwrap_or(""));
        let bytes = std::fs::read(&input).map_err(|e| format!("{}: {e}", input.display()))?;
        let rel_in = t["input"].as_str().unwrap_or("");
        let base = opt["base"]
            .as_str()
            .map(str::to_string)
            .unwrap_or_else(|| format!("{JLD_BASE}tests/{rel_in}"));
        let mut parser = oxjsonld::JsonLdParser::new();
        if opt["processingMode"] == "json-ld-1.0" {
            parser = parser.with_processing_mode(oxjsonld::JsonLdProcessingMode::JsonLd1_0);
        }
        let parser = match parser.with_base_iri(base.clone()) {
            Ok(p) => p,
            Err(e) => {
                report.record(
                    "parser",
                    manifest,
                    "toRdf",
                    &id,
                    Outcome::Fail(e.to_string()),
                );
                continue;
            },
        };
        let parsed: Result<Dataset, String> = parser
            .for_slice(&bytes)
            .with_load_document_callback(jsonld_loader(tests.clone()))
            .map(|q| q.map_err(|e| e.to_string()))
            .collect::<Result<Vec<Quad>, String>>()
            .map(|qs| qs.into_iter().collect());

        let out = if negative {
            match &parsed {
                Err(err) if err.contains("outside the suite") => {
                    Outcome::Skip("rejected only because a context lives outside the suite")
                },
                Err(_) => Outcome::Pass,
                Ok(_) => Outcome::Fail("accepted".into()),
            }
        } else {
            match (&parsed, syntax_only) {
                (Err(err), _) => Outcome::Fail(format!("rejected: {err}")),
                (Ok(_), true) => Outcome::Pass,
                (Ok(got), false) => {
                    let want = t["expect"].as_str().and_then(|p| {
                        let b = std::fs::read(tests.join(p)).ok()?;
                        parse_rdf(Fmt::NQuads, &b, "").ok()
                    });
                    match want {
                        Some(want) if isomorphic(got.clone(), want.clone()) => Outcome::Pass,
                        Some(_) => Outcome::Fail("not isomorphic to the expected dataset".into()),
                        None => Outcome::Skip("expected N-Quads unreadable"),
                    }
                },
            }
        };
        report.record("parser", manifest, "toRdf", &id, out);

        // Adapter layer: accept or reject through the offline-context entry point.
        if opt["processingMode"] == "json-ld-1.0" {
            report.record(
                "adapter",
                manifest,
                "toRdf",
                &id,
                Outcome::Skip("processingMode json-ld-1.0: the adapter always runs 1.1"),
            );
            report.record(
                "engine",
                manifest,
                "toRdf",
                &id,
                Outcome::Skip("processingMode json-ld-1.0: the engine always runs 1.1"),
            );
            continue;
        }
        let adapted =
            mere::from_jsonld_with_contexts_and_base_iri(&bytes, cache.clone(), Some(&base));
        let out = match (adapted, negative) {
            (Err(err), true) if err.to_string().contains("refused remote @context") => {
                Outcome::Skip("rejected only because a context lives outside the suite")
            },
            (Err(_), true) => Outcome::Pass,
            (Ok(_), true) => Outcome::Fail("accepted".into()),
            (Ok(_), false) => Outcome::Pass,
            (Err(err), false) => Outcome::Fail(format!("rejected: {err}")),
        };
        report.record("adapter", manifest, "toRdf", &id, out);

        // Engine layer: `mere::parse_jsonld` through the offline cache, scored by dataset.
        {
            let got = mere::parse_jsonld(&bytes, cache.clone(), Some(&base))
                .map(|d| crate::rdf::dataset_of(&d));
            let out = if negative {
                match &got {
                    Err(err) if err.to_string().contains("refused remote @context") => {
                        Outcome::Skip("rejected only because a context lives outside the suite")
                    },
                    Err(_) => Outcome::Pass,
                    Ok(_) => Outcome::Fail("accepted".into()),
                }
            } else {
                match (&got, syntax_only) {
                    (Err(err), _) => Outcome::Fail(format!("rejected: {err}")),
                    (Ok(_), true) => Outcome::Pass,
                    (Ok(got), false) => {
                        let want = t["expect"].as_str().and_then(|p| {
                            let b = std::fs::read(tests.join(p)).ok()?;
                            parse_rdf(Fmt::NQuads, &b, "").ok()
                        });
                        match want {
                            Some(want) if isomorphic(got.clone(), want.clone()) => Outcome::Pass,
                            Some(_) => {
                                Outcome::Fail("not isomorphic to the expected dataset".into())
                            },
                            None => Outcome::Skip("expected N-Quads unreadable"),
                        }
                    },
                }
            };
            report.record("engine", manifest, "toRdf", &id, out);
        }
    }

    // The other API algorithms: the adapter exposes no entry point for any of them.
    for name in [
        "expand",
        "compact",
        "flatten",
        "fromRdf",
        "html",
        "remote-doc",
    ] {
        let doc = manifest_json(&format!("{name}-manifest.jsonld"))?;
        let manifest = format!("tests/{name}-manifest.jsonld");
        for t in doc["sequence"].as_array().ok_or("no sequence")? {
            let id = t["@id"].as_str().unwrap_or("?");
            for layer in layers() {
                report.record(
                    layer,
                    &manifest,
                    name,
                    id,
                    Outcome::Skip("no expansion, compaction, flattening or framing API in the adapter or its parser"),
                );
            }
        }
    }
    Ok(())
}
