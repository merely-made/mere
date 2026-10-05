# Graph semantics plan: assertions, resources, saved queries, residency

**Date:** 2026-10-04
**Status (2026-10-05):** in progress. P1 implemented and validated on the
isolated `graph-semantics` branch, with C1, stable-root attribution,
retract/assert, A1 and B1 resolved. Mark authorized P2 with "Proceed".
P2 inventory has begun; C2 is resolved and work is stopped at C3 before code
changes. C4 remains open. P3–P5 have not begun; nothing has been integrated
into main.

Four questions were put to Mark from outside the project: what a link records,
what makes two things the same thing, what a saved query can become, and how
much of the graph must be present to use it. Each was grounded against the
tree (§2), then put to him as a multiple-choice round. §3 quotes his answers
verbatim; anything beyond his words is marked *Reading, not ruled*.

Parents: the [statements-over-schema stance](../technical_architecture/2026-05-22_statements_over_schema_stance.md),
the [statement kernel brief](../technical_architecture/2026-06-19_statement_kernel_brief.md),
the [petgraph-RDF plan](2026-06-18_petgraph_rdf_plan.md), the
[node navigation lineage plan](2026-06-05_node_navigation_lineage_wiring_plan.md),
the [ambiance design](../design/2026-09-23_ambiance_design.md) and the
[family composition thesis](../../2026-08-12_family_composition_thesis_brief.md)
(the narrowing gradient).

## 1. The questions

1. **What does a link record?** "A cites B" could mean the page says so, the
   user says so, an extractor found it, or a peer shared the claim. One
   relationship with supporting sources, or several independently attributable
   statements?
2. **What makes two things the same?** A URL, a captured version of a page and
   an encounter with it have different identities. Do two visits reach one
   node, and what happens when the page changes?
3. **What can a saved query become?** A temporary result, a collection that
   updates itself, or a frozen selection that can be annotated and shared?
4. **How much of the graph must be present?** If Mere works on a neighborhood,
   an absent relation may mean "not loaded" rather than "does not exist".

## 2. Findings

Verified 2026-10-04 against Mere `68d2a928`.

- **F1. One claim, one provenance slot, last writer wins.**
  `SemanticData::assert_statement` dedups on `(recognized_sub_kind, predicate,
  graph_scope)` within a node pair and, on a match, overwrites `label`,
  `provenance_iri` and `asserted_at_ms` in place
  (`crates/graph/graph-kernel/src/graph/edge_data.rs` 157-198;
  `insert_statement` at 214-251 does the same). Nothing accumulates.
- **F2. Scope could separate the four cases; writers do not use it.**
  `GraphScope` is `Default | Source | User | Agent | Moot | Custom`
  (`crates/graph/graph-kernel/src/types.rs` 83-91), one variant per case in
  question 1. Only JSON-LD ingest carries a scope through
  (`crates/graph/linked-data/src/ingest/apply.rs` 125-155).
  `GraphDelta::AssertSemanticPredicate` lands in `GraphScope::Default`
  (`edge_ops.rs` 116); every `Source` and `User` hit in `linked-data` sits in a
  test module.
- **F3. Attribution lives in the journal, not on the statement.** Each journal
  entry carries an `Author { kind: Person | Rule | Script | Engine, id,
  version, via }` (`crates/graph/graph-kernel/src/graph/journal.rs` 64-98).
  The history of who asserted what exists; the live statement shows only the
  last provenance.
- **F4. Retract removes a merged claim for every writer.**
  `retract_statement(id)` removes the statement by id (`edge_data.rs` 203);
  under F1 that id stands for everyone who asserted the claim in that scope.
- **F5. Three identity rules, no reconciliation.** Plain `add_node` mints a
  random UUID, treats the address as a property, and lets several nodes share
  one URL (`url_to_nodes: HashMap<String, Vec<NodeKey>>`,
  `crates/graph/graph-kernel/src/graph/mod.rs` 438-480). Linked-data ingest
  mints `node_namespace_id`, a UUIDv5 over the raw URL (`ingest/apply.rs` 74).
  The kernel keeps URLs verbatim (`address.rs` 153), so a `utm_` variant is a
  different key. Eidetic's page table identifies a page by content (blake3 over
  normalized text plus a simhash for near-duplicates), strips tracking
  parameters, and falls back to a canonical URL that it names in the type
  (`crates/eidetic/eidetic-core/src/browsing/page.rs`).
- **F6. The live node is a browsing surface, and its claims ride along.** The
  2026-05-18 identity brief (archived) decided UUID identity with no URL dedup;
  the 2026-06-05 lineage plan confirmed "a node is a browsing surface".
  `GraphDelta::NavigateNode` records the visit and calls `update_node_url`,
  which replaces the primary address and touches nothing else
  (`apply.rs` 1152-1171, `mod.rs` 519-543; pictograph's `navigate_member`,
  `crates/canvas/pictograph/src/canvas/nodes.rs` 300-315). A content statement
  asserted about page 1 stays on the node when it shows page 2. Canvas `visit`,
  the one URL-dedup path (`input.rs` 417), is called only from canvas tests.
- **F7. Four homes for a query result.** `>sparql` returns rows and cannot
  return a graph (`crates/graph/linked-data/src/query.rs`: `CONSTRUCT` and
  `DESCRIBE` unsupported). `SubgraphBinding` is `UnlinkedSession | Linked {
  spec } | Branched { parent_spec, reason }`, and `SubgraphKind` is a closed
  list of nine shapes (`crates/forme/forme/src/subgraph.rs` 51-88). A supernode
  is display-only. A nested graph is authored and syncs, borne by
  `Container.nested: Option<LogId>` (`crates/eidetic/chartulary/src/container.rs`
  76-79). TERMINOLOGY (ruled 2026-07-17): a subgraph is "a scope, never a
  container, never synced as a thing". Nothing freezes a live subgraph into a
  nested graph.
