// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The headless resource inspector (engine split P5, D141 F11).
//!
//! `inspect <file> [--base IRI] [--query SPARQL]` parses a Turtle, TriG,
//! N-Quads or JSON-LD file with the engine's own entry points and reads the
//! returned dataset with SPARQL: resources, assertions per asserter (held and
//! withdrawn, from the RDF 1.2 reifier metadata), named graphs with quad
//! counts, reason keys, and optionally one query's rows. It depends on the
//! engine alone. Run with `cargo run -p mere --example inspect --features
//! query -- <file>`. The logic is `inspect()`; `tests/inspect_example.rs`
//! includes this file and calls it.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use mere::model::CoverageNote;
use mere::query::{IndexedDataset, QueryRows, View, sparql_on};
use mere::{
    ASSERTION_HISTORY_GRAPH_IRI, ContextCache, MERE_METHOD_VERSION, WITHDRAWAL_ACTIVITY_PREFIX,
};

const PROV: &str = "http://www.w3.org/ns/prov#";
const RDF_REIFIES: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#reifies";
const RDFS_LABEL: &str = "http://www.w3.org/2000/01/rdf-schema#label";

/// Parse `text` by the file extension (`RdfFormat::from_extension`).
pub fn parse(ext: &str, text: &str, base: Option<&str>) -> Result<IndexedDataset, String> {
    let format = mere::formats::RdfFormat::from_extension(ext)
        .ok_or_else(|| format!("unknown extension .{ext}: use ttl, trig, nq, nt, jsonld, rdf"))?;
    mere::formats::parse::parse(format, text.as_bytes(), base, ContextCache::full())
        .map_err(|e| e.to_string())
}

/// Held and withdrawn assertion counts for one asserter.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Tally {
    pub held: usize,
    pub withdrawn: usize,
}

/// What the inspector found; `Display` is the text report.
#[derive(Debug)]
pub struct Report {
    pub rows: usize,
    pub terms: usize,
    /// Resource IRI -> `predicate object [graph]` lines.
    pub resources: BTreeMap<String, Vec<String>>,
    /// Asserter IRI (or `(none)`) -> counts.
    pub asserters: BTreeMap<String, Tally>,
    /// One line per assertion: asserter, state, scope, reifier, reason.
    pub assertions: Vec<String>,
    /// Graph name (`(default)` for the default graph) -> quad count.
    pub graphs: BTreeMap<String, usize>,
    /// Reason key (the assertion method IRI) -> assertions citing it.
    pub reasons: BTreeMap<String, usize>,
    pub query: Option<QueryRows>,
}

fn select(ds: &IndexedDataset, q: &str) -> Result<QueryRows, String> {
    sparql_on(ds, q, View::Live, CoverageNote::default())
}

fn cell(row: &[Option<String>], i: usize) -> Option<&str> {
    row.get(i).and_then(|c| c.as_deref())
}

