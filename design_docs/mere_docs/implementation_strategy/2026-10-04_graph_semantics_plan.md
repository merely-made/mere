# Graph semantics plan: assertions, resources, saved queries, residency

**Date:** 2026-10-04
**Status (2026-10-05):** in progress. P1 implemented and gated on
`graph-semantics`, with the ruling-9 exact-journal and legacy-checkpoint
attribution repair complete after the original `459cad84` receipt.
Reconciled main `62219dd1` rulings 9–19 before P2 source edits; the graph
plan is unchanged at main `3b220f90`. A1/B1/C1 selected by "All 1";
`ResourceNode`/`SurfaceNode` are settled by ruling 19, and B1/C1 remain
approved. Mark authorized P2 after the P1 repair at `4bc9ae96`.
P2 inventory is complete at C7/C8; stopped for those rulings before source
edits. Replay-first migration and per-predicate placement govern P2.
P3–P5 have not begun. Main integration awaits Mark's review.

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
38, 55). Chartulary is already in Eidetic's lockfile dependency list.
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
resolve collisions before a policy is implemented. C3's endpoint policy
is resolved below; collision handling remains open.

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

C3 options presented, recommendation first; selection recorded below:

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

**C3 ruling (2026-10-05).** Mark: **"1"**, selecting currently shown
resources with a migration record. Attach old content claims to the
canonical resources their surfaces show at migration; preserve original
endpoints, ids, times and asserters in the record, and mark uncertain
historical page ownership. This does not select a collision survivor.

Regardless of the endpoint policy, collapse is a separate unresolved choice.
The existing fixture has two surfaces at one URL (`graph/tests/snapshot_basic.rs`
547–631). Two distinct old handles can become one dedup key on one resource
pair; normal upsert would lose a handle, while exact restore keeps both.
Same-id/different-payload input would silently reject one record in the
existing helper (`graph/edge_data.rs` 256–292). No such conflict was measured
in Mark's data, and no migration is implemented. Return the collision policy
as a further fork before code chooses a survivor or changes an id.

### Remaining P2 forks (2026-10-05)

**A. C4 naming.** The kernel has one `Node` wrapper and `NodeKey` index
alias (`graph/node.rs` 35, `graph/identity.rs` 32). `SurfaceId` already
names chrome/accessibility elements (`crates/graph/graph-kernel/src/accessibility.rs`
49), so that spelling must not be repurposed. TERMINOLOGY's link amendment
leaves the two node names open. Options, recommendation first:

1. **Resource and surface.** Prose distinguishes a resource from a browsing
   surface; code uses `ResourceNode` and `SurfaceNode`. Keep `Node` and
   `NodeKey` as compatibility names for the existing surface API. The word
   node can describe either graph element; rendered bodies remain gnodes.
2. **Resource and node.** Keep `Node` as the primary name for the browsing
   object; add `ResourceNode` for content. Less API naming change, but the
   prose distinction between a resource node and a node needs qualification.

**B. Migration collisions.** Two same-URL surfaces can supply two distinct
handles for one resource-pair assertion key. Exact-id retraction exists,
but both live assertion routes currently select the first matching content
key even when a carried id differs (`graph/edge_data.rs` 174, 257;
`graph/edge_ops.rs` 346–367, 377). RDF reifiers use statement ids across
the dataset, so conflicting reuse needs dataset-wide preflight. Options:

1. **Preserve handles; stop on conflicting id reuse (recommended).** Keep
   every distinct legacy handle, extending C1's exact-restore exception.
   Ambiguous migrated keys require exact-id edits; refuse content-only
   updates that would select an arbitrary record. Preflight migration and
   stop an affected session if one id names divergent records; keep the
   originals intact. This needs an additive precise edit/replace path.
2. **Collapse with aliases.** Select one active survivor, retain all originals
   and aliases, and define how old-id retractions affect it. This changes
   active identity semantics and requires a survivor rule; caller-supplied
   ids in A1 did not authorize caller-selected merge survivors.