- **F8. Whole-graph residency, and one path that discards.**
  `Graph::from_snapshot` loads a whole snapshot
  (`crates/graph/graph-kernel/src/graph/snapshot/from.rs` 48).
  `apply_link_statements` returns a recognized link whose target is not in the
  graph as `pending_targets` (`crates/graph/linked-data/src/statements.rs`
  80-83) and no caller reads that list, so the link is lost. The stance doc
  says "absence is not falsehood".
- **F9. Absence already has four reasons.** The narrowing gradient:
  possession ⊇ disclosure ⊇ synchronization ⊇ projection, each owned by a
  mechanism; a projection exceeding its sync scope is a named bug class. It
  does not require a result to say which layer limited it.

### Branch-base review (2026-10-04)

Verified against branch `graph-semantics` base `36893553`, in the isolated
`mere-graph-semantics` worktree. Fifteen of the sixteen source files checked
for F1–F8 are unchanged from `68d2a928`. The remaining file,
`crates/canvas/pictograph/src/canvas/input.rs`, changes camera following on
drag, pan and zoom; `Canvas::visit` is unchanged and now starts at line 424.
F1–F9's architectural gaps remain; no phase changes.

- **F2 precision.** The production RDF scope decoder and encoder do mention
  `GraphScope::Source` and `GraphScope::User`
  (`crates/graph/linked-data/src/ingest.rs` 210–211 and
  `crates/graph/linked-data/src/lib.rs` 149–150). The original statement that
  every hit is in a test is too broad. These translate a supplied scope;
  they do not supply attribution to the other assertion writers. P1 stands.
- **F8 consumer status.** A search of all Rust files under `crates/` and
  `ports/` finds zero production call sites of `apply_link_statements` and
  zero production readers of `pending_targets`; its executable callers and
  readers are tests in `crates/graph/linked-data/src/statements.rs`.
  This is an unconsumed pending-target path, rather than evidence of observed
  production data loss. P3 still needs the index and its consumer wiring.
- **F9 document drift.** The family composition thesis now carries the
  2026-10-04 ruling-4 amendment (lines 154–159); the stance document carries
  ruling-6 and ruling-1/2/4 amendments (lines 65 and 76). They record the
  intended resource split and coverage, not implemented mechanisms.

### C1 evidence (2026-10-04)

`PersistedSemanticStatement` has seven fields: id, predicate, recognized
sub-kind, label, scope, optional provenance and optional time
(`crates/graph/graph-kernel/src/persistence_edge.rs` 173–186). It has no
recorder `Author`; `GraphSnapshot` has no author or journal input
(`crates/graph/graph-kernel/src/persistence.rs` 237–253). The journal's Author
is separate (`crates/graph/graph-kernel/src/graph/journal.rs` 88–99).

There are two legacy restore paths: a statement bucket preserves missing
provenance verbatim, while an aggregate-only semantic edge synthesizes
statements (`crates/graph/graph-kernel/src/graph/snapshot/from.rs` 291–329).
The same restore helper also serves exact undo restoration and merge
(`crates/graph/graph-kernel/src/graph/edge_ops.rs` 430;
`crates/graph/graph-kernel/src/graph/merge.rs` 109). Consequently, assigning
the current user or an ingest engine in that helper would attribute an old
claim without evidence and would affect more than disk loading. The
snapshot alone cannot recover the original asserter or expand an already
merged claim back into its lost assertions.

C1 options returned to Mark, recommendation first:

1. **Legacy marker (recommended).** Missing provenance receives a stable IRI
   explicitly meaning unknown legacy asserter. Keep existing statement ids,
   times and supplied provenance. This preserves the claim without claiming
   that the current user or an engine authored it; later known assertions
   remain separate from the legacy assertion.
2. **Local user.** Attribute missing provenance to the user opening the
   graph. This makes later assertions by that user update the legacy record,
   but assigns authorship the stored record does not establish.
3. **Ingest engine.** Attribute missing provenance to an assumed historic
   ingest engine. This makes its later assertions update the legacy record,
   but assigns an engine the stored record does not establish.

**C1 ruling (2026-10-04).** Mark: **"1"**, selecting the legacy marker above.
Missing legacy provenance receives an explicit unknown-asserter IRI; existing
statement ids, times and supplied provenance are preserved. Attribution to
the current user or an assumed ingest engine is not selected. This resolves
the load-policy checkpoint; production writers still supply a known asserter.

### P1 implementation findings (2026-10-04)

- **Assertion identity.** Both live insert paths now dedup with attribution.
  Ingest of a carried handle updates the held assertion for that asserter,
  retaining its id. Exact snapshot/undo restoration remains a separate path:
  C1 requires preserving every legacy handle, including several records whose
  unknown authors collapse to the same marker
  (`crates/graph/graph-kernel/src/graph/edge_data.rs`,
  `crates/graph/graph-kernel/src/graph/edge_payload.rs`).
- **Writer attribution and capture.** The session previously assigned its
  Author only after executing an edit. `Graph::write_as` now supplies that
  context during the edit and restores it on return or panic. An attributed
  source overrides it. Semantic edits capture exact pair state, including
  updates that mint no id and precise retractions. The existing capture
  schema is unchanged; author-aware replay recovers the journal Author for
  old raw assertions, while bare captures without an envelope use the C1
  unknown marker (`crates/system/pandect/src/graph_session.rs`,
  `crates/graph/graph-kernel/src/graph/capture.rs`,
  `crates/graph/graph-kernel/src/graph/journal.rs`). Predicate annotation can
  create a statement, so its delta also carries attribution.
- **RDF ingest.** Several reifiers on one triple previously overwrote the
  same contribution. Ingest consumes the base-triple slot once, then emits
  one contribution per further reifier; reifier traversal is deterministic.
  Export already emitted one reifier per statement
  (`crates/graph/linked-data/src/ingest.rs`,
  `crates/graph/linked-data/src/lib.rs`). Query fixtures still wrote literal
  properties onto the removed node field; they now use the facet write API
  (`crates/graph/linked-data/src/query.rs`).
