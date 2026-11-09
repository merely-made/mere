// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Controls for the SPARQL evaluation and RDF/XML failures: each loads its data by hand (inline
//! TriG, no manifest reading and no comparator), runs `spareval` or `oxrdfxml` directly, and
//! prints the result beside the expected one. A failure the harness and a control both show is
//! the engine's; a control that matches means the harness was at fault. The controls print
//! rather than assert, so a dependency upgrade that fixes a case shows up as `MATCH`.

#![cfg(feature = "query")]

use oxrdf::{Dataset, GraphName};
use oxttl::TriGParser;
use spareval::{QueryEvaluator, QueryResults};
use spargebra::SparqlParser;

const PREFIX: &str = "http://example.org/";

/// Evaluate `query` over TriG `data` (default graph and named graphs as written).
fn eval(data: &str, query: &str) -> Result<Vec<String>, String> {
    let mut ds = Dataset::new();
    for q in TriGParser::new().for_slice(data) {
        ds.insert(&q.map_err(|e| e.to_string())?);
    }
    let query = SparqlParser::new()
        .parse_query(query)
        .map_err(|e| format!("rejected: {e}"))?;
    let mut rows = match QueryEvaluator::new()
        .prepare(&query)
        .execute(&ds)
        .map_err(|e| e.to_string())?
    {
        QueryResults::Boolean(b) => vec![b.to_string()],
        QueryResults::Solutions(sols) => {
            let mut rows = Vec::new();
            for s in sols {
                let s = s.map_err(|e| e.to_string())?;
                let mut cells: Vec<String> = s
                    .iter()
                    .map(|(v, t)| format!("{}={t}", v.as_str()))
                    .collect();
                cells.sort();
                rows.push(cells.join(" "));
            }
            rows
        },
        QueryResults::Graph(_) => return Err("graph result".into()),
    };
    rows.sort();
    Ok(rows)
}

fn show(name: &str, got: Result<Vec<String>, String>, want: &[&str]) -> bool {
    let mut want: Vec<String> = want.iter().map(|s| s.to_string()).collect();
    want.sort();
    let verdict = if got.as_ref().is_ok_and(|g| *g == want) {
        "MATCH"
    } else {
        "DIVERGES"
    };
    println!("CONTROL\t{name}\t{verdict}\tgot {got:?}\twant {want:?}");
    verdict == "MATCH"
}