3. **Deterministically remint conflicting records.** Preserve records but
   change conflicting active handles, recording old-endpoints/old-id to
   new-id mappings. Replay and retraction need translation; signed historical
   operations remain unchanged.

**C. Shared identity dependency/lock stop.** Chartulary and Eidetic have no
UUID dependency. The lock already contains UUID 1.26.1 and sha1_smol 1.0.1;
UUIDv5 is already enabled by the kernel. Both package sources and archives
are cached in the default Cargo home. Expected changes are exactly:
add `uuid = { version = "1", features = ["v5"] }` to Chartulary; make
Eidetic's existing chartulary dependency unconditional and leave `lineage`
as an empty feature; add `uuid` to Chartulary's lockfile dependency list.
No new package, version, checksum or source is expected. Options:

1. **Reuse locked UUIDv5 (recommended).** Authorize that dependency-edge
   change and only its matching lock update, resolved offline. Stop if
   resolution requests any download or larger lock delta; no broad update.
2. **Defer the shared helper.** Preserve manifests/lock and leave P2's
   independent resource-id minting gate open. Revisit this seam before P2
   can land.

At that checkpoint none of A, B or C had been selected; no source,
dependency or lock mutation had been made. The later ruling and main
reconciliation are recorded below.

### Main reconciliation and P1 contract finding (2026-10-05)

**Branch follow-up ruling.** Mark: **"All 1"**, selecting A1/B1/C1 above.
A1's names match main ruling 19. B1 preserves distinct old handles, requires
exact-id edits for ambiguous migrated keys and stops conflicting-id migration.
C1 approves only the shared locked UUIDv5 dependency edge and corresponding
offline lock update. This does not settle C7/C8.

Mark then requested a check against current main. Verified main `62219dd1`:
its rulings 9–19 had reached main after this branch's base `36893553`.
Those rulings are incorporated verbatim into §3, with §4's placement table,
phase gates and §5's new checkpoints. Earlier branch choices and receipts
remain historical evidence; the following reconciliations govern new code:

- **C3:** rulings 11/18 select replay-first, not current-resource-for-all.
  Recover the resources shown when journal-reachable claims were minted;
  only baseline/unreachable claims default to current resources, with the
  branch's preserved-original migration record and uncertainty mark.
- **Placement:** the fixed containment split agrees with the table, but
  blanket family placement does not. `UserGrouped`, `CopiedFrom`,
  `BookmarkFolder`, `HistoryImport` and `SessionImport` remain surface-owned.
  Placement is declared per predicate's nature in the registry.
- **Tags:** ruling 15 adds SKOS tag resources and tagging assertions to P2.
  C7 tag-IRI identity and C8 classification placement remain open.
- **Words:** ruling 19 resolves C4; rulings 13/16/17 supply strata, aspect
  and ambient level. No duplicate naming question is needed.
- **Coordination:** the compiler request was intended for the recipe-pass
  owner, not this lane. This lane has no ownership obligation in
  `scenomise::projection`; there is currently no reason to edit it.
  Stack-seams S5/S8/S10 identify workbench forme relations, curation records
  and Pandect as Eidetic's session layer; migrate identifiers when touched,
  without creating an unrelated rename pass.

**P1 mismatch with ruling 9, proven by source inspection.**
`replay_delta_as` substitutes Author on three raw assertion variants but
passes `ReplaySetEdgesByIds` unchanged (`graph/capture.rs` 537–545,
582–592). `set_edges_between` supplies unknown attribution to every missing
source (`graph/edge_ops.rs` 484). The existing test
`legacy_exact_capture_attribution_matches_checkpoint_replay` introduces a
new exact handle in an entry with a known Author, then explicitly expects
unknown and rejects that Author (`graph/journal.rs` 659, 709, 724–727).
Ruling 9 instead requires the minting journal Author for such a handle;
baseline-era claims remain unknown and explicit sources must be preserved.
The prior passing P1 receipts do not prove this revised invariant. No new
Cargo run or source fix was performed during this audit.