- **Consumer boundary.** A read-only Rust-source search in Turnstone and
  knot-editor found zero constructors or matches of the six changed
  asserting/predicate-setting `GraphDelta` variants. Turnstone has an
  exhaustive capture match in its behaviors module, so P1 adds neither a
  capture variant nor serialized fields. Public assertion helper and
  `EdgeAssertion` signatures remain stable. This is source inspection,
  not a sibling build receipt; no sibling files were changed.

### Additional P1 forks (2026-10-04, answered with follow-up review)

**Peer identity.** The personal sync fold is a production assertion writer
(`ports/graphshell/src/personal_sync.rs`, `apply_event`, line 1384). The signed
operation gives two 32-byte identities: its device signer and a verified
stable persona root; `WriterReceipt` retains both (line 399), and
`materialize` verifies the root before applying events (line 1213). The fold
currently passes neither to the assertion. The required workspace check
fails with E0063 at this writer because its delta now requires attribution.
Consequences below follow from the dedup key, rather than a device trial:

1. **Stable persona root (recommended).** Two devices of one persona update
   one assertion; signing-key rotation does not add an asserter. Keep the
   signer in the existing receipt.
2. **Device signing key.** Two devices create two assertions; key rotation
   adds another asserter. Keep the persona root in the existing receipt.

**Predicate mutation.** `SemanticData::set_statement_predicate` rewrites all
predicates on a pair (`crates/graph/graph-kernel/src/graph/edge_data.rs`, line
306). For two open predicates under one scope and known asserter, a bulk stamp
leaves two ids but only one dedup key. This follows directly from its loop;
the ordinary assertion APIs have no such collision. A Rust-source search
finds zero production callers of the live `SetEdgeSemanticPredicate` delta
in Mere, Turnstone or knot-editor; its fixture and legacy replay paths remain.
The new asserter field alone does not settle this identity conflict.

1. **Retire the live bulk setter (recommended).** Writers assert the correct
   predicate upfront. Changing a claim retracts it and asserts its replacement;
   legacy decoding remains available.
2. **Edit one assertion.** Target its id, preserve that id when changing its
   predicate, and reject a collision with another assertion.
3. **Merge collisions.** Keep a caller-selected assertion id, retire collided
   ids, and journal the complete change.

At this checkpoint neither policy had been selected in code. These choices
were returned through the user input panel; the subsequent answer and review
are recorded below.

### P1 co-op and sync findings (2026-10-05)

**Follow-up ruling.** Mark: **"yeah, the stable root. retract/assert sounds
fine, but double check against caller-selected id. idk but i'd think co-op
might need it"**. The stable root is now supplied by `materialize` after
verification (`ports/graphshell/src/personal_sync.rs`, lines 1214–1216);
the signing device remains in `WriterReceipt`. No bulk-setter retirement or
collision merge has been implemented.

**Caller identity is distinct from a merge survivor.** The kernel's
`assert_persisted_semantic_statement` accepts an already-minted id; a repeated
dedup key keeps its existing handle (`crates/graph/graph-kernel/src/graph/edge_ops.rs`,
line 346; `crates/graph/graph-kernel/src/graph/tests/assertion_replay.rs`,
line 125). Commons assigns its own
`(writer, counter)` edge ids in an atomic commit; Knot assertions use the
signed operation hash and retract an exact causally observed assertion.
The bounded sibling audit found no kernel `statement_id` consumers and no
established caller-selected survivor operation. This is source inspection,
not a sibling build or cross-device receipt.

**The personal sync grammar loses assertion identity.** `AssertRelation`
carries only endpoints and `EdgeAssertion`; `RetractRelation` carries only
endpoints and a selector (`personal_sync.rs`, lines 133–142). Each call to
`materialize` creates a new graph (line 1204) and replays live assertions
(line 1388), so ids are minted anew. The same path supplies no assertion time.
The selector replay (line 1399) removes all matching sources rather than
one assertion.

Three regression tests now assert the P1 invariants using two admitted stable
roots and real signed operations, with positive controls in the same run
(`personal_sync.rs`, lines 1731, 1755, 1772). Results: **1 passed, 2 failed**.
Attribution yields two statements with distinct ids and the correct roots;
device signing keys differ from those roots. Rebuilding the unchanged log
changes **both of two ids**; all assertion times are `None`. One root's
retraction reduces **two statements to zero**, where the other source's one
statement must remain. The failing regressions are retained uncommitted;
they are not weakened or disabled.

**Fork A: portable assertion identity.** Both options add a signed authoring
time for new assertions; historical operations without a recorded time keep
an explicit unknown time. Both preserve the first handle when the same
asserter reasserts its existing claim, as ruling 1 requires.

1. **Carry caller-supplied assertion ids (recommended).** Add assertion id
   and time to the signed assertion event, plus an exact-id retraction event.
   Derive a deterministic fallback handle from the signed operation and
   event position for old assertions. This preserves an id minted by a
   co-op host or imported assertion before sync; conflicting id/content
   reuse must be refused.
2. **Derive all sync assertion ids.** Use the signed operation and event
   position to allocate the first assertion's handle; add an exact-id
   retraction event. Co-op callers learn the id after authoring and cannot
   preserve a preexisting handle through this event grammar.

**Fork B: old selector-only retractions.** New exact-id events are independent
of this historical decoding decision.

1. **Withdraw only the writer's matching assertions (recommended).** Interpret
   an old selector event as that stable root withdrawing its selected claims.
   Other roots' assertions survive. Reopening an old log can restore a claim
   that the old broad retraction previously hid.
2. **Preserve historical broad deletion.** Old selector events continue to
   erase every matching source; new writers use exact-id events only. This
   reproduces historical results but explicitly exempts those old events
   from P1's independent-retraction invariant.

