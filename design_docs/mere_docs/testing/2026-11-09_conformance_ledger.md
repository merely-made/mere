# Conformance ledger

Opened 2026-11-09 by P0 of the [engine split plan](../implementation_strategy/2026-11-07_engine_split_plan.md).
Rule (plan, Constraints, "Standards follow the suites"): engine capabilities ship behind a ledger
row (spec revision or pack version, supported profile, fixture suite, result), and no capability
is claimed without its row. Rows are appended or revised when a suite is rerun; a rerun states its
date and the adapter commit. Run evidence (commands, per-manifest counts, failing test names) is in
the [receipts](../../archive_docs/receipts/2026-11-07_engine_split_plan_receipts.md), not here.

## How a result is scored

- **Harness**: `crates/graph/engine/tests/w3c_conformance/` (`main.rs` plus five modules) (package `mere`, lib
  `mere`; moved from `crates/graph/linked-data` by engine split P5 L2, 2026-11-12). Ignored by default; skips cleanly if the suites are absent. Command:
  `W3C_SUITES_DIR=<Code/crates> cargo test -p mere --features query --test w3c_conformance -- --ignored --nocapture`
  (`W3C_VERBOSE=1` lists failing tests). Without `--features query` the same command runs the RDF and JSON-LD
  rows at every layer, engine included, and skips the SPARQL rows and the evaluation controls (P6 L1, 2026-11-12).
- **Layers.** `parser` runs the parsers the adapter wraps (`oxttl`, `oxrdfxml`, `oxjsonld`, `spargebra`, `spareval`, `sparesults`)
  with the adapter's features and options, and scores what the suite specifies: accept or reject
  for syntax tests, isomorphism of the produced dataset against the expected one for evaluation
  and toRdf tests. `adapter` runs the crate's own public entry points (`from_nquads`, `from_trig`,
  `from_jsonld_with_contexts_and_base_iri`) and can score accept or reject only: they return a
  graph contribution, not an RDF dataset, and take no base IRI or processing mode.
- **The `engine` layer** (from 2026-11-12, engine split P5 L4, F8 and F9; RDF and JSON-LD rows need no `query` since P6 L1, SPARQL rows do) runs the
  engine's dataset entry points, `mere::parse_turtle`, `parse_ntriples`, `parse_nquads`, `parse_trig`,
  `parse_rdfxml` and `parse_jsonld`, each with the test's base IRI (JSON-LD through the offline
  `ContextCache` of the suite's contexts), and reads the returned `IndexedDataset` back through
  `IndexedDataset::quads(View::Live)` (P6 L1; before it, through the query seam, with identical rows). Syntax tests score accept or reject; evaluation and toRdf tests score isomorphism of that dataset
  against the expected one. SPARQL query evaluation loads every data file (manifest data, graph data and
  FROM graphs) through those entry points, merges the quads into one `IndexedDataset` and evaluates with
  `spareval` over it, scored as the parser layer is. SPARQL syntax and update are not scored here: the
  engine has no syntax-only or update entry point, so those stay the parser layer's. **The engine rows are
  the declared profile** (section below); the parser layer remains the control for them.
- **The `indexed` layer** (SPARQL query evaluation only, from 2026-11-10, quad index plan P2) loads each case's data
  through `mere::query::IndexedDataset::from_quads` and evaluates with the same `spareval`, scored as the
  parser layer is; it is the engine's own query dataset, so a difference from the parser layer is the dataset's.
- **Counts** are `pass / run`, where run excludes skipped tests. A skip is a test the layer cannot
  express (reason given); it is not a pass. Isomorphism uses `oxrdf`'s dataset canonicalisation,
  as an equality check only; it is not a claim on the RDFC-1.0 row.
- Parser, adapter and indexed stay beside the engine layer (F8). D61's provisional two-layer scoring ended
  when the engine API existed (P5 L4, 2026-11-12).