#[test]
fn engine_controls() {
    let g = |n: &str| format!("<{PREFIX}{n}>");
    let int = |n: u8| format!("\"{n}\"^^<http://www.w3.org/2001/XMLSchema#integer>");
    let trig = "@prefix : <http://example.org/> . GRAPH :g1 { :s :p :o } GRAPH :g2 { :s :p :o }";
    let pre = "PREFIX ex: <http://example.org/>\n";

    // Positive control: the instrument returns a right answer on a plain query, and FROM NAMED
    // restricts a plain GRAPH pattern.
    assert!(show(
        "positive: plain pattern",
        eval(trig, "SELECT ?g WHERE { GRAPH ?g { ?s ?p ?o } }"),
        &[&format!("g={}", g("g1")), &format!("g={}", g("g2"))],
    ));
    assert!(show(
        "positive: FROM NAMED over a plain GRAPH pattern",
        eval(
            trig,
            &format!("{pre}SELECT ?g FROM NAMED ex:g1 WHERE {{ GRAPH ?g {{ ?s ?p ?o }} }}")
        ),
        &[&format!("g={}", g("g1"))],
    ));

    let mut diverges = 0;
    let mut check = |ok: bool| diverges += usize::from(!ok);

    check(show(
        "graph-from-named-1",
        eval(
            trig,
            &format!(
                "{pre}SELECT ?g ?x FROM NAMED ex:g1 WHERE {{ GRAPH ?g {{ VALUES (?g ?x) {{ (UNDEF 1) (ex:g1 2) (ex:g2 3) (ex:unknown 4) }} }} }}"
            ),
        ),
        &[
            &format!("g={} x={}", g("g1"), int(1)),
            &format!("g={} x={}", g("g1"), int(2)),
        ],
    ));
    check(show(
        "graph-from-named-2",
        eval(
            trig,
            &format!(
                "{pre}SELECT ?g ?x FROM NAMED ex:g1 WHERE {{ VALUES ?g {{ ex:g1 ex:g2 ex:unknown }} GRAPH ?g {{ VALUES ?x {{ 1 }} }} }}"
            ),
        ),
        &[&format!("g={} x={}", g("g1"), int(1))],
    ));
    check(show(
        "graph-from-named-3",
        eval(
            trig,
            &format!(
                "{pre}SELECT ?x FROM NAMED ex:g1 WHERE {{ {{ GRAPH ex:g1 {{ VALUES ?x {{ 1 }} }} }} UNION {{ GRAPH ex:g2 {{ VALUES ?x {{ 2 }} }} }} UNION {{ GRAPH ex:unknown {{ VALUES ?x {{ 3 }} }} }} }}"
            ),
        ),
        &[&format!("x={}", int(1))],
    ));
    check(show(
        "graph-minus",
        eval(
            "@prefix : <http://example/> . GRAPH :g { :a :p :o }",
            "PREFIX : <http://example/> SELECT ?a WHERE { GRAPH ?g { ?a :p :o MINUS { ?b :p :o } } }",
        ),
        &["a=<http://example/a>"],
    ));
    for (name, query, want) in [
        (
            "zero_or_more_set_start",
            "SELECT ?s WHERE { ?s :p* :o }",
            "s=<http://example/o>",
        ),
        (
            "zero_or_more_set_end",
            "SELECT ?o WHERE { :s :p* ?o }",
            "o=<http://example/s>",
        ),
        (
            "zero_or_one_set_start",
            "SELECT ?s WHERE { ?s :p? :o }",
            "s=<http://example/o>",
        ),
        (
            "zero_or_one_set_end",
            "SELECT ?o WHERE { :s :p? ?o }",
            "o=<http://example/s>",
        ),
    ] {
        check(show(
            name,
            eval("", &format!("PREFIX : <http://example/>\n{query}")),
            &[want],
        ));
    }
    check(show(
        "agg-groupconcat-04",
        eval(
            "",
            r#"ASK { {SELECT (GROUP_CONCAT(?o) AS ?g) WHERE { VALUES ?o { "1"@en "2"@en } }} FILTER(?g = "1 2" || ?g = "2 1") }"#,
        ),
        &["true"],
    ));
    check(show(
        "agg-groupconcat-06",
        eval(
            "",
            r#"ASK { {SELECT (GROUP_CONCAT(?o) AS ?g) WHERE { VALUES ?o { "1"@en } }} FILTER(?g = "1") }"#,
        ),
        &["true"],
    ));
    let sq15 = "@prefix : <http://example.com/> . GRAPH :g1 { :s :p :o } GRAPH :g2 { :s :p :o }";
    check(show(
        "subquery15",
        eval(
            sq15,
            "SELECT ?x WHERE { GRAPH ?g { SELECT ?x { VALUES ?x { 1 2 3 } } ORDER BY ?x OFFSET 1 LIMIT 1 } }",
        ),
        &[&format!("x={}", int(2)), &format!("x={}", int(2))],
    ));
    let sq16 = "@prefix : <http://example.com/> .
        GRAPH :g1 { :s :p :o . :s1 :p :o1, :o2, :o3 }
        GRAPH :g2 { :s :p :o . :s2 :p :o1, :o2 }
        GRAPH :g3 { :s :p :o }";
    let e = |s: &str, o: &str| format!("o=<http://example.com/{o}> s=<http://example.com/{s}>");
    check(show(
        "subquery16",
        eval(
            sq16,
            "PREFIX : <http://example.com/> SELECT ?s ?o WHERE { GRAPH ?g { SELECT ?s ?o { ?s :p ?o } ORDER BY ?s ?o OFFSET 1 LIMIT 2 } }",
        ),
        &[
            &e("s1", "o1"),
            &e("s1", "o2"),
            &e("s2", "o1"),
            &e("s2", "o2"),
        ],
    ));

    // bnode01: the expected result gives each solution its own blank nodes, so the four rows use
    // six distinct labels; BNODE(str) here is the same node across solutions.
    let data = r#"@prefix : <http://example.org/> . :s1 :str "foo" . :s3 :str "BAZ" ."#;
    let rows = eval(
        data,
        "PREFIX : <http://example.org/> SELECT ?b1 ?b2 WHERE { ?a :str ?s1 . ?b :str ?s2 . BIND(BNODE(?s1) AS ?b1) BIND(BNODE(?s2) AS ?b2) }",
    );
    let labels: std::collections::BTreeSet<String> = rows
        .iter()
        .flatten()
        .flat_map(|r| {
            r.split(' ')
                .filter_map(|c| c.split_once('=').map(|(_, t)| t.to_string()))
        })
        .filter(|t| t.starts_with("_:"))
        .collect();
    println!(
        "CONTROL\tbnode01\t{}\tdistinct blank nodes across 4 rows: {}\twant 6\t{rows:?}",
        if labels.len() == 6 {
            "MATCH"
        } else {
            "DIVERGES"
        },
        labels.len()
    );

    // Parse-level: the grammar accepts both, the dependency rejects them.
    for (name, query) in [
        (
            "values-construct-group-by",
            "PREFIX : <http://example.org/> CONSTRUCT { ?s :rank ?rank } WHERE { ?s ?p ?o } GROUP BY ?s VALUES (?s ?rank) { (:a 1) (:b 2) (:d 4) }",
        ),
        (
            "select-variable-reuse",
            "SELECT (COUNT(?v) AS ?count) (?count + 1 AS ?countPlusOne) WHERE { VALUES ?v { 0 1 2 3 } }",
        ),
    ] {
        let r = SparqlParser::new().parse_query(query);
        println!(
            "CONTROL\t{name}\t{}\tparse: {:?}\twant accepted",
            if r.is_ok() { "MATCH" } else { "DIVERGES" },
            r.as_ref().err().map(|e| e.to_string())
        );
    }

    // RDF/XML: the XMLLiteral is serialised as written, not in exclusive canonical form.
    let xml = br#"<?xml version="1.0"?>
<rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#" xmlns:eg="http://example.org/" xml:base="http://example.com/">
  <rdf:Description rdf:about="http://www.example.org/a">
    <eg:prop rdf:parseType="Literal"><br /></eg:prop>
  </rdf:Description>
</rdf:RDF>"#;
    for t in oxrdfxml::RdfXmlParser::new().for_slice(xml) {
        let t = t.expect("RDF/XML parses");
        println!(
            "CONTROL\txml-canon (inline)\t{}\tgot {}\twant \"<br></br>\"^^rdf:XMLLiteral",
            if t.object.to_string().contains("<br></br>") {
                "MATCH"
            } else {
                "DIVERGES"
            },
            t.object
        );
    }
    // The suite's four failing RDF/XML evaluation files: the rest of each graph is the expected
    // one, and only the XMLLiteral lexical form differs.
    let suites = std::env::var("W3C_SUITES_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| {
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../../../crates")
        })
        .join("rdf-tests/rdf");
    for (name, dir, stem) in [
        ("xml-canon-test001", "rdf11/rdf-xml/xml-canon", "test001"),
        ("xml-canon-test002", "rdf11/rdf-xml/xml-canon", "test002"),
        ("rdf12-xml-an-13", "rdf12/rdf-xml/eval", "rdf12-xml-an-13"),
        ("rdf12-xml-an-14", "rdf12/rdf-xml/eval", "rdf12-xml-an-14"),
    ] {
        let (rdf, nt) = (
            suites.join(dir).join(format!("{stem}.rdf")),
            suites.join(dir).join(format!("{stem}.nt")),
        );
        let (Ok(rdf_bytes), Ok(nt_bytes)) = (std::fs::read(&rdf), std::fs::read(&nt)) else {
            println!("CONTROL	{name}	UNCHECKED	suite files absent");
            continue;
        };
        let base = "http://example.com/";
        let got = crate::rdf::parse_rdf(crate::rdf::Fmt::Xml, &rdf_bytes, base);
        let want = crate::rdf::parse_rdf(crate::rdf::Fmt::NTriples, &nt_bytes, base);
        let (Ok(got), Ok(want)) = (got, want) else {
            println!("CONTROL	{name}	UNCHECKED	file unparseable");
            continue;
        };
        let literal_of = |d: &Dataset| {
            d.iter()
                .filter_map(|q| match q.object {
                    oxrdf::TermRef::Literal(l)
                        if l.datatype().as_str().ends_with("#XMLLiteral") =>
                    {
                        Some(l.value().to_string())
                    },
                    _ => None,
                })
                .collect::<Vec<_>>()
        };
        fn blank_term(t: oxrdf::Term) -> oxrdf::Term {
            match t {
                oxrdf::Term::Literal(l) if l.datatype().as_str().ends_with("#XMLLiteral") => {
                    oxrdf::Literal::new_typed_literal("", l.datatype().into_owned()).into()
                },
                oxrdf::Term::Triple(t) => {
                    let mut t = *t;
                    t.object = blank_term(t.object);
                    oxrdf::Term::Triple(Box::new(t))
                },
                other => other,
            }
        }
        let blank = |d: &Dataset| -> Dataset {
            d.iter()
                .map(|q| {
                    let mut q = q.into_owned();
                    q.object = blank_term(q.object);
                    q
                })
                .collect()
        };
        let same = crate::rdf::isomorphic(blank(&got), blank(&want));
        println!(
            "CONTROL	{name}	{}	got XMLLiteral {:?}	want {:?}	rest of graph equal: {same}",
            if crate::rdf::isomorphic(got.clone(), want.clone()) {
                "MATCH"
            } else {
                "DIVERGES"
            },
            literal_of(&got),
            literal_of(&want)
        );
    }
    let _ = (GraphName::DefaultGraph, diverges);
}