At this checkpoint neither fork had been selected or implemented. P1 could
not land until the choices and their gates were resolved; P2 had not begun.

**Follow-up ruling (2026-10-05).** Mark: **"A1, b1, sure"**. Carry signed
caller-supplied assertion ids and times, derive fallback handles for old
assertions, and use exact-id retractions for new semantic writes. Decode
old selector-only semantic retractions as that stable root withdrawing its
own matching assertions. This permits previously hidden claims to reappear
when an old log is reopened. The co-op review is complete; live bulk
predicate edits use retract/assert rather than selecting a merge survivor.

### P1 implementation findings after A1/B1 (2026-10-05)

The optional `statement_id` and `asserted_at_ms` fields on personal-sync
assertion events are omitted when absent, preserving old CBOR bodies and
signatures. New semantic authoring fills them before signing; a caller's
supplied id/time is retained. Old assertions use a deterministic handle
from the signed operation hash and event position, retaining unknown time.
`RetractAssertion` targets a handle; new semantic selector withdrawals are
refused rather than written as old events. Decoding those old events
withdraws only the verified stable root's claims, for both sub-kind and
family selectors (`ports/graphshell/src/personal_sync/assertions.rs`,
`prepare_event`, `validate_event`, `retract_legacy`).

The rebuildable identity index refuses an id reused for different endpoints,
predicate or asserter at intake when the body is readable, and again during
projection. Reasserting the same claim updates its first handle, matching
ruling 1. Keyless retention still defers checks of sealed contents until
they can be read; no retained operations are discarded
(`ports/graphshell/src/personal_sync.rs`, `accept_into`,
`observe_assertion_ids`, `materialize`). The seven signed-operation tests
include two roots, receiver replication, explicit id/time carriage, unchanged
legacy bytes, fallback id stability, missing-id no-op and real exact
withdrawal, collision refusal with a distinct-id positive control, and both
old selector forms. The original failing tests are now green.

Removed the live bulk-predicate delta; exact retract/assert replacements
mint a new handle and preserve the other asserter's full record, including
through captured replay. Legacy capture decoding and its serialized schema
remain (`crates/graph/graph-kernel/src/graph/apply.rs`,
`crates/graph/graph-kernel/src/graph/tests/assertion_replay.rs`). A bounded
Rust-source search found no sibling use of the removed live variant or of
the expanded personal-sync event enum. Turnstone's exhaustive capture match
continues to use the retained legacy variants; sibling builds were not run.

Review found an attribution discrepancy between historical exact journal
replay and checkpoint loading. Exact pair restoration now fills missing
provenance with the C1 marker on every restored parallel bucket and on
aggregate-only legacy records. It does not merge duplicate handles or
alter their ids/times; known sources remain unchanged. Controls compare
full journal replay, checkpoint-plus-tail replay and snapshot reopening
(`crates/graph/graph-kernel/src/graph/edge_ops.rs`, `set_edges_between`;
`crates/graph/graph-kernel/src/graph/journal.rs`,
`legacy_exact_capture_attribution_matches_checkpoint_replay`).

### P2 containment checkpoint (2026-10-05)

Verified at P1 commit `459cad84`. The seven containment sub-kinds have
seven explicit construction sites outside test modules in Mere: two
`UrlPath` and one `Domain` in `graph/query.rs` (297, 386, 395), one
`CollectionMember` editor mapping in `ports/graphshell/src/product.rs`
(202, applied at 627), and three built-in fixture assertions in
`ports/graphshell/src/mere_host_fixture.rs` (174, 192, 200). The other
four sub-kinds have no named production origin writer found. Generic
assertion and replay paths accept them; snapshot reconstruction restores
all seven (`graph/snapshot/from.rs` 378–387). Kernel paths here are under
`crates/graph/graph-kernel/src/`.

`CollectionMember` is broader than layout in current fixtures: scene to
file, and persona to device twice. A read-only sibling search found one
named containment writer in Turnstone (`turnstone/src/overmap.rs` 111),
joining session containers referenced by `sub_graph_refs`, and none in
Knot-editor. These are source findings, not consumer build receipts.

C2 options presented, recommendation first; selection recorded below:

1. **Fixed split by sub-kind (recommended).** Resources carry `UrlPath`,
   `Domain`, `FileSystem`, `ClipSource`; surfaces carry `UserFolder`,
   `NotebookSection`, `CollectionMember`. This follows the plan's proposed
   split and preserves the editor/overmap meanings. Persona/device fixture
   membership also stays surface-owned; resource collections would need a
   distinct later relation.
2. **Explicit graph per writer.** URL-derived containment is resource-owned;
   authored containment can use explicit resource or surface endpoints.
   Resource collections and layout collections can share `CollectionMember`,
   but routing, migration and walks must distinguish both endpoint kinds.
3. **All containment on surfaces.** Preserve the current writers' surface
   meaning; URL hierarchy remains about addressed surfaces and is unavailable
   to resource-only containment queries.

**C2 ruling (2026-10-05).** Mark: **"1"**, selecting the fixed split.
Resources carry `UrlPath`, `Domain`, `FileSystem`, `ClipSource`; surfaces
carry `UserFolder`, `NotebookSection`, `CollectionMember`. The existing
persona/device fixture membership therefore remains surface-owned.

**Lane coordination.** Mark relayed the stack-seams P1 request covering
`scenomise::projection` and its dataset types. This lane holds those files
for that owner. The current Pictograph canvas path calls `scenomise::solve`
(`crates/canvas/pictograph/src/canvas/strategy.rs` 268); no dependency on a
relationship-compiler change has been identified for graph semantics P2.
This lane owns neither the recipe pass nor Woodshed's captures, so its
response cannot qualify that owner's pitch change or repin. Current main's
stack-seams plan records the separate S12/S15–S18 coordination; it is not
part of this branch's base.

