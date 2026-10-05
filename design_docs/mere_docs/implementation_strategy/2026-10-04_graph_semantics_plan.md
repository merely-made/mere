# Graph semantics plan: assertions, resources, saved queries, residency

**Date:** 2026-10-04
**Status (2026-10-04):** plan. Rounds 1 to 4 ruled (rulings 1 to 17); phases
final; checkpoints C1 to C4 ruled, C5 to C8 open (§5). No code.

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

## 4. Phases

### Placement by stratum (rulings 10, 14, 15)

| Family | Resource stratum | Node stratum |
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
  and tagging statements (ruling 15). Code and docs say "resource" and
  "stratum" (rulings 12, 13).
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
- **C4 (P2). Words.** Ruled: rulings 12 and 13 (resource; strata).
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
- **2026-10-04.** Round 3 ruled C1 to C4 ahead of the lane (rulings 9 to 15,
  ruling 12 after Mark's own coinage was answered and put back, ruling 15
  after his free text on `UserGrouped` was put back); placement table added;
  checkpoints C7 and C8 opened by ruling 15.
- **2026-10-04.** Round 4 (rulings 16 and 17), raised by Mark: "aspect" is the
  umbrella for the graph's divisions, and keeping's bottom step is the ambient
  level. Carried into TERMINOLOGY (aspect, ambiance), the ambiance design and
  the reservoir plan.
