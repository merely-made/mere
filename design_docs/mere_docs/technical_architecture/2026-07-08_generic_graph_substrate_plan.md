# Generic Graph Substrate Plan

**Date:** 2026-07-08
**Status (2026-10-06):** G0 to G2 have landed; G3 is met in substance
(woodshed's practice history is a stemma), pending a ruling on its reworded
clause; G4 has only its export half (`chartulary::rdf`); G5 has begun from the
top, with eleven mere crates depending on chartulary. The substrate is
`chartulary` 0.2.2 (the name was decided in §8), and its home is
`crates/eidetic/chartulary` inside mere, not a standalone repo; woodshed
consumes it from mere.git. Decisions locked with Mark 2026-07-08.

## 1. Decisions locked

1. **Fully generic core.** The substrate is `Graph<N, E>`: no mandated container
   struct, capability traits for interop, provided default payloads for apps
   that do not need bespoke ones. Mere's web node becomes one instantiation.
2. **Fresh minimal core, mere re-bases last.** Build small, validate on a simple
   consumer, harvest mere's mature machinery piece by piece (RDF projection
   above all), and re-base mere at the end. Not an in-place generalization of
   the 14.8k-LOC concrete model.
3. **One history spine.** codicil is the graph-level authority (an ordered log
   of edits); snapshots are muniment-stored materializations (compaction, not a
   second history); node-level lineage is a projection over the spine, promoted
   from `node-lineage`. Fork/copy/duplicate handling flows from lineage's model
   into codicil's roadmap.
4. **Names (locked 2026-07-08).** The substrate is **chartulary**, aliased
   `chart` in consumer workspaces (`chart = { package = "chartulary" }`, the
   tinct/tincture pattern in reverse) and called "chart" colloquially. The
   lineage crate is **stemma**; the RDF projection harvest is **scholia**. All
   three checked free on crates.io; `chart` itself is also free but reads as a
   plotting library on a shelf, so the long form is the published identity
   (section 8).

## 2. The stack

```text
muniment    bytes: content-addressed blobs (node content) + slots (snapshots)
codicil     the ordered edit log (graph-level authority)
substrate   Graph<N, E> on petgraph: ops, filter, capability traits,
            provided Container/Relation defaults
stemma      per-node lineage: a projection over the spine (from node-lineage)
scholia     RDF projection over the semantic ring (from linked-data)  [Tier 2]
analytics   aether / signals / arrangements, retargeted               [Tier 2]
content     notes / tags / lists (the commonplace layer)              [Tier 2]
```

Consumers: isometry (entity/world graph), woodshed (notes, tags, practice
sets), strophe (session relations), mere (the full web graph, at re-base).

**Corrected 2026-10-06 (S14 pass):** two layers of this stack, and the codicil
named in §1 and §5, no longer name current mechanisms. The edit log is
`muniment::Journal`: chartulary's `GraphLog` journals into it
(`crates/eidetic/chartulary/src/spine.rs`, lines 7 and 34), and "codicil" now
names Eidetic's renamed engrams (`c51b9704`, 2026-08-31;
`crates/eidetic/eidetic-core/src/codicil.rs`). The RDF projection is
`chartulary::rdf`, folded from the standalone scholia crate (`275448cd`,
2026-08-31; `crates/eidetic/chartulary/src/rdf.rs`).

## 3. The generic core

- `Graph<N, E>` over petgraph `StableGraph` (stable keys, serde). Typed
  add/remove/query/filter ops; the shape of mere's `graph/` ops without the
  concrete payload.