**Integration boundary.** A full merge of main would additionally bring
1,050 changed root-lockfile lines and unrelated source/dependency work since
the base. The authorized UUID edge does not authorize a broad resolver update.
This pass reconciles governing documentation only, leaving code/manifest/lock
integration for a separately recorded step. Main's dirty Scrying files are
unrelated and untouched.

**Independent P1 validation reported (2026-10-05).** The plan-authoring
session reported a separate detached-worktree run at `459cad84`, offline
and locked, using `C:/t/cargo-targets/mere`. Counts reproduce the branch
receipt exactly: kernel 323 passed/one doc example ignored; linked-data
with query 40; Pandect 305; Pictograph with canvas 293/13 ignored;
Graphshell with personal-sync 326 library plus five other tests/four
ignored; workspace and wasm32 kernel checks exit 0. Removing provenance
from both dedup keys made the separate-asserter and legacy-insert invariant
tests fail, then the verifier restored the code. These are the independent
session's reported results, not a new run by this lane. Its cited legacy
author test covers raw journal assertions, not the exact-capture case
identified above. Ignored, sibling, headed and device tests remain unrun.
The verifier removed its own worktree and reported approximately 16 GB
additional shared-target output due to its different source path. This
lane did not delete shared build output or create another target.

### P1 exact-journal repair findings (2026-10-05)

The strengthened `legacy_exact_capture_attribution_matches_checkpoint_replay`
test failed against the original P1 implementation: **one run, one failure**,
unknown legacy attribution where the minting engine Author was required.
The explicit-source iteration and baseline-unknown control passed before the
failure. The initial Cargo invocation's unqualified exact filter ran zero
tests; the qualified test was run directly from that freshly built test binary.
Only this lane's own waiting Cargo processes were stopped after that receipt.

`ReplayAttribution` retains the first source for each stable assertion handle,
seeded from the baseline. Exact journal records recover the minting Author
only when no earlier source is available; explicit sources remain intact.
Removing a handle does not remove its attribution, so later restoration cannot
attribute it to the undoing Author (`graph/capture.rs` 797).
`snapshot_at_from` and `replay_from_with_baseline` accept the retained baseline;
the latter scans the journal prefix for carried handles before applying the
checkpoint tail (`graph/journal.rs` 319, 383). Raw historical records did not
capture stable handles; their existing Author-aware replay remains, without
inventing identities absent from the history. Context-free `replay_from`
preserves conservative unknown attribution for exact records lacking a source.

Pandect uses those baseline-aware paths both on reopening and when obtaining
a historical graph (`graph_session.rs` 443, 866). The kernel regression checks
all five checkpoint cursors, including a cursor where the old handles are
absent (`graph/journal.rs` 872). The session persistence regression restores a
missing-source mint after a checkpointed withdrawal by a different persona,
with explicit-source and baseline-unknown controls in the same run
(`graph_session.rs` 1312). No public signature or capture schema is removed
or changed; the two baseline-aware methods are additive.

Read-only review identified a second case: historical checkpoints may already
hold journal-minted handles with missing sources, converted to the unknown
marker at snapshot load. Scanning the prefix alone left these unknown when
the tail was empty. The persisted session regression reproduced this:
**zero passed, one failed**, after the explicit-source and baseline-unknown
controls passed. `repair_checkpoint` now replaces missing/unknown sources
only from the recovered handle ledger before tail replay, preserving explicit
sources and baseline unknowns (`graph/capture.rs` 821). It preserves ids/times,
advances the revision when it changes attribution and emits no capture.
The kernel's empty-tail regression (`graph/journal.rs` 789) and the session's genuinely old stored
checkpoint cover this repair. A second read-only review found no further
concrete blocker. Gate results follow in Progress.

### P2 tag and classification checkpoints (2026-10-05)

Resumed on `4bc9ae96` after Mark's "Proceed". Main is `75e13d8d`; its
graph plan last changed at `62219dd1`, with C7/C8 still open. The branch
worktree is clean before this documentation pass. Naming, predicate
placement, replay-first migration, distinct legacy handles and the narrow
locked UUID dependency approval remain settled.