**Dependency finding.** The canonicalizer is std-only, but Eidetic currently
gates chartulary behind `lineage` (`crates/eidetic/eidetic-core/Cargo.toml`
44, 70). Chartulary is already in Eidetic's lockfile dependency list.
Neither crate has a UUID dependency, while the kernel has UUIDv5 support.
A complete shared resource-id helper in chartulary would add an edge to the
existing UUID package and require a lockfile change. That remains an explicit
stop before implementation; no manifest or lockfile has been changed.
The existing raw-URL surface helper's namespace is documented as fixed
(`graph/mod.rs` 454–466), so resource identity needs a separate helper.

**C3 inventory, no migration choice.** A persisted semantic assertion has
seven fields, with no endpoint URL, visit or version reference; its edge
names two surface UUIDs (`persistence_edge.rs` 173, 429). Its optional time
can change on reassertion. Navigation has visit creation and latest-access
times, not a complete chronological cursor log; old snapshots may have no
history (`graph/history.rs` 190–242; `persistence.rs` 250–255). A complete
retained journal and baseline can recover shown URLs by sequence, but
baseline assertions can predate that log (`crates/system/pandect/src/graph_session.rs`
363–447). Nearest-visit assignment would therefore be inference.
Collapsing same-URL surfaces also combines assertion buckets: live upsert
can lose a distinct legacy handle while exact restore preserves it
(`graph/edge_data.rs` 256–292). Migration must retain handles and explicitly
resolve collisions before a policy is implemented. C3 remains open.

### C3 migration checkpoint (2026-10-05)

Rechecked at `5666943e`, which changes documentation only. Three assertion
helper routes supply no time (`graph/edge_ops.rs` 39–45, 119, 157), and
same-source reassertion can overwrite a handle's time (`graph/edge_data.rs`
184–205). Imported linear history uses array indices as synthetic
milliseconds (`graph/history.rs` 392–415). A nearest-visit assignment cannot
recover the original endpoints reliably. These are schema and code findings;
no census of Mark's saved data was performed.

Current page identity is available on restoration: the shared navigation
cursor overrides the stored primary URL except for a Graphshell clip route
(`graph/snapshot/from.rs` 155–173). A retained journal can offer stronger
historical evidence, but claims in its baseline may predate it. Existing
facet authority must be loaded beside the snapshot: current snapshots write
empty legacy properties/classifications/derivations, while session loading
reads the facet store (`graph/snapshot/to.rs` 101–109;
`crates/system/pandect/src/graph_session.rs` 435–440).

C3 options, recommendation first; none selected:

1. **Currently shown resources, with a migration record (recommended).**
   Move old content claims to the canonical resources their endpoint surfaces
   show at migration. Preserve original endpoints, ids, times and asserters
   in a migration record; mark historical page ownership as uncertain.
   Deterministic and complete, but an old claim can be attached to a page
   reached after it was asserted.
2. **Nearest timestamped visits.** Infer historical endpoints from visits
   nearest each assertion time. Requires further choices for missing and
   synthetic times, ties, back/forward and first assertion versus reassertion.
   Preserve the originals and mark the inferred mapping.
3. **Remove uncertain claims from active results, retain originals for review.**
   Do not assign uncertain historical content to a resource. Preserve the
   original claims in a recoverable migration record and require a later
   placement decision; old content claims disappear from active queries.

Regardless of the endpoint policy, collapse is a separate unresolved choice.
The existing fixture has two surfaces at one URL (`graph/tests/snapshot_basic.rs`
547–631). Two distinct old handles can become one dedup key on one resource
pair; normal upsert would lose a handle, while exact restore keeps both.
Same-id/different-payload input would silently reject one record in the
existing helper (`graph/edge_data.rs` 256–292). No such conflict was measured
in Mark's data, and no migration is implemented. Return the collision policy
as a further fork before code chooses a survivor or changes an id.

## 3. Rulings

Mark's answers, from multiple-choice rounds; each is the option label quoted
verbatim unless marked as his free text.

**Ruling 1.** *What does a link record?* Options: separate assertions, one
statement per (predicate, scope, asserter), each with its own id, time and
provenance, retracted alone, grouped into one claim by the view; one claim
with a supporter set; fix scopes only, keeping the merge. Mark: **"Separate
assertions (Recommended)"**. Follows: the dedup key gains the asserter, so a
re-assertion by the same asserter updates its own record and a different
asserter adds a record; retraction is per assertion (F4 no longer applies); the
drawn link groups a claim's assertions. *Reading, not ruled*: this is RDF 1.2's
model of several reifiers on one triple term, so projection emits one reifier
per assertion. What counts as the asserter is fork 5.1.

**Ruling 2.** *What is a node?* Options: resources under surfaces; node is the
resource; node stays the surface. Mark: **"Resources under surfaces
(Recommended)"**. Follows: the browsing-surface model of the 2026-05-18 brief
and the 2026-06-05 lineage plan stays for what the user browses in. Content
statements attach to a resource identity beneath it: one id per canonical URL,
using eidetic's canonicalizer, minted as a UUIDv5 over the canonical URL
everywhere. A surface shows one resource at a time, and navigating changes
which resource it shows instead of carrying claims along (F6). Page versions
are content-hashed captures under the resource. Amends the 2026-06-05 model
only in that content claims no longer live on the surface. *Reading, not
ruled*: how resources are represented is fork 5.2; the words for the two
senses are not chosen.

**Ruling 3.** *What can a saved query become?* Options: share spec, freeze on
demand; make subgraphs syncable; saved queries are view state. Mark: **"Share
spec, freeze on demand (Recommended)"**. Follows: a saved query is a `Linked`
subgraph whose spec may be any selector or a SPARQL query. Sharing sends the
spec, and each receiver evaluates it over their own graph. "Freeze"
materializes the current members into a nested graph that records the spec and
revision it came from. The 2026-07-17 ruling stands: a subgraph never syncs as
a thing. *Reading, not ruled*: what a frozen nested graph holds is fork 5.4.