- **One required bound:** a stable identity on `N` (the `Identified` trait: a
  key that survives serialization, mere's `id: Uuid` generalized).
- **Capability traits, each optional, each unlocking a feature:**
  - `Addressed`: multi-scheme address claims, primary + aliases (gemini, https,
    file, mere, app schemes). Unlocks address lookup and the RDF `@id`.
  - `ContentBearing`: a content reference (muniment `Hash`) + media-type hint.
    Unlocks content-addressed container behavior.
  - `Labeled`: title + tags. Unlocks curated RDF literals (`schema:name`,
    `schema:keywords`).
  - On `E`: `Classified` (family/kind discriminant, for filtering and render
    policy) and `Predicated` (an optional predicate IRI: the edge joins the
    semantic ring and projects to RDF).
- **Provided defaults:** a `Container` struct and a `Relation` type implementing
  the traits, so woodshed or isometry can start without designing payloads.
  Mere skips the defaults and implements the traits on its own `WebNode`.
- **Deliberately not core:** position/velocity (canvas concerns; mere carries
  them in its payload), rendering, physics, RDF (scholia), analytics, and the
  webview-runtime facets (compat mode, session scroll, favicon).

## 4. Relations: two rings

- **The shared semantic ring** (interop): a recognized core (Cites, Supports,
  Contradicts, DependsOn, SameEntityAs, ...) with canonical IRIs and standard
  vocabulary alignment, plus open predicates (any raw IRI passes through
  verbatim and round-trips). Only this ring projects to RDF.
- **App-private families** (experience): registered per app, typed, never
  projected. isometry: Occupies, FacesToward, InInitiativeAfter. strophe:
  TrackContains, OverdubsOnto. woodshed: InPracticeSet, PrecedesInRotation.
  mere: Traversal, Containment, Arrangement, Imported (its current experience
  families, unchanged in meaning).
- **Open design point (settle at G0):** the family registry. mere's precedent
  is a closed enum pair with a `u32` tag encoding (family byte + sub-kind
  ordinal) for transport through layers that cannot see the types. The generic
  form needs an app-registered namespace: candidate shapes are (a) a recognized
  core enum + `(family: interned str, kind: u16)` for app families, keeping a
  compact tag encoding, or (b) fully string-keyed. Bias to (a): mere's tag
  trick is load-bearing for cheap canvas hit-test transport.

## 5. History: one spine, three views

The unification, per Mark's framing: lineage is node-level history, codicil is
graph-level, and they integrate. Refined into mechanism:

- **codicil is the single authority.** Graph mutations are `GraphEdit` entries
  appended to a codicil. The materialized `Graph<N, E>` is `replay(log)`. One
  clarification against the "log of snapshots" phrasing: the codicil logs
  *edits*; snapshots are periodic muniment-stored materializations so load is
  checkpoint + tail-replay rather than full replay (codicil's existing P2
  roadmap). Both exist; the log is the truth, snapshots are an optimization.
- **stemma is a projection, not a second store.** The survey's key finding:
  `node-lineage` is *already generic* (its `EntryIdentityKey` / `OwnerIdentity`
  / `MemoryPayload` are blanket trait bounds; the navigation wording is
  vocabulary, not coupling), so it promotes near-verbatim, the armillary way.
  Its standing rule ("visits own the tree; edges are projected, never stored
  separately") and its R0 invariants (temporal-integrity: append-only, the past
  is never rewritten; replay-isolation: reads never mutate; shared-projection:
  derived views are projections over one authority) become the *whole spine's*
  contract, not just lineage's.
- **Fork / copy / duplicates: lineage's lessons flow into codicil.** From
  mere's cross-graph work (`cross_graph.rs`, `NodeDerivation`,
  `ProvenanceSubKind::CopiedFrom`): forking a graph is forking its log (a new
  codicil whose header records source log id + seq at fork, git-style);
  duplicates across graphs are handled by derivation provenance records, never
  by log deduplication. These land as codicil roadmap items (a fork primitive
  and a provenance header), not as substrate complexity.
- **What retires at mere's re-base:** the bespoke `graph/history.rs` and
  `capture.rs` snapshot machinery re-derives over the spine; the in-tree
  `node-lineage` copy retires in favor of stemma.

## 6. scholia: the RDF projection (Tier 2 harvest)

mere's `linked-data` (3,450 LOC) is mature: expanded/compacted JSON-LD both
ways, N-Quads/TriG, SPARQL via spareval, named-graph scopes, RDF 1.2 reified
statement metadata (label, provenance, assertion time), datatype and language
tags, and 3-category standard-vocabulary alignment. Harvest, do not rewrite.
The port re-seams it from the concrete kernel onto the capability traits
(`Predicated` + `Addressed` + `Labeled` drive the quads). The acceptance bar is
its own existing gate test: the full-profile dataset round-trip
("lossless under the profile"), re-passed over the generic substrate.

## 7. Phases

Done-conditions, not durations.