**C7 evidence.** Tags are exact-string membership, with no concept id,
namespace, tagger or assertion time (`crates/eidetic/chartulary/src/container.rs`
75; `crates/graph/graph-kernel/src/persistence.rs` 160). Three kernel mutation
routes insert one, insert many or remove one (`graph/node_props.rs` 218, 236;
`graph/node_facets.rs` 132). The kernel performs no label normalization;
case and Unicode differences are currently distinct. Two presentation fields
are keyed by label (`types.rs` 662–663). Ingestion promotes three literal
predicates to tags and deduplicates their strings, retaining no tagger on the
promoted tag (`crates/graph/linked-data/src/ingest.rs` 153–158, 646–647).

Tag captures contain only node id and text (`graph/capture.rs` 123–130).
The Author envelope recovers successful journaled additions. Adding the
same string again is a no-op, so an unrecorded second tagger cannot be
recovered. Personal sync likewise carries node/text, but its verified stable
root is available during the fold (`ports/graphshell/src/personal_sync.rs`
128–135, 1274, 1431–1440). A stable mere namespace is available from
`MereId::derive`, UUIDv5 over persona and domain
(`crates/system/pandect/src/reservoir.rs` 108–118).

C7 options, recommendation first. Each preserves existing exact label
distinctions and separates concept identity from the tagging assertion's
asserter. Existing missing taggers remain explicitly unknown; absent history
does not justify attributing a baseline tag to the current user.

1. **Identity namespace (recommended).** A tag vocabulary belongs to its
   stable identity across meres. Equal labels from different owners are
   distinct concepts; a person may deliberately use another owner's concept
   by its IRI. Journal-covered legacy tags use their proven identity;
   unattributed legacy concepts use the original mere's namespace, with
   unknown-attributed tagging assertions.
2. **Mere namespace.** Equal labels within one mere identify one concept,
   with each tagger's assertion separate. The same label in another mere is
   a different concept. Legacy labels stay in their original mere's vocabulary;
   recover tagger attribution where proven, else mark unknown.
3. **Global label identity.** An exact equal label identifies one concept
   everywhere, with separate tagging assertions. This merges different
   owners' meanings automatically. Legacy labels use that global identity;
   tagger attribution is recovered where proven, else marked unknown.

The fallback is part of option 1's proposed policy, not an already decided
ruling. Persistent concept references must remain usable independently of
their display labels; exact IRI encoding and label-edit behavior have not
been selected by this inventory.

**C8 evidence.** Classification records have seven fields, three schemes,
six provenance categories and five statuses: Accepted, Suggested, Rejected,
Verified and Imported. They have no assertion id, asserter, source IRI or
time (`crates/graph/graph-kernel/src/types.rs` 337–425). Dedup uses only
`(scheme, value)`, keeping the first record even when the incoming status or
provenance differs (`graph/node_props.rs` 334–351;
`graph/node_facets.rs` 213–246). Add/remove/status/primary are four mutation
operations with replay forms (`graph/apply.rs` 370–392).

The bounded `crates/` and `ports/` search found one production producer:
the RDF type importer, which records Imported status/provenance
(`crates/graph/linked-data/src/ingest/apply.rs` 48–57, 97–104).
No production acceptance/rejection caller or Rejected-status usage was found.
Four consumers ignore lifecycle: Kind selection (`graph/field_ops.rs` 117–135),
UDC facets (`graph/facet_projection.rs` 155–174), RDF type display
(`graph/display.rs` 112–119), and RDF export
(`crates/graph/linked-data/src/lib.rs` 308–324). A Rejected type would therefore
still export as an affirmative type. Suggested selection has an existing
control (`graph/facet_projection.rs` 319–348); rejection has none. These are
source findings, not a new runtime proof or a ruling on lifecycle filtering.

C8 options, recommendation first:

1. **Resource records (recommended).** Move the complete classifications
   to the resource, retaining lifecycle and primary selection together.
   Every surface showing it reads the same records and review state.
2. **Resource claim, separate review.** Keep the classification proposition
   on the resource, with acceptance/rejection and primary choice in a
   separate reviewer or surface record. Viewers may disagree; this needs
   review identity that the existing record lacks.
