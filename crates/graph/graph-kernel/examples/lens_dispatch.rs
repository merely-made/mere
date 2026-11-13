// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The hot-path probe (declared-lens L3, D9): axis-row lookup by dense id
//! against today's enum match, in one run, with the engine's headless frame
//! share (P5's rows) in the same run as the denominator.
//!
//! `cargo run --release -p mere-kernel --features query,fixtures --example lens_dispatch -- 10000 100000`
//! (`LENS_MUTATE=1` corrupts one pack row: the checksum assertion must fail.)
//!
//! Per edge, the engine matches on `RelationKind` today in
//! `built_in_relation_stratum` (admission, placement, migration) and in
//! `RelationKind::family` (selectors). Those are the two reads probed. The
//! other six observed axes have no enum path today; `pack_flat_all` reads them
//! too, as the cost of a full row. The headless frame (`is_current` plus a held
//! query) reads no edge's kind, so the sweep is an upper bound: a pass that
//! reads every edge's placement and family once per frame.
//!
//! Enum path: a `RelationKind` column, matched per edge, called across the
//! crate boundary as the kernel calls it. Pack path: the edge's relation
//! interned once, outside the timed loop (at load or projection), to a dense id
//! column; per frame, one indexed read of an axis row. `pack_loaded` reads the
//! loader's own `TermTable` entries; `pack_flat` reads a compact table of the
//! observed axes copied once at load; `pack_flat_call` is the same read behind
//! `#[inline(never)]`, the bound if the lookup crosses a crate boundary. The
//! negative controls are `black_box` reads of each column with no dispatch.

use std::hint::black_box;
use std::time::{Duration, Instant};

use eidetic::pack::{Origin, SignerReview, TrustPolicy, compile_authored};
use eidetic::schema::TrustEnvelope;
use identity::{Ed25519Keypair, Ed25519PublicKey};
use kernel::graph::fixtures::GraphFixtures;
use kernel::graph::{Graph, NodeKey, SemanticStatementSpec};
use mere::model::{
    ArrangementSubKind, ContainmentSubKind, EdgeFamily, GraphStratum, ImportedSubKind,
    ProvenanceSubKind, RelationKind, SemanticSubKind, built_in_relation_stratum,
};
use mere::profile::intern::{TermId, TermTable};
use mere::profile::load::{Limits, Profile, load};
use mere::profile::resolve::{Offered, PackRef, PackSource};
use mere::query::{IndexedDataset, View, sparql_on};
use strum::IntoEnumIterator;

const WORKSPACE: &str = include_str!("../../engine/src/profile/fixtures/workspace.toml");
const WORKSPACE_LENS: &str = include_str!("../../engine/src/profile/fixtures/workspace_lens.toml");
/// Traversal's IRI, minted in the stand-in only (D13); the test support's constant.
const TRAVERSAL_IRI: &str = "https://mere.computer/ns/test/workspace#traversal";
const FAMILY_NS: &str = "https://mere.computer/ns/family#";
const NEXT: &str = "https://example.test/vocab#next";
const POINT: &str = "SELECT ?o WHERE { <https://n.test/42> <https://example.test/vocab#next> ?o }";
const TWO_HOP: &str = "SELECT (COUNT(*) AS ?n) WHERE { ?a <https://example.test/vocab#next> ?b . ?b <https://example.test/vocab#next> ?c }";

struct Offers(Vec<Offered>);
impl PackSource for Offers {
    fn offer(&self, id: &str) -> Vec<Offered> {
        let same = |o: &&Offered| {
            eidetic::pack::PackHeader::from_canonical(&o.canonical).is_ok_and(|h| h.id == id)
        };
        self.0.iter().filter(same).cloned().collect()
    }
    fn handler_known(&self, _id: &str) -> bool {
        false
    }
}
struct NoReview;
impl SignerReview for NoReview {
    fn accepts(&self, _pack: &str, _signer: &Ed25519PublicKey) -> bool {
        false
    }
}

/// The core pack and the workspace stand-in, through the fixture lens (D13).
fn workspace_profile() -> Profile {
    let local = |text: &str| Offered {
        canonical: compile_authored(text).expect("fixture compiles").1,
        envelope: TrustEnvelope::self_asserted(),
        origin: Origin::Local,
    };
    let source = Offers(vec![local(WORKSPACE), local(WORKSPACE_LENS)]);
    let lens = PackRef::Select(eidetic::pack::Selection {
        pack: "https://mere.computer/packs/test/workspace-lens".to_string(),
        version: "0.1.0".to_string(),
    });
    let policy = TrustPolicy {
        release_key: Ed25519Keypair::from_seed([1; 32]).public_key(),
        review: NoReview,
    };
    load(&lens, &source, &policy, &Limits::default()).expect("the workspace fixture loads")
}