- **G0: skeleton.** The named substrate repo: `Graph<N, E>`, capability traits,
  default `Container`/`Relation`, the family-registry decision. **Done when** a
  toy graph builds, queries, filters, and serde round-trips on the default
  payloads.
- **G1: the spine.** `GraphEdit<N, E>` entries through codicil; apply/replay;
  muniment snapshot + tail-replay. **Done when** replay(log) == live graph
  holds property-style, and checkpoint + tail load equals full replay.
- **G2: stemma.** Promote node-lineage (rename vocabulary, keep the machinery
  and R0 contract), wire as a projection fed by the spine; codicil grows the
  fork primitive + provenance header. **Done when** a forked graph carries
  lineage across the fork and derivation records survive round-trip.
- **G3: first consumer.** One app ships real user data on the substrate.
  Candidates: woodshed's notes/tags/practice-set graph (small, low-risk) or
  isometry's entity graph (wanted by the DM lane, but behind isometry's own
  keystones). Pick whichever app is actively in hand when G2 lands. **Done
  when** user-authored content lives in a substrate graph through muniment.
- **G4: scholia.** The linked-data harvest over the trait seam. **Done when**
  the losslessness gate passes over the generic substrate and a cross-app
  triple (a woodshed note cites a mere document) exports as real JSON-LD.
- **G5: mere re-base + analytics.** mere implements the traits on WebNode,
  re-derives history/capture over the spine, retires in-tree node-lineage;
  aether/signals/arrangements retarget onto the substrate and become promotable.
  **Done when** meerkat runs on the substrate graph with no behavior change.

G5 is the long tail and deliberately last; G0 through G4 never block on it.

## Where the rungs actually stand (checked against code 2026-08-08)

The ladder above went stale in both directions: work landed without being
credited, and two done conditions can no longer be met as written.

**G0, G1, G2: landed.** `chartulary` ships `caps`, `graph`, `container`,
`taxonomy`, `edit`, `spine` (`GraphLog` over codicil with muniment snapshots),
`commit`, `facet`, `content_class`, `nested`, and `stemma`, the last folded in
from the standalone crate on 2026-07-12.

**Corrected 2026-10-06 (S14 pass):** `GraphLog` journals into
`muniment::Journal`, not codicil (`crates/eidetic/chartulary/src/spine.rs`,
lines 7 and 34). chartulary 0.2.2 also ships `rdf`, the folded scholia
(`275448cd`).

**G3: three consumers exist outside mere, and none of them meets the written
done condition.** The rung reads "user-authored content lives in a substrate
graph through muniment". What is actually true:

- **woodshed** exercises both halves of the substrate in the running app.
  `woodshed-graph` (1112 lines, 14 tests) builds the theory catalog as a
  container graph, and `relation_index` feeds `related_neighbors`, which
  `woodshed-core` calls to drive what the user sees. `PracticeHistory` in
  `woodshed-core::history` is `Stemma<String, (), String, Engagement>`, adopted
  2026-07-26: every practice engagement the user records is a dated stemma
  visit. That is user-authored content in a substrate graph, and it survives
  restarts.
- **turnstone** uses `chartulary::FacetId` and `AcceptAll` on nodes it already
  owns, including mere's provenance facets. Facets on a host graph, not a
  substrate graph of its own content.
- **cleromancy** uses `GraphLog`, `Container`, `Relation`, and `EditSpec` in
  its servitor lane, plus facets in host and sync.

**So G3 is met in substance, and its last clause should be restated rather than
chased.** "Through muniment" names a mechanism where the rung meant a property.
Woodshed persists through its own `Storage` seam: one serde form moved by a
host realization, filesystem on desktop and OPFS in the browser, sealed with
`personae::seal_bytes`. That is the same seam muniment provides, already built,
already sealed, already working on both targets. Satisfying the clause
literally means either replacing that with an equivalent abstraction and
redoing the sealing, or splitting one practice session across two persistence
paths. Both make woodshed worse to make a sentence true.

The property the rung wanted, real user data living in a substrate graph
durably, holds. Recommend crediting G3 and rewording the clause to name
durability rather than the crate that provides it.

**G4: the export half exists, the gate does not.** `scholia` ships `to_jsonld`,
`to_nquads`, and `to_quads`, projecting only the shared semantic ring so an
app's private relation families stay private, with five tests. Its own module
docs name the remainder as roadmap and say linked-data's losslessness gate has
**not** been re-passed. The cross-app triple the rung names, a woodshed note
citing a mere document, has nothing blocking it now that woodshed is on the
substrate.