3. **Surface records.** Keep independent classifications and review state
   on each surface. Two surfaces may disagree about one resource; how those
   content classifications behave on navigation would require a further ruling.

Placement alone does not select classification-to-statement representation,
lifecycle filtering, or a survivor when two old surfaces contribute conflicting
records for one `(scheme, value)`. These remain implementation checkpoints
if the selected C8 option meets them. Migration must preserve the originals
instead of silently applying today's first-wins dedup. Journal replay can
recover recorder and historical resource placement; provenance categories
alone cannot supply baseline asserter identities.

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

Round 3 put checkpoints C1 to C4 before the lane could meet them.

**Ruling 9 (C1).** *What asserter do existing statements get?* Options: journal
replay, else unknown; journal replay, else the user; unknown for all. Mark:
**"Journal replay, else unknown (Recommended)"**. Follows: a statement minted
inside a session takes the author of the journal entry that minted it
(`AttributedDelta`, with ids captured by `ReplaySetEdgesByIds`), read through
ruling 5, so a link a page makes is the page's. A statement from a baseline
(imported graphs, sessions older than their journal) gets an explicit
"unknown" asserter, shown as such and never merged with the user's.

**Ruling 10 (C2).** *Which statements move to the resource graph?* Options: per
predicate, by nature; per family; Semantic only. Mark: **"Per predicate, by
nature (Recommended)"**. Follows: each sub-kind declares its nature (content
or experience) in the predicate registry the statement kernel brief names, and
placement follows it. The table is in §4, after ruling 14 settled its two-way
rows.

**Ruling 11 (C3).** *Where do existing claims land?* Options: replay, else
current; replay, else hold aside; current for all. Mark: **"Replay, else
current (Recommended)"**. Follows: journal replay attaches each claim to the
resource its node showed when the claim was minted; a baseline-era claim goes
to the currently shown resource, and the migration writes a note listing the
claims that landed by default.

**Ruling 12 (C4a).** *What are the two senses called?* Options: node stays and
"resource" is added; "resource" and "surface" as terms. Mark (free text):
**"R-gnode, for resource graph node, contrasted against just graph node, which
a resource graph depends on right? Should be a name for that relationship… not
like a subgraph or a nested graph, but an aspect of the graph or something.
Idk!"** Answered: the dependency runs the other way (a node shows a resource;
the resource graph stands alone, and only arrival and residency flow from
nodes), and "gnode" is ruled as a node's rendered body (TERMINOLOGY,
2026-07-02), which a resource mostly lacks. Put back as two questions. Mark,
on the name: **"Resource (Recommended)"**. Follows: "resource" names what a
canonical URL identifies; "node" and "gnode" keep their meanings; "browsing
surface" stays a description, not a term.

**Ruling 13 (C4b).** *What is the resource graph to the mere's graph?* Options:
strata; aspects; noumenal and phenomenal; no term. Mark: **"Strata
(Recommended)"**. Follows: a mere's graph has two strata, the resource stratum
(what things are, and the claims about them) under the node stratum (how they
were met: trail, layout, groupings). A node rests on the resource it shows.

**Ruling 14 (C2 rows).** *Which two-way rows go to the resource stratum?*
Options (several allowed): `AgentDerived`; `DependsOn`, `Blocks`, `NextStep`;
`SharedCollection`; `UserGrouped`. Mark: **"AgentDerived (Recommended),
DependsOn, Blocks, NextStep, SharedCollection, That usergrouped distinction
sounds like it would be better with user tags as resources and usergrouping as
layout"**. Follows: the first five go to the resource stratum; `UserGrouped`
stays in the node stratum as layout; user tags move to the resource stratum
(ruling 15 settles how).

**Ruling 15.** *"User tags as resources": which reading?* Options: tags are
resources; tags live on resources. Mark: **"Tags are resources
(Recommended)"**. Follows: a tag becomes a resource with its own IRI (a SKOS
concept, local by default and shareable, able to carry a label and broader or
narrower tags), and tagging is the tagger's statement linking a thing's
resource to the tag's. This takes the stance doc's named candidate, "tags and
classifications as recognized vocabulary types", for tags; classifications
are checkpoint C8.

