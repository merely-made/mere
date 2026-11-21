// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Run the W3C SHACL 1.0 Core manifest through Mere. Takes the local tests/core directory.
//! Emits one TSV ROW per test, with separate conformance and report-graph verdicts.

use std::collections::{BTreeSet, HashSet};
use std::error::Error;
use std::path::{Path, PathBuf};

use mere::dataset::{IndexedDataset, View};
use mere::formats::parse::parse_turtle;
use mere_shacl::{GraphSelector, validate};
use oxrdf::vocab::rdf;
use oxrdf::{Graph, NamedNode, NamedOrBlankNode, Term, Triple};

mod report;

type Result<T> = std::result::Result<T, Box<dyn Error>>;
const BASE: &str = "https://w3c.github.io/data-shapes/data-shapes-test-suite/tests/core/";
const MF: &str = "http://www.w3.org/2001/sw/DataAccess/tests/test-manifest#";
const SHT: &str = "http://www.w3.org/ns/shacl-test#";

fn iri(ns: &str, local: &str) -> NamedNode {
    NamedNode::new_unchecked(format!("{ns}{local}"))
}

fn objects(graph: &Graph, subject: &NamedOrBlankNode, predicate: &NamedNode) -> Vec<Term> {
    graph
        .objects_for_subject_predicate(subject.as_ref(), predicate.as_ref())
        .map(|term| term.into_owned())
        .collect()
}

fn one(graph: &Graph, subject: &NamedOrBlankNode, predicate: &NamedNode) -> Result<Term> {
    match objects(graph, subject, predicate).as_slice() {
        [term] => Ok(term.clone()),
        _ => Err(format!("expected one {predicate} for {subject}").into()),
    }
}

fn read(root: &Path, url: &NamedNode) -> Result<IndexedDataset> {
    let rel = url
        .as_str()
        .strip_prefix(BASE)
        .ok_or_else(|| format!("fixture URL outside the Core suite: {url}"))?;
    let file = root.join(rel);
    if !file.canonicalize()?.starts_with(root) {
        return Err(format!("fixture path outside the Core suite: {}", file.display()).into());
    }
    Ok(parse_turtle(
        &std::fs::read_to_string(file)?,
        Some(url.as_str()),
    )?)
}

fn graph(data: &IndexedDataset) -> Graph {
    data.quads(View::Live).map(Triple::from).collect()
}

fn entries(graph: &Graph, mut list: Term) -> Result<Vec<NamedOrBlankNode>> {
    let mut visited = HashSet::new();
    let mut entries = Vec::new();
    while list != Term::from(rdf::NIL) {
        let cell = NamedOrBlankNode::try_from(list)?;
        if !visited.insert(cell.clone()) {
            return Err("cyclic mf:entries list".into());
        }
        entries.push(NamedOrBlankNode::try_from(one(
            graph,
            &cell,
            &rdf::FIRST.into_owned(),
        )?)?);
        list = one(graph, &cell, &rdf::REST.into_owned())?;
    }
    Ok(entries)
}

fn fixture(
    root: &Path,
    manifest: &Graph,
    case: &NamedOrBlankNode,
) -> Result<(&'static str, &'static str, String)> {
    let action = NamedOrBlankNode::try_from(one(manifest, case, &iri(MF, "action"))?)?;
    let shapes = read(
        root,
        &NamedNode::try_from(one(manifest, &action, &iri(SHT, "shapesGraph"))?)?,
    )?;
    let data = read(
        root,
        &NamedNode::try_from(one(manifest, &action, &iri(SHT, "dataGraph"))?)?,
    )?;
    let expected = one(manifest, case, &iri(MF, "result"))?;
    let shapes = graph(&shapes);
    let actual = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        validate(data, View::Live, GraphSelector::Default, &shapes)
    }));
    let actual = match actual {
        Err(_) => return Ok(("error", "error", "validator panicked".into())),
        Ok(actual) => actual,
    };
    if expected == Term::from(iri(SHT, "Failure")) {
        return Ok(if actual.is_err() {
            (
                "pass",
                "skip",
                "suite expects validation failure rather than a report".into(),
            )
        } else {
            (
                "fail",
                "skip",
                "suite expects validation failure, but Mere produced a report".into(),
            )
        });
    }
    let actual = match actual {
        Ok(report) => report,
        Err(error) => return Ok(("error", "error", error.to_string())),
    };
    let expected = NamedOrBlankNode::try_from(expected)?;
    let expected_conforms = report::conforms(manifest, &expected)?;
    let boolean = if actual.conforms == expected_conforms {
        "pass"
    } else {
        "fail"
    };
    let actual_graph: Graph = actual
        .dataset
        .iter()
        .filter(|q| q.graph_name.is_default_graph())
        .map(|q| Triple::new(q.subject, q.predicate, q.object))
        .collect();
    let actual_report = match report::compare(manifest, &expected, &actual_graph) {
        Ok(None) => return Ok((boolean, "pass", String::new())),
        Ok(Some(diff)) => ("fail", diff),
        Err(error) => ("error", error.to_string()),
    };
    Ok((boolean, actual_report.0, actual_report.1))
}

fn run(root: &Path) -> Result<()> {
    let mut pending = BTreeSet::from([iri(BASE, "manifest.ttl")]);
    let mut seen = BTreeSet::new();
    let mut case_count = 0;
    println!("kind\tfamily\tfixture\tconforms\treport_graph\tdetail");
    while let Some(url) = pending.pop_first() {
        if !seen.insert(url.clone()) {
            continue;
        }
        let manifest = graph(&read(root, &url)?);
        let subject = NamedOrBlankNode::from(url.clone());
        for next in objects(&manifest, &subject, &iri(MF, "include")) {
            pending.insert(NamedNode::try_from(next)?);
        }
        for list in objects(&manifest, &subject, &iri(MF, "entries")) {
            for case in entries(&manifest, list)? {
                case_count += 1;
                let rel = url.as_str().strip_prefix(BASE).unwrap();
                let family = rel.split('/').next().unwrap();
                let (boolean, report, detail) = if objects(&manifest, &case, &iri(MF, "status"))
                    != [Term::from(iri(SHT, "approved"))]
                {
                    ("skip", "skip", "fixture is not approved".into())
                } else if !manifest
                    .contains(Triple::new(case.clone(), rdf::TYPE, iri(SHT, "Validate")).as_ref())
                {
                    ("skip", "skip", "fixture is not sht:Validate".into())
                } else {
                    match fixture(root, &manifest, &case) {
                        Ok(scores) => scores,
                        Err(error) => ("error", "error", format!("harness: {error}")),
                    }
                };
                println!(
                    "ROW\t{family}\t{case}\t{boolean}\t{report}\t{}",
                    detail.escape_default()
                );
            }
        }
    }
    if case_count == 0 {
        return Err("Core manifest selected no tests".into());
    }
    eprintln!("Evaluated {case_count} Core manifest entries");
    Ok(())
}

fn main() -> Result<()> {
    let root = PathBuf::from(
        std::env::args_os()
            .nth(1)
            .ok_or("expected a path to the W3C tests/core directory")?,
    );
    run(&root.canonicalize()?)
}