fn all_kinds() -> Vec<RelationKind> {
    SemanticSubKind::iter()
        .map(RelationKind::Semantic)
        .chain(ContainmentSubKind::iter().map(RelationKind::Containment))
        .chain(ArrangementSubKind::iter().map(RelationKind::Arrangement))
        .chain(ImportedSubKind::iter().map(RelationKind::Imported))
        .chain(ProvenanceSubKind::iter().map(RelationKind::Provenance))
        .chain([RelationKind::Traversal, RelationKind::OpenPredicate])
        .collect()
}

/// The observed axes of one term, copied once at load into one dense row.
#[derive(Clone, Copy, PartialEq, Debug)]
struct Flat {
    stratum: u8,
    family: u8,
    rest: [u8; 7],
}

fn family_of(iri: Option<&str>) -> EdgeFamily {
    match iri.and_then(|i| i.strip_prefix(FAMILY_NS)) {
        Some("traversal") => EdgeFamily::Traversal,
        Some("containment") => EdgeFamily::Containment,
        Some("arrangement") => EdgeFamily::Arrangement,
        Some("imported") => EdgeFamily::Imported,
        Some("provenance") => EdgeFamily::Provenance,
        _ => EdgeFamily::Semantic,
    }
}

/// What load or projection precomputes: the dense id of each kind (an open
/// predicate takes the extra slot after the last term, the core defaults), and
/// the flat axis table over those ids.
struct Pack {
    terms: TermTable,
    id_of: Vec<(RelationKind, u32)>,
    flat: Vec<Flat>,
    open_stratum: GraphStratum,
}

impl Pack {
    fn new(profile: Profile) -> Self {
        let terms = profile.terms;
        let open = terms.len() as u32;
        let iri = |k: RelationKind| match k {
            RelationKind::Traversal => Some(TRAVERSAL_IRI),
            RelationKind::OpenPredicate => None,
            other => other.predicate_iri(),
        };
        let id_of = all_kinds()
            .into_iter()
            .map(|k| (k, iri(k).map_or(open, |i| terms.id(i).expect("a pack row").0)))
            .collect();
        let mut flat: Vec<Flat> = terms
            .iter()
            .map(|(_, e)| Flat {
                stratum: e.row.stratum as u8,
                family: family_of(e.family.as_deref()) as u8,
                rest: [
                    e.row.durability as u8,
                    e.row.retractability as u8,
                    e.row.projectability as u8,
                    e.row.force as u8,
                    e.row.symmetry as u8,
                    e.row.presentation as u8,
                    e.row.derivation as u8,
                ],
            })
            .collect();
        let d = terms.row("https://foreign.test/default");
        flat.push(Flat {
            stratum: d.stratum as u8,
            family: EdgeFamily::Semantic as u8,
            rest: [
                d.durability as u8,
                d.retractability as u8,
                d.projectability as u8,
                d.force as u8,
                d.symmetry as u8,
                d.presentation as u8,
                d.derivation as u8,
            ],
        });
        Self {
            open_stratum: d.stratum,
            terms,
            id_of,
            flat,
        }
    }

    fn id(&self, kind: RelationKind) -> u32 {
        self.id_of.iter().find(|(k, _)| *k == kind).expect("every kind").1
    }
}

/// The enum path's values for one kind: what the engine reads today.
fn enum_read(kind: RelationKind) -> (u8, u8) {
    (built_in_relation_stratum(kind) as u8, kind.family() as u8)
}

fn pack_flat_read(flat: &[Flat], id: u32) -> (u8, u8) {
    let row = flat[id as usize];
    (row.stratum, row.family)
}

/// Every kind: both paths agree on stratum and family, through the loaded
/// table, the flat table and the IRI lookup. `corrupt` flips one flat row.
fn check_equal(pack: &Pack, corrupt: bool) -> Result<(), String> {
    for kind in all_kinds() {
        let id = pack.id(kind);
        let (stratum, family) = enum_read(kind);
        let mut row = pack.flat[id as usize];
        if corrupt && kind == RelationKind::Semantic(SemanticSubKind::Hyperlink) {
            row.stratum ^= 1;
        }
        let loaded = pack.terms.get(TermId(id)).map_or(pack.open_stratum, |e| e.row.stratum);
        if (row.stratum, row.family) != (stratum, family) || loaded as u8 != stratum {
            return Err(format!("{kind:?}: enum ({stratum},{family}) pack {row:?} loaded {loaded:?}"));
        }
    }
    Ok(())
}

