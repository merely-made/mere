# Graph semantics plan: assertions, resources, saved queries, residency

**Date:** 2026-10-04
**Status (2026-10-09):** P1 and the six bounded P2 closure requirements
are qualified and published to main: source `526f2ddb5`, documentation receipt
`3b3afa289`. Integration includes main `15fe2a943` and its Rapier,
projection-editor, identity/Secret Service and physics-registry updates.
P3 is implemented and qualified on the graph-semantics lane. Full affected
native suites, the locked workspace check, kernel/Pandect/query-feature wasm
checks and the full standalone browser wasm check pass. Fresh source review
cleared both reported defects after observed-negative regressions and repairs.
P3 branch publication is recorded below; its main publication is a separate
checkpoint. Rulings 1–46 remain authoritative; C5/C6 are settled. P4–P5 code
has not begun. Earlier Windows/Linux receipts below are historical.
Consumer repinning and vocabulary/oracle cleanup remain separate under §4.

Four questions were put to Mark from outside the project: what a link records,
what makes two things the same thing, what a saved query can become, and how
much of the graph must be present to use it. Each was grounded against the
tree (§2), then put to him as a multiple-choice round. §3 quotes his answers
verbatim; anything beyond his words is marked *Reading, not ruled*.

Parents: the [statements-over-schema stance](../technical_architecture/2026-05-22_statements_over_schema_stance.md),
the [statement kernel brief](../technical_architecture/2026-06-19_statement_kernel_brief.md),
the [petgraph-RDF plan](../../archive_docs/2026-10-06_completed_plans/2026-06-18_petgraph_rdf_plan.md), the
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

The fallback was proposed with option 1 and approved in ruling 20 on
2026-10-05. Persistent concept references must remain usable independently of
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

### P2 shared identity slice and remaining checkpoints (2026-10-05)

C7/C8 were answered in rulings 20–21. The independent initial slice moves
the existing canonicalizer, unchanged, to `chartulary/src/canonical.rs`,
keeps Eidetic's public reexports and makes its already-locked Chartulary edge
unconditional. Five shared tests pair canonical aliases with meaningful
path/query/authority differences. `graph/node.rs` now declares `SurfaceNode`,
with `Node` a compatibility alias; `graph/identity.rs` adds `SurfaceNodeKey`
and preserves `NodeKey`. No resource graph or migration is implemented yet.

**C9: resource UUID namespace.** The existing surface namespace is fixed at
`graph/mod.rs` 456, and linked-data ingest hashes the raw URL through it
(`linked-data/src/ingest/apply.rs` 74). There is no resource namespace in
the plan or existing helper. UUIDv5 needs both namespace and canonical IRI;
changing either changes every resource id. Options, recommendation first:

1. **Standard URL namespace.** UUID's `NAMESPACE_URL` with the shared
   canonical IRI. Portable without a Mere-specific namespace, and separate
   from the current surface namespace.
2. **Dedicated Mere resource namespace.** Freeze a new namespace constant
   for resources. Same deterministic identity across hosts, but other
   implementations must also carry that Mere-specific constant.
3. **Existing node namespace.** Reuse the surface helper's namespace with
   the canonical IRI. Already-canonical ingested surfaces and their resources
   then have equal UUIDs in different strata; raw noncanonical URLs still
   differ. Every id lookup must carry its stratum.

The approved narrow UUID dependency edge remains held until this answer;
the lockfile is unchanged.

**C10: classification collisions.** The seven-field record lacks an id,
asserter and time (`graph-kernel/src/types.rs` 413–425). Dedup compares only
scheme/value (`graph/node_facets.rs` 213–246; `graph/node_props.rs` 334–351).
A source-derived Python reproduction, not a Rust runtime or saved-data
census, fed Accepted/primary and Rejected/nonprimary records at the same
key: two inputs become one Accepted/primary record; reverse insertion
becomes one Rejected/nonprimary record. Identical-record and distinct-key
controls retain their expected records; different values may retain two
primaries. Status and primary edits currently select by scheme/value
(`graph/node_props.rs` 404–449). Options:

1. **Preserve variants.** Keep divergent records with their original surface
   references, mark the shared resource conflicted, and require precise
   record selection before editing status or primary choice.
2. **Hold conflicts for review.** Migrate unambiguous records; retain
   conflicting groups and origins outside ordinary active classifications
   until reviewed.
3. **Select by an explicit survivor policy.** Preserve originals in the
   migration record but select one active record. Status precedence, primary
   ties and missing journal evidence need further rulings.

**C11: classification lifecycle.** Four ordinary consumers ignore all five
statuses; see C8's source references above. Suggested has an existing
selection test; Rejected has no control and would currently export as an
affirmative RDF type. C8 decides record placement, not filtering. Options:

1. **Affirmative only.** Accepted, Verified and Imported enter ordinary
   selection/display/export; Suggested and Rejected remain review data.
2. **Exclude Rejected only.** Suggested remains in ordinary results and
   affirmative export, preserving its current selection behavior.
3. **Keep all statuses.** Lifecycle remains metadata, with all records
   participating in ordinary results and affirmative export.

**C12: resource journal protocol and consumer seam.** One Turnstone
`CapturedDelta` match is exhaustive (`repos/turnstone/src/behaviors.rs` 104–177); new
variants break that API. Knot-editor has no matching uses. The existing
facet carrier has arbitrary JSON (`graph/capture.rs` 114–118), but replay
currently first resolves a surface (`graph/apply.rs` 888), and undo treats
it as an ordinary facet (`graph/revert.rs` 344–355). Options:

1. **Typed resource captures.** Add explicit resource-record, exact resource
   pair and shown-resource forms; old captures retain their surface grammar.
   Clear typed semantics, but stop for coordinated consumer authorization
   before introducing the known Turnstone API break.
2. **Versioned reserved facet protocol.** Carry resource commands in explicit
   versioned JSON with record kind, stratum and resource ids. Enum consumers
   still compile, but replay, attribution and undo must interpret those
   reserved facets as graph commands, including resource-only records.

Both require wake-up fanout to surfaces showing changed resources. Turnstone
currently collects literal capture ids and resolves only surface ancestry
(`repos/turnstone/src/behaviors.rs` 109, 183); a resource UUID alone does not accomplish that.
Existing edge captures cannot distinguish old surface endpoints from new
resource endpoints (`graph/apply.rs` 1087). UUID or presence inference must
not silently select the stratum. These are read-only consumer findings;
siblings are untouched.

**Validation finding.** Eidetic's optional no-default-feature probe has
104 passing tests and two failures: `json_schema_validates_simple_object`
and `json_schema_rejects_constraint_violation`. The identical 104/2 result
was reproduced with this slice's two Eidetic files temporarily restored to
branch baseline `27f9f428`, then restored byte-for-byte in `finally`.
The unaffected tests assume the `json-schema` feature (`schema_def/tests.rs`
119, 135), while `schema_def/validators.rs` 30–34 correctly reports validator
unavailability without it. Default-feature Eidetic passes all 109 tests,
including both controls. No feature-guard fix was folded into graph semantics;
the optional suite is not reported green.

### P2 identity, lifecycle and replay preparation (2026-10-05)

After Mark accepted C9–C12 option 1, Chartulary gained `resource_id`, UUIDv5
in `NAMESPACE_URL` over its sole canonicalizer (`canonical.rs` 59). The
kernel's `ResourceNode` owns immutable identity/address (`graph/resource.rs`)
and indexes correctly in a Chartulary graph; Eidetic re-exports the same
helper. This is identity plumbing, not production population of the second
graph. The approved dependency edge adds only `uuid` to Chartulary and one
dependency-list line to the lockfile, with no package/version/checksum changes.

`ClassificationStatus::is_affirmative` governs kind selection, classification
facets, type display and affirmative RDF export. Five-status tests include
Accepted/Verified/Imported controls and exact retained review records;
JSON-LD reingest and SPARQL share the export test. The initial new export test
assumed ingest retains an id-only node; a title positive control corrected
that fixture. No ingest behavior was changed. Full resource classification
migration and conflict editing from C10 are not implemented yet.

The final kernel run exposed a real clock-dependent recreation defect:
`Revert::recreate` compared original facets against a scratch birth, so a
same-millisecond visit clock could omit its restoration. A later replay birth
then changed the timestamp. Recreation now writes every original facet and
removes birth facets absent from the original. The deterministic regression
uses planner clock 100, replay clock 101, original clocks 100/90/absent, and
an untouched node at 42. Ordinary undo keeps its before/after/live checks.

The prepared C12 contract appends three typed capture kinds: resource record
by resource id, exact resource pair by both resource ids, and shown resource
by surface id. Optional record/reference values restore prior absence without
selecting blob purge or navigation GC. The old `ReplaySetEdgesByIds` fields
and postcard ordinals stay unchanged. Turnstone's exhaustive capture match
still needs a coordinated update (`repos/turnstone/src/behaviors.rs` 109–177,
292). Resource changes fan out to every surface showing affected resources,
then expand through existing surface ancestry. A resource UUID alone is not
a surface notification. Historical legacy effects use explicit migration
endpoints, never the surface's current URL.

Legacy exact-pair replay needs a surface-pair membership ledger and a
statement-id-to-original-resource-endpoints ledger. Clearing one old surface
pair must retain another pair's assertion handles on the same resource pair;
minting binds endpoints at the historical entry, including withdraw/restore
after navigation. This follows rulings 11/18 and B1, rather than a new
migration policy. The known Turnstone consumer break remains unapplied.
The active Turnstone lane received the concrete coordination contract and
holds its existing Scry pins; siblings remain untouched by this lane.

The read-only persistence inventory found no production rkyv graph-byte
reader/writer in Mere, Turnstone or Knot-editor. Production graph stores use
JSON, but this does not promise old rkyv-byte compatibility. Before resource
population, snapshot save/load, Pandect snapshot composition and Graphshell
selection export must retain the new resource records/references/edges.
Their current old-column-only paths cannot be reused unchanged.

### P2 consumer authorization and further forks (2026-10-05)

Mark authorized the coordinated Turnstone consumer patch: **"authorize, feel
free to communicate with the turnstone agent and/or orchestrate. proceed"**.
The active Turnstone lane owns its behavior-match update and keeps existing
Scry proof pins frozen. Agreed read contract: `surface_ids_showing_resource`
returns sorted, deduplicated surface UUIDs; `shown_resource_id` reads a current
surface binding. Both remain current-state reads, never historical inference.

The typed resource-record DTO can carry canonical IRI and ordinary resource
facets (`facet`, `value_json` strings) without changing its posted shape when
classification variants arrive. A resource-specific FacetStore is the one
live metadata authority; the DTO is its projection. Facets carry data, not
reserved commands. Strings follow existing postcard-safe facet captures.
Exact restoration preflights JSON/duplicate keys and replaces only the named
resource's metadata. Guarded inverse creation does not define cascading
resource deletion, unloading or blob purge.

**C13: unfamiliar predicate placement.** Linked-data ingest is one concrete
production producer with three raw-predicate branches (`crates/graph/linked-data/src/ingest/apply.rs`
115); four public kernel write surfaces accept raw predicates. Seven explicit
runtime handling sites classify them as Semantic. The registry has zero
implemented placement declarations; the statement brief leaves its concrete
shape undesigned. The placement table's open-predicate resource default is
still explicitly *Reading, not ruled*. Known predicate rulings remain fixed.

1. **Resource default, explicit overrides (recommended).** Unfamiliar
   predicates default to resources; an explicit per-predicate declaration can
   select surfaces. Preserve that declared nature across persisted replay.
2. **Require declaration.** Retain unresolved contributions and admit them
   only after placement is supplied; writers gain a declaration requirement.
3. **Always resources.** Every unfamiliar predicate is content; custom surface
   relations require another representation or recognition by the kernel.

**C14: composition and surface identity.** Pandect intentionally remaps B's
same-URL surface to A's first identity (`crates/system/pandect/src/snapshot_merge.rs`
68–122). Four unit tests and one nonempty graph-codicil composition test
encode this; Athanor has one external production caller. Given two codicils
with two surfaces each and one common URL, current behavior retains three
surfaces. C10 already requires preserving divergent resource classifications
and origins; existing whole-facet first-wins cannot serve that merge.

1. **Preserve surface identities (recommended).** Retain four surfaces and
   three resources; merge resources by canonical identity. Amend old URL
   consolidation expectations explicitly.
2. **Keep URL consolidation.** Retain three surfaces; remap the second
   shared-URL surface and its relations/binding to the first, retaining the
   existing A-first conflicting surface facets.
3. **Compose setting.** Support both, with a separately ruled default and
   tests for each mode.

**C15: selection export and tag concepts.** Graphshell has four transfer
scopes, one production filtered-snapshot call (`ports/graphshell/src/product.rs`
698) and one mixed round-trip test (1095). That test includes a tagged file;
tags currently travel in surface rows. Merely adding resource columns would
leak unselected resource data. This product export is distinct from ruling
8's later frozen-query selection.

1. **Shown resources plus tag targets (recommended).** Retain selected
   surfaces' shown resources and concept targets needed by their tagging
   statements; keep exact resource statements among retained endpoints.
2. **Strict induced resource set.** Retain only resources shown by selected
   surfaces and statements among them; omit tags to standalone concepts.

**C16: vocabulary IRI identity.** The sole page canonicalizer removes fragments
(`crates/eidetic/chartulary/src/canonical.rs` 38). The common resource helper
therefore gives `https://vocab.example/#Cat` and `#Dog` one UUID. C7 permits
external concept reuse by IRI; C9 fixes the UUID namespace, not vocabulary
IRI equivalence. Keep the ruled page fragment/tracking aliases and standard
URL UUID namespace in both choices.

1. **Exact vocabulary IRIs (recommended).** Canonicalize document addresses
   for page resources; use the exact vocabulary-term IRI for concept identity,
   including fragment, query and case. A common UUID primitive hashes the
   prepared identity IRI for both.
2. **Normalize vocabulary web addresses.** Also normalize term IRI scheme,
   host and default port, while preserving fragments and every query value.
   This merges differently cased host spellings of one concept, unlike exact
   RDF-term equality. Non-web vocabulary IRIs remain exact.

No dependent predicate-routing, composition, export-closure or vocabulary
identity policy is selected in code at this checkpoint. Neutral resource
storage and typed capture work can continue while answers are pending.

**C17: rejecting invalid resource snapshots.** Review found that the existing
`Graph::from_snapshot` return type cannot report a rejected new resource
record or pair. The current new-column restore loop ignores helper refusals;
a conflicting reused handle can therefore silently lose a pair. Invalid facet
JSON similarly loses the resource and its dependent links. New resource pair
captures also need exact carried statements: legacy aggregate-only semantic
records synthesize fresh handles during conversion and cannot be replayed
idempotently. The typed setter rejects those records; the old surface
fallback is unchanged. Code: `graph/resource.rs`, `graph/snapshot/from.rs`,
`store.rs`, all under `crates/graph/graph-kernel/src`.

The audit found ten production materializers of this kernel graph: two kernel,
five Pandect and three Graphshell call sites. Both native file loaders already
return `io::Result`; session and transfer APIs have corruption/error results.
The rkyv `Deserialize` implementation has only a generic `Fallible` error
bound, so converting a semantic snapshot error there requires an explicit
API decision. The additive checked path can be tested independently.

Options submitted to Mark, recommendation first:

1. Add a checked `try_from_snapshot` returning a clear error; use it at
   fallible load boundaries. Retain `from_snapshot` as a compatibility wrapper
   that stops with that error rather than returning partial truth.
2. Change `from_snapshot` itself to return `Result`, coordinating the wider
   consumer API patch.

The additive checked materializer rejects invalid new resource columns
before creating a graph. Wrapper and load-boundary behavior remains unselected
at C17: old `from_snapshot`, file loaders and rkyv deserialization are unchanged.
The checked API is therefore not yet the production load gate. Its tests
include identical valid duplicates and preserve the entire rejected input.
A final review aligned active surface-handle detection with materialization:
orphan legacy edge records are not active collisions, while uppercase UUID
spellings resolving to existing surfaces remain active. Both controls run
alongside the active-collision rejection in the same test.
No new lexical restriction is imposed on caller-selected statement handles.

### P2 edge ownership checkpoint C18 (2026-10-05)

The next production routing edit exposes a new API choice. `EdgeKey` is a
bare surface `EdgeIndex` (`crates/graph/graph-kernel/src/graph/identity.rs`
38); both graph stores allocate independently, and Chartulary `connect`
delegates to `add_edge` (`crates/eidetic/chartulary/src/graph.rs` 112).
`get_edge` and `find_edge_key` read only the surface store
(`crates/graph/graph-kernel/src/graph/edge_ops.rs` 528, 533).
Five mixed assertion APIs return this key, including the statement writers
(316, 346) and apply helpers (1875, 1897, 1909);
`GraphDeltaResult::EdgeAdded` also carries it
(`crates/graph/graph-kernel/src/graph/apply.rs` 481).

An executed temporary kernel probe created surface `UserGrouped` and
resource `Cites` edges, both at raw index 0. In the same run both proper
store readers passed. Passing the resource index to the existing surface
reader then failed its Cites expectation: exit 1, one expected failure.
The probe was removed and the original source restored byte-for-byte
(SHA256 `9BCCBBF6588608A502CBFE306C8C5CDDB9091B71D18E24F792FA427907A7C536`).
This is evidence for the checkpoint, not a failing committed gate.

The Mere audit found seven production `get_edge` calls: six semantic/label
readers require resource-aware reads; one traversal reader stays surface.
The six are `crates/graph/graph-kernel/src/graph/display.rs` 81,
`crates/graph/linked-data/src/lib.rs` 362/442,
`crates/canvas/pictograph/src/canvas/selection.rs` 335,
`crates/mere/src/roster.rs` 664 and
`ports/graphshell/src/personal_sync/assertions.rs` 158.
The traversal reader is `crates/graph/graph-kernel/src/graph/apply.rs` 710.
These counts concern production Rust, excluding tests.
Six direct/scoped external statement writers discard their returned key.
The authorized Turnstone peer inspected all 142 tracked Rust source files at
`c3b14cb`: zero EdgeKey, EdgeAdded, GraphDeltaResult, get_edge or find_edge_key
callers; four generic relation writes and two Canvas writes discard results.
Its pins, production sources and 152 frozen Scry inputs remain unchanged.
This read-only audit does not requalify the earlier supplier receipt.

1. **Typed outer handle (recommended).** Keep surface `EdgeKey` and its
   surface readers; add an opaque `ResourceEdgeKey` and
   `RelationKey::{Surface, Resource}` for mixed assertion APIs and EdgeAdded.
   A separate handle reader and resource/projected relation iterators read
   the single owning store. No implicit resource-to-surface conversion.
2. **Separate routed writers.** Keep old writers surface-only and add routed
   APIs returning the typed outer handle. Migrate all production content
   callers; old writers must reject resource predicates, adding compatibility
   names and a risk that unmigrated callers cease recording content.
3. **Tag EdgeKey itself.** Replace the alias with a discriminated key and
   make get_edge dispatch by stratum. This commits to broader petgraph
   conversion changes, including surface traversal/query internals.

C18 is open. No handle API choice or production routing has been implemented.
The independently authorized C14–C17 changes can be gated before this stop.

### Opaque assertion handles and RDF checkpoint C19 (2026-10-05)