Round 4 came from Mark mid-round, after ruling 13: "Hm. Maybe aspect could
distinguish the various senses of graph there too, the planes, layers, tiers
and whatnot". An inventory found six ways a mere's graph is divided (strata,
planes, keeping, attention, the socialization tiers, coverage); "aspect" unused
as a term; "layer" in TERMINOLOGY almost entirely architectural; and "tier"
naming two divisions, with a sentence in TERMINOLOGY warning they differ.

**Ruling 16.** *Should "aspect" be the umbrella for the ways a mere's graph is
divided?* Options: umbrella, with each aspect's parts keeping their names;
"aspect" replacing the part words; no umbrella. Mark: **"Umbrella, parts keep
names (Recommended)"**. Follows: TERMINOLOGY gains **aspect**, with a table of
the six, their parts, their shape and where each is ruled. A new way of
dividing the graph enters as an aspect and says how it differs from the
others.

**Ruling 17.** *Which division keeps "tier"?* Options: keeping's bottom step
becomes the ambient level; keep "ambient tier". Mark: **"Ambient level
(Recommended)"**. Follows: keeping's steps are all levels; "tier" names only
the t1 to t4 socialization aspect. Amends the 2026-09-23 wording, not its
meaning; text written before keeps its words, under a dated note. Cleromancy's
and Isocosm's design records also say "ambient tier"; they are outside this
repository and are left for their own sessions.

Round 5, 2026-10-05. The lane carrying this plan works on branch
`graph-semantics`, cut at `36893553`, before rulings 9 to 17 reached main.
It put C1, C2 and C3 to Mark again there, and was about to put C4. Its C1
("1", a legacy marker for missing provenance, with author-aware journal
replay in the code) and its C2 ("1", the fixed containment split) agree with
rulings 9, 10 and 14. Its C3 ("1", the currently shown resource for every
old claim, with a migration record) did not offer replay, which ruling 11
had chosen.

**Ruling 18 (C3 reconciled).** *Which C3 stands: ruling 11 on main, or the
branch's current resource for all?* Options: replay first, as on main;
current for all, as on the branch. Mark: **"Replay first, as on main
(Recommended)"**. Follows: ruling 11 stands. Claims inside a retained journal
land on the resource their node showed when they were minted; only claims the
journal cannot reach (baseline-era) fall back to the currently shown resource.
The branch's migration record (original endpoints, ids, times, asserters, an
uncertainty mark) applies to those fallback cases.

