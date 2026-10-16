# Link model: assertions are the truth, links are readings

**Date:** 2026-10-15
**Status:** design with a qualified bounded P1 shared reading and P5
fixed-endpoint Graphshell experiment on source `1e3fb426c`. P2's semantic
lifecycle and Knot supplier/intake gate are qualified by the continuation on
`b69510949`. Bounded P3 is qualified: its chartulary prerequisite and shared
Graphshell opened-link slice are qualified on `732c1537a`; Knot receiving
qualification is published at `921b4e0`, and Woodshed receiving qualification is
published at `943c762` on native source `474d10a`, including authoring, ordered
audition and fresh-process sealed reopen. Bounded P4 rest-stroke and fan
retirement is qualified on Mere source `91467c0bb` and Woodshed receiving source `fe54425`, published with receipt
`4716b71`: canonical resting links, exact member opening, acknowledged whole-link
hides and reviewed browser/native reopening. Bounded P6 (§8) is qualified on
published Mere `a45a0b697`, with Woodshed source `d896e39` and published receiving
receipt `c32b60a`. Bounded P7 promotion is qualified in Graphshell on
`6d53d4439`, from refreshed main `a4c1814c8`, including immutable Resource
copies, receiver coverage, review assertions and browser reopening.
Mark ruled L1–L6 on
2026-10-15 and clarified the dynamics purpose: links respond to fields and
perhaps to compatible links, making different dynamics legible (§7.3). The
[implementation plan](../implementation_strategy/2026-10-15_link_model_plan.md)
owns current assignments, exact evidence and qualification limits.
Mark ruled the remaining open questions as L7–L13 on 2026-10-15 (§4); none
remain open (§9).

**Inputs.** The verified seven-agent survey of 2026-10-15
(`testing/mere/link-edge-survey-20261015/survey.json`, outside this repo,
covering history, kernel, RDF, semantics, presentation and consumers, plus a
completeness critic). Also the primary sources it cites, re-read for this
doc: [graph semantics plan](../implementation_strategy/2026-10-04_graph_semantics_plan.md)
(ruling 1, §7), the [edge system audit](../research/2026-06-13_edge_system_audit.md)
(thread 2), the [petgraph-RDF plan](../../archive_docs/2026-10-06_completed_plans/2026-06-18_petgraph_rdf_plan.md)
(2026-07-04 revision), the [statements-over-schema stance](2026-05-22_statements_over_schema_stance.md),
the [statement kernel brief](2026-06-19_statement_kernel_brief.md),
[projection grammar catalog](../research/2026-08-15_projection_grammar_catalog.md):89,
[graph projections research](../research/2026-06-22_graph_projections_research.md),
[swatch design](../design/2026-06-27_swatch_primitive_design.md) §7 and §11,
the [dynamics grammar plan](../implementation_strategy/2026-10-02_dynamics_grammar_plan.md)
(design-language inputs) and [TERMINOLOGY](../../TERMINOLOGY.md).

**Origin.** Woodshed's adoption of Graphshell's canvas controls exposed links
bending in the relationship editor. Cambium's `graph_canvas` fans parallel
relations 12 px apart. Mark asked for the link/edge subsystem to be thought
through as its own lane, with simplification and a clear purpose as goals:
"Lot of factors/aspects/parameters/tiers/planes/levels to keep track of…
simplification and a clarity of purpose should be a goal too." Success is
measured by how many concepts this design retires, not how many it adds.

## 1. Purpose

The link subsystem records why two things are related, as separate,
attributed reasons. Each reason is an **assertion**: one asserter's claim,
with its own identity, meaning, time span and basis. Any view can then answer
three questions about a pair: what are all the reasons, who gave each one, and
when and on what basis each held. The link that gathers them, its stroke and
weight, and its opened list or timeline are derived readings. A responsive
path may also carry temporary simulation state in a view (§7.3). These
readings and live states are rebuildable; they are never additional
relationship truth. A concept stays only if it records an assertion, states
what one means, or reads assertions for a view. Any other concept is retired.

## 2. Mark's four ideas, evaluated

1. **One reified link per pair, with members inside it.** *Kept in substance,
   changed in mechanism.* One link per pair, with its assertions inside, is
   the model (§3). The link is a **derived reading**, not a stored node with
   edges to its members (ruling L1). Ruling 1 (2026-10-04) already rejected
   "one claim with a supporter set" in favor of separate assertions grouped by
   the view, and the catalog's rule
   [catalog](../research/2026-08-15_projection_grammar_catalog.md):89 says the same: "Multiple reasons
   remain multiple relations even when a renderer visually bundles them". A
   stored link-node would hold no fact of its own and would need a sync join.
   It would also put a second identity beside the assertions, which already
   have ids, times and reifiers. The link-to-member connection is not an
   **edge** in the 2026-10-13 sense either. An edge crosses semantic strata
   of truth. A link's membership is a grouping inside a reading, like a
   supernode's members. Members sit *inside* the opened link (§7.2), so no
   strokes are needed to show that they belong.
2. **Members on a timeline of when and why.** *Kept, as the default
   arrangement when the disclosed data is dated.* It needs first-assertion time, timed withdrawal and a basis (§6).
   P2 now records these for semantic assertions; other families and unknown
   source times still need an **undated lane** (§7.2).
3. **Links respond to fields and perhaps to like links.** *Kept as an
   experiment.* With one stroke per link (ruling L4), its path can help make
   different dynamics legible. A field may deflect the path along its length;
   compatible links may attract, align or respond together to a disturbance.
   The first proof holds endpoints fixed so these responses can be compared.
   Endpoint feedback and sympathetic resonance remain experiment choices,
   rather than consequences of today's endpoint spring. P5 gates nothing else.