Kernel handles remain arbitrary strings, including whitespace, under A1/B1
and the checked snapshot controls. The existing RDF emitter concatenates
`urn:mere:statement:` with the raw handle, then silently returns if that is
not a valid NamedNode (`crates/graph/linked-data/src/lib.rs` 187, 214).
The base triple survives but its reifier, attribution and time disappear.
Ingest knows only suffix stripping under that prefix
(`crates/graph/linked-data/src/ingest.rs` 337, 559–562).
Without a reifier it receives no carried handle, source or time; the apply
path then mints a fresh assertion instead of preserving the caller's handle
(`crates/graph/linked-data/src/ingest/apply.rs` 129–139).

A temporary probe used two checked resource fixtures in one run. For
`valid-handle`, base/reifier/author/time counts were 1/1/1/1; ingest retained
that id, `https://probe.test/author` and time 42. For a handle containing a
newline, counts were 1/0/0/0 and ingest's id/author/time were all absent.
The final expected-reifier assertion failed (exit 1, one expected failure,
44 filtered). Both base triples and the valid metadata were positive controls.
Source was restored byte-for-byte, backup removed: SHA256
`461cb6ce3f374a11f49277e4c7e496d13920c38a8c165b27990ce4f66b5fe42d`.
This pre-existing gap reopens general round-trip qualification; the existing
44-test gate covers its present fixtures, not arbitrary opaque ids.

1. **Preserve valid URNs, encode unsafe handles (recommended).** Retain every
   valid legacy reifier IRI. Unsafe handles use reversible UTF-8 byte encoding
   under a distinct versioned namespace outside `urn:mere:statement:`;
   ingest recognizes both. A disjoint namespace prevents collisions with
   legacy handles resembling encoded suffixes. Existing valid exports remain
   stable, and arbitrary caller-selected handles round-trip exactly.
2. **Encode every handle.** Emit all handles under the new reversible
   namespace and ingest both forms. This simplifies the new output grammar
   but changes every existing reifier IRI and normalized export.
3. **Fallible export.** Explicitly reject unsafe reifier handles at checked
   RDF export/query boundaries while retaining unrestricted kernel handles.
   This preserves legacy output but prevents those assertions from RDF
   exchange until the caller resolves the error; wrapper error policy is
   also required. Silent omission is not an option.

C19 is open. No encoding, kernel lexical restriction or export error policy
has been chosen. Production routing and general RDF round-trip qualification
stop at C18/C19; independent checked loads and identity/export seams stay
reviewable on the isolated branch.

### Resource-bearing import boundary (2026-10-05)

Review of the new resource-preserving export found that the old kernel
`import_edits` emits only surface/facet/pair captures
(`crates/graph/graph-kernel/src/graph/merge.rs` 40). Its one production caller,
Graphshell product import, would otherwise report success while omitting all
three resource columns (`ports/graphshell/src/product.rs` 720).
Resource-aware import remains part of the held P2 integration. Until then the
kernel compatibility importer stops on incoming resources, and the fallible
product path refuses before live edits or journal writes. The same codicil can
be opened as a new session with its exact resources, assertions and bindings;
resource-empty legacy imports remain supported. Tests guard both refusals with
those positive controls. This guard is not a resource merge implementation.

### Typed edge handle implementation findings (2026-10-06)

`ResourceEdgeKey` is an opaque wrapper with crate-private raw access;
`RelationKey` names its owning stratum (`crates/graph/graph-kernel/src/graph/identity.rs`).
Mixed assertion writers and `GraphDeltaResult::EdgeAdded` now return that
outer key; their current writes are explicitly tagged Surface. Surface
`EdgeKey`, `get_edge` and `find_edge_key` remain surface-only.

The new `graph/relation_read.rs` in that crate dispatches `get_relation` to
one owning store and exposes typed resource edges and projected directed
pair/incoming/outgoing iterators. They preserve parallel edges and lift
resource links through explicit shown-resource associations without mirrored
surface payloads. Kernel display roles now read that projection. This seam
does not populate resources, route predicates or implement resource retractions.

Three invariant tests cover both stores at raw index 0 with distinct content,
an absent resource index beside a present surface control, parallel/reverse
edges, self-loops, binding fanout/unbinding and projected display alongside
the old surface display. A compile-fail doctest guards ResourceEdgeKey passed
to the surface getter. Existing assertion tests retain their invariants while
reading mixed-return keys with `get_relation`. The first full test compile
caught one additional query fixture passing a mixed key to `get_edge`; it
was corrected to `get_relation` without weakening its row assertions.

Read-only caller review found zero production result inspectors outside kernel
apply, six external semantic writer sites discarding their results, and no
new handle consumers in Turnstone's earlier 142-file inventory. Remaining
semantic mutation readers stay on their current surface path until predicate
routing and exact resource retraction are implemented. The governed placement
table has 44 closed kinds, 32 resource and 12 surface. No new routing choice
is implied by this sequencing; P2 routing is already authorized.

### Malformed encoded assertion IDs checkpoint C20 (2026-10-06)

Ruling 32 chooses reversible encoding for unsafe String handles. The authored
codec (`crates/graph/linked-data/src/reifier.rs`) uses
`urn:mere:statement-id:v1:` with lowercase hexadecimal UTF-8 bytes,
disjoint from the legacy namespace. Three valid RDF reifier IRIs illustrate
invalid encoded payloads: suffix `0` (odd length), `gg` (nonhex), and `ff`
(invalid UTF-8). None can recover the original String handle.

Existing ingest recognizes only the legacy prefix and treats other reifiers
as foreign (`crates/graph/linked-data/src/ingest.rs` 559–562): no carried id,
but label/source/time survive. The apply path mints a new assertion handle
(`crates/graph/linked-data/src/ingest/apply.rs` 145–158). Owning the new format
requires a choice about malformed input; unrelated foreign reifiers remain
unchanged in both options.

1. **Reject malformed recognized IDs (recommended).** Return an ingest error
   for invalid hex or UTF-8 under the exact reserved v1 namespace. No partial
   contribution is returned and the caller can retain/review the input.
   Adds a restriction only to malformed reserved-format input.
2. **Treat as foreign.** Preserve label/source/time but mint a new assertion
   id, matching prior external-input behavior. The inability to recover the
   claimed carried identity becomes an intentional fallback.

Unknown namespaces/versions continue through the existing foreign path.
C20 has been put to Mark; ingest policy wiring waits for the answer. Helper
and typed edge work can be verified independently. No policy is selected here.

Executed control: a temporary test fed all three malformed v1 names through
the unchanged ingest path. Each returned one edge, no carried statement id,
author `https://probe.test/author` and time 42. All three metadata controls
passed alongside the codec's valid/legacy round-trip controls: four tests
passed (three permanent helper tests and the one probe). Removed the probe
and its backup after exact helper restoration, SHA256
`2afdf5ea4bbc805586aaccc23ce4736b8ac3a49cbd0ed0daa5e71bfd525abc97` before
formatting. The codec remains test-only; production export/ingest retain
their original behavior while C20 is pending. This does not repair or qualify
the arbitrary-handle dataset/reingest path yet.

### Opaque assertion RDF integration findings (2026-10-06)

- **Exact handles in production.** The shared codec is now compiled in
  production and export calls its encoder (`crates/graph/linked-data/src/lib.rs`
  46–48, 214). Valid legacy reifier IRIs stay byte-for-byte stable. Unsafe
  String handles encode as lower-hex UTF-8 in the disjoint v1 namespace;
  empty and ordinary Unicode handles still use their valid legacy IRIs
  (`crates/graph/linked-data/src/reifier.rs` 15–53).
- **Failing reserved input is atomic.** Ingest decodes both formats while
  collecting reifiers in pass A. Invalid v1 hex/UTF-8 returns an explicit
  existing `IngestError::Parse` before a contribution can escape; the parser
  has no live graph reference (`crates/graph/linked-data/src/ingest.rs` 393–399).
  Foreign IRIs and unknown versions retain their existing no-carried-ID path.
  The duplicate legacy prefix and export formatter are removed.
- **Production-path controls.** The malformed-input test covers three error
  classes in both valid/invalid stream orders, with legacy, encoded, foreign
  and unknown-version positive controls. Ten independently attributed IDs on
  one triple retain exact handles, source, time, label and User scope through
  quad ingest, N-Quads, TriG, apply and normalized dataset comparison. Both
  surface and explicit resource projections are exercised. Borrowed SPARQL
  sees every exact reifier, with a Default-scope negative beside the User-scope
  positive. Two unsafe literal handles preserve datatype/language, provenance,
  time and Source/User scope through file parsing and application
  (`crates/graph/linked-data/src/reifier.rs` 167, 205, 376).
- **Bounds unchanged.** These fixtures use valid source IRIs and representable
  millisecond timestamps. Invalid source IRIs and times outside the serializer's
  range retain their previous omission behavior (`lib.rs` 191–238, under the
  same linked-data source root). This repair qualifies arbitrary String IDs
  under the existing RDF profile, not metadata outside that profile. Applying
  contributions still uses the existing surface writer pending P2 routing;
  exact RDF identity fidelity does not qualify resource placement on ingest.

### Custom placement registry ownership checkpoint C21 (2026-10-06)

The settled placement table covers 43 recognized non-traversal sub-kinds
(17 Semantic, three Arrangement, seven Containment, seven Imported and nine
Provenance); traversal stays on surfaces. Thirty-two are resource kinds and
eleven are surface kinds. There are zero implemented predicate-placement
registry declarations (`crates/graph/graph-kernel/src/graph/edge_taxonomy.rs`,
`edge_data.rs` 384). The statement brief explicitly leaves the registry's
concrete shape open (`../technical_architecture/2026-06-19_statement_kernel_brief.md`
162). Ruling 26 fixes the unfamiliar-predicate resource default and permits
explicit per-predicate surface overrides; it does not settle who owns those
overrides for subsequent writes after reopening elsewhere.

Exact pair captures already preserve a historical statement's owning store
(`crates/graph/graph-kernel/src/graph/capture.rs` 261–274). The snapshot stores
both strata and shown associations, with no predicate registry column
(`crates/graph/graph-kernel/src/persistence.rs` 288–310). Mere's host registry
depends on the kernel, so the kernel cannot depend back on that registry
(`crates/system/registry/Cargo.toml`). Either choice needs a kernel-level
placement input; persisted declarations additionally need durable ownership.

1. **Persist declarations with the mere (recommended).** A custom predicate's
   declared nature travels with the mere and governs subsequent writes after
   reopening in another host. Composition must retain and resolve conflicting
   declarations; its conflict policy remains a separate design concern.
2. **Installed host registry.** Local installed declarations govern new
   writes. Exact captures preserve historical placement independently, but a
   later assertion can land differently when the mere opens in another host.

C21 is put to Mark before mutable override installation. The fixed table and
legacy/live replay split do not depend on that answer. Review found two raw
legacy replay arms delegate to live assertion arms (`graph/apply.rs` 615–638
and 1619–1642, under `crates/graph/graph-kernel/src`). Before live routing,
keep those old operations explicitly surface-only; migration must recover
mint-time endpoints separately under ruling 11. New resource writes use the
existing typed exact resource-pair captures rather than reinterpret old
surface grammar. No routing or registry policy is implemented at this point.

### Fixed placement catalog findings (2026-10-06)

`crates/graph/graph-kernel/src/graph/predicate_registry.rs` owns the settled
built-in placement table. `GraphStratum` distinguishes Resource and Surface;
`built_in_relation_stratum` exhaustively covers all 43 recognized sub-kinds
(32 resource, eleven surface), plus traversal on surfaces and the open-predicate
resource default. `built_in_predicate_stratum` recognizes the 17 canonical
Semantic IRIs and exact tagging predicate; `default_predicate_stratum` adds
only the unfamiliar-resource fallback (lines 18, 25, 87, 96).

Two tests enumerate every known kind against the ruled surface set and count
both strata, with traversal/open controls in the same run. Predicate tests
check every canonical Semantic IRI, tags, resource dependencies and shared
collections, contrasting exact known IRIs with unfamiliar strings and a cased
lookalike (lines 107, 153). Defaults are explicitly not an effective resolver
for custom per-mere declarations. The only other source edit re-exports this
module from `graph/mod.rs`; no live caller routes through it yet. No capture,
snapshot, dependency, registry back-edge or mutable override grammar changed.
This installs the fixed table independently of the two open policy forks.

### Durable placement seams and checkpoints C22–C23 (2026-10-06)

C21's per-mere authority can use ordinary typed metadata on an exact-IRI
predicate resource (`ResourceNode::for_term`), persisted in
`PersistedResourceRecord` and captured by `ReplaySetResourceRecordById`.
This uses zero new snapshot columns and zero new public capture variants
(`crates/graph/graph-kernel/src/graph/resource.rs` 46, 118;
`graph/capture.rs` 261; `graph/revert.rs` 337, under the same kernel root).
There is no general graph-level metadata slot; surface facets require a real
surface owner and separate sidecar. Predicate resources fit the already
ruled exact vocabulary identity and ordinary resource metadata authority.
This is an implementation seam, not a selected declaration conflict policy.

**C22: conflicting declarations on composition.** Two inputs can declare
one custom predicate as Resource versus Surface. Current composition returns
an error when one resource facet key has two unequal values; equal values
compose, and disjoint keys retain both values. The existing test checks one
conflicting pair, equal metadata and two retained disjoint keys, while both
source records remain intact (`crates/system/pandect/src/snapshot_merge.rs`
162–187, `resource_conflicts_return_errors_with_equal_and_disjoint_controls`
485). A variant-bearing declaration facet could keep both values in a valid
combined resource record, but its effective-write behavior is not ruled.

1. **Retain variants, require a choice (recommended).** Keep the declarations
   and their origins; require explicit selection before new writes using that
   predicate. Exact historical claims and unaffected predicates remain usable.
2. **Reject composition until reconciled.** Keep both input meres intact, but
   return a conflict rather than create the combined mere. This reuses the
   existing atomic composition boundary and does not select a survivor.

**C23: changing a custom predicate's nature after claims exist.** Exact
surface and resource captures already name different owning stores. Resource
pair preflight rejects reuse of one handle across active stores
(`crates/graph/graph-kernel/src/graph/resource.rs` 230–291); precise live
retraction still reads only the surface pair (`graph/edge_ops.rs` 382,
under the same kernel root). A declaration edit therefore cannot implicitly
move held statements or make its current nature the sole handle locator.
Two strata can hold historical assertions of a predicate without rewriting
the original capture grammar, but later declaration changes need a rule.

1. **Future assertions only (recommended).** Apply the changed declaration
   to newly created assertions. Existing handles keep their recorded placement;
   exact replay, updates and undo preserve it. Reads and precise retractions
   search both strata rather than trust the latest declaration alone.
2. **Forbid changes while claims exist.** Keep the declaration stable while
   that predicate has held claims. Changing it later requires a separate,
   reviewed migration; no automatic movement is selected here.

C22 and C23 are put to Mark together before mutable declaration policy lands.
The built-in catalog and durable storage inspection proceed independently.
Selection export must also retain the declaration resources needed by retained
assertions; today's closure includes shown resources and tag targets only
(`ports/graphshell/src/product.rs` 832). This follows from C21's authority
traveling with the mere. The session-import guard continues to refuse resource
inputs pending its already-held recorded resource import path
(`crates/graph/graph-kernel/src/graph/merge.rs` 39). No guard is weakened here.

### Resumption and integration checkpoints C24–C27 (2026-10-06)

At resumption, the paused source was on `graph-semantics` at committed
`b46b53f8` plus uncommitted P2 work. Independent source review found two bounded bugs rather
than new policy choices: content-key reassertion updates only one identical
handle copy in parallel resource buckets (`graph/assertion_write.rs` 458),
and Pandect treats orphan legacy surface rows as active handle owners
(`crates/system/pandect/src/snapshot_merge.rs` 236), unlike checked loading
(`graph/snapshot/checked.rs` 213). The writer regression is strengthened before
repair; composition now uses materialized endpoints, with active-collision,
orphan, malformed and distinct-handle controls. New gate receipts are pending.
Kernel paths in this finding are under `crates/graph/graph-kernel/src`.

**C24: old aggregate relations after withdrawal/restoration.** The 23
Provenance, Imported and Containment kinds store sets of kinds, with no
assertion handle, time or asserter; 16 have ruled resource placement and seven
surface placement (`graph/edge_data.rs` 442–453;
`graph/predicate_registry.rs` 32–90). The outer journal carries an Author,
but aggregate captures identify only a surface pair and kind or whole payload,
without an undo-versus-new intent (`graph/journal.rs` 187;
`graph/capture.rs` 50, 58, 224). Placement is settled; identity on reappearance
is not. No census of Mark's saved graphs is claimed.

1. **Stable legacy pair/kind identity (recommended).** Preserve the first
   observed resource endpoints through absence/restoration. This preserves
   restoration of an old claim, but cannot identify a fresh same-kind claim
   after navigation from those old bytes.
2. **Fresh identity on each return.** Each absent-to-present interval binds
   to the resource shown then. Fresh reassertions fit, but an indistinguishable
   undo can rebind an old claim to a later page.
3. **Explicit authored promotion.** Retain legacy aggregates on surfaces
   until promoted; preserves inputs but delays the 16 resource kinds and P2.

**C25: shown associations in component/Ego walks.** Ruling 6 already requires
both strata; shown-only adjacency cannot satisfy P2. The remaining choice is
hop cost. Executed model, not a kernel receipt: A and A' show RA; RA has two
Cites edges through unshown RB to RC shown by C; C has a Traversal to D.
Current surface BFS reaches only A at every radius; C reaches D at radius one
as the positive control (`graph/query.rs` 753, 782;
`graph/relation_read.rs` 117, 151).

1. **Zero-hop shown joins (recommended).** Ego zero includes A and A'; two
   relation hops reach C and three reach D. Components include A, A', C and D.
   Radius measures claims/traversals, rather than the representation binding.
2. **Count each shown join as a hop.** Ego zero retains only A; A' first
   appears at two hops, C at four and D at five. Components are unchanged.

**C26: first materialization of RDF subject identity.** Ruling 29 fixes
canonical pages versus exact term IRIs. `NodeContribution` carries arbitrary
types and an id but no page/term intent (`crates/graph/linked-data/src/ingest.rs`
43, 91, 178). Scheme classification cannot supply it. The frozen fixture in
`crates/graph/linked-data/src/ingest/tests.rs` 25 expects four untyped RDF
subjects to create four distinct surfaces but two page bindings: foreign #A
and #B collapse, while ordinary page aliases correctly share a resource.
Two added exact-term records remain distinct. This fixture passes in the
51-test linked-data/query Cargo run; the identity-input repair remains held at
C26. Existing records already preserve the prepared IRI and need no new replay
identity mode.

1. **Explicit intent, exact RDF default (recommended).** Add per-subject
   intent at the apply boundary; generic foreign RDF defaults to exact IRIs,
   while page extraction explicitly names page subjects. Supports mixed input
   without changing the contribution or capture grammar.
2. **Require intent for every new subject.** Refuse unresolved inputs before
   mutation; each caller must classify every new subject.

**C27: automatic legacy migration activation.** A modern custom surface
claim with a later Resource declaration can have the same bytes as an old
surface claim. C23 requires preserving the former store. `GraphSnapshot` has
zero version/profile fields (`persistence.rs` 288–310), and bare JSON stores
read it directly (`crates/system/pandect/src/session_graph_store.rs` 78–88).
Codicil v1/v2 distinguish facet sidecars, not placement
(`crates/system/pandect/src/graph_codicil.rs` 39–40, 58–65).

1. **Durable placement profile at the session/codicil boundary (recommended).**
   Record whether input is legacy or already migrated; use an explicitly
   caller-qualified legacy adapter until that profile is present. Do not infer
   historical store from current declarations or resource-column presence.