**Corrected 2026-10-06 (S14 pass):** there is no separate scholia crate.
`275448cd` (2026-08-31) folded it into `chartulary::rdf`, which ships
`to_quads`, `to_jsonld` and `to_nquads` (`crates/eidetic/chartulary/src/rdf.rs`,
lines 120, 144 and 188). Its module docs (lines 7-25) still list losslessness,
compact JSON-LD and SPARQL as future work, so the G4 reading above holds.

**G5's done condition is unmeetable as written.** It reads "Done when meerkat
runs on the substrate graph with no behavior change." Meerkat was decomposed
into mere's crates and then turnstone, so nothing can satisfy that sentence.
Note also that mere's own adoption, which is what G5 describes, has already
begun: `graph-kernel`, `session-runtime`, `graphshell`, `commons`, `gemot`,
`servitor`, `signals`, `seiche`, and `eidetic-core` all depend on chartulary.
The ladder is being climbed from the top as well as the bottom.

**Corrected 2026-10-06 (S14 pass):** the dependent list has moved with the
consolidations. At `535bca11` the mere crates that depend on chartulary are
graph-kernel, pandect, graphshell, commons, gemot, servitor, pictograph,
seiche, eidetic-core, alembic and athanor.

**Open, raised by the S14 pass (2026-10-06):** this section's recommendation to
credit G3 and reword "through muniment" was never ruled, and G5's meerkat
clause cannot be met. How are the two rungs settled? Options: restate both
rungs in place; move G5 into a mere re-base plan.

## 8. The name: chartulary, "chart" for short

**Decided 2026-07-08: chartulary.** The attested variant spelling of cartulary
(cartulary itself is taken on crates.io): the register book into which a house
copied its charters and muniments. The meaning is exact: the thing that
organizes muniments and their relations is literally a chartulary. Considered
and passed over: pandect, matricula (free; less muniment-tied), catena (free;
implies a linear chain, the wrong shape), cartulary/trellis/tela/rete (taken).

**The short form.** `chart` is free on crates.io and is the same root (charta:
the charter, the paper), but published under that name the crate would read as
a plotting library, and it would sit confusingly near mere's `cartography`. So
the published identity is `chartulary`; consumer workspaces alias it
(`chart = { package = "chartulary" }`, the tinct/tincture pattern in reverse)
so code reads `use chart::Graph`, and "chart" is the colloquial name in docs
and conversation.

Family read: muniment (the kept records), codicil (the appended amendment),
chartulary (the register that binds them), stemma (the descent of copies),
scholia (the commentary in the margins).

## 9. Open questions

1. Family-registry mechanics (section 4): compact-tag hybrid vs string-keyed.
2. Edge multiplicity: mere holds one edge per node pair carrying multiple
   statements; petgraph supports true multigraphs. Pick one semantics at G0.
3. Serialization: core is serde-first; mere's snapshots use rkyv. rkyv as an
   optional feature at G1, or mere-side only at G5?
4. Crate granularity: core + defaults in one crate, or a taxonomy split. Bias
   to one crate until a consumer proves the split.
5. Stemma wiring: does the spine feed stemma automatically (every edit emits a
   visit-shaped event) or is wiring consumer-side? Decide at G2 with real use.

(Resolved: the substrate name, section 8.)

## Provenance

Grounded in 2026-07-08 reads of mere's `graph/edge_taxonomy.rs`,
`graph/node.rs`, `graph/identity.rs`, `linked-data/src/lib.rs`, and
`node-lineage/src/lib.rs`, the muniment/codicil founding proposals
(repos/muniment, repos/codicil), and the tiering + decisions conversation with
Mark (fully generic; fresh core, mere last; one history spine; stemma).

## Progress

- **2026-10-06 (S14 pass).** Status and claims corrected against the tree at
  mere 535bca11, from the D2 record in
  support/doc-audit/d2/batch_50_s14_phase_b12.md: the status records G0 to G2
  as landed and the in-mere `crates/eidetic/chartulary` home, codicil and
  scholia are corrected to `muniment::Journal` and `chartulary::rdf`, and the
  unruled G3 and G5 restatements are raised as an open question.
