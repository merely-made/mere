// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The headless resource inspector (engine split P5, D141 F11; P6 L1).
//!
//! `inspect <file> [--base IRI] [--query SPARQL]` parses a Turtle, TriG,
//! N-Quads or JSON-LD file with the engine's own entry points and reads the
//! returned dataset through `IndexedDataset::quads`: resources, assertions per
//! asserter (held and withdrawn, from the RDF 1.2 reifier metadata), named
//! graphs with quad counts and reason keys. Those sections need no SPARQL;
//! `--query` runs one query and needs the `query` feature. It depends on the
//! engine alone. Run with `cargo run -p mere --example inspect -- <file>`. The
//! logic is `inspect()`; `tests/inspect_example.rs` includes this file and
//! calls it.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fmt::Write as _;

use mere::dataset::{IndexedDataset, View};
#[cfg(feature = "query")]
use mere::query::QueryRows;
use mere::{
    ASSERTION_HISTORY_GRAPH_IRI, ContextCache, MERE_METHOD_VERSION, WITHDRAWAL_ACTIVITY_PREFIX,
};
use oxrdf::{GraphName, Term};

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
    #[cfg(feature = "query")]
    pub query: Option<QueryRows>,
}

/// A term as the report shows it: the bare IRI or lexical value, `_:id` for a blank node.
fn show(term: &Term) -> String {
    match term {
        Term::NamedNode(n) => n.as_str().to_string(),
        Term::Literal(l) => l.value().to_string(),
        Term::BlankNode(b) => format!("_:{}", b.as_str()),
        Term::Triple(triple) => triple.to_string(),
    }
}

/// One live quad with every position as report text; `None` is the default graph.
struct Row {
    graph: Option<String>,
    subject: String,
    predicate: String,
    object: String,
}

/// What each node says, graph by graph: `(graph, subject)` -> `(predicate, object)` pairs.
type Props = HashMap<(Option<String>, String), Vec<(String, String)>>;

/// The first object of `subject`'s `predicate` in `graph`, by sorted text.
fn first<'a>(
    props: &'a Props,
    graph: &Option<String>,
    subject: &str,
    predicate: &str,
) -> Option<&'a str> {
    props
        .get(&(graph.clone(), subject.to_string()))?
        .iter()
        .filter(|(p, _)| p == predicate)
        .map(|(_, o)| o.as_str())
        .min()
}

/// Read the dataset into a [`Report`], running `query` last when given.
///
/// # Errors
///
/// A query error, or `query` given when the `query` feature is off.
pub fn inspect(ds: &IndexedDataset, query: Option<&str>) -> Result<Report, String> {
    let mut report = Report {
        rows: ds.len(),
        terms: ds.term_count(),
        resources: BTreeMap::new(),
        asserters: BTreeMap::new(),
        assertions: Vec::new(),
        graphs: BTreeMap::new(),
        reasons: BTreeMap::new(),
        #[cfg(feature = "query")]
        query: None,
    };

    let quads: Vec<Row> = ds
        .quads(View::Live)
        .map(|q| Row {
            graph: match q.graph_name {
                GraphName::DefaultGraph => None,
                GraphName::NamedNode(n) => Some(n.as_str().to_string()),
                GraphName::BlankNode(b) => Some(format!("_:{}", b.as_str())),
            },
            subject: show(&Term::from(q.subject)),
            predicate: q.predicate.as_str().to_string(),
            object: show(&q.object),
        })
        .collect();
    let mut props = Props::new();
    for q in &quads {
        props
            .entry((q.graph.clone(), q.subject.clone()))
            .or_default()
            .push((q.predicate.clone(), q.object.clone()));
    }

    // Reifiers: every `?r rdf:reifies ?t`, with its metadata beside it, in any graph.
    let mut reifier_ids = BTreeSet::new();
    for q in quads.iter().filter(|q| q.predicate == RDF_REIFIES) {
        let (g, r) = (&q.graph, q.subject.as_str());
        let prop = |p: &str| first(&props, g, r, p);
        let withdrawn = g.as_deref() == Some(ASSERTION_HISTORY_GRAPH_IRI)
            || prop(&format!("{PROV}wasInvalidatedBy")).is_some();
        let by = prop(&format!("{PROV}wasAttributedTo")).unwrap_or("(none)");
        let tally = report.asserters.entry(by.to_string()).or_default();
        if withdrawn {
            tally.withdrawn += 1;
        } else {
            tally.held += 1;
        }
        let mut reason = String::new();
        if let Some(method) = prop(&format!("{PROV}wasGeneratedBy")) {
            *report.reasons.entry(method.to_string()).or_default() += 1;
            reason.push_str(method);
            if let Some(version) = prop(MERE_METHOD_VERSION) {
                let _ = write!(reason, " v{version}");
            }
        }
        if let Some(note) = prop(RDFS_LABEL) {
            if !reason.is_empty() {
                reason.push_str("; ");
            }
            let _ = write!(reason, "\"{note}\"");
        }
        report.assertions.push(format!(
            "{by}  {}  in {}  {r}  reason: {}",
            if withdrawn { "withdrawn" } else { "held" },
            g.as_deref().unwrap_or("(default)"),
            if reason.is_empty() { "(none)" } else { &reason },
        ));
        reifier_ids.insert(r.to_string());
    }
    report.assertions.sort();

    // Quads: resources and the per-graph counts.
    for q in &quads {
        let g = q.graph.as_deref();
        *report
            .graphs
            .entry(g.unwrap_or("(default)").to_string())
            .or_default() += 1;
        let (s, p, o) = (&q.subject, &q.predicate, &q.object);
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
    for lines in report.resources.values_mut() {
        lines.sort();
    }

    if let Some(q) = query {
        #[cfg(feature = "query")]
        {
            report.query = Some(mere::query::sparql_on(
                ds,
                q,
                View::Live,
                mere::model::CoverageNote::default(),
            )?);
        }
        #[cfg(not(feature = "query"))]
        return Err(format!("--query needs the `query` feature: {q}"));
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
        #[cfg(feature = "query")]
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
