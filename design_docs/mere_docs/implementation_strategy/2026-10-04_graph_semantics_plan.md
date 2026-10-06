# Graph semantics plan: assertions, resources, saved queries, residency

**Date:** 2026-10-04
**Status (2026-10-05):** in progress. P1 implemented and gated on
`graph-semantics`, with the ruling-9 exact-journal and legacy-checkpoint
attribution repair complete after the original `459cad84` receipt. Those
receipts covered IRI-safe handles; C19 now exposes an opaque-id reifier gap.
Reconciled main `62219dd1` rulings 9–19 before P2 source edits; the graph
plan is unchanged at main `d2d6ac3d`. A1/B1/C1 selected by "All 1";
`ResourceNode`/`SurfaceNode` are settled by ruling 19, and B1/C1 remain
approved. Mark authorized P2 after the P1 repair at `4bc9ae96`, then
selected C7 identity namespaces with future moot aggregation and C8 resource
classification records (rulings 20–21), then accepted C9–C12 option 1
(rulings 22–25). Shared canonicalization, the `SurfaceNode`/`SurfaceNodeKey`
names, the common resource UUID helper, `ResourceNode` identity and affirmative
classification readers are implemented. Resource graph population, conflict
migration remain incomplete. Mark authorized Turnstone coordination; typed
resource storage, captures and undo pass the full kernel gate; the prepared
Turnstone consumer patch passes its bounded source gate. Mark accepted C13–C17 recommendations with "go ahead?" (rulings 26–30).
The independent composition/export, exact vocabulary identity, checked load
and borrowed query-adapter checkpoint passes its gates. Production predicate
routing is held at C18; C19 holds the newly exposed RDF handle encoding choice.
Resource population and conflict migration remain incomplete.
Replay-first migration and per-predicate placement govern P2. The committed
identity/lifecycle slice and recreation repair pass their gates; P2 is incomplete.
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

- **C18 (P2). Edge handle ownership.** Open: independently allocated
  resource/surface indices collide. The dated Findings above give the executed
  probe, caller counts and three API options. Production routing stops here.

- **C19 (P1/P2). Opaque assertion reifier ids.** Open: valid kernel handles
  can lose RDF reifiers and carried metadata. The dated Findings give the
  executed positive/negative probe and three encoding/error choices.

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
