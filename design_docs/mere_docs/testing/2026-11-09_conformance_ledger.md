# Conformance ledger

Opened 2026-11-09 by P0 of the [engine split plan](../implementation_strategy/2026-11-07_engine_split_plan.md).
Rule (plan, Constraints, "Standards follow the suites"): engine capabilities ship behind a ledger
row (spec revision or pack version, supported profile, fixture suite, result), and no capability
is claimed without its row. Rows are appended or revised when a suite is rerun; a rerun states its
date and the adapter commit. Run evidence (commands, per-manifest counts, failing test names) is in
the [receipts](../../archive_docs/receipts/2026-11-07_engine_split_plan_receipts.md), not here.

## How a result is scored

- **Harness**: `crates/graph/linked-data/tests/w3c_conformance/` (`main.rs` plus five modules) (package `mere-rdf`, lib
  `linked_data`). Ignored by default; skips cleanly if the suites are absent. Command:
  `W3C_SUITES_DIR=<Code/crates> cargo test -p mere-rdf --features query --test w3c_conformance -- --ignored --nocapture`
  (`W3C_VERBOSE=1` lists failing tests).
- **Two layers.** `parser` runs the parsers the adapter wraps (`oxttl`, `oxrdfxml`, `oxjsonld`, `spargebra`, `spareval`, `sparesults`)
  with the adapter's features and options, and scores what the suite specifies: accept or reject
  for syntax tests, isomorphism of the produced dataset against the expected one for evaluation
  and toRdf tests. `adapter` runs the crate's own public entry points (`from_nquads`, `from_trig`,
  `from_jsonld_with_contexts_and_base_iri`) and can score accept or reject only: they return a
  graph contribution, not an RDF dataset, and take no base IRI or processing mode.
- **The `indexed` layer** (SPARQL query evaluation only, from 2026-11-10, quad index plan P2) loads each case's data
  through `linked_data::query::IndexedDataset::from_quads` and evaluates with the same `spareval`, scored as the
  parser layer is; it is the engine's own query dataset, so a difference from the parser layer is the dataset's.
- **Counts** are `pass / run`, where run excludes skipped tests. A skip is a test the layer cannot
  express (reason given); it is not a pass. Isomorphism uses `oxrdf`'s dataset canonicalisation,
  as an equality check only; it is not a claim on the RDFC-1.0 row.
