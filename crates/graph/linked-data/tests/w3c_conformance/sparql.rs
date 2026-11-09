// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! SPARQL syntax and query evaluation tests; update evaluation is recorded as not run.

use std::path::Path;

#[cfg(feature = "query")]
use crate::rdf::{Fmt, isomorphic, parse_rdf};

use crate::manifest::{Entry, local, rel, test_base};
use crate::report::{Outcome, Report};

#[cfg(feature = "query")]
pub(crate) fn run_sparql(root: &Path, entries: &[Entry], report: &mut Report) {
    use spargebra::SparqlParser;
    for e in entries {
        let manifest = rel(root, e.manifest.parent().unwrap_or(&e.manifest));
        let kind = e.kinds.first().map(String::as_str).unwrap_or("");
        let id = local(&e.id);
        let positive = kind.starts_with("Positive");
        let update = kind.contains("Update");
        let syntax = (kind.starts_with("Positive") || kind.starts_with("Negative"))
            && kind.contains("Syntax");
        let (category, parser_out) = if syntax {
            let out = match e.action.as_ref().map(std::fs::read) {
                Some(Ok(bytes)) => {
                    let text = String::from_utf8_lossy(&bytes).into_owned();
                    // The suite's relative IRIs resolve against the test's own address.
                    let base = test_base(root, e.action.as_ref().unwrap());
                    let parser = SparqlParser::new().with_base_iri(base).unwrap();
                    let r = if update {
                        parser.parse_update(&text).map(|_| ())
                    } else {
                        parser.parse_query(&text).map(|_| ())
                    };
                    match (r, positive) {
                        (Ok(()), true) | (Err(_), false) => Outcome::Pass,
                        (Err(err), true) => Outcome::Fail(format!("rejected: {err}")),
                        (Ok(()), false) => Outcome::Fail("accepted".into()),
                    }
                },
                _ => Outcome::Fail("action file unreadable".into()),
            };
            ("syntax", out)
        } else if kind == "QueryEvaluationTest" {
            ("query-eval", query_eval(root, e))
        } else if kind == "UpdateEvaluationTest" {
            (
                "update-eval",
                Outcome::Skip(
                    "no update executor in the dependencies (spareval runs queries and DELETE/INSERT deltas only)",
                ),
            )
        } else {
            (
                "other",
                Outcome::Skip("protocol, service and results-format tests are out of scope"),
            )
        };
        report.record("parser", &manifest, category, &id, parser_out);
        report.record(
            "adapter",
            &manifest,
            category,
            &id,
            Outcome::Skip("sparql() makes the same SparqlParser call (no base IRI), then evaluates SELECT and ASK over a kernel Graph; no separate syntax or update entry point"),
        );
    }
}

#[cfg(feature = "query")]
fn fmt_of_path(p: &Path) -> Option<Fmt> {
    match p.extension()?.to_str()? {
        "ttl" => Some(Fmt::Turtle),
        "nt" => Some(Fmt::NTriples),
        "nq" => Some(Fmt::NQuads),
        "trig" => Some(Fmt::TriG),
        "rdf" | "xml" => Some(Fmt::Xml),
        _ => None,
    }
}

/// Load a data file; a named graph's triples go into the graph named by the file's address.
#[cfg(feature = "query")]
fn load(root: &Path, path: &Path, named: bool) -> Result<oxrdf::Dataset, String> {
    let fmt = fmt_of_path(path).ok_or("unrecognised data format")?;
    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    let base = test_base(root, path);
    let ds = parse_rdf(fmt, &bytes, &base)?;
    if !named {
        return Ok(ds);
    }
    let graph = oxrdf::NamedNode::new(base).map_err(|e| e.to_string())?;
    let mut out = oxrdf::Dataset::new();
    for q in ds.iter() {
        let q = q.into_owned();
        out.insert(&oxrdf::Quad::new(
            q.subject,
            q.predicate,
            q.object,
            oxrdf::GraphName::NamedNode(graph.clone()),
        ));
    }
    Ok(out)
}

/// Numeric literals compare by value, as the suite's result comparison does.
#[cfg(feature = "query")]
fn norm(term: oxrdf::Term) -> oxrdf::Term {
    use oxrdf::Term;
    if let Term::Literal(l) = &term {
        let dt = l.datatype().as_str();
        if let Some(local) = dt.strip_prefix("http://www.w3.org/2001/XMLSchema#")
            && matches!(local, "integer" | "decimal" | "double" | "float")
            && let Ok(v) = l.value().trim_start_matches('+').parse::<f64>()
        {
            return Term::Literal(oxrdf::Literal::new_typed_literal(
                format!("{v:?}"),
                l.datatype().into_owned(),
            ));
        }
    }
    term
}