/// Read the dataset into a [`Report`], running `query` last when given.
pub fn inspect(ds: &IndexedDataset, query: Option<&str>) -> Result<Report, String> {
    let mut report = Report {
        rows: ds.len(),
        terms: ds.term_count(),
        resources: BTreeMap::new(),
        asserters: BTreeMap::new(),
        assertions: Vec::new(),
        graphs: BTreeMap::new(),
        reasons: BTreeMap::new(),
        query: None,
    };

    // Reifiers: every `?r rdf:reifies ?t`, with its metadata, in any graph.
    let body = format!(
        "?r <{RDF_REIFIES}> ?t .
         OPTIONAL {{ ?r <{PROV}wasAttributedTo> ?by }}
         OPTIONAL {{ ?r <{PROV}wasGeneratedBy> ?method }}
         OPTIONAL {{ ?r <{MERE_METHOD_VERSION}> ?version }}
         OPTIONAL {{ ?r <{RDFS_LABEL}> ?note }}
         OPTIONAL {{ ?r <{PROV}wasInvalidatedBy> ?gone }}"
    );
    let reifiers = select(
        ds,
        &format!(
            "SELECT ?g ?r ?by ?method ?version ?note ?gone WHERE {{
               {{ {body} }} UNION {{ GRAPH ?g {{ {body} }} }} }}"
        ),
    )?;
    let mut reifier_ids = BTreeSet::new();
    for row in &reifiers.rows {
        let (g, r) = (cell(row, 0), cell(row, 1).unwrap_or_default());
        let withdrawn = g == Some(ASSERTION_HISTORY_GRAPH_IRI) || cell(row, 6).is_some();
        let by = cell(row, 2).unwrap_or("(none)");
        let tally = report.asserters.entry(by.to_string()).or_default();
        if withdrawn {
            tally.withdrawn += 1;
        } else {
            tally.held += 1;
        }
        let mut reason = String::new();
        if let Some(method) = cell(row, 3) {
            *report.reasons.entry(method.to_string()).or_default() += 1;
            reason.push_str(method);
            if let Some(version) = cell(row, 4) {
                let _ = write!(reason, " v{version}");
            }
        }
        if let Some(note) = cell(row, 5) {
            if !reason.is_empty() {
                reason.push_str("; ");
            }
            let _ = write!(reason, "\"{note}\"");
        }
        report.assertions.push(format!(
            "{by}  {}  in {}  {r}  reason: {}",
            if withdrawn { "withdrawn" } else { "held" },
            g.unwrap_or("(default)"),
            if reason.is_empty() { "(none)" } else { &reason },
        ));
        reifier_ids.insert(r.to_string());
    }

    // Quads: resources and the per-graph counts.
    let quads = select(
        ds,
        "SELECT ?g ?s ?p ?o WHERE {
           { ?s ?p ?o } UNION { GRAPH ?g { ?s ?p ?o } } }",
    )?;
    for row in &quads.rows {
        let g = cell(row, 0);
        *report
            .graphs
            .entry(g.unwrap_or("(default)").to_string())
            .or_default() += 1;
        let (Some(s), Some(p), Some(o)) = (cell(row, 1), cell(row, 2), cell(row, 3)) else {
            continue;
        };
        let machinery = reifier_ids.contains(s) || s.starts_with(WITHDRAWAL_ACTIVITY_PREFIX);
        let alignment = g == Some(mere::profile::vocab::GRAPH_VOCABULARY);
        if machinery || alignment || s.starts_with("_:") || p == RDF_REIFIES {
            continue;
        }
        report
            .resources
            .entry(s.to_string())
            .or_default()
            .push(format!("{p} {o} [{}]", g.unwrap_or("(default)")));
    }

    if let Some(q) = query {
        report.query = Some(select(ds, q)?);
    }
    Ok(report)
}

impl std::fmt::Display for Report {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "dataset: {} rows, {} terms", self.rows, self.terms)?;
        writeln!(f, "\nresources ({}):", self.resources.len())?;
        for (iri, lines) in &self.resources {
            writeln!(f, "  {iri}")?;
            for line in lines {
                writeln!(f, "    {line}")?;
            }
        }
        writeln!(f, "\nasserters ({}):", self.asserters.len())?;
        for (by, t) in &self.asserters {
            writeln!(f, "  {by}: {} held, {} withdrawn", t.held, t.withdrawn)?;
        }
        writeln!(f, "\nassertions ({}):", self.assertions.len())?;
        for line in &self.assertions {
            writeln!(f, "  {line}")?;
        }
        writeln!(f, "\nnamed graphs ({}):", self.graphs.len())?;
        for (g, n) in &self.graphs {
            writeln!(f, "  {g}: {n} quads")?;
        }
        writeln!(f, "\nreason keys ({}):", self.reasons.len())?;
        for (key, n) in &self.reasons {
            writeln!(f, "  {key}: {n} assertions")?;
        }
        if let Some(q) = &self.query {
            writeln!(
                f,
                "\nquery ({} rows): {}",
                q.rows.len(),
                q.variables.join(" | ")
            )?;
            for row in &q.rows {
                let cells: Vec<&str> = row.iter().map(|c| c.as_deref().unwrap_or("-")).collect();
                writeln!(f, "  {}", cells.join(" | "))?;
            }
        }
        Ok(())
    }
}

#[allow(
    dead_code,
    reason = "the integration test includes this file without running its main"
)]
fn main() -> Result<(), String> {
    let mut args = std::env::args().skip(1);
    let (mut file, mut base, mut query) = (None, None, None);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--base" => base = args.next(),
            "--query" => query = args.next(),
            _ if file.is_none() => file = Some(arg),
            _ => return Err(format!("unexpected argument {arg}")),
        }
    }
    let usage = "usage: inspect <file> [--base IRI] [--query SPARQL]";
    let file = file.ok_or(usage)?;
    let ext = std::path::Path::new(&file)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    let text = std::fs::read_to_string(&file).map_err(|e| format!("{file}: {e}"))?;
    let ds = parse(&ext, &text, base.as_deref())?;
    print!("{}", inspect(&ds, query.as_deref())?);
    Ok(())
}