fn enum_stratum(col: &[RelationKind]) -> u64 {
    col.iter().map(|&k| built_in_relation_stratum(k) as u64).sum()
}
fn enum_both(col: &[RelationKind]) -> u64 {
    col.iter().map(|&k| u64::from(enum_read(k).0) | u64::from(enum_read(k).1) << 1).sum()
}
fn pack_loaded(terms: &TermTable, open: GraphStratum, ids: &[u32]) -> u64 {
    let read = |&i: &u32| terms.get(TermId(i)).map_or(open, |e| e.row.stratum) as u64;
    ids.iter().map(read).sum()
}
fn pack_flat(flat: &[Flat], ids: &[u32]) -> u64 {
    ids.iter().map(|&i| u64::from(flat[i as usize].stratum)).sum()
}
#[inline(never)]
fn flat_stratum_at(flat: &[Flat], id: u32) -> u8 {
    flat[id as usize].stratum
}
fn pack_flat_call(flat: &[Flat], ids: &[u32]) -> u64 {
    ids.iter().map(|&i| u64::from(flat_stratum_at(flat, i))).sum()
}
fn pack_flat_both(flat: &[Flat], ids: &[u32]) -> u64 {
    let both = |&i: &u32| {
        let (s, f) = pack_flat_read(flat, i);
        u64::from(s) | u64::from(f) << 1
    };
    ids.iter().map(both).sum()
}
fn pack_flat_all(flat: &[Flat], ids: &[u32]) -> u64 {
    let all = |&i: &u32| {
        let r = flat[i as usize];
        r.rest.iter().fold(u64::from(r.stratum) | u64::from(r.family) << 1, |a, &v| a + u64::from(v))
    };
    ids.iter().map(all).sum()
}
/// The negative control: a `black_box` read of every element, no dispatch.
fn empty<T: Copy>(col: &[T]) -> u64 {
    let mut n = 0u64;
    for x in col {
        black_box(*x);
        n += 1;
    }
    n
}

/// A deterministic mixed column over all kinds (xorshift), or one kind.
fn column(pack: &Pack, n: usize, mixed: bool) -> (Vec<RelationKind>, Vec<u32>) {
    let kinds = all_kinds();
    let mut x = 0x9e37_79b9_7f4a_7c15u64;
    let pick = |x: &mut u64| {
        *x ^= *x << 13;
        *x ^= *x >> 7;
        *x ^= *x << 17;
        if mixed { kinds[(*x % kinds.len() as u64) as usize] } else { kinds[3] }
    };
    let col: Vec<_> = (0..n).map(|_| pick(&mut x)).collect();
    let ids = col.iter().map(|&k| pack.id(k)).collect();
    (col, ids)
}

fn row(scale: usize, name: &str, mut s: Vec<Duration>) {
    s.sort();
    let at = |i: usize| format!("{:.5}", s[i].as_secs_f64() * 1000.0);
    println!("{scale}\t{name}\t{}\t{}\t{}\t{}", s.len(), at(s.len() / 2), at(0), at(s.len() - 1));
}

fn dispatch_rows(scale: usize, pack: &Pack, corrupt: bool) {
    let reps = if scale >= 100_000 { 101 } else { 301 };
    for mixed in [true, false] {
        let tag = if mixed { "mixed" } else { "uniform" };
        let (col, ids) = column(pack, scale, mixed);
        let mut flat = pack.flat.clone();
        if corrupt {
            // The mutation: the pack path returns a wrong value for one kind.
            let id = pack.id(RelationKind::Semantic(SemanticSubKind::Hyperlink));
            flat[id as usize].stratum ^= 1;
        }
        let (loaded_open, flat) = (pack.open_stratum, &flat[..]);
        type Run<'a> = Box<dyn Fn() -> u64 + 'a>;
        let c = &col[..];
        let i = &ids[..];
        let variants: Vec<(&str, Run, Option<usize>)> = vec![
            ("empty_kinds", Box::new(move || empty(c)), None),
            ("empty_ids", Box::new(move || empty(i)), None),
            ("enum_stratum", Box::new(move || enum_stratum(black_box(c))), Some(0)),
            ("pack_loaded_stratum", Box::new(move || pack_loaded(&pack.terms, loaded_open, black_box(i))), Some(0)),
            ("pack_flat_stratum", Box::new(move || pack_flat(flat, black_box(i))), Some(0)),
            ("pack_flat_call_stratum", Box::new(move || pack_flat_call(flat, black_box(i))), Some(0)),
            ("enum_stratum_family", Box::new(move || enum_both(black_box(c))), Some(1)),
            ("pack_flat_stratum_family", Box::new(move || pack_flat_both(flat, black_box(i))), Some(1)),
            ("pack_flat_all_axes", Box::new(move || pack_flat_all(flat, black_box(i))), None),
        ];
        let mut times = vec![Vec::new(); variants.len()];
        let mut sums = vec![0u64; variants.len()];
        for _ in 0..reps {
            for (n, (_, run, _)) in variants.iter().enumerate() {
                let t = Instant::now();
                sums[n] = black_box(run());
                times[n].push(t.elapsed());
            }
        }
        // Correctness: same checksum within each group (the enum path is the control).
        for group in 0..2 {
            let members: Vec<_> = variants.iter().enumerate().filter(|(_, v)| v.2 == Some(group)).collect();
            let want = sums[members[0].0];
            for (n, v) in &members {
                assert_eq!(sums[*n], want, "{tag} {}: checksum differs from the enum path", v.0);
            }
        }
        for (n, (name, _, _)) in variants.iter().enumerate() {
            row(scale, &format!("dispatch_{tag}_{name}"), std::mem::take(&mut times[n]));
        }
    }
}