/// A solution sequence as one dataset: a blank node per row, one triple per bound variable.
/// Isomorphism of two such datasets is multiset equality of rows up to blank-node renaming;
/// row order is not checked.
#[cfg(feature = "query")]
fn rows_dataset(
    vars: &[String],
    rows: impl Iterator<Item = Vec<Option<oxrdf::Term>>>,
) -> oxrdf::Dataset {
    let mut ds = oxrdf::Dataset::new();
    for (i, row) in rows.enumerate() {
        let subject = oxrdf::BlankNode::new(format!("row{i}")).unwrap();
        for (var, value) in vars.iter().zip(row) {
            if let Some(value) = value.map(norm) {
                let pred = oxrdf::NamedNode::new(format!("urn:var:{var}")).unwrap();
                ds.insert(&oxrdf::Quad::new(
                    subject.clone(),
                    pred,
                    value,
                    oxrdf::GraphName::DefaultGraph,
                ));
            }
        }
        // A row with no bindings still counts once.
        let marker = oxrdf::NamedNode::new("urn:row").unwrap();
        ds.insert(&oxrdf::Quad::new(
            subject,
            marker.clone(),
            marker,
            oxrdf::GraphName::DefaultGraph,
        ));
    }
    ds
}

#[cfg(feature = "query")]
type Rows = Vec<Vec<Option<oxrdf::Term>>>;

/// Read an expected solution sequence: its variables and its rows.
#[cfg(feature = "query")]
fn expected_rows(
    format: sparesults::QueryResultsFormat,
    bytes: &[u8],
) -> Result<Option<(Vec<String>, Rows)>, String> {
    use sparesults::{QueryResultsParser, SliceQueryResultsParserOutput};
    match QueryResultsParser::from_format(format).for_slice(bytes) {
        Ok(SliceQueryResultsParserOutput::Solutions(sols)) => {
            let vars: Vec<String> = sols.variables().iter().map(|v| v.as_str().into()).collect();
            let mut rows = Vec::new();
            for s in sols {
                let s = s.map_err(|e| format!("expected result unparseable: {e}"))?;
                rows.push(vars.iter().map(|v| s.get(v.as_str()).cloned()).collect());
            }
            Ok(Some((vars, rows)))
        },
        Ok(SliceQueryResultsParserOutput::Boolean(_)) => Ok(None),
        Err(err) => Err(format!("expected result unparseable: {err}")),
    }
}

/// The graph IRIs a query's FROM and FROM NAMED clauses name.
#[cfg(feature = "query")]
fn from_iris(text: &str, base: &str) -> Vec<oxrdf::NamedNode> {
    use spargebra::Query;
    let Ok(parser) = spargebra::SparqlParser::new().with_base_iri(base) else {
        return Vec::new();
    };
    let Ok(query) = parser.parse_query(text) else {
        return Vec::new();
    };
    let dataset = match &query {
        Query::Select { dataset, .. }
        | Query::Construct { dataset, .. }
        | Query::Describe { dataset, .. }
        | Query::Ask { dataset, .. } => dataset,
    };
    dataset
        .iter()
        .flat_map(|d| d.default.iter().chain(d.named.iter().flatten()))
        .cloned()
        .collect()
}

