# Graph semantics plan: assertions, resources, saved queries, residency

**Date:** 2026-10-04
**Status (2026-10-04):** plan. Round 1 ruled (rulings 1 to 4); round 2 forks
open (§5); phases are a draft until those land. No code.

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

## 4. Phases (draft)

*Reading, not ruled*: an order and done-conditions drafted from rulings 1 to 4.
Final after round 2.

- **P1. Assertions (ruling 1).** The statement dedup key gains the asserter;
  every production writer names one; projection emits one reifier per
  assertion; the view groups a claim's assertions; retract is per assertion.
  Done when: two asserters of one `(predicate, scope)` on one pair yield two
  statements with distinct ids, provenance and times; retracting one leaves the
  other; a same-asserter re-assertion updates in place; the
  `RDF -> kernel -> RDF` round-trip gate stays green with two reifiers on one
  triple; existing snapshots load with each old statement as one assertion.
- **P2. Resource identity (ruling 2).** One canonicalizer shared by kernel,
  linked-data and eidetic; resource ids as UUIDv5 over the canonical URL;
  content statements keyed to resources; a surface shows one resource and
  `NavigateNode` changes which; captures under the resource. Done when: two
  surfaces showing one canonical URL see the same content statements; a
  surface navigating from page 1 to page 2 shows page 2's statements and page
  1's stay on page 1's resource; a `utm_` variant resolves to the same
  resource; ingest and eidetic produce the same resource id for one page.
- **P3. Coverage and the pending index (ruling 4).** Every query and
  projection result carries a coverage note; the pending index replaces the
  discard and is rebuildable and purgeable. Done when: a link whose target is
  absent is re-derived when the target arrives, with a positive control; a
  purge empties the index without touching graph truth; a SPARQL result and a
  scene both report which layer limited them.
- **P4. Saved queries (ruling 3).** SPARQL as a subgraph spec; share by spec;
  freeze into a nested graph with spec and revision. Done when: a SPARQL-spec
  `Linked` subgraph reconciles on revision change like the other kinds; a
  shared spec evaluates over the receiver's graph; a frozen nested graph keeps
  its members when the source graph changes and names the spec and revision.
- **P5. Partial residency (ruling 4).** Neighborhood loading from muniment;
  absence unknown by default; "not loaded" reported through P3's coverage.
  Done when: a graph larger than the resident set navigates with only the
  neighborhood in memory, an unloaded neighbor is reported as not loaded
  rather than absent, and loading it restores its relations.

## 5. Open forks

- **5.1 Who is the asserter?** (Ruling 1.) The journal's `Author` (kind and id,
  with version as metadata or as part of the key), the statement's existing
  `provenance_iri`, or both.
- **5.2 How are resources represented?** (Ruling 2.) A second node kind in the
  one petgraph, or a resource graph beside the surface graph joined by a
  "shows" relation.
- **5.3 What is the residency unit?** (Ruling 4.) A k-hop neighborhood of what
  is in view, the scope of the active projections plus a margin, or whole
  sessions and nested graphs; and whether full residency stays a mode.
- **5.4 What does a frozen selection hold?** (Ruling 3.) References to resource
  and statement ids at the freeze revision, or copies of the members.

## 6. Progress

- **2026-10-04.** Plan written; questions grounded (§2); round 1 ruled
  (rulings 1 to 4). Round 2 (§5) next.