2. **Placement format on every GraphSnapshot.** Add a version field and
   propagate it through all producers/composition, coordinating consumer
   struct-literal breaks before integration. Missing legacy values need a
   defined policy.
3. **Caller/load setting only.** Select legacy input explicitly without a
   durable profile; transferred bare snapshots remain ambiguous.

These checkpoints are put to Mark before implementation selects an answer.
The bounded Semantic replay adapter and already ruled declaration/canvas work
may proceed independently; ordinary activation remains held.

### Mixed raw/carried lineage checkpoint C28 (2026-10-06)

Raw journal assertions lack a carried id; that documented compatibility
limitation remains unchanged. A later exact payload can introduce an id with
the same predicate, scope and asserter after navigation. Those bytes do not
prove whether the carried handle continues the earlier claim or introduces a
distinct copy. B1 forbids silently aliasing distinct handles; it does not
require rejecting distinct ids merely because their content matches.

The source fixture at `graph/legacy_resource_migration.rs` 1530 contains three
entries: raw Cites at the old page, navigation, and an exact payload with id
`carried`. Its read-only probe expects one diagnostic with indices 0 and 2,
two possible resource endpoints, and an unchanged journal. A distinct
predicate and a distinct explicit Author are zero-diagnostic controls;
the unknown-asserter marker remains uncertain. This test passes in the
resumed 395-test kernel/store executable run.

1. **Explicit endpoint resolution (recommended).** Retain the original
   history and require an explicit resource-origin choice for each ambiguous
   carried handle before activating that migration. Keep its exact id; do not
   alias it to a generated raw handle. Unambiguous histories can proceed.
2. **First carried appearance is a new claim.** Bind the carried handle to
   the resource shown at that entry. Raw and carried identities stay distinct,
   but an actual continuation can be bound to a later page.

No diagnostic currently chooses either policy. Automatic activation remains
held along with C27; exact-carried prefixes and raw-only author/endpoints are
qualified separately.

### Main archive handoff drift (2026-10-06)

Read-only comparison with main `26857eaf` finds new received notes in its plan
at lines 426–430 and 444–450, absent from the earlier `d2d6ac3d` scope. Stack
seams ruling S53 transfers the archived petgraph-RDF open items to this plan;
S68 keeps its Phase 4 in the separate backlog. No replacement of rulings
1–36, merge, main edit or additional implementation is performed here.

The inherited P2 obligations include JSON-LD named-scope and statement-metadata
shaping, absent/default metadata omission and snapshot-size controls, review
of the unaccepted ExampleOf/Summarizes vocabulary proposals, and removal of
the older Oxigraph dependency/oracle. The prior-production spareval
`sparql_materialized` oracle remains the P2 parity control
(`crates/graph/linked-data/src/query.rs` 84–96, 129);
removing Oxigraph does not require removing that control. Any needed lock
change remains a stop under this lane's rules.

Raw-IRI Semantic support and the borrowed QueryableDataset are already
covered by this lane. The archived Phase 3 status question is still open;
its explicitly deferred term-dictionary on-ramp is not a new P2 requirement.
CONSTRUCT/DESCRIBE graph results belong to P4 under the existing ruling 3.
The new JSON-LD/serialization obligations materially enlarge P2; record this
drift and stop further implementation as the lane brief requires. Existing
source qualification can finish while C24–C28 remain pending. Vocabulary
alignment must return as its own evidence-backed fork before any new mapping.

### Replay and canvas qualification findings (2026-10-07)

The legacy predicate setter changed visible payload without advancing the
graph revision (`graph/edge_ops.rs` 573). It now advances that revision only
when the complete payload changes, retaining the existing-edge return value
and all held assertion metadata. Two kernel tests cover predicate edits,
clears, attribution-only repair, repeat inputs and missing edges. The canvas
test `strategy_cache_tracks_replayed_raw_predicate_edits_with_identical_classifier_rows`
checks the public replay path after warming the layout cache.

Resource creation and shown bindings correctly advance graph truth, even
when the visible relations stay unchanged. The private canvas strategy memo
therefore compares surface identities, projected rows including multiplicity,
and their complete owning payloads once per graph revision
(`crates/canvas/pictograph/src/canvas/strategy_inputs.rs`). Steady frames reuse
the memo. Graph replacement clears it. Walk-dependent inputs will need an
extension when C25's consumers change; this does not qualify their current
surface-only traversal.

Runtime checks exposed two existing footprint lifecycle gaps: construction
left the dependency baseline empty, and restoration claimed its cache hold
before applying saved face sizes. Construction now records the resolved sizes
without simulation work (`crates/canvas/pictograph/src/canvas/lifecycle.rs`
236); restoration claims its hold after geometry is final
(`crates/canvas/pictograph/src/canvas/strategy.rs` 306). Tests retain unchanged
extents across no-link navigation, invalidate on a real resize, retain restored
layouts until visible inputs change, and cover distinct assertion insertion,
duplicate input, precise retraction and equal-revision graph replacement.
The five-surface projection fixture uses distinct surface hosts so checked
snapshot rebuilding does not introduce unrelated derived Domain rows.

The opt-in migration adapter initially documented the C28 hold but did not
enforce it: a direct call could bind the carried handle to the current page.
It now preflights the read-only diagnostic and reports checkpoint C28 before
constructing migrated output (`graph/legacy_resource_migration.rs` 198).
All three additive journal APIs delegate through this guard. The mixed fixture
checks refusal, unchanged graph, revisions, Author/session, journal and recorder,
then uses a real title edit to prove the retained recorder still works.
Distinct explicit Author and predicate controls migrate successfully. This is
enforcement of the pending checkpoint, not a lineage resolution policy; C28
is still unanswered. Detection depends on the supplied retained history and
cannot recover raw events absent from it.

### Resource codicil import qualification (2026-10-07)

The first full Graphshell run passed 326 library tests and failed six: one
staged transfer, the mixed scene/codicil round-trip and four H6 transfers.
Each reached the temporary resource-bearing import refusal; the source log
is retained as `graph-semantics-graphshell-resource-merge-failures.log` in
the shared Mere target. Valid resource import must compose both strata,
rather than discard resource columns or refuse the already ruled union.

`product_import_edits` (`ports/graphshell/src/product.rs` 774) now applies
C1's unknown-source marker without dropping raw records, then checks global
handles before materialization can coalesce a conflicting surface record.
It uses Pandect's checked union for resources and preserves the existing
surface-field and incoming-facet import rules. Resource records, exact pair
payloads and shown bindings are prepared in dependency order. Quiet scratch
replay must reproduce the resource union and every carried assertion at its
exact endpoints and store before one live journaled change is applied.
The kernel's legacy surface-only importer retains its resource refusal.

Two Product tests (lines 1583 and 1697) cover same-page distinct surfaces,
exact handles, persistence and undo, empty-resource compatibility, no-op
reimport, malformed records and conflicting handle reuse. Every refusal
checks unchanged graph, journal and change count beside valid controls.
Missing legacy sources normalize to C1 without changing explicit sources,
ids or times. Independent read-only review found no actionable defect.
The first focused Product run passed seven tests and failed two fixtures:
one tried to declare fixed built-in taggedWith placement; the other expected
only a semantic row where selected same-host surfaces also retain derived
Domain containment. The custom-declaration control now uses a custom
predicate, with the built-in refusal retained. Export checks all selected
surface relations, explicit semantic inclusion/exclusion and valid containment,
preserving the same-page shared-resource fixture. The unchanged production
helper is rerun through the full suite; the failed log is retained as
`graph-semantics-graphshell-product-fixture-failures.log`.
The next full run passed 332 library tests with one declaration-control
failure and four ignored. All six original import/transfer failures passed.
That final control compared raw struct-ordered JSON against stored JSON values;
resource loading parses facets to `serde_json::Value` and snapshot output
serializes those values (`graph/resource.rs` 117, 161). The corrected control
checks exact resource id/IRI, full typed declaration equality and every facet's
parsed value. Production code is unchanged. This failed run is retained as
`graph-semantics-graphshell-declaration-control-failure.log`.
Runtime gates are recorded in Progress. This adds no migration activation,
schema or consumer API change and selects none of C24–C28.

### P2 remaining-work review (2026-10-07)

Independent source review confirms that this checkpoint cannot close P2.
C24 holds migration of the 16 resource-placed legacy aggregate kinds;
C25 holds component/Ego joins (`graph/query.rs` 753–789), and C26 holds
first-subject identity at RDF apply (`crates/graph/linked-data/src/ingest/apply.rs`
69–99). C28's origin resolution and C27's durable input profile precede
ordinary migration activation. The current adapter is explicitly qualified
and Semantic-only; ordinary replay preserves held stores.

Already ruled production routing also remains: compatibility semantic
writers and deltas use surface writers (`graph/edge_ops.rs` 346–356,
`graph/apply.rs` 1649–1668); tags, properties and classifications still use
surface facets (`graph/node_facets.rs` 132, 170, 213). P2 needs resource tag
concepts and tagging statements, resource literal metadata, full classification
records and shared reads, then their RDF projection. Page-version captures'
durable association with ResourceNode still needs verification; existing
Eidetic fingerprints alone are not that receipt. S53's received work and
its vocabulary/lock checkpoints remain as recorded above. The final P2
done-condition battery follows these changes. No P3–P5 work begins here.
Kernel paths in this finding are under `crates/graph/graph-kernel/src`.

### P2 replay and JSON-LD forks C29/C30 (2026-10-07)

**C29: retain translated legacy effects.** The original raw assertion capture
has no statement id (`graph/capture.rs`, `ReplayAssertRelationByIds`). The
qualified adapter returns exact `baseline_effects` and one `entry_effects`
list per retained entry (`graph/legacy_resource_migration.rs`, `MigratedReplay`).
Pandect retains the original baseline and full journal, and its `edits_of`
and `graph_at` still read those original captures (`graph_session.rs`). A
checkpoint alone cannot make a later raw-history replay reproduce the first
migration's generated handles. The pending fork was put to Mark with:

1. **Persist translated effects (recommended).** Retain exact effects with
   the placement profile, bound to the retained source digest; reopen,
   history and undo use the same first-migration handles.
2. **Checkpoint only.** Preserve the raw history and document that replay
   outside the checkpoint may remint handles and cannot promise exact undo.

Mark selected option 1 on 2026-10-07; ruling 42 records his answer. Exact
receipt/session integration now proceeds. Neutral profile APIs and
recorded-strata reads were implemented before that answer.

**C30: JSON-LD assertion representation.** Current expanded and compact
shapers use default-scope surface quads (`linked-data/src/lib.rs`,
`node_quads`, `node_object`, `compact_node_object`). The complete dataset
projection includes resource assertions and one RDF 1.2 reifier per assertion,
but those shapers do not handle its triple terms. Ingest recognizes
`rdf:reifies` triple terms, without a classic reification bridge
(`linked-data/src/ingest.rs`, pass A). Pending choices:

1. **RDF reification objects plus ingest bridge (recommended).** Export
   subject/predicate/object records with exact handles, scope and provenance;
   translate them into assertion records on import.
2. **Mere assertion records.** Define a Mere-specific JSON-LD metadata graph
   grammar, requiring Mere-aware readers for lossless assertion round trips.
3. **Keep JSON-LD partial.** Require RDF 1.2 N-Quads/TriG for exact transfer;
   this revises S53's inherited lossless JSON-LD done-condition.

Mark selected standard RDF reification on 2026-10-07; ruling 43 records his
answer. Human-readable default/absent metadata omission is
independent, and must preserve the complete positional binary shape.

Kernel paths above are under `crates/graph/graph-kernel/src`; Pandect paths are
under `crates/system/pandect/src`; linked-data paths are under `crates/graph`.

### C24–C28 source and S53 omission review (2026-10-07)

The implemented adapter now covers all 23 legacy aggregate kinds: 16 on
resources and seven retained on surfaces. Its lifetime catalog is keyed by
original surface pair and kind; withdrawal changes active membership, not
first endpoints or the first migration note. Explicit C28 choices must name
an evidenced pair; exact carried handles stay distinct from raw-generated
ones. New custom carried handles under unresolved declaration conflicts
are refused; known held handles retain their store. These controls are in
`graph/legacy_resource_migration.rs` (15 tests), with journal prefix entry
points in `graph/journal.rs`.

Component/Ego derivation in `graph/query.rs` collapses shown bindings at zero
hops, then counts relations across both strata. Hidden resources carry the
walk; public results remain surface ids. Canvas uses that same walk. Tests
cover radii 0–3, aliases, reverse/parallel relations, self-loops, selectors,
equal UUIDs in separate strata and read-only snapshots/revisions.

RDF apply accepts explicit per-subject identity intent through
`apply_contribution_with_identity` (`linked-data/src/ingest/apply.rs`).
Existing bindings and resource metadata survive; generic RDF defaults to
exact term identity and pages opt into canonical URL identity. No contribution
DTO or capture grammar changed, and current production extractor callers have
not yet been wired to the additive page callback.

Pandect's `graph_placement.rs`, `graph_session.rs`, `session_graph_store.rs`
and `graph_codicil.rs` persist explicit profiles at atomic boundaries and expose
profiled snapshots/v3 codicils. Bare v1/v2 inputs remain unqualified. Known
legacy inputs cannot bypass qualification through compatibility materializers.
Ordinary legacy activation and production profile threading remain incomplete;
C29's receipt integration is now authorized by ruling 42.

Human-readable serialization in `types.rs` and `persistence_edge.rs` omits
absent metadata and default scope, retaining every populated value. Binary
serializers keep all eight property fields and all seven statement fields in
their prior order. Four tests verify JSON omission/populated controls, exact
Postcard byte parity and rkyv round trips. The size battery now includes actual
Author attribution and a 50-surface/25-resource/50-binding baseline with 49
resource assertions. Measured incremental cost is 198 bytes/property and 546
bytes/assertion, including its edge wrapper; the existing 280/760-byte ceilings
are unchanged. Annotated properties cost 301 bytes each and the rich resource
snapshot grows from 63,594 to 67,465 bytes (bare 36,841), a positive size control.
No binary layout or legacy edge compatibility fields were changed.

The first kernel compile caught a test's nonexistent `GraphScope::Named`,
corrected to `Custom`; the first linked-data compile caught three private
resource-reader calls in a new test, replaced by complete public snapshot
record comparisons. An initial cartography command used the library name
instead of package `mere-cartography` and was corrected. The failed logs are
retained alongside fresh gates; these are command/fixture failures, not passed
receipts. Kernel/store now passes 407 units and the compile-fail doctest,
linked-data/query 53, Pandect 321; dependent gates are still in progress.

Kernel paths above are under `crates/graph/graph-kernel/src`, Pandect paths
under `crates/system/pandect/src`, and linked-data paths under `crates/graph`.

### Atomic legacy qualification checkpoint C31 (2026-10-07)

The C29 receipt must be installed against the retained source it translated.
A check followed by `Backend::apply` leaves a concurrent-write window. Four
shipped backends support consistent transactional reads and atomic writes:
Memory (`muniment/src/backend.rs`), Directory (`muniment/src/directory_backend.rs`),
redb (`muniment/src/redb_backend.rs`) and IndexedDB (`muniment/src/indexeddb_backend.rs`).
Pandect's `wallet_sealed_backend.rs` explicitly refuses transactions; the
default `Backend::transact` also returns `NotTransactional`. The fork is:

1. **Require transactions; defer wrapper (recommended).** Qualification
   refuses sealed and other nontransactional backends; ordinary reads and
   writes retain their existing contract.
2. **Require transactions; extend wallet wrapper.** Add sealed transaction
   support in Mere in this lane and qualify its read/write behavior.
3. **Allow exclusive-writer fallback.** Require caller exclusivity without
   a transaction; concurrent source changes are detected only on later reopen.

Mark selected extending the wallet-sealed wrapper on 2026-10-07; ruling 44
records his answer. Transactional activation and sealed transaction support
now proceed together. The source
transaction recheck includes the profile, receipt, retained baseline and the
journal, including modern entries present at qualification.

### C29 exact replay review (2026-10-07)

Persisting baseline effects alone is insufficient: materializing aggregate-only
surface edges can mint a handle that no baseline delta changes. The receipt
therefore freezes the complete migrated baseline from the same materialization
as the translated entry effects. Each entry retains its original Author; the
cutoff is permanent and the later journal replays as recorded. Raw source bytes
are captured at session open and checked again before qualification, preventing
an old in-memory graph from being bound to a newer retained source.

Independent review found that `GraphSnapshot::timestamp_secs` is stamped by
`graph/snapshot/to.rs`, so comparing freshly replayed snapshot envelopes could
reject an unchanged receipt on later reopen. `FrozenGraph` now compares all
graph and facet truth while excluding only that envelope clock. The checksum
still covers its exact stored bytes. A deterministic changed-clock positive
control accompanies changed-resource and changed-facet refusal controls in
`pandect/src/graph_placement.rs`. C31 still holds storage activation; its
prepared transaction guard checks the retained source under the commit reader.

The same review found a second loader seam: the legacy snapshot loader
unconditionally rederives surface `Domain`/`UrlPath` relations
(`graph/snapshot/from.rs`, `graph/query.rs`). Receipt materialization and
receipt-bound checkpoints must instead restore their recorded claims without
that inference, retaining checked resource validation. A narrow additive
recorded loader and same-host parent/child controls are saved;
unqualified legacy loading retains its existing behavior. This repair follows
C29's exact replay contract and does not infer a placement profile. Receipt
baselines and receipt-bound checkpoints both use `try_from_recorded_snapshot`;
ordinary/unqualified loading remains unchanged. The source is frozen but its
fresh Cargo gates remain unrun after the shared-cache wait was interrupted.

### Checkpoint growth and implementation hold (2026-10-07)

Mark challenged the increase from six original checkpoints to 31 and whether
the plan had actually been finished. Comparison with branch base `36893553`
confirms that its Progress called the phases final, while P2 combined identity,
dual-stratum population, navigation, projection and query adaptation without
settling several related preservation and boundary contracts. The current
plan has 29 settled numbered checkpoints; original C5 (P3 purge default) and
C6 (P4 frozen node) remain open. C7–C31 added 25 numbered checkpoints, all
attached to P2 or P1/P2 boundaries. This count is checkpoint growth, not five
times as many implementation phases.

Three sources of growth are visible in the record. P2's original design left
identity and assertion-preservation contracts under-specified; branch/main
drift caused avoidable repeat C1–C3 questions before reconciliation; and S53
later imported additional JSON-LD, serialization, vocabulary and dependency
work. The lane did not consolidate those changes into a bounded review before
continuing implementation. It also did not consistently distinguish new policy
from consequences of earlier rulings. These are scope and planning failures;
the existence of valid individual questions does not justify serial growth.

New implementation is held for review, with saved WIP retained and all three
owners stopped. C30/C31 answers remain recorded; neither implementation has
begun. No C32 is added. *Reading, not ruled*: ruling 43's sentence about an
input-policy checkpoint is an implementation interpretation, not a separately
answered user decision. Existing C20 and RDF semantics must be checked before
presenting any new input-policy fork. Review must separate original required
work, explicitly added work and avoidable decision churn before continuation.

### RDF import representation exception (2026-10-08)

The required exact profile comparison catches a distinction lost at an existing
public boundary. `crates/graph/linked-data/src/ingest.rs` creates a generated
NodeProperty handle for a plain literal (544), then replaces it with the exact
carried handle for a reified assertion (700). Both may have absent source/time.
`NodeContribution` (61–70) does not carry which path produced the property.
A complete tag concept projects plain SKOS prefLabel and attributed owner
metadata; applying the current contribution both reconstructs that facet and
mints ordinary assertions for those descriptions. Re-export therefore adds
reifiers absent from the original. ID syntax is not a safe discriminator:
caller-selected UUID-shaped, empty and opaque IDs remain valid.