- **A fail is "engine" only with a control**: the test `engine_controls` in the harness loads the case's data by hand (inline TriG, or the suite file for RDF/XML) and runs `spareval`, `spargebra` or `oxrdfxml` directly, printing the result beside the expected one; a positive control (a plain pattern, and FROM NAMED over a plain GRAPH pattern) proves the instrument answers correctly. Fails with no control built are unconfirmed: the SPARQL and RDF/XML syntax fails, `ntriples-version-1`, and the JSON-LD toRdf fails. An engine-layer fail is confirmed upstream when the parser layer fails the same test, because the parser layer calls the dependency with no engine code in between; every engine fail below is such a case (the fail lists were diffed, 2026-11-12), and the controls above, or their absence, carry over.

## Starting point

Adapter commit: Mere `31dc9107d` (`linked-data` last changed in `60622438`, 2026-11-04). Parsers
as locked: `oxttl` 0.2.5, `oxjsonld` 0.2.7, `oxrdf` 0.3.5, `spargebra` 0.4.8, `spareval` 0.2.9
(the last two behind the `query` feature). Run 2026-11-09. Suites under `Code/crates`, plain
clones of `origin`:

| Suite | Revision (HEAD) | Commit date |
|---|---|---|
| `rdf-tests` (RDF 1.1, RDF 1.2, SPARQL 1.0, 1.1, 1.2) | `202a74eb81f4` | 2026-11-04 |
| `json-ld-api` | `55514738d854` | 2026-10-07 |
| `data-shapes` | `9c102cbb5309` | 2026-11-08 |
| `rdf-canon` | `15619df2fda7` | 2026-02-24 |

## Rows

"Spec revision" is the spec as the suite's directory encodes it; the suites carry no dated spec
snapshot, so none is recorded (a fork in the lane report).