4. **Reification was the original point of families, payloads, kinds and
   types.** *Half supported.* Families (2026-03-14) were **policy** classes:
   persistence, visibility, deletion, layout force and navigator precedence.
   The payload (2026-03-21) was **storage** multiplexing: "Payload is
   additive", one hot adjacency per pair. Kinds and selectors (2026-05-11)
   were **addressing**, so hit-test and hide could target one relation.
   Reification entered separately, through the stance doc (2026-05-22) and
   the audit's "biggest fork" (2026-06-13). It landed per assertion only:
   `StatementId` plus an RDF 1.2 reifier. The grouping instinct runs through
   all of them, and this design completes it: every family's facts become
   assertions, and the pair becomes their reading.

## 3. The concept set

Five concepts and one view rule. Each definition aligns with TERMINOLOGY and
states only what is new.

- **relation**: unchanged. The umbrella for a link within one stratum and
  an edge across strata (TERMINOLOGY, refinement adopted 2026-10-13).
- **assertion**: the one unit of relationship truth, for every family. It is
  one asserter's statement that a subject relates to an object by a
  predicate, within a scope. It holds:

  | Field | Meaning | Today |
  |---|---|---|
  | `id` | opaque, device-safe, stable across snapshot, replay and sync | canonical assertion ID for every recognized relation predicate |
  | `subject`, `object` | direction is kept; the link reading is unordered | pair bucket direction |
  | `predicate` | meaning, as an IRI; recognized predicates carry behavior | one of 44 canonical recognized predicate IRIs, or an open predicate |
  | `scope` | named graph | `graph_scope` |
  | `source` | the store that owns the record, with revision availability. Edits and withdrawals route to it, and an opened member keeps its source's explanation and actions | MereHost session/root identity in shared readings, immutable revision unavailable; scenomise `source`, `source_revision` |
  | `asserter` | attributed source, else the journal `Author` (ruling 5) | `provenance_iri` |
  | `first_asserted_ms` | set when the assertion is minted, never overwritten | canonical `first_asserted_ms`; unknown remains explicit |
  | `last_asserted_ms` | updated on reassertion by the same asserter | `asserted_at_ms` (often `None`) |
  | `withdrawn` | `{ at_ms, by }` once withdrawn; the record remains until compaction | retained lifecycle for every recognized relation predicate |
  | `basis` | why: `note`, `method`, `evidence`, `occurrences` (§6.2) | canonical note/method/evidence; occurrences remain future work |

  Mark's "linking factor" is an assertion. `statement` remains the code
  synonym under the `edge` rule. **Held** means not withdrawn.
- **predicate**: what an assertion means. A recognized predicate declares
  its properties once in the predicate registry: its **stratum** (nature),
  **family** (the policy group), lifecycle (durability and compaction),
  retractability, projectability and **force contribution** (behavior). These
  are the four axes of the [statement kernel brief](2026-06-19_statement_kernel_brief.md),
  plus stratum, which `predicate_registry.rs` already declares, and force.
  Unrecognized predicates are stored and drawn generically
  ("recognized core gets behavior, open tail gets stored").
- **link**: the derived reading of all assertions between one unordered pair
  of endpoints within one stratum, held ones plus withdrawn ones that have not
  yet been compacted. Its key is `(stratum, {a, b})`. For the resource
  stratum the endpoints are canonical-IRI resource ids, so the key is the
  same on every device without being stored. A link is never stored, synced
  or minted as an entity. Its reading can be cached, and a view's responsive
  path can carry rebuildable live state keyed by the link and its appearance
  (§7.3). When a Resource link is drawn between surfaces it is one link
  with many appearances, and it keeps its Resource origin (TERMINOLOGY). If
  a pair ever needs an identity of its own, for comments or shared review, it
  is **promoted**: it is frozen into a Resource the way a selection is frozen
  (ruling 8), with assertion copies at a revision (L11, phase P7). Under the
  resource-leaf draft, a selector alone mints no entity.
- **edge**: unchanged. A relation across strata, today the zero-hop shown
  binding. This design adds no new edge kind.