**Ruling 4.** *What does absence mean?* Options: whole graph with honest
results (full residency; pending link statements ambient behind a rebuildable
index; every result carries a coverage note naming the layer that limited it,
with room reserved for "not loaded"); store pending as dangling; build partial
residency now (neighborhood loading from muniment, absence unknown by
default); closed world per mere. Mark (free text): **"Build partial residency
so we have the "not loaded" of 1 too; pending link statements stay ambient but
a rebuildable index, rederivable when the target returns, but also purgeable.
The rest of 1 makes sense, with the addition of 3."** Follows: partial
residency is built, loading neighborhoods from muniment with absence unknown by
default, so "not loaded" is a real coverage layer rather than a reserved one.
Pending link statements stay ambient, held in an index that re-derives them
when the target returns and that can be purged. Every query and projection
result carries a coverage note naming the layer that limited it. F8's discard
ends. *Reading, not ruled*: "not loaded" sits inside possession (held on disk
or on another device, not in memory), so the coverage layers are possession,
residency, disclosure, synchronization, projection; the purge policy is a
setting whose default is open; the residency unit, and whether full residency
remains a mode, are fork 5.3.

Round 2 put the forks rulings 1 to 4 opened, which the round-1 draft numbered
5.1 to 5.4; they are rulings 5 to 8, in that order.

**Ruling 5.** *Who is the asserter in the dedup key?* Options: the attributed
source, else the journal Author; the journal Author only; the Author with its
version in the key. Mark: **"Attributed source, else Author (Recommended)"**.
Follows: the asserter is whoever the claim is attributed to: the page for a
claim the page makes, the peer for a claim a peer shared, otherwise the journal
`Author` who wrote it (a persona id for a person; a rule's, script's or
engine's name, with its version kept as metadata, so a new extractor version
updates its predecessor's assertion rather than adding one). `provenance_iri`
holds the asserter; the recorder stays in the journal.

**Ruling 6.** *How are resources represented?* Options: a resource graph beside
the surface graph; a second node kind in one graph; a resource table keyed by
IRI. Mark (free text): **"I favor 1, but could we compare 1 and 3?"** The
comparison: option 3 makes the table the truth and a petgraph index derived
from it, which is the held-RDF-truth design measured on 2026-06-18 at about 11x
memory, 19x load and 18x mutation (petgraph-RDF plan), applied to the content
half; it needs the walk, selector, journal and snapshot machinery a second
time; its two wins, IRI-keyed storage and SPARQL without a per-query rebuild,
are available to option 1. Put back with options: resource graph, IRI-keyed;
resource table keyed by IRI; measure both first. Mark: **"Resource graph,
IRI-keyed (Recommended)"**. Follows: two chartulary graphs per mere. The
resource graph carries content statements between resources; the surface
graph keeps traversal, arrangement and layout containment; a surface records
the resource it shows, and the canvas lifts content links onto surfaces;
cross-family walks join through that relation. Resource storage is keyed by
resource id (the UUIDv5 of the canonical IRI, so lookup by IRI is one hash),
and SPARQL is served by a `QueryableDataset` adapter over the resource graph
instead of a dataset rebuilt per query. Petgraph-as-truth (2026-06-18) holds.

**Ruling 7.** *What is the residency unit?* Options: the neighborhood of what
is in view; projection scope plus a margin; whole sessions or nested graphs.
Mark: **"Neighborhood of what's in view (Recommended)"**. Follows: k hops, a
setting, around every surface in an open projection, plus pinned and focused
items; anything else loads on demand and reports "not loaded" until then. Full
residency stays a setting for small meres.

**Ruling 8.** *What does a frozen selection hold?* Options: resource
references with statement copies; references only; full copies. Mark:
**"Resource refs, statement copies (Recommended)"**. Follows: members are
referenced by resource id, which is portable because it is deterministic; the
statements among them are copied as they stood at the freeze revision, so a
later retraction does not change the frozen record. Annotations attach to the
frozen node. A receiver missing a resource sees it through ruling 4's coverage.

## 4. Phases

In order; each phase lands green before the next starts. Code samples: none.

- **P1. Assertions (rulings 1, 5).** The statement dedup key becomes
  `(recognized_sub_kind, predicate, graph_scope, asserter)`, with the asserter
  in `provenance_iri`: the attributed source when the writer records a claim on
  someone's behalf (the page for a claim a page makes, the peer for a shared
  claim, the `prov:wasAttributedTo` agent on ingest), else the journal `Author`
  rendered as an IRI (persona id for a person; name for a rule, script or
  engine, version as metadata). Every production writer supplies one; the
  `GraphDelta` variants that assert statements carry it. Retract is per
  assertion. The view groups a claim's assertions into one drawn link.
  Projection emits one RDF 1.2 reifier per assertion.
  Done when: two asserters of one `(predicate, scope)` on one pair yield two
  statements with distinct ids, provenance and times; retracting one leaves the
  other; the same asserter re-asserting updates its own record in place; an
  extractor at a new version updates its old assertion rather than adding one;
  no production writer passes `None` (a test enumerates the writers);
  `dataset_round_trip_is_lossless_under_the_profile` stays green with a case of
  two reifiers on one triple term; existing snapshots load (checkpoint C1).