fn build(n: usize) -> (Graph, Vec<NodeKey>) {
    let mut graph = Graph::new();
    let keys: Vec<_> = (0..n)
        .map(|i| graph.add_node(format!("https://n.test/{i}"), Default::default()))
        .collect();
    for (i, key) in keys.iter().enumerate() {
        let spec = SemanticStatementSpec { predicate: NEXT.to_string(), ..Default::default() };
        graph.assert_semantic_statement(*key, keys[(i + 1) % n], spec);
    }
    (graph, keys)
}

fn query_rows(indexed: &IndexedDataset, graph: &Graph, query: &str) -> usize {
    sparql_on(indexed, query, View::Live, graph.coverage_note()).expect("held query").rows.len()
}

/// P5's headless frame rows (adapter_scale's `frame_*`), the denominators.
fn frame_rows(scale: usize, graph: &mut Graph, keys: &[NodeKey]) {
    let mut indexed = IndexedDataset::for_graph(graph);
    let rebuilds = if scale >= 100_000 { 7 } else { 21 };
    for (name, query) in [("point", POINT), ("two_hop", TWO_HOP)] {
        let mut steady = vec![];
        for _ in 0..51 {
            let t = Instant::now();
            assert!(indexed.is_current(graph));
            let rows = query_rows(&indexed, graph, query);
            steady.push(t.elapsed());
            assert_eq!(rows, 1, "held {name} row count");
        }
        row(scale, &format!("frame_steady_{name}"), steady);
        let mut edited = vec![];
        for k in 0..rebuilds {
            let spec = SemanticStatementSpec {
                predicate: format!("https://example.test/vocab#frame{name}{k}"),
                ..Default::default()
            };
            graph.assert_semantic_statement(keys[k], keys[(k + 7) % keys.len()], spec);
            let t = Instant::now();
            if !indexed.is_current(graph) {
                indexed = IndexedDataset::for_graph(graph);
            }
            let rows = query_rows(&indexed, graph, query);
            edited.push(t.elapsed());
            assert_eq!(rows, 1, "held {name} row count after edit");
        }
        row(scale, &format!("frame_after_edit_{name}"), edited);
    }
}

fn main() {
    let scales: Vec<usize> = std::env::args().skip(1).filter_map(|a| a.parse().ok()).collect();
    let scales = if scales.is_empty() { vec![10_000, 100_000] } else { scales };
    let corrupt = std::env::var_os("LENS_MUTATE").is_some();
    let pack = Pack::new(workspace_profile());
    check_equal(&pack, false).expect("the enum and pack paths agree for every kind");
    if corrupt {
        let err = check_equal(&pack, true).expect_err("a corrupted row is caught");
        println!("# LENS_MUTATE: the equality check caught it: {err}");
    }
    println!("scale\tmeasure\treps\tmedian_ms\tmin_ms\tmax_ms\t# lens_dispatch");
    for scale in scales {
        let (mut graph, keys) = build(scale);
        dispatch_rows(scale, &pack, corrupt);
        frame_rows(scale, &mut graph, &keys);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enum_and_pack_agree_for_every_relation_kind() {
        let pack = Pack::new(workspace_profile());
        assert_eq!(all_kinds().len(), 46);
        check_equal(&pack, false).unwrap();
        for kind in all_kinds() {
            assert_eq!(enum_read(kind), pack_flat_read(&pack.flat, pack.id(kind)), "{kind:?}");
        }
    }

    #[test]
    fn a_wrong_pack_row_fails_the_equality_check_and_the_checksum() {
        let pack = Pack::new(workspace_profile());
        assert!(check_equal(&pack, true).is_err());
        let (col, ids) = column(&pack, 4096, true);
        let mut flat = pack.flat.clone();
        assert_eq!(pack_flat(&flat, &ids), enum_stratum(&col));
        flat[pack.id(RelationKind::Semantic(SemanticSubKind::Hyperlink)) as usize].stratum ^= 1;
        assert_ne!(pack_flat(&flat, &ids), enum_stratum(&col));
    }
}