- **strength** (view rule): a per-view function from a link's held
  assertions to one number. Its default is the sum over held assertions of
  each predicate's declared force contribution, which is 1 for every
  predicate until the dynamics grammar plan sets otherwise (L9). Stroke weight reads it, and
  so may a link spring's stiffness. Each assertion's term is its
  **allocated share**. That share is not a measured force (§7.3). Strength
  is a setting, not truth (audit thread 4: "the strength projection probably
  wants to be settings"). Field response and compatibility between links are
  declared view/dynamics rules; one strength number does not define them.

Traversal remains a separate `TraversalData` event collection with an
`EdgeMetrics` rollup; it has not yet been converted into assertion
occurrences. The longer-term model in this design treats navigation history
as one `mere:traversed` assertion per direction, with dated events as its
**occurrences**. Woodshed practice evidence has the same target shape: an
evidence assertion whose occurrences are dated visits. This answers the §7
"Occurrences or events" candidate for pair relations without a new stratum.
An occurrence is dated support inside an assertion, not an entity.

## 4. Rulings (Mark, 2026-10-15)

L1–L4 were asked as four questions, and L5–L6 as two more after the review (§11). Mark took the recommended option each time.

- **L1. Link identity: derived reading.** A link is computed from its
  assertions and keyed by `(stratum, unordered endpoints)`. It is never
  stored or synced. A link that needs its own identity is promoted by freeze.
- **L2. Withdrawal: kept until compaction.** A withdrawn assertion stays in
  its pair bucket with `withdrawn { at_ms, by }`. It is hidden from held
  readers until its predicate's lifecycle compacts it, locally, or an
  accepted checkpoint does, when shared. This follows the
  [deletion, retention and native drop plan](../implementation_strategy/2026-07-12_deletion_retention_and_native_drop_plan.md)
  §3.3 law: a withdrawal stays visible until a checkpoint captures the
  absence. Erasure remains a separate act.
- **L3. Unification scope: all families.** Containment, arrangement,
  imported and provenance facts become assertion records too. The asserter is
  the writer, and a rule such as URL-path containment becomes the `method`.
  `EdgePayload`'s six sidecars collapse to one assertion list. Family becomes
  a declared predicate property. This comes late (P6), after the read model
  proves itself.
- **L4. Rest drawing: one stroke per link, fan retired.** A closed link is
  one stroke. Between a surface pair there are at most two strokes, one per
  stratum, with the ruled 2026-10-13 cues. A single member is picked by
  opening the link. All three fan implementations retire. Direction and
  opposition (for example one link holding both `Supports` and
  `Contradicts`) stay legible on the closed stroke as marks: arrowheads, a
  both-ways mark, an opposition mark. They are never drawn as extra lines.
- **L5. History in queries (formerly O2).** Default SPARQL, export and held
  readers see held assertions only. Withdrawn assertions are visible through
  an explicit history scope.
- **L6. The shared shape (formerly O3).** The portable assertion and link-key
  shape lives in `scenograph`, beside the fields of `DisclosedRelationship`.
  It carries `source` and its revision. The kernel adapts its native
  assertions into it, and Woodshed and Knot disclose into it through
  scenomise.

Mark ruled the open questions on 2026-10-15, after the implementer's first
slice (§11):

- **L7. Valid time: not yet (formerly O1).** Assertions carry record time
  only. When a consumer needs asserter-claimed valid time, it is added as one
  optional interval in `basis` and never becomes the timeline's default axis.
- **L8. Chartulary converges (formerly O4).** Before P3's Woodshed proof,
  chartulary relations gain asserter, span and basis under §6's rules.
  Chartulary's `EdgeId(writer, counter)` is the assertion id, and
  Connect/Disconnect become assert/withdraw.
- **L9. Strength defaults are settled with the dynamics grammar plan
  (formerly O5).** Until then every held assertion contributes 1 to
  strength, which matches today's multiplicity. Traversal count and recency
  do not feed strength yet.
- **L10. Selection opens; lanes are rows (formerly O6).** Selecting a link
  opens it. Zoom-to-open is a per-view setting, off by default. Timeline
  lanes are stacked rows inside the existing Timeline family, with no new
  family and no second axis.
- **L11. Promotion is planned now (formerly O7).** A link can be frozen into
  a Resource for comments and shared review, as phase P7 after P3 (§8). This
  was not the recommended option, which had kept promotion as an escape
  hatch only.
- **L12. Compaction by predicate nature (formerly O8).** Content predicates
  keep withdrawals until a checkpoint. Session and arrangement predicates
  drop withdrawals at session end. Traversal compacts into its rollup.
- **L13. Two reasons from one asserter are two assertions.** When one
  asserter gives two independent reasons for the same predicate on one pair,
  each is its own assertion, separately withdrawable and with its own time.
  P2 extends ruling 1's dedup key with a **reason key**: the method IRI
  without its version, plus the sorted evidence references, and empty for a
  plain explicit claim. A reassertion with the same reason key still updates
  its own record, and a new extractor version still updates its predecessor
  (ruling 5). P2 implements the method IRI plus sorted unique evidence key;
  note and method version do not split a reason. The first P1 delivery did
  not change deduplication.

## 5. Retire, subsume, keep

**Retire** means the concept is deleted. **Subsume** means another concept
covers its job. **Keep** means it stays, possibly re-expressed.

| Concept (where) | Verdict | Becomes |
|---|---|---|
| `EdgeFamily` (6; `edge_taxonomy.rs`) | Subsume | A declared predicate property (the policy group). No longer a storage shape. The stance doc's curated lens survives as registry vocabulary. |
| Sub-kinds (`SemanticSubKind` and the four others; 44 recognized kinds) | Subsume | Recognized predicates. Each enum is the recognized-predicate table for its family. |
| `EdgePayload` family sidecars (`edge_payload.rs`) | Retired (P6 implementation) | One authoritative retained assertion collection plus the separate Traversal summary. The type remains the pair-bucket payload. |
| `SemanticData` compatibility aggregates (`sub_kinds`, `label`, `predicate`) | Derived projection | Reads are computed from canonical assertions; they are not a second store. |
| `ArrangementData` / `ContainmentData` / `ImportedData` / `ProvenanceData` sub-kind sets | Derived projection | Compatibility accessors compute their sets from canonical assertions. |
| `TraversalData` + `EdgeMetrics` | Keep for now | Separate traversal events and rollup remain pending the future occurrence model; do not infer assertion timestamps from them. |
| `EdgeKind`: graphshell 4-variant tag set, kernel index | Already retired 2026-05-13 | Delete the surviving doc comments when touched (`mod.rs`:154, `edge_payload.rs`:14). |
| VGCP `EdgeKind` | Already "not current direction" | No action. |
| tabard `EdgeStyleRegistry`, `EdgeStyleKey`, `ThemeEdgeKindToken` | Retired (P6 implementation) | The unused relation-specific registry is gone; the generic theme edge-family styling contract remains. |
| `RelationKind` (read classifier) | Subsume | The recognized predicate. |
| `RelationSelector` (filter and retract discriminant) | Subsume | A predicate filter: one predicate, a family group or a set. Exact retract uses the assertion id. |
| `EdgeAssertion` (write contract) | Keep, simplified | `assert(spec)` and `withdraw(id)`, the only two writes. |
| `RelationKey` (pair-bucket handle) | Keep | Internal. Not a link id, not an assertion id. |
| `StatementId` | Keep | The assertion id, the only member identity. |
| `decay_progress` (written, then discarded: `edge_ops.rs`:104, snapshot constant 0.0) | Retire | Strength (view rule) and lifecycle (predicate). |
| `RelationOrigin` (deferred 2026-05-11, never built); Woodshed `RelationAuthority` {Catalog, Computed, Evidence}; forme member `Provenance`; "explicit / computed / inferred" | Subsume | One derived **origin** reading, a set: `computed` when `basis.method` is set, `evidence` when evidence or occurrences are set, `explicit` when neither is. A claim can be both computed and evidenced. Origin says how a claim arose and grants nothing; edit and withdraw rights come from the source and the predicate's retractability. The asserter tells whose. No stored origin field. |
| `GraphScope` | Keep | Unchanged. |
| Journal `Author {kind, id, version, via}` | Keep | Records who recorded the assertion. For Rule, Script and Engine, `id` and `version` are copied into `basis.method` at write time (ruling 5 already keeps the version). |
| RDF 1.2 reifier per assertion | Keep | Gains first/last time, invalidation and basis terms (§6.3). |
| Classic-reification JSON-LD bridge (ruling 43) | Keep | Carries the same terms. |
| `RelationCell` / `EdgeCell` / `CellOwner` / `PairInputs` (`relationship_topology.rs`, `types.rs`) | Retire | Link plus assertion. A cell is coarser than an assertion: two same-kind assertions share one cell. |
| Roster `LinkBundle` / `RelationCell` / `LinkCard` (`crates/mere/src/roster.rs`) | Subsume | The link reading. Its card becomes the opened link (§7.2). |
| Graphshell `LinkTarget` / `LinkInspection` (`ports/graphshell/src/link_inspection.rs`) | Subsume | The link reading. Its detail panel becomes the opened link. |
| Woodshed `compact_relation_summaries`, `SetGraphRelationChoice`, `StageRelationKey` | Subsume | The link reading over Woodshed's disclosed assertions. Hide and select key on the assertion id. |
| Knot operation-hash assertion id | Keep | The assertion id when Knot assertions are ingested. Its signed `asserted_at_ms` is kept as given. |
| Four member-identity schemes (StageRelationKey, EdgeCell, statement id with ordinal buckets, Knot op hash) | Subsume | One: the assertion id. |
| Three pair-bundle keys (Woodshed `ordered_pair`, Graphshell min/max, roster) | Subsume | One: the link key. |
| "Bundle" × 3 | Retire two | Roster bundle becomes link. The fold boundary count is a link whose endpoint is the fold (next row). "Bundle" stays only as the catalog's edge-bundling layout rung. |
| Folds, supernodes, astroid (`forme/fold.rs`, `fold_projection.rs`) | Keep | Node collapse is unchanged. A fold's boundary relations become links read with the fold standing in for its members. This subsumes `fold_projection`'s (outside endpoint, direction, family, count) grouping. |
| Fan implementations: cambium `FAN_GAP_PX` 12, pictograph `CELL_FAN_GAP` 6 overlay above the collapsed underlay | Retire (L4) | One rest stroke per link. |
| Query-view DOM stroke (`web_tree/query_view.rs`) | Keep | It is already one stroke per bucket. It re-keys to the link. |
| Multiplicity as thickness (2026-06-24) and as N spring slots (`seiche_bridge.rs`) | Subsume | Strength; a selected link spring may read it as stiffness. Field response and link compatibility have their own declared dynamics rules (§7.3). |
| Visual edge-collapse "LOD setting" (petgraph plan; swatch design §7, never built as a switch) | Subsume | Rest stroke and opened link. |
| Swatch design §7 "the addressable edge is a cell" | Retire | The addressable unit is the assertion; the drawable unit is the link. |
| Hide-only vs retract by authorship | Keep | Withdraw is allowed to the asserter, per the predicate's retractability. Hide is curation. |
| pandect `HiddenRelationRecord {from, to, relation_tag}` (`view_intent_store.rs`) | Keep, re-keyed | Hides one assertion by id, a predicate filter on a link, or a whole link. A family tag cannot tell two assertions apart. |
| sceno `RoutedRelation` | Keep | Gains a source reference to the link key, so the scene path keeps identity (`scene_out.rs` collapses it today). |
| `VisualRelationKind` / `StrokeCue` (query view) | Keep | The stratum cue of the rest stroke. |
| scenomise `DisclosedRelationship` with `RelationshipProvenance` | Keep | The disclosure shape of one assertion from a host. Its fields align with `basis`: method, method version, provider, evidence. |
| Woodshed `MaterialRelation` | Keep, disclosed | A domain record disclosed as an assertion. `method` is the Woodshed computation and version. Its 0..100 weight is a domain value the strength function may read. |
| `EdgePath` (`numen/edge_path.rs`) | Keep | `Spline` and `FieldLine` remain path declarations. Numen now has a separate bounded fixed-endpoint responsive-path evaluator; the implementation plan owns P5's host and visual proof (§7.3). |
| Field couplings | Keep | Existing couplings remain node-targeted. P5 explores a view-local path response without adding an `EdgeFamily` or a stored link entity. |
| Nested graph | Keep | Node-borne. Not used for links; promotion uses the freeze pattern instead. |
| Swatch | Keep | Gains a **link** scope (§7.2). |
| Timeline arrangement (sceno `score.rs`) | Keep | The default opened-link arrangement for dated data; list is the default for mostly undated data (§7.2). |
| "Timeline" × 6 | Clarify | In this doc it means only the arrangement family. |
| Linked-data first-arc surface projection (`linked-data/src/lib.rs`) | Retired in the bounded P1 delivery | Reads every outgoing bucket through the kernel's adjacency-indexed `surface_outgoing_relation_buckets`; preserves source direction and scope and excludes incoming-only arcs. |

**Count.** Of 47 rows, 23 are retired or subsumed and two were already
retired. They include three fans, three bundle
keys, four identity schemes, two Link Cards, the style registry, the
authority enums and the dead decay field. Added: four fields
(`source`, `first_asserted_ms`, `withdrawn`, `basis`), one read model that replaces four,
and one swatch scope. No new stratum, entity, edge kind or stored type.

## 6. Time and why

### 6.1 Time

An assertion carries **record time**: when its asserter said it, when they
last confirmed it and when it was withdrawn. The span is
`[first_asserted_ms, withdrawn.at_ms or open)`. Reassertion by the same
asserter after a withdrawal mints a **new** assertion, because dedup matches
held records only, so a pair's history is a sequence of closed intervals,
never a revived one. Asserter-claimed **valid time** ("these were related from
2010 to 2015") is not a field yet (L7).

Clock rule: the caller's time is used when the caller supplies one: Knot's
signed header, an ingested `prov:generatedAtTime`, a sync event. Otherwise the
kernel stamps wall clock once at authored admission and records that value.
Import, replay and restore preserve a missing source time. A record with no
time is undated (legacy
data, or a source that gives none). It is shown in the undated lane and never
as 1970.

An explicit authored reason or exact-ID confirmation advances last time.
An idempotent membership ensure without a confirmation request leaves the
existing time unchanged. These are separate entry points in the kernel.

### 6.2 Why: `basis`

| Field | Holds | Source today |
|---|---|---|
| `note` | free-text reason | semantic `basis.note`; `label` is its compatibility alias |
| `method` | `{ iri, version }` of the rule, script, engine or model that computed it | semantic `basis.method`; scenomise `method`/`method_version`; journal `Author` remains authorship, not a fabricated method |
| `evidence` | IRIs of supporting resources, captures or other assertions | semantic `basis.evidence`; Knot `evidence`; scenomise `evidence` refs |
| `occurrences` | dated events plus rollup (count, first, last) for event-backed predicates | `TraversalData` + `EdgeMetrics`; Woodshed `Engagement` |

There is no stored confidence, weight or origin field. The origin reading
(§5) and strength (§3) are both derived. Knot's `qualification` maps to
`note` until a consumer needs it separately.

### 6.3 Write-path changes

This is a persistence change, not a field addition. It touches snapshots,
replay, sync, query projection and compaction, and these rules bind every
item below:

- **Replay never invents a time.** A time is fixed once, when the change is
  first written, and is carried in the recorded change. Replay, snapshot
  restore and sync fold reproduce it and never stamp again.
- **Wall clocks never decide causal order.** Journal sequence and causal
  parents decide order. Times are informative, as Knot's already are.
- **A stale delivery cannot revive a withdrawn assertion.** Assert and
  withdraw match by assertion id. A late assert for an id that is already
  withdrawn is a no-op.
- **Old records stay undated.** Migration does not guess times.
- **Compaction follows checkpoints for shared assertions** (L2 and the
  deletion plan §3.3). Local compaction follows predicate nature (L12).

1. **Stamp every route at first write.** `apply.rs` (`AssertRelation`),
   `edge_ops.rs` (`assert_relation`, predicate setter) and graphshell personal
   sync stop passing `None` and record the time in the change. Replay deltas
   carry the recorded time and do not stamp. The graph semantics plan's P1
   test that enumerates production writers is extended to assert that each
   records a time.
2. **First and last.** `assert_statement` sets `first_asserted_ms` on mint and
   leaves it alone on reassertion. Today's `asserted_at_ms` becomes the last
   time, and its persisted name stays. Legacy records load with `first = last =`
   the stored value, or undated.
3. **Withdraw, don't delete.** `retract_statement` sets
   `withdrawn { at_ms, by }` instead of removing (`assertion_write.rs`; `edge_data.rs`). Held readers filter it out. The bucket is removed only when
   it holds no record at all. Graphshell's `RetractAssertion` event gains a
   time. Compaction is per predicate lifecycle (statement kernel brief) or by
   accepted checkpoint for shared assertions. A late assert or withdraw is
   resolved by id, as the rules above state.
4. **Basis.** Persist `note`, `method` and `evidence` in
   `PersistedSemanticStatement`. Ingest and export round-trip them. Event-backed
   `occurrences` remain in their current sidecars until those families migrate
   in P6; this semantic slice does not invent occurrence records.
5. **RDF.** On the reifier: `prov:generatedAtTime` = first;
   `mere:lastAssertedAt` = last; `prov:invalidatedAtTime` for a withdrawal;
   `prov:wasGeneratedBy` for the method; `prov:wasDerivedFrom` for evidence.
   `prov:wasInvalidatedBy` points to an activity, not an agent
   ([PROV-O](https://www.w3.org/TR/prov-o/#wasInvalidatedBy)), so the
   withdrawer is a withdrawal activity node: `prov:wasInvalidatedBy` that
   activity, which is `prov:wasAssociatedWith` the withdrawer. Ingest lifts
   all of these. Withdrawn
   reifiers appear only in the history scope (L5).
6. **All families (P6).** The four sidecar families write assertions through
   the same `assert` and `withdraw`. Their compatibility reads become predicate
   filters.

## 7. Consequences, not layers

### 7.1 The link

`link(stratum, a, b)` reads every directed bucket in both pair orientations
through `projected_relations_between`, including lifted Resource appearances.
Explicit retained history uses `retained_relations_between`. Its canonical
members preserve exact IDs across all recognized relation families; direction,
predicate, asserter and family are attributes of members, not levels. The
grouped rest-stroke projection counts held records by exact ID and omits pairs
whose only records are withdrawn. Traversal remains a separate event summary.
P1 gives Graphshell and Pictograph one shared implementation; roster, Cambium
and Woodshed adopt it in their later phases. Hosts outside the kernel (Woodshed
and Knot via Scenomise) disclose assertions in the same shape, which lives in
Scenograph (L6). A reading over a partly resident graph reports its coverage
(ruling 4), so "no more assertions" and "not loaded" stay distinct.

### 7.2 The opened link, which is the "nested link"

Selecting a link opens it (L10). Opening past a zoom threshold is a
per-view setting, off by default. An opened link shows a **swatch at link
scope**. That is the existing swatch with one new scope value, compiled to a
scene in one of two arrangements, chosen from the disclosed data and
switchable:

- **List**, grouped by meaning (predicate family) or by source, with
  direction shown per row. This is the default when few members are dated,
  as with theory, containment and imported relations.
- **Timeline**, the default when members are dated, as with traversal,
  practice evidence and authored claims.

On the timeline:

- x is `first_asserted_ms`, with a span drawn to the withdrawal or to now;
  withdrawn members show as ended spans.
- Lanes are by family by default, switchable to asserter or direction.
- An **undated lane** holds timeless members, as most theory relations are
  (precedents: Woodshed `Engagement.dated = false`, sceno `TimelineFallback`).
- Occurrences render as ticks on their assertion's span, the Trail ribbon at
  link scale.
- Each member shows its basis: note, method and version, evidence links.

This one surface replaces roster's Link Card, Graphshell's link inspector
panel and Woodshed's relation inventory for a pair. Selecting, hiding,
withdrawing and auditioning one member (Woodshed P4f) all act on the
assertion id. Withdrawing routes to the member's source, which keeps its
own explanation and actions. Its accessible alternate is the same list, read in time order
with the undated members last. Nothing in it is stored. It is a projection
of the link, so it inherits the swatch's representation-never-identity rule.

### 7.3 Responsive paths that make dynamics legible (experiment)

The purpose is to understand different dynamics through how links respond.
A field may influence a path along its length. Compatible links may attract,
align or respond together to a disturbance: the proposed **sympathetic**
response. Attraction is a first candidate; resonance or propagation between
links remains an experiment choice. The bend is one observable of that
response. It need not be reconstructed from an endpoint spring's force.

Compatibility is a declared rule in the view's dynamics: for example a shared
predicate or a source-disclosed affinity. It reads the eligible, disclosed
assertions and explains how a link with several assertions combines their
contributions. Shared appearance alone grants no compatibility. Gathering
paths preserves each link's endpoints, selection identity and distinct
assertions.

Start with **fixed endpoints**. Compare the same links under an attractor,
flow and vortex, first with field response alone, then compatible-link
attraction alone, then both. The proof asks whether their different effects
are legible and attributable to the enabled response. Controls disable each
influence independently and include an incompatible link. Feeding path
responses back into endpoint motion is a later decision; P5's first proof
does not require it.

Control points, velocities and other path state belong to an appearance in
one view, keyed by its link and instance. They are temporary, bounded and
rebuildable. Authored response settings are view/dynamics configuration.
Neither the live path nor its motion mints an entity, changes an assertion,
or becomes shared relationship truth. Hiding a member changes only that
instance's eligible contributions (swatch design §8). "Relationship
membership alone does not install a force" (dynamics grammar plan) still
holds: responses must be declared and may be zero.

Current implementation boundaries:

- **Seiche's `EdgeSpring` acts only along the endpoint line**
  (`crates/conatus/seiche/src/forces.rs`:184). It supplies no lateral path
  state. A field-responsive path needs its own evaluator and response model.
  Existing node-targeted couplings do not already act on paths.
- **Numen's `EdgePath::Spline` and `EdgePath::FieldLine` remain vocabulary**
  (`crates/conatus/numen/src/edge_path.rs`). The separate bounded evaluator in
  `crates/conatus/numen/src/responsive_path.rs` implements P5's fixed-endpoint
  response. Pictograph adapts shared link readings and Graphshell supplies the
  reference host. The implementation plan owns its bounds and visual evidence;
  the declarations alone do not install a response law.
- **One spring per link** can separately replace N spring slots per visible
  row (`seiche_bridge.rs`) when that law is selected. With additive linear
  stiffness contributions and a common rest length, an assertion has an
  exact **allocated spring share**. The inspector labels that allocation
  separately from measured forces or observed path motion. It does not
  establish attribution for fields, repulsion, pins or another law. The
  `dynamics_inspection.rs`:138 limitation on measured per-assertion force
  remains until such measurement is implemented.

P5's outcome gates neither the link model nor the family migration.

### 7.4 What falls away

The fan-or-collapse switch the swatch design promised but never built is no
longer a choice. The swatch's §11 question about per-cell fan geometry ("how
many parallel lines … stay legible") dissolves, because no view fans.
Multiplicity stops meaning three different things (thickness, spring slots,
cell count). The audit's thread 2, edge reification, is answered: assertions
are addressable, and the pair is their reading. Thread 4, unified strength,
is answered as a view rule.

Simplification is proved by deleted responsibilities, not by rows in §5.
Several jobs survive as registry properties, filters, adapters and lifecycle
rules. Each phase's done condition names the grouping, selection and
identity machinery its consumers delete.

## 8. Phases

P1–P4 land in order, followed by P6's family migration. P5 is an independent
dynamics experiment and is never a prerequisite for P6. Each phase has a
first proof consumer and names what it deletes or, for P5, what it must
establish. Reordered 2026-10-15 (§11): the shared reading comes first, over
current storage, so a consumer benefits before any persistence change.

- **P1. One link reading, bounded (L6).** The shared shape in `scenograph`
  and one `link(stratum, a, b)` over current storage. Covers parallel
  assertions, both directions, both strata, lifted Resource links, exact ids,
  id-less sidecar members (§7.1) and incomplete coverage. Includes the
  linked-data every-arc fix. Proof: **Graphshell's link inspector** reads it
  preserving existing record fields and adding explicit source/revision
  availability. Deletes Graphshell's own pair grouping (`LinkTarget`
  expansion). A second consumer (roster `LinkCard`/`LinkBundle`, or
  pictograph `PairInputs`/`EdgeCell`) adopts it and deletes its grouping
  before the phase is called done.
- **P2. Time and withdrawal (kernel, linked-data).** §6.3 items 1 to 5 for
  semantic assertions, under §6.3's rules. Proof: Graphshell's inspector
  sorts by first time, shows withdrawn spans and lists undated assertions
  last. A Knot assertion and retraction round-trip with their times. Done
  when authored writers record their admission time, import/replay preserve
  an unknown source time, replay reproduces recorded times exactly,
  reassertion keeps the first time, withdraw-then-reassert yields two
  intervals, a stale assert does not revive a withdrawal, and default SPARQL
  shows held assertions only (L5).
- **P3. Opened link.** Swatch link scope with list and timeline arrangements
  (§7.2). Proof: Graphshell, then **Woodshed** (selection, explanation,
  method provenance, withdraw routed to Woodshed's source, P4f audition per
  assertion), then **Knot** (authored time and retraction). Before Woodshed's
  proof, chartulary relations gain asserter, span and basis, with `EdgeId` as
  the assertion id (L8). Timeline lanes are stacked rows inside the existing
  Timeline family (L10). Deletes Woodshed's
  `compact_relation_summaries`, `SetGraphRelationChoice` and
  `StageRelationKey` identity.
- **P7. Promoted link (L11; after P3, independent of P4–P6).** Freezing an
  opened link mints a Resource holding the link key (stratum and endpoints),
  copies of its assertions as they stood at the freeze revision, and each
  source's revision, by ruling 8's pattern. Each freeze is its own Resource.
  Comments and review attach to it as assertions about that Resource. The live
  link keeps reading current assertions, and the frozen record shows how it
  has drifted from them. Proof: Graphshell, where selection freezing already
  exists. Done when a later withdrawal leaves the frozen copy unchanged, a
  receiver evaluates the shared Resource under ruling 4's coverage, and no
  stored pair entity appears outside the promoted Resource.
- **P4. Rest stroke, fan retired.** One stroke per link with direction and
  opposition marks (L4) in **Woodshed's relationship editor** first, the
  surface that started this lane. Then pictograph, cambium `graph_canvas` and
  the scene path (`RoutedRelation` gains its link key). Hit-test resolves the
  link; opening resolves a member. pandect hide is re-keyed. Deletes the three
  fan implementations.
- **P5. Responsive-path experiment (independent, gates nothing).** Links
  respond to fields and compatible links under declared rules (§7.3).
  Proof: a bounded Graphshell canvas fixture with fixed endpoints, compatible
  and incompatible links, and attractor, flow and vortex comparisons. Done
  when field response and like-link attraction can each be enabled and
  disabled independently with reproducible path changes, while endpoint and
  assertion identities remain intact. Review captures for whether the
  different dynamics are legible. The inspector distinguishes configured
  response, observed path motion and any allocated spring share. Record the
  evaluator, bounds and response rules used; endpoint feedback and resonance
  are further choices, not completion requirements.
- **P6. All families as assertions.** The four former sidecar families now
  use canonical assertion records, with compatibility set accessors derived
  from them; `TraversalData` remains a separate summary. The unused Tabard
  relation-specific style registry is retired while generic family styling
  remains. Persistence migration loads old snapshots, with asserters from
  journal replay as in ruling 9. Bounded P6 is qualified: exact-ID withdrawal
  retains the independent same-pair reason; shared reads, acknowledged
  presentation hiding and persistence preserve source records; old snapshots,
  journals, codicils, sync and transfer retain their checked behavior. The
  source, reviewed browser/receiving and substantial-history budget receipts
  are recorded in the implementation plan. Traversal's future occurrence model
  and P7 promotion keep their own scope.

### 8.1 Execution recommendation: a core sequence and an independent probe

Keep **P1 → P2 → P3 → P4 → P6** as one integration sequence. P7 follows P3
on the same contracts and does not block P4 or P6. P1 settles the
portable link key, assertion reference, source/revision and coverage contract
with two real consumers. P2 then owns persistence, replay, withdrawal and
query semantics together. P3 and P4 consume those contracts before P6 replaces
the remaining storage families. Splitting these changes into simultaneous
kernel migrations would make the shared contract move under its consumers.

**P5 can proceed in parallel as an isolated probe.** Its evaluator can start
with fixed, disclosed fixtures and ordinary endpoint coordinates. It need not
wait for timestamp migration, all-family assertions or a new stored shape.
Once P1's contract is stable, a thin adapter supplies real links and their
source references. Fixture results alone do not close P5's Graphshell proof.

Opened-link layout and consumer-adapter preparation can overlap P2 after
P1's contract is stable; their acceptance still waits for P2. Serialize
changes to shared Graphshell/Pictograph rendering and hit-testing files with
P3/P4, so the dynamics adapter follows the current link and selection path.
The dynamics evaluator can be reviewed independently of that host integration.

Keep persistence and the portable contract under one owner at a time. Give
the probe separate worktree and output ownership if it runs concurrently.
Qualify the combined consumer once the source and adapter changes are in
place; retain the distinction between a fixture probe, an integrated host
and reviewed native behavior. This routing does not claim phase acceptance. Execution began on 2026-10-15 under the
[implementation plan](../implementation_strategy/2026-10-15_link_model_plan.md).

## 9. Open rulings for Mark

None. O1–O8 were ruled as L5–L12, and the same-asserter question as L13 (§4).

## 10. Vocabulary

TERMINOLOGY gains **assertion** and an L1 note on **link** (a derived reading
keyed by stratum and unordered endpoints, never stored). "Linking factor" is
recorded as Mark's word for an assertion. **Cell** and the roster sense of
**bundle** retire from design language. Code identifiers keep their names
until touched, under the `edge` rule.

## 11. Amendments

**2026-10-15, after an implementing agent's review.** The review read the
doc at `a9765ad01` against the kernel, inspection, disclosure and dynamics
code, and its code claims were checked (`forces.rs`:184,
`scenomise/src/projection.rs`:43). Accepted:

- `source` on the assertion; withdrawals route to their source (§3, L6).
- "Authority reading" renamed **origin** and made a set (§5).
- §6.3 recast as a persistence change, with replay, clock, stale-delivery and
  undated rules.
- The bend demoted to a separate experiment; allocated share distinguished
  from measured force (§2, §7.3, P5).
- List as well as timeline for the opened link (§7.2).
- Simplification measured by deleted responsibilities (§7.4, §8).
- PROV mapping corrected: `prov:wasInvalidatedBy` points to an activity
  (§6.3).
- Phases reordered so the bounded shared reading comes first (§8).

Kept against it: the closed link stays one stroke (L4). The review's
"direction and opposing predicates apparent when closed" is met by marks on
that stroke, never by extra lines. Mark ruled O2 and O3 as L5 and L6.

**2026-10-15, Mark's dynamics clarification.** Mark described fields and
perhaps sympathetic forces influencing like links, to help clarify the
behavior of different dynamics. §2, §3, §7.3 and P5 now frame the experiment
as responsive paths, beginning with fixed endpoints and independent field
and compatibility controls. Temporary path state is distinct from stored
relationship truth; endpoint feedback and resonance remain choices within
the experiment. The existing source, origin, replay, query, list/timeline
and migration recommendations stand. The earlier universal timeline-default
wording and the phase preamble were reconciled with those recommendations.
No new ruling or implementation is claimed.

**2026-10-15, implementation routing review.** §8.1 recommends the core
integration sequence and an isolated P5 probe in parallel, with a stable P1
contract and serialized shared host edits. The recommendation preserves P5's
independence without making persistence or consumer adoption follow a moving
experimental interface.


**2026-10-15, first bounded implementation.** The shared scenograph reading,
Pictograph reader and two consumer adapters are implemented as a first slice,
alongside the outgoing parallel-bucket export fix and an isolated Numen path
probe. P1 preserves existing records while adding explicit source availability
to Graphshell's output; graph-only consumers report missing source context
rather than inventing public identity or revision. The
[implementation plan](../implementation_strategy/2026-10-15_link_model_plan.md)
records that delivery's checks and open gates. At that point P2–P4/P6 and
P5's host/visual proof remained open; the continuation below updates status.

**2026-10-15, open rulings.** Mark ruled O1 and O4–O8, and the plan's pending
same-asserter question, as L7–L13. They took the recommended option on each
except O7, choosing to plan promotion now (P7) rather than keep it as an
escape hatch.


**2026-10-15, lifecycle and dynamics continuation.** Source `1e3fb426c` adds
local MereHost session/root source identity with unavailable immutable
revision, semantic first/last/withdrawal and basis, L13's exact reason key,
held-default/history-explicit reads, JSON reopen and additive worker-wire
compatibility, RDF lifecycle projection, and Graphshell's history inspector.
The bounded P1 consumers pass their affected checks. P5 now adapts real shared
readings into the bounded evaluator; eight reviewed browser configurations
make field and compatible-link responses independently visible with fixed
endpoints and retained assertion identities. P5's stated experiment is
qualified. P2 remains open for Knot's timed retraction projection and a real
Mere intake round-trip; the browser inspector fixture does not demonstrate
withdrawn/undated spans visually. The plan and its receipt preserve these
limits, including unqualified newer-main dependencies, pre-P2 binary archive
migration and later-family/history budgets. No ruling changed.


**2026-10-15, P2 supplier/intake qualified.** Knot `0020a50` preserves optional
signed retraction time, including old checkpoint byte identity when absent.
Djinn imports an opaque store-verified projection through exact captured-head
and existing Surface/Resource admission. It keeps full operation/signer/space
identity and source basis/clocks, preflights complete batches and commits through
Pandect's acknowledged transaction. A real signed source, checkpoint/reopen and
fresh-process Mere JSON reopen pass. The first Knot fold retraction supplies the
effective tombstone; later source retraction events stay in Knot, and conflicting
effective withdrawal metadata refuses intake. Four reviewed Graphshell captures
now demonstrate held, timed withdrawn, time-unknown withdrawn and undated held
records. This closes P2's recorded gate; automatic sync, desktop-native editing
and later phases remain separate. The implementation plan owns exact evidence.

**2026-10-16, P3 prerequisite and first consumer.** Chartulary retains stable
`EdgeId` assertions with source asserter, clocks, basis and terminal withdrawal;
checkpoint/replay and scoped-admission controls pass. A shared link-scoped
swatch compiles the existing Stack and Timeline arrangements and renders source
metadata, exact actions and a chronological alternate. Graphshell's reviewed
browser fixture shows timed, time-unknown and undated assertions while preserving
selection. The source owner rechecks full source/stratum/direction/ID action
targets. This qualifies the bounded supplier and first consumer on `732c1537a`,
not P3 as a whole: Woodshed's source-owned action/audition flow and Knot's
opened-link consumer proof remain. No ruling changed. The implementation plan
records the personal-only Woodshed withdrawal choice and the exact test, browser and
documentation evidence; desktop-native acceptance and later phases stay separate.

**2026-10-16, receiving-host continuation.** Knot now uses the shared opened-link
reader over its authenticated signed source projection. Native seed and a fresh
process preserve exact assertion IDs, directions, source basis and authored/
withdrawal clocks, with unknown times left unknown. Its source freshness and
unavailable immutable revision are explicit. Woodshed has the session-owned
personal assertion source and exact actions, with generated claims read-only;
the 784-case source gate and final 191-case view follow-up pass, with a rebuilt
desktop binary. Final native source `474d10a` passes ordinary note entry, personal
assertion admission, exact withdrawal and ordered audition with Set unchanged.
A different process reopens the same sealed source UUID, both assertion IDs and
withdrawal tombstone. Seven reviewed List/Timeline captures close the bounded P3
receiving proof, published with receipt `943c762`. The implementation plan records
exact revisions, receiving receipts and the separate Knot checkpoint
confidentiality finding. L1–L13 and the later phase gates are unchanged.


**2026-10-16, bounded P6 qualified.** All 44 recognized predicates now use
retained assertion records; the four former set families have derived
compatibility reads. Exact withdrawal and exact view hiding preserve independent
reasons and their source metadata. Versioned legacy conversion preserves unknown
clocks and only repairs attribution from proven journal history. The
[implementation plan](../implementation_strategy/2026-10-15_link_model_plan.md#p6-qualification-and-receiving-proof)
records source, compatibility, 50,000-withdrawal budgets, reviewed WebGPU
hide/reopen proof and the published Woodshed receiving receipt. Traversal remains
event-shaped; fields and sympathetic influences retain the dynamics framing in
§7.3, and P7 promotion remains later work.


### P7 promoted-link qualification (2026-10-16)

The [implementation plan](../implementation_strategy/2026-10-15_link_model_plan.md#p7-qualification-2026-10-16)
and [receipt](../../../support/receipts/link-model-p7-20261016/receipt.json)
record bounded Graphshell P7 on `6d53d4439`. Each freeze creates an independent
Resource with the original stratum, endpoint references, exact directed assertion
copies and explicit source revision availability. The immutable facet survives
later source withdrawal, while live drift and Resource-owned review comments
remain separate readings/assertions. A receiving process reopens the carried
Resource without inventing endpoint payloads and reports missing Surface and
Resource references through coverage. Six reviewed final-source browser captures
cover ordinary freeze, withdrawal, review and fresh-page reopening. This proof
has one owning graph source per freeze; broader receiving products and automatic
resident synchronization keep their separate gates.