- Both layers stay, and the rows are provisional until P5 (Mark, plan D61, 2026-11-09).
- **A fail is "engine" only with a control**: the test `engine_controls` in the harness loads the case's data by hand (inline TriG, or the suite file for RDF/XML) and runs `spareval`, `spargebra` or `oxrdfxml` directly, printing the result beside the expected one; a positive control (a plain pattern, and FROM NAMED over a plain GRAPH pattern) proves the instrument answers correctly. Fails with no control built are unconfirmed: the SPARQL and RDF/XML syntax fails, `ntriples-version-1`, and the JSON-LD toRdf fails.

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
| RDF 1.2 (`rdf12`) | N-Triples syntax | `rdf-tests` `rdf/rdf12/rdf-n-triples/syntax` | parser 33/34 (fails `ntriples-version-1`: `VERSION` directive rejected); adapter: no N-Triples entry point |
| RDF 1.2 | N-Quads syntax | `rdf-tests` `rdf/rdf12/rdf-n-quads/syntax` | parser 31/31; adapter `from_nquads` 31/31 |
| RDF 1.2 | Turtle syntax | `rdf-tests` `rdf/rdf12/rdf-turtle/syntax` | parser 77/77; adapter: no Turtle entry point |
| RDF 1.2 | Turtle evaluation | `rdf-tests` `rdf/rdf12/rdf-turtle/eval` | parser 32/32; adapter not scorable (no dataset out) |
| RDF 1.2 | TriG syntax | `rdf-tests` `rdf/rdf12/rdf-trig/syntax` | parser 38/38; adapter `from_trig` 38/38 |
| RDF 1.2 | TriG evaluation | `rdf-tests` `rdf/rdf12/rdf-trig/eval` | parser 26/26; adapter not scorable |
| RDF 1.2 | RDF/XML syntax and evaluation | `rdf-tests` `rdf/rdf12/rdf-xml` | parser (`oxrdfxml` 0.2.5) syntax 2/2, evaluation 27/29; the 2 fails (`rdf12-xml-an-13`, `rdf12-xml-an-14`) are engine: `oxrdfxml` does not write an `rdf:parseType="Literal"` value in exclusive canonical form (it keeps in-scope `xmlns` declarations on `<br>`), and the rest of each graph equals the expected one (control `engine_controls`); adapter: no RDF/XML entry point |
| RDF 1.2 | N-Triples and N-Quads canonical form | `rdf-tests` `.../c14n` | not run: no canonical serialisation entry point (82 tests) |
| RDF 1.2 | Semantics (entailment) | `rdf-tests` `rdf/rdf12/rdf-semantics` | not run: no engine stage yet (29 tests) |
| RDF 1.1 baseline | N-Triples syntax | `rdf-tests` `rdf/rdf11/rdf-n-triples` | parser 70/70; adapter: no entry point |
| RDF 1.1 baseline | N-Quads syntax | `rdf-tests` `rdf/rdf11/rdf-n-quads` | parser 87/87; adapter 87/87 |
| RDF 1.1 baseline | Turtle syntax and evaluation | `rdf-tests` `rdf/rdf11/rdf-turtle` | parser syntax 168/168, evaluation 145/145; adapter: no entry point |
| RDF 1.1 baseline | TriG syntax and evaluation | `rdf-tests` `rdf/rdf11/rdf-trig` | parser syntax 214/214, evaluation 143/143; adapter `from_trig` syntax 199/214 (15 fail: relative IRIs, `from_trig` takes no base IRI) |
| RDF 1.1 baseline | RDF/XML syntax and evaluation | `rdf-tests` `rdf/rdf11/rdf-xml` | parser syntax 40/40, evaluation 124/126; the 2 fails (`xml-canon-test001`, `xml-canon-test002`) are engine, same cause as above, rest of graph equal (control); adapter: no entry point |
| RDF 1.1 baseline | Semantics (entailment) | `rdf-tests` `rdf/rdf11/rdf-mt` | not run: no entailment stage yet (48 tests) |
| SPARQL 1.1 | Query syntax | `rdf-tests` `sparql/sparql11` (`manifest-sparql11-query.ttl`) | parser 106/106 (with the test's base IRI); adapter `query::sparql` parses with the same call and no base IRI, with no separate syntax entry point |
| SPARQL 1.1 | Update syntax | `rdf-tests` `sparql/sparql11` (`manifest-sparql11-update.ttl`) | parser 63/63; adapter: no update entry point |
| SPARQL 1.1 | Query evaluation | same | parser (`spareval` 0.2.9 over an `oxrdf::Dataset`, results read with `sparesults`) 230/244; 3 skipped (result file in a format the harness does not read); adapter: not scorable (its `query::sparql` evaluates SELECT and ASK over a kernel `Graph`); indexed (2026-11-10, `IndexedDataset::from_quads`) 230/244, 3 skipped, the same 14 fails. Row order is not checked; numeric results compare by value. All 14 fails are engine, each with a control that loads its data by hand and runs `spareval` or `spargebra` directly (`engine_controls`): `agg-groupconcat-04`, `-06` (ASK over GROUP_CONCAT of language-tagged strings answers false, expected true); `graph-from-named-1`, `-2`, `-3` (FROM NAMED is not applied to GRAPH over VALUES: VALUES leaves `?g` unbound, graphs outside FROM NAMED are visited); `graph-minus` (outer GRAPH variable treated as shared with MINUS: no rows); `zero_or_more_set_start`, `zero_or_more_set_end`, `zero_or_one_set_start`, `zero_or_one_set_end` (zero-length path with a constant end over an empty dataset gives no row); `subquery15`, `subquery16` (subquery with OFFSET and LIMIT inside GRAPH is evaluated once, not per graph); `bnode01` (BNODE(str) returns the same node across solutions, 2 distinct against 6 expected); `values-construct-group-by` (`spargebra` rejects a query with `VALUES` after `GROUP BY`) |
| SPARQL 1.1 | Update evaluation | same | not run: no update stage (94 tests; D65) |
| SPARQL 1.2 | Syntax | `rdf-tests` `sparql/sparql12` | parser 200/203 (accepts `nested-aggregate-functions`, `tripleterm-subject-03`, `tripleterm-subject-06`, which the suite says to reject) |
| SPARQL 1.2 | Query evaluation | `rdf-tests` `sparql/sparql12` | parser 64/65; the fail (`select-variable-reuse`) is engine: `spargebra` with `sparql-12` rejects reuse of a variable bound earlier in the same SELECT (control parses the query directly); adapter: not scorable; indexed (2026-11-10, `IndexedDataset::from_quads`) 64/65, the same fail |
| SPARQL 1.2 | Update evaluation | `rdf-tests` `sparql/sparql12` | not run: no update stage (3 tests; D65) |
| JSON-LD 1.1 | toRdf (JSON-LD to RDF) | `json-ld-api` `tests/toRdf-manifest.jsonld` (467 tests) | parser 439/449 (10 fail: 9 dataset mismatches, `tli12` rejects its base); 18 skipped (11 are 1.0-only, 4 `rdfDirection`, 1 `expandContext`, 1 expected output unreadable, 1 rejected only for an out-of-suite context). Adapter `from_jsonld_with_contexts_and_base_iri` accept or reject 441/442 (fails `tli12`); 25 skipped (those 18 less 1 plus 8 `processingMode` 1.0 tests) |
| JSON-LD 1.1 | Expand, compact, flatten, fromRdf, HTML, remote-doc | `json-ld-api` `tests/*-manifest.jsonld` | not run: no such API in the adapter or `oxjsonld` (386 + 246 + 58 + 54 + 50 + 18 tests) |
| SHACL (Core, SPARQL) | none yet | `data-shapes` | not run: no engine stage yet (P6) |
| RDFC-1.0 | none yet | `rdf-canon` | not run: no engine stage yet (P6) |

Reading the starting point: the parsers behind the adapter are strong on syntax; the adapter
exposes less than the parsers do (no N-Triples, Turtle or RDF/XML entry point, no base IRI, no
processing mode, no dataset output, no update, no evaluation beyond SELECT and ASK); no capability
beyond the rows marked pass is claimed.