**Ruling 19 (C4 amended).** *Which names?* Options: resource and surface;
resource and node. Mark first asked why resource and surface had been the
first proposal, and said he did not mind using them (free text: **"Hm.
Resource and surface were your first instincts too. Tell me why, and I don't
mind using those."**). Answered: the record's own model calls the node "a
browsing surface" (2026-05-18 brief, 2026-06-05 lineage plan); the pair is
symmetric; and with two strata "node" naturally means any graph element, so
reserving it for one kind needs "resource node versus node" qualification.
Against it: 52 public types use "surface" in the UI sense and `SurfaceId`
already names chrome and accessibility elements (`graph-kernel/src/accessibility.rs`
49); "node" is the product's commonest word. The lane's form, `ResourceNode`
and `SurfaceNode` in code with `Node` and `NodeKey` kept as compatibility
names, answers both. Mark: **"Resource and surface (Recommended)"**. Follows:
amends rulings 12 and 13. Prose says resource and surface; code gets
`ResourceNode` and `SurfaceNode`, with `Node` and `NodeKey` kept as
compatibility names for surfaces; "node" means any graph element in either
stratum; gnode is unchanged; `SurfaceId` is not reused. The strata are the
resource stratum under the surface stratum.

## 4. Phases

### Placement by stratum (rulings 10, 14, 15)

| Family | Resource stratum | Surface stratum |
|---|---|---|
| Semantic | `Hyperlink`, `Cites`, `Quotes`, `Summarizes`, `Elaborates`, `ExampleOf`, `Supports`, `Contradicts`, `Questions`, `SameEntityAs`, `DuplicateOf`, `CanonicalMirrorOf`, `AgentDerived`, `DependsOn`, `Blocks`, `NextStep` | `UserGrouped` |
| Provenance | `ClippedFrom`, `ExcerptedFrom`, `SummarizedFrom`, `TranslatedFrom`, `RewrittenFrom`, `GeneratedFrom`, `ExtractedFrom`, `ImportedFromSource` | `CopiedFrom` |
| Imported | `RssMembership`, `FileSystemImport`, `ArchiveMembership`, `SharedCollection` | `BookmarkFolder`, `HistoryImport`, `SessionImport` |
| Containment | `UrlPath`, `Domain`, `FileSystem`, `ClipSource` | `UserFolder`, `NotebookSection`, `CollectionMember` |
| Traversal | none | all |
| Arrangement | none | `FrameMember`, `TileGroup`, `SplitPair` |
| Tags | tag resources, and tagging statements to them | none |

*Reading, not ruled*: open-predicate statements (no recognized sub-kind, from
JSON-LD ingest and readers) are content and sit in the resource stratum.

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
  two reifiers on one triple term; existing snapshots load, with asserters
  recovered by journal replay and baseline-era statements marked unknown
  (ruling 9), and a test pins both cases.
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
  the per-query `dataset_quads` rebuild in `linked-data::query`. Placement
  follows the table above, declared per predicate in the registry (ruling 10).
  Existing claims migrate by journal replay, else to the currently shown
  resource with a note of the defaults (ruling 11). Tags become tag resources
  and tagging statements (ruling 15). Code and docs say "resource" and "surface", with
  `ResourceNode` and `SurfaceNode` and compatibility `Node`/`NodeKey` names
  (ruling 19); the two strata are resource and surface.
  Done when: two surfaces showing one canonical URL see the same content
  statements; a surface navigating from page 1 to page 2 shows page 2's
  statements while page 1's stay on page 1's resource; a `utm_` variant and a
  fragment variant resolve to one resource; the kernel, ingest and eidetic mint
  the same resource id for one page; a component walk selecting Semantic plus
  Traversal crosses from a surface into the resource graph and back; the SPARQL
  path builds no dataset per call and returns the same rows as the old path on
  the existing query tests (the old path survives only as the test oracle);
  a node's tags read back as tagging statements to tag resources, and two
  nodes showing one resource show the same tags; a replayed journal puts a
  claim made before a node navigated on the earlier page's resource, and a
  baseline-era claim lands on the current resource and in the migration note;
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

- **C1 (P1). Legacy statements.** Ruled: ruling 9.
- **C2 (P2). Which families sit on resources.** Ruled: rulings 10 and 14; the
  table is in §4.
- **C3 (P2). Migration of existing claims.** Ruled: ruling 11.
- **C4 (P2). Words.** Ruled: ruling 19 amends rulings 12 and 13
  (resource and surface; resource and surface strata).
- **C5 (P3). Purge default.** The pending index's default purge policy.
- **C6 (P4). The frozen node.** What kind of node bears a frozen selection, and
  where it is placed.
- **C7 (P2). Tag IRIs.** How a local tag's IRI is minted (a per-mere namespace,
  a per-identity namespace, or a hash of the label), and how two tags with one
  label from different taggers relate.
- **C8 (P2). Classifications.** Whether node classifications (with their
  suggested, accepted and rejected states) follow tags into the resource
  stratum.

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

- **2026-10-05. C3 resolved; remaining P2 forks.** Mark selected "1", current
  resources with preserved originals and uncertain historical ownership.
  Rechecked assertion update/retraction and identity types; recorded C4,
  collision and the concrete dependency-edge/lock proposal together above.
  No P2 code, manifest or lockfile changed; Cargo/wasm and sibling builds
  remain unrun at this documentation checkpoint. Retained worktree and target
  ownership and the compiler hold are unchanged.