The fresh full RDF gate passes 62 tests and fails four exact round-trip checks
in `C:/t/cargo-targets/mere/graph-semantics-p2-rdf-exception-controls.log`.
Resource-only expanded JSON-LD grows from 21 rows/four reifiers to 26 rows/six
reifiers (25 distinct rows). N-Quads and TriG grow from 19 rows/four reifiers to
24 rows/six reifiers (23 distinct rows). The full-profile apply counts six
edges where the five carried relation/tagging assertions are expected. No
comparison filters those additions out. The Resource-only four-format gate
compares every normalized quad, retaining exact property/tagging metadata and
foreign partial/ambiguous SKOS controls
(`linked-data/src/resource_metadata.rs`). The existing full-profile and file
round-trip gates also retain their exact dataset comparisons. This is a required
P2 fidelity defect, not optional RDF vocabulary cleanup. The single consolidated
scope-exception question asks Mark to choose an additive import envelope/API,
a contribution DTO extension with consumer adaptation, or explicit assertion
semantics for tag definitions. None is implemented pending his answer. No new
numbered checkpoint is added; accepted rulings remain unchanged.

**Answer (2026-10-08).** Mark selected **"A. Add an import envelope and matching
parse/apply APIs (Recommended). Carry parser evidence alongside the existing
DTOs, preserve explicit assertion IDs, and adopt these APIs at Mere’s
profile-import paths. Existing public DTO construction stays compatible."**
The envelope carries parser evidence of explicit assertions separately from
plain tag-definition metadata. Assertion intent is not inferred from handle
syntax, source or timestamp. Existing contribution DTO fields remain unchanged;
the profile-import paths adopt the matching APIs. This resolves the existing
consolidated exception without adding a phase or numbered checkpoint. Exact
quad round trips and explicit metadata-free assertion controls must pass before
P2 completion.

### Remaining P2 seams and current consumer boundary (2026-10-08)

The live writer paths now route the fixed families and custom declarations to
Resource or Surface truth (`graph/edge_ops.rs`, `assertion_write.rs`). Exact
captures preserve all Arrangement kinds; only the durable snapshot retains its
existing Session-kind filter (`graph/snapshot/to.rs::persisted_edges_between`).
Raw replay explicitly uses legacy Surface writers; live tag, literal and
classification writers use the shown resource (`resource_content.rs`,
`node_props.rs`, `apply.rs`). Property-handle checks include the Surface facet
sidecar after overlay (`resource_content.rs`, Pandect `graph_placement.rs`).
Unknown administrative facets and retained origin notes remain preserved.
Switcher thumbnails and Cartography's radial/spectral producers now consume
projected Resource and Surface topology (`pandect/src/switcher_thumbnail.rs`,
`cartography/src/adapters/producers.rs`). The initial Cartography run passed
42 tests and failed two unchanged placement goldens because the producers read
only Surface neighbors. The repair retains every golden and its tolerance,
adds raw Surface and mixed-store controls, and passes all 45 tests in the fresh
gate. Canvas signals and fold traversal now read the same projected topology
(`pictograph/src/signals.rs`, `canvas/fold_projection.rs`); algorithm goldens
remain unchanged. Resource rebindings invalidate cached topology even when
the displayed URL's hostname is unchanged (`canvas/tests/rings.rs`).
The first full canvas run passed 252, failed 48, and ignored 13 tests. Its
failures exposed Surface-only readers and fixtures; the saved repairs include
mixed-store, alias, parallel-bucket and self-loop controls. A private helper
call in the fold repair was replaced with the public classifier API before
the final rerun, which passes 303 tests with 13 ignored.
The large fold reference oracle now materializes its immutable projected rows
once, then retains the full-row scan at every frontier. Rebuilding the same
Resource-to-Surface projection for each of 600 members added repeated binding
scans; the interrupted run is retained in
`graph-semantics-p2-pictograph-oracle-slow.log`. Fixture sizes, member counts,
direction controls and goldens are unchanged. This is test setup only; the
production walk still uses incident relations.
The next full run passed 302, failed one cache control and ignored 13. That
control intended to isolate URL grouping while its live content link changed
the visible topology on navigation. It now explicitly holds its fixture link
on Surfaces; the separate Resource-rebinding test keeps the positive topology
invalidation control (`canvas/tests/rings.rs`). The failure receipt is
`graph-semantics-p2-pictograph-cache-control.log`; the final rerun passes
303 tests with 13 ignored.

Shared content retains owner-scoped tag concepts, separate tagging asserters,
full classification variants and original Surface origins. Divergent review
versions survive composition and require precise record edits. Legacy unknown
tag qualification needs the original mere IRI, carried in the source-bound
translation receipt; the destination and Surface identity are not substitutes
(`legacy_content_migration.rs`, Pandect `graph_session_content_tests.rs`).
The fresh session gate exposed missing Surface-tag clearing in translated
effects. A retained raw tag/property write can also disappear during migration
without a net pre-entry change. Translation now carries those transient-write
corrections and restores the final tag presentation, while retaining the
original birth-clock captures (`legacy_resource_migration.rs::effects_between`,
`legacy_content_tests.rs`). Baseline and historical controls compare complete
snapshots and facet stores, including retained tag presentation.
Recorded-profile session reopening now uses the exact loader rather than
recreating URL-derived Surface relations (`graph_session.rs`). Existing host,
native codicil and product-transfer boundaries carry an additive local profile;
absent profiles remain unqualified and the remote protocol is unchanged.
Final replay and historical reads now recheck active Resource assertion handles
after journal edits (`graph_session.rs`, `graph_placement.rs`). A valid baseline
cannot authorize a later colliding Surface sidecar. Supplied host graphs are
checked before replacing a session or advancing its epoch (`mere_host.rs`).
Replication also preflights and merges Resource truth when every transferred
Surface ID already exists (`transfer.rs`); Surface presence alone cannot prove
that Resource records or assertions were received. Controls retain permissive
Surface-only legacy handles, exact new Resource assertions, unchanged identical
retries and zero writes on a destination conflict. The final Pandect gate passes
344 tests and its wasm32 check exits 0; Graphshell's final gate remains pending.
The first executing Graphshell gate passed 329, failed 16 and ignored four
(`graph-semantics-p2-graphshell-reader-controls.log`). Surface-only assertion
readers and convergence counts missed Resource truth; the legacy sync
withdrawal also missed Resource handles (`personal_sync.rs`,
`personal_sync/assertions.rs::retract_legacy`). The repair reads both stores
and retains Author filtering and exact per-handle withdrawal, with mixed-store
peer-preservation controls. Transfer fixtures retain the protocol's Surface
edge count while checking full Resource payloads, shown bindings and copy
provenance (`transfer.rs`, `native/transfer_staging.rs`, `product.rs`).
Hosted metadata controls now edit under the actual actor and retain peer claims
(`local_edit/tests.rs`). Canvas refresh publishes title-only changes by comparing
titles directly; Graph's revision remains structural, and identical refresh
remains a no-op (`pictograph/src/canvas/lifecycle.rs`). The fresh final Canvas
gate passes 303 tests, with 13 ignored, after this repair.
Graphshell's next full run passes 345 and fails only the new mixed-store
retraction fixture, with four ignored. That fixture asserted that legacy/raw
claims had timestamps without supplying them (`personal_sync.rs:2209`);
explicit timestamped fixture inputs are now supplied, with exact peer records
still checked. Fresh qualification remains pending. The failed receipt
is retained as `graph-semantics-p2-graphshell-timestamp-control.log`. The prior
16 failures are resolved; workspace checking has not yet run in this sequence.
Read-only admission review found that a translated checkpoint's receipt marker
does not prove its graph and facets match receipt replay at the checkpoint
cursor (`graph_session.rs::open`). A valid metadata or facet edit can escape the
handle check and disagree with historical reconstruction. Exact comparison to
the retained receipt is a repair within the existing replay requirement;
`translated_checkpoint_rejects_valid_truth_changes_before_admission` reproduces
both admissions before the repair, while intact and restored checkpoints pass
and persisted bytes remain unchanged in the same run
(`graph-semantics-p2-pandect-checkpoint-negative.log`: refusal flags were
`[false, false]`, expected `[true, true]`). Admission now compares complete
`FrozenGraph` truth with receipt replay at the same cursor before journal-tail
replay or session publication. Ordinary checkpoint admission is unchanged.
The full Pandect gate passes 345 tests after this repair, and its wasm32 check
exits 0 (`graph-semantics-p2-pandect-checkpoint-qualified.log`). Graphshell then
exposed a persisted/runtime enum mismatch in the timestamp fixture; correcting
it retains the explicit 100/200 timestamps. Its next run passes 345 and fails
only the geometry fixture's ordered collection of a `HashSet` of tag labels.
The fixture now sorts that set before exact membership comparison; full peer
records and all geometry/camera/selection/play assertions are unchanged.
The final Graphshell gate passes 346 units and five integration tests, with
four ignored (`graph-semantics-p2-graphshell-full.log`). The compile and unordered
label failures remain in `graph-semantics-p2-graphshell-timestamp-compile.log`
and `graph-semantics-p2-graphshell-label-order-control.log`. The locked workspace
check exits 0 (`graph-semantics-p2-workspace-final.log`). The package-cache lock
cleared without changing Cargo settings,
isolating a Cargo home or disturbing another lane's processes/cache.

Eidetic already holds immutable text blobs with a mutable current-URL reference.
The additive `browsing/captures.rs` ledger associates those existing hashes with
the common resource ID. Acquired capture bytes and normalized page text remain
distinct kinds. Fleece's existing annotation save path records the actual capture
hash (`document-lanes/src/eidetic_bridge.rs`); navigation preserves earlier
resource associations. Resource RDF metadata includes SKOS concepts, attributable
tagging and typed/language literals (`linked-data/src/resource_metadata.rs`).
Imported keyword tags use their unique explicit attributed agent, else RDF subject.

The Turnstone owner reports prepared Inspector/Roster/recycle reader adoption in
`78a59e1` and notes in `eeb4a50`. Its nine ThinkPad tests pass against the existing
`3ded2cd7` pin, with the resource patches unexecuted. That receipt does not qualify
this supplier. Existing Keep/feed labels act as Surface/session controls
(Turnstone `src/app/node_arms.rs:125`, `src/app/feed_arms.rs:44,70,98,236,279`);
reader preparation leaves those behaviors and pins unchanged. Consumer repinning
and integration are held, with adoption owned by that lane. No sibling files were
edited here, and no new numbered checkpoint is introduced.

### Bounded scope audit (2026-10-07)

Mark instructed: **"Bound it."** Three read-only inventories found a finite
P2 closure: production writer routing; shared resource metadata; exact durable
migration and boundary profiles; existing page-capture association; projection,
query and interchange parity; and final qualification. The six requirements
in §4 consolidate those inventories without adding implementation phases or
checkpoint numbers. Existing APIs already implement resource storage, typed
captures/handles, predicate declarations, shown bindings and the borrowed query
adapter; unfinished production paths and metadata representations remain.

Verified seams: Surface-only production wrappers in
`graph-kernel/src/graph/edge_ops.rs`; Surface tag/property/classification writers
in `graph/node_facets.rs` and `graph/node_props.rs`; Resource semantic-only RDF
projection and Surface metadata shaping in `linked-data/src/lib.rs`; Pandect's
qualification transaction stub and sealed wrapper refusal; Graphshell/Pandect
profile propagation; and capture hashes without a ResourceNode association in
Eidetic's `browsing/text.rs` and document-lanes' `eidetic_bridge.rs`. The paths
are bounded existing Mere responsibilities, not reasons for new feature lanes.
Kernel and linked-data paths in this finding are under `crates/graph/`.

The RDF inventory also found literal assertion metadata overwritten when two
reifiers share one property slot (`linked-data/src/ingest.rs`), and property
dedup omitting the asserter (`graph-kernel/src/types.rs`, `NodeProperty::content_eq`).
Repair belongs to the accepted separate-assertion and RDF-profile invariants,
alongside Resource property routing; it does not create a new decision stop.

Scope growth from unaccepted ExampleOf/Summarizes mapping proposals and old
Oxigraph retirement is independent of the dual-stratum invariants. These two
items move to the existing archived-plan tails backlog. Current mappings and
locked dependencies remain in use. JSON-LD fidelity and sealed transactions
remain explicitly accepted work (rulings 43–44); they are not deferred.

### Remaining original questions C5/C6 (2026-10-07)

On Mark's **"Ask the questions"**, the two remaining original checkpoints
were presented together in native prompts. The turn remains active for the
answers; no dependent implementation starts while they are pending.

**C5, pending-link cache retention.** `StatementOutcome::pending_targets`
currently returns unresolved target URLs without retaining a rebuildable index
(`crates/graph/linked-data/src/statements.rs`, lines 56 and 86). P3 creates
that index; it remains derived data, separate from source documents and graph
truth. The default retention policy remains configurable. Choices presented:

- **A, session-only cache (recommended):** retain while the mere is open,
  clear on close, reconstruct from retained sources when needed.
- **B, budgeted persistent cache:** retain across restarts, automatically
  purge under configurable size/age limits.
- **C, explicit-purge cache:** retain across restarts until explicitly
  cleared; no automatic eviction policy is selected by that option.

Every option preserves source documents and asserted claims when the index
is purged. This question selects the default retention policy, not a new
keeping level, storage engine or synchronization feature.

**C6, frozen-selection ownership.** The current `GraphBearing for Node`
implementation reports the SurfaceNode's nested log identity
(`crates/graph/graph-kernel/src/graph/chart.rs`, line 73). Member resource
references, copied statements, spec and revision remain fixed by rulings 3
and 8; this question only selects which newly minted node bears them:

- **A, frozen ResourceNode (recommended):** the resource owns the immutable
  nested selection; surfaces display it. Identity and annotations are shared
  independently of workspace appearances. Resource graph-bearing support is
  part of the corresponding P4 implementation.
- **B, frozen SurfaceNode:** the stable saved surface owns the immutable
  nested selection through the existing bearing model; the surface and its
  snapshot are shared together. Ownership follows that workspace identity,
  with content metadata still following the existing resource rules.

Neither answer has arrived at this finding's creation. No new checkpoint
number, new feature target or answer is inferred.

### Bounded P2 assertion and transaction repairs (2026-10-07)

C30's complete dataset JSON-LD shaper is implemented in
`crates/graph/linked-data/src/jsonld.rs` (export at 143, classic import bridge
at 187). Both public shapers use it. Eligible records preserve carried assertion
IDs and metadata; incomplete, ambiguous or unasserted descriptions retain
ordinary RDF treatment. The profile gate covers all seven scopes, separate
edge/literal asserters, opaque IDs, blank identity and idempotent reapplication
across expanded/compact JSON-LD and N-Quads/TriG. This does not add missing
Resource metadata to the underlying projection or finish production routing.

C31's wallet adapter authenticates transactional reads, latches caught read
errors and seals the whole batch before writes
(`crates/system/pandect/src/wallet_sealed_backend.rs`, 133, 166, 256).
Legacy activation now uses the retained-source guard inside the transaction
(`graph_session.rs`, 337, 845); refusal precedes live state replacement.
The additive qualification methods require native `Sync`, while wasm keeps
local futures and ordinary session methods retain their existing bounds.
An independent read-only review found no concrete atomicity or API-bound defect.

The first full Pandect run passed 331 tests and failed two exact-replay tests.
Diagnostics established identical complete Surface edge records in a different
row order after graph slot reuse. `FrozenGraph::of` now sorts only complete
Surface edge records (`graph_placement.rs`, 82), keeping strict equality,
nested lists, Resource rows and exact stored-byte checksums. The control at
370 verifies that changed IDs, sources, times, endpoints, missing records and
duplicates remain differences. Temporary diagnostics were removed; the failing
logs remain evidence. The repaired full suite passes 334 tests.

The pre-existing literal assertion defect is repaired in
`crates/graph/graph-kernel/src/types.rs` (531): equality includes the asserter.
Single and batched live writes retain the first assertion ID when refreshing
its time, and captures carry that stored ID (`graph/apply.rs`, 1405;
`graph/node_facets.rs`, 180). Raw replay grammar is unchanged. Removing the
asserter comparison makes both new invariant tests fail; restoring it passes
the full 411-test kernel/store suite. This does not complete Resource literal
routing. Fresh dependent checks and remaining P2 work are recorded in Progress.

### Continuation handoff (2026-10-08)

Mark requested a compact handoff because this chat is long. Implementation and
all three agents are stopped; every in-flight edit is saved. P2 is incomplete.
P1–P5 and the six P2 closure requirements remain the bound. P3–P5 have not begun.

**Qualified base:** `72c68b6d`, already pushed on `graph-semantics`, after
`6399fe6c` and `586167bd`. Worktree:
`C:/Users/mark_/Code/worktrees/mere-graph-semantics`. Main is shared and untouched.
Mark authorized branch commit/push and Turnstone coordination; main integration
and consumer repinning remain review-gated. Retain this isolated worktree until
the lane is integrated. Target `C:/t/cargo-targets/mere` is shared, reusable and
contains the gate receipts; no compiler/test process belongs to this lane now.

**Only remaining closure:** the RDF import representation exception above.
Mark selected A, the additive import envelope, preserving public DTO construction.
The current WIP is formatted and diff-clean but has not compiled or run tests:

- `linked-data/src/ingest/envelope.rs` encloses the unchanged contribution and
  parser evidence privately. `contribution()` borrows it; `into_contribution()`
  explicitly discards evidence. Complete default-graph SKOS definitions are
  recognized from original quads; reified label/owner entries remain assertions.
- `ingest.rs`, `ingest/apply.rs`, `serialize.rs` and `lib.rs` add/export
  `from_jsonld_envelope` and its context/base variants, `from_quads_envelope`,
  `from_nquads_envelope`, `from_trig_envelope`, and `apply_import` with its identity
  variant. Old DTO fields and old parser/apply signatures remain unchanged.
  Scoped `rdf:type` no longer gets flattened into a default classification.
- `ingest/exact.rs` attempts exact carried literal and semantic admission using
  existing record/pair replay deltas and global-handle preflight. Resource
  properties use exact handle union, preserving distinct IDs and absent source
  or time. Ordinary live reassertion dedup remains unchanged.
- The four previously failing profile/format tests in `lib.rs`, `serialize.rs`
  and `resource_metadata.rs` now use envelope APIs. Their whole-quad comparisons
  are unchanged. New paired-ID and foreign-definition controls are still missing.
- `document-lanes/src/structured_data.rs` and `lib.rs` add parallel
  `JsonLdImportBlockProjection`/`JsonLdImportOutcome` and
  `project_json_ld_import_blocks[_with_base_iri]`/`json_ld_imports[_with_base_iri]`.
  The old public helpers and outcome payload remain compatible. Saved controls
  cover DOM text/order/errors, cached contexts/base IRIs, complete versus partial
  concepts, explicit metadata-free IDs and repeated import. They are unrun.

**Known defect to repair first:** `ingest/exact.rs::import_edge` passes a full
Surface pair to `ReplaySetEdgesByIds`. `graph-kernel/src/graph/edge_ops.rs`
`set_edges_between` rebuilds through `snapshot/from.rs::restore_persisted_edge`,
which merges parallel Surface records and can overwrite an earlier Traversal
array. Preflight preserves the rows; mutation does not. Reproduce with two valid
recorded Surface rows on one pair, distinct Traversal arrays, and an incoming
carried assertion for a declared Surface predicate. Assert complete original
rows/payloads survive, alongside the new assertion. Resource pair replay is exact.
Keep established C1 missing-source normalization on legacy Surface claims;
qualify exact Surface metadata with source-present controls. No new semantic
policy or numbered checkpoint is needed for this invariant repair.