- **P2. Resource graph (rulings 2, 6).** Eidetic's `canonical_url` moves to
  chartulary, the common dependency of the kernel and eidetic-core, and both
  use it; nothing keeps a private copy. Each mere holds two chartulary graphs:
  a resource graph (one resource per canonical URL, id = UUIDv5 over the
  canonical IRI, minted the same way by the kernel, linked-data ingest and
  eidetic) carrying content statements, and the surface graph keeping
  traversal, arrangement and layout containment. A surface records the
  resource it shows; `NavigateNode` changes that record and no longer carries
  claims. The canvas lifts a content link onto every pair of surfaces showing
  its two resources. Cross-family selector walks join through the shown
  resource. Page versions are content-hashed captures under the resource. SPARQL
  is served by a `QueryableDataset` adapter over the resource graph, retiring
  the per-query `dataset_quads` rebuild in `linked-data::query`.
  Done when: two surfaces showing one canonical URL see the same content
  statements; a surface navigating from page 1 to page 2 shows page 2's
  statements while page 1's stay on page 1's resource; a `utm_` variant and a
  fragment variant resolve to one resource; the kernel, ingest and eidetic mint
  the same resource id for one page; a component walk selecting Semantic plus
  Traversal crosses from a surface into the resource graph and back; the SPARQL
  path builds no dataset per call and returns the same rows as the old path on
  the existing query tests (the old path survives only as the test oracle);
  `cargo check -p mere-kernel --target wasm32-unknown-unknown` stays green.
- **P3. Coverage and the pending index (ruling 4).** A coverage note travels
  with every query result and scene projection, naming each layer that limited
  it: possession, residency, disclosure, synchronization, projection.
  `apply_link_statements` stops discarding: a recognized link whose target
  resource is absent goes to a pending index that is not graph truth, can be
  rebuilt from source documents, re-derives the statement when the target
  resource enters the graph, and can be purged under a policy that is a
  setting.
  Done when: a recognized link to an absent target is re-derived when the
  target arrives, and the same test with the index purged first derives
  nothing (the control); rebuilding the index from documents reproduces it;
  purging touches no graph truth (snapshot equal before and after); a SPARQL
  result and a scene projection each carry a coverage note, and a test makes
  each layer fire at least once.
- **P4. Saved queries (rulings 3, 8).** A `Linked` subgraph spec may be a SPARQL
  query beside the nine shapes, reconciled on revision change like the rest.
  Sharing a saved query sends the spec. Freeze mints a node bearing a nested
  graph that holds member resource ids, copies of the statements among them as
  of the freeze revision, the spec and the revision; annotations attach to that
  node. A subgraph itself never syncs.
  Done when: a SPARQL-spec subgraph reconciles on a revision change and skips
  on an unchanged revision; a shared spec evaluated on a second graph yields
  that graph's members; a frozen selection is unchanged after a member
  statement is retracted at the source (and a live `Linked` subgraph over the
  same spec does change, the control); a frozen selection opened where a member
  resource is absent reports it through P3's coverage.
- **P5. Partial residency (rulings 4, 7).** Muniment stores both graphs
  addressably, keyed by id (resource id for resources), so one node and its
  statements load without the whole snapshot. The resident set is k hops (a
  setting) around every surface in an open projection, plus pinned and focused
  items. Anything else loads on demand and reports "not loaded" through P3's
  coverage until it does. Full residency stays a setting.
  Done when: a graph several times the resident set navigates with only the
  neighborhood in memory (a counting allocator or the footprint probe shows
  it); an unloaded neighbor reports "not loaded", never "absent"; loading it
  restores its relations exactly (snapshot-equal to the full-residency load);
  with full residency set, the same tests report no residency coverage; the
  IndexedDB backend passes the same residency tests as redb.

## 5. Checkpoints

Choices with more than one defensible answer that the phases will meet. Each
comes back to Mark as a fork, with evidence, before the code commits to one.

- **C1 (P1). Legacy statements.** What asserter does a stored statement with no
  provenance get on load: a legacy marker, the local user, or the ingest engine
  that probably wrote it. **Resolved 2026-10-04:** Mark selected **"1"**, the
  explicit unknown legacy asserter marker; see the dated C1 ruling in §2.
- **C2 (P2). Which families sit on resources.** Semantic, Imported and
  Provenance are content and Traversal and Arrangement are experience; the
  seven containment sub-kinds split three ways: URL-derived (`UrlPath`,
  `Domain`), resource-level (`FileSystem`, `ClipSource`) and user layout
  (`UserFolder`, `NotebookSection`, `CollectionMember`).
  **Resolved 2026-10-05:** Mark selected **"1"**, the fixed split; see the
  dated C2 ruling in §2.
- **C3 (P2). Migration of existing claims.** A surface's content statements
  were asserted while it showed whichever page it showed then; the store does
  not record which. Attach them to the currently shown resource, to the
  resource of the visit nearest the assertion time, or drop them with a record.
- **C4 (P2). Words.** Names for the two senses of node (resource, surface) in
  TERMINOLOGY and in code.
- **C5 (P3). Purge default.** The pending index's default purge policy.
- **C6 (P4). The frozen node.** What kind of node bears a frozen selection, and
  where it is placed.

## 6. Progress

- **2026-10-04.** Plan written; questions grounded (§2); round 1 ruled
  (rulings 1 to 4); round 2 ruled (rulings 5 to 8, ruling 6 after a comparison
  Mark asked for); phases final; checkpoints C1 to C6 listed.
- **2026-10-04.** Created the requested `graph-semantics` branch and isolated
  worktree at `36893553`; reviewed F1–F9 and began P1 by inspecting assertion,
  persistence and restore paths. Stopped at C1 before code changes. The
  documentation audit's planted-defect/clean-fixture self-test passed; the
  baseline audit exited 0 but reported existing findings (10 index orphans,
  one statusless plan, 40 broken relative links, 206 missing known-root paths
  and two stale historical annotations). The worktree layout also makes
  sibling-path resolution relative to `Code/worktrees` rather than
  `Code/repos`. The graph semantics plan has no audit findings. No new active
  document was added. Cargo tests, the workspace check and the wasm32 check
  were not run: no Rust code or manifest changed before this checkpoint.