- **2026-10-05. All 1 and main reconciliation.** Recorded the branch's A1/B1/C1
  answer, then checked main at Mark's request. Incorporated canonical rulings
  9–19, predicate placement, tag-resource gates, C7/C8 and current terminology
  without deleting branch findings or prior progress. C3 replay-first is the
  reconciled ruling; C4 is already decided. Source inspection found the exact
  journal attribution mismatch with ruling 9 recorded above. Stopped before
  P2 source work because the current phase contract differs. No merge of
  unrelated main source or lock changes, Cargo tests, wasm or sibling builds
  was performed. Main and its dirty files remain untouched. Worktree and
  reusable target remain retained for this lane; independent gates may also
  be using the shared target.
  The independent session then reported matching receipts and a successful
  dedup-defect control, recorded above; those do not close the exact-capture
  attribution gap. Mark reaffirmed `ResourceNode` and `SurfaceNode`; C4 is
  settled and is not reopened.

- **2026-10-05. P1 ruling-9 repair complete.** Recover exact assertion
  attribution from the minting journal Author, retain it through withdrawal
  and restoration by later Authors, and repair old checkpoint attribution
  even with an empty tail. Explicit sources, baseline unknowns, ids and times
  remain intact. Pandect reopening and historical graph materialization use
  the retained baseline. Capture serialization and existing public signatures
  are unchanged; baseline-aware replay methods are additive. Naming remains
  `ResourceNode`/`SurfaceNode`, with `Node`/`NodeKey` surface compatibility
  names when P2 implements the two strata.

  Negative receipts and their positive controls are recorded in Findings.
  The initial new Pandect fixture used a private kernel helper and failed to
  compile; it was corrected to use public snapshots before the persisted
  old-checkpoint negative run. Final validation, offline, locked, with one
  Cargo job and `C:/t/cargo-targets/mere`: **kernel 325 passed**, one doc
  example ignored; **Pandect 306 passed**; **linked-data with query 40
  passed**, including `dataset_round_trip_is_lossless_under_the_profile`;
  **workspace check exit 0**; **wasm32 kernel check exit 0**. Source review
  independently found and then confirmed repair of the old-checkpoint case.
  Main recheck at `3b220f90` found the graph plan's last change remains
  `62219dd1`; unrelated dirty Scrying source/docs remain untouched.

  The documentation audit and its planted-defect/clean-fixture self-test
  exited **0**; existing finding counts remain 10 orphans, one statusless
  plan, 40 broken relative links, 202 missing known-root paths and two stale
  historical annotations. This plan has no audit findings. `git diff --check`
  passed. No active document was added. Source and canonical documentation
  are committed on this branch, awaiting Mark's review; stopped before P2.

  Pictograph and Graphshell suites were not rerun for this narrow repair;
  their prior P1 receipts stand. Ignored tests, sibling builds, headed
  browser/UI/device proofs and later-phase gates were not run. No dependency,
  download, manifest/lock change, patch override or isolated Cargo home was
  needed. One PowerShell formatting launch failed with memory pressure;
  formatting succeeded on retry without profiles, preserving other live
  build owners. Retained: `C:/Users/mark_/Code/worktrees/mere-graph-semantics`,
  owned by this lane for review and eventual integration; shared reusable
  `C:/t/cargo-targets/mere`, retained for Mere builds. No generated output was
  deleted or additional target created.

- **2026-10-05. P2 resumed, stopped at C7/C8.** Mark authorized continuation
  after `4bc9ae96`. Re-read governing docs, checked current main, and used
  two bounded read-only inventories to ground tag identity and classification
  placement. Recorded the evidence and options above. Resource/surface naming
  and earlier approvals are not reopened. No P2 source, manifest or lockfile
  changed. Cargo, wasm, sibling and headed/device gates were not rerun for
  this documentation checkpoint; P1's latest receipts remain recorded above.
  Documentation audit and planted-defect/clean-fixture controls exited 0;
  existing finding counts are unchanged and this plan has no audit findings.
  `git diff --check` passed. No active document was added. The worktree remains
  owned by this lane for P2 and review; the shared Mere target remains for
  validation. No download, dependency, extra target or isolated Cargo home
  was created.