**Then add controls:** paired explicit prefLabel, owner and ordinary-literal
assertions on the same triple, with distinct opaque/UUID/empty handles and absent
source/time. Assert exact Resource records and complete quad sets. Include plain
complete definitions, partial/multiple owner or label definitions, typed/language
labels and scoped Concept type/metadata as same-run controls. Syntax/source/time
heuristics must not replace parser evidence. Keep ordinary live dedup controls.

**Qualification:** root alone runs Cargo, serially, offline/locked, one build job
and `--test-threads=1`, reusing the stable target. Run full
`cargo test -p mere-linked-data --features query --offline --locked -j 1`, full
`cargo test -p mere-document-lanes --features eidetic-bridge,fleece-json-ld --offline --locked -j 1`,
then `cargo check --workspace --offline --locked -j 1`. If kernel changes, rerun
kernel/store and affected dependent tests plus the wasm32 kernel check. Record
all failures with positive controls; do not weaken normalized whole-quad equality.
Run `python scripts/mere_doc_audit.py` and diff checks after document edits.

**Standing receipts at the qualified base:** kernel/store 430 plus one
compile-fail doctest; Pandect 345; Eidetic/Fjall 118; Canvas 303; Cartography 45;
document-lanes 17 units plus one peer-transfer integration; Graphshell 346 units
plus five integration tests. Locked workspace and kernel/Pandect wasm checks
exit 0. The previous RDF/query run is 62 passed, four failed at
`graph-semantics-p2-rdf-exception-controls.log`. Individual receipt names are in
the latest Progress entry. Ignored/headed/browser/device tests and final supplier
sibling builds are unrun. Turnstone's existing-pin proof is separate: prepared
reader adoption `78a59e1`, notes `eeb4a50`, old Mere pin `3ded2cd7`.

Finish the bounded repair, qualify and update this plan/index, then commit/push
on the lane branch and report P2 to Mark. Stop at the phase boundary. Do not
integrate main, repin consumers or start P3 without Mark's review. No new
dependencies, downloads, manifests/lock changes, sibling edits or generated
output trees are authorized. Original naming remains ResourceNode/SurfaceNode,
with Node/NodeKey as Surface compatibility names and SurfaceId kept distinct.

### Linux continuation qualification (2026-10-08)

Resumed `origin/graph-semantics` at exact `c49e3e0b1` in
`/home/markik/Code/worktrees/mere-graph-semantics`. The retained Windows
worktree and its receipts are on the other machine. This chat's native
worktree tool rejected the non-repository chat directory; the Git fallback
created a local isolated lane. The existing `/home/markik/Code/target` is
shared and reusable. Mark authorized missing locked dependency downloads and
autonomous dependency setup; manifests and the lockfile remain unchanged.
His latest instruction requires review before a source push or merge, replacing
the earlier branch-push authorization. He separately authorized updating and
pushing this plan during qualification. This documentation checkpoint carries
no repair source: the repair and new controls remain local and uncommitted.
No main or sibling edits are made.

The first offline RDF gate stopped before compilation on uncached `md-5
0.11.0`. After the authorized cache fill, a fixture-only private helper error
was corrected to the public node-creation API. The executing full RDF gate
then passed 68 tests and failed the new Surface preservation control: two
held rows and one incoming assertion collapsed to one row. A direct kernel
control also failed, with one traversal array overwritten, while recorded
snapshot loading retained both rows in the same run. Exact Surface pair
replay now decodes each row independently and inserts in reverse capture
order, retaining the existing legacy missing-source normalization. The first
repaired full kernel/store run passes 431 tests plus the compile-fail doctest;
one doc example remains ignored.

Independent read-only review found that the importer still used durable
snapshot rows: Session `TileGroup`/`SplitPair` arrangements were omitted,
and snapshot iteration order differed from exact pair-capture order. The
strengthened control reproduces both before repair, comparing the complete
first live payload and every held payload. The importer now reads the
existing `persisted_edges_between` helper, exposed as an additive public
read method, and preflights the same complete capture-order rows it replays.
The legacy durable snapshot filter is unchanged. Follow-up review confirms
both findings are resolved, with no new findings in these changes.

Paired explicit prefLabel, owner and ordinary literals retain separate
opaque/UUID/empty handles with absent source/time, including exact Resource
records and complete normalized quad sets across expanded/compact JSON-LD,
N-Quads and TriG. Complete plain definitions, partial or multiple owner/label
definitions, typed/language labels and scoped Concept metadata are same-run
controls. Repeated application is unchanged. Conflicting literal payloads,
cross-resource handle reuse, literal/edge collisions and conflicting edge
metadata refuse without changing held truth; distinct-ID positive controls
succeed. Ordinary live reassertion dedup controls remain intact. The final
RDF/query gate passes all 70 tests.

The final document-lanes gate passes 18 units and one peer-transfer integration.
Pandect passes all 335 Linux tests. Its earlier Windows total was 345: 11
Windows-only wallet tests versus one non-Windows control account for the
difference; no test was removed or weakened. Graphshell passes 345 units and
five integrations, with four explicitly ignored units. Canvas passes 303
units, with 13 explicitly ignored units; Cartography passes all 45. These
are current Linux counts, distinct from the historical Windows receipts.
Graphshell first stopped at two missing `include_str!` fixture paths. A
symlink at `/home/markik/Code/worktrees/woodshed` to the existing clean
Woodshed checkout supplies the unchanged fixture; no sibling file was edited.
The missing-fixture log is retained. The locked workspace check initially
needed additional uncached dependencies, then passed online and passed its
warm offline repeat. The final kernel/store rerun passes 431 units and the
compile-fail doctest; one doc example is ignored. The kernel
`wasm32-unknown-unknown` check exits 0.

All Cargo gates ran serially, root-only, with one build job and one test
thread where applicable. Mark removed the offline restriction; locked
cache fills were authorized without another permission stop. Manifests and
`Cargo.lock` remain unchanged. Gate logs are named
`graph-semantics-linux-*.log` in `/home/markik/Code/target`; failed controls
are retained alongside successful receipts. The twelve changed Rust-file
hashes in `graph-semantics-linux-qualified-source.sha256` match the qualified
source. Documentation audit adds no findings: six existing broken Woodshed links
now resolve through the fixture symlink, reducing that count from 46 to 40;
all other findings match the baseline. The audit's planted-defect self-test
passes, and diff checks pass.

P2's bounded supplier work is complete locally. P3–P5, consumer repinning
and main integration have not begun. Ignored/headed/browser/device tests,
final supplier sibling builds, and consumer adoption against this source
remain unrun. Eidetic/Fjall 118 and Pandect wasm receipts remain historical;
the unchanged Eidetic crate is not a kernel dependent. This continuation
reruns the affected kernel dependents rather than claiming those historical
receipts as current Linux executions.

The documentation-only checkpoint `792e3ea91` records qualification progress;
this final receipt supersedes its pending gates. Both documentation updates
are authorized for push. HTTPS push failed because the existing GitHub CLI
token is invalid; the existing SSH agent is not accepted by GitHub, and the
connected GitHub integration denies tree writes. Login refresh is pending.
No new SSH identity was created. The qualified repair and controls remain
local for a separate source commit and Mark's source-push review. Retain the
worktree and shared target for review and receipts.

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

**Ruling 20 (C7).** Mark: **"C7, 1, but eventually we will want to be able to
aggregate to 3, but that’s a moot thing we can build from 1 probably."**
Selects the identity namespace option above: a vocabulary belongs to a stable
identity across meres; different owners' equal labels remain distinct concepts,
and another owner's concept can be explicitly reused by IRI. Proven historical
taggers supply their identities; unattributed legacy concepts remain scoped
to their original mere with unknown-attributed tagging assertions. Future
aggregation across vocabularies belongs in the moot direction. *Reading,
not ruled*: aggregation can be built over those concept references without
erasing their origins; its mechanism and implementation are not selected.

**Ruling 21 (C8).** Mark: **"C8, 1"**. Selects resource records: the full
classification record, including lifecycle and primary selection, lives on
the resource. Surfaces showing one resource read the same classification and
review state. This settles placement; the existing first-wins collision and
lifecycle-ignorant read paths are not newly endorsed by this answer.

**Ruling 22 (C9).** Mark: **"i accept the recommendations. proceed."**
Selects option 1: resource ids use UUIDv5 under the standard URL namespace
over the sole shared canonical IRI. The existing surface namespace stays
fixed. The already-approved dependency edge may use the locked UUID package;
any broader lock change remains a stop.

**Ruling 23 (C10).** The same answer selects option 1: preserve conflicting
classification variants with their original surface references, mark the
resource conflicted and require precise record selection before editing
review state or primary choice. No insertion-order survivor is selected.

**Ruling 24 (C11).** The same answer selects option 1: Accepted, Verified
and Imported classifications participate in ordinary selection, display and
affirmative RDF export. Suggested and Rejected remain retained review data.

**Ruling 25 (C12).** The same answer selects option 1: explicit typed
resource-record, exact resource-pair and shown-resource journal captures.
Legacy captures retain their surface grammar. The recommended route included
coordination before the known Turnstone API break lands. The lane's Mere-only
scope and stop-on-consumer-break rule still apply; this records the protocol
choice, not a completed consumer update.

**Ruling 26 (C13).** Mark: **"go ahead?"** in response to the five
recommended choices C13–C17. Selects option 1: unfamiliar predicates default
to resources, with explicit per-predicate surface overrides declared by
nature in the registry and preserved through replay. Known placement stays
as ruled in the table below.

**Ruling 27 (C14).** The same answer selects option 1: composition preserves
distinct surface identities and merges resources by canonical identity. Two
snapshots with two surfaces each and one common URL retain four surfaces
and three resources. Conflicting records retain their variants and origins
under ruling 23; no URL-based surface collapse is performed.

**Ruling 28 (C15).** The same answer selects option 1: selection export
retains selected surfaces' shown resources and concept targets needed by
their tagging statements, plus exact statements among retained endpoints.
All three resource columns are filtered together. This product export does
not change ruling 8's frozen-query selection.

**Ruling 29 (C16).** The same answer selects option 1: page identities use
`canonical_url`; vocabulary term IRIs remain exact, including fragments,
query and case. Both hash the prepared identity IRI under the shared UUID
namespace. Page canonicalization must not collapse distinct concepts.

**Ruling 30 (C17).** The same answer selects option 1: add the checked
`try_from_snapshot` result and use it at fallible load boundaries. Retain
`from_snapshot` as a compatibility wrapper that stops with the checked
error, and preserve rkyv's existing generic error bounds. Invalid resource
columns do not silently lose claims; legacy surface compatibility remains.

**Ruling 31 (C18, 2026-10-06).** Mark: **"Yep. Proceed."** in response
to C18 option 1 and C19 option 1. Keep surface `EdgeKey`, `get_edge` and
`find_edge_key`; add opaque `ResourceEdgeKey` and
`RelationKey::{Surface, Resource}` for mixed assertion writers/results,
with explicit owning-store readers and resource/projected relation iterators.
Independent resource indices must never be implicitly read as surface keys.

**Ruling 32 (C19, 2026-10-06).** The same answer selects option 1:
preserve valid legacy statement reifier IRIs; reversibly encode unsafe
caller-selected handles under a distinct versioned namespace outside
`urn:mere:statement:` and decode both formats. Existing valid output stays
stable; arbitrary kernel String handles retain their identity and metadata.

**Ruling 33 (C20, 2026-10-06).** Mark: **"Option 1"**. Reject ingest
when a reifier uses the exact reserved v1 assertion-ID namespace but carries
invalid hexadecimal bytes or invalid UTF-8. Return an explicit error with no
partial contribution. Unrelated foreign reifiers and unknown versions keep
their existing behavior.

**Ruling 34 (C21, 2026-10-06).** Mark: **"With the mere, proceed"**.
Selects declarations persisted with the mere, governing subsequent writes
consistently after opening in another host. Installed host registries do not
replace that authority. Composition must retain and resolve conflicting
declarations; this ruling does not choose the conflict policy.

**Ruling 35 (C22, 2026-10-06).** Mark: **"1 & 1"**. Selects option 1:
composition retains conflicting predicate placement declarations and their
origins. New assertions using a conflicted predicate require an explicit
selection; historical claims and unaffected predicates remain usable.

**Ruling 36 (C23, 2026-10-06).** The same answer selects option 1:
nature changes govern newly created assertions. Existing handles retain their
recorded owning store; reads and precise retractions search both strata rather
than treating the current declaration as their locator.

**Ruling 37 (C24, 2026-10-07).** Mark: **"Agreed. Proceed."** Accepts
option 1 for C24–C28. Legacy aggregate identity is its original surface pair
and kind; its first observed resource endpoints survive withdrawal and
restoration. Those old bytes cannot distinguish a fresh same-kind claim after
navigation from restoration of the old claim.

**Ruling 38 (C25, 2026-10-07).** The same answer accepts zero-hop shown
joins. Component/Ego walks traverse both strata; radius counts relation hops,
so surfaces showing one resource share its zero-hop neighborhood. Unshown
resources remain traversable between shown resources.

**Ruling 39 (C26, 2026-10-07).** The same answer accepts explicit
per-subject identity intent at RDF apply. Generic foreign RDF defaults to
exact IRIs; page extraction explicitly requests canonical page identity.
Mixed input is supported without changing contribution or capture grammar.

**Ruling 40 (C27, 2026-10-07).** The same answer accepts a durable
placement profile at the session/codicil boundary. A caller-qualified legacy
adapter remains necessary until that profile is present. Current declarations
or resource-column presence cannot prove a historical placement profile.

**Ruling 41 (C28, 2026-10-07).** The same answer accepts explicit
resource-origin resolution for each ambiguous carried handle before that
migration activates. Retain the original history and exact carried id; never
alias it to a generated raw handle. Unambiguous histories can proceed.

**Ruling 42 (C29, 2026-10-07).** Mark:
**"1. Persist translated effects (Recommended)"**. Qualified legacy sessions
retain the exact translated effects with their placement qualification, bound
to a digest of the retained source. Reopening, undo and historical reads use
the first migration's assertion ids, including held claims whose legacy
baseline materialization would otherwise mint a fresh id. Original baseline
and journal remain retained; the qualified legacy prefix ends at a permanent
cutover and later recorded-strata entries keep their owning stores.

**Ruling 43 (C30, 2026-10-07).** Mark:
**"A. Standard RDF reification (Recommended): subject/predicate/object records plus metadata in one JSON-LD file. General RDF tools can inspect them; Mere adds an import bridge."**
Lossless JSON-LD transfer uses classic RDF reification records for individual
assertions, retaining exact identity, scope, attribution and timestamps. The
bridge maps those records to the existing assertion model. Treatment of
ambiguous or incomplete records remains an explicit input-policy checkpoint.

**Ruling 44 (C31, 2026-10-07).** Mark:
**"B. Extend the sealed wrapper now: implement and test sealed transactions in this lane, allowing qualification there too. Adds storage and encryption-wrapper work."**
Legacy qualification checks the retained source and installs its receipt in
one transaction. Mere's wallet-sealed wrapper gains transactional reads and
writes in this lane, preserving authenticated values and atomic refusal.
An underlying backend without transactions still refuses qualification.

**Ruling 45 (C5, 2026-10-07).** Mark:
**"A. Session-only cache (Recommended): keep pending links while the mere is open, then clear the index. Rebuild from retained sources when needed. Limits retained cache data, but reopening may require more reconstruction."**
The pending-link index defaults to session-only retention. Its retention policy
remains a setting; clearing this rebuildable index preserves retained source
documents and asserted claims.

**Ruling 46 (C6, 2026-10-07).** Mark:
**"A. Frozen ResourceNode (Recommended): a new resource owns the immutable selection, and surfaces display it. Its identity and annotations are shared independently of workspace appearances. This requires resource support for bearing a nested graph."**
Freeze creates a ResourceNode bearing the immutable nested selection. Surfaces
display that resource; its identity and annotations are shared independently
of those appearances. The selection references member resources by id and
copies their statements at the freeze revision, retaining its spec and revision.

## 4. Phases

### Scope boundary (2026-10-07)

Authority: Mark's **"Bound it."**, following the checkpoint-growth review.
This dated amendment controls the remaining work; the historical findings and
rulings above remain intact. The lane has five phases, P1–P5. P1 remains
complete; P2-induced invariant repairs are included in P2's qualification.
Accepted rulings 1–46 remain the semantic contract. Phase-end report/review
stops and the isolated Mere-only worktree boundary remain.

P2 has exactly six closure requirements. They are completion checks inside P2,
not additional phases, user questions or a mechanism for expanding its scope.

| Closure requirement | Remaining work | Completion evidence |
| --- | --- | --- |
| Production resource/surface routing | Wire existing live assertion APIs and GraphDelta paths to the settled placement table and custom declarations. Preserve held owning stores and legacy replay grammar. | Every listed Resource/Surface family lands in its ruled stratum; navigating leaves earlier content behind; custom overrides/conflicts, exact handles, retract, replay and undo controls pass. |
| Shared resource content | Implement the already-ruled tag concepts/tagging assertions, typed literal properties, full classification variants/origins and precise record edits; wire production readers to the shown resource. | Two surfaces share content; navigation keeps earlier content on its resource; tag-owner identity/reuse, separate asserters, literal metadata, conflict variants and classification lifecycle controls pass. |
| Exact durable migration | Activate the saved receipt using a source-checked transaction; implement C31's sealed wrapper transactions; carry explicit profiles through existing Graphshell, native/codicil and product/transfer boundaries. | First assertion IDs, Authors, facets and stores survive activation, history, undo/redo, checkpoint/fallback and reopen. Changed sources, damaged receipts and unsupported backends refuse atomically. Sealed reads/writes remain authenticated; absent profiles stay unqualified. |
| Page-capture association | Associate existing immutable content hashes/captures with the common resource ID through current storage paths. | Association survives reopen, is shared by two surfaces and remains with the earlier resource after navigation. Existing capture/fetch/render machinery is reused. |
| Projection, query and interchange | Complete Resource RDF-profile metadata projection; qualify existing canvas lifting/mixed walks and the borrowed query adapter; implement C30's classic reification JSON-LD bridge. | Adapter/materialized row parity; zero per-query dataset rebuild; radius/alias/hidden-resource controls; expanded/compact JSON-LD and N-Quads/TriG preserve profile assertion IDs, scope, source, time and typed/language literals. |
| Final qualification and compatibility | Run the P2 done-condition battery and required touched-crate, locked workspace and wasm checks; report established consumer compatibility and branch commits. | Fresh source-qualified receipts; meaningful negative/positive controls; limitations and unrun checks stated. A consumer break stops integration with a concrete report. Main integration still requires Mark's review. |

