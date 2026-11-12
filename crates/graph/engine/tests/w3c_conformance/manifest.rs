// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Reading W3C test manifests (Turtle) into a flat list of entries.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use oxrdf::Term;
use oxttl::TurtleParser;

pub(crate) const MF: &str = "http://www.w3.org/2001/sw/DataAccess/tests/test-manifest#";
pub(crate) const RDF: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#";
pub(crate) const RDFT: &str = "http://www.w3.org/ns/rdftest#";
pub(crate) const TEST_BASE: &str = "https://w3c.github.io/rdf-tests/";

pub(crate) struct Entry {
    pub(crate) id: String,
    pub(crate) kinds: Vec<String>,
    pub(crate) action: Option<PathBuf>,
    pub(crate) result: Option<PathBuf>,
    pub(crate) rejected: bool,
    pub(crate) manifest: PathBuf,
    /// Files named by a blank-node action (`qt:query`, `qt:data`, `qt:graphData`,
    /// `ut:request`, `ut:data`), by predicate local name.
    pub(crate) parts: HashMap<String, Vec<PathBuf>>,
}

pub(crate) fn file_url(path: &Path) -> String {
    format!("file:///{}", path.to_string_lossy().replace('\\', "/"))
}

pub(crate) fn url_path(url: &str) -> PathBuf {
    PathBuf::from(url.strip_prefix("file:///").unwrap_or(url))
}

pub(crate) fn local(iri: &str) -> String {
    iri.rsplit(['#', '/']).next().unwrap_or(iri).to_string()
}

pub(crate) struct Index {
    by_subject: HashMap<String, Vec<(String, Term)>>,
}

impl Index {
    pub(crate) fn load(path: &Path) -> Result<Self, String> {
        let bytes = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
        let parser = TurtleParser::new()
            .with_base_iri(file_url(path))
            .map_err(|e| e.to_string())?;
        let mut by_subject: HashMap<String, Vec<(String, Term)>> = HashMap::new();
        for triple in parser.for_slice(&bytes) {
            let t = triple.map_err(|e| format!("{}: {e}", path.display()))?;
            by_subject
                .entry(t.subject.to_string())
                .or_default()
                .push((t.predicate.as_str().to_string(), t.object));
        }
        Ok(Self { by_subject })
    }

    pub(crate) fn objects(&self, subject: &str, predicate: &str) -> Vec<Term> {
        self.by_subject
            .get(subject)
            .map(|v| {
                v.iter()
                    .filter(|(p, _)| p == predicate)
                    .map(|(_, o)| o.clone())
                    .collect()
            })
            .unwrap_or_default()
    }

    pub(crate) fn list(&self, head: &Term) -> Vec<Term> {
        let nil = format!("<{RDF}nil>");
        let mut out = Vec::new();
        let mut cur = head.to_string();
        while cur != nil {
            let first = self.objects(&cur, &format!("{RDF}first"));
            let rest = self.objects(&cur, &format!("{RDF}rest"));
            let (Some(first), Some(rest)) = (first.into_iter().next(), rest.into_iter().next())
            else {
                break;
            };
            out.push(first);
            cur = rest.to_string();
        }
        out
    }
}

pub(crate) fn iri_path(t: &Term) -> Option<PathBuf> {
    match t {
        Term::NamedNode(n) => Some(url_path(n.as_str())),
        _ => None,
    }
}

/// Every entry reachable from `manifest` through `mf:include`, depth first.
pub(crate) fn walk(manifest: &Path, out: &mut Vec<Entry>) -> Result<(), String> {
    let index = Index::load(manifest)?;
    let mut subjects: Vec<&String> = index.by_subject.keys().collect();
    subjects.sort();
    for subject in subjects {
        for head in index.objects(subject, &format!("{MF}include")) {
            for item in index.list(&head) {
                if let Some(path) = iri_path(&item) {
                    walk(&path, out)?;
                }
            }
        }
        for head in index.objects(subject, &format!("{MF}entries")) {
            for item in index.list(&head) {
                let key = item.to_string();
                let kinds = index
                    .objects(&key, &format!("{RDF}type"))
                    .iter()
                    .filter_map(|t| match t {
                        Term::NamedNode(n) => Some(local(n.as_str())),
                        _ => None,
                    })
                    .collect();
                let action = index
                    .objects(&key, &format!("{MF}action"))
                    .first()
                    .and_then(iri_path);
                let result = index
                    .objects(&key, &format!("{MF}result"))
                    .first()
                    .and_then(iri_path);
                let rejected = index
                    .objects(&key, &format!("{RDFT}approval"))
                    .iter()
                    .any(|t| matches!(t, Term::NamedNode(n) if local(n.as_str()) == "Rejected"));
                let mut parts: HashMap<String, Vec<PathBuf>> = HashMap::new();
                if let Some(a) = index.objects(&key, &format!("{MF}action")).first() {
                    for (p, o) in index.by_subject.get(&a.to_string()).into_iter().flatten() {
                        if let Some(path) = iri_path(o) {
                            parts.entry(local(p)).or_default().push(path);
                        }
                    }
                }
                out.push(Entry {
                    parts,
                    id: key,
                    kinds,
                    action,
                    result,
                    rejected,
                    manifest: manifest.to_path_buf(),
                });
            }
        }
    }
    Ok(())
}

pub(crate) fn rel(root: &Path, path: &Path) -> String {
    let r = root.to_string_lossy().replace('\\', "/");
    let p = path.to_string_lossy().replace('\\', "/");
    p.strip_prefix(&format!("{r}/")).unwrap_or(&p).to_string()
}

pub(crate) fn test_base(root: &Path, action: &Path) -> String {
    format!("{TEST_BASE}{}", rel(root, action))
}