- **2026-10-04. P1 partial implementation after C1.** Implemented per-asserter
  assertion identity, Author IRI rendering and scoped writer context, explicit
  page attribution, legacy-marker loading with ids/times preserved, exact
  capture of assertion updates/retractions, Author-aware replay, and separate
  RDF reifiers on one triple. Added writer enumeration, engine-version,
  source-priority, load-fidelity, replay, reingest and drawn-link grouping
  controls. Source changes remain uncommitted while the additional P1 forks
  above await Mark; P2 has not begun.
  Validation, all with the reusable `C:/t/cargo-targets/mere`, offline and locked:
  kernel **320 passed**; linked-data with `query` **40 passed**; Pandect
  **305 passed**; Pictograph with `canvas` **293 passed, 13 ignored** (the
  portable-only run also passed 15). The kernel's one ignored documentation
  example was not run. The wasm32 kernel check exited **0**. The required
  workspace check exited **101**, with one E0063 at the personal-sync writer
  awaiting the peer-identity ruling. No sibling builds, headed UI, GPU/device
  integration tests or later-phase gates were run. No manifest/lock changes,
  dependencies, downloads, isolated Cargo home or local patch override were
  needed. The documentation audit and its planted-defect/clean-fixture
  self-test exited **0**; existing audit findings remain as recorded above.

- **2026-10-05. P1 sync checkpoint.** Resumed after Mark's pause; wired the
  verified stable root into the personal-sync assertion writer and added
  three signed-operation regression tests. The focused run reported
  **1 passed, 2 failed**; the full Graphshell library suite with
  `personal-sync` reported **320 passed, 2 failed, 4 ignored**. Both failures
  are the measured invariant violations in the dated Findings above. The
  required `cargo check --workspace --offline --locked` now exits **0**.
  All Cargo commands used `C:/t/cargo-targets/mere`; no new dependency,
  download, manifest/lock change, patch override or isolated Cargo home was
  needed. The stable-root source edit and failing regressions remain
  uncommitted at forks A and B; the live bulk setter remains unchanged.
  Kernel, linked-data, Pandect, Pictograph and wasm32 gates were not rerun:
  their source has not changed since the previous recorded passing gates.
  Graphshell binaries, integration-test targets, sibling builds and headed
  UI/device gates were not run. The reusable target and isolated worktree
  remain owned by this P1 lane until review and integration. P2 has not begun.
  The documentation audit and its planted-defect/clean-fixture self-test
  exited **0** with the same existing finding counts as the previous
  checkpoint; this plan has no findings. No active document was added.

- **2026-10-05. P1 complete after A1/B1.** Separate attributable assertions,
  precise retraction and exact update/replacement replay are implemented.
  Production writer controls enumerate nine kernel entry points, twelve
  linked-data ingestion cases, page sources, scoped journal/session Authors,
  canvas grouping and signed personal-sync operations. RDF projection/ingest
  retains separate reifiers on one triple; the profile round-trip gate passes.
  Legacy snapshot and journal controls preserve handles, times and known
  sources, supplying the C1 marker where attribution is missing. The live
  bulk setter is retired; real legacy capture decoding is retained.
  A1 carries caller ids/times in signed events and supplies deterministic
  old-event handles; B1 scopes old semantic selector withdrawals to their
  stable root. Conflicting id reuse is refused; new semantic withdrawals
  require exact ids. No serialized capture variant or field was added.

  Final package validation, all offline, locked and using the reusable
  `C:/t/cargo-targets/mere`:
  **kernel 323 passed**, with **one documentation example ignored**;
  **mere-linked-data with query 40 passed**;
  **Pandect 305 passed**;
  **Pictograph with canvas 293 passed, 13 ignored**;
  **Graphshell with personal-sync 326 library tests plus five other tests
  passed, four ignored**. The focused sync run also passed all seven
  regressions. The first linked-data gate invocation used the library name
  instead of its package name and ran no tests; the corrected invocation
  above passed. **Workspace check and wasm32 kernel check both exited 0**.
  The documentation audit and planted-defect/clean-fixture self-test exited
  **0**, with unchanged existing failure counts and no findings for this
  plan. `git diff --check` passed. No active document was added.

  Source and canonical docs are committed together on this branch. Prior
  documentation checkpoints: `8dabea00`, `9ec20ffb`, `64845ccc`. No main
  integration, sibling build, headed browser/UI/device proof or later-phase
  gate was run. Ignored tests remain unrun. No dependency, download,
  manifest/lock change, patch override or isolated Cargo home was needed.
  Retained: `C:/Users/mark_/Code/worktrees/mere-graph-semantics`, owned by
  this lane for Mark's review and eventual integration; shared reusable
  target `C:/t/cargo-targets/mere`, retained for Mere validation. Stopped
  before P2 as required.

- **2026-10-05. P2 inventory, stopped at C2.** Mark authorized continuation.
  Re-read the documentation policy, plan and relevant terminology; inventoried
  containment writers, shared canonicalization and the query seam. Recorded
  C2's evidence and three options above; no P2 code or naming decision made.
  C3/C4 and dependency/lock review remain ahead of implementation. Compiler
  files are held for the stack-seams owner. No source, manifest or lockfile
  changed; Cargo, wasm, consumer and headed gates were not rerun for this
  documentation checkpoint. The isolated worktree and reusable Mere target
  remain owned by this lane for review and continued validation.
  The documentation audit and its planted-defect/clean-fixture self-test
  exited 0; this plan has no audit findings. Existing counts: 10 orphans,
  one statusless plan, 40 broken relative links, 202 missing known-root
  paths and two stale historical annotations. `git diff --check` passed.
  No new active document was added.

- **2026-10-05. C2 resolved; stopped at C3.** Mark selected "1", the fixed
  containment split, recorded verbatim above. Rechecked migration schema,
  timestamp limitations, shared-navigation restoration and session loading;
  returned the three C3 endpoint policies. Handle collisions, C4 naming and
  the shared UUID helper's dependency/lock change remain open. No P2 source,
  manifest or lockfile was changed; Cargo/wasm and consumer gates were not
  rerun. Compiler hold and retained worktree/target ownership are unchanged.