Implementation applies already-selected contracts rather than generating a
new checkpoint for each error path or API seam. For classic reification,
complete unambiguous records with an asserted base quad can represent an
assertion. Incomplete, multi-valued or standalone descriptions remain ordinary
RDF; they do not justify selecting a survivor or inventing an assertion.
Reification alone does not assert its described triple, as specified by
[W3C RDF Semantics](https://www.w3.org/TR/rdf-mt/#Reif).
C20's malformed reserved-ID refusal remains atomic. These conservative input
rules preserve data and existing identity decisions; no C32 is created.
RDF fidelity is parity under the existing Mere projection profile, not a full
GraphSnapshot backup or a promise to export arbitrary administrative facets.

Explicitly outside this lane's completion gate: new ExampleOf/Summarizes
alignments; retirement of the old Oxigraph oracle/dependency; a term dictionary,
slotmap/backend replacement or new general RDF reasoning; new fetch/render or
capture engines; new remote wire formats or transport protocols; sibling
implementation; and new CONSTRUCT/DESCRIBE query capability. The vocabulary
and oracle tails are retained in the
[archived-plan tails backlog](2026-07-03_archived_plan_tails_plan.md#rdf-archive-cleanup-deferred-by-graph-semantics-2026-10-07).
Current mappings/dependencies and the existing materialized query oracle are
kept. Required compatibility changes remain subject to the established
consumer gate; this amendment does not authorize silently breaking an API.

P3–P5 retain only their original feature targets and done-conditions below:

- **P3:** coverage on query/projection results and a rebuildable, rederiving,
  purgeable pending index, defaulting to session-only retention (ruling 45).
- **P4:** saved SPARQL specs through the existing query capability, sharing by
  spec, and a revision-frozen nested selection referencing resources and
  copying statements, owned by a new ResourceNode (ruling 46).
- **P5:** addressable storage of both strata and configured neighborhood
  residency, using P3's "not loaded" result and full-residency control. Reuse
  redb/IndexedDB; add no storage engine, scheduler or synchronization protocol.

C5 and C6 were the remaining planned decisions at the bound; both are now
settled in rulings 45–46. No planned design question remains, and no further
numbered C is automatically added.
An optional newly discovered behavior is deferred with evidence; a mandatory
contradiction with an accepted ruling or a required done-condition stops the
phase with one consolidated scope-exception report. It does not silently
change the contract or start another serial architecture round. Routine
implementation choices and invariant repairs are the implementation lane's
responsibility. Nothing received from another lane enlarges this bound without
an explicit scope amendment from Mark.

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

Open-predicate statements (no recognized sub-kind, from JSON-LD ingest and
readers) default to the resource stratum, with explicit per-predicate surface
overrides declared by nature and preserved through replay (ruling 26).

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
- **P3. Coverage and the pending index (rulings 4, 45).** A coverage note travels
  with every query result and scene projection, naming each layer that limited
  it: possession, residency, disclosure, synchronization, projection.
  `apply_link_statements` stops discarding: a recognized link whose target
  resource is absent goes to a pending index that is not graph truth, can be
  rebuilt from source documents, re-derives the statement when the target
  resource enters the graph, and can be purged under a policy that is a
  setting, defaulting to session-only retention. Clearing the cache on close
  preserves source documents and asserted claims.
  Done when: a recognized link to an absent target is re-derived when the
  target arrives, and the same test with the index purged first derives
  nothing (the control); rebuilding the index from documents reproduces it;
  purging touches no graph truth (snapshot equal before and after); a SPARQL
  result and a scene projection each carry a coverage note, and a test makes
  each layer fire at least once.
- **P4. Saved queries (rulings 3, 8, 46).** A `Linked` subgraph spec may be a SPARQL
  query beside the nine shapes, reconciled on revision change like the rest.
  Sharing a saved query sends the spec. Freeze mints a ResourceNode bearing a nested
  graph that holds member resource ids, copies of the statements among them as
  of the freeze revision, the spec and the revision; annotations attach to that
  resource, which surfaces display. A subgraph itself never syncs.
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

Historical numbered decisions and the two remaining original checkpoints.
The scope boundary in §4 controls future questions. Existing answers remain
binding; this list no longer grows as a running implementation checklist.

- **C1 (P1). Legacy statements.** Ruled: ruling 9.
- **C2 (P2). Which families sit on resources.** Ruled: rulings 10 and 14; the
  table is in §4.
- **C3 (P2). Migration of existing claims.** Ruled: ruling 11.
- **C4 (P2). Words.** Ruled: ruling 19 amends rulings 12 and 13
  (resource and surface; resource and surface strata).
- **C5 (P3). Purge default.** Ruled: ruling 45, session-only cache by
  default, with retention policy configurable.
- **C6 (P4). The frozen node.** Ruled: ruling 46, a new ResourceNode owns
  the frozen nested selection; surfaces display it.
- **C7 (P2). Tag IRIs.** Ruled: ruling 20, identity namespace with an
  original-mere fallback for unattributed legacy concepts. Future moot
  aggregation is a direction; its mechanism remains open.
- **C8 (P2). Classifications.** Ruled: ruling 21, complete resource records.
- **C9 (P2). Resource UUID namespace.** Ruled: ruling 22, standard URL
  namespace over the shared canonical IRI.
- **C10 (P2). Classification collisions.** Ruled: ruling 23, preserve
  variants/origins with precise edits.
- **C11 (P2). Classification lifecycle.** Ruled: ruling 24, affirmative
  statuses only in ordinary readers; retain review records.
- **C12 (P2). Resource journal captures.** Ruled: ruling 25, typed captures.
  Coordination authorized 2026-10-05. Typed source checkpoint `cff35712`
  passes the bounded Turnstone consumer gate at `c3b14cb`; production patch,
  dependency integration and main review remain held.
- **C13–C17 (P2).** Ruled: recommendations accepted as rulings 26–30.
  Implementation and gates are in progress; the dated Findings retain the
  evidence and alternatives.

- **C18 (P2). Edge handle ownership.** Ruled: option 1, ruling 31.
  The dated Findings retain the probe, caller counts and alternatives.

- **C19 (P1/P2). Opaque assertion reifier ids.** Ruled: option 1, ruling 32.
  The dated Findings retain the positive/negative probe and alternatives.

- **C20 (P1/P2). Malformed encoded assertion IDs.** Ruled: option 1, ruling
  33. Reject malformed reserved v1 hex/UTF-8 without a partial contribution;
  unrelated foreign namespaces/versions retain their existing behavior.

- **C21 (P2). Custom predicate placement ownership.** Ruled: with the mere,
  ruling 34. Historical placement remains preserved by exact captures; durable
  declarations govern subsequent writes. Ruling 35 settles conflict policy.

- **C22 (P2). Conflicting placement declarations.** Ruled: option 1,
  ruling 35. Retain variants and origins; require selection for new writes.
- **C23 (P2). Later nature changes.** Ruled: option 1, ruling 36.
  Affect new assertions while preserving held handles in their owning stores.
- **C24 (P2). Legacy aggregate restoration identity.** Ruled: option 1,
  ruling 37; original surface pair/kind and first resource endpoints.
- **C25 (P2). Walk hop costs.** Ruled: option 1, ruling 38;
  zero-hop shown joins across both strata.
- **C26 (P2). RDF identity input.** Ruled: option 1, ruling 39;
  explicit intent with foreign-exact default and canonical page opt-in.
- **C27 (P2). Legacy activation profile.** Ruled: option 1, ruling 40;
  durable boundary profile, caller-qualified adapter until present.
- **C28 (P2). Mixed raw/carried lineage.** Ruled: option 1, ruling 41;
  explicit resource-origin resolution, preserving distinct exact ids.
- **C29 (P2). Translated legacy effects receipt.** Ruled: option 1,
  ruling 42; persist source-bound exact effects for reopening, undo and history.
- **C30 (P2). Lossless JSON-LD assertions.** Ruled: option A, ruling 43;
  standard RDF reification plus import bridge in one JSON-LD file.
- **C31 (P2). Atomic legacy qualification.** Ruled: option B, ruling 44;
  transactional qualification and sealed wrapper support in this lane.

## 6. Progress

- **2026-10-09, P2 main publication verified.** Qualified integration
  `526f2ddb5` is pushed atomically to `main` and `graph-semantics`, with both
  remote hashes verified. It contains main through `15fe2a943`. Fresh review
  clears the identity and Secret Service metadata moves and the physics
  channel registry's production topology/cache boundaries. The new physics
  test helpers initially read only Surface relations from the Resource
  fixture: the full negative run passed 373 and failed two. Both reads now
  use projected relations, retaining all fixture, cache-count and hide/show
  assertions. The focused three tests and complete Canvas 375-test rerun
  pass (18 unit tests and one doctest ignored). Full Graphshell 411 plus
  seven integrations pass (five unit tests ignored). Chatelaine 55, Personae
  145 plus its residue checks and doctest, agent-enabled Personae 210,
  Castellan 118 plus ten integrations/residue checks/doctest, default document
  lanes eight, Pelt four plus 32 integrations, and all-feature document lanes
  55 plus two integrations pass. Two Castellan live Linux D-Bus tests remain
  ignored. Locked workspace, kernel/Pandect wasm and full standalone browser
  wasm checks exit 0; earlier unchanged core/RDF/archive receipts above remain
  qualified. Frozen code/asset hashes match. The audit adds no new defects
  against pristine main; two ambiguity entries in this plan are historical,
  and the ignored web lock clears one inherited missing-path finding. These
  are native test and wasm build receipts, without headed-browser,
  physical-GPU or sibling-adoption claims. P3 proceeds under the original
  coverage/pending-index contract, with a report at that phase boundary.

- **2026-10-09, Rapier and projection-editor integration qualified.** Main
  `bdc89a053` merges cleanly onto the first qualified integration checkpoint
  `768a2a94f`, which is pushed on `graph-semantics`. Kernel/store 426 plus its
  compile-fail doctest, Pandect 342 plus two integrations, RDF/query 70,
  document-lanes 18 plus one integration, Cartography 55, Canvas 372 and
  Graphshell 411 plus seven integrations pass. The existing Canvas and
  Graphshell ignored controls remain unchanged. Reticulum passes 62 tests
  with its child helper ignored; its test targets compile. Scenograph passes
  13 plus its compile-fail doctest, scenomise 157, Graphshell-client 69 plus
  13 integrations, conatus 18 plus one integration and seiche 158 plus nine
  integrations (10 unit tests ignored). Locked workspace, kernel/Pandect wasm
  and full standalone browser wasm checks exit 0. Frozen code and asset
  hashes match; the documentation audit adds no new findings. The transport
  runner's inherited thread setting initially put the child's ticket on the
  libtest title line; removing that variable while retaining the parent's
  serial test flag restores the expected protocol line, without source edits.
  The negative log is retained. Fresh review clears the next identity and
  Secret Service tail through `5df06f802`; its targeted gates follow. Main
  publication and P3 remain pending.

- **2026-10-09, first P2 main integration candidate qualified.** The resolved
  merge against main `e5a24a4af` passes kernel/store 426 unit tests plus its
  compile-fail doctest, Pandect 342 plus two reservoir integrations, RDF/query
  70, document-lanes 18 plus one peer-transfer integration, Cartography 55,
  Canvas 372, and Graphshell 407 plus seven integrations. Canvas has 18 ignored
  unit tests and one ignored doctest; Graphshell has five ignored unit tests.
  Locked workspace, kernel wasm, Pandect wasm and the full standalone browser
  wasm checks exit 0. The latter uses its generated ignored standalone lock
  and committed target configuration. Dataset-view controls now prove exact
  projected pairs and positions through the real Canvas underlay, alongside
  the retained fixture hash, labels, counts and refusals. Independent review
  clears this control and both private blob-owner cleanup paths. Source
  hashes match the frozen receipt; the documentation audit adds no findings
  and its planted-defect self-test passes. Generating the ignored standalone
  web lock clears one inherited missing-path finding (175 to 174). Main advanced to `bdc89a053` during
  qualification. The upstream-tail review finds no P2 source incompatibility;
  the newer tail still needs its affected tests after merging. These are
  native tests and wasm build receipts, without headed-browser, physical-GPU
  or sibling-repin claims. Main publication and P3 remain pending.

- **2026-10-09, consumer-gate lifecycle repair.** Graphshell's full gate first
  exposed a stale `Result<bool>` fixture call, then stalled while reopening a
  dedicated personal-sync host. The isolated test and temporary stage probes
  establish that first open, writes, projection and close finish; second open
  waits in the private blob store before any graph, key or transport work.
  Main's existing close path released graph storage but did not shut down its
  private blob actor. A bounded negative test reproduces the wait. Hosts now
  record explicit private-store ownership and call the existing shutdown
  operation after sync and transport close; process-owned shared custody is
  borrowed. Thirteen host tests pass, including private reopen and a real
  shared disk-store control that remains writable/readable after lane closure
  and reopens after its owner's shutdown. A second bounded negative test
  reproduces the same wait after refused construction. Private setup now
  retains its owner through success/error and shuts down on error; two
  refusals followed by corrected configuration pass on the same files.
  The independent reviewer cleared both ownership boundaries. Temporary
  probes are removed. This is a repair required by the existing consumer gate,
  not a new transport or sync capability. Complete Graphshell, workspace and
  wasm qualification continues; main publication and P3 remain pending.

- **2026-10-09, P2 main integration qualification in progress.** Mark authorized
  source push, main merge and proceeding. `640dbd587` is pushed on
  `graph-semantics`. The retained worktree now carries a pending merge against
  current main; the shared primary checkout is untouched. Conflicts were
  resolved around current clock-free snapshots, isolated candidate validation,
  archive histories, registry disclosures and product v3 scenes. Archive
  save/open/compose preserve the explicit placement profile and exact parallel
  Surface rows; complete archived facets replace loader defaults. The optional
  SPARQL gate exposed main's spareval digest mismatch: its sha2 reference now
  uses the already locked 0.11.0 package, without changing package versions or
  manifests. A mixed-topology test moved to the crate that now owns its
  producers; Spiral controls supply the current registry channels.
  Native kernel/store 426 plus its compile-fail doctest, Pandect 342 plus two
  reservoir integrations, RDF/query 70, document-lanes 18 plus peer transfer,
  Cartography 55, and Canvas 372 tests pass. Canvas retains 18 ignored tests
  and one ignored doctest. The historical arrangement constants remain;
  independently captured Linux main goldens match all 22 legacy controls.
  Paired Surface/Resource fixtures compare exact geometry and the complete
  score with only generation normalized. Recorded URL replay isolates site
  grouping; a live navigation control proves cached topology loses and regains
  the earlier Resource's links. The independent integration reviewer cleared
  the repaired controls. Remaining Graphshell/workspace/wasm gates and the
  upstream-tail qualification are running. No main merge is published and P3
  has not begun. Device, headed browser and sibling consumer repins remain
  outside these receipts. Logs and the local source hash receipt remain in
  the shared target.

- **2026-10-08. Fresh-chat handoff requested; envelope WIP frozen.** Mark asked
  for a handoff. All agents stopped and saved their edits. The exact import helper
  has a known Surface parallel-record preservation defect; paired-ID and scoped
  definition controls remain to be added. No Cargo gate has run on the WIP.
  The continuation handoff in Findings names the qualified base, pending files,
  concrete repair, gate commands and review boundaries. Branch commit/push is
  authorized; any saved WIP checkpoint must be labelled unqualified. P2 remains
  incomplete, and P3–P5 remain unstarted.

- **2026-10-08. RDF exception answered; bounded repair resumed.** Mark selected
  the additive import envelope and matching parse/apply APIs, preserving public
  DTO construction. The existing RDF lane implements the parser-evidence seam
  and Mere profile-import adoption; root retains serial Cargo ownership and the
  exact whole-quad controls. Prior independent gates and pushed source
  `72c68b6d` remain qualified. P2 stays incomplete until the fresh RDF gate and
  affected checks pass. P3–P5 have not begun; main and consumer integration
  remain review-gated.

- **2026-10-08. Remaining P2 source and fresh qualification in progress.**
  Production routing, shared content, replay/profile guards, existing immutable
  capture association and projection readers are saved in the existing lane.
  Full final kernel/store: 430 passed plus one compile-fail doctest, one example
  ignored. Pandect: 345 passed after final checkpoint admission repairs.
  Exact migration controls now compare complete
  snapshots and facet stores for both baseline and retained raw content writes.
  Eidetic/Fjall: 118 passed, two doc examples ignored. Linked-data/query:
  62 passed, four round-trip failures on redundant SKOS definition assertions.
  One consolidated representation exception is awaiting Mark's answer; its
  three alternatives are recorded in Findings. No answer is inferred from
  elapsed time. Document-lanes with both bridge features passes 17 units and one
  peer-transfer test. Kernel and Pandect wasm32 checks exit 0. Cartography
  passes all 45 tests with unchanged goldens and added mixed-store controls.
  Canvas's first full run passed 252, failed 48 and ignored 13; after the reader
  repairs it passed 302 and failed one remaining URL-grouping fixture. That
  fixture repair retains distinct Surface and Resource cache controls. The
  canvas suite passed 303 tests, with 13 ignored. The next Graphshell run passed
  329, failed 16 and ignored four; mixed-store reader and exact-retraction
  repairs, actor-correct metadata controls and title-only Canvas publication
  are now saved. Fresh canvas passes 303 tests with 13 ignored. Graphshell
  initially passed 345 and failed its new timestamp fixture, then exposed a
  fixture enum mismatch and an unordered-set comparison. Those repairs retain
  all exact peer metadata controls. The final Graphshell gate passes 346 units
  and five integration tests, with four ignored. Locked workspace checking
  exits 0 after all final repairs.
  Read-only review found a translated
  checkpoint consistency gap; exact checkpoint-versus-receipt comparison and
  its admission controls are implemented within the replay closure requirement.
  The control fails on both valid altered payloads before the guard; full
  qualification after the guard passes all 345 Pandect tests and its wasm check.
  Current receipts are `graph-semantics-p2-kernel-final.log`,
  `graph-semantics-p2-pandect-checkpoint-qualified.log`,
  `graph-semantics-resource-capture-eidetic-final.log`,
  `graph-semantics-p2-graphshell-full.log`,
  `graph-semantics-p2-workspace-final.log` and
  `graph-semantics-p2-rdf-exception-controls.log` in the reusable Mere target.
  Intermediate doc audit and diff check exit 0. A branch checkpoint preserves
  this in-progress source under Mark's commit/push authorization at `586167bd`.
  Two final test-fixture corrections and this passing gate record follow that
  checkpoint. P2 remains incomplete at the RDF import representation exception;
  no planned checkpoint remains open, and
  P3–P5 have not begun. Final supplier compatibility is
  unqualified at Turnstone; its existing-pin receipt remains separate.

- **2026-10-08. Qualified source checkpoint authorized; local continuation.**
  Mark authorized commit/push and permitted continuation on this machine.
  Retain the existing lane worktree and reusable target; main integration still
  awaits review. Commit `6399fe6c`, pushed on `graph-semantics`, contains the
  qualified C30/C31, exact translated receipt and literal attribution source
  covered by the fresh October 7 receipts below. Read-only audits
  locate remaining production routing in `graph/edge_ops.rs` and `graph/apply.rs`,
  resource metadata in the kernel facet writers/readers, profile propagation in
  Pandect and Graphshell, and capture references beside Eidetic's existing page
  text store. These remain inside the six P2 closure requirements. No new
  checkpoint, phase, dependency or sibling edit is introduced.

- **2026-10-07. Bounded C30/C31 and literal fixes pass fresh full suites.**
  Final JSON-LD source, including the split nine-test control modules, passes
  linked-data/query: 62 tests, no doctests. Literal repair passes kernel/store:
  411 tests and one compile-fail doctest, one doc example ignored. Removing the
  asserter comparison fails both literal invariant tests; restoring it passes.
  Pandect passes 334 tests after its exact Surface-row freeze repair; the
  plain/sealed activation lifecycle and full-record/multiplicity controls pass.
  Initial C30 fixture compile/runtime failures and C31 trait/fixture compile
  failures, the 331-pass/two-failure replay run and field diagnostics are retained
  in the shared target logs. The repair preserves exact record content and
  stored checksums. Read-only C31 review found no concrete defect; it supplies
  no test receipt. Fresh dependent gates pass: cartography 44; pictograph/canvas
  300, 13 ignored; Graphshell/personal-sync 333 library tests plus five others,
  four ignored. Locked workspace, wasm32 kernel and wasm32 Pandect checks exit
  zero. Cargo ran offline/locked with one build job; tests ran serially. Final
  doc audit and diff checks pass. Ignored tests, headed/device proofs and sibling
  builds were not run. No dependency/lock changes, downloads, isolated target
  or Cargo home were introduced. The existing worktree and reusable Mere target
  remain for unfinished P2 and its receipts. All source remains uncommitted;
  this is not P2 completion or main integration. P3–P5 have not begun.
  Receipts: `graph-semantics-c30-linked-data-final.log`,
  `graph-semantics-kernel-literal-full.log`,
  `graph-semantics-kernel-literal-broken-control.log`,
  `graph-semantics-pandect-sealed-qualification.log`,
  `graph-semantics-pandect-receipt-order-control.log`, the preserved
  `graph-semantics-pandect-sealed-runtime-failure.log`,
  `graph-semantics-closure-cartography.log`,
  `graph-semantics-closure-pictograph.log`,
  `graph-semantics-closure-graphshell.log`,
  `graph-semantics-closure-kernel-wasm.log`,
  `graph-semantics-closure-pandect-wasm.log`,
  `graph-semantics-closure-workspace.log` and
  `graph-semantics-doc-audit-c5-c6-qualification.log`.

- **2026-10-07. C6 answered; planned questions closed.** Mark selected a
  frozen ResourceNode. Recorded ruling 46 and applied rulings 45–46 to the
  original P3/P4 requirements. The five phases and six P2 closure requirements
  remain bounded; no new checkpoint is added. P3–P5 have not begun. The saved
  literal repair passes the full kernel/store suite: 411 units and one
  compile-fail doctest, with one example ignored. Removing its asserter key
  made both new literal writer tests fail; source was restored before the
  successful full run. Logs: `graph-semantics-kernel-literal-broken-control.log`
  and `graph-semantics-kernel-literal-full.log`. C30/C31 qualification is still
  running; no complete P2 receipt or source commit is claimed.

- **2026-10-07. Independent bounded P2 work resumed with C6 still open.**
  C6 governs P4 frozen-selection ownership and is not a dependency of P2's
  already-selected contracts. Resumed two existing owners for C30 JSON-LD
  reification and C31 sealed transactions/receipt activation. Root owns the
  existing literal-attribution defect: `NodeProperty::content_eq` omitted its
  asserter (`types.rs`); batched writes and live append captures also carried
  the incoming handle on a same-asserter update rather than the stored first
  handle (`node_facets.rs`, `apply.rs`). Repairs and writer/replay controls are
  saved; fresh qualification is in progress. No new checkpoint, capability,
  dependency or sibling implementation is added. C6 remains unanswered in a
  standalone native prompt, and no P4 choice is inferred.

- **2026-10-07. C5 answered; C6 prompt remains open.** Mark selected
  session-only pending-link retention. Recorded ruling 45 without changing
  earlier rulings. The frozen-selection owner remains unanswered; elapsed time
  does not settle it. The saved kernel's offline/locked wasm32 check exits 0
  (`graph-semantics-kernel-receipt-wasm.log`). The locked offline workspace
  check subsequently exits 0 (`graph-semantics-receipt-workspace-check.log`).
  These checks qualify compilation of the saved source, not P2 completion.
  Source edits remain held.

- **2026-10-07. Saved Pandect receipt preparation checked while prompts remain open.**
  The offline/locked `pandect --no-run` test build exits 0. The prepared
  transaction's retained-input guard test passes, and all three
  `graph_placement::tests` pass, including same-host resource containment and
  exact frozen receipt validation. Logs are
  `graph-semantics-pandect-receipt-compile.log`,
  `graph-semantics-pandect-retained-inputs.log` and
  `graph-semantics-pandect-placement-receipts.log`. The initial short-name
  `--exact` filter selected zero tests; that log is retained and is not a gate.
  The corrected filter ran the guard test. This does not qualify activation:
  `commit_translation` still refuses transactions pending C31 implementation.
  The full Pandect suite and remaining source gates were not run.

- **2026-10-07. Saved kernel replay repair verified while C5/C6 await answers.**
  `cargo test -p mere-kernel --features store --offline --locked -j1 --
  --test-threads=1` exits 0: 409 unit tests and the compile-fail doctest pass;
  one doc example is ignored. The exact facet replay fixture retains its full
  equality check and reversed-order control. The successful log is
  `graph-semantics-kernel-replay-order.log`; the earlier failure log is retained.
  This verifies the saved kernel repair only. Pandect activation, the remaining
  touched crates, workspace and wasm32 gates have not been rerun for this source.
  C5/C6 remain unanswered and implementation owners remain held.

- **2026-10-07. Scope bounded on Mark's instruction.** Mark: "Bound it."
  Completed three read-only owner inventories and added §4's six P2 closure
  requirements, explicit exclusions and the original P3–P5 limits. No accepted
  ruling was edited or reopened, no C32 was created and no source work resumed.
  C30/C31 remain included. Independent vocabulary/oracle cleanup is retained
  in the existing backlog; it does not block graph-semantics completion.
  The only planned future questions are C5/C6, to be presented together. The
  saved source and failed receipt remain uncommitted and unqualified; this
  pass changes documentation only. Documentation audit and its planted-defect
  self-test exit 0, and diff checks pass. Fresh source Cargo gates were not run.

- **2026-10-07. Scope challenged; implementation owners held.** Mark asked
  why the original six checkpoints had multiplied to 31 and whether the plan
  was unfinished. Held new source edits and stopped all three owners with WIP
  intact. C30 interchange and C31 sealed transaction support have zero new
  implementation edits. The migration fixture's failure was resource record
  insertion order, not differing assertion ids; the source-only ordering repair
  is saved, with full equality checks retained and its fresh gate still unrun.
  No Cargo session remains active for this lane and no commit was made.
  The existing worktree and stable target remain owned by graph-semantics for
  retained WIP, failure receipts and review. The dated Finding above records
  the comparison and scope failure; prior rulings remain unchanged.

- **2026-10-07. C30/C31 answered and kernel replay gate resumed.** Mark
  selected standard reification and extending the sealed transaction wrapper.
  Recorded rulings 43–44 and released the two bounded implementation owners.
  Keeping the active turn open allowed both native prompt answers to arrive.
  The independent full kernel/store run compiled and returned 408 passed and
  one failure: the exact facet replay fixture exposed resource insertion-order
  differences while its UserGrouped assertion handle matched. The failed log is retained as
  `graph-semantics-kernel-facet-replay-failure.log`; investigation precedes a
  fresh gate. No receipt implementation is qualified or committed yet.

- **2026-10-07. Continuation gates and receipt review.** Before C29 receipt
  source changes, the C24–C28/S53 omission slice passed kernel/store 407 units
  and its compile-fail doctest (one example ignored), linked-data/query 53,
  Pandect 321, cartography 44, pictograph/canvas 300 (13 ignored), and
  Graphshell/personal-sync 333 library plus five integration tests (four
  ignored). Those binaries do not qualify the subsequent C29 source. The
  fresh kernel receipt gate was interrupted while waiting on another workspace
  lane's Cargo package-cache lock; the queued Pandect compile was interrupted
  before source repair. No fresh receipt or activation test is claimed green.
  Documentation audit and its planted-defect/clean-fixture self-test exit 0.
  Changed Rust implementation files pass formatting with child traversal
  disabled, and diff checks pass; a broader formatting scan also reports
  existing unrelated module/import formatting, which was left unchanged.
  Workspace/wasm32 checks have not been rerun for this continuation. Ignored,
  headed, browser-hosted, device and sibling checks were not run. All source
  remains uncommitted on `graph-semantics`; C30/C31 remain pending.

- **2026-10-07. C29 accepted; exact receipt preparation.** Mark selected
  "1. Persist translated effects (Recommended)". Recorded ruling 42. The
  migration adapter now captures exact facet changes as well as graph effects,
  preserving node birth/visit clocks and unknown facet values. Pandect is
  preparing a complete migrated baseline, source-bound per-entry effects and
  exact history/undo replay. C31 holds the activation storage policy; C30
  remains pending. Fresh receipt gates have not yet qualified this source.

- **2026-10-07. P2 continuation after C24–C28 agreement.** Mark accepted
  the five option-1 recommendations with "Agreed. Proceed.". Recorded rulings
  37–41 without changing earlier rulings. Reused clean `f98c7d43`, the existing
  worktree and three agents: migration/origin resolutions, RDF identity input,
  and durable placement profiles; root owns mixed-stratum walks and integration.
  Main remains `e7ec66af`, with the graph plan unchanged from the recorded
  S53 handoff. No main/sibling edits or integration are authorized here.
  Implementation and fresh gates are pending. Previous checkpoint receipts
  remain historical evidence, not qualification of the next source slice.

- **2026-10-07. Qualified P2 branch checkpoint.** Reused the existing
  worktree, stable target and three agents. The saved source includes durable
  declarations, checked assertion APIs, live page bindings, canvas content
  lifts and the explicitly qualified legacy Semantic adapter. Independent
  review repaired the missing legacy setter revision, canvas footprint
  lifecycle and the unenforced C28 hold; no unresolved lineage policy is
  installed. Fresh offline, locked Cargo gates pass: kernel/store 397 units
  plus one compile-fail doctest (one example ignored), linked-data/query 51,
  Pandect 317, cartography 44, and pictograph/canvas 299 (13 ignored). The
  focused cache suite also passes all eight controls; the corrected Product
  suite passes all nine. Full Graphshell/personal-sync passes 333 library
  tests and five integration tests (four ignored). The fresh post-import
  locked workspace check and wasm32 kernel check both exit 0. Only Product
  fixtures changed after the other crate/wasm receipts; their source remains
  the gated source. Final rustfmt and diff checks pass. Documentation audit
  exits 0, its planted-defect/clean-fixture self-test passes, and every audit
  bucket equals HEAD baseline documents in the same environment. No active
  document was added.
  Logs are retained under `C:/t/cargo-targets/mere` as
  `graph-semantics-<gate>.log`; the earlier canvas failures are preserved in
  `graph-semantics-pictograph-cache-failures.log` and
  `graph-semantics-pictograph-first-run.log`. The first linked-data build hit
  Windows error 1450 before running tests; an unchanged offline/locked retry
  passed. Its failure log and the stale Pandect sequence-count failure are
  retained separately. The original six Graphshell import failures, the
  subsequent two Product fixture failures and the final JSON-ordering control
  failure are retained in their named logs; all pass on final source.
  Ignored tests, headed/device or browser-hosted proofs,
  sibling builds, P3–P5 and main integration are not qualified. A final
  read-only check of main `e7ec66af` finds no further change to this plan since
  `26857eaf`. C24–C28 and S53's phase-changing received scope remain the stop
  points; ordinary migration activation and production routing are incomplete.

  Qualified source and documentation are checkpointed on `graph-semantics`
  for Mark's review. Nothing is pushed or integrated into main. No dependency,
  manifest/lock change, download, extra target, Cargo home or worktree was
  introduced by this resumption. Retained: the existing graph-semantics
  worktree/branch, owned by this lane for incomplete P2 and review; the shared
  stable Mere target and named gate logs for reusable builds and receipts.
  No generated output belonging to another owner was removed. Stop before
  choosing C24–C28 or implementing the phase-changing S53 additions.

- **2026-10-06. Resumed source review and unit gate.** Saved the bounded
  legacy Semantic adapter and three additive journal APIs, with eleven tests
  for exact carried handles, historical endpoints, baseline uncertainty,
  declarations, parallel copies, collisions, effective replay/undo and the
  read-only mixed-history probe. Review repaired duplicate-copy reassertion,
  orphan-row composition and unknown-marker diagnostics. Rustfmt and diff
  checks pass. The focused Cargo regression passes (one test); running the
  same freshly built kernel/store executable serially passes all 395 units
  (`C:/t/cargo-targets/mere/graph-semantics-kernel-unit.log`). This is not a
  Cargo documentation-test or broader workspace receipt. The full Cargo run
  is cache-lock blocked; touched consumer crates, workspace and wasm checks
  remain pending. Documentation audit exits 0 with counts equal to HEAD.
  Source remains uncommitted, main integration held, and C24–C28 unanswered.
  A separate read-only check records S53's phase-changing received scope from
  main `26857eaf`; further implementation is stopped at that drift as well.

- **2026-10-06. Resumed after Mark's pause.** Mark: "Ok, proceed,
  orchestrating". Reused the existing worktree and three subagents; ownership
  is replay adapter/journal, source review/composition, and walk/identity fork
  qualification. The pre-pause 384-test kernel receipt is historical; remaining
  crate/workspace/wasm gates are not claimed. Record C24–C28 before choosing
  any of their policies. No main/sibling checkout or dependency change.

- **2026-10-06. C22/C23 continuation authorized.** Mark selected both
  recommendations with "1 & 1". Implement durable declarations on exact-IRI
  predicate resources, using existing resource records and captures; preserve
  conflicts in composition and declaration dependencies in selection exports.
  Then connect new writes to effective placement, retaining exact historical
  ownership for handle reads and retraction. P2 remains incomplete; this entry
  records authorization and work in progress, not a new test receipt.

- **2026-10-06. Fixed placement catalog qualified; stopped at C22/C23.**
  Added the kernel-owned built-in table, stratum type and default lookups;
  no mutable declaration, routing, snapshot/capture grammar or host-registry
  authority is installed. Independent review matched all 43 recognized kinds
  (32 resource, eleven surface), surface traversal, the open-resource default,
  all 17 canonical Semantic IRIs and tagging, and found zero live callers.

  Kernel with `store`, offline/locked and one job in the shared Mere target:
  **367 passed**, one ignored doc example; resource-key compile-fail doctest
  **one passed**. The wrong-placement control deliberately put `UserGrouped`
  on resources and failed exactly one invariant test (expected Surface, got
  Resource). Restored exact source bytes; the full kernel gate passes again.
  Pandect's unchanged `resource_conflicts_return_errors_with_equal_and_disjoint_controls`
  passes **one test**, 312 filtered out, with conflicting/equal/disjoint controls
  and intact source records in the same run. Full Pandect tests are not claimed.
  `cargo check --workspace --offline --locked -j 1` and the wasm32 kernel
  check exit 0. The workspace check waited for the shared package-cache lock;
  no other owner's process/cache was changed and no extra Cargo home was made.
  Existing compiler/configuration warnings remain; no lockfile changes.

  The durable seam needs zero new snapshot fields/public capture variants:
  typed metadata on exact-IRI predicate resources can use existing resource
  snapshots, captures and conditional facet undo. Selection export must retain
  declaration dependencies; recorded resource import remains held. C22/C23
  were asked together before declaration-conflict or later-edit behavior is
  selected. P2 is incomplete; P3–P5 remain unbegun.

  Final touched-file rustfmt/diff checks, documentation audit and its
  planted-defect/clean-fixture self-test pass; all audit buckets remain equal
  to HEAD baseline in this environment. No new active doc or D2 record.
  Full Pandect, linked-data, Pictograph and Graphshell suites were not rerun;
  neither were ignored examples, full sibling builds or headed/browser/physical
  proofs. No downloads, dependencies, source pins, capture grammar, snapshot
  fields or main checkout changed. Nothing is pushed or integrated into main.
  Keep the existing graph-semantics worktree for its unfinished P2 and Mark's
  review, and reuse `C:/t/cargo-targets/mere` for shared builds/receipts. No new
  worktree, target, Cargo home or scratch source was created.

- **2026-10-06. C21 continuation authorized.** Mark: "With the mere,
  proceed". Resuming from clean `bd8df55c`. Add the settled built-in placement
  catalog independently while reviewing durable declaration capture,
  persistence, composition and conflicting changes. No override conflict or
  migration policy is selected by this continuation. P2 remains incomplete.

- **2026-10-06. C19/C20 production RDF repair qualified; stopped at C21.**
  Promoted the shared codec into production export/ingest and added three
  production-path invariant tests alongside its three helper controls.
  Linked-data with `query`, offline/locked and one Cargo job in the shared
  Mere target, passes **50 tests**, zero failed/ignored; no doc tests exist.
  The old-encoder control runs exactly one round-trip invariant test and
  fails with **four reifiers instead of ten**. Restored exact source bytes;
  the full 50-test gate passes again. The initial control invocation selected
  zero tests because its exact filter omitted the module path; corrected
  before recording any control qualification. No temporary source/backup
  files remain. `cargo check --workspace --offline --locked -j 1` exits 0.
  Existing compiler/configuration warnings remain; no lockfile changes.

  Independent source review found no codec or atomic-ingest defect and
  confirmed the unchanged metadata bounds above. Routing review identified
  the legacy/live replay split and C21 registry ownership; no P2 routing
  edits or custom override policy were selected. C21 is put to Mark under
  this lane's stop-at-forks rule. P2 remains incomplete; P3–P5 remain unbegun.

  Final touched-file rustfmt and diff checks pass. Documentation audit and
  its planted-defect/clean-fixture self-test pass; all finding buckets remain
  equal to HEAD baseline in the same environment. No new active doc or D2
  record. Kernel/store tests and wasm32 kernel check were not rerun because
  no kernel source changed in this bounded repair; their `9058352e` gates
  remain historical evidence. Untouched crate tests, ignored tests, sibling
  builds and headed/browser/physical proofs were not run. No downloads,
  dependencies, source pins, capture grammar or main checkout changed.
  Nothing is pushed or integrated into main. Retain the existing isolated
  worktree for the graph-semantics lane's unfinished P2 and Mark's review;
  reuse the shared stable `C:/t/cargo-targets/mere` for builds/receipts. No
  extra target, Cargo home, worktree or scratch source was created.

- **2026-10-06. C20 continuation authorized.** Mark selected "Option 1".
  Resuming from clean branch checkpoint `9058352e`; promote the shared
  reversible codec into export/ingest and qualify exact assertion identity,
  metadata and malformed-input rejection. P2 routing review runs independently;
  no sibling, dependency or lock changes are authorized by this step.

- **2026-10-06. C18/C19 continuation authorized.** Mark accepted option 1
  for both with "Yep. Proceed." Branch state is clean at `3536e074` before
  edits. Kernel handle and linked-data encoding work are delegated within
  the existing graph-semantics worktree; parent owns caller review and docs.
  Main integration, dependency changes and sibling edits remain outside
  this continuation. P2 is incomplete; final gates remain pending.

- **2026-10-06. C18 checkpoint qualified; stopped at C20.** Added opaque
  resource edge keys, stratum-bearing mixed results, owning-store readers,
  projected directed pair/adjacency reads and resource-aware display roles.
  Three runtime tests and one compile-fail guard cover the ownership and
  projection invariants with same-run positive controls. Mixed-return test
  callers use the typed reader; an initial query test compile mismatch was
  corrected without changing its assertions. Production predicate routing,
  resource retractions and migration remain incomplete.

  C19's independent test-only codec preserves valid legacy IRIs and exactly
  encodes unsafe handles in a disjoint versioned namespace. Three tests cover
  empty/Unicode legacy handles, unsafe bytes, namespace lookalikes and
  malformed/foreign distinctions. Production emission and ingest remain
  unchanged pending C20; the general arbitrary-handle RDF round-trip gate
  is still open. The temporary three-input C20 probe and backup were removed.

  Final gates, offline/locked, one Cargo job in the shared Mere target:
  kernel with `store` **365 passed**, one doc example ignored and the new
  compile-fail doctest passed; linked-data/query **47 passed**; Pandect
  **313 passed**; Pictograph/canvas **293 passed**, 13 ignored;
  Graphshell/personal-sync **331 library plus five integration tests passed**,
  four ignored, with `--test-threads=1`; workspace check and wasm32 kernel
  check **exit 0**. The initial default-parallel Graphshell run was
  **330 passed / one failed / four ignored**: the unchanged
  `p2panda_murm_grant_is_refused_before_projection_bytes` timed out accepting
  its projection at `ports/graphshell/src/carrier.rs` 678. The carrier-filtered
  rerun passed **14 tests**, including real acceptance/refusal controls; the
  full serial run then passed unchanged. No timeout or carrier code changed.
  Contention is a possible explanation, not established by a baseline probe;
  no renewed default-parallel pass is claimed.

  Touched-file rustfmt/diff checks pass. Final documentation audit and its
  planted-defect/clean-fixture self-test pass; audit buckets remain equal to
  HEAD baseline in the same environment. No new active doc or D2 record.
  Separate Eidetic tests, ignored tests, full sibling builds and
  headed/browser/physical proofs were not run. Main's graph plan still last
  changed at `62219dd1`; this lane changes neither main nor sibling sources,
  dependencies, lockfile, source pins or capture grammar. No download, extra
  target, Cargo home or worktree was created. Keep the existing worktree and
  branch for unfinished P2 and Mark's review, and the shared stable Mere target
  for reusable builds/receipts. C20 awaits Mark; P3–P5 remain unbegun.

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

- **2026-10-05. Initial P2 slice gated; stopped at C9–C12.** Recorded
  Mark's C7/C8 choices as rulings 20–21, including future moot aggregation
  as a direction rather than an implemented mechanism. Shared canonicalization
  now belongs to Chartulary; Eidetic keeps its public import paths. All four
  moved function bodies match branch baseline exactly. The existing Chartulary
  dependency is unconditional without a lockfile change. `SurfaceNode` and
  `SurfaceNodeKey` are exported with `Node`/`NodeKey` compatibility aliases.
  `ResourceNode`, the resource graph, migration and the query adapter are not
  implemented; P2 is incomplete. Resource UUID namespace, classification
  collisions/lifecycle and journal protocol are evidence-backed forks above.
  No sibling API changed.

  Gates, offline and locked with `-j 1` in `C:/t/cargo-targets/mere`:
  Chartulary **64 passed**, Eidetic default **109 passed** (two doc examples
  ignored), kernel **325 passed** (one doc example ignored), workspace check
  **exit 0**, wasm32 kernel check **exit 0**. The optional Eidetic no-default
  suite is **104 passed / 2 failed**, identically reproduced at the unchanged
  baseline with positive controls in both runs; see the validation finding.
  Initial build attempts lost a shared fingerprint directory, then execution
  session handles disappeared without completion receipts. Required root
  gates were rerun to completion; no other owner's processes/output were
  removed. Formatting and `git diff --check` passed. Documentation audit and
  planted-defect/clean-fixture controls exited 0. All finding buckets match
  branch-baseline documents audited in the same current environment, with
  no findings on this plan. Missing-root references rose from 202 earlier
  in the session to 206 on both baseline and edited documents; this slice
  adds none. No active document was added.

  Pandect, linked-data/query, Pictograph and Graphshell test suites were not
  rerun for this bounded slice; their prior receipts remain above. Ignored
  tests, sibling builds, headed/browser/device proofs and later-phase gates
  were not run. No downloads, new dependency, lock change, patch override,
  extra target or isolated Cargo home. Retained: the graph-semantics worktree
  and branch, owned by this lane for P2/review; the shared Mere target for
  reusable builds. Nothing is integrated into main.

- **2026-10-05. P2 identity/lifecycle slice gated; held at the existing
  consumer authorization boundary.** Mark accepted C9–C12 option 1, recorded
  as rulings 22–25. Added the shared UUIDv5 resource helper, Eidetic export,
  immutable `ResourceNode` identity/address and Chartulary indexing tests.
  Ordinary classification selection, facets, type display and RDF export now
  use Accepted/Verified/Imported while retaining Suggested/Rejected records.
  Full conflict migration, production resource graph, tags, navigation routing,
  canvas lifting and query adapter remain unimplemented; P2 is incomplete.
  Typed captures are prepared as a concrete consumer contract, unapplied
  pending coordinated authorization under the Mere-only brief. The active
  Turnstone lane received that contract and holds existing Scry pins. This
  lane changed no sibling source or API and no Scenomise projection files.

  The required final kernel run found the clock-dependent recreation defect
  above (329 passed / one failed before repair). `4f11d63c` restores every
  original facet on recreation, with ordinary conditional undo preserved.
  The deterministic control removes that forced restoration: original clock
  90 and absent-clock controls complete first; equal planner/original clock
  100 then fails against replay clock 101. The untouched-node 42 control is
  checked in the same run. The mutated run exits 101 with one failure; source
  is restored byte-for-byte in `finally`, then the full kernel gate passes.

  Final gates, `--offline --locked -j 1`, shared `C:/t/cargo-targets/mere`:
  **Chartulary 66 passed**, **Eidetic default 109 passed** (two doc examples
  ignored), **kernel 331 passed** (one doc example ignored), **linked-data
  with query 41 passed**, **workspace check exit 0**, **wasm32 kernel check
  exit 0**. The first Eidetic command used its directory name as a package
  name and failed selection; the corrected `-p eidetic` gate completed.
  The first new linked-data test's id-only-node assumption was corrected
  using a title positive control before its final passing runs. Cargo waited
  for shared cache/build locks; no other owner's process or output was removed.

  Touched-file formatting and `git diff --check` pass. Documentation audit
  and planted-defect/clean-fixture self-test exit 0. This plan has no findings;
  every audit bucket matches branch-baseline documents in the same environment
  (333 active, 322 indexed, 10 orphans, one statusless plan, 40 broken relative
  links, 158 ambiguous paths, 206 missing known-root paths, two stale historical
  annotations). No active document was added. A main recheck confirms its
  graph plan's last change remains `62219dd1`.

  Pandect, Pictograph and Graphshell test suites, the known optional Eidetic
  no-default baseline failures, ignored tests, sibling builds, headed/browser/
  device proofs and later-phase gates were not rerun. The sole manifest/lock
  change is the approved existing locked UUID dependency edge; no new package,
  version/checksum change, download, patch override or isolated Cargo home.
  Retained: the graph-semantics worktree/branch for P2 and Mark's review,
  owned by this lane; the shared Mere target for reusable validation. No
  additional generated-output directory was created. Nothing reached main.

- **2026-10-05. P2 typed storage and consumer seam, C13–C17 pending.**
  Mark authorized coordinated Turnstone work. Added the resource graph and
  its ordinary facet store beside the surface graph; explicit shown-resource
  associations; typed resource-record, resource-pair and shown-resource
  captures; and conditional undo for each. All old capture fields and
  postcard ordinals stay unchanged. Resource record writes validate identity
  and all JSON before mutation; exact resource pairs retain each carried
  payload, order, statement handle, time and source. Conflicting active
  handle reuse and aggregate-only semantic input are refused atomically.
  Retained migration records are not active assertions. No production
  navigation, migration, routing, tag vocabulary or query adapter is wired
  by this slice; those remain P2 work.

  Source: `crates/graph/graph-kernel/src/graph/resource.rs`,
  `graph/apply.rs`, `graph/capture.rs`, `graph/revert.rs`, and
  `graph/snapshot/{checked,from,to,tests}.rs` under the same crate.
  Shared conversion keeps surface restore behavior and structural revision
  changes; snapshot resource pairs preserve their exact iteration order.
  Legacy JSON without new columns yields an empty resource graph. JSON,
  current-layout rkyv and postcard DTO/capture roundtrips pass. No old rkyv
  byte-layout compatibility is claimed.

  The additive `try_from_snapshot` checks complete resource input before
  materializing a graph. Existing infallible wrappers and production load
  boundaries remain unchanged pending C17; unchecked new-schema load can
  still lose rejected records. This is an explicit integration hold, not a
  completed load gate. C13–C16 likewise hold their dependent policy work.

  Final gates, offline/locked, one Cargo job, shared Mere target:
  **Chartulary 66 passed**, **kernel 353 passed** (one doc example ignored),
  **linked-data with query 41 passed**, **Pandect 306 passed**,
  **wasm32 kernel check exit 0**, **workspace check exit 0**. Focused capture and undo runs passed three
  and six tests; the full suite also covers ten snapshot/checked-load tests.
  Negative controls have valid same-run controls. First fixture attempts
  exposed a noncanonical empty-path slash and existing session-only layout
  exclusion; corrected fixtures retain the intended positive controls. The
  new checked-load assertion initially required Graph Debug through
  `expect_err`; an explicit Result match fixed the test without changing
  Graph's API. Final full kernel gate is green.

  The Turnstone owner records **six executed bounded consumer regressions
  passed** at `feb2594`, with all 209 supplier/fixture source hashes unchanged over the
  final run. That earlier receipt predates the final checked-loader orphan
  correction. The owner subsequently requalified exact Mere `cff35712` at
  Turnstone `c3b14cb`: **six passed**, all 209 fresh source hashes unchanged,
  supplier HEAD exact and checkout clean before/after. All 152 frozen Scry
  source hashes remain unchanged. Its artifact remains unapplied to production behavior and its
  portable pins/Scry proof remain frozen. This does not qualify the full
  App/drain/native integration. Mere root changed no sibling source.

  After the final orphan/active consistency correction, the kernel, wasm and
  workspace gates were rerun successfully. Formatting and diff checks pass;
  doc audit and its planted-defect/clean-fixture self-test exit 0. Every audit
  bucket matches branch-baseline documents in the same environment; this plan
  has no findings (333 active, 322 indexed). No active doc was added.

  Pictograph/Graphshell suites, a separate Eidetic rerun, ignored tests, full
  sibling application builds and headed/browser/device proofs were not run.
  P2 is incomplete; P3–P5 have not begun. No new dependencies, downloads,
  Cargo homes or build-output directories. The
  graph-semantics worktree/branch remains owned by this lane for P2 and
  Mark's review; the shared Mere target remains reusable. Main is untouched.

- **2026-10-05. Typed API checkpoint and exact consumer receipt recorded.**
  Mere `cff35712` contains the gated typed-storage/capture slice. Turnstone
  `c3b14cb` records bounded consumer requalification against that exact
  commit, six passed, stable 209 supplier/fixture source hashes and unchanged
  152 frozen Scry inputs. This documentation follow-up changes no source.
  Production behavior/pins and full App integration remain held for the
  compatible reviewed integration set; C13–C17 still await Mark's rulings.
  Main integration is not authorized by the bounded supplier review.

- **2026-10-05. P2 continuation, C13–C17 accepted.** Mark accepted all five
  recommendations with "go ahead?". Recorded rulings 26–30 and delegated
  checked load boundaries, composition/export and query adapter work in the
  existing isolated lane. No main integration or dependency change authorized
  by this continuation. Production resource routing is still incomplete.

- **2026-10-05. P2 independent checkpoint gates in progress.** Full kernel
  including the native `store` feature: 361 passed, one doc example ignored.
  Linked-data with `query`: 44 passed; the nine-query battery matches both
  materialized spareval and Store oracles, and the existing 3x Store-copy
  performance tripwire passes. Pandect: 313 passed. All Cargo commands are
  offline, locked, `-j 1`, sharing `C:/t/cargo-targets/mere`. Touched Rust
  format and diff checks pass. Doc audit exits 0, self-test exits 0, all
  audit buckets equal the HEAD baseline in the same worktree (333 active,
  322 indexed). Graphshell, workspace and wasm gates still pending here.
  C18 holds routing; no phase completion or main integration is claimed.
  Review also found an opaque-id reifier gap, being checked before a C19 fork.

- **2026-10-05. P2 import omission caught before checkpoint.** Added the
  resource-bearing import refusal after source review found the legacy merger
  ignores resource columns. New-session opening preserves them, and legacy
  imports stay supported; complete merge integration remains held. The first
  Graphshell gate caught a fixture-only private API/disabled fixtures feature
  mistake, corrected to the existing public add-node helper without manifest
  edits. Kernel and Graphshell gates are being rerun for these final changes.
  Main is now `2c4eaa1b`; read-only check confirms its graph plan is still the
  `62219dd1` revision with rulings 1–19, without new C18/C19 answers. No main
  merge was performed. The branch's recorded rulings 20–30 remain governing.

- **2026-10-05. P2 independent checkpoint qualified, stopped at C18/C19.**
  Recorded accepted rulings 26–30. Composition retains distinct surface ids
  and unions equal/disjoint resource metadata; conflicting facets and reused
  conflicting handles fail before loss. Selection export retains shown
  resources and needed tag concepts with exact statements, and Copy retains
  resource/assertion ids while remapping shown surface ids. Exact vocabulary
  term identity preserves fragment/query/case. Checked materialization is used
  at native/session/codicil/host/transfer fallible loads, while the compatible
  rkyv wrapper retains its generic bounds. Explicit resource semantic RDF
  projection and the borrowed QueryableDataset remove the per-query whole
  dataset rebuild. Each pattern still scans the projection; only bounded
  node/pair buffers and matched-quad set deduplication are allocated.
  Valid resource-bearing merge import is explicitly refused pending resource
  integration; opening the same codicil as a new session preserves all three
  columns. The kernel compatibility import stops rather than discarding them.
  Full final gates: kernel with `store` 362 passed, one doc example ignored;
  linked-data/query 44 passed; Pandect 313 passed; Graphshell/personal-sync
  331 library tests plus five integration tests passed, four ignored.
  Workspace locked check and wasm32 kernel check both exit 0. Touched Rust
  format/diff checks pass. Offline/locked `-j 1` throughout; no dependency or
  lock changes, new packages, downloads or sibling edits. Existing Cargo
  warnings remain. Gate logs are the bounded named `graph-semantics-*.log`
  files in the shared stable Mere target, retained by this lane as receipts.
  The two deliberately failing probes established index-0 wrong-store lookup
  and opaque-handle metadata loss with same-run positive controls; both source
  backups were restored exactly and removed. General RDF round-trip safety
  for arbitrary handles remains open at C19 despite the present fixture gate.
  No production routing, migration, resource classification variant merger,
  tag writer or saved query/residency phases have landed. Separate Eidetic and
  Pictograph suites, ignored tests, headed/browser/device proofs and full
  sibling builds were not run this turn. Turnstone's read-only caller audit
  informs C18; its prior consumer gate remains qualified only at `cff35712`.
  Final doc audit and self-test exit 0; all audit buckets are unchanged
  from the HEAD baseline in the same worktree. Retain the existing isolated
  worktree for P2 continuation and Mark's review; no new target/home/worktree.
  Nothing is pushed or integrated into main. C18/C19 options remain open;
  this checkpoint does not select an answer or complete P2.

### P3 implementation and qualification (2026-10-09)

Implemented and qualified in the graph-semantics lane:

- Kernel pending inputs retain the source Resource UUID, canonical target IRI,
  complete source-attributed statement spec and, for Surface-owned predicates,
  the original stable Surface UUID. Completed Resource, Surface and shown-binding
  admissions retry the cache. Navigation does not redirect source ownership.
  Resource-only assertions preserve held handles and placement, refuse ambiguous
  content matches or declaration conflicts before allocating new relation buckets,
  and require legitimate appearance context for new Surface-owned claims.
- Replay suppresses ambient extraction for the complete replay and restores both
  suppression and recorder on unwind. Clones start live. Native RDF apply suppresses
  retry through the complete carried batch, then retries successful admissions so
  carried handles and metadata win. Import remains the existing per-row API;
  snapshot-equal refusal is qualified for an incoming row with no admission, not
  a new whole-envelope transaction guarantee.
- Inker's real EngineDocument walk rebuilds inputs from caller-owned documents.
  Missing sources are reported, not created. Prepared source identities can be
  supplied explicitly. Purge changes no source document, assertion, snapshot or
  journal entry.
- GraphSession keeps live pending inputs outside historical baseline/replay.
  SessionOnly is the default; UntilPurged is a setting persisted in a separate
  version-1 pending-links.json slot. Unsupported versions are refused. Cache and
  coverage observations advance the session revision without fake journal changes.
  Only retained-state changes advance the durable cache revision, including a
  return to the previously saved value while an older batch is outstanding.
  Backend batches commit in creation order; delayed receipts cannot replace a
  newer saved-cache receipt. Session-only inputs and host coverage do not dirty
  or write the durable slot. Legacy qualification preserves live runtime context.
- CoverageNote carries possession, residency, disclosure, synchronization and
  projection limits. Host observations stay runtime context, outside graph truth
  and geometry/channel cache keys. An empty note means no known limit in this
  supplied graph, never world completeness. Pending missing endpoints add aggregate
  possession notes, except where a known residency boundary explains the absence.
  Automatic notes expose counts and reasons without pending target IRIs or IDs.
- SELECT, empty SELECT and ASK retain notes. Projection metadata carries host
  limits and actual omitted Surfaces, Resources without projected appearances,
  relation pairs, missing channels, missing focus and unavailable strategies.
  Parallel/directional strokes count as represented endpoint pairs under the
  existing underlay contract; geometry is unchanged. CoveredScene retains the
  portable sceno Scene alongside coverage. CanvasStrategyProjection retains notes
  when reducing geometry, with fresh notes even when channels remain cached.

Focused controls establish live arrival versus purge, Resource-only arrival,
source navigation/deletion, duplicate reapply, carried import handles/metadata,
Surface context/ambiguity, suppression unwind/clone, actual document rebuilding,
prepared identity, unsupported cache version, both retention policies, actual
backend batch/reopen ordering, all five host coverage layers, projection omissions
and missing-channel faults. The reviewer found the returned-to-saved-state cache
write omission and navigation to an already held target; both have observed
negative regressions and repairs. Full serial gates and fresh source review
pass; receipts follow. P5 supplies actual partial residency. External live
peers, headed browser, physical GPU and sibling adoption remain outside these
native tests and wasm compile receipts.

P3 serial qualification receipts, all with --locked, -j1 and native
tests using --test-threads=1 (inherited RUST_TEST_THREADS removed):

| Gate | Receipt |
| --- | --- |
| Kernel, store feature | 426 tests; compile-fail handle doctest passes; one capture-hook doctest ignored |
| Pandect | 348 tests and two reopen integrations; one isolated child helper ignored |
| Linked-data, query feature | 84 tests |
| Document lanes, all features | 55 tests and two integrations, including peer transfer and streaming render |
| Cartography | 57 tests |
| Canvas, canvas feature | 376 tests; existing 18 ignored tests and one ignored doctest |
| Graphshell, personal-sync feature | 411 tests and seven integrations; existing five ignored tests |
| Locked workspace | cargo check --workspace passes |
| Kernel and Pandect wasm | Both wasm32-unknown-unknown checks pass |
| Linked-data query wasm | Explicit query-feature wasm32-unknown-unknown check passes, from browser cwd with root manifest |
| Full standalone browser wasm | Default browser feature build passes, with its retained ignored lock |

The frozen source/assets/manifests map contains 3067 files, including the
ignored standalone browser lock, and remains unchanged through every gate.
Logs are retained under the shared Cargo target as graph-semantics-p3-*.log.
The fresh reviewer cleared the complete source after both repairs. The only
manifest/lock addition is linked-data's direct dependency edge to the already
locked uuid package for explicit prepared-source rebuilding; package versions
and the standalone browser lock are unchanged. Source, index and plan are
ready for normal branch publication. Report at the P3 phase boundary before
main integration or P4/P5 work.