| Spec revision or pack version | Supported profile | Fixture suite | Result |
|---|---|---|---|
| RDF 1.2 (`rdf12`) | N-Triples syntax | `rdf-tests` `rdf/rdf12/rdf-n-triples/syntax` | parser 33/34 (fails `ntriples-version-1`: `VERSION` directive rejected); adapter: no N-Triples entry point; engine `parse_ntriples` 33/34, the same fail |
| RDF 1.2 | N-Quads syntax | `rdf-tests` `rdf/rdf12/rdf-n-quads/syntax` | parser 31/31; adapter `from_nquads` 31/31; engine `parse_nquads` 31/31 |
| RDF 1.2 | Turtle syntax | `rdf-tests` `rdf/rdf12/rdf-turtle/syntax` | parser 77/77; adapter: no Turtle entry point; engine `parse_turtle` 77/77 |
| RDF 1.2 | Turtle evaluation | `rdf-tests` `rdf/rdf12/rdf-turtle/eval` | parser 32/32; adapter not scorable (no dataset out); engine `parse_turtle` 32/32 |
| RDF 1.2 | TriG syntax | `rdf-tests` `rdf/rdf12/rdf-trig/syntax` | parser 38/38; adapter `from_trig` 38/38; engine `parse_trig` 38/38 |
| RDF 1.2 | TriG evaluation | `rdf-tests` `rdf/rdf12/rdf-trig/eval` | parser 26/26; adapter not scorable; engine `parse_trig` 26/26 |
| RDF 1.2 | RDF/XML syntax and evaluation | `rdf-tests` `rdf/rdf12/rdf-xml` | rerun 2026-11-14 (Mere `0d7e587ca`, fork `mark-ik/oxigraph` `d3ba9ecc`; `oxrdfxml` 0.2.5 from the fork): parser (`oxrdfxml`) syntax 2/2, evaluation 29/29; adapter: no RDF/XML entry point; engine `parse_rdfxml` syntax 2/2, evaluation 29/29. Before (2026-11-09, crates.io `oxrdfxml` 0.2.5): evaluation 27/29, fails `rdf12-xml-an-13` and `-14` (`rdf:parseType="Literal"` not written in exclusive canonical form), fixed by the fork's RDF/XML literal patch (D155) |
| RDF 1.2 | N-Triples and N-Quads canonical form | `rdf-tests` `.../c14n` | not run: no canonical serialisation entry point (82 tests) |
| RDF 1.2 | Semantics (entailment) | `rdf-tests` `rdf/rdf12/rdf-semantics` | not run: no engine stage yet (29 tests) |
| RDF 1.1 baseline | N-Triples syntax | `rdf-tests` `rdf/rdf11/rdf-n-triples` | parser 70/70; adapter: no entry point; engine `parse_ntriples` 70/70 |
| RDF 1.1 baseline | N-Quads syntax | `rdf-tests` `rdf/rdf11/rdf-n-quads` | parser 87/87; adapter 87/87; engine `parse_nquads` 87/87 |
| RDF 1.1 baseline | Turtle syntax and evaluation | `rdf-tests` `rdf/rdf11/rdf-turtle` | parser syntax 168/168, evaluation 145/145; adapter: no entry point; engine `parse_turtle` syntax 168/168, evaluation 145/145 |
| RDF 1.1 baseline | TriG syntax and evaluation | `rdf-tests` `rdf/rdf11/rdf-trig` | parser syntax 214/214, evaluation 143/143; adapter `from_trig` syntax 199/214 (15 fail: relative IRIs, `from_trig` takes no base IRI); engine `parse_trig` (takes the base IRI) syntax 214/214, evaluation 143/143 |
| RDF 1.1 baseline | RDF/XML syntax and evaluation | `rdf-tests` `rdf/rdf11/rdf-xml` | rerun 2026-11-14 (Mere `0d7e587ca`, fork `mark-ik/oxigraph` `d3ba9ecc`): parser syntax 40/40, evaluation 126/126; adapter: no entry point; engine `parse_rdfxml` syntax 40/40, evaluation 126/126. Before: evaluation 124/126, fails `xml-canon-test001` and `-002`, same cause, fixed by the fork (D155) |
| RDF 1.1 baseline | Semantics (entailment) | `rdf-tests` `rdf/rdf11/rdf-mt` | not run: no entailment stage yet (48 tests) |
| SPARQL 1.1 | Query syntax | `rdf-tests` `sparql/sparql11` (`manifest-sparql11-query.ttl`) | parser 106/106 (with the test's base IRI); adapter `query::sparql` parses with the same call and no base IRI, with no separate syntax entry point; engine: not scored (no syntax-only entry point) |
| SPARQL 1.1 | Update syntax | `rdf-tests` `sparql/sparql11` (`manifest-sparql11-update.ttl`) | parser 63/63; adapter: no update entry point; engine: no update entry point |
| SPARQL 1.1 | Query evaluation | same | parser (`spareval` 0.2.9 over an `oxrdf::Dataset`, results read with `sparesults`) 230/244; 3 skipped (result file in a format the harness does not read); adapter: not scorable (its `query::sparql` evaluates SELECT and ASK over a kernel `Graph`); indexed (2026-11-10, `IndexedDataset::from_quads`) 230/244, 3 skipped, the same 14 fails; engine (2026-11-12, data read by `parse_*`, evaluated over the merged dataset) 230/244, 3 skipped, the same 14 fails. Row order is not checked; numeric results compare by value. All 14 fails are engine, each with a control that loads its data by hand and runs `spareval` or `spargebra` directly (`engine_controls`): `agg-groupconcat-04`, `-06` (ASK over GROUP_CONCAT of language-tagged strings answers false, expected true); `graph-from-named-1`, `-2`, `-3` (FROM NAMED is not applied to GRAPH over VALUES: VALUES leaves `?g` unbound, graphs outside FROM NAMED are visited); `graph-minus` (outer GRAPH variable treated as shared with MINUS: no rows); `zero_or_more_set_start`, `zero_or_more_set_end`, `zero_or_one_set_start`, `zero_or_one_set_end` (zero-length path with a constant end over an empty dataset gives no row); `subquery15`, `subquery16` (subquery with OFFSET and LIMIT inside GRAPH is evaluated once, not per graph); `bnode01` (BNODE(str) returns the same node across solutions, 2 distinct against 6 expected); `values-construct-group-by` (`spargebra` rejects a query with `VALUES` after `GROUP BY`) |
| SPARQL 1.1 | Update evaluation | same | not run: no update stage (94 tests; D65) |
| SPARQL 1.2 | Syntax | `rdf-tests` `sparql/sparql12` | parser 200/203 (accepts `nested-aggregate-functions`, `tripleterm-subject-03`, `tripleterm-subject-06`, which the suite says to reject); engine: not scored |
| SPARQL 1.2 | Query evaluation | `rdf-tests` `sparql/sparql12` | rerun 2026-11-14 (Mere `0d7e587ca`, fork `mark-ik/oxigraph` `d3ba9ecc`): parser 65/65; adapter: not scorable; indexed 65/65; engine 65/65. Before (2026-11-09 to 2026-11-12): 64/65, the one fail `select-variable-reuse` (`spargebra` with `sparql-12` rejected reuse of a variable bound earlier in the same SELECT), fixed by the fork's SPARQL fixes (D162) |
| SPARQL 1.2 | Update evaluation | `rdf-tests` `sparql/sparql12` | not run: no update stage (3 tests; D65) |
| JSON-LD 1.1 | toRdf (JSON-LD to RDF) | `json-ld-api` `tests/toRdf-manifest.jsonld` (467 tests) | rerun 2026-11-14 (Mere `0d7e587ca`, fork `mark-ik/oxigraph` `d3ba9ecc`; `oxjsonld` 0.2.7 from the fork): parser 443/449 (6 fail: `tc013`, `tdi02`, `tdi04`, `tdi05`, `tdi06` are dataset mismatches, `tli12` rejects its base); 18 skipped (11 are 1.0-only, 4 `rdfDirection`, 1 `expandContext`, 1 expected output unreadable, 1 rejected only for an out-of-suite context). Adapter `from_jsonld_with_contexts_and_base_iri` accept or reject 441/442 (fails `tli12`); 25 skipped (those 18 less 1 plus 8 `processingMode` 1.0 tests). Engine `parse_jsonld` (dataset isomorphism, base IRI, offline cache and `processingMode`; P6 L1 F-R2) 2026-11-14 443/449, the same 6 fails as the parser layer; 18 skipped, the same as the parser layer's. Before (2026-11-12): 439/449 with 10 fails, the six above and `t0122`, `t0123`, `te062`, `te091`, fixed by the fork; before F-R2: 432/442 and 25, the 8 `processingMode` 1.0 tests skipped. The adapter is unchanged (441/442, 25 skipped, fails `tli12`) |
| JSON-LD 1.1 | Expand, compact, flatten, fromRdf, HTML, remote-doc | `json-ld-api` `tests/*-manifest.jsonld` | not run: no such API in the adapter or `oxjsonld` (386 + 246 + 58 + 54 + 50 + 18 tests) |
| SHACL (Core, SPARQL) | none yet | `data-shapes` | not run: no engine stage yet (P6) |
| RDFC-1.0 | Canonical N-Quads, issued identifiers, poison refusal; SHA-256 and SHA-384; RDF 1.2 triple terms refused (RDFC-1.0 defines none) | `rdf-canon` `tests/manifest.ttl` (86 tests: 64 eval, 21 map, 1 negative) | engine `mere::canonicalize_quads` (2026-11-13, carried `mere-rdfc`) eval 64/64, map 21/21, negative 1/1; control (raw `oxrdf` 0.3.5 `rdfc-10`, no budget) eval 64/64, map 21/21, negative 0/1: it does not return on `test074c`. The engine refuses `test074c` after 4,000 Hash N-Degree Quads calls (0.4 s release, 2.2 s debug); the positive tests use at most 468 calls and 2,880 permutations. Why a carried copy: [comparison](#rdfc-10-the-carried-implementation) |
| `mere.pack/2` (Eidetic `41ec454`); core pack 0.1.0 | Typed vocabulary and lens bodies; the loader's resolution (caret lower bounds, highest match, the core pack first), trust admission and interning ([declared-lens plan](../implementation_strategy/2026-11-12_declared_lens_plan.md) L2, D13) | No published suite: own fixtures, with Cargo's resolver, OCI manifests, TUF and `owl:imports` compared (plan §3.5); `mere` `profile::tests` and the placement test | 2026-11-13: one refusal per resolution rule, each beside a positive control: a cycle; two versions of one id; an unsatisfied requirement (three bounds and a missing id); an unsettled overlap; `Broken` at both origins; unsigned and unaccepted signers across trust. Two golden selection records (BLAKE3 of the record's canonical CBOR) and a pinned reopen; order independence over 256 shuffles of the offers (proptest). Pack-derived strata equal `built_in_relation_stratum` for all 44 sub-kinds, Traversal and an open predicate, with the core pack plus a test-only workspace stand-in; activation: 42 engine IRIs in the core pack, each with a generated constant, and 4 workspace IRIs in the stand-in only. Codec (Eidetic): body byte identity and set order, 16 body refusals, 10 authored-form refusals, CDDL with 3 negative controls, a row-order proptest |

## The declared profile (engine layer, 2026-11-12; rerun 2026-11-14)

Engine split P5 L4, Mere `65f27a4a4` (entry points) and `d3ed0ece8` (harness layer). `mere` 0.1.0,
`oxttl` 0.2.5, `oxrdfxml` 0.2.5, `oxjsonld` 0.2.7, `spargebra` 0.4.8, `spareval` 0.2.9, suites at the
revisions above. Counts are `pass / fail / skip` from the run's `ROW` lines. The table shows the rerun of 2026-11-14 (Mere `0d7e587ca`, fork `mark-ik/oxigraph` `d3ba9ecc`, the Oxigraph crates through `[patch.crates-io]`, D155, D159, D162; same suite revisions); the rows that moved are RDF 1.2 evaluation (85 to 87 pass), RDF 1.1 evaluation (412 to 414), SPARQL 1.2 query evaluation (64 to 65) and JSON-LD toRdf (439 to 443); every other row is as first measured. D142 sets what is engine: RDF/XML
is parse only; Turtle, TriG, N-Triples, N-Quads and JSON-LD are engine; SPARQL Update stays not run (F7, D65).

| Suite and category | Engine | Parser (control) | Before the engine layer (adapter) |
|---|---|---|---|
| RDF 1.2 syntax | 181 / 1 / 0 | 181 / 1 / 0 | 69 / 0 / 113 |
| RDF 1.2 evaluation | 87 / 0 / 0 | 87 / 0 / 0 | not scorable |
| RDF 1.1 syntax | 579 / 0 / 0 | 579 / 0 / 0 | 286 / 15 / 278 |
| RDF 1.1 evaluation | 414 / 0 / 0 | 414 / 0 / 0 | not scorable |
| SPARQL 1.1 query evaluation | 230 / 14 / 3 | 230 / 14 / 3 | not scorable |
| SPARQL 1.2 query evaluation | 65 / 0 / 0 | 65 / 0 / 0 | not scorable |
| JSON-LD 1.1 toRdf | 443 / 6 / 18 | 443 / 6 / 18 | 441 / 1 / 25 (accept or reject) |
| SPARQL 1.1 and 1.2 syntax, SPARQL Update | not scored | parser | none |
| RDFC-1.0 (`rdf-canon`; engine is `mere-rdfc`, parser column is raw `oxrdf`, the control) | 86 / 0 / 0 | 85 / 1 / 0 | none |
| RDF 1.1 and 1.2 c14n, entailment; JSON-LD expand, compact, flatten, fromRdf, html, remote-doc; SHACL | not run | not run | not run |

What changed against the adapter layer: the RDF 1.1 TriG syntax row went from 199/214 to 214/214 (the
15 fails were relative IRIs the old `from_trig` could not resolve), and N-Triples, Turtle and RDF/XML, which
had no entry point, are now scored on both syntax and evaluation. The engine's fails are the parser layer's
fails. As measured 2026-11-12 these were the two RDF/XML canonical-literal evaluation fails per suite (`xml-canon-test001`, `-002`,
`rdf12-xml-an-13`, `-14`), `ntriples-version-1`, the 14 and 1 SPARQL evaluation fails listed in their rows, and
the JSON-LD toRdf fails (nine dataset mismatches and `tli12`); the fork fixed the four RDF/XML fails, the SPARQL 1.2 fail and four of the nine mismatches (2026-11-14, rows above), and the rest still hold. No engine fail lacks a parser-layer fail on the
same test. JSON-LD's engine row first read 432 / 10 / 25, below the parser layer's by the 8 `processingMode`
1.0 tests the engine skipped (the adapter still does); P6 L1 F-R2 (2026-11-12, `parse_jsonld` takes a
`JsonLdOptions` with a `ProcessingMode`, default 1.1) scores them: the engine row is 439 / 10 / 18, the
parser layer's counts, and the two fail lists are the same ten tests. The RDF and JSON-LD rows above are the
same with and without the `query` feature (P6 L1 F-R1: the harness reads the dataset through
`IndexedDataset::quads`); the SPARQL rows need `query`.

The unconfirmed fails (no control built) are the ones listed under the scoring rule above; this lane built
none. The parser layer's pass on a case is what separates an engine-code fault from an upstream one, and no
case passes at the parser layer and fails at the engine layer.

Reading the starting point: the parsers behind the adapter are strong on syntax; the adapter
exposes less than the parsers do (no N-Triples, Turtle or RDF/XML entry point, no base IRI, no
processing mode, no dataset output, no update, no evaluation beyond SELECT and ASK); no capability
beyond the rows marked pass is claimed.

## RDFC-1.0, the carried implementation

Engine split P6 L2 (2026-11-13, D143 F-R7). `mere::canonicalize` and `canonicalize_quads` (module `formats::canon`) wrap
the crate `mere-rdfc` (`crates/graph/rdfc`), which is `oxrdf` 0.3.5's `rdfc-10` carried, not depended on. The reason is in
the comparison (`Code/testing/engine-split/rdfc_comparison.md`): `oxrdf`'s
`rdfc-10` and the other maintained Rust implementation, zkp-ld `rdf-canon` 0.15.3, both pass the 85 eval and map tests, and each
deviates from the spec elsewhere. `oxrdf` has no poison-dataset limit (`test074c`, a 10-node clique, does not return), builds all
n! permutations before iterating, and collapses repeated mentions in Hash N-Degree Quads step 3; `rdf-canon` double counts a
quad that names one blank node twice (4.4.3 step 2) and orders temporary identifiers as strings (`b10` before `b2`). Mark ruled
(2026-11-12) that upstreams are not to be bothered with AI-generated fixes, and (2026-11-13) to take and improve the more
spec-compliant of the two: `oxrdf`'s. Nothing was filed upstream.

What `mere-rdfc` changes, each marked in its source header: every mention kept (4.8.3 step 3), a budget on Hash N-Degree Quads
calls (default 4,000; the largest positive test uses 468) and on permutations (default 100,000; the largest uses 2,880),
lazy permutations, an own table interner, triple terms refused with `CanonError::TripleTerm`, `sha2` 0.11. There is no
blank-node-count limit: poison datasets are not told apart by it (the clique has 10 blank nodes, positive tests reach 19).
The arbiter was a differential fuzz against `rdf-canon` with its two deviations fixed in a scratch copy: 16 modes of 20,000
random datasets (default, named and blank-node graphs, literals, repeated mentions) and 48 cycle and chain shapes; the outputs
agree except where an implementation is not invariant under relabelling its input (both are, in a few of 20,000, since RDFC-1.0
leaves tie order open), and where both are invariant they agree on all 20,000.