#[cfg(feature = "query")]
fn query_eval(root: &Path, e: &Entry) -> Outcome {
    use sparesults::{QueryResultsFormat, QueryResultsParser, SliceQueryResultsParserOutput};
    use spareval::{QueryEvaluator, QueryResults};
    use spargebra::SparqlParser;

    let (Some(q), Some(res)) = (
        e.parts.get("query").and_then(|v| v.first()),
        e.result.as_ref(),
    ) else {
        return Outcome::Skip("no query or result file in the manifest entry");
    };
    let mut data = oxrdf::Dataset::new();
    let loaded = e
        .parts
        .get("data")
        .into_iter()
        .flatten()
        .map(|p| load(root, p, false))
        .chain(
            e.parts
                .get("graphData")
                .into_iter()
                .flatten()
                .map(|p| load(root, p, true)),
        );
    for d in loaded {
        match d {
            Ok(d) => d.iter().for_each(|q| {
                data.insert(q);
            }),
            Err(err) => return Outcome::Fail(format!("data unreadable: {err}")),
        }
    }
    let Ok(text) = std::fs::read_to_string(q) else {
        return Outcome::Fail("query file unreadable".into());
    };
    // FROM and FROM NAMED name graphs: one the manifest data does not hold is loaded from the
    // suite directory, from the file the IRI addresses.
    for iri in from_iris(&text, &test_base(root, q)) {
        let held = data.iter().any(
            |d| matches!(d.graph_name, oxrdf::GraphNameRef::NamedNode(n) if n == iri.as_ref()),
        );
        if held {
            continue;
        }
        let Some(rest) = iri.as_str().strip_prefix(crate::manifest::TEST_BASE) else {
            continue;
        };
        let file = root.join(rest);
        if file.is_file() {
            match load(root, &file, true) {
                Ok(d) => d.iter().for_each(|q| {
                    data.insert(q);
                }),
                Err(err) => return Outcome::Fail(format!("FROM data unreadable: {err}")),
            }
        }
    }
    let run = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| -> Result<bool, String> {
        let parser = SparqlParser::new()
            .with_base_iri(test_base(root, q))
            .map_err(|e| e.to_string())?;
        let query = parser
            .parse_query(&text)
            .map_err(|e| format!("rejected: {e}"))?;
        let results = QueryEvaluator::new()
            .prepare(&query)
            .execute(&data)
            .map_err(|e| format!("evaluation: {e}"))?;
        let ext = res.extension().and_then(|x| x.to_str()).unwrap_or("");
        let result_bytes = std::fs::read(res).map_err(|e| e.to_string())?;
        let format = match ext {
            "srx" => Some(QueryResultsFormat::Xml),
            "srj" => Some(QueryResultsFormat::Json),
            "tsv" => Some(QueryResultsFormat::Tsv),
            _ => None,
        };
        match (results, format) {
            (QueryResults::Boolean(got), Some(format)) => {
                match QueryResultsParser::from_format(format).for_slice(&result_bytes) {
                    Ok(SliceQueryResultsParserOutput::Boolean(want)) => Ok(got == want),
                    Ok(_) => Err("expected solutions, got boolean".to_string()),
                    Err(err) => Err(format!("expected result unparseable: {err}")),
                }
            },
            (QueryResults::Solutions(sols), Some(format)) => {
                let vars: Vec<String> =
                    sols.variables().iter().map(|v| v.as_str().into()).collect();
                let mut got: Rows = Vec::new();
                for s in sols {
                    let s = s.map_err(|e| format!("evaluation: {e}"))?;
                    got.push(vars.iter().map(|v| s.get(v.as_str()).cloned()).collect());
                }
                let Some((want_vars, want)) = expected_rows(format, &result_bytes)? else {
                    return Err("expected a boolean, got solutions".to_string());
                };
                // Compare over the expected variable list; a missing column is a mismatch.
                let got: Rows = got
                    .iter()
                    .map(|row| {
                        want_vars
                            .iter()
                            .map(|v| {
                                vars.iter()
                                    .position(|g| g == v)
                                    .and_then(|i| row[i].clone())
                            })
                            .collect()
                    })
                    .collect();
                let (mut a, mut b) = (vars.clone(), want_vars.clone());
                a.sort();
                b.sort();
                let note = format!(
                    "got {} rows {:?}, want {} rows {:?}",
                    got.len(),
                    got.first().map(|r| r
                        .iter()
                        .map(|t| t.as_ref().map(|t| t.to_string()))
                        .collect::<Vec<_>>()),
                    want.len(),
                    want.first().map(|r| r
                        .iter()
                        .map(|t| t.as_ref().map(|t| t.to_string()))
                        .collect::<Vec<_>>()),
                );
                if a == b
                    && isomorphic(
                        rows_dataset(&want_vars, got.into_iter()),
                        rows_dataset(&want_vars, want.into_iter()),
                    )
                {
                    Ok(true)
                } else {
                    Err(format!("results differ: {note}"))
                }
            },
            (QueryResults::Graph(triples), _) => {
                let fmt = fmt_of_path(res).ok_or("unsupported result file type")?;
                let want = parse_rdf(fmt, &result_bytes, &test_base(root, res))?;
                let mut got = oxrdf::Dataset::new();
                for t in triples {
                    let t = t.map_err(|e| format!("evaluation: {e}"))?;
                    got.insert(&t.in_graph(oxrdf::GraphName::DefaultGraph));
                }
                Ok(isomorphic(got, want))
            },
            (_, None) => Err("unsupported result file type".to_string()),
        }
    }));
    match run {
        Ok(Ok(true)) => Outcome::Pass,
        Ok(Ok(false)) => Outcome::Fail("results differ from the expected ones".into()),
        Ok(Err(msg)) if msg.starts_with("unsupported result file type") => {
            Outcome::Skip("expected result in a format the harness does not read")
        },
        Ok(Err(msg)) => Outcome::Fail(msg),
        Err(_) => Outcome::Fail("evaluator panicked".into()),
    }
}
